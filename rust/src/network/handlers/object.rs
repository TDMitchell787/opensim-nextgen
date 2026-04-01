//! Handles object-related messages for Second Life protocol

use std::{collections::HashMap, sync::Arc};
use anyhow::{anyhow, Result};
use tracing::info;
use uuid::Uuid;

use crate::{
    network::{session::Session, llsd::LLSDValue},
    region::RegionManager,
    asset::AssetManager,
};

/// Represents a 3D position
#[derive(Debug, Clone, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Represents a rotation quaternion
#[derive(Debug, Clone, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Object types supported in the virtual world
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    Primitive,    // Basic geometric shapes (cube, sphere, etc.)
    Mesh,        // Custom mesh objects
    Attachment,  // Objects attached to avatars
    HUD,         // Heads-up display objects
}

/// Permission flags for objects
#[derive(Debug, Clone)]
pub struct ObjectPermissions {
    pub owner_mask: u32,
    pub group_mask: u32,
    pub everyone_mask: u32,
    pub next_owner_mask: u32,
}

impl Default for ObjectPermissions {
    fn default() -> Self {
        Self {
            owner_mask: 0x7FFFFFFF,  // Full permissions for owner
            group_mask: 0x00000000,  // No group permissions
            everyone_mask: 0x00000000, // No public permissions
            next_owner_mask: 0x00082000, // Transfer and modify for next owner
        }
    }
}

/// Represents a virtual world object
#[derive(Debug, Clone)]
pub struct SimObject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub group_id: Option<String>,
    pub object_type: ObjectType,
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub permissions: ObjectPermissions,
    pub asset_id: Option<String>,  // For mesh objects
    pub texture_ids: Vec<String>,  // Texture assets applied to faces
    pub is_phantom: bool,          // Whether object has physics collision
    pub is_temporary: bool,        // Whether object is temporary
    pub created_at: u64,           // Unix timestamp
    pub updated_at: u64,           // Unix timestamp
}

/// Handles object-related messages
#[derive(Default)]
pub struct ObjectHandler;

impl ObjectHandler {
    /// Handles object creation requests
    pub async fn handle_object_add(
        &self,
        session: Arc<tokio::sync::RwLock<Session>>,
        region_manager: Arc<RegionManager>,
        _asset_manager: Arc<AssetManager>,
        object_data: LLSDValue,
    ) -> Result<LLSDValue> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        let agent_id = session_guard.agent_id.clone();
        drop(session_guard);

        info!("Handling object creation for session: {}", session_id);

        // Parse object data from LLSD
        let object = self.parse_object_from_llsd(object_data, &agent_id.to_string())?;

        // Validate object creation permissions
        self.validate_object_creation(&object, &agent_id.to_string(), &region_manager).await?;

        // Create object in region
        let object_id = region_manager.add_object(object.clone()).await
            .map_err(|e| anyhow!("Failed to add object to region: {}", e))?;

        info!("Created object {} for session {}", object_id, session_id);

        // Return success response
        let mut response = HashMap::new();
        response.insert("success".to_string(), LLSDValue::Boolean(true));
        response.insert("object_id".to_string(), LLSDValue::String(object_id));
        response.insert("message".to_string(), LLSDValue::String("Object created successfully".to_string()));

