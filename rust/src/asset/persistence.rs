//! Asset Persistence Module
//!
//! Handles long-term storage and retrieval of assets with support for
//! multiple storage backends and persistence strategies.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::storage::{StorageBackend, StorageBackendType, StorageStats};
use super::{AssetData, AssetType};

/// Asset persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Primary storage backend type
    pub primary_backend: StorageBackendType,
    /// Secondary backup storage backend
    pub backup_backend: Option<StorageBackendType>,
    /// Enable automatic backup to secondary storage
    pub enable_backup: bool,
    /// Backup interval in seconds
    pub backup_interval_seconds: u64,
    /// Enable asset versioning
    pub enable_versioning: bool,
    /// Maximum versions to keep per asset
    pub max_versions: u32,
    /// Enable asset compression
    pub enable_compression: bool,
    /// Enable asset encryption at rest
    pub enable_encryption: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            primary_backend: StorageBackendType::FileSystem,
            backup_backend: Some(StorageBackendType::Database),
            enable_backup: true,
            backup_interval_seconds: 3600, // 1 hour
            enable_versioning: false,
            max_versions: 5,
            enable_compression: true,
            enable_encryption: false,
        }
    }
}

/// Asset version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetVersion {
    pub version: u32,
    pub created_at: std::time::SystemTime,
    pub size_bytes: u64,
    pub checksum: String,
    pub compressed: bool,
    pub encrypted: bool,
}

/// Asset persistence manager
pub struct AssetPersistenceManager {
    config: PersistenceConfig,
    primary_backend: Box<dyn StorageBackend>,
    backup_backend: Option<Box<dyn StorageBackend>>,
    asset_versions: Arc<RwLock<HashMap<Uuid, Vec<AssetVersion>>>>,
    persistence_stats: Arc<RwLock<PersistenceStats>>,
}

/// Persistence statistics
#[derive(Debug, Clone, Default)]
pub struct PersistenceStats {
    pub assets_stored: u64,
    pub assets_retrieved: u64,
    pub assets_deleted: u64,
    pub backup_operations: u64,
    pub compression_savings_bytes: u64,
    pub last_backup_time: Option<std::time::SystemTime>,
}

impl AssetPersistenceManager {
    /// Create a new asset persistence manager
    pub fn new(config: PersistenceConfig) -> Result<Self> {
        // For now, create a basic file system backend
        let primary_backend = Box::new(FileSystemBackend::new("./assets")?);

        let backup_backend = if config.enable_backup {
            match &config.backup_backend {
                Some(StorageBackendType::Database) => {
                    Some(Box::new(DatabaseBackend::new()?) as Box<dyn StorageBackend>)
                }
                Some(StorageBackendType::FileSystem) => {
                    Some(Box::new(FileSystemBackend::new("./assets_backup")?)
                        as Box<dyn StorageBackend>)
                }
                _ => None,
            }
        } else {
            None
        };

        Ok(Self {
            config,
            primary_backend,
            backup_backend,
            asset_versions: Arc::new(RwLock::new(HashMap::new())),
            persistence_stats: Arc::new(RwLock::new(PersistenceStats::default())),
        })
    }

    /// Store an asset with persistence and optional backup
    pub async fn store_asset(
        &self,
        asset_id: Uuid,
        asset_type: AssetType,
        data: AssetData,
    ) -> Result<()> {
        // Store in primary backend
        self.primary_backend
            .store_asset(&asset_id, asset_type.clone(), &data)
            .await?;

        // Store in backup backend if enabled
        if let Some(backup_backend) = &self.backup_backend {
            if let Err(e) = backup_backend
                .store_asset(&asset_id, asset_type.clone(), &data)
                .await
            {
                warn!(
                    "Failed to store asset {} in backup backend: {}",
                    asset_id, e
                );
            }
        }

        // Update versioning if enabled
        if self.config.enable_versioning {
            self.add_asset_version(asset_id, &data).await;
        }

        // Update statistics
        {
            let mut stats = self.persistence_stats.write().await;
            stats.assets_stored += 1;
        }

        info!("Asset {} stored successfully", asset_id);
        Ok(())
    }

