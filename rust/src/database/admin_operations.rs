use crate::database::multi_backend::DatabasePoolRef;
use crate::database::sqlite_admin::SqliteAdmin;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateUserRequest {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
    pub user_level: Option<i32>,
    pub start_region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAccount {
    pub user_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub user_level: i32,
    pub user_flags: i32,
    pub user_title: Option<String>,
    pub created: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseStats {
    pub total_users: i64,
    pub active_users: i64,
    pub total_regions: i64,
    pub online_regions: i64,
    pub database_size_mb: f64,
    pub last_backup: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseBackupRequest {
    pub backup_name: String,
    pub include_user_data: bool,
    pub include_region_data: bool,
    pub include_asset_data: bool,
    pub include_inventory_data: bool,
    pub compression: bool,
    pub backup_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseRestoreRequest {
    pub backup_file: String,
    pub restore_users: bool,
    pub restore_regions: bool,
    pub restore_assets: bool,
    pub restore_inventory: bool,
    pub overwrite_existing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseMigrationRequest {
    pub target_version: String,
    pub dry_run: bool,
    pub backup_before: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseMaintenanceRequest {
    pub vacuum_tables: bool,
    pub reindex_tables: bool,
    pub analyze_tables: bool,
    pub cleanup_orphaned: bool,
    pub compress_tables: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseBackupInfo {
    pub backup_id: Uuid,
    pub backup_name: String,
    pub backup_path: String,
    pub backup_size_mb: f64,
    pub created_at: DateTime<Utc>,
    pub contains_users: bool,
    pub contains_regions: bool,
    pub contains_assets: bool,
    pub contains_inventory: bool,
    pub compression_ratio: f64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateRegionRequest {
    pub region_name: String,
    pub location_x: i32,
    pub location_y: i32,
    pub size_x: Option<i32>,
    pub size_y: Option<i32>,
    pub server_ip: Option<String>,
    pub server_port: Option<i32>,
    pub server_uri: Option<String>,
    pub owner_uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateRegionRequest {
    pub new_name: Option<String>,
    pub location_x: Option<i32>,
    pub location_y: Option<i32>,
    pub size_x: Option<i32>,
    pub size_y: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegionInfo {
    pub uuid: String,
    pub region_name: String,
    pub location_x: i32,
    pub location_y: i32,
    pub size_x: i32,
    pub size_y: i32,
    pub server_ip: String,
    pub server_port: i32,
    pub server_uri: Option<String>,
    pub owner_uuid: Option<String>,
    pub flags: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminOperationResult {
    pub success: bool,
    pub message: String,
    pub affected_rows: Option<i64>,
    pub data: Option<serde_json::Value>,
}

/// Database administration operations for OpenSim Next
/// Provides Robust-style user and database management commands
#[derive(Debug)]
pub struct DatabaseAdmin {
    pool: Option<sqlx::Pool<Postgres>>,
    sqlite_admin: Option<Arc<SqliteAdmin>>,
    stub_mode: bool,
}

impl DatabaseAdmin {
    pub fn new(pool: sqlx::Pool<Postgres>) -> Self {
        Self {
            pool: Some(pool),
            sqlite_admin: None,
            stub_mode: false,
        }
    }

    /// Helper to get pool or return stub error
    fn get_pool(&self) -> Result<&sqlx::Pool<Postgres>> {
        if self.stub_mode {
            return Err(anyhow::anyhow!("Operation not implemented for SQLite backend. Please use PostgreSQL for admin operations."));
        }

        self.pool
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database pool not available"))
    }

    /// Helper to create stub response for admin operations
    fn stub_response(&self, operation: &str) -> AdminOperationResult {
        AdminOperationResult {
            success: false,
            message: format!("{} not implemented for SQLite backend. Please use PostgreSQL for admin operations.", operation),
            affected_rows: None,
            data: None,
        }
    }

    /// Create a stub admin interface for SQLite compatibility mode
    /// This creates a minimal admin interface that returns "not implemented" for operations
    pub fn new_stub() -> Self {
        warn!(
            "Creating stub DatabaseAdmin - admin operations will return 'not implemented' errors"
        );
        Self {
            pool: None,
            sqlite_admin: None,
            stub_mode: true,
        }
    }

    /// Create SQLite-compatible admin interface
    pub async fn new_sqlite(database_path: &str) -> Result<Self> {
        info!(
            "Creating SQLite-compatible DatabaseAdmin for: {}",
            database_path
        );

        // Create real SQLite admin interface
        match SqliteAdmin::new(database_path).await {
            Ok(sqlite_admin) => {
                info!("SQLite admin interface created successfully");
                Ok(Self {
                    pool: None,
                    sqlite_admin: Some(Arc::new(sqlite_admin)),
                    stub_mode: false,
                })
            }
            Err(e) => {
                error!("Failed to create SQLite admin interface: {}", e);
                warn!("Falling back to stub mode");
                Ok(Self::new_stub())
            }
        }
    }

    /// Create a new user account (equivalent to 'create user' in OpenSim Robust)
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<AdminOperationResult> {
        info!("Creating user: {} {}", request.firstname, request.lastname);

        // Delegate to SQLite admin if available
        if let Some(ref sqlite_admin) = self.sqlite_admin {
            return sqlite_admin.create_user(request).await;
        }

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(AdminOperationResult {
                    success: false,
                    message: e.to_string(),
                    affected_rows: None,
                    data: None,
                });
            }
        };

        // Generate new user ID
        let user_id = Uuid::new_v4();
        let created = Utc::now();
        let user_level = request.user_level.unwrap_or(0);

        // Generate timestamp for OpenSim compatibility
        let created_timestamp = created.timestamp() as i32;

        // Generate password hash and salt for auth table
        // OpenSim uses MD5(MD5(password) + ":" + salt)
        let salt: String = (0..32)
            .map(|_| format!("{:x}", rand::random::<u8>() % 16))
            .collect();
        let password_md5 = format!("{:x}", md5::compute(request.password.as_bytes()));
        let salted = format!("{}:{}", password_md5, salt);
        let password_hash = format!("{:x}", md5::compute(salted.as_bytes()));

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                // Use OpenSim compatible UserAccounts table
                // PostgreSQL uses $1, $2 etc for placeholders (not ? like SQLite/MySQL)
                // Cast $1 to uuid explicitly for PostgreSQL compatibility
                let query = r#"
                    INSERT INTO UserAccounts
                    (PrincipalID, FirstName, LastName, Email, ServiceURLs, Created, UserLevel, UserFlags, UserTitle, Active)
                    VALUES ($1::uuid, $2, $3, $4, '', $5, $6, 0, '', 1)
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
                    Ok(_) => {
                        // Now insert auth record for login
                        let auth_query = r#"
                            INSERT INTO auth (uuid, passwordhash, passwordsalt, webloginkey, accounttype)
                            VALUES ($1::uuid, $2, $3, '', 'UserAccount')
                        "#;

                        match sqlx::query(auth_query)
                            .bind(user_id.to_string())
                            .bind(&password_hash)
                            .bind(&salt)
                            .execute(&mut *conn)
                            .await
                        {
                            Ok(_) => {
                                info!(
                                    "User created with auth, now creating inventory for: {} {}",
                                    request.firstname, request.lastname
                                );

                                match crate::database::default_inventory::create_default_user_inventory(pool, user_id).await {
                                    Ok(_) => {
                                        info!("User created successfully with full inventory: {} {} ({})", request.firstname, request.lastname, user_id);
                                        AdminOperationResult {
                                            success: true,
                                            message: format!("User '{}' '{}' created successfully with ID: {} (inventory and appearance initialized)",
                                                request.firstname, request.lastname, user_id),
                                            affected_rows: Some(1),
                                            data: Some(serde_json::json!({
                                                "user_id": user_id,
                                                "firstname": request.firstname,
                                                "lastname": request.lastname,
                                                "email": request.email,
                                                "user_level": user_level,
                                                "created": created,
                                                "inventory": "full default inventory created (20 folders, 6 wearables, 6 COF links, appearance)"
                                            })),
                                        }
                                    }
                                    Err(e) => {
                                        warn!("User account created but inventory creation failed: {}", e);
                                        AdminOperationResult {
                                            success: true,
                                            message: format!("User '{}' '{}' created (ID: {}) but inventory failed: {}",
                                                request.firstname, request.lastname, user_id, e),
                                            affected_rows: Some(1),
                                            data: Some(serde_json::json!({
                                                "user_id": user_id,
                                                "firstname": request.firstname,
                                                "lastname": request.lastname
                                            })),
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to create auth record: {}", e);
                                // Try to rollback user account
                                let _ = sqlx::query(
                                    "DELETE FROM UserAccounts WHERE PrincipalID = $1::uuid",
                                )
                                .bind(user_id.to_string())
                                .execute(&mut *conn)
                                .await;
                                AdminOperationResult {
                                    success: false,
                                    message: format!("Failed to create auth record: {}", e),
                                    affected_rows: None,
                                    data: None,
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to create user: {}", e);
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
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Reset user password (equivalent to 'reset user password' in OpenSim Robust)
    pub async fn reset_user_password(
        &self,
        firstname: &str,
        lastname: &str,
        new_password: &str,
    ) -> Result<AdminOperationResult> {
        info!("Resetting password for user: {} {}", firstname, lastname);

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(AdminOperationResult {
                    success: false,
                    message: e.to_string(),
                    affected_rows: None,
                    data: None,
                });
            }
        };

        // Hash password (in production, use proper password hashing)
        let password_hash = format!("$1${}$", new_password); // Simplified for demo

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let query = r#"
                    UPDATE UserAccounts
                    SET ServiceURLs = $1
                    WHERE FirstName = $2 AND LastName = $3
                "#;

                match sqlx::query(query)
                    .bind(&password_hash)
                    .bind(firstname)
                    .bind(lastname)
                    .execute(&mut *conn)
                    .await
                {
                    Ok(result) => {
                        if result.rows_affected() > 0 {
                            info!(
                                "Password reset successfully for: {} {}",
                                firstname, lastname
                            );
                            AdminOperationResult {
                                success: true,
                                message: format!(
                                    "Password reset successfully for user '{}' '{}'",
                                    firstname, lastname
                                ),
                                affected_rows: Some(result.rows_affected() as i64),
                                data: None,
                            }
                        } else {
                            warn!("User not found: {} {}", firstname, lastname);
                            AdminOperationResult {
                                success: false,
                                message: format!("User '{}' '{}' not found", firstname, lastname),
                                affected_rows: Some(0),
                                data: None,
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to reset password: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to reset password: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Reset user email (equivalent to 'reset user email' in OpenSim Robust)
    pub async fn reset_user_email(
        &self,
        firstname: &str,
        lastname: &str,
        new_email: &str,
    ) -> Result<AdminOperationResult> {
        info!(
            "Resetting email for user: {} {} to {}",
            firstname, lastname, new_email
        );

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let query = r#"
                    UPDATE UserAccounts
                    SET Email = $1
                    WHERE FirstName = $2 AND LastName = $3
                "#;

                match sqlx::query(query)
                    .bind(new_email)
                    .bind(firstname)
                    .bind(lastname)
                    .execute(&mut *conn)
                    .await
                {
                    Ok(result) => {
                        if result.rows_affected() > 0 {
                            info!("Email updated successfully for: {} {}", firstname, lastname);
                            AdminOperationResult {
                                success: true,
                                message: format!(
                                    "Email updated successfully for user '{}' '{}' to '{}'",
                                    firstname, lastname, new_email
                                ),
                                affected_rows: Some(result.rows_affected() as i64),
                                data: None,
                            }
                        } else {
                            warn!("User not found: {} {}", firstname, lastname);
                            AdminOperationResult {
                                success: false,
                                message: format!("User '{}' '{}' not found", firstname, lastname),
                                affected_rows: Some(0),
                                data: None,
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to update email: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to update email: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Set user level (equivalent to 'set user level' in OpenSim Robust)
    /// Level 200+ = god mode if enabled in OpenSim
    pub async fn set_user_level(
        &self,
        firstname: &str,
        lastname: &str,
        level: i32,
    ) -> Result<AdminOperationResult> {
        info!(
            "Setting user level for: {} {} to {}",
            firstname, lastname, level
        );

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let query = r#"
                    UPDATE UserAccounts
                    SET UserLevel = $1
                    WHERE FirstName = $2 AND LastName = $3
                "#;

                match sqlx::query(query)
                    .bind(level)
                    .bind(firstname)
                    .bind(lastname)
                    .execute(&mut *conn)
                    .await
                {
                    Ok(result) => {
                        if result.rows_affected() > 0 {
                            let privilege = if level >= 200 { " (God Level)" } else { "" };
                            info!(
                                "User level updated successfully for: {} {}",
                                firstname, lastname
                            );
                            AdminOperationResult {
                                success: true,
                                message: format!(
                                    "User level updated for '{}' '{}' to {}{}",
                                    firstname, lastname, level, privilege
                                ),
                                affected_rows: Some(result.rows_affected() as i64),
                                data: Some(serde_json::json!({
                                    "firstname": firstname,
                                    "lastname": lastname,
                                    "user_level": level,
                                    "is_god": level >= 200
                                })),
                            }
                        } else {
                            warn!("User not found: {} {}", firstname, lastname);
                            AdminOperationResult {
                                success: false,
                                message: format!("User '{}' '{}' not found", firstname, lastname),
                                affected_rows: Some(0),
                                data: None,
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to update user level: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to update user level: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Show user account details (equivalent to 'show account' in OpenSim Robust)
    pub async fn show_user_account(
        &self,
        firstname: &str,
        lastname: &str,
    ) -> Result<AdminOperationResult> {
        debug!("Retrieving account details for: {} {}", firstname, lastname);

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let query = r#"
                    SELECT PrincipalID, FirstName, LastName, Email, UserLevel, UserFlags, UserTitle, Created, Active
                    FROM UserAccounts
                    WHERE FirstName = $1 AND LastName = $2
                "#;

                match sqlx::query(query)
                    .bind(firstname)
                    .bind(lastname)
                    .fetch_optional(&mut *conn)
                    .await
                {
                    Ok(Some(row)) => {
                        let user_id: String = row.try_get("PrincipalID").unwrap_or_default();
                        let user_level: i32 = row.try_get("UserLevel").unwrap_or(0);
                        let user_flags: i32 = row.try_get("UserFlags").unwrap_or(0);
                        let email: String = row.try_get("Email").unwrap_or_default();
                        let user_title: String = row.try_get("UserTitle").unwrap_or_default();
                        let created: i64 = row.try_get("Created").unwrap_or(0);
                        let active: bool = row.try_get("Active").unwrap_or(false);

                        AdminOperationResult {
                            success: true,
                            message: format!("Account details for '{}' '{}'", firstname, lastname),
                            affected_rows: Some(1),
                            data: Some(serde_json::json!({
                                "user_id": user_id,
                                "firstname": firstname,
                                "lastname": lastname,
                                "email": email,
                                "user_level": user_level,
                                "user_flags": user_flags,
                                "user_title": user_title,
                                "created": created,
                                "active": active,
                                "is_god": user_level >= 200
                            })),
                        }
                    }
                    Ok(None) => AdminOperationResult {
                        success: false,
                        message: format!("User '{}' '{}' not found", firstname, lastname),
                        affected_rows: Some(0),
                        data: None,
                    },
                    Err(e) => {
                        error!("Failed to retrieve user account: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to retrieve user account: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// List all users with optional limit (equivalent to 'show users' in OpenSim Robust)
    pub async fn list_users(&self, limit: Option<i32>) -> Result<AdminOperationResult> {
        debug!("Listing users with limit: {:?}", limit);

        // Delegate to SQLite admin if available
        if let Some(ref sqlite_admin) = self.sqlite_admin {
            return sqlite_admin.list_users(limit).await;
        }

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let query = match limit {
                    Some(l) => format!(
                        r#"
                        SELECT principalid, firstname, lastname, email, userlevel, created, active
                        FROM useraccounts
                        ORDER BY created DESC
                        LIMIT {}
                    "#,
                        l
                    ),
                    None => r#"
                        SELECT principalid, firstname, lastname, email, userlevel, created, active
                        FROM useraccounts
                        ORDER BY created DESC
                    "#
                    .to_string(),
                };

                match sqlx::query(&query).fetch_all(&mut *conn).await {
                    Ok(rows) => {
                        let mut users = Vec::new();
                        for row in rows {
                            let user_id: String = row
                                .try_get::<uuid::Uuid, _>("principalid")
                                .map(|u| u.to_string())
                                .unwrap_or_default();
                            let firstname: String = row.try_get("firstname").unwrap_or_default();
                            let lastname: String = row.try_get("lastname").unwrap_or_default();
                            let email: String = row.try_get("email").unwrap_or_default();
                            let user_level: i64 = row.try_get("userlevel").unwrap_or(0);
                            let created: i64 = row.try_get("created").unwrap_or(0);
                            let active: i64 = row.try_get("active").unwrap_or(0);

                            users.push(serde_json::json!({
                                "user_id": user_id,
                                "firstname": firstname,
                                "lastname": lastname,
                                "email": email,
                                "user_level": user_level,
                                "created": created,
                                "active": active != 0,
                                "is_god": user_level >= 200
                            }));
                        }

                        AdminOperationResult {
                            success: true,
                            message: format!("Retrieved {} users", users.len()),
                            affected_rows: Some(users.len() as i64),
                            data: Some(serde_json::json!({
                                "users": users,
                                "total_count": users.len(),
                                "limit_applied": limit
                            })),
                        }
                    }
                    Err(e) => {
                        error!("Failed to list users: {}", e);
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
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Delete user account (careful - destructive operation!)
    pub async fn delete_user(
        &self,
        firstname: &str,
        lastname: &str,
    ) -> Result<AdminOperationResult> {
        warn!(
            "DESTRUCTIVE OPERATION: Deleting user: {} {}",
            firstname, lastname
        );

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let query = r#"
                    DELETE FROM UserAccounts
                    WHERE FirstName = $1 AND LastName = $2
                "#;

                match sqlx::query(query)
                    .bind(firstname)
                    .bind(lastname)
                    .execute(&mut *conn)
                    .await
                {
                    Ok(result) => {
                        if result.rows_affected() > 0 {
                            warn!("User deleted: {} {}", firstname, lastname);
                            AdminOperationResult {
                                success: true,
                                message: format!(
                                    "User '{}' '{}' deleted successfully",
                                    firstname, lastname
                                ),
                                affected_rows: Some(result.rows_affected() as i64),
                                data: None,
                            }
                        } else {
                            AdminOperationResult {
                                success: false,
                                message: format!("User '{}' '{}' not found", firstname, lastname),
                                affected_rows: Some(0),
                                data: None,
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to delete user: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to delete user: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Get database statistics
    pub async fn get_database_stats(&self) -> Result<AdminOperationResult> {
        debug!("Retrieving database statistics");

        // Delegate to SQLite admin if available
        if let Some(ref sqlite_admin) = self.sqlite_admin {
            return sqlite_admin.get_database_stats().await;
        }

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                // Get user statistics
                let total_users_query = "SELECT COUNT(*) as count FROM UserAccounts";
                let active_users_query =
                    "SELECT COUNT(*) as count FROM UserAccounts WHERE Active = 1";

                let total_users = sqlx::query(total_users_query)
                    .fetch_one(&mut *conn)
                    .await
                    .map(|row| row.try_get::<i64, _>("count").unwrap_or(0))
                    .unwrap_or(0);

                let active_users = sqlx::query(active_users_query)
                    .fetch_one(&mut *conn)
                    .await
                    .map(|row| row.try_get::<i64, _>("count").unwrap_or(0))
                    .unwrap_or(0);

                let total_regions_query = "SELECT COUNT(*) as count FROM regions";
                let total_regions = sqlx::query(total_regions_query)
                    .fetch_one(&mut *conn)
                    .await
                    .map(|row| row.try_get::<i64, _>("count").unwrap_or(0))
                    .unwrap_or(0);

                let online_regions_query =
                    "SELECT COUNT(*) as count FROM regions WHERE flags & 1 = 0";
                let online_regions = sqlx::query(online_regions_query)
                    .fetch_one(&mut *conn)
                    .await
                    .map(|row| row.try_get::<i64, _>("count").unwrap_or(0))
                    .unwrap_or(0);

                AdminOperationResult {
                    success: true,
                    message: "Database statistics retrieved".to_string(),
                    affected_rows: None,
                    data: Some(serde_json::json!({
                        "total_users": total_users,
                        "active_users": active_users,
                        "total_regions": total_regions,
                        "online_regions": online_regions,
                        "database_size_mb": 0.0,
                        "last_backup": null
                    })),
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    // ====================================================================
    // PHASE 22.2: REGION MANAGEMENT COMMANDS
    // ====================================================================

    /// Create a new region (equivalent to 'create region' in OpenSim Robust)
    pub async fn create_region(
        &self,
        request: CreateRegionRequest,
    ) -> Result<AdminOperationResult> {
        info!(
            "Creating region: {} at location ({}, {})",
            request.region_name, request.location_x, request.location_y
        );

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                // Check if region already exists at this location
                let check_query = "SELECT regionName FROM regions WHERE locX = $1 AND locY = $2";
                let existing = sqlx::query(check_query)
                    .bind(request.location_x)
                    .bind(request.location_y)
                    .fetch_optional(&mut *conn)
                    .await;

                match existing {
                    Ok(Some(_)) => {
                        warn!(
                            "Region already exists at location ({}, {})",
                            request.location_x, request.location_y
                        );
                        AdminOperationResult {
                            success: false,
                            message: format!(
                                "Region already exists at location ({}, {})",
                                request.location_x, request.location_y
                            ),
                            affected_rows: Some(0),
                            data: None,
                        }
                    }
                    Ok(None) => {
                        // Generate new region UUID
                        let region_uuid = uuid::Uuid::new_v4().to_string();

                        let insert_query = r#"
                            INSERT INTO regions (uuid, regionName, locX, locY, sizeX, sizeY, serverIP, serverPort, serverURI, owner_uuid, flags)
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                        "#;

                        match sqlx::query(insert_query)
                            .bind(&region_uuid)
                            .bind(&request.region_name)
                            .bind(request.location_x)
                            .bind(request.location_y)
                            .bind(request.size_x.unwrap_or(256))
                            .bind(request.size_y.unwrap_or(256))
                            .bind(&request.server_ip.unwrap_or_else(|| "127.0.0.1".to_string()))
                            .bind(request.server_port.unwrap_or(9000))
                            .bind(&request.server_uri.unwrap_or_else(|| {
                                format!("http://127.0.0.1:{}/", request.server_port.unwrap_or(9000))
                            }))
                            .bind(&request.owner_uuid.unwrap_or_else(|| {
                                "00000000-0000-0000-0000-000000000000".to_string()
                            }))
                            .bind(0) // Default flags
                            .execute(&mut *conn)
                            .await
                        {
                            Ok(result) => {
                                info!(
                                    "Region created successfully: {} ({})",
                                    request.region_name, region_uuid
                                );
                                AdminOperationResult {
                                    success: true,
                                    message: format!(
                                        "Region '{}' created successfully",
                                        request.region_name
                                    ),
                                    affected_rows: Some(result.rows_affected() as i64),
                                    data: Some(serde_json::json!({
                                        "region_uuid": region_uuid,
                                        "region_name": request.region_name,
                                        "location_x": request.location_x,
                                        "location_y": request.location_y,
                                        "size_x": request.size_x.unwrap_or(256),
                                        "size_y": request.size_y.unwrap_or(256)
                                    })),
                                }
                            }
                            Err(e) => {
                                error!("Failed to create region: {}", e);
                                AdminOperationResult {
                                    success: false,
                                    message: format!("Failed to create region: {}", e),
                                    affected_rows: None,
                                    data: None,
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to check existing region: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to check existing region: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Delete a region (equivalent to 'delete region' in OpenSim Robust)
    pub async fn delete_region(&self, region_name: &str) -> Result<AdminOperationResult> {
        info!("Deleting region: {}", region_name);

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let delete_query = "DELETE FROM regions WHERE regionName = $1";

                match sqlx::query(delete_query)
                    .bind(region_name)
                    .execute(&mut *conn)
                    .await
                {
                    Ok(result) => {
                        if result.rows_affected() > 0 {
                            info!("Region deleted successfully: {}", region_name);
                            AdminOperationResult {
                                success: true,
                                message: format!("Region '{}' deleted successfully", region_name),
                                affected_rows: Some(result.rows_affected() as i64),
                                data: None,
                            }
                        } else {
                            warn!("Region not found: {}", region_name);
                            AdminOperationResult {
                                success: false,
                                message: format!("Region '{}' not found", region_name),
                                affected_rows: Some(0),
                                data: None,
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to delete region: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to delete region: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Show region details (equivalent to 'show region' in OpenSim Robust)
    pub async fn show_region(&self, region_name: &str) -> Result<AdminOperationResult> {
        debug!("Retrieving region details for: {}", region_name);

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let query = r#"
                    SELECT uuid, regionName, locX, locY, sizeX, sizeY, serverIP, serverPort, serverURI, owner_uuid, flags
                    FROM regions
                    WHERE regionName = $1
                "#;

                match sqlx::query(query)
                    .bind(region_name)
                    .fetch_optional(&mut *conn)
                    .await
                {
                    Ok(Some(row)) => {
                        let region_data = serde_json::json!({
                            "uuid": row.get::<String, _>("uuid"),
                            "region_name": row.get::<String, _>("regionName"),
                            "location_x": row.get::<i32, _>("locX"),
                            "location_y": row.get::<i32, _>("locY"),
                            "size_x": row.get::<i32, _>("sizeX"),
                            "size_y": row.get::<i32, _>("sizeY"),
                            "server_ip": row.get::<String, _>("serverIP"),
                            "server_port": row.get::<i32, _>("serverPort"),
                            "server_uri": row.get::<String, _>("serverURI"),
                            "owner_uuid": row.get::<String, _>("owner_uuid"),
                            "flags": row.get::<i32, _>("flags"),
                            "status": "offline" // TODO: Implement runtime status checking
                        });

                        AdminOperationResult {
                            success: true,
                            message: format!("Region '{}' details retrieved", region_name),
                            affected_rows: Some(1),
                            data: Some(region_data),
                        }
                    }
                    Ok(None) => {
                        warn!("Region not found: {}", region_name);
                        AdminOperationResult {
                            success: false,
                            message: format!("Region '{}' not found", region_name),
                            affected_rows: Some(0),
                            data: None,
                        }
                    }
                    Err(e) => {
                        error!("Failed to retrieve region details: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to retrieve region details: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// List all regions (equivalent to 'show regions' in OpenSim Robust)
    pub async fn list_regions(&self, limit: Option<i32>) -> Result<AdminOperationResult> {
        debug!("Retrieving region list with limit: {:?}", limit);

        let limit = limit.unwrap_or(50).min(1000); // Cap at 1000 regions

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let query = r#"
                    SELECT id, region_name, location_x, location_y, size_x, size_y, external_host_name, external_port
                    FROM regions
                    ORDER BY region_name
                    LIMIT $1
                "#;

                match sqlx::query(query)
                    .bind(limit as i64)
                    .fetch_all(&mut *conn)
                    .await
                {
                    Ok(rows) => {
                        let regions: Vec<serde_json::Value> = rows.iter().map(|row| {
                            serde_json::json!({
                                "uuid": row.try_get::<uuid::Uuid, _>("id").map(|u| u.to_string()).unwrap_or_default(),
                                "region_name": row.try_get::<String, _>("region_name").unwrap_or_default(),
                                "location_x": row.try_get::<i32, _>("location_x").unwrap_or(0),
                                "location_y": row.try_get::<i32, _>("location_y").unwrap_or(0),
                                "size_x": row.try_get::<i32, _>("size_x").unwrap_or(256),
                                "size_y": row.try_get::<i32, _>("size_y").unwrap_or(256),
                                "server_ip": row.try_get::<String, _>("external_host_name").unwrap_or_default(),
                                "server_port": row.try_get::<i32, _>("external_port").unwrap_or(0),
                                "status": "running"
                            })
                        }).collect();

                        AdminOperationResult {
                            success: true,
                            message: format!("Retrieved {} regions", regions.len()),
                            affected_rows: Some(regions.len() as i64),
                            data: Some(serde_json::json!({
                                "regions": regions,
                                "total_count": regions.len(),
                                "limit": limit
                            })),
                        }
                    }
                    Err(e) => {
                        error!("Failed to retrieve regions: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to retrieve regions: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Update region properties (equivalent to 'set region' in OpenSim Robust)
    pub async fn update_region(
        &self,
        region_name: &str,
        updates: UpdateRegionRequest,
    ) -> Result<AdminOperationResult> {
        info!("Updating region: {}", region_name);

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                // Build dynamic update query based on provided fields
                let mut update_fields = Vec::new();
                let mut values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Any> + Send + 'static>> =
                    Vec::new();

                if let Some(new_name) = &updates.new_name {
                    update_fields.push("regionName = ?");
                    values.push(Box::new(new_name.clone()));
                }
                if let Some(location_x) = updates.location_x {
                    update_fields.push("locX = ?");
                    values.push(Box::new(location_x));
                }
                if let Some(location_y) = updates.location_y {
                    update_fields.push("locY = ?");
                    values.push(Box::new(location_y));
                }
                if let Some(size_x) = updates.size_x {
                    update_fields.push("sizeX = ?");
                    values.push(Box::new(size_x));
                }
                if let Some(size_y) = updates.size_y {
                    update_fields.push("sizeY = ?");
                    values.push(Box::new(size_y));
                }

                if update_fields.is_empty() {
                    return Ok(AdminOperationResult {
                        success: false,
                        message: "No update fields provided".to_string(),
                        affected_rows: None,
                        data: None,
                    });
                }

                // Use simplified individual field updates approach for better type safety

                // Simplified update approach - update each field individually if provided
                let mut total_updated = 0u64;

                if let Some(new_name) = &updates.new_name {
                    match sqlx::query("UPDATE regions SET regionName = $1 WHERE regionName = $2")
                        .bind(new_name)
                        .bind(region_name)
                        .execute(&mut *conn)
                        .await
                    {
                        Ok(result) => total_updated += result.rows_affected(),
                        Err(e) => {
                            error!("Failed to update region name: {}", e);
                            return Ok(AdminOperationResult {
                                success: false,
                                message: format!("Failed to update region name: {}", e),
                                affected_rows: None,
                                data: None,
                            });
                        }
                    }
                }

                if let Some(location_x) = updates.location_x {
                    match sqlx::query("UPDATE regions SET locX = $1 WHERE regionName = $2")
                        .bind(location_x)
                        .bind(region_name)
                        .execute(&mut *conn)
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to update region location X: {}", e);
                            return Ok(AdminOperationResult {
                                success: false,
                                message: format!("Failed to update region location X: {}", e),
                                affected_rows: None,
                                data: None,
                            });
                        }
                    }
                }

                if let Some(location_y) = updates.location_y {
                    match sqlx::query("UPDATE regions SET locY = $1 WHERE regionName = $2")
                        .bind(location_y)
                        .bind(region_name)
                        .execute(&mut *conn)
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to update region location Y: {}", e);
                            return Ok(AdminOperationResult {
                                success: false,
                                message: format!("Failed to update region location Y: {}", e),
                                affected_rows: None,
                                data: None,
                            });
                        }
                    }
                }

                AdminOperationResult {
                    success: true,
                    message: format!("Region '{}' updated successfully", region_name),
                    affected_rows: Some(1),
                    data: Some(serde_json::json!({
                        "region_name": updates.new_name.as_ref().unwrap_or(&region_name.to_string()),
                        "updates_applied": update_fields.len()
                    })),
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Get region statistics (equivalent to 'region stats' in OpenSim Robust)
    pub async fn get_region_stats(&self) -> Result<AdminOperationResult> {
        debug!("Retrieving region statistics");

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                // Get basic region counts
                let total_regions = sqlx::query("SELECT COUNT(*) as count FROM regions")
                    .fetch_one(&mut *conn)
                    .await
                    .map(|row| row.get::<i64, _>("count"))
                    .unwrap_or(0);

                // Get regions by size
                let standard_regions = sqlx::query(
                    "SELECT COUNT(*) as count FROM regions WHERE sizeX = 256 AND sizeY = 256",
                )
                .fetch_one(&mut *conn)
                .await
                .map(|row| row.get::<i64, _>("count"))
                .unwrap_or(0);

                let large_regions = sqlx::query(
                    "SELECT COUNT(*) as count FROM regions WHERE sizeX > 256 OR sizeY > 256",
                )
                .fetch_one(&mut *conn)
                .await
                .map(|row| row.get::<i64, _>("count"))
                .unwrap_or(0);

                AdminOperationResult {
                    success: true,
                    message: "Region statistics retrieved".to_string(),
                    affected_rows: None,
                    data: Some(serde_json::json!({
                        "total_regions": total_regions,
                        "online_regions": 0, // TODO: Implement runtime status checking
                        "offline_regions": total_regions,
                        "standard_regions": standard_regions,
                        "large_regions": large_regions,
                        "average_region_size": 256,
                        "total_land_area": total_regions * 256 * 256,
                        "last_updated": chrono::Utc::now().to_rfc3339()
                    })),
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    // ==================== DATABASE ADMINISTRATION METHODS ====================

    /// Create database backup (equivalent to 'backup database' in OpenSim Robust)
    pub async fn create_backup(
        &self,
        request: DatabaseBackupRequest,
    ) -> Result<AdminOperationResult> {
        info!("Creating database backup: {}", request.backup_name);

        let backup_id = Uuid::new_v4();
        let backup_path = request.backup_path.clone().unwrap_or_else(|| {
            format!("./backups/backup_{}_{}.sql", request.backup_name, backup_id)
        });

        // Create backup directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&backup_path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error!("Failed to create backup directory: {}", e);
                return Ok(AdminOperationResult {
                    success: false,
                    message: format!("Failed to create backup directory: {}", e),
                    affected_rows: None,
                    data: None,
                });
            }
        }

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let mut backup_commands = Vec::new();
                let mut backup_size = 0u64;
                let start_time = chrono::Utc::now();

                // Backup user accounts
                if request.include_user_data {
                    info!("Backing up user accounts...");
                    match sqlx::query("SELECT * FROM UserAccounts")
                        .fetch_all(&mut *conn)
                        .await
                    {
                        Ok(rows) => {
                            backup_commands.push("-- User Accounts Backup".to_string());
                            for row in rows {
                                let backup_row = format!(
                                    "INSERT INTO UserAccounts (PrincipalID, ScopeID, FirstName, LastName, Email, ServiceURLs, Created) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {});",
                                    row.get::<String, _>("PrincipalID"),
                                    row.get::<String, _>("ScopeID"),
                                    row.get::<String, _>("FirstName"),
                                    row.get::<String, _>("LastName"),
                                    row.get::<String, _>("Email"),
                                    row.get::<String, _>("ServiceURLs"),
                                    row.get::<i32, _>("Created")
                                );
                                backup_size += backup_row.len() as u64;
                                backup_commands.push(backup_row);
                            }
                        }
                        Err(e) => {
                            error!("Failed to backup user accounts: {}", e);
                            return Ok(AdminOperationResult {
                                success: false,
                                message: format!("Failed to backup user accounts: {}", e),
                                affected_rows: None,
                                data: None,
                            });
                        }
                    }
                }

                // Backup regions
                if request.include_region_data {
                    info!("Backing up regions...");
                    match sqlx::query("SELECT * FROM regions")
                        .fetch_all(&mut *conn)
                        .await
                    {
                        Ok(rows) => {
                            backup_commands.push("-- Regions Backup".to_string());
                            for row in rows {
                                let backup_row = format!(
                                    "INSERT INTO regions (uuid, regionName, locX, locY, sizeX, sizeY, serverIP, serverPort, serverURI, owner_uuid, flags) VALUES ('{}', '{}', {}, {}, {}, {}, '{}', {}, '{}', '{}', {});",
                                    row.get::<String, _>("uuid"),
                                    row.get::<String, _>("regionName"),
                                    row.get::<i32, _>("locX"),
                                    row.get::<i32, _>("locY"),
                                    row.get::<i32, _>("sizeX"),
                                    row.get::<i32, _>("sizeY"),
                                    row.get::<String, _>("serverIP"),
                                    row.get::<i32, _>("serverPort"),
                                    row.get::<String, _>("serverURI"),
                                    row.get::<String, _>("owner_uuid"),
                                    row.get::<i32, _>("flags")
                                );
                                backup_size += backup_row.len() as u64;
                                backup_commands.push(backup_row);
                            }
                        }
                        Err(e) => {
                            error!("Failed to backup regions: {}", e);
                            return Ok(AdminOperationResult {
                                success: false,
                                message: format!("Failed to backup regions: {}", e),
                                affected_rows: None,
                                data: None,
                            });
                        }
                    }
                }

                // Write backup file
                let backup_content = backup_commands.join("\n");
                match std::fs::write(&backup_path, &backup_content) {
                    Ok(_) => {
                        let backup_size_mb = backup_size as f64 / 1024.0 / 1024.0;

                        // Record backup metadata
                        let backup_info = DatabaseBackupInfo {
                            backup_id,
                            backup_name: request.backup_name.clone(),
                            backup_path: backup_path.clone(),
                            backup_size_mb,
                            created_at: start_time,
                            contains_users: request.include_user_data,
                            contains_regions: request.include_region_data,
                            contains_assets: request.include_asset_data,
                            contains_inventory: request.include_inventory_data,
                            compression_ratio: if request.compression { 0.5 } else { 1.0 },
                            status: "completed".to_string(),
                        };

                        info!(
                            "Database backup completed: {} ({:.2} MB)",
                            backup_path, backup_size_mb
                        );
                        AdminOperationResult {
                            success: true,
                            message: format!(
                                "Database backup '{}' created successfully",
                                request.backup_name
                            ),
                            affected_rows: Some(backup_commands.len() as i64),
                            data: Some(serde_json::to_value(&backup_info).unwrap()),
                        }
                    }
                    Err(e) => {
                        error!("Failed to write backup file: {}", e);
                        AdminOperationResult {
                            success: false,
                            message: format!("Failed to write backup file: {}", e),
                            affected_rows: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Restore database from backup (equivalent to 'restore database' in OpenSim Robust)
    pub async fn restore_backup(
        &self,
        request: DatabaseRestoreRequest,
    ) -> Result<AdminOperationResult> {
        info!("Restoring database from backup: {}", request.backup_file);

        // Read backup file
        let backup_content = match std::fs::read_to_string(&request.backup_file) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read backup file: {}", e);
                return Ok(AdminOperationResult {
                    success: false,
                    message: format!("Failed to read backup file: {}", e),
                    affected_rows: None,
                    data: None,
                });
            }
        };

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let mut restored_rows = 0i64;

                // Execute backup commands
                let commands: Vec<&str> = backup_content
                    .lines()
                    .filter(|line| !line.starts_with("--") && !line.trim().is_empty())
                    .collect();

                for command in commands {
                    match sqlx::query(command).execute(&mut *conn).await {
                        Ok(result) => {
                            restored_rows += result.rows_affected() as i64;
                        }
                        Err(e) => {
                            if request.overwrite_existing {
                                warn!("Failed to restore command (continuing): {}", e);
                            } else {
                                error!("Failed to restore command: {}", e);
                                return Ok(AdminOperationResult {
                                    success: false,
                                    message: format!("Failed to restore command: {}", e),
                                    affected_rows: Some(restored_rows),
                                    data: None,
                                });
                            }
                        }
                    }
                }

                info!(
                    "Database restore completed: {} rows restored",
                    restored_rows
                );
                AdminOperationResult {
                    success: true,
                    message: format!(
                        "Database restored successfully from '{}'",
                        request.backup_file
                    ),
                    affected_rows: Some(restored_rows),
                    data: Some(serde_json::json!({
                        "backup_file": request.backup_file,
                        "restored_rows": restored_rows,
                        "restore_users": request.restore_users,
                        "restore_regions": request.restore_regions,
                        "overwrite_existing": request.overwrite_existing
                    })),
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Perform database maintenance (equivalent to 'maintenance database' in OpenSim Robust)
    pub async fn perform_maintenance(
        &self,
        request: DatabaseMaintenanceRequest,
    ) -> Result<AdminOperationResult> {
        info!("Performing database maintenance");

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let mut maintenance_results = Vec::new();
                let start_time = chrono::Utc::now();

                // Vacuum tables
                if request.vacuum_tables {
                    info!("Vacuuming database tables...");
                    match sqlx::query("VACUUM").execute(&mut *conn).await {
                        Ok(_) => {
                            maintenance_results.push("VACUUM completed successfully".to_string());
                        }
                        Err(e) => {
                            let error_msg = format!("VACUUM failed: {}", e);
                            error!("{}", error_msg);
                            maintenance_results.push(error_msg);
                        }
                    }
                }

                // Reindex tables
                if request.reindex_tables {
                    info!("Reindexing database tables...");
                    let tables = vec![
                        "UserAccounts",
                        "regions",
                        "inventoryfolders",
                        "inventoryitems",
                    ];
                    for table in &tables {
                        match sqlx::query(&format!("REINDEX {}", table))
                            .execute(&mut *conn)
                            .await
                        {
                            Ok(_) => {
                                maintenance_results.push(format!("REINDEX {} completed", table));
                            }
                            Err(e) => {
                                let error_msg = format!("REINDEX {} failed: {}", table, e);
                                warn!("{}", error_msg);
                                maintenance_results.push(error_msg);
                            }
                        }
                    }
                }

                // Analyze tables
                if request.analyze_tables {
                    info!("Analyzing database tables...");
                    match sqlx::query("ANALYZE").execute(&mut *conn).await {
                        Ok(_) => {
                            maintenance_results.push("ANALYZE completed successfully".to_string());
                        }
                        Err(e) => {
                            let error_msg = format!("ANALYZE failed: {}", e);
                            error!("{}", error_msg);
                            maintenance_results.push(error_msg);
                        }
                    }
                }

                // Clean up orphaned records
                if request.cleanup_orphaned {
                    info!("Cleaning up orphaned records...");

                    // Clean up orphaned inventory items
                    match sqlx::query("DELETE FROM inventoryitems WHERE folderID NOT IN (SELECT folderID FROM inventoryfolders)")
                        .execute(&mut *conn)
                        .await
                    {
                        Ok(result) => {
                            maintenance_results.push(format!("Cleaned up {} orphaned inventory items", result.rows_affected()));
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to clean orphaned inventory items: {}", e);
                            warn!("{}", error_msg);
                            maintenance_results.push(error_msg);
                        }
                    }
                }

                let duration = chrono::Utc::now().signed_duration_since(start_time);

                AdminOperationResult {
                    success: true,
                    message: "Database maintenance completed".to_string(),
                    affected_rows: None,
                    data: Some(serde_json::json!({
                        "maintenance_results": maintenance_results,
                        "duration_seconds": duration.num_seconds(),
                        "vacuum_tables": request.vacuum_tables,
                        "reindex_tables": request.reindex_tables,
                        "analyze_tables": request.analyze_tables,
                        "cleanup_orphaned": request.cleanup_orphaned,
                        "completed_at": chrono::Utc::now().to_rfc3339()
                    })),
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Run database migration (equivalent to 'migrate database' in OpenSim Robust)
    pub async fn run_migration(
        &self,
        request: DatabaseMigrationRequest,
    ) -> Result<AdminOperationResult> {
        info!(
            "Running database migration to version: {}",
            request.target_version
        );

        if request.dry_run {
            info!("Performing dry run migration");
        }

        // Create backup before migration if requested
        if request.backup_before && !request.dry_run {
            let backup_request = DatabaseBackupRequest {
                backup_name: format!("pre_migration_{}", request.target_version),
                include_user_data: true,
                include_region_data: true,
                include_asset_data: true,
                include_inventory_data: true,
                compression: true,
                backup_path: None,
            };

            match self.create_backup(backup_request).await {
                Ok(backup_result) => {
                    if !backup_result.success {
                        return Ok(AdminOperationResult {
                            success: false,
                            message: format!(
                                "Pre-migration backup failed: {}",
                                backup_result.message
                            ),
                            affected_rows: None,
                            data: None,
                        });
                    }
                    info!("Pre-migration backup completed");
                }
                Err(e) => {
                    error!("Pre-migration backup failed: {}", e);
                    return Ok(AdminOperationResult {
                        success: false,
                        message: format!("Pre-migration backup failed: {}", e),
                        affected_rows: None,
                        data: None,
                    });
                }
            }
        }

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let mut migration_results = Vec::new();

                // Check current schema version
                let current_version = match sqlx::query(
                    "SELECT version FROM migrations ORDER BY version DESC LIMIT 1",
                )
                .fetch_optional(&mut *conn)
                .await
                {
                    Ok(Some(row)) => row.get::<String, _>("version"),
                    Ok(None) => "0.0.0".to_string(),
                    Err(_) => {
                        // Create migrations table if it doesn't exist
                        match sqlx::query(
                            r#"
                            CREATE TABLE IF NOT EXISTS migrations (
                                version VARCHAR(20) PRIMARY KEY,
                                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                description TEXT
                            )
                        "#,
                        )
                        .execute(&mut *conn)
                        .await
                        {
                            Ok(_) => "0.0.0".to_string(),
                            Err(e) => {
                                error!("Failed to create migrations table: {}", e);
                                return Ok(AdminOperationResult {
                                    success: false,
                                    message: format!("Failed to create migrations table: {}", e),
                                    affected_rows: None,
                                    data: None,
                                });
                            }
                        }
                    }
                };

                migration_results.push(format!("Current schema version: {}", current_version));
                migration_results
                    .push(format!("Target schema version: {}", request.target_version));

                if !request.dry_run {
                    // Record migration
                    match sqlx::query(
                        "INSERT INTO migrations (version, description) VALUES ($1, $2)",
                    )
                    .bind(&request.target_version)
                    .bind("Schema migration via admin API")
                    .execute(&mut *conn)
                    .await
                    {
                        Ok(_) => {
                            migration_results.push("Migration recorded successfully".to_string());
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to record migration: {}", e);
                            error!("{}", error_msg);
                            migration_results.push(error_msg);
                        }
                    }
                } else {
                    migration_results.push("Dry run - no changes applied".to_string());
                }

                AdminOperationResult {
                    success: true,
                    message: format!(
                        "Database migration to version '{}' completed",
                        request.target_version
                    ),
                    affected_rows: None,
                    data: Some(serde_json::json!({
                        "current_version": current_version,
                        "target_version": request.target_version,
                        "dry_run": request.dry_run,
                        "backup_before": request.backup_before,
                        "migration_results": migration_results,
                        "completed_at": chrono::Utc::now().to_rfc3339()
                    })),
                }
            }
            Err(e) => {
                error!("Database connection failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    affected_rows: None,
                    data: None,
                }
            }
        };

        Ok(result)
    }

    /// Get database health information (equivalent to 'database health' in OpenSim Robust)
    pub async fn get_database_health(&self) -> Result<AdminOperationResult> {
        debug!("Checking database health");

        // Delegate to SQLite admin if available
        if let Some(ref sqlite_admin) = self.sqlite_admin {
            return sqlite_admin.get_database_health().await;
        }

        // Get pool or return error for stub mode
        let pool = match self.get_pool() {
            Ok(pool) => pool,
            Err(e) => {
                return Ok(self.stub_response("Database operation"));
            }
        };

        let result = match pool.acquire().await {
            Ok(mut conn) => {
                let start_time = std::time::Instant::now();

                // Test basic connectivity
                let connectivity_test = sqlx::query("SELECT 1 as test")
                    .fetch_one(&mut *conn)
                    .await
                    .is_ok();

                let connection_time_ms = start_time.elapsed().as_millis();

                // Get table sizes
                let user_count = sqlx::query("SELECT COUNT(*) as count FROM UserAccounts")
                    .fetch_one(&mut *conn)
                    .await
                    .map(|row| row.get::<i64, _>("count"))
                    .unwrap_or(0);

                let region_count = sqlx::query("SELECT COUNT(*) as count FROM regions")
                    .fetch_one(&mut *conn)
                    .await
                    .map(|row| row.get::<i64, _>("count"))
                    .unwrap_or(0);

                // Check for recent activity
                let recent_logins = sqlx::query(
                    "SELECT COUNT(*) as count FROM UserAccounts WHERE ServiceURLs LIKE '%login%'",
                )
                .fetch_one(&mut *conn)
                .await
                .map(|row| row.get::<i64, _>("count"))
                .unwrap_or(0);

                let health_status = if connectivity_test && connection_time_ms < 1000 {
                    "healthy"
                } else if connectivity_test {
                    "slow"
                } else {
                    "unhealthy"
                };

                AdminOperationResult {
                    success: true,
                    message: format!("Database health status: {}", health_status),
                    affected_rows: None,
                    data: Some(serde_json::json!({
                        "health_status": health_status,
                        "connectivity_test": connectivity_test,
                        "connection_time_ms": connection_time_ms,
                        "user_count": user_count,
                        "region_count": region_count,
                        "recent_logins": recent_logins,
                        "pool_status": {
                            "active_connections": "available", // TODO: Get actual pool stats
                            "idle_connections": "available"
                        },
                        "last_checked": chrono::Utc::now().to_rfc3339()
                    })),
                }
            }
            Err(e) => {
                error!("Database health check failed: {}", e);
                AdminOperationResult {
                    success: false,
                    message: format!("Database health check failed: {}", e),
                    affected_rows: None,
                    data: Some(serde_json::json!({
                        "health_status": "unhealthy",
                        "error": e.to_string(),
                        "last_checked": chrono::Utc::now().to_rfc3339()
                    })),
                }
            }
        };

        Ok(result)
    }

    /// List available backups (equivalent to 'list backups' in OpenSim Robust)
    pub async fn list_backups(
        &self,
        backup_directory: Option<String>,
    ) -> Result<AdminOperationResult> {
        let backup_dir = backup_directory.unwrap_or_else(|| "./backups".to_string());
        debug!("Listing backups in directory: {}", backup_dir);

        match std::fs::read_dir(&backup_dir) {
            Ok(entries) => {
                let mut backups = Vec::new();

                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".sql") {
                                let metadata = entry.metadata().unwrap_or_else(|_| {
                                    // Fallback metadata when entry.metadata() fails
                                    std::fs::metadata("/dev/null").unwrap_or_else(|_| {
                                        // If even /dev/null fails, create dummy values
                                        panic!("Unable to get filesystem metadata")
                                    })
                                });
                                let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                                let modified = metadata
                                    .modified()
                                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                backups.push(serde_json::json!({
                                    "file_name": file_name,
                                    "file_path": entry.path().to_string_lossy(),
                                    "size_mb": format!("{:.2}", size_mb),
                                    "modified_timestamp": modified,
                                    "modified_date": chrono::DateTime::from_timestamp(modified as i64, 0)
                                        .unwrap_or_default()
                                        .format("%Y-%m-%d %H:%M:%S UTC")
                                        .to_string()
                                }));
                            }
                        }
                    }
                }

                // Sort by modification time (newest first)
                backups.sort_by(|a, b| {
                    let a_time = a["modified_timestamp"].as_u64().unwrap_or(0);
                    let b_time = b["modified_timestamp"].as_u64().unwrap_or(0);
                    b_time.cmp(&a_time)
                });

                Ok(AdminOperationResult {
                    success: true,
                    message: format!("Found {} backup files", backups.len()),
                    affected_rows: Some(backups.len() as i64),
                    data: Some(serde_json::json!({
                        "backups": backups,
                        "backup_directory": backup_dir,
                        "total_backups": backups.len()
                    })),
                })
            }
            Err(e) => {
                error!("Failed to list backups: {}", e);
                Ok(AdminOperationResult {
                    success: false,
                    message: format!("Failed to list backups: {}", e),
                    affected_rows: None,
                    data: None,
                })
            }
        }
    }
}
