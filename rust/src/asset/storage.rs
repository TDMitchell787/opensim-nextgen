//! Asset Storage Backend Traits and Implementations
//!
//! Provides storage backend interfaces and implementations for asset persistence.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AssetData, AssetType};

/// Storage backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackendType {
    FileSystem,
    Database,
    S3Compatible,
    Redis,
    Memory,
}

/// Storage backend interface
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store asset data
    async fn store_asset(
        &self,
        asset_id: &Uuid,
        asset_type: AssetType,
        data: &AssetData,
    ) -> Result<()>;

    /// Retrieve asset data
    async fn retrieve_asset(&self, asset_id: &Uuid) -> Result<Option<(AssetType, AssetData)>>;

    /// Delete asset
    async fn delete_asset(&self, asset_id: &Uuid) -> Result<bool>;

    /// Check if asset exists
    async fn asset_exists(&self, asset_id: &Uuid) -> Result<bool>;

    /// List all asset IDs
    async fn list_assets(&self) -> Result<Vec<Uuid>>;

    /// Get storage statistics
    async fn get_stats(&self) -> Result<StorageStats>;

    /// Load asset data from storage path
    async fn load(&self, storage_path: &str) -> Result<AssetData>;

    /// Store asset data to storage and return path
    async fn store(&self, asset_id: &Uuid, data: &AssetData) -> Result<String>;
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_assets: u64,
    pub total_size_bytes: u64,
    pub backend_type: StorageBackendType,
    pub last_backup: Option<std::time::SystemTime>,
    pub available_space_bytes: Option<u64>,
}

/// File system storage implementation
pub struct FileSystemStorage {
    base_path: std::path::PathBuf,
}

impl FileSystemStorage {
    pub fn new(base_path: std::path::PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }
}

#[async_trait]
impl StorageBackend for FileSystemStorage {
    async fn store_asset(
        &self,
        asset_id: &Uuid,
        asset_type: AssetType,
        data: &AssetData,
    ) -> Result<()> {
        let path = self.base_path.join(format!("{}.asset", asset_id));
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn retrieve_asset(&self, asset_id: &Uuid) -> Result<Option<(AssetType, AssetData)>> {
        let path = self.base_path.join(format!("{}.asset", asset_id));
        match tokio::fs::read(path).await {
            Ok(data) => Ok(Some((AssetType::Texture, data))),
            Err(_) => Ok(None),
        }
    }

    async fn delete_asset(&self, asset_id: &Uuid) -> Result<bool> {
        let path = self.base_path.join(format!("{}.asset", asset_id));
        match tokio::fs::remove_file(path).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn asset_exists(&self, asset_id: &Uuid) -> Result<bool> {
        let path = self.base_path.join(format!("{}.asset", asset_id));
        Ok(path.exists())
    }

    async fn list_assets(&self) -> Result<Vec<Uuid>> {
        let mut assets = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".asset") {
                    let uuid_str = filename.trim_end_matches(".asset");
                    if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                        assets.push(uuid);
                    }
                }
            }
        }
        Ok(assets)
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        let assets = self.list_assets().await?;
        Ok(StorageStats {
            total_assets: assets.len() as u64,
            total_size_bytes: 0, // Would calculate actual size
            backend_type: StorageBackendType::FileSystem,
            last_backup: None,
            available_space_bytes: None,
        })
    }

    async fn load(&self, storage_path: &str) -> Result<AssetData> {
        let data = tokio::fs::read(storage_path).await?;
        Ok(data)
    }

    async fn store(&self, asset_id: &Uuid, data: &AssetData) -> Result<String> {
        let path = self.base_path.join(format!("{}.asset", asset_id));
        tokio::fs::write(&path, data).await?;
        Ok(path.to_string_lossy().to_string())
    }
}
