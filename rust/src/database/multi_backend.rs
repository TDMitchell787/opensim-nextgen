//! Multi-database backend support for OpenSim Next
//!
//! Provides unified database interface supporting:
//! - PostgreSQL (production recommended)
//! - MySQL/MariaDB (legacy compatibility)

use std::time::Duration;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, MySql, Row, Executor, Database, Error as SqlxError};
use tracing::{info, warn, debug};
use std::future::Future;

/// Supported database types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MariaDB,  // Same as MySQL but with version detection
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PostgreSQL => write!(f, "PostgreSQL"),
            Self::MySQL => write!(f, "MySQL"),
            Self::MariaDB => write!(f, "MariaDB"),
        }
    }
}

impl DatabaseType {
    /// Detect database type from connection string
    pub fn from_url(url: &str) -> Result<Self> {
        if url.starts_with("postgresql://") || url.starts_with("postgres://") {
            Ok(Self::PostgreSQL)
        } else if url.starts_with("mysql://") {
            Ok(Self::MySQL)
        } else if url.starts_with("mariadb://") {
            Ok(Self::MariaDB)
        } else {
            Err(anyhow!("Unsupported database URL format: {}", url))
        }
    }

    /// Get default port for database type
    pub fn default_port(&self) -> u16 {
        match self {
            Self::PostgreSQL => 5432,
            Self::MySQL | Self::MariaDB => 3306,
        }
    }

    /// Get database-specific features
    pub fn features(&self) -> DatabaseFeatures {
        match self {
            Self::PostgreSQL => DatabaseFeatures {
                supports_json: true,
                supports_arrays: true,
                supports_uuids: true,
                supports_full_text_search: true,
                supports_concurrent_writes: true,
                max_connections_recommended: 100,
                supports_transactions: true,
                supports_foreign_keys: true,
            },
            Self::MySQL | Self::MariaDB => DatabaseFeatures {
                supports_json: true,
                supports_arrays: false,
                supports_uuids: false, // Need to use VARCHAR(36)
                supports_full_text_search: true,
                supports_concurrent_writes: true,
                max_connections_recommended: 80,
                supports_transactions: true,
                supports_foreign_keys: true,
            },
        }
    }
}

/// Database feature matrix
#[derive(Debug, Clone)]
pub struct DatabaseFeatures {
    pub supports_json: bool,
    pub supports_arrays: bool,
    pub supports_uuids: bool,
    pub supports_full_text_search: bool,
    pub supports_concurrent_writes: bool,
    pub max_connections_recommended: u32,
    pub supports_transactions: bool,
    pub supports_foreign_keys: bool,
}

/// Multi-database connection wrapper
#[derive(Debug)]
pub enum DatabaseConnection {
    PostgreSQL(Pool<Postgres>),
    MySQL(Pool<MySql>),
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows_affected: u64,
}

#[derive(Debug, Clone)]
pub enum QueryParam {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Uuid(uuid::Uuid),
    Bytes(Vec<u8>),
}

