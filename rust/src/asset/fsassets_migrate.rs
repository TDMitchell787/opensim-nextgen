use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};

use super::fsassets::FSAssetsStorage;
use crate::database::multi_backend::DatabaseConnection;

#[derive(Debug, Clone)]
pub struct MigrationStats {
    pub total_assets: u64,
    pub migrated: u64,
    pub skipped_existing: u64,
    pub skipped_null_data: u64,
    pub errors: u64,
    pub bytes_processed: u64,
    pub elapsed_secs: f64,
}

pub struct FSAssetsMigrator {
    storage: Arc<FSAssetsStorage>,
    connection: Arc<DatabaseConnection>,
    migrated: AtomicU64,
    skipped_existing: AtomicU64,
    skipped_null: AtomicU64,
    errors: AtomicU64,
    bytes_processed: AtomicU64,
}

impl FSAssetsMigrator {
    pub fn new(storage: Arc<FSAssetsStorage>, connection: Arc<DatabaseConnection>) -> Self {
        Self {
            storage,
            connection,
            migrated: AtomicU64::new(0),
            skipped_existing: AtomicU64::new(0),
            skipped_null: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            bytes_processed: AtomicU64::new(0),
        }
    }

    pub async fn migrate_all(&self, batch_size: i64) -> Result<MigrationStats> {
        let start = Instant::now();

        let total = self.count_source_assets().await?;
        info!("Starting FSAssets migration: {} assets to process", total);

        let mut offset: i64 = 0;
        loop {
            let batch = self.fetch_batch(offset, batch_size).await?;
            if batch.is_empty() {
                break;
            }

            for (id, asset_type, asset_flags, data) in &batch {
                if let Err(e) = self
                    .migrate_one(id, *asset_type, *asset_flags, data.as_deref())
                    .await
                {
                    error!("Failed to migrate asset {}: {}", id, e);
                    self.errors.fetch_add(1, Ordering::Relaxed);
                }
            }

            offset += batch.len() as i64;
            let m = self.migrated.load(Ordering::Relaxed);
            let s = self.skipped_existing.load(Ordering::Relaxed);
            if offset % 1000 < batch_size {
                info!(
                    "Migration progress: {}/{} processed, {} migrated, {} skipped (existing), {} errors",
                    offset, total, m, s, self.errors.load(Ordering::Relaxed)
                );
            }
        }

        let elapsed = start.elapsed().as_secs_f64();
        let stats = MigrationStats {
            total_assets: total as u64,
            migrated: self.migrated.load(Ordering::Relaxed),
            skipped_existing: self.skipped_existing.load(Ordering::Relaxed),
            skipped_null_data: self.skipped_null.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            bytes_processed: self.bytes_processed.load(Ordering::Relaxed),
            elapsed_secs: elapsed,
        };

        info!(
            "FSAssets migration complete: {} migrated, {} skipped, {} errors, {:.1} GB processed in {:.0}s",
            stats.migrated,
            stats.skipped_existing + stats.skipped_null_data,
            stats.errors,
            stats.bytes_processed as f64 / (1024.0 * 1024.0 * 1024.0),
            stats.elapsed_secs
        );

        Ok(stats)
    }

