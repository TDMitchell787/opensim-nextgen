//! IAR (Inventory Archive) writer
//!
//! Saves user inventory to an IAR file from the database.

use anyhow::{anyhow, Context, Result};
use sqlx::PgPool;
use std::collections::HashSet;
use std::path::Path;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::xml_schemas::{create_archive_xml, create_folder_xml, create_item_xml, IarInventoryItemXml};
use crate::archives::common::{paths, AssetType, SaveStatistics};
use crate::archives::tar_handler::TarGzWriter;

/// Result of saving an IAR
#[derive(Debug, Clone)]
pub struct IarSaveResult {
    pub success: bool,
    pub stats: SaveStatistics,
    pub output_path: std::path::PathBuf,
    pub warnings: Vec<String>,
}

/// IAR save options
#[derive(Debug, Clone)]
pub struct IarSaveOptions {
    pub user_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub include_assets: bool,
}

/// IAR writer implementation
pub struct IarWriter {
    db_pool: PgPool,
}

impl IarWriter {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Save user inventory to an IAR file
    pub async fn save<P: AsRef<Path>>(
        &self,
        path: P,
        options: IarSaveOptions,
    ) -> Result<IarSaveResult> {
        let start = std::time::Instant::now();
        let mut stats = SaveStatistics::default();
        let mut warnings = Vec::new();
        let mut collected_assets: HashSet<Uuid> = HashSet::new();

        info!("Saving IAR to {:?} for user {}", path.as_ref(), options.user_id);

        let mut archive = TarGzWriter::create(path.as_ref())
            .with_context(|| "Failed to create IAR archive")?;

        // Write archive.xml
        let archive_xml = create_archive_xml(1, 0);
        archive.add_file(paths::ARCHIVE_XML, archive_xml.as_bytes())?;

        // Determine starting folder
        let start_folder_id = match options.folder_id {
            Some(id) => id,
            None => self.get_user_root_folder(&options.user_id).await?,
        };

        // Get folder structure recursively
        let folders = self.get_folder_tree(&options.user_id, &start_folder_id).await?;

        // Build path mapping for folders
        let mut folder_paths: std::collections::HashMap<Uuid, String> = std::collections::HashMap::new();
        let root_name = self.get_folder_name(&start_folder_id).await?.unwrap_or_else(|| "My Inventory".to_string());
        folder_paths.insert(start_folder_id, root_name.clone());

        // Write folders and build paths
        for folder in &folders {
            let parent_path = folder_paths.get(&folder.parent_id)
                .cloned()
                .unwrap_or_default();

            let folder_path = if parent_path.is_empty() {
                folder.name.clone()
            } else {
                format!("{}/{}", parent_path, folder.name)
            };
            folder_paths.insert(folder.id, folder_path.clone());

            // Write folder metadata
            let metadata_path = format!(
                "{}{}/{}",
                paths::INVENTORY_PATH,
                folder_path,
                paths::FOLDER_METADATA
            );
            let metadata_xml = create_folder_xml(
                &folder.name,
                &folder.id,
                folder.folder_type,
                &options.user_id,
            );
            archive.add_file(&metadata_path, metadata_xml.as_bytes())?;
            stats.folders_saved += 1;
        }

        // Write root folder metadata
        let root_metadata_path = format!(
            "{}{}/{}",
            paths::INVENTORY_PATH,
            root_name,
            paths::FOLDER_METADATA
        );
        let root_folder_type = self.get_folder_type(&start_folder_id).await?.unwrap_or(8);
        let root_metadata_xml = create_folder_xml(
            &root_name,
            &start_folder_id,
            root_folder_type,
            &options.user_id,
        );
        archive.add_file(&root_metadata_path, root_metadata_xml.as_bytes())?;
        stats.folders_saved += 1;

        info!("Saved {} folders", stats.folders_saved);

        // Write items for each folder
        let all_folder_ids: Vec<Uuid> = std::iter::once(start_folder_id)
            .chain(folders.iter().map(|f| f.id))
            .collect();

        for folder_id in &all_folder_ids {
            let items = self.get_folder_items(folder_id).await?;
            let folder_path = folder_paths.get(folder_id)
                .cloned()
                .unwrap_or_default();

            for item in items {
                // Collect asset ID for later
                if let Some(asset_id) = item.asset_uuid() {
                    collected_assets.insert(asset_id);
                }

                // Write item XML
                let item_filename = format!(
                    "{}__{}",
                    sanitize_filename(&item.name),
                    item.id
                );
                let item_path = format!(
                    "{}{}/{}.xml",
                    paths::INVENTORY_PATH,
                    folder_path,
                    item_filename
                );
                let item_xml = create_item_xml(&item);
                archive.add_file(&item_path, item_xml.as_bytes())?;
                stats.items_saved += 1;
            }
        }

        info!("Saved {} inventory items", stats.items_saved);

        // Write assets if requested
        if options.include_assets {
            info!("Collecting {} assets...", collected_assets.len());
            for asset_id in collected_assets {
                match self.get_asset_data(&asset_id).await {
                    Ok(Some((asset_type, data))) => {
                        let extension = AssetType::from_i32(asset_type).extension();
                        let asset_path = format!(
                            "{}{}{}",
                            paths::ASSETS_PATH,
                            asset_id.to_string().to_lowercase(),
                            extension
                        );
                        archive.add_file(&asset_path, &data)?;
                        stats.assets_saved += 1;
                    }
                    Ok(None) => {
                        warnings.push(format!("Asset not found: {}", asset_id));
                    }
                    Err(e) => {
                        warnings.push(format!("Failed to get asset {}: {}", asset_id, e));
                    }
                }
            }
            info!("Saved {} assets", stats.assets_saved);
        }

        // Finalize archive
        archive.finish()?;

        // Get file size
        stats.archive_size_bytes = std::fs::metadata(path.as_ref())
            .map(|m| m.len())
            .unwrap_or(0);

        stats.elapsed_ms = start.elapsed().as_millis() as u64;

        Ok(IarSaveResult {
            success: true,
            stats,
            output_path: path.as_ref().to_path_buf(),
            warnings,
        })
    }

