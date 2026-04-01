//! Inventory item management for Second Life/OpenSim compatibility

use super::*;

/// Inventory item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    /// Unique item ID
    pub id: Uuid,
    /// Asset ID that this item references
    pub asset_id: Uuid,
    /// Folder ID where this item is located
    pub folder_id: Uuid,
    /// Owner user ID
    pub owner_id: Uuid,
    /// Creator user ID
    pub creator_id: Uuid,
    /// Item name
    pub name: String,
    /// Item description
    pub description: String,
    /// Asset type
    pub asset_type: InventoryAssetType,
    /// Inventory type (usually same as asset type)
    pub inventory_type: InventoryAssetType,
    /// Item permissions
    pub permissions: InventoryPermissions,
    /// Flags (for special items)
    pub flags: u32,
    /// Sale type (0 = not for sale, 1 = original, 2 = copy)
    pub sale_type: u8,
    /// Sale price
    pub sale_price: i32,
    /// Group ID (if owned by group)
    pub group_id: Option<Uuid>,
    /// Group permissions
    pub group_owned: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,
}

impl InventoryItem {
    /// Create a new inventory item
    pub fn new(
        asset_id: Uuid,
        folder_id: Uuid,
        owner_id: Uuid,
        creator_id: Uuid,
        name: String,
        description: String,
        asset_type: InventoryAssetType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            name,
            description,
            asset_type,
            inventory_type: asset_type, // Usually the same
            permissions: InventoryPermissions::default(),
            flags: 0,
            sale_type: 0, // Not for sale
            sale_price: 0,
            group_id: None,
            group_owned: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn new_with_id(
        id: Uuid,
        asset_id: Uuid,
        folder_id: Uuid,
        owner_id: Uuid,
        creator_id: Uuid,
        name: String,
        description: String,
        asset_type: InventoryAssetType,
        flags: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            name,
            description,
            asset_type,
            inventory_type: asset_type,
            permissions: InventoryPermissions::default(),
            flags,
            sale_type: 0,
            sale_price: 0,
            group_id: None,
            group_owned: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a texture item
    pub fn new_texture(
        asset_id: Uuid,
        folder_id: Uuid,
        owner_id: Uuid,
        creator_id: Uuid,
        name: String,
    ) -> Self {
        Self::new(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            name,
            "A texture".to_string(),
            InventoryAssetType::Texture,
        )
    }
    
    /// Create an object item
    pub fn new_object(
        asset_id: Uuid,
        folder_id: Uuid,
        owner_id: Uuid,
        creator_id: Uuid,
        name: String,
    ) -> Self {
        Self::new(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            name,
            "An object".to_string(),
            InventoryAssetType::Object,
        )
    }
    
    /// Create a script item
    pub fn new_script(
        asset_id: Uuid,
        folder_id: Uuid,
        owner_id: Uuid,
        creator_id: Uuid,
        name: String,
    ) -> Self {
        Self::new(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            name,
            "A script".to_string(),
            InventoryAssetType::LSLText,
        )
    }
    
    /// Create a notecard item
    pub fn new_notecard(
        asset_id: Uuid,
        folder_id: Uuid,
        owner_id: Uuid,
        creator_id: Uuid,
        name: String,
    ) -> Self {
        Self::new(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            name,
            "A notecard".to_string(),
            InventoryAssetType::Notecard,
        )
    }
    
    /// Convert to login response format (if needed for advanced inventory)
    pub fn to_login_item(&self) -> LoginInventoryItem {
        LoginInventoryItem {
            item_id: self.id.to_string(),
            asset_id: self.asset_id.to_string(),
            folder_id: self.folder_id.to_string(),
            owner_id: self.owner_id.to_string(),
            creator_id: self.creator_id.to_string(),
            name: self.name.clone(),
            description: self.description.clone(),
            asset_type: self.asset_type.as_string(),
            inventory_type: self.inventory_type.as_string(),
            flags: self.flags.to_string(),
            sale_type: self.sale_type.to_string(),
            sale_price: self.sale_price.to_string(),
        }
    }
    
    /// Update item
    pub fn update(&mut self, name: Option<String>, description: Option<String>) {
        if let Some(new_name) = name {
            self.name = new_name;
        }
        if let Some(new_description) = description {
            self.description = new_description;
        }
        self.updated_at = Utc::now();
    }
    
    /// Check if item is for sale
    pub fn is_for_sale(&self) -> bool {
        self.sale_type > 0 && self.sale_price > 0
    }
    
    /// Set item for sale
    pub fn set_for_sale(&mut self, sale_type: u8, price: i32) {
        self.sale_type = sale_type;
        self.sale_price = price;
        self.updated_at = Utc::now();
    }
    
    /// Remove item from sale
    pub fn remove_from_sale(&mut self) {
        self.sale_type = 0;
        self.sale_price = 0;
        self.updated_at = Utc::now();
    }
}

/// Login inventory item format (for advanced inventory responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInventoryItem {
    /// Item ID as string
    pub item_id: String,
    /// Asset ID as string
    pub asset_id: String,
    /// Folder ID as string
    pub folder_id: String,
    /// Owner ID as string
    pub owner_id: String,
    /// Creator ID as string
    pub creator_id: String,
    /// Item name
    pub name: String,
    /// Item description
    pub description: String,
    /// Asset type as string
    pub asset_type: String,
    /// Inventory type as string
    pub inventory_type: String,
    /// Flags as string
    pub flags: String,
    /// Sale type as string
    pub sale_type: String,
    /// Sale price as string
    pub sale_price: String,
}

impl LoginInventoryItem {
    /// Convert to XMLRPC struct format
    pub fn to_xmlrpc_struct(&self) -> String {
        format!(
            r#"<value>
              <struct>
                <member>
                  <name>item_id</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>asset_id</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>folder_id</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>owner_id</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>creator_id</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>name</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>description</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>asset_type</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>inventory_type</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>flags</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>sale_type</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>sale_price</name>
                  <value><string>{}</string></value>
                </member>
              </struct>
            </value>"#,
            self.item_id, self.asset_id, self.folder_id, self.owner_id, self.creator_id,
            self.name, self.description, self.asset_type, self.inventory_type,
            self.flags, self.sale_type, self.sale_price
        )
    }
}

/// Item creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateItemRequest {
    /// Asset ID
    pub asset_id: Uuid,
    /// Folder ID where to place the item
    pub folder_id: Uuid,
    /// Item name
    pub name: String,
    /// Item description
    pub description: String,
    /// Asset type
    pub asset_type: InventoryAssetType,
}

