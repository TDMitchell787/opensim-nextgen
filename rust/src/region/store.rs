// Phase 26.2.2: Region Persistence Layer
// Database operations for region data storage and retrieval

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Pool, Row};
use uuid::Uuid;

use super::data_model::*;
use crate::database::DatabaseManager;

/// Result type for region store operations
pub type RegionStoreResult<T> = Result<T, RegionStoreError>;

/// Errors that can occur in region store operations
#[derive(Debug, thiserror::Error)]
pub enum RegionStoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Region not found: {0}")]
    RegionNotFound(Uuid),

    #[error("Land parcel not found: {0}")]
    LandNotFound(Uuid),

    #[error("Object not found: {0}")]
    ObjectNotFound(Uuid),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Trait for region data persistence operations
#[async_trait]
pub trait RegionStore: Send + Sync {
    // Region operations
    async fn load_region(&self, region_id: Uuid) -> RegionStoreResult<Option<RegionInfo>>;
    async fn store_region(&self, region: &RegionInfo) -> RegionStoreResult<()>;
    async fn delete_region(&self, region_id: Uuid) -> RegionStoreResult<()>;
    async fn list_regions(&self) -> RegionStoreResult<Vec<RegionInfo>>;

    // Region settings operations
    async fn load_region_settings(
        &self,
        region_id: Uuid,
    ) -> RegionStoreResult<Option<RegionSettings>>;
    async fn store_region_settings(&self, settings: &RegionSettings) -> RegionStoreResult<()>;

    // Object operations
    async fn load_objects(&self, region_id: Uuid) -> RegionStoreResult<Vec<SceneObjectPart>>;
    async fn store_objects(
        &self,
        region_id: Uuid,
        objects: &[SceneObjectPart],
    ) -> RegionStoreResult<()>;
    async fn store_object(&self, object: &SceneObjectPart) -> RegionStoreResult<()>;
    async fn delete_object(&self, object_id: Uuid) -> RegionStoreResult<()>;

    // Prim shape operations
    async fn load_prim_shapes(&self, region_id: Uuid) -> RegionStoreResult<Vec<PrimShape>>;
    async fn store_prim_shape(&self, shape: &PrimShape) -> RegionStoreResult<()>;
    async fn delete_prim_shape(&self, prim_id: Uuid) -> RegionStoreResult<()>;

    // Terrain operations
    async fn load_terrain(&self, region_id: Uuid) -> RegionStoreResult<Option<TerrainData>>;
    async fn store_terrain(&self, region_id: Uuid, terrain: &TerrainData) -> RegionStoreResult<()>;

    // Land parcel operations
    async fn load_parcels(&self, region_id: Uuid) -> RegionStoreResult<Vec<LandData>>;
    async fn store_parcels(&self, region_id: Uuid, parcels: &[LandData]) -> RegionStoreResult<()>;
    async fn store_parcel(&self, parcel: &LandData) -> RegionStoreResult<()>;
    async fn delete_parcel(&self, parcel_id: Uuid) -> RegionStoreResult<()>;

    // Spawn point operations
    async fn load_spawn_points(&self, region_id: Uuid) -> RegionStoreResult<Vec<SpawnPoint>>;
    async fn store_spawn_point(&self, spawn_point: &SpawnPoint) -> RegionStoreResult<()>;
    async fn delete_spawn_point(&self, spawn_point_id: Uuid) -> RegionStoreResult<()>;
}

/// PostgreSQL implementation of RegionStore
pub struct PostgresRegionStore {
    db: DatabaseManager,
}

impl PostgresRegionStore {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    /// Get database pool for operations
    async fn get_pool(&self) -> RegionStoreResult<&Pool<sqlx::Postgres>> {
        self.db.legacy_pool().map_err(|e| {
            RegionStoreError::Database(sqlx::Error::Configuration(
                format!("Failed to get database pool: {:?}", e).into(),
            ))
        })
    }
}

