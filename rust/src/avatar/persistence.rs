//! Avatar Persistence Layer for OpenSim Next
//! 
//! Handles database storage and retrieval of avatar data.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use sqlx::Row;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Helper function to parse UUID from database string
fn parse_uuid_from_db(value: &str) -> Result<Uuid, uuid::Error> {
    Uuid::parse_str(value)
}

/// Helper function to parse DateTime from database string
fn parse_datetime_from_db(value: &str) -> Result<chrono::DateTime<chrono::Utc>, chrono::ParseError> {
    chrono::DateTime::parse_from_rfc3339(value).map(|dt| dt.with_timezone(&chrono::Utc))
}

/// Avatar search criteria for database queries
#[derive(Debug, Clone)]
pub struct AvatarSearchCriteria {
    pub name_pattern: Option<String>,
    pub user_id: Option<Uuid>,
    pub online_only: bool,
    pub region_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Avatar persistence management
#[derive(Debug)]
pub struct AvatarPersistence {
    database: Arc<DatabaseManager>,
}

impl AvatarPersistence {
    /// Create new avatar persistence layer
    pub fn new(database: Arc<DatabaseManager>) -> Self {
        Self { database }
    }

    /// Store avatar in database
    pub async fn store_avatar(&self, avatar: &EnhancedAvatar) -> AvatarResult<()> {
        info!("Storing avatar in database: {}", avatar.id);

        let appearance_json = serde_json::to_string(&avatar.appearance)?;
        let behavior_json = serde_json::to_string(&avatar.behavior)?;
        let social_profile_json = serde_json::to_string(&avatar.social_profile)?;
        let persistence_data_json = serde_json::to_string(&avatar.persistence_data)?;

        sqlx::query(
            r#"
            INSERT INTO enhanced_avatars (
                id, user_id, name, appearance, behavior, social_profile, 
                persistence_data, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                appearance = EXCLUDED.appearance,
                behavior = EXCLUDED.behavior,
                social_profile = EXCLUDED.social_profile,
                persistence_data = EXCLUDED.persistence_data,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(avatar.id.to_string())
        .bind(avatar.user_id.to_string())
        .bind(&avatar.name)
        .bind(appearance_json)
        .bind(behavior_json)
        .bind(social_profile_json)
        .bind(persistence_data_json)
        .bind(avatar.created_at.to_rfc3339())
        .bind(avatar.updated_at.to_rfc3339())
        .execute(self.database.legacy_pool()?)
        .await?;

        info!("Avatar stored successfully: {}", avatar.id);
        Ok(())
    }

    /// Load avatar from database
    pub async fn load_avatar(&self, avatar_id: Uuid) -> AvatarResult<EnhancedAvatar> {
        debug!("Loading avatar from database: {}", avatar_id);

        let row = sqlx::query(
            "SELECT id, user_id, name, appearance, behavior, social_profile, persistence_data, created_at, updated_at FROM enhanced_avatars WHERE id = $1"
        )
        .bind(avatar_id.to_string())
        .fetch_optional(self.database.legacy_pool()?)
        .await?;

        if let Some(row) = row {
            let appearance: AvatarAppearance = serde_json::from_str(row.try_get::<String, _>("appearance")?.as_str())?;
            let behavior: AvatarBehavior = serde_json::from_str(row.try_get::<String, _>("behavior")?.as_str())?;
            let social_profile: AvatarSocialProfile = serde_json::from_str(row.try_get::<String, _>("social_profile")?.as_str())?;
            let persistence_data: AvatarPersistenceData = serde_json::from_str(row.try_get::<String, _>("persistence_data")?.as_str())?;

            // ELEGANT ARCHIVE SOLUTION: Parse UUID and DateTime from strings
            let id_str: String = row.try_get("id")?;
            let user_id_str: String = row.try_get("user_id")?;
            let created_at_str: String = row.try_get("created_at")?;
            let updated_at_str: String = row.try_get("updated_at")?;

            let avatar = EnhancedAvatar {
                id: parse_uuid_from_db(&id_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid UUID: {}", e) })?,
                user_id: parse_uuid_from_db(&user_id_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid UUID: {}", e) })?,
                name: row.try_get::<String, _>("name")?,
                appearance,
                behavior,
                social_profile,
                persistence_data,
                created_at: parse_datetime_from_db(&created_at_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid DateTime: {}", e) })?,
                updated_at: parse_datetime_from_db(&updated_at_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid DateTime: {}", e) })?,
            };

            debug!("Avatar loaded successfully: {}", avatar_id);
            Ok(avatar)
        } else {
            Err(AvatarError::NotFound { id: avatar_id })
        }
    }

    /// Load avatar by user ID
    pub async fn load_avatar_by_user(&self, user_id: Uuid) -> AvatarResult<EnhancedAvatar> {
        debug!("Loading avatar by user ID: {}", user_id);

        let row = sqlx::query(
            "SELECT id, user_id, name, appearance, behavior, social_profile, persistence_data, created_at, updated_at FROM enhanced_avatars WHERE user_id = $1"
        )
        .bind(user_id.to_string())
        .fetch_optional(self.database.legacy_pool()?)
        .await?;

        if let Some(row) = row {
            let appearance: AvatarAppearance = serde_json::from_str(row.try_get::<String, _>("appearance")?.as_str())?;
            let behavior: AvatarBehavior = serde_json::from_str(row.try_get::<String, _>("behavior")?.as_str())?;
            let social_profile: AvatarSocialProfile = serde_json::from_str(row.try_get::<String, _>("social_profile")?.as_str())?;
            let persistence_data: AvatarPersistenceData = serde_json::from_str(row.try_get::<String, _>("persistence_data")?.as_str())?;

            // ELEGANT ARCHIVE SOLUTION: Parse UUID and DateTime from strings
            let id_str: String = row.try_get("id")?;
            let user_id_str: String = row.try_get("user_id")?;
            let created_at_str: String = row.try_get("created_at")?;
            let updated_at_str: String = row.try_get("updated_at")?;

            let avatar = EnhancedAvatar {
                id: parse_uuid_from_db(&id_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid UUID: {}", e) })?,
                user_id: parse_uuid_from_db(&user_id_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid UUID: {}", e) })?,
                name: row.try_get::<String, _>("name")?,
                appearance,
                behavior,
                social_profile,
                persistence_data,
                created_at: parse_datetime_from_db(&created_at_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid DateTime: {}", e) })?,
                updated_at: parse_datetime_from_db(&updated_at_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid DateTime: {}", e) })?,
            };

            debug!("Avatar loaded by user ID successfully: {}", user_id);
            Ok(avatar)
        } else {
            Err(AvatarError::NotFound { id: user_id })
        }
    }

    /// Update avatar in database
    pub async fn update_avatar(&self, avatar: &EnhancedAvatar) -> AvatarResult<()> {
        debug!("Updating avatar in database: {}", avatar.id);

        let appearance_json = serde_json::to_string(&avatar.appearance)?;
        let behavior_json = serde_json::to_string(&avatar.behavior)?;
        let social_profile_json = serde_json::to_string(&avatar.social_profile)?;
        let persistence_data_json = serde_json::to_string(&avatar.persistence_data)?;

        let result = sqlx::query(
            r#"
            UPDATE enhanced_avatars SET
                name = $2,
                appearance = $3,
                behavior = $4,
                social_profile = $5,
                persistence_data = $6,
                updated_at = $7
            WHERE id = $1
            "#
        )
        .bind(avatar.id.to_string())
        .bind(&avatar.name)
        .bind(appearance_json)
        .bind(behavior_json)
        .bind(social_profile_json)
        .bind(persistence_data_json)
        .bind(avatar.updated_at.to_rfc3339())
        .execute(self.database.legacy_pool()?)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AvatarError::NotFound { id: avatar.id });
        }

        debug!("Avatar updated successfully: {}", avatar.id);
        Ok(())
    }

    /// Delete avatar from database
    pub async fn delete_avatar(&self, avatar_id: Uuid) -> AvatarResult<()> {
        info!("Deleting avatar from database: {}", avatar_id);

        let result = sqlx::query(
            "DELETE FROM enhanced_avatars WHERE id = $1"
        )
        .bind(avatar_id.to_string())
        .execute(self.database.legacy_pool()?)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AvatarError::NotFound { id: avatar_id });
        }

        info!("Avatar deleted successfully: {}", avatar_id);
        Ok(())
    }

    /// Search avatars by criteria
    pub async fn search_avatars(&self, criteria: AvatarSearchCriteria) -> AvatarResult<Vec<EnhancedAvatar>> {
        debug!("Searching avatars with criteria: {:?}", criteria);

        // ELEGANT ARCHIVE SOLUTION: Simplified query to avoid complex dynamic parameter binding
        // TODO: Implement advanced search criteria in future phases
        let limit = criteria.limit.unwrap_or(100);
        let offset = criteria.offset.unwrap_or(0);
        
        let rows = if let Some(name_pattern) = &criteria.name_pattern {
            // Search by name pattern
            sqlx::query("SELECT * FROM enhanced_avatars WHERE name ILIKE $1 ORDER BY updated_at DESC LIMIT $2 OFFSET $3")
                .bind(format!("%{}%", name_pattern))
                .bind(limit)
                .bind(offset)
                .fetch_all(self.database.legacy_pool()?)
                .await?
        } else if let Some(user_id) = criteria.user_id {
            // Search by user ID
            sqlx::query("SELECT * FROM enhanced_avatars WHERE user_id = $1 ORDER BY updated_at DESC LIMIT $2 OFFSET $3")
                .bind(user_id.to_string())
                .bind(limit)
                .bind(offset)
                .fetch_all(self.database.legacy_pool()?)
                .await?
        } else {
            // Default search - all avatars
            sqlx::query("SELECT * FROM enhanced_avatars ORDER BY updated_at DESC LIMIT $1 OFFSET $2")
                .bind(limit)
                .bind(offset)
                .fetch_all(self.database.legacy_pool()?)
                .await?
        };

        let mut avatars = Vec::new();

        for row in rows {
            let appearance: AvatarAppearance = serde_json::from_str(
                row.try_get::<String, _>("appearance")?.as_str()
            )?;
            let behavior: AvatarBehavior = serde_json::from_str(
                row.try_get::<String, _>("behavior")?.as_str()
            )?;
            let social_profile: AvatarSocialProfile = serde_json::from_str(
                row.try_get::<String, _>("social_profile")?.as_str()
            )?;
            let persistence_data: AvatarPersistenceData = serde_json::from_str(
                row.try_get::<String, _>("persistence_data")?.as_str()
            )?;

            // ELEGANT ARCHIVE SOLUTION: Parse UUID and DateTime from strings
            let id_str: String = row.try_get("id")?;
            let user_id_str: String = row.try_get("user_id")?;
            let created_at_str: String = row.try_get("created_at")?;
            let updated_at_str: String = row.try_get("updated_at")?;

            let avatar = EnhancedAvatar {
                id: parse_uuid_from_db(&id_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid UUID: {}", e) })?,
                user_id: parse_uuid_from_db(&user_id_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid UUID: {}", e) })?,
                name: row.try_get("name")?,
                appearance,
                behavior,
                social_profile,
                persistence_data,
                created_at: parse_datetime_from_db(&created_at_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid DateTime: {}", e) })?,
                updated_at: parse_datetime_from_db(&updated_at_str).map_err(|e| AvatarError::SystemError { message: format!("Invalid DateTime: {}", e) })?,
            };

            avatars.push(avatar);
        }

        debug!("Avatar search completed: {} results", avatars.len());
        Ok(avatars)
    }

    /// Get regions visited count for avatar
    pub async fn get_regions_visited_count(&self, avatar_id: Uuid) -> AvatarResult<i64> {
        let result = sqlx::query(
            "SELECT COUNT(DISTINCT region_id) as count FROM avatar_region_visits WHERE avatar_id = $1"
        )
        .bind(avatar_id.to_string())
        .fetch_optional(self.database.legacy_pool()?)
        .await?;

        let count = result
            .map(|row| row.try_get::<i64, _>("count").unwrap_or(0))
            .unwrap_or(0);

        Ok(count)
    }

    /// Get total avatar count
    pub async fn get_total_avatar_count(&self) -> AvatarResult<i64> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM enhanced_avatars")
            .fetch_one(self.database.legacy_pool()?)
            .await?;

        let count = result.try_get::<i64, _>("count").unwrap_or(0);
        Ok(count)
    }

    /// Initialize database tables
    pub async fn initialize_tables(&self) -> Result<()> {
        info!("Initializing avatar database tables");

        // Create enhanced_avatars table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS enhanced_avatars (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                appearance JSONB NOT NULL,
                behavior JSONB NOT NULL,
                social_profile JSONB NOT NULL,
                persistence_data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        // Create index on user_id
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_enhanced_avatars_user_id ON enhanced_avatars (user_id)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        // Create index on name for searching
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_enhanced_avatars_name ON enhanced_avatars (name)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        // Create avatar_region_visits table for tracking regions visited
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS avatar_region_visits (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                avatar_id UUID NOT NULL REFERENCES enhanced_avatars(id) ON DELETE CASCADE,
                region_id UUID NOT NULL,
                first_visit TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                last_visit TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                visit_count INTEGER NOT NULL DEFAULT 1,
                total_time_seconds BIGINT NOT NULL DEFAULT 0,
                UNIQUE(avatar_id, region_id)
            )
            "#
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        // Create indexes for avatar_region_visits
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_avatar_region_visits_avatar_id ON avatar_region_visits (avatar_id)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_avatar_region_visits_region_id ON avatar_region_visits (region_id)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        info!("Avatar database tables initialized successfully");
        Ok(())
    }

    /// Record avatar visit to region
    pub async fn record_region_visit(
        &self,
        avatar_id: Uuid,
        region_id: Uuid,
        time_spent_seconds: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO avatar_region_visits (avatar_id, region_id, visit_count, total_time_seconds)
            VALUES ($1, $2, 1, $3)
            ON CONFLICT (avatar_id, region_id) DO UPDATE SET
                last_visit = NOW(),
                visit_count = avatar_region_visits.visit_count + 1,
                total_time_seconds = avatar_region_visits.total_time_seconds + $3
            "#
        )
        .bind(avatar_id.to_string())
        .bind(region_id.to_string())
        .bind(time_spent_seconds)
        .execute(self.database.legacy_pool()?)
        .await?;

        Ok(())
    }

    /// Get avatar visit statistics for a region
    pub async fn get_region_visit_stats(&self, region_id: Uuid) -> Result<Vec<RegionVisitStats>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                arv.avatar_id,
                ea.name as avatar_name,
                arv.first_visit,
                arv.last_visit,
                arv.visit_count,
                arv.total_time_seconds
            FROM avatar_region_visits arv
            JOIN enhanced_avatars ea ON arv.avatar_id = ea.id
            WHERE arv.region_id = $1
            ORDER BY arv.total_time_seconds DESC
            "#
        )
        .bind(region_id.to_string())
        .fetch_all(self.database.legacy_pool()?)
        .await?;

        let stats = rows.into_iter().map(|row| {
            let avatar_id_str: String = row.try_get("avatar_id").unwrap();
            let first_visit_str: String = row.try_get("first_visit").unwrap();
            let last_visit_str: String = row.try_get("last_visit").unwrap();
            
            RegionVisitStats {
                avatar_id: Uuid::parse_str(&avatar_id_str).unwrap_or_default(),
                avatar_name: row.try_get("avatar_name").unwrap(),
                first_visit: chrono::DateTime::parse_from_rfc3339(&first_visit_str).unwrap().with_timezone(&chrono::Utc),
                last_visit: chrono::DateTime::parse_from_rfc3339(&last_visit_str).unwrap().with_timezone(&chrono::Utc),
                visit_count: row.try_get("visit_count").unwrap(),
                total_time_seconds: row.try_get("total_time_seconds").unwrap(),
            }
        }).collect();

        Ok(stats)
    }

    /// Get avatar statistics (placeholder implementation)
    pub async fn get_avatar_statistics(&self, _avatar_id: Uuid) -> AvatarResult<()> {
        // Placeholder implementation - would return comprehensive avatar statistics
        Ok(())
    }

    /// Store avatar visit (simplified version for compatibility)
    pub async fn store_avatar_visit(&self, avatar_id: Uuid, region_id: String) -> AvatarResult<()> {
        // Convert string region_id to UUID for database storage
        let region_uuid = Uuid::parse_str(&region_id)
            .unwrap_or_else(|_| Uuid::new_v4()); // Fallback to new UUID if parsing fails
        
        self.record_region_visit(avatar_id, region_uuid, 0).await
            .map_err(|e| AvatarError::SystemError { message: e.to_string() })?;
        
        Ok(())
    }

    /// Get avatar visits (simplified version for compatibility)
    pub async fn get_avatar_visits(&self, avatar_id: Uuid) -> AvatarResult<Vec<String>> {
        let rows = sqlx::query(
            "SELECT DISTINCT region_id FROM avatar_region_visits WHERE avatar_id = $1"
        )
        .bind(avatar_id.to_string())
        .fetch_all(self.database.legacy_pool()?)
        .await
        .map_err(|e| AvatarError::SystemError { message: e.to_string() })?;

        let region_ids = rows.into_iter()
            .map(|row| row.try_get::<String, _>("region_id")
                .unwrap_or_else(|_| "unknown".to_string()))
            .collect();

        Ok(region_ids)
    }
}

/// Region visit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionVisitStats {
    pub avatar_id: Uuid,
    pub avatar_name: String,
    pub first_visit: chrono::DateTime<chrono::Utc>,
    pub last_visit: chrono::DateTime<chrono::Utc>,
    pub visit_count: i32,
    pub total_time_seconds: i64,
}