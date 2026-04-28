//! Database Migration Engine for OpenSim Next
//!
//! Rust implementation of OpenSim's Migration.cs system
//! Provides Ruby on Rails-style database migrations with SQLite support

use anyhow::{anyhow, Result};
use sqlx::{Row, SqliteConnection};
use std::collections::BTreeMap;
use tracing::{debug, error, info, warn};

/// Migration engine that manages database schema updates
#[derive(Debug)]
pub struct MigrationEngine {
    store_name: String,
}

impl MigrationEngine {
    /// Create new migration engine for a specific store
    pub fn new(store_name: String) -> Self {
        Self { store_name }
    }

    /// Initialize migrations table if it doesn't exist
    pub async fn init_migrations_table(conn: &mut SqliteConnection) -> Result<()> {
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS migrations (
                name VARCHAR(100),
                version INT
            )
        "#;

        sqlx::query(create_table_sql).execute(conn).await?;

        debug!("Migrations table initialized");
        Ok(())
    }

    /// Find current version for a specific store
    pub async fn find_version(conn: &mut SqliteConnection, store_name: &str) -> Result<i32> {
        Self::init_migrations_table(conn).await?;

        let row = sqlx::query("SELECT version FROM migrations WHERE name = ?")
            .bind(store_name)
            .fetch_optional(conn)
            .await?;

        match row {
            Some(row) => Ok(row.get::<i32, _>("version")),
            None => Ok(0), // No migrations applied yet
        }
    }

    /// Update store version in migrations table
    pub async fn update_store_version(
        conn: &mut SqliteConnection,
        store_name: &str,
        version: i32,
    ) -> Result<()> {
        // First try to update existing record
        let rows_affected = sqlx::query("UPDATE migrations SET version = ? WHERE name = ?")
            .bind(version)
            .bind(store_name)
            .execute(&mut *conn)
            .await?
            .rows_affected();

        // If no rows were updated, insert new record
        if rows_affected == 0 {
            sqlx::query("INSERT INTO migrations (name, version) VALUES (?, ?)")
                .bind(store_name)
                .bind(version)
                .execute(&mut *conn)
                .await?;
        }

        debug!("Updated {} to version {}", store_name, version);
        Ok(())
    }

    /// Execute a single migration script
    pub async fn execute_migration(
        conn: &mut SqliteConnection,
        script: &str,
        version: i32,
        store_name: &str,
    ) -> Result<()> {
        info!("Applying migration {} version {}", store_name, version);

        // Split script into individual statements (simple implementation)
        let statements: Vec<&str> = script
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for statement in statements {
            if !statement.is_empty() {
                debug!("Executing: {}", statement);
                sqlx::query(statement)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| anyhow!("Failed to execute statement '{}': {}", statement, e))?;
            }
        }

        info!(
            "Successfully applied migration {} version {}",
            store_name, version
        );
        Ok(())
    }

    /// Update a store to latest version using provided migrations
    pub async fn update_store(
        &self,
        conn: &mut SqliteConnection,
        migrations: BTreeMap<i32, String>,
    ) -> Result<()> {
        let current_version = Self::find_version(conn, &self.store_name).await?;

        let pending_migrations: BTreeMap<i32, String> = migrations
            .into_iter()
            .filter(|(version, _)| *version > current_version)
            .collect();

        if pending_migrations.is_empty() {
            debug!(
                "{} is already at latest version {}",
                self.store_name, current_version
            );
            return Ok(());
        }

        let latest_version = *pending_migrations.keys().last().unwrap();
        info!(
            "Upgrading {} from version {} to version {}",
            self.store_name, current_version, latest_version
        );

        for (version, script) in pending_migrations {
            Self::execute_migration(conn, &script, version, &self.store_name).await?;
            Self::update_store_version(conn, &self.store_name, version).await?;
        }

        info!(
            "Successfully upgraded {} to version {}",
            self.store_name, latest_version
        );
        Ok(())
    }
}

