//! Library asset management for default assets and avatar construction
//!
//! Loads actual OpenSim assets from bin/assets/ and bin/inventory/ directories
//! following the Nini XML format used by OpenSim master.

use crate::asset::{AssetData, AssetType};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Global library asset manager instance
static LIBRARY_ASSET_MANAGER: OnceLock<Arc<RwLock<LibraryAssetManager>>> = OnceLock::new();

/// Initialize the global library asset manager
pub fn set_global_library_manager(manager: Arc<RwLock<LibraryAssetManager>>) {
    if LIBRARY_ASSET_MANAGER.set(manager).is_err() {
        warn!("Global library asset manager was already initialized");
    }
}

/// Get the global library asset manager
pub fn get_global_library_manager() -> Option<Arc<RwLock<LibraryAssetManager>>> {
    LIBRARY_ASSET_MANAGER.get().cloned()
}

/// Asset definition loaded from XML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryAsset {
    pub asset_id: Uuid,
    pub name: String,
    pub asset_type: i32,
    pub file_name: String,
    pub file_path: PathBuf,
    pub data: Option<Vec<u8>>,
}

/// Inventory folder definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryFolder {
    pub folder_id: Uuid,
    pub parent_folder_id: Uuid,
    pub name: String,
    pub folder_type: i32,
}

/// Inventory item definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItem {
    pub inventory_id: Uuid,
    pub asset_id: Uuid,
    pub folder_id: Uuid,
    pub name: String,
    pub description: String,
    pub asset_type: i32,
    pub inventory_type: i32,
    pub flags: u32,
}

/// Animation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDef {
    pub name: String,
    pub uuid: Uuid,
    pub state: String,
}

/// Default wearable information (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultWearable {
    pub asset_id: Uuid,
    pub item_id: Uuid,
    pub name: String,
    pub description: String,
    pub wearable_type: u32,
    pub asset_type: AssetType,
    pub file_path: PathBuf,
    pub everyone_mask: u32,
    pub owner_mask: u32,
    pub group_mask: u32,
    pub next_owner_mask: u32,
}

/// Default texture information (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultTexture {
    pub asset_id: Uuid,
    pub name: String,
    pub description: String,
    pub file_path: PathBuf,
    pub texture_type: String,
}

/// Default avatar configuration (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultAvatar {
    pub name: String,
    pub gender: String,
    pub wearables: Vec<DefaultWearable>,
    pub textures: Vec<DefaultTexture>,
    pub shape_asset_id: Uuid,
    pub skin_asset_id: Uuid,
    pub hair_asset_id: Uuid,
    pub eyes_asset_id: Uuid,
    pub shirt_asset_id: Option<Uuid>,
    pub pants_asset_id: Option<Uuid>,
}

/// Library asset manager - loads from actual bin/ directories
pub struct LibraryAssetManager {
    bin_path: PathBuf,
    assets_path: PathBuf,
    inventory_path: PathBuf,
    data_path: PathBuf,
    openmetaverse_path: PathBuf,

    // Loaded data
    assets: HashMap<Uuid, LibraryAsset>,
    folders: HashMap<Uuid, LibraryFolder>,
    items: HashMap<Uuid, LibraryItem>,
    animations: HashMap<String, AnimationDef>,
    animations_by_uuid: HashMap<Uuid, AnimationDef>,

    // Legacy support
    default_avatars: HashMap<String, DefaultAvatar>,

    is_initialized: bool,
}

impl LibraryAssetManager {
    /// Create a new library asset manager
    pub fn new(bin_directory: &Path) -> Result<Self> {
        let bin_path = bin_directory.to_path_buf();
        let assets_path = bin_path.join("assets");
        let inventory_path = bin_path.join("inventory");
        let data_path = bin_path.join("data");
        let openmetaverse_path = bin_path.join("openmetaverse_data");

        Ok(Self {
            bin_path,
            assets_path,
            inventory_path,
            data_path,
            openmetaverse_path,
            assets: HashMap::new(),
            folders: HashMap::new(),
            items: HashMap::new(),
            animations: HashMap::new(),
            animations_by_uuid: HashMap::new(),
            default_avatars: HashMap::new(),
            is_initialized: false,
        })
    }

