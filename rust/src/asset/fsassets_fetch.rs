use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use anyhow::Result;
use bytes::Bytes;
use tracing::{debug, info, warn};

use super::fsassets::FSAssetsStorage;
use super::cache::AssetCache;

#[derive(Debug, Default)]
pub struct FetchStats {
    pub requests: AtomicU64,
    pub cache_hits: AtomicU64,
    pub fs_hits: AtomicU64,
    pub db_hits: AtomicU64,
    pub misses: AtomicU64,
    pub lazy_migrated: AtomicU64,
}

pub struct AssetFetcher {
    fsassets: Option<Arc<FSAssetsStorage>>,
    cache: Option<Arc<AssetCache>>,
    stats: Arc<FetchStats>,
    lazy_migrate: bool,
}

impl AssetFetcher {
    pub fn new(fsassets: Option<Arc<FSAssetsStorage>>, lazy_migrate: bool) -> Self {
        Self {
            fsassets,
            cache: None,
            stats: Arc::new(FetchStats::default()),
            lazy_migrate,
        }
    }

    pub fn new_legacy() -> Self {
        Self {
            fsassets: None,
            cache: None,
            stats: Arc::new(FetchStats::default()),
            lazy_migrate: false,
        }
    }

    pub fn with_cache(mut self, cache: Arc<AssetCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub async fn fetch_asset_data_pg(
        &self,
        asset_id: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Vec<u8>>> {
        self.stats.requests.fetch_add(1, Ordering::Relaxed);

        if let Some(ref cache) = self.cache {
            if let Ok(Some(cached)) = cache.get(asset_id).await {
                let data = if cached.compressed {
                    cache.decompress_asset(cached)?.data.to_vec()
                } else {
                    cached.data.to_vec()
                };
                self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(data));
            }
        }

        if let Some(ref fs) = self.fsassets {
            if let Some(data) = self.try_fsassets_pg(asset_id, pool, fs).await? {
                self.stats.fs_hits.fetch_add(1, Ordering::Relaxed);
                self.promote_to_cache(asset_id, &data).await;
                return Ok(Some(data));
            }
        }

        let data = self.fetch_legacy_pg(asset_id, pool).await?;

        if let Some(ref blob) = data {
            self.stats.db_hits.fetch_add(1, Ordering::Relaxed);
            self.promote_to_cache(asset_id, blob).await;

            if self.lazy_migrate {
                if let Some(ref fs) = self.fsassets {
                    if let Err(e) = self.lazy_migrate_pg(asset_id, blob, pool, fs).await {
                        warn!("Lazy migration failed for {}: {}", asset_id, e);
                    } else {
                        self.stats.lazy_migrated.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            return Ok(Some(blob.clone()));
        }

        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }

    pub async fn fetch_asset_data_typed_pg(
        &self,
        asset_id: &str,
        asset_type_filter: Option<i32>,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Vec<u8>>> {
        self.stats.requests.fetch_add(1, Ordering::Relaxed);

        if let Some(ref cache) = self.cache {
            if let Ok(Some(cached)) = cache.get(asset_id).await {
                let data = if cached.compressed {
                    cache.decompress_asset(cached)?.data.to_vec()
                } else {
                    cached.data.to_vec()
                };
                self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(data));
            }
        }

        if let Some(ref fs) = self.fsassets {
            if let Some(data) = self.try_fsassets_typed_pg(asset_id, asset_type_filter, pool, fs).await? {
                self.stats.fs_hits.fetch_add(1, Ordering::Relaxed);
                self.promote_to_cache(asset_id, &data).await;
                return Ok(Some(data));
            }
        }

        let data = match asset_type_filter {
            Some(at) => {
                let row: Option<(Vec<u8>,)> = sqlx::query_as(
                    "SELECT data FROM assets WHERE id = $1::uuid AND assettype = $2"
                )
                .bind(asset_id)
                .bind(at)
                .fetch_optional(pool)
                .await?;
                row.map(|(d,)| d)
            }
            None => self.fetch_legacy_pg(asset_id, pool).await?,
        };

        if let Some(ref blob) = data {
            self.stats.db_hits.fetch_add(1, Ordering::Relaxed);
            self.promote_to_cache(asset_id, blob).await;
            if self.lazy_migrate {
                if let Some(ref fs) = self.fsassets {
                    let _ = self.lazy_migrate_pg(asset_id, blob, pool, fs).await;
                }
            }
            return Ok(data);
        }

        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }

    async fn try_fsassets_pg(
        &self,
        asset_id: &str,
        pool: &sqlx::PgPool,
        fs: &FSAssetsStorage,
    ) -> Result<Option<Vec<u8>>> {
        use sqlx::Row;
        let row = sqlx::query(
            "SELECT hash FROM fsassets WHERE id = $1::uuid"
        )
        .bind(asset_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            let hash: String = row.try_get("hash")?;
            let hash = hash.trim().to_string();
            if let Some(data) = fs.retrieve(&hash).await? {
                debug!("FSAssets hit for {}", asset_id);
                return Ok(Some(data));
            }
            warn!("FSAssets metadata exists for {} but file missing (hash={})", asset_id, hash);
        }

        Ok(None)
    }

    async fn try_fsassets_typed_pg(
        &self,
        asset_id: &str,
        asset_type_filter: Option<i32>,
        pool: &sqlx::PgPool,
        fs: &FSAssetsStorage,
    ) -> Result<Option<Vec<u8>>> {
        use sqlx::Row;
        let row = match asset_type_filter {
            Some(at) => {
                sqlx::query(
                    "SELECT hash FROM fsassets WHERE id = $1::uuid AND type = $2"
                )
                .bind(asset_id)
                .bind(at)
                .fetch_optional(pool)
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT hash FROM fsassets WHERE id = $1::uuid"
                )
                .bind(asset_id)
                .fetch_optional(pool)
                .await?
            }
        };

        if let Some(row) = row {
            let hash: String = row.try_get("hash")?;
            let hash = hash.trim().to_string();
            if let Some(data) = fs.retrieve(&hash).await? {
                return Ok(Some(data));
            }
        }

        Ok(None)
    }

    async fn fetch_legacy_pg(
        &self,
        asset_id: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Vec<u8>>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT data FROM assets WHERE id = $1::uuid"
        )
        .bind(asset_id)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|(d,)| d))
    }

