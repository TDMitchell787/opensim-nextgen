//! Caching layer implementation with Redis and in-memory support

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Cache backend types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CacheBackend {
    InMemory,
    Redis,
    Distributed,
    Hybrid, // In-memory + Redis
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub access_count: u64,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub size_bytes: usize,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub hit_ratio: f64,
    pub average_access_time_ms: f64,
    pub backend_type: CacheBackend,
    pub last_cleanup: chrono::DateTime<chrono::Utc>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub backend: CacheBackend,
    pub max_entries: usize,
    pub max_size_bytes: usize,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
    pub redis_url: Option<String>,
    pub redis_key_prefix: String,
    pub enable_compression: bool,
    pub compression_threshold: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            backend: CacheBackend::InMemory,
            max_entries: 10000,
            max_size_bytes: 100 * 1024 * 1024,          // 100MB
            default_ttl: Duration::from_secs(3600),     // 1 hour
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            redis_url: None,
            redis_key_prefix: "opensim:".to_string(),
            enable_compression: true,
            compression_threshold: 1024, // 1KB
        }
    }
}

/// Generic cache interface
#[async_trait::async_trait]
pub trait CacheInterface<K, V>: Send + Sync
where
    K: Send + Sync + Clone + Eq + Hash,
    V: Send + Sync + Clone + Serialize + DeserializeOwned,
{
    async fn get(&self, key: &K) -> Result<Option<V>>;
    async fn set(&self, key: K, value: V, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &K) -> Result<bool>;
    async fn exists(&self, key: &K) -> Result<bool>;
    async fn clear(&self) -> Result<()>;
    async fn get_statistics(&self) -> Result<CacheStatistics>;
}

/// In-memory cache implementation
pub struct InMemoryCache<K, V>
where
    K: Send + Sync + Clone + Eq + Hash + 'static,
    V: Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    config: CacheConfig,
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    statistics: Arc<RwLock<CacheStatistics>>,
}

impl<K, V> InMemoryCache<K, V>
where
    K: Send + Sync + Clone + Eq + Hash + 'static,
    V: Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    pub fn new(config: CacheConfig) -> Self {
        let statistics = CacheStatistics {
            total_entries: 0,
            total_size_bytes: 0,
            hit_count: 0,
            miss_count: 0,
            eviction_count: 0,
            hit_ratio: 0.0,
            average_access_time_ms: 0.0,
            backend_type: config.backend.clone(),
            last_cleanup: chrono::Utc::now(),
        };

        let cache = Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(statistics)),
        };

        // Start cleanup task
        cache.start_cleanup_task();
        cache
    }

    fn start_cleanup_task(&self) {
        let entries = self.entries.clone();
        let statistics = self.statistics.clone();
        let cleanup_interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;

                let now = chrono::Utc::now();
                let mut entries_guard = entries.write().await;
                let mut stats_guard = statistics.write().await;

                let initial_count = entries_guard.len();

                // Remove expired entries
                entries_guard.retain(|_, entry| {
                    if let Some(expires_at) = entry.expires_at {
                        expires_at > now
                    } else {
                        true
                    }
                });

                let final_count = entries_guard.len();
                let evicted = initial_count - final_count;

                if evicted > 0 {
                    stats_guard.eviction_count += evicted as u64;
                    stats_guard.total_entries = final_count;
                    debug!("Cache cleanup: evicted {} expired entries", evicted);
                }

                stats_guard.last_cleanup = now;
            }
        });
    }

    fn estimate_size(&self, value: &V) -> usize {
        // Rough estimation - in practice, you'd use a more accurate method
        match serde_json::to_vec(value) {
            Ok(data) => data.len(),
            Err(_) => std::mem::size_of::<V>(),
        }
    }

    async fn evict_if_needed(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        let mut stats = self.statistics.write().await;

        // Check if we need to evict entries
        if entries.len() >= self.config.max_entries {
            // LRU eviction - remove least recently accessed
            let mut entries_vec: Vec<_> = entries.iter().collect();
            entries_vec.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));

            // Collect keys to remove first
            let to_remove_count = (entries.len() / 10).max(1);
            let keys_to_remove: Vec<_> = entries_vec
                .iter()
                .take(to_remove_count)
                .map(|(key, _)| (*key).clone())
                .collect();

            // Now remove the entries
            for key in keys_to_remove {
                entries.remove(&key);
                stats.eviction_count += 1;
            }

            info!(
                "Evicted {} entries due to max entries limit",
                to_remove_count
            );
        }

        // Update total entries count
        stats.total_entries = entries.len();

        Ok(())
    }
}