/// Reference to database pool for direct SQLx operations
pub enum DatabasePoolRef<'a> {
    PostgreSQL(&'a Pool<Postgres>),
    MySQL(&'a Pool<MySql>),
}

impl<'a> DatabasePoolRef<'a> {
    /// Begin a transaction on any database type
    pub async fn begin(&self) -> Result<DatabaseTransaction> {
        match self {
            Self::PostgreSQL(pool) => {
                let tx = pool.begin().await?;
                Ok(DatabaseTransaction::PostgreSQL(tx))
            },
            Self::MySQL(pool) => {
                let tx = pool.begin().await?;
                Ok(DatabaseTransaction::MySQL(tx))
            },
        }
    }

    /// Execute a query on the database pool - EADS fix for Executor trait issues
    pub async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        match self {
            Self::PostgreSQL(pool) => {
                let result = sqlx::query(query).execute(*pool).await?;
                Ok(QueryResult {
                    rows_affected: result.rows_affected(),
                })
            },
            Self::MySQL(pool) => {
                let result = sqlx::query(query).execute(*pool).await?;
                Ok(QueryResult {
                    rows_affected: result.rows_affected(),
                })
            },
        }
    }

    /// Execute a query with bindings on the database pool
    pub async fn execute_with_params(&self, query: &str, params: Vec<QueryParam>) -> Result<QueryResult> {
        match self {
            Self::PostgreSQL(pool) => {
                let mut q = sqlx::query(query);
                for param in params {
                    q = match param {
                        QueryParam::String(s) => q.bind(s),
                        QueryParam::Int(i) => q.bind(i),
                        QueryParam::Float(f) => q.bind(f),
                        QueryParam::Bool(b) => q.bind(b),
                        QueryParam::Uuid(u) => q.bind(u.to_string()),
                        QueryParam::Bytes(bytes) => q.bind(bytes),
                    };
                }
                let result = q.execute(*pool).await?;
                Ok(QueryResult {
                    rows_affected: result.rows_affected(),
                })
            },
            Self::MySQL(pool) => {
                let mut q = sqlx::query(query);
                for param in params {
                    q = match param {
                        QueryParam::String(s) => q.bind(s),
                        QueryParam::Int(i) => q.bind(i),
                        QueryParam::Float(f) => q.bind(f),
                        QueryParam::Bool(b) => q.bind(b),
                        QueryParam::Uuid(u) => q.bind(u.to_string()),
                        QueryParam::Bytes(bytes) => q.bind(bytes),
                    };
                }
                let result = q.execute(*pool).await?;
                Ok(QueryResult {
                    rows_affected: result.rows_affected(),
                })
            },
        }
    }

    /// Get PostgreSQL pool reference for direct SQLx operations
    pub fn as_postgres_pool(&self) -> Result<&Pool<Postgres>> {
        match self {
            Self::PostgreSQL(pool) => Ok(pool),
            _ => Err(anyhow::anyhow!("Database is not PostgreSQL")),
        }
    }

    /// Get MySQL pool reference for direct SQLx operations
    pub fn as_mysql_pool(&self) -> Result<&Pool<MySql>> {
        match self {
            Self::MySQL(pool) => Ok(pool),
            _ => Err(anyhow::anyhow!("Database is not MySQL")),
        }
    }
}

/// Multi-database transaction wrapper
#[derive(Debug)]
pub enum DatabaseTransaction {
    PostgreSQL(sqlx::Transaction<'static, Postgres>),
    MySQL(sqlx::Transaction<'static, MySql>),
}

impl DatabaseTransaction {
    /// Commit the transaction
    pub async fn commit(self) -> Result<()> {
        match self {
            Self::PostgreSQL(tx) => {
                tx.commit().await?;
                Ok(())
            },
            Self::MySQL(tx) => {
                tx.commit().await?;
                Ok(())
            },
        }
    }

    /// Get PostgreSQL transaction reference for direct operations
    pub fn as_postgres_tx(&mut self) -> Result<&mut sqlx::Transaction<'static, Postgres>> {
        match self {
            Self::PostgreSQL(tx) => Ok(tx),
            _ => Err(anyhow::anyhow!("Transaction is not PostgreSQL")),
        }
    }
}

/// Multi-database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiDatabaseConfig {
    pub database_type: DatabaseType,
    pub connection_string: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub enable_logging: bool,
    pub migration_table: String,
}

impl Default for MultiDatabaseConfig {
    fn default() -> Self {
        Self {
            database_type: DatabaseType::PostgreSQL,
            connection_string: "postgresql://opensim@localhost/opensim_pg".to_string(),
            max_connections: 20,
            min_connections: 2,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 1800,
            enable_logging: true,
            migration_table: "_opensim_migrations".to_string(),
        }
    }
}

