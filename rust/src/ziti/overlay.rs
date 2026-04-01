//! OpenZiti Overlay Network Management
//!
//! Manages the encrypted overlay network for secure communication between
//! virtual world components including regions, asset servers, and authentication services.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use super::config::ZitiConfig;
use super::{ZitiConnection, ZitiProtocol, ZitiEncryption, ZitiConnectionStatus};

/// Encrypted overlay network for secure region communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayNetworkTopology {
    /// Network identifier
    pub network_id: String,
    /// Connected regions in the overlay
    pub regions: HashMap<String, RegionEndpoint>,
    /// Encryption tunnels between regions
    pub tunnels: HashMap<String, EncryptedTunnel>,
    /// Network mesh configuration
    pub mesh_config: NetworkMeshConfig,
}

/// Individual region endpoint in the overlay network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionEndpoint {
    pub region_id: String,
    pub region_name: String,
    pub overlay_address: String,
    pub public_endpoint: String,
    pub encryption_key: String,
    pub last_heartbeat: u64,
    pub status: RegionStatus,
    pub connected_tunnels: Vec<String>,
}

/// Encrypted tunnel between two regions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTunnel {
    pub tunnel_id: String,
    pub source_region: String,
    pub destination_region: String,
    pub encryption_algorithm: ZitiEncryption,
    pub tunnel_key: String,
    pub created_at: u64,
    pub bytes_transferred: u64,
    pub latency_ms: f32,
    pub status: TunnelStatus,
}

