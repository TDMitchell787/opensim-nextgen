//! Local Grid Service Implementation
//!
//! Provides direct database access for grid/region operations.
//! Used in standalone mode with PostgreSQL backend.
//!
//! Reference: OpenSim/Services/GridService/GridService.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{GridServiceTrait, RegionInfo};
use crate::database::multi_backend::DatabaseConnection;

pub struct LocalGridService {
    connection: Arc<DatabaseConnection>,
}

impl LocalGridService {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        info!("Initializing local grid service");
        Self { connection }
    }

    fn get_pg_pool(&self) -> Result<&sqlx::PgPool> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => Ok(pool),
            _ => Err(anyhow!("LocalGridService requires PostgreSQL connection")),
        }
    }
}

#[async_trait]
impl GridServiceTrait for LocalGridService {
    async fn register_region(&self, region: &RegionInfo) -> Result<bool> {
        info!("Registering region: {} at ({}, {})",
              region.region_name, region.region_loc_x, region.region_loc_y);

        let pool = self.get_pg_pool()?;

        sqlx::query(
            r#"
            INSERT INTO regions (
                id, region_name, location_x, location_y,
                size_x, size_y, serverip, internal_port,
                external_host_name, flags, scopeid, owner_uuid,
                created_at, updated_at, last_seen
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW(), EXTRACT(EPOCH FROM NOW())::int)
            ON CONFLICT(id) DO UPDATE SET
                region_name = EXCLUDED.region_name,
                location_x = EXCLUDED.location_x,
                location_y = EXCLUDED.location_y,
                size_x = EXCLUDED.size_x,
                size_y = EXCLUDED.size_y,
                serverip = EXCLUDED.serverip,
                internal_port = EXCLUDED.internal_port,
                external_host_name = EXCLUDED.external_host_name,
                flags = EXCLUDED.flags,
                updated_at = NOW(),
                last_seen = EXTRACT(EPOCH FROM NOW())::int
            "#
        )
        .bind(region.region_id)
        .bind(&region.region_name)
        .bind(region.region_loc_x as i32)
        .bind(region.region_loc_y as i32)
        .bind(region.region_size_x as i32)
        .bind(region.region_size_y as i32)
        .bind(&region.server_ip)
        .bind(region.server_port as i32)
        .bind(&region.server_uri)
        .bind(region.region_flags as i32)
        .bind(region.scope_id)
        .bind(region.owner_id)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to register region: {}", e))?;

        info!("Registered region: {} ({})", region.region_name, region.region_id);
        Ok(true)
    }

    async fn deregister_region(&self, region_id: Uuid) -> Result<bool> {
        warn!("Deregistering region: {}", region_id);

        let pool = self.get_pg_pool()?;
        let result = sqlx::query("DELETE FROM regions WHERE id = $1")
            .bind(region_id)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to deregister region: {}", e))?;
        Ok(result.rows_affected() > 0)
    }

