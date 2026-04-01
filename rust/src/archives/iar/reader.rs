//! IAR (Inventory Archive) reader
//!
//! Loads user inventory from an IAR file into the database.

use anyhow::{anyhow, Context, Result};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::xml_schemas::{IarArchiveXml, IarInventoryItemXml};
use crate::archives::common::{extract_asset_uuid_from_path, paths, AssetType, LoadStatistics};
use crate::archives::tar_handler::TarGzReader;

#[derive(Debug, Clone)]
pub struct IarLoadResult {
    pub success: bool,
    pub stats: LoadStatistics,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IarLoadOptions {
    pub user_id: Uuid,
    pub target_folder: Option<Uuid>,
    pub merge: bool,
    pub skip_existing_assets: bool,
}

pub struct IarReader {
    db_pool: PgPool,
}

fn unescape_iar_path(s: &str) -> String {
    s.replace("&#47;", "/")
        .replace("&#38;", "&")
        .replace("&#60;", "<")
        .replace("&#62;", ">")
        .replace("&#34;", "\"")
        .replace("&#39;", "'")
}

fn parse_iar_path_component(component: &str) -> (String, Option<Uuid>) {
    if let Some(sep_idx) = component.rfind("__") {
        let name_part = &component[..sep_idx];
        let uuid_part = &component[sep_idx + 2..];
        let uuid = Uuid::parse_str(uuid_part).ok();
        (unescape_iar_path(name_part), uuid)
    } else {
        (unescape_iar_path(component), None)
    }
}

impl IarReader {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn load<P: AsRef<Path>>(
        &self,
        path: P,
        options: IarLoadOptions,
    ) -> Result<IarLoadResult> {
        let start = std::time::Instant::now();
        let mut stats = LoadStatistics::default();
        let mut warnings = Vec::new();
        let errors = Vec::new();

        info!("Loading IAR from {:?} for user {}", path.as_ref(), options.user_id);

        let archive = TarGzReader::open(path.as_ref())
            .with_context(|| "Failed to open IAR archive")?;

        if let Some(archive_xml_data) = archive.get_archive_xml() {
            match self.parse_archive_xml(archive_xml_data) {
                Ok(meta) => {
                    info!(
                        "IAR version: {}.{}",
                        meta.major_version, meta.minor_version
                    );
                }
                Err(e) => {
                    warnings.push(format!("Could not parse archive.xml: {}", e));
                }
            }
        }

        let root_folder_id = match options.target_folder {
            Some(id) => id,
            None => self.get_user_root_folder(&options.user_id).await?,
        };

        info!("Processing assets...");
        for (path, data) in archive.get_asset_entries() {
            match self.import_asset(path, data, options.skip_existing_assets).await {
                Ok(imported) => {
                    if imported {
                        stats.assets_loaded += 1;
                    } else {
                        stats.assets_skipped += 1;
                    }
                }
                Err(e) => {
                    stats.assets_failed += 1;
                    if stats.assets_failed <= 10 {
                        warnings.push(format!("Failed to import asset {}: {}", path, e));
                    }
                }
            }
        }
        info!(
            "Assets: {} loaded, {} skipped, {} failed",
            stats.assets_loaded, stats.assets_skipped, stats.assets_failed
        );

        info!("Processing inventory structure...");
        let mut folder_map: HashMap<String, Uuid> = HashMap::new();
        folder_map.insert(String::new(), root_folder_id);

        let inventory_entries = archive.get_inventory_entries();
        let inv_dirs = inventory_entries.iter().filter(|(p, _)| p.ends_with('/')).count();
        let inv_files = inventory_entries.iter().filter(|(p, _)| p.ends_with(".xml")).count();
        info!("Inventory entries: {} total ({} dirs, {} xml files)", inventory_entries.len(), inv_dirs, inv_files);

        let mut dir_entries: Vec<String> = inventory_entries
            .iter()
            .filter(|(p, _)| p.ends_with('/') && p.as_str() != "inventory/")
            .map(|(p, _)| p.to_string())
            .collect();
        dir_entries.sort_by_key(|p| p.matches('/').count());

        for dir_path in &dir_entries {
            let rel_path = dir_path
                .strip_prefix("inventory/")
                .unwrap_or(dir_path)
                .trim_end_matches('/');

            if rel_path.is_empty() {
                continue;
            }

            if folder_map.contains_key(rel_path) {
                continue;
            }

            let components: Vec<&str> = rel_path.split('/').collect();
            let mut current_parent_id = root_folder_id;
            let mut current_path = String::new();

            for component in &components {
                if !current_path.is_empty() {
                    current_path.push('/');
                }
                current_path.push_str(component);

                if let Some(&existing_id) = folder_map.get(&current_path) {
                    current_parent_id = existing_id;
                    continue;
                }

                let (folder_name, folder_uuid) = parse_iar_path_component(component);
                let folder_id = folder_uuid.unwrap_or_else(Uuid::new_v4);

                match self
                    .create_folder(
                        folder_id,
                        &options.user_id,
                        current_parent_id,
                        &folder_name,
                    )
                    .await
                {
                    Ok(_) => {
                        stats.folders_created += 1;
                        folder_map.insert(current_path.clone(), folder_id);
                        current_parent_id = folder_id;
                        debug!("Created folder: {} -> {}", folder_name, folder_id);
                    }
                    Err(e) => {
                        warnings.push(format!("Failed to create folder {}: {}", folder_name, e));
                        folder_map.insert(current_path.clone(), folder_id);
                        current_parent_id = folder_id;
                    }
                }
            }
        }
        info!("Created {} folders", stats.folders_created);

        for (entry_path, data) in &inventory_entries {
            if entry_path.ends_with('/') || !entry_path.ends_with(".xml") {
                continue;
            }

            let rel_path = entry_path
                .strip_prefix("inventory/")
                .unwrap_or(entry_path);

            let folder_rel_path = rel_path
                .rsplit_once('/')
                .map(|(p, _)| p)
                .unwrap_or("");

            let folder_id = if let Some(&id) = folder_map.get(folder_rel_path) {
                id
            } else {
                let components: Vec<&str> = folder_rel_path.split('/').filter(|s| !s.is_empty()).collect();
                let mut current_parent_id = root_folder_id;
                let mut current_path = String::new();

                for component in &components {
                    if !current_path.is_empty() {
                        current_path.push('/');
                    }
                    current_path.push_str(component);

                    if let Some(&existing_id) = folder_map.get(&current_path) {
                        current_parent_id = existing_id;
                        continue;
                    }

                    let (folder_name, folder_uuid) = parse_iar_path_component(component);
                    let fid = folder_uuid.unwrap_or_else(Uuid::new_v4);
                    let _ = self.create_folder(fid, &options.user_id, current_parent_id, &folder_name).await;
                    stats.folders_created += 1;
                    folder_map.insert(current_path.clone(), fid);
                    current_parent_id = fid;
                }

                current_parent_id
            };

            match self
                .process_inventory_item(data, &options.user_id, folder_id)
                .await
            {
                Ok(_) => stats.items_created += 1,
                Err(e) => {
                    if warnings.len() < 50 {
                        warnings.push(format!("Failed to create item {}: {}", entry_path, e));
                    }
                }
            }
        }
        info!("Created {} inventory items", stats.items_created);

        stats.elapsed_ms = start.elapsed().as_millis() as u64;

        Ok(IarLoadResult {
            success: errors.is_empty(),
            stats,
            warnings,
            errors,
        })
    }

