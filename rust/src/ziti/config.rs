//! OpenZiti Configuration for Zero Trust Networking
//!
//! Manages configuration for OpenZiti network connectivity, identity,
//! and security policies.

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

/// OpenZiti network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiConfig {
    /// Controller endpoint URL
    pub controller_url: String,
    
    /// Identity configuration
    pub identity: ZitiIdentityConfig,
    
    /// Network configuration
    pub network: ZitiNetworkConfig,
    
    /// Security configuration
    pub security: ZitiSecurityConfig,
    
    /// Logging configuration
    pub logging: ZitiLoggingConfig,
    
    /// Service configuration
    pub services: ZitiServicesConfig,
    
    /// Custom configuration options
    pub custom: HashMap<String, String>,
}

/// Identity configuration for zero trust authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiIdentityConfig {
    /// Identity certificate file path
    pub cert_file: PathBuf,
    
    /// Identity private key file path
    pub key_file: PathBuf,
    
    /// CA bundle file path
    pub ca_bundle: Option<PathBuf>,
    
    /// Identity name
    pub name: String,
    
    /// Identity type
    pub identity_type: ZitiIdentityType,
    
    /// Auto-enrollment configuration
    pub auto_enroll: Option<ZitiAutoEnrollConfig>,
    
    /// Identity refresh interval in seconds
    pub refresh_interval: u64,
    
    /// Enable identity caching
    pub enable_caching: bool,
}

/// Types of OpenZiti identities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiIdentityType {
    /// Device identity for endpoints
    Device,
    /// Service identity for services
    Service,
    /// Router identity for edge routers
    Router,
    /// Controller identity for controllers
    Controller,
}

/// Auto-enrollment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiAutoEnrollConfig {
    /// Enrollment token
    pub token: String,
    
    /// Enrollment endpoint
    pub endpoint: String,
    
    /// Certificate validity period in days
    pub cert_validity_days: u32,
    
    /// Enable automatic renewal
    pub auto_renew: bool,
}

/// Network configuration for zero trust connectivity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiNetworkConfig {
    /// Edge router endpoints
    pub edge_routers: Vec<String>,
    
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    
    /// Keep-alive interval in seconds
    pub keepalive_interval: u64,
    
    /// Maximum retry attempts
    pub max_retries: u32,
    
    /// Retry backoff strategy
    pub retry_backoff: ZitiRetryBackoff,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Buffer sizes
    pub buffer_config: ZitiBufferConfig,
    
    /// Tunnel configuration
    pub tunnel: ZitiTunnelConfig,
}

/// Retry backoff strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiRetryBackoff {
    /// Fixed delay between retries
    Fixed { delay_ms: u64 },
    /// Exponential backoff
    Exponential { initial_ms: u64, max_ms: u64, multiplier: f64 },
    /// Linear backoff
    Linear { initial_ms: u64, increment_ms: u64 },
}

/// Buffer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiBufferConfig {
    /// Receive buffer size
    pub receive_buffer_size: usize,
    
    /// Send buffer size
    pub send_buffer_size: usize,
    
    /// Maximum message size
    pub max_message_size: usize,
    
    /// Enable buffer pooling
    pub enable_pooling: bool,
}

/// Tunnel configuration for overlay network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiTunnelConfig {
    /// Enable tunnel mode
    pub enabled: bool,
    
    /// Tunnel interface name
    pub interface_name: String,
    
    /// MTU size
    pub mtu: u16,
    
    /// DNS configuration
    pub dns: ZitiDnsConfig,
    
    /// IP assignment
    pub ip_assignment: ZitiIpAssignment,
}

/// DNS configuration for zero trust networking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiDnsConfig {
    /// Enable DNS interception
    pub enabled: bool,
    
    /// DNS servers
    pub servers: Vec<String>,
    
    /// DNS search domains
    pub search_domains: Vec<String>,
    
    /// DNS timeout in seconds
    pub timeout: u64,
}

