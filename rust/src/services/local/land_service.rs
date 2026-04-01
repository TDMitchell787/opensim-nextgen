use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use tracing::debug;

use crate::services::traits::{LandServiceTrait, LandData};
use crate::database::DatabaseConnection;

pub struct LocalLandService {
    db: Arc<DatabaseConnection>,
}

impl LocalLandService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn region_handle_to_uuid(handle: u64) -> (u32, u32) {
        let x = ((handle >> 32) & 0xFFFF_FFFF) as u32;
        let y = (handle & 0xFFFF_FFFF) as u32;
        (x, y)
    }
}

#[async_trait]
impl LandServiceTrait for LocalLandService {
    async fn get_land_data(
        &self,
        _scope_id: Uuid,
        region_handle: u64,
        x: u32,
        y: u32,
    ) -> Result<Option<LandData>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        debug!("[LAND] get_land_data: handle={} x={} y={}", region_handle, x, y);

        let row = sqlx::query(
            "SELECT uuid, name, description, owneruuid, isgroupowned, area, \
             userlocationx, userlocationy, userlocationz, regionuuid, landflags, \
             saleprice, snapshotid, dwell \
             FROM land WHERE regionuuid = (SELECT regionuuid FROM regions WHERE regionhandle = $1 LIMIT 1) \
             LIMIT 1"
        )
        .bind(region_handle as i64)
        .fetch_optional(pool)
        .await;

        let row = match row {
            Ok(Some(r)) => r,
            Ok(None) => {
                let (_gx, _gy) = Self::region_handle_to_uuid(region_handle);
                debug!("[LAND] No parcel found for handle {}", region_handle);
                return Ok(None);
            }
            Err(e) => {
                debug!("[LAND] Query error (falling back to direct regionuuid): {}", e);
                return Ok(None);
            }
        };

        use sqlx::Row;
        let land = LandData {
            local_id: 0,
            global_id: row.try_get::<Uuid, _>("uuid").unwrap_or_default(),
            name: row.try_get::<String, _>("name").unwrap_or_default(),
            description: row.try_get::<String, _>("description").unwrap_or_default(),
            owner_id: row.try_get::<Uuid, _>("owneruuid").unwrap_or_default(),
            is_group_owned: row.try_get::<i32, _>("isgroupowned").unwrap_or(0) != 0,
            area: row.try_get::<i32, _>("area").unwrap_or(0),
            landing_x: row.try_get::<f32, _>("userlocationx").unwrap_or(128.0),
            landing_y: row.try_get::<f32, _>("userlocationy").unwrap_or(128.0),
            landing_z: row.try_get::<f32, _>("userlocationz").unwrap_or(25.0),
            region_id: row.try_get::<Uuid, _>("regionuuid").unwrap_or_default(),
            flags: row.try_get::<i32, _>("landflags").unwrap_or(0) as u32,
            sale_price: row.try_get::<i32, _>("saleprice").unwrap_or(0),
            snapshot_id: row.try_get::<Uuid, _>("snapshotid").unwrap_or_default(),
            dwell: row.try_get::<f32, _>("dwell").unwrap_or(0.0),
        };

        Ok(Some(land))
    }
}
