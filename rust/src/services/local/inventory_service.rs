//! Local Inventory Service Implementation
//!
//! Provides direct database access for inventory operations.
//! Used in standalone mode with PostgreSQL backend.
//!
//! Reference: OpenSim/Services/InventoryService/XInventoryService.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{InventoryServiceTrait, InventoryFolder, InventoryItem, InventoryCollection};
use crate::database::multi_backend::DatabaseConnection;

pub struct LocalInventoryService {
    connection: Arc<DatabaseConnection>,
}

impl LocalInventoryService {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        info!("Initializing local inventory service");
        Self { connection }
    }

    fn get_pg_pool(&self) -> Result<&sqlx::PgPool> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => Ok(pool),
            _ => Err(anyhow!("LocalInventoryService requires PostgreSQL connection")),
        }
    }

    fn row_to_folder(&self, row: &sqlx::postgres::PgRow) -> Result<InventoryFolder> {
        use sqlx::Row;
        Ok(InventoryFolder {
            folder_id: row.try_get("folderid")?,
            parent_id: row.try_get("parentfolderid")?,
            owner_id: row.try_get("agentid")?,
            name: row.try_get("foldername")?,
            folder_type: row.try_get("type")?,
            version: row.try_get("version")?,
        })
    }

    fn row_to_item(&self, row: &sqlx::postgres::PgRow) -> Result<InventoryItem> {
        use sqlx::Row;
        Ok(InventoryItem {
            item_id: row.try_get("inventoryid")?,
            asset_id: row.try_get("assetid")?,
            folder_id: row.try_get("parentfolderid")?,
            owner_id: row.try_get("avatarid")?,
            creator_id: row.try_get::<String, _>("creatorid")
                .ok()
                .and_then(|s| Uuid::parse_str(&s).ok())
                .unwrap_or_default(),
            creator_data: {
                let full: String = row.try_get::<String, _>("creatorid").unwrap_or_default();
                if let Some(pos) = full.find(';') { full[pos+1..].to_string() } else { String::new() }
            },
            name: row.try_get("inventoryname")?,
            description: row.try_get("inventorydescription").unwrap_or_default(),
            asset_type: row.try_get("assettype")?,
            inv_type: row.try_get("invtype")?,
            flags: row.try_get::<i32, _>("flags").unwrap_or(0) as u32,
            creation_date: row.try_get::<i32, _>("creationdate").unwrap_or(0) as i64,
            base_permissions: row.try_get::<Option<i32>, _>("inventorybasepermissions").ok().flatten().unwrap_or(0x7FFFFFFF) as u32,
            current_permissions: row.try_get::<Option<i32>, _>("inventorycurrentpermissions").ok().flatten().unwrap_or(0x7FFFFFFF) as u32,
            everyone_permissions: row.try_get::<Option<i32>, _>("inventoryeveryonepermissions").ok().flatten().unwrap_or(0) as u32,
            next_permissions: row.try_get::<Option<i32>, _>("inventorynextpermissions").ok().flatten().unwrap_or(0x7FFFFFFF) as u32,
            group_permissions: row.try_get::<Option<i32>, _>("inventorygrouppermissions").ok().flatten().unwrap_or(0) as u32,
            group_id: row.try_get::<Option<Uuid>, _>("groupid").ok().flatten().unwrap_or_default(),
            group_owned: row.try_get::<Option<i32>, _>("groupowned").ok().flatten().map(|v| v != 0).unwrap_or(false),
            sale_price: row.try_get::<Option<i32>, _>("saleprice").ok().flatten().unwrap_or(0),
            sale_type: row.try_get::<Option<i32>, _>("saletype").ok().flatten().unwrap_or(0),
        })
    }
}

