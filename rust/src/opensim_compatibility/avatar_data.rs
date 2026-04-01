//! Avatar data loader for openmetaverse_data files
//!
//! Loads avatar_lad.xml, avatar_skeleton.xml, and TGA textures from
//! the bin/openmetaverse_data/ directory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, error};
use anyhow::{Result, anyhow};
use uuid::Uuid;

/// Global avatar data manager instance
static AVATAR_DATA_MANAGER: OnceLock<Arc<RwLock<AvatarDataManager>>> = OnceLock::new();

/// Initialize the global avatar data manager
pub fn set_global_avatar_data_manager(manager: Arc<RwLock<AvatarDataManager>>) {
    if AVATAR_DATA_MANAGER.set(manager).is_err() {
        warn!("Global avatar data manager was already initialized");
    }
}

/// Get the global avatar data manager
pub fn get_global_avatar_data_manager() -> Option<Arc<RwLock<AvatarDataManager>>> {
    AVATAR_DATA_MANAGER.get().cloned()
}

/// Get body part wearable data by UUID from global manager (async)
pub async fn get_body_part_data_by_uuid(uuid: &uuid::Uuid) -> Option<Vec<u8>> {
    if let Some(manager) = get_global_avatar_data_manager() {
        let guard = manager.read().await;
        guard.get_wearable_data_by_uuid(uuid)
    } else {
        None
    }
}

/// Check if UUID is a body part asset (async)
pub async fn is_body_part_asset(uuid: &uuid::Uuid) -> bool {
    if let Some(manager) = get_global_avatar_data_manager() {
        let guard = manager.read().await;
        guard.is_body_part_uuid(uuid)
    } else {
        false
    }
}

/// Avatar attachment point definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentPoint {
    pub id: u32,
    pub group: u32,
    pub pie_slice: Option<u32>,
    pub name: String,
    pub joint: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub visible_in_first_person: bool,
    pub max_attachment_offset: Option<f32>,
}

/// Avatar visual parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualParam {
    pub id: u32,
    pub name: String,
    pub group: u32,
    pub wearable: Option<String>,
    pub edit_group: Option<String>,
    pub label: Option<String>,
    pub label_min: Option<String>,
    pub label_max: Option<String>,
    pub value_min: f32,
    pub value_max: f32,
    pub value_default: f32,
    pub sex: Option<String>,
}

/// Avatar bone definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarBone {
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    pub pivot: Option<[f32; 3]>,
    pub children: Vec<AvatarBone>,
    pub collision_volumes: Vec<CollisionVolume>,
}

/// Collision volume definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionVolume {
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

/// Avatar skeleton definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarSkeleton {
    pub version: String,
    pub num_bones: u32,
    pub num_collision_volumes: u32,
    pub root_bone: Option<AvatarBone>,
}

/// Avatar texture data (loaded from TGA files)
#[derive(Debug, Clone)]
pub struct AvatarTexture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Linden Avatar Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LindenAvatarDefinition {
    pub version: String,
    pub wearable_definition_version: u32,
    pub attachment_points: Vec<AttachmentPoint>,
    pub visual_params: Vec<VisualParam>,
}

/// Avatar data manager - loads from openmetaverse_data directory
#[derive(Debug)]
pub struct AvatarDataManager {
    openmetaverse_data_path: PathBuf,
    bin_directory: PathBuf,
    avatar_definition: Option<LindenAvatarDefinition>,
    skeleton: Option<AvatarSkeleton>,
    textures: HashMap<String, AvatarTexture>,
    attachment_points_by_id: HashMap<u32, AttachmentPoint>,
    visual_params_by_id: HashMap<u32, VisualParam>,
    body_parts_loader: Option<BodyPartsLoader>,
}

impl AvatarDataManager {
    /// Create a new avatar data manager
    pub fn new(bin_directory: &Path) -> Result<Self> {
        let openmetaverse_data_path = bin_directory.join("openmetaverse_data");

        if !openmetaverse_data_path.exists() {
            warn!("openmetaverse_data directory not found at {}", openmetaverse_data_path.display());
        }

        Ok(Self {
            openmetaverse_data_path,
            bin_directory: bin_directory.to_path_buf(),
            avatar_definition: None,
            skeleton: None,
            textures: HashMap::new(),
            attachment_points_by_id: HashMap::new(),
            visual_params_by_id: HashMap::new(),
            body_parts_loader: None,
        })
    }

    /// Initialize the avatar data system by loading all files
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing avatar data manager from {}", self.openmetaverse_data_path.display());

        // Load avatar_lad.xml
        self.load_avatar_lad().await?;

        // Load avatar_skeleton.xml
        self.load_avatar_skeleton().await?;

        // Load TGA textures
        self.load_textures().await?;

        // Load body parts (LLWearable .dat files)
        let mut body_parts = BodyPartsLoader::new(&self.bin_directory);
        body_parts.load().await?;
        self.body_parts_loader = Some(body_parts);

        info!("Avatar data manager initialized: {} attachment points, {} visual params, {} textures",
              self.attachment_points_by_id.len(),
              self.visual_params_by_id.len(),
              self.textures.len());

