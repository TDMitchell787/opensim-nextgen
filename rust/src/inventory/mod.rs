//! Inventory system for Second Life/OpenSim compatibility
//! Handles inventory folders, items, and structures required for login responses

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tracing::{info, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub mod folders;
pub mod items;
pub mod login_inventory;

pub use folders::*;
pub use items::*;
pub use login_inventory::*;

/// Inventory folder types as defined by Second Life protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InventoryFolderType {
    /// Root inventory folder
    Root = 8,
    /// Textures folder
    Texture = 0,
    /// Sounds folder
    Sound = 1,
    /// Calling cards folder
    CallingCard = 2,
    /// Landmarks folder
    Landmark = 3,
    /// Clothing folder
    Clothing = 5,
    /// Objects folder
    Object = 6,
    /// Notecards folder
    Notecard = 7,
    /// Scripts folder
    LSLText = 10,
    /// Body parts folder
    BodyPart = 13,
    /// Trash folder
    Trash = 14,
    /// Snapshot/Photo album folder
    Snapshot = 15,
    /// Lost and found folder
    LostAndFound = 16,
    /// Animations folder
    Animation = 20,
    /// Gestures folder
    Gesture = 21,
    /// Favorites folder
    Favorites = 23,
    /// Current outfit folder
    CurrentOutfit = 46,
    /// Outfits folder
    Outfit = 47,
    /// My outfits folder
    MyOutfits = 48,
    /// Mesh folder
    Mesh = 49,
    /// Inbox folder (Received Items)
    Inbox = 50,
    /// Outbox folder
    Outbox = 51,
    /// Marketplace listings folder
    MarketplaceListings = 53,
    /// Settings folder
    Settings = 56,
    /// Material folder
    Material = 57,
}

impl InventoryFolderType {
    /// Get the folder type as a string for XMLRPC responses
    pub fn as_string(&self) -> String {
        (*self as u8).to_string()
    }
    
    /// Get the default name for this folder type
    pub fn default_name(&self) -> &'static str {
        match self {
            Self::Root => "My Inventory",
            Self::Texture => "Textures",
            Self::Sound => "Sounds",
            Self::CallingCard => "Calling Cards",
            Self::Landmark => "Landmarks",
            Self::Clothing => "Clothing",
            Self::Object => "Objects",
            Self::Notecard => "Notecards",
            Self::LSLText => "Scripts",
            Self::BodyPart => "Body Parts",
            Self::Trash => "Trash",
            Self::Snapshot => "Photo Album",
            Self::LostAndFound => "Lost And Found",
            Self::Animation => "Animations",
            Self::Gesture => "Gestures",
            Self::Favorites => "Favorites",
            Self::CurrentOutfit => "Current Outfit",
            Self::Outfit => "Outfits",
            Self::MyOutfits => "My Outfits",
            Self::Mesh => "Meshes",
            Self::Inbox => "Received Items",
            Self::Outbox => "Merchant Outbox",
            Self::MarketplaceListings => "Marketplace Listings",
            Self::Settings => "Settings",
            Self::Material => "Materials",
        }
    }
    
    /// Check if this folder type should be created by default
    pub fn is_default_folder(&self) -> bool {
        matches!(self,
            Self::Root | Self::Texture | Self::Sound | Self::CallingCard |
            Self::Landmark | Self::Clothing | Self::Object | Self::Notecard |
            Self::LSLText | Self::BodyPart | Self::Trash | Self::Snapshot |
            Self::LostAndFound | Self::Animation | Self::Gesture | Self::Favorites |
            Self::CurrentOutfit | Self::MyOutfits | Self::Settings | Self::Inbox |
            Self::Material
        )
    }
}

/// Inventory asset types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InventoryAssetType {
    /// Texture asset
    Texture = 0,
    /// Sound asset
    Sound = 1,
    /// Calling card asset
    CallingCard = 2,
    /// Landmark asset
    Landmark = 3,
    /// Script asset
    Script = 4,
    /// Clothing asset
    Clothing = 5,
    /// Object asset
    Object = 6,
    /// Notecard asset
    Notecard = 7,
    /// Folder asset
    Folder = 8,
    /// Root folder asset
    RootFolder = 9,
    /// LSL text asset
    LSLText = 10,
    /// LSL bytecode asset
    LSLBytecode = 11,
    /// Texture TGA asset
    TextureTGA = 12,
    /// Body part asset
    Bodypart = 13,
    /// Wearable inventory type
    Wearable = 18,
    /// Gesture asset
    Gesture = 21,
    /// Animation asset
    Animation = 20,
    /// Link asset (inventory link to another item)
    Link = 24,
    /// Mesh asset
    Mesh = 49,
}