        Ok(LLSDValue::Map(response))
    }

    /// Handles object update requests
    pub async fn handle_object_update(
        &self,
        session: Arc<tokio::sync::RwLock<Session>>,
        region_manager: Arc<RegionManager>,
        object_id: &str,
        update_data: LLSDValue,
    ) -> Result<LLSDValue> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        let agent_id = session_guard.agent_id.clone();
        drop(session_guard);

        info!("Handling object update for object {} from session {}", object_id, session_id);

        // Get existing object
        let existing_object = region_manager.get_object(object_id).await
            .map_err(|e| anyhow!("Failed to get object: {}", e))?
            .ok_or_else(|| anyhow!("Object not found: {}", object_id))?;

        // Check permissions
        if existing_object.owner_id != agent_id.to_string() {
            return Err(anyhow!("No permission to modify object"));
        }

        // Apply updates
        let updated_object = self.apply_object_updates(existing_object, update_data)?;

        // Update object in region
        region_manager.update_object(object_id, updated_object).await
            .map_err(|e| anyhow!("Failed to update object: {}", e))?;

        info!("Updated object {} for session {}", object_id, session_id);

        // Return success response
        let mut response = HashMap::new();
        response.insert("success".to_string(), LLSDValue::Boolean(true));
        response.insert("object_id".to_string(), LLSDValue::String(object_id.to_string()));
        response.insert("message".to_string(), LLSDValue::String("Object updated successfully".to_string()));

        Ok(LLSDValue::Map(response))
    }

    /// Handles object deletion requests
    pub async fn handle_object_delete(
        &self,
        session: Arc<tokio::sync::RwLock<Session>>,
        region_manager: Arc<RegionManager>,
        object_id: &str,
    ) -> Result<LLSDValue> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        let agent_id = session_guard.agent_id.clone();
        drop(session_guard);

        info!("Handling object deletion for object {} from session {}", object_id, session_id);

        // Get existing object to check permissions
        let existing_object = region_manager.get_object(object_id).await
            .map_err(|e| anyhow!("Failed to get object: {}", e))?
            .ok_or_else(|| anyhow!("Object not found: {}", object_id))?;

        // Check permissions
        if existing_object.owner_id != agent_id.to_string() {
            return Err(anyhow!("No permission to delete object"));
        }

        // Remove object from region
        region_manager.remove_object(object_id).await
            .map_err(|e| anyhow!("Failed to remove object: {}", e))?;

        info!("Deleted object {} for session {}", object_id, session_id);

        // Return success response
        let mut response = HashMap::new();
        response.insert("success".to_string(), LLSDValue::Boolean(true));
        response.insert("object_id".to_string(), LLSDValue::String(object_id.to_string()));
        response.insert("message".to_string(), LLSDValue::String("Object deleted successfully".to_string()));

        Ok(LLSDValue::Map(response))
    }

    /// Handles object query requests (get object information)
    pub async fn handle_object_query(
        &self,
        session: Arc<tokio::sync::RwLock<Session>>,
        region_manager: Arc<RegionManager>,
        object_id: &str,
    ) -> Result<LLSDValue> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        drop(session_guard);

        info!("Handling object query for object {} from session {}", object_id, session_id);

        // Get object from region
        let object = region_manager.get_object(object_id).await
            .map_err(|e| anyhow!("Failed to get object: {}", e))?
            .ok_or_else(|| anyhow!("Object not found: {}", object_id))?;

        // Convert object to LLSD format
        let object_llsd = self.object_to_llsd(&object)?;

        info!("Retrieved object {} for session {}", object_id, session_id);

        Ok(object_llsd)
    }

    /// Parses object data from LLSD format
    fn parse_object_from_llsd(&self, data: LLSDValue, owner_id: &str) -> Result<SimObject> {
        let map = match data {
            LLSDValue::Map(m) => m,
            _ => return Err(anyhow!("Object data must be a map")),
        };

        let name = match map.get("name") {
            Some(LLSDValue::String(s)) => s.clone(),
            _ => "Unnamed Object".to_string(),
        };

        let description = match map.get("description") {
            Some(LLSDValue::String(s)) => s.clone(),
            _ => "".to_string(),
        };

        // Parse position
        let position = match map.get("position") {
            Some(LLSDValue::Array(arr)) if arr.len() >= 3 => {
                let x = self.extract_float(&arr[0])?;
                let y = self.extract_float(&arr[1])?;
                let z = self.extract_float(&arr[2])?;
                Vector3 { x, y, z }
            }
            _ => Vector3 { x: 128.0, y: 128.0, z: 21.0 }, // Default region center
        };

        // Parse rotation
        let rotation = match map.get("rotation") {
            Some(LLSDValue::Array(arr)) if arr.len() >= 4 => {
                let x = self.extract_float(&arr[0])?;
                let y = self.extract_float(&arr[1])?;
                let z = self.extract_float(&arr[2])?;
                let w = self.extract_float(&arr[3])?;
                Quaternion { x, y, z, w }
            }
            _ => Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }, // Identity rotation
        };

        // Parse scale
        let scale = match map.get("scale") {
            Some(LLSDValue::Array(arr)) if arr.len() >= 3 => {
                let x = self.extract_float(&arr[0])?;
                let y = self.extract_float(&arr[1])?;
                let z = self.extract_float(&arr[2])?;
                Vector3 { x, y, z }
            }
            _ => Vector3 { x: 1.0, y: 1.0, z: 1.0 }, // Default unit scale
        };

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(SimObject {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            owner_id: owner_id.to_string(),
            group_id: None,
            object_type: ObjectType::Primitive, // Default to primitive
            position,
            rotation,
            scale,
            permissions: ObjectPermissions::default(),
            asset_id: None,
            texture_ids: Vec::new(),
            is_phantom: false,
            is_temporary: false,
            created_at: current_time,
            updated_at: current_time,
        })
    }

    /// Validates object creation permissions
    async fn validate_object_creation(
        &self,
        _object: &SimObject,
        _agent_id: &str,
        _region_manager: &RegionManager,
    ) -> Result<()> {
        // Basic validation - in a real implementation, this would check:
        // - Region permissions (can user create objects?)
        // - Land permissions (does user own the land or have build rights?)
        // - Object limits (prim count, script limits, etc.)
        // - Avatar permissions (is user banned, suspended, etc.)
        
        // For now, allow all object creation
        Ok(())
    }

    /// Applies updates to an existing object
    fn apply_object_updates(&self, mut object: SimObject, update_data: LLSDValue) -> Result<SimObject> {
        let map = match update_data {
            LLSDValue::Map(m) => m,
            _ => return Err(anyhow!("Update data must be a map")),
        };

        // Update name if provided
        if let Some(LLSDValue::String(name)) = map.get("name") {
            object.name = name.clone();
        }

        // Update description if provided
        if let Some(LLSDValue::String(desc)) = map.get("description") {
            object.description = desc.clone();
        }

        // Update position if provided
        if let Some(LLSDValue::Array(arr)) = map.get("position") {
            if arr.len() >= 3 {
                object.position.x = self.extract_float(&arr[0])?;
                object.position.y = self.extract_float(&arr[1])?;
                object.position.z = self.extract_float(&arr[2])?;
            }
        }

        // Update rotation if provided
        if let Some(LLSDValue::Array(arr)) = map.get("rotation") {
            if arr.len() >= 4 {
                object.rotation.x = self.extract_float(&arr[0])?;
                object.rotation.y = self.extract_float(&arr[1])?;
                object.rotation.z = self.extract_float(&arr[2])?;
                object.rotation.w = self.extract_float(&arr[3])?;
            }
        }

        // Update scale if provided
        if let Some(LLSDValue::Array(arr)) = map.get("scale") {
            if arr.len() >= 3 {
                object.scale.x = self.extract_float(&arr[0])?;
                object.scale.y = self.extract_float(&arr[1])?;
                object.scale.z = self.extract_float(&arr[2])?;
            }
        }

        // Update timestamp
        object.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(object)
    }

    /// Converts an object to LLSD format
    fn object_to_llsd(&self, object: &SimObject) -> Result<LLSDValue> {
        let mut map = HashMap::new();

        map.insert("id".to_string(), LLSDValue::String(object.id.clone()));
        map.insert("name".to_string(), LLSDValue::String(object.name.clone()));
        map.insert("description".to_string(), LLSDValue::String(object.description.clone()));
        map.insert("owner_id".to_string(), LLSDValue::String(object.owner_id.clone()));

        // Position array
        let position = vec![
            LLSDValue::Real(object.position.x as f64),
            LLSDValue::Real(object.position.y as f64),
            LLSDValue::Real(object.position.z as f64),
        ];
        map.insert("position".to_string(), LLSDValue::Array(position));

        // Rotation array
        let rotation = vec![
            LLSDValue::Real(object.rotation.x as f64),
            LLSDValue::Real(object.rotation.y as f64),
            LLSDValue::Real(object.rotation.z as f64),
            LLSDValue::Real(object.rotation.w as f64),
        ];
        map.insert("rotation".to_string(), LLSDValue::Array(rotation));

        // Scale array
        let scale = vec![
            LLSDValue::Real(object.scale.x as f64),
            LLSDValue::Real(object.scale.y as f64),
            LLSDValue::Real(object.scale.z as f64),
        ];
        map.insert("scale".to_string(), LLSDValue::Array(scale));

        map.insert("is_phantom".to_string(), LLSDValue::Boolean(object.is_phantom));
        map.insert("is_temporary".to_string(), LLSDValue::Boolean(object.is_temporary));
        map.insert("created_at".to_string(), LLSDValue::Integer(object.created_at as i32));
        map.insert("updated_at".to_string(), LLSDValue::Integer(object.updated_at as i32));

        Ok(LLSDValue::Map(map))
    }

    /// Helper function to extract float from LLSD value
    fn extract_float(&self, value: &LLSDValue) -> Result<f32> {
        match value {
            LLSDValue::Real(f) => Ok(*f as f32),
            LLSDValue::Integer(i) => Ok(*i as f32),
            _ => Err(anyhow!("Expected numeric value")),
        }
    }
}