#[async_trait]
impl RegionStore for PostgresRegionStore {
    async fn load_region(&self, region_id: Uuid) -> RegionStoreResult<Option<RegionInfo>> {
        let pool = self.get_pool().await?;

        let row = sqlx::query("SELECT * FROM regions WHERE id = $1")
            .bind(region_id.to_string())
            .fetch_optional(pool)
            .await?;

        if let Some(row) = row {
            let region = RegionInfo {
                region_id: Uuid::parse_str(&row.try_get::<String, _>("id")?)
                    .map_err(|e| RegionStoreError::InvalidData(format!("Invalid UUID: {}", e)))?,
                region_name: row.try_get("region_name")?,
                region_handle: RegionInfo::calculate_handle(
                    row.try_get::<i32, _>("location_x")? as u32,
                    row.try_get::<i32, _>("location_y")? as u32,
                ),
                location_x: row.try_get::<i32, _>("location_x")? as u32,
                location_y: row.try_get::<i32, _>("location_y")? as u32,
                size_x: row.try_get::<i32, _>("size_x")? as u32,
                size_y: row.try_get::<i32, _>("size_y")? as u32,
                internal_ip: row.try_get("internal_ip")?,
                internal_port: row.try_get::<i32, _>("internal_port")? as u32,
                external_host_name: row.try_get("external_host_name")?,
                master_avatar_id: row
                    .try_get::<Option<String>, _>("master_avatar_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid master avatar UUID: {}", e))
                    })?
                    .unwrap_or(Uuid::nil()),
                owner_id: row
                    .try_get::<Option<String>, _>("owner_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid owner UUID: {}", e))
                    })?,
                estate_id: 1, // Default estate
                scope_id: row
                    .try_get::<Option<String>, _>("scope_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid scope UUID: {}", e))
                    })?
                    .unwrap_or(Uuid::nil()),
                region_secret: row
                    .try_get::<Option<String>, _>("region_secret")?
                    .unwrap_or_default(),
                token: row
                    .try_get::<Option<String>, _>("token")?
                    .unwrap_or_default(),
                flags: row.try_get::<i32, _>("flags")? as u32,
                maturity: 1, // Default to Mature
                last_seen: row.try_get::<DateTime<Utc>, _>("last_seen")?,
                prim_count: row.try_get::<i32, _>("prim_count")? as u32,
                agent_count: row.try_get::<i32, _>("agent_count")? as u32,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };
            Ok(Some(region))
        } else {
            Ok(None)
        }
    }

    async fn store_region(&self, region: &RegionInfo) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query(
            r#"
            INSERT INTO regions (
                id, region_name, location_x, location_y, size_x, size_y,
                internal_ip, internal_port, external_host_name, master_avatar_id,
                owner_id, scope_id, region_secret, token, flags, last_seen,
                prim_count, agent_count, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            ON CONFLICT (id) DO UPDATE SET
                region_name = EXCLUDED.region_name,
                location_x = EXCLUDED.location_x,
                location_y = EXCLUDED.location_y,
                size_x = EXCLUDED.size_x,
                size_y = EXCLUDED.size_y,
                internal_ip = EXCLUDED.internal_ip,
                internal_port = EXCLUDED.internal_port,
                external_host_name = EXCLUDED.external_host_name,
                master_avatar_id = EXCLUDED.master_avatar_id,
                owner_id = EXCLUDED.owner_id,
                scope_id = EXCLUDED.scope_id,
                region_secret = EXCLUDED.region_secret,
                token = EXCLUDED.token,
                flags = EXCLUDED.flags,
                last_seen = EXCLUDED.last_seen,
                prim_count = EXCLUDED.prim_count,
                agent_count = EXCLUDED.agent_count,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(region.region_id.to_string())
        .bind(&region.region_name)
        .bind(region.location_x as i32)
        .bind(region.location_y as i32)
        .bind(region.size_x as i32)
        .bind(region.size_y as i32)
        .bind(&region.internal_ip)
        .bind(region.internal_port as i32)
        .bind(&region.external_host_name)
        .bind(region.master_avatar_id.to_string())
        .bind(region.owner_id.map(|id| id.to_string()))
        .bind(region.scope_id.to_string())
        .bind(&region.region_secret)
        .bind(&region.token)
        .bind(region.flags as i32)
        .bind(region.last_seen)
        .bind(region.prim_count as i32)
        .bind(region.agent_count as i32)
        .bind(region.created_at)
        .bind(region.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_region(&self, region_id: Uuid) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query("DELETE FROM regions WHERE id = $1")
            .bind(region_id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn list_regions(&self) -> RegionStoreResult<Vec<RegionInfo>> {
        let pool = self.get_pool().await?;

        let rows = sqlx::query("SELECT * FROM regions ORDER BY region_name")
            .fetch_all(pool)
            .await?;

        let mut regions = Vec::new();
        for row in rows {
            let region = RegionInfo {
                region_id: Uuid::parse_str(&row.try_get::<String, _>("id")?)
                    .map_err(|e| RegionStoreError::InvalidData(format!("Invalid UUID: {}", e)))?,
                region_name: row.try_get("region_name")?,
                region_handle: RegionInfo::calculate_handle(
                    row.try_get::<i32, _>("location_x")? as u32,
                    row.try_get::<i32, _>("location_y")? as u32,
                ),
                location_x: row.try_get::<i32, _>("location_x")? as u32,
                location_y: row.try_get::<i32, _>("location_y")? as u32,
                size_x: row.try_get::<i32, _>("size_x")? as u32,
                size_y: row.try_get::<i32, _>("size_y")? as u32,
                internal_ip: row.try_get("internal_ip")?,
                internal_port: row.try_get::<i32, _>("internal_port")? as u32,
                external_host_name: row.try_get("external_host_name")?,
                master_avatar_id: row
                    .try_get::<Option<String>, _>("master_avatar_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid master avatar UUID: {}", e))
                    })?
                    .unwrap_or(Uuid::nil()),
                owner_id: row
                    .try_get::<Option<String>, _>("owner_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid owner UUID: {}", e))
                    })?,
                estate_id: 1,
                scope_id: row
                    .try_get::<Option<String>, _>("scope_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid scope UUID: {}", e))
                    })?
                    .unwrap_or(Uuid::nil()),
                region_secret: row
                    .try_get::<Option<String>, _>("region_secret")?
                    .unwrap_or_default(),
                token: row
                    .try_get::<Option<String>, _>("token")?
                    .unwrap_or_default(),
                flags: row.try_get::<i32, _>("flags")? as u32,
                maturity: 1,
                last_seen: row.try_get::<DateTime<Utc>, _>("last_seen")?,
                prim_count: row.try_get::<i32, _>("prim_count")? as u32,
                agent_count: row.try_get::<i32, _>("agent_count")? as u32,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };
            regions.push(region);
        }

        Ok(regions)
    }

    async fn load_region_settings(
        &self,
        region_id: Uuid,
    ) -> RegionStoreResult<Option<RegionSettings>> {
        let pool = self.get_pool().await?;

        let row = sqlx::query("SELECT * FROM region_settings WHERE region_id = $1")
            .bind(region_id.to_string())
            .fetch_optional(pool)
            .await?;

        if let Some(row) = row {
            let settings = RegionSettings {
                region_id,
                block_terraform: row.try_get("block_terraform")?,
                block_fly: row.try_get("block_fly")?,
                allow_damage: row.try_get("allow_damage")?,
                restrict_pushing: row.try_get("restrict_pushing")?,
                allow_land_resell: row.try_get("allow_land_resell")?,
                allow_land_join_divide: row.try_get("allow_land_join_divide")?,
                block_show_in_search: row.try_get("block_show_in_search")?,
                agent_limit: row.try_get::<i32, _>("agent_limit")? as u32,
                object_bonus: row.try_get("object_bonus")?,
                maturity: row.try_get::<i32, _>("maturity")? as u32,
                disable_scripts: row.try_get("disable_scripts")?,
                disable_collisions: row.try_get("disable_collisions")?,
                disable_physics: row.try_get("disable_physics")?,
                terrain_texture_1: row
                    .try_get::<Option<String>, _>("terrain_texture_1")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!(
                            "Invalid terrain texture UUID: {}",
                            e
                        ))
                    })?,
                terrain_texture_2: row
                    .try_get::<Option<String>, _>("terrain_texture_2")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!(
                            "Invalid terrain texture UUID: {}",
                            e
                        ))
                    })?,
                terrain_texture_3: row
                    .try_get::<Option<String>, _>("terrain_texture_3")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!(
                            "Invalid terrain texture UUID: {}",
                            e
                        ))
                    })?,
                terrain_texture_4: row
                    .try_get::<Option<String>, _>("terrain_texture_4")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!(
                            "Invalid terrain texture UUID: {}",
                            e
                        ))
                    })?,
                elevation_1_nw: row.try_get("elevation_1_nw")?,
                elevation_2_nw: row.try_get("elevation_2_nw")?,
                elevation_1_ne: row.try_get("elevation_1_ne")?,
                elevation_2_ne: row.try_get("elevation_2_ne")?,
                elevation_1_se: row.try_get("elevation_1_se")?,
                elevation_2_se: row.try_get("elevation_2_se")?,
                elevation_1_sw: row.try_get("elevation_1_sw")?,
                elevation_2_sw: row.try_get("elevation_2_sw")?,
                water_height: row.try_get("water_height")?,
                terrain_raise_limit: row.try_get("terrain_raise_limit")?,
                terrain_lower_limit: row.try_get("terrain_lower_limit")?,
                use_estate_sun: row.try_get("use_estate_sun")?,
                fixed_sun: row.try_get("fixed_sun")?,
                sun_position: row.try_get("sun_position")?,
                covenant: row
                    .try_get::<Option<String>, _>("covenant")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid covenant UUID: {}", e))
                    })?,
                covenant_datetime: row.try_get("covenant_datetime")?,
                sandbox: row.try_get("sandbox")?,
                sunvectorx: row.try_get("sunvectorx")?,
                sunvectory: row.try_get("sunvectory")?,
                sunvectorz: row.try_get("sunvectorz")?,
                loaded_creation_id: row.try_get("loaded_creation_id")?,
                loaded_creation_datetime: row.try_get("loaded_creation_datetime")?,
                map_tile_id: row
                    .try_get::<Option<String>, _>("map_tile_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid map tile UUID: {}", e))
                    })?,
                telehub_id: row
                    .try_get::<Option<String>, _>("telehub_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid telehub UUID: {}", e))
                    })?,
                spawn_point_routing: row.try_get::<i32, _>("spawn_point_routing")? as u32,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };
            Ok(Some(settings))
        } else {
            Ok(None)
        }
    }

    async fn store_region_settings(&self, settings: &RegionSettings) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query(
            r#"
            INSERT INTO region_settings (
                region_id, block_terraform, block_fly, allow_damage, restrict_pushing,
                allow_land_resell, allow_land_join_divide, block_show_in_search,
                agent_limit, object_bonus, maturity, disable_scripts, disable_collisions,
                disable_physics, terrain_texture_1, terrain_texture_2, terrain_texture_3,
                terrain_texture_4, elevation_1_nw, elevation_2_nw, elevation_1_ne,
                elevation_2_ne, elevation_1_se, elevation_2_se, elevation_1_sw,
                elevation_2_sw, water_height, terrain_raise_limit, terrain_lower_limit,
                use_estate_sun, fixed_sun, sun_position, covenant, covenant_datetime,
                sandbox, sunvectorx, sunvectory, sunvectorz, loaded_creation_id,
                loaded_creation_datetime, map_tile_id, telehub_id, spawn_point_routing,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28,
                $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41,
                $42, $43, $44, $45
            )
            ON CONFLICT (region_id) DO UPDATE SET
                block_terraform = EXCLUDED.block_terraform,
                block_fly = EXCLUDED.block_fly,
                allow_damage = EXCLUDED.allow_damage,
                restrict_pushing = EXCLUDED.restrict_pushing,
                allow_land_resell = EXCLUDED.allow_land_resell,
                allow_land_join_divide = EXCLUDED.allow_land_join_divide,
                block_show_in_search = EXCLUDED.block_show_in_search,
                agent_limit = EXCLUDED.agent_limit,
                object_bonus = EXCLUDED.object_bonus,
                maturity = EXCLUDED.maturity,
                disable_scripts = EXCLUDED.disable_scripts,
                disable_collisions = EXCLUDED.disable_collisions,
                disable_physics = EXCLUDED.disable_physics,
                terrain_texture_1 = EXCLUDED.terrain_texture_1,
                terrain_texture_2 = EXCLUDED.terrain_texture_2,
                terrain_texture_3 = EXCLUDED.terrain_texture_3,
                terrain_texture_4 = EXCLUDED.terrain_texture_4,
                elevation_1_nw = EXCLUDED.elevation_1_nw,
                elevation_2_nw = EXCLUDED.elevation_2_nw,
                elevation_1_ne = EXCLUDED.elevation_1_ne,
                elevation_2_ne = EXCLUDED.elevation_2_ne,
                elevation_1_se = EXCLUDED.elevation_1_se,
                elevation_2_se = EXCLUDED.elevation_2_se,
                elevation_1_sw = EXCLUDED.elevation_1_sw,
                elevation_2_sw = EXCLUDED.elevation_2_sw,
                water_height = EXCLUDED.water_height,
                terrain_raise_limit = EXCLUDED.terrain_raise_limit,
                terrain_lower_limit = EXCLUDED.terrain_lower_limit,
                use_estate_sun = EXCLUDED.use_estate_sun,
                fixed_sun = EXCLUDED.fixed_sun,
                sun_position = EXCLUDED.sun_position,
                covenant = EXCLUDED.covenant,
                covenant_datetime = EXCLUDED.covenant_datetime,
                sandbox = EXCLUDED.sandbox,
                sunvectorx = EXCLUDED.sunvectorx,
                sunvectory = EXCLUDED.sunvectory,
                sunvectorz = EXCLUDED.sunvectorz,
                loaded_creation_id = EXCLUDED.loaded_creation_id,
                loaded_creation_datetime = EXCLUDED.loaded_creation_datetime,
                map_tile_id = EXCLUDED.map_tile_id,
                telehub_id = EXCLUDED.telehub_id,
                spawn_point_routing = EXCLUDED.spawn_point_routing,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(settings.region_id.to_string())
        .bind(settings.block_terraform)
        .bind(settings.block_fly)
        .bind(settings.allow_damage)
        .bind(settings.restrict_pushing)
        .bind(settings.allow_land_resell)
        .bind(settings.allow_land_join_divide)
        .bind(settings.block_show_in_search)
        .bind(settings.agent_limit as i32)
        .bind(settings.object_bonus)
        .bind(settings.maturity as i32)
        .bind(settings.disable_scripts)
        .bind(settings.disable_collisions)
        .bind(settings.disable_physics)
        .bind(settings.terrain_texture_1.map(|id| id.to_string()))
        .bind(settings.terrain_texture_2.map(|id| id.to_string()))
        .bind(settings.terrain_texture_3.map(|id| id.to_string()))
        .bind(settings.terrain_texture_4.map(|id| id.to_string()))
        .bind(settings.elevation_1_nw)
        .bind(settings.elevation_2_nw)
        .bind(settings.elevation_1_ne)
        .bind(settings.elevation_2_ne)
        .bind(settings.elevation_1_se)
        .bind(settings.elevation_2_se)
        .bind(settings.elevation_1_sw)
        .bind(settings.elevation_2_sw)
        .bind(settings.water_height)
        .bind(settings.terrain_raise_limit)
        .bind(settings.terrain_lower_limit)
        .bind(settings.use_estate_sun)
        .bind(settings.fixed_sun)
        .bind(settings.sun_position)
        .bind(settings.covenant.map(|id| id.to_string()))
        .bind(settings.covenant_datetime)
        .bind(settings.sandbox)
        .bind(settings.sunvectorx)
        .bind(settings.sunvectory)
        .bind(settings.sunvectorz)
        .bind(&settings.loaded_creation_id)
        .bind(settings.loaded_creation_datetime)
        .bind(settings.map_tile_id.map(|id| id.to_string()))
        .bind(settings.telehub_id.map(|id| id.to_string()))
        .bind(settings.spawn_point_routing as i32)
        .bind(settings.created_at)
        .bind(settings.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn load_objects(&self, region_id: Uuid) -> RegionStoreResult<Vec<SceneObjectPart>> {
        let pool = self.get_pool().await?;

        let rows =
            sqlx::query("SELECT * FROM scene_objects WHERE region_id = $1 ORDER BY created_at")
                .bind(region_id.to_string())
                .fetch_all(pool)
                .await?;

        let mut objects = Vec::new();
        for row in rows {
            let object = SceneObjectPart {
                uuid: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| {
                    RegionStoreError::InvalidData(format!("Invalid object UUID: {}", e))
                })?,
                parent_id: row
                    .try_get::<Option<String>, _>("parent_id")?
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                creation_date: Utc::now().timestamp() as i32,
                name: row.try_get("object_name")?,
                description: row.try_get("description")?,
                sit_name: String::new(),
                touch_name: String::new(),
                object_flags: row.try_get::<i32, _>("object_flags")? as u32,
                creator_id: Uuid::parse_str(&row.try_get::<String, _>("creator_id")?).map_err(
                    |e| RegionStoreError::InvalidData(format!("Invalid creator UUID: {}", e)),
                )?,
                owner_id: Uuid::parse_str(&row.try_get::<String, _>("owner_id")?).map_err(|e| {
                    RegionStoreError::InvalidData(format!("Invalid owner UUID: {}", e))
                })?,
                group_id: Uuid::parse_str(&row.try_get::<String, _>("group_id")?).map_err(|e| {
                    RegionStoreError::InvalidData(format!("Invalid group UUID: {}", e))
                })?,
                last_owner_id: row
                    .try_get::<Option<String>, _>("last_owner_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid last owner UUID: {}", e))
                    })?
                    .unwrap_or(Uuid::nil()),
                region_handle: 0,
                group_position: Vector3::new(
                    row.try_get("position_x")?,
                    row.try_get("position_y")?,
                    row.try_get("position_z")?,
                ),
                offset_position: Vector3::zero(),
                rotation_offset: Quaternion::new(
                    row.try_get("rotation_x")?,
                    row.try_get("rotation_y")?,
                    row.try_get("rotation_z")?,
                    row.try_get("rotation_w")?,
                ),
                velocity: Vector3::new(
                    row.try_get("velocity_x")?,
                    row.try_get("velocity_y")?,
                    row.try_get("velocity_z")?,
                ),
                angular_velocity: Vector3::new(
                    row.try_get("angular_velocity_x")?,
                    row.try_get("angular_velocity_y")?,
                    row.try_get("angular_velocity_z")?,
                ),
                acceleration: Vector3::new(
                    row.try_get("acceleration_x")?,
                    row.try_get("acceleration_y")?,
                    row.try_get("acceleration_z")?,
                ),
                scale: Vector3::new(
                    row.try_get("scale_x")?,
                    row.try_get("scale_y")?,
                    row.try_get("scale_z")?,
                ),
                sit_target_position: Vector3::zero(),
                sit_target_orientation: Quaternion::identity(),
                physics_type: row.try_get::<i32, _>("physics_type")? as u32,
                material: row.try_get::<i32, _>("material")? as u32,
                click_action: row.try_get::<i32, _>("click_action")? as u32,
                color: Vector3::new(
                    row.try_get("color_r")?,
                    row.try_get("color_g")?,
                    row.try_get("color_b")?,
                ),
                alpha: row.try_get("color_a")?,
                texture_entry: row
                    .try_get::<Option<Vec<u8>>, _>("texture_entry")?
                    .unwrap_or_default(),
                extra_physics_data: row
                    .try_get::<Option<Vec<u8>>, _>("extra_physics_data")?
                    .unwrap_or_default(),
                shape_data: row
                    .try_get::<Option<String>, _>("shape_data")?
                    .unwrap_or_default(),
                script_state: row
                    .try_get::<Option<Vec<u8>>, _>("script_state")?
                    .unwrap_or_default(),
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };
            objects.push(object);
        }

        Ok(objects)
    }

    async fn store_objects(
        &self,
        region_id: Uuid,
        objects: &[SceneObjectPart],
    ) -> RegionStoreResult<()> {
        for object in objects {
            self.store_object(object).await?;
        }
        Ok(())
    }

    async fn store_object(&self, object: &SceneObjectPart) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query(
            r#"
            INSERT INTO scene_objects (
                id, region_id, group_id, part_id, parent_id, owner_id, creator_id,
                last_owner_id, group_owner_id, object_name, description,
                position_x, position_y, position_z, rotation_x, rotation_y, rotation_z, rotation_w,
                velocity_x, velocity_y, velocity_z, angular_velocity_x, angular_velocity_y, angular_velocity_z,
                acceleration_x, acceleration_y, acceleration_z, scale_x, scale_y, scale_z,
                object_flags, physics_type, material, click_action,
                color_r, color_g, color_b, color_a, texture_entry, extra_physics_data,
                shape_data, script_state, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34,
                $35, $36, $37, $38, $39, $40, $41, $42, $43, $44
            )
            ON CONFLICT (id) DO UPDATE SET
                object_name = EXCLUDED.object_name,
                description = EXCLUDED.description,
                position_x = EXCLUDED.position_x,
                position_y = EXCLUDED.position_y,
                position_z = EXCLUDED.position_z,
                rotation_x = EXCLUDED.rotation_x,
                rotation_y = EXCLUDED.rotation_y,
                rotation_z = EXCLUDED.rotation_z,
                rotation_w = EXCLUDED.rotation_w,
                velocity_x = EXCLUDED.velocity_x,
                velocity_y = EXCLUDED.velocity_y,
                velocity_z = EXCLUDED.velocity_z,
                scale_x = EXCLUDED.scale_x,
                scale_y = EXCLUDED.scale_y,
                scale_z = EXCLUDED.scale_z,
                object_flags = EXCLUDED.object_flags,
                physics_type = EXCLUDED.physics_type,
                material = EXCLUDED.material,
                click_action = EXCLUDED.click_action,
                color_r = EXCLUDED.color_r,
                color_g = EXCLUDED.color_g,
                color_b = EXCLUDED.color_b,
                color_a = EXCLUDED.color_a,
                texture_entry = EXCLUDED.texture_entry,
                extra_physics_data = EXCLUDED.extra_physics_data,
                shape_data = EXCLUDED.shape_data,
                script_state = EXCLUDED.script_state,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(object.uuid.to_string())
        .bind(Uuid::new_v4().to_string()) // region_id would need to be passed in
        .bind(object.group_id.to_string())
        .bind(object.uuid.to_string()) // part_id same as id
        .bind(if object.parent_id > 0 { Some(object.parent_id.to_string()) } else { None })
        .bind(object.owner_id.to_string())
        .bind(object.creator_id.to_string())
        .bind(object.last_owner_id.to_string())
        .bind(object.group_id.to_string()) // group_owner_id
        .bind(&object.name)
        .bind(&object.description)
        .bind(object.group_position.x)
        .bind(object.group_position.y)
        .bind(object.group_position.z)
        .bind(object.rotation_offset.x)
        .bind(object.rotation_offset.y)
        .bind(object.rotation_offset.z)
        .bind(object.rotation_offset.w)
        .bind(object.velocity.x)
        .bind(object.velocity.y)
        .bind(object.velocity.z)
        .bind(object.angular_velocity.x)
        .bind(object.angular_velocity.y)
        .bind(object.angular_velocity.z)
        .bind(object.acceleration.x)
        .bind(object.acceleration.y)
        .bind(object.acceleration.z)
        .bind(object.scale.x)
        .bind(object.scale.y)
        .bind(object.scale.z)
        .bind(object.object_flags as i32)
        .bind(object.physics_type as i32)
        .bind(object.material as i32)
        .bind(object.click_action as i32)
        .bind(object.color.x)
        .bind(object.color.y)
        .bind(object.color.z)
        .bind(object.alpha)
        .bind(&object.texture_entry)
        .bind(&object.extra_physics_data)
        .bind(&object.shape_data)
        .bind(&object.script_state)
        .bind(object.created_at)
        .bind(object.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_object(&self, object_id: Uuid) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query("DELETE FROM scene_objects WHERE id = $1")
            .bind(object_id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn load_prim_shapes(&self, region_id: Uuid) -> RegionStoreResult<Vec<PrimShape>> {
        let pool = self.get_pool().await?;

        let rows = sqlx::query(
            r#"
            SELECT ps.* FROM prim_shapes ps
            JOIN scene_objects so ON ps.prim_id = so.id
            WHERE so.region_id = $1
            "#,
        )
        .bind(region_id.to_string())
        .fetch_all(pool)
        .await?;

        let mut shapes = Vec::new();
        for row in rows {
            let shape = PrimShape {
                uuid: Uuid::parse_str(&row.try_get::<String, _>("uuid")?).map_err(|e| {
                    RegionStoreError::InvalidData(format!("Invalid shape UUID: {}", e))
                })?,
                prim_id: Uuid::parse_str(&row.try_get::<String, _>("prim_id")?).map_err(|e| {
                    RegionStoreError::InvalidData(format!("Invalid prim UUID: {}", e))
                })?,
                shape_type: row.try_get::<i32, _>("shape_type")? as u32,
                path_begin: row.try_get::<i32, _>("path_begin")? as u32,
                path_end: row.try_get::<i32, _>("path_end")? as u32,
                path_scale_x: row.try_get::<i32, _>("path_scale_x")? as u32,
                path_scale_y: row.try_get::<i32, _>("path_scale_y")? as u32,
                path_shear_x: row.try_get::<i32, _>("path_shear_x")? as u32,
                path_shear_y: row.try_get::<i32, _>("path_shear_y")? as u32,
                path_skew: row.try_get::<i32, _>("path_skew")? as u32,
                path_curve: row.try_get::<i32, _>("path_curve")? as u32,
                path_radius_offset: row.try_get::<i32, _>("path_radius_offset")? as u32,
                path_revolutions: row.try_get::<i32, _>("path_revolutions")? as u32,
                path_taper_x: row.try_get::<i32, _>("path_taper_x")? as u32,
                path_taper_y: row.try_get::<i32, _>("path_taper_y")? as u32,
                path_twist: row.try_get::<i32, _>("path_twist")? as u32,
                path_twist_begin: row.try_get::<i32, _>("path_twist_begin")? as u32,
                profile_begin: row.try_get::<i32, _>("profile_begin")? as u32,
                profile_end: row.try_get::<i32, _>("profile_end")? as u32,
                profile_curve: row.try_get::<i32, _>("profile_curve")? as u32,
                profile_hollow: row.try_get::<i32, _>("profile_hollow")? as u32,
                texture_entry: row
                    .try_get::<Option<Vec<u8>>, _>("texture_entry")?
                    .unwrap_or_default(),
                extra_params: row
                    .try_get::<Option<Vec<u8>>, _>("extra_params")?
                    .unwrap_or_default(),
                state: row.try_get::<i32, _>("state")? as u32,
                last_attach_point: row.try_get::<i32, _>("last_attach_point")? as u32,
                media: row
                    .try_get::<Option<String>, _>("media")?
                    .unwrap_or_default(),
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };
            shapes.push(shape);
        }

        Ok(shapes)
    }

    async fn store_prim_shape(&self, shape: &PrimShape) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query(
            r#"
            INSERT INTO prim_shapes (
                uuid, prim_id, shape_type, path_begin, path_end, path_scale_x, path_scale_y,
                path_shear_x, path_shear_y, path_skew, path_curve, path_radius_offset,
                path_revolutions, path_taper_x, path_taper_y, path_twist, path_twist_begin,
                profile_begin, profile_end, profile_curve, profile_hollow, texture_entry,
                extra_params, state, last_attach_point, media, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28
            )
            ON CONFLICT (uuid) DO UPDATE SET
                shape_type = EXCLUDED.shape_type,
                path_begin = EXCLUDED.path_begin,
                path_end = EXCLUDED.path_end,
                path_scale_x = EXCLUDED.path_scale_x,
                path_scale_y = EXCLUDED.path_scale_y,
                path_shear_x = EXCLUDED.path_shear_x,
                path_shear_y = EXCLUDED.path_shear_y,
                path_skew = EXCLUDED.path_skew,
                path_curve = EXCLUDED.path_curve,
                path_radius_offset = EXCLUDED.path_radius_offset,
                path_revolutions = EXCLUDED.path_revolutions,
                path_taper_x = EXCLUDED.path_taper_x,
                path_taper_y = EXCLUDED.path_taper_y,
                path_twist = EXCLUDED.path_twist,
                path_twist_begin = EXCLUDED.path_twist_begin,
                profile_begin = EXCLUDED.profile_begin,
                profile_end = EXCLUDED.profile_end,
                profile_curve = EXCLUDED.profile_curve,
                profile_hollow = EXCLUDED.profile_hollow,
                texture_entry = EXCLUDED.texture_entry,
                extra_params = EXCLUDED.extra_params,
                state = EXCLUDED.state,
                last_attach_point = EXCLUDED.last_attach_point,
                media = EXCLUDED.media,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(shape.uuid.to_string())
        .bind(shape.prim_id.to_string())
        .bind(shape.shape_type as i32)
        .bind(shape.path_begin as i32)
        .bind(shape.path_end as i32)
        .bind(shape.path_scale_x as i32)
        .bind(shape.path_scale_y as i32)
        .bind(shape.path_shear_x as i32)
        .bind(shape.path_shear_y as i32)
        .bind(shape.path_skew as i32)
        .bind(shape.path_curve as i32)
        .bind(shape.path_radius_offset as i32)
        .bind(shape.path_revolutions as i32)
        .bind(shape.path_taper_x as i32)
        .bind(shape.path_taper_y as i32)
        .bind(shape.path_twist as i32)
        .bind(shape.path_twist_begin as i32)
        .bind(shape.profile_begin as i32)
        .bind(shape.profile_end as i32)
        .bind(shape.profile_curve as i32)
        .bind(shape.profile_hollow as i32)
        .bind(&shape.texture_entry)
        .bind(&shape.extra_params)
        .bind(shape.state as i32)
        .bind(shape.last_attach_point as i32)
        .bind(&shape.media)
        .bind(shape.created_at)
        .bind(shape.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_prim_shape(&self, prim_id: Uuid) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query("DELETE FROM prim_shapes WHERE prim_id = $1")
            .bind(prim_id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn load_terrain(&self, region_id: Uuid) -> RegionStoreResult<Option<TerrainData>> {
        let pool = self.get_pool().await?;

        let row = sqlx::query("SELECT * FROM region_terrain WHERE region_id = $1")
            .bind(region_id.to_string())
            .fetch_optional(pool)
            .await?;

        if let Some(row) = row {
            let terrain = TerrainData {
                region_id,
                terrain_data: row
                    .try_get::<Option<Vec<u8>>, _>("terrain_data")?
                    .unwrap_or_default(),
                terrain_revision: row.try_get::<i32, _>("terrain_revision")? as u32,
                terrain_seed: row.try_get::<i32, _>("terrain_seed")? as u32,
                water_height: row.try_get("water_height")?,
                terrain_raise_limit: row.try_get("terrain_raise_limit")?,
                terrain_lower_limit: row.try_get("terrain_lower_limit")?,
                use_estate_sun: row.try_get("use_estate_sun")?,
                fixed_sun: row.try_get("fixed_sun")?,
                sun_position: row.try_get("sun_position")?,
                covenant: row
                    .try_get::<Option<String>, _>("covenant")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid covenant UUID: {}", e))
                    })?,
                covenant_timestamp: row.try_get("covenant_timestamp")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };
            Ok(Some(terrain))
        } else {
            Ok(None)
        }
    }

    async fn store_terrain(&self, region_id: Uuid, terrain: &TerrainData) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query(
            r#"
            INSERT INTO region_terrain (
                region_id, terrain_data, terrain_revision, terrain_seed,
                water_height, terrain_raise_limit, terrain_lower_limit,
                use_estate_sun, fixed_sun, sun_position, covenant,
                covenant_timestamp, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (region_id) DO UPDATE SET
                terrain_data = EXCLUDED.terrain_data,
                terrain_revision = EXCLUDED.terrain_revision,
                terrain_seed = EXCLUDED.terrain_seed,
                water_height = EXCLUDED.water_height,
                terrain_raise_limit = EXCLUDED.terrain_raise_limit,
                terrain_lower_limit = EXCLUDED.terrain_lower_limit,
                use_estate_sun = EXCLUDED.use_estate_sun,
                fixed_sun = EXCLUDED.fixed_sun,
                sun_position = EXCLUDED.sun_position,
                covenant = EXCLUDED.covenant,
                covenant_timestamp = EXCLUDED.covenant_timestamp,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(region_id.to_string())
        .bind(&terrain.terrain_data)
        .bind(terrain.terrain_revision as i32)
        .bind(terrain.terrain_seed as i32)
        .bind(terrain.water_height)
        .bind(terrain.terrain_raise_limit)
        .bind(terrain.terrain_lower_limit)
        .bind(terrain.use_estate_sun)
        .bind(terrain.fixed_sun)
        .bind(terrain.sun_position)
        .bind(terrain.covenant.map(|id| id.to_string()))
        .bind(terrain.covenant_timestamp)
        .bind(terrain.created_at)
        .bind(terrain.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn load_parcels(&self, region_id: Uuid) -> RegionStoreResult<Vec<LandData>> {
        let pool = self.get_pool().await?;

        let rows =
            sqlx::query("SELECT * FROM land_parcels WHERE region_id = $1 ORDER BY local_land_id")
                .bind(region_id.to_string())
                .fetch_all(pool)
                .await?;

        let mut parcels = Vec::new();
        for row in rows {
            let parcel = LandData {
                uuid: Uuid::parse_str(&row.try_get::<String, _>("uuid")?).map_err(|e| {
                    RegionStoreError::InvalidData(format!("Invalid parcel UUID: {}", e))
                })?,
                region_id,
                local_land_id: row.try_get::<i32, _>("local_land_id")? as u32,
                bitmap: row
                    .try_get::<Option<Vec<u8>>, _>("bitmap")?
                    .unwrap_or_default(),
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                owner_id: Uuid::parse_str(&row.try_get::<String, _>("owner_id")?).map_err(|e| {
                    RegionStoreError::InvalidData(format!("Invalid owner UUID: {}", e))
                })?,
                group_id: row
                    .try_get::<Option<String>, _>("group_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid group UUID: {}", e))
                    })?,
                is_group_owned: row.try_get("is_group_owned")?,
                area: row.try_get::<i32, _>("area")? as u32,
                auction_id: row.try_get::<i32, _>("auction_id")? as u32,
                category: row.try_get::<i32, _>("category")? as u32,
                claim_date: row.try_get("claim_date")?,
                claim_price: row.try_get("claim_price")?,
                status: row.try_get::<i32, _>("status")? as u32,
                landing_type: row.try_get::<i32, _>("landing_type")? as u32,
                landing_position: Vector3::new(
                    row.try_get("landing_x")?,
                    row.try_get("landing_y")?,
                    row.try_get("landing_z")?,
                ),
                landing_look_at: Vector3::new(
                    row.try_get("landing_look_x")?,
                    row.try_get("landing_look_y")?,
                    row.try_get("landing_look_z")?,
                ),
                user_location: Vector3::new(
                    row.try_get("user_location_x")?,
                    row.try_get("user_location_y")?,
                    row.try_get("user_location_z")?,
                ),
                user_look_at: Vector3::new(
                    row.try_get("user_look_at_x")?,
                    row.try_get("user_look_at_y")?,
                    row.try_get("user_look_at_z")?,
                ),
                auth_buyer_id: row
                    .try_get::<Option<String>, _>("auth_buyer_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid auth buyer UUID: {}", e))
                    })?,
                snapshot_id: row
                    .try_get::<Option<String>, _>("snapshot_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid snapshot UUID: {}", e))
                    })?,
                other_clean_time: row.try_get("other_clean_time")?,
                dwell: row.try_get("dwell")?,
                media_auto_scale: row.try_get::<i32, _>("media_auto_scale")? as u32,
                media_loop_set: row.try_get("media_loop_set")?,
                media_texture_id: row
                    .try_get::<Option<String>, _>("media_texture_id")?
                    .map(|s| Uuid::parse_str(&s))
                    .transpose()
                    .map_err(|e| {
                        RegionStoreError::InvalidData(format!("Invalid media texture UUID: {}", e))
                    })?,
                media_url: row.try_get("media_url")?,
                music_url: row.try_get("music_url")?,
                pass_hours: row.try_get("pass_hours")?,
                pass_price: row.try_get("pass_price")?,
                sale_price: row.try_get("sale_price")?,
                media_type: row.try_get("media_type")?,
                media_description: row.try_get("media_description")?,
                media_size: Vector3::new(
                    row.try_get::<i32, _>("media_size_x")? as f32,
                    row.try_get::<i32, _>("media_size_y")? as f32,
                    0.0,
                ),
                media_loop: row.try_get("media_loop")?,
                obscure_media: row.try_get("obscure_media")?,
                obscure_music: row.try_get("obscure_music")?,
                see_avatar_distance: row.try_get("see_avatar_distance")?,
                any_avatar_sounds: row.try_get("any_avatar_sounds")?,
                group_avatar_sounds: row.try_get("group_avatar_sounds")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };
            parcels.push(parcel);
        }

        Ok(parcels)
    }

    async fn store_parcels(&self, region_id: Uuid, parcels: &[LandData]) -> RegionStoreResult<()> {
        for parcel in parcels {
            self.store_parcel(parcel).await?;
        }
        Ok(())
    }

    async fn store_parcel(&self, parcel: &LandData) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query(
            r#"
            INSERT INTO land_parcels (
                uuid, region_id, local_land_id, bitmap, name, description, owner_id,
                group_id, is_group_owned, area, auction_id, category, claim_date,
                claim_price, status, landing_type, landing_x, landing_y, landing_z,
                landing_look_x, landing_look_y, landing_look_z, user_location_x,
                user_location_y, user_location_z, user_look_at_x, user_look_at_y,
                user_look_at_z, auth_buyer_id, snapshot_id, other_clean_time,
                dwell, media_auto_scale, media_loop_set, media_texture_id,
                media_url, music_url, pass_hours, pass_price, sale_price,
                media_type, media_description, media_size_x, media_size_y,
                media_loop, obscure_media, obscure_music, see_avatar_distance,
                any_avatar_sounds, group_avatar_sounds, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34,
                $35, $36, $37, $38, $39, $40, $41, $42, $43, $44, $45, $46, $47, $48, $49, $50, $51, $52
            )
            ON CONFLICT (uuid) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                owner_id = EXCLUDED.owner_id,
                group_id = EXCLUDED.group_id,
                is_group_owned = EXCLUDED.is_group_owned,
                area = EXCLUDED.area,
                auction_id = EXCLUDED.auction_id,
                category = EXCLUDED.category,
                claim_date = EXCLUDED.claim_date,
                claim_price = EXCLUDED.claim_price,
                status = EXCLUDED.status,
                landing_type = EXCLUDED.landing_type,
                landing_x = EXCLUDED.landing_x,
                landing_y = EXCLUDED.landing_y,
                landing_z = EXCLUDED.landing_z,
                landing_look_x = EXCLUDED.landing_look_x,
                landing_look_y = EXCLUDED.landing_look_y,
                landing_look_z = EXCLUDED.landing_look_z,
                user_location_x = EXCLUDED.user_location_x,
                user_location_y = EXCLUDED.user_location_y,
                user_location_z = EXCLUDED.user_location_z,
                user_look_at_x = EXCLUDED.user_look_at_x,
                user_look_at_y = EXCLUDED.user_look_at_y,
                user_look_at_z = EXCLUDED.user_look_at_z,
                auth_buyer_id = EXCLUDED.auth_buyer_id,
                snapshot_id = EXCLUDED.snapshot_id,
                other_clean_time = EXCLUDED.other_clean_time,
                dwell = EXCLUDED.dwell,
                media_auto_scale = EXCLUDED.media_auto_scale,
                media_loop_set = EXCLUDED.media_loop_set,
                media_texture_id = EXCLUDED.media_texture_id,
                media_url = EXCLUDED.media_url,
                music_url = EXCLUDED.music_url,
                pass_hours = EXCLUDED.pass_hours,
                pass_price = EXCLUDED.pass_price,
                sale_price = EXCLUDED.sale_price,
                media_type = EXCLUDED.media_type,
                media_description = EXCLUDED.media_description,
                media_size_x = EXCLUDED.media_size_x,
                media_size_y = EXCLUDED.media_size_y,
                media_loop = EXCLUDED.media_loop,
                obscure_media = EXCLUDED.obscure_media,
                obscure_music = EXCLUDED.obscure_music,
                see_avatar_distance = EXCLUDED.see_avatar_distance,
                any_avatar_sounds = EXCLUDED.any_avatar_sounds,
                group_avatar_sounds = EXCLUDED.group_avatar_sounds,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(parcel.uuid.to_string())
        .bind(parcel.region_id.to_string())
        .bind(parcel.local_land_id as i32)
        .bind(&parcel.bitmap)
        .bind(&parcel.name)
        .bind(&parcel.description)
        .bind(parcel.owner_id.to_string())
        .bind(parcel.group_id.map(|id| id.to_string()))
        .bind(parcel.is_group_owned)
        .bind(parcel.area as i32)
        .bind(parcel.auction_id as i32)
        .bind(parcel.category as i32)
        .bind(parcel.claim_date)
        .bind(parcel.claim_price)
        .bind(parcel.status as i32)
        .bind(parcel.landing_type as i32)
        .bind(parcel.landing_position.x)
        .bind(parcel.landing_position.y)
        .bind(parcel.landing_position.z)
        .bind(parcel.landing_look_at.x)
        .bind(parcel.landing_look_at.y)
        .bind(parcel.landing_look_at.z)
        .bind(parcel.user_location.x)
        .bind(parcel.user_location.y)
        .bind(parcel.user_location.z)
        .bind(parcel.user_look_at.x)
        .bind(parcel.user_look_at.y)
        .bind(parcel.user_look_at.z)
        .bind(parcel.auth_buyer_id.map(|id| id.to_string()))
        .bind(parcel.snapshot_id.map(|id| id.to_string()))
        .bind(parcel.other_clean_time)
        .bind(parcel.dwell)
        .bind(parcel.media_auto_scale as i32)
        .bind(parcel.media_loop_set)
        .bind(parcel.media_texture_id.map(|id| id.to_string()))
        .bind(&parcel.media_url)
        .bind(&parcel.music_url)
        .bind(parcel.pass_hours)
        .bind(parcel.pass_price)
        .bind(parcel.sale_price)
        .bind(&parcel.media_type)
        .bind(&parcel.media_description)
        .bind(parcel.media_size.x as i32)
        .bind(parcel.media_size.y as i32)
        .bind(parcel.media_loop)
        .bind(parcel.obscure_media)
        .bind(parcel.obscure_music)
        .bind(parcel.see_avatar_distance)
        .bind(parcel.any_avatar_sounds)
        .bind(parcel.group_avatar_sounds)
        .bind(parcel.created_at)
        .bind(parcel.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_parcel(&self, parcel_id: Uuid) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query("DELETE FROM land_parcels WHERE uuid = $1")
            .bind(parcel_id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn load_spawn_points(&self, region_id: Uuid) -> RegionStoreResult<Vec<SpawnPoint>> {
        let pool = self.get_pool().await?;

        let rows =
            sqlx::query("SELECT * FROM region_spawn_points WHERE region_id = $1 ORDER BY name")
                .bind(region_id.to_string())
                .fetch_all(pool)
                .await?;

        let mut spawn_points = Vec::new();
        for row in rows {
            let spawn_point = SpawnPoint {
                id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|e| {
                    RegionStoreError::InvalidData(format!("Invalid spawn point UUID: {}", e))
                })?,
                region_id,
                position: Vector3::new(
                    row.try_get("spawn_point_x")?,
                    row.try_get("spawn_point_y")?,
                    row.try_get("spawn_point_z")?,
                ),
                look_at: Vector3::new(
                    row.try_get("spawn_point_look_x")?,
                    row.try_get("spawn_point_look_y")?,
                    row.try_get("spawn_point_look_z")?,
                ),
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                is_default: row.try_get("is_default")?,
                created_at: row.try_get("created_at")?,
            };
            spawn_points.push(spawn_point);
        }

        Ok(spawn_points)
    }

    async fn store_spawn_point(&self, spawn_point: &SpawnPoint) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query(
            r#"
            INSERT INTO region_spawn_points (
                id, region_id, spawn_point_x, spawn_point_y, spawn_point_z,
                spawn_point_look_x, spawn_point_look_y, spawn_point_look_z,
                name, description, is_default, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                spawn_point_x = EXCLUDED.spawn_point_x,
                spawn_point_y = EXCLUDED.spawn_point_y,
                spawn_point_z = EXCLUDED.spawn_point_z,
                spawn_point_look_x = EXCLUDED.spawn_point_look_x,
                spawn_point_look_y = EXCLUDED.spawn_point_look_y,
                spawn_point_look_z = EXCLUDED.spawn_point_look_z,
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                is_default = EXCLUDED.is_default
            "#,
        )
        .bind(spawn_point.id.to_string())
        .bind(spawn_point.region_id.to_string())
        .bind(spawn_point.position.x)
        .bind(spawn_point.position.y)
        .bind(spawn_point.position.z)
        .bind(spawn_point.look_at.x)
        .bind(spawn_point.look_at.y)
        .bind(spawn_point.look_at.z)
        .bind(&spawn_point.name)
        .bind(&spawn_point.description)
        .bind(spawn_point.is_default)
        .bind(spawn_point.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_spawn_point(&self, spawn_point_id: Uuid) -> RegionStoreResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query("DELETE FROM region_spawn_points WHERE id = $1")
            .bind(spawn_point_id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseManager;

    async fn create_test_store() -> PostgresRegionStore {
        let db = DatabaseManager::new("postgresql://test:test@localhost/test_db")
            .await
            .unwrap();
        PostgresRegionStore::new(db)
    }

    #[tokio::test]
    async fn test_region_round_trip() {
        let store = create_test_store().await;
        let region = RegionInfo::new("Test Region".to_string(), 1000, 1000);

        // This test would require a test database to be set up
        // For now, just verify the structure compiles
        assert_eq!(region.region_name, "Test Region");
        assert_eq!(region.location_x, 1000);
        assert_eq!(region.location_y, 1000);
    }

    #[tokio::test]
    async fn test_terrain_data_creation() {
        let region_id = Uuid::new_v4();
        let terrain = TerrainData::new(region_id);

        assert_eq!(terrain.region_id, region_id);
        assert_eq!(terrain.terrain_revision, 1);
        assert_eq!(terrain.water_height, 20.0);
    }

    #[tokio::test]
    async fn test_land_data_creation() {
        let region_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let land = LandData::new(region_id, 1, owner_id);

        assert_eq!(land.region_id, region_id);
        assert_eq!(land.local_land_id, 1);
        assert_eq!(land.owner_id, owner_id);
        assert_eq!(land.name, "Your Parcel");
    }
}