#[async_trait]
impl InventoryServiceTrait for LocalInventoryService {
    async fn get_folder(&self, folder_id: Uuid) -> Result<Option<InventoryFolder>> {
        debug!("Getting folder: {}", folder_id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            "SELECT folderid, parentfolderid, agentid, foldername, type, version FROM inventoryfolders WHERE folderid = $1"
        )
        .bind(folder_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get folder: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_folder(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_root_folder(&self, principal_id: Uuid) -> Result<Option<InventoryFolder>> {
        debug!("Getting root folder for user: {}", principal_id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            "SELECT folderid, parentfolderid, agentid, foldername, type, version FROM inventoryfolders WHERE agentid = $1 AND parentfolderid = '00000000-0000-0000-0000-000000000000'"
        )
        .bind(principal_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get root folder: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_folder(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_folder_content(&self, principal_id: Uuid, folder_id: Uuid) -> Result<InventoryCollection> {
        debug!("Getting folder content: {} for user: {}", folder_id, principal_id);

        let pool = self.get_pg_pool()?;

        let folder_rows = sqlx::query(
            "SELECT folderid, parentfolderid, agentid, foldername, type, version FROM inventoryfolders WHERE agentid = $1 AND parentfolderid = $2"
        )
        .bind(principal_id)
        .bind(folder_id)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to get subfolders: {}", e))?;

        let item_rows = sqlx::query(
            "SELECT inventoryid, assetid, assettype, parentfolderid, avatarid, inventoryname, inventorydescription, invtype, creatorid, creatordata, flags, creationdate, inventorybasepermissions, inventorycurrentpermissions, inventoryeveryonepermissions, inventorynextpermissions, inventorygrouppermissions, groupid, groupowned, saleprice, saletype FROM inventoryitems WHERE avatarid = $1 AND parentfolderid = $2"
        )
        .bind(principal_id)
        .bind(folder_id)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to get items: {}", e))?;

        let mut folders = Vec::new();
        for row in folder_rows {
            folders.push(self.row_to_folder(&row)?);
        }

        let mut items = Vec::new();
        for row in item_rows {
            items.push(self.row_to_item(&row)?);
        }

        Ok(InventoryCollection { folders, items })
    }

    async fn create_folder(&self, folder: &InventoryFolder) -> Result<bool> {
        info!("Creating folder: {} ({})", folder.name, folder.folder_id);

        let pool = self.get_pg_pool()?;
        sqlx::query(
            "INSERT INTO inventoryfolders (folderid, parentfolderid, agentid, foldername, type, version) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(folder.folder_id)
        .bind(folder.parent_id)
        .bind(folder.owner_id)
        .bind(&folder.name)
        .bind(folder.folder_type)
        .bind(folder.version)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create folder: {}", e))?;

        Ok(true)
    }

    async fn update_folder(&self, folder: &InventoryFolder) -> Result<bool> {
        info!("Updating folder: {} ({})", folder.name, folder.folder_id);

        let pool = self.get_pg_pool()?;
        let result = sqlx::query(
            "UPDATE inventoryfolders SET parentfolderid = $1, foldername = $2, type = $3, version = $4 WHERE folderid = $5 AND agentid = $6"
        )
        .bind(folder.parent_id)
        .bind(&folder.name)
        .bind(folder.folder_type)
        .bind(folder.version)
        .bind(folder.folder_id)
        .bind(folder.owner_id)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update folder: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_folders(&self, principal_id: Uuid, folder_ids: &[Uuid]) -> Result<bool> {
        warn!("Deleting {} folders for user: {}", folder_ids.len(), principal_id);

        let pool = self.get_pg_pool()?;
        for folder_id in folder_ids {
            sqlx::query("DELETE FROM inventoryfolders WHERE folderid = $1 AND agentid = $2")
                .bind(folder_id)
                .bind(principal_id)
                .execute(pool)
                .await
                .map_err(|e| anyhow!("Failed to delete folder: {}", e))?;
        }

        Ok(true)
    }

    async fn get_item(&self, item_id: Uuid) -> Result<Option<InventoryItem>> {
        debug!("Getting item: {}", item_id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            "SELECT inventoryid, assetid, assettype, parentfolderid, avatarid, inventoryname, inventorydescription, invtype, creatorid, creatordata, flags, creationdate, inventorybasepermissions, inventorycurrentpermissions, inventoryeveryonepermissions, inventorynextpermissions, inventorygrouppermissions, groupid, groupowned, saleprice, saletype FROM inventoryitems WHERE inventoryid = $1"
        )
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get item: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_item(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn add_item(&self, item: &InventoryItem) -> Result<bool> {
        info!("Adding item: {} ({})", item.name, item.item_id);

        let pool = self.get_pg_pool()?;
        let creator_str = if item.creator_data.is_empty() {
            item.creator_id.to_string()
        } else {
            format!("{};{}", item.creator_id, item.creator_data)
        };
        sqlx::query(
            "INSERT INTO inventoryitems (inventoryid, assetid, assettype, parentfolderid, avatarid, inventoryname, inventorydescription, invtype, creatorid, flags, creationdate, inventorybasepermissions, inventorycurrentpermissions, inventoryeveryonepermissions, inventorynextpermissions, inventorygrouppermissions, groupid, groupowned, saleprice, saletype) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)"
        )
        .bind(item.item_id)
        .bind(item.asset_id)
        .bind(item.asset_type)
        .bind(item.folder_id)
        .bind(item.owner_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.inv_type)
        .bind(&creator_str)
        .bind(item.flags as i32)
        .bind(item.creation_date as i32)
        .bind(item.base_permissions as i32)
        .bind(item.current_permissions as i32)
        .bind(item.everyone_permissions as i32)
        .bind(item.next_permissions as i32)
        .bind(item.group_permissions as i32)
        .bind(item.group_id)
        .bind(if item.group_owned { 1i32 } else { 0i32 })
        .bind(item.sale_price as i32)
        .bind(item.sale_type as i32)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to add item: {}", e))?;

        Ok(true)
    }

    async fn update_item(&self, item: &InventoryItem) -> Result<bool> {
        info!("Updating item: {} ({})", item.name, item.item_id);

        let pool = self.get_pg_pool()?;
        let creator_str = if item.creator_data.is_empty() {
            item.creator_id.to_string()
        } else {
            format!("{};{}", item.creator_id, item.creator_data)
        };
        let result = sqlx::query(
            "UPDATE inventoryitems SET assetid = $1, assettype = $2, parentfolderid = $3, inventoryname = $4, inventorydescription = $5, invtype = $6, flags = $7, creatorid = $8, inventorybasepermissions = $9, inventorycurrentpermissions = $10, inventoryeveryonepermissions = $11, inventorynextpermissions = $12, inventorygrouppermissions = $13, groupid = $14, groupowned = $15, saleprice = $16, saletype = $17 WHERE inventoryid = $18 AND avatarid = $19"
        )
        .bind(item.asset_id)
        .bind(item.asset_type)
        .bind(item.folder_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.inv_type)
        .bind(item.flags as i32)
        .bind(&creator_str)
        .bind(item.base_permissions as i32)
        .bind(item.current_permissions as i32)
        .bind(item.everyone_permissions as i32)
        .bind(item.next_permissions as i32)
        .bind(item.group_permissions as i32)
        .bind(item.group_id)
        .bind(if item.group_owned { 1i32 } else { 0i32 })
        .bind(item.sale_price as i32)
        .bind(item.sale_type as i32)
        .bind(item.item_id)
        .bind(item.owner_id)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update item: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_items(&self, principal_id: Uuid, item_ids: &[Uuid]) -> Result<bool> {
        warn!("Deleting {} items for user: {}", item_ids.len(), principal_id);

        let pool = self.get_pg_pool()?;
        for item_id in item_ids {
            sqlx::query("DELETE FROM inventoryitems WHERE inventoryid = $1 AND avatarid = $2")
                .bind(item_id)
                .bind(principal_id)
                .execute(pool)
                .await
                .map_err(|e| anyhow!("Failed to delete item: {}", e))?;
        }

        Ok(true)
    }

    async fn get_inventory_skeleton(&self, principal_id: Uuid) -> Result<Vec<InventoryFolder>> {
        debug!("Getting inventory skeleton for user: {}", principal_id);

        let pool = self.get_pg_pool()?;
        let rows = sqlx::query(
            "SELECT folderid, parentfolderid, agentid, foldername, type, version FROM inventoryfolders WHERE agentid = $1 ORDER BY foldername"
        )
        .bind(principal_id)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to get inventory skeleton: {}", e))?;

        let mut folders = Vec::new();
        for row in rows {
            folders.push(self.row_to_folder(&row)?);
        }

        Ok(folders)
    }

    async fn move_items(&self, principal_id: Uuid, items: &[(Uuid, Uuid)]) -> Result<bool> {
        info!("Moving {} items for user: {}", items.len(), principal_id);
        let pool = self.get_pg_pool()?;
        for &(item_id, new_folder_id) in items {
            sqlx::query("UPDATE inventoryitems SET parentfolderid = $1 WHERE inventoryid = $2 AND avatarid = $3")
                .bind(new_folder_id)
                .bind(item_id)
                .bind(principal_id)
                .execute(pool)
                .await
                .map_err(|e| anyhow!("Failed to move item {}: {}", item_id, e))?;
        }
        Ok(true)
    }

    async fn move_folder(&self, principal_id: Uuid, folder_id: Uuid, new_parent_id: Uuid) -> Result<bool> {
        info!("Moving folder {} to parent {} for user {}", folder_id, new_parent_id, principal_id);
        let pool = self.get_pg_pool()?;
        let result = sqlx::query("UPDATE inventoryfolders SET parentfolderid = $1 WHERE folderid = $2 AND agentid = $3")
            .bind(new_parent_id)
            .bind(folder_id)
            .bind(principal_id)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to move folder: {}", e))?;
        Ok(result.rows_affected() > 0)
    }

    async fn purge_folder(&self, principal_id: Uuid, folder_id: Uuid) -> Result<bool> {
        warn!("Purging folder {} for user {}", folder_id, principal_id);
        let pool = self.get_pg_pool()?;
        sqlx::query("DELETE FROM inventoryitems WHERE parentfolderid = $1 AND avatarid = $2")
            .bind(folder_id)
            .bind(principal_id)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to purge items: {}", e))?;
        sqlx::query("DELETE FROM inventoryfolders WHERE parentfolderid = $1 AND agentid = $2")
            .bind(folder_id)
            .bind(principal_id)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to purge subfolders: {}", e))?;
        Ok(true)
    }

    async fn get_active_gestures(&self, principal_id: Uuid) -> Result<Vec<InventoryItem>> {
        debug!("Getting active gestures for user: {}", principal_id);
        let pool = self.get_pg_pool()?;
        let rows = sqlx::query(
            "SELECT inventoryid, assetid, assettype, parentfolderid, avatarid, inventoryname, inventorydescription, invtype, creatorid, creatordata, flags, creationdate, inventorybasepermissions, inventorycurrentpermissions, inventoryeveryonepermissions, inventorynextpermissions, inventorygrouppermissions, groupid, groupowned, saleprice, saletype FROM inventoryitems WHERE avatarid = $1 AND assettype = 21 AND flags = 1"
        )
        .bind(principal_id)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to get active gestures: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(self.row_to_item(&row)?);
        }
        Ok(items)
    }

    async fn get_multiple_folders_content(&self, principal_id: Uuid, folder_ids: &[Uuid]) -> Result<Vec<InventoryCollection>> {
        debug!("Getting content for {} folders for user: {}", folder_ids.len(), principal_id);
        let mut results = Vec::new();
        for &folder_id in folder_ids {
            let collection = self.get_folder_content(principal_id, folder_id).await?;
            results.push(collection);
        }
        Ok(results)
    }

    async fn get_asset_permissions(&self, principal_id: Uuid, asset_id: Uuid) -> Result<i32> {
        let pool = self.get_pg_pool()?;
        let row: Option<(i32,)> = sqlx::query_as(
            "SELECT bit_or(inventorycurrentpermissions) FROM inventoryitems WHERE avatarid = $1 AND assetid = $2 GROUP BY assetid"
        )
        .bind(principal_id)
        .bind(asset_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get asset permissions: {}", e))?;

        Ok(row.map(|(p,)| p).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_folder_default() {
        let folder = InventoryFolder {
            folder_id: Uuid::new_v4(),
            parent_id: Uuid::nil(),
            owner_id: Uuid::new_v4(),
            name: "Test Folder".to_string(),
            folder_type: 0,
            version: 1,
        };

        assert!(!folder.folder_id.is_nil());
        assert!(folder.parent_id.is_nil());
        assert_eq!(folder.name, "Test Folder");
    }

    #[test]
    fn test_inventory_item_default() {
        let item = InventoryItem {
            item_id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            folder_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            creator_id: Uuid::new_v4(),
            creator_data: String::new(),
            name: "Test Item".to_string(),
            description: "Test Description".to_string(),
            asset_type: 0,
            inv_type: 0,
            flags: 0,
            creation_date: 0,
            base_permissions: 0x7FFFFFFF,
            current_permissions: 0x7FFFFFFF,
            everyone_permissions: 0,
            next_permissions: 0x7FFFFFFF,
            group_permissions: 0,
            group_id: Uuid::nil(),
            group_owned: false,
            sale_price: 0,
            sale_type: 0,
        };

        assert!(!item.item_id.is_nil());
        assert_eq!(item.name, "Test Item");
    }
}
