use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::HGFriendsServiceTrait;

pub struct HGFriendsService {
    db_pool: Arc<sqlx::PgPool>,
}

#[derive(Debug, Clone)]
pub struct HGFriendship {
    pub principal_id: Uuid,
    pub friend_id: String,
    pub flags: i32,
    pub secret: String,
}

impl HGFriendsService {
    pub fn new(db_pool: Arc<sqlx::PgPool>) -> Self {
        Self { db_pool }
    }

    fn generate_secret() -> String {
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = (0..8)
            .map(|_| {
                let idx: u32 = rng.gen_range(0..36);
                if idx < 10 {
                    (b'0' + idx as u8) as char
                } else {
                    (b'a' + (idx - 10) as u8) as char
                }
            })
            .collect();
        chars.into_iter().collect()
    }
}

#[async_trait]
impl HGFriendsServiceTrait for HGFriendsService {
    async fn get_friend_perms(&self, principal_id: Uuid, friend_id: &str) -> Result<i32> {
        let row = sqlx::query_scalar::<_, i32>(
            "SELECT flags FROM friends WHERE \"PrincipalID\" = $1 AND \"Friend\" = $2",
        )
        .bind(principal_id.to_string())
        .bind(friend_id)
        .fetch_optional(self.db_pool.as_ref())
        .await?;

        Ok(row.unwrap_or(-1))
    }

    async fn new_friendship(
        &self,
        principal_id: Uuid,
        friend_id: &str,
        secret: &str,
        verified: bool,
    ) -> Result<bool> {
        if !verified {
            // Bug fix #1: bind order was reversed — principal_id=$1, friend_id=$2
            let existing = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM friends WHERE \"PrincipalID\" = $1 AND \"Friend\" = $2",
            )
            .bind(principal_id.to_string())
            .bind(friend_id)
            .fetch_one(self.db_pool.as_ref())
            .await?;

            if existing == 0 {
                warn!("[HG-FRIENDS] Unverified friendship request from {} to {} — no reverse friendship found", principal_id, friend_id);
                return Ok(false);
            }
        }

        let secret_to_use = if secret.is_empty() {
            Self::generate_secret()
        } else {
            secret.to_string()
        };

        // Bug fix #2: use flags=0 for unverified (pending), 1 for verified — matches C#
        let flags: i32 = if verified { 1 } else { 0 };

        sqlx::query(
            "INSERT INTO friends (\"PrincipalID\", \"Friend\", \"Flags\", \"Secret\") \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (\"PrincipalID\", \"Friend\") DO UPDATE SET \"Flags\" = $3, \"Secret\" = $4"
        )
        .bind(principal_id.to_string())
        .bind(friend_id)
        .bind(flags)
        .bind(&secret_to_use)
        .execute(self.db_pool.as_ref())
        .await?;

        info!(
            "[HG-FRIENDS] New friendship: {} -> {} (verified={})",
            principal_id, friend_id, verified
        );
        Ok(true)
    }

    async fn delete_friendship(
        &self,
        principal_id: Uuid,
        friend_id: &str,
        secret: &str,
    ) -> Result<bool> {
        let stored_secret = sqlx::query_scalar::<_, String>(
            "SELECT \"Secret\" FROM friends WHERE \"PrincipalID\" = $1 AND \"Friend\" = $2",
        )
        .bind(principal_id.to_string())
        .bind(friend_id)
        .fetch_optional(self.db_pool.as_ref())
        .await?;

        if let Some(ref stored) = stored_secret {
            if stored != secret {
                warn!(
                    "[HG-FRIENDS] Delete friendship secret mismatch for {} -> {}",
                    principal_id, friend_id
                );
                return Ok(false);
            }
        }

        sqlx::query("DELETE FROM friends WHERE \"PrincipalID\" = $1 AND \"Friend\" = $2")
            .bind(principal_id.to_string())
            .bind(friend_id)
            .execute(self.db_pool.as_ref())
            .await?;

        sqlx::query("DELETE FROM friends WHERE \"PrincipalID\" = $1 AND \"Friend\" = $2")
            .bind(friend_id)
            .bind(principal_id.to_string())
            .execute(self.db_pool.as_ref())
            .await?;

        info!(
            "[HG-FRIENDS] Deleted friendship: {} <-> {}",
            principal_id, friend_id
        );
        Ok(true)
    }

    async fn validate_friendship_offered(
        &self,
        principal_id: Uuid,
        friend_id: &str,
    ) -> Result<bool> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM friends WHERE \"PrincipalID\" = $1 AND \"Friend\" = $2",
        )
        .bind(principal_id.to_string())
        .bind(friend_id)
        .fetch_one(self.db_pool.as_ref())
        .await?;

        Ok(count > 0)
    }

    // Bug fix #3: added user_id parameter, use it as local user for perm lookup
    async fn status_notification(
        &self,
        friends: &[String],
        user_id: Uuid,
        online: bool,
    ) -> Result<Vec<String>> {
        let mut online_friends = Vec::new();
        for friend_id in friends {
            let _friend_uuid = if let Ok(u) = Uuid::parse_str(friend_id) {
                u
            } else {
                continue;
            };

            let has_perm = self
                .get_friend_perms(user_id, friend_id)
                .await
                .unwrap_or(-1);
            if has_perm < 0 {
                continue;
            }

            debug!(
                "[HG-FRIENDS] Status notification: {} is {}",
                friend_id,
                if online { "online" } else { "offline" }
            );
            online_friends.push(friend_id.clone());
        }
        Ok(online_friends)
    }
}
