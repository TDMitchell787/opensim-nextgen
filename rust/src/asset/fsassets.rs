use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::AssetData;

#[derive(Debug, Clone)]
pub struct FSAssetsConfig {
    pub base_directory: PathBuf,
    pub spool_directory: PathBuf,
    pub use_osgrid_format: bool,
    pub compression_level: u32,
}

impl Default for FSAssetsConfig {
    fn default() -> Self {
        Self {
            base_directory: PathBuf::from("./fsassets/data"),
            spool_directory: PathBuf::from("./fsassets/tmp"),
            use_osgrid_format: false,
            compression_level: 6,
        }
    }
}

#[derive(Debug, Default)]
pub struct FSAssetsStats {
    pub reads: AtomicU64,
    pub read_bytes: AtomicU64,
    pub writes: AtomicU64,
    pub write_bytes: AtomicU64,
    pub dedup_hits: AtomicU64,
    pub spool_reads: AtomicU64,
    pub gz_reads: AtomicU64,
    pub missing: AtomicU64,
    pub compressions: AtomicU64,
}

pub struct FSAssetsStorage {
    config: FSAssetsConfig,
    stats: Arc<FSAssetsStats>,
    spool_notify: Arc<Notify>,
}

impl FSAssetsStorage {
    pub fn new(config: FSAssetsConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.base_directory)?;
        std::fs::create_dir_all(&config.spool_directory)?;
        std::fs::create_dir_all(config.spool_directory.join("spool"))?;