        Ok(())
    }

    /// Load avatar_lad.xml (Linden Avatar Definition)
    async fn load_avatar_lad(&mut self) -> Result<()> {
        let lad_path = self.openmetaverse_data_path.join("avatar_lad.xml");

        if !lad_path.exists() {
            warn!("avatar_lad.xml not found at {}", lad_path.display());
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&lad_path).await
            .map_err(|e| anyhow!("Failed to read avatar_lad.xml: {}", e))?;

        // Parse the XML manually since it has a complex structure
        let mut attachment_points = Vec::new();
        let mut visual_params = Vec::new();
        let mut version = "1.0".to_string();
        let mut wearable_version = 22u32;

        // Extract version info
        if let Some(cap) = regex::Regex::new(r#"version="([^"]+)""#).ok()
            .and_then(|re| re.captures(&content)) {
            version = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or(version);
        }

        if let Some(cap) = regex::Regex::new(r#"wearable_definition_version="(\d+)""#).ok()
            .and_then(|re| re.captures(&content)) {
            wearable_version = cap.get(1)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(wearable_version);
        }

        // Parse attachment points
        let attachment_re = regex::Regex::new(
            r#"<attachment_point\s+id="(\d+)"\s+group="(\d+)"(?:\s+pie_slice="(\d+)")?\s+name="([^"]+)"\s+joint="([^"]+)"\s+position="([^"]+)"\s+rotation="([^"]+)"(?:\s+visible_in_first_person="([^"]+)")?(?:\s+max_attachment_offset="([^"]+)")?\s*/>"#
        ).map_err(|e| anyhow!("Regex error: {}", e))?;

        for cap in attachment_re.captures_iter(&content) {
            let position = Self::parse_vector3(cap.get(6).map(|m| m.as_str()).unwrap_or("0 0 0"));
            let rotation = Self::parse_vector3(cap.get(7).map(|m| m.as_str()).unwrap_or("0 0 0"));

            let ap = AttachmentPoint {
                id: cap.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                group: cap.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                pie_slice: cap.get(3).and_then(|m| m.as_str().parse().ok()),
                name: cap.get(4).map(|m| m.as_str().to_string()).unwrap_or_default(),
                joint: cap.get(5).map(|m| m.as_str().to_string()).unwrap_or_default(),
                position,
                rotation,
                visible_in_first_person: cap.get(8).map(|m| m.as_str() == "true").unwrap_or(true),
                max_attachment_offset: cap.get(9).and_then(|m| m.as_str().parse().ok()),
            };

            self.attachment_points_by_id.insert(ap.id, ap.clone());
            attachment_points.push(ap);
        }

        // Parse visual parameters (simplified - just extract id, name, value ranges)
        let param_re = regex::Regex::new(
            r#"<param\s+id="(\d+)"(?:\s+group="(\d+)")?(?:\s+wearable="([^"]*)")?(?:\s+edit_group="([^"]*)")?(?:\s+name="([^"]*)")?[^>]*value_min="([^"]*)"[^>]*value_max="([^"]*)"[^>]*value_default="([^"]*)"#
        ).map_err(|e| anyhow!("Regex error: {}", e))?;

        for cap in param_re.captures_iter(&content) {
            let vp = VisualParam {
                id: cap.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                group: cap.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                wearable: cap.get(3).map(|m| m.as_str().to_string()),
                edit_group: cap.get(4).map(|m| m.as_str().to_string()),
                name: cap.get(5).map(|m| m.as_str().to_string()).unwrap_or_default(),
                label: None,
                label_min: None,
                label_max: None,
                value_min: cap.get(6).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0),
                value_max: cap.get(7).and_then(|m| m.as_str().parse().ok()).unwrap_or(1.0),
                value_default: cap.get(8).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0),
                sex: None,
            };

            if vp.id > 0 {
                self.visual_params_by_id.insert(vp.id, vp.clone());
                visual_params.push(vp);
            }
        }

        self.avatar_definition = Some(LindenAvatarDefinition {
            version,
            wearable_definition_version: wearable_version,
            attachment_points,
            visual_params,
        });

        info!("Loaded avatar_lad.xml: {} attachment points, {} visual params",
              self.attachment_points_by_id.len(),
              self.visual_params_by_id.len());

        Ok(())
    }

    /// Load avatar_skeleton.xml
    async fn load_avatar_skeleton(&mut self) -> Result<()> {
        let skeleton_path = self.openmetaverse_data_path.join("avatar_skeleton.xml");

        if !skeleton_path.exists() {
            warn!("avatar_skeleton.xml not found at {}", skeleton_path.display());
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&skeleton_path).await
            .map_err(|e| anyhow!("Failed to read avatar_skeleton.xml: {}", e))?;

        // Parse skeleton header
        let mut version = "1.0".to_string();
        let mut num_bones = 0u32;
        let mut num_collision_volumes = 0u32;

        if let Some(cap) = regex::Regex::new(r#"version="([^"]+)""#).ok()
            .and_then(|re| re.captures(&content)) {
            version = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or(version);
        }

        if let Some(cap) = regex::Regex::new(r#"num_bones="(\d+)""#).ok()
            .and_then(|re| re.captures(&content)) {
            num_bones = cap.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        }

        if let Some(cap) = regex::Regex::new(r#"num_collision_volumes="(\d+)""#).ok()
            .and_then(|re| re.captures(&content)) {
            num_collision_volumes = cap.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        }

        // Parse root bone (mPelvis)
        let root_bone = self.parse_bone_recursive(&content, "mPelvis");

        self.skeleton = Some(AvatarSkeleton {
            version,
            num_bones,
            num_collision_volumes,
            root_bone,
        });

        info!("Loaded avatar_skeleton.xml: {} bones, {} collision volumes",
              num_bones, num_collision_volumes);

        Ok(())
    }

    /// Parse a bone and its children recursively
    fn parse_bone_recursive(&self, content: &str, bone_name: &str) -> Option<AvatarBone> {
        // Find bone definition
        let bone_pattern = format!(
            r#"<bone name="{}" pos="([^"]*)" rot="([^"]*)" scale="([^"]*)"(?:\s+pivot="([^"]*)")?"#,
            regex::escape(bone_name)
        );

        let bone_re = regex::Regex::new(&bone_pattern).ok()?;
        let cap = bone_re.captures(content)?;

        let position = Self::parse_vector3(cap.get(1).map(|m| m.as_str()).unwrap_or("0 0 0"));
        let rotation = Self::parse_vector3(cap.get(2).map(|m| m.as_str()).unwrap_or("0 0 0"));
        let scale = Self::parse_vector3(cap.get(3).map(|m| m.as_str()).unwrap_or("1 1 1"));
        let pivot = cap.get(4).map(|m| Self::parse_vector3(m.as_str()));

        // Parse collision volumes for this bone
        let mut collision_volumes = Vec::new();
        let cv_pattern = format!(
            r#"<bone name="{}[^>]*>.*?<collision_volume name="([^"]*)" pos\s*=\s*"([^"]*)" rot="([^"]*)" scale="([^"]*)""#,
            regex::escape(bone_name)
        );

        if let Ok(cv_re) = regex::Regex::new(&cv_pattern) {
            for cv_cap in cv_re.captures_iter(content) {
                collision_volumes.push(CollisionVolume {
                    name: cv_cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                    position: Self::parse_vector3(cv_cap.get(2).map(|m| m.as_str()).unwrap_or("0 0 0")),
                    rotation: Self::parse_vector3(cv_cap.get(3).map(|m| m.as_str()).unwrap_or("0 0 0")),
                    scale: Self::parse_vector3(cv_cap.get(4).map(|m| m.as_str()).unwrap_or("1 1 1")),
                });
            }
        }

        Some(AvatarBone {
            name: bone_name.to_string(),
            position,
            rotation,
            scale,
            pivot,
            children: Vec::new(), // Children parsed separately for simplicity
            collision_volumes,
        })
    }

    /// Load TGA texture files
    async fn load_textures(&mut self) -> Result<()> {
        let mut loaded_count = 0;

        let mut entries = match tokio::fs::read_dir(&self.openmetaverse_data_path).await {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read openmetaverse_data directory: {}", e);
                return Ok(());
            }
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("tga") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    match self.load_tga_texture(&path).await {
                        Ok(texture) => {
                            self.textures.insert(name.to_string(), texture);
                            loaded_count += 1;
                        }
                        Err(e) => {
                            debug!("Failed to load TGA texture {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        info!("Loaded {} TGA textures from openmetaverse_data", loaded_count);
        Ok(())
    }

    /// Load a single TGA texture file
    async fn load_tga_texture(&self, path: &Path) -> Result<AvatarTexture> {
        let data = tokio::fs::read(path).await
            .map_err(|e| anyhow!("Failed to read TGA file: {}", e))?;

        // Parse TGA header (18 bytes minimum)
        if data.len() < 18 {
            return Err(anyhow!("TGA file too small"));
        }

        let id_length = data[0] as usize;
        let color_map_type = data[1];
        let image_type = data[2];
        let width = u16::from_le_bytes([data[12], data[13]]) as u32;
        let height = u16::from_le_bytes([data[14], data[15]]) as u32;
        let pixel_depth = data[16];

        // We only support uncompressed true-color (type 2) and grayscale (type 3)
        if image_type != 2 && image_type != 3 {
            return Err(anyhow!("Unsupported TGA image type: {}", image_type));
        }

        let header_size = 18 + id_length + if color_map_type == 1 {
            // Color map size calculation would go here
            0
        } else {
            0
        };

        let pixel_data = data[header_size..].to_vec();

        Ok(AvatarTexture {
            name: path.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            width,
            height,
            data: pixel_data,
        })
    }

    /// Parse a space-separated vector3 string
    fn parse_vector3(s: &str) -> [f32; 3] {
        let parts: Vec<f32> = s.split_whitespace()
            .filter_map(|p| p.parse().ok())
            .collect();

        [
            parts.get(0).copied().unwrap_or(0.0),
            parts.get(1).copied().unwrap_or(0.0),
            parts.get(2).copied().unwrap_or(0.0),
        ]
    }

    /// Get attachment point by ID
    pub fn get_attachment_point(&self, id: u32) -> Option<&AttachmentPoint> {
        self.attachment_points_by_id.get(&id)
    }

    /// Get all attachment points
    pub fn get_attachment_points(&self) -> &HashMap<u32, AttachmentPoint> {
        &self.attachment_points_by_id
    }

    /// Get visual parameter by ID
    pub fn get_visual_param(&self, id: u32) -> Option<&VisualParam> {
        self.visual_params_by_id.get(&id)
    }

    /// Get all visual parameters
    pub fn get_visual_params(&self) -> &HashMap<u32, VisualParam> {
        &self.visual_params_by_id
    }

    /// Get skeleton definition
    pub fn get_skeleton(&self) -> Option<&AvatarSkeleton> {
        self.skeleton.as_ref()
    }

    /// Get avatar definition
    pub fn get_avatar_definition(&self) -> Option<&LindenAvatarDefinition> {
        self.avatar_definition.as_ref()
    }

    /// Get texture by name
    pub fn get_texture(&self, name: &str) -> Option<&AvatarTexture> {
        self.textures.get(name)
    }

    /// Get all texture names
    pub fn get_texture_names(&self) -> Vec<&String> {
        self.textures.keys().collect()
    }

    /// Get texture count
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Get default visual parameter value
    pub fn get_default_param_value(&self, id: u32) -> f32 {
        self.visual_params_by_id
            .get(&id)
            .map(|p| p.value_default)
            .unwrap_or(0.0)
    }

    /// Get the body parts loader
    pub fn get_body_parts(&self) -> Option<&BodyPartsLoader> {
        self.body_parts_loader.as_ref()
    }

    /// Get default shape wearable
    pub fn get_default_shape(&self) -> Option<&LLWearable> {
        self.body_parts_loader.as_ref().and_then(|l| l.get_default_shape())
    }

    /// Get default skin wearable
    pub fn get_default_skin(&self) -> Option<&LLWearable> {
        self.body_parts_loader.as_ref().and_then(|l| l.get_default_skin())
    }

    /// Get default hair wearable
    pub fn get_default_hair(&self) -> Option<&LLWearable> {
        self.body_parts_loader.as_ref().and_then(|l| l.get_default_hair())
    }

    /// Get default eyes wearable
    pub fn get_default_eyes(&self) -> Option<&LLWearable> {
        self.body_parts_loader.as_ref().and_then(|l| l.get_default_eyes())
    }

    /// Get all default body part parameters combined
    pub fn get_all_default_body_params(&self) -> HashMap<u32, f32> {
        self.body_parts_loader
            .as_ref()
            .map(|l| l.get_all_default_parameters())
            .unwrap_or_default()
    }

    pub fn get_wearable_by_uuid(&self, uuid: &Uuid) -> Option<&LLWearable> {
        self.body_parts_loader.as_ref().and_then(|l| l.get_wearable_by_uuid(uuid))
    }

    pub fn get_wearable_data_by_uuid(&self, uuid: &Uuid) -> Option<Vec<u8>> {
        self.body_parts_loader.as_ref().and_then(|l| l.get_wearable_data_by_uuid(uuid))
    }

    pub fn is_body_part_uuid(&self, uuid: &Uuid) -> bool {
        self.body_parts_loader.as_ref().map(|l| l.is_body_part_uuid(uuid)).unwrap_or(false)
    }
}

/// LLWearable type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
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
    Physics = 21,
    Universal = 22,
    Invalid = 255,
}

impl From<u8> for WearableType {
    fn from(value: u8) -> Self {
        match value {
            0 => WearableType::Shape,
            1 => WearableType::Skin,
            2 => WearableType::Hair,
            3 => WearableType::Eyes,
            4 => WearableType::Shirt,
            5 => WearableType::Pants,
            6 => WearableType::Shoes,
            7 => WearableType::Socks,
            8 => WearableType::Jacket,
            9 => WearableType::Gloves,
            10 => WearableType::Undershirt,
            11 => WearableType::Underpants,
            12 => WearableType::Skirt,
            13 => WearableType::Alpha,
            14 => WearableType::Tattoo,
            21 => WearableType::Physics,
            22 => WearableType::Universal,
            _ => WearableType::Invalid,
        }
    }
}

impl WearableType {
    pub fn is_body_part(&self) -> bool {
        matches!(self, WearableType::Shape | WearableType::Skin | WearableType::Hair | WearableType::Eyes)
    }

    pub fn is_clothing(&self) -> bool {
        matches!(self,
            WearableType::Shirt | WearableType::Pants | WearableType::Shoes |
            WearableType::Socks | WearableType::Jacket | WearableType::Gloves |
            WearableType::Undershirt | WearableType::Underpants | WearableType::Skirt |
            WearableType::Alpha | WearableType::Tattoo | WearableType::Physics |
            WearableType::Universal
        )
    }
}

/// Permissions for LLWearable
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WearablePermissions {
    pub base_mask: u32,
    pub owner_mask: u32,
    pub group_mask: u32,
    pub everyone_mask: u32,
    pub next_owner_mask: u32,
    pub creator_id: String,
    pub owner_id: String,
    pub last_owner_id: String,
    pub group_id: String,
}

/// Sale info for LLWearable
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WearableSaleInfo {
    pub sale_type: String,
    pub sale_price: i32,
}

/// Visual parameter entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WearableParameter {
    pub id: u32,
    pub value: f32,
}

/// Texture entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WearableTexture {
    pub slot: u8,
    pub uuid: String,
}

/// Parsed LLWearable data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLWearable {
    pub version: u32,
    pub name: String,
    pub permissions: WearablePermissions,
    pub sale_info: WearableSaleInfo,
    pub wearable_type: WearableType,
    pub parameters: Vec<WearableParameter>,
    pub textures: Vec<WearableTexture>,
}

impl LLWearable {
    /// Parse LLWearable from text content
    pub fn parse(content: &str) -> Result<Self> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Err(anyhow!("Empty wearable data"));
        }

        // Parse header: "LLWearable version <number>"
        let version = Self::parse_version(lines[0])?;

        // Line 1 is the name
        let name = lines.get(1)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Parse permissions block
        let permissions = Self::parse_permissions(&lines)?;

        // Parse sale info block
        let sale_info = Self::parse_sale_info(&lines)?;

        // Parse type
        let wearable_type = Self::parse_type(&lines)?;

        // Parse parameters
        let parameters = Self::parse_parameters(&lines)?;

        // Parse textures
        let textures = Self::parse_textures(&lines)?;

        Ok(Self {
            version,
            name,
            permissions,
            sale_info,
            wearable_type,
            parameters,
            textures,
        })
    }

    fn parse_version(line: &str) -> Result<u32> {
        let line = line.trim();
        if !line.starts_with("LLWearable version ") {
            return Err(anyhow!("Invalid wearable header: {}", line));
        }
        line.strip_prefix("LLWearable version ")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| anyhow!("Invalid version number"))
    }

    fn parse_permissions(lines: &[&str]) -> Result<WearablePermissions> {
        let mut perms = WearablePermissions::default();

        for line in lines {
            let line = line.trim();
            if line.starts_with("base_mask") {
                perms.base_mask = Self::parse_hex_value(line, "base_mask");
            } else if line.starts_with("owner_mask") {
                perms.owner_mask = Self::parse_hex_value(line, "owner_mask");
            } else if line.starts_with("group_mask") {
                perms.group_mask = Self::parse_hex_value(line, "group_mask");
            } else if line.starts_with("everyone_mask") {
                perms.everyone_mask = Self::parse_hex_value(line, "everyone_mask");
            } else if line.starts_with("next_owner_mask") {
                perms.next_owner_mask = Self::parse_hex_value(line, "next_owner_mask");
            } else if line.starts_with("creator_id") {
                perms.creator_id = Self::parse_string_value(line, "creator_id");
            } else if line.starts_with("owner_id") {
                perms.owner_id = Self::parse_string_value(line, "owner_id");
            } else if line.starts_with("last_owner_id") {
                perms.last_owner_id = Self::parse_string_value(line, "last_owner_id");
            } else if line.starts_with("group_id") {
                perms.group_id = Self::parse_string_value(line, "group_id");
            }
        }

        Ok(perms)
    }

    fn parse_sale_info(lines: &[&str]) -> Result<WearableSaleInfo> {
        let mut info = WearableSaleInfo::default();

        for line in lines {
            let line = line.trim();
            if line.starts_with("sale_type") {
                info.sale_type = Self::parse_string_value(line, "sale_type");
            } else if line.starts_with("sale_price") {
                info.sale_price = Self::parse_string_value(line, "sale_price")
                    .parse()
                    .unwrap_or(10);
            }
        }

        Ok(info)
    }

    fn parse_type(lines: &[&str]) -> Result<WearableType> {
        for line in lines {
            let line = line.trim();
            if line.starts_with("type ") {
                let type_num: u8 = line.strip_prefix("type ")
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(255);
                return Ok(WearableType::from(type_num));
            }
        }
        Ok(WearableType::Invalid)
    }

    fn parse_parameters(lines: &[&str]) -> Result<Vec<WearableParameter>> {
        let mut params = Vec::new();
        let mut in_params = false;
        let mut param_count = 0;
        let mut params_parsed = 0;

        for line in lines {
            let line = line.trim();

            if line.starts_with("parameters ") {
                param_count = line.strip_prefix("parameters ")
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                in_params = true;
                continue;
            }

            if in_params && params_parsed < param_count {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let (Some(id), Some(value)) = (
                        parts[0].parse::<u32>().ok(),
                        parts[1].parse::<f32>().ok()
                    ) {
                        params.push(WearableParameter { id, value });
                        params_parsed += 1;
                    }
                }
            }

            if line.starts_with("textures ") {
                break;
            }
        }

        Ok(params)
    }

    fn parse_textures(lines: &[&str]) -> Result<Vec<WearableTexture>> {
        let mut textures = Vec::new();
        let mut in_textures = false;
        let mut tex_count = 0;
        let mut textures_parsed = 0;

        for line in lines {
            let line = line.trim();

            if line.starts_with("textures ") {
                tex_count = line.strip_prefix("textures ")
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                in_textures = true;
                continue;
            }

            if in_textures && textures_parsed < tex_count {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Some(slot) = parts[0].parse::<u8>().ok() {
                        let uuid = parts[1].to_string();
                        textures.push(WearableTexture { slot, uuid });
                        textures_parsed += 1;
                    }
                }
            }
        }

        Ok(textures)
    }

    fn parse_hex_value(line: &str, prefix: &str) -> u32 {
        line.strip_prefix(prefix)
            .map(|s| s.trim())
            .and_then(|s| u32::from_str_radix(s, 16).ok())
            .unwrap_or(0)
    }

    fn parse_string_value(line: &str, prefix: &str) -> String {
        line.strip_prefix(prefix)
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }

    /// Serialize to LLWearable format
    pub fn to_wearable_string(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("LLWearable version {}\n", self.version));
        output.push_str(&format!("{}\n\n", self.name));

        output.push_str("\tpermissions 0\n\t{\n");
        output.push_str(&format!("\t\tbase_mask\t{:08x}\n", self.permissions.base_mask));
        output.push_str(&format!("\t\towner_mask\t{:08x}\n", self.permissions.owner_mask));
        output.push_str(&format!("\t\tgroup_mask\t{:08x}\n", self.permissions.group_mask));
        output.push_str(&format!("\t\teveryone_mask\t{:08x}\n", self.permissions.everyone_mask));
        output.push_str(&format!("\t\tnext_owner_mask\t{:08x}\n", self.permissions.next_owner_mask));
        output.push_str(&format!("\t\tcreator_id\t{}\n", self.permissions.creator_id));
        output.push_str(&format!("\t\towner_id\t{}\n", self.permissions.owner_id));
        output.push_str(&format!("\t\tlast_owner_id\t{}\n", self.permissions.last_owner_id));
        output.push_str(&format!("\t\tgroup_id\t{}\n", self.permissions.group_id));
        output.push_str("\t}\n");

        output.push_str("\tsale_info\t0\n\t{\n");
        output.push_str(&format!("\t\tsale_type\t{}\n", self.sale_info.sale_type));
        output.push_str(&format!("\t\tsale_price\t{}\n", self.sale_info.sale_price));
        output.push_str("\t}\n");

        output.push_str(&format!("type {}\n", self.wearable_type as u8));
        output.push_str(&format!("parameters {}\n", self.parameters.len()));
        for param in &self.parameters {
            output.push_str(&format!("{} {}\n", param.id, param.value));
        }

        output.push_str(&format!("textures {}\n", self.textures.len()));
        for tex in &self.textures {
            output.push_str(&format!("{} {}\n", tex.slot, tex.uuid));
        }

        output
    }

    /// Get parameter value by ID
    pub fn get_parameter(&self, id: u32) -> Option<f32> {
        self.parameters.iter()
            .find(|p| p.id == id)
            .map(|p| p.value)
    }

    /// Get texture UUID by slot
    pub fn get_texture(&self, slot: u8) -> Option<&str> {
        self.textures.iter()
            .find(|t| t.slot == slot)
            .map(|t| t.uuid.as_str())
    }
}

