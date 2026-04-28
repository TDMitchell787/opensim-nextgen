//! OpenSim-compatible asset cache system
//!
//! Provides disk-based asset caching compatible with OpenSimulator's
//! assetcache directory structure and Flotsam cache format.

use anyhow::Result;
use hex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// OpenSim-compatible asset cache manager
pub struct AssetCacheManager {
    cache_directory: PathBuf,
    config: AssetCacheConfig,
    cache_buckets: Vec<PathBuf>,
    memory_cache: HashMap<String, CachedAsset>,
    cache_stats: CacheStatistics,
}

/// Asset cache configuration
#[derive(Debug, Clone)]
pub struct AssetCacheConfig {
    pub cache_directory: PathBuf,
    pub cache_buckets: u32,
    pub cache_timeout_hours: u32,
    pub memory_cache_size: usize,
    pub disk_cache_size_mb: u64,
    pub enable_memory_cache: bool,
    pub enable_disk_cache: bool,
    pub cleanup_interval_hours: u32,
    pub log_cache_hits: bool,
}

/// Cached asset data
#[derive(Debug, Clone)]
pub struct CachedAsset {
    pub asset_id: String,
    pub asset_type: i32,
    pub data: Vec<u8>,
    pub cached_time: u64,
    pub access_count: u32,
    pub last_access: u64,
    pub file_path: Option<PathBuf>,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    pub memory_hits: u64,
    pub disk_hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub disk_writes: u64,
    pub cleanup_runs: u64,
    pub total_requests: u64,
}

impl AssetCacheManager {
    /// Create a new asset cache manager
    pub fn new(config: AssetCacheConfig) -> Result<Self> {
        let mut cache_buckets = Vec::new();

        // Create cache directory structure (compatible with Flotsam cache)
        fs::create_dir_all(&config.cache_directory)?;

        // Create cache buckets (00 to FF for 256 buckets, or configurable)
        for i in 0..config.cache_buckets {
            let bucket_name = format!("{:02x}", i);
            let bucket_path = config.cache_directory.join(&bucket_name);
            fs::create_dir_all(&bucket_path)?;
            cache_buckets.push(bucket_path);
        }

        tracing::info!(
            "Initialized asset cache with {} buckets at: {}",
            config.cache_buckets,
            config.cache_directory.display()
        );

        Ok(Self {
            cache_directory: config.cache_directory.clone(),
            config,
            cache_buckets,
            memory_cache: HashMap::new(),
            cache_stats: CacheStatistics::default(),
        })
    }

    /// Get asset from cache
    pub async fn get_asset(&mut self, asset_id: &str) -> Option<Vec<u8>> {
        self.cache_stats.total_requests += 1;

        // Check memory cache first
        if self.config.enable_memory_cache {
            if let Some(cached) = self.memory_cache.get_mut(asset_id) {
                cached.access_count += 1;
                cached.last_access = Self::current_timestamp();
                self.cache_stats.memory_hits += 1;

                if self.config.log_cache_hits {
                    tracing::debug!("Memory cache hit for asset: {}", asset_id);
                }

                return Some(cached.data.clone());
            }
        }

        // Check disk cache
        if self.config.enable_disk_cache {
            if let Some(data) = self.load_from_disk(asset_id).await {
                self.cache_stats.disk_hits += 1;

                if self.config.log_cache_hits {
                    tracing::debug!("Disk cache hit for asset: {}", asset_id);
                }

                // Add to memory cache if enabled
                if self.config.enable_memory_cache {
                    self.add_to_memory_cache(asset_id, 0, data.clone());
                }

                return Some(data);
            }
        }

        self.cache_stats.misses += 1;
        None
    }

    /// Store asset in cache
    pub async fn store_asset(
        &mut self,
        asset_id: &str,
        asset_type: i32,
        data: Vec<u8>,
    ) -> Result<()> {
        // Store in memory cache
        if self.config.enable_memory_cache {
            self.add_to_memory_cache(asset_id, asset_type, data.clone());
        }

        // Store in disk cache
        if self.config.enable_disk_cache {
            self.save_to_disk(asset_id, asset_type, &data).await?;
            self.cache_stats.disk_writes += 1;
        }

        Ok(())
    }