/// IP assignment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiIpAssignment {
    /// Assignment strategy
    pub strategy: ZitiIpStrategy,
    
    /// IP range for assignment
    pub ip_range: String,
    
    /// Subnet mask
    pub subnet_mask: String,
    
    /// Gateway IP
    pub gateway: Option<String>,
}

/// IP assignment strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiIpStrategy {
    /// Dynamic assignment
    Dynamic,
    /// Static assignment
    Static,
    /// DHCP-based assignment
    Dhcp,
}

/// Security configuration for zero trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiSecurityConfig {
    /// Minimum TLS version
    pub min_tls_version: ZitiTlsVersion,
    
    /// Allowed cipher suites
    pub cipher_suites: Vec<String>,
    
    /// Certificate verification
    pub cert_verification: ZitiCertVerification,
    
    /// Encryption configuration
    pub encryption: ZitiEncryptionConfig,
    
    /// Rate limiting
    pub rate_limiting: ZitiRateLimitConfig,
    
    /// Session configuration
    pub session: ZitiSessionConfig,
}

/// TLS version requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiTlsVersion {
    #[serde(rename = "1.2")]
    Tls12,
    #[serde(rename = "1.3")]
    Tls13,
}

/// Certificate verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiCertVerification {
    /// Verify certificate chain
    pub verify_chain: bool,
    
    /// Verify hostname
    pub verify_hostname: bool,
    
    /// Allow self-signed certificates
    pub allow_self_signed: bool,
    
    /// Custom CA certificates
    pub custom_cas: Vec<PathBuf>,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiEncryptionConfig {
    /// Default encryption algorithm
    pub default_algorithm: String,
    
    /// Key exchange algorithm
    pub key_exchange: String,
    
    /// Enable perfect forward secrecy
    pub perfect_forward_secrecy: bool,
    
    /// Key rotation interval in hours
    pub key_rotation_hours: u64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiRateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    
    /// Requests per second limit
    pub requests_per_second: u32,
    
    /// Burst limit
    pub burst_limit: u32,
    
    /// Window size in seconds
    pub window_size: u64,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiSessionConfig {
    /// Session timeout in seconds
    pub timeout: u64,
    
    /// Enable session persistence
    pub persistent: bool,
    
    /// Session storage location
    pub storage_path: Option<PathBuf>,
    
    /// Maximum concurrent sessions
    pub max_concurrent: u32,
}

/// Logging configuration for OpenZiti
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiLoggingConfig {
    /// Log level
    pub level: ZitiLogLevel,
    
    /// Log output destination
    pub output: ZitiLogOutput,
    
    /// Log format
    pub format: ZitiLogFormat,
    
    /// Enable structured logging
    pub structured: bool,
    
    /// Log rotation configuration
    pub rotation: Option<ZitiLogRotation>,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Log output destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiLogOutput {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// File output
    File { path: PathBuf },
    /// Syslog output
    Syslog { facility: String },
}

/// Log formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiLogFormat {
    /// Plain text format
    Text,
    /// JSON format
    Json,
    /// Structured format
    Structured,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiLogRotation {
    /// Maximum file size in MB
    pub max_size_mb: u64,
    
    /// Maximum number of files to keep
    pub max_files: u32,
    
    /// Rotation interval
    pub interval: ZitiRotationInterval,
}

/// Log rotation intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiRotationInterval {
    Daily,
    Weekly,
    Monthly,
    Size,
}

/// Services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiServicesConfig {
    /// Service discovery configuration
    pub discovery: ZitiServiceDiscovery,
    
    /// Default service timeout
    pub default_timeout: u64,
    
    /// Health check configuration
    pub health_check: ZitiHealthCheckConfig,
    
    /// Load balancing configuration
    pub load_balancing: ZitiLoadBalancingConfig,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiServiceDiscovery {
    /// Enable automatic discovery
    pub enabled: bool,
    
    /// Discovery interval in seconds
    pub interval: u64,
    
    /// Service cache TTL in seconds
    pub cache_ttl: u64,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiHealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    
    /// Health check interval in seconds
    pub interval: u64,
    
    /// Health check timeout in seconds
    pub timeout: u64,
    
    /// Failure threshold
    pub failure_threshold: u32,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiLoadBalancingConfig {
    /// Load balancing strategy
    pub strategy: ZitiLoadBalancingStrategy,
    
    /// Enable session affinity
    pub session_affinity: bool,
    
    /// Health-based routing
    pub health_based_routing: bool,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiLoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    Random,
    HealthBased,
}

impl Default for ZitiConfig {
    fn default() -> Self {
        Self {
            controller_url: "https://controller.ziti.local:1280".to_string(),
            identity: ZitiIdentityConfig::default(),
            network: ZitiNetworkConfig::default(),
            security: ZitiSecurityConfig::default(),
            logging: ZitiLoggingConfig::default(),
            services: ZitiServicesConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl Default for ZitiIdentityConfig {
    fn default() -> Self {
        Self {
            cert_file: PathBuf::from("ziti/identity.cert"),
            key_file: PathBuf::from("ziti/identity.key"),
            ca_bundle: Some(PathBuf::from("ziti/ca-bundle.cert")),
            name: "opensim-next".to_string(),
            identity_type: ZitiIdentityType::Device,
            auto_enroll: None,
            refresh_interval: 3600, // 1 hour
            enable_caching: true,
        }
    }
}

impl Default for ZitiNetworkConfig {
    fn default() -> Self {
        Self {
            edge_routers: vec!["edge-router.ziti.local:3022".to_string()],
            connect_timeout: 30,
            keepalive_interval: 60,
            max_retries: 3,
            retry_backoff: ZitiRetryBackoff::Exponential {
                initial_ms: 1000,
                max_ms: 30000,
                multiplier: 2.0,
            },
            enable_compression: true,
            buffer_config: ZitiBufferConfig::default(),
            tunnel: ZitiTunnelConfig::default(),
        }
    }
}

impl Default for ZitiBufferConfig {
    fn default() -> Self {
        Self {
            receive_buffer_size: 65536,  // 64KB
            send_buffer_size: 65536,     // 64KB
            max_message_size: 1048576,   // 1MB
            enable_pooling: true,
        }
    }
}

impl Default for ZitiTunnelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interface_name: "ziti0".to_string(),
            mtu: 1500,
            dns: ZitiDnsConfig::default(),
            ip_assignment: ZitiIpAssignment::default(),
        }
    }
}

impl Default for ZitiDnsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            search_domains: vec![],
            timeout: 5,
        }
    }
}

