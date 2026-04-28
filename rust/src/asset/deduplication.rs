//! Asset Deduplication Module
//!
//! Provides asset deduplication capabilities to reduce storage usage
//! by detecting and eliminating duplicate assets across the system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use super::{AssetData, AssetType};

/// Deduplication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationConfig {
    /// Enable asset deduplication
    pub enabled: bool,
    /// Hash algorithm to use for deduplication
    pub hash_algorithm: HashAlgorithm,
    /// Minimum asset size for deduplication (bytes)
    pub min_asset_size: usize,
    /// Maximum asset size for deduplication (bytes)
    pub max_asset_size: usize,
    /// Enable content-based deduplication (slower but more accurate)
    pub enable_content_dedup: bool,
    /// Enable perceptual hashing for images
    pub enable_perceptual_hash: bool,
    /// Similarity threshold for perceptual hashing (0.0-1.0)
    pub similarity_threshold: f32,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hash_algorithm: HashAlgorithm::Blake3,
            min_asset_size: 1024,              // 1KB
            max_asset_size: 100 * 1024 * 1024, // 100MB
            enable_content_dedup: true,
            enable_perceptual_hash: false,
            similarity_threshold: 0.95,
        }
    }
}

/// Hash algorithms supported for deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
    Xxhash,
}

/// Asset hash information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetHash {
    pub asset_id: Uuid,
    pub hash: String,
    pub algorithm: HashAlgorithm,
    pub size_bytes: usize,
    pub asset_type: AssetType,
    pub created_at: std::time::SystemTime,
    pub reference_count: u32,
}

/// Duplicate asset group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub canonical_id: Uuid,
    pub duplicate_ids: Vec<Uuid>,
    pub hash: String,
    pub size_bytes: usize,
    pub savings_bytes: usize,
    pub asset_type: AssetType,
}

/// Deduplication statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeduplicationStats {
    pub total_assets_processed: u64,
    pub duplicate_assets_found: u64,
    pub storage_saved_bytes: u64,
    pub deduplication_ratio: f32,
    pub last_dedup_run: Option<std::time::SystemTime>,
    pub processing_time_ms: u64,
}

/// Asset deduplication manager
pub struct AssetDeduplicationManager {
    config: DeduplicationConfig,
    asset_hashes: Arc<RwLock<HashMap<String, AssetHash>>>,
    duplicate_groups: Arc<RwLock<HashMap<String, DuplicateGroup>>>,
    asset_references: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>, // asset_id -> list of referencing assets
    dedup_stats: Arc<RwLock<DeduplicationStats>>,
}

impl AssetDeduplicationManager {
    /// Create a new asset deduplication manager
    pub fn new(config: DeduplicationConfig) -> Self {
        Self {
            config,
            asset_hashes: Arc::new(RwLock::new(HashMap::new())),
            duplicate_groups: Arc::new(RwLock::new(HashMap::new())),
            asset_references: Arc::new(RwLock::new(HashMap::new())),
            dedup_stats: Arc::new(RwLock::new(DeduplicationStats::default())),
        }
    }

    /// Process an asset for deduplication
    pub async fn process_asset(
        &self,
        asset_id: Uuid,
        asset_type: AssetType,
        data: &AssetData,
    ) -> Result<DeduplicationResult> {
        if !self.config.enabled {
            return Ok(DeduplicationResult::NotProcessed);
        }

        // Check size constraints
        if data.len() < self.config.min_asset_size || data.len() > self.config.max_asset_size {
            return Ok(DeduplicationResult::SkippedSize);
        }

        let start_time = std::time::Instant::now();

        // Calculate hash
        let hash = self.calculate_hash(data)?;

        // Check for existing asset with same hash
        let mut asset_hashes = self.asset_hashes.write().await;

        if let Some(existing_hash) = asset_hashes.get(&hash) {
            // Found duplicate
            let duplicate_id = existing_hash.asset_id;

            // Update reference count
            let mut existing_hash = existing_hash.clone();
            existing_hash.reference_count += 1;
            asset_hashes.insert(hash.clone(), existing_hash);

            // Update duplicate groups
            self.add_to_duplicate_group(
                hash.clone(),
                asset_id,
                duplicate_id,
                data.len(),
                asset_type,
            )
            .await;

            // Update statistics
            self.update_stats_duplicate_found(data.len()).await;

            info!(
                "Duplicate asset detected: {} is duplicate of {}",
                asset_id, duplicate_id
            );

            Ok(DeduplicationResult::Duplicate {
                canonical_id: duplicate_id,
                savings_bytes: data.len(),
            })
        } else {
            // New unique asset
            let asset_hash = AssetHash {
                asset_id,
                hash: hash.clone(),
                algorithm: self.config.hash_algorithm.clone(),
                size_bytes: data.len(),
                asset_type,
                created_at: std::time::SystemTime::now(),
                reference_count: 1,
            };

            asset_hashes.insert(hash, asset_hash);

            // Update statistics
            self.update_stats_processed().await;

            debug!("New unique asset processed: {}", asset_id);

            Ok(DeduplicationResult::Unique)
        }
    }

