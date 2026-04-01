//! Common types and utilities shared between IAR and OAR handling

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Archive format version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    pub major_version: u32,
    pub minor_version: u32,
    pub created_at: Option<DateTime<Utc>>,
    pub creator_id: Option<Uuid>,
    pub creator_name: Option<String>,
    pub description: Option<String>,
}

impl Default for ArchiveMetadata {
    fn default() -> Self {
        Self {
            major_version: 1,
            minor_version: 0,
            created_at: Some(Utc::now()),
            creator_id: None,
            creator_name: None,
            description: None,
        }
    }
}

/// Asset entry within an archive
#[derive(Debug, Clone)]
pub struct ArchiveAsset {
    pub uuid: Uuid,
    pub asset_type: i32,
    pub name: String,
    pub data: Vec<u8>,
}

/// Result of loading statistics
#[derive(Debug, Clone, Default, Serialize)]
pub struct LoadStatistics {
    pub assets_loaded: u32,
    pub assets_skipped: u32,
    pub assets_failed: u32,
    pub folders_created: u32,
    pub items_created: u32,
    pub objects_created: u32,
    pub parcels_loaded: u32,
    pub terrain_loaded: bool,
    pub elapsed_ms: u64,
}

/// Result of saving statistics
#[derive(Debug, Clone, Default, Serialize)]
pub struct SaveStatistics {
    pub assets_saved: u32,
    pub folders_saved: u32,
    pub items_saved: u32,
    pub objects_saved: u32,
    pub parcels_saved: u32,
    pub terrain_saved: bool,
    pub archive_size_bytes: u64,
    pub elapsed_ms: u64,
}

/// OpenSim asset type constants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AssetType {
    Unknown = -1,
    Texture = 0,
    Sound = 1,
    CallingCard = 2,
    Landmark = 3,
    Clothing = 5,
    Object = 6,
    Notecard = 7,
    Folder = 8,
    LSLText = 10,
    LSLBytecode = 11,
    TextureTGA = 12,
    Bodypart = 13,
    SoundWAV = 17,
    ImageTGA = 18,
    ImageJPEG = 19,
    Animation = 20,
    Gesture = 21,
    Simstate = 22,
    Mesh = 49,
    Settings = 56,
    Material = 57,
}

impl AssetType {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => AssetType::Texture,
            1 => AssetType::Sound,
            2 => AssetType::CallingCard,
            3 => AssetType::Landmark,
            5 => AssetType::Clothing,
            6 => AssetType::Object,
            7 => AssetType::Notecard,
            8 => AssetType::Folder,
            10 => AssetType::LSLText,
            11 => AssetType::LSLBytecode,
            12 => AssetType::TextureTGA,
            13 => AssetType::Bodypart,
            17 => AssetType::SoundWAV,
            18 => AssetType::ImageTGA,
            19 => AssetType::ImageJPEG,
            20 => AssetType::Animation,
            21 => AssetType::Gesture,
            22 => AssetType::Simstate,
            49 => AssetType::Mesh,
            56 => AssetType::Settings,
            57 => AssetType::Material,
            _ => AssetType::Unknown,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            AssetType::Texture => "_texture.jp2",
            AssetType::Sound => "_sound.ogg",
            AssetType::CallingCard => "_callingcard.txt",
            AssetType::Landmark => "_landmark.txt",
            AssetType::Clothing => "_clothing.txt",
            AssetType::Object => "_object.xml",
            AssetType::Notecard => "_notecard.txt",
            AssetType::LSLText => "_script.lsl",
            AssetType::LSLBytecode => "_bytecode.lso",
            AssetType::TextureTGA => "_texture.tga",
            AssetType::Bodypart => "_bodypart.txt",
            AssetType::SoundWAV => "_sound.wav",
            AssetType::ImageTGA => "_image.tga",
            AssetType::ImageJPEG => "_image.jpg",
            AssetType::Animation => "_animation.bvh",
            AssetType::Gesture => "_gesture.txt",
            AssetType::Simstate => "_simstate.bin",
            AssetType::Mesh => "_mesh.llmesh",
            AssetType::Settings => "_settings.xml",
            AssetType::Material => "_material.gltf",
            _ => ".bin",
        }
    }
}

/// Inventory folder type constants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FolderType {
    None = -1,
    Root = 8,
    Texture = 0,
    Sound = 1,
    CallingCard = 2,
    Landmark = 3,
    Clothing = 5,
    Object = 6,
    Notecard = 7,
    LSLText = 10,
    BodyPart = 13,
    Trash = 14,
    Snapshot = 15,
    LostAndFound = 16,
    Animation = 20,
    Gesture = 21,
    Favorites = 23,
    CurrentOutfit = 46,
    Outfit = 47,
    MyOutfits = 48,
    Mesh = 49,
    Inbox = 50,
    Outbox = 51,
    BasicRoot = 52,
    MarketplaceListings = 53,
    MarkplaceStock = 54,
    Settings = 56,
    Material = 57,
    Suitcase = 100,
}

/// Archive path constants matching OpenSim format
pub mod paths {
    pub const ARCHIVE_XML: &str = "archive.xml";
    pub const ASSETS_PATH: &str = "assets/";
    pub const INVENTORY_PATH: &str = "inventory/";
    pub const OBJECTS_PATH: &str = "objects/";
    pub const TERRAINS_PATH: &str = "terrains/";
    pub const SETTINGS_PATH: &str = "settings/";
    pub const LANDDATA_PATH: &str = "landdata/";
    pub const REGION_PATH: &str = "region/";
    pub const FOLDER_METADATA: &str = "__folder_metadata.xml";
}

/// Parse a UUID from OpenSim archive format (handles both with and without dashes)
pub fn parse_uuid(s: &str) -> Option<Uuid> {
    // Try standard format first
    if let Ok(uuid) = Uuid::parse_str(s) {
        return Some(uuid);
    }
    // Try without dashes
    if s.len() == 32 {
        let with_dashes = format!(
            "{}-{}-{}-{}-{}",
            &s[0..8], &s[8..12], &s[12..16], &s[16..20], &s[20..32]
        );
        if let Ok(uuid) = Uuid::parse_str(&with_dashes) {
            return Some(uuid);
        }
    }
    None
}

/// Extract UUID from asset filename (format: {uuid}_{type}.ext)
pub fn extract_asset_uuid_from_path(path: &str) -> Option<Uuid> {
    let filename = std::path::Path::new(path)
        .file_name()?
        .to_str()?;

    // Find the first underscore which separates UUID from type
    let uuid_part = filename.split('_').next()?;
    parse_uuid(uuid_part)
}

/// Format a UUID for archive paths (lowercase, with dashes)
pub fn format_uuid_for_archive(uuid: &Uuid) -> String {
    uuid.to_string().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid_with_dashes() {
        let uuid = parse_uuid("12345678-1234-5678-9abc-123456789abc").unwrap();
        assert_eq!(uuid.to_string(), "12345678-1234-5678-9abc-123456789abc");
    }

    #[test]
    fn test_parse_uuid_without_dashes() {
        let uuid = parse_uuid("12345678123456789abc123456789abc").unwrap();
        assert_eq!(uuid.to_string(), "12345678-1234-5678-9abc-123456789abc");
    }

    #[test]
    fn test_extract_asset_uuid() {
        let uuid = extract_asset_uuid_from_path("assets/12345678-1234-5678-9abc-123456789abc_texture.jp2").unwrap();
        assert_eq!(uuid.to_string(), "12345678-1234-5678-9abc-123456789abc");
    }
}
