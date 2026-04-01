//! Local User Account Service Implementation
//!
//! Provides direct database access for user account operations.
//! Used in standalone mode with PostgreSQL backend.
//!
//! Reference: OpenSim/Services/UserAccountService/UserAccountService.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::services::traits::{UserAccountServiceTrait, UserAccount};
use crate::database::multi_backend::DatabaseConnection;

pub struct LocalUserAccountService {
    connection: Arc<DatabaseConnection>,
}

impl LocalUserAccountService {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        info!("Initializing local user account service");
        Self { connection }
    }

    fn get_pg_pool(&self) -> Result<&sqlx::PgPool> {
        match self.connection.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => Ok(pool),
            _ => Err(anyhow!("LocalUserAccountService requires PostgreSQL connection")),
        }
    }
}

#[async_trait]
impl UserAccountServiceTrait for LocalUserAccountService {
    async fn get_user_account(&self, scope_id: Uuid, user_id: Uuid) -> Result<Option<UserAccount>> {
        debug!("Getting user account by ID: {} (scope: {})", user_id, scope_id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            r#"
            SELECT principalid, scopeid, firstname, lastname, email,
                   serviceurls, created, userlevel, userflags, usertitle
            FROM useraccounts
            WHERE principalid = $1
            AND (scopeid = $2 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $2 = '00000000-0000-0000-0000-000000000000')
            "#
        )
        .bind(user_id)
        .bind(scope_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get user account: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_user_account(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_user_account_by_name(&self, scope_id: Uuid, first: &str, last: &str) -> Result<Option<UserAccount>> {
        debug!("Getting user account by name: {} {} (scope: {})", first, last, scope_id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            r#"
            SELECT principalid, scopeid, firstname, lastname, email,
                   serviceurls, created, userlevel, userflags, usertitle
            FROM useraccounts
            WHERE LOWER(firstname) = LOWER($1) AND LOWER(lastname) = LOWER($2)
            AND (scopeid = $3 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $3 = '00000000-0000-0000-0000-000000000000')
            "#
        )
        .bind(first)
        .bind(last)
        .bind(scope_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get user account by name: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_user_account(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn get_user_account_by_email(&self, scope_id: Uuid, email: &str) -> Result<Option<UserAccount>> {
        debug!("Getting user account by email: {} (scope: {})", email, scope_id);

        let pool = self.get_pg_pool()?;
        let row = sqlx::query(
            r#"
            SELECT principalid, scopeid, firstname, lastname, email,
                   serviceurls, created, userlevel, userflags, usertitle
            FROM useraccounts
            WHERE email = $1
            AND (scopeid = $2 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $2 = '00000000-0000-0000-0000-000000000000')
            "#
        )
        .bind(email)
        .bind(scope_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to get user account by email: {}", e))?;

        if let Some(row) = row {
            Ok(Some(self.row_to_user_account(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn store_user_account(&self, data: &UserAccount) -> Result<bool> {
        info!("Storing user account: {} {} ({})", data.first_name, data.last_name, data.principal_id);

        let service_urls = self.serialize_service_urls(&data.service_urls);

        let pool = self.get_pg_pool()?;
        sqlx::query(
            r#"
            INSERT INTO useraccounts (
                principalid, scopeid, firstname, lastname, email,
                serviceurls, created, userlevel, userflags, usertitle
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT(principalid) DO UPDATE SET
                firstname = EXCLUDED.firstname,
                lastname = EXCLUDED.lastname,
                email = EXCLUDED.email,
                serviceurls = EXCLUDED.serviceurls,
                userlevel = EXCLUDED.userlevel,
                userflags = EXCLUDED.userflags,
                usertitle = EXCLUDED.usertitle
            "#
        )
        .bind(data.principal_id)
        .bind(data.scope_id)
        .bind(&data.first_name)
        .bind(&data.last_name)
        .bind(&data.email)
        .bind(&service_urls)
        .bind(data.created)
        .bind(data.user_level)
        .bind(data.user_flags)
        .bind(&data.user_title)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to store user account: {}", e))?;

        info!("Stored user account: {} {}", data.first_name, data.last_name);
        Ok(true)
    }

    async fn get_user_accounts(&self, scope_id: Uuid, query: &str) -> Result<Vec<UserAccount>> {
        debug!("Searching user accounts: {} (scope: {})", query, scope_id);

        let search_pattern = format!("%{}%", query);

        let pool = self.get_pg_pool()?;
        let rows = sqlx::query(
            r#"
            SELECT principalid, scopeid, firstname, lastname, email,
                   serviceurls, created, userlevel, userflags, usertitle
            FROM useraccounts
            WHERE (firstname ILIKE $1 OR lastname ILIKE $1 OR email ILIKE $1)
            AND (scopeid = $2 OR scopeid = '00000000-0000-0000-0000-000000000000' OR $2 = '00000000-0000-0000-0000-000000000000')
            ORDER BY firstname, lastname
            LIMIT 100
            "#
        )
        .bind(&search_pattern)
        .bind(scope_id)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to search user accounts: {}", e))?;

        let mut accounts = Vec::new();
        for row in rows {
            accounts.push(self.row_to_user_account(&row)?);
        }
        Ok(accounts)
    }
}

impl LocalUserAccountService {
    fn row_to_user_account(&self, row: &sqlx::postgres::PgRow) -> Result<UserAccount> {
        use sqlx::Row;

        let service_urls_str: String = row.try_get("serviceurls").unwrap_or_default();

        Ok(UserAccount {
            principal_id: row.try_get("principalid")?,
            scope_id: row.try_get("scopeid")?,
            first_name: row.try_get("firstname")?,
            last_name: row.try_get("lastname")?,
            email: row.try_get("email").unwrap_or_default(),
            service_urls: self.parse_service_urls(&service_urls_str),
            created: row.try_get("created")?,
            user_level: row.try_get("userlevel")?,
            user_flags: row.try_get("userflags")?,
            user_title: row.try_get("usertitle").unwrap_or_default(),
        })
    }

    fn parse_service_urls(&self, data: &str) -> HashMap<String, String> {
        let mut urls = HashMap::new();
        if data.is_empty() {
            return urls;
        }

        for pair in data.split(';') {
            if let Some((key, value)) = pair.split_once('=') {
                urls.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        urls
    }

    fn serialize_service_urls(&self, urls: &HashMap<String, String>) -> String {
        urls.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(";")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_account_default() {
        let account = UserAccount::default();
        assert!(account.principal_id.is_nil());
        assert!(account.first_name.is_empty());
        assert!(account.last_name.is_empty());
    }
}
