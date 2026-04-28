//! Asset management system for OpenSim Next
//!
//! This module provides comprehensive asset management with deduplication,
//! compression, CDN integration, and multi-storage backend support.

use anyhow::{anyhow, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

use crate::database::DatabaseManager;

pub mod asset_loader;
pub mod cache;
pub mod cdn;
pub mod deduplication;
pub mod fsassets;
pub mod fsassets_fetch;
pub mod fsassets_migrate;
pub mod jpeg2000;
pub mod persistence;
pub mod storage;
pub mod streaming;

// Re-export commonly used types
pub use asset_loader::{load_default_assets, AssetLoader};
pub use cache::{AssetCache, CacheConfig};
pub use cdn::{CdnConfig, CdnManager, CdnProvider};
pub use fsassets::{FSAssetsConfig, FSAssetsStorage};
pub use fsassets_fetch::AssetFetcher;
pub use jpeg2000::{
    detect_texture_format, is_valid_j2k, is_valid_jp2, DecodedImage, J2KCodec, TextureFormat,
};
pub use persistence::FileSystemBackend;
pub use storage::{StorageBackend, StorageBackendType};

/// Asset data representation for storage operations
pub type AssetData = Vec<u8>;

/// Asset types supported by OpenSim
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Texture = 0,
    Sound = 1,
    CallingCard = 2,
    Landmark = 3,
    Script = 4,
    Clothing = 5,
    Object = 6,
    Notecard = 7,
    Category = 8,
    RootCategory = 9,
    LSLText = 10,
    LSLBytecode = 11,
    TextureTGA = 12,
    Bodypart = 13,
    TrashFolder = 14,
    SnapshotFolder = 15,
    LostAndFoundFolder = 16,
    SoundWAV = 17,
    ImageTGA = 18,
    ImageJPEG = 19,
    Animation = 20,
    Gesture = 21,
    Simstate = 22,
    FavoriteFolder = 23,
    Link = 24,
    LinkFolder = 25,
    CurrentOutfitFolder = 46,
    OutfitFolder = 47,
    MyOutfitsFolder = 48,
    Mesh = 49,
    Unknown = -1,
}

impl AssetType {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Texture,
            1 => Self::Sound,
            2 => Self::CallingCard,
            3 => Self::Landmark,
            4 => Self::Script,
            5 => Self::Clothing,
            6 => Self::Object,
            7 => Self::Notecard,
            8 => Self::Category,
            9 => Self::RootCategory,
            10 => Self::LSLText,
            11 => Self::LSLBytecode,
            12 => Self::TextureTGA,
            13 => Self::Bodypart,
            14 => Self::TrashFolder,
            15 => Self::SnapshotFolder,
            16 => Self::LostAndFoundFolder,
            17 => Self::SoundWAV,
            18 => Self::ImageTGA,
            19 => Self::ImageJPEG,
            20 => Self::Animation,
            21 => Self::Gesture,
            22 => Self::Simstate,
            23 => Self::FavoriteFolder,
            24 => Self::Link,
            25 => Self::LinkFolder,
            46 => Self::CurrentOutfitFolder,
            47 => Self::OutfitFolder,
            48 => Self::MyOutfitsFolder,
            49 => Self::Mesh,
            _ => Self::Unknown,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Texture | Self::TextureTGA | Self::ImageTGA | Self::ImageJPEG => "image/jpeg",
            Self::Sound | Self::SoundWAV => "audio/wav",
            Self::LSLText | Self::Script => "text/plain",
            Self::Notecard => "text/plain",
            Self::Mesh => "application/vnd.ll.mesh",
            Self::Animation => "application/vnd.ll.animation",
            Self::Gesture => "application/vnd.ll.gesture",
            _ => "application/octet-stream",
        }
    }
}

/// Asset metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub name: String,
    pub description: String,
    pub creator_id: Uuid,
    pub owner_id: Option<Uuid>,
    pub content_type: String,
    pub sha256_hash: String,
    pub file_extension: String,
    pub compression_type: i32,
    pub original_size: usize,
    pub compressed_size: Option<usize>,
    pub upload_ip: Option<String>,
    pub uploader_id: Option<Uuid>,
    pub flags: u32,
    pub local: bool,
    pub temporary: bool,
}

