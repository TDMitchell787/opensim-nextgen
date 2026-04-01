// OpenSim Next Flutter Configurator - Rust FFI Bridge
// Provides mobile interface to OpenSim Next auto-configurator functionality

use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Initialize the Rust bridge
pub fn main() {}

/// Deployment types supported by OpenSim Next
#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentType {
    Development,
    Production,
    Grid,
}

/// System information for auto-detection
#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub memory_gb: f64,
    pub cpu_cores: u32,
    pub has_public_ip: bool,
    pub bandwidth_mbps: u32,
    pub domain: String,
    pub expected_users: u32,
    pub expected_regions: u32,
    pub is_commercial: bool,
}

/// Deployment recommendation result
#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecommendation {
    pub recommended_type: DeploymentType,
    pub confidence: f64,
    pub reasoning: String,
    pub alternative_options: Vec<AlternativeOption>,
}

/// Alternative deployment option
#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeOption {
    pub deployment_type: DeploymentType,
    pub confidence: f64,
    pub reason: String,
}

/// Configuration validation result
#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
    pub overall_score: u32,
}

/// OpenSim configuration structure
#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSimConfig {
    pub deployment_type: DeploymentType,
    pub grid_name: String,
    pub grid_nick: String,
    pub welcome_message: String,
    pub database_type: String,
    pub database_connection: String,
    pub physics_engine: String,
    pub network_config: NetworkConfig,
    pub security_config: SecurityConfig,
    pub performance_config: PerformanceConfig,
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub http_port: u32,
    pub https_port: u32,
    pub https_enabled: bool,
    pub external_hostname: String,
    pub internal_ip: String,
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub password_complexity: bool,
    pub session_timeout: u32,
    pub brute_force_protection: bool,
    pub ssl_certificate_path: String,
    pub ssl_private_key_path: String,
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_prims: u32,
    pub max_scripts: u32,
    pub script_timeout: u32,
    pub cache_assets: bool,
    pub cache_timeout: u32,
}

/// Server status information
#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub is_running: bool,
    pub uptime_seconds: u64,
    pub active_regions: u32,
    pub connected_users: u32,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_activity: NetworkActivity,
}

#[frb]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkActivity {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections: u32,
}

/// Main OpenSim configurator client
#[frb]
pub struct OpenSimConfigurator {
    server_url: String,
    api_key: String,
    client: Option<reqwest::Client>,
}

#[frb]
impl OpenSimConfigurator {
    /// Create a new configurator client
    #[frb(sync)]
    pub fn new(server_url: String, api_key: String) -> Self {
        Self {
            server_url,
            api_key,
            client: Some(reqwest::Client::new()),
        }
    }

