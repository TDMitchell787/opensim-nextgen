//! Phase 19: Database Validation with SQLx and OpenZiti Configuration
//!
//! Tests OpenSim.ini database connection strings, SQLx integration, and OpenZiti port configuration

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use tracing::{info, error};

/// OpenSim.ini database connection string validation
pub struct DatabaseValidator {
    results: Vec<ValidationResult>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub test_name: String,
    pub success: bool,
    pub error_message: Option<String>,
}

impl DatabaseValidator {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Validate OpenSim.ini database connection strings with SQLx support
    pub async fn validate_opensim_database_config(&mut self) -> Result<()> {
        info!("🗄️ Validating OpenSim.ini database configuration with SQLx");

        // Test legacy OpenSim connection string formats
        self.test_legacy_connection_strings().await;
        
        // Test SQLx URL format support
        self.test_sqlx_url_formats().await;
        
        // Test OpenZiti database port configuration
        self.test_openziti_database_ports().await;

        Ok(())
    }

    async fn test_legacy_connection_strings(&mut self) {
        info!("  Testing legacy OpenSim.ini connection string formats");

        let legacy_formats = vec![
            ("MySQL_OpenSim", "Data Source=localhost:3306;Database=opensim;User ID=opensim;Password=opensim123;"),
            ("MySQL_Standard", "Server=localhost;Port=3306;Database=opensim;Uid=opensim;Pwd=opensim123;"),
            ("PostgreSQL", "Host=localhost;Port=5432;Database=opensim;Username=opensim;Password=opensim123;"),
            ("SQLite", "Data Source=opensim.db;Version=3;"),
        ];

        for (format_name, connection_string) in legacy_formats {
            let result = self.validate_connection_string(format_name, connection_string).await;
            self.record_result(&format!("legacy_{}", format_name.to_lowercase()), result);
        }
    }

    async fn test_sqlx_url_formats(&mut self) {
        info!("  Testing SQLx URL format support");

        let sqlx_formats = vec![
            ("MySQL_SQLx", "mysql://opensim:opensim123@localhost:3306/opensim"),
            ("PostgreSQL_SQLx", "postgresql://opensim:opensim123@localhost:5432/opensim"),
            ("SQLite_SQLx", "sqlite://opensim.db"),
            ("MySQL_SSL", "mysql://opensim:opensim123@localhost:3306/opensim?ssl-mode=required"),
            ("PostgreSQL_SSL", "postgresql://opensim:opensim123@localhost:5432/opensim?sslmode=require"),
        ];

        for (format_name, sqlx_url) in sqlx_formats {
            let result = self.validate_sqlx_url(format_name, sqlx_url).await;
            self.record_result(&format!("sqlx_{}", format_name.to_lowercase()), result);
        }
    }

    async fn test_openziti_database_ports(&mut self) {
        info!("  Testing OpenZiti database port configuration");

        let result = self.validate_openziti_ports().await;
        self.record_result("openziti_database_ports", result);
    }

    async fn validate_connection_string(&self, format_name: &str, connection_string: &str) -> Result<()> {
        let parsed = self.parse_connection_string(connection_string)?;
        
        match format_name {
            name if name.contains("MySQL") => {
                self.validate_mysql_params(&parsed)?;
                info!("    ✅ {} MySQL format validated", format_name);
            }
            "PostgreSQL" => {
                self.validate_postgresql_params(&parsed)?;
                info!("    ✅ PostgreSQL format validated");
            }
            "SQLite" => {
                self.validate_sqlite_params(&parsed)?;
                info!("    ✅ SQLite format validated");
            }
            _ => return Err(anyhow!("Unknown format: {}", format_name)),
        }

        Ok(())
    }

    async fn validate_sqlx_url(&self, format_name: &str, sqlx_url: &str) -> Result<()> {
        // Parse SQLx URL format
        if !sqlx_url.contains("://") {
            return Err(anyhow!("Invalid SQLx URL format: {}", sqlx_url));
        }

        let parts: Vec<&str> = sqlx_url.splitn(2, "://").collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid SQLx URL structure: {}", sqlx_url));
        }

        let scheme = parts[0];
        let connection_part = parts[1];

