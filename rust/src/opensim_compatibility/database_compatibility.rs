//! OpenSim database schema compatibility
//!
//! Provides compatibility with existing OpenSimulator database schemas
//! including migration utilities and schema detection.

use anyhow::Result;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// Database compatibility manager
pub struct DatabaseCompatibilityManager {
    pool: PgPool,
    detected_schema: Option<OpenSimSchema>,
    migration_state: MigrationState,
}

/// OpenSim database schema information
#[derive(Debug, Clone)]
pub struct OpenSimSchema {
    pub version: String,
    pub detected_tables: Vec<String>,
    pub missing_tables: Vec<String>,
    pub schema_type: SchemaType,
    pub needs_migration: bool,
}

/// Type of OpenSim database schema
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    OpenSimStandalone,
    OpenSimGrid,
    RobustServices,
    OpenSimNext,
    Unknown,
}

/// Migration state tracking
#[derive(Debug, Clone)]
pub struct MigrationState {
    pub current_version: Option<String>,
    pub target_version: String,
    pub completed_migrations: Vec<String>,
    pub pending_migrations: Vec<String>,
    pub failed_migrations: Vec<String>,
}

/// Required OpenSim tables for compatibility
const OPENSIM_CORE_TABLES: &[&str] = &[
    "users",
    "agents",
    "avatars",
    "inventoryfolders",
    "inventoryitems",
    "assets",
    "regions",
    "regionsettings",
    "land",
    "landaccesslist",
    "primitems",
    "prims",
    "primshapes",
    "terrain",
];

/// ROBUST service tables
const ROBUST_TABLES: &[&str] = &[
    "UserAccounts",
    "GridUser",
    "Presence",
    "Friends",
    "InventoryFolders",
    "InventoryItems",
    "Assets",
    "Regions",
];

/// OpenSim Next enhanced tables
const OPENSIM_NEXT_TABLES: &[&str] = &[
    "users",
    "user_sessions",
    "regions",
    "region_statistics",
    "assets",
    "asset_cache",
    "inventory_folders",
    "inventory_items",
    "physics_engines",
    "script_instances",
    "grid_events",
    "performance_metrics",
];

impl DatabaseCompatibilityManager {
    /// Create a new database compatibility manager
    pub async fn new(pool: PgPool) -> Result<Self> {
        let mut manager = Self {
            pool,
            detected_schema: None,
            migration_state: MigrationState {
                current_version: None,
                target_version: "OpenSimNext-1.0".to_string(),
                completed_migrations: Vec::new(),
                pending_migrations: Vec::new(),
                failed_migrations: Vec::new(),
            },
        };

        manager.detect_schema().await?;
        Ok(manager)
    }

    /// Detect existing database schema
    pub async fn detect_schema(&mut self) -> Result<()> {
        let mut detected_tables = Vec::new();

        // Query all tables in the database
        let rows = sqlx::query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            let table_name: String = row.get("table_name");
            detected_tables.push(table_name);
        }

        // Determine schema type
        let schema_type = self.determine_schema_type(&detected_tables);

        // Check for missing tables
        let missing_tables = self.find_missing_tables(&detected_tables, &schema_type);

        // Check if migration is needed
        let needs_migration = !missing_tables.is_empty() || schema_type != SchemaType::OpenSimNext;

        // Try to detect version
        let version = self
            .detect_database_version()
            .await
            .unwrap_or_else(|| match schema_type {
                SchemaType::OpenSimStandalone => "OpenSim-0.9.x".to_string(),
                SchemaType::OpenSimGrid => "OpenSim-Grid-0.9.x".to_string(),
                SchemaType::RobustServices => "ROBUST-0.9.x".to_string(),
                SchemaType::OpenSimNext => "OpenSimNext-1.0".to_string(),
                SchemaType::Unknown => "Unknown".to_string(),
            });

        self.detected_schema = Some(OpenSimSchema {
            version,
            detected_tables,
            missing_tables,
            schema_type,
            needs_migration,
        });

        if let Some(schema) = &self.detected_schema {
            tracing::info!(
                "Detected database schema: {:?} (version: {})",
                schema.schema_type,
                schema.version
            );

            if schema.needs_migration {
                tracing::warn!(
                    "Database migration required. Missing tables: {:?}",
                    schema.missing_tables
                );
            }
        }

