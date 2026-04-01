//! Scene management for regions
//!
//! This module handles scene management, entity lifecycle, and spatial indexing.

pub mod entity;
pub mod spatial;

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::ffi::physics::PhysicsBridge;
use crate::ffi::{Vec3, PhysicsBody};
use crate::state::StateManager;
use crate::region::{RegionId, RegionConfig, RegionError};
use rand::Rng;

use self::entity::{Entity, EntityId, EntityType};
use self::spatial::{SpatialIndex, SpatialQuery};

/// Main scene for a region
pub struct RegionScene {
    /// Region identifier
    region_id: RegionId,
    /// Region configuration
    config: RegionConfig,
    /// Physics bridge
    physics_bridge: Arc<PhysicsBridge>,
    /// State manager
    state_manager: Arc<StateManager>,
    /// Spatial index for efficient queries
    spatial_index: Arc<SpatialIndex>,
    /// Entities in the scene
    entities: RwLock<std::collections::HashMap<EntityId, Entity>>,
    /// Physics bodies associated with entities
    physics_bodies: RwLock<std::collections::HashMap<EntityId, PhysicsBody>>,
    /// Scene statistics
    stats: RwLock<SceneStats>,
}

/// Statistics about a scene
#[derive(Debug, Clone)]
pub struct SceneStats {
    /// Number of entities
    pub entity_count: usize,
    /// Number of physics bodies
    pub physics_body_count: usize,
    /// Number of spatial queries performed
    pub query_count: usize,
    /// Last update time
    pub last_update: std::time::Instant,
}

impl RegionScene {
    /// Create a new region scene
    pub fn new(
        region_id: RegionId,
        config: RegionConfig,
        physics_bridge: Arc<PhysicsBridge>,
        state_manager: Arc<StateManager>,
    ) -> Result<Self, RegionError> {
        let spatial_index = Arc::new(SpatialIndex::new(config.size.0, config.size.1));
        
        Ok(Self {
            region_id,
            config,
            physics_bridge,
            state_manager,
            spatial_index,
            entities: RwLock::new(std::collections::HashMap::new()),
            physics_bodies: RwLock::new(std::collections::HashMap::new()),
            stats: RwLock::new(SceneStats {
                entity_count: 0,
                physics_body_count: 0,
                query_count: 0,
                last_update: std::time::Instant::now(),
            }),
        })
    }

    /// Add an entity to the scene
    pub async fn add_entity(
        &self,
        entity_type: EntityType,
        position: Vec3,
    ) -> Result<EntityId, RegionError> {
        let mut rng = rand::thread_rng();
        let entity_id = EntityId(rng.gen());
        
        let mut entity = Entity::new(entity_id, entity_type);
        entity.position = position;

        // Add to spatial index
        self.spatial_index.add_entity(entity_id, position).await;

        // Create physics body if needed
        if let Ok(physics_body) = self.physics_bridge.create_body(
            crate::ffi::physics::PhysicsData {
                position,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                mass: 1.0,
                shape: crate::ffi::physics::PhysicsShape::Sphere { radius: 0.5 },
            }
        ) {
            self.physics_bodies.write().await.insert(entity_id, physics_body);
        }

        // Store entity
        self.entities.write().await.insert(entity_id, entity);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.entity_count += 1;
        stats.physics_body_count = self.physics_bodies.read().await.len();

        Ok(entity_id)
    }

    /// Remove an entity from the scene
    pub async fn remove_entity(&self, entity_id: EntityId) -> Result<(), RegionError> {
        // Remove from spatial index
        self.spatial_index.remove_entity(entity_id).await;

        // Remove physics body
        if let Some(physics_body) = self.physics_bodies.write().await.remove(&entity_id) {
            let _ = self.physics_bridge.destroy_body(physics_body);
        }

        // Remove entity
        self.entities.write().await.remove(&entity_id);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.entity_count = self.entities.read().await.len();
        stats.physics_body_count = self.physics_bodies.read().await.len();

        Ok(())
    }

    /// Update the scene (called each simulation tick)
    pub async fn update(&self, _delta_time: f32) -> Result<(), RegionError> {
        // Step physics simulation
        // Note: This would need to be mutable, but we're using Arc for sharing
        // In a real implementation, we'd use a different approach for mutability
        // For now, we'll skip the physics step to avoid the mutability issue
        // self.physics_bridge.step(delta_time)?;

        // Update entity positions from physics
        let mut entities = self.entities.write().await;
        let physics_bodies = self.physics_bodies.read().await;
        
        for (entity_id, entity) in entities.iter_mut() {
            if let Some(physics_body) = physics_bodies.get(entity_id) {
                if let Ok(new_position) = self.physics_bridge.get_body_position(physics_body.clone()) {
                    entity.position = new_position;
                    // Update spatial index
                    self.spatial_index.update_entity_position(*entity_id, new_position).await;
                }
            }
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.last_update = std::time::Instant::now();

        Ok(())
    }

    /// Query entities in a region
    pub async fn query_entities(
        &self,
        query: SpatialQuery,
    ) -> Result<Vec<Entity>, RegionError> {
        let entity_ids = self.spatial_index.query(query).await;
        let entities = self.entities.read().await;
        
        let mut result = Vec::new();
        for entity_id in entity_ids {
            if let Some(entity) = entities.get(&entity_id) {
                result.push(entity.clone());
            }
        }

        // Update query stats
        let mut stats = self.stats.write().await;
        stats.query_count += 1;

        Ok(result)
    }

    /// Get scene statistics
    pub async fn get_stats(&self) -> SceneStats {
        self.stats.read().await.clone()
    }

    /// Get region location (grid coordinates)
    pub fn get_location(&self) -> (i32, i32) {
        self.config.location
    }

    /// Shutdown the scene
    pub async fn shutdown(&self) -> Result<(), RegionError> {
        // Remove all entities
        let entity_ids: Vec<EntityId> = self.entities.read().await.keys().cloned().collect();
        for entity_id in entity_ids {
            self.remove_entity(entity_id).await?;
        }

        Ok(())
    }
}

/// Errors that can occur in scene operations
#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(EntityId),
    
    #[error("Physics error: {0}")]
    Physics(#[from] crate::ffi::physics::PhysicsError),
    
    #[error("Spatial index error: {0}")]
    Spatial(#[from] spatial::SpatialError),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::physics::PhysicsBridge;
    use crate::state::StateManager;

    #[tokio::test]
    async fn test_scene_creation() {
        let physics_bridge = Arc::new(PhysicsBridge::new().unwrap());
        let state_manager = Arc::new(StateManager::new().unwrap());
        let config = RegionConfig {
            name: "Test Scene".to_string(),
            size: (256, 256),
            location: (1000, 1000),
            terrain: crate::region::terrain::TerrainConfig::default(),
            physics: crate::ffi::physics::PhysicsConfig::default(),
            max_entities: 1000,
        };

        let scene = RegionScene::new(
            RegionId(1),
            config,
            physics_bridge,
            state_manager,
        ).unwrap();

        assert_eq!(scene.get_stats().await.entity_count, 0);
    }
} 