    async fn get_region_by_uuid(&self, scopeid: Uuid, region_id: Uuid) -> Result<Option<RegionInfo>> {
        debug!("Getting region by UUID: {} (scope: {})", region_id, scopeid);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            r#"
            SELECT id, region_name, location_x, location_y, size_x, size_y,
                   serverip, internal_port, external_host_name, flags, scopeid, owner_uuid, 1 as estate_id
            FROM regions
            WHERE id = $1 AND (scopeid = $2 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $2 = '00000000-0000-0000-0000-000000000000')
            "#
        )
        .bind(region_id)
        .bind(scopeid)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get region: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_region_info(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_region_by_name(&self, scopeid: Uuid, name: &str) -> Result<Option<RegionInfo>> {
        debug!("Getting region by name: {} (scope: {})", name, scopeid);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            r#"
            SELECT id, region_name, location_x, location_y, size_x, size_y,
                   serverip, internal_port, external_host_name, flags, scopeid, owner_uuid, 1 as estate_id
            FROM regions
            WHERE LOWER(region_name) = LOWER($1) AND (scopeid = $2 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $2 = '00000000-0000-0000-0000-000000000000')
            "#
        )
        .bind(name)
        .bind(scopeid)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get region by name: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_region_info(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_region_by_position(&self, scopeid: Uuid, x: u32, y: u32) -> Result<Option<RegionInfo>> {
        debug!("Getting region by position: ({}, {}) (scope: {})", x, y, scopeid);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            r#"
            SELECT id, region_name, location_x, location_y, size_x, size_y,
                   serverip, internal_port, external_host_name, flags, scopeid, owner_uuid, 1 as estate_id
            FROM regions
            WHERE location_x = $1 AND location_y = $2
            AND (scopeid = $3 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $3 = '00000000-0000-0000-0000-000000000000')
            "#
        )
        .bind(x as i32)
        .bind(y as i32)
        .bind(scopeid)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get region by position: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_region_info(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_neighbours(&self, scopeid: Uuid, region_id: Uuid, range: u32) -> Result<Vec<RegionInfo>> {
        debug!("Getting neighbours for region: {} (range: {})", region_id, range);

        let center = self.get_region_by_uuid(scopeid, region_id).await?;
        if center.is_none() {
            return Ok(Vec::new());
        }
        let center = center.unwrap();

        let min_x = center.region_loc_x.saturating_sub(range);
        let max_x = center.region_loc_x + range;
        let min_y = center.region_loc_y.saturating_sub(range);
        let max_y = center.region_loc_y + range;

        let pool = self.get_pg_pool()?;
        let rows = sqlx::query(
            r#"
            SELECT id, region_name, location_x, location_y, size_x, size_y,
                   serverip, internal_port, external_host_name, flags, scopeid, owner_uuid, 1 as estate_id
            FROM regions
            WHERE location_x >= $1 AND location_x <= $2
            AND location_y >= $3 AND location_y <= $4
            AND id != $5
            AND (scopeid = $6 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $6 = '00000000-0000-0000-0000-000000000000')
            "#
        )
        .bind(min_x as i32)
        .bind(max_x as i32)
        .bind(min_y as i32)
        .bind(max_y as i32)
        .bind(region_id)
        .bind(scopeid)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to get neighbours: {}", e))?;

        let mut regions = Vec::new();
        for row in rows {
            regions.push(self.row_to_region_info(&row)?);
        }
        Ok(regions)
    }

    async fn get_default_regions(&self, scopeid: Uuid) -> Result<Vec<RegionInfo>> {
        debug!("Getting default regions (scope: {})", scopeid);

        const DEFAULT_REGION_FLAG: i32 = 0x01;

        let pool = self.get_pg_pool()?;
        let rows = sqlx::query(
            r#"
            SELECT id, region_name, location_x, location_y, size_x, size_y,
                   serverip, internal_port, external_host_name, flags, scopeid, owner_uuid, 1 as estate_id
            FROM regions
            WHERE (flags & $1) != 0
            AND (scopeid = $2 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $2 = '00000000-0000-0000-0000-000000000000')
            "#
        )
        .bind(DEFAULT_REGION_FLAG)
        .bind(scopeid)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to get default regions: {}", e))?;

        let mut regions = Vec::new();
        for row in rows {
            regions.push(self.row_to_region_info(&row)?);
        }
        Ok(regions)
    }

    async fn get_regions(&self, scopeid: Uuid, flags: u32) -> Result<Vec<RegionInfo>> {
        debug!("Getting regions with flags: {} (scope: {})", flags, scopeid);

        let pool = self.get_pg_pool()?;
        let rows = if flags == 0 {
            sqlx::query(
                r#"
                SELECT id, region_name, location_x, location_y, size_x, size_y,
                       serverip, internal_port, external_host_name, flags, scopeid, owner_uuid, 1 as estate_id
                FROM regions
                WHERE (scopeid = $1 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $1 = '00000000-0000-0000-0000-000000000000')
                "#
            )
            .bind(scopeid)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, region_name, location_x, location_y, size_x, size_y,
                       serverip, internal_port, external_host_name, flags, scopeid, owner_uuid, 1 as estate_id
                FROM regions
                WHERE (flags & $1) != 0
                AND (scopeid = $2 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $2 = '00000000-0000-0000-0000-000000000000')
                "#
            )
            .bind(flags as i32)
            .bind(scopeid)
            .fetch_all(pool)
            .await
        }
        .map_err(|e| anyhow!("Failed to get regions: {}", e))?;

        let mut regions = Vec::new();
        for row in rows {
            regions.push(self.row_to_region_info(&row)?);
        }
        Ok(regions)
    }
}

impl LocalGridService {
    fn row_to_region_info(&self, row: &sqlx::postgres::PgRow) -> Result<RegionInfo> {
        use sqlx::Row;

        Ok(RegionInfo {
            region_id: row.try_get("id")?,
            region_name: row.try_get("region_name")?,
            region_loc_x: row.try_get::<i32, _>("location_x")? as u32,
            region_loc_y: row.try_get::<i32, _>("location_y")? as u32,
            region_size_x: row.try_get::<i32, _>("size_x")? as u32,
            region_size_y: row.try_get::<i32, _>("size_y")? as u32,
            server_ip: row.try_get("serverip")?,
            server_port: row.try_get::<i32, _>("internal_port")? as u16,
            server_uri: row.try_get("external_host_name")?,
            region_flags: row.try_get::<i32, _>("flags")? as u32,
            scope_id: row.try_get("scopeid")?,
            owner_id: row.try_get("owner_uuid")?,
            estate_id: row.try_get::<i32, _>("estate_id")? as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_info_default() {
        let info = RegionInfo::default();
        assert_eq!(info.region_size_x, 256);
        assert_eq!(info.region_size_y, 256);
        assert_eq!(info.server_port, 9000);
    }
}