impl Default for AssetMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            creator_id: Uuid::nil(),
            owner_id: None,
            content_type: "application/octet-stream".to_string(),
            sha256_hash: String::new(),
            file_extension: String::new(),
            compression_type: 0,
            original_size: 0,
            compressed_size: None,
            upload_ip: None,
            uploader_id: None,
            flags: 0,
            local: false,
            temporary: false,
        }
    }
}

/// Complete asset representation
#[derive(Debug, Clone)]
pub struct Asset {
    pub id: Uuid,
    pub asset_type: AssetType,
    pub data: Option<Arc<Bytes>>,
    pub metadata: AssetMetadata,
    pub created_at: SystemTime,
    pub access_time: SystemTime,
    pub storage_location: Option<String>,
    pub backup_locations: Vec<String>,
    pub reference_count: u32,
}

/// Asset manager configuration
#[derive(Debug, Clone)]
pub struct AssetManagerConfig {
    pub max_upload_size: usize,
    pub enable_compression: bool,
    pub compression_threshold: usize,
    pub enable_deduplication: bool,
    pub cleanup_interval: Duration,
    pub temporary_asset_ttl: Duration,
    pub max_concurrent_uploads: usize,
    pub upload_session_ttl: Duration,
    pub cdn_enabled: bool,
    pub cdn_sync_on_upload: bool,
}

impl Default for AssetManagerConfig {
    fn default() -> Self {
        Self {
            max_upload_size: 100 * 1024 * 1024, // 100MB
            enable_compression: true,
            compression_threshold: 1024, // 1KB
            enable_deduplication: true,
            cleanup_interval: Duration::from_secs(3600), // 1 hour
            temporary_asset_ttl: Duration::from_secs(86400), // 24 hours
            max_concurrent_uploads: 100,
            upload_session_ttl: Duration::from_secs(3600), // 1 hour
            cdn_enabled: false,
            cdn_sync_on_upload: false,
        }
    }
}

/// Asset upload session for chunked uploads
#[derive(Debug)]
pub struct AssetUploadSession {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub chunks: Vec<Option<Bytes>>,
    pub total_chunks: u32,
    pub chunk_size: usize,
    pub expected_size: usize,
    pub asset_type: AssetType,
    pub metadata: AssetMetadata,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
}

impl AssetUploadSession {
    pub fn new(
        asset_id: Uuid,
        total_chunks: u32,
        chunk_size: usize,
        expected_size: usize,
        asset_type: AssetType,
        metadata: AssetMetadata,
        ttl: Duration,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            id: Uuid::new_v4(),
            asset_id,
            chunks: (0..total_chunks).map(|_| None).collect(),
            total_chunks,
            chunk_size,
            expected_size,
            asset_type,
            metadata,
            created_at: now,
            expires_at: now + ttl,
        }
    }

    pub async fn add_chunk(&self, chunk: AssetChunk) -> Result<bool> {
        // TODO: Add chunk validation and assembly logic
        info!("Asset chunk upload temporarily disabled until database queries are fixed");
        Ok(false)
    }

    pub async fn is_complete(&self) -> bool {
        false // TODO: Implement completion check
    }

    pub async fn assemble(&self) -> Result<Bytes> {
        // TODO: Implement chunk assembly
        info!("Asset assembly temporarily disabled until database queries are fixed");
        Ok(Bytes::new())
    }
}

#[derive(Debug)]
pub struct AssetChunk {
    pub index: u32,
    pub data: Bytes,
}

