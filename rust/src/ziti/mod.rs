//! OpenZiti Zero Trust Networking for OpenSim Next
//!
//! Provides application-embedded zero trust networking capabilities for secure,
//! encrypted, and policy-based communication between virtual world components.

pub mod config;
pub mod identity;
pub mod services;
pub mod overlay;
pub mod policies;
pub mod monitoring;
pub mod ffi;
pub mod integration;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

/// OpenZiti network manager for zero trust virtual world networking
pub struct ZitiNetworkManager {
    /// Network configuration
    config: config::ZitiConfig,
    
    /// Identity manager for zero trust authentication
    identity_manager: identity::ZitiIdentityManager,
    
    /// Service manager for dark services and communication
    service_manager: services::ZitiServiceManager,
    
    /// Overlay network manager for secure tunneling
    overlay_manager: overlay::ZitiOverlayManager,
    
    /// Policy engine for access control
    policy_engine: policies::ZitiPolicyEngine,
    
    /// Monitoring and analytics
    monitoring: monitoring::ZitiMonitoring,
    
    /// Active connections
    connections: Arc<RwLock<HashMap<String, ZitiConnection>>>,
    
    /// Network state
    is_initialized: bool,
    is_connected: bool,
}

/// Zero trust connection representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiConnection {
    pub connection_id: String,
    pub service_name: String,
    pub identity_id: String,
    pub source_address: String,
    pub destination_address: String,
    pub protocol: ZitiProtocol,
    pub encryption_type: ZitiEncryption,
    pub created_at: u64,
    pub last_activity: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub status: ZitiConnectionStatus,
}

/// Supported zero trust protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiProtocol {
    /// Secure HTTP for web services
    Https,
    /// Secure WebSocket for real-time communication
    Wss,
    /// Custom protocol for virtual world data
    OpenSimSecure,
    /// Secure file transfer
    Sftp,
    /// Secure database connections
    SecureDatabase,
}

/// Encryption types supported by OpenZiti
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiEncryption {
    /// AES-256-GCM encryption
    Aes256Gcm,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305,
    /// End-to-end encryption
    EndToEnd,
}

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZitiConnectionStatus {
    Initializing,
    Authenticating,
    Connected,
    Disconnected,
    Failed,
    Terminated,
}

/// Zero trust service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiService {
    pub service_id: String,
    pub name: String,
    pub description: String,
    pub service_type: ZitiServiceType,
    pub endpoint_address: String,
    pub port: u16,
    pub protocol: ZitiProtocol,
    pub policies: Vec<String>,
    pub encryption_required: bool,
    pub max_connections: Option<u32>,
    pub created_at: u64,
    pub enabled: bool,
}

/// Types of zero trust services
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZitiServiceType {
    /// Virtual world region service
    Region,
    /// Asset management service
    AssetService,
    /// Authentication service
    AuthService,
    /// Database service
    DatabaseService,
    /// Monitoring service
    MonitoringService,
    /// WebSocket service for web clients
    WebSocketService,
    /// Custom application service
    CustomService,
}

impl ZitiNetworkManager {
    /// Create a new zero trust network manager
    pub fn new(config: config::ZitiConfig) -> Result<Self> {
        Ok(Self {
            identity_manager: identity::ZitiIdentityManager::new(&config)?,
            service_manager: services::ZitiServiceManager::new(&config)?,
            overlay_manager: overlay::ZitiOverlayManager::new(&config)?,
            policy_engine: policies::ZitiPolicyEngine::new(&config)?,
            monitoring: monitoring::ZitiMonitoring::new(&config)?,
            connections: Arc::new(RwLock::new(HashMap::new())),
            config,
            is_initialized: false,
            is_connected: false,
        })
    }

    /// Initialize the zero trust network
    pub async fn initialize(&mut self) -> Result<()> {
        if self.is_initialized {
            return Ok(());
        }

        tracing::info!("Initializing OpenZiti zero trust network");

        // Initialize core components
        self.identity_manager.initialize().await?;
        self.service_manager.initialize().await?;
        self.overlay_manager.initialize().await?;
        self.policy_engine.initialize().await?;
        self.monitoring.initialize().await?;

        self.is_initialized = true;
        tracing::info!("OpenZiti zero trust network initialized successfully");
        Ok(())
    }

    /// Connect to the zero trust network
    pub async fn connect(&mut self) -> Result<()> {
        if !self.is_initialized {
            return Err(anyhow!(
                "ZitiNetworkManager not initialized".to_string()
            ));
        }

        if self.is_connected {
            return Ok(());
        }

        tracing::info!("Connecting to OpenZiti zero trust network");

        // Authenticate identity
        self.identity_manager.authenticate().await?;

        // Connect to overlay network
        self.overlay_manager.connect().await?;

        // Start monitoring
        self.monitoring.start().await?;

        self.is_connected = true;
        tracing::info!("Connected to OpenZiti zero trust network");
        Ok(())
    }

    /// Disconnect from the zero trust network
    pub async fn disconnect(&mut self) -> Result<()> {
        if !self.is_connected {
            return Ok(());
        }

        tracing::info!("Disconnecting from OpenZiti zero trust network");

        // Close all active connections
        let mut connections = self.connections.write().await;
        for (_, mut connection) in connections.drain() {
            connection.status = ZitiConnectionStatus::Terminated;
        }

        // Disconnect components
        self.monitoring.stop().await?;
        self.overlay_manager.disconnect().await?;
        self.identity_manager.logout().await?;

        self.is_connected = false;
        tracing::info!("Disconnected from OpenZiti zero trust network");
        Ok(())
    }