    /// Initialize the library asset system by loading from bin/ directories
    pub async fn initialize(&mut self) -> Result<()> {
        if self.is_initialized {
            return Ok(());
        }

        info!(
            "Initializing library asset manager from {}",
            self.bin_path.display()
        );

        // Load asset sets from bin/assets/
        if let Err(e) = self.load_asset_sets().await {
            warn!("Failed to load asset sets: {}", e);
        }

        // Load library inventory from bin/inventory/
        if let Err(e) = self.load_library_inventory().await {
            warn!("Failed to load library inventory: {}", e);
        }

        // Load animations from bin/data/avataranimations.xml
        if let Err(e) = self.load_animations().await {
            warn!("Failed to load animations: {}", e);
        }

        // Build default avatar configurations from loaded data
        self.build_default_avatars();

        // Load Ruth baked textures for avatar appearance
        self.load_ruth_baked_textures();

        self.is_initialized = true;
        info!(
            "Library asset manager initialized: {} assets, {} folders, {} items, {} animations",
            self.assets.len(),
            self.folders.len(),
            self.items.len(),
            self.animations.len()
        );

        Ok(())
    }

    fn load_ruth_baked_textures(&mut self) {
        use crate::database::initialization::{
            load_ruth_eyes_texture, load_ruth_hair_texture, load_ruth_head_texture,
            load_ruth_lower_texture, load_ruth_upper_texture,
        };

        let ruth_textures = [
            (
                "5a9f4a74-30f2-821c-b88d-70499d3e7183",
                "Ruth Baked Head",
                load_ruth_head_texture(),
            ),
            (
                "ae2de45c-d252-50b8-5c6e-19f39ce79317",
                "Ruth Baked Upper Body",
                load_ruth_upper_texture(),
            ),
            (
                "24daea5f-0539-cfcf-047f-fbc40b2786ba",
                "Ruth Baked Lower Body",
                load_ruth_lower_texture(),
            ),
            (
                "52cc6bb6-2ee5-e632-d3ad-50197b1dcb8a",
                "Ruth Baked Eyes",
                load_ruth_eyes_texture(),
            ),
            (
                "09aac1fb-6bce-0bee-7d44-caac6dbb6c63",
                "Ruth Baked Hair",
                load_ruth_hair_texture(),
            ),
        ];

        for (uuid_str, name, data) in ruth_textures {
            if let Ok(asset_id) = Uuid::parse_str(uuid_str) {
                let asset = LibraryAsset {
                    asset_id,
                    name: name.to_string(),
                    asset_type: 0,
                    file_name: format!("{}.j2c", name),
                    file_path: PathBuf::new(),
                    data: Some(data.clone()),
                };
                info!("Added Ruth baked texture: {} ({} bytes)", name, data.len());
                self.assets.insert(asset_id, asset);
            }
        }
    }

    /// Load asset sets from bin/assets/AssetSets.xml
    async fn load_asset_sets(&mut self) -> Result<()> {
        let asset_sets_path = self.assets_path.join("AssetSets.xml");

        if !asset_sets_path.exists() {
            warn!("AssetSets.xml not found at {}", asset_sets_path.display());
            return Ok(());
        }

        let content = fs::read_to_string(&asset_sets_path)
            .map_err(|e| anyhow!(format!("Failed to read AssetSets.xml: {}", e)))?;

        // Parse Nini XML to extract asset set file references
        let asset_set_files = self.parse_nini_asset_sets(&content)?;

        info!(
            "Found {} asset sets in AssetSets.xml",
            asset_set_files.len()
        );

        // Load each asset set
        for (set_name, file_path) in asset_set_files {
            let full_path = self.assets_path.join(&file_path);
            if full_path.exists() {
                if let Err(e) = self.load_asset_set(&set_name, &full_path).await {
                    warn!("Failed to load asset set '{}': {}", set_name, e);
                }
            } else {
                debug!("Asset set file not found: {}", full_path.display());
            }
        }

        Ok(())
    }

    /// Parse Nini XML format for AssetSets.xml
    fn parse_nini_asset_sets(&self, content: &str) -> Result<Vec<(String, String)>> {
        let mut sets = Vec::new();

        // Simple XML parsing for Nini format
        // <Section Name="..."><Key Name="file" Value="..."/></Section>
        let mut current_section: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();

            // Match Section start
            if line.starts_with("<Section Name=") {
                if let Some(name) = extract_attribute(line, "Name") {
                    current_section = Some(name);
                }
            }
            // Match Key with file value
            else if line.starts_with("<Key Name=\"file\"") {
                if let Some(ref section_name) = current_section {
                    if let Some(value) = extract_attribute(line, "Value") {
                        sets.push((section_name.clone(), value));
                    }
                }
            }
            // Match Section end
            else if line.starts_with("</Section>") {
                current_section = None;
            }
        }

