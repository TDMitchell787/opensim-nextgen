//! Instance Configuration Loader
//!
//! Parses the instances.toml configuration file and provides
//! access to instance definitions for the multi-instance manager.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn};

use super::types::{AuthMethod, Environment};

/// Top-level configuration structure for instances.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstancesConfig {
    pub controller: ControllerConfig,
    #[serde(default)]
    pub instances: Vec<InstanceConfig>,
}

/// Controller settings for the management dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    #[serde(default = "default_discovery_mode")]
    pub discovery_mode: String,
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_ms: u64,
    #[serde(default = "default_heartbeat_timeout")]
    pub heartbeat_timeout_ms: u64,
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay_ms: u64,
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,
    #[serde(default = "default_command_timeout")]
    pub command_timeout_ms: u64,
    #[serde(default = "default_controller_port")]
    pub controller_port: u16,
    #[serde(default = "default_instances_base_dir")]
    pub instances_base_dir: String,
    #[serde(default)]
    pub binary_path: String,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            discovery_mode: default_discovery_mode(),
            health_check_interval_ms: default_health_check_interval(),
            heartbeat_timeout_ms: default_heartbeat_timeout(),
            reconnect_delay_ms: default_reconnect_delay(),
            max_reconnect_attempts: default_max_reconnect_attempts(),
            command_timeout_ms: default_command_timeout(),
            controller_port: default_controller_port(),
            instances_base_dir: default_instances_base_dir(),
            binary_path: String::new(),
        }
    }
}

fn default_discovery_mode() -> String { "config".to_string() }
fn default_health_check_interval() -> u64 { 5000 }
fn default_heartbeat_timeout() -> u64 { 15000 }
fn default_reconnect_delay() -> u64 { 3000 }
fn default_max_reconnect_attempts() -> u32 { 5 }
fn default_command_timeout() -> u64 { 30000 }
fn default_controller_port() -> u16 { 9300 }
fn default_instances_base_dir() -> String { "./Instances".to_string() }

/// Configuration for a single instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub host: String,
    #[serde(default = "default_websocket_port")]
    pub websocket_port: u16,
    #[serde(default = "default_admin_port")]
    pub admin_port: u16,
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_udp_port")]
    pub udp_port: u16,
    pub api_key: String,
    #[serde(default)]
    pub environment: Environment,
    #[serde(default = "default_auto_connect")]
    pub auto_connect: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub authentication: AuthenticationConfig,
    #[serde(default)]
    pub tls: TlsConfig,
}

fn default_websocket_port() -> u16 { 9001 }
fn default_admin_port() -> u16 { 9200 }
fn default_metrics_port() -> u16 { 9100 }
fn default_http_port() -> u16 { 9000 }
fn default_udp_port() -> u16 { 9000 }
fn default_auto_connect() -> bool { true }

/// Authentication configuration for an instance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    #[serde(default)]
    pub method: AuthMethod,
    pub credentials_file: Option<String>,
}

/// TLS configuration for an instance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlsConfig {
    #[serde(default)]
    pub enabled: bool,
    pub ca_cert_path: Option<String>,
    #[serde(default = "default_verify_host")]
    pub verify_host: bool,
}

fn default_verify_host() -> bool { true }

impl InstanceConfig {
    /// Get the WebSocket URL for this instance
    pub fn websocket_url(&self) -> String {
        let protocol = if self.tls.enabled { "wss" } else { "ws" };
        format!("{}://{}:{}", protocol, self.host, self.websocket_port)
    }

    /// Get the Admin API URL for this instance
    pub fn admin_url(&self) -> String {
        let protocol = if self.tls.enabled { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.host, self.admin_port)
    }

    /// Get the Metrics URL for this instance
    pub fn metrics_url(&self) -> String {
        let protocol = if self.tls.enabled { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.host, self.metrics_port)
    }

    /// Get the HTTP URL for this instance
    pub fn http_url(&self) -> String {
        let protocol = if self.tls.enabled { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.host, self.http_port)
    }

    /// Check if this instance should auto-connect on startup
    pub fn should_auto_connect(&self) -> bool {
        self.auto_connect
    }

    /// Get API key, checking environment variable override first
    pub fn get_api_key(&self) -> String {
        let env_key = format!("OPENSIM_INSTANCE_{}_API_KEY", self.id.to_uppercase().replace('-', "_"));
        std::env::var(&env_key).unwrap_or_else(|_| self.api_key.clone())
    }

    /// Get host, checking environment variable override first
    pub fn get_host(&self) -> String {
        let env_key = format!("OPENSIM_INSTANCE_{}_HOST", self.id.to_uppercase().replace('-', "_"));
        std::env::var(&env_key).unwrap_or_else(|_| self.host.clone())
    }
}

