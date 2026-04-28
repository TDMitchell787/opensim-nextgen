//! Database persistence layer for OpenSim

pub mod admin_operations;
pub mod connection;
pub mod default_inventory;
pub mod initialization;
pub mod inventory;
pub mod migration_engine;
pub mod migration_manager;
pub mod multi_backend;
pub mod region_data;
pub mod sqlite_admin;
pub mod user_accounts;

use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info, warn};

// Re-export commonly used types
pub use self::admin_operations::{AdminOperationResult, CreateUserRequest, DatabaseAdmin};
pub use self::connection::DatabaseConnection as LegacyDatabaseConnection;
pub use self::initialization::DatabaseInitializer;
pub use self::inventory::InventoryDatabase;
pub use self::multi_backend::{
    DatabaseConnection, DatabaseFeatures, DatabaseHealth, DatabasePoolRef, DatabaseStats,
    DatabaseTransaction, DatabaseType, MultiDatabaseConfig,
};
pub use self::region_data::RegionDatabase;
pub use self::user_accounts::UserAccountDatabase;

/// Main database manager coordinating all persistence systems
#[derive(Debug)]
pub struct DatabaseManager {
    connection: Arc<DatabaseConnection>,
    legacy_connection: Option<Arc<LegacyDatabaseConnection>>,
    user_accounts: Arc<UserAccountDatabase>,
    region_data: Arc<RegionDatabase>,
    inventory: Arc<InventoryDatabase>,
    admin: Arc<DatabaseAdmin>,
}

impl DatabaseManager {
    /// Create a new database manager with an existing connection
    pub async fn with_connection(connection: Arc<DatabaseConnection>) -> Result<Self> {
        info!("Initializing database manager with existing connection");

        // Skip legacy connection creation since we already have a working connection
        let legacy_connection = None;

        Self::initialize_with_connections(connection, legacy_connection).await
    }

    /// Create a new database manager
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Initializing database manager with URL: {}", database_url);

        // Create multi-database config from URL
        let db_config = MultiDatabaseConfig {
            database_type: DatabaseType::from_url(database_url)?,
            connection_string: database_url.to_string(),
            ..Default::default()
        };

        let connection = Arc::new(DatabaseConnection::new(&db_config).await?);

        let skip_migrations = std::env::var("OPENSIM_SERVICE_MODE").unwrap_or_default() == "grid";
        if skip_migrations {
            info!("Grid mode: skipping migrations (Robust server handles them)");
        } else {
            connection.migrate().await?;
        }

        // For now, also create legacy connection for existing code compatibility
        let legacy_connection = match LegacyDatabaseConnection::new(database_url).await {
            Ok(conn) => Some(Arc::new(conn)),
            Err(e) => {
                warn!(
                    "Failed to create legacy connection: {}. Some features may not work.",
                    e
                );
                None
            }
        };

        Self::initialize_with_connections(connection, legacy_connection).await
    }

    /// Internal helper to initialize with given connections
    async fn initialize_with_connections(
        connection: Arc<DatabaseConnection>,
        legacy_connection: Option<Arc<LegacyDatabaseConnection>>,
    ) -> Result<Self> {
        info!("Creating multi-backend database subsystems");
        let (user_accounts, region_data, inventory) = match connection.as_ref() {
            DatabaseConnection::PostgreSQL(_pool) => {
                info!("Creating PostgreSQL subsystems with main database connection");
                let user_accounts = Arc::new(UserAccountDatabase::new(connection.clone()).await?);
                let region_data = Arc::new(RegionDatabase::new(connection.clone()).await?);
                let inventory = Arc::new(InventoryDatabase::new(connection.clone()).await?);
                (user_accounts, region_data, inventory)
            }
            DatabaseConnection::MySQL(_pool) => {
                info!("Creating MariaDB/MySQL subsystems with main database connection");
                let user_accounts = Arc::new(UserAccountDatabase::new(connection.clone()).await?);
                let region_data = Arc::new(RegionDatabase::new(connection.clone()).await?);
                let inventory = Arc::new(InventoryDatabase::new(connection.clone()).await?);
                (user_accounts, region_data, inventory)
            }
        };

        // Create admin interface using the database connection pool
        // ELEGANT SOLUTION: Use the same connection pool as the main database
        let admin = if let Some(ref legacy_conn) = legacy_connection {
            Arc::new(DatabaseAdmin::new(legacy_conn.pool().clone()))
        } else {
            // For multi-backend mode, create admin that works with the current connection
            info!("Creating multi-backend admin implementation");
            match connection.as_ref() {
                DatabaseConnection::PostgreSQL(pool) => Arc::new(DatabaseAdmin::new(pool.clone())),
                DatabaseConnection::MySQL(_pool) => {
                    // MySQL pools can't be used with PostgreSQL admin, so use stub for now
                    warn!("MySQL admin not fully implemented yet, using stub");
                    Arc::new(DatabaseAdmin::new_stub())
                }
            }
        };

        info!("Database manager initialized successfully");

        Ok(Self {
            connection,
            legacy_connection,
            user_accounts,
            region_data,
            inventory,
            admin,
        })
    }

    /// Get user account database
    pub fn user_accounts(&self) -> Arc<UserAccountDatabase> {
        self.user_accounts.clone()
    }

    /// Get region data database
    pub fn region_data(&self) -> Arc<RegionDatabase> {
        self.region_data.clone()
    }

    /// Get inventory database
    pub fn inventory(&self) -> Arc<InventoryDatabase> {
        self.inventory.clone()
    }

    /// Get database connection
    pub fn connection(&self) -> Arc<DatabaseConnection> {
        self.connection.clone()
    }

    /// Get admin operations interface
    pub fn admin(&self) -> Arc<DatabaseAdmin> {
        self.admin.clone()
    }

    /// Perform database health check
    pub async fn health_check(&self) -> Result<DatabaseHealth> {
        match self.connection.health_check().await {
            DatabaseHealth::Healthy => Ok(DatabaseHealth::Healthy),
            DatabaseHealth::Warning(msg) => Ok(DatabaseHealth::Warning(msg)),
            DatabaseHealth::Critical(msg) => Ok(DatabaseHealth::Critical(msg)),
            DatabaseHealth::Disconnected => Ok(DatabaseHealth::Disconnected),
        }
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> DatabaseStats {
        self.connection.get_stats().await
    }

    /// Begin a database transaction
    pub async fn begin(&self) -> Result<DatabaseTransaction> {
        self.connection.begin().await
    }

    /// Get database pool - EADS fix for compilation errors
    pub fn get_pool(&self) -> Result<DatabasePoolRef> {
        self.connection.get_pool()
    }

    /// Get legacy database pool for Phase 25 compatibility
    /// TODO: Remove this once multi-backend system is fully implemented
    pub fn legacy_pool(&self) -> Result<&sqlx::Pool<sqlx::Postgres>> {
        match &*self.connection {
            DatabaseConnection::PostgreSQL(pool) => Ok(pool),
            _ => Err(anyhow::anyhow!(
                "Legacy pool only supports PostgreSQL connections"
            )),
        }
    }

    /// Shutdown database connections gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down database manager");
        self.connection.close().await;
        info!("Database manager shutdown complete");
        Ok(())
    }
}