impl Default for ZitiIpAssignment {
    fn default() -> Self {
        Self {
            strategy: ZitiIpStrategy::Dynamic,
            ip_range: "100.64.0.0/10".to_string(),
            subnet_mask: "255.192.0.0".to_string(),
            gateway: None,
        }
    }
}

impl Default for ZitiSecurityConfig {
    fn default() -> Self {
        Self {
            min_tls_version: ZitiTlsVersion::Tls13,
            cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_AES_128_GCM_SHA256".to_string(),
            ],
            cert_verification: ZitiCertVerification::default(),
            encryption: ZitiEncryptionConfig::default(),
            rate_limiting: ZitiRateLimitConfig::default(),
            session: ZitiSessionConfig::default(),
        }
    }
}

impl Default for ZitiCertVerification {
    fn default() -> Self {
        Self {
            verify_chain: true,
            verify_hostname: true,
            allow_self_signed: false,
            custom_cas: vec![],
        }
    }
}

impl Default for ZitiEncryptionConfig {
    fn default() -> Self {
        Self {
            default_algorithm: "AES-256-GCM".to_string(),
            key_exchange: "ECDHE-X25519".to_string(),
            perfect_forward_secrecy: true,
            key_rotation_hours: 24,
        }
    }
}

impl Default for ZitiRateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_second: 1000,
            burst_limit: 5000,
            window_size: 60,
        }
    }
}