        Ok(Self {
            config,
            stats: Arc::new(FSAssetsStats::default()),
            spool_notify: Arc::new(Notify::new()),
        })
    }

    pub fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    pub fn hash_to_path(&self, hash: &str) -> PathBuf {
        if self.config.use_osgrid_format {
            self.config
                .base_directory
                .join(&hash[0..3])
                .join(&hash[3..6])
                .join(format!("{}.gz", hash))
        } else {
            self.config
                .base_directory
                .join(&hash[0..2])
                .join(&hash[2..4])
                .join(&hash[4..6])
                .join(&hash[6..10])
                .join(format!("{}.gz", hash))
        }
    }

    fn spool_path(&self, hash: &str) -> PathBuf {
        self.config
            .spool_directory
            .join("spool")
            .join(format!("{}.asset", hash))
    }

    fn staging_path(&self, hash: &str) -> PathBuf {
        self.config.spool_directory.join(format!("{}.asset", hash))
    }

    pub async fn store(&self, data: &[u8]) -> Result<String> {
        let hash = Self::compute_hash(data);

        let final_path = self.hash_to_path(&hash);
        if final_path.exists() {
            self.stats.dedup_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(hash);
        }

        let staging = self.staging_path(&hash);
        if staging.exists() {
            self.stats.dedup_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(hash);
        }

        let spool = self.spool_path(&hash);
        tokio::fs::write(&spool, data).await?;

        let staging_dir = self.config.spool_directory.clone();
        tokio::fs::rename(&spool, staging_dir.join(format!("{}.asset", hash))).await?;

        self.stats.writes.fetch_add(1, Ordering::Relaxed);
        self.stats
            .write_bytes
            .fetch_add(data.len() as u64, Ordering::Relaxed);

        self.spool_notify.notify_one();

        Ok(hash)
    }

    pub async fn retrieve(&self, hash: &str) -> Result<Option<Vec<u8>>> {
        self.stats.reads.fetch_add(1, Ordering::Relaxed);

        let staging = self.staging_path(hash);
        if staging.exists() {
            match tokio::fs::read(&staging).await {
                Ok(data) => {
                    self.stats.spool_reads.fetch_add(1, Ordering::Relaxed);
                    self.stats
                        .read_bytes
                        .fetch_add(data.len() as u64, Ordering::Relaxed);
                    return Ok(Some(data));
                }
                Err(e) => {
                    debug!("Failed to read spool file {}: {}", staging.display(), e);
                }
            }
        }

        let gz_path = self.hash_to_path(hash);
        if gz_path.exists() {
            match tokio::fs::read(&gz_path).await {
                Ok(compressed) => {
                    let decompressed = tokio::task::spawn_blocking(move || {
                        use std::io::Read;
                        let mut decoder = GzDecoder::new(&compressed[..]);
                        let mut buf = Vec::new();
                        decoder.read_to_end(&mut buf)?;
                        Ok::<Vec<u8>, std::io::Error>(buf)
                    })
                    .await??;
                    self.stats.gz_reads.fetch_add(1, Ordering::Relaxed);
                    self.stats
                        .read_bytes
                        .fetch_add(decompressed.len() as u64, Ordering::Relaxed);
                    return Ok(Some(decompressed));
                }
                Err(e) => {
                    error!("Failed to read gz file {}: {}", gz_path.display(), e);
                }
            }
        }

        self.stats.missing.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }

    pub async fn exists(&self, hash: &str) -> bool {
        let staging = self.staging_path(hash);
        if staging.exists() {
            return true;
        }
        let gz_path = self.hash_to_path(hash);
        gz_path.exists()
    }

    pub fn notify_spool(&self) {
        self.spool_notify.notify_one();
    }

    pub fn start_background_writer(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let storage = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                storage.spool_notify.notified().await;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                if let Err(e) = storage.compress_spool_files().await {
                    error!("Background writer error: {}", e);
                }
            }
        })
    }

    async fn compress_spool_files(&self) -> Result<()> {
        let spool_dir = &self.config.spool_directory;
        let mut entries = tokio::fs::read_dir(spool_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) if name.ends_with(".asset") => name.to_string(),
                _ => continue,
            };
            let hash = file_name.trim_end_matches(".asset").to_string();
            if hash.len() != 64 {
                continue;
            }

            let final_path = self.hash_to_path(&hash);
            if final_path.exists() {
                let _ = tokio::fs::remove_file(&path).await;
                continue;
            }

            let data = match tokio::fs::read(&path).await {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to read spool file {}: {}", path.display(), e);
                    continue;
                }
            };

            let data_len = data.len();
            let level = self.config.compression_level;
            let compressed = tokio::task::spawn_blocking(move || {
                use std::io::Write;
                let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
                encoder.write_all(&data)?;
                encoder.finish()
            })
            .await??;

            if let Some(parent) = final_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            let tmp_gz = final_path.with_extension("gz.tmp");
            tokio::fs::write(&tmp_gz, &compressed).await?;
            tokio::fs::rename(&tmp_gz, &final_path).await?;
            tokio::fs::remove_file(&path).await?;

            self.stats.compressions.fetch_add(1, Ordering::Relaxed);
            debug!(
                "Compressed {} ({} -> {} bytes)",
                hash,
                data_len,
                compressed.len()
            );
        }

        Ok(())
    }

    pub fn get_stats(&self) -> FSAssetsStatsSnapshot {
        FSAssetsStatsSnapshot {
            reads: self.stats.reads.load(Ordering::Relaxed),
            read_bytes: self.stats.read_bytes.load(Ordering::Relaxed),
            writes: self.stats.writes.load(Ordering::Relaxed),
            write_bytes: self.stats.write_bytes.load(Ordering::Relaxed),
            dedup_hits: self.stats.dedup_hits.load(Ordering::Relaxed),
            spool_reads: self.stats.spool_reads.load(Ordering::Relaxed),
            gz_reads: self.stats.gz_reads.load(Ordering::Relaxed),
            missing: self.stats.missing.load(Ordering::Relaxed),
            compressions: self.stats.compressions.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FSAssetsStatsSnapshot {
    pub reads: u64,
    pub read_bytes: u64,
    pub writes: u64,
    pub write_bytes: u64,
    pub dedup_hits: u64,
    pub spool_reads: u64,
    pub gz_reads: u64,
    pub missing: u64,
    pub compressions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config(dir: &TempDir) -> FSAssetsConfig {
        FSAssetsConfig {
            base_directory: dir.path().join("data"),
            spool_directory: dir.path().join("tmp"),
            use_osgrid_format: false,
            compression_level: 6,
        }
    }

    #[test]
    fn test_compute_hash() {
        let hash = FSAssetsStorage::compute_hash(b"hello world");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_to_path_standard() {
        let dir = TempDir::new().unwrap();
        let config = test_config(&dir);
        let storage = FSAssetsStorage::new(config.clone()).unwrap();
        let hash = "a1b2c3d4e5f60000000000000000000000000000000000000000000000000000";
        let path = storage.hash_to_path(hash);
        let expected = config
            .base_directory
            .join("a1")
            .join("b2")
            .join("c3")
            .join("d4e5")
            .join(format!("{}.gz", hash));
        assert_eq!(path, expected);
    }

    #[test]
    fn test_hash_to_path_osgrid() {
        let dir = TempDir::new().unwrap();
        let mut config = test_config(&dir);
        config.use_osgrid_format = true;
        let storage = FSAssetsStorage::new(config.clone()).unwrap();
        let hash = "a1b2c3d4e5f60000000000000000000000000000000000000000000000000000";
        let path = storage.hash_to_path(hash);
        let expected = config
            .base_directory
            .join("a1b")
            .join("2c3")
            .join(format!("{}.gz", hash));
        assert_eq!(path, expected);
    }

    #[test]
    fn test_dedup_same_data() {
        let hash1 = FSAssetsStorage::compute_hash(b"identical data");
        let hash2 = FSAssetsStorage::compute_hash(b"identical data");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_data_different_hash() {
        let hash1 = FSAssetsStorage::compute_hash(b"data one");
        let hash2 = FSAssetsStorage::compute_hash(b"data two");
        assert_ne!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_from_spool() {
        let dir = TempDir::new().unwrap();
        let config = test_config(&dir);
        let storage = FSAssetsStorage::new(config).unwrap();

        let data = b"test asset data for spool retrieval";
        let hash = storage.store(data).await.unwrap();

        assert_eq!(hash, FSAssetsStorage::compute_hash(data));

        let retrieved = storage.retrieve(&hash).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), data.to_vec());
    }

    #[tokio::test]
    async fn test_store_dedup() {
        let dir = TempDir::new().unwrap();
        let config = test_config(&dir);
        let storage = FSAssetsStorage::new(config).unwrap();

        let data = b"duplicate test data";
        let hash1 = storage.store(data).await.unwrap();
        let hash2 = storage.store(data).await.unwrap();

        assert_eq!(hash1, hash2);
        let stats = storage.get_stats();
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.dedup_hits, 1);
    }

    #[tokio::test]
    async fn test_retrieve_missing() {
        let dir = TempDir::new().unwrap();
        let config = test_config(&dir);
        let storage = FSAssetsStorage::new(config).unwrap();

        let result = storage
            .retrieve("0000000000000000000000000000000000000000000000000000000000000000")
            .await
            .unwrap();
        assert!(result.is_none());

        let stats = storage.get_stats();
        assert_eq!(stats.missing, 1);
    }

    #[tokio::test]
    async fn test_exists() {
        let dir = TempDir::new().unwrap();
        let config = test_config(&dir);
        let storage = FSAssetsStorage::new(config).unwrap();

        let data = b"existence test data";
        let hash = storage.store(data).await.unwrap();

        assert!(storage.exists(&hash).await);
        assert!(
            !storage
                .exists("0000000000000000000000000000000000000000000000000000000000000000")
                .await
        );
    }

    #[tokio::test]
    async fn test_background_compression() {
        let dir = TempDir::new().unwrap();
        let config = test_config(&dir);
        let storage = Arc::new(FSAssetsStorage::new(config.clone()).unwrap());

        let data = b"data to compress in background writer";
        let hash = storage.store(data).await.unwrap();

        storage.compress_spool_files().await.unwrap();

        let gz_path = storage.hash_to_path(&hash);
        assert!(gz_path.exists(), "GZ file should exist after compression");

        let staging = storage.staging_path(&hash);
        assert!(
            !staging.exists(),
            "Staging file should be removed after compression"
        );

        let retrieved = storage.retrieve(&hash).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), data.to_vec());

        let stats = storage.get_stats();
        assert_eq!(stats.compressions, 1);
        assert_eq!(stats.gz_reads, 1);
    }

    #[tokio::test]
    async fn test_large_asset_roundtrip() {
        let dir = TempDir::new().unwrap();
        let config = test_config(&dir);
        let storage = Arc::new(FSAssetsStorage::new(config).unwrap());

        let data: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
        let hash = storage.store(&data).await.unwrap();

        storage.compress_spool_files().await.unwrap();

        let retrieved = storage.retrieve(&hash).await.unwrap().unwrap();
        assert_eq!(retrieved.len(), data.len());
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_gzip_roundtrip() {
        use std::io::{Read, Write};
        let original = b"test gzip compression roundtrip data";
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(6));
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        assert_eq!(decompressed, original);
    }
}
