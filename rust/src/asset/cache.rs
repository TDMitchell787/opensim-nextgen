//! Asset caching layer with Redis integration
//!
//! Provides multi-tier caching for assets including:
//! - In-memory LRU cache for frequently accessed assets
//! - Redis distributed cache for shared assets across instances
//! - Cache invalidation and TTL management
//! - Compression and serialization optimization

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Custom serialization for Bytes
mod bytes_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use bytes::Bytes;

    pub fn serialize<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        bytes.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        Ok(Bytes::from(vec))
    }
}
use tracing::{debug, error, info, warn};

#[cfg(feature = "redis-cache")]
use redis::{AsyncCommands, Client as RedisClient};

use crate::asset::AssetType;

/// Configuration for asset caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of assets to keep in memory cache
    pub memory_cache_size: usize,
    /// TTL for assets in memory cache (seconds)
    pub memory_ttl_seconds: u64,
    /// TTL for assets in Redis cache (seconds)
    pub redis_ttl_seconds: u64,
    /// Redis connection URL
    pub redis_url: Option<String>,
    /// Enable compression for cached assets
    pub enable_compression: bool,
    /// Minimum asset size for compression (bytes)
    pub compression_threshold: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            memory_cache_size: 1000,
            memory_ttl_seconds: 300, // 5 minutes
            redis_ttl_seconds: 3600, // 1 hour
            redis_url: std::env::var("REDIS_URL").ok(),
            enable_compression: true,
            compression_threshold: 1024, // 1KB
        }
    }
}

/// Cached asset entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAsset {
    pub id: String,
    #[serde(with = "bytes_serde")]
    pub data: Bytes,
    pub asset_type: AssetType,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
    pub compressed: bool,
}

impl CachedAsset {
    pub fn new(id: String, data: Bytes, asset_type: AssetType) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id,
            data,
            asset_type,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            compressed: false,
        }
    }
    
    pub fn update_access(&mut self) {
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.access_count += 1;
    }
    
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.created_at > ttl_seconds
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub memory_hits: u64,
    pub memory_misses: u64,
    pub redis_hits: u64,
    pub redis_misses: u64,
    pub memory_size: usize,
    pub redis_size: Option<usize>,
    pub compression_ratio: f64,
}

/// Multi-tier asset cache
pub struct AssetCache {
    config: CacheConfig,
    memory_cache: Arc<RwLock<LruCache<String, CachedAsset>>>,
    #[cfg(feature = "redis-cache")]
    redis_client: Option<RedisClient>,
    stats: Arc<RwLock<CacheStats>>,
}