/// Body parts loader - loads and parses all default body parts
#[derive(Debug)]
pub struct BodyPartsLoader {
    body_parts_path: PathBuf,
    default_shape: Option<LLWearable>,
    default_skin: Option<LLWearable>,
    default_hair: Option<LLWearable>,
    default_eyes: Option<LLWearable>,
}

impl BodyPartsLoader {
    pub fn new(bin_directory: &Path) -> Self {
        Self {
            body_parts_path: bin_directory.join("assets/BodyPartsAssetSet"),
            default_shape: None,
            default_skin: None,
            default_hair: None,
            default_eyes: None,
        }
    }

    pub async fn load(&mut self) -> Result<()> {
        info!("Loading body parts from {}", self.body_parts_path.display());

        // Load base_shape.dat
        let shape_path = self.body_parts_path.join("base_shape.dat");
        if shape_path.exists() {
            match tokio::fs::read_to_string(&shape_path).await {
                Ok(content) => match LLWearable::parse(&content) {
                    Ok(wearable) => {
                        info!("Loaded base_shape.dat: {} parameters", wearable.parameters.len());
                        self.default_shape = Some(wearable);
                    }
                    Err(e) => warn!("Failed to parse base_shape.dat: {}", e),
                }
                Err(e) => warn!("Failed to read base_shape.dat: {}", e),
            }
        }

        // Load base_skin.dat
        let skin_path = self.body_parts_path.join("base_skin.dat");
        if skin_path.exists() {
            match tokio::fs::read_to_string(&skin_path).await {
                Ok(content) => match LLWearable::parse(&content) {
                    Ok(wearable) => {
                        info!("Loaded base_skin.dat: {} parameters, {} textures",
                              wearable.parameters.len(), wearable.textures.len());
                        self.default_skin = Some(wearable);
                    }
                    Err(e) => warn!("Failed to parse base_skin.dat: {}", e),
                }
                Err(e) => warn!("Failed to read base_skin.dat: {}", e),
            }
        }

        // Load base_hair.dat
        let hair_path = self.body_parts_path.join("base_hair.dat");
        if hair_path.exists() {
            match tokio::fs::read_to_string(&hair_path).await {
                Ok(content) => match LLWearable::parse(&content) {
                    Ok(wearable) => {
                        info!("Loaded base_hair.dat: {} parameters, {} textures",
                              wearable.parameters.len(), wearable.textures.len());
                        self.default_hair = Some(wearable);
                    }
                    Err(e) => warn!("Failed to parse base_hair.dat: {}", e),
                }
                Err(e) => warn!("Failed to read base_hair.dat: {}", e),
            }
        }

        // Load base_eyes.dat
        let eyes_path = self.body_parts_path.join("base_eyes.dat");
        if eyes_path.exists() {
            match tokio::fs::read_to_string(&eyes_path).await {
                Ok(content) => match LLWearable::parse(&content) {
                    Ok(wearable) => {
                        info!("Loaded base_eyes.dat: {} parameters, {} textures",
                              wearable.parameters.len(), wearable.textures.len());
                        self.default_eyes = Some(wearable);
                    }
                    Err(e) => warn!("Failed to parse base_eyes.dat: {}", e),
                }
                Err(e) => warn!("Failed to read base_eyes.dat: {}", e),
            }
        }

        let loaded = [
            self.default_shape.is_some(),
            self.default_skin.is_some(),
            self.default_hair.is_some(),
            self.default_eyes.is_some(),
        ].iter().filter(|&&x| x).count();

        info!("Body parts loader initialized: {}/4 body parts loaded", loaded);
        Ok(())
    }

