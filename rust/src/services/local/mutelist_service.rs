use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::debug;

use crate::database::DatabaseConnection;
use crate::services::traits::{MuteData, MuteListServiceTrait};

pub struct LocalMuteListService {
    db: Arc<DatabaseConnection>,
}

impl LocalMuteListService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MuteListServiceTrait for LocalMuteListService {
    async fn get_mutes(&self, agent_id: &str) -> Result<Vec<MuteData>> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let rows = sqlx::query(
            "SELECT agentid, muteid, mutename, mutetype, muteflags, stamp \
             FROM mutelist WHERE agentid = $1",
        )
        .bind(agent_id)
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for row in &rows {
            use sqlx::Row;
            result.push(MuteData {
                agent_id: row.try_get::<String, _>("agentid").unwrap_or_default(),
                mute_id: row.try_get::<String, _>("muteid").unwrap_or_default(),
                mute_name: row.try_get::<String, _>("mutename").unwrap_or_default(),
                mute_type: row.try_get::<i32, _>("mutetype").unwrap_or(0),
                mute_flags: row.try_get::<i32, _>("muteflags").unwrap_or(0),
                stamp: row.try_get::<i32, _>("stamp").unwrap_or(0),
            });
        }

        debug!("[MUTELIST] Got {} mutes for {}", result.len(), agent_id);
        Ok(result)
    }

    async fn update_mute(&self, mute: &MuteData) -> Result<bool> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO mutelist (agentid, muteid, mutename, mutetype, muteflags, stamp) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (agentid, muteid, mutename) DO UPDATE SET \
             mutetype = $4, muteflags = $5, stamp = $6",
        )
        .bind(&mute.agent_id)
        .bind(&mute.mute_id)
        .bind(&mute.mute_name)
        .bind(mute.mute_type)
        .bind(mute.mute_flags)
        .bind(mute.stamp)
        .execute(pool)
        .await?;

        debug!(
            "[MUTELIST] Updated mute for {} -> {}",
            mute.agent_id, mute.mute_name
        );
        Ok(true)
    }

    async fn remove_mute(&self, agent_id: &str, mute_id: &str, mute_name: &str) -> Result<bool> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let result = sqlx::query(
            "DELETE FROM mutelist WHERE agentid = $1 AND muteid = $2 AND mutename = $3",
        )
        .bind(agent_id)
        .bind(mute_id)
        .bind(mute_name)
        .execute(pool)
        .await?;

        debug!(
            "[MUTELIST] Removed mute for {} -> {} ({})",
            agent_id,
            mute_name,
            result.rows_affected()
        );
        Ok(result.rows_affected() > 0)
    }
}