/// Load instances configuration from a TOML file
pub fn load_instances_config<P: AsRef<Path>>(path: P) -> Result<InstancesConfig> {
    let path = path.as_ref();

    info!("Loading instances configuration from: {}", path.display());

    if !path.exists() {
        return Err(anyhow!("Instances configuration file not found: {}", path.display()));
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read instances config: {}", path.display()))?;

    let config: InstancesConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse instances config: {}", path.display()))?;

    validate_config(&config)?;

    info!(
        "Loaded {} instance(s) with discovery mode: {}",
        config.instances.len(),
        config.controller.discovery_mode
    );

    for instance in &config.instances {
        info!(
            "  - {} ({}) at {} [{}]",
            instance.name,
            instance.id,
            instance.host,
            if instance.auto_connect { "auto-connect" } else { "manual" }
        );
    }

    Ok(config)
}

/// Load instances configuration with a default path
pub fn load_default_instances_config() -> Result<InstancesConfig> {
    let default_paths = [
        "instances.toml",
        "config/instances.toml",
        "../instances.toml",
    ];

    for path in &default_paths {
        if Path::new(path).exists() {
            return load_instances_config(path);
        }
    }

    warn!("No instances.toml found, using empty configuration");
    Ok(InstancesConfig {
        controller: ControllerConfig::default(),
        instances: Vec::new(),
    })
}

/// Validate the configuration for correctness
fn validate_config(config: &InstancesConfig) -> Result<()> {
    let mut ids = std::collections::HashSet::new();

    for instance in &config.instances {
        if instance.id.is_empty() {
            return Err(anyhow!("Instance ID cannot be empty"));
        }

        if instance.name.is_empty() {
            return Err(anyhow!("Instance '{}' name cannot be empty", instance.id));
        }

        if instance.host.is_empty() {
            return Err(anyhow!("Instance '{}' host cannot be empty", instance.id));
        }

        if instance.api_key.is_empty() {
            return Err(anyhow!("Instance '{}' api_key cannot be empty", instance.id));
        }

        if !ids.insert(&instance.id) {
            return Err(anyhow!("Duplicate instance ID: {}", instance.id));
        }

        if instance.tls.enabled && instance.tls.ca_cert_path.is_some() {
            let cert_path = instance.tls.ca_cert_path.as_ref().unwrap();
            if !Path::new(cert_path).exists() {
                warn!(
                    "Instance '{}' TLS CA cert not found: {}",
                    instance.id, cert_path
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ControllerConfig::default();
        assert_eq!(config.discovery_mode, "config");
        assert_eq!(config.health_check_interval_ms, 5000);
    }

    #[test]
    fn test_instance_urls() {
        let instance = InstanceConfig {
            id: "test".to_string(),
            name: "Test Instance".to_string(),
            description: "".to_string(),
            host: "localhost".to_string(),
            websocket_port: 9001,
            admin_port: 9200,
            metrics_port: 9100,
            http_port: 9000,
            udp_port: 9000,
            api_key: "test-key".to_string(),
            environment: Environment::Development,
            auto_connect: true,
            tags: vec![],
            authentication: AuthenticationConfig::default(),
            tls: TlsConfig::default(),
        };

        assert_eq!(instance.websocket_url(), "ws://localhost:9001");
        assert_eq!(instance.admin_url(), "http://localhost:9200");
        assert_eq!(instance.metrics_url(), "http://localhost:9100");
    }

    #[test]
    fn test_instance_urls_with_tls() {
        let instance = InstanceConfig {
            id: "test".to_string(),
            name: "Test Instance".to_string(),
            description: "".to_string(),
            host: "secure.example.com".to_string(),
            websocket_port: 9001,
            admin_port: 9200,
            metrics_port: 9100,
            http_port: 9000,
            udp_port: 9000,
            api_key: "test-key".to_string(),
            environment: Environment::Production,
            auto_connect: true,
            tags: vec![],
            authentication: AuthenticationConfig::default(),
            tls: TlsConfig {
                enabled: true,
                ca_cert_path: None,
                verify_host: true,
            },
        };

        assert_eq!(instance.websocket_url(), "wss://secure.example.com:9001");
        assert_eq!(instance.admin_url(), "https://secure.example.com:9200");
    }

    #[test]
    fn test_parse_toml() {
        let toml_content = r#"
[controller]
discovery_mode = "config"
health_check_interval_ms = 5000

[[instances]]
id = "test-1"
name = "Test Instance"
host = "localhost"
api_key = "test-key"
environment = "development"
        "#;

        let config: InstancesConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.instances.len(), 1);
        assert_eq!(config.instances[0].id, "test-1");
        assert_eq!(config.instances[0].websocket_port, 9001); // default
    }
}