    fn parse_archive_xml(&self, data: &[u8]) -> Result<IarArchiveXml> {
        let xml_str = std::str::from_utf8(data)?;
        quick_xml::de::from_str(xml_str).map_err(|e| anyhow!("XML parse error: {}", e))
    }

    async fn get_user_root_folder(&self, user_id: &Uuid) -> Result<Uuid> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT folderid FROM inventoryfolders WHERE agentid = $1 AND type = 8 LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        row.map(|(id,)| id)
            .ok_or_else(|| anyhow!("No root folder found for user {}", user_id))
    }

    async fn import_asset(&self, path: &str, data: &[u8], skip_existing: bool) -> Result<bool> {
        let uuid = extract_asset_uuid_from_path(path)
            .ok_or_else(|| anyhow!("Could not extract UUID from path: {}", path))?;

        let asset_type = self.determine_asset_type_from_path(path);

        if skip_existing {
            let exists: Option<(i32,)> =
                sqlx::query_as("SELECT 1 FROM assets WHERE id = $1")
                    .bind(uuid)
                    .fetch_optional(&self.db_pool)
                    .await?;

            if exists.is_some() {
                return Ok(false);
            }
        }

        sqlx::query(
            r#"INSERT INTO assets (id, assettype, name, description, data, create_time, local, temporary)
               VALUES ($1, $2, $3, '', $4, EXTRACT(EPOCH FROM NOW())::bigint, 0, 0)
               ON CONFLICT (id) DO UPDATE SET data = $4"#,
        )
        .bind(uuid)
        .bind(asset_type as i32)
        .bind(format!("IAR Import {}", uuid))
        .bind(data)
        .execute(&self.db_pool)
        .await?;

        Ok(true)
    }

