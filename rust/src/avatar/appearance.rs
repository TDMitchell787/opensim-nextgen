//! Avatar Appearance Engine for OpenSim Next
//! 
//! Provides advanced avatar appearance management including wearables,
//! textures, attachments, and visual parameters.

use super::*;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Avatar appearance management engine
#[derive(Debug)]
pub struct AppearanceEngine {
    default_wearables: HashMap<WearableType, WearableItem>,
    default_textures: HashMap<String, String>,
    visual_param_definitions: HashMap<i32, VisualParameterDefinition>,
    attachment_point_limits: HashMap<AttachmentPoint, i32>,
}

/// Visual parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualParameterDefinition {
    pub param_id: i32,
    pub name: String,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub category: VisualParameterCategory,
    pub description: String,
    pub affects_mesh: bool,
    pub affects_texture: bool,
}

impl AppearanceEngine {
    /// Create new appearance engine
    pub fn new() -> Self {
        let mut engine = Self {
            default_wearables: HashMap::new(),
            default_textures: HashMap::new(),
            visual_param_definitions: HashMap::new(),
            attachment_point_limits: HashMap::new(),
        };

        engine.initialize_defaults();
        engine.initialize_visual_parameters();
        engine.initialize_attachment_limits();
        
        engine
    }

    /// Create default avatar appearance
    pub fn create_default_appearance(&self) -> AvatarAppearance {
        info!("Creating default avatar appearance");

        AvatarAppearance {
            height: 1.8, // Default height in meters
            proportions: AvatarProportions::default(),
            wearables: self.create_default_wearables(),
            textures: self.default_textures.clone(),
            attachments: Vec::new(),
            visual_params: self.create_default_visual_params(),
        }
    }

    /// Validate avatar appearance
    pub fn validate_appearance(&self, appearance: &AvatarAppearance) -> AvatarResult<()> {
        debug!("Validating avatar appearance");

        // Validate height
        if appearance.height < 0.5 || appearance.height > 3.0 {
            return Err(AvatarError::InvalidData {
                reason: "Height must be between 0.5 and 3.0 meters".to_string(),
            });
        }

        // Validate proportions
        self.validate_proportions(&appearance.proportions)?;

        // Validate wearables
        self.validate_wearables(&appearance.wearables)?;

        // Validate attachments
        self.validate_attachments(&appearance.attachments)?;

        // Validate visual parameters
        self.validate_visual_parameters(&appearance.visual_params)?;

        debug!("Avatar appearance validation successful");
        Ok(())
    }

    /// Update wearable item
    pub fn update_wearable(
        &self,
        appearance: &mut AvatarAppearance,
        wearable: WearableItem,
    ) -> AvatarResult<()> {
        info!("Updating wearable: {:?}", wearable.wearable_type);

        // Validate wearable
        self.validate_single_wearable(&wearable)?;

        // Remove existing wearable of same type
        appearance.wearables.retain(|w| w.wearable_type != wearable.wearable_type);

        // Add new wearable
        appearance.wearables.push(wearable);

        // Sort wearables by layer
        appearance.wearables.sort_by_key(|w| w.layer);

        info!("Wearable updated successfully");
        Ok(())
    }

    /// Remove wearable item
    pub fn remove_wearable(
        &self,
        appearance: &mut AvatarAppearance,
        wearable_type: WearableType,
    ) -> AvatarResult<bool> {
        info!("Removing wearable: {:?}", wearable_type);

        let initial_len = appearance.wearables.len();
        appearance.wearables.retain(|w| w.wearable_type != wearable_type);
        
        let removed = appearance.wearables.len() < initial_len;
        if removed {
            info!("Wearable removed successfully");
        } else {
            info!("Wearable not found for removal");
        }

        Ok(removed)
    }

