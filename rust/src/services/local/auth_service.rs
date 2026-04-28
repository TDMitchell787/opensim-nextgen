//! Local Authentication Service Implementation
//!
//! Provides direct database access for authentication operations.
//! Used in standalone mode with PostgreSQL backend.
//!
//! Reference: OpenSim/Services/AuthenticationService/PasswordAuthenticationService.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::database::multi_backend::DatabaseConnection;
use crate::services::traits::{AuthInfo, AuthenticationServiceTrait};

pub struct LocalAuthenticationService {
    connection: Arc<DatabaseConnection>,
}

impl LocalAuthenticationService {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        info!("Initializing local authentication service");
        Self { connection }
    }

    fn get_pg_pool(&self) -> Result<&sqlx::PgPool> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => Ok(pool),
            _ => Err(anyhow!(
                "LocalAuthenticationService requires PostgreSQL connection"
            )),
        }
    }

    fn hash_password(&self, password: &str, salt: &str) -> String {
        let salted = format!("{}:{}", password, salt);
        let mut hasher = Sha256::new();
        hasher.update(salted.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    fn generate_salt(&self) -> String {
        Uuid::new_v4().to_string()
    }

    fn generate_token(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

#[async_trait]
impl AuthenticationServiceTrait for LocalAuthenticationService {
    async fn authenticate(
        &self,
        principal_id: Uuid,
        password: &str,
        _lifetime: i32,
    ) -> Result<Option<String>> {
        debug!("Authenticating user: {}", principal_id);

        let pool = self.get_pg_pool()?;
        let row: Option<(String, String)> =
            sqlx::query_as("SELECT passwordhash, passwordsalt FROM auth WHERE uuid = $1")
                .bind(principal_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| anyhow!("Failed to get auth info: {}", e))?;

        if let Some((stored_hash, salt)) = row {
            let computed_hash = self.hash_password(password, &salt);

            if computed_hash == stored_hash {
                let token = self.generate_token();
                info!("Authentication successful for user: {}", principal_id);
                Ok(Some(token))
            } else {
                warn!(
                    "Authentication failed for user: {} (password mismatch)",
                    principal_id
                );
                Ok(None)
            }
        } else {
            warn!(
                "Authentication failed for user: {} (not found)",
                principal_id
            );
            Ok(None)
        }
    }

    async fn verify(&self, _principal_id: Uuid, token: &str, _lifetime: i32) -> Result<bool> {
        debug!("Verifying token");
        Ok(!token.is_empty())
    }

    async fn release(&self, _principal_id: Uuid, _token: &str) -> Result<bool> {
        debug!("Releasing authentication token");
        Ok(true)
    }

    async fn set_password(&self, principal_id: Uuid, password: &str) -> Result<bool> {
        info!("Setting password for user: {}", principal_id);

        let salt = self.generate_salt();
        let hash = self.hash_password(password, &salt);

        let pool = self.get_pg_pool()?;
        sqlx::query(
            r#"
            INSERT INTO auth (uuid, passwordhash, passwordsalt, webloginkey, accounttype)
            VALUES ($1, $2, $3, '', 'UserAccount')
            ON CONFLICT(uuid) DO UPDATE SET
                passwordhash = EXCLUDED.passwordhash,
                passwordsalt = EXCLUDED.passwordsalt
            "#,
        )
        .bind(principal_id)
        .bind(&hash)
        .bind(&salt)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to set password: {}", e))?;

        info!("Password set for user: {}", principal_id);
        Ok(true)
    }

    async fn get_authentication(&self, principal_id: Uuid) -> Result<Option<AuthInfo>> {
        debug!("Getting authentication info for: {}", principal_id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            "SELECT uuid, passwordhash, passwordsalt, webloginkey, accounttype FROM auth WHERE uuid = $1"
        )
        .bind(principal_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get auth info: {}", e))?;

        if let Some(row) = row {
            use sqlx::Row;
            Ok(Some(AuthInfo {
                principal_id: row.try_get("uuid")?,
                password_hash: row.try_get("passwordhash")?,
                password_salt: row.try_get("passwordsalt")?,
                web_login_key: row.try_get("webloginkey").unwrap_or_default(),
                account_type: row
                    .try_get("accounttype")
                    .unwrap_or_else(|_| "UserAccount".to_string()),
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let service = LocalAuthenticationService {
            connection: Arc::new(DatabaseConnection::PostgreSQL(
                sqlx::postgres::PgPoolOptions::new()
                    .connect_lazy("postgres://localhost/test")
                    .unwrap(),
            )),
        };

        let hash1 = service.hash_password("password", "salt");
        let hash2 = service.hash_password("password", "salt");
        assert_eq!(hash1, hash2);

        let hash3 = service.hash_password("password", "different_salt");
        assert_ne!(hash1, hash3);
    }
}
