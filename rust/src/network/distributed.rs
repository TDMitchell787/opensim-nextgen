//! Distributed region management for multi-server OpenSim grids

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    network::{
        inter_region::{InterRegionManager, InterRegionMessageType},
        llsd::LLSDValue,
    },
    region::{RegionConfig, RegionId, RegionManager},
    state::StateManager,
};

/// Information about a distributed region server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionServerInfo {
    pub server_id: String,
    pub server_name: String,
    pub endpoint: SocketAddr,
    pub capacity: RegionCapacity,
    pub regions: Vec<RegionId>,
    pub last_heartbeat: u64,
    pub status: ServerStatus,
    pub load_metrics: LoadMetrics,
}

/// Server capacity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCapacity {
    pub max_regions: u32,
    pub current_regions: u32,
    pub max_avatars_per_region: u32,
    pub max_objects_per_region: u32,
    pub cpu_cores: u32,
    pub memory_gb: u32,
}

/// Server status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    Online,
    Starting,
    Degraded,
    Maintenance,
    Offline,
}

/// Load metrics for a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub network_io: f64,
    pub disk_io: f64,
    pub active_avatars: u32,
    pub total_objects: u32,
    pub script_time_ms: f32,
}

/// Region placement strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlacementStrategy {
    /// Place on least loaded server
    LeastLoaded,
    /// Place on server with most available capacity
    MostCapacity,
    /// Place on specific server
    Specific(String),
    /// Place geographically close to other regions
    Geographic,
}

/// Request to create a region on the grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCreationRequest {
    pub region_config: RegionConfig,
    pub placement_strategy: PlacementStrategy,
    pub priority: CreationPriority,
    pub requester: String,
}

/// Priority for region creation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CreationPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Result of region creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCreationResult {
    pub success: bool,
    pub region_id: Option<RegionId>,
    pub server_id: Option<String>,
    pub error_message: Option<String>,
    pub estimated_startup_time: Option<Duration>,
}

/// Manages distributed regions across multiple servers
pub struct DistributedRegionManager {
    /// Local region manager
    local_region_manager: Arc<RegionManager>,
    /// Inter-region communication manager
    inter_region_manager: Arc<InterRegionManager>,
    /// State manager for persistence
    state_manager: Arc<StateManager>,
    /// Information about all servers in the grid
    servers: RwLock<HashMap<String, RegionServerInfo>>,
    /// Mapping of regions to servers
    region_to_server: RwLock<HashMap<RegionId, String>>,
    /// Our own server info
    local_server_info: RwLock<RegionServerInfo>,
    /// Pending region creation requests
    pending_requests: RwLock<HashMap<String, RegionCreationRequest>>,
    /// Message channel for distributed operations
    message_tx: mpsc::UnboundedSender<DistributedMessage>,
    message_rx: RwLock<Option<mpsc::UnboundedReceiver<DistributedMessage>>>,
}

/// Internal messages for distributed operations
#[derive(Debug, Clone)]
enum DistributedMessage {
    ServerHeartbeat(RegionServerInfo),
    RegionCreationRequest(String, RegionCreationRequest),
    RegionCreationResult(String, RegionCreationResult),
    ServerShutdown(String),
    LoadBalanceRequest,
}

impl DistributedRegionManager {
    /// Create a new distributed region manager
    pub fn new(
        local_region_manager: Arc<RegionManager>,
        inter_region_manager: Arc<InterRegionManager>,
        state_manager: Arc<StateManager>,
        server_name: String,
        endpoint: SocketAddr,
        capacity: RegionCapacity,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let local_server_info = RegionServerInfo {
            server_id: Uuid::new_v4().to_string(),
            server_name,
            endpoint,
            capacity,
            regions: Vec::new(),
            last_heartbeat: Self::current_timestamp(),
            status: ServerStatus::Starting,
            load_metrics: LoadMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                network_io: 0.0,
                disk_io: 0.0,
                active_avatars: 0,
                total_objects: 0,
                script_time_ms: 0.0,
            },
        };

