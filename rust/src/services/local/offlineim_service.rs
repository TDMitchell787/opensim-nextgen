use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use crate::database::DatabaseConnection;
use crate::services::traits::{OfflineIM, OfflineIMServiceTrait};

pub struct LocalOfflineIMService {
    db: Arc<DatabaseConnection>,
}

impl LocalOfflineIMService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl OfflineIMServiceTrait for LocalOfflineIMService {
    async fn get_messages(&self, principal_id: &str) -> Result<Vec<OfflineIM>> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let rows = sqlx::query(
            r#"SELECT "ID", "PrincipalID", "FromID", "Message", "TMStamp" FROM im_offline WHERE "PrincipalID" = $1"#
        )
        .bind(principal_id)
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for row in &rows {
            use sqlx::Row;
            result.push(OfflineIM {
                id: row.try_get::<i32, _>("ID").unwrap_or(0),
                principal_id: row.try_get::<String, _>("PrincipalID").unwrap_or_default(),
                from_id: row.try_get::<String, _>("FromID").unwrap_or_default(),
                message: row.try_get::<String, _>("Message").unwrap_or_default(),
                timestamp: row.try_get::<i32, _>("TMStamp").unwrap_or(0),
            });
        }

        debug!(
            "[OFFLINEIM] get_messages({}) -> {} messages",
            principal_id,
            result.len()
        );
        Ok(result)
    }

    async fn store_message(&self, im: &OfflineIM) -> Result<bool> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i32;

        info!(
            "[OFFLINEIM] store_message: from={} to={}",
            im.from_id, im.principal_id
        );

        sqlx::query(
            r#"INSERT INTO im_offline ("PrincipalID", "FromID", "Message", "TMStamp")
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(&im.principal_id)
        .bind(&im.from_id)
        .bind(&im.message)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(true)
    }

    async fn delete_messages(&self, principal_id: &str) -> Result<bool> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        info!("[OFFLINEIM] delete_messages: {}", principal_id);

        sqlx::query(r#"DELETE FROM im_offline WHERE "PrincipalID" = $1"#)
            .bind(principal_id)
            .execute(pool)
            .await?;

        Ok(true)
    }
}