    /// Add attachment
    pub fn add_attachment(
        &self,
        appearance: &mut AvatarAppearance,
        attachment: AvatarAttachment,
    ) -> AvatarResult<()> {
        info!("Adding attachment to point: {:?}", attachment.attachment_point);

        // Validate attachment
        self.validate_single_attachment(&attachment)?;

        // Check attachment point limits
        let current_count = appearance
            .attachments
            .iter()
            .filter(|a| a.attachment_point == attachment.attachment_point)
            .count();

        let limit = self.attachment_point_limits
            .get(&attachment.attachment_point)
            .copied()
            .unwrap_or(1);

        if current_count >= limit as usize {
            return Err(AvatarError::InvalidData {
                reason: format!(
                    "Attachment point {:?} already at limit ({}/{})",
                    attachment.attachment_point, current_count, limit
                ),
            });
        }

        appearance.attachments.push(attachment);
        info!("Attachment added successfully");
        Ok(())
    }

    /// Remove attachment
    pub fn remove_attachment(
        &self,
        appearance: &mut AvatarAppearance,
        item_id: Uuid,
    ) -> AvatarResult<bool> {
        info!("Removing attachment: {}", item_id);

        let initial_len = appearance.attachments.len();
        appearance.attachments.retain(|a| a.item_id != item_id);
        
        let removed = appearance.attachments.len() < initial_len;
        if removed {
            info!("Attachment removed successfully");
        } else {
            info!("Attachment not found for removal");
        }

        Ok(removed)
    }

    /// Update visual parameter
    pub fn update_visual_parameter(
        &self,
        appearance: &mut AvatarAppearance,
        param_id: i32,
        value: f32,
    ) -> AvatarResult<()> {
        debug!("Updating visual parameter: {} = {}", param_id, value);

        // Validate parameter
        if let Some(definition) = self.visual_param_definitions.get(&param_id) {
            if value < definition.min_value || value > definition.max_value {
                return Err(AvatarError::InvalidData {
                    reason: format!(
                        "Visual parameter {} value {} out of range [{}, {}]",
                        param_id, value, definition.min_value, definition.max_value
                    ),
                });
            }
        } else {
            warn!("Unknown visual parameter: {}", param_id);
        }

        // Update or add parameter
        if let Some(param) = appearance.visual_params.iter_mut().find(|p| p.param_id == param_id) {
            param.value = value;
        } else {
            // Add new parameter
            if let Some(definition) = self.visual_param_definitions.get(&param_id) {
                appearance.visual_params.push(VisualParameter {
                    param_id,
                    name: definition.name.clone(),
                    value,
                    min_value: definition.min_value,
                    max_value: definition.max_value,
                    default_value: definition.default_value,
                    category: definition.category.clone(),
                });
            }
        }

        debug!("Visual parameter updated successfully");
        Ok(())
    }

