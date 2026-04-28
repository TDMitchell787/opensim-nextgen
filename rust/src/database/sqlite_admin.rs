use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::admin_operations::{
    AdminOperationResult, CreateRegionRequest, CreateUserRequest, DatabaseBackupRequest,
    DatabaseMaintenanceRequest, DatabaseMigrationRequest, DatabaseRestoreRequest, DatabaseStats,
    UpdateRegionRequest,
};

/// SQLite-specific admin operations for OpenSim compatibility
#[derive(Debug)]
pub struct SqliteAdmin {
    pool: Pool<Sqlite>,
}

impl SqliteAdmin {
    /// Create new SQLite admin interface
    pub async fn new(database_path: &str) -> Result<Self> {
        info!("Creating SQLite admin interface for: {}", database_path);

        // Handle different SQLite URL formats
        let connection_string = if database_path.starts_with("sqlite://") {
            database_path.replace("sqlite://", "sqlite:")
        } else if database_path.starts_with("sqlite:") {
            database_path.to_string()
        } else {
            format!("sqlite:{}", database_path)
        };

        info!("SQLite connection string: {}", connection_string);

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&connection_string)
            .await?;

        info!("SQLite admin interface connected successfully");
        Ok(Self { pool })
    }

    /// Create a new user account using OpenSim master compatible schema
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<AdminOperationResult> {
        info!(
            "Creating user in SQLite: {} {}",
            request.firstname, request.lastname
        );

        let user_id = Uuid::new_v4();
        let created_timestamp = chrono::Utc::now().timestamp() as i32;
        let user_level = request.user_level.unwrap_or(0);

        let result = match self.pool.acquire().await {
            Ok(mut conn) => {
                // Use OpenSim compatible UserAccounts table
                let query = r#"
                    INSERT INTO UserAccounts 
                    (PrincipalID, FirstName, LastName, Email, ServiceURLs, Created, UserLevel, UserFlags, UserTitle, active)
                    VALUES (?, ?, ?, ?, '', ?, ?, 0, '', 1)
                "#;

                match sqlx::query(query)
                    .bind(user_id.to_string())
                    .bind(&request.firstname)
                    .bind(&request.lastname)
                    .bind(&request.email)
                    .bind(created_timestamp)
                    .bind(user_level)
                    .execute(&mut *conn)
                    .await
                {
                    Ok(result) => {
                        info!(
                            "SQLite user created successfully: {} {} ({})",
                            request.firstname, request.lastname, user_id
                        );

                        // Also insert auth record
                        let auth_query = r#"
                            INSERT INTO auth 
                            (UUID, passwordHash, passwordSalt, webLoginKey, accountType)
                            VALUES (?, ?, ?, '', 'UserAccount')
                        "#;

                        let password_hash = format!("$1${}", &request.password); // Simple hash for demo
                        let salt = "opensim_salt"; // Simple salt for demo

                        let _ = sqlx::query(auth_query)
                            .bind(user_id.to_string())
                            .bind(&password_hash)
                            .bind(salt)
                            .execute(&mut *conn)
                            .await;

                        AdminOperationResult {
                            success: true,
                            message: format!(
                                "User '{}' '{}' created successfully with ID: {}",
                                request.firstname, request.lastname, user_id
                            ),
                            affected_rows: Some(result.rows_affected() as i64),
                            data: Some(serde_json::json!({
                                "user_id": user_id,
                                "firstname": request.firstname,
                                "lastname": request.lastname,
                                "email": request.email,
                                "user_level": user_level,
                                "created": created_timestamp
                            })),
                        }
                    }
                    Err(e) => {
                        error!("Failed to create SQLite user: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to create user: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("SQLite connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("SQLite connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// List all users
    pub async fn list_users(&self, limit: Option<i32>) -> Result<AdminOperationResult> {
        debug!("Listing SQLite users with limit: {:?}", limit);

        let result = match self.pool.acquire().await {
            Ok(mut conn) => {
                let query = match limit {
                    Some(l) => format!(
                        r#"
                        SELECT PrincipalID, FirstName, LastName, Email, UserLevel, Created, active
                        FROM UserAccounts 
                        ORDER BY Created DESC 
                        LIMIT {}
                    "#,
                        l
                    ),
                    None => r#"
                        SELECT PrincipalID, FirstName, LastName, Email, UserLevel, Created, active
                        FROM UserAccounts 
                        ORDER BY Created DESC
                    "#
                    .to_string(),
                };

                match sqlx::query(&query).fetch_all(&mut *conn).await {
                    Ok(rows) => {
                        let mut users = Vec::new();
                        for row in rows {
                            let user_id: String = row.try_get("PrincipalID").unwrap_or_default();
                            let firstname: String = row.try_get("FirstName").unwrap_or_default();
                            let lastname: String = row.try_get("LastName").unwrap_or_default();
                            let email: String = row.try_get("Email").unwrap_or_default();
                            let user_level: i32 = row.try_get("UserLevel").unwrap_or(0);
                            let created: i32 = row.try_get("Created").unwrap_or(0);
                            let active: i32 = row.try_get("active").unwrap_or(0);

                            users.push(serde_json::json!({
                                "user_id": user_id,
                                "firstname": firstname,
                                "lastname": lastname,
                                "email": email,
                                "user_level": user_level,
                                "created": created,
                                "active": active > 0,
                                "is_god": user_level >= 200
                            }));
                        }

                        AdminOperationResult {
                            success: true,
                            message: format!("Retrieved {} users from SQLite", users.len()),
                            affected_rows: Some(users.len() as i64),
                            data: Some(serde_json::json!({
                                "users": users,
                                "total_count": users.len(),
                                "limit_applied": limit
                            })),
                        }
                    }
                    Err(e) => {
                        error!("Failed to list SQLite users: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to list users: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("SQLite connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("SQLite connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Get database statistics for SQLite
    pub async fn get_database_stats(&self) -> Result<AdminOperationResult> {
        debug!("Retrieving SQLite database statistics");

        let result = match self.pool.acquire().await {
            Ok(mut conn) => {
                // Get user statistics
                let total_users = sqlx::query("SELECT COUNT(*) as count FROM UserAccounts")
                    .fetch_one(&mut *conn)
                    .await
                    .map(|row| row.try_get::<i64, _>("count").unwrap_or(0))
                    .unwrap_or(0);

                let active_users =
                    sqlx::query("SELECT COUNT(*) as count FROM UserAccounts WHERE active = 1")
                        .fetch_one(&mut *conn)
                        .await
                        .map(|row| row.try_get::<i64, _>("count").unwrap_or(0))
                        .unwrap_or(0);

                let total_regions = sqlx::query("SELECT COUNT(*) as count FROM regions")
                    .fetch_one(&mut *conn)
                    .await
                    .map(|row| row.try_get::<i64, _>("count").unwrap_or(0))
                    .unwrap_or(0);

                AdminOperationResult {
                    success: true,
                    message: "SQLite database statistics retrieved".to_string(),
                    affected_rows: None,
                    data: Some(serde_json::json!({
                        "total_users": total_users,
                        "active_users": active_users,
                        "total_regions": total_regions,
                        "online_regions": 0,
                        "database_type": "SQLite",
                        "database_size_mb": 0.0,
                        "last_backup": null
                    })),
                }
            }
            Err(e) => {
                error!("SQLite connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("SQLite connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Health check for SQLite
    pub async fn get_database_health(&self) -> Result<AdminOperationResult> {
        debug!("Checking SQLite database health");

        let result = match self.pool.acquire().await {
            Ok(mut conn) => {
                let start_time = std::time::Instant::now();

                // Test basic connectivity
                let connectivity_test = sqlx::query("SELECT 1 as test")
                    .fetch_one(&mut *conn)
                    .await
                    .is_ok();

                let connection_time_ms = start_time.elapsed().as_millis();

                let health_status = if connectivity_test && connection_time_ms < 1000 {
                    "healthy"
                } else if connectivity_test {
                    "slow"
                } else {
                    "unhealthy"
                };

                AdminOperationResult {
                    success: true,
                    message: format!("SQLite database health status: {}", health_status),
                    affected_rows: None,
                    data: Some(serde_json::json!({
                        "health_status": health_status,
                        "connectivity_test": connectivity_test,
                        "connection_time_ms": connection_time_ms,
                        "database_type": "SQLite",
                        "last_checked": chrono::Utc::now().to_rfc3339()
                    })),
                }
            }
            Err(e) => {
                error!("SQLite health check failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("SQLite health check failed: {}", e),
                    affected_rows: None,
                    data: Some(serde_json::json!({
                        "health_status": "unhealthy",
                        "error": e.to_string(),
                        "database_type": "SQLite",
                        "last_checked": chrono::Utc::now().to_rfc3339()
                    })),
                }
            }
        };

        Ok(result)
    }
}
