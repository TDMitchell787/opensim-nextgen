use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, debug};

use crate::services::traits::{FriendsServiceTrait, FriendInfo};
use crate::database::DatabaseConnection;

pub struct LocalFriendsService {
    db: Arc<DatabaseConnection>,
}

impl LocalFriendsService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FriendsServiceTrait for LocalFriendsService {
    async fn get_friends(&self, principal_id: &str) -> Result<Vec<FriendInfo>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let rows = sqlx::query(
            r#"SELECT "PrincipalID", "Friend", "Flags", "Offered" FROM friends WHERE "PrincipalID" = $1"#
        )
        .bind(principal_id)
        .fetch_all(pool)
        .await?;

        let mut results = Vec::new();
        for row in &rows {
            use sqlx::Row;
            let principal: String = row.try_get("PrincipalID").unwrap_or_default();
            let friend: String = row.try_get("Friend").unwrap_or_default();
            let my_flags: i32 = row.try_get("Flags").unwrap_or(0);

            let their_row = sqlx::query(
                r#"SELECT "Flags" FROM friends WHERE "PrincipalID" = $1 AND "Friend" LIKE $2"#
            )
            .bind(&friend)
            .bind(format!("{}%", principal))
            .fetch_optional(pool)
            .await?;

            let their_flags = their_row
                .map(|r| {
                    use sqlx::Row;
                    r.try_get::<i32, _>("Flags").unwrap_or(0)
                })
                .unwrap_or(0);

            results.push(FriendInfo {
                principal_id: principal,
                friend: friend.clone(),
                my_flags,
                their_flags,
            });
        }

        debug!("[FRIENDS] get_friends({}) -> {} entries", principal_id, results.len());
        Ok(results)
    }

    async fn store_friend(&self, principal_id: &str, friend: &str, flags: i32) -> Result<bool> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        info!("[FRIENDS] store_friend: {} -> {} flags={}", principal_id, friend, flags);

        sqlx::query(
            r#"INSERT INTO friends ("PrincipalID", "Friend", "Flags", "Offered")
               VALUES ($1, $2, $3, 0)
               ON CONFLICT ("PrincipalID", "Friend") DO UPDATE SET "Flags" = $3"#
        )
        .bind(principal_id)
        .bind(friend)
        .bind(flags)
        .execute(pool)
        .await?;

        Ok(true)
    }

    async fn delete_friend(&self, principal_id: &str, friend: &str) -> Result<bool> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        info!("[FRIENDS] delete_friend: {} -> {}", principal_id, friend);

        let result = sqlx::query(
            r#"DELETE FROM friends WHERE "PrincipalID" = $1 AND "Friend" = $2"#
        )
        .bind(principal_id)
        .bind(friend)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
