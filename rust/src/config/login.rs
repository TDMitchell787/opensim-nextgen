//! Login service configuration management
//! Handles configuration loading, validation, and defaults for the login system

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::UdpSocket;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub fn resolve_system_ip() -> String {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Login service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginConfig {
    /// Server binding configuration
    pub server: ServerConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Session management configuration
    pub session: SessionConfig,

    /// Region configuration for login responses
    pub region: RegionConfig,

    /// Inventory configuration
    pub inventory: InventoryConfig,

    /// Capabilities configuration
    pub capabilities: CapabilitiesConfig,

    /// UI and messaging configuration
    pub ui: UiConfig,
}

/// Server binding and network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// HTTP bind address for XMLRPC login
    pub bind_address: String,

    /// UDP port for simulator connection
    pub udp_port: u16,

    /// Base URL for capabilities and other services
    pub base_url: String,

    /// Simulator IP address sent to viewers
    pub sim_ip: String,

    /// Maximum concurrent sessions
    pub max_sessions: usize,

    /// Request timeout in seconds
    pub request_timeout: u64,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable MD5 password verification
    pub enable_md5_auth: bool,

    /// Password hash prefix validation
    pub require_hash_prefix: bool,

    /// Maximum login attempts per IP
    pub max_login_attempts: u32,

    /// Login attempt window in minutes
    pub attempt_window_minutes: u32,

    /// Enable authentication bypass for testing
    pub bypass_auth_for_testing: bool,
}

/// Session management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout in minutes
    pub timeout_minutes: i64,

    /// Session cleanup interval in minutes
    pub cleanup_interval_minutes: u64,

    /// Maximum session duration in hours
    pub max_duration_hours: i64,

    /// Enable session persistence
    pub enable_persistence: bool,
}

/// Region configuration for login responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    /// Default region name
    pub default_region_name: String,

    /// Default region coordinates
    pub region_x: i32,
    pub region_y: i32,

    /// Region size
    pub region_size_x: i32,
    pub region_size_y: i32,

    /// Default spawn position
    pub default_position: Position,

    /// Default look at direction
    pub default_look_at: Position,
}

/// Inventory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryConfig {
    /// Enable inventory system
    pub enabled: bool,

    /// Default inventory folders
    pub default_folders: Vec<InventoryFolder>,

    /// Library inventory enabled
    pub enable_library: bool,

    /// Maximum inventory items per user
    pub max_items_per_user: u32,
}

/// Capabilities configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesConfig {
    /// Enable capabilities system
    pub enabled: bool,

    /// Base capabilities URL
    pub base_url: String,

    /// Capability timeout in seconds
    pub timeout_seconds: u64,

    /// Available capabilities
    pub available_caps: Vec<String>,
}

/// UI and messaging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Welcome message shown on login
    pub welcome_message: String,

    /// Login page title
    pub login_page_title: String,

    /// Grid name displayed to users
    pub grid_name: String,

    /// Support contact information
    pub support_contact: String,

    /// Terms of Service URL
    pub tos_url: Option<String>,
}

/// 3D position structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Inventory folder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryFolder {
    pub name: String,
    pub type_default: String,
    pub version: String,
}

impl Default for LoginConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            session: SessionConfig::default(),
            region: RegionConfig::default(),
            inventory: InventoryConfig::default(),
            capabilities: CapabilitiesConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        let ip = resolve_system_ip();
        Self {
            bind_address: "0.0.0.0:9000".to_string(),
            udp_port: 9000,
            base_url: format!("http://{}:9000", ip),
            sim_ip: ip,
            max_sessions: 1000,
            request_timeout: 30,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable_md5_auth: true,
            require_hash_prefix: true,
            max_login_attempts: 5,
            attempt_window_minutes: 15,
            bypass_auth_for_testing: false,
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_minutes: 30,
            cleanup_interval_minutes: 5,
            max_duration_hours: 24,
            enable_persistence: true,
        }
    }
}