impl MultiDatabaseConfig {
    /// Create configuration for PostgreSQL
    pub fn postgresql(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        Self {
            database_type: DatabaseType::PostgreSQL,
            connection_string: format!(
                "postgresql://{}:{}@{}:{}/{}",
                username, password, host, port, database
            ),
            max_connections: 100,
            min_connections: 5,
            ..Default::default()
        }
    }

    /// Create configuration for MySQL/MariaDB
    pub fn mysql(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        Self {
            database_type: DatabaseType::MySQL,
            connection_string: format!(
                "mysql://{}:{}@{}:{}/{}",
                username, password, host, port, database
            ),
            max_connections: 80,
            min_connections: 3,
            ..Default::default()
        }
    }


    /// Parse configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://opensim@localhost/opensim_pg".to_string());
        
        let database_type = DatabaseType::from_url(&database_url)?;
        
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| database_type.features().max_connections_recommended.to_string())
            .parse()
            .unwrap_or(database_type.features().max_connections_recommended);

        Ok(Self {
            database_type,
            connection_string: database_url,
            max_connections,
            enable_logging: std::env::var("DB_ENABLE_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            ..Default::default()
        })
    }
}

impl DatabaseConnection {
    /// Get PostgreSQL pool if available
    pub fn postgres_pool(&self) -> Option<&Pool<Postgres>> {
        match self {
            Self::PostgreSQL(pool) => Some(pool),
            _ => None,
        }
    }

    /// Get MySQL pool if available  
    pub fn mysql_pool(&self) -> Option<&Pool<MySql>> {
        match self {
            Self::MySQL(pool) => Some(pool),
            _ => None,
        }
    }


    /// Get pool reference for backwards compatibility with legacy code
    /// Returns a DatabasePoolRef that can be used with existing SQLx code
    pub fn pool(&self) -> DatabasePoolRef {
        match self {
            Self::PostgreSQL(pool) => DatabasePoolRef::PostgreSQL(pool),
            Self::MySQL(pool) => DatabasePoolRef::MySQL(pool),
        }
    }

    /// Get database pool - EADS fix for compilation errors
    pub fn get_pool(&self) -> Result<DatabasePoolRef> {
        Ok(self.pool())
    }

    /// Create new database connection based on configuration
    pub async fn new(config: &MultiDatabaseConfig) -> Result<Self> {
        info!("Connecting to {} database: {}", 
              format!("{:?}", config.database_type).to_lowercase(),
              Self::sanitize_url(&config.connection_string));

        match config.database_type {
            DatabaseType::PostgreSQL => {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .acquire_timeout(Duration::from_secs(config.acquire_timeout_seconds))
                    .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
                    .max_lifetime(Duration::from_secs(config.max_lifetime_seconds))
                    .connect(&config.connection_string)
                    .await?;
                
                Ok(Self::PostgreSQL(pool))
            },
            DatabaseType::MySQL | DatabaseType::MariaDB => {
                let pool = sqlx::mysql::MySqlPoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .acquire_timeout(Duration::from_secs(config.acquire_timeout_seconds))
                    .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
                    .max_lifetime(Duration::from_secs(config.max_lifetime_seconds))
                    .connect(&config.connection_string)
                    .await?;
                
                Ok(Self::MySQL(pool))
            },
        }
    }

    /// Get database type
    pub fn database_type(&self) -> DatabaseType {
        match self {
            Self::PostgreSQL(_) => DatabaseType::PostgreSQL,
            Self::MySQL(_) => DatabaseType::MySQL,
        }
    }

    /// Get database features
    pub fn features(&self) -> DatabaseFeatures {
        self.database_type().features()
    }

    /// Test database connection
    pub async fn test_connection(&self) -> Result<()> {
        match self {
            Self::PostgreSQL(pool) => {
                let row: (i64,) = sqlx::query_as("SELECT 1::bigint")
                    .fetch_one(pool)
                    .await?;
                if row.0 != 1 {
                    return Err(anyhow!("PostgreSQL connection test failed"));
                }
            },
            Self::MySQL(pool) => {
                let row: (i64,) = sqlx::query_as("SELECT 1")
                    .fetch_one(pool)
                    .await?;
                if row.0 != 1 {
                    return Err(anyhow!("MySQL connection test failed"));
                }
            },
        }
        Ok(())
    }

