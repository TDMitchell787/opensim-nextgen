//! Local Asset Service Implementation
//!
//! Provides direct database access for asset storage and retrieval.
//! Used in standalone mode with PostgreSQL backend.
//!
//! Reference: OpenSim/Services/AssetService/AssetService.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::database::multi_backend::DatabaseConnection;
use crate::services::traits::{AssetBase, AssetMetadata, AssetServiceTrait};

pub struct LocalAssetService {
    connection: Arc<DatabaseConnection>,
}

impl LocalAssetService {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        info!("Initializing local asset service");
        Self { connection }
    }

    fn get_pg_pool(&self) -> Result<&sqlx::PgPool> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => Ok(pool),
            _ => Err(anyhow!("LocalAssetService requires PostgreSQL connection")),
        }
    }
}

#[async_trait]
impl AssetServiceTrait for LocalAssetService {
    async fn get(&self, id: &str) -> Result<Option<AssetBase>> {
        debug!("Getting asset: {}", id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            r#"
            SELECT id, name, description, asset_type, local, temporary, data,
                   creator_id, asset_flags, created_at
            FROM assets
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get asset: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_asset_base(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_metadata(&self, id: &str) -> Result<Option<AssetMetadata>> {
        debug!("Getting asset metadata: {}", id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            r#"
            SELECT id, name, description, asset_type, local, temporary,
                   creator_id, asset_flags, created_at
            FROM assets
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get asset metadata: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_asset_metadata(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_data(&self, id: &str) -> Result<Option<Vec<u8>>> {
        debug!("Getting asset data: {}", id);

        let pool = self.get_pg_pool()?;
        let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT data FROM assets WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to get asset data: {}", e))?;

        Ok(row.map(|(data,)| data))
    }

    async fn store(&self, asset: &AssetBase) -> Result<String> {
        info!("Storing asset: {} ({})", asset.name, asset.id);

        let pool = self.get_pg_pool()?;
        sqlx::query(
            r#"
            INSERT INTO assets (
                id, name, description, asset_type, local, temporary,
                data, creator_id, asset_flags, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            ON CONFLICT(id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                data = EXCLUDED.data,
                asset_flags = EXCLUDED.asset_flags
            "#,
        )
        .bind(&asset.id)
        .bind(&asset.name)
        .bind(&asset.description)
        .bind(asset.asset_type as i32)
        .bind(asset.local)
        .bind(asset.temporary)
        .bind(&asset.data)
        .bind(&asset.creator_id)
        .bind(asset.flags)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to store asset: {}", e))?;

        info!("Stored asset: {}", asset.id);
        Ok(asset.id.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        warn!("Deleting asset: {}", id);

        let pool = self.get_pg_pool()?;
        let result = sqlx::query("DELETE FROM assets WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to delete asset: {}", e))?;
        Ok(result.rows_affected() > 0)
    }

    async fn asset_exists(&self, id: &str) -> Result<bool> {
        debug!("Checking if asset exists: {}", id);

        let pool = self.get_pg_pool()?;
        let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM assets WHERE id = $1 LIMIT 1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to check asset existence: {}", e))?;
        Ok(result.is_some())
    }
}

impl LocalAssetService {
    fn row_to_asset_base(&self, row: &sqlx::postgres::PgRow) -> Result<AssetBase> {
        use sqlx::Row;

        Ok(AssetBase {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description").unwrap_or_default(),
            asset_type: row.try_get::<i32, _>("asset_type")? as i8,
            local: row.try_get("local").unwrap_or(false),
            temporary: row.try_get("temporary").unwrap_or(false),
            data: row.try_get("data").unwrap_or_default(),
            creator_id: row.try_get("creator_id").unwrap_or_default(),
            flags: row.try_get("asset_flags").unwrap_or(0),
        })
    }

    fn row_to_asset_metadata(&self, row: &sqlx::postgres::PgRow) -> Result<AssetMetadata> {
        use chrono::{DateTime, Utc};
        use sqlx::Row;

        let created_at: DateTime<Utc> = row.try_get("created_at").unwrap_or_else(|_| Utc::now());

        Ok(AssetMetadata {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description").unwrap_or_default(),
            asset_type: row.try_get::<i32, _>("asset_type")? as i8,
            local: row.try_get("local").unwrap_or(false),
            temporary: row.try_get("temporary").unwrap_or(false),
            creator_id: row.try_get("creator_id").unwrap_or_default(),
            flags: row.try_get("asset_flags").unwrap_or(0),
            created_date: created_at.timestamp(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_base_default() {
        let asset = AssetBase {
            id: "test-id".to_string(),
            name: "Test Asset".to_string(),
            description: "Test Description".to_string(),
            asset_type: 0,
            local: false,
            temporary: false,
            data: vec![1, 2, 3],
            creator_id: "creator".to_string(),
            flags: 0,
        };

        assert_eq!(asset.id, "test-id");
        assert_eq!(asset.data.len(), 3);
    }
}