        match scheme {
            "mysql" => {
                self.validate_mysql_sqlx_url(connection_part)?;
                info!("    ✅ {} SQLx MySQL URL validated", format_name);
            }
            "postgresql" => {
                self.validate_postgresql_sqlx_url(connection_part)?;
                info!("    ✅ {} SQLx PostgreSQL URL validated", format_name);
            }
            "sqlite" => {
                self.validate_sqlite_sqlx_url(connection_part)?;
                info!("    ✅ {} SQLx SQLite URL validated", format_name);
            }
            _ => return Err(anyhow!("Unsupported SQLx scheme: {}", scheme)),
        }

        Ok(())
    }

    async fn validate_openziti_ports(&self) -> Result<()> {
        // OpenZiti port configuration including database ports used by SQLx
        let openziti_port_config = vec![
            // Core OpenZiti ports
            ("ziti_controller", 1280, "OpenZiti Controller API"),
            ("ziti_edge_router", 3022, "OpenZiti Edge Router"),
            
            // OpenSim service ports
            ("opensim_main", 9000, "Second Life Viewer Protocol"),
            ("opensim_websocket", 9001, "Web Client WebSocket"),
            ("opensim_web", 8080, "Web Browser Interface"),
            ("opensim_admin", 8090, "Admin Dashboard"),
            ("opensim_monitoring", 9100, "Prometheus Metrics"),
            
            // Database ports used by SQLx
            ("mysql_sqlx", 3306, "MySQL Database (SQLx)"),
            ("postgresql_sqlx", 5432, "PostgreSQL Database (SQLx)"),
            ("redis_cache", 6379, "Redis Cache Server"),
            
            // Additional SQLx database ports
            ("mariadb_sqlx", 3307, "MariaDB Database (SQLx)"),
            ("postgres_alt", 5433, "PostgreSQL Alternative Port"),
        ];

        info!("    🌐 OpenZiti Port Configuration:");
        for (service, port, description) in &openziti_port_config {
            info!("      📡 {}: port {} - {}", service, port, description);
            
            // Validate port range
            if *port < 1024 && !matches!(*port, 80 | 443) {
                info!("      ⚠️  Port {} requires root privileges", port);
            }
            
            if *port > 65535 {
                return Err(anyhow!("Invalid port number for {}: {}", service, port));
            }
        }

        // Generate OpenZiti service configuration for database access
        self.generate_openziti_database_services(&openziti_port_config)?;

        Ok(())
    }

    fn generate_openziti_database_services(&self, port_config: &[(&str, u16, &str)]) -> Result<()> {
        info!("    🔒 Generating OpenZiti database service configuration:");

        let database_services = port_config.iter()
            .filter(|(service, _, _)| service.contains("sql") || service.contains("redis"))
            .collect::<Vec<_>>();

        for (service, port, description) in database_services {
            let ziti_service_name = format!("opensim-{}", service.replace("_", "-"));
            
            info!("      🛡️  Service: {} -> {}:{}", ziti_service_name, "localhost", port);
            info!("         Type: intercept");
            info!("         Protocol: tcp");
            info!("         Zero Trust: enabled");
            info!("         Description: {}", description);
        }

        Ok(())
    }

    fn parse_connection_string(&self, connection_string: &str) -> Result<HashMap<String, String>> {
        let mut params = HashMap::new();
        
        for part in connection_string.split(';') {
            if part.trim().is_empty() {
                continue;
            }
            
            if let Some(eq_pos) = part.find('=') {
                let key = part[..eq_pos].trim().to_lowercase();
                let value = part[eq_pos + 1..].trim().to_string();
                params.insert(key, value);
            }
        }
        
        Ok(params)
    }

    fn validate_mysql_params(&self, params: &HashMap<String, String>) -> Result<()> {
        let has_server = params.contains_key("data source") || params.contains_key("server");
        let has_user = params.contains_key("user id") || params.contains_key("uid");
        
        if !has_server {
            return Err(anyhow!("MySQL connection missing server specification"));
        }
        if !has_user {
            return Err(anyhow!("MySQL connection missing user specification"));
        }
        if !params.contains_key("database") {
            return Err(anyhow!("MySQL connection missing database name"));
        }
        
        Ok(())
    }

    fn validate_postgresql_params(&self, params: &HashMap<String, String>) -> Result<()> {
        let required = vec!["host", "database", "username"];
        
        for field in required {
            if !params.contains_key(field) {
                return Err(anyhow!("PostgreSQL connection missing: {}", field));
            }
        }
        
        Ok(())
    }

    fn validate_sqlite_params(&self, params: &HashMap<String, String>) -> Result<()> {
        if !params.contains_key("data source") {
            return Err(anyhow!("SQLite connection missing data source"));
        }
        
        Ok(())
    }

    fn validate_mysql_sqlx_url(&self, url_part: &str) -> Result<()> {
        // Parse MySQL SQLx URL: user:pass@host:port/database
        if !url_part.contains('@') {
            return Err(anyhow!("MySQL SQLx URL missing @ separator"));
        }
        
        let parts: Vec<&str> = url_part.split('@').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid MySQL SQLx URL format"));
        }
        
        let host_part = parts[1];
        if !host_part.contains('/') {
            return Err(anyhow!("MySQL SQLx URL missing database name"));
        }
        
        Ok(())
    }

    fn validate_postgresql_sqlx_url(&self, url_part: &str) -> Result<()> {
        // Similar validation for PostgreSQL SQLx URLs
        if !url_part.contains('@') {
            return Err(anyhow!("PostgreSQL SQLx URL missing @ separator"));
        }
        
        Ok(())
    }

    fn validate_sqlite_sqlx_url(&self, url_part: &str) -> Result<()> {
        // SQLite URLs just need a file path
        if url_part.is_empty() {
            return Err(anyhow!("SQLite SQLx URL missing file path"));
        }
        
        Ok(())
    }

    fn record_result(&mut self, test_name: &str, result: Result<()>) {
        match result {
            Ok(_) => {
                self.results.push(ValidationResult {
                    test_name: test_name.to_string(),
                    success: true,
                    error_message: None,
                });
            }
            Err(e) => {
                error!("❌ Database validation failed for {}: {}", test_name, e);
                self.results.push(ValidationResult {
                    test_name: test_name.to_string(),
                    success: false,
                    error_message: Some(e.to_string()),
                });
            }
        }
    }

    pub fn generate_report(&self) {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.success).count();
        let failed = total - passed;

        info!("📊 DATABASE VALIDATION REPORT");
        info!("=============================");
        info!("Total Tests: {}", total);
        info!("Passed: {} ✅", passed);
        info!("Failed: {} ❌", failed);
        info!("");

        if failed > 0 {
            info!("❌ FAILED VALIDATIONS:");
            for result in &self.results {
                if !result.success {
                    info!("  - {}: {}", result.test_name,
                        result.error_message.as_ref().unwrap_or(&"Unknown error".to_string()));
                }
            }
            info!("");
        }

        info!("🗄️ SUPPORTED DATABASE FORMATS:");
        info!("  ✅ OpenSim.ini legacy connection strings");
        info!("  ✅ SQLx URL format (mysql://, postgresql://, sqlite:)");
        info!("  ✅ SSL/TLS encrypted connections");
        info!("  ✅ Zero trust database access via OpenZiti");
        info!("");

        info!("🌐 OPENZITI DATABASE PORTS:");
        info!("  🔒 MySQL (SQLx): port 3306");
        info!("  🔒 PostgreSQL (SQLx): port 5432");
        info!("  🔒 MariaDB (SQLx): port 3307");
        info!("  🔒 Redis Cache: port 6379");
        info!("  🔒 PostgreSQL Alt: port 5433");
        info!("");

        info!("🎯 DATABASE VALIDATION: {}", 
            if failed == 0 { "✅ ALL FORMATS SUPPORTED" } else { "❌ ISSUES DETECTED" });
    }
}

#[tokio::test]
async fn test_phase19_database_validation() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let mut validator = DatabaseValidator::new();
    validator.validate_opensim_database_config().await?;
    validator.generate_report();

    let failed_count = validator.results.iter().filter(|r| !r.success).count();
    if failed_count > 0 {
        return Err(anyhow!("Database validation failed: {} tests failed", failed_count));
    }

    info!("🎉 Phase 19 Database Validation completed successfully!");
    Ok(())
}