        Self {
            local_region_manager,
            inter_region_manager,
            state_manager,
            servers: RwLock::new(HashMap::new()),
            region_to_server: RwLock::new(HashMap::new()),
            local_server_info: RwLock::new(local_server_info),
            pending_requests: RwLock::new(HashMap::new()),
            message_tx,
            message_rx: RwLock::new(Some(message_rx)),
        }
    }

    /// Start the distributed region management system
    pub async fn start(&self) -> Result<()> {
        info!("Starting distributed region management system");

        // Update local server status to online
        {
            let mut local_info = self.local_server_info.write().await;
            local_info.status = ServerStatus::Online;
            local_info.last_heartbeat = Self::current_timestamp();
        }

        // Take the receiver out of the option
        let mut message_rx = self
            .message_rx
            .write()
            .await
            .take()
            .ok_or_else(|| anyhow!("Distributed region manager already started"))?;

        // Start message processing loop
        let manager = self.clone();
        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                if let Err(e) = manager.process_distributed_message(message).await {
                    error!("Error processing distributed message: {}", e);
                }
            }
        });

        // Start heartbeat sender
        let manager_for_heartbeat = self.clone();
        tokio::spawn(async move {
            manager_for_heartbeat.heartbeat_sender().await;
        });

        // Start load balancing monitor
        let manager_for_load_balance = self.clone();
        tokio::spawn(async move {
            manager_for_load_balance.load_balance_monitor().await;
        });

        // Start server health monitor
        let manager_for_health = self.clone();
        tokio::spawn(async move {
            manager_for_health.server_health_monitor().await;
        });

        info!("Distributed region management system started");
        Ok(())
    }

    /// Register a new server in the grid
    pub async fn register_server(&self, server_info: RegionServerInfo) -> Result<()> {
        info!(
            "Registering server: {} ({})",
            server_info.server_name, server_info.server_id
        );

        self.servers
            .write()
            .await
            .insert(server_info.server_id.clone(), server_info.clone());

        // Register the server with inter-region communication
        self.inter_region_manager
            .register_region(
                RegionId(server_info.server_id.parse::<u64>().unwrap_or(0)),
                server_info.server_name.clone(),
                server_info.endpoint.to_string(),
            )
            .await?;

        info!("Server {} registered successfully", server_info.server_name);
        Ok(())
    }

    /// Request creation of a new region
    pub async fn request_region_creation(&self, request: RegionCreationRequest) -> Result<String> {
        let request_id = Uuid::new_v4().to_string();
        info!("Processing region creation request {}", request_id);

        // Store the request
        self.pending_requests
            .write()
            .await
            .insert(request_id.clone(), request.clone());

        // Send message to process the request
        if let Err(e) = self
            .message_tx
            .send(DistributedMessage::RegionCreationRequest(
                request_id.clone(),
                request,
            ))
        {
            error!("Failed to queue region creation request: {}", e);
            return Err(anyhow!("Failed to queue region creation request"));
        }

        Ok(request_id)
    }

    /// Get the status of a region creation request
    pub async fn get_creation_status(&self, request_id: &str) -> Option<RegionCreationRequest> {
        self.pending_requests.read().await.get(request_id).cloned()
    }

    /// Get information about all servers in the grid
    pub async fn get_servers(&self) -> HashMap<String, RegionServerInfo> {
        self.servers.read().await.clone()
    }

    /// Get information about our local server
    pub async fn get_local_server_info(&self) -> RegionServerInfo {
        self.local_server_info.read().await.clone()
    }

    /// Find the best server for a new region
    async fn find_best_server(&self, strategy: &PlacementStrategy) -> Result<String> {
        let servers = self.servers.read().await;

        match strategy {
            PlacementStrategy::LeastLoaded => {
                let best_server = servers
                    .values()
                    .filter(|s| s.status == ServerStatus::Online)
                    .filter(|s| s.capacity.current_regions < s.capacity.max_regions)
                    .min_by(|a, b| {
                        a.load_metrics
                            .cpu_usage
                            .partial_cmp(&b.load_metrics.cpu_usage)
                            .unwrap()
                    })
                    .ok_or_else(|| anyhow!("No available servers"))?;

                Ok(best_server.server_id.clone())
            }
            PlacementStrategy::MostCapacity => {
                let best_server = servers
                    .values()
                    .filter(|s| s.status == ServerStatus::Online)
                    .max_by_key(|s| s.capacity.max_regions - s.capacity.current_regions)
                    .ok_or_else(|| anyhow!("No available servers"))?;

                Ok(best_server.server_id.clone())
            }
            PlacementStrategy::Specific(server_id) => {
                if servers
                    .get(server_id)
                    .map(|s| s.status == ServerStatus::Online)
                    .unwrap_or(false)
                {
                    Ok(server_id.clone())
                } else {
                    Err(anyhow!("Specified server {} not available", server_id))
                }
            }
            PlacementStrategy::Geographic => {
                // For now, fallback to least loaded
                // In a real implementation, this would consider geographic location
                Box::pin(self.find_best_server(&PlacementStrategy::LeastLoaded)).await
            }
        }
    }

    /// Process a region creation request
    async fn process_region_creation(
        &self,
        request_id: String,
        request: RegionCreationRequest,
    ) -> Result<()> {
        info!("Processing region creation request {}", request_id);

        // Find the best server for this region
        let target_server = match self.find_best_server(&request.placement_strategy).await {
            Ok(server_id) => server_id,
            Err(e) => {
                error!("Failed to find suitable server: {}", e);
                self.send_creation_result(
                    request_id,
                    RegionCreationResult {
                        success: false,
                        region_id: None,
                        server_id: None,
                        error_message: Some(format!("No suitable server found: {}", e)),
                        estimated_startup_time: None,
                    },
                )
                .await?;
                return Ok(());
            }
        };

        // Check if it's our local server
        let local_server_id = self.local_server_info.read().await.server_id.clone();
        if target_server == local_server_id {
            // Create region locally
            match self.create_region_locally(request.region_config).await {
                Ok(region_id) => {
                    info!("Created region {} locally", region_id);
                    self.send_creation_result(
                        request_id,
                        RegionCreationResult {
                            success: true,
                            region_id: Some(region_id),
                            server_id: Some(target_server),
                            error_message: None,
                            estimated_startup_time: Some(Duration::from_secs(30)),
                        },
                    )
                    .await?;
                }
                Err(e) => {
                    error!("Failed to create region locally: {}", e);
                    self.send_creation_result(
                        request_id,
                        RegionCreationResult {
                            success: false,
                            region_id: None,
                            server_id: Some(target_server),
                            error_message: Some(format!("Local creation failed: {}", e)),
                            estimated_startup_time: None,
                        },
                    )
                    .await?;
                }
            }
        } else {
            // Send request to remote server
            if let Err(e) = self
                .send_remote_creation_request(&target_server, request_id.clone(), request)
                .await
            {
                error!("Failed to send remote creation request: {}", e);
                self.send_creation_result(
                    request_id,
                    RegionCreationResult {
                        success: false,
                        region_id: None,
                        server_id: Some(target_server),
                        error_message: Some(format!("Remote request failed: {}", e)),
                        estimated_startup_time: None,
                    },
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Create a region on the local server
    async fn create_region_locally(&self, config: RegionConfig) -> Result<RegionId> {
        let region_id = RegionId(rand::random());

        let _region = self
            .local_region_manager
            .create_region(region_id, config)
            .await
            .map_err(|e| anyhow!("Failed to create region: {}", e))?;

        // Update local server info
        {
            let mut local_info = self.local_server_info.write().await;
            local_info.regions.push(region_id);
            local_info.capacity.current_regions += 1;
        }

        // Update region to server mapping
        {
            let local_server_id = self.local_server_info.read().await.server_id.clone();
            self.region_to_server
                .write()
                .await
                .insert(region_id, local_server_id);
        }

        Ok(region_id)
    }

    /// Send a region creation request to a remote server
    async fn send_remote_creation_request(
        &self,
        target_server: &str,
        request_id: String,
        request: RegionCreationRequest,
    ) -> Result<()> {
        let payload = LLSDValue::Map({
            let mut map = HashMap::new();
            map.insert("request_id".to_string(), LLSDValue::String(request_id));
            map.insert(
                "region_name".to_string(),
                LLSDValue::String(request.region_config.name.clone()),
            );
            map.insert(
                "region_size_x".to_string(),
                LLSDValue::Integer(request.region_config.size.0 as i32),
            );
            map.insert(
                "region_size_y".to_string(),
                LLSDValue::Integer(request.region_config.size.1 as i32),
            );
            map.insert(
                "max_entities".to_string(),
                LLSDValue::Integer(request.region_config.max_entities as i32),
            );
            map.insert(
                "requester".to_string(),
                LLSDValue::String(request.requester),
            );
            map
        });

        let message = self.inter_region_manager.create_message(
            InterRegionMessageType::GridAnnouncement,
            Some(RegionId(target_server.parse::<u64>().unwrap_or(0))),
            payload,
            crate::network::inter_region::MessagePriority::High,
            true,
        );

        self.inter_region_manager.send_message(message).await?;
        Ok(())
    }

    /// Send a region creation result
    async fn send_creation_result(
        &self,
        request_id: String,
        result: RegionCreationResult,
    ) -> Result<()> {
        info!(
            "Sending creation result for request {}: success={}",
            request_id, result.success
        );

        // Remove from pending requests
        self.pending_requests.write().await.remove(&request_id);

        // In a real implementation, this would notify the requester
        // For now, we'll just log the result
        info!("Region creation result: {:?}", result);

        Ok(())
    }

    /// Process distributed messages
    async fn process_distributed_message(&self, message: DistributedMessage) -> Result<()> {
        match message {
            DistributedMessage::ServerHeartbeat(server_info) => {
                self.handle_server_heartbeat(server_info).await?;
            }
            DistributedMessage::RegionCreationRequest(request_id, request) => {
                self.process_region_creation(request_id, request).await?;
            }
            DistributedMessage::RegionCreationResult(request_id, result) => {
                self.handle_creation_result(request_id, result).await?;
            }
            DistributedMessage::ServerShutdown(server_id) => {
                self.handle_server_shutdown(server_id).await?;
            }
            DistributedMessage::LoadBalanceRequest => {
                self.handle_load_balance_request().await?;
            }
        }
        Ok(())
    }

    /// Handle server heartbeat
    async fn handle_server_heartbeat(&self, server_info: RegionServerInfo) -> Result<()> {
        self.servers
            .write()
            .await
            .insert(server_info.server_id.clone(), server_info);
        Ok(())
    }

    /// Handle region creation result
    async fn handle_creation_result(
        &self,
        request_id: String,
        result: RegionCreationResult,
    ) -> Result<()> {
        info!(
            "Received creation result for request {}: success={}",
            request_id, result.success
        );

        if let Some(region_id) = result.region_id {
            if let Some(server_id) = result.server_id {
                self.region_to_server
                    .write()
                    .await
                    .insert(region_id, server_id);
            }
        }

        Ok(())
    }

    /// Handle server shutdown
    async fn handle_server_shutdown(&self, server_id: String) -> Result<()> {
        info!("Server {} is shutting down", server_id);

        // Remove server from active servers
        if let Some(_server_info) = self.servers.write().await.remove(&server_id) {
            // Mark all its regions as needing migration
            let region_mapping = self.region_to_server.read().await;
            let affected_regions: Vec<RegionId> = region_mapping
                .iter()
                .filter(|(_, sid)| *sid == &server_id)
                .map(|(rid, _)| *rid)
                .collect();

            info!("Server shutdown affects {} regions", affected_regions.len());

            // In a real implementation, this would trigger region migration
            for region_id in affected_regions {
                warn!(
                    "Region {} needs migration due to server shutdown",
                    region_id
                );
            }
        }

        Ok(())
    }

    /// Handle load balance request
    async fn handle_load_balance_request(&self) -> Result<()> {
        info!("Processing load balance request");

        // Analyze current load distribution
        let servers = self.servers.read().await;
        let mut overloaded_servers = Vec::new();
        let mut underloaded_servers = Vec::new();

        for (server_id, server_info) in servers.iter() {
            if server_info.status != ServerStatus::Online {
                continue;
            }

            let cpu_load = server_info.load_metrics.cpu_usage;
            let region_utilization = server_info.capacity.current_regions as f32
                / server_info.capacity.max_regions as f32;

            if cpu_load > 80.0 || region_utilization > 0.9 {
                overloaded_servers.push(server_id.clone());
            } else if cpu_load < 40.0 && region_utilization < 0.5 {
                underloaded_servers.push(server_id.clone());
            }
        }

        if !overloaded_servers.is_empty() && !underloaded_servers.is_empty() {
            info!(
                "Load balancing needed: {} overloaded servers, {} underloaded servers",
                overloaded_servers.len(),
                underloaded_servers.len()
            );

            // In a real implementation, this would trigger region migration
            for overloaded in overloaded_servers {
                info!("Server {} needs load reduction", overloaded);
            }
        }

        Ok(())
    }

    /// Send heartbeat to other servers
    async fn heartbeat_sender(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            // Update local metrics
            self.update_local_metrics().await;

            // Send heartbeat via inter-region communication
            let local_info = self.local_server_info.read().await.clone();

            let payload = LLSDValue::Map({
                let mut map = HashMap::new();
                map.insert(
                    "server_id".to_string(),
                    LLSDValue::String(local_info.server_id),
                );
                map.insert(
                    "server_name".to_string(),
                    LLSDValue::String(local_info.server_name),
                );
                map.insert(
                    "status".to_string(),
                    LLSDValue::String(format!("{:?}", local_info.status)),
                );
                map.insert(
                    "cpu_usage".to_string(),
                    LLSDValue::Real(local_info.load_metrics.cpu_usage as f64),
                );
                map.insert(
                    "memory_usage".to_string(),
                    LLSDValue::Real(local_info.load_metrics.memory_usage as f64),
                );
                map.insert(
                    "current_regions".to_string(),
                    LLSDValue::Integer(local_info.capacity.current_regions as i32),
                );
                map.insert(
                    "max_regions".to_string(),
                    LLSDValue::Integer(local_info.capacity.max_regions as i32),
                );
                map
            });

            if let Err(e) = self
                .inter_region_manager
                .broadcast_message(
                    InterRegionMessageType::StatusUpdate,
                    payload,
                    crate::network::inter_region::MessagePriority::Low,
                )
                .await
            {
                error!("Failed to send heartbeat: {}", e);
            }
        }
    }

    /// Update local server metrics
    async fn update_local_metrics(&self) {
        let mut local_info = self.local_server_info.write().await;

        // In a real implementation, these would be actual system metrics
        local_info.load_metrics.cpu_usage = rand::random::<f32>() * 100.0;
        local_info.load_metrics.memory_usage = rand::random::<f32>() * 100.0;
        local_info.load_metrics.network_io = rand::random::<f64>() * 1000.0;
        local_info.load_metrics.disk_io = rand::random::<f64>() * 500.0;
        local_info.last_heartbeat = Self::current_timestamp();
    }

    /// Monitor server health and load balancing
    async fn load_balance_monitor(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes

        loop {
            interval.tick().await;

            if let Err(e) = self.message_tx.send(DistributedMessage::LoadBalanceRequest) {
                error!("Failed to queue load balance request: {}", e);
            }
        }
    }

    /// Monitor server health
    async fn server_health_monitor(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            let current_time = Self::current_timestamp();
            let mut servers = self.servers.write().await;

            for (server_id, server_info) in servers.iter_mut() {
                let time_since_heartbeat = current_time.saturating_sub(server_info.last_heartbeat);

                if time_since_heartbeat > 180 {
                    // 3 minutes
                    if server_info.status != ServerStatus::Offline {
                        warn!("Server {} appears to be offline", server_id);
                        server_info.status = ServerStatus::Offline;

                        // Queue shutdown message
                        if let Err(e) = self
                            .message_tx
                            .send(DistributedMessage::ServerShutdown(server_id.clone()))
                        {
                            error!("Failed to queue server shutdown message: {}", e);
                        }
                    }
                } else if time_since_heartbeat > 90 {
                    // 1.5 minutes
                    if server_info.status == ServerStatus::Online {
                        warn!("Server {} connection degraded", server_id);
                        server_info.status = ServerStatus::Degraded;
                    }
                }
            }
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

// Implement Clone for DistributedRegionManager
impl Clone for DistributedRegionManager {
    fn clone(&self) -> Self {
        let (message_tx, _) = mpsc::unbounded_channel();

        Self {
            local_region_manager: self.local_region_manager.clone(),
            inter_region_manager: self.inter_region_manager.clone(),
            state_manager: self.state_manager.clone(),
            servers: RwLock::new(HashMap::new()),
            region_to_server: RwLock::new(HashMap::new()),
            local_server_info: RwLock::new(RegionServerInfo {
                server_id: "clone".to_string(),
                server_name: "clone".to_string(),
                endpoint: "127.0.0.1:0".parse().unwrap(),
                capacity: RegionCapacity {
                    max_regions: 0,
                    current_regions: 0,
                    max_avatars_per_region: 0,
                    max_objects_per_region: 0,
                    cpu_cores: 0,
                    memory_gb: 0,
                },
                regions: Vec::new(),
                last_heartbeat: 0,
                status: ServerStatus::Offline,
                load_metrics: LoadMetrics {
                    cpu_usage: 0.0,
                    memory_usage: 0.0,
                    network_io: 0.0,
                    disk_io: 0.0,
                    active_avatars: 0,
                    total_objects: 0,
                    script_time_ms: 0.0,
                },
            }),
            pending_requests: RwLock::new(HashMap::new()),
            message_tx,
            message_rx: RwLock::new(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ffi::physics::PhysicsBridge, region::terrain::TerrainConfig};

    #[tokio::test]
    async fn test_distributed_region_manager_creation() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        let inter_region_manager = Arc::new(InterRegionManager::new(
            region_manager.clone(),
            state_manager.clone(),
        ));

        let capacity = RegionCapacity {
            max_regions: 10,
            current_regions: 0,
            max_avatars_per_region: 100,
            max_objects_per_region: 10000,
            cpu_cores: 8,
            memory_gb: 32,
        };

        let distributed_manager = DistributedRegionManager::new(
            region_manager,
            inter_region_manager,
            state_manager,
            "Test Server".to_string(),
            "127.0.0.1:9000".parse().unwrap(),
            capacity,
        );

        let local_info = distributed_manager.get_local_server_info().await;
        assert_eq!(local_info.server_name, "Test Server");
        assert_eq!(local_info.capacity.max_regions, 10);

        Ok(())
    }

    #[tokio::test]
    async fn test_region_creation_request() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        let inter_region_manager = Arc::new(InterRegionManager::new(
            region_manager.clone(),
            state_manager.clone(),
        ));

        let capacity = RegionCapacity {
            max_regions: 10,
            current_regions: 0,
            max_avatars_per_region: 100,
            max_objects_per_region: 10000,
            cpu_cores: 8,
            memory_gb: 32,
        };

        let distributed_manager = DistributedRegionManager::new(
            region_manager,
            inter_region_manager,
            state_manager,
            "Test Server".to_string(),
            "127.0.0.1:9000".parse().unwrap(),
            capacity,
        );

        let region_config = RegionConfig {
            name: "Test Region".to_string(),
            size: (256, 256),
            location: (1000, 1000),
            terrain: TerrainConfig::default(),
            physics: crate::ffi::physics::PhysicsConfig::default(),
            max_entities: 1000,
        };

        let request = RegionCreationRequest {
            region_config,
            placement_strategy: PlacementStrategy::LeastLoaded,
            priority: CreationPriority::Normal,
            requester: "test_user".to_string(),
        };

        let request_id = distributed_manager.request_region_creation(request).await?;
        assert!(!request_id.is_empty());

        // Check that request is pending
        let status = distributed_manager.get_creation_status(&request_id).await;
        assert!(status.is_some());

        Ok(())
    }
}