        Ok(sets)
    }

    /// Load a single asset set XML file
    async fn load_asset_set(&mut self, set_name: &str, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).map_err(|e| {
            anyhow!(format!(
                "Failed to read asset set {}: {}",
                path.display(),
                e
            ))
        })?;

        let set_dir = path.parent().unwrap_or(Path::new("."));
        let mut count = 0;

        // Parse each asset section
        let mut current_asset: Option<LibraryAsset> = None;
        let mut in_section = false;

        for line in content.lines() {
            let line = line.trim();

            // Skip comments
            if line.starts_with("<!--") || line.ends_with("-->") {
                continue;
            }

            // Section start
            if line.starts_with("<Section Name=") {
                if let Some(name) = extract_attribute(line, "Name") {
                    current_asset = Some(LibraryAsset {
                        asset_id: Uuid::nil(),
                        name: name,
                        asset_type: 0,
                        file_name: String::new(),
                        file_path: PathBuf::new(),
                        data: None,
                    });
                    in_section = true;
                }
            }
            // Parse keys within section
            else if in_section && line.starts_with("<Key Name=") {
                if let Some(ref mut asset) = current_asset {
                    if let Some(key_name) = extract_attribute(line, "Name") {
                        if let Some(value) = extract_attribute(line, "Value") {
                            match key_name.as_str() {
                                "assetID" => {
                                    if let Ok(uuid) = Uuid::parse_str(&value) {
                                        asset.asset_id = uuid;
                                    }
                                }
                                "name" => {
                                    asset.name = value;
                                }
                                "assetType" => {
                                    if let Ok(t) = value.parse() {
                                        asset.asset_type = t;
                                    }
                                }
                                "fileName" => {
                                    asset.file_name = value.clone();
                                    asset.file_path = set_dir.join(&value);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            // Section end
            else if line.starts_with("</Section>") {
                if let Some(asset) = current_asset.take() {
                    if !asset.asset_id.is_nil() {
                        // Load asset data if file exists
                        let mut final_asset = asset;
                        if final_asset.file_path.exists() {
                            if let Ok(data) = fs::read(&final_asset.file_path) {
                                final_asset.data = Some(data);
                            }
                        }
                        self.assets.insert(final_asset.asset_id, final_asset);
                        count += 1;
                    }
                }
                in_section = false;
            }
        }

        debug!("Loaded {} assets from {}", count, set_name);
        Ok(())
    }

    /// Load library inventory structure from bin/inventory/
    async fn load_library_inventory(&mut self) -> Result<()> {
        let libraries_path = self.inventory_path.join("Libraries.xml");

        if !libraries_path.exists() {
            warn!("Libraries.xml not found at {}", libraries_path.display());
            return Ok(());
        }

        let content = fs::read_to_string(&libraries_path)
            .map_err(|e| anyhow!(format!("Failed to read Libraries.xml: {}", e)))?;

        // Parse Libraries.xml to get folder/item file references
        let library_refs = self.parse_nini_libraries(&content)?;

        info!(
            "Found {} library references in Libraries.xml",
            library_refs.len()
        );

        // Add root library folder
        let root_folder_id = Uuid::parse_str("00000112-000f-0000-0000-000100bba000").unwrap();
        let library_owner_id = Uuid::parse_str("11111111-1111-0000-0000-000100bba000").unwrap();

        self.folders.insert(
            root_folder_id,
            LibraryFolder {
                folder_id: root_folder_id,
                parent_folder_id: Uuid::nil(),
                name: "OpenSim Library".to_string(),
                folder_type: 8, // Root folder type
            },
        );

        // Load each library's folders and items
        for (lib_name, folders_file, items_file) in library_refs {
            // Load folders
            if let Some(ref folders) = folders_file {
                let full_path = self.inventory_path.join(folders);
                if full_path.exists() {
                    if let Err(e) = self.load_inventory_folders(&full_path).await {
                        warn!("Failed to load folders for '{}': {}", lib_name, e);
                    }
                }
            }

            // Load items
            if let Some(ref items) = items_file {
                let full_path = self.inventory_path.join(items);
                if full_path.exists() {
                    if let Err(e) = self.load_inventory_items(&full_path).await {
                        warn!("Failed to load items for '{}': {}", lib_name, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse Nini XML format for Libraries.xml
    fn parse_nini_libraries(
        &self,
        content: &str,
    ) -> Result<Vec<(String, Option<String>, Option<String>)>> {
        let mut libs = Vec::new();

        let mut current_section: Option<String> = None;
        let mut folders_file: Option<String> = None;
        let mut items_file: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();

            // Skip comments
            if line.starts_with("<!--") {
                continue;
            }

            if line.starts_with("<Section Name=") {
                // Save previous section if any
                if let Some(ref name) = current_section {
                    libs.push((name.clone(), folders_file.take(), items_file.take()));
                }

                if let Some(name) = extract_attribute(line, "Name") {
                    current_section = Some(name);
                    folders_file = None;
                    items_file = None;
                }
            } else if line.starts_with("<Key Name=") {
                if let Some(key_name) = extract_attribute(line, "Name") {
                    if let Some(value) = extract_attribute(line, "Value") {
                        match key_name.as_str() {
                            "foldersFile" => folders_file = Some(value),
                            "itemsFile" => items_file = Some(value),
                            _ => {}
                        }
                    }
                }
            }
        }

        // Don't forget last section
        if let Some(name) = current_section {
            libs.push((name, folders_file, items_file));
        }

        Ok(libs)
    }

    /// Load inventory folders from XML file
    async fn load_inventory_folders(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).map_err(|e| {
            anyhow!(format!(
                "Failed to read folders file {}: {}",
                path.display(),
                e
            ))
        })?;

        let mut in_section = false;
        let mut current_folder = LibraryFolder {
            folder_id: Uuid::nil(),
            parent_folder_id: Uuid::nil(),
            name: String::new(),
            folder_type: 0,
        };

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("<!--") {
                continue;
            }

            if line.starts_with("<Section Name=") {
                in_section = true;
                current_folder = LibraryFolder {
                    folder_id: Uuid::nil(),
                    parent_folder_id: Uuid::nil(),
                    name: String::new(),
                    folder_type: 0,
                };
            } else if in_section && line.starts_with("<Key Name=") {
                if let Some(key_name) = extract_attribute(line, "Name") {
                    if let Some(value) = extract_attribute(line, "Value") {
                        match key_name.as_str() {
                            "folderID" => {
                                if let Ok(uuid) = Uuid::parse_str(&value) {
                                    current_folder.folder_id = uuid;
                                }
                            }
                            "parentFolderID" => {
                                if let Ok(uuid) = Uuid::parse_str(&value) {
                                    current_folder.parent_folder_id = uuid;
                                }
                            }
                            "name" => {
                                current_folder.name = value;
                            }
                            "type" => {
                                if let Ok(t) = value.parse() {
                                    current_folder.folder_type = t;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            } else if line.starts_with("</Section>") {
                if !current_folder.folder_id.is_nil() {
                    self.folders
                        .insert(current_folder.folder_id, current_folder.clone());
                }
                in_section = false;
            }
        }

        Ok(())
    }

    /// Load inventory items from XML file
    async fn load_inventory_items(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).map_err(|e| {
            anyhow!(format!(
                "Failed to read items file {}: {}",
                path.display(),
                e
            ))
        })?;

        let mut in_section = false;
        let mut current_item = LibraryItem {
            inventory_id: Uuid::nil(),
            asset_id: Uuid::nil(),
            folder_id: Uuid::nil(),
            name: String::new(),
            description: String::new(),
            asset_type: 0,
            inventory_type: 0,
            flags: 0,
        };

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("<!--") {
                continue;
            }

            if line.starts_with("<Section Name=") {
                in_section = true;
                current_item = LibraryItem {
                    inventory_id: Uuid::nil(),
                    asset_id: Uuid::nil(),
                    folder_id: Uuid::nil(),
                    name: String::new(),
                    description: String::new(),
                    asset_type: 0,
                    inventory_type: 0,
                    flags: 0,
                };
            } else if in_section && line.starts_with("<Key Name=") {
                if let Some(key_name) = extract_attribute(line, "Name") {
                    if let Some(value) = extract_attribute(line, "Value") {
                        match key_name.as_str() {
                            "inventoryID" => {
                                if let Ok(uuid) = Uuid::parse_str(&value) {
                                    current_item.inventory_id = uuid;
                                }
                            }
                            "assetID" => {
                                if let Ok(uuid) = Uuid::parse_str(&value) {
                                    current_item.asset_id = uuid;
                                }
                            }
                            "folderID" => {
                                if let Ok(uuid) = Uuid::parse_str(&value) {
                                    current_item.folder_id = uuid;
                                }
                            }
                            "name" => {
                                current_item.name = value;
                            }
                            "description" => {
                                current_item.description = value;
                            }
                            "assetType" => {
                                if let Ok(t) = value.parse() {
                                    current_item.asset_type = t;
                                }
                            }
                            "inventoryType" => {
                                if let Ok(t) = value.parse() {
                                    current_item.inventory_type = t;
                                }
                            }
                            "flags" => {
                                if let Ok(f) = value.parse() {
                                    current_item.flags = f;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            } else if line.starts_with("</Section>") {
                if !current_item.inventory_id.is_nil() {
                    self.items
                        .insert(current_item.inventory_id, current_item.clone());
                }
                in_section = false;
            }
        }

        Ok(())
    }

    /// Load animations from bin/data/avataranimations.xml
    async fn load_animations(&mut self) -> Result<()> {
        let anim_path = self.data_path.join("avataranimations.xml");

        if !anim_path.exists() {
            warn!("avataranimations.xml not found at {}", anim_path.display());
            return Ok(());
        }

        let content = fs::read_to_string(&anim_path)
            .map_err(|e| anyhow!(format!("Failed to read avataranimations.xml: {}", e)))?;

        // Parse <animation name="..." state="...">UUID</animation>
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("<animation ") && line.contains("</animation>") {
                if let Some(name) = extract_attribute(line, "name") {
                    let state = extract_attribute(line, "state").unwrap_or_default();

                    // Extract UUID from between > and </animation>
                    if let Some(start) = line.find('>') {
                        if let Some(end) = line.find("</animation>") {
                            let uuid_str = &line[start + 1..end];
                            if let Ok(uuid) = Uuid::parse_str(uuid_str.trim()) {
                                let anim = AnimationDef {
                                    name: name.clone(),
                                    uuid,
                                    state,
                                };
                                self.animations.insert(name, anim.clone());
                                self.animations_by_uuid.insert(uuid, anim);
                            }
                        }
                    }
                }
            }
        }

        info!("Loaded {} animations", self.animations.len());
        Ok(())
    }

    /// Build default avatar configurations from loaded library data
    fn build_default_avatars(&mut self) {
        // Find the body parts from loaded items
        let mut shape_asset = Uuid::nil();
        let mut skin_asset = Uuid::nil();
        let mut hair_asset = Uuid::nil();
        let mut eyes_asset = Uuid::nil();

        // Look up assets by their well-known names
        for item in self.items.values() {
            match item.name.as_str() {
                "Shape" => shape_asset = item.asset_id,
                "Skin" => skin_asset = item.asset_id,
                "Hair" => hair_asset = item.asset_id,
                "Default Eyes" | "Eyes" => eyes_asset = item.asset_id,
                _ => {}
            }
        }

        // Create default avatar using loaded assets
        let default_avatar = DefaultAvatar {
            name: "Default Avatar".to_string(),
            gender: "male".to_string(),
            wearables: Vec::new(),
            textures: Vec::new(),
            shape_asset_id: if shape_asset.is_nil() {
                Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfab73").unwrap()
            } else {
                shape_asset
            },
            skin_asset_id: if skin_asset.is_nil() {
                Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bbabbb").unwrap()
            } else {
                skin_asset
            },
            hair_asset_id: if hair_asset.is_nil() {
                Uuid::parse_str("d342e6c0-b9d2-11dc-95ff-0800200c9a66").unwrap()
            } else {
                hair_asset
            },
            eyes_asset_id: if eyes_asset.is_nil() {
                Uuid::parse_str("4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7").unwrap()
            } else {
                eyes_asset
            },
            shirt_asset_id: None,
            pants_asset_id: None,
        };

        self.default_avatars
            .insert("male".to_string(), default_avatar.clone());
        self.default_avatars
            .insert("female".to_string(), default_avatar);

        info!(
            "Built default avatar configurations with shape={}, skin={}, hair={}, eyes={}",
            shape_asset, skin_asset, hair_asset, eyes_asset
        );
    }

    // ==================== PUBLIC API ====================

    /// Get asset data by UUID
    pub fn get_asset(&self, asset_id: &Uuid) -> Option<&LibraryAsset> {
        self.assets.get(asset_id)
    }

    /// Get asset data bytes by UUID
    pub fn get_asset_data(&self, asset_id: &Uuid) -> Option<Vec<u8>> {
        self.assets.get(asset_id).and_then(|a| a.data.clone())
    }

    /// Check if an asset exists in library
    pub fn has_asset(&self, asset_id: &Uuid) -> bool {
        self.assets.contains_key(asset_id)
    }

    /// Get library folder by ID
    pub fn get_folder(&self, folder_id: &Uuid) -> Option<&LibraryFolder> {
        self.folders.get(folder_id)
    }

    /// Get all library folders
    pub fn get_all_folders(&self) -> &HashMap<Uuid, LibraryFolder> {
        &self.folders
    }

    /// Get library item by ID
    pub fn get_item(&self, inventory_id: &Uuid) -> Option<&LibraryItem> {
        self.items.get(inventory_id)
    }

    /// Get items in a folder
    pub fn get_items_in_folder(&self, folder_id: &Uuid) -> Vec<&LibraryItem> {
        self.items
            .values()
            .filter(|item| &item.folder_id == folder_id)
            .collect()
    }

    /// Get subfolders of a folder
    pub fn get_subfolders(&self, parent_id: &Uuid) -> Vec<&LibraryFolder> {
        self.folders
            .values()
            .filter(|folder| &folder.parent_folder_id == parent_id)
            .collect()
    }

    /// Get animation by name
    pub fn get_animation(&self, name: &str) -> Option<&AnimationDef> {
        self.animations.get(name)
    }

    /// Get animation UUID by name
    pub fn get_animation_uuid(&self, name: &str) -> Option<Uuid> {
        self.animations.get(name).map(|a| a.uuid)
    }

    /// Get animation by UUID
    pub fn get_animation_by_uuid(&self, uuid: &Uuid) -> Option<&AnimationDef> {
        self.animations_by_uuid.get(uuid)
    }

    /// Get all animations
    pub fn get_all_animations(&self) -> &HashMap<String, AnimationDef> {
        &self.animations
    }

    /// Get library root folder ID
    pub fn get_library_root_folder_id(&self) -> Uuid {
        Uuid::parse_str("00000112-000f-0000-0000-000100bba000").unwrap()
    }

    /// Get library owner ID
    pub fn get_library_owner_id(&self) -> Uuid {
        Uuid::parse_str("11111111-1111-0000-0000-000100bba000").unwrap()
    }

    // ==================== BACKWARD COMPATIBILITY ====================

    /// Get default avatar configuration (legacy API)
    pub fn get_default_avatar(&self, gender: &str) -> Option<&DefaultAvatar> {
        self.default_avatars.get(gender)
    }

    /// Get all default avatar configurations (legacy API)
    pub fn get_all_default_avatars(&self) -> &HashMap<String, DefaultAvatar> {
        &self.default_avatars
    }

    /// Get default wearables for a specific gender (legacy API)
    pub fn get_default_wearables(&self, gender: &str) -> Option<&Vec<DefaultWearable>> {
        self.default_avatars
            .get(gender)
            .map(|avatar| &avatar.wearables)
    }

    /// Get asset data for a library asset (legacy API)
    pub async fn get_library_asset(&self, asset_id: &Uuid) -> Option<AssetData> {
        self.get_asset_data(asset_id)
    }

    /// Check if an asset exists in the library (legacy API)
    pub fn is_library_asset(&self, asset_id: &Uuid) -> bool {
        self.has_asset(asset_id)
    }

    /// Get library statistics
    pub fn get_statistics(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_assets".to_string(), self.assets.len() as u64);
        stats.insert("total_folders".to_string(), self.folders.len() as u64);
        stats.insert("total_items".to_string(), self.items.len() as u64);
        stats.insert("total_animations".to_string(), self.animations.len() as u64);
        stats.insert(
            "default_avatars".to_string(),
            self.default_avatars.len() as u64,
        );

        // Count assets with loaded data
        let loaded = self.assets.values().filter(|a| a.data.is_some()).count();
        stats.insert("loaded_asset_data".to_string(), loaded as u64);

        stats
    }

    /// Get library asset paths
    pub fn get_asset_paths(&self) -> HashMap<String, PathBuf> {
        let mut paths = HashMap::new();
        paths.insert("bin".to_string(), self.bin_path.clone());
        paths.insert("assets".to_string(), self.assets_path.clone());
        paths.insert("inventory".to_string(), self.inventory_path.clone());
        paths.insert("data".to_string(), self.data_path.clone());
        paths.insert("openmetaverse".to_string(), self.openmetaverse_path.clone());
        paths
    }

    // ==================== LOGIN RESPONSE METHODS ====================

    /// Get library root folder for login response
    pub fn get_library_root_for_login(&self) -> Vec<LibraryLoginFolder> {
        let root_id = self.get_library_root_folder_id();
        vec![LibraryLoginFolder {
            folder_id: root_id.to_string(),
            parent_id: Uuid::nil().to_string(),
            name: "OpenSim Library".to_string(),
            type_default: "8".to_string(),
            version: "1".to_string(),
        }]
    }

    /// Get library skeleton (all folders INCLUDING root) for login response
    /// OpenSim includes the root folder in inventory-skel-lib
    pub fn get_library_skeleton_for_login(&self) -> Vec<LibraryLoginFolder> {
        let root_id = self.get_library_root_folder_id();

        // Start with the root folder
        let mut folders = vec![LibraryLoginFolder {
            folder_id: root_id.to_string(),
            parent_id: Uuid::nil().to_string(),
            name: "OpenSim Library".to_string(),
            type_default: "8".to_string(),
            version: "1".to_string(),
        }];

        // Add all other folders
        folders.extend(
            self.folders
                .values()
                .filter(|folder| folder.folder_id != root_id)
                .map(|folder| LibraryLoginFolder {
                    folder_id: folder.folder_id.to_string(),
                    parent_id: if folder.parent_folder_id.is_nil() {
                        // Folders with nil parent should be children of the library root
                        root_id.to_string()
                    } else {
                        folder.parent_folder_id.to_string()
                    },
                    name: folder.name.clone(),
                    type_default: folder.folder_type.to_string(),
                    version: "1".to_string(),
                }),
        );

        folders
    }

    /// Get library owner info for login response
    pub fn get_library_owner_for_login(&self) -> Vec<LibraryLoginOwner> {
        vec![LibraryLoginOwner {
            agent_id: self.get_library_owner_id().to_string(),
        }]
    }

    /// Get folder descendants for FetchLibDescendents2 CAPS handler
    /// Returns (subfolders, items) for the requested folder
    pub fn get_folder_descendants(
        &self,
        folder_id: &Uuid,
    ) -> Option<(Vec<&LibraryFolder>, Vec<&LibraryItem>)> {
        let root_id = self.get_library_root_folder_id();

        // Find subfolders whose parent is this folder
        let subfolders: Vec<&LibraryFolder> = self
            .folders
            .values()
            .filter(|f| {
                if f.parent_folder_id == *folder_id {
                    true
                } else if f.parent_folder_id.is_nil() && *folder_id == root_id {
                    // Items with nil parent belong to root folder
                    true
                } else {
                    false
                }
            })
            .collect();

        // Find items in this folder
        let items: Vec<&LibraryItem> = self
            .items
            .values()
            .filter(|i| i.folder_id == *folder_id)
            .collect();

        // Check if folder exists (root folder or one of our folders)
        if *folder_id == root_id || self.folders.contains_key(folder_id) {
            Some((subfolders, items))
        } else {
            None
        }
    }
}

/// Library folder for login response (simplified format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryLoginFolder {
    pub folder_id: String,
    pub parent_id: String,
    pub name: String,
    pub type_default: String,
    pub version: String,
}

/// Library owner for login response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryLoginOwner {
    pub agent_id: String,
}

/// Helper function to extract XML attribute value
fn extract_attribute(line: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(start) = line.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = line[value_start..].find('"') {
            return Some(line[value_start..value_start + end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attribute() {
        let line = r#"<Key Name="assetID" Value="d342e6c0-b9d2-11dc-95ff-0800200c9a66"/>"#;
        assert_eq!(extract_attribute(line, "Name"), Some("assetID".to_string()));
        assert_eq!(
            extract_attribute(line, "Value"),
            Some("d342e6c0-b9d2-11dc-95ff-0800200c9a66".to_string())
        );
    }

    #[test]
    fn test_extract_section_name() {
        let line = r#"<Section Name="Hair">"#;
        assert_eq!(extract_attribute(line, "Name"), Some("Hair".to_string()));
    }
}