    /// Create a secure service for zero trust access
    pub async fn create_service(&mut self, service: ZitiService) -> Result<String> {
        if !self.is_connected {
            return Err(anyhow!(
                "Not connected to zero trust network".to_string()
            ));
        }

        let service_id = self.service_manager.create_service(service).await?;
        tracing::info!("Created zero trust service: {}", service_id);
        Ok(service_id)
    }

    /// Establish a secure connection to a service
    pub async fn connect_to_service(&mut self, service_name: &str, identity_id: &str) -> Result<String> {
        if !self.is_connected {
            return Err(anyhow!(
                "Not connected to zero trust network".to_string()
            ));
        }

        // Verify identity and policies
        self.policy_engine.verify_access(identity_id, service_name).await?;

        // Create secure connection
        let connection = self.overlay_manager.create_connection(service_name, identity_id).await?;
        let connection_id = connection.connection_id.clone();

        // Store connection
        let mut connections = self.connections.write().await;
        connections.insert(connection_id.clone(), connection);

        // Update monitoring
        self.monitoring.record_connection(&connection_id, service_name, "default-identity").await?;

        tracing::info!("Established secure connection to service: {} (connection: {})", 
                      service_name, connection_id);
        Ok(connection_id)
    }

    /// Send data through a secure connection
    pub async fn send_data(&mut self, connection_id: &str, data: &[u8]) -> Result<usize> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(connection_id) {
            if connection.status != ZitiConnectionStatus::Connected {
                return Err(anyhow!(
                    format!("Connection {} not in connected state", connection_id)
                ));
            }

            let bytes_sent = self.overlay_manager.send_data(connection_id, data).await?;
            
            // Update monitoring
            self.monitoring.record_data_sent(connection_id, bytes_sent, None).await?;
            
            Ok(bytes_sent)
        } else {
            Err(anyhow!(
                format!("Connection {} not found", connection_id)
            ))
        }
    }

    /// Receive data from a secure connection
    pub async fn receive_data(&mut self, connection_id: &str, buffer: &mut [u8]) -> Result<usize> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(connection_id) {
            if connection.status != ZitiConnectionStatus::Connected {
                return Err(anyhow!(
                    format!("Connection {} not in connected state", connection_id)
                ));
            }

            let bytes_received = self.overlay_manager.receive_data(connection_id, buffer).await?;
            
            // Update monitoring
            self.monitoring.record_data_received(connection_id, bytes_received, None).await?;
            
            Ok(bytes_received)
        } else {
            Err(anyhow!(
                format!("Connection {} not found", connection_id)
            ))
        }
    }

    /// Close a secure connection
    pub async fn close_connection(&mut self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        if let Some(mut connection) = connections.remove(connection_id) {
            connection.status = ZitiConnectionStatus::Terminated;
            
            // Close overlay connection
            self.overlay_manager.close_connection(connection_id).await?;
            
            // Update monitoring
            self.monitoring.record_connection_closed(connection_id).await?;
            
            tracing::info!("Closed secure connection: {}", connection_id);
        }
        Ok(())
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<HashMap<String, u64>> {
        self.monitoring.get_statistics().await
    }

    /// Get active connections
    pub async fn get_active_connections(&self) -> Result<Vec<ZitiConnection>> {
        let connections = self.connections.read().await;
        Ok(connections.values().cloned().collect())
    }

    /// Update access policies
    pub async fn update_policies(&mut self, policies: Vec<policies::ZitiPolicy>) -> Result<()> {
        self.policy_engine.update_policies(policies).await
    }

    /// Check if identity has access to service
    pub async fn check_access(&self, identity_id: &str, service_name: &str) -> Result<bool> {
        self.policy_engine.check_access(identity_id, service_name).await
    }

    /// Get configuration
    pub fn get_config(&self) -> &config::ZitiConfig {
        &self.config
    }

    /// Update configuration
    pub async fn update_config(&mut self, config: config::ZitiConfig) -> Result<()> {
        if self.is_connected {
            return Err(anyhow!(
                "Cannot update configuration while connected".to_string()
            ));
        }

        self.config = config;
        Ok(())
    }

    /// Check connection status
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Check initialization status
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

/// Zero trust network events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZitiNetworkEvent {
    /// Identity authenticated
    IdentityAuthenticated { identity_id: String },
    
    /// Service created
    ServiceCreated { service_id: String, service_name: String },
    
    /// Connection established
    ConnectionEstablished { connection_id: String, service_name: String },
    
    /// Connection closed
    ConnectionClosed { connection_id: String, reason: String },
    
    /// Policy violation
    PolicyViolation { identity_id: String, service_name: String, reason: String },
    
    /// Network error
    NetworkError { error: String },
}

/// Zero trust network capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiCapabilities {
    pub supports_encryption: bool,
    pub supports_dark_services: bool,
    pub supports_policy_enforcement: bool,
    pub supports_identity_verification: bool,
    pub supports_network_analytics: bool,
    pub max_connections: u32,
    pub supported_protocols: Vec<ZitiProtocol>,
    pub supported_encryption: Vec<ZitiEncryption>,
}

impl Default for ZitiCapabilities {
    fn default() -> Self {
        Self {
            supports_encryption: true,
            supports_dark_services: true,
            supports_policy_enforcement: true,
            supports_identity_verification: true,
            supports_network_analytics: true,
            max_connections: 10000,
            supported_protocols: vec![
                ZitiProtocol::Https,
                ZitiProtocol::Wss,
                ZitiProtocol::OpenSimSecure,
                ZitiProtocol::Sftp,
                ZitiProtocol::SecureDatabase,
            ],
            supported_encryption: vec![
                ZitiEncryption::Aes256Gcm,
                ZitiEncryption::ChaCha20Poly1305,
                ZitiEncryption::EndToEnd,
            ],
        }
    }
}