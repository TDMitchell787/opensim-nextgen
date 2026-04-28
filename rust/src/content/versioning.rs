//! Content Versioning for OpenSim Next
//!
//! Provides version control, change tracking, and rollback capabilities.

use super::{ContentError, ContentMetadata, ContentResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

mod version_serde {
    use semver::Version;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        version.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentVersion {
    pub version_id: Uuid,
    pub content_id: Uuid,
    #[serde(with = "version_serde")]
    pub version_number: semver::Version,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub change_description: String,
    pub file_checksum: String,
    pub file_size: u64,
    pub is_current: bool,
    pub parent_version_id: Option<Uuid>,
    pub data_snapshot: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub from_version: Uuid,
    pub to_version: Uuid,
    pub changes: Vec<VersionChange>,
    pub diff_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    pub change_type: ChangeType,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

pub struct ContentVersionManager {
    versions: Arc<RwLock<HashMap<Uuid, Vec<ContentVersion>>>>,
    version_snapshots: Arc<RwLock<HashMap<Uuid, Vec<u8>>>>,
    max_versions_per_content: usize,
}

impl ContentVersionManager {
    pub fn new() -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
            version_snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_versions_per_content: 100,
        }
    }

    pub fn with_max_versions(max_versions: usize) -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
            version_snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_versions_per_content: max_versions,
        }
    }

    pub async fn create_version(
        &mut self,
        content_id: Uuid,
        metadata: &ContentMetadata,
        change_description: String,
    ) -> ContentResult<Uuid> {
        let version_id = Uuid::new_v4();
        let now = Utc::now();

        let mut versions = self.versions.write().await;
        let content_versions = versions.entry(content_id).or_insert_with(Vec::new);

        for v in content_versions.iter_mut() {
            v.is_current = false;
        }

        let (new_version_number, parent_version_id) = if let Some(latest) = content_versions.last()
        {
            let mut ver = latest.version_number.clone();
            ver.patch += 1;
            (ver, Some(latest.version_id))
        } else {
            (semver::Version::new(1, 0, 0), None)
        };

        let checksum = self.calculate_metadata_checksum(metadata);

        let version = ContentVersion {
            version_id,
            content_id,
            version_number: new_version_number.clone(),
            created_at: now,
            created_by: metadata.creator_id,
            change_description: change_description.clone(),
            file_checksum: checksum,
            file_size: metadata.file_size as u64,
            is_current: true,
            parent_version_id,
            data_snapshot: None,
        };

        content_versions.push(version);

        if content_versions.len() > self.max_versions_per_content {
            let remove_count = content_versions.len() - self.max_versions_per_content;
            for i in 0..remove_count {
                let old_version_id = content_versions[i].version_id;
                let mut snapshots = self.version_snapshots.write().await;
                snapshots.remove(&old_version_id);
            }
            content_versions.drain(0..remove_count);
        }

        info!(
            "Created version {} ({}) for content {}: {}",
            version_id, new_version_number, content_id, change_description
        );

        Ok(version_id)
    }

    pub async fn create_version_with_data(
        &mut self,
        content_id: Uuid,
        metadata: &ContentMetadata,
        change_description: String,
        data: Vec<u8>,
    ) -> ContentResult<Uuid> {
        let version_id = self
            .create_version(content_id, metadata, change_description)
            .await?;

        let mut snapshots = self.version_snapshots.write().await;
        snapshots.insert(version_id, data);

        debug!("Stored data snapshot for version {}", version_id);
        Ok(version_id)
    }

    pub async fn get_version_history(
        &self,
        content_id: Uuid,
    ) -> ContentResult<Vec<ContentVersion>> {
        let versions = self.versions.read().await;

        if let Some(content_versions) = versions.get(&content_id) {
            let mut result: Vec<ContentVersion> = content_versions.clone();
            result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            debug!(
                "Retrieved {} versions for content {}",
                result.len(),
                content_id
            );
            Ok(result)
        } else {
            debug!("No version history for content {}", content_id);
            Ok(Vec::new())
        }
    }

    pub async fn get_version(
        &self,
        content_id: Uuid,
        version_id: Uuid,
    ) -> ContentResult<ContentVersion> {
        let versions = self.versions.read().await;

        if let Some(content_versions) = versions.get(&content_id) {
            content_versions
                .iter()
                .find(|v| v.version_id == version_id)
                .cloned()
                .ok_or_else(|| ContentError::NotFound { id: version_id })
        } else {
            Err(ContentError::NotFound { id: content_id })
        }
    }

    pub async fn get_current_version(&self, content_id: Uuid) -> ContentResult<ContentVersion> {
        let versions = self.versions.read().await;

        if let Some(content_versions) = versions.get(&content_id) {
            content_versions
                .iter()
                .find(|v| v.is_current)
                .cloned()
                .ok_or_else(|| ContentError::NotFound { id: content_id })
        } else {
            Err(ContentError::NotFound { id: content_id })
        }
    }

    pub async fn get_version_data(&self, version_id: Uuid) -> ContentResult<Vec<u8>> {
        let snapshots = self.version_snapshots.read().await;

        snapshots
            .get(&version_id)
            .cloned()
            .ok_or_else(|| ContentError::NotFound { id: version_id })
    }

    pub async fn rollback_to_version(
        &mut self,
        content_id: Uuid,
        version_id: Uuid,
    ) -> ContentResult<()> {
        let mut versions = self.versions.write().await;

        if let Some(content_versions) = versions.get_mut(&content_id) {
            let target_idx = content_versions
                .iter()
                .position(|v| v.version_id == version_id)
                .ok_or_else(|| ContentError::NotFound { id: version_id })?;

            for v in content_versions.iter_mut() {
                v.is_current = false;
            }

            let mut rollback_version = content_versions[target_idx].clone();
            rollback_version.version_id = Uuid::new_v4();
            rollback_version.created_at = Utc::now();
            rollback_version.is_current = true;
            rollback_version.parent_version_id = content_versions.last().map(|v| v.version_id);
            rollback_version.change_description = format!(
                "Rollback to version {}",
                content_versions[target_idx].version_number
            );

            if let Some(latest) = content_versions.last() {
                let mut new_ver = latest.version_number.clone();
                new_ver.patch += 1;
                rollback_version.version_number = new_ver;
            }

            let new_version_id = rollback_version.version_id;

            if let Some(original_data) = {
                let snapshots = self.version_snapshots.read().await;
                snapshots.get(&version_id).cloned()
            } {
                let mut snapshots = self.version_snapshots.write().await;
                snapshots.insert(new_version_id, original_data);
            }

            content_versions.push(rollback_version.clone());

            info!(
                "Rolled back content {} to version {}, created new version {}",
                content_id, version_id, new_version_id
            );

            Ok(())
        } else {
            Err(ContentError::NotFound { id: content_id })
        }
    }

    pub async fn compare_versions(
        &self,
        content_id: Uuid,
        from_version_id: Uuid,
        to_version_id: Uuid,
    ) -> ContentResult<VersionDiff> {
        let from_version = self.get_version(content_id, from_version_id).await?;
        let to_version = self.get_version(content_id, to_version_id).await?;

        let mut changes = Vec::new();

        if from_version.file_size != to_version.file_size {
            changes.push(VersionChange {
                change_type: ChangeType::Modified,
                field_name: "file_size".to_string(),
                old_value: Some(from_version.file_size.to_string()),
                new_value: Some(to_version.file_size.to_string()),
            });
        }

        if from_version.file_checksum != to_version.file_checksum {
            changes.push(VersionChange {
                change_type: ChangeType::Modified,
                field_name: "content".to_string(),
                old_value: Some(from_version.file_checksum.clone()),
                new_value: Some(to_version.file_checksum.clone()),
            });
        }

        let diff_size =
            (to_version.file_size as i64 - from_version.file_size as i64).unsigned_abs();

        Ok(VersionDiff {
            from_version: from_version_id,
            to_version: to_version_id,
            changes,
            diff_size_bytes: diff_size,
        })
    }

    pub async fn delete_version(
        &mut self,
        content_id: Uuid,
        version_id: Uuid,
    ) -> ContentResult<()> {
        let mut versions = self.versions.write().await;

        if let Some(content_versions) = versions.get_mut(&content_id) {
            let idx = content_versions
                .iter()
                .position(|v| v.version_id == version_id)
                .ok_or_else(|| ContentError::NotFound { id: version_id })?;

            if content_versions[idx].is_current && content_versions.len() > 1 {
                return Err(ContentError::InvalidOperation {
                    reason: "Cannot delete current version when other versions exist".to_string(),
                });
            }

            content_versions.remove(idx);

            let mut snapshots = self.version_snapshots.write().await;
            snapshots.remove(&version_id);

            if !content_versions.is_empty() && !content_versions.iter().any(|v| v.is_current) {
                if let Some(latest) = content_versions.last_mut() {
                    latest.is_current = true;
                }
            }

            info!("Deleted version {} from content {}", version_id, content_id);
            Ok(())
        } else {
            Err(ContentError::NotFound { id: content_id })
        }
    }

    pub async fn prune_old_versions(
        &mut self,
        content_id: Uuid,
        keep_count: usize,
    ) -> ContentResult<u32> {
        let mut versions = self.versions.write().await;

        if let Some(content_versions) = versions.get_mut(&content_id) {
            if content_versions.len() <= keep_count {
                return Ok(0);
            }

            let remove_count = content_versions.len() - keep_count;
            let mut removed = 0u32;
            let mut snapshots = self.version_snapshots.write().await;

            let version_ids_to_remove: Vec<Uuid> = content_versions
                .iter()
                .take(remove_count)
                .filter(|v| !v.is_current)
                .map(|v| v.version_id)
                .collect();

            for version_id in &version_ids_to_remove {
                snapshots.remove(version_id);
                removed += 1;
            }

            content_versions
                .retain(|v| v.is_current || !version_ids_to_remove.contains(&v.version_id));

            info!(
                "Pruned {} old versions from content {}",
                removed, content_id
            );
            Ok(removed)
        } else {
            Ok(0)
        }
    }

    fn calculate_metadata_checksum(&self, metadata: &ContentMetadata) -> String {
        let mut hasher = Sha256::new();
        hasher.update(metadata.content_id.as_bytes());
        hasher.update(metadata.file_size.to_le_bytes());
        hasher.update(metadata.name.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}

impl Default for ContentVersionManager {
    fn default() -> Self {
        Self::new()
    }
}