    /// Remove an asset from deduplication tracking
    pub async fn remove_asset(&self, asset_id: &Uuid) -> Result<()> {
        let mut asset_hashes = self.asset_hashes.write().await;

        // Find and remove the asset hash
        let hash_to_remove = asset_hashes
            .iter()
            .find(|(_, hash_info)| hash_info.asset_id == *asset_id)
            .map(|(hash, _)| hash.clone());

        if let Some(hash) = hash_to_remove {
            if let Some(mut asset_hash) = asset_hashes.get(&hash).cloned() {
                asset_hash.reference_count = asset_hash.reference_count.saturating_sub(1);

                if asset_hash.reference_count == 0 {
                    // Remove completely if no references
                    asset_hashes.remove(&hash);

                    // Remove from duplicate groups
                    let mut duplicate_groups = self.duplicate_groups.write().await;
                    duplicate_groups.remove(&hash);
                } else {
                    // Update reference count
                    asset_hashes.insert(hash, asset_hash);
                }
            }
        }

        // Remove references
        let mut references = self.asset_references.write().await;
        references.remove(asset_id);

        Ok(())
    }

    /// Get duplicate groups for cleanup
    pub async fn get_duplicate_groups(&self) -> Vec<DuplicateGroup> {
        let duplicate_groups = self.duplicate_groups.read().await;
        duplicate_groups.values().cloned().collect()
    }

    /// Get potential storage savings
    pub async fn get_potential_savings(&self) -> u64 {
        let duplicate_groups = self.duplicate_groups.read().await;
        duplicate_groups
            .values()
            .map(|group| group.savings_bytes as u64)
            .sum()
    }

    /// Perform full deduplication scan
    pub async fn perform_deduplication_scan<F, Fut>(
        &self,
        asset_provider: F,
    ) -> Result<DeduplicationScanResult>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<Vec<(Uuid, AssetType, AssetData)>>>,
    {
        let start_time = std::time::Instant::now();
        info!("Starting full deduplication scan");

        // Get all assets
        let assets = asset_provider().await?;
        let total_assets = assets.len();

        let mut processed = 0;
        let mut duplicates_found = 0;
        let mut total_savings = 0;

        for (asset_id, asset_type, data) in assets {
            match self.process_asset(asset_id, asset_type, &data).await? {
                DeduplicationResult::Duplicate { savings_bytes, .. } => {
                    duplicates_found += 1;
                    total_savings += savings_bytes;
                }
                _ => {}
            }
            processed += 1;

            if processed % 1000 == 0 {
                info!(
                    "Deduplication scan progress: {}/{} assets processed",
                    processed, total_assets
                );
            }
        }

        let duration = start_time.elapsed();

        // Update final statistics
        {
            let mut stats = self.dedup_stats.write().await;
            stats.last_dedup_run = Some(std::time::SystemTime::now());
            stats.processing_time_ms = duration.as_millis() as u64;
        }

        info!(
            "Deduplication scan completed: {} duplicates found, {} bytes saved in {:?}",
            duplicates_found, total_savings, duration
        );

        Ok(DeduplicationScanResult {
            total_assets_scanned: total_assets,
            duplicates_found,
            storage_saved_bytes: total_savings,
            scan_duration: duration,
        })
    }

