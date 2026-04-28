//! Input validation and sanity checking
//!
//! Validates configuration settings and provides helpful error messages
//! for common configuration issues.

use anyhow::{anyhow, Result};
use std::net::{IpAddr, ToSocketAddrs};
use std::path::Path;
use tracing::{info, warn};

use crate::setup::questions::SetupConfig;

/// Configuration validator
pub struct Validator {
    strict_mode: bool,
}

impl Validator {
    /// Create a new validator
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    /// Create validator in strict mode (more rigorous validation)
    pub fn strict() -> Self {
        Self { strict_mode: true }
    }

    /// Validate complete setup configuration
    pub fn validate(&self, config: &SetupConfig) -> Result<()> {
        info!("Validating OpenSim Next configuration");

        // Validate basic settings
        self.validate_basic_settings(config)?;

        // Validate region configuration
        self.validate_region_config(config)?;

        // Validate network configuration
        self.validate_network_config(config)?;

        // Validate database configuration
        self.validate_database_config(config)?;

        // Validate grid configuration (if applicable)
        if config.grid_mode {
            self.validate_grid_config(config)?;
        }

        // Validate performance settings
        self.validate_performance_config(config)?;

        info!("Configuration validation passed");
        Ok(())
    }

    /// Validate basic setup settings
    fn validate_basic_settings(&self, config: &SetupConfig) -> Result<()> {
        // Validate setup mode
        let valid_modes = ["standalone", "grid-region", "grid-robust"];
        if !valid_modes.contains(&config.setup_mode.as_str()) {
            return Err(anyhow!(
                "Invalid setup mode: {}. Must be one of: {}",
                config.setup_mode,
                valid_modes.join(", ")
            ));
        }

        // Validate physics engine
        let valid_physics = ["ubODE", "Bullet", "basicphysics"];
        if !valid_physics.contains(&config.physics_engine.as_str()) {
            return Err(anyhow!(
                "Invalid physics engine: {}. Must be one of: {}",
                config.physics_engine,
                valid_physics.join(", ")
            ));
        }

        // Validate script engine
        let valid_scripts = ["XEngine", "YEngine"];
        if !valid_scripts.contains(&config.script_engine.as_str()) {
            return Err(anyhow!(
                "Invalid script engine: {}. Must be one of: {}",
                config.script_engine,
                valid_scripts.join(", ")
            ));
        }

        // Validate log level
        let valid_levels = ["ERROR", "WARN", "INFO", "DEBUG"];
        if !valid_levels.contains(&config.log_level.as_str()) {
            return Err(anyhow!(
                "Invalid log level: {}. Must be one of: {}",
                config.log_level,
                valid_levels.join(", ")
            ));
        }

        Ok(())
    }

    /// Validate region configuration
    fn validate_region_config(&self, config: &SetupConfig) -> Result<()> {
        // Validate region name
        if config.region_name.trim().is_empty() {
            return Err(anyhow!("Region name cannot be empty"));
        }

        if config.region_name.len() > 64 {
            return Err(anyhow!("Region name too long (max 64 characters)"));
        }

        // Check for invalid characters in region name
        if config
            .region_name
            .contains(['|', ',', ';', ':', '"', '\'', '\\', '/', '<', '>'])
        {
            return Err(anyhow!("Region name contains invalid characters"));
        }

        // Validate region UUID
        if uuid::Uuid::parse_str(&config.region_uuid).is_err() {
            return Err(anyhow!("Invalid region UUID format"));
        }

        // Validate region location
        self.validate_region_location(&config.region_location)?;

        // Validate region size
        if config.region_size_x < 32 || config.region_size_x > 8192 {
            return Err(anyhow!(
                "Invalid region size X: {}. Must be between 32 and 8192",
                config.region_size_x
            ));
        }

        if config.region_size_y < 32 || config.region_size_y > 8192 {
            return Err(anyhow!(
                "Invalid region size Y: {}. Must be between 32 and 8192",
                config.region_size_y
            ));
        }

        if config.region_size_z < 256 || config.region_size_z > 8192 {
            return Err(anyhow!(
                "Invalid region size Z: {}. Must be between 256 and 8192",
                config.region_size_z
            ));
        }

        // Check if region size is power of 2 (recommended)
        if self.strict_mode {
            if !self.is_power_of_two(config.region_size_x)
                || !self.is_power_of_two(config.region_size_y)
            {
                warn!("Region size should be power of 2 for optimal performance");
            }
        }

        Ok(())
    }

