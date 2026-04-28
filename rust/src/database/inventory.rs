//! Inventory database operations and management
//! Uses legacy OpenSim schema for compatibility (inventoryfolders, inventoryitems)

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::multi_backend::DatabaseConnection;

/// Inventory folder data - maps to legacy OpenSim `inventoryfolders` table
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct InventoryFolder {
    #[sqlx(rename = "folderid")]
    pub id: Uuid,
    #[sqlx(rename = "agentid")]
    pub user_id: Uuid,
    #[sqlx(rename = "parentfolderid")]
    pub parent_id: Option<Uuid>,
    #[sqlx(rename = "foldername")]
    pub folder_name: String,
    #[sqlx(rename = "type")]
    pub folder_type: i32,
    pub version: i32,
}

/// Inventory item data - maps to legacy OpenSim `inventoryitems` table
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct InventoryItem {
    #[sqlx(rename = "inventoryid")]
    pub id: Uuid,
    #[sqlx(rename = "avatarid")]
    pub user_id: Uuid,
    #[sqlx(rename = "parentfolderid")]
    pub folder_id: Uuid,
    #[sqlx(rename = "assetid")]
    pub asset_id: Uuid,
    #[sqlx(rename = "assettype")]
    pub asset_type: i32,
    #[sqlx(rename = "invtype")]
    pub inventory_type: i32,
    #[sqlx(rename = "inventoryname")]
    pub item_name: String,
    #[sqlx(rename = "inventorydescription")]
    pub description: String,
    #[sqlx(rename = "creationdate")]
    pub creation_date: i32,
    #[sqlx(rename = "creatorid")]
    pub creator_id: String,
    #[sqlx(rename = "groupid")]
    pub group_id: Option<Uuid>,
    #[sqlx(rename = "groupowned")]
    pub group_owned: i32,
    #[sqlx(rename = "saleprice")]
    pub sale_price: i32,
    #[sqlx(rename = "saletype")]
    pub sale_type: i32,
    pub flags: i32,
    #[sqlx(rename = "inventorybasepermissions")]
    pub base_permissions: i32,
    #[sqlx(rename = "inventorycurrentpermissions")]
    pub current_permissions: i32,
    #[sqlx(rename = "inventoryeveryonepermissions")]
    pub everyone_permissions: i32,
    #[sqlx(rename = "inventorygrouppermissions")]
    pub group_permissions: i32,
    #[sqlx(rename = "inventorynextpermissions")]
    pub next_permissions: i32,
}

/// Asset data - maps to legacy OpenSim `assets` table
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AssetData {
    pub id: Uuid,
    #[sqlx(rename = "name")]
    pub asset_name: String,
    pub description: String,
    #[sqlx(rename = "assettype")]
    pub asset_type: i32,
    pub local: bool,
    pub temporary: bool,
    pub data: Option<Vec<u8>>,
    #[sqlx(rename = "create_time")]
    pub create_time: i32,
    #[sqlx(rename = "access_time")]
    pub access_time: i32,
    #[sqlx(rename = "asset_flags")]
    pub flags: i32,
    #[sqlx(rename = "creatorid")]
    pub creator_id: Option<String>,
}

/// Wearable item data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WearableItem {
    pub item_id: Uuid,
    pub wearable_type: i32,
    pub permissions: i32,
    pub for_sale: i32,
    pub sale_price: i32,
    pub wearable_data: Option<String>,
}

/// Create folder request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub folder_name: String,
    pub folder_type: i32,
}

/// Create item request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateItemRequest {
    pub user_id: Uuid,
    pub folder_id: Uuid,
    pub asset_id: Uuid,
    pub asset_type: i32,
    pub inventory_type: i32,
    pub item_name: String,
    pub description: String,
    pub creator_id: String,
    pub base_permissions: i32,
    pub current_permissions: i32,
    pub everyone_permissions: i32,
    pub group_permissions: i32,
    pub next_permissions: i32,
}