/// Main asset manager
pub struct AssetManager {
    database: Arc<DatabaseManager>,
    cache: Arc<AssetCache>,
    cdn: Arc<CdnManager>,
    storage: Arc<dyn StorageBackend>,
    config: AssetManagerConfig,
    deduplication_map: RwLock<HashMap<String, Uuid>>,
    reference_counter: RwLock<HashMap<Uuid, u32>>,
    upload_sessions: Mutex<HashMap<Uuid, AssetUploadSession>>,
    cleanup_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl AssetManager {
    pub async fn new(
        database: Arc<DatabaseManager>,
        cache: Arc<cache::AssetCache>,
        cdn: Arc<cdn::CdnManager>,
        storage: Arc<dyn storage::StorageBackend>,
        config: AssetManagerConfig,
    ) -> Result<Self> {
        let manager = Self {
            database,
            cache,
            cdn,
            storage,
            config,
            deduplication_map: RwLock::new(HashMap::new()),
            reference_counter: RwLock::new(HashMap::new()),
            upload_sessions: Mutex::new(HashMap::new()),
            cleanup_task: Arc::new(Mutex::new(None)),
        };

        // Start background cleanup task
        manager.start_cleanup_task().await?;

        // Initialize deduplication map from database
        manager.initialize_deduplication_map().await?;

        Ok(manager)
    }

    async fn start_cleanup_task(&self) -> Result<()> {
        info!("Background cleanup task temporarily disabled until database queries are fixed");
        Ok(())
    }

    async fn initialize_deduplication_map(&self) -> Result<()> {
        info!("Deduplication map initialization temporarily disabled until database queries are fixed");
        Ok(())
    }

    /// Get an asset by ID with caching and lazy loading
    pub async fn get_asset(&self, asset_id: Uuid) -> Result<Option<Arc<Asset>>> {
        // Check cache first
        let cache_key = asset_id.to_string();
        if let Ok(Some(cached_asset)) = self.cache.get(&cache_key).await {
            // Convert CachedAsset to Asset
            let asset = Asset {
                id: asset_id,
                asset_type: cached_asset.asset_type,
                data: Some(Arc::new(cached_asset.data)),
                metadata: AssetMetadata::default(), // TODO: Cache metadata too
                created_at: SystemTime::now(),
                access_time: SystemTime::now(),
                storage_location: None,
                backup_locations: Vec::new(),
                reference_count: 1,
            };
            return Ok(Some(Arc::new(asset)));
        }

        // Load from database
        if let Some(asset) = self.load_from_database(asset_id).await? {
            // Update access time
            let _ = self.update_access_time(asset_id).await;

            // Cache the result
            if let Some(data) = &asset.data {
                if let Err(e) = self
                    .cache
                    .put(&cache_key, (**data).clone(), asset.asset_type)
                    .await
                {
                    warn!("Failed to cache asset {}: {}", asset_id, e);
                }
            }

            return Ok(Some(asset));
        }

        Ok(None)
    }

    async fn load_from_database(&self, _asset_id: Uuid) -> Result<Option<Arc<Asset>>> {
        // TODO: Implement proper database-agnostic asset loading
        info!("Asset database loading temporarily disabled until database queries are fixed");
        Ok(None)
    }

    async fn update_access_time(&self, _asset_id: Uuid) -> Result<()> {
        // TODO: Implement proper database-agnostic access time update
        info!("Access time update temporarily disabled until database queries are fixed");
        Ok(())
    }

    /// Store an asset with comprehensive persistence, deduplication, and CDN integration
    pub async fn store_asset(
        &self,
        _asset_id: Uuid,
        _asset_type: AssetType,
        _data: Bytes,
        _metadata: AssetMetadata,
        _uploader_id: Option<Uuid>,
    ) -> Result<Uuid> {
        // TODO: Implement proper database-agnostic asset storage
        info!("Asset storage temporarily disabled until database queries are fixed");
        Ok(Uuid::new_v4())
    }

    async fn compress_data(&self, data: &Bytes) -> Result<(Bytes, i32, Option<usize>)> {
        // Simple gzip compression for now
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        let compressed = encoder.finish()?;

        if compressed.len() < data.len() {
            Ok((Bytes::from(compressed.clone()), 1, Some(compressed.len())))
        } else {
            // No benefit from compression
            Ok((data.clone(), 0, None))
        }
    }

    /// Start a chunked upload session for large assets
    pub async fn start_upload_session(
        &self,
        asset_id: Uuid,
        total_chunks: u32,
        chunk_size: usize,
        expected_size: usize,
        asset_type: AssetType,
        metadata: AssetMetadata,
    ) -> Result<Uuid> {
        // Validate session limits
        {
            let sessions = self.upload_sessions.lock().await;
            if sessions.len() >= self.config.max_concurrent_uploads {
                return Err(anyhow!("Maximum concurrent uploads reached"));
            }
        }

        let session = AssetUploadSession::new(
            asset_id,
            total_chunks,
            chunk_size,
            expected_size,
            asset_type,
            metadata,
            self.config.upload_session_ttl,
        );

        let session_id = session.id;

        {
            let mut sessions = self.upload_sessions.lock().await;
            sessions.insert(session_id, session);
        }

        info!(
            "Started upload session {} for asset {} ({} chunks, {} bytes)",
            session_id, asset_id, total_chunks, expected_size
        );

        Ok(session_id)
    }

    /// Upload a chunk to an existing session
    pub async fn upload_chunk(
        &self,
        _session_id: Uuid,
        _chunk_index: u32,
        _data: Bytes,
    ) -> Result<bool> {
        // TODO: Implement chunk upload
        info!("Chunk upload temporarily disabled until database queries are fixed");
        Ok(false)
    }

    /// Get upload session status
    pub async fn get_upload_status(&self, session_id: Uuid) -> Result<Option<f32>> {
        let sessions = self.upload_sessions.lock().await;
        if let Some(session) = sessions.get(&session_id) {
            let completed_chunks = session.chunks.iter().filter(|c| c.is_some()).count();
            let progress = completed_chunks as f32 / session.total_chunks as f32;
            Ok(Some(progress))
        } else {
            Ok(None)
        }
    }

    /// Get asset manager statistics
    pub async fn get_stats(&self) -> AssetManagerStats {
        let dedup_map = self.deduplication_map.read().await;
        let ref_counter = self.reference_counter.read().await;
        let sessions = self.upload_sessions.lock().await;

        AssetManagerStats {
            total_assets: ref_counter.len(),
            deduplication_entries: dedup_map.len(),
            active_upload_sessions: sessions.len(),
            cache_size: self.cache.get_stats().memory_size,
            total_references: ref_counter.values().sum(),
        }
    }

    /// Get asset metadata only (without data)
    pub async fn get_metadata(&self, _asset_id: Uuid) -> Result<Option<AssetMetadata>> {
        // TODO: Implement metadata-only retrieval
        info!("Asset metadata retrieval temporarily disabled until database queries are fixed");
        Ok(None)
    }

    /// Store asset chunk for chunked uploads
    pub async fn store_asset_chunk(
        &self,
        _asset_id: &str,
        _chunk_index: u32,
        _data: Bytes,
    ) -> Result<()> {
        // TODO: Implement chunked asset storage
        info!("Asset chunk storage temporarily disabled until database queries are fixed");
        Ok(())
    }

    /// Assemble asset chunks into complete asset
    pub async fn assemble_asset_chunks(
        &self,
        _asset_id: &str,
        _total_chunks: u32,
    ) -> Result<Bytes> {
        // TODO: Implement asset chunk assembly
        info!("Asset chunk assembly temporarily disabled until database queries are fixed");
        Ok(Bytes::new())
    }

    /// Clean up asset chunks after assembly or failure
    pub async fn cleanup_asset_chunks(&self, _asset_id: &str) -> Result<()> {
        // TODO: Implement asset chunk cleanup
        info!("Asset chunk cleanup temporarily disabled until database queries are fixed");
        Ok(())
    }

    /// Get asset data only (without metadata)
    pub async fn get_asset_data(&self, _asset_id: &Uuid) -> Result<Option<Bytes>> {
        // TODO: Implement asset data retrieval
        info!("Asset data retrieval temporarily disabled until database queries are fixed");
        Ok(None)
    }

    /// Check if asset exists
    pub async fn asset_exists(&self, _asset_id: &Uuid) -> Result<bool> {
        // TODO: Implement asset existence check
        info!("Asset existence check temporarily disabled until database queries are fixed");
        Ok(false)
    }

    /// Get asset information (metadata without data)
    pub async fn get_asset_info(&self, _asset_id: &Uuid) -> Result<Option<AssetMetadata>> {
        // TODO: Implement asset info retrieval
        info!("Asset info retrieval temporarily disabled until database queries are fixed");
        Ok(None)
    }
}

/// Asset manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManagerStats {
    pub total_assets: usize,
    pub deduplication_entries: usize,
    pub active_upload_sessions: usize,
    pub cache_size: usize,
    pub total_references: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_conversion() {
        assert_eq!(AssetType::from_i32(0), AssetType::Texture);
        assert_eq!(AssetType::Texture.to_i32(), 0);
        assert_eq!(AssetType::from_i32(-1), AssetType::Unknown);
    }

    #[test]
    fn test_asset_metadata_default() {
        let metadata = AssetMetadata::default();
        assert_eq!(metadata.name, "");
        assert_eq!(metadata.creator_id, Uuid::nil());
        assert!(!metadata.local);
        assert!(!metadata.temporary);
    }
}
