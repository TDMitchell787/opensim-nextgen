use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use tracing::{info, warn, debug};

use crate::services::traits::{GridUserServiceTrait, GridUserInfo};
use crate::database::DatabaseConnection;

pub struct LocalGridUserService {
    db: Arc<DatabaseConnection>,
}

impl LocalGridUserService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn now_unix() -> String {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string()
    }

    async fn row_to_info(row: &sqlx::postgres::PgRow) -> GridUserInfo {
        use sqlx::Row;
        let user_id: String = row.try_get("userid").unwrap_or_default();
        let home_region_id: Uuid = row.try_get("homeregionid").unwrap_or_default();
        let home_position: String = row.try_get::<String, _>("homeposition").unwrap_or_else(|_| "<0,0,0>".to_string());
        let home_look_at: String = row.try_get::<String, _>("homelookat").unwrap_or_else(|_| "<0,0,0>".to_string());
        let last_region_id: Uuid = row.try_get("lastregionid").unwrap_or_default();
        let last_position: String = row.try_get::<String, _>("lastposition").unwrap_or_else(|_| "<0,0,0>".to_string());
        let last_look_at: String = row.try_get::<String, _>("lastlookat").unwrap_or_else(|_| "<0,0,0>".to_string());
        let online_str: String = row.try_get::<String, _>("online").unwrap_or_else(|_| "false".to_string());
        let login: String = row.try_get::<String, _>("login").unwrap_or_else(|_| "0".to_string());
        let logout: String = row.try_get::<String, _>("logout").unwrap_or_else(|_| "0".to_string());

        GridUserInfo {
            user_id,
            home_region_id,
            home_position: home_position.trim().to_string(),
            home_look_at: home_look_at.trim().to_string(),
            last_region_id,
            last_position: last_position.trim().to_string(),
            last_look_at: last_look_at.trim().to_string(),
            online: online_str.trim().eq_ignore_ascii_case("true"),
            login: login.trim().to_string(),
            logout: logout.trim().to_string(),
        }
    }
}

#[async_trait]
impl GridUserServiceTrait for LocalGridUserService {
    async fn logged_in(&self, user_id: &str) -> Result<Option<GridUserInfo>> {
        debug!("[GRIDUSER] User {} is online", user_id);

        let now = Self::now_unix();

        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO griduser (userid, homeregionid, homeposition, homelookat, \
             lastregionid, lastposition, lastlookat, online, login, logout) \
             VALUES ($1, '00000000-0000-0000-0000-000000000000', '<0,0,0>', '<0,0,0>', \
             '00000000-0000-0000-0000-000000000000', '<0,0,0>', '<0,0,0>', 'true', $2, '0') \
             ON CONFLICT (userid) DO UPDATE SET online = 'true', login = $2"
        )
        .bind(user_id)
        .bind(&now)
        .execute(pool)
        .await?;

        self.get_grid_user_info(user_id).await
    }

    async fn logged_out(&self, user_id: &str, region_id: Uuid, position: &str, look_at: &str) -> Result<bool> {
        debug!("[GRIDUSER] User {} is offline", user_id);

        let now = Self::now_unix();

        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO griduser (userid, homeregionid, homeposition, homelookat, \
             lastregionid, lastposition, lastlookat, online, login, logout) \
             VALUES ($1, '00000000-0000-0000-0000-000000000000', '<0,0,0>', '<0,0,0>', \
             $2::uuid, $3, $4, 'false', '0', $5) \
             ON CONFLICT (userid) DO UPDATE SET online = 'false', logout = $5, \
             lastregionid = $2::uuid, lastposition = $3, lastlookat = $4"
        )
        .bind(user_id)
        .bind(region_id.to_string())
        .bind(position)
        .bind(look_at)
        .bind(&now)
        .execute(pool)
        .await?;

        Ok(true)
    }

    async fn set_home(&self, user_id: &str, home_id: Uuid, position: &str, look_at: &str) -> Result<bool> {
        debug!("[GRIDUSER] SetHome for {}: region={}", user_id, home_id);

        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO griduser (userid, homeregionid, homeposition, homelookat, \
             lastregionid, lastposition, lastlookat, online, login, logout) \
             VALUES ($1, $2::uuid, $3, $4, \
             '00000000-0000-0000-0000-000000000000', '<0,0,0>', '<0,0,0>', 'false', '0', '0') \
             ON CONFLICT (userid) DO UPDATE SET homeregionid = $2::uuid, homeposition = $3, homelookat = $4"
        )
        .bind(user_id)
        .bind(home_id.to_string())
        .bind(position)
        .bind(look_at)
        .execute(pool)
        .await?;

        Ok(true)
    }

    async fn set_last_position(&self, user_id: &str, region_id: Uuid, position: &str, look_at: &str) -> Result<bool> {
        debug!("[GRIDUSER] SetLastPosition for {}: region={}", user_id, region_id);

        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO griduser (userid, homeregionid, homeposition, homelookat, \
             lastregionid, lastposition, lastlookat, online, login, logout) \
             VALUES ($1, '00000000-0000-0000-0000-000000000000', '<0,0,0>', '<0,0,0>', \
             $2::uuid, $3, $4, 'false', '0', '0') \
             ON CONFLICT (userid) DO UPDATE SET lastregionid = $2::uuid, lastposition = $3, lastlookat = $4"
        )
        .bind(user_id)
        .bind(region_id.to_string())
        .bind(position)
        .bind(look_at)
        .execute(pool)
        .await?;

        Ok(true)
    }

    async fn get_grid_user_info(&self, user_id: &str) -> Result<Option<GridUserInfo>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let row = sqlx::query(
            "SELECT userid, homeregionid, homeposition, homelookat, \
             lastregionid, lastposition, lastlookat, online, login, logout \
             FROM griduser WHERE userid = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => Ok(Some(Self::row_to_info(&r).await)),
            None => Ok(None),
        }
    }

    async fn get_grid_user_infos(&self, user_ids: &[String]) -> Result<Vec<GridUserInfo>> {
        let mut result = Vec::new();
        for uid in user_ids {
            if let Some(info) = self.get_grid_user_info(uid).await? {
                result.push(info);
            }
        }
        Ok(result)
    }
}