impl AssetCache {
    /// Create a new asset cache with the given configuration
    pub async fn new(config: CacheConfig) -> Result<Self> {
        let memory_cache = Arc::new(RwLock::new(
            LruCache::new(std::num::NonZeroUsize::new(config.memory_cache_size).unwrap())
        ));
        
        #[cfg(feature = "redis-cache")]
        let redis_client = if let Some(ref redis_url) = config.redis_url {
            match RedisClient::open(redis_url.as_str()) {
                Ok(client) => {
                    // Test connection
                    match client.get_async_connection().await {
                        Ok(mut conn) => {
                            // Test with a simple command instead of ping
                            let _: Result<(), _> = redis::cmd("PING").query_async(&mut conn).await;
                            info!("Redis cache connected successfully");
                            Some(client)
                        }
                        Err(e) => {
                            warn!("Failed to connect to Redis: {}, continuing without Redis cache", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create Redis client: {}, continuing without Redis cache", e);
                    None
                }
            }
        } else {
            None
        };
        
        let stats = Arc::new(RwLock::new(CacheStats {
            memory_hits: 0,
            memory_misses: 0,
            redis_hits: 0,
            redis_misses: 0,
            memory_size: 0,
            redis_size: None,
            compression_ratio: 1.0,
        }));
        
        Ok(Self {
            config,
            memory_cache,
            #[cfg(feature = "redis-cache")]
            redis_client,
            stats,
        })
    }
    
    /// Get an asset from cache (checks memory first, then Redis)
    pub async fn get(&self, asset_id: &str) -> Result<Option<CachedAsset>> {
        // First check memory cache
        if let Some(mut asset) = self.memory_cache.write().get_mut(asset_id) {
            if !asset.is_expired(self.config.memory_ttl_seconds) {
                asset.update_access();
                self.stats.write().memory_hits += 1;
                debug!("Asset {} found in memory cache", asset_id);
                return Ok(Some(asset.clone()));
            } else {
                // Remove expired asset
                self.memory_cache.write().pop(asset_id);
            }
        }
        
        self.stats.write().memory_misses += 1;
        
        // Check Redis cache
        #[cfg(feature = "redis-cache")]
        if let Some(ref redis_client) = self.redis_client {
            match self.get_from_redis(redis_client, asset_id).await {
                Ok(Some(asset)) => {
                    self.stats.write().redis_hits += 1;
                    // Promote to memory cache
                    self.memory_cache.write().put(asset_id.to_string(), asset.clone());
                    debug!("Asset {} found in Redis cache and promoted to memory", asset_id);
                    return Ok(Some(asset));
                }
                Ok(None) => {
                    self.stats.write().redis_misses += 1;
                }
                Err(e) => {
                    error!("Redis cache error for asset {}: {}", asset_id, e);
                    self.stats.write().redis_misses += 1;
                }
            }
        }
        
        debug!("Asset {} not found in any cache", asset_id);
        Ok(None)
    }
    
    /// Store an asset in cache (memory and Redis)
    pub async fn put(&self, asset_id: &str, data: Bytes, asset_type: AssetType) -> Result<()> {
        let mut cached_asset = CachedAsset::new(asset_id.to_string(), data, asset_type);
        
        // Apply compression if enabled and asset is large enough
        if self.config.enable_compression && cached_asset.data.len() >= self.config.compression_threshold {
            cached_asset = self.compress_asset(cached_asset)?;
        }
        
        // Store in memory cache
        self.memory_cache.write().put(asset_id.to_string(), cached_asset.clone());
        
        // Store in Redis cache
        #[cfg(feature = "redis-cache")]
        if let Some(ref redis_client) = self.redis_client {
            if let Err(e) = self.put_to_redis(redis_client, &cached_asset).await {
                error!("Failed to store asset {} in Redis: {}", asset_id, e);
            } else {
                debug!("Asset {} stored in Redis cache", asset_id);
            }
        }
        
        self.update_stats();
        info!("Asset {} cached successfully", asset_id);
        Ok(())
    }
    
    /// Remove an asset from all cache tiers
    pub async fn remove(&self, asset_id: &str) -> Result<()> {
        // Remove from memory cache
        self.memory_cache.write().pop(asset_id);
        
        // Remove from Redis cache
        #[cfg(feature = "redis-cache")]
        if let Some(ref redis_client) = self.redis_client {
            if let Err(e) = self.remove_from_redis(redis_client, asset_id).await {
                error!("Failed to remove asset {} from Redis: {}", asset_id, e);
            }
        }
        
        info!("Asset {} removed from cache", asset_id);
        Ok(())
    }

    /// Set an asset in cache (alias for put, accepting Arc<Asset>)
    pub async fn set(&self, asset_id: &str, asset: Arc<crate::asset::Asset>) -> Result<()> {
        self.put(asset_id, (**asset.data.as_ref().unwrap()).clone(), asset.asset_type.clone()).await
    }

    /// Invalidate (remove) an asset from cache (alias for remove)
    pub async fn invalidate(&self, asset_id: &str) -> Result<()> {
        self.remove(asset_id).await
    }

    /// Flush the cache (alias for clear)
    pub async fn flush(&self) -> Result<()> {
        self.clear().await
    }

    /// Get cache statistics with different return type for compatibility
    pub async fn get_statistics(&self) -> Result<CacheStatistics> {
        let stats = self.get_stats();
        Ok(CacheStatistics {
            hit_ratio: if stats.memory_hits + stats.memory_misses > 0 {
                stats.memory_hits as f64 / (stats.memory_hits + stats.memory_misses) as f64
            } else {
                0.0
            },
            size: stats.memory_size,
        })
    }
    
    /// Clear all cached assets
    pub async fn clear(&self) -> Result<()> {
        // Clear memory cache
        self.memory_cache.write().clear();
        
        // Clear Redis cache
        #[cfg(feature = "redis-cache")]
        if let Some(ref redis_client) = self.redis_client {
            if let Err(e) = self.clear_redis(redis_client).await {
                error!("Failed to clear Redis cache: {}", e);
            }
        }
        
        // Reset stats
        *self.stats.write() = CacheStats {
            memory_hits: 0,
            memory_misses: 0,
            redis_hits: 0,
            redis_misses: 0,
            memory_size: 0,
            redis_size: None,
            compression_ratio: 1.0,
        };
        
        info!("All caches cleared");
        Ok(())
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().clone()
    }
    
    /// Update cache statistics
    fn update_stats(&self) {
        let mut stats = self.stats.write();
        stats.memory_size = self.memory_cache.read().len();
    }
    
    /// Compress an asset using lightweight compression
    fn compress_asset(&self, mut asset: CachedAsset) -> Result<CachedAsset> {
        // Simple compression using deflate
        use std::io::Write;
        let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&asset.data)?;
        let compressed = encoder.finish()?;
        
        let original_size = asset.data.len();
        let compressed_size = compressed.len();
        
        // Only use compression if it actually reduces size
        if compressed_size < original_size {
            asset.data = Bytes::from(compressed);
            asset.compressed = true;
            
            // Update compression ratio in stats
            let ratio = compressed_size as f64 / original_size as f64;
            self.stats.write().compression_ratio = ratio;
            
            debug!("Asset {} compressed from {} to {} bytes (ratio: {:.2})", 
                asset.id, original_size, compressed_size, ratio);
        }
        
        Ok(asset)
    }
    
    /// Decompress an asset if it's compressed
    pub fn decompress_asset(&self, mut asset: CachedAsset) -> Result<CachedAsset> {
        if asset.compressed {
            use std::io::Read;
            let mut decoder = flate2::read::DeflateDecoder::new(&asset.data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            
            asset.data = Bytes::from(decompressed);
            asset.compressed = false;
            
            debug!("Asset {} decompressed", asset.id);
        }
        
        Ok(asset)
    }
    
    #[cfg(feature = "redis-cache")]
    async fn get_from_redis(&self, redis_client: &RedisClient, asset_id: &str) -> Result<Option<CachedAsset>> {
        let mut conn = redis_client.get_async_connection().await?;
        let key = format!("asset:{}", asset_id);
        
        let data: Option<Vec<u8>> = conn.get(&key).await?;
        if let Some(data) = data {
            let asset: CachedAsset = bincode::deserialize(&data)
                .map_err(|e| anyhow!("Failed to deserialize cached asset: {}", e))?;
            
            if !asset.is_expired(self.config.redis_ttl_seconds) {
                Ok(Some(asset))
            } else {
                // Remove expired asset
                let _: () = conn.del(&key).await?;
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    #[cfg(feature = "redis-cache")]
    async fn put_to_redis(&self, redis_client: &RedisClient, asset: &CachedAsset) -> Result<()> {
        let mut conn = redis_client.get_async_connection().await?;
        let key = format!("asset:{}", asset.id);
        
        let data = bincode::serialize(asset)
            .map_err(|e| anyhow!("Failed to serialize asset for Redis: {}", e))?;
        
        let _: () = conn.set_ex(&key, data, self.config.redis_ttl_seconds).await?;
        Ok(())
    }
    
    #[cfg(feature = "redis-cache")]
    async fn remove_from_redis(&self, redis_client: &RedisClient, asset_id: &str) -> Result<()> {
        let mut conn = redis_client.get_async_connection().await?;
        let key = format!("asset:{}", asset_id);
        let _: () = conn.del(&key).await?;
        Ok(())
    }
    
    #[cfg(feature = "redis-cache")]
    async fn clear_redis(&self, redis_client: &RedisClient) -> Result<()> {
        let mut conn = redis_client.get_async_connection().await?;
        let keys: Vec<String> = conn.keys("asset:*").await?;
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }
        Ok(())
    }
}

/// Cache statistics for compatibility
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub hit_ratio: f64,
    pub size: usize,
} 