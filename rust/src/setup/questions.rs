//! Setup question definitions and data structures
//!
//! Defines all the interactive questions asked during setup,
//! based on the OpenSim master configuration flow.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

/// Types of questions that can be asked
#[derive(Debug, Clone, PartialEq)]
pub enum QuestionType {
    String,
    StringNotEmpty,
    Integer,
    Boolean,
    IpAddress,
    Port,
    Uuid,
    Choice(Vec<String>),
    DatabaseUrl,
}

/// Individual setup question
#[derive(Debug, Clone)]
pub struct SetupQuestion {
    pub key: String,
    pub prompt: String,
    pub description: Option<String>,
    pub default_value: String,
    pub question_type: QuestionType,
    pub required: bool,
    pub validator: Option<fn(&str) -> Result<(), String>>,
    pub depends_on: Option<String>, // Only ask if this other question has specific value
    pub depends_value: Option<String>,
}

impl SetupQuestion {
    pub fn new(key: &str, prompt: &str) -> Self {
        Self {
            key: key.to_string(),
            prompt: prompt.to_string(),
            description: None,
            default_value: String::new(),
            question_type: QuestionType::String,
            required: false,
            validator: None,
            depends_on: None,
            depends_value: None,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_default(mut self, default: &str) -> Self {
        self.default_value = default.to_string();
        self
    }

    pub fn with_type(mut self, question_type: QuestionType) -> Self {
        self.question_type = question_type;
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn depends_on(mut self, key: &str, value: &str) -> Self {
        self.depends_on = Some(key.to_string());
        self.depends_value = Some(value.to_string());
        self
    }

    pub fn with_validator(mut self, validator: fn(&str) -> Result<(), String>) -> Self {
        self.validator = Some(validator);
        self
    }
}

/// Complete configuration collected from setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    // Grid information
    pub grid_name: String,
    pub admin_first_name: String,
    pub admin_last_name: String,
    pub admin_email: String,
    pub admin_password: String,

    // Database configuration
    pub database_provider: String,
    pub database_connection: String,

    // Network configuration
    pub http_port: u16,
    pub region_port_start: u16,
    pub external_hostname: String,

    // Region configuration
    pub region_count: u32,

    // Features
    pub enable_hypergrid: bool,
    pub enable_ossl: bool,
    pub ossl_threat_level: String,

    // Setup metadata
    pub preset: crate::setup::SetupPreset,

    // Legacy fields (for backward compatibility)
    pub setup_mode: String,
    pub region_name: String,
    pub region_uuid: String,
    pub region_location: String,
    pub region_size_x: u32,
    pub region_size_y: u32,
    pub region_size_z: u32,
    pub internal_address: String,
    pub internal_port: u16,
    pub resolve_address: bool,
    pub database_type: String,
    pub database_url: String,
    pub database_create: bool,
    pub grid_mode: bool,
    pub grid_uri: Option<String>,
    pub asset_service_uri: Option<String>,
    pub inventory_service_uri: Option<String>,
    pub user_service_uri: Option<String>,
    pub physics_engine: String,
    pub script_engine: String,
    pub https_enabled: bool,
    pub https_port: Option<u16>,
    pub log_level: String,
    pub max_threads: Option<u32>,
    pub enable_monitoring: bool,
    pub additional_config: HashMap<String, String>,
}

impl Default for SetupConfig {
    fn default() -> Self {
        Self {
            // Primary fields
            grid_name: "My OpenSim Grid".to_string(),
            admin_first_name: "Admin".to_string(),
            admin_last_name: "User".to_string(),
            admin_email: "admin@localhost".to_string(),
            admin_password: "admin123".to_string(),
            database_provider: "SQLite".to_string(),
            database_connection: "URI=file:opensim.db,version=3".to_string(),
            http_port: 8080,
            region_port_start: 9000,
            external_hostname: "127.0.0.1".to_string(),
            region_count: 1,
            enable_hypergrid: false,
            enable_ossl: true,
            ossl_threat_level: "Moderate".to_string(),
            preset: crate::setup::SetupPreset::Standalone,

            // Legacy fields
            setup_mode: "standalone".to_string(),
            region_name: "My Region".to_string(),
            region_uuid: Uuid::new_v4().to_string(),
            region_location: "1000,1000".to_string(),
            region_size_x: 256,
            region_size_y: 256,
            region_size_z: 4096,
            internal_address: "0.0.0.0".to_string(),
            internal_port: 9000,
            resolve_address: false,
            database_type: "sqlite".to_string(),
            database_url: "./opensim.db".to_string(),
            database_create: true,
            grid_mode: false,
            grid_uri: None,
            asset_service_uri: None,
            inventory_service_uri: None,
            user_service_uri: None,
            physics_engine: "ubODE".to_string(),
            script_engine: "XEngine".to_string(),
            https_enabled: false,
            https_port: None,
            log_level: "INFO".to_string(),
            max_threads: None,
            enable_monitoring: true,
            additional_config: HashMap::new(),
        }
    }
}

/// Get all setup questions in the correct order
pub fn get_setup_questions() -> Vec<SetupQuestion> {
    vec![
        // Setup mode selection
        SetupQuestion::new("setup_mode", "Setup mode")
            .with_description("Choose the type of OpenSim setup")
            .with_type(QuestionType::Choice(vec![
                "standalone".to_string(),
                "grid-region".to_string(),
                "grid-robust".to_string(),
            ]))
            .with_default("standalone")
            .required(),
        // Preset selection
        SetupQuestion::new("preset", "Configuration preset")
            .with_description("Choose a preset configuration")
            .with_type(QuestionType::Choice(vec![
                "development".to_string(),
                "production".to_string(),
                "custom".to_string(),
            ]))
            .with_default("development"),
        // Region configuration
        SetupQuestion::new("region_name", "Region name")
            .with_description("Enter a name for your region")
            .with_type(QuestionType::StringNotEmpty)
            .with_default("My Region")
            .required(),
        SetupQuestion::new("region_location", "Region location")
            .with_description("Region coordinates in the grid (X,Y format)")
            .with_default("1000,1000")
            .with_validator(validate_region_location),
        SetupQuestion::new("internal_address", "Internal IP address")
            .with_description("IP address for the region server to listen on")
            .with_type(QuestionType::IpAddress)
            .with_default("0.0.0.0"),
        SetupQuestion::new("internal_port", "Internal port")
            .with_description("Port for the region server to listen on")
            .with_type(QuestionType::Port)
            .with_default("9000"),
        SetupQuestion::new("external_hostname", "External hostname")
            .with_description("Hostname or IP that clients will connect to")
            .with_default("SYSTEMIP"),
        SetupQuestion::new("resolve_address", "Resolve hostname to IP")
            .with_description("Resolve hostname to IP on start (useful for Docker)")
            .with_type(QuestionType::Boolean)
            .with_default("false"),
        // Database configuration
        SetupQuestion::new("database_type", "Database type")
            .with_description("Choose the database backend to use")
            .with_type(QuestionType::Choice(vec![
                "sqlite".to_string(),
                "postgresql".to_string(),
                "mysql".to_string(),
                "mariadb".to_string(),
            ]))
            .with_default("sqlite"),
        SetupQuestion::new("database_url", "Database connection")
            .with_description("Database connection string or file path")
            .with_type(QuestionType::DatabaseUrl)
            .with_default("./opensim.db"),
        // Grid configuration (only for grid mode)
        SetupQuestion::new("grid_uri", "Grid URI")
            .with_description("URI of the grid services (Robust server)")
            .depends_on("setup_mode", "grid-region")
            .with_default("http://localhost:8003"),
        // Advanced configuration
        SetupQuestion::new("physics_engine", "Physics engine")
            .with_description("Physics simulation engine to use")
            .with_type(QuestionType::Choice(vec![
                "ubODE".to_string(),
                "Bullet".to_string(),
                "basicphysics".to_string(),
            ]))
            .with_default("ubODE"),
        SetupQuestion::new("script_engine", "Script engine")
            .with_description("Scripting engine for LSL scripts")
            .with_type(QuestionType::Choice(vec![
                "XEngine".to_string(),
                "YEngine".to_string(),
            ]))
            .with_default("XEngine"),
        SetupQuestion::new("log_level", "Log level")
            .with_description("Logging verbosity level")
            .with_type(QuestionType::Choice(vec![
                "ERROR".to_string(),
                "WARN".to_string(),
                "INFO".to_string(),
                "DEBUG".to_string(),
            ]))
            .with_default("INFO"),
        SetupQuestion::new("enable_monitoring", "Enable monitoring")
            .with_description("Enable Prometheus metrics and health monitoring")
            .with_type(QuestionType::Boolean)
            .with_default("true"),
    ]
}

// Validation functions
fn validate_region_location(input: &str) -> Result<(), String> {
    let parts: Vec<&str> = input.split(',').collect();
    if parts.len() != 2 {
        return Err("Region location must be in X,Y format (e.g., 1000,1000)".to_string());
    }

    for part in parts {
        if part.trim().parse::<u32>().is_err() {
            return Err("Region coordinates must be positive integers".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SetupConfig::default();
        assert_eq!(config.setup_mode, "standalone");
        assert_eq!(config.region_name, "My Region");
        assert_eq!(config.internal_port, 9000);
    }

    #[test]
    fn test_validate_region_location() {
        assert!(validate_region_location("1000,1000").is_ok());
        assert!(validate_region_location("0,0").is_ok());
        assert!(validate_region_location("invalid").is_err());
        assert!(validate_region_location("1000").is_err());
    }

    #[test]
    fn test_question_builder() {
        let question = SetupQuestion::new("test", "Test question")
            .with_description("A test")
            .with_default("default")
            .required();

        assert_eq!(question.key, "test");
        assert_eq!(question.prompt, "Test question");
        assert_eq!(question.default_value, "default");
        assert!(question.required);
    }
}