/// Item update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateItemRequest {
    /// New item name (optional)
    pub name: Option<String>,
    /// New item description (optional)
    pub description: Option<String>,
    /// New folder ID (optional)
    pub folder_id: Option<Uuid>,
    /// Sale information (optional)
    pub sale_info: Option<SaleInfo>,
}

/// Sale information for items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleInfo {
    /// Sale type (0 = not for sale, 1 = original, 2 = copy)
    pub sale_type: u8,
    /// Sale price
    pub price: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_item_creation() {
        let asset_id = Uuid::new_v4();
        let folder_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        
        let item = InventoryItem::new(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            "Test Item".to_string(),
            "A test item".to_string(),
            InventoryAssetType::Texture,
        );
        
        assert_eq!(item.asset_id, asset_id);
        assert_eq!(item.folder_id, folder_id);
        assert_eq!(item.owner_id, owner_id);
        assert_eq!(item.creator_id, creator_id);
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.description, "A test item");
        assert_eq!(item.asset_type, InventoryAssetType::Texture);
        assert!(!item.is_for_sale());
    }
    
    #[test]
    fn test_texture_item_creation() {
        let asset_id = Uuid::new_v4();
        let folder_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        
        let item = InventoryItem::new_texture(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            "My Texture".to_string(),
        );
        
        assert_eq!(item.name, "My Texture");
        assert_eq!(item.description, "A texture");
        assert_eq!(item.asset_type, InventoryAssetType::Texture);
    }
    
    #[test]
    fn test_item_sale_operations() {
        let asset_id = Uuid::new_v4();
        let folder_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        
        let mut item = InventoryItem::new_object(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            "My Object".to_string(),
        );
        
        assert!(!item.is_for_sale());
        
        // Set for sale
        item.set_for_sale(2, 100); // Copy for 100
        assert!(item.is_for_sale());
        assert_eq!(item.sale_type, 2);
        assert_eq!(item.sale_price, 100);
        
        // Remove from sale
        item.remove_from_sale();
        assert!(!item.is_for_sale());
        assert_eq!(item.sale_type, 0);
        assert_eq!(item.sale_price, 0);
    }
    
    #[test]
    fn test_item_update() {
        let asset_id = Uuid::new_v4();
        let folder_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        
        let mut item = InventoryItem::new_notecard(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            "Old Name".to_string(),
        );
        
        let initial_updated = item.updated_at;
        
        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        item.update(
            Some("New Name".to_string()),
            Some("New description".to_string()),
        );
        
        assert_eq!(item.name, "New Name");
        assert_eq!(item.description, "New description");
        assert!(item.updated_at > initial_updated);
    }
    
    #[test]
    fn test_login_item_conversion() {
        let asset_id = Uuid::new_v4();
        let folder_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        
        let item = InventoryItem::new_script(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            "My Script".to_string(),
        );
        
        let login_item = item.to_login_item();
        
        assert_eq!(login_item.item_id, item.id.to_string());
        assert_eq!(login_item.asset_id, asset_id.to_string());
        assert_eq!(login_item.name, "My Script");
        assert_eq!(login_item.asset_type, "10"); // LSLText
    }
    
    #[test]
    fn test_xmlrpc_item_struct() {
        let asset_id = Uuid::new_v4();
        let folder_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        
        let item = InventoryItem::new_object(
            asset_id,
            folder_id,
            owner_id,
            creator_id,
            "Test Object".to_string(),
        );
        
        let login_item = item.to_login_item();
        let xmlrpc = login_item.to_xmlrpc_struct();
        
        assert!(xmlrpc.contains(&format!("<string>{}</string>", item.id)));
        assert!(xmlrpc.contains("<string>Test Object</string>"));
        assert!(xmlrpc.contains("<string>6</string>")); // Object type
    }
}