    /// Execute database migrations
    pub async fn migrate(&self) -> Result<()> {
        match self {
            Self::PostgreSQL(pool) => {
                info!("Running PostgreSQL migrations");
                sqlx::migrate!("./migrations/postgres").run(pool).await?;
            },
            Self::MySQL(pool) => {
                info!("Running MySQL migrations");
                sqlx::migrate!("./migrations/mysql").run(pool).await?;
            },
        }
        Ok(())
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> DatabaseStats {
        match self {
            Self::PostgreSQL(pool) => DatabaseStats {
                database_type: DatabaseType::PostgreSQL,
                total_connections: pool.size(),
                active_connections: pool.size().saturating_sub(pool.num_idle() as u32),
                idle_connections: pool.num_idle() as u32,
                is_closed: pool.is_closed(),
            },
            Self::MySQL(pool) => DatabaseStats {
                database_type: DatabaseType::MySQL,
                total_connections: pool.size(),
                active_connections: pool.size().saturating_sub(pool.num_idle() as u32),
                idle_connections: pool.num_idle() as u32,
                is_closed: pool.is_closed(),
            },
        }
    }

    /// Begin a transaction
    pub async fn begin(&self) -> Result<DatabaseTransaction> {
        match self {
            Self::PostgreSQL(pool) => {
                let tx = pool.begin().await?;
                Ok(DatabaseTransaction::PostgreSQL(tx))
            },
            Self::MySQL(pool) => {
                let tx = pool.begin().await?;
                Ok(DatabaseTransaction::MySQL(tx))
            },
        }
    }

    /// Begin a transaction (alias for compatibility)
    pub async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        self.begin().await
    }

    /// Close all connections
    pub async fn close(&self) {
        match self {
            Self::PostgreSQL(pool) => pool.close().await,
            Self::MySQL(pool) => pool.close().await,
        }
    }