        Ok(())
    }

    /// Determine schema type based on detected tables
    fn determine_schema_type(&self, tables: &[String]) -> SchemaType {
        let table_set: std::collections::HashSet<&String> = tables.iter().collect();

        // Check for OpenSim Next tables
        let opensim_next_count = OPENSIM_NEXT_TABLES
            .iter()
            .filter(|&&table| table_set.contains(&table.to_string()))
            .count();

        if opensim_next_count >= OPENSIM_NEXT_TABLES.len() / 2 {
            return SchemaType::OpenSimNext;
        }

        // Check for ROBUST tables
        let robust_count = ROBUST_TABLES
            .iter()
            .filter(|&&table| table_set.contains(&table.to_string()))
            .count();

        if robust_count >= ROBUST_TABLES.len() / 2 {
            return SchemaType::RobustServices;
        }

        // Check for OpenSim core tables
        let opensim_count = OPENSIM_CORE_TABLES
            .iter()
            .filter(|&&table| table_set.contains(&table.to_string()))
            .count();

        if opensim_count >= OPENSIM_CORE_TABLES.len() / 2 {
            if table_set.contains(&"regions".to_string()) {
                SchemaType::OpenSimGrid
            } else {
                SchemaType::OpenSimStandalone
            }
        } else {
            SchemaType::Unknown
        }
    }

    /// Find missing tables for the target schema
    fn find_missing_tables(
        &self,
        existing_tables: &[String],
        schema_type: &SchemaType,
    ) -> Vec<String> {
        let table_set: std::collections::HashSet<&String> = existing_tables.iter().collect();
        let mut missing = Vec::new();

        // Check required tables based on target (OpenSim Next)
        for &required_table in OPENSIM_NEXT_TABLES {
            if !table_set.contains(&required_table.to_string()) {
                missing.push(required_table.to_string());
            }
        }

        missing
    }

    /// Detect database version from metadata
    async fn detect_database_version(&self) -> Option<String> {
        // Try to get version from migrations table
        if let Ok(row) = sqlx::query("SELECT version FROM migrations ORDER BY id DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await
        {
            if let Some(row) = row {
                return Some(row.get("version"));
            }
        }

        // Try to get version from legacy version table
        if let Ok(row) = sqlx::query("SELECT Revision FROM version LIMIT 1")
            .fetch_optional(&self.pool)
            .await
        {
            if let Some(row) = row {
                let revision: i32 = row.get("Revision");
                return Some(format!("OpenSim-r{}", revision));
            }
        }

        None
    }

    /// Migrate database to OpenSim Next schema
    pub async fn migrate_to_opensim_next(&mut self) -> Result<()> {
        if let Some(schema) = &self.detected_schema {
            tracing::info!(
                "Starting migration from {:?} to OpenSimNext",
                schema.schema_type
            );

            match schema.schema_type {
                SchemaType::OpenSimStandalone => self.migrate_from_standalone().await?,
                SchemaType::OpenSimGrid => self.migrate_from_grid().await?,
                SchemaType::RobustServices => self.migrate_from_robust().await?,
                SchemaType::OpenSimNext => {
                    tracing::info!("Database already uses OpenSim Next schema");
                    return Ok(());
                }
                SchemaType::Unknown => {
                    return Err(anyhow::anyhow!(
                        "Cannot migrate from unknown database schema".to_string()
                    ));
                }
            }

            // Update schema detection after migration
            self.detect_schema().await?;

            tracing::info!("Database migration completed successfully");
        }

        Ok(())
    }

    /// Migrate from OpenSim standalone database
    async fn migrate_from_standalone(&mut self) -> Result<()> {
        tracing::info!("Migrating from OpenSim standalone database");

        // Create OpenSim Next tables
        self.create_opensim_next_tables().await?;

        // Migrate user data
        self.migrate_user_data().await?;

        // Migrate region data
        self.migrate_region_data().await?;

        // Migrate inventory data
        self.migrate_inventory_data().await?;

        // Migrate asset data
        self.migrate_asset_data().await?;

        Ok(())
    }

    /// Migrate from OpenSim grid database
    async fn migrate_from_grid(&mut self) -> Result<()> {
        tracing::info!("Migrating from OpenSim grid database");

        // Similar to standalone but with additional grid-specific tables
        self.migrate_from_standalone().await?;

        // Migrate grid-specific data
        self.migrate_grid_data().await?;

        Ok(())
    }

    /// Migrate from ROBUST services database
    async fn migrate_from_robust(&mut self) -> Result<()> {
        tracing::info!("Migrating from ROBUST services database");

        // Create OpenSim Next tables
        self.create_opensim_next_tables().await?;

        // Migrate ROBUST data with case-sensitive table names
        self.migrate_robust_user_accounts().await?;
        self.migrate_robust_inventory().await?;
        self.migrate_robust_assets().await?;
        self.migrate_robust_regions().await?;

        Ok(())
    }

    /// Create OpenSim Next database tables
    async fn create_opensim_next_tables(&self) -> Result<()> {
        tracing::info!("Creating OpenSim Next database tables");

        // Read and execute migration SQL files
        let migrations = [
            include_str!("../database/migrations/001_create_users.sql"),
            include_str!("../database/migrations/002_create_regions.sql"),
            include_str!("../database/migrations/003_create_inventory.sql"),
            include_str!("../database/migrations/004_create_assets.sql"),
            include_str!("../database/migrations/005_create_sessions.sql"),
        ];

        for (i, migration_sql) in migrations.iter().enumerate() {
            tracing::debug!("Executing migration {}", i + 1);
            sqlx::query(migration_sql).execute(&self.pool).await?;
        }

        Ok(())
    }

    /// Migrate user data from legacy format
    async fn migrate_user_data(&self) -> Result<()> {
        tracing::info!("Migrating user data");

        // Check if legacy users table exists
        let legacy_exists = sqlx::query(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'users')",
        )
        .fetch_one(&self.pool)
        .await?;

        if legacy_exists.get::<bool, _>(0) {
            // Migrate from legacy users table
            sqlx::query(r#"
                INSERT INTO users (id, username, password_hash, email, first_name, last_name, created_at, updated_at)
                SELECT UUID, CONCAT(username, ' ', lastname), passwordHash, email, username, lastname, created, created
                FROM users 
                WHERE NOT EXISTS (SELECT 1 FROM users WHERE users.username = CONCAT(users.username, ' ', users.lastname))
            "#)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Migrate region data
    async fn migrate_region_data(&self) -> Result<()> {
        tracing::info!("Migrating region data");

        // Migrate regions if legacy table exists
        let legacy_exists = sqlx::query(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'regions')",
        )
        .fetch_one(&self.pool)
        .await?;

        if legacy_exists.get::<bool, _>(0) {
            sqlx::query(r#"
                INSERT INTO regions (id, name, location_x, location_y, size_x, size_y, owner_id, created_at, updated_at)
                SELECT uuid, regionName, locX, locY, 256, 256, owner_uuid, NOW(), NOW()
                FROM regions
                WHERE NOT EXISTS (SELECT 1 FROM regions WHERE regions.id = regions.uuid)
            "#)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Migrate inventory data
    async fn migrate_inventory_data(&self) -> Result<()> {
        tracing::info!("Migrating inventory data");

        // Migrate inventory folders
        let folders_exist = sqlx::query("SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'inventoryfolders')")
            .fetch_one(&self.pool)
            .await?;

        if folders_exist.get::<bool, _>(0) {
            sqlx::query(r#"
                INSERT INTO inventory_folders (id, name, owner_id, parent_id, folder_type, version, created_at, updated_at)
                SELECT folderID, folderName, agentID, parentFolderID, type, version, NOW(), NOW()
                FROM inventoryfolders
                WHERE NOT EXISTS (SELECT 1 FROM inventory_folders WHERE inventory_folders.id = inventoryfolders.folderID)
            "#)
            .execute(&self.pool)
            .await?;
        }

        // Migrate inventory items
        let items_exist = sqlx::query("SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'inventoryitems')")
            .fetch_one(&self.pool)
            .await?;

        if items_exist.get::<bool, _>(0) {
            sqlx::query(r#"
                INSERT INTO inventory_items (id, name, description, asset_type, inventory_type, folder_id, owner_id, creator_id, asset_id, base_permissions, current_permissions, everyone_permissions, group_permissions, next_permissions, created_at, updated_at)
                SELECT inventoryID, inventoryName, inventoryDescription, assetType, invType, parentFolderID, avatarID, creatorsID, assetID, inventoryBasePermissions, inventoryCurrentPermissions, inventoryEveryOnePermissions, inventoryGroupPermissions, inventoryNextPermissions, creationDate, creationDate
                FROM inventoryitems
                WHERE NOT EXISTS (SELECT 1 FROM inventory_items WHERE inventory_items.id = inventoryitems.inventoryID)
            "#)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Migrate asset data
    async fn migrate_asset_data(&self) -> Result<()> {
        tracing::info!("Migrating asset data");

        let assets_exist = sqlx::query(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'assets')",
        )
        .fetch_one(&self.pool)
        .await?;

        if assets_exist.get::<bool, _>(0) {
            sqlx::query(r#"
                INSERT INTO assets (id, name, description, asset_type, local, temporary, data, created_at, updated_at)
                SELECT id, name, description, assetType, local, temporary, data, create_time, access_time
                FROM assets
                WHERE NOT EXISTS (SELECT 1 FROM assets WHERE assets.id = assets.id)
            "#)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Migrate grid-specific data
    async fn migrate_grid_data(&self) -> Result<()> {
        tracing::info!("Migrating grid-specific data");

        // Additional migrations for grid mode
        // This would include estate data, land data, etc.

        Ok(())
    }

    /// Migrate ROBUST UserAccounts table
    async fn migrate_robust_user_accounts(&self) -> Result<()> {
        tracing::info!("Migrating ROBUST UserAccounts");

        let robust_users_exist = sqlx::query("SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'UserAccounts')")
            .fetch_one(&self.pool)
            .await?;

        if robust_users_exist.get::<bool, _>(0) {
            sqlx::query(r#"
                INSERT INTO users (id, username, password_hash, email, first_name, last_name, created_at, updated_at)
                SELECT "PrincipalID", CONCAT("FirstName", ' ', "LastName"), '', "Email", "FirstName", "LastName", to_timestamp("Created"), to_timestamp("Created")
                FROM "UserAccounts"
                WHERE NOT EXISTS (SELECT 1 FROM users WHERE users.id = "UserAccounts"."PrincipalID")
            "#)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Migrate ROBUST inventory data
    async fn migrate_robust_inventory(&self) -> Result<()> {
        tracing::info!("Migrating ROBUST inventory data");

        // Migrate ROBUST InventoryFolders
        let robust_folders_exist = sqlx::query("SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'InventoryFolders')")
            .fetch_one(&self.pool)
            .await?;

        if robust_folders_exist.get::<bool, _>(0) {
            sqlx::query(r#"
                INSERT INTO inventory_folders (id, name, owner_id, parent_id, folder_type, version, created_at, updated_at)
                SELECT "folderID", "folderName", "agentID", "parentFolderID", "type", "version", NOW(), NOW()
                FROM "InventoryFolders"
                WHERE NOT EXISTS (SELECT 1 FROM inventory_folders WHERE inventory_folders.id = "InventoryFolders"."folderID")
            "#)
            .execute(&self.pool)
            .await?;
        }

        // Migrate ROBUST InventoryItems
        let robust_items_exist = sqlx::query("SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'InventoryItems')")
            .fetch_one(&self.pool)
            .await?;

        if robust_items_exist.get::<bool, _>(0) {
            sqlx::query(r#"
                INSERT INTO inventory_items (id, name, description, asset_type, inventory_type, folder_id, owner_id, creator_id, asset_id, base_permissions, current_permissions, everyone_permissions, group_permissions, next_permissions, created_at, updated_at)
                SELECT "inventoryID", "inventoryName", "inventoryDescription", "assetType", "invType", "parentFolderID", "avatarID", "creatorID", "assetID", "inventoryBasePermissions", "inventoryCurrentPermissions", "inventoryEveryOnePermissions", "inventoryGroupPermissions", "inventoryNextPermissions", NOW(), NOW()
                FROM "InventoryItems"
                WHERE NOT EXISTS (SELECT 1 FROM inventory_items WHERE inventory_items.id = "InventoryItems"."inventoryID")
            "#)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Migrate ROBUST assets
    async fn migrate_robust_assets(&self) -> Result<()> {
        tracing::info!("Migrating ROBUST assets");

        let robust_assets_exist = sqlx::query(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'Assets')",
        )
        .fetch_one(&self.pool)
        .await?;

        if robust_assets_exist.get::<bool, _>(0) {
            sqlx::query(r#"
                INSERT INTO assets (id, name, description, asset_type, local, temporary, data, created_at, updated_at)
                SELECT "ID", "Name", "Description", "Type", "Local", "Temporary", "Data", NOW(), NOW()
                FROM "Assets"
                WHERE NOT EXISTS (SELECT 1 FROM assets WHERE assets.id = "Assets"."ID")
            "#)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Migrate ROBUST regions
    async fn migrate_robust_regions(&self) -> Result<()> {
        tracing::info!("Migrating ROBUST regions");

        let robust_regions_exist = sqlx::query(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'Regions')",
        )
        .fetch_one(&self.pool)
        .await?;

        if robust_regions_exist.get::<bool, _>(0) {
            sqlx::query(r#"
                INSERT INTO regions (id, name, location_x, location_y, size_x, size_y, owner_id, created_at, updated_at)
                SELECT "uuid", "regionName", "locX", "locY", "sizeX", "sizeY", "owner_uuid", NOW(), NOW()
                FROM "Regions"
                WHERE NOT EXISTS (SELECT 1 FROM regions WHERE regions.id = "Regions"."uuid")
            "#)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Check database compatibility
    pub fn is_compatible(&self) -> bool {
        if let Some(schema) = &self.detected_schema {
            !schema.needs_migration
        } else {
            false
        }
    }

    /// Get detected schema information
    pub fn get_schema_info(&self) -> Option<&OpenSimSchema> {
        self.detected_schema.as_ref()
    }

    /// Get migration state
    pub fn get_migration_state(&self) -> &MigrationState {
        &self.migration_state
    }
}