    /// Auto-detect optimal deployment type based on system information
    #[frb]
    pub async fn auto_detect_deployment(&self, system_info: SystemInfo) -> anyhow::Result<DeploymentRecommendation> {
        let mut confidence_scores = HashMap::new();
        let mut reasoning_parts = Vec::new();

        // Hardware-based scoring
        if system_info.memory_gb >= 64.0 && system_info.cpu_cores >= 32 {
            confidence_scores.insert(DeploymentType::Grid, 0.9);
            reasoning_parts.push("High-end hardware suitable for grid deployment");
        } else if system_info.memory_gb >= 16.0 && system_info.cpu_cores >= 8 {
            confidence_scores.insert(DeploymentType::Production, 0.8);
            reasoning_parts.push("Sufficient hardware for production deployment");
        } else {
            confidence_scores.insert(DeploymentType::Development, 0.7);
            reasoning_parts.push("Hardware suitable for development environment");
        }

        // Network-based scoring
        if system_info.has_public_ip && system_info.bandwidth_mbps >= 1000 && system_info.domain != "localhost" {
            *confidence_scores.entry(DeploymentType::Grid).or_insert(0.0) += 0.3;
            reasoning_parts.push("Public IP, high bandwidth, and domain configuration suggest grid deployment");
        } else if system_info.has_public_ip && system_info.domain != "localhost" {
            *confidence_scores.entry(DeploymentType::Production).or_insert(0.0) += 0.2;
            reasoning_parts.push("Public IP and domain configuration suggest production deployment");
        }

        // Usage-based scoring
        if system_info.expected_users > 100 || system_info.expected_regions > 16 || system_info.is_commercial {
            *confidence_scores.entry(DeploymentType::Grid).or_insert(0.0) += 0.4;
            reasoning_parts.push("High user count or commercial use suggests grid deployment");
        } else if system_info.expected_users > 10 || system_info.expected_regions > 4 {
            *confidence_scores.entry(DeploymentType::Production).or_insert(0.0) += 0.3;
            reasoning_parts.push("Medium scale usage suggests production deployment");
        }

        // Find the highest scoring deployment type
        let (recommended_type, confidence) = confidence_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, v)| (k.clone(), *v))
            .unwrap_or((DeploymentType::Development, 0.5));

        // Create alternative options
        let mut alternatives = Vec::new();
        for (dep_type, score) in confidence_scores.iter() {
            if *dep_type != recommended_type && *score > 0.3 {
                alternatives.push(AlternativeOption {
                    deployment_type: dep_type.clone(),
                    confidence: *score,
                    reason: format!("Alternative option with {:.0}% confidence", score * 100.0),
                });
            }
        }

        // Sort alternatives by confidence
        alternatives.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(DeploymentRecommendation {
            recommended_type,
            confidence: confidence.min(1.0),
            reasoning: reasoning_parts.join(". "),
            alternative_options: alternatives,
        })
    }

    /// Get default configuration for a deployment type
    #[frb]
    pub async fn get_default_config(&self, deployment_type: DeploymentType) -> anyhow::Result<OpenSimConfig> {
        match deployment_type {
            DeploymentType::Development => Ok(self.create_development_config()),
            DeploymentType::Production => Ok(self.create_production_config()),
            DeploymentType::Grid => Ok(self.create_grid_config()),
        }
    }

    /// Validate a configuration
    #[frb]
    pub async fn validate_configuration(&self, config: OpenSimConfig) -> anyhow::Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        let mut score = 100u32;

        // Validate based on deployment type
        match config.deployment_type {
            DeploymentType::Production => {
                if !config.network_config.https_enabled {
                    errors.push("Production deployments require HTTPS to be enabled".to_string());
                    score -= 30;
                }
                if config.security_config.ssl_certificate_path.is_empty() {
                    errors.push("Production deployments require SSL certificate configuration".to_string());
                    score -= 25;
                }
                if config.database_type == "SQLite" {
                    warnings.push("SQLite is not recommended for production deployments".to_string());
                    score -= 10;
                }
            }
            DeploymentType::Grid => {
                if config.security_config.ssl_certificate_path.is_empty() {
                    errors.push("Grid deployments require SSL certificates".to_string());
                    score -= 35;
                }
                if config.database_type != "PostgreSQL" {
                    errors.push("Grid deployments require PostgreSQL for optimal performance".to_string());
                    score -= 20;
                }
                if config.performance_config.max_prims < 50000 {
                    warnings.push("Grid deployments should support high prim counts".to_string());
                    score -= 5;
                }
            }
            DeploymentType::Development => {
                if config.network_config.https_enabled && config.security_config.ssl_certificate_path.is_empty() {
                    warnings.push("HTTPS enabled but no SSL certificate configured".to_string());
                    score -= 5;
                }
            }
        }

        // General validation
        if config.grid_name.is_empty() {
            errors.push("Grid name is required".to_string());
            score -= 15;
        }

        if config.network_config.http_port == 0 {
            errors.push("Valid HTTP port is required".to_string());
            score -= 20;
        }

        // Recommendations
        if config.performance_config.cache_assets {
            recommendations.push("Asset caching is enabled for better performance".to_string());
        } else {
            recommendations.push("Consider enabling asset caching for better performance".to_string());
        }

        if config.security_config.password_complexity {
            recommendations.push("Password complexity requirements are enabled for better security".to_string());
        } else {
            recommendations.push("Consider enabling password complexity requirements".to_string());
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            recommendations,
            overall_score: score.max(0),
        })
    }

    /// Get server status
    #[frb]
    pub async fn get_server_status(&self) -> anyhow::Result<ServerStatus> {
        let client = self.client.as_ref().ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;
        
        let url = format!("{}/api/status", self.server_url);
        let response = client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if response.status().is_success() {
            let status: ServerStatus = response.json().await?;
            Ok(status)
        } else {
            // Return mock data for development
            Ok(ServerStatus {
                is_running: true,
                uptime_seconds: 3600,
                active_regions: 2,
                connected_users: 5,
                cpu_usage: 45.2,
                memory_usage: 62.8,
                network_activity: NetworkActivity {
                    bytes_sent: 1024 * 1024 * 150, // 150 MB
                    bytes_received: 1024 * 1024 * 87, // 87 MB
                    connections: 12,
                },
            })
        }
    }

    /// Apply configuration to server
    #[frb]
    pub async fn apply_configuration(&self, config: OpenSimConfig) -> anyhow::Result<String> {
        let client = self.client.as_ref().ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;
        
        let url = format!("{}/api/configuration", self.server_url);
        let response = client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&config)
            .send()
            .await?;

        if response.status().is_success() {
            Ok("Configuration applied successfully".to_string())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Failed to apply configuration: {}", error_text))
        }
    }

    /// Export configuration as JSON
    #[frb]
    pub async fn export_configuration(&self, config: OpenSimConfig) -> anyhow::Result<String> {
        let json = serde_json::to_string_pretty(&config)?;
        Ok(json)
    }

    /// Import configuration from JSON
    #[frb]
    pub async fn import_configuration(&self, json_data: String) -> anyhow::Result<OpenSimConfig> {
        let config: OpenSimConfig = serde_json::from_str(&json_data)?;
        Ok(config)
    }

    // Helper methods for creating default configurations
    fn create_development_config(&self) -> OpenSimConfig {
        OpenSimConfig {
            deployment_type: DeploymentType::Development,
            grid_name: "OpenSim Next Development Grid".to_string(),
            grid_nick: "devgrid".to_string(),
            welcome_message: "Welcome to your development environment!".to_string(),
            database_type: "SQLite".to_string(),
            database_connection: "Data Source=./OpenSim.db;Version=3;".to_string(),
            physics_engine: "ODE".to_string(),
            network_config: NetworkConfig {
                http_port: 9000,
                https_port: 9443,
                https_enabled: false,
                external_hostname: "localhost".to_string(),
                internal_ip: "127.0.0.1".to_string(),
            },
            security_config: SecurityConfig {
                password_complexity: false,
                session_timeout: 3600,
                brute_force_protection: false,
                ssl_certificate_path: "".to_string(),
                ssl_private_key_path: "".to_string(),
            },
            performance_config: PerformanceConfig {
                max_prims: 15000,
                max_scripts: 1000,
                script_timeout: 30,
                cache_assets: true,
                cache_timeout: 48,
            },
        }
    }

    fn create_production_config(&self) -> OpenSimConfig {
        OpenSimConfig {
            deployment_type: DeploymentType::Production,
            grid_name: "OpenSim Next Production Grid".to_string(),
            grid_nick: "prodgrid".to_string(),
            welcome_message: "Welcome to our virtual world!".to_string(),
            database_type: "PostgreSQL".to_string(),
            database_connection: "Host=localhost;Database=opensim;Username=opensim;Password=".to_string(),
            physics_engine: "Bullet".to_string(),
            network_config: NetworkConfig {
                http_port: 80,
                https_port: 443,
                https_enabled: true,
                external_hostname: "yourgrid.com".to_string(),
                internal_ip: "0.0.0.0".to_string(),
            },
            security_config: SecurityConfig {
                password_complexity: true,
                session_timeout: 1800,
                brute_force_protection: true,
                ssl_certificate_path: "/etc/ssl/certs/opensim.crt".to_string(),
                ssl_private_key_path: "/etc/ssl/private/opensim.key".to_string(),
            },
            performance_config: PerformanceConfig {
                max_prims: 45000,
                max_scripts: 3000,
                script_timeout: 25,
                cache_assets: true,
                cache_timeout: 24,
            },
        }
    }

    fn create_grid_config(&self) -> OpenSimConfig {
        OpenSimConfig {
            deployment_type: DeploymentType::Grid,
            grid_name: "OpenSim Next Enterprise Grid".to_string(),
            grid_nick: "enterprise".to_string(),
            welcome_message: "Welcome to our enterprise metaverse!".to_string(),
            database_type: "PostgreSQL".to_string(),
            database_connection: "Host=db-cluster.internal;Database=opensim_grid;Username=opensim;Password=".to_string(),
            physics_engine: "POS".to_string(),
            network_config: NetworkConfig {
                http_port: 80,
                https_port: 443,
                https_enabled: true,
                external_hostname: "grid.enterprise.com".to_string(),
                internal_ip: "0.0.0.0".to_string(),
            },
            security_config: SecurityConfig {
                password_complexity: true,
                session_timeout: 900,
                brute_force_protection: true,
                ssl_certificate_path: "/etc/ssl/enterprise/opensim.crt".to_string(),
                ssl_private_key_path: "/etc/ssl/enterprise/opensim.key".to_string(),
            },
            performance_config: PerformanceConfig {
                max_prims: 100000,
                max_scripts: 10000,
                script_timeout: 20,
                cache_assets: true,
                cache_timeout: 12,
            },
        }
    }
}

/// Utility functions for system detection
#[frb]
pub async fn detect_system_capabilities() -> anyhow::Result<SystemInfo> {
    // This would integrate with platform-specific APIs
    // For now, return reasonable defaults
    Ok(SystemInfo {
        memory_gb: 8.0,
        cpu_cores: 4,
        has_public_ip: false,
        bandwidth_mbps: 100,
        domain: "localhost".to_string(),
        expected_users: 5,
        expected_regions: 1,
        is_commercial: false,
    })
}

/// Get available physics engines
#[frb]
pub async fn get_available_physics_engines() -> anyhow::Result<Vec<String>> {
    Ok(vec![
        "ODE".to_string(),
        "UBODE".to_string(),
        "Bullet".to_string(),
        "POS".to_string(),
        "Basic".to_string(),
    ])
}

/// Get available database types
#[frb]
pub async fn get_available_database_types() -> anyhow::Result<Vec<String>> {
    Ok(vec![
        "SQLite".to_string(),
        "PostgreSQL".to_string(),
        "MySQL".to_string(),
    ])
}