    pub async fn seed_default_assets(&self) -> Result<()> {
        use uuid::Uuid;

        info!("Checking default Ruth avatar assets...");

        let creator = Uuid::parse_str("11111111-1111-0000-0000-000100bba000").unwrap();

        struct AssetEntry {
            id: Uuid,
            name: &'static str,
            description: &'static str,
            asset_type: i64,
            data: Vec<u8>,
        }

        let assets = vec![
            AssetEntry { id: Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfab73").unwrap(), name: "Default Shape", description: "Default avatar shape", asset_type: 13, data: super::initialization::create_ruth_shape_data() },
            AssetEntry { id: Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bbabbb").unwrap(), name: "Default Skin", description: "Default avatar skin", asset_type: 13, data: super::initialization::create_ruth_skin_data() },
            AssetEntry { id: Uuid::parse_str("d342e6c0-b9d2-11dc-95ff-0800200c9a66").unwrap(), name: "Default Hair", description: "Default avatar hair", asset_type: 13, data: super::initialization::create_ruth_hair_data() },
            AssetEntry { id: Uuid::parse_str("4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7").unwrap(), name: "Default Eyes", description: "Default avatar eyes", asset_type: 13, data: super::initialization::create_ruth_eyes_data() },
            AssetEntry { id: Uuid::parse_str("00000000-38f9-1111-024e-222222111110").unwrap(), name: "Default Shirt", description: "Default avatar shirt", asset_type: 5, data: super::initialization::create_ruth_shirt_data() },
            AssetEntry { id: Uuid::parse_str("00000000-38f9-1111-024e-222222111120").unwrap(), name: "Default Pants", description: "Default avatar pants", asset_type: 5, data: super::initialization::create_ruth_pants_data() },
            AssetEntry { id: Uuid::parse_str("5748decc-f629-461c-9a36-a35a221fe21f").unwrap(), name: "Default Clothing Texture", description: "White clothing texture", asset_type: 0, data: super::initialization::create_default_j2k_texture() },
            AssetEntry { id: Uuid::parse_str("7ca39b4c-bd19-4699-aff7-f93fd03d3e7b").unwrap(), name: "Default Hair Texture", description: "Brown hair texture", asset_type: 0, data: super::initialization::create_default_j2k_texture() },
            AssetEntry { id: Uuid::parse_str("6522e74d-1660-4e7f-b601-6f48c1659a77").unwrap(), name: "Default Eyes Texture", description: "Brown eyes texture", asset_type: 0, data: super::initialization::create_default_j2k_texture() },
            AssetEntry { id: Uuid::parse_str("00000000-0000-1111-9999-000000000012").unwrap(), name: "Default Skin Head Texture", description: "Default skin head", asset_type: 0, data: super::initialization::create_default_j2k_texture() },
            AssetEntry { id: Uuid::parse_str("00000000-0000-1111-9999-000000000010").unwrap(), name: "Default Skin Upper Texture", description: "Default skin upper body", asset_type: 0, data: super::initialization::create_default_j2k_texture() },
            AssetEntry { id: Uuid::parse_str("00000000-0000-1111-9999-000000000011").unwrap(), name: "Default Skin Lower Texture", description: "Default skin lower body", asset_type: 0, data: super::initialization::create_default_j2k_texture() },
            AssetEntry { id: Uuid::parse_str("5a9f4a74-30f2-821c-b88d-70499d3e7183").unwrap(), name: "Baked Head Texture", description: "Ruth baked head texture for AvatarAppearance", asset_type: 0, data: super::initialization::load_ruth_head_texture() },
            AssetEntry { id: Uuid::parse_str("ae2de45c-d252-50b8-5c6e-19f39ce79317").unwrap(), name: "Baked Upper Body Texture", description: "Ruth baked upper body texture for AvatarAppearance", asset_type: 0, data: super::initialization::load_ruth_upper_texture() },
            AssetEntry { id: Uuid::parse_str("24daea5f-0539-cfcf-047f-fbc40b2786ba").unwrap(), name: "Baked Lower Body Texture", description: "Ruth baked lower body texture for AvatarAppearance", asset_type: 0, data: super::initialization::load_ruth_lower_texture() },
            AssetEntry { id: Uuid::parse_str("52cc6bb6-2ee5-e632-d3ad-50197b1dcb8a").unwrap(), name: "Baked Eyes Texture", description: "Ruth baked eyes texture for AvatarAppearance", asset_type: 0, data: super::initialization::load_ruth_eyes_texture() },
            AssetEntry { id: Uuid::parse_str("09aac1fb-6bce-0bee-7d44-caac6dbb6c63").unwrap(), name: "Baked Hair Texture", description: "Ruth baked hair texture for AvatarAppearance", asset_type: 0, data: super::initialization::load_ruth_hair_texture() },
            AssetEntry { id: Uuid::parse_str("c228d1cf-4b5d-4ba8-84f4-899a0796aa97").unwrap(), name: "Default Skin Texture", description: "Default skin texture (base)", asset_type: 0, data: super::initialization::create_default_j2k_texture() },
        ];

        let mut seeded = 0;
        match self {
            Self::PostgreSQL(pool) => {
                for asset in &assets {
                    let exists: Option<i64> = sqlx::query_scalar(
                        "SELECT 1::bigint FROM assets WHERE id = $1"
                    )
                    .bind(asset.id)
                    .fetch_optional(pool)
                    .await?;

                    if exists.is_none() {
                        sqlx::query(
                            "INSERT INTO assets (id, name, description, assetType, local, temporary, data, create_time, access_time, asset_flags, CreatorID) VALUES ($1, $2, $3, $4, 1, 0, $5, EXTRACT(EPOCH FROM NOW())::bigint, EXTRACT(EPOCH FROM NOW())::bigint, 0, $6)"
                        )
                        .bind(asset.id)
                        .bind(asset.name)
                        .bind(asset.description)
                        .bind(asset.asset_type)
                        .bind(&asset.data)
                        .bind(creator.to_string())
                        .execute(pool)
                        .await?;
                        seeded += 1;
                    }
                }
            }
            Self::MySQL(pool) => {
                for asset in &assets {
                    let exists: Option<i64> = sqlx::query_scalar(
                        "SELECT 1 FROM assets WHERE id = ?"
                    )
                    .bind(asset.id.to_string())
                    .fetch_optional(pool)
                    .await?;

                    if exists.is_none() {
                        sqlx::query(
                            "INSERT INTO assets (id, name, description, assetType, local, temporary, data, create_time, access_time, asset_flags, CreatorID) VALUES (?, ?, ?, ?, 1, 0, ?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP(), 0, ?)"
                        )
                        .bind(asset.id.to_string())
                        .bind(asset.name)
                        .bind(asset.description)
                        .bind(asset.asset_type)
                        .bind(&asset.data)
                        .bind(creator.to_string())
                        .execute(pool)
                        .await?;
                        seeded += 1;
                    }
                }
            }
        }

        if seeded > 0 {
            info!("Seeded {} default Ruth avatar assets (including baked textures) into database", seeded);
        } else {
            info!("All default Ruth avatar assets (including baked textures) already present");
        }

        Ok(())
    }

    /// Sanitize URL for logging (remove password)
    fn sanitize_url(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            let mut sanitized = parsed.clone();
            if parsed.password().is_some() {
                let _ = sanitized.set_password(Some("***"));
            }
            sanitized.to_string()
        } else {
            url.to_string()
        }
    }
}

/// Database connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub database_type: DatabaseType,
    pub total_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub is_closed: bool,
}