    fn determine_asset_type_from_path(&self, path: &str) -> i32 {
        if path.contains("_texture") {
            return AssetType::Texture as i32;
        }
        if path.contains("_sound") {
            return AssetType::Sound as i32;
        }
        if path.contains("_clothing") {
            return AssetType::Clothing as i32;
        }
        if path.contains("_bodypart") {
            return AssetType::Bodypart as i32;
        }
        if path.contains("_object") {
            return AssetType::Object as i32;
        }
        if path.contains("_notecard") {
            return AssetType::Notecard as i32;
        }
        if path.contains("_script") || path.contains("_lsl") {
            return AssetType::LSLText as i32;
        }
        if path.contains("_animation") {
            return AssetType::Animation as i32;
        }
        if path.contains("_gesture") {
            return AssetType::Gesture as i32;
        }
        if path.contains("_mesh") {
            return AssetType::Mesh as i32;
        }
        if path.contains("_landmark") {
            return AssetType::Landmark as i32;
        }
        AssetType::Unknown as i32
    }

    async fn create_folder(
        &self,
        folder_id: Uuid,
        user_id: &Uuid,
        parent_id: Uuid,
        name: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO inventoryfolders (folderid, agentid, parentfolderid, foldername, type, version)
               VALUES ($1, $2, $3, $4, -1, 1)
               ON CONFLICT (folderid) DO UPDATE SET foldername = $4"#,
        )
        .bind(folder_id)
        .bind(user_id)
        .bind(parent_id)
        .bind(name)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn process_inventory_item(
        &self,
        data: &[u8],
        user_id: &Uuid,
        folder_id: Uuid,
    ) -> Result<()> {
        let xml_str = std::str::from_utf8(data)?;
        let item: IarInventoryItemXml = quick_xml::de::from_str(xml_str)
            .map_err(|e| anyhow!("Failed to parse item XML: {}", e))?;

        let item_id = item.id_uuid().unwrap_or_else(Uuid::new_v4);
        let asset_id = item.asset_uuid().unwrap_or_default();
        let creator_id = item.creator_uuid().unwrap_or(*user_id);
        let group_id = item.group_uuid().unwrap_or_default();

        sqlx::query(
            r#"INSERT INTO inventoryitems (
                inventoryid, assetid, assettype, parentfolderid, avatarid, creatorid,
                inventoryname, inventorydescription, inventorynextpermissions,
                inventorycurrentpermissions, inventorybasepermissions,
                inventoryeveryonepermissions, inventorygrouppermissions,
                invtype, saleprice, saletype, creationdate, groupid, groupowned, flags
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            ON CONFLICT (inventoryid) DO UPDATE SET inventoryname = $7"#,
        )
        .bind(item_id)
        .bind(asset_id)
        .bind(item.asset_type)
        .bind(folder_id)
        .bind(user_id)
        .bind(creator_id.to_string())
        .bind(&item.name)
        .bind(item.description.as_deref().unwrap_or(""))
        .bind(item.next_permissions as i32)
        .bind(item.current_permissions as i32)
        .bind(item.base_permissions as i32)
        .bind(item.everyone_permissions as i32)
        .bind(item.group_permissions as i32)
        .bind(item.inv_type)
        .bind(item.sale_price)
        .bind(item.sale_type as i32)
        .bind(item.creation_date)
        .bind(group_id)
        .bind(if item.group_owned { 1i32 } else { 0i32 })
        .bind(item.flags as i32)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}
