//! Handles avatar-related messages

use std::{collections::HashMap, sync::Arc};
use anyhow::Result;
use tracing::{info, warn};

use crate::{
    network::{session::Session, llsd::LLSDValue},
    region::{
        avatar::appearance::Appearance,
        RegionManager,
    },
    asset::AssetManager,
};

/// Wearable types in Second Life
#[derive(Debug, Clone, PartialEq)]
pub enum WearableType {
    Shape = 0,
    Skin = 1,
    Hair = 2,
    Eyes = 3,
    Shirt = 4,
    Pants = 5,
    Shoes = 6,
    Socks = 7,
    Jacket = 8,
    Gloves = 9,
    Undershirt = 10,
    Underpants = 11,
    Skirt = 12,
    Alpha = 13,
    Tattoo = 14,
    Physics = 15,
}

impl WearableType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(WearableType::Shape),
            1 => Some(WearableType::Skin),
            2 => Some(WearableType::Hair),
            3 => Some(WearableType::Eyes),
            4 => Some(WearableType::Shirt),
            5 => Some(WearableType::Pants),
            6 => Some(WearableType::Shoes),
            7 => Some(WearableType::Socks),
            8 => Some(WearableType::Jacket),
            9 => Some(WearableType::Gloves),
            10 => Some(WearableType::Undershirt),
            11 => Some(WearableType::Underpants),
            12 => Some(WearableType::Skirt),
            13 => Some(WearableType::Alpha),
            14 => Some(WearableType::Tattoo),
            15 => Some(WearableType::Physics),
            _ => None,
        }
    }
}

/// Represents a single wearable item
#[derive(Debug, Clone)]
pub struct Wearable {
    pub wearable_type: WearableType,
    pub asset_id: String,
    pub item_id: String,
    pub name: String,
    pub permissions: u32,
}

/// Handles avatar-related messages
#[derive(Default)]
pub struct AvatarHandler;