    /// Calculate hash for asset data
    fn calculate_hash(&self, data: &AssetData) -> Result<String> {
        match self.config.hash_algorithm {
            HashAlgorithm::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlgorithm::Blake3 => {
                // For now, use SHA256 as placeholder
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(format!("blake3:{:x}", hasher.finalize()))
            }
            HashAlgorithm::Xxhash => {
                // For now, use SHA256 as placeholder
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(format!("xxhash:{:x}", hasher.finalize()))
            }
        }
    }

    /// Add asset to duplicate group
    async fn add_to_duplicate_group(
        &self,
        hash: String,
        new_asset_id: Uuid,
        canonical_id: Uuid,
        size: usize,
        asset_type: AssetType,
    ) {
        let mut duplicate_groups = self.duplicate_groups.write().await;

        if let Some(group) = duplicate_groups.get_mut(&hash) {
            group.duplicate_ids.push(new_asset_id);
            group.savings_bytes += size;
        } else {
            let group = DuplicateGroup {
                canonical_id,
                duplicate_ids: vec![new_asset_id],
                hash,
                size_bytes: size,
                savings_bytes: size, // First duplicate saves the full size
                asset_type,
            };
            duplicate_groups.insert(group.hash.clone(), group);
        }
    }

    /// Update statistics for processed asset
    async fn update_stats_processed(&self) {
        let mut stats = self.dedup_stats.write().await;
        stats.total_assets_processed += 1;
    }

    /// Update statistics for duplicate found
    async fn update_stats_duplicate_found(&self, savings_bytes: usize) {
        let mut stats = self.dedup_stats.write().await;
        stats.total_assets_processed += 1;
        stats.duplicate_assets_found += 1;
        stats.storage_saved_bytes += savings_bytes as u64;

        // Calculate deduplication ratio
        if stats.total_assets_processed > 0 {
            stats.deduplication_ratio =
                stats.duplicate_assets_found as f32 / stats.total_assets_processed as f32;
        }
    }

    /// Get deduplication statistics
    pub async fn get_stats(&self) -> DeduplicationStats {
        let stats = self.dedup_stats.read().await;
        stats.clone()
    }

    /// Clear all deduplication data
    pub async fn clear(&self) {
        let mut asset_hashes = self.asset_hashes.write().await;
        let mut duplicate_groups = self.duplicate_groups.write().await;
        let mut references = self.asset_references.write().await;

        asset_hashes.clear();
        duplicate_groups.clear();
        references.clear();

        info!("Deduplication data cleared");
    }
}

/// Result of asset deduplication processing
#[derive(Debug, Clone)]
pub enum DeduplicationResult {
    /// Asset is unique (not a duplicate)
    Unique,
    /// Asset is a duplicate of existing asset
    Duplicate {
        canonical_id: Uuid,
        savings_bytes: usize,
    },
    /// Asset was not processed (deduplication disabled)
    NotProcessed,
    /// Asset was skipped due to size constraints
    SkippedSize,
}

/// Result of full deduplication scan
#[derive(Debug, Clone)]
pub struct DeduplicationScanResult {
    pub total_assets_scanned: usize,
    pub duplicates_found: usize,
    pub storage_saved_bytes: usize,
    pub scan_duration: std::time::Duration,
}

/// Utility functions for asset deduplication
pub mod utils {
    use super::*;

    /// Calculate potential savings for a list of assets
    pub fn calculate_potential_savings(assets: &[(Uuid, AssetType, AssetData)]) -> usize {
        let mut hash_map: HashMap<String, usize> = HashMap::new();
        let mut total_savings = 0;

        for (_, _, data) in assets {
            // Simple hash calculation for estimation
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(data);
            let hash = format!("{:x}", hasher.finalize());

            if let Some(existing_size) = hash_map.get(&hash) {
                // This is a duplicate, add to savings
                total_savings += data.len();
            } else {
                hash_map.insert(hash, data.len());
            }
        }

        total_savings
    }