impl Default for RegionConfig {
    fn default() -> Self {
        Self {
            default_region_name: "OpenSim Next".to_string(),
            region_x: 1000 * 256,
            region_y: 1000 * 256,
            region_size_x: 256,
            region_size_y: 256,
            default_position: Position {
                x: 128.0,
                y: 128.0,
                z: 25.0,
            },
            default_look_at: Position {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }
}

impl Default for InventoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_folders: vec![
                InventoryFolder {
                    name: "My Inventory".to_string(),
                    type_default: "8".to_string(),
                    version: "1".to_string(),
                },
                InventoryFolder {
                    name: "Objects".to_string(),
                    type_default: "6".to_string(),
                    version: "1".to_string(),
                },
                InventoryFolder {
                    name: "Textures".to_string(),
                    type_default: "0".to_string(),
                    version: "1".to_string(),
                },
                InventoryFolder {
                    name: "Clothing".to_string(),
                    type_default: "5".to_string(),
                    version: "1".to_string(),
                },
                InventoryFolder {
                    name: "Gestures".to_string(),
                    type_default: "21".to_string(),
                    version: "1".to_string(),
                },
            ],
            enable_library: true,
            max_items_per_user: 50000,
        }
    }
}

impl Default for CapabilitiesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "http://127.0.0.1:9000/cap".to_string(),
            timeout_seconds: 60,
            available_caps: vec![
                "seed_capability".to_string(),
                "event_queue".to_string(),
                "upload_baked_texture".to_string(),
                "texture_upload".to_string(),
                "mesh_upload".to_string(),
            ],
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            welcome_message: "Welcome to OpenSim Next!".to_string(),
            login_page_title: "OpenSim Next Login".to_string(),
            grid_name: "OpenSim Next Grid".to_string(),
            support_contact: "admin@opensim-next.local".to_string(),
            tos_url: None,
        }
    }
}

/// Login configuration manager
pub struct LoginConfigManager {
    config: LoginConfig,
}

impl LoginConfigManager {
    /// Create new config manager with default configuration
    pub fn new() -> Self {
        info!("Initializing login configuration with defaults");
        Self {
            config: LoginConfig::default(),
        }
    }

    /// Load configuration from file
    pub fn load_from_file(file_path: &str) -> Result<Self> {
        info!("Loading login configuration from: {}", file_path);

        let config_content = std::fs::read_to_string(file_path)
            .map_err(|e| anyhow!("Failed to read config file {}: {}", file_path, e))?;

        let config: LoginConfig = if file_path.ends_with(".toml") {
            toml::from_str(&config_content)
                .map_err(|e| anyhow!("Failed to parse TOML config: {}", e))?
        } else if file_path.ends_with(".json") {
            serde_json::from_str(&config_content)
                .map_err(|e| anyhow!("Failed to parse JSON config: {}", e))?
        } else {
            return Err(anyhow!(
                "Unsupported config file format. Use .toml or .json"
            ));
        };

        info!("Successfully loaded login configuration");
        Ok(Self { config })
    }

