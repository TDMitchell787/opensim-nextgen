//! Physics bridge for communicating with Zig physics engine
//! 
//! This module provides the interface between Rust and the Zig physics engine.

pub use crate::ffi::{Vec3, PhysicsBody, FFIError, PhysicsHeightfield};

/// Physics data for entities
#[derive(Debug, Clone)]
pub struct PhysicsData {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub shape: PhysicsShape,
}

/// Physics shape types
#[derive(Debug, Clone)]
pub enum PhysicsShape {
    Box { size: Vec3 },
    Sphere { radius: f32 },
    Capsule { radius: f32, height: f32 },
    ConvexHulls { hulls: Vec<Vec<[f32; 3]>> },
}

/// Physics configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub time_step: f32,
    pub max_iterations: u32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_step: 1.0 / 60.0,
            max_iterations: 10,
        }
    }
}

/// Physics bridge for communicating with Zig physics engine
pub struct PhysicsBridge {
    physics: Option<crate::ffi::Physics>,
}

impl PhysicsBridge {
    /// Create a new physics bridge
    pub fn new() -> Result<Self, FFIError> {
        let physics = crate::ffi::Physics::new()?;
        Ok(Self { physics: Some(physics) })
    }

    /// Create a disabled physics bridge (no FFI initialization required)
    pub fn new_disabled() -> Self {
        Self { physics: None }
    }

    /// Step the physics simulation
    pub fn step(&mut self, delta_time: f32) -> Result<(), FFIError> {
        if let Some(physics) = &mut self.physics {
            physics.step(delta_time)
        } else {
            Ok(()) // No-op when disabled
        }
    }

    /// Create a physics body
    pub fn create_body(&self, _physics_data: PhysicsData) -> Result<PhysicsBody, FFIError> {
        // For now, return a placeholder - this would integrate with the actual physics engine
        Ok(PhysicsBody { _private: [] })
    }

    /// Destroy a physics body
    pub fn destroy_body(&self, _body: PhysicsBody) -> Result<(), FFIError> {
        // For now, do nothing - this would integrate with the actual physics engine
        Ok(())
    }

    /// Set body position
    pub fn set_body_position(&self, _body: PhysicsBody, _position: Vec3) -> Result<(), FFIError> {
        // For now, do nothing - this would integrate with the actual physics engine
        Ok(())
    }

    /// Get body position
    pub fn get_body_position(&self, _body: PhysicsBody) -> Result<Vec3, FFIError> {
        // For now, return zero position - this would integrate with the actual physics engine
        Ok(Vec3::new(0.0, 0.0, 0.0))
    }

    /// Get error count from physics engine
    pub fn get_error_count(&self) -> i32 {
        self.physics.as_ref().map(|p| p.get_error_count()).unwrap_or(0)
    }

    /// Get last error from physics engine
    pub fn get_last_error(&self) -> String {
        self.physics.as_ref().map(|p| p.get_last_error()).unwrap_or_else(|| "Physics disabled".to_string())
    }

    pub fn create_hull_body(
        &self,
        hull_array: &[f32],
        id: u32,
        pos: Vec3,
        rot: crate::ffi::Quat,
    ) -> Result<*mut std::ffi::c_void, FFIError> {
        if self.physics.is_none() {
            return Err(FFIError::NotInitialized);
        }
        if hull_array.is_empty() {
            return Err(FFIError::InvalidParameter);
        }

        let hull_count = hull_array[0] as i32;
        let shape = unsafe {
            crate::ffi::physics_create_hull_shape(hull_count, hull_array.as_ptr())
        };
        if shape.is_null() {
            tracing::warn!("[MESH-PHYSICS] CreateHullShape2 returned null for id={}, falling back to capsule", id);
            return Err(FFIError::CreationFailed);
        }

        let body = unsafe {
            crate::ffi::physics_create_body_from_shape(
                shape, id,
                pos.x, pos.y, pos.z,
                rot.x, rot.y, rot.z, rot.w,
            )
        };
        if body.is_null() {
            unsafe { crate::ffi::physics_delete_collision_shape(shape); }
            tracing::warn!("[MESH-PHYSICS] CreateBodyFromShape2 returned null for id={}", id);
            return Err(FFIError::CreationFailed);
        }

        Ok(body)
    }
}

/// Errors that can occur in physics operations
#[derive(Debug, thiserror::Error)]
pub enum PhysicsError {
    #[error("FFI error: {0}")]
    FFI(#[from] FFIError),
    
    #[error("Invalid physics data: {0}")]
    InvalidData(String),
    
    #[error("Physics body not found")]
    BodyNotFound,
    
    #[error("Physics simulation error: {0}")]
    SimulationError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
} 