    pub fn get_default_shape(&self) -> Option<&LLWearable> {
        self.default_shape.as_ref()
    }

    pub fn get_default_skin(&self) -> Option<&LLWearable> {
        self.default_skin.as_ref()
    }

    pub fn get_default_hair(&self) -> Option<&LLWearable> {
        self.default_hair.as_ref()
    }

    pub fn get_default_eyes(&self) -> Option<&LLWearable> {
        self.default_eyes.as_ref()
    }

    pub fn get_all_default_parameters(&self) -> HashMap<u32, f32> {
        let mut params = HashMap::new();

        if let Some(shape) = &self.default_shape {
            for p in &shape.parameters {
                params.insert(p.id, p.value);
            }
        }
        if let Some(skin) = &self.default_skin {
            for p in &skin.parameters {
                params.insert(p.id, p.value);
            }
        }
        if let Some(hair) = &self.default_hair {
            for p in &hair.parameters {
                params.insert(p.id, p.value);
            }
        }
        if let Some(eyes) = &self.default_eyes {
            for p in &eyes.parameters {
                params.insert(p.id, p.value);
            }
        }

        params
    }

    pub fn get_wearable_by_uuid(&self, uuid: &Uuid) -> Option<&LLWearable> {
        let uuid_str = uuid.to_string().to_lowercase();
        match uuid_str.as_str() {
            "66c41e39-38f9-f75a-024e-585989bfab73" => self.default_shape.as_ref(),
            "77c41e39-38f9-f75a-024e-585989bbabbb" => self.default_skin.as_ref(),
            "d342e6c0-b9d2-11dc-95ff-0800200c9a66" => self.default_hair.as_ref(),
            "4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7" => self.default_eyes.as_ref(),
            _ => None,
        }
    }