    /// Get wearable by type
    pub fn get_wearable<'a>(
        &self,
        appearance: &'a AvatarAppearance,
        wearable_type: WearableType,
    ) -> Option<&'a WearableItem> {
        appearance.wearables.iter().find(|w| w.wearable_type == wearable_type)
    }

    /// Get attachment by item ID
    pub fn get_attachment<'a>(
        &self,
        appearance: &'a AvatarAppearance,
        item_id: Uuid,
    ) -> Option<&'a AvatarAttachment> {
        appearance.attachments.iter().find(|a| a.item_id == item_id)
    }

    /// Get visual parameter value
    pub fn get_visual_parameter_value(
        &self,
        appearance: &AvatarAppearance,
        param_id: i32,
    ) -> f32 {
        appearance
            .visual_params
            .iter()
            .find(|p| p.param_id == param_id)
            .map(|p| p.value)
            .or_else(|| {
                self.visual_param_definitions
                    .get(&param_id)
                    .map(|d| d.default_value)
            })
            .unwrap_or(0.0)
    }

    /// Calculate avatar bounding box
    pub fn calculate_bounding_box(&self, appearance: &AvatarAppearance) -> BoundingBox {
        let height = appearance.height;
        let width = height * 0.3; // Approximate width based on height
        let depth = width;

        BoundingBox {
            min: Vector3 {
                x: -width / 2.0,
                y: -depth / 2.0,
                z: 0.0,
            },
            max: Vector3 {
                x: width / 2.0,
                y: depth / 2.0,
                z: height,
            },
        }
    }

    /// Generate appearance hash for change detection
    pub fn generate_appearance_hash(&self, appearance: &AvatarAppearance) -> String {
        use sha2::{Digest, Sha256};
        
        let serialized = serde_json::to_string(appearance).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    // Private initialization methods

    fn initialize_defaults(&mut self) {
        // Initialize default wearables
        self.default_wearables.insert(
            WearableType::Skin,
            WearableItem {
                item_id: Uuid::new_v4(),
                asset_id: Uuid::parse_str("c228d1cf-4b5d-4ba8-84f4-899a0796aa97").unwrap(),
                wearable_type: WearableType::Skin,
                name: "Default Skin".to_string(),
                layer: 0,
                permissions: WearablePermissions {
                    owner_can_modify: true,
                    owner_can_copy: true,
                    owner_can_transfer: false,
                    group_can_modify: false,
                    everyone_can_modify: false,
                },
                parameters: Vec::new(),
            },
        );

        self.default_wearables.insert(
            WearableType::Hair,
            WearableItem {
                item_id: Uuid::new_v4(),
                asset_id: Uuid::parse_str("d342e6c0-b9d2-11dc-95ff-0800200c9a66").unwrap(),
                wearable_type: WearableType::Hair,
                name: "Default Hair".to_string(),
                layer: 1,
                permissions: WearablePermissions {
                    owner_can_modify: true,
                    owner_can_copy: true,
                    owner_can_transfer: false,
                    group_can_modify: false,
                    everyone_can_modify: false,
                },
                parameters: Vec::new(),
            },
        );

        // Initialize default textures
        self.default_textures.insert(
            "head_bodypaint".to_string(),
            "5a9f4a74-30f2-821c-b88d-70499d3e7183".to_string(),
        );
        self.default_textures.insert(
            "upper_bodypaint".to_string(),
            "5a9f4a74-30f2-821c-b88d-70499d3e7183".to_string(),
        );
        self.default_textures.insert(
            "lower_bodypaint".to_string(),
            "5a9f4a74-30f2-821c-b88d-70499d3e7183".to_string(),
        );
    }

    fn initialize_visual_parameters(&mut self) {
        // Shape parameters
        self.visual_param_definitions.insert(
            33,
            VisualParameterDefinition {
                param_id: 33,
                name: "Shape".to_string(),
                min_value: 0.0,
                max_value: 1.0,
                default_value: 0.5,
                category: VisualParameterCategory::Shape,
                description: "Overall body shape".to_string(),
                affects_mesh: true,
                affects_texture: false,
            },
        );

        self.visual_param_definitions.insert(
            80,
            VisualParameterDefinition {
                param_id: 80,
                name: "Male".to_string(),
                min_value: 0.0,
                max_value: 1.0,
                default_value: 0.0,
                category: VisualParameterCategory::Shape,
                description: "Male/Female body type".to_string(),
                affects_mesh: true,
                affects_texture: false,
            },
        );

        // Height and proportions
        self.visual_param_definitions.insert(
            25,
            VisualParameterDefinition {
                param_id: 25,
                name: "Height".to_string(),
                min_value: -2.3,
                max_value: 2.0,
                default_value: 0.0,
                category: VisualParameterCategory::Shape,
                description: "Avatar height".to_string(),
                affects_mesh: true,
                affects_texture: false,
            },
        );

        // Add more visual parameters as needed...
    }

    fn initialize_attachment_limits(&mut self) {
        // Most attachment points can have only 1 attachment
        for point in [
            AttachmentPoint::Chest,
            AttachmentPoint::Skull,
            AttachmentPoint::LeftShoulder,
            AttachmentPoint::RightShoulder,
            AttachmentPoint::LeftHand,
            AttachmentPoint::RightHand,
            AttachmentPoint::LeftFoot,
            AttachmentPoint::RightFoot,
            AttachmentPoint::Spine,
            AttachmentPoint::Pelvis,
            AttachmentPoint::Mouth,
            AttachmentPoint::Chin,
            AttachmentPoint::LeftEar,
            AttachmentPoint::RightEar,
            AttachmentPoint::Nose,
            AttachmentPoint::Neck,
        ] {
            self.attachment_point_limits.insert(point, 1);
        }

        // HUD attachment points can have multiple attachments
        for point in [
            AttachmentPoint::HudCenter2,
            AttachmentPoint::HudTopRight,
            AttachmentPoint::HudTop,
            AttachmentPoint::HudTopLeft,
            AttachmentPoint::HudCenter,
            AttachmentPoint::HudBottomLeft,
            AttachmentPoint::HudBottom,
            AttachmentPoint::HudBottomRight,
        ] {
            self.attachment_point_limits.insert(point, 5);
        }
    }

    // Private validation methods

    fn validate_proportions(&self, proportions: &AvatarProportions) -> AvatarResult<()> {
        let fields = [
            ("body_height", proportions.body_height),
            ("body_width", proportions.body_width),
            ("head_size", proportions.head_size),
            ("leg_length", proportions.leg_length),
            ("arm_length", proportions.arm_length),
            ("torso_length", proportions.torso_length),
        ];

        for (name, value) in fields {
            if value < 0.1 || value > 3.0 {
                return Err(AvatarError::InvalidData {
                    reason: format!("{} must be between 0.1 and 3.0", name),
                });
            }
        }

        Ok(())
    }

    fn validate_wearables(&self, wearables: &[WearableItem]) -> AvatarResult<()> {
        for wearable in wearables {
            self.validate_single_wearable(wearable)?;
        }
        Ok(())
    }

    fn validate_single_wearable(&self, wearable: &WearableItem) -> AvatarResult<()> {
        if wearable.name.is_empty() {
            return Err(AvatarError::InvalidData {
                reason: "Wearable name cannot be empty".to_string(),
            });
        }

        if wearable.layer < 0 || wearable.layer > 100 {
            return Err(AvatarError::InvalidData {
                reason: "Wearable layer must be between 0 and 100".to_string(),
            });
        }

        Ok(())
    }

    fn validate_attachments(&self, attachments: &[AvatarAttachment]) -> AvatarResult<()> {
        for attachment in attachments {
            self.validate_single_attachment(attachment)?;
        }
        Ok(())
    }

    fn validate_single_attachment(&self, attachment: &AvatarAttachment) -> AvatarResult<()> {
        // Validate scale
        if attachment.scale.x <= 0.0 || attachment.scale.y <= 0.0 || attachment.scale.z <= 0.0 {
            return Err(AvatarError::InvalidData {
                reason: "Attachment scale must be positive".to_string(),
            });
        }

        if attachment.scale.x > 10.0 || attachment.scale.y > 10.0 || attachment.scale.z > 10.0 {
            return Err(AvatarError::InvalidData {
                reason: "Attachment scale too large (max 10.0)".to_string(),
            });
        }

        Ok(())
    }

    fn validate_visual_parameters(&self, params: &[VisualParameter]) -> AvatarResult<()> {
        for param in params {
            if let Some(definition) = self.visual_param_definitions.get(&param.param_id) {
                if param.value < definition.min_value || param.value > definition.max_value {
                    return Err(AvatarError::InvalidData {
                        reason: format!(
                            "Visual parameter {} value {} out of range [{}, {}]",
                            param.param_id, param.value, definition.min_value, definition.max_value
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    fn create_default_wearables(&self) -> Vec<WearableItem> {
        self.default_wearables.values().cloned().collect()
    }

    fn create_default_visual_params(&self) -> Vec<VisualParameter> {
        self.visual_param_definitions
            .values()
            .map(|def| VisualParameter {
                param_id: def.param_id,
                name: def.name.clone(),
                value: def.default_value,
                min_value: def.min_value,
                max_value: def.max_value,
                default_value: def.default_value,
                category: def.category.clone(),
            })
            .collect()
    }
}

/// Bounding box for avatar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Vector3,
    pub max: Vector3,
}

impl Default for AppearanceEngine {
    fn default() -> Self {
        Self::new()
    }
}