#[async_trait::async_trait]
impl<K, V> CacheInterface<K, V> for InMemoryCache<K, V>
where
    K: Send + Sync + Clone + Eq + Hash + 'static,
    V: Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<V>> {
        let start_time = Instant::now();

        let mut entries = self.entries.write().await;
        let mut stats = self.statistics.write().await;

        if let Some(entry) = entries.get_mut(key) {
            // Check if entry is expired
            let now = chrono::Utc::now();
            if let Some(expires_at) = entry.expires_at {
                if expires_at <= now {
                    entries.remove(key);
                    stats.miss_count += 1;
                    return Ok(None);
                }
            }

            // Update access statistics
            entry.access_count += 1;
            entry.last_accessed = now;

            stats.hit_count += 1;
            stats.hit_ratio = stats.hit_count as f64 / (stats.hit_count + stats.miss_count) as f64;

            let access_time = start_time.elapsed().as_millis() as f64;
            stats.average_access_time_ms = (stats.average_access_time_ms + access_time) / 2.0;

            debug!("Cache hit for key (access time: {:.2}ms)", access_time);
            Ok(Some(entry.value.clone()))
        } else {
            stats.miss_count += 1;
            stats.hit_ratio = stats.hit_count as f64 / (stats.hit_count + stats.miss_count) as f64;

            debug!("Cache miss for key");
            Ok(None)
        }
    }

    async fn set(&self, key: K, value: V, ttl: Option<Duration>) -> Result<()> {
        let now = chrono::Utc::now();
        let expires_at = ttl.map(|duration| now + chrono::Duration::from_std(duration).unwrap());
        let size_bytes = self.estimate_size(&value);

        let entry = CacheEntry {
            value,
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
            size_bytes,
        };

        // Evict entries if needed
        self.evict_if_needed().await?;

        let mut entries = self.entries.write().await;
        let mut stats = self.statistics.write().await;

        // Update size statistics
        if let Some(old_entry) = entries.get(&key) {
            stats.total_size_bytes = stats.total_size_bytes.saturating_sub(old_entry.size_bytes);
        } else {
            stats.total_entries += 1;
        }

        stats.total_size_bytes += size_bytes;
        entries.insert(key, entry);

        debug!("Cache entry set (size: {} bytes)", size_bytes);
        Ok(())
    }

    async fn delete(&self, key: &K) -> Result<bool> {
        let mut entries = self.entries.write().await;
        let mut stats = self.statistics.write().await;

        if let Some(entry) = entries.remove(key) {
            stats.total_entries = stats.total_entries.saturating_sub(1);
            stats.total_size_bytes = stats.total_size_bytes.saturating_sub(entry.size_bytes);

            debug!("Cache entry deleted");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn exists(&self, key: &K) -> Result<bool> {
        let entries = self.entries.read().await;
        let exists = entries.contains_key(key);

        if exists {
            // Check if expired
            if let Some(entry) = entries.get(key) {
                if let Some(expires_at) = entry.expires_at {
                    let now = chrono::Utc::now();
                    return Ok(expires_at > now);
                }
            }
        }

        Ok(exists)
    }

    async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        let mut stats = self.statistics.write().await;

        let cleared_count = entries.len();
        entries.clear();

        stats.total_entries = 0;
        stats.total_size_bytes = 0;

        info!("Cache cleared: {} entries removed", cleared_count);
        Ok(())
    }

    async fn get_statistics(&self) -> Result<CacheStatistics> {
        let stats = self.statistics.read().await;
        Ok(stats.clone())
    }
}

/// Distributed cache client for Redis integration
pub struct DistributedCache {
    config: CacheConfig,
    redis_client: Option<Arc<redis::Client>>,
    local_cache: InMemoryCache<String, Vec<u8>>,
    statistics: Arc<RwLock<CacheStatistics>>,
}