    /// Load configuration from environment variables
    pub fn load_from_env() -> Result<Self> {
        info!("Loading login configuration from environment variables");

        let mut config = LoginConfig::default();

        // Server configuration
        if let Ok(bind_addr) = std::env::var("OPENSIM_LOGIN_BIND_ADDRESS") {
            config.server.bind_address = bind_addr;
        }

        if let Ok(udp_port) =
            std::env::var("OPENSIM_LOGIN_UDP_PORT").or_else(|_| std::env::var("OPENSIM_LOGIN_PORT"))
        {
            let port: u16 = udp_port
                .parse()
                .map_err(|e| anyhow!("Invalid UDP port: {}", e))?;
            config.server.udp_port = port;
            // Update base_url and capabilities base_url to match the new port
            config.server.base_url = format!("http://{}:{}", config.server.sim_ip, port);
            config.capabilities.base_url = format!("http://{}:{}/cap", config.server.sim_ip, port);
            info!(
                "Login port set to {} — base_url updated to {}",
                port, config.server.base_url
            );
        }

        if let Ok(base_url) = std::env::var("OPENSIM_LOGIN_BASE_URL") {
            config.server.base_url = base_url;
        }

        if let Ok(sim_ip) = std::env::var("OPENSIM_LOGIN_SIM_IP") {
            config.server.sim_ip = sim_ip;
        }

        // Authentication configuration
        if let Ok(bypass_auth) = std::env::var("OPENSIM_LOGIN_BYPASS_AUTH") {
            config.auth.bypass_auth_for_testing = bypass_auth.parse().unwrap_or(false);
        }

        // Session configuration
        if let Ok(timeout) = std::env::var("OPENSIM_LOGIN_SESSION_TIMEOUT") {
            config.session.timeout_minutes = timeout
                .parse()
                .map_err(|e| anyhow!("Invalid session timeout: {}", e))?;
        }

        // UI configuration
        if let Ok(welcome) = std::env::var("OPENSIM_LOGIN_WELCOME_MESSAGE") {
            config.ui.welcome_message = welcome;
        }

        if let Ok(grid_name) = std::env::var("OPENSIM_LOGIN_GRID_NAME") {
            config.ui.grid_name = grid_name;
        }

        info!("Successfully loaded configuration from environment");
        Ok(Self { config })
    }

    /// Get the current configuration
    pub fn config(&self) -> &LoginConfig {
        &self.config
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        debug!("Validating login configuration");

        // Validate server configuration
        if self.config.server.bind_address.is_empty() {
            return Err(anyhow!("Server bind address cannot be empty"));
        }

        if self.config.server.udp_port == 0 {
            return Err(anyhow!("UDP port must be greater than 0"));
        }

        if self.config.server.base_url.is_empty() {
            return Err(anyhow!("Base URL cannot be empty"));
        }

        // Validate URL format
        if let Err(e) = url::Url::parse(&self.config.server.base_url) {
            return Err(anyhow!("Invalid base URL format: {}", e));
        }

        // Validate session timeout
        if self.config.session.timeout_minutes <= 0 {
            return Err(anyhow!("Session timeout must be greater than 0"));
        }

        // Validate region coordinates
        if self.config.region.region_size_x <= 0 || self.config.region.region_size_y <= 0 {
            return Err(anyhow!("Region size must be greater than 0"));
        }

        info!("Login configuration validation successful");
        Ok(())
    }

    /// Save configuration to file
    pub fn save_to_file(&self, file_path: &str) -> Result<()> {
        info!("Saving login configuration to: {}", file_path);

        let config_content = if file_path.ends_with(".toml") {
            toml::to_string_pretty(&self.config)
                .map_err(|e| anyhow!("Failed to serialize to TOML: {}", e))?
        } else if file_path.ends_with(".json") {
            serde_json::to_string_pretty(&self.config)
                .map_err(|e| anyhow!("Failed to serialize to JSON: {}", e))?
        } else {
            return Err(anyhow!(
                "Unsupported config file format. Use .toml or .json"
            ));
        };

        std::fs::write(file_path, config_content)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        info!("Successfully saved login configuration");
        Ok(())
    }

    /// Get configuration summary for logging
    pub fn get_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();

