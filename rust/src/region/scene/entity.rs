//! Entity management for scenes
//! 
//! This module provides entity-related functionality for scene management.

use crate::ffi::Vec3;

/// Entity identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

/// Entity types
#[derive(Debug, Clone)]
pub enum EntityType {
    Avatar,
    Object,
    Vehicle,
    Attachment,
}

/// Entity data
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub position: Vec3,
    pub rotation: Vec3,
    pub velocity: Vec3,
}

impl Entity {
    pub fn new(id: EntityId, entity_type: EntityType) -> Self {
        Self {
            id,
            entity_type,
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

/// Errors that can occur in entity operations
#[derive(Debug, thiserror::Error)]
pub enum EntityError {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(EntityId),
    
    #[error("Invalid entity data: {0}")]
    InvalidData(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
} 