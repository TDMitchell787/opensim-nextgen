// Multi-backend database migration manager
// Supports both SQLite and PostgreSQL with OpenSim master compatibility

use anyhow::{anyhow, Result};
use tracing::{info, debug, warn, error};

pub enum DatabaseType {
    SQLite,
    PostgreSQL,
    MySQL,
    MariaDB,
}

pub struct MigrationManager {
    database_type: DatabaseType,
}

impl MigrationManager {
    pub fn new(database_type: DatabaseType) -> Self {
        Self { database_type }
    }

    /// Get the appropriate migration files for the database type
    pub fn get_postgres_migrations(&self) -> Vec<&'static str> {
        vec![
            include_str!("../../migrations/postgres/001_initial_schema.sql"),
            include_str!("../../migrations/postgres/002_opensim_master_compatibility.sql"),
            include_str!("../../migrations/postgres/003_regionstore_complete.sql"),
            include_str!("../../migrations/postgres/004_social_systems_complete.sql"),
            include_str!("../../migrations/postgres/005_advanced_stores_complete.sql"),
            include_str!("../../migrations/postgres/006_complete_stores_final.sql"),
            include_str!("../../migrations/postgres/007_extended_features_complete.sql"),
            include_str!("../../migrations/postgres/008_ai_learning_patterns.sql"),
            include_str!("../../migrations/postgres/010_primitems_table.sql"),
            include_str!("../../migrations/postgres/011_landaccesslist_table.sql"),
            include_str!("../../migrations/postgres/012_economy_tables.sql"),
            // Phase 198: FSAssets — separate namespace from upstream OpenSim
            include_str!("../../migrations/postgres/fsassets/001_fsassets_table.sql"),
        ]
    }

    /// Get SQLite migrations (existing system)
    pub fn get_sqlite_migrations(&self) -> Vec<&'static str> {
        vec![
            include_str!("migrations/001_create_users.sql"),
            include_str!("migrations/002_create_regions.sql"),
            include_str!("migrations/003_create_inventory.sql"),
            include_str!("migrations/004_create_assets.sql"),
            include_str!("migrations/005_create_sessions.sql"),
            include_str!("migrations/006_opensim_compatibility.sql"),
            // RegionStore v52-67 migrations for OpenSim master compatibility
            include_str!("migrations/007_regionstore_v52_avination.sql"),
            include_str!("migrations/008_regionstore_v53_rotation_locks.sql"),
            include_str!("migrations/009_regionstore_v54_baked_terrain.sql"),
            include_str!("migrations/010_regionstore_v55_windlight_precision.sql"),
            include_str!("migrations/011_regionstore_v56_rezzer_id.sql"),
            include_str!("migrations/012_regionstore_v57_physics_inertia.sql"),
            include_str!("migrations/013_regionstore_v58_sop_animations.sql"),
            include_str!("migrations/014_regionstore_v59_stand_sit_targets.sql"),
            include_str!("migrations/015_regionstore_v60_precision_fix.sql"),
            include_str!("migrations/016_regionstore_v61_pseudo_crc.sql"),
            include_str!("migrations/017_regionstore_v62_environment_size.sql"),
            include_str!("migrations/018_regionstore_v63_parcel_environment.sql"),
            include_str!("migrations/019_regionstore_v64_material_overrides.sql"),
            include_str!("migrations/020_regionstore_v65_linkset_data.sql"),
            include_str!("migrations/021_regionstore_v66_pbr_terrain.sql"),
            include_str!("migrations/022_regionstore_v67_rez_start_string.sql"),
            // Phase 2: Social system migrations
            include_str!("migrations/023_friends_v2_to_v4.sql"),
            include_str!("migrations/024_presence_v3_to_v4.sql"),
            include_str!("migrations/025_avatar_v3.sql"),
            include_str!("migrations/026_griduser_v2.sql"),
            // Phase 3: Complete store implementations
            include_str!("migrations/027_estate_store_v36.sql"),
            include_str!("migrations/028_userprofiles_v5.sql"),
            include_str!("migrations/029_agentprefs_v1.sql"),
            include_str!("migrations/030_xasset_store_v2.sql"),
            include_str!("migrations/031_grid_store_v10.sql"),
            include_str!("migrations/032_im_store_v5.sql"),
            include_str!("migrations/033_hgtravel_store_v2.sql"),
            include_str!("migrations/034_mute_list_v1.sql"),
            include_str!("migrations/035_os_groups_v3.sql"),
            include_str!("migrations/036_log_store_v1.sql"),
            // Phase 68.25: AI Learning Persistence
            include_str!("migrations/037_ai_learning_patterns.sql"),
            // Phase 134: Prim inventory (scripts, notecards in object Contents tab)
            include_str!("migrations/039_primitems_table.sql"),
            // Parcel access control
            include_str!("migrations/040_landaccesslist_table.sql"),
            // Inventory schema + NULL permission fix for HG items
            include_str!("migrations/041_inventory_opensim_compat.sql"),
            // Phase 197: Economy system (currency, transactions, marketplace, Gloebit)
            include_str!("migrations/042_economy_tables.sql"),
            // Phase 198: FSAssets — separate namespace from upstream OpenSim
            include_str!("migrations/fsassets/001_fsassets_table.sql"),
        ]
    }

    /// Get MySQL migrations 
    pub fn get_mysql_migrations(&self) -> Vec<&'static str> {
        vec![
            include_str!("../../migrations/mysql/001_initial_schema.sql"),
            include_str!("../../migrations/mysql/002_opensim_master_compatibility.sql"),
            include_str!("../../migrations/mysql/003_regionstore_complete.sql"),
            include_str!("../../migrations/mysql/004_social_systems_complete.sql"),
            include_str!("../../migrations/mysql/005_regionstore_advanced.sql"),
            include_str!("../../migrations/mysql/006_complete_stores.sql"),
            include_str!("../../migrations/mysql/007_extended_features.sql"),
            include_str!("../../migrations/mysql/008_ai_learning_patterns.sql"),
            include_str!("../../migrations/mysql/009_primitems_table.sql"),
            include_str!("../../migrations/mysql/010_landaccesslist_table.sql"),
            include_str!("../../migrations/mysql/011_inventory_defaults_fix.sql"),
            include_str!("../../migrations/mysql/012_economy_tables.sql"),
            // Phase 198: FSAssets — separate namespace from upstream OpenSim
            include_str!("../../migrations/mysql/fsassets/001_fsassets_table.sql"),
        ]
    }

    /// Get appropriate migrations based on database type
    pub fn get_migrations(&self) -> Vec<&'static str> {
        match self.database_type {
            DatabaseType::PostgreSQL => self.get_postgres_migrations(),
            DatabaseType::SQLite => self.get_sqlite_migrations(),
            DatabaseType::MySQL => self.get_mysql_migrations(),
            DatabaseType::MariaDB => {
                // MariaDB is highly MySQL-compatible, use MySQL migrations
                info!("Using MySQL-compatible migrations for MariaDB");
                self.get_mysql_migrations()
            }
        }
    }

    /// Detect database type from connection string
    pub fn detect_database_type(connection_string: &str) -> DatabaseType {
        if connection_string.starts_with("postgresql://") || connection_string.starts_with("postgres://") {
            DatabaseType::PostgreSQL
        } else if connection_string.starts_with("mysql://") {
            DatabaseType::MySQL
        } else if connection_string.starts_with("mariadb://") {
            DatabaseType::MariaDB
        } else if connection_string.ends_with(".db") || connection_string.contains("sqlite") {
            DatabaseType::SQLite
        } else {
            // Default to PostgreSQL for unknown connection strings
            warn!("Unknown database type in connection string, defaulting to PostgreSQL");
            DatabaseType::PostgreSQL
        }
    }

    /// Check if migration should be skipped based on error
    pub fn should_skip_migration_error(&self, error: &str) -> bool {
        let skip_patterns = [
            "already exists",
            "no such column",
            "duplicate column",
            "table",
            "exists",
            "relation already exists",
            "column already exists",
            "index already exists",
        ];

        skip_patterns.iter().any(|pattern| error.to_lowercase().contains(pattern))
    }

    /// Detect MariaDB vs MySQL at runtime
    pub fn detect_mariadb_runtime(&self) -> bool {
        false
    }

    pub fn extract_table_names(migrations: &[&str]) -> Vec<String> {
        let mut tables = Vec::new();
        for sql in migrations {
            let upper = sql.to_uppercase();
            let tokens: Vec<&str> = upper.split_whitespace().collect();
            for i in 0..tokens.len().saturating_sub(1) {
                if tokens[i] == "CREATE" && tokens.get(i + 1) == Some(&"TABLE") {
                    let name_idx = if tokens.get(i + 2) == Some(&"IF") {
                        i + 5
                    } else {
                        i + 2
                    };
                    if let Some(raw) = tokens.get(name_idx) {
                        let clean = raw
                            .trim_matches(|c| c == '(' || c == '"' || c == '`' || c == '\'' || c == ';')
                            .to_lowercase();
                        if !clean.is_empty() && clean.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                            if !tables.contains(&clean) {
                                tables.push(clean);
                            }
                        }
                    }
                }
            }
        }
        tables
    }

    pub fn required_opensim_tables() -> Vec<&'static str> {
        vec![
            "useraccounts", "auth", "tokens",
            "avatars", "friends", "griduser", "presence",
            "assets",
            "inventoryfolders", "inventoryitems",
            "regions",
            "prims", "primshapes", "primitems",
            "land", "landaccesslist",
            "regionsettings", "regionwindlight", "regionenvironment", "bakedterrain",
            "estate_settings", "estate_map", "estate_groups", "estate_managers",
            "estate_users", "estateban",
            "userprofile", "userpicks", "userclassifieds", "usernotes", "usersettings",
            "agentprefs", "mutelist",
            "os_groups_groups", "os_groups_roles", "os_groups_membership",
            "os_groups_rolemembership", "os_groups_principals",
            "os_groups_invites", "os_groups_notices",
            "xassetsdata", "xassetsmeta",
            "im_offline", "hg_traveling_data", "logs",
            "currency_balances", "currency_definitions", "economy_transactions",
            "marketplace_categories", "marketplace_listings",
            "purchase_orders", "escrow_accounts", "fraud_alerts", "gloebit_tokens",
            "fsassets",
        ]
    }

    pub fn sqlite_table_aliases() -> Vec<(&'static str, &'static str)> {
        vec![
            ("prims", "scene_objects"),
            ("primshapes", "prim_shapes"),
            ("land", "land_parcels"),
            ("useraccounts", "users"),
        ]
    }

    fn has_table_or_alias(tables: &[String], name: &str, aliases: &[(&str, &str)]) -> bool {
        if tables.contains(&name.to_string()) {
            return true;
        }
        for &(canonical, alias) in aliases {
            if canonical == name && tables.contains(&alias.to_string()) {
                return true;
            }
        }
        false
    }

    pub fn verify_migration_consistency() -> Vec<String> {
        let pg_mgr = MigrationManager::new(DatabaseType::PostgreSQL);
        let mysql_mgr = MigrationManager::new(DatabaseType::MySQL);
        let sqlite_mgr = MigrationManager::new(DatabaseType::SQLite);

        let pg_tables = Self::extract_table_names(&pg_mgr.get_postgres_migrations());
        let mysql_tables = Self::extract_table_names(&mysql_mgr.get_mysql_migrations());
        let sqlite_tables = Self::extract_table_names(&sqlite_mgr.get_sqlite_migrations());
        let aliases = Self::sqlite_table_aliases();

        let mut warnings = Vec::new();

        for &table in &Self::required_opensim_tables() {
            let t = table.to_lowercase();
            if !pg_tables.contains(&t) {
                warnings.push(format!("PostgreSQL MISSING required table: {}", table));
            }
            if !mysql_tables.contains(&t) {
                warnings.push(format!("MySQL MISSING required table: {}", table));
            }
            if !Self::has_table_or_alias(&sqlite_tables, &t, &aliases) {
                warnings.push(format!("SQLite MISSING required table: {} (checked aliases too)", table));
            }
        }

        let all_tables: std::collections::HashSet<String> = pg_tables.iter()
            .chain(mysql_tables.iter())
            .cloned()
            .collect();

        for table in &all_tables {
            let in_pg = pg_tables.contains(table);
            let in_mysql = mysql_tables.contains(table);
            if in_pg && !in_mysql {
                warnings.push(format!("Table '{}' in PostgreSQL but MISSING from MySQL", table));
            } else if !in_pg && in_mysql {
                warnings.push(format!("Table '{}' in MySQL but MISSING from PostgreSQL", table));
            }
        }

        warnings
    }

    pub fn log_migration_consistency_warnings() {
        let warnings = Self::verify_migration_consistency();
        if warnings.is_empty() {
            info!("Migration consistency check: all backends have matching table coverage");
        } else {
            warn!("Migration consistency check found {} issue(s):", warnings.len());
            for w in &warnings {
                warn!("  - {}", w);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_table_names_basic() {
        let sql = &["CREATE TABLE IF NOT EXISTS users (id INT);"];
        let tables = MigrationManager::extract_table_names(sql);
        assert!(tables.contains(&"users".to_string()), "Should extract 'users' from CREATE TABLE IF NOT EXISTS");
    }

    #[test]
    fn test_extract_table_names_quoted() {
        let sql = &["CREATE TABLE \"UserAccounts\" (id INT);"];
        let tables = MigrationManager::extract_table_names(sql);
        assert!(tables.contains(&"useraccounts".to_string()), "Should extract lowercase table name from quoted CREATE TABLE");
    }

    #[test]
    fn test_extract_table_names_backtick() {
        let sql = &["CREATE TABLE IF NOT EXISTS `inventoryitems` (id INT);"];
        let tables = MigrationManager::extract_table_names(sql);
        assert!(tables.contains(&"inventoryitems".to_string()), "Should extract table name from backtick-quoted CREATE TABLE");
    }

    #[test]
    fn test_extract_no_duplicates() {
        let sql = &[
            "CREATE TABLE IF NOT EXISTS foo (id INT);",
            "CREATE TABLE IF NOT EXISTS foo (id INT);",
        ];
        let tables = MigrationManager::extract_table_names(sql);
        assert_eq!(tables.iter().filter(|t| *t == "foo").count(), 1, "Should not have duplicate table names");
    }

    #[test]
    fn test_postgres_extracts_tables() {
        let mgr = MigrationManager::new(DatabaseType::PostgreSQL);
        let tables = MigrationManager::extract_table_names(&mgr.get_postgres_migrations());
        assert!(!tables.is_empty(), "PostgreSQL should produce tables");
        assert!(tables.contains(&"prims".to_string()), "PostgreSQL should have prims table");
        assert!(tables.contains(&"inventoryitems".to_string()), "PostgreSQL should have inventoryitems table");
    }

    #[test]
    fn test_mysql_extracts_tables() {
        let mgr = MigrationManager::new(DatabaseType::MySQL);
        let tables = MigrationManager::extract_table_names(&mgr.get_mysql_migrations());
        assert!(!tables.is_empty(), "MySQL should produce tables");
        assert!(tables.contains(&"prims".to_string()), "MySQL should have prims table");
        assert!(tables.contains(&"primitems".to_string()), "MySQL should have primitems table");
    }

    #[test]
    fn test_sqlite_extracts_tables() {
        let mgr = MigrationManager::new(DatabaseType::SQLite);
        let tables = MigrationManager::extract_table_names(&mgr.get_sqlite_migrations());
        assert!(!tables.is_empty(), "SQLite should produce tables");
        let aliases = MigrationManager::sqlite_table_aliases();
        assert!(
            MigrationManager::has_table_or_alias(&tables, "prims", &aliases),
            "SQLite should have prims table (or scene_objects alias)"
        );
    }

    #[test]
    fn test_migration_consistency_report() {
        let warnings = MigrationManager::verify_migration_consistency();
        for w in &warnings {
            eprintln!("MIGRATION WARNING: {}", w);
        }
    }

    #[test]
    fn test_pg_mysql_parity() {
        let core_tables = vec![
            "inventoryfolders", "inventoryitems",
            "prims", "primshapes", "primitems",
            "land", "landaccesslist",
            "regionsettings", "regionwindlight", "regionenvironment", "bakedterrain",
            "estate_settings", "estate_map", "estate_groups", "estate_managers",
            "estate_users", "estateban",
            "xassetsdata", "xassetsmeta",
            "im_offline", "hg_traveling_data", "logs",
        ];

        let pg_mgr = MigrationManager::new(DatabaseType::PostgreSQL);
        let mysql_mgr = MigrationManager::new(DatabaseType::MySQL);

        let pg_tables = MigrationManager::extract_table_names(&pg_mgr.get_postgres_migrations());
        let mysql_tables = MigrationManager::extract_table_names(&mysql_mgr.get_mysql_migrations());

        let mut failures = Vec::new();
        for table in &core_tables {
            let t = table.to_lowercase();
            if !pg_tables.contains(&t) {
                failures.push(format!("PostgreSQL missing: {}", table));
            }
            if !mysql_tables.contains(&t) {
                failures.push(format!("MySQL missing: {}", table));
            }
        }

        if !failures.is_empty() {
            for f in &failures {
                eprintln!("CORE TABLE MISSING: {}", f);
            }
            panic!("{} core table(s) missing from production backends — see output above", failures.len());
        }
    }

    #[test]
    fn test_all_backends_have_core_tables_with_aliases() {
        let core_tables = vec![
            "inventoryfolders", "inventoryitems",
            "prims", "primshapes", "primitems",
            "land", "landaccesslist",
            "regionsettings", "regionwindlight", "regionenvironment", "bakedterrain",
            "estate_settings", "estate_map", "estate_groups", "estate_managers",
            "estate_users", "estateban",
            "xassetsdata", "xassetsmeta",
            "im_offline", "hg_traveling_data", "logs",
        ];

        let pg_mgr = MigrationManager::new(DatabaseType::PostgreSQL);
        let mysql_mgr = MigrationManager::new(DatabaseType::MySQL);
        let sqlite_mgr = MigrationManager::new(DatabaseType::SQLite);

        let pg_tables = MigrationManager::extract_table_names(&pg_mgr.get_postgres_migrations());
        let mysql_tables = MigrationManager::extract_table_names(&mysql_mgr.get_mysql_migrations());
        let sqlite_tables = MigrationManager::extract_table_names(&sqlite_mgr.get_sqlite_migrations());
        let aliases = MigrationManager::sqlite_table_aliases();

        let mut failures = Vec::new();
        for table in &core_tables {
            let t = table.to_lowercase();
            if !pg_tables.contains(&t) {
                failures.push(format!("PostgreSQL missing: {}", table));
            }
            if !mysql_tables.contains(&t) {
                failures.push(format!("MySQL missing: {}", table));
            }
            if !MigrationManager::has_table_or_alias(&sqlite_tables, &t, &aliases) {
                failures.push(format!("SQLite missing: {} (checked aliases)", table));
            }
        }

        if !failures.is_empty() {
            for f in &failures {
                eprintln!("CORE TABLE MISSING: {}", f);
            }
            panic!("{} core table(s) missing across backends — see output above", failures.len());
        }
    }
}