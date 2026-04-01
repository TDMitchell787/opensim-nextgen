pub mod appearance;

use uuid::Uuid;
use crate::region::scene::entity::{Entity, EntityId, EntityType};
use crate::region::avatar::appearance::Appearance;
use crate::ffi::Vec3;

/// Represents an avatar in the scene
#[derive(Debug, Clone)]
pub struct Avatar {
    pub entity: Entity,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub appearance: Appearance,
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub is_online: bool,
    pub session_id: Option<String>,
}

impl Avatar {
    /// Create a new avatar
    pub fn new(user_id: Uuid, first_name: String, last_name: String) -> Self {
        let entity_id = EntityId(user_id.as_u128() as u64); // Use UUID as entity ID
        let entity = Entity::new(entity_id, EntityType::Avatar);
        
        Self {
            entity,
            user_id,
            first_name,
            last_name,
            appearance: Appearance::default_female(),
            position: [128.0, 128.0, 25.0], // Center of region, slightly elevated
            look_at: [1.0, 0.0, 0.0], // Looking east
            is_online: true,
            session_id: None,
        }
    }
    
    /// Set avatar position
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = [x, y, z];
        self.entity.position = Vec3::new(x, y, z);
    }
    
    /// Set avatar look at direction
    pub fn set_look_at(&mut self, x: f32, y: f32, z: f32) {
        self.look_at = [x, y, z];
    }
    
    /// Get avatar display name
    pub fn display_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
    
    /// Set session ID for this avatar
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }
    
    /// Mark avatar as online/offline
    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
    }
    
    /// Get avatar's entity ID
    pub fn entity_id(&self) -> EntityId {
        self.entity.id
    }
} 