    /// Add asset to memory cache with LRU eviction
    fn add_to_memory_cache(&mut self, asset_id: &str, asset_type: i32, data: Vec<u8>) {
        // Check if we need to evict entries
        if self.memory_cache.len() >= self.config.memory_cache_size {
            self.evict_lru_memory_cache();
        }

        let cached_asset = CachedAsset {
            asset_id: asset_id.to_string(),
            asset_type,
            data,
            cached_time: Self::current_timestamp(),
            access_count: 1,
            last_access: Self::current_timestamp(),
            file_path: None,
        };

        self.memory_cache.insert(asset_id.to_string(), cached_asset);
    }

    /// Evict least recently used entry from memory cache
    fn evict_lru_memory_cache(&mut self) {
        if let Some((lru_key, _)) = self
            .memory_cache
            .iter()
            .min_by_key(|(_, asset)| asset.last_access)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.memory_cache.remove(&lru_key);
            self.cache_stats.evictions += 1;

            tracing::debug!("Evicted asset from memory cache: {}", lru_key);
        }
    }

    /// Load asset from disk cache
    async fn load_from_disk(&self, asset_id: &str) -> Option<Vec<u8>> {
        let file_path = self.get_cache_file_path(asset_id);

        if !file_path.exists() {
            return None;
        }

        // Check if file is expired
        if self.is_cache_file_expired(&file_path) {
            // Clean up expired file
            let _ = fs::remove_file(&file_path);
            return None;
        }

        match fs::read(&file_path) {
            Ok(data) => {
                // Update file access time
                let _ = Self::touch_file(&file_path);
                Some(data)
            }
            Err(e) => {
                tracing::warn!("Failed to read cached asset {}: {}", asset_id, e);
                None
            }
        }
    }

    /// Save asset to disk cache
    async fn save_to_disk(&self, asset_id: &str, _asset_type: i32, data: &[u8]) -> Result<()> {
        let file_path = self.get_cache_file_path(asset_id);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write asset data to file
        fs::write(&file_path, data)?;

        // Write metadata file (OpenSim Flotsam compatible)
        let metadata_path = file_path.with_extension("metadata");
        let metadata = format!("{}|{}|{}", asset_id, Self::current_timestamp(), data.len());
        fs::write(&metadata_path, metadata)?;

        tracing::debug!("Cached asset to disk: {} ({} bytes)", asset_id, data.len());
        Ok(())
    }

    /// Get cache file path for an asset (using bucket distribution)
    fn get_cache_file_path(&self, asset_id: &str) -> PathBuf {
        // Use first two characters of asset ID to determine bucket
        let bucket_index = if asset_id.len() >= 2 {
            u32::from_str_radix(&asset_id[..2], 16).unwrap_or(0) % self.config.cache_buckets
        } else {
            0
        };

        let bucket_path = &self.cache_buckets[bucket_index as usize];
        bucket_path.join(format!("{}.asset", asset_id))
    }

    /// Check if cache file is expired
    fn is_cache_file_expired(&self, file_path: &Path) -> bool {
        if let Ok(metadata) = fs::metadata(file_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    let age_hours = (Self::current_timestamp() - duration.as_secs()) / 3600;
                    return age_hours > self.config.cache_timeout_hours as u64;
                }
            }
        }
        false
    }

    /// Touch file to update access time
    fn touch_file(file_path: &Path) -> Result<()> {
        if let Ok(mut file) = fs::OpenOptions::new().write(true).open(file_path) {
            let _ = file.flush();
        }
        Ok(())
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Clean up expired cache entries
    pub async fn cleanup_cache(&mut self) -> Result<()> {
        let mut cleaned_files = 0;
        let mut cleaned_bytes = 0u64;

        for bucket_path in &self.cache_buckets {
            if let Ok(entries) = fs::read_dir(bucket_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();

                        if path.extension().and_then(|s| s.to_str()) == Some("asset") {
                            if self.is_cache_file_expired(&path) {
                                if let Ok(metadata) = fs::metadata(&path) {
                                    cleaned_bytes += metadata.len();
                                }

                                let _ = fs::remove_file(&path);

                                // Also remove metadata file
                                let metadata_path = path.with_extension("metadata");
                                let _ = fs::remove_file(&metadata_path);

                                cleaned_files += 1;
                            }
                        }
                    }
                }
            }
        }

        self.cache_stats.cleanup_runs += 1;

        tracing::info!(
            "Cache cleanup completed: {} files removed, {} bytes freed",
            cleaned_files,
            cleaned_bytes
        );
        Ok(())
    }

    /// Get cache statistics
    pub fn get_statistics(&self) -> CacheStatistics {
        self.cache_stats.clone()
    }

    /// Get cache size information
    pub async fn get_cache_size_info(&self) -> CacheSizeInfo {
        let mut total_files = 0;
        let mut total_size = 0u64;
        let mut oldest_file_time = Self::current_timestamp();
        let mut newest_file_time = 0u64;

        for bucket_path in &self.cache_buckets {
            if let Ok(entries) = fs::read_dir(bucket_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();

                        if path.extension().and_then(|s| s.to_str()) == Some("asset") {
                            if let Ok(metadata) = fs::metadata(&path) {
                                total_files += 1;
                                total_size += metadata.len();

                                if let Ok(modified) = metadata.modified() {
                                    if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                                        let timestamp = duration.as_secs();
                                        oldest_file_time = oldest_file_time.min(timestamp);
                                        newest_file_time = newest_file_time.max(timestamp);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        CacheSizeInfo {
            total_files,
            total_size_bytes: total_size,
            total_size_mb: total_size / (1024 * 1024),
            memory_cache_entries: self.memory_cache.len(),
            oldest_file_age_hours: (Self::current_timestamp() - oldest_file_time) / 3600,
            newest_file_age_hours: (Self::current_timestamp() - newest_file_time) / 3600,
        }
    }

    /// Check asset exists in cache
    pub fn has_asset(&self, asset_id: &str) -> bool {
        // Check memory cache
        if self.config.enable_memory_cache && self.memory_cache.contains_key(asset_id) {
            return true;
        }

        // Check disk cache
        if self.config.enable_disk_cache {
            let file_path = self.get_cache_file_path(asset_id);
            return file_path.exists() && !self.is_cache_file_expired(&file_path);
        }

        false
    }

    /// Clear all cached assets
    pub async fn clear_cache(&mut self) -> Result<()> {
        // Clear memory cache
        self.memory_cache.clear();

        // Clear disk cache
        for bucket_path in &self.cache_buckets {
            if let Ok(entries) = fs::read_dir(bucket_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }

        // Reset statistics
        self.cache_stats = CacheStatistics::default();

        tracing::info!("Asset cache cleared");
        Ok(())
    }

    /// Get hit ratio
    pub fn get_hit_ratio(&self) -> f64 {
        if self.cache_stats.total_requests == 0 {
            return 0.0;
        }

        let total_hits = self.cache_stats.memory_hits + self.cache_stats.disk_hits;
        total_hits as f64 / self.cache_stats.total_requests as f64
    }
}

/// Cache size information
#[derive(Debug, Clone)]
pub struct CacheSizeInfo {
    pub total_files: u32,
    pub total_size_bytes: u64,
    pub total_size_mb: u64,
    pub memory_cache_entries: usize,
    pub oldest_file_age_hours: u64,
    pub newest_file_age_hours: u64,
}

impl Default for AssetCacheConfig {
    fn default() -> Self {
        Self {
            cache_directory: PathBuf::from("./bin/assetcache"),
            cache_buckets: 256, // 00-FF buckets like Flotsam
            cache_timeout_hours: 48,
            memory_cache_size: 1000,
            disk_cache_size_mb: 1024,
            enable_memory_cache: true,
            enable_disk_cache: true,
            cleanup_interval_hours: 24,
            log_cache_hits: false,
        }
    }
}