        summary.insert(
            "bind_address".to_string(),
            self.config.server.bind_address.clone(),
        );
        summary.insert(
            "udp_port".to_string(),
            self.config.server.udp_port.to_string(),
        );
        summary.insert("base_url".to_string(), self.config.server.base_url.clone());
        summary.insert("sim_ip".to_string(), self.config.server.sim_ip.clone());
        summary.insert(
            "max_sessions".to_string(),
            self.config.server.max_sessions.to_string(),
        );
        summary.insert(
            "session_timeout".to_string(),
            self.config.session.timeout_minutes.to_string(),
        );
        summary.insert("grid_name".to_string(), self.config.ui.grid_name.clone());
        summary.insert(
            "inventory_enabled".to_string(),
            self.config.inventory.enabled.to_string(),
        );
        summary.insert(
            "capabilities_enabled".to_string(),
            self.config.capabilities.enabled.to_string(),
        );
        summary.insert(
            "auth_bypass".to_string(),
            self.config.auth.bypass_auth_for_testing.to_string(),
        );

        summary
    }
}

impl Default for LoginConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config_creation() {
        let config = LoginConfig::default();

        assert_eq!(config.server.bind_address, "0.0.0.0:9000");
        assert_eq!(config.server.udp_port, 9001);
        assert_eq!(config.ui.grid_name, "OpenSim Next Grid");
        assert!(config.inventory.enabled);
        assert!(config.capabilities.enabled);
    }

    #[test]
    fn test_config_validation() {
        let manager = LoginConfigManager::new();
        assert!(manager.validate().is_ok());

        let mut invalid_config = LoginConfig::default();
        invalid_config.server.bind_address = "".to_string();

        let invalid_manager = LoginConfigManager {
            config: invalid_config,
        };
        assert!(invalid_manager.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = LoginConfig::default();

        // Test JSON serialization
        let json_str = serde_json::to_string(&config).unwrap();
        let deserialized: LoginConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(config.server.udp_port, deserialized.server.udp_port);

        // Test TOML serialization
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: LoginConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.server.udp_port, deserialized.server.udp_port);
    }

    #[test]
    fn test_config_file_operations() {
        let manager = LoginConfigManager::new();

        // Test JSON save/load
        let mut json_file = NamedTempFile::new().unwrap();
        let json_path = json_file.path().with_extension("json");
        manager.save_to_file(json_path.to_str().unwrap()).unwrap();

        let loaded_manager =
            LoginConfigManager::load_from_file(json_path.to_str().unwrap()).unwrap();
        assert_eq!(
            manager.config.server.udp_port,
            loaded_manager.config.server.udp_port
        );

        // Test TOML save/load
        let mut toml_file = NamedTempFile::new().unwrap();
        let toml_path = toml_file.path().with_extension("toml");
        manager.save_to_file(toml_path.to_str().unwrap()).unwrap();

        let loaded_manager =
            LoginConfigManager::load_from_file(toml_path.to_str().unwrap()).unwrap();
        assert_eq!(
            manager.config.server.udp_port,
            loaded_manager.config.server.udp_port
        );
    }

    #[test]
    fn test_environment_config_loading() {
        std::env::set_var("OPENSIM_LOGIN_UDP_PORT", "9001");
        std::env::set_var("OPENSIM_LOGIN_GRID_NAME", "Test Grid");
        std::env::set_var("OPENSIM_LOGIN_BYPASS_AUTH", "true");

        let manager = LoginConfigManager::load_from_env().unwrap();

        assert_eq!(manager.config.server.udp_port, 9001);
        assert_eq!(manager.config.ui.grid_name, "Test Grid");
        assert!(manager.config.auth.bypass_auth_for_testing);

        // Clean up
        std::env::remove_var("OPENSIM_LOGIN_UDP_PORT");
        std::env::remove_var("OPENSIM_LOGIN_GRID_NAME");
        std::env::remove_var("OPENSIM_LOGIN_BYPASS_AUTH");
    }

    #[test]
    fn test_config_summary() {
        let manager = LoginConfigManager::new();
        let summary = manager.get_summary();

        assert!(summary.contains_key("bind_address"));
        assert!(summary.contains_key("udp_port"));
        assert!(summary.contains_key("grid_name"));
        assert_eq!(summary.get("udp_port").unwrap(), "9001");
    }
}
