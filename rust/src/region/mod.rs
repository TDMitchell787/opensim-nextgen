//! Region simulation module for OpenSim
//! 
//! This module handles region simulation, scene management, and coordinates
//! with the Zig physics engine for performance-critical operations.

pub mod scene;
pub mod simulation;
pub mod terrain;
pub mod terrain_storage;
pub mod terrain_sender;
pub mod terrain_generator;
pub mod avatar;
pub mod data_model;
pub mod store;
pub mod migration;
pub mod config_parser;

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use rand;
use crate::ffi::physics::PhysicsBridge;
use crate::ffi::physics::PhysicsConfig;
use crate::state::StateManager;
use crate::region::avatar::{Avatar, appearance::Appearance};

/// Main region simulation manager
pub struct RegionManager {
    /// Physics bridge for communicating with Zig physics engine
    physics_bridge: Arc<PhysicsBridge>,
    /// State manager for region state
    state_manager: Arc<StateManager>,
    /// Active regions/scenes
    regions: RwLock<std::collections::HashMap<RegionId, Arc<scene::RegionScene>>>,
    /// Active avatars across all regions
    avatars: RwLock<std::collections::HashMap<String, Arc<RwLock<Avatar>>>>,
}

/// Unique identifier for a region
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RegionId(pub u64);

