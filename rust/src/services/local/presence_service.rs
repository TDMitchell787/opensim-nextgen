//! Local Presence Service Implementation
//!
//! Provides direct database access for presence/session tracking.
//! Used in standalone mode with PostgreSQL backend.
//!
//! Reference: OpenSim/Services/PresenceService/PresenceService.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{PresenceServiceTrait, PresenceInfo};
use crate::database::multi_backend::DatabaseConnection;

pub struct LocalPresenceService {
    connection: Arc<DatabaseConnection>,
}

impl LocalPresenceService {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        info!("Initializing local presence service");
        Self { connection }
    }

    fn get_pg_pool(&self) -> Result<&sqlx::PgPool> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => Ok(pool),
            _ => Err(anyhow!("LocalPresenceService requires PostgreSQL connection")),
        }
    }

    fn row_to_presence_info(&self, row: &sqlx::postgres::PgRow) -> Result<PresenceInfo> {
        use sqlx::Row;
        Ok(PresenceInfo {
            user_id: row.try_get("userid")?,
            session_id: row.try_get("sessionid")?,
            secure_session_id: row.try_get("securesessionid")?,
            region_id: row.try_get("regionid")?,
            online: true,
            login_time: row.try_get("login").unwrap_or(0),
            logout_time: row.try_get("logout").unwrap_or(0),
        })
    }
}

#[async_trait]
impl PresenceServiceTrait for LocalPresenceService {
    async fn login_agent(&self, user_id: Uuid, session_id: Uuid, secure_session_id: Uuid, region_id: Uuid) -> Result<bool> {
        info!("Logging in agent: {} to region: {}", user_id, region_id);

        let now = chrono::Utc::now().timestamp();

        let pool = self.get_pg_pool()?;
        sqlx::query(
            r#"
            INSERT INTO presence (userid, sessionid, securesessionid, regionid, login, logout)
            VALUES ($1, $2, $3, $4, $5, 0)
            ON CONFLICT(sessionid) DO UPDATE SET
                regionid = EXCLUDED.regionid,
                securesessionid = EXCLUDED.securesessionid,
                login = EXCLUDED.login
            "#
        )
        .bind(user_id)
        .bind(session_id)
        .bind(secure_session_id)
        .bind(region_id)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to login agent: {}", e))?;

        info!("Agent logged in: {} (session: {})", user_id, session_id);
        Ok(true)
    }

    async fn logout_agent(&self, session_id: Uuid) -> Result<bool> {
        info!("Logging out agent session: {}", session_id);

        let now = chrono::Utc::now().timestamp();

        let pool = self.get_pg_pool()?;
        let result = sqlx::query(
            "UPDATE presence SET logout = $1 WHERE sessionid = $2"
        )
        .bind(now)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to logout agent: {}", e))?;

        if result.rows_affected() > 0 {
            sqlx::query("DELETE FROM presence WHERE sessionid = $1")
                .bind(session_id)
                .execute(pool)
                .await
                .map_err(|e| anyhow!("Failed to remove presence record: {}", e))?;
            info!("Agent logged out: {}", session_id);
            Ok(true)
        } else {
            warn!("No presence record found for session: {}", session_id);
            Ok(false)
        }
    }

    async fn report_agent(&self, session_id: Uuid, region_id: Uuid) -> Result<bool> {
        debug!("Reporting agent location: {} in region: {}", session_id, region_id);

        let pool = self.get_pg_pool()?;
        let result = sqlx::query(
            "UPDATE presence SET regionid = $1 WHERE sessionid = $2"
        )
        .bind(region_id)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to report agent: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_agent(&self, session_id: Uuid) -> Result<Option<PresenceInfo>> {
        debug!("Getting agent presence: {}", session_id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            "SELECT userid, sessionid, securesessionid, regionid, login, logout FROM presence WHERE sessionid = $1"
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get agent presence: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_presence_info(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_agents(&self, user_ids: &[Uuid]) -> Result<Vec<PresenceInfo>> {
        debug!("Getting presence for {} users", user_ids.len());

        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        let pool = self.get_pg_pool()?;

        let mut results = Vec::new();
        for user_id in user_ids {
            let rows = sqlx::query(
                "SELECT userid, sessionid, securesessionid, regionid, login, logout FROM presence WHERE userid = $1"
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow!("Failed to get agent presence: {}", e))?;

            for row in rows {
                results.push(self.row_to_presence_info(&row)?);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_info_default() {
        let info = PresenceInfo {
            user_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            secure_session_id: Uuid::new_v4(),
            region_id: Uuid::new_v4(),
            online: true,
            login_time: 0,
            logout_time: 0,
        };

        assert!(info.online);
        assert!(!info.user_id.is_nil());
        assert!(!info.session_id.is_nil());
    }
}