impl DistributedCache {
    pub async fn new(config: CacheConfig) -> Result<Self> {
        let redis_client = if let Some(redis_url) = &config.redis_url {
            let client = redis::Client::open(redis_url.as_str())
                .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;
            Some(Arc::new(client))
        } else {
            None
        };

        let local_config = CacheConfig {
            backend: CacheBackend::InMemory,
            max_entries: config.max_entries / 10, // Smaller local cache
            ..config.clone()
        };

        let statistics = CacheStatistics {
            total_entries: 0,
            total_size_bytes: 0,
            hit_count: 0,
            miss_count: 0,
            eviction_count: 0,
            hit_ratio: 0.0,
            average_access_time_ms: 0.0,
            backend_type: config.backend.clone(),
            last_cleanup: chrono::Utc::now(),
        };

        Ok(Self {
            config,
            redis_client,
            local_cache: InMemoryCache::new(local_config),
            statistics: Arc::new(RwLock::new(statistics)),
        })
    }

    async fn get_redis_connection(&self) -> Result<redis::aio::Connection> {
        if let Some(client) = &self.redis_client {
            client
                .get_async_connection()
                .await
                .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))
        } else {
            Err(anyhow!("Redis client not configured"))
        }
    }

    fn make_redis_key(&self, key: &str) -> String {
        format!("{}{}", self.config.redis_key_prefix, key)
    }

    async fn compress_if_needed(&self, data: &[u8]) -> Result<Vec<u8>> {
        if self.config.enable_compression && data.len() >= self.config.compression_threshold {
            use flate2::{write::GzEncoder, Compression};
            use std::io::Write;

            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(data)
                .map_err(|e| anyhow!("Compression failed: {}", e))?;
            let compressed = encoder
                .finish()
                .map_err(|e| anyhow!("Compression finalization failed: {}", e))?;

            // Add compression marker (simple prefix)
            let mut result = vec![0xFF, 0xFE]; // Compression marker
            result.extend_from_slice(&compressed);

            debug!(
                "Compressed {} bytes to {} bytes (ratio: {:.2})",
                data.len(),
                result.len(),
                result.len() as f64 / data.len() as f64
            );

            Ok(result)
        } else {
            Ok(data.to_vec())
        }
    }

    async fn decompress_if_needed(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Check for compression marker
        if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xFE {
            use flate2::read::GzDecoder;
            use std::io::Read;

            let compressed_data = &data[2..]; // Skip marker
            let mut decoder = GzDecoder::new(compressed_data);
            let mut decompressed = Vec::new();

            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| anyhow!("Decompression failed: {}", e))?;

            debug!(
                "Decompressed {} bytes to {} bytes",
                compressed_data.len(),
                decompressed.len()
            );
            Ok(decompressed)
        } else {
            Ok(data.to_vec())
        }
    }
}

#[async_trait::async_trait]
impl CacheInterface<String, Vec<u8>> for DistributedCache {
    async fn get(&self, key: &String) -> Result<Option<Vec<u8>>> {
        let start_time = Instant::now();

        // Try local cache first
        if let Ok(Some(value)) = self.local_cache.get(key).await {
            let mut stats = self.statistics.write().await;
            stats.hit_count += 1;
            debug!("Distributed cache: local hit for key '{}'", key);
            return Ok(Some(value));
        }

        // Try Redis
        if let Ok(mut conn) = self.get_redis_connection().await {
            let redis_key = self.make_redis_key(key);

            match redis::cmd("GET")
                .arg(&redis_key)
                .query_async::<_, Option<Vec<u8>>>(&mut conn)
                .await
            {
                Ok(Some(data)) => {
                    let decompressed = self.decompress_if_needed(&data).await?;

                    // Store in local cache for faster access
                    let _ = self
                        .local_cache
                        .set(
                            key.to_string(),
                            decompressed.clone(),
                            Some(Duration::from_secs(300)),
                        )
                        .await;

                    let mut stats = self.statistics.write().await;
                    stats.hit_count += 1;

                    let access_time = start_time.elapsed().as_millis() as f64;
                    stats.average_access_time_ms =
                        (stats.average_access_time_ms + access_time) / 2.0;

                    debug!(
                        "Distributed cache: Redis hit for key '{}' ({}ms)",
                        key, access_time
                    );
                    return Ok(Some(decompressed));
                }
                Ok(None) => {
                    debug!("Distributed cache: Redis miss for key '{}'", key);
                }
                Err(e) => {
                    warn!("Redis get error for key '{}': {}", key, e);
                }
            }
        }

        let mut stats = self.statistics.write().await;
        stats.miss_count += 1;
        stats.hit_ratio = stats.hit_count as f64 / (stats.hit_count + stats.miss_count) as f64;

        Ok(None)
    }