    /// Estimate deduplication ratio for asset collection
    pub fn estimate_deduplication_ratio(assets: &[(Uuid, AssetType, AssetData)]) -> f32 {
        if assets.is_empty() {
            return 0.0;
        }

        let mut hash_set = std::collections::HashSet::new();
        let mut duplicates = 0;

        for (_, _, data) in assets {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(data);
            let hash = format!("{:x}", hasher.finalize());

            if !hash_set.insert(hash) {
                duplicates += 1;
            }
        }

        duplicates as f32 / assets.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deduplication_unique_asset() {
        let config = DeduplicationConfig::default();
        let manager = AssetDeduplicationManager::new(config);

        let asset_id = Uuid::new_v4();
        let asset_data = b"unique asset data".to_vec();

        let result = manager
            .process_asset(asset_id, AssetType::Texture, &asset_data)
            .await
            .unwrap();

        match result {
            DeduplicationResult::Unique => {}
            _ => panic!("Expected unique result"),
        }

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_assets_processed, 1);
        assert_eq!(stats.duplicate_assets_found, 0);
    }

    #[tokio::test]
    async fn test_deduplication_duplicate_asset() {
        let config = DeduplicationConfig::default();
        let manager = AssetDeduplicationManager::new(config);

        let asset_id1 = Uuid::new_v4();
        let asset_id2 = Uuid::new_v4();
        let asset_data = b"duplicate asset data".to_vec();

        // Process first asset (should be unique)
        let result1 = manager
            .process_asset(asset_id1, AssetType::Texture, &asset_data)
            .await
            .unwrap();
        assert!(matches!(result1, DeduplicationResult::Unique));

        // Process second asset with same data (should be duplicate)
        let result2 = manager
            .process_asset(asset_id2, AssetType::Texture, &asset_data)
            .await
            .unwrap();

        match result2 {
            DeduplicationResult::Duplicate {
                canonical_id,
                savings_bytes,
            } => {
                assert_eq!(canonical_id, asset_id1);
                assert_eq!(savings_bytes, asset_data.len());
            }
            _ => panic!("Expected duplicate result"),
        }

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_assets_processed, 2);
        assert_eq!(stats.duplicate_assets_found, 1);
        assert_eq!(stats.storage_saved_bytes, asset_data.len() as u64);
    }

    #[tokio::test]
    async fn test_deduplication_size_limits() {
        let mut config = DeduplicationConfig::default();
        config.min_asset_size = 10;
        config.max_asset_size = 20;

        let manager = AssetDeduplicationManager::new(config);

        let asset_id = Uuid::new_v4();

        // Too small
        let small_data = b"small".to_vec(); // 5 bytes
        let result = manager
            .process_asset(asset_id, AssetType::Texture, &small_data)
            .await
            .unwrap();
        assert!(matches!(result, DeduplicationResult::SkippedSize));

        // Too large
        let large_data = vec![0u8; 25]; // 25 bytes
        let result = manager
            .process_asset(asset_id, AssetType::Texture, &large_data)
            .await
            .unwrap();
        assert!(matches!(result, DeduplicationResult::SkippedSize));

        // Just right
        let right_data = vec![0u8; 15]; // 15 bytes
        let result = manager
            .process_asset(asset_id, AssetType::Texture, &right_data)
            .await
            .unwrap();
        assert!(matches!(result, DeduplicationResult::Unique));
    }

    #[test]
    fn test_potential_savings_calculation() {
        let assets = vec![
            (Uuid::new_v4(), AssetType::Texture, b"data1".to_vec()),
            (Uuid::new_v4(), AssetType::Texture, b"data2".to_vec()),
            (Uuid::new_v4(), AssetType::Texture, b"data1".to_vec()), // duplicate
            (Uuid::new_v4(), AssetType::Texture, b"data3".to_vec()),
            (Uuid::new_v4(), AssetType::Texture, b"data2".to_vec()), // duplicate
        ];

        let savings = utils::calculate_potential_savings(&assets);
        assert_eq!(savings, 10); // 5 bytes for each duplicate

        let ratio = utils::estimate_deduplication_ratio(&assets);
        assert_eq!(ratio, 0.4); // 2 duplicates out of 5 assets
    }
}
