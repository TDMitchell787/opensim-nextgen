//! Network protocol definitions for OpenSim
//! 
//! This module defines the network protocol types and structures used for
//! communication between clients and the server.

use crate::ffi::Vec3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Position in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Position {
    fn from(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
    }
}

impl From<Position> for Vec3 {
    fn from(pos: Position) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            z: pos.z,
        }
    }
}

/// Rotation in 3D space (quaternion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Velocity in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Velocity {
    fn from(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
    }
}

impl From<Velocity> for Vec3 {
    fn from(vel: Velocity) -> Self {
        Self {
            x: vel.x,
            y: vel.y,
            z: vel.z,
        }
    }
}

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Client connection request
    Connect { client_id: String },
    /// Client disconnection
    Disconnect { client_id: String },
    /// Entity position update
    PositionUpdate { entity_id: u64, position: Position },
    /// Entity rotation update
    RotationUpdate { entity_id: u64, rotation: Rotation },
    /// Entity velocity update
    VelocityUpdate { entity_id: u64, velocity: Velocity },
    /// Chat message
    ChatMessage { from: String, message: String },
    /// Error response
    Error { code: u32, message: String },
}

/// Network protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum message size
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB

/// Network error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkErrorCode {
    /// Invalid message format
    InvalidMessage = 1001,
    /// Protocol version mismatch
    VersionMismatch = 1002,
    /// Authentication failed
    AuthenticationFailed = 1003,
    /// Rate limit exceeded
    RateLimitExceeded = 1004,
    /// Server error
    ServerError = 1005,
}

impl NetworkErrorCode {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    // Authentication
    Login {
        username: String,
        password: String,
    },
    LoginResponse {
        success: bool,
        message: String,
        session_id: Option<Uuid>,
    },

    // Asset Management
    AssetRequest {
        asset_id: Uuid,
        asset_type: AssetType,
    },
    AssetResponse {
        asset_id: Uuid,
        data: Option<Vec<u8>>,
        error: Option<String>,
    },

    // State Management
    StateUpdate {
        entity_id: Uuid,
        position: Position,
        rotation: Rotation,
        velocity: Velocity,
    },
    
    // Error
    Error {
        code: u32,
        message: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Sound,
    Animation,
    Mesh,
    Script,
} 