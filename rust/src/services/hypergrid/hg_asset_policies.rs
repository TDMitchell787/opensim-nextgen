use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::uui;
use crate::services::traits::{AssetBase, AssetMetadata, AssetServiceTrait};

#[derive(Debug, Clone)]
pub struct AssetPolicyConfig {
    pub disallow_export: HashSet<i8>,
    pub disallow_import: HashSet<i8>,
    pub home_uri: String,
}

impl Default for AssetPolicyConfig {
    fn default() -> Self {
        Self {
            disallow_export: HashSet::new(),
            disallow_import: HashSet::new(),
            home_uri: std::env::var("OPENSIM_HOME_URI")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
        }
    }
}

impl AssetPolicyConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(export_str) = std::env::var("OPENSIM_HG_DISALLOW_EXPORT") {
            for type_name in export_str.split(',') {
                if let Some(type_id) = asset_type_from_name(type_name.trim()) {
                    config.disallow_export.insert(type_id);
                }
            }
        }

        if let Ok(import_str) = std::env::var("OPENSIM_HG_DISALLOW_IMPORT") {
            for type_name in import_str.split(',') {
                if let Some(type_id) = asset_type_from_name(type_name.trim()) {
                    config.disallow_import.insert(type_id);
                }
            }
        }

        info!(
            "[HG-ASSET] Policy: disallow_export={:?}, disallow_import={:?}",
            config.disallow_export, config.disallow_import
        );
        config
    }
}

fn asset_type_from_name(name: &str) -> Option<i8> {
    match name.to_lowercase().as_str() {
        "texture" => Some(0),
        "sound" => Some(1),
        "callingcard" => Some(2),
        "landmark" => Some(3),
        "clothing" => Some(5),
        "object" => Some(6),
        "notecard" => Some(7),
        "lsltext" => Some(10),
        "lslbytecode" => Some(11),
        "bodypart" => Some(13),
        "animation" => Some(20),
        "gesture" => Some(21),
        "mesh" => Some(49),
        "material" => Some(57),
        "settings" => Some(56),
        _ => name.parse::<i8>().ok(),
    }
}

pub struct HGAssetPolicyService {
    inner: Arc<dyn AssetServiceTrait>,
    config: AssetPolicyConfig,
}

impl HGAssetPolicyService {
    pub fn new(inner: Arc<dyn AssetServiceTrait>, config: AssetPolicyConfig) -> Self {
        Self { inner, config }
    }

    fn allowed_export(&self, asset_type: i8) -> bool {
        !self.config.disallow_export.contains(&asset_type)
    }

    fn allowed_import(&self, asset_type: i8) -> bool {
        !self.config.disallow_import.contains(&asset_type)
    }

    fn rewrite_creator_id(&self, creator_id: &str) -> String {
        if creator_id.contains(';') || creator_id.contains('@') {
            return creator_id.to_string();
        }
        if let Ok(uuid) = uuid::Uuid::parse_str(creator_id) {
            if uuid.is_nil() {
                return creator_id.to_string();
            }
            format!("{};{};", creator_id, self.config.home_uri)
        } else {
            creator_id.to_string()
        }
    }
}

#[async_trait]
impl AssetServiceTrait for HGAssetPolicyService {
    async fn get(&self, id: &str) -> Result<Option<AssetBase>> {
        let asset = self.inner.get(id).await?;
        if let Some(ref a) = asset {
            if !self.allowed_export(a.asset_type) {
                warn!(
                    "[HG-ASSET] Export blocked for asset {} (type={})",
                    id, a.asset_type
                );
                return Ok(None);
            }
            let mut exported = a.clone();
            exported.creator_id = self.rewrite_creator_id(&exported.creator_id);
            return Ok(Some(exported));
        }
        Ok(asset)
    }

    async fn get_metadata(&self, id: &str) -> Result<Option<AssetMetadata>> {
        let meta = self.inner.get_metadata(id).await?;
        if let Some(ref m) = meta {
            if !self.allowed_export(m.asset_type) {
                warn!(
                    "[HG-ASSET] Export blocked for asset metadata {} (type={})",
                    id, m.asset_type
                );
                return Ok(None);
            }
            let mut exported = m.clone();
            exported.creator_id = self.rewrite_creator_id(&exported.creator_id);
            return Ok(Some(exported));
        }
        Ok(meta)
    }

    async fn get_data(&self, id: &str) -> Result<Option<Vec<u8>>> {
        if let Some(meta) = self.inner.get_metadata(id).await? {
            if !self.allowed_export(meta.asset_type) {
                warn!(
                    "[HG-ASSET] Export blocked for asset data {} (type={})",
                    id, meta.asset_type
                );
                return Ok(None);
            }
        }
        self.inner.get_data(id).await
    }

    async fn store(&self, asset: &AssetBase) -> Result<String> {
        if !self.allowed_import(asset.asset_type) {
            warn!(
                "[HG-ASSET] Import blocked for asset {} (type={})",
                asset.id, asset.asset_type
            );
            return Ok(String::new());
        }
        self.inner.store(asset).await
    }

    async fn delete(&self, _id: &str) -> Result<bool> {
        warn!("[HG-ASSET] Delete blocked for foreign asset requests");
        Ok(false)
    }

    async fn asset_exists(&self, id: &str) -> Result<bool> {
        self.inner.asset_exists(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_from_name() {
        assert_eq!(asset_type_from_name("Texture"), Some(0));
        assert_eq!(asset_type_from_name("LSLText"), Some(10));
        assert_eq!(asset_type_from_name("Notecard"), Some(7));
        assert_eq!(asset_type_from_name("Object"), Some(6));
        assert_eq!(asset_type_from_name("49"), Some(49));
        assert_eq!(asset_type_from_name("unknown"), None);
    }

    #[test]
    fn test_rewrite_creator_id() {
        let config = AssetPolicyConfig {
            home_uri: "http://mygrid.example.com:9000".to_string(),
            ..Default::default()
        };
        let service = HGAssetPolicyService {
            inner: Arc::new(DummyAsset),
            config,
        };
        assert_eq!(
            service.rewrite_creator_id("12345678-1234-1234-1234-123456789012"),
            "12345678-1234-1234-1234-123456789012;http://mygrid.example.com:9000;"
        );
        let already_uui = "12345678-1234-1234-1234-123456789012;http://other.grid.com;";
        assert_eq!(service.rewrite_creator_id(already_uui), already_uui);
        assert_eq!(
            service.rewrite_creator_id("00000000-0000-0000-0000-000000000000"),
            "00000000-0000-0000-0000-000000000000"
        );
    }

    struct DummyAsset;
    #[async_trait]
    impl AssetServiceTrait for DummyAsset {
        async fn get(&self, _id: &str) -> Result<Option<AssetBase>> {
            Ok(None)
        }
        async fn get_metadata(&self, _id: &str) -> Result<Option<AssetMetadata>> {
            Ok(None)
        }
        async fn get_data(&self, _id: &str) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
        async fn store(&self, _asset: &AssetBase) -> Result<String> {
            Ok(String::new())
        }
        async fn delete(&self, _id: &str) -> Result<bool> {
            Ok(false)
        }
        async fn asset_exists(&self, _id: &str) -> Result<bool> {
            Ok(false)
        }
    }
}