/// Asset Store migrations converted from MySQL to SQLite
pub fn get_asset_store_migrations() -> BTreeMap<i32, String> {
    let mut migrations = BTreeMap::new();

    // Version 10: Create assets table (converted from MySQL)
    migrations.insert(
        10,
        r#"
        CREATE TABLE IF NOT EXISTS assets (
            name VARCHAR(64) NOT NULL,
            description VARCHAR(64) NOT NULL,
            assetType INTEGER NOT NULL,
            local INTEGER NOT NULL,
            temporary INTEGER NOT NULL,
            data BLOB NOT NULL,
            id CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            create_time INTEGER DEFAULT 0,
            access_time INTEGER DEFAULT 0,
            asset_flags INTEGER NOT NULL DEFAULT 0,
            CreatorID VARCHAR(128) NOT NULL DEFAULT '',
            PRIMARY KEY (id)
        )
    "#
        .to_string(),
    );

    migrations
}

/// Inventory Store migrations converted from MySQL to SQLite  
pub fn get_inventory_store_migrations() -> BTreeMap<i32, String> {
    let mut migrations = BTreeMap::new();

    // Version 7: Create inventory tables (converted from MySQL)
    migrations.insert(7, r#"
        CREATE TABLE IF NOT EXISTS inventoryitems (
            assetID VARCHAR(36) DEFAULT NULL,
            assetType INTEGER DEFAULT NULL,
            inventoryName VARCHAR(64) DEFAULT NULL,
            inventoryDescription VARCHAR(128) DEFAULT NULL,
            inventoryNextPermissions INTEGER NOT NULL DEFAULT 0,
            inventoryCurrentPermissions INTEGER NOT NULL DEFAULT 0,
            invType INTEGER DEFAULT NULL,
            creatorID VARCHAR(255) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            inventoryBasePermissions INTEGER NOT NULL DEFAULT 0,
            inventoryEveryOnePermissions INTEGER NOT NULL DEFAULT 0,
            salePrice INTEGER NOT NULL DEFAULT 0,
            saleType INTEGER NOT NULL DEFAULT 0,
            creationDate INTEGER NOT NULL DEFAULT 0,
            groupID VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            groupOwned INTEGER NOT NULL DEFAULT 0,
            flags INTEGER NOT NULL DEFAULT 0,
            inventoryID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            avatarID CHAR(36) DEFAULT NULL,
            parentFolderID CHAR(36) DEFAULT NULL,
            inventoryGroupPermissions INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (inventoryID)
        );

        CREATE INDEX IF NOT EXISTS inventoryitems_avatarid ON inventoryitems(avatarID);
        CREATE INDEX IF NOT EXISTS inventoryitems_parentFolderid ON inventoryitems(parentFolderID);

        CREATE TABLE IF NOT EXISTS inventoryfolders (
            folderName VARCHAR(64) DEFAULT NULL,
            type INTEGER NOT NULL DEFAULT 0,
            version INTEGER NOT NULL DEFAULT 0,
            folderID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            agentID CHAR(36) DEFAULT NULL,
            parentFolderID CHAR(36) DEFAULT NULL,
            PRIMARY KEY (folderID)
        );

        CREATE INDEX IF NOT EXISTS inventoryfolders_agentid ON inventoryfolders(agentID);
        CREATE INDEX IF NOT EXISTS inventoryfolders_parentFolderid ON inventoryfolders(parentFolderID);
    "#.to_string());

    migrations
}

/// Estate Store migrations (complete implementation to version 36)
pub fn get_estate_store_migrations() -> BTreeMap<i32, String> {
    let mut migrations = BTreeMap::new();

    // Version 34: Create estate tables (converted from MySQL)
    migrations.insert(
        34,
        r#"
        CREATE TABLE IF NOT EXISTS estate_groups (
            EstateID INTEGER NOT NULL,
            uuid CHAR(36) NOT NULL
        );

        CREATE INDEX IF NOT EXISTS estate_groups_EstateID ON estate_groups(EstateID);

        CREATE TABLE IF NOT EXISTS estate_managers (
            EstateID INTEGER NOT NULL,
            uuid CHAR(36) NOT NULL
        );

        CREATE INDEX IF NOT EXISTS estate_managers_EstateID ON estate_managers(EstateID);

        CREATE TABLE IF NOT EXISTS estate_map (
            RegionID CHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            EstateID INTEGER NOT NULL,
            PRIMARY KEY (RegionID)
        );

        CREATE INDEX IF NOT EXISTS estate_map_EstateID ON estate_map(EstateID);

        CREATE TABLE IF NOT EXISTS estate_settings (
            EstateID INTEGER PRIMARY KEY AUTOINCREMENT,
            EstateName VARCHAR(64) DEFAULT NULL,
            AbuseEmailToEstateOwner INTEGER NOT NULL,
            DenyAnonymous INTEGER NOT NULL,
            ResetHomeOnTeleport INTEGER NOT NULL,
            FixedSun INTEGER NOT NULL,
            DenyTransacted INTEGER NOT NULL,
            BlockDwell INTEGER NOT NULL,
            DenyIdentified INTEGER NOT NULL,
            AllowVoice INTEGER NOT NULL,
            UseGlobalTime INTEGER NOT NULL,
            PricePerMeter INTEGER NOT NULL,
            TaxFree INTEGER NOT NULL,
            AllowDirectTeleport INTEGER NOT NULL,
            RedirectGridX INTEGER NOT NULL,
            RedirectGridY INTEGER NOT NULL,
            ParentEstateID INTEGER NOT NULL,
            SunPosition REAL NOT NULL,
            EstateSkipScripts INTEGER NOT NULL,
            BillableFactor REAL NOT NULL,
            PublicAccess INTEGER NOT NULL,
            AbuseEmail VARCHAR(255) NOT NULL,
            EstateOwner VARCHAR(36) NOT NULL,
            DenyMinors INTEGER NOT NULL,
            AllowLandmark INTEGER NOT NULL DEFAULT 1,
            AllowParcelChanges INTEGER NOT NULL DEFAULT 1,
            AllowSetHome INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS estate_users (
            EstateID INTEGER NOT NULL,
            uuid CHAR(36) NOT NULL
        );

        CREATE INDEX IF NOT EXISTS estate_users_EstateID ON estate_users(EstateID);

        CREATE TABLE IF NOT EXISTS estateban (
            EstateID INTEGER NOT NULL,
            bannedUUID VARCHAR(36) NOT NULL,
            bannedIp VARCHAR(16) NOT NULL,
            bannedIpHostMask VARCHAR(16) NOT NULL,
            bannedNameMask VARCHAR(64) DEFAULT NULL
        );

        CREATE INDEX IF NOT EXISTS estateban_EstateID ON estateban(EstateID);
    "#
        .to_string(),
    );

    // Version 35: Add banning fields to estateban table
    migrations.insert(35, r#"
        ALTER TABLE estateban ADD COLUMN banningUUID VARCHAR(36) NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
        ALTER TABLE estateban ADD COLUMN banTime INTEGER NOT NULL DEFAULT 0;
    "#.to_string());

    // Version 36: Add environment override to estate settings
    migrations.insert(
        36,
        r#"
        ALTER TABLE estate_settings ADD COLUMN AllowEnviromentOverride INTEGER NOT NULL DEFAULT 0;
    "#
        .to_string(),
    );

    migrations
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_migration_engine() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let mut conn = pool.acquire().await.unwrap();

        // Test migrations table creation
        MigrationEngine::init_migrations_table(&mut conn)
            .await
            .unwrap();

        // Test version tracking
        let version = MigrationEngine::find_version(&mut conn, "test_store")
            .await
            .unwrap();
        assert_eq!(version, 0);

        // Test version update
        MigrationEngine::update_store_version(&mut conn, "test_store", 5)
            .await
            .unwrap();
        let version = MigrationEngine::find_version(&mut conn, "test_store")
            .await
            .unwrap();
        assert_eq!(version, 5);
    }
}