    /// Validate network configuration
    fn validate_network_config(&self, config: &SetupConfig) -> Result<()> {
        // Validate internal IP address
        if config.internal_address != "0.0.0.0" && config.internal_address != "SYSTEMIP" {
            if config.internal_address.parse::<IpAddr>().is_err() {
                return Err(anyhow!(
                    "Invalid internal IP address: {}",
                    config.internal_address
                ));
            }
        }

        // Validate ports
        if config.internal_port == 0 || config.internal_port > 65535 {
            return Err(anyhow!(
                "Invalid internal port: {}. Must be between 1 and 65535",
                config.internal_port
            ));
        }

        if config.http_port == 0 || config.http_port > 65535 {
            return Err(anyhow!(
                "Invalid HTTP port: {}. Must be between 1 and 65535",
                config.http_port
            ));
        }

        // Check for port conflicts
        if config.internal_port == config.http_port {
            return Err(anyhow!("Internal port and HTTP port cannot be the same"));
        }

        // Validate HTTPS configuration
        if config.https_enabled {
            if let Some(https_port) = config.https_port {
                if https_port == 0 || https_port > 65535 {
                    return Err(anyhow!(
                        "Invalid HTTPS port: {}. Must be between 1 and 65535",
                        https_port
                    ));
                }
                if https_port == config.http_port || https_port == config.internal_port {
                    return Err(anyhow!("HTTPS port conflicts with other ports"));
                }
            } else {
                return Err(anyhow!("HTTPS enabled but no HTTPS port specified"));
            }
        }

        // Validate external hostname
        if config.external_hostname.trim().is_empty() {
            return Err(anyhow!("External hostname cannot be empty"));
        }

        // If not using SYSTEMIP, validate hostname format
        if config.external_hostname != "SYSTEMIP" && self.strict_mode {
            self.validate_hostname(&config.external_hostname)?;
        }

        Ok(())
    }

    /// Validate database configuration
    fn validate_database_config(&self, config: &SetupConfig) -> Result<()> {
        // Validate database type
        let valid_types = ["sqlite", "postgresql", "mysql", "mariadb"];
        if !valid_types.contains(&config.database_type.as_str()) {
            return Err(anyhow!(
                "Invalid database type: {}. Must be one of: {}",
                config.database_type,
                valid_types.join(", ")
            ));
        }

        // Validate database URL/connection string
        if config.database_url.trim().is_empty() {
            return Err(anyhow!("Database URL cannot be empty"));
        }

        // Type-specific validation
        match config.database_type.as_str() {
            "sqlite" => {
                self.validate_sqlite_path(&config.database_url)?;
            }
            "postgresql" => {
                self.validate_postgres_url(&config.database_url)?;
            }
            "mysql" | "mariadb" => {
                self.validate_mysql_url(&config.database_url)?;
            }
            _ => {} // Already validated above
        }

        Ok(())
    }

    /// Validate grid configuration
    fn validate_grid_config(&self, config: &SetupConfig) -> Result<()> {
        if config.setup_mode == "grid-region" {
            // Grid URI is required for grid regions
            let grid_uri = config
                .grid_uri
                .as_ref()
                .ok_or_else(|| anyhow!("Grid URI is required for grid-region mode"))?;

            if grid_uri.trim().is_empty() {
                return Err(anyhow!("Grid URI cannot be empty"));
            }

            // Validate URI format
            if !grid_uri.starts_with("http://") && !grid_uri.starts_with("https://") {
                return Err(anyhow!("Grid URI must start with http:// or https://"));
            }

            // Try to parse as URL
            if url::Url::parse(grid_uri).is_err() {
                return Err(anyhow!("Invalid Grid URI format"));
            }
        }

        Ok(())
    }

    /// Validate performance configuration
    fn validate_performance_config(&self, config: &SetupConfig) -> Result<()> {
        // Validate max threads if specified
        if let Some(max_threads) = config.max_threads {
            if max_threads == 0 {
                return Err(anyhow!("Max threads cannot be zero"));
            }
            if max_threads > 1000 {
                warn!(
                    "Very high thread count ({}) may impact performance",
                    max_threads
                );
            }
        }

        Ok(())
    }

    // Helper validation methods

    fn validate_region_location(&self, location: &str) -> Result<()> {
        let parts: Vec<&str> = location.split(',').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Region location must be in X,Y format"));
        }

        for part in parts {
            let coord: u32 = part
                .trim()
                .parse()
                .map_err(|_| anyhow!("Region coordinates must be positive integers"))?;

            // Check coordinate bounds (OpenSim limit)
            if coord > 1000000 {
                return Err(anyhow!("Region coordinate too large (max 1,000,000)"));
            }
        }