impl InventoryAssetType {
    /// Get the asset type as a string
    pub fn as_string(&self) -> String {
        (*self as u8).to_string()
    }
}

/// Inventory permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryPermissions {
    /// Next owner permissions
    pub next_perms: u32,
    /// Current owner permissions
    pub owner_perms: u32,
    /// Group permissions
    pub group_perms: u32,
    /// Everyone permissions
    pub everyone_perms: u32,
    /// Base permissions
    pub base_perms: u32,
}

impl Default for InventoryPermissions {
    fn default() -> Self {
        const FULL_PERMS: u32 = 0x7FFFFFFF;
        Self {
            next_perms: FULL_PERMS,
            owner_perms: FULL_PERMS,
            group_perms: 0,
            everyone_perms: 0,
            base_perms: FULL_PERMS,
        }
    }
}

/// Inventory manager for handling user inventory
#[derive(Debug)]
pub struct InventoryManager {
    /// Cache of user inventory structures
    user_inventories: HashMap<Uuid, UserInventory>,
}

impl InventoryManager {
    /// Create a new inventory manager
    pub fn new() -> Self {
        info!("Initializing inventory manager");
        Self {
            user_inventories: HashMap::new(),
        }
    }
    
    /// Get or create user inventory structure
    pub fn get_user_inventory(&mut self, user_id: Uuid) -> &UserInventory {
        debug!("🔍 INVENTORY TRACE: get_user_inventory starting for user: {}", user_id);
        let result = self.user_inventories.entry(user_id)
            .or_insert_with(|| {
                debug!("🔍 INVENTORY TRACE: Creating default inventory structure for user: {}", user_id);
                debug!("🔍 INVENTORY TRACE: About to call UserInventory::create_default");
                let inventory = UserInventory::create_default(user_id);
                debug!("🔍 INVENTORY TRACE: UserInventory::create_default completed");
                inventory
            });
        debug!("🔍 INVENTORY TRACE: get_user_inventory completed successfully");
        result
    }
    
    /// Get user inventory for login response
    pub fn get_login_inventory(&mut self, user_id: Uuid) -> LoginInventoryResponse {
        debug!("🔍 INVENTORY TRACE: InventoryManager::get_login_inventory starting");
        debug!("🔍 INVENTORY TRACE: About to call get_user_inventory");
        let inventory = self.get_user_inventory(user_id);
        debug!("🔍 INVENTORY TRACE: get_user_inventory completed, calling to_login_response");
        let result = inventory.to_login_response();
        debug!("🔍 INVENTORY TRACE: to_login_response completed successfully");
        result
    }
    
    /// Clear cached inventory for user
    pub fn clear_user_cache(&mut self, user_id: Uuid) {
        self.user_inventories.remove(&user_id);
        debug!("Cleared inventory cache for user: {}", user_id);
    }
    
    /// Get the number of cached inventories
    pub fn cache_size(&self) -> usize {
        self.user_inventories.len()
    }
}