/// Database health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseHealth {
    Healthy,
    Warning(String),
    Critical(String),
    Disconnected,
}

impl DatabaseConnection {
    /// Check database health
    pub async fn health_check(&self) -> DatabaseHealth {
        match self.test_connection().await {
            Ok(_) => {
                let stats = self.get_stats().await;
                if stats.is_closed {
                    DatabaseHealth::Disconnected
                } else if stats.active_connections as f32 / stats.total_connections as f32 > 0.9 {
                    DatabaseHealth::Warning("High connection usage".to_string())
                } else {
                    DatabaseHealth::Healthy
                }
            },
            Err(e) => DatabaseHealth::Critical(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_detection() {
        assert_eq!(DatabaseType::from_url("postgresql://localhost/test").unwrap(), DatabaseType::PostgreSQL);
        assert_eq!(DatabaseType::from_url("postgres://localhost/test").unwrap(), DatabaseType::PostgreSQL);
        assert_eq!(DatabaseType::from_url("mysql://localhost/test").unwrap(), DatabaseType::MySQL);
        assert_eq!(DatabaseType::from_url("mariadb://localhost/test").unwrap(), DatabaseType::MariaDB);
        
        assert!(DatabaseType::from_url("unsupported://localhost/test").is_err());
    }

    #[test]
    fn test_database_config_creation() {
        let pg_config = MultiDatabaseConfig::postgresql("localhost", 5432, "opensim", "user", "pass");
        assert_eq!(pg_config.database_type, DatabaseType::PostgreSQL);
        assert!(pg_config.connection_string.contains("postgresql://"));

        let mysql_config = MultiDatabaseConfig::mysql("localhost", 3306, "opensim", "user", "pass");
        assert_eq!(mysql_config.database_type, DatabaseType::MySQL);
        assert!(mysql_config.connection_string.contains("mysql://"));

    }

    #[test]
    fn test_url_sanitization() {
        let url = "postgresql://user:password@localhost:5432/opensim";
        let sanitized = DatabaseConnection::sanitize_url(url);
        assert!(sanitized.contains("user:***@"));
        assert!(!sanitized.contains("password"));
    }
}