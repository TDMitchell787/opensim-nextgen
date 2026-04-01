//! Asset Loader - Loads default assets from OpenSim XML asset sets into database
//!
//! This module replicates the OpenSim AssetLoaderFileSystem functionality,
//! loading assets from XML definitions in bin/assets/ directory at server startup.
//! This is CRITICAL for viewer functionality - without these assets, viewers cannot render.

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{anyhow, Result};
use tracing::{info, warn, debug};
use uuid::Uuid;
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone)]
pub struct AssetDefinition {
    pub asset_id: Uuid,
    pub name: String,
    pub description: String,
    pub asset_type: i32,
    pub file_name: String,
}

#[derive(Debug)]
pub struct AssetLoader {
    assets_base_path: PathBuf,
    loaded_count: usize,
    skipped_count: usize,
}

impl AssetLoader {
    pub fn new(assets_base_path: PathBuf) -> Self {
        Self {
            assets_base_path,
            loaded_count: 0,
            skipped_count: 0,
        }
    }

    pub async fn load_all_assets(&mut self, pool: &Pool<Postgres>) -> Result<()> {
        info!("🎨 Starting OpenSim asset loading from XML asset sets...");
        info!("📁 Asset base path: {:?}", self.assets_base_path);

        let asset_sets_path = self.assets_base_path.join("AssetSets.xml");
        if !asset_sets_path.exists() {
            return Err(anyhow!("AssetSets.xml not found at {:?}", asset_sets_path));
        }

        let asset_set_files = self.parse_asset_sets_xml(&asset_sets_path)?;
        info!("📋 Found {} asset sets to process", asset_set_files.len());

        for (set_name, set_file) in &asset_set_files {
            let set_path = self.assets_base_path.join(set_file);
            debug!("  Processing asset set: {} -> {:?}", set_name, set_path);
            if set_path.exists() {
                match self.load_asset_set(pool, set_name, &set_path).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("  ✅ {}: {} assets loaded", set_name, count);
                        } else {
                            debug!("  ℹ️  {}: 0 new assets (all may exist)", set_name);
                        }
                    }
                    Err(e) => {
                        warn!("  ⚠️  {}: Failed to load - {}", set_name, e);
                    }
                }
            } else {
                warn!("  ⚠️  {}: File not found at {:?}", set_name, set_path);
            }
        }

        info!("🎨 Asset loading complete: {} loaded, {} skipped (already exist)",
              self.loaded_count, self.skipped_count);

        Ok(())
    }

    fn parse_asset_sets_xml(&self, path: &Path) -> Result<Vec<(String, String)>> {
        let content = fs::read_to_string(path)?;
        let mut asset_sets = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("<Section Name=") {
                if let Some(name) = self.extract_attribute(line, "Name") {
                    let mut file_path = String::new();
                    continue;
                }
            }
            if line.starts_with("<Key Name=\"file\"") {
                if let Some(file) = self.extract_attribute(line, "Value") {
                    if !file.is_empty() {
                        let section_name = self.find_previous_section_name(&content, line)?;
                        asset_sets.push((section_name, file));
                    }
                }
            }
        }

        if asset_sets.is_empty() {
            for section in content.split("<Section ") {
                if section.contains("Name=") && section.contains("file") {
                    if let (Some(name), Some(file)) = (
                        self.extract_attribute_from_section(section, "Section", "Name"),
                        self.extract_key_value(section, "file")
                    ) {
                        if !file.is_empty() && !file.contains("MyAssetSet") {
                            asset_sets.push((name, file));
                        }
                    }
                }
            }
        }

        Ok(asset_sets)
    }

    fn find_previous_section_name(&self, content: &str, target_line: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let target_idx = lines.iter().position(|l| l.trim() == target_line);

        if let Some(idx) = target_idx {
            for i in (0..idx).rev() {
                let line = lines[i].trim();
                if line.starts_with("<Section Name=") {
                    if let Some(name) = self.extract_attribute(line, "Name") {
                        return Ok(name);
                    }
                }
            }
        }

        Err(anyhow!("Could not find section name"))
    }

    fn extract_attribute(&self, line: &str, attr_name: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr_name);
        if let Some(start) = line.find(&pattern) {
            let value_start = start + pattern.len();
            if let Some(end) = line[value_start..].find('"') {
                return Some(line[value_start..value_start + end].to_string());
            }
        }
        None
    }

    fn extract_attribute_from_section(&self, section: &str, _elem: &str, attr_name: &str) -> Option<String> {
        self.extract_attribute(section, attr_name)
    }

    fn extract_key_value(&self, section: &str, key_name: &str) -> Option<String> {
        for line in section.lines() {
            let line = line.trim();
            if line.contains(&format!("Name=\"{}\"", key_name)) {
                return self.extract_attribute(line, "Value");
            }
        }
        None
    }

    async fn load_asset_set(&mut self, pool: &Pool<Postgres>, set_name: &str, set_path: &Path) -> Result<usize> {
        let content = fs::read_to_string(set_path)?;
        let base_dir = set_path.parent().unwrap_or(Path::new("."));
        let mut loaded = 0;

        let assets = self.parse_asset_set_xml(&content)?;
        debug!("  📄 {}: parsed {} asset definitions from {:?}", set_name, assets.len(), set_path);

        for asset_def in assets {
            match self.load_single_asset(pool, base_dir, &asset_def).await {
                Ok(true) => {
                    loaded += 1;
                    self.loaded_count += 1;
                }
                Ok(false) => {
                    self.skipped_count += 1;
                }
                Err(e) => {
                    debug!("  Failed to load asset {}: {}", asset_def.name, e);
                }
            }
        }

        Ok(loaded)
    }

    fn parse_asset_set_xml(&self, content: &str) -> Result<Vec<AssetDefinition>> {
        let mut assets = Vec::new();

        for section in content.split("<Section ") {
            if section.trim().is_empty() || section.starts_with("?>") {
                continue;
            }

            let asset_id = self.extract_key_value(section, "assetID");
            let name = self.extract_key_value(section, "name");
            let asset_type_str = self.extract_key_value(section, "assetType");
            let file_name = self.extract_key_value(section, "fileName");
            let description = self.extract_key_value(section, "description").unwrap_or_default();

            if let (Some(id_str), Some(name), Some(type_str), Some(file)) =
                (asset_id, name, asset_type_str, file_name)
            {
                if let (Ok(uuid), Ok(asset_type)) = (Uuid::parse_str(&id_str), type_str.parse::<i32>()) {
                    assets.push(AssetDefinition {
                        asset_id: uuid,
                        name,
                        description,
                        asset_type,
                        file_name: file,
                    });
                }
            }
        }

        Ok(assets)
    }

    async fn load_single_asset(&self, pool: &Pool<Postgres>, base_dir: &Path, asset_def: &AssetDefinition) -> Result<bool> {
        let file_path = base_dir.join(&asset_def.file_name);
        if !file_path.exists() {
            return Err(anyhow!("Asset file not found: {:?}", file_path));
        }

        let data = fs::read(&file_path)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let existing_size: Option<i32> = sqlx::query_scalar(
            "SELECT length(data)::int4 FROM assets WHERE id = $1"
        )
        .bind(asset_def.asset_id)
        .fetch_optional(pool)
        .await?;

        match existing_size {
            Some(size) if (size as usize) >= data.len() => {
                return Ok(false);
            }
            Some(size) => {
                info!("Replacing stub asset {} ({} bytes -> {} bytes): {}",
                    asset_def.asset_id, size, data.len(), asset_def.name);
                sqlx::query("UPDATE assets SET data = $2, access_time = $3 WHERE id = $1")
                    .bind(asset_def.asset_id)
                    .bind(&data)
                    .bind(now)
                    .execute(pool)
                    .await?;
                return Ok(true);
            }
            None => {}
        }

        sqlx::query(r#"
            INSERT INTO assets (id, name, description, assetType, local, temporary, data, create_time, access_time, asset_flags, CreatorID)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET data = $7, access_time = $9
        "#)
        .bind(asset_def.asset_id)
        .bind(&asset_def.name)
        .bind(&asset_def.description)
        .bind(asset_def.asset_type as i64)
        .bind(0i64)
        .bind(0i64)
        .bind(&data)
        .bind(now)
        .bind(now)
        .bind(0i64)
        .bind("")
        .execute(pool)
        .await?;

        Ok(true)
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.loaded_count, self.skipped_count)
    }
}

pub async fn load_default_assets(pool: &Pool<Postgres>, assets_base_path: &Path) -> Result<()> {
    let mut loader = AssetLoader::new(assets_base_path.to_path_buf());
    loader.load_all_assets(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_asset_definition() {
        let xml = r#"
        <Section Name="Default Alpha">
            <Key Name="assetID" Value="1578a2b1-5179-4b53-b618-fe00ca5a5594" />
            <Key Name="name" Value="alpha" />
            <Key Name="assetType" Value="0" />
            <Key Name="fileName" Value="default_alpha.jp2" />
        </Section>
        "#;

        let loader = AssetLoader::new(PathBuf::from("."));
        let assets = loader.parse_asset_set_xml(xml).unwrap();

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "alpha");
        assert_eq!(assets[0].asset_type, 0);
    }
}