impl Default for InventoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete user inventory structure
#[derive(Debug, Clone)]
pub struct UserInventory {
    /// User ID
    pub user_id: Uuid,
    /// Root folder
    pub root_folder: InventoryFolder,
    /// All folders by ID
    pub folders: HashMap<Uuid, InventoryFolder>,
    /// All items by ID  
    pub items: HashMap<Uuid, InventoryItem>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl UserInventory {
    /// Create default inventory structure for a user
    pub fn create_default(user_id: Uuid) -> Self {
        Self::create_with_db_folders(user_id, None)
    }

    /// Phase 85.8: Create inventory structure using database folder UUIDs when available
    /// If db_folders is Some, uses real database UUIDs for matching folder types
    /// If db_folders is None or folder type not found, generates new UUIDs
    pub fn create_with_db_folders(
        user_id: Uuid,
        db_folders: Option<&std::collections::HashMap<i64, DbFolderInfo>>,
    ) -> Self {
        let mut folders = HashMap::new();
        let mut items = HashMap::new();

        // Create root folder - use database UUID if available (type 8)
        let root_folder = if let Some(ref db_map) = db_folders {
            if let Some(db_info) = db_map.get(&8) {
                debug!("📦 Phase 85.8: Using database root folder UUID: {}", db_info.folder_id);
                InventoryFolder {
                    id: db_info.folder_id,
                    parent_id: None,
                    owner_id: user_id,
                    name: db_info.name.clone(),
                    folder_type: InventoryFolderType::Root,
                    version: db_info.version as u32,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            } else {
                InventoryFolder::new_root(user_id)
            }
        } else {
            InventoryFolder::new_root(user_id)
        };
        let root_id = root_folder.id;
        folders.insert(root_id, root_folder.clone());

        let mut bodypart_folder_id = Uuid::nil();
        let mut clothing_folder_id = Uuid::nil();
        let mut cof_folder_id = Uuid::nil();

        // Create default system folders (all required by viewer)
        // Phase 85.8: Use database UUIDs when available, generate otherwise
        for folder_type in [
            InventoryFolderType::Texture,
            InventoryFolderType::Sound,
            InventoryFolderType::CallingCard,
            InventoryFolderType::Landmark,
            InventoryFolderType::Clothing,
            InventoryFolderType::Object,
            InventoryFolderType::Notecard,
            InventoryFolderType::LSLText,
            InventoryFolderType::BodyPart,
            InventoryFolderType::Trash,
            InventoryFolderType::Snapshot,
            InventoryFolderType::LostAndFound,
            InventoryFolderType::Animation,
            InventoryFolderType::Gesture,
            InventoryFolderType::Favorites,
            InventoryFolderType::CurrentOutfit,
            InventoryFolderType::MyOutfits,
            InventoryFolderType::Inbox,
            InventoryFolderType::Settings,
            InventoryFolderType::Material,
        ] {
            let folder_type_i64 = folder_type as i64;
            let folder = if let Some(ref db_map) = db_folders {
                if let Some(db_info) = db_map.get(&folder_type_i64) {
                    debug!("📦 Phase 85.8: Using database UUID for {} folder: {}",
                           folder_type.default_name(), db_info.folder_id);
                    InventoryFolder {
                        id: db_info.folder_id,
                        parent_id: Some(root_id),
                        owner_id: user_id,
                        name: db_info.name.clone(),
                        folder_type,
                        version: db_info.version as u32,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }
                } else {
                    debug!("📦 Phase 85.8: No database UUID for {} folder, generating new",
                           folder_type.default_name());
                    InventoryFolder::new_system_folder(user_id, root_id, folder_type)
                }
            } else {
                InventoryFolder::new_system_folder(user_id, root_id, folder_type)
            };

            if folder_type == InventoryFolderType::BodyPart {
                bodypart_folder_id = folder.id;
            } else if folder_type == InventoryFolderType::Clothing {
                clothing_folder_id = folder.id;
            } else if folder_type == InventoryFolderType::CurrentOutfit {
                cof_folder_id = folder.id;
            }
            folders.insert(folder.id, folder);
        }

        // Add default Ruth wearable items matching AgentWearablesUpdate UUIDs
        let ruth_wearables: [(Uuid, Uuid, &str, InventoryAssetType, u32, Uuid); 6] = [
            // (item_id, asset_id, name, asset_type, wearable_type_flag, folder_id)
            (
                Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfaba9").unwrap(),
                Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfab73").unwrap(),
                "Default Shape", InventoryAssetType::Bodypart, 0, bodypart_folder_id,
            ),
            (
                Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bfabc9").unwrap(),
                Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bbabbb").unwrap(),
                "Default Skin", InventoryAssetType::Bodypart, 1, bodypart_folder_id,
            ),
            (
                Uuid::parse_str("d342e6c1-b9d2-11dc-95ff-0800200c9a66").unwrap(),
                Uuid::parse_str("d342e6c0-b9d2-11dc-95ff-0800200c9a66").unwrap(),
                "Default Hair", InventoryAssetType::Bodypart, 2, bodypart_folder_id,
            ),
            (
                Uuid::parse_str("cdc31054-eed8-4021-994f-4e0c6e861b50").unwrap(),
                Uuid::parse_str("4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7").unwrap(),
                "Default Eyes", InventoryAssetType::Bodypart, 3, bodypart_folder_id,
            ),
            (
                Uuid::parse_str("77c41e39-38f9-f75a-0000-585989bf0000").unwrap(),
                Uuid::parse_str("00000000-38f9-1111-024e-222222111110").unwrap(),
                "Default Shirt", InventoryAssetType::Clothing, 4, clothing_folder_id,
            ),
            (
                Uuid::parse_str("77c41e39-38f9-f75a-0000-5859892f1111").unwrap(),
                Uuid::parse_str("00000000-38f9-1111-024e-222222111120").unwrap(),
                "Default Pants", InventoryAssetType::Clothing, 5, clothing_folder_id,
            ),
        ];

        for (item_id, asset_id, name, asset_type, wearable_flag, folder_id) in ruth_wearables {
            let item = InventoryItem::new_with_id(
                item_id,
                asset_id,
                folder_id,
                user_id,
                user_id,
                name.to_string(),
                format!("{} wearable", name),
                asset_type,
                wearable_flag,
            );
            items.insert(item.id, item);
        }

        // Phase 95.5: Create COF (Current Outfit Folder) link items
        // These link items point to the wearable ITEMS above, telling the viewer what to wear.
        // Without these, first-login avatars appear as clouds.
        // Matches OpenSim-master CreateDefaultAppearanceEntries() / CreateCurrentOutfitLink()
        if !cof_folder_id.is_nil() {
            const COPY_PERMS: u32 = 32768; // PermissionMask.Copy
            let cof_links: [(Uuid, &str, u32); 6] = [
                // (wearable_item_id, name, wearable_type_flag)
                (Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfaba9").unwrap(), "Default Shape", 0),
                (Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bfabc9").unwrap(), "Default Skin", 1),
                (Uuid::parse_str("d342e6c1-b9d2-11dc-95ff-0800200c9a66").unwrap(), "Default Hair", 2),
                (Uuid::parse_str("cdc31054-eed8-4021-994f-4e0c6e861b50").unwrap(), "Default Eyes", 3),
                (Uuid::parse_str("77c41e39-38f9-f75a-0000-585989bf0000").unwrap(), "Default Shirt", 4),
                (Uuid::parse_str("77c41e39-38f9-f75a-0000-5859892f1111").unwrap(), "Default Pants", 5),
            ];

            for (wearable_item_id, name, wearable_type) in cof_links {
                let mut link_item = InventoryItem::new_with_id(
                    Uuid::new_v4(),
                    wearable_item_id, // AssetID = wearable ITEM UUID (not asset UUID)
                    cof_folder_id,
                    user_id,
                    user_id,
                    name.to_string(),
                    format!("@{}", name),
                    InventoryAssetType::Link, // assetType = 24
                    wearable_type,            // flags = WearableType
                );
                link_item.inventory_type = InventoryAssetType::Wearable; // invType = 18
                link_item.permissions = InventoryPermissions {
                    base_perms: COPY_PERMS,
                    owner_perms: COPY_PERMS,
                    group_perms: COPY_PERMS,
                    everyone_perms: COPY_PERMS,
                    next_perms: COPY_PERMS,
                };
                items.insert(link_item.id, link_item);
            }
            debug!("Phase 95.5: Created 6 COF link items for default outfit");
        }

        Self {
            user_id,
            root_folder,
            folders,
            items,
            created_at: Utc::now(),
        }
    }
    
    /// Convert to login response format
    pub fn to_login_response(&self) -> LoginInventoryResponse {
        // Get root folder
        let inventory_root = vec![self.root_folder.to_login_folder()];

        // Get skeleton folders - MUST include ALL folders including root!
        // The viewer's buildParentChildMap() looks up parents in this list.
        // System folders have root as parent, so root MUST be in skeleton.
        // Sort order: system folders first, regular folders, then # folders
        let mut inventory_skeleton: Vec<LoginInventoryFolder> = self.folders
            .values()
            .map(|f| f.to_login_folder())
            .collect();

        inventory_skeleton.sort_by(|a, b| {
            let a_type: i32 = a.type_default.parse().unwrap_or(-1);
            let b_type: i32 = b.type_default.parse().unwrap_or(-1);
            let a_system = a_type >= 0;
            let b_system = b_type >= 0;
            let a_hash = a.name.starts_with('#');
            let b_hash = b.name.starts_with('#');
            let a_group = if a_system { 0 } else if a_hash { 2 } else { 1 };
            let b_group = if b_system { 0 } else if b_hash { 2 } else { 1 };
            a_group.cmp(&b_group).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        // Get library inventory from the global library asset manager
        let library_response = LoginInventoryService::get_library_inventory();

        LoginInventoryResponse {
            inventory_root,
            inventory_skeleton,
            inventory_lib_root: library_response.inventory_lib_root,
            inventory_skel_lib: library_response.inventory_skel_lib,
            inventory_lib_owner: library_response.inventory_lib_owner,
        }
    }
    
    /// Add a folder to the inventory
    pub fn add_folder(&mut self, folder: InventoryFolder) {
        self.folders.insert(folder.id, folder);
    }
    
    /// Add an item to the inventory
    pub fn add_item(&mut self, item: InventoryItem) {
        self.items.insert(item.id, item);
    }
    
    /// Get folder by ID
    pub fn get_folder(&self, folder_id: Uuid) -> Option<&InventoryFolder> {
        self.folders.get(&folder_id)
    }
    
    /// Get item by ID
    pub fn get_item(&self, item_id: Uuid) -> Option<&InventoryItem> {
        self.items.get(&item_id)
    }
    
    /// Get all folders in a parent folder
    pub fn get_child_folders(&self, parent_id: Uuid) -> Vec<&InventoryFolder> {
        self.folders
            .values()
            .filter(|f| f.parent_id == Some(parent_id))
            .collect()
    }
    
    /// Get all items in a folder
    pub fn get_folder_items(&self, folder_id: Uuid) -> Vec<&InventoryItem> {
        self.items
            .values()
            .filter(|i| i.folder_id == folder_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_folder_type_string_conversion() {
        assert_eq!(InventoryFolderType::Root.as_string(), "8");
        assert_eq!(InventoryFolderType::Texture.as_string(), "0");
        assert_eq!(InventoryFolderType::Object.as_string(), "6");
    }
    
    #[test]
    fn test_default_folder_names() {
        assert_eq!(InventoryFolderType::Root.default_name(), "My Inventory");
        assert_eq!(InventoryFolderType::Texture.default_name(), "Textures");
        assert_eq!(InventoryFolderType::Object.default_name(), "Objects");
    }
    
    #[test]
    fn test_inventory_manager_creation() {
        let manager = InventoryManager::new();
        assert_eq!(manager.cache_size(), 0);
    }
    
    #[test]
    fn test_user_inventory_creation() {
        let user_id = Uuid::new_v4();
        let inventory = UserInventory::create_default(user_id);
        
        assert_eq!(inventory.user_id, user_id);
        assert!(inventory.folders.len() > 1); // Root + system folders
        assert_eq!(inventory.items.len(), 12); // 6 Ruth wearables + 6 COF links
        
        // Check root folder exists
        assert_eq!(inventory.root_folder.folder_type, InventoryFolderType::Root);
        assert_eq!(inventory.root_folder.owner_id, user_id);
    }
    
    #[test]
    fn test_login_inventory_response() {
        let user_id = Uuid::new_v4();
        let inventory = UserInventory::create_default(user_id);
        let response = inventory.to_login_response();
        
        assert_eq!(response.inventory_root.len(), 1);
        assert!(response.inventory_skeleton.len() > 0);
        assert_eq!(response.inventory_lib_root.len(), 0);
    }
    
    #[test]
    fn test_inventory_manager_caching() {
        let mut manager = InventoryManager::new();
        let user_id = Uuid::new_v4();
        
        // First access creates inventory
        let _inventory1 = manager.get_user_inventory(user_id);
        assert_eq!(manager.cache_size(), 1);
        
        // Second access uses cache
        let _inventory2 = manager.get_user_inventory(user_id);
        assert_eq!(manager.cache_size(), 1);
        
        // Clear cache
        manager.clear_user_cache(user_id);
        assert_eq!(manager.cache_size(), 0);
    }
}