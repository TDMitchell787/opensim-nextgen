//! Manages user inventory data.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::network::llsd::LLSDValue;

/// Represents a single item in an inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_id: Uuid,
    pub asset_id: Uuid,
    pub name: String,
    pub description: String,
    pub parent_folder_id: Uuid,
    // Add other relevant fields like creation date, permissions, etc.
}

impl From<InventoryItem> for LLSDValue {
    fn from(item: InventoryItem) -> Self {
        let mut map = HashMap::new();
        map.insert("item_id".to_string(), LLSDValue::UUID(item.item_id));
        map.insert("asset_id".to_string(), LLSDValue::UUID(item.asset_id));
        map.insert("name".to_string(), LLSDValue::String(item.name));
        map.insert("description".to_string(), LLSDValue::String(item.description));
        map.insert("parent_folder_id".to_string(), LLSDValue::UUID(item.parent_folder_id));
        LLSDValue::Map(map)
    }
}

/// Represents a folder in an inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryFolder {
    pub folder_id: Uuid,
    pub name: String,
    pub parent_folder_id: Uuid,
    pub children: Vec<Uuid>, // Contains IDs of both items and folders
}

impl From<InventoryFolder> for LLSDValue {
    fn from(folder: InventoryFolder) -> Self {
        let mut map = HashMap::new();
        map.insert("folder_id".to_string(), LLSDValue::UUID(folder.folder_id));
        map.insert("name".to_string(), LLSDValue::String(folder.name));
        map.insert("parent_folder_id".to_string(), LLSDValue::UUID(folder.parent_folder_id));
        let children_array = folder.children.into_iter().map(LLSDValue::UUID).collect();
        map.insert("children".to_string(), LLSDValue::Array(children_array));
        LLSDValue::Map(map)
    }
}

/// Represents the complete inventory for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub root_folder_id: Uuid,
    pub items: HashMap<Uuid, InventoryItem>,
    pub folders: HashMap<Uuid, InventoryFolder>,
}

impl From<Inventory> for LLSDValue {
    fn from(inventory: Inventory) -> Self {
        let items_array = inventory.items.into_values().map(LLSDValue::from).collect();
        let folders_array = inventory.folders.into_values().map(LLSDValue::from).collect();

        let mut map = HashMap::new();
        map.insert("root_folder_id".to_string(), LLSDValue::UUID(inventory.root_folder_id));
        map.insert("items".to_string(), LLSDValue::Array(items_array));
        map.insert("folders".to_string(), LLSDValue::Array(folders_array));

        LLSDValue::Map(map)
    }
}

impl Inventory {
    /// Creates a new, empty inventory for a user.
    pub fn new() -> Self {
        let root_id = Uuid::new_v4();
        let mut folders = HashMap::new();
        folders.insert(root_id, InventoryFolder {
            folder_id: root_id,
            name: "My Inventory".to_string(),
            parent_folder_id: Uuid::nil(),
            children: Vec::new(),
        });

        Self {
            root_folder_id: root_id,
            items: HashMap::new(),
            folders,
        }
    }
}

/// Manages inventories for all users.
pub struct InventoryManager {
    // Key is User ID
    inventories: HashMap<Uuid, Inventory>,
}

impl InventoryManager {
    pub fn new() -> Self {
        Self {
            inventories: HashMap::new(),
        }
    }

    /// Gets or creates an inventory for a user.
    pub fn get_or_create_inventory(&mut self, user_id: &Uuid) -> &mut Inventory {
        self.inventories.entry(*user_id).or_insert_with(Inventory::new)
    }
} 