    /// Retrieve an asset from storage
    pub async fn retrieve_asset(&self, asset_id: &Uuid) -> Result<Option<(AssetType, AssetData)>> {
        // Try primary backend first
        match self.primary_backend.retrieve_asset(asset_id).await {
            Ok(Some(asset)) => {
                // Update statistics
                {
                    let mut stats = self.persistence_stats.write().await;
                    stats.assets_retrieved += 1;
                }
                Ok(Some(asset))
            }
            Ok(None) => {
                // Try backup backend if available
                if let Some(backup_backend) = &self.backup_backend {
                    match backup_backend.retrieve_asset(asset_id).await {
                        Ok(Some(asset)) => {
                            info!("Asset {} recovered from backup backend", asset_id);
                            Ok(Some(asset))
                        }
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                error!("Failed to retrieve asset {}: {}", asset_id, e);
                Err(e)
            }
        }
    }

    /// Delete an asset from all storage backends
    pub async fn delete_asset(&self, asset_id: &Uuid) -> Result<bool> {
        let mut deleted = false;

        // Delete from primary backend
        if self
            .primary_backend
            .delete_asset(asset_id)
            .await
            .unwrap_or(false)
        {
            deleted = true;
        }

        // Delete from backup backend if available
        if let Some(backup_backend) = &self.backup_backend {
            backup_backend.delete_asset(asset_id).await.unwrap_or(false);
        }

        // Remove versioning information
        if self.config.enable_versioning {
            let mut versions = self.asset_versions.write().await;
            versions.remove(asset_id);
        }

        if deleted {
            // Update statistics
            let mut stats = self.persistence_stats.write().await;
            stats.assets_deleted += 1;

            info!("Asset {} deleted successfully", asset_id);
        }

        Ok(deleted)
    }

    /// Check if an asset exists in storage
    pub async fn asset_exists(&self, asset_id: &Uuid) -> Result<bool> {
        // Check primary backend first
        if self
            .primary_backend
            .asset_exists(asset_id)
            .await
            .unwrap_or(false)
        {
            return Ok(true);
        }

        // Check backup backend if available
        if let Some(backup_backend) = &self.backup_backend {
            return backup_backend.asset_exists(asset_id).await;
        }

        Ok(false)
    }

    /// Get asset versions (if versioning is enabled)
    pub async fn get_asset_versions(&self, asset_id: &Uuid) -> Vec<AssetVersion> {
        if !self.config.enable_versioning {
            return Vec::new();
        }

        let versions = self.asset_versions.read().await;
        versions.get(asset_id).cloned().unwrap_or_default()
    }

    /// Add a new asset version
    async fn add_asset_version(&self, asset_id: Uuid, data: &AssetData) {
        let version = AssetVersion {
            version: 1, // TODO: Implement proper version numbering
            created_at: std::time::SystemTime::now(),
            size_bytes: data.len() as u64,
            checksum: self.calculate_checksum(data),
            compressed: self.config.enable_compression,
            encrypted: self.config.enable_encryption,
        };

        let mut versions = self.asset_versions.write().await;
        let asset_versions = versions.entry(asset_id).or_insert_with(Vec::new);
        asset_versions.push(version);

        // Keep only the maximum number of versions
        if asset_versions.len() > self.config.max_versions as usize {
            asset_versions.remove(0);
        }
    }

    /// Calculate checksum for asset data
    fn calculate_checksum(&self, data: &AssetData) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Get persistence statistics
    pub async fn get_stats(&self) -> PersistenceStats {
        let stats = self.persistence_stats.read().await;
        stats.clone()
    }

    /// Perform backup operation
    pub async fn perform_backup(&self) -> Result<()> {
        if !self.config.enable_backup {
            return Ok(());
        }

        let backup_backend = match &self.backup_backend {
            Some(backend) => backend,
            None => return Ok(()),
        };

        info!("Starting asset backup operation");

        // Get list of all assets from primary backend
        let asset_ids = self.primary_backend.list_assets().await?;
        let mut backed_up = 0;

        for asset_id in asset_ids {
            // Retrieve from primary and store in backup
            if let Ok(Some((asset_type, data))) =
                self.primary_backend.retrieve_asset(&asset_id).await
            {
                if let Err(e) = backup_backend
                    .store_asset(&asset_id, asset_type, &data)
                    .await
                {
                    warn!("Failed to backup asset {}: {}", asset_id, e);
                } else {
                    backed_up += 1;
                }
            }
        }

        // Update statistics
        {
            let mut stats = self.persistence_stats.write().await;
            stats.backup_operations += 1;
            stats.last_backup_time = Some(std::time::SystemTime::now());
        }

        info!("Backup operation completed: {} assets backed up", backed_up);
        Ok(())
    }
}

/// Basic file system storage backend
pub struct FileSystemBackend {
    base_path: PathBuf,
}

impl FileSystemBackend {
    pub fn new(base_path: &str) -> Result<Self> {
        let path = PathBuf::from(base_path);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(Self { base_path: path })
    }
}

#[async_trait]
impl StorageBackend for FileSystemBackend {
    async fn store_asset(
        &self,
        asset_id: &Uuid,
        _asset_type: AssetType,
        data: &AssetData,
    ) -> Result<()> {
        let file_path = self.base_path.join(format!("{}.asset", asset_id));
        tokio::fs::write(file_path, data).await?;
        Ok(())
    }

    async fn retrieve_asset(&self, asset_id: &Uuid) -> Result<Option<(AssetType, AssetData)>> {
        let file_path = self.base_path.join(format!("{}.asset", asset_id));
        match tokio::fs::read(file_path).await {
            Ok(data) => Ok(Some((AssetType::Unknown, data))), // TODO: Store asset type
            Err(_) => Ok(None),
        }
    }

    async fn delete_asset(&self, asset_id: &Uuid) -> Result<bool> {
        let file_path = self.base_path.join(format!("{}.asset", asset_id));
        match tokio::fs::remove_file(file_path).await {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn asset_exists(&self, asset_id: &Uuid) -> Result<bool> {
        let file_path = self.base_path.join(format!("{}.asset", asset_id));
        Ok(file_path.exists())
    }

    async fn list_assets(&self) -> Result<Vec<Uuid>> {
        let mut assets = Vec::new();
        let mut dir = tokio::fs::read_dir(&self.base_path).await?;

        while let Some(entry) = dir.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".asset") {
                    let uuid_str = file_name.trim_end_matches(".asset");
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
        let mut total_size = 0u64;

        for asset_id in &assets {
            let file_path = self.base_path.join(format!("{}.asset", asset_id));
            if let Ok(metadata) = tokio::fs::metadata(file_path).await {
                total_size += metadata.len();
            }
        }

        Ok(StorageStats {
            total_assets: assets.len() as u64,
            total_size_bytes: total_size,
            backend_type: StorageBackendType::FileSystem,
            last_backup: None,
            available_space_bytes: None, // TODO: Calculate available disk space
        })
    }

    async fn load(&self, storage_path: &str) -> Result<AssetData> {
        let file_path = PathBuf::from(storage_path);
        let data = tokio::fs::read(file_path).await?;
        Ok(data)
    }

    async fn store(&self, asset_id: &Uuid, data: &AssetData) -> Result<String> {
        let file_path = self.base_path.join(format!("{}.asset", asset_id));
        tokio::fs::write(&file_path, data).await?;
        Ok(file_path.to_string_lossy().to_string())
    }
}

/// Basic database storage backend (placeholder)
pub struct DatabaseBackend;

impl DatabaseBackend {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl StorageBackend for DatabaseBackend {
    async fn store_asset(
        &self,
        _asset_id: &Uuid,
        _asset_type: AssetType,
        _data: &AssetData,
    ) -> Result<()> {
        // TODO: Implement database storage
        Ok(())
    }

    async fn retrieve_asset(&self, _asset_id: &Uuid) -> Result<Option<(AssetType, AssetData)>> {
        // TODO: Implement database retrieval
        Ok(None)
    }

    async fn delete_asset(&self, _asset_id: &Uuid) -> Result<bool> {
        // TODO: Implement database deletion
        Ok(false)
    }

    async fn asset_exists(&self, _asset_id: &Uuid) -> Result<bool> {
        // TODO: Implement database existence check
        Ok(false)
    }

    async fn list_assets(&self) -> Result<Vec<Uuid>> {
        // TODO: Implement database asset listing
        Ok(Vec::new())
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        Ok(StorageStats {
            total_assets: 0,
            total_size_bytes: 0,
            backend_type: StorageBackendType::Database,
            last_backup: None,
            available_space_bytes: None,
        })
    }

    async fn load(&self, _storage_path: &str) -> Result<AssetData> {
        // TODO: Implement database loading
        Ok(Vec::new())
    }

    async fn store(&self, _asset_id: &Uuid, _data: &AssetData) -> Result<String> {
        // TODO: Implement database storage
        Ok("database://placeholder".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_filesystem_backend() {
        let temp_dir = TempDir::new().unwrap();
        let backend = FileSystemBackend::new(temp_dir.path().to_str().unwrap()).unwrap();

        let asset_id = Uuid::new_v4();
        let asset_data = b"test asset data".to_vec();

        // Test store
        backend
            .store_asset(&asset_id, AssetType::Texture, &asset_data)
            .await
            .unwrap();

        // Test exists
        assert!(backend.asset_exists(&asset_id).await.unwrap());

        // Test retrieve
        let retrieved = backend.retrieve_asset(&asset_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().1, asset_data);

        // Test delete
        assert!(backend.delete_asset(&asset_id).await.unwrap());
        assert!(!backend.asset_exists(&asset_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_persistence_manager() {
        let config = PersistenceConfig::default();
        let manager = AssetPersistenceManager::new(config).unwrap();

        let asset_id = Uuid::new_v4();
        let asset_data = b"test persistence data".to_vec();

        // Test store and retrieve
        manager
            .store_asset(asset_id, AssetType::Texture, asset_data.clone())
            .await
            .unwrap();
        let retrieved = manager.retrieve_asset(&asset_id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().1, asset_data);

        // Test statistics
        let stats = manager.get_stats().await;
        assert_eq!(stats.assets_stored, 1);
        assert_eq!(stats.assets_retrieved, 1);
    }
}