/// Network mesh configuration for overlay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMeshConfig {
    /// Full mesh or hub-and-spoke topology
    pub topology_type: TopologyType,
    /// Maximum tunnels per region
    pub max_tunnels_per_region: u32,
    /// Automatic tunnel creation enabled
    pub auto_tunnel_creation: bool,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u32,
    /// Tunnel timeout in seconds
    pub tunnel_timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopologyType {
    FullMesh,
    HubAndSpoke { hub_region: String },
    Hierarchical { levels: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionStatus {
    Online,
    Connecting,
    Offline,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelStatus {
    Active,
    Establishing,
    Degraded,
    Failed,
}

/// Message types for region-to-region communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverlayMessage {
    /// Avatar crossing between regions
    AvatarCrossing {
        avatar_id: String,
        source_region: String,
        destination_region: String,
        position: (f32, f32, f32),
        velocity: (f32, f32, f32),
        avatar_data: Vec<u8>,
    },
    /// Object transfer between regions
    ObjectTransfer {
        object_id: String,
        object_data: Vec<u8>,
        destination_region: String,
    },
    /// Asset replication across regions
    AssetReplication {
        asset_id: String,
        asset_data: Vec<u8>,
        replication_targets: Vec<String>,
    },
    /// Region heartbeat and status
    RegionHeartbeat {
        region_id: String,
        status: RegionStatus,
        active_avatars: u32,
        cpu_usage: f32,
        memory_usage: f32,
    },
    /// Chat message broadcast
    ChatBroadcast {
        message: String,
        sender_id: String,
        channel: u32,
        regions: Vec<String>,
    },
    /// Grid-wide event notification
    GridEvent {
        event_type: String,
        event_data: Vec<u8>,
        target_regions: Vec<String>,
    },
}

/// Overlay network manager for secure region communication
pub struct ZitiOverlayManager {
    config: ZitiConfig,
    network_topology: Arc<RwLock<OverlayNetworkTopology>>,
    active_connections: Arc<RwLock<HashMap<String, ZitiConnection>>>,
    message_sender: Option<mpsc::UnboundedSender<OverlayMessage>>,
    message_receiver: Option<mpsc::UnboundedReceiver<OverlayMessage>>,
    is_initialized: bool,
    is_connected: bool,
}

impl ZitiOverlayManager {
    pub fn new(config: &ZitiConfig) -> Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let network_topology = OverlayNetworkTopology {
            network_id: Uuid::new_v4().to_string(),
            regions: HashMap::new(),
            tunnels: HashMap::new(),
            mesh_config: NetworkMeshConfig {
                topology_type: TopologyType::FullMesh,
                max_tunnels_per_region: 100,
                auto_tunnel_creation: true,
                heartbeat_interval: 30,
                tunnel_timeout: 300,
            },
        };

        Ok(Self {
            config: config.clone(),
            network_topology: Arc::new(RwLock::new(network_topology)),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            message_sender: Some(tx),
            message_receiver: Some(rx),
            is_initialized: false,
            is_connected: false,
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        if self.is_initialized {
            return Ok(());
        }

        tracing::info!("Initializing OpenZiti encrypted overlay network");

        // Initialize network topology
        let mut topology = self.network_topology.write().await;
        topology.network_id = format!("overlay_{}", Uuid::new_v4());
        
        // Start message processing loop
        if let Some(mut receiver) = self.message_receiver.take() {
            let topology_clone = self.network_topology.clone();
            let connections_clone = self.active_connections.clone();
            
            tokio::spawn(async move {
                while let Some(message) = receiver.recv().await {
                    if let Err(e) = Self::process_overlay_message(message, &topology_clone, &connections_clone).await {
                        tracing::error!("Failed to process overlay message: {}", e);
                    }
                }
            });
        }

        self.is_initialized = true;
        tracing::info!("OpenZiti encrypted overlay network initialized");
        Ok(())
    }

    pub async fn connect(&mut self) -> Result<()> {
        if !self.is_initialized {
            return Err(anyhow!("Service not initialized".to_string()));
        }

        if self.is_connected {
            return Ok(());
        }

        tracing::info!("Connecting to OpenZiti encrypted overlay network");
        
        // Initialize local region endpoint
        let local_region = self.create_local_region_endpoint().await?;
        
        let mut topology = self.network_topology.write().await;
        topology.regions.insert(local_region.region_id.clone(), local_region);
        
        self.is_connected = true;
        tracing::info!("Connected to OpenZiti encrypted overlay network");
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if !self.is_connected {
            return Ok(());
        }

        tracing::info!("Disconnecting from OpenZiti encrypted overlay network");
        
        // Close all active tunnels
        let mut connections = self.active_connections.write().await;
        connections.clear();
        
        // Clear network topology
        let mut topology = self.network_topology.write().await;
        topology.regions.clear();
        topology.tunnels.clear();
        
        self.is_connected = false;
        tracing::info!("Disconnected from OpenZiti encrypted overlay network");
        Ok(())
    }

    /// Register a new region in the overlay network
    pub async fn register_region(&self, region_id: &str, region_name: &str, endpoint: &str) -> Result<String> {
        let region_endpoint = RegionEndpoint {
            region_id: region_id.to_string(),
            region_name: region_name.to_string(),
            overlay_address: format!("overlay://{}", region_id),
            public_endpoint: endpoint.to_string(),
            encryption_key: self.generate_encryption_key(),
            last_heartbeat: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: RegionStatus::Online,
            connected_tunnels: Vec::new(),
        };

        let mut topology = self.network_topology.write().await;
        topology.regions.insert(region_id.to_string(), region_endpoint);
        
        // Auto-create tunnels to other regions if enabled
        if topology.mesh_config.auto_tunnel_creation {
            self.create_tunnels_for_region(region_id).await?;
        }

        tracing::info!("Registered region {} in overlay network", region_id);
        Ok(region_id.to_string())
    }

    /// Create encrypted tunnel between two regions
    pub async fn create_tunnel(&self, source_region: &str, destination_region: &str) -> Result<String> {
        let tunnel_id = format!("tunnel_{}_{}", source_region, destination_region);
        
        let tunnel = EncryptedTunnel {
            tunnel_id: tunnel_id.clone(),
            source_region: source_region.to_string(),
            destination_region: destination_region.to_string(),
            encryption_algorithm: ZitiEncryption::Aes256Gcm,
            tunnel_key: self.generate_tunnel_key(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            bytes_transferred: 0,
            latency_ms: 0.0,
            status: TunnelStatus::Establishing,
        };

        let mut topology = self.network_topology.write().await;
        topology.tunnels.insert(tunnel_id.clone(), tunnel);
        
        // Update region tunnel lists
        if let Some(source) = topology.regions.get_mut(source_region) {
            source.connected_tunnels.push(tunnel_id.clone());
        }
        if let Some(destination) = topology.regions.get_mut(destination_region) {
            destination.connected_tunnels.push(tunnel_id.clone());
        }

        tracing::info!("Created encrypted tunnel {} between {} and {}", tunnel_id, source_region, destination_region);
        Ok(tunnel_id)
    }

    /// Send message through encrypted overlay
    pub async fn send_overlay_message(&self, message: OverlayMessage) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            sender.send(message).map_err(|_| anyhow!(
                "Failed to send overlay message".to_string()
            ))?;
        }
        Ok(())
    }

    /// Get network topology information
    pub async fn get_network_topology(&self) -> Result<OverlayNetworkTopology> {
        let topology = self.network_topology.read().await;
        Ok(topology.clone())
    }

    /// Get active tunnel statistics
    pub async fn get_tunnel_statistics(&self) -> Result<HashMap<String, TunnelStatistics>> {
        let topology = self.network_topology.read().await;
        let mut stats = HashMap::new();
        
        for (tunnel_id, tunnel) in &topology.tunnels {
            stats.insert(tunnel_id.clone(), TunnelStatistics {
                tunnel_id: tunnel_id.clone(),
                bytes_transferred: tunnel.bytes_transferred,
                latency_ms: tunnel.latency_ms,
                status: tunnel.status.clone(),
                uptime_seconds: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() - tunnel.created_at,
            });
        }
        
        Ok(stats)
    }

    pub async fn create_connection(&self, service_name: &str, identity_id: &str) -> Result<ZitiConnection> {
        let connection = ZitiConnection {
            connection_id: format!("conn_{}", Uuid::new_v4()),
            service_name: service_name.to_string(),
            identity_id: identity_id.to_string(),
            source_address: "overlay".to_string(),
            destination_address: service_name.to_string(),
            protocol: ZitiProtocol::OpenSimSecure,
            encryption_type: ZitiEncryption::Aes256Gcm,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            bytes_sent: 0,
            bytes_received: 0,
            status: ZitiConnectionStatus::Connected,
        };

        let mut connections = self.active_connections.write().await;
        connections.insert(connection.connection_id.clone(), connection.clone());
        
        Ok(connection)
    }

    pub async fn send_data(&self, connection_id: &str, data: &[u8]) -> Result<usize> {
        let mut connections = self.active_connections.write().await;
        
        if let Some(connection) = connections.get_mut(connection_id) {
            // Update connection statistics
            connection.bytes_sent += data.len() as u64;
            connection.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            // In a real implementation, this would encrypt and send data through the tunnel
            tracing::debug!("Sent {} bytes through encrypted tunnel {}", data.len(), connection_id);
            Ok(data.len())
        } else {
            Err(anyhow!(
                format!("Connection {} not found", connection_id)
            ))
        }
    }

    pub async fn receive_data(&self, connection_id: &str, buffer: &mut [u8]) -> Result<usize> {
        let mut connections = self.active_connections.write().await;
        
        if let Some(connection) = connections.get_mut(connection_id) {
            // In a real implementation, this would receive and decrypt data from the tunnel
            // For now, return no data available
            connection.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(0)
        } else {
            Err(anyhow!(
                format!("Connection {} not found", connection_id)
            ))
        }
    }

    pub async fn close_connection(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.active_connections.write().await;
        connections.remove(connection_id);
        tracing::debug!("Closed encrypted tunnel connection {}", connection_id);
        Ok(())
    }

    // Private helper methods
    
    async fn create_local_region_endpoint(&self) -> Result<RegionEndpoint> {
        Ok(RegionEndpoint {
            region_id: format!("region_{}", Uuid::new_v4()),
            region_name: "Local Region".to_string(),
            overlay_address: "overlay://local".to_string(),
            public_endpoint: "localhost:9000".to_string(),
            encryption_key: self.generate_encryption_key(),
            last_heartbeat: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: RegionStatus::Online,
            connected_tunnels: Vec::new(),
        })
    }

    async fn create_tunnels_for_region(&self, region_id: &str) -> Result<()> {
        let region_ids: Vec<String> = {
            let topology = self.network_topology.read().await;
            topology.regions.keys().cloned().collect()
        };
        
        // Create tunnels to all other regions (full mesh)
        for other_region_id in region_ids {
            if other_region_id != region_id {
                self.create_tunnel(region_id, &other_region_id).await?;
            }
        }
        
        Ok(())
    }

    async fn process_overlay_message(
        message: OverlayMessage,
        topology: &Arc<RwLock<OverlayNetworkTopology>>,
        connections: &Arc<RwLock<HashMap<String, ZitiConnection>>>,
    ) -> Result<()> {
        match message {
            OverlayMessage::RegionHeartbeat { region_id, status, .. } => {
                let mut topology = topology.write().await;
                if let Some(region) = topology.regions.get_mut(&region_id) {
                    region.status = status;
                    region.last_heartbeat = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                }
                tracing::debug!("Processed heartbeat from region {}", region_id);
            }
            OverlayMessage::AvatarCrossing { avatar_id, source_region, destination_region, .. } => {
                tracing::info!("Processing avatar crossing: {} from {} to {}", avatar_id, source_region, destination_region);
                // In a real implementation, this would handle avatar state transfer
            }
            OverlayMessage::ChatBroadcast { message, sender_id, channel, regions } => {
                tracing::debug!("Broadcasting chat message from {} to {} regions", sender_id, regions.len());
                // In a real implementation, this would forward the message to target regions
            }
            _ => {
                tracing::debug!("Processed overlay message: {:?}", message);
            }
        }
        Ok(())
    }

    fn generate_encryption_key(&self) -> String {
        // In a real implementation, this would generate a proper encryption key
        format!("key_{}", Uuid::new_v4())
    }

    fn generate_tunnel_key(&self) -> String {
        // In a real implementation, this would generate a proper tunnel key
        format!("tunnel_key_{}", Uuid::new_v4())
    }
}

/// Tunnel performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatistics {
    pub tunnel_id: String,
    pub bytes_transferred: u64,
    pub latency_ms: f32,
    pub status: TunnelStatus,
    pub uptime_seconds: u64,
}