    async fn set(&self, key: String, value: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
        let compressed = self.compress_if_needed(&value).await?;

        // Store in local cache
        let _ = self.local_cache.set(key.clone(), value.clone(), ttl).await;

        // Store in Redis
        if let Ok(mut conn) = self.get_redis_connection().await {
            let redis_key = self.make_redis_key(&key);

            let mut cmd = redis::cmd("SET");
            cmd.arg(&redis_key).arg(&compressed);

            if let Some(ttl) = ttl {
                cmd.arg("EX").arg(ttl.as_secs());
            }

            if let Err(e) = cmd.query_async::<_, ()>(&mut conn).await {
                warn!("Redis set error for key '{}': {}", key, e);
            } else {
                debug!("Distributed cache: set key '{}' in Redis", key);
            }
        }

        let mut stats = self.statistics.write().await;
        stats.total_entries += 1;
        stats.total_size_bytes += value.len();

        Ok(())
    }

    async fn delete(&self, key: &String) -> Result<bool> {
        let mut deleted = false;

        // Delete from local cache
        if self.local_cache.delete(key).await? {
            deleted = true;
        }

        // Delete from Redis
        if let Ok(mut conn) = self.get_redis_connection().await {
            let redis_key = self.make_redis_key(key);

            match redis::cmd("DEL")
                .arg(&redis_key)
                .query_async::<_, i32>(&mut conn)
                .await
            {
                Ok(count) => {
                    if count > 0 {
                        deleted = true;
                        debug!("Distributed cache: deleted key '{}' from Redis", key);
                    }
                }
                Err(e) => {
                    warn!("Redis delete error for key '{}': {}", key, e);
                }
            }
        }

        if deleted {
            let mut stats = self.statistics.write().await;
            stats.total_entries = stats.total_entries.saturating_sub(1);
        }

        Ok(deleted)
    }

    async fn exists(&self, key: &String) -> Result<bool> {
        // Check local cache first
        if self.local_cache.exists(key).await? {
            return Ok(true);
        }

        // Check Redis
        if let Ok(mut conn) = self.get_redis_connection().await {
            let redis_key = self.make_redis_key(key);

            match redis::cmd("EXISTS")
                .arg(&redis_key)
                .query_async::<_, i32>(&mut conn)
                .await
            {
                Ok(exists) => return Ok(exists > 0),
                Err(e) => {
                    warn!("Redis exists error for key '{}': {}", key, e);
                }
            }
        }

        Ok(false)
    }

    async fn clear(&self) -> Result<()> {
        // Clear local cache
        self.local_cache.clear().await?;

        // Clear Redis (all keys with our prefix)
        if let Ok(mut conn) = self.get_redis_connection().await {
            let pattern = format!("{}*", self.config.redis_key_prefix);

            match redis::cmd("KEYS")
                .arg(&pattern)
                .query_async::<_, Vec<String>>(&mut conn)
                .await
            {
                Ok(keys) => {
                    if !keys.is_empty() {
                        match redis::cmd("DEL")
                            .arg(&keys)
                            .query_async::<_, i32>(&mut conn)
                            .await
                        {
                            Ok(deleted) => {
                                info!("Distributed cache: cleared {} keys from Redis", deleted);
                            }
                            Err(e) => {
                                warn!("Redis clear error: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Redis keys error: {}", e);
                }
            }
        }

        let mut stats = self.statistics.write().await;
        stats.total_entries = 0;
        stats.total_size_bytes = 0;

        Ok(())
    }

    async fn get_statistics(&self) -> Result<CacheStatistics> {
        let stats = self.statistics.read().await;
        Ok(stats.clone())
    }
}

/// Cache manager for coordinating multiple cache layers
pub struct CacheManager {
    primary_cache: Arc<dyn CacheInterface<String, Vec<u8>>>,
    config: CacheConfig,
    metrics: Arc<RwLock<CacheMetrics>>,
}

/// Enhanced cache metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_errors: u64,
    pub total_entries: usize,
    pub memory_usage_bytes: usize,
    pub redis_usage_bytes: usize,
    pub compression_ratio: f64,
    pub average_compression_ratio: f64,
    pub items_compressed: u64,
    pub items_uncompressed: u64,
    pub total_operations: u64,
    pub last_cleanup: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            cache_errors: 0,
            total_entries: 0,
            memory_usage_bytes: 0,
            redis_usage_bytes: 0,
            compression_ratio: 1.0,
            average_compression_ratio: 1.0,
            items_compressed: 0,
            items_uncompressed: 0,
            total_operations: 0,
            last_cleanup: chrono::Utc::now(),
            uptime_seconds: 0,
        }
    }
}