impl Default for ZitiSessionConfig {
    fn default() -> Self {
        Self {
            timeout: 7200, // 2 hours
            persistent: true,
            storage_path: Some(PathBuf::from("ziti/sessions")),
            max_concurrent: 1000,
        }
    }
}

impl Default for ZitiLoggingConfig {
    fn default() -> Self {
        Self {
            level: ZitiLogLevel::Info,
            output: ZitiLogOutput::Stdout,
            format: ZitiLogFormat::Structured,
            structured: true,
            rotation: Some(ZitiLogRotation::default()),
        }
    }
}

impl Default for ZitiLogRotation {
    fn default() -> Self {
        Self {
            max_size_mb: 100,
            max_files: 10,
            interval: ZitiRotationInterval::Daily,
        }
    }
}

impl Default for ZitiServicesConfig {
    fn default() -> Self {
        Self {
            discovery: ZitiServiceDiscovery::default(),
            default_timeout: 30,
            health_check: ZitiHealthCheckConfig::default(),
            load_balancing: ZitiLoadBalancingConfig::default(),
        }
    }
}

impl Default for ZitiServiceDiscovery {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 30,
            cache_ttl: 300,
        }
    }
}

impl Default for ZitiHealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 30,
            timeout: 10,
            failure_threshold: 3,
        }
    }
}

impl Default for ZitiLoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: ZitiLoadBalancingStrategy::HealthBased,
            session_affinity: false,
            health_based_routing: true,
        }
    }
}

impl ZitiConfig {
    /// Load configuration from file
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!(
                format!("Failed to read config file {}: {}", path.display(), e)
            ))?;

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)
                .map_err(|e| anyhow!(
                    format!("Failed to parse JSON config: {}", e)
                ))
        } else {
            toml::from_str(&content)
                .map_err(|e| anyhow!(
                    format!("Failed to parse TOML config: {}", e)
                ))
        }
    }

    /// Save configuration to file
    pub fn to_file(&self, path: &std::path::Path) -> Result<()> {
        let content = if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::to_string_pretty(self)
                .map_err(|e| anyhow!(
                    format!("Failed to serialize config to JSON: {}", e)
                ))?
        } else {
            toml::to_string_pretty(self)
                .map_err(|e| anyhow!(
                    format!("Failed to serialize config to TOML: {}", e)
                ))?
        };

        std::fs::write(path, content)
            .map_err(|e| anyhow!(
                format!("Failed to write config file {}: {}", path.display(), e)
            ))?;

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate controller URL
        if self.controller_url.is_empty() {
            return Err(anyhow!(
                "Controller URL cannot be empty".to_string()
            ));
        }

        // Validate identity configuration
        if self.identity.name.is_empty() {
            return Err(anyhow!(
                "Identity name cannot be empty".to_string()
            ));
        }

        // Validate network configuration
        if self.network.edge_routers.is_empty() {
            return Err(anyhow!(
                "At least one edge router must be configured".to_string()
            ));
        }

        // Validate buffer sizes
        if self.network.buffer_config.receive_buffer_size == 0 {
            return Err(anyhow!(
                "Receive buffer size must be greater than 0".to_string()
            ));
        }

        if self.network.buffer_config.send_buffer_size == 0 {
            return Err(anyhow!(
                "Send buffer size must be greater than 0".to_string()
            ));
        }

        // Validate timeouts
        if self.network.connect_timeout == 0 {
            return Err(anyhow!(
                "Connect timeout must be greater than 0".to_string()
            ));
        }

        Ok(())
    }

    /// Get default OpenSim services configuration
    pub fn with_opensim_services(mut self) -> Self {
        // Add OpenSim-specific configuration
        self.custom.insert("opensim_mode".to_string(), "true".to_string());
        self.custom.insert("virtual_world_services".to_string(), "enabled".to_string());
        self.custom.insert("asset_service_encryption".to_string(), "required".to_string());
        self.custom.insert("region_communication_security".to_string(), "zero_trust".to_string());
        
        self
    }
}