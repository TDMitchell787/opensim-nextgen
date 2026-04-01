use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Represents the visual parameters of an avatar (shape, skin, hair, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualParams {
    // A map of visual parameter IDs to their values
    pub params: HashMap<u32, f32>,
}

/// Represents a baked texture layer on an avatar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureEntry {
    pub texture_id: Uuid,
    pub face: u32,
}

/// Represents a wearable item on an avatar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wearable {
    pub item_id: Uuid,
    pub asset_id: Uuid,
}

/// Represents a glow value for a visual parameter
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Glow {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Represents an object attached to an avatar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub item_id: Uuid,
    pub asset_id: Uuid,
    pub point: u32,
}

/// Represents the complete appearance of an avatar
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Appearance {
    pub serial: u32,
    pub visual_params: VisualParams,
    pub textures: Vec<TextureEntry>,
    pub wearables: Vec<Wearable>,
    pub attachments: Vec<Attachment>,
    pub hover_height: f32,
    pub height: f32,
    pub glow: Glow,
}

impl Appearance {
    /// Create default female appearance
    pub fn default_female() -> Self {
        let mut visual_params = HashMap::new();
        
        // Female-specific visual parameters
        visual_params.insert(31, 0.0);    // Body thickness - slim
        visual_params.insert(32, 0.0);    // Body fat - low
        visual_params.insert(33, 0.5);    // Bust size - medium
        visual_params.insert(80, 0.0);    // Masculine features - none
        visual_params.insert(126, 0.8);   // Breast physics - enabled
        visual_params.insert(659, 0.3);   // Breast bounce - moderate
        visual_params.insert(662, 0.5);   // Breast cleavage - medium
        
        // General appearance parameters
        visual_params.insert(0, 0.5);     // Shape - average
        visual_params.insert(1, 0.5);     // Skin tone - medium
        visual_params.insert(2, 0.6);     // Hair color - brown
        visual_params.insert(3, 0.5);     // Eye color - brown
        visual_params.insert(4, 0.7);     // Height - tall
        
        Self {
            serial: 1,
            visual_params: VisualParams { params: visual_params },
            textures: vec![
                // Default skin texture
                TextureEntry {
                    texture_id: Uuid::parse_str("8dcd4a48-2d1a-4bee-aed5-6808c71ed1e7").unwrap_or_default(),
                    face: 0,
                },
                // Default hair texture
                TextureEntry {
                    texture_id: Uuid::parse_str("7ca39b4c-bd19-4699-aff7-f93fd03d3e7b").unwrap_or_default(),
                    face: 1,
                },
            ],
            wearables: vec![
                // Default shape
                Wearable {
                    item_id: Uuid::new_v4(),
                    asset_id: Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfaba9").unwrap_or_default(),
                },
                // Default skin
                Wearable {
                    item_id: Uuid::new_v4(),
                    asset_id: Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bfab73").unwrap_or_default(),
                },
                // Default hair
                Wearable {
                    item_id: Uuid::new_v4(),
                    asset_id: Uuid::parse_str("88c41e39-38f9-f75a-024e-585989bfab84").unwrap_or_default(),
                },
            ],
            attachments: vec![],
            hover_height: 0.0,
            height: 1.75, // Average female height in meters
            glow: Glow::default(),
        }
    }
    
    /// Create default male appearance
    pub fn default_male() -> Self {
        let mut visual_params = HashMap::new();
        
        // Male-specific visual parameters
        visual_params.insert(31, 0.3);    // Body thickness - broader
        visual_params.insert(32, 0.1);    // Body fat - low
        visual_params.insert(33, 0.0);    // Bust size - none
        visual_params.insert(80, 1.0);    // Masculine features - full
        visual_params.insert(126, 0.0);   // Breast physics - disabled
        visual_params.insert(659, 0.0);   // Breast bounce - none
        visual_params.insert(662, 0.0);   // Breast cleavage - none
        
        // General appearance parameters
        visual_params.insert(0, 0.6);     // Shape - broader
        visual_params.insert(1, 0.4);     // Skin tone - slightly tan
        visual_params.insert(2, 0.3);     // Hair color - dark brown
        visual_params.insert(3, 0.6);     // Eye color - blue
        visual_params.insert(4, 0.8);     // Height - tall
        
        Self {
            serial: 1,
            visual_params: VisualParams { params: visual_params },
            textures: vec![
                // Default male skin texture
                TextureEntry {
                    texture_id: Uuid::parse_str("9dcd4a48-2d1a-4bee-aed5-6808c71ed1e8").unwrap_or_default(),
                    face: 0,
                },
                // Default male hair texture
                TextureEntry {
                    texture_id: Uuid::parse_str("8ca39b4c-bd19-4699-aff7-f93fd03d3e8c").unwrap_or_default(),
                    face: 1,
                },
            ],
            wearables: vec![
                // Default male shape
                Wearable {
                    item_id: Uuid::new_v4(),
                    asset_id: Uuid::parse_str("99c41e39-38f9-f75a-024e-585989bfaba8").unwrap_or_default(),
                },
                // Default male skin
                Wearable {
                    item_id: Uuid::new_v4(),
                    asset_id: Uuid::parse_str("88c41e39-38f9-f75a-024e-585989bfab84").unwrap_or_default(),
                },
                // Default male hair
                Wearable {
                    item_id: Uuid::new_v4(),
                    asset_id: Uuid::parse_str("99c41e39-38f9-f75a-024e-585989bfab95").unwrap_or_default(),
                },
            ],
            attachments: vec![],
            hover_height: 0.0,
            height: 1.85, // Average male height in meters
            glow: Glow::default(),
        }
    }
} 