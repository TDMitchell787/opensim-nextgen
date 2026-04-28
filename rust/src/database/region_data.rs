//! Region data persistence and management

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::multi_backend::DatabaseConnection;

/// Region configuration data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RegionData {
    pub id: Uuid,
    pub region_name: String,
    pub location_x: i32,
    pub location_y: i32,
    pub size_x: i32,
    pub size_y: i32,
    pub internal_ip: String,
    pub internal_port: i32,
    pub external_host_name: String,
    pub master_avatar_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub scope_id: Uuid,
    pub region_secret: Option<String>,
    pub token: Option<String>,
    pub flags: i32,
    pub last_seen: DateTime<Utc>,
    pub prim_count: i32,
    pub agent_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Region terrain data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RegionTerrain {
    pub region_id: Uuid,
    pub terrain_data: Option<Vec<u8>>,
    pub terrain_revision: i32,
    pub terrain_seed: i32,
    pub water_height: f32,
    pub terrain_raise_limit: f32,
    pub terrain_lower_limit: f32,
    pub use_estate_sun: bool,
    pub fixed_sun: bool,
    pub sun_position: f32,
    pub covenant: Option<Uuid>,
    pub covenant_timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scene object (prim) data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SceneObject {
    pub id: Uuid,
    pub region_id: Uuid,
    pub group_id: Uuid,
    pub part_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub creator_id: Uuid,
    pub last_owner_id: Option<Uuid>,
    pub group_owner_id: Option<Uuid>,
    pub object_name: String,
    pub description: String,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub rotation_w: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub angular_velocity_x: f32,
    pub angular_velocity_y: f32,
    pub angular_velocity_z: f32,
    pub acceleration_x: f32,
    pub acceleration_y: f32,
    pub acceleration_z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    pub object_flags: i32,
    pub physics_type: i32,
    pub material: i32,
    pub click_action: i32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub color_a: f32,
    pub texture_entry: Option<Vec<u8>>,
    pub extra_physics_data: Option<Vec<u8>>,
    pub shape_data: Option<String>,
    pub script_state: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Estate data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EstateData {
    pub id: i32,
    pub estate_name: String,
    pub estate_owner: Uuid,
    pub parent_estate_id: Option<i32>,
    pub flags: i32,
    pub bill_cycle: i32,
    pub price_per_meter: i32,
    pub redirect_grid_x: i32,
    pub redirect_grid_y: i32,
    pub force_landing: bool,
    pub reset_home_on_teleport: bool,
    pub deny_anonymous: bool,
    pub deny_identified: bool,
    pub deny_transacted: bool,
    pub abuse_email: String,
    pub estate_owner_email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create region request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRegionRequest {
    pub region_name: String,
    pub location_x: i32,
    pub location_y: i32,
    pub size_x: Option<i32>,
    pub size_y: Option<i32>,
    pub internal_ip: String,
    pub internal_port: i32,
    pub external_host_name: String,
    pub master_avatar_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
}

/// Update region request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRegionRequest {
    pub region_name: Option<String>,
    pub internal_ip: Option<String>,
    pub internal_port: Option<i32>,
    pub external_host_name: Option<String>,
    pub master_avatar_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub flags: Option<i32>,
    pub prim_count: Option<i32>,
    pub agent_count: Option<i32>,
}

/// Region database operations
#[derive(Debug)]
pub struct RegionDatabase {
    connection: Option<Arc<DatabaseConnection>>,
}

impl RegionDatabase {
    /// Create a new region database
    pub async fn new(connection: Arc<DatabaseConnection>) -> Result<Self> {
        info!("Initializing region database");
        Ok(Self {
            connection: Some(connection),
        })
    }

    /// Create a stub region database for SQLite compatibility
    pub fn new_stub() -> Self {
        info!("Creating stub region database for SQLite compatibility");
        Self { connection: None }
    }

    /// Get database connection pool (handles stub mode gracefully)
    fn pool(&self) -> Result<&sqlx::PgPool> {
        match &self.connection {
            Some(conn) => match conn.as_ref() {
                super::multi_backend::DatabaseConnection::PostgreSQL(pool) => Ok(pool),
                _ => Err(anyhow!("Database is not PostgreSQL")),
            },
            None => Err(anyhow!("Database operation not available in stub mode")),
        }
    }

    /// Create a new region
    pub async fn create_region(&self, request: CreateRegionRequest) -> Result<RegionData> {
        info!(
            "Creating new region: {} at ({}, {})",
            request.region_name, request.location_x, request.location_y
        );

        let region = sqlx::query_as::<_, RegionData>(
            r#"
            INSERT INTO regions (
                region_name, location_x, location_y, size_x, size_y,
                internal_ip, internal_port, external_host_name,
                master_avatar_id, owner_id, created_at, updated_at, last_seen
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(&request.region_name)
        .bind(request.location_x)
        .bind(request.location_y)
        .bind(request.size_x.unwrap_or(256))
        .bind(request.size_y.unwrap_or(256))
        .bind(&request.internal_ip)
        .bind(request.internal_port)
        .bind(&request.external_host_name)
        .bind(request.master_avatar_id)
        .bind(request.owner_id)
        .fetch_one(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to create region: {}", e))?;

        // Create default terrain
        self.create_default_terrain(region.id).await?;

        info!("Created region: {} ({})", region.region_name, region.id);
        Ok(region)
    }

    /// Get region by ID
    pub async fn get_region_by_id(&self, region_id: Uuid) -> Result<Option<RegionData>> {
        debug!("Getting region by ID: {}", region_id);

        let region = sqlx::query_as::<_, RegionData>("SELECT * FROM regions WHERE id = $1")
            .bind(region_id)
            .fetch_optional(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to get region by ID: {}", e))?;

        Ok(region)
    }

    /// Get region by name
    pub async fn get_region_by_name(&self, region_name: &str) -> Result<Option<RegionData>> {
        debug!("Getting region by name: {}", region_name);

        let region =
            sqlx::query_as::<_, RegionData>("SELECT * FROM regions WHERE region_name = $1")
                .bind(region_name)
                .fetch_optional(self.pool()?)
                .await
                .map_err(|e| anyhow!("Failed to get region by name: {}", e))?;

        Ok(region)
    }

    /// Get region by location
    pub async fn get_region_by_location(&self, x: i32, y: i32) -> Result<Option<RegionData>> {
        debug!("Getting region by location: ({}, {})", x, y);

        let region = sqlx::query_as::<_, RegionData>(
            "SELECT * FROM regions WHERE location_x = $1 AND location_y = $2",
        )
        .bind(x)
        .bind(y)
        .fetch_optional(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to get region by location: {}", e))?;

        Ok(region)
    }

    /// Update region data
    pub async fn update_region(
        &self,
        region_id: Uuid,
        request: UpdateRegionRequest,
    ) -> Result<RegionData> {
        debug!("Updating region: {}", region_id);

        // Build dynamic update query
        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 1;

        if let Some(region_name) = &request.region_name {
            updates.push(format!("region_name = ${}", param_count));
            params.push(Box::new(region_name.clone()));
            param_count += 1;
        }

        if let Some(internal_ip) = &request.internal_ip {
            updates.push(format!("internal_ip = ${}", param_count));
            params.push(Box::new(internal_ip.clone()));
            param_count += 1;
        }

        if let Some(internal_port) = request.internal_port {
            updates.push(format!("internal_port = ${}", param_count));
            params.push(Box::new(internal_port));
            param_count += 1;
        }

        if let Some(external_host_name) = &request.external_host_name {
            updates.push(format!("external_host_name = ${}", param_count));
            params.push(Box::new(external_host_name.clone()));
            param_count += 1;
        }

        if let Some(master_avatar_id) = request.master_avatar_id {
            updates.push(format!("master_avatar_id = ${}", param_count));
            params.push(Box::new(master_avatar_id));
            param_count += 1;
        }

        if let Some(owner_id) = request.owner_id {
            updates.push(format!("owner_id = ${}", param_count));
            params.push(Box::new(owner_id));
            param_count += 1;
        }

        if let Some(flags) = request.flags {
            updates.push(format!("flags = ${}", param_count));
            params.push(Box::new(flags));
            param_count += 1;
        }

        if let Some(prim_count) = request.prim_count {
            updates.push(format!("prim_count = ${}", param_count));
            params.push(Box::new(prim_count));
            param_count += 1;
        }

        if let Some(agent_count) = request.agent_count {
            updates.push(format!("agent_count = ${}", param_count));
            params.push(Box::new(agent_count));
            param_count += 1;
        }

        if updates.is_empty() {
            return Err(anyhow!("No fields to update"));
        }

        updates.push("updated_at = NOW()".to_string());
        updates.push("last_seen = NOW()".to_string());

        let query = format!(
            "UPDATE regions SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_count
        );

        // For simplicity, use a manual approach (in production, use a proper query builder)
        let updated_region = sqlx::query_as::<_, RegionData>(&query)
            .bind(region_id)
            .fetch_one(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to update region: {}", e))?;

        info!(
            "Updated region: {} ({})",
            updated_region.region_name, updated_region.id
        );
        Ok(updated_region)
    }

    /// Delete region
    pub async fn delete_region(&self, region_id: Uuid) -> Result<bool> {
        warn!("Deleting region: {}", region_id);

        // Delete in transaction to ensure consistency
        let connection = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow!("Database not available in stub mode"))?;
        let mut tx = connection.begin_transaction().await?;

        // Delete terrain data
        let pg_tx = tx.as_postgres_tx()?;

        let _ = sqlx::query("DELETE FROM region_terrain WHERE region_id = $1")
            .bind(region_id)
            .execute(&mut **pg_tx)
            .await;

        // Delete scene objects
        let _ = sqlx::query("DELETE FROM scene_objects WHERE region_id = $1")
            .bind(region_id)
            .execute(&mut **pg_tx)
            .await;

        // Delete region
        let result = sqlx::query("DELETE FROM regions WHERE id = $1")
            .bind(region_id)
            .execute(&mut **pg_tx)
            .await
            .map_err(|e| anyhow!("Failed to delete region: {}", e))?;

        tx.commit()
            .await
            .map_err(|e| anyhow!("Failed to commit region deletion: {}", e))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted region: {}", region_id);
        }

        Ok(deleted)
    }

    /// Get region terrain
    pub async fn get_region_terrain(&self, region_id: Uuid) -> Result<Option<RegionTerrain>> {
        debug!("Getting terrain for region: {}", region_id);

        let terrain =
            sqlx::query_as::<_, RegionTerrain>("SELECT * FROM region_terrain WHERE region_id = $1")
                .bind(region_id)
                .fetch_optional(self.pool()?)
                .await
                .map_err(|e| anyhow!("Failed to get region terrain: {}", e))?;

        Ok(terrain)
    }

    /// Update region terrain
    pub async fn update_region_terrain(&self, region_id: Uuid, terrain_data: &[u8]) -> Result<()> {
        debug!("Updating terrain for region: {}", region_id);

        sqlx::query(
            r#"
            UPDATE region_terrain 
            SET terrain_data = $2, terrain_revision = terrain_revision + 1, updated_at = NOW()
            WHERE region_id = $1
            "#,
        )
        .bind(region_id)
        .bind(terrain_data)
        .execute(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to update region terrain: {}", e))?;

        debug!("Updated terrain for region: {}", region_id);
        Ok(())
    }

    /// Save scene object
    pub async fn save_scene_object(&self, object: &SceneObject) -> Result<()> {
        debug!(
            "Saving scene object: {} in region {}",
            object.id, object.region_id
        );

        sqlx::query(
            r#"
            INSERT INTO scene_objects (
                id, region_id, group_id, part_id, parent_id, owner_id, creator_id,
                last_owner_id, group_owner_id, object_name, description,
                position_x, position_y, position_z,
                rotation_x, rotation_y, rotation_z, rotation_w,
                velocity_x, velocity_y, velocity_z,
                angular_velocity_x, angular_velocity_y, angular_velocity_z,
                acceleration_x, acceleration_y, acceleration_z,
                scale_x, scale_y, scale_z,
                object_flags, physics_type, material, click_action,
                color_r, color_g, color_b, color_a,
                texture_entry, extra_physics_data, shape_data, script_state,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16, $17, $18,
                $19, $20, $21, $22, $23, $24,
                $25, $26, $27, $28, $29, $30,
                $31, $32, $33, $34, $35, $36, $37, $38,
                $39, $40, $41, $42, NOW(), NOW()
            )
            ON CONFLICT (id) DO UPDATE SET
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
                angular_velocity_x = EXCLUDED.angular_velocity_x,
                angular_velocity_y = EXCLUDED.angular_velocity_y,
                angular_velocity_z = EXCLUDED.angular_velocity_z,
                acceleration_x = EXCLUDED.acceleration_x,
                acceleration_y = EXCLUDED.acceleration_y,
                acceleration_z = EXCLUDED.acceleration_z,
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
                updated_at = NOW()
            "#,
        )
        .bind(object.id)
        .bind(object.region_id)
        .bind(object.group_id)
        .bind(object.part_id)
        .bind(object.parent_id)
        .bind(object.owner_id)
        .bind(object.creator_id)
        .bind(object.last_owner_id)
        .bind(object.group_owner_id)
        .bind(&object.object_name)
        .bind(&object.description)
        .bind(object.position_x)
        .bind(object.position_y)
        .bind(object.position_z)
        .bind(object.rotation_x)
        .bind(object.rotation_y)
        .bind(object.rotation_z)
        .bind(object.rotation_w)
        .bind(object.velocity_x)
        .bind(object.velocity_y)
        .bind(object.velocity_z)
        .bind(object.angular_velocity_x)
        .bind(object.angular_velocity_y)
        .bind(object.angular_velocity_z)
        .bind(object.acceleration_x)
        .bind(object.acceleration_y)
        .bind(object.acceleration_z)
        .bind(object.scale_x)
        .bind(object.scale_y)
        .bind(object.scale_z)
        .bind(object.object_flags)
        .bind(object.physics_type)
        .bind(object.material)
        .bind(object.click_action)
        .bind(object.color_r)
        .bind(object.color_g)
        .bind(object.color_b)
        .bind(object.color_a)
        .bind(&object.texture_entry)
        .bind(&object.extra_physics_data)
        .bind(&object.shape_data)
        .bind(&object.script_state)
        .execute(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to save scene object: {}", e))?;

        debug!("Saved scene object: {}", object.id);
        Ok(())
    }

    /// Load scene objects for region
    pub async fn load_scene_objects(&self, region_id: Uuid) -> Result<Vec<SceneObject>> {
        debug!("Loading scene objects for region: {}", region_id);

        let objects = sqlx::query_as::<_, SceneObject>(
            "SELECT * FROM scene_objects WHERE region_id = $1 ORDER BY created_at",
        )
        .bind(region_id)
        .fetch_all(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to load scene objects: {}", e))?;

        debug!(
            "Loaded {} scene objects for region: {}",
            objects.len(),
            region_id
        );
        Ok(objects)
    }

    /// Get total region count
    pub async fn get_region_count(&self) -> Result<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM regions")
            .fetch_one(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to get region count: {}", e))?;

        let count: i64 = row.try_get("count")?;
        Ok(count as u64)
    }

    /// Create default terrain for a region
    async fn create_default_terrain(&self, region_id: Uuid) -> Result<()> {
        debug!("Creating default terrain for region: {}", region_id);

        sqlx::query(
            r#"
            INSERT INTO region_terrain (
                region_id, terrain_revision, terrain_seed,
                water_height, terrain_raise_limit, terrain_lower_limit,
                use_estate_sun, fixed_sun, sun_position,
                created_at, updated_at
            )
            VALUES ($1, 1, 0, 20.0, 100.0, -100.0, TRUE, FALSE, 0.0, NOW(), NOW())
            "#,
        )
        .bind(region_id)
        .execute(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to create default terrain: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_region_request() {
        let request = CreateRegionRequest {
            region_name: "Test Region".to_string(),
            location_x: 1000,
            location_y: 1000,
            size_x: Some(256),
            size_y: Some(256),
            internal_ip: "127.0.0.1".to_string(),
            internal_port: 9000,
            external_host_name: "localhost".to_string(),
            master_avatar_id: None,
            owner_id: None,
        };

        assert_eq!(request.region_name, "Test Region");
        assert_eq!(request.location_x, 1000);
        assert_eq!(request.location_y, 1000);
        assert_eq!(request.size_x.unwrap(), 256);
    }
}