impl AvatarHandler {
    /// Handles avatar appearance updates
    pub async fn handle_update_appearance(
        &self,
        session: Arc<Session>,
        region_manager: Arc<RegionManager>,
        appearance: Appearance,
    ) -> Result<()> {
        info!(
            "Handling update appearance for session: {:?}",
            session.session_id
        );
        region_manager
            .update_avatar_appearance(&session.agent_id, appearance)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Handles agent wearables requests from clients
    pub async fn handle_agent_wearables_request(
        &self,
        session: Arc<tokio::sync::RwLock<Session>>,
        region_manager: Arc<RegionManager>,
        asset_manager: Arc<AssetManager>,
    ) -> Result<LLSDValue> {
        let session_guard = session.read().await;
        let session_id = session_guard.session_id.clone();
        let agent_id = session_guard.agent_id.clone();
        drop(session_guard);

        info!("Handling agent wearables request for session: {}", session_id);

        // Get current avatar appearance from region manager
        let appearance = match region_manager.get_avatar_appearance(&agent_id.to_string()).await {
            Ok(Some(appearance)) => appearance,
            Ok(None) => {
                info!("No appearance found for agent {}, creating default", agent_id);
                self.create_default_appearance(&agent_id.to_string(), &asset_manager).await?
            }
            Err(e) => {
                warn!("Failed to get appearance for agent {}: {}", agent_id, e);
                return Err(anyhow::anyhow!("Failed to retrieve avatar appearance"));
            }
        };

        // Convert appearance to wearables data
        let wearables = self.appearance_to_wearables(&appearance).await?;

        // Create LLSD response
        let mut wearables_array = Vec::new();
        for wearable in wearables {
            let mut wearable_map = HashMap::new();
            wearable_map.insert("type".to_string(), LLSDValue::Integer(wearable.wearable_type as i32));
            wearable_map.insert("asset_id".to_string(), LLSDValue::String(wearable.asset_id));
            wearable_map.insert("item_id".to_string(), LLSDValue::String(wearable.item_id));
            wearable_map.insert("name".to_string(), LLSDValue::String(wearable.name));
            wearable_map.insert("permissions".to_string(), LLSDValue::Integer(wearable.permissions as i32));
            
            wearables_array.push(LLSDValue::Map(wearable_map));
        }

        let mut response = HashMap::new();
        response.insert("wearables".to_string(), LLSDValue::Array(wearables_array));
        response.insert("agent_id".to_string(), LLSDValue::String(agent_id.to_string()));

        info!("Sent wearables data for session {}", session_id);
        Ok(LLSDValue::Map(response))
    }

    /// Creates default appearance for new avatars
    async fn create_default_appearance(
        &self,
        agent_id: &str,
        _asset_manager: &AssetManager,
    ) -> Result<Appearance> {
        info!("Creating default appearance for agent {}", agent_id);

        // Create basic default wearables using the actual Appearance struct format
        let default_wearables = vec![
            crate::region::avatar::appearance::Wearable {
                item_id: uuid::Uuid::new_v4(),
                asset_id: uuid::Uuid::new_v4(),
            },
            crate::region::avatar::appearance::Wearable {
                item_id: uuid::Uuid::new_v4(),
                asset_id: uuid::Uuid::new_v4(),
            },
        ];

        // Create default textures
        let textures = vec![
            crate::region::avatar::appearance::TextureEntry {
                texture_id: uuid::Uuid::new_v4(),
                face: 0,
            },
        ];

        // Set default visual parameters
        let mut visual_params = crate::region::avatar::appearance::VisualParams::default();
        for i in 0..10 {
            visual_params.params.insert(i, 127.0); // Middle values
        }

        Ok(Appearance {
            serial: 1,
            visual_params,
            textures,
            wearables: default_wearables,
            attachments: Vec::new(),
            hover_height: 0.0,
            height: 1.8,
            glow: crate::region::avatar::appearance::Glow::default(),
        })
    }

    /// Converts appearance data to wearables format
    async fn appearance_to_wearables(&self, appearance: &Appearance) -> Result<Vec<Wearable>> {
        let mut wearables = Vec::new();

        for (i, wearable) in appearance.wearables.iter().enumerate() {
            let wearable_type = match i {
                0 => WearableType::Shape,
                1 => WearableType::Skin,
                2 => WearableType::Hair,
                3 => WearableType::Eyes,
                _ => WearableType::Shirt,
            };
            
            let handler_wearable = Wearable {
                wearable_type: wearable_type.clone(),
                asset_id: wearable.asset_id.to_string(),
                item_id: wearable.item_id.to_string(),
                name: format!("{:?}", wearable_type),
                permissions: 0x7FFFFFFF, // Full permissions for now
            };
            wearables.push(handler_wearable);
        }

        Ok(wearables)
    }

    /// Creates placeholder texture data for different wearable types
    fn create_placeholder_texture(&self, _wearable_type: &WearableType) -> bytes::Bytes {
        // Create a minimal 1x1 PNG as placeholder
        let placeholder_png = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
            0x49, 0x48, 0x44, 0x52, // IHDR
            0x00, 0x00, 0x00, 0x01, // Width: 1
            0x00, 0x00, 0x00, 0x01, // Height: 1
            0x08, 0x02, 0x00, 0x00, 0x00, // Bit depth: 8, Color type: 2 (RGB), etc.
            0x90, 0x77, 0x53, 0xDE, // CRC
            0x00, 0x00, 0x00, 0x0C, // IDAT chunk length
            0x49, 0x44, 0x41, 0x54, // IDAT
            0x08, 0x99, 0x01, 0x01, 0x00, 0x00, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x02, // Minimal image data
            0x00, 0x01, 0x01, 0x24, // CRC
            0x00, 0x00, 0x00, 0x00, // IEND chunk length
            0x49, 0x45, 0x4E, 0x44, // IEND
            0xAE, 0x42, 0x60, 0x82, // CRC
        ];

        bytes::Bytes::from(placeholder_png)
    }
} 