    async fn migrate_one(
        &self,
        id: &str,
        asset_type: i32,
        asset_flags: i32,
        data: Option<&[u8]>,
    ) -> Result<()> {
        let data = match data {
            Some(d) if !d.is_empty() => d,
            _ => {
                self.skipped_null.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
        };

        let hash = FSAssetsStorage::compute_hash(data);

        if self.fsassets_row_exists(&hash).await? {
            self.insert_fsassets_id_if_missing(id, asset_type, asset_flags, &hash)
                .await?;
            self.skipped_existing.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }

        self.storage.store(data).await?;

        let now = chrono::Utc::now().timestamp() as i32;
        self.insert_fsassets_metadata(id, asset_type, &hash, now, asset_flags)
            .await?;

        self.migrated.fetch_add(1, Ordering::Relaxed);
        self.bytes_processed
            .fetch_add(data.len() as u64, Ordering::Relaxed);
        Ok(())
    }

    async fn count_source_assets(&self) -> Result<i64> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => {
                let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM assets")
                    .fetch_one(pool)
                    .await?;
                Ok(row.0)
            }
            _ => Err(anyhow!("FSAssets migration currently requires PostgreSQL")),
        }
    }

    async fn fetch_batch(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<(String, i32, i32, Option<Vec<u8>>)>> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => {
                use sqlx::Row;
                let rows = sqlx::query(
                    "SELECT id::text, assettype, COALESCE(asset_flags, 0) as asset_flags, data \
                     FROM assets ORDER BY id OFFSET $1 LIMIT $2",
                )
                .bind(offset)
                .bind(limit)
                .fetch_all(pool)
                .await?;

                let mut results = Vec::with_capacity(rows.len());
                for row in &rows {
                    let id: String = row.try_get("id")?;
                    let asset_type: i32 = row.try_get("assettype")?;
                    let asset_flags: i32 = row.try_get("asset_flags").unwrap_or(0);
                    let data: Option<Vec<u8>> = row.try_get("data").ok();
                    results.push((id, asset_type, asset_flags, data));
                }
                Ok(results)
            }
            _ => Err(anyhow!("FSAssets migration currently requires PostgreSQL")),
        }
    }

    async fn fsassets_row_exists(&self, hash: &str) -> Result<bool> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => {
                let row: Option<(i32,)> =
                    sqlx::query_as("SELECT 1 FROM fsassets WHERE hash = $1 LIMIT 1")
                        .bind(hash)
                        .fetch_optional(pool)
                        .await?;
                Ok(row.is_some())
            }
            _ => Err(anyhow!("FSAssets migration currently requires PostgreSQL")),
        }
    }

    async fn insert_fsassets_id_if_missing(
        &self,
        id: &str,
        asset_type: i32,
        asset_flags: i32,
        hash: &str,
    ) -> Result<()> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => {
                let now = chrono::Utc::now().timestamp() as i32;
                sqlx::query(
                    "INSERT INTO fsassets (id, type, hash, create_time, access_time, asset_flags) \
                     VALUES ($1::uuid, $2, $3, $4, $5, $6) \
                     ON CONFLICT (id) DO NOTHING",
                )
                .bind(id)
                .bind(asset_type)
                .bind(hash)
                .bind(now)
                .bind(now)
                .bind(asset_flags)
                .execute(pool)
                .await?;
                Ok(())
            }
            _ => Err(anyhow!("FSAssets migration currently requires PostgreSQL")),
        }
    }

    async fn insert_fsassets_metadata(
        &self,
        id: &str,
        asset_type: i32,
        hash: &str,
        create_time: i32,
        asset_flags: i32,
    ) -> Result<()> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query(
                    "INSERT INTO fsassets (id, type, hash, create_time, access_time, asset_flags) \
                     VALUES ($1::uuid, $2, $3, $4, $5, $6) \
                     ON CONFLICT (id) DO UPDATE SET hash = EXCLUDED.hash",
                )
                .bind(id)
                .bind(asset_type)
                .bind(hash)
                .bind(create_time)
                .bind(create_time)
                .bind(asset_flags)
                .execute(pool)
                .await?;
                Ok(())
            }
            _ => Err(anyhow!("FSAssets migration currently requires PostgreSQL")),
        }
    }

    pub async fn verify_migration(&self) -> Result<(i64, i64, Vec<String>)> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => {
                let (asset_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM assets")
                    .fetch_one(pool)
                    .await?;
                let (fs_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fsassets")
                    .fetch_one(pool)
                    .await?;

                let mut issues = Vec::new();

                if fs_count < asset_count {
                    issues.push(format!(
                        "fsassets has {} rows but assets has {} — {} assets not migrated",
                        fs_count,
                        asset_count,
                        asset_count - fs_count
                    ));
                }

                use sqlx::Row;
                let missing_files: Vec<sqlx::postgres::PgRow> = sqlx::query(
                    "SELECT f.id::text, f.hash FROM fsassets f \
                     ORDER BY RANDOM() LIMIT 10",
                )
                .fetch_all(pool)
                .await?;

                for row in &missing_files {
                    let id: String = row.try_get("id")?;
                    let hash: String = row.try_get("hash")?;
                    let hash = hash.trim().to_string();
                    if !self.storage.exists(&hash).await {
                        issues.push(format!(
                            "Asset {} hash {} not found on filesystem",
                            id, hash
                        ));
                    }
                }

                info!(
                    "Migration verification: assets={}, fsassets={}, issues={}",
                    asset_count,
                    fs_count,
                    issues.len()
                );

                Ok((asset_count, fs_count, issues))
            }
            _ => Err(anyhow!("Verification currently requires PostgreSQL")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_stats_default() {
        let stats = MigrationStats {
            total_assets: 100,
            migrated: 80,
            skipped_existing: 15,
            skipped_null_data: 3,
            errors: 2,
            bytes_processed: 1024 * 1024 * 50,
            elapsed_secs: 30.0,
        };
        assert_eq!(
            stats.migrated + stats.skipped_existing + stats.skipped_null_data + stats.errors,
            100
        );
    }

    #[test]
    fn test_hash_consistency_with_fsassets() {
        let data = b"migration test data";
        let hash = FSAssetsStorage::compute_hash(data);
        assert_eq!(hash.len(), 64);
        let hash2 = FSAssetsStorage::compute_hash(data);
        assert_eq!(hash, hash2);
    }
}
