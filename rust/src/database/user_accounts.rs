//! User account database operations

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::multi_backend::DatabaseConnection;

/// User account data structure - PostgreSQL compatible
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserAccount {
    #[sqlx(rename = "principalid")]
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub salt: String,
    #[sqlx(rename = "firstname")]
    pub first_name: String,
    #[sqlx(rename = "lastname")]
    pub last_name: String,
    pub home_region_id: Option<Uuid>,
    pub home_position_x: f32,
    pub home_position_y: f32,
    pub home_position_z: f32,
    pub home_look_at_x: f32,
    pub home_look_at_y: f32,
    pub home_look_at_z: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub user_level: i32,
    pub user_flags: i32,
    pub god_level: i32,
    pub custom_type: String,
}

/// User profile data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub about_text: String,
    pub first_life_about_text: String,
    pub image_id: Option<Uuid>,
    pub first_life_image_id: Option<Uuid>,
    pub web_url: String,
    pub wants_to_mask: i32,
    pub wants_to_text: String,
    pub skills_mask: i32,
    pub skills_text: String,
    pub languages: String,
    pub partner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User session data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_token: String,
    pub secure_session_token: String,
    pub region_id: Option<Uuid>,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub look_at_x: f32,
    pub look_at_y: f32,
    pub look_at_z: f32,
    pub login_time: DateTime<Utc>,
    pub logout_time: Option<DateTime<Utc>>,
    pub last_seen: DateTime<Utc>,
    pub is_active: bool,
}

/// User appearance data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserAppearance {
    pub user_id: Uuid,
    pub appearance_data: Option<String>,
    pub wearables: Option<String>,
    pub attachments: Option<String>,
    pub visual_params: Option<Vec<u8>>,
    pub texture_data: Option<String>,
    pub avatar_height: f32,
    pub hip_offset: f32,
    pub updated_at: DateTime<Utc>,
}

/// Create user request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub home_region_id: Option<Uuid>,
}

/// Update user request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub home_region_id: Option<Uuid>,
    pub home_position_x: Option<f32>,
    pub home_position_y: Option<f32>,
    pub home_position_z: Option<f32>,
    pub user_level: Option<i32>,
    pub user_flags: Option<i32>,
    pub god_level: Option<i32>,
}

/// User account database operations
#[derive(Debug)]
pub struct UserAccountDatabase {
    connection: Option<Arc<DatabaseConnection>>,
}

impl UserAccountDatabase {
    /// Create a new user account database
    pub async fn new(connection: Arc<DatabaseConnection>) -> Result<Self> {
        info!("Initializing user account database");
        Ok(Self {
            connection: Some(connection),
        })
    }

    /// Create a stub user account database for SQLite compatibility
    /// ELEGANT SOLUTION: Provides database interface without requiring legacy PostgreSQL connection
    pub fn new_stub() -> Self {
        info!("Creating stub user account database for SQLite compatibility");
        Self { connection: None }
    }

    /// Get database connection pool (ELEGANT SOLUTION: handles stub mode gracefully)
    fn pool(&self) -> Result<&sqlx::PgPool> {
        match &self.connection {
            Some(conn) => match conn.as_ref() {
                super::multi_backend::DatabaseConnection::PostgreSQL(pool) => Ok(pool),
                _ => Err(anyhow!("Database is not PostgreSQL")),
            },
            None => Err(anyhow!("Database operation not available in stub mode - database manager will use admin pool"))
        }
    }

