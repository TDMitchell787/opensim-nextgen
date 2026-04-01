//! Content Security and DRM for OpenSim Next
//!
//! Provides DRM protection, content validation, and anti-piracy measures.

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use super::{ContentResult, ContentError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrmProtectionLevel {
    None,
    Basic,
    Enhanced,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentProtection {
    pub content_id: Uuid,
    pub protection_level: DrmProtectionLevel,
    pub owner_id: Uuid,
    pub allowed_users: Vec<Uuid>,
    pub allowed_groups: Vec<Uuid>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub transfer_allowed: bool,
    pub copy_allowed: bool,
    pub modify_allowed: bool,
    pub encryption_key_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLog {
    pub content_id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub timestamp: u64,
    pub granted: bool,
    pub reason: Option<String>,
}

pub struct ContentSecurityManager {
    protections: Arc<RwLock<HashMap<Uuid, ContentProtection>>>,
    access_logs: Arc<RwLock<Vec<AccessLog>>>,
    max_log_entries: usize,
}

impl ContentSecurityManager {
    pub fn new() -> Self {
        Self {
            protections: Arc::new(RwLock::new(HashMap::new())),
            access_logs: Arc::new(RwLock::new(Vec::new())),
            max_log_entries: 10000,
        }
    }

    pub async fn apply_drm_protection(
        &self,
        content_id: Uuid,
        level: DrmProtectionLevel,
    ) -> ContentResult<()> {
        self.apply_drm_protection_with_owner(content_id, level, Uuid::nil()).await
    }

    pub async fn apply_drm_protection_with_owner(
        &self,
        content_id: Uuid,
        level: DrmProtectionLevel,
        owner_id: Uuid,
    ) -> ContentResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let encryption_key_hash = match &level {
            DrmProtectionLevel::None => None,
            DrmProtectionLevel::Basic => Some(format!("basic_{}", content_id)),
            DrmProtectionLevel::Enhanced => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(content_id.as_bytes());
                hasher.update(owner_id.as_bytes());
                hasher.update(&now.to_le_bytes());
                Some(hex::encode(hasher.finalize()))
            }
            DrmProtectionLevel::Enterprise => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(b"enterprise_");
                hasher.update(content_id.as_bytes());
                hasher.update(owner_id.as_bytes());
                hasher.update(&now.to_le_bytes());
                Some(hex::encode(hasher.finalize()))
            }
        };

        let protection = ContentProtection {
            content_id,
            protection_level: level.clone(),
            owner_id,
            allowed_users: vec![owner_id],
            allowed_groups: Vec::new(),
            created_at: now,
            expires_at: None,
            transfer_allowed: matches!(level, DrmProtectionLevel::None | DrmProtectionLevel::Basic),
            copy_allowed: matches!(level, DrmProtectionLevel::None),
            modify_allowed: matches!(level, DrmProtectionLevel::None | DrmProtectionLevel::Basic),
            encryption_key_hash,
        };

        self.protections.write().await.insert(content_id, protection);

        info!("Applied {:?} DRM protection to content {}", level, content_id);
        Ok(())
    }

    pub async fn validate_content_access(
        &self,
        content_id: Uuid,
        user_id: Uuid,
    ) -> ContentResult<bool> {
        self.validate_access_with_action(content_id, user_id, "view").await
    }

    pub async fn validate_access_with_action(
        &self,
        content_id: Uuid,
        user_id: Uuid,
        action: &str,
    ) -> ContentResult<bool> {
        let protections = self.protections.read().await;

        let (granted, reason) = match protections.get(&content_id) {
            None => (true, Some("No protection applied".to_string())),
            Some(protection) => {
                if protection.owner_id == user_id {
                    (true, Some("Owner access".to_string()))
                } else if protection.allowed_users.contains(&user_id) {
                    match action {
                        "view" | "access" => (true, Some("Allowed user".to_string())),
                        "copy" => (protection.copy_allowed,
                            if protection.copy_allowed { Some("Copy allowed".to_string()) }
                            else { Some("Copy not allowed".to_string()) }),
                        "modify" | "edit" => (protection.modify_allowed,
                            if protection.modify_allowed { Some("Modify allowed".to_string()) }
                            else { Some("Modify not allowed".to_string()) }),
                        "transfer" => (protection.transfer_allowed,
                            if protection.transfer_allowed { Some("Transfer allowed".to_string()) }
                            else { Some("Transfer not allowed".to_string()) }),
                        _ => (true, Some(format!("Action {} allowed for permitted user", action))),
                    }
                } else {
                    match protection.protection_level {
                        DrmProtectionLevel::None => (true, Some("No DRM restrictions".to_string())),
                        _ => (false, Some("Access denied - not in allowed list".to_string())),
                    }
                }
            }
        };

        drop(protections);

        self.log_access(content_id, user_id, action.to_string(), granted, reason.clone()).await;

        debug!("Access check for {} by {}: {} ({})", content_id, user_id, granted, reason.unwrap_or_default());
        Ok(granted)
    }

    pub async fn grant_user_access(&self, content_id: Uuid, user_id: Uuid) -> ContentResult<()> {
        let mut protections = self.protections.write().await;

        if let Some(protection) = protections.get_mut(&content_id) {
            if !protection.allowed_users.contains(&user_id) {
                protection.allowed_users.push(user_id);
                info!("Granted access to content {} for user {}", content_id, user_id);
            }
            Ok(())
        } else {
            Err(ContentError::NotFound { id: content_id })
        }
    }

    pub async fn revoke_user_access(&self, content_id: Uuid, user_id: Uuid) -> ContentResult<()> {
        let mut protections = self.protections.write().await;

        if let Some(protection) = protections.get_mut(&content_id) {
            if protection.owner_id == user_id {
                return Err(ContentError::ValidationError {
                    reason: "Cannot revoke owner access".to_string()
                });
            }
            protection.allowed_users.retain(|&id| id != user_id);
            info!("Revoked access to content {} for user {}", content_id, user_id);
            Ok(())
        } else {
            Err(ContentError::NotFound { id: content_id })
        }
    }

    pub async fn get_protection(&self, content_id: Uuid) -> ContentResult<Option<ContentProtection>> {
        Ok(self.protections.read().await.get(&content_id).cloned())
    }

    pub async fn remove_protection(&self, content_id: Uuid) -> ContentResult<()> {
        self.protections.write().await.remove(&content_id);
        info!("Removed DRM protection from content {}", content_id);
        Ok(())
    }

    pub async fn get_access_logs(&self, content_id: Uuid, limit: usize) -> Vec<AccessLog> {
        self.access_logs.read().await
            .iter()
            .filter(|log| log.content_id == content_id)
            .take(limit)
            .cloned()
            .collect()
    }

    async fn log_access(&self, content_id: Uuid, user_id: Uuid, action: String, granted: bool, reason: Option<String>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let log = AccessLog {
            content_id,
            user_id,
            action,
            timestamp: now,
            granted,
            reason,
        };

        let mut logs = self.access_logs.write().await;
        logs.push(log);

        let len = logs.len();
        if len > self.max_log_entries {
            let drain_count = len - self.max_log_entries;
            logs.drain(0..drain_count);
        }
    }
}

impl Default for ContentSecurityManager {
    fn default() -> Self {
        Self::new()
    }
}