impl CacheManager {
    pub async fn new(config: CacheConfig) -> Result<Self> {
        let primary_cache: Arc<dyn CacheInterface<String, Vec<u8>>> = match config.backend {
            CacheBackend::InMemory => Arc::new(InMemoryCache::new(config.clone())),
            CacheBackend::Redis | CacheBackend::Distributed | CacheBackend::Hybrid => {
                Arc::new(DistributedCache::new(config.clone()).await?)
            }
        };

        let manager = Self {
            primary_cache,
            config,
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        };

        // Start metrics update task
        manager.start_metrics_updater();

        Ok(manager)
    }

    fn start_metrics_updater(&self) {
        let metrics = self.metrics.clone();
        let cache = self.primary_cache.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            let start_time = Instant::now();

            loop {
                interval.tick().await;

                if let Ok(cache_stats) = cache.get_statistics().await {
                    let mut metrics_guard = metrics.write().await;

                    metrics_guard.cache_hits = cache_stats.hit_count;
                    metrics_guard.cache_misses = cache_stats.miss_count;
                    metrics_guard.total_entries = cache_stats.total_entries;
                    metrics_guard.memory_usage_bytes = cache_stats.total_size_bytes;
                    metrics_guard.uptime_seconds = start_time.elapsed().as_secs();
                    metrics_guard.total_operations = cache_stats.hit_count + cache_stats.miss_count;

                    // Calculate hit ratio
                    if metrics_guard.total_operations > 0 {
                        let hit_ratio =
                            metrics_guard.cache_hits as f64 / metrics_guard.total_operations as f64;
                        debug!(
                            "Cache metrics updated: hit_ratio={:.2}, entries={}, operations={}",
                            hit_ratio, metrics_guard.total_entries, metrics_guard.total_operations
                        );
                    }
                }
            }
        });
    }

    /// Cache a serializable object
    pub async fn cache_object<T>(&self, key: &str, object: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize,
    {
        let data =
            serde_json::to_vec(object).map_err(|e| anyhow!("Failed to serialize object: {}", e))?;

        self.primary_cache.set(key.to_string(), data, ttl).await
    }

    /// Retrieve a cached object
    pub async fn get_object<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        if let Some(data) = self.primary_cache.get(&key.to_string()).await? {
            let object: T = serde_json::from_slice(&data)
                .map_err(|e| anyhow!("Failed to deserialize object: {}", e))?;
            Ok(Some(object))
        } else {
            Ok(None)
        }
    }

    /// Cache raw bytes
    pub async fn cache_bytes(&self, key: &str, data: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
        self.primary_cache.set(key.to_string(), data, ttl).await
    }

    /// Retrieve cached bytes
    pub async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.primary_cache.get(&key.to_string()).await
    }

    /// Delete from cache
    pub async fn delete(&self, key: &str) -> Result<bool> {
        self.primary_cache.delete(&key.to_string()).await
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        self.primary_cache.exists(&key.to_string()).await
    }

    /// Clear all cache entries
    pub async fn clear_all(&self) -> Result<()> {
        self.primary_cache.clear().await
    }

    /// Get cache statistics
    pub async fn get_statistics(&self) -> Result<CacheStatistics> {
        self.primary_cache.get_statistics().await
    }

    /// Get cache configuration
    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }

    /// Get enhanced cache metrics
    pub async fn get_enhanced_metrics(&self) -> Result<CacheMetrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    /// Warm up cache with preloaded data
    pub async fn warmup<T>(&self, data: HashMap<String, T>, ttl: Option<Duration>) -> Result<usize>
    where
        T: Serialize,
    {
        let mut loaded = 0;

        for (key, value) in data {
            if let Err(e) = self.cache_object(&key, &value, ttl).await {
                warn!("Failed to warm up cache entry '{}': {}", key, e);
            } else {
                loaded += 1;
            }
        }

        info!("Cache warmup completed: {} entries loaded", loaded);
        Ok(loaded)
    }

    /// Batch get operation for multiple keys
    pub async fn get_batch(&self, keys: &[&str]) -> Result<HashMap<String, Vec<u8>>> {
        let mut results = HashMap::new();

        for key in keys {
            if let Some(value) = self.get_bytes(key).await? {
                results.insert(key.to_string(), value);
            }
        }

        Ok(results)
    }

    /// Batch set operation for multiple key-value pairs
    pub async fn set_batch(
        &self,
        data: HashMap<String, Vec<u8>>,
        ttl: Option<Duration>,
    ) -> Result<usize> {
        let mut stored = 0;

        for (key, value) in data {
            if let Err(e) = self.cache_bytes(&key, value, ttl).await {
                warn!("Failed to batch set cache entry '{}': {}", key, e);
            } else {
                stored += 1;
            }
        }

        Ok(stored)
    }

    /// Invalidate cache entries matching a pattern
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<usize> {
        // For now, this is a simplified implementation
        // In production, you'd want pattern matching support in Redis
        warn!(
            "Pattern invalidation not fully implemented for pattern: {}",
            pattern
        );
        Ok(0)
    }

    /// Get cache hit ratio as percentage
    pub async fn get_hit_ratio(&self) -> Result<f64> {
        let stats = self.get_statistics().await?;
        if stats.hit_count + stats.miss_count > 0 {
            Ok(stats.hit_count as f64 / (stats.hit_count + stats.miss_count) as f64 * 100.0)
        } else {
            Ok(0.0)
        }
    }

    /// Force a cache cleanup/eviction cycle
    pub async fn force_cleanup(&self) -> Result<()> {
        info!("Forcing cache cleanup cycle");

        // This would trigger cleanup in the underlying cache
        // For now, we'll just update the metrics
        let mut metrics = self.metrics.write().await;
        metrics.last_cleanup = chrono::Utc::now();

        Ok(())
    }

    /// Get cache memory efficiency ratio
    pub async fn get_memory_efficiency(&self) -> Result<f64> {
        let stats = self.get_statistics().await?;
        if stats.total_entries > 0 {
            let avg_entry_size = stats.total_size_bytes as f64 / stats.total_entries as f64;
            // Return efficiency as ratio of useful data vs overhead
            Ok(avg_entry_size / (avg_entry_size + 64.0)) // Assume ~64 bytes overhead per entry
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_cache() -> Result<()> {
        let config = CacheConfig::default();
        let cache = InMemoryCache::<String, String>::new(config);

        // Test set and get
        cache
            .set("key1".to_string(), "value1".to_string(), None)
            .await?;
        let value = cache.get(&"key1".to_string()).await?;
        assert_eq!(value, Some("value1".to_string()));

        // Test miss
        let missing = cache.get(&"key2".to_string()).await?;
        assert_eq!(missing, None);

        // Test exists
        assert!(cache.exists(&"key1".to_string()).await?);
        assert!(!cache.exists(&"key2".to_string()).await?);

        // Test delete
        assert!(cache.delete(&"key1".to_string()).await?);
        assert!(!cache.exists(&"key1".to_string()).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_manager() -> Result<()> {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config).await?;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestObject {
            id: u32,
            name: String,
        }

        let obj = TestObject {
            id: 123,
            name: "test".to_string(),
        };

        // Cache object
        manager.cache_object("test_obj", &obj, None).await?;

        // Retrieve object
        let retrieved: Option<TestObject> = manager.get_object("test_obj").await?;
        assert_eq!(retrieved, Some(obj));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_expiration() -> Result<()> {
        let config = CacheConfig::default();
        let cache = InMemoryCache::<String, String>::new(config);

        // Set with short TTL
        cache
            .set(
                "key1".to_string(),
                "value1".to_string(),
                Some(Duration::from_millis(100)),
            )
            .await?;

        // Should exist immediately
        assert!(cache.exists(&"key1".to_string()).await?);

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        let value = cache.get(&"key1".to_string()).await?;
        assert_eq!(value, None);

        Ok(())
    }
}