    /// Create a new user account
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserAccount> {
        info!("Creating new user account: {}", request.username);

        // Generate password hash and salt
        let salt = generate_salt();
        let password_hash = hash_password(&request.password, &salt)?;

        let user = sqlx::query_as::<_, UserAccount>(
            r#"
            INSERT INTO useraccounts (
                username, email, password_hash, salt, firstname, lastname, home_region_id,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(&request.username)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&salt)
        .bind(&request.first_name)
        .bind(&request.last_name)
        .bind(request.home_region_id)
        .fetch_one(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to create user: {}", e))?;

        // Create default profile
        self.create_user_profile(user.id).await?;

        // Create default appearance
        self.create_user_appearance(user.id).await?;

        info!("Created user account: {} ({})", user.username, user.id);
        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<UserAccount>> {
        debug!("Getting user by ID: {}", user_id);

        let user =
            sqlx::query_as::<_, UserAccount>("SELECT * FROM useraccounts WHERE principalid = $1")
                .bind(user_id)
                .fetch_optional(self.pool()?)
                .await
                .map_err(|e| anyhow!("Failed to get user by ID: {}", e))?;

        Ok(user)
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<UserAccount>> {
        debug!("Getting user by username: {}", username);

        let user =
            sqlx::query_as::<_, UserAccount>("SELECT * FROM useraccounts WHERE username = $1")
                .bind(username)
                .fetch_optional(self.pool()?)
                .await
                .map_err(|e| anyhow!("Failed to get user by username: {}", e))?;

        Ok(user)
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<UserAccount>> {
        debug!("Getting user by email: {}", email);

        let user = sqlx::query_as::<_, UserAccount>("SELECT * FROM useraccounts WHERE email = $1")
            .bind(email)
            .fetch_optional(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to get user by email: {}", e))?;

        Ok(user)
    }

    /// Get user by first and last name (for SL login compatibility)
    pub async fn get_user_by_name(
        &self,
        first_name: &str,
        last_name: &str,
    ) -> Result<Option<UserAccount>> {
        debug!("Getting user by name: {} {}", first_name, last_name);

        // Use authenticate_user_opensim function instead to get proper UserAccount
        return self
            .authenticate_user_opensim(first_name, last_name, "")
            .await;
    }

    /// Check if user exists by first and last name
    pub async fn user_exists(&self, first_name: &str, last_name: &str) -> Result<bool> {
        debug!("Checking if user exists: {} {}", first_name, last_name);

        let result = sqlx::query(
            "SELECT 1 FROM useraccounts WHERE firstname = $1 AND lastname = $2 LIMIT 1",
        )
        .bind(first_name)
        .bind(last_name)
        .fetch_optional(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to check user existence: {}", e))?;

        Ok(result.is_some())
    }

    /// Update user account
    pub async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<UserAccount> {
        debug!("Updating user: {}", user_id);

        let mut query = "UPDATE useraccounts SET updated_at = NOW()".to_string();
        let mut params = Vec::new();
        let mut param_count = 1;

        if let Some(email) = &request.email {
            query.push_str(&format!(", email = ${}", param_count));
            params.push(email.clone());
            param_count += 1;
        }

        if let Some(first_name) = &request.first_name {
            query.push_str(&format!(", first_name = ${}", param_count));
            params.push(first_name.clone());
            param_count += 1;
        }

        if let Some(last_name) = &request.last_name {
            query.push_str(&format!(", last_name = ${}", param_count));
            params.push(last_name.clone());
            param_count += 1;
        }

        if let Some(home_region_id) = request.home_region_id {
            query.push_str(&format!(", home_region_id = ${}", param_count));
            params.push(home_region_id.to_string());
            param_count += 1;
        }

        if let Some(pos_x) = request.home_position_x {
            query.push_str(&format!(", home_position_x = ${}", param_count));
            params.push(pos_x.to_string());
            param_count += 1;
        }

        if let Some(pos_y) = request.home_position_y {
            query.push_str(&format!(", home_position_y = ${}", param_count));
            params.push(pos_y.to_string());
            param_count += 1;
        }

        if let Some(pos_z) = request.home_position_z {
            query.push_str(&format!(", home_position_z = ${}", param_count));
            params.push(pos_z.to_string());
            param_count += 1;
        }

        if let Some(user_level) = request.user_level {
            query.push_str(&format!(", user_level = ${}", param_count));
            params.push(user_level.to_string());
            param_count += 1;
        }

        if let Some(user_flags) = request.user_flags {
            query.push_str(&format!(", user_flags = ${}", param_count));
            params.push(user_flags.to_string());
            param_count += 1;
        }

        if let Some(god_level) = request.god_level {
            query.push_str(&format!(", god_level = ${}", param_count));
            params.push(god_level.to_string());
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        // Build the actual query (simplified - in production would use query builder)
        let updated_user = sqlx::query_as::<_, UserAccount>(&query)
            .bind(user_id)
            .fetch_one(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to update user: {}", e))?;

        info!(
            "Updated user: {} ({})",
            updated_user.username, updated_user.id
        );
        Ok(updated_user)
    }

    /// Delete user account
    pub async fn delete_user(&self, user_id: Uuid) -> Result<bool> {
        warn!("Deleting user account: {}", user_id);

        let result = sqlx::query("DELETE FROM useraccounts WHERE principalid = $1")
            .bind(user_id)
            .execute(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to delete user: {}", e))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted user account: {}", user_id);
        }

        Ok(deleted)
    }

    /// Authenticate user with username/password
    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<UserAccount>> {
        debug!("Authenticating user: {}", username);

        let user = match self.get_user_by_username(username).await? {
            Some(user) => user,
            None => return Ok(None),
        };

        // Verify password
        if !verify_password(password, &user.password_hash, &user.salt)? {
            debug!("Password verification failed for user: {}", username);
            return Ok(None);
        }

        // Update last login time
        let _ = sqlx::query("UPDATE useraccounts SET last_login = NOW() WHERE principalid = $1")
            .bind(user.id)
            .execute(self.pool()?)
            .await;

        debug!("User authenticated successfully: {}", username);
        Ok(Some(user))
    }

    /// Authenticate user with first/last name for OpenSim compatibility
    pub async fn authenticate_user_opensim(
        &self,
        first_name: &str,
        last_name: &str,
        password: &str,
    ) -> Result<Option<UserAccount>> {
        debug!(
            "Authenticating OpenSim user: firstname='{}' lastname='{}' password_len={}",
            first_name,
            last_name,
            password.len()
        );

        // Query PostgreSQL useraccounts table directly
        debug!("Executing query: SELECT ua.principalid, ua.firstname, ua.lastname, ua.email, a.passwordhash, a.passwordsalt FROM useraccounts ua JOIN auth a ON ua.principalid = a.uuid WHERE LOWER(ua.firstname) = LOWER('{}') AND LOWER(ua.lastname) = LOWER('{}')", first_name, last_name);
        let user_row = sqlx::query(
            "SELECT ua.principalid, ua.firstname, ua.lastname, ua.email, 
                    a.passwordhash, a.passwordsalt
             FROM useraccounts ua 
             JOIN auth a ON ua.principalid = a.uuid 
             WHERE LOWER(ua.firstname) = LOWER($1) AND LOWER(ua.lastname) = LOWER($2)",
        )
        .bind(first_name)
        .bind(last_name)
        .fetch_optional(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to authenticate OpenSim user: {}", e))?;

        if let Some(row) = user_row {
            let stored_hash: String = row.try_get("passwordhash")?;
            let stored_salt: String = row.try_get("passwordsalt")?;

            // Proper MD5 password verification with $1$ prefix handling (Phase 59 spec)
            let viewer_hash = if password.starts_with("$1$") {
                password.strip_prefix("$1$").unwrap_or(password)
            } else {
                &password
            };

            // OpenSim auth: stored_hash = MD5(MD5(password) + ":" + salt)
            // Also support legacy accounts where stored_hash = MD5(password) directly
            let auth_success = if !stored_salt.is_empty() {
                let salted = format!("{}:{}", viewer_hash, stored_salt);
                let computed = format!("{:x}", md5::compute(salted.as_bytes()));
                if computed == stored_hash {
                    true
                } else {
                    viewer_hash == stored_hash
                }
            } else {
                viewer_hash == stored_hash
            };
            debug!(
                "Password verification for {} {}: salt_present={}, success={}",
                first_name,
                last_name,
                !stored_salt.is_empty(),
                auth_success
            );

            if auth_success {
                debug!(
                    "OpenSim authentication successful for: {} {}",
                    first_name, last_name
                );

                // Create UserAccount from database row
                let user_account = UserAccount {
                    id: row.try_get("principalid")?,
                    username: format!("{} {}", first_name, last_name),
                    email: row.try_get("email")?,
                    password_hash: stored_hash,
                    salt: stored_salt,
                    first_name: first_name.to_string(),
                    last_name: last_name.to_string(),
                    home_region_id: None,
                    home_position_x: 128.0,
                    home_position_y: 128.0,
                    home_position_z: 25.0,
                    home_look_at_x: 1.0,
                    home_look_at_y: 0.0,
                    home_look_at_z: 0.0,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    last_login: Some(chrono::Utc::now()),
                    user_level: 0,
                    user_flags: 0,
                    god_level: 0,
                    custom_type: "UserAccount".to_string(),
                };

                return Ok(Some(user_account));
            } else {
                debug!(
                    "Password verification failed for OpenSim user: {} {}",
                    first_name, last_name
                );
            }
        }

        Ok(None)
    }

    /// Create user session
    pub async fn create_session(
        &self,
        user_id: Uuid,
        region_id: Option<Uuid>,
    ) -> Result<UserSession> {
        debug!("Creating session for user: {}", user_id);

        let session_token = generate_session_token();
        let secure_session_token = generate_session_token();

        let session = sqlx::query_as::<_, UserSession>(
            r#"
            INSERT INTO user_sessions (
                user_id, session_token, secure_session_token, region_id,
                login_time, last_seen, is_active
            )
            VALUES ($1, $2, $3, $4, NOW(), NOW(), TRUE)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&session_token)
        .bind(&secure_session_token)
        .bind(region_id)
        .fetch_one(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to create session: {}", e))?;

        debug!(
            "Created session for user: {} (session: {})",
            user_id, session.id
        );
        Ok(session)
    }

    /// Get user profile
    pub async fn get_user_profile(&self, user_id: Uuid) -> Result<Option<UserProfile>> {
        debug!("Getting user profile: {}", user_id);

        let profile =
            sqlx::query_as::<_, UserProfile>("SELECT * FROM user_profiles WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(self.pool()?)
                .await
                .map_err(|e| anyhow!("Failed to get user profile: {}", e))?;

        Ok(profile)
    }

    /// Get user appearance
    pub async fn get_user_appearance(&self, user_id: Uuid) -> Result<Option<UserAppearance>> {
        debug!("Getting user appearance: {}", user_id);

        let appearance =
            sqlx::query_as::<_, UserAppearance>("SELECT * FROM user_appearance WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(self.pool()?)
                .await
                .map_err(|e| anyhow!("Failed to get user appearance: {}", e))?;

        Ok(appearance)
    }

    /// Get total user count
    pub async fn get_user_count(&self) -> Result<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM useraccounts")
            .fetch_one(self.pool()?)
            .await
            .map_err(|e| anyhow!("Failed to get user count: {}", e))?;

        let count: i64 = row.try_get("count")?;
        Ok(count as u64)
    }

    /// Create default user profile
    async fn create_user_profile(&self, user_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_profiles (user_id, created_at, updated_at)
            VALUES ($1, NOW(), NOW())
            "#,
        )
        .bind(user_id)
        .execute(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to create user profile: {}", e))?;

        Ok(())
    }

    /// Create default user appearance
    async fn create_user_appearance(&self, user_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_appearance (user_id, updated_at)
            VALUES ($1, NOW())
            "#,
        )
        .bind(user_id)
        .execute(self.pool()?)
        .await
        .map_err(|e| anyhow!("Failed to create user appearance: {}", e))?;

        Ok(())
    }
}

/// Generate a random salt for password hashing
fn generate_salt() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let salt: String = (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    salt
}

/// Hash a password with salt
fn hash_password(password: &str, salt: &str) -> Result<String> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

/// Verify a password against hash and salt
fn verify_password(password: &str, hash: &str, salt: &str) -> Result<bool> {
    let computed_hash = hash_password(password, salt)?;
    Ok(computed_hash == hash)
}

/// Generate a random session token
fn generate_session_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let token: String = (0..64)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password";
        let salt = "test_salt";

        let hash1 = hash_password(password, salt).unwrap();
        let hash2 = hash_password(password, salt).unwrap();

        // Same password and salt should produce same hash
        assert_eq!(hash1, hash2);

        // Verification should work
        assert!(verify_password(password, &hash1, salt).unwrap());
        assert!(!verify_password("wrong_password", &hash1, salt).unwrap());
    }

    #[test]
    fn test_session_token_generation() {
        let token1 = generate_session_token();
        let token2 = generate_session_token();

        // Tokens should be different
        assert_ne!(token1, token2);

        // Tokens should be the expected length
        assert_eq!(token1.len(), 64);
        assert_eq!(token2.len(), 64);
    }

    #[test]
    fn test_salt_generation() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        // Salts should be different
        assert_ne!(salt1, salt2);

        // Salts should be the expected length
        assert_eq!(salt1.len(), 32);
        assert_eq!(salt2.len(), 32);
    }
}