    pub fn get_wearable_data_by_uuid(&self, uuid: &Uuid) -> Option<Vec<u8>> {
        self.get_wearable_by_uuid(uuid)
            .map(|w| w.to_wearable_string().into_bytes())
    }

    pub fn is_body_part_uuid(&self, uuid: &Uuid) -> bool {
        let uuid_str = uuid.to_string().to_lowercase();
        matches!(uuid_str.as_str(),
            "66c41e39-38f9-f75a-024e-585989bfab73" |
            "77c41e39-38f9-f75a-024e-585989bbabbb" |
            "d342e6c0-b9d2-11dc-95ff-0800200c9a66" |
            "4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vector3() {
        assert_eq!(AvatarDataManager::parse_vector3("1.0 2.0 3.0"), [1.0, 2.0, 3.0]);
        assert_eq!(AvatarDataManager::parse_vector3("0 0 0"), [0.0, 0.0, 0.0]);
        assert_eq!(AvatarDataManager::parse_vector3("-1.5 0.5 2"), [-1.5, 0.5, 2.0]);
    }

    #[test]
    fn test_wearable_type_conversion() {
        assert_eq!(WearableType::from(0), WearableType::Shape);
        assert_eq!(WearableType::from(1), WearableType::Skin);
        assert_eq!(WearableType::from(2), WearableType::Hair);
        assert_eq!(WearableType::from(3), WearableType::Eyes);
        assert!(WearableType::Shape.is_body_part());
        assert!(!WearableType::Shirt.is_body_part());
        assert!(WearableType::Shirt.is_clothing());
    }

    #[test]
    fn test_llwearable_parse_shape() {
        let content = r#"LLWearable version 22
New Shape

	permissions 0
	{
		base_mask	7fffffff
		owner_mask	7fffffff
		group_mask	00000000
		everyone_mask	00000000
		next_owner_mask	00082000
		creator_id	11111111-1111-0000-0000-000100bba000
		owner_id	11111111-1111-0000-0000-000100bba000
		last_owner_id	00000000-0000-0000-0000-000000000000
		group_id	00000000-0000-0000-0000-000000000000
	}
	sale_info	0
	{
		sale_type	not
		sale_price	10
	}
type 0
parameters 3
1 0
2 0.5
3 1.0
textures 0"#;

        let wearable = LLWearable::parse(content).unwrap();
        assert_eq!(wearable.version, 22);
        assert_eq!(wearable.name, "New Shape");
        assert_eq!(wearable.wearable_type, WearableType::Shape);
        assert_eq!(wearable.parameters.len(), 3);
        assert_eq!(wearable.parameters[0].id, 1);
        assert_eq!(wearable.parameters[0].value, 0.0);
        assert_eq!(wearable.parameters[1].id, 2);
        assert_eq!(wearable.parameters[1].value, 0.5);
        assert_eq!(wearable.textures.len(), 0);
    }

    #[test]
    fn test_llwearable_parse_skin_with_textures() {
        let content = r#"LLWearable version 22
Test Skin

	permissions 0
	{
		base_mask	00000000
		owner_mask	00000000
		group_mask	00000000
		everyone_mask	00000000
		next_owner_mask	00000000
		creator_id	11111111-1111-0000-0000-000100bba000
		owner_id	11111111-1111-0000-0000-000100bba000
		last_owner_id	00000000-0000-0000-0000-000000000000
		group_id	00000000-0000-0000-0000-000000000000
	}
	sale_info	0
	{
		sale_type	not
		sale_price	10
	}
type 1
parameters 2
108 0
110 0.5
textures 2
0 00000000-0000-1111-9999-000000000012
5 00000000-0000-1111-9999-000000000010"#;

        let wearable = LLWearable::parse(content).unwrap();
        assert_eq!(wearable.wearable_type, WearableType::Skin);
        assert_eq!(wearable.parameters.len(), 2);
        assert_eq!(wearable.textures.len(), 2);
        assert_eq!(wearable.textures[0].slot, 0);
        assert_eq!(wearable.textures[0].uuid, "00000000-0000-1111-9999-000000000012");
        assert_eq!(wearable.textures[1].slot, 5);
    }

    #[test]
    fn test_llwearable_get_parameter() {
        let content = r#"LLWearable version 22
Test

	permissions 0
	{
	}
	sale_info	0
	{
	}
type 0
parameters 2
100 0.25
200 0.75
textures 0"#;

        let wearable = LLWearable::parse(content).unwrap();
        assert_eq!(wearable.get_parameter(100), Some(0.25));
        assert_eq!(wearable.get_parameter(200), Some(0.75));
        assert_eq!(wearable.get_parameter(999), None);
    }

    #[test]
    fn test_llwearable_roundtrip() {
        let original = LLWearable {
            version: 22,
            name: "Test Wearable".to_string(),
            permissions: WearablePermissions {
                base_mask: 0x7fffffff,
                owner_mask: 0x7fffffff,
                ..Default::default()
            },
            sale_info: WearableSaleInfo {
                sale_type: "not".to_string(),
                sale_price: 10,
            },
            wearable_type: WearableType::Shape,
            parameters: vec![
                WearableParameter { id: 1, value: 0.5 },
                WearableParameter { id: 2, value: 0.25 },
            ],
            textures: vec![],
        };

        let serialized = original.to_wearable_string();
        let parsed = LLWearable::parse(&serialized).unwrap();

        assert_eq!(parsed.version, original.version);
        assert_eq!(parsed.name, original.name);
        assert_eq!(parsed.wearable_type, original.wearable_type);
        assert_eq!(parsed.parameters.len(), original.parameters.len());
    }

    #[test]
    fn test_body_part_uuid_detection() {
        let mut loader = BodyPartsLoader::new(std::path::Path::new("/nonexistent"));
        loader.default_shape = Some(LLWearable {
            version: 22,
            name: "Test Shape".to_string(),
            permissions: WearablePermissions::default(),
            sale_info: WearableSaleInfo { sale_type: "not".to_string(), sale_price: 0 },
            wearable_type: WearableType::Shape,
            parameters: vec![],
            textures: vec![],
        });

        let shape_uuid = Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfab73").unwrap();
        let skin_uuid = Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bbabbb").unwrap();
        let random_uuid = Uuid::new_v4();

        assert!(loader.is_body_part_uuid(&shape_uuid));
        assert!(loader.is_body_part_uuid(&skin_uuid));
        assert!(!loader.is_body_part_uuid(&random_uuid));

        assert!(loader.get_wearable_by_uuid(&shape_uuid).is_some());
        assert!(loader.get_wearable_by_uuid(&skin_uuid).is_none());

        let data = loader.get_wearable_data_by_uuid(&shape_uuid);
        assert!(data.is_some());
        assert!(data.unwrap().contains(&b'L'));
    }
}