        Ok(())
    }

    fn validate_hostname(&self, hostname: &str) -> Result<()> {
        // Basic hostname validation
        if hostname.len() > 253 {
            return Err(anyhow!("Hostname too long (max 253 characters)"));
        }

        // Check if it's an IP address
        if hostname.parse::<IpAddr>().is_ok() {
            return Ok(());
        }

        // Check hostname format
        for label in hostname.split('.') {
            if label.is_empty() || label.len() > 63 {
                return Err(anyhow!("Invalid hostname format"));
            }

            if !label.chars().all(|c| c.is_alphanumeric() || c == '-') {
                return Err(anyhow!("Hostname contains invalid characters"));
            }

            if label.starts_with('-') || label.ends_with('-') {
                return Err(anyhow!("Hostname labels cannot start or end with hyphen"));
            }
        }

        Ok(())
    }

    fn validate_sqlite_path(&self, path: &str) -> Result<()> {
        // Check if path is reasonable
        if path.contains("..") && self.strict_mode {
            warn!("SQLite path contains '..' which may be insecure");
        }

        // Check if directory exists (if path contains directory)
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(anyhow!(
                    "SQLite database directory does not exist: {}",
                    parent.display()
                ));
            }
        }

        Ok(())
    }

    fn validate_postgres_url(&self, url: &str) -> Result<()> {
        // Basic PostgreSQL URL validation
        if !url.starts_with("postgresql://") && !url.starts_with("postgres://") {
            return Err(anyhow!(
                "PostgreSQL URL must start with postgresql:// or postgres://"
            ));
        }

        // Try to parse as URL
        if url::Url::parse(url).is_err() {
            return Err(anyhow!("Invalid PostgreSQL URL format"));
        }

        Ok(())
    }

    fn validate_mysql_url(&self, url: &str) -> Result<()> {
        // Basic MySQL URL validation
        if url.starts_with("mysql://") {
            // URL format
            if url::Url::parse(url).is_err() {
                return Err(anyhow!("Invalid MySQL URL format"));
            }
        } else if url.contains("Data Source") || url.contains("Server") {
            // Connection string format - basic validation
            if !url.contains("Database") && !url.contains("Initial Catalog") {
                return Err(anyhow!("MySQL connection string must specify database"));
            }
        } else {
            return Err(anyhow!(
                "MySQL URL must be either mysql:// format or connection string"
            ));
        }

        Ok(())
    }

    fn is_power_of_two(&self, n: u32) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SetupConfig {
        SetupConfig {
            setup_mode: "standalone".to_string(),
            region_name: "Test Region".to_string(),
            region_uuid: uuid::Uuid::new_v4().to_string(),
            region_location: "1000,1000".to_string(),
            region_size_x: 256,
            region_size_y: 256,
            region_size_z: 4096,
            internal_address: "0.0.0.0".to_string(),
            internal_port: 9000,
            external_hostname: "SYSTEMIP".to_string(),
            database_type: "sqlite".to_string(),
            database_url: "./test.db".to_string(),
            http_port: 8080,
            ..Default::default()
        }
    }

    #[test]
    fn test_valid_config() {
        let validator = Validator::new();
        let config = create_test_config();
        assert!(validator.validate(&config).is_ok());
    }

    #[test]
    fn test_invalid_setup_mode() {
        let validator = Validator::new();
        let mut config = create_test_config();
        config.setup_mode = "invalid".to_string();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_empty_region_name() {
        let validator = Validator::new();
        let mut config = create_test_config();
        config.region_name = "".to_string();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_invalid_region_location() {
        let validator = Validator::new();
        let mut config = create_test_config();
        config.region_location = "invalid".to_string();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_port_conflict() {
        let validator = Validator::new();
        let mut config = create_test_config();
        config.http_port = config.internal_port; // Same port
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_invalid_database_type() {
        let validator = Validator::new();
        let mut config = create_test_config();
        config.database_type = "invalid".to_string();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_region_location_validation() {
        let validator = Validator::new();

        assert!(validator.validate_region_location("1000,1000").is_ok());
        assert!(validator.validate_region_location("0,0").is_ok());
        assert!(validator.validate_region_location("invalid").is_err());
        assert!(validator.validate_region_location("1000").is_err());
        assert!(validator.validate_region_location("1000,2000000").is_err()); // Too large
    }

    #[test]
    fn test_hostname_validation() {
        let validator = Validator::new();

        assert!(validator.validate_hostname("example.com").is_ok());
        assert!(validator.validate_hostname("192.168.1.1").is_ok());
        assert!(validator.validate_hostname("sub.example.com").is_ok());
        assert!(validator.validate_hostname("-invalid.com").is_err());
        assert!(validator.validate_hostname("invalid-.com").is_err());
    }

    #[test]
    fn test_postgres_url_validation() {
        let validator = Validator::new();

        assert!(validator
            .validate_postgres_url("postgresql://user:pass@localhost/db")
            .is_ok());
        assert!(validator
            .validate_postgres_url("postgres://user:pass@localhost/db")
            .is_ok());
        assert!(validator
            .validate_postgres_url("mysql://user:pass@localhost/db")
            .is_err());
        assert!(validator.validate_postgres_url("invalid-url").is_err());
    }

    #[test]
    fn test_is_power_of_two() {
        let validator = Validator::new();

        assert!(validator.is_power_of_two(256));
        assert!(validator.is_power_of_two(512));
        assert!(!validator.is_power_of_two(300));
        assert!(!validator.is_power_of_two(0));
    }
}