    async fn get_user_root_folder(&self, user_id: &Uuid) -> Result<Uuid> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT folderid FROM inventoryfolders WHERE agentid = $1 AND type = 8::bigint LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        row.map(|(id,)| id)
            .ok_or_else(|| anyhow!("No root folder found for user {}", user_id))
    }

    async fn get_folder_name(&self, folder_id: &Uuid) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT foldername FROM inventoryfolders WHERE folderid = $1"
        )
        .bind(folder_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.map(|(name,)| name))
    }

    async fn get_folder_type(&self, folder_id: &Uuid) -> Result<Option<i32>> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT type FROM inventoryfolders WHERE folderid = $1"
        )
        .bind(folder_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.map(|(t,)| t as i32))
    }

    async fn get_folder_tree(&self, user_id: &Uuid, root_id: &Uuid) -> Result<Vec<FolderInfo>> {
        let rows: Vec<(Uuid, Uuid, String, i64)> = sqlx::query_as(
            r#"WITH RECURSIVE folder_tree AS (
                SELECT folderid, parentfolderid, foldername, type
                FROM inventoryfolders
                WHERE parentfolderid = $1 AND agentid = $2

                UNION ALL

                SELECT f.folderid, f.parentfolderid, f.foldername, f.type
                FROM inventoryfolders f
                JOIN folder_tree ft ON f.parentfolderid = ft.folderid
                WHERE f.agentid = $2
            )
            SELECT folderid, parentfolderid, foldername, type FROM folder_tree"#
        )
        .bind(root_id)
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows.into_iter().map(|(id, parent_id, name, folder_type)| {
            FolderInfo { id, parent_id, name, folder_type: folder_type as i32 }
        }).collect())
    }

    async fn get_folder_items(&self, folder_id: &Uuid) -> Result<Vec<IarInventoryItemXml>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"SELECT
                inventoryid, inventoryname, invtype, creatorid, avatarid,
                inventorydescription, assetid, assettype,
                inventorycurrentpermissions, inventorybasepermissions,
                inventoryeveryonepermissions, inventorynextpermissions,
                inventorygrouppermissions, groupid, groupowned,
                saleprice, saletype, flags, creationdate
            FROM inventoryitems
            WHERE parentfolderid = $1"#
        )
        .bind(folder_id)
        .fetch_all(&self.db_pool)
        .await?;

        let mut items = Vec::new();
        for row in rows {
            let id: Uuid = row.get("inventoryid");
            let name: String = row.get("inventoryname");
            let inv_type: i64 = row.get("invtype");
            let creator_id: String = row.get("creatorid");
            let owner_id: Uuid = row.get("avatarid");
            let description: Option<String> = row.try_get("inventorydescription").ok();
            let asset_id: Uuid = row.get("assetid");
            let asset_type: i64 = row.get("assettype");
            let current_perms: i32 = row.get("inventorycurrentpermissions");
            let base_perms: i32 = row.get("inventorybasepermissions");
            let everyone_perms: i32 = row.get("inventoryeveryonepermissions");
            let next_perms: i32 = row.get("inventorynextpermissions");
            let group_perms: i32 = row.get("inventorygrouppermissions");
            let group_id: Uuid = row.try_get("groupid").unwrap_or_default();
            let group_owned_int: i32 = row.try_get("groupowned").unwrap_or(0);
            let sale_price: i32 = row.get("saleprice");
            let sale_type: i32 = row.get("saletype");
            let flags: i64 = row.get("flags");
            let creation_date: i64 = row.get("creationdate");

            items.push(IarInventoryItemXml {
                id: id.to_string(),
                name,
                inv_type: inv_type as i32,
                creator_id,
                creator_data: None,
                owner_id: owner_id.to_string(),
                description,
                asset_id: asset_id.to_string(),
                asset_type: asset_type as i32,
                current_permissions: current_perms as u32,
                base_permissions: base_perms as u32,
                everyone_permissions: everyone_perms as u32,
                next_permissions: next_perms as u32,
                group_permissions: group_perms as u32,
                group_id: group_id.to_string(),
                group_owned: group_owned_int != 0,
                sale_price,
                sale_type: sale_type as u8,
                flags: flags as u32,
                creation_date: creation_date as i32,
            });
        }
        Ok(items)
    }

    async fn get_asset_data(&self, asset_id: &Uuid) -> Result<Option<(i32, Vec<u8>)>> {
        let row: Option<(i64, Vec<u8>)> = sqlx::query_as(
            "SELECT assettype, data FROM assets WHERE id = $1"
        )
        .bind(asset_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.map(|(t, d)| (t as i32, d)))
    }
}

#[derive(Debug)]
struct FolderInfo {
    id: Uuid,
    parent_id: Uuid,
    name: String,
    folder_type: i32,
}

/// Sanitize a string for use as a filename
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
