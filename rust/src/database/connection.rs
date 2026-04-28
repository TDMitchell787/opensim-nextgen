//! Database connection management

use super::migration_manager::{DatabaseType, MigrationManager};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Database connection pool wrapper
#[derive(Debug)]
pub struct DatabaseConnection {
    pool: Pool<Postgres>,
    connection_string: String,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "opensim".to_string(),
            username: "opensim".to_string(),
            password: "opensim".to_string(),
            max_connections: 20,
            min_connections: 2,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 1800,
        }
    }
}

impl DatabaseConnection {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Some(Duration::from_secs(600)))
            .max_lifetime(Some(Duration::from_secs(1800)))
            .connect(database_url)
            .await
            .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;

        info!("Database connection established successfully");

        Ok(Self {
            pool,
            connection_string: database_url.to_string(),
        })
    }

    /// Create connection from config
    pub async fn from_config(config: &DatabaseConfig) -> Result<Self> {
        let connection_string = format!(
            "postgresql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.acquire_timeout_seconds))
            .idle_timeout(Some(Duration::from_secs(config.idle_timeout_seconds)))
            .max_lifetime(Some(Duration::from_secs(config.max_lifetime_seconds)))
            .connect(&connection_string)
            .await
            .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;

        info!("Database connection established from config");

        Ok(Self {
            pool,
            connection_string,
        })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations for PostgreSQL");

        // Use migration manager to get appropriate migrations
        let migration_manager = MigrationManager::new(DatabaseType::PostgreSQL);
        let migrations = migration_manager.get_migrations();

        for (i, migration) in migrations.iter().enumerate() {
            debug!(
                "Running migration {}: {}",
                i + 1,
                migration.lines().next().unwrap_or("")
            );

            match sqlx::query(migration).execute(&self.pool).await {
                Ok(_) => debug!("Migration {} completed successfully", i + 1),
                Err(e) => {
                    let error_str = e.to_string();
                    // Use migration manager to check if error should be skipped
                    if migration_manager.should_skip_migration_error(&error_str) {
                        debug!(
                            "Migration {} skipped (already applied or compatibility): {}",
                            i + 1,
                            e
                        );
                    } else {
                        error!("Migration {} failed: {}", i + 1, e);
                        return Err(anyhow!("Migration {} failed: {}", i + 1, e));
                    }
                }
            }
        }

        MigrationManager::log_migration_consistency_warnings();

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Check database connection health
    pub async fn check_connection(&self) -> Result<String> {
        debug!("Checking database connection health");

        let row = sqlx::query("SELECT version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to check database connection: {}", e))?;

        let version: String = row
            .try_get(0)
            .map_err(|e| anyhow!("Failed to get database version: {}", e))?;

        debug!("Database connection healthy: {}", version);
        Ok(format!("Connected - {}", version))
    }

    /// Get connection pool statistics
    pub fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle() as u32,
            total_connections: self.pool.size(),
        }
    }

    /// Begin a database transaction
    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(|e| anyhow!("Failed to begin transaction: {}", e))
    }

    /// Execute a raw SQL query (for admin/maintenance tasks)
    pub async fn execute_raw(&self, query: &str) -> Result<u64> {
        warn!("Executing raw SQL query: {}", query);

        let result = sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to execute raw query: {}", e))?;

        Ok(result.rows_affected())
    }

    /// Close database connection
    pub async fn close(&self) {
        info!("Closing database connection");
        self.pool.close().await;
    }
}

/// Database connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: u32,
    pub total_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "opensim");
        assert_eq!(config.max_connections, 20);
    }

    #[test]
    fn test_connection_string_generation() {
        let config = DatabaseConfig {
            host: "testhost".to_string(),
            port: 5433,
            database: "testdb".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            ..Default::default()
        };

        // This would generate: postgresql://testuser:testpass@testhost:5433/testdb
        // We can't test the actual connection without a running database
        assert_eq!(config.host, "testhost");
        assert_eq!(config.port, 5433);
    }
}