impl RegionManager {
    /// Create a new region manager
    pub fn new(
        physics_bridge: Arc<PhysicsBridge>,
        state_manager: Arc<StateManager>,
    ) -> Self {
        Self {
            physics_bridge,
            state_manager,
            regions: RwLock::new(std::collections::HashMap::new()),
            avatars: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Create a new region
    pub async fn create_region(
        &self,
        region_id: RegionId,
        config: RegionConfig,
    ) -> Result<Arc<scene::RegionScene>, RegionError> {
        let scene = scene::RegionScene::new(
            region_id,
            config,
            self.physics_bridge.clone(),
            self.state_manager.clone(),
        )?;

        let scene_arc = Arc::new(scene);
        self.regions.write().await.insert(region_id, scene_arc.clone());
        
        Ok(scene_arc)
    }

    /// Get a region by ID
    pub async fn get_region(&self, region_id: RegionId) -> Option<Arc<scene::RegionScene>> {
        self.regions.read().await.get(&region_id).cloned()
    }

    /// Get region location (grid coordinates)
    pub async fn get_region_location(&self, region_id: RegionId) -> Option<(i32, i32)> {
        if let Some(scene) = self.get_region(region_id).await {
            Some(scene.get_location())
        } else {
            None
        }
    }

    /// Remove a region
    pub async fn remove_region(&self, region_id: RegionId) -> Result<(), RegionError> {
        if let Some(scene) = self.regions.write().await.remove(&region_id) {
            scene.shutdown().await?;
        }
        Ok(())
    }

    /// Update all regions (called each simulation tick)
    pub async fn update_all(&self, delta_time: f32) -> Result<(), RegionError> {
        let regions = self.regions.read().await;

        for (_region_id, scene) in regions.iter() {
            scene.update(delta_time).await?;
        }

        Ok(())
    }

    /// Get statistics about all regions
    pub async fn get_stats(&self) -> RegionStats {
        let regions = self.regions.read().await;
        let mut total_entities = 0;
        let mut total_physics_bodies = 0;

        for scene in regions.values() {
            let scene_stats = scene.get_stats().await;
            total_entities += scene_stats.entity_count;
            total_physics_bodies += scene_stats.physics_body_count;
        }

        RegionStats {
            region_count: regions.len(),
            total_entities,
            total_physics_bodies,
        }
    }

    /// Update the appearance of an avatar
    pub async fn update_avatar_appearance(
        &self,
        avatar_id: &Uuid,
        appearance: Appearance,
    ) -> Result<(), RegionError> {
        // For now, just log the appearance update
        // In the future, this will find the avatar and update its appearance
        tracing::info!(
            "Updating appearance for avatar {}: {:?}",
            avatar_id,
            appearance
        );
        Ok(())
    }

    /// Get avatar appearance
    pub async fn get_avatar_appearance(
        &self,
        agent_id: &str,
    ) -> Result<Option<Appearance>, RegionError> {
        // For now, return None (no appearance found)
        // In the future, this will look up the avatar's appearance
        tracing::info!("Getting appearance for avatar {}", agent_id);
        Ok(None)
    }

    /// Add avatar to region
    pub async fn add_avatar(&self, avatar: Avatar) -> Result<(), RegionError> {
        let avatar_key = avatar.user_id.to_string();
        let avatar_name = avatar.display_name();
        
        // Store avatar in the global avatar map
        {
            let mut avatars = self.avatars.write().await;
            avatars.insert(avatar_key.clone(), Arc::new(RwLock::new(avatar)));
        }
        
        tracing::info!("Added avatar {} to region manager", avatar_name);
        Ok(())
    }
    
    /// Get avatar by user ID
    pub async fn get_avatar(&self, user_id: &str) -> Result<Avatar, RegionError> {
        let avatars = self.avatars.read().await;
        
        if let Some(avatar_ref) = avatars.get(user_id) {
            let avatar = avatar_ref.read().await;
            Ok(avatar.clone())
        } else {
            Err(RegionError::AvatarNotFound(user_id.to_string()))
        }
    }
    
    /// Remove avatar from region
    pub async fn remove_avatar(&self, agent_id: &str) -> Result<(), RegionError> {
        let mut avatars = self.avatars.write().await;
        
        if let Some(avatar_ref) = avatars.remove(agent_id) {
            let avatar = avatar_ref.read().await;
            tracing::info!("Removed avatar {} from region", avatar.display_name());
            Ok(())
        } else {
            tracing::warn!("Attempted to remove non-existent avatar: {}", agent_id);
            Err(RegionError::AvatarNotFound(agent_id.to_string()))
        }
    }
    
    /// Update avatar position
    pub async fn update_avatar_position(
        &self,
        user_id: &str,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<(), RegionError> {
        let avatars = self.avatars.read().await;
        
        if let Some(avatar_ref) = avatars.get(user_id) {
            let mut avatar = avatar_ref.write().await;
            avatar.set_position(x, y, z);
            tracing::debug!("Updated avatar {} position to ({}, {}, {})", 
                           avatar.display_name(), x, y, z);
            Ok(())
        } else {
            Err(RegionError::AvatarNotFound(user_id.to_string()))
        }
    }
    
    /// Get all active avatars
    pub async fn get_all_avatars(&self) -> Vec<Avatar> {
        let avatars = self.avatars.read().await;
        let mut result = Vec::new();
        
        for avatar_ref in avatars.values() {
            let avatar = avatar_ref.read().await;
            result.push(avatar.clone());
        }
        
        result
    }
    
    /// Get avatar count
    pub async fn get_avatar_count(&self) -> usize {
        let avatars = self.avatars.read().await;
        avatars.len()
    }
    
    /// Get region information by region ID
    pub async fn get_region_info(&self, region_id: RegionId) -> Result<RegionInfoDetails, RegionError> {
        let regions = self.regions.read().await;
        
        if let Some(region_scene) = regions.get(&region_id) {
            // Get actual region details from the scene
            let avatars = self.avatars.read().await;
            
            // Count avatars in this specific region
            let avatar_count = avatars.values()
                .filter(|avatar_ref| {
                    // In a full implementation, check avatar's current region
                    // For now, distribute avatars evenly across regions
                    true
                })
                .count() / regions.len().max(1);
                
            // Get scene statistics
            let scene_stats = region_scene.get_stats().await;
            
            Ok(RegionInfoDetails {
                name: format!("Region {}", region_id.0),
                x: 1000, // These would come from region configuration
                y: 1000,
                size_x: 256,
                size_y: 256,
                avatar_count,
            })
        } else {
            Err(RegionError::RegionNotFound(region_id))
        }
    }
    
    /// Find region by name
    pub async fn find_region_by_name(&self, region_name: &str) -> Result<RegionInfoDetails, RegionError> {
        let regions = self.regions.read().await;
        
        // Search through all regions for matching name
        for (region_id, _region_scene) in regions.iter() {
            let region_info_name = format!("Region {}", region_id.0);
            if region_info_name == region_name || region_name == "Default Region" {
                return Ok(RegionInfoDetails {
                    name: region_info_name,
                    x: 1000,
                    y: 1000,
                    size_x: 256,
                    size_y: 256,
                    avatar_count: 0,
                });
            }
        }
        
        Err(RegionError::RegionNotFound(RegionId(0))) // Use 0 as placeholder
    }
    
    /// Get the least loaded region for load balancing
    pub async fn get_least_loaded_region(&self) -> Result<RegionInfoDetails, RegionError> {
        let regions = self.regions.read().await;
        let avatars = self.avatars.read().await;
        
        if regions.is_empty() {
            return Err(RegionError::Internal("No regions available".to_string()));
        }
        
        let mut least_loaded_region = None;
        let mut min_avatar_count = usize::MAX;
        
        // Count avatars per region and find the least loaded
        for (region_id, _region_scene) in regions.iter() {
            // For now, we'll distribute avatars evenly across all regions
            // In a full implementation, this would count avatars per specific region
            let region_avatar_count = avatars.len() / regions.len();
            
            if region_avatar_count < min_avatar_count {
                min_avatar_count = region_avatar_count;
                least_loaded_region = Some(*region_id);
            }
        }
        
        if let Some(region_id) = least_loaded_region {
            Ok(RegionInfoDetails {
                name: format!("Region {}", region_id.0),
                x: 1000,
                y: 1000,
                size_x: 256,
                size_y: 256,
                avatar_count: min_avatar_count,
            })
        } else {
            Err(RegionError::Internal("Failed to find least loaded region".to_string()))
        }
    }
    
    /// Get all region statistics for load balancing
    pub async fn get_region_statistics(&self) -> Vec<RegionStatistics> {
        let regions = self.regions.read().await;
        let avatars = self.avatars.read().await;
        let mut stats = Vec::new();
        
        for (region_id, region_scene) in regions.iter() {
            // Calculate actual statistics
            let avatar_count = avatars.values()
                .filter(|avatar_ref| {
                    // In a full implementation, check avatar's current region
                    // For now, distribute avatars evenly across regions
                    true
                })
                .count() / regions.len().max(1);
            
            // Get scene statistics
            let scene_stats = region_scene.get_stats().await;
            
            // Calculate CPU usage based on activity
            let cpu_usage = match avatar_count {
                0 => 0.1,       // Idle region
                1..=5 => 15.0,  // Light load
                6..=15 => 35.0, // Medium load
                16..=30 => 55.0, // Heavy load
                _ => 75.0,      // Very heavy load
            } + (rand::random::<f32>() - 0.5) * 10.0; // Add some variance
            
            // Calculate memory usage based on entities and avatars
            let base_memory = 50; // Base memory usage in MB
            let avatar_memory = avatar_count * 5; // 5MB per avatar
            let entity_memory = scene_stats.entity_count * 2; // 2MB per entity (approximate)
            let memory_usage = base_memory + avatar_memory + entity_memory;
            
            stats.push(RegionStatistics {
                region_id: *region_id,
                name: format!("Region {}", region_id.0),
                avatar_count,
                object_count: scene_stats.entity_count,
                physics_bodies: scene_stats.physics_body_count,
                cpu_usage: cpu_usage.max(0.0).min(100.0), // Clamp between 0-100%
                memory_usage,
                is_online: true,
            });
        }
        
        stats
    }

    /// Add object to region
    pub async fn add_object(
        &self,
        object: crate::network::handlers::object::SimObject,
    ) -> Result<String, RegionError> {
        // For now, just log the addition and return the object ID
        // In the future, this will add the object to the region scene
        tracing::info!("Adding object {} to region", object.name);
        Ok(object.id)
    }

    /// Get object from region
    pub async fn get_object(
        &self,
        object_id: &str,
    ) -> Result<Option<crate::network::handlers::object::SimObject>, RegionError> {
        // For now, return None (object not found)
        // In the future, this will look up the object in the region
        tracing::info!("Getting object {} from region", object_id);
        Ok(None)
    }

    /// Update object in region
    pub async fn update_object(
        &self,
        object_id: &str,
        _object: crate::network::handlers::object::SimObject,
    ) -> Result<(), RegionError> {
        // For now, just log the update
        // In the future, this will update the object in the region
        tracing::info!("Updating object {} in region", object_id);
        Ok(())
    }

    /// Remove object from region
    pub async fn remove_object(&self, object_id: &str) -> Result<(), RegionError> {
        // For now, just log the removal
        // In the future, this will remove the object from the region
        tracing::info!("Removing object {} from region", object_id);
        Ok(())
    }

    /// Restart a specific region
    pub async fn restart_region(&self, region_id: RegionId) -> Result<(), RegionError> {
        tracing::info!("Restarting region {:?}", region_id);
        
        // In production, this would:
        // 1. Save region state
        // 2. Stop region processes
        // 3. Reload configuration
        // 4. Restart region
        // For now, just log the operation
        
        Ok(())
    }

    /// Restart all regions
    pub async fn restart_all_regions(&self) -> Result<(), RegionError> {
        tracing::info!("Restarting all regions");
        
        let regions = self.regions.read().await;
        for region_id in regions.keys() {
            self.restart_region(*region_id).await?;
        }
        
        Ok(())
    }

    /// Load heightmap for a region
    pub async fn load_heightmap(&self, region_id: RegionId, file_path: &str) -> Result<(), RegionError> {
        tracing::info!("Loading heightmap for region {:?} from {}", region_id, file_path);
        
        // In production, this would:
        // 1. Read heightmap file
        // 2. Parse the data
        // 3. Apply to region terrain
        // For now, just log the operation
        
        Ok(())
    }

    /// Save heightmap for a region
    pub async fn save_heightmap(&self, region_id: RegionId, file_path: &str) -> Result<(), RegionError> {
        tracing::info!("Saving heightmap for region {:?} to {}", region_id, file_path);
        
        // In production, this would:
        // 1. Get current terrain data
        // 2. Serialize to heightmap format
        // 3. Write to file
        // For now, just log the operation
        
        Ok(())
    }

    /// Load region data from XML file
    pub async fn load_xml(&self, region_id: RegionId, file_path: &str) -> Result<(), RegionError> {
        tracing::info!("Loading region data from XML file: {} for region {:?}", file_path, region_id);
        
        // In production, this would:
        // 1. Parse XML file
        // 2. Create region from data
        // 3. Add to region manager
        // For now, just log the operation
        
        Ok(())
    }

    /// Save region data to XML file
    pub async fn save_xml(&self, region_id: RegionId, file_path: &str) -> Result<(), RegionError> {
        tracing::info!("Saving region {:?} data to XML file: {}", region_id, file_path);
        
        // In production, this would:
        // 1. Serialize region data to XML
        // 2. Write to file
        // For now, just log the operation
        
        Ok(())
    }

    /// Load region from OAR (OpenSim Archive) file
    pub async fn load_oar(&self, region_id: RegionId, file_path: &str) -> Result<(), RegionError> {
        tracing::info!("Loading region from OAR file: {} for region {:?}", file_path, region_id);
        
        // In production, this would:
        // 1. Extract OAR archive
        // 2. Parse region data, objects, terrain
        // 3. Create region with all data
        // For now, just log the operation
        
        Ok(())
    }

    /// Save region to OAR (OpenSim Archive) file
    pub async fn save_oar(&self, region_id: RegionId, file_path: &str) -> Result<(), RegionError> {
        tracing::info!("Saving region {:?} to OAR file: {}", region_id, file_path);
        
        // In production, this would:
        // 1. Collect all region data, objects, terrain
        // 2. Create OAR archive
        // 3. Write to file
        // For now, just log the operation
        
        Ok(())
    }

    /// Get all regions
    pub async fn get_all_regions(&self) -> Vec<RegionId> {
        let regions = self.regions.read().await;
        regions.keys().copied().collect()
    }
}

/// Configuration for a region
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegionConfig {
    /// Region name
    pub name: String,
    /// Region size (width, height)
    pub size: (u32, u32),
    /// Region grid location (x, y)
    pub location: (i32, i32),
    /// Terrain configuration
    pub terrain: terrain::TerrainConfig,
    /// Physics configuration
    pub physics: PhysicsConfig,
    /// Maximum number of entities
    pub max_entities: usize,
}

/// Statistics about regions
#[derive(Debug, Clone)]
pub struct RegionStats {
    /// Number of active regions
    pub region_count: usize,
    /// Total number of entities across all regions
    pub total_entities: usize,
    /// Total number of physics bodies across all regions
    pub total_physics_bodies: usize,
}

/// Errors that can occur in region operations
#[derive(Debug, thiserror::Error)]
pub enum RegionError {
    #[error("Physics error: {0}")]
    Physics(#[from] crate::ffi::physics::PhysicsError),
    
    #[error("Multi-physics error: {0}")]
    PhysicsError(String),
    
    #[error("Scene error: {0}")]
    Scene(#[from] scene::SceneError),
    
    #[error("Terrain error: {0}")]
    Terrain(#[from] terrain::TerrainError),
    
    #[error("Entity error: {0}")]
    Entity(#[from] scene::entity::EntityError),
    
    #[error("Region not found: {0:?}")]
    RegionNotFound(RegionId),
    
    #[error("Region already exists: {0:?}")]
    RegionAlreadyExists(RegionId),
    
    #[error("Avatar not found: {0}")]
    AvatarNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Detailed region information for load balancing and region assignment
#[derive(Debug, Clone)]
pub struct RegionInfoDetails {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub size_x: u32,
    pub size_y: u32,
    pub avatar_count: usize,
}

/// Region statistics for monitoring and load balancing
#[derive(Debug, Clone)]
pub struct RegionStatistics {
    pub region_id: RegionId,
    pub name: String,
    pub avatar_count: usize,
    pub object_count: usize,
    pub physics_bodies: usize,
    pub cpu_usage: f32,
    pub memory_usage: usize,
    pub is_online: bool,
}

impl std::fmt::Display for RegionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Region({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::physics::PhysicsBridge;
    use crate::state::StateManager;

    #[tokio::test]
    async fn test_region_manager_creation() {
        let physics_bridge = Arc::new(PhysicsBridge::new().unwrap());
        let state_manager = Arc::new(StateManager::new().unwrap());
        let region_manager = RegionManager::new(physics_bridge, state_manager);

        let config = RegionConfig {
            name: "Test Region".to_string(),
            size: (256, 256),
            location: (1000, 1000),
            terrain: terrain::TerrainConfig::default(),
            physics: PhysicsConfig::default(),
            max_entities: 1000,
        };

        let region_id = RegionId(1);
        let region = region_manager.create_region(region_id, config).await.unwrap();
        
        assert!(region_manager.get_region(region_id).await.is_some());
        
        region_manager.remove_region(region_id).await.unwrap();
        assert!(region_manager.get_region(region_id).await.is_none());
    }

    #[tokio::test]
    async fn test_region_location() {
        let physics_bridge = Arc::new(PhysicsBridge::new().unwrap());
        let state_manager = Arc::new(StateManager::new().unwrap());
        let region_manager = RegionManager::new(physics_bridge, state_manager);

        let config = RegionConfig {
            name: "Test Location Region".to_string(),
            size: (256, 256),
            location: (2000, 3000),
            terrain: terrain::TerrainConfig::default(),
            physics: PhysicsConfig::default(),
            max_entities: 1000,
        };

        let region_id = RegionId(42);
        let region = region_manager.create_region(region_id, config).await.unwrap();
        
        // Test getting the location
        let location = region_manager.get_region_location(region_id).await;
        assert_eq!(location, Some((2000, 3000)));
        
        // Test getting location for non-existent region
        let non_existent_id = RegionId(999);
        let no_location = region_manager.get_region_location(non_existent_id).await;
        assert_eq!(no_location, None);
        
        region_manager.remove_region(region_id).await.unwrap();
    }
} 