/// Create asset request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssetRequest {
    pub asset_name: String,
    pub description: String,
    pub asset_type: i32,
    pub data: Vec<u8>,
    pub creator_id: Option<String>,
    pub local: bool,
    pub temporary: bool,
}

/// Inventory database operations
#[derive(Debug)]
pub struct InventoryDatabase {
    connection: Option<Arc<DatabaseConnection>>,
}

impl InventoryDatabase {
    /// Create a new inventory database
    pub async fn new(connection: Arc<DatabaseConnection>) -> Result<Self> {
        info!("Initializing inventory database");
        Ok(Self {
            connection: Some(connection),
        })
    }

    /// Create a stub inventory database for SQLite compatibility
    pub fn new_stub() -> Self {
        info!("Creating stub inventory database for SQLite compatibility");
        Self { connection: None }
    }

    /// Get database connection pool (handles stub mode gracefully)
    fn pool(&self) -> Result<&sqlx::PgPool> {
        match &self.connection {
            Some(conn) => match conn.as_ref() {
                super::multi_backend::DatabaseConnection::PostgreSQL(pool) => Ok(pool),
                _ => Err(anyhow!("Database is not PostgreSQL")),
            },
            None => Err(anyhow!("Database operation not available in stub mode")),
        }
    }

    // FOLDER OPERATIONS

    /// Create a new inventory folder
    pub async fn create_folder(&self, request: CreateFolderRequest) -> Result<InventoryFolder> {
        info!(
            "Creating inventory folder: {} for user {}",
            request.folder_name, request.user_id
        );

        let folder_id = Uuid::new_v4();

        let folder = sqlx::query_as::<_, InventoryFolder>(
            r#"
            INSERT INTO inventoryfolders (
                folderid, agentid, parentfolderid, foldername, type, version
            )
            VALUES ($1, $2, $3, $4, $5, 1)
            RETURNING folderid, agentid, parentfolderid, foldername, type, version
            "#,
        )
        .bind(folder_id)
        .bind(request.user_id)
        .bind(request.parent_id)
        .bind(&request.folder_name)
        .bind(request.folder_type)
        .fetch_one(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to create inventory folder: {}", e))?;

        debug!(
            "Created inventory folder: {} ({})",
            folder.folder_name, folder.id
        );
        Ok(folder)
    }

    /// Get folder by ID
    pub async fn get_folder_by_id(&self, folder_id: Uuid) -> Result<Option<InventoryFolder>> {
        debug!("Getting inventory folder by ID: {}", folder_id);

        let folder = sqlx::query_as::<_, InventoryFolder>(
            "SELECT folderid, agentid, parentfolderid, foldername, type, version FROM inventoryfolders WHERE folderid = $1"
        )
        .bind(folder_id)
        .fetch_optional(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get inventory folder: {}", e))?;

        Ok(folder)
    }

    /// Get user's root folders
    pub async fn get_user_root_folders(&self, user_id: Uuid) -> Result<Vec<InventoryFolder>> {
        debug!("Getting root folders for user: {}", user_id);

        let folders = sqlx::query_as::<_, InventoryFolder>(
            "SELECT folderid, agentid, parentfolderid, foldername, type, version FROM inventoryfolders WHERE agentid = $1 AND parentfolderid IS NULL ORDER BY foldername"
        )
        .bind(user_id)
        .fetch_all(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get user root folders: {}", e))?;

        debug!("Found {} root folders for user: {}", folders.len(), user_id);
        Ok(folders)
    }

    /// Get all folders for a user
    pub async fn get_user_folders(&self, user_id: Uuid) -> Result<Vec<InventoryFolder>> {
        debug!("Getting all folders for user: {}", user_id);

        let folders = sqlx::query_as::<_, InventoryFolder>(
            "SELECT folderid, agentid, parentfolderid, foldername, type, version FROM inventoryfolders WHERE agentid = $1 ORDER BY foldername"
        )
        .bind(user_id)
        .fetch_all(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get user folders: {}", e))?;

        debug!("Found {} folders for user: {}", folders.len(), user_id);
        Ok(folders)
    }

    /// Get folder by user and type
    pub async fn get_folder_by_type(
        &self,
        user_id: Uuid,
        folder_type: i32,
    ) -> Result<Option<InventoryFolder>> {
        debug!(
            "Getting folder of type {} for user: {}",
            folder_type, user_id
        );

        let folder = sqlx::query_as::<_, InventoryFolder>(
            "SELECT folderid, agentid, parentfolderid, foldername, type, version FROM inventoryfolders WHERE agentid = $1 AND type = $2 LIMIT 1"
        )
        .bind(user_id)
        .bind(folder_type)
        .fetch_optional(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get folder by type: {}", e))?;

        Ok(folder)
    }

    /// Get folder contents (subfolders)
    pub async fn get_folder_contents(&self, folder_id: Uuid) -> Result<Vec<InventoryFolder>> {
        debug!("Getting contents of folder: {}", folder_id);

        let folders = sqlx::query_as::<_, InventoryFolder>(
            "SELECT folderid, agentid, parentfolderid, foldername, type, version FROM inventoryfolders WHERE parentfolderid = $1 ORDER BY foldername"
        )
        .bind(folder_id)
        .fetch_all(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get folder contents: {}", e))?;

        debug!(
            "Found {} subfolders in folder: {}",
            folders.len(),
            folder_id
        );
        Ok(folders)
    }

    /// Update folder
    pub async fn update_folder(
        &self,
        folder_id: Uuid,
        folder_name: &str,
        folder_type: i32,
    ) -> Result<InventoryFolder> {
        debug!("Updating inventory folder: {}", folder_id);

        let folder = sqlx::query_as::<_, InventoryFolder>(
            r#"
            UPDATE inventoryfolders
            SET foldername = $2, type = $3, version = version + 1
            WHERE folderid = $1
            RETURNING folderid, agentid, parentfolderid, foldername, type, version
            "#,
        )
        .bind(folder_id)
        .bind(folder_name)
        .bind(folder_type)
        .fetch_one(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to update inventory folder: {}", e))?;

        debug!(
            "Updated inventory folder: {} ({})",
            folder.folder_name, folder.id
        );
        Ok(folder)
    }

    /// Delete folder (and all contents)
    pub async fn delete_folder(&self, folder_id: Uuid) -> Result<bool> {
        warn!("Deleting inventory folder: {}", folder_id);

        let connection = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow!("Database not available in stub mode"))?;
        let mut tx = connection.begin_transaction().await?;

        let pg_tx = tx.as_postgres_tx()?;

        let _ = sqlx::query("DELETE FROM inventoryitems WHERE parentfolderid = $1")
            .bind(folder_id)
            .execute(&mut **pg_tx)
            .await;

        let subfolders =
            sqlx::query("SELECT folderid FROM inventoryfolders WHERE parentfolderid = $1")
                .bind(folder_id)
                .fetch_all(&mut **pg_tx)
                .await
                .map_err(|e| anyhow!("Failed to get subfolders: {}", e))?;

        for row in subfolders {
            let subfolder_id: Uuid = row.try_get("folderid")?;
            let _ = sqlx::query("DELETE FROM inventoryitems WHERE parentfolderid = $1")
                .bind(subfolder_id)
                .execute(&mut **pg_tx)
                .await;
            let _ = sqlx::query("DELETE FROM inventoryfolders WHERE folderid = $1")
                .bind(subfolder_id)
                .execute(&mut **pg_tx)
                .await;
        }

        let result = sqlx::query("DELETE FROM inventoryfolders WHERE folderid = $1")
            .bind(folder_id)
            .execute(&mut **pg_tx)
            .await
            .map_err(|e| anyhow!("Failed to delete inventory folder: {}", e))?;

        tx.commit()
            .await
            .map_err(|e| anyhow!("Failed to commit folder deletion: {}", e))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted inventory folder: {}", folder_id);
        }

        Ok(deleted)
    }

    // ITEM OPERATIONS

    /// Create a new inventory item
    pub async fn create_item(&self, request: CreateItemRequest) -> Result<InventoryItem> {
        info!(
            "Creating inventory item: {} for user {}",
            request.item_name, request.user_id
        );

        let item_id = Uuid::new_v4();
        let creation_date = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i32;

        let item = sqlx::query_as::<_, InventoryItem>(
            r#"
            INSERT INTO inventoryitems (
                inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                inventoryname, inventorydescription, creationdate, creatorid,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
                groupid, groupowned, saleprice, saletype, flags
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NULL, 0, 0, 0, 0)
            RETURNING inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                inventoryname, inventorydescription, creationdate, creatorid,
                groupid, groupowned, saleprice, saletype, flags,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions
            "#
        )
        .bind(item_id)
        .bind(request.user_id)
        .bind(request.folder_id)
        .bind(request.asset_id)
        .bind(request.asset_type)
        .bind(request.inventory_type)
        .bind(&request.item_name)
        .bind(&request.description)
        .bind(creation_date)
        .bind(&request.creator_id)
        .bind(request.base_permissions)
        .bind(request.current_permissions)
        .bind(request.everyone_permissions)
        .bind(request.group_permissions)
        .bind(request.next_permissions)
        .fetch_one(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to create inventory item: {}", e))?;

        debug!("Created inventory item: {} ({})", item.item_name, item.id);
        Ok(item)
    }

    /// Get item by ID
    pub async fn get_item_by_id(&self, item_id: Uuid) -> Result<Option<InventoryItem>> {
        debug!("Getting inventory item by ID: {}", item_id);

        let item = sqlx::query_as::<_, InventoryItem>(
            r#"SELECT inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                inventoryname, inventorydescription, creationdate, creatorid,
                groupid, groupowned, saleprice, saletype, flags,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions
            FROM inventoryitems WHERE inventoryid = $1"#,
        )
        .bind(item_id)
        .fetch_optional(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get inventory item: {}", e))?;

        Ok(item)
    }

    /// Get items in folder
    pub async fn get_folder_items(&self, folder_id: Uuid) -> Result<Vec<InventoryItem>> {
        debug!("Getting items in folder: {}", folder_id);

        let items = sqlx::query_as::<_, InventoryItem>(
            r#"SELECT inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                inventoryname, inventorydescription, creationdate, creatorid,
                groupid, groupowned, saleprice, saletype, flags,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions
            FROM inventoryitems WHERE parentfolderid = $1 ORDER BY inventoryname"#,
        )
        .bind(folder_id)
        .fetch_all(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get folder items: {}", e))?;

        debug!("Found {} items in folder: {}", items.len(), folder_id);
        Ok(items)
    }

    /// Get user's items by type
    pub async fn get_user_items_by_type(
        &self,
        user_id: Uuid,
        inventory_type: i32,
    ) -> Result<Vec<InventoryItem>> {
        debug!(
            "Getting items of type {} for user: {}",
            inventory_type, user_id
        );

        let items = sqlx::query_as::<_, InventoryItem>(
            r#"SELECT inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                inventoryname, inventorydescription, creationdate, creatorid,
                groupid, groupowned, saleprice, saletype, flags,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions
            FROM inventoryitems WHERE avatarid = $1 AND invtype = $2 ORDER BY inventoryname"#,
        )
        .bind(user_id)
        .bind(inventory_type)
        .fetch_all(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get user items by type: {}", e))?;

        debug!(
            "Found {} items of type {} for user: {}",
            items.len(),
            inventory_type,
            user_id
        );
        Ok(items)
    }

    /// Get all user's items
    pub async fn get_user_items(&self, user_id: Uuid) -> Result<Vec<InventoryItem>> {
        debug!("Getting all items for user: {}", user_id);

        let items = sqlx::query_as::<_, InventoryItem>(
            r#"SELECT inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                inventoryname, inventorydescription, creationdate, creatorid,
                groupid, groupowned, saleprice, saletype, flags,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions
            FROM inventoryitems WHERE avatarid = $1 ORDER BY inventoryname"#,
        )
        .bind(user_id)
        .fetch_all(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get user items: {}", e))?;

        debug!("Found {} items for user: {}", items.len(), user_id);
        Ok(items)
    }

    /// Update inventory item
    pub async fn update_item(
        &self,
        item_id: Uuid,
        item_name: &str,
        description: &str,
    ) -> Result<InventoryItem> {
        debug!("Updating inventory item: {}", item_id);

        let item = sqlx::query_as::<_, InventoryItem>(
            r#"
            UPDATE inventoryitems
            SET inventoryname = $2, inventorydescription = $3
            WHERE inventoryid = $1
            RETURNING inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                inventoryname, inventorydescription, creationdate, creatorid,
                groupid, groupowned, saleprice, saletype, flags,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions
            "#,
        )
        .bind(item_id)
        .bind(item_name)
        .bind(description)
        .fetch_one(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to update inventory item: {}", e))?;

        debug!("Updated inventory item: {} ({})", item.item_name, item.id);
        Ok(item)
    }

    /// Move item to different folder
    pub async fn move_item(&self, item_id: Uuid, new_folder_id: Uuid) -> Result<InventoryItem> {
        debug!(
            "Moving inventory item {} to folder {}",
            item_id, new_folder_id
        );

        let item = sqlx::query_as::<_, InventoryItem>(
            r#"
            UPDATE inventoryitems
            SET parentfolderid = $2
            WHERE inventoryid = $1
            RETURNING inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                inventoryname, inventorydescription, creationdate, creatorid,
                groupid, groupowned, saleprice, saletype, flags,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions
            "#,
        )
        .bind(item_id)
        .bind(new_folder_id)
        .fetch_one(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to move inventory item: {}", e))?;

        debug!(
            "Moved inventory item: {} to folder: {}",
            item.item_name, new_folder_id
        );
        Ok(item)
    }

    /// Delete inventory item
    pub async fn delete_item(&self, item_id: Uuid) -> Result<bool> {
        debug!("Deleting inventory item: {}", item_id);

        let result = sqlx::query("DELETE FROM inventoryitems WHERE inventoryid = $1")
            .bind(item_id)
            .execute(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to delete inventory item: {}", e))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted inventory item: {}", item_id);
        }

        Ok(deleted)
    }

    // ASSET OPERATIONS

    /// Create a new asset
    pub async fn create_asset(&self, request: CreateAssetRequest) -> Result<AssetData> {
        info!(
            "Creating asset: {} (type: {})",
            request.asset_name, request.asset_type
        );

        let asset_id = Uuid::new_v4();
        let create_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i32;

        let asset = sqlx::query_as::<_, AssetData>(
            r#"
            INSERT INTO assets (
                id, name, description, assettype, local, temporary,
                data, create_time, access_time, asset_flags, creatorid
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8, 0, $9)
            RETURNING id, name, description, assettype, local, temporary,
                data, create_time, access_time, asset_flags, creatorid
            "#,
        )
        .bind(asset_id)
        .bind(&request.asset_name)
        .bind(&request.description)
        .bind(request.asset_type)
        .bind(request.local)
        .bind(request.temporary)
        .bind(&request.data)
        .bind(create_time)
        .bind(&request.creator_id)
        .fetch_one(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to create asset: {}", e))?;

        debug!("Created asset: {} ({})", asset.asset_name, asset.id);
        Ok(asset)
    }

    /// Get asset by ID
    pub async fn get_asset_by_id(&self, asset_id: Uuid) -> Result<Option<AssetData>> {
        debug!("Getting asset by ID: {}", asset_id);

        let access_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i32;

        let _ = sqlx::query("UPDATE assets SET access_time = $2 WHERE id = $1")
            .bind(asset_id)
            .bind(access_time)
            .execute(self.pool()?)
            .await;

        let asset = sqlx::query_as::<_, AssetData>(
            "SELECT id, name, description, assettype, local, temporary, data, create_time, access_time, asset_flags, creatorid FROM assets WHERE id = $1"
        )
            .bind(asset_id)
            .fetch_optional(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to get asset: {}", e))?;

        Ok(asset)
    }

    /// Get asset metadata (without data payload)
    pub async fn get_asset_metadata(&self, asset_id: Uuid) -> Result<Option<AssetData>> {
        debug!("Getting asset metadata: {}", asset_id);

        let asset = sqlx::query_as::<_, AssetData>(
            "SELECT id, name, description, assettype, local, temporary, NULL as data, create_time, access_time, asset_flags, creatorid FROM assets WHERE id = $1"
        )
        .bind(asset_id)
        .fetch_optional(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get asset metadata: {}", e))?;

        Ok(asset)
    }

    /// Delete asset
    pub async fn delete_asset(&self, asset_id: Uuid) -> Result<bool> {
        warn!("Deleting asset: {}", asset_id);

        let result = sqlx::query("DELETE FROM assets WHERE id = $1")
            .bind(asset_id)
            .execute(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to delete asset: {}", e))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted asset: {}", asset_id);
        }

        Ok(deleted)
    }

    // UTILITY FUNCTIONS

    /// Initialize default inventory structure for a new user
    pub async fn initialize_user_inventory(&self, user_id: Uuid) -> Result<Vec<InventoryFolder>> {
        info!("Initializing inventory for user: {}", user_id);

        let mut folders = Vec::new();

        let folder_types = vec![
            ("My Inventory", 8),
            ("Calling Cards", 2),
            ("Clothing", 5),
            ("Gestures", 21),
            ("Landmarks", 3),
            ("Lost And Found", 16),
            ("Notecards", 7),
            ("Objects", 6),
            ("Photo Album", 15),
            ("Scripts", 10),
            ("Sounds", 1),
            ("Textures", 0),
            ("Trash", 14),
            ("Body Parts", 13),
            ("Animations", 20),
        ];

        for (name, folder_type) in folder_types {
            let request = CreateFolderRequest {
                user_id,
                parent_id: None,
                folder_name: name.to_string(),
                folder_type,
            };

            let folder = self.create_folder(request).await?;
            folders.push(folder);
        }

        info!(
            "Created {} default folders for user: {}",
            folders.len(),
            user_id
        );
        Ok(folders)
    }

    /// Phase 95.5: Create default COF (Current Outfit Folder) link items for a user
    /// These are inventory links (assetType=24, invType=18) pointing to wearable ITEM UUIDs
    /// Matches OpenSim-master CreateDefaultAppearanceEntries() / CreateCurrentOutfitLink()
    pub async fn create_default_cof_links(&self, user_id: Uuid, cof_folder_id: Uuid) -> Result<()> {
        info!("Creating default COF links for user: {}", user_id);

        let creation_date = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i32;
        let creator = "11111111-1111-0000-0000-000100bba000";
        const COPY_PERMS: i32 = 32768;

        // (wearable_item_id, name, wearable_type_flag)
        let cof_links: [(&str, &str, i32); 6] = [
            ("66c41e39-38f9-f75a-024e-585989bfaba9", "Default Shape", 0),
            ("77c41e39-38f9-f75a-024e-585989bfabc9", "Default Skin", 1),
            ("d342e6c1-b9d2-11dc-95ff-0800200c9a66", "Default Hair", 2),
            ("cdc31054-eed8-4021-994f-4e0c6e861b50", "Default Eyes", 3),
            ("77c41e39-38f9-f75a-0000-585989bf0000", "Default Shirt", 4),
            ("77c41e39-38f9-f75a-0000-5859892f1111", "Default Pants", 5),
        ];

        for (wearable_item_id, name, wearable_type) in cof_links {
            let link_id = Uuid::new_v4();
            let item_uuid =
                Uuid::parse_str(wearable_item_id).map_err(|e| anyhow!("Invalid UUID: {}", e))?;

            sqlx::query(
                r#"
                INSERT INTO inventoryitems (
                    inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                    inventoryname, inventorydescription, creationdate, creatorid,
                    inventorybasepermissions, inventorycurrentpermissions,
                    inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
                    groupid, groupowned, saleprice, saletype, flags
                )
                VALUES ($1, $2, $3, $4, 24, 18, $5, $6, $7, $8, $9, $9, $9, $9, $9, NULL, 0, 0, 0, $10)
                ON CONFLICT (inventoryid) DO NOTHING
                "#
            )
            .bind(link_id)
            .bind(user_id)
            .bind(cof_folder_id)
            .bind(item_uuid)       // assetID = wearable ITEM UUID
            .bind(name)
            .bind(format!("@{}", name))
            .bind(creation_date)
            .bind(creator)
            .bind(COPY_PERMS)
            .bind(wearable_type)
            .execute(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to create COF link for {}: {}", name, e))?;
        }

        info!("Created 6 default COF links for user: {}", user_id);
        Ok(())
    }

    /// Get inventory statistics for a user
    pub async fn get_user_inventory_stats(&self, user_id: Uuid) -> Result<InventoryStats> {
        debug!("Getting inventory statistics for user: {}", user_id);

        let folder_count_row =
            sqlx::query("SELECT COUNT(*) as count FROM inventoryfolders WHERE agentid = $1")
                .bind(user_id)
                .fetch_one(self.pool()?)
                .await?;
        let folder_count: i64 = folder_count_row.try_get("count")?;

        let item_count_row =
            sqlx::query("SELECT COUNT(*) as count FROM inventoryitems WHERE avatarid = $1")
                .bind(user_id)
                .fetch_one(self.pool()?)
                .await?;
        let item_count: i64 = item_count_row.try_get("count")?;

        Ok(InventoryStats {
            folder_count: folder_count as u64,
            item_count: item_count as u64,
        })
    }

    /// Recursive folder deletion helper
    fn delete_folder_recursive<'a>(
        &'a self,
        tx: &'a mut sqlx::Transaction<'_, sqlx::Postgres>,
        folder_id: Uuid,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let _ = sqlx::query("DELETE FROM inventoryitems WHERE parentfolderid = $1")
                .bind(folder_id)
                .execute(&mut **tx)
                .await;

            let subfolders =
                sqlx::query("SELECT folderid FROM inventoryfolders WHERE parentfolderid = $1")
                    .bind(folder_id)
                    .fetch_all(&mut **tx)
                    .await?;

            for row in subfolders {
                let subfolder_id: Uuid = row.try_get("folderid")?;
                self.delete_folder_recursive(tx, subfolder_id).await?;
            }

            let _ = sqlx::query("DELETE FROM inventoryfolders WHERE folderid = $1")
                .bind(folder_id)
                .execute(&mut **tx)
                .await;

            Ok(())
        })
    }
}

/// Inventory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryStats {
    pub folder_count: u64,
    pub item_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_folder_request() {
        let request = CreateFolderRequest {
            user_id: Uuid::new_v4(),
            parent_id: None,
            folder_name: "Test Folder".to_string(),
            folder_type: 8,
        };

        assert_eq!(request.folder_name, "Test Folder");
        assert_eq!(request.folder_type, 8);
        assert!(request.parent_id.is_none());
    }

    #[test]
    fn test_create_item_request() {
        let request = CreateItemRequest {
            user_id: Uuid::new_v4(),
            folder_id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            asset_type: 0,
            inventory_type: 0,
            item_name: "Test Item".to_string(),
            description: "A test item".to_string(),
            creator_id: Uuid::new_v4().to_string(),
            base_permissions: 647168,
            current_permissions: 647168,
            everyone_permissions: 0,
            group_permissions: 0,
            next_permissions: 647168,
        };

        assert_eq!(request.item_name, "Test Item");
        assert_eq!(request.asset_type, 0);
        assert_eq!(request.inventory_type, 0);
    }
}