    async fn lazy_migrate_pg(
        &self,
        asset_id: &str,
        data: &[u8],
        pool: &sqlx::PgPool,
        fs: &FSAssetsStorage,
    ) -> Result<()> {
        let hash = FSAssetsStorage::compute_hash(data);
        fs.store(data).await?;

        use sqlx::Row;
        let type_row = sqlx::query(
            "SELECT assettype, COALESCE(asset_flags, 0) as asset_flags FROM assets WHERE id = $1::uuid"
        )
        .bind(asset_id)
        .fetch_optional(pool)
        .await?;

        let (asset_type, asset_flags) = if let Some(row) = type_row {
            (
                row.try_get::<i32, _>("assettype").unwrap_or(0),
                row.try_get::<i32, _>("asset_flags").unwrap_or(0),
            )
        } else {
            (0, 0)
        };

        let now = chrono::Utc::now().timestamp() as i32;
        sqlx::query(
            "INSERT INTO fsassets (id, type, hash, create_time, access_time, asset_flags) \
             VALUES ($1::uuid, $2, $3, $4, $5, $6) \
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(asset_id)
        .bind(asset_type)
        .bind(&hash)
        .bind(now)
        .bind(now)
        .bind(asset_flags)
        .execute(pool)
        .await?;

        debug!("Lazy migrated asset {} to fsassets", asset_id);
        Ok(())
    }

    pub async fn batch_fetch_pg(
        &self,
        asset_ids: &[String],
        pool: &sqlx::PgPool,
    ) -> Result<std::collections::HashMap<String, Vec<u8>>> {
        use std::collections::HashMap;

        if asset_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let unique: Vec<String> = {
            let mut set = std::collections::HashSet::new();
            asset_ids.iter().filter(|id| set.insert(id.clone())).cloned().collect()
        };

        let count = unique.len();
        self.stats.requests.fetch_add(count as u64, Ordering::Relaxed);

        let mut results: HashMap<String, Vec<u8>> = HashMap::with_capacity(count);

        if let Some(ref fs) = self.fsassets {
            use sqlx::Row;
            let rows = sqlx::query(
                "SELECT id::text, hash FROM fsassets WHERE id = ANY($1::uuid[])"
            )
            .bind(&unique)
            .fetch_all(pool)
            .await?;

            let mut fs_futures = Vec::with_capacity(rows.len());
            for row in &rows {
                let id: String = row.try_get("id")?;
                let hash: String = row.try_get::<String, _>("hash")?.trim().to_string();
                let fs_clone = fs.clone();
                fs_futures.push(async move {
                    let data = fs_clone.retrieve(&hash).await;
                    (id, data)
                });
            }

            let fs_results = futures::future::join_all(fs_futures).await;
            for (id, data_result) in fs_results {
                match data_result {
                    Ok(Some(data)) => {
                        self.stats.fs_hits.fetch_add(1, Ordering::Relaxed);
                        results.insert(id, data);
                    }
                    Ok(None) => {
                        warn!("FSAssets metadata exists for {} but file missing", id);
                    }
                    Err(e) => {
                        warn!("FSAssets read error for {}: {}", id, e);
                    }
                }
            }
        }

        let missing: Vec<String> = unique.iter()
            .filter(|id| !results.contains_key(*id))
            .cloned()
            .collect();

        if !missing.is_empty() {
            use sqlx::Row;
            let rows = sqlx::query(
                "SELECT id::text, data FROM assets WHERE id = ANY($1::uuid[]) AND data IS NOT NULL"
            )
            .bind(&missing)
            .fetch_all(pool)
            .await?;

            for row in &rows {
                let id: String = row.try_get("id")?;
                let data: Vec<u8> = row.try_get("data")?;
                if !data.is_empty() {
                    self.stats.db_hits.fetch_add(1, Ordering::Relaxed);

                    if self.lazy_migrate {
                        if let Some(ref fs) = self.fsassets {
                            if let Err(e) = self.lazy_migrate_pg(&id, &data, pool, fs).await {
                                warn!("Batch lazy migration failed for {}: {}", id, e);
                            } else {
                                self.stats.lazy_migrated.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }

                    results.insert(id, data);
                }
            }
        }

        let hit_count = results.len() as u64;
        let miss_count = count as u64 - hit_count;
        self.stats.misses.fetch_add(miss_count, Ordering::Relaxed);

        debug!("Batch fetch: {} requested, {} unique, {} found, {} missing",
            asset_ids.len(), count, hit_count, miss_count);

        Ok(results)
    }

    async fn promote_to_cache(&self, asset_id: &str, data: &[u8]) {
        if let Some(ref cache) = self.cache {
            let asset_type = super::AssetType::Unknown;
            if let Err(e) = cache.put(asset_id, Bytes::copy_from_slice(data), asset_type).await {
                debug!("Cache promotion failed for {}: {}", asset_id, e);
            }
        }
    }

    pub fn get_stats(&self) -> FetchStatsSnapshot {
        FetchStatsSnapshot {
            requests: self.stats.requests.load(Ordering::Relaxed),
            cache_hits: self.stats.cache_hits.load(Ordering::Relaxed),
            fs_hits: self.stats.fs_hits.load(Ordering::Relaxed),
            db_hits: self.stats.db_hits.load(Ordering::Relaxed),
            misses: self.stats.misses.load(Ordering::Relaxed),
            lazy_migrated: self.stats.lazy_migrated.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchStatsSnapshot {
    pub requests: u64,
    pub cache_hits: u64,
    pub fs_hits: u64,
    pub db_hits: u64,
    pub misses: u64,
    pub lazy_migrated: u64,
}

impl FetchStatsSnapshot {
    pub fn hit_rate(&self) -> f64 {
        if self.requests == 0 {
            return 0.0;
        }
        (self.cache_hits + self.fs_hits) as f64 / self.requests as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_fetcher() {
        let fetcher = AssetFetcher::new_legacy();
        let stats = fetcher.get_stats();
        assert_eq!(stats.requests, 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_stats_snapshot_hit_rate() {
        let snap = FetchStatsSnapshot {
            requests: 100,
            cache_hits: 50,
            fs_hits: 30,
            db_hits: 15,
            misses: 5,
            lazy_migrated: 15,
        };
        assert!((snap.hit_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_stats_zero_requests() {
        let snap = FetchStatsSnapshot {
            requests: 0,
            cache_hits: 0,
            fs_hits: 0,
            db_hits: 0,
            misses: 0,
            lazy_migrated: 0,
        };
        assert_eq!(snap.hit_rate(), 0.0);
    }
}
