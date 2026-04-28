//! Inter-region communication system for distributed OpenSim regions

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    network::llsd::LLSDValue,
    region::{RegionId, RegionManager},
    state::StateManager,
};

/// Types of inter-region messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterRegionMessageType {
    /// Avatar crossing notification
    AvatarCrossing,
    /// Object transfer between regions
    ObjectTransfer,
    /// Chat/communication relay
    ChatRelay,
    /// Status update
    StatusUpdate,
    /// Grid-wide announcement
    GridAnnouncement,
    /// Economy transaction
    EconomyTransaction,
    /// Land/parcel management
    LandManagement,
    /// Group management
    GroupManagement,
}

/// Inter-region message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterRegionMessage {
    pub message_id: String,
    pub message_type: InterRegionMessageType,
    pub source_region: RegionId,
    pub target_region: Option<RegionId>, // None for broadcast messages
    pub payload: LLSDValue,
    pub timestamp: u64,
    pub priority: MessagePriority,
    pub requires_response: bool,
}

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Response to an inter-region message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterRegionResponse {
    pub response_id: String,
    pub original_message_id: String,
    pub success: bool,
    pub payload: Option<LLSDValue>,
    pub error_message: Option<String>,
    pub timestamp: u64,
}

/// Information about a connected region
#[derive(Debug, Clone)]
pub struct RegionConnection {
    pub region_id: RegionId,
    pub region_name: String,
    pub endpoint: String,
    pub last_heartbeat: u64,
    pub status: RegionStatus,
    pub connection_established: u64,
}

/// Status of a region connection
#[derive(Debug, Clone, PartialEq)]
pub enum RegionStatus {
    Online,
    Degraded,
    Offline,
    Unknown,
}

/// Manages inter-region communication
pub struct InterRegionManager {
    /// Connected regions
    connected_regions: RwLock<HashMap<RegionId, RegionConnection>>,
    /// Message queue for outgoing messages
    outgoing_queue: RwLock<Vec<InterRegionMessage>>,
    /// Pending responses
    pending_responses: RwLock<HashMap<String, tokio::sync::oneshot::Sender<InterRegionResponse>>>,
    /// Region manager for local region operations
    region_manager: Arc<RegionManager>,
    /// State manager for persistence
    state_manager: Arc<StateManager>,
    /// Message handler channel
    message_tx: mpsc::UnboundedSender<InterRegionMessage>,
    message_rx: RwLock<Option<mpsc::UnboundedReceiver<InterRegionMessage>>>,
}

impl InterRegionManager {
    /// Create a new inter-region manager
    pub fn new(region_manager: Arc<RegionManager>, state_manager: Arc<StateManager>) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Self {
            connected_regions: RwLock::new(HashMap::new()),
            outgoing_queue: RwLock::new(Vec::new()),
            pending_responses: RwLock::new(HashMap::new()),
            region_manager,
            state_manager,
            message_tx,
            message_rx: RwLock::new(Some(message_rx)),
        }
    }

    /// Start the inter-region communication system
    pub async fn start(&self) -> Result<()> {
        info!("Starting inter-region communication system");

        // Take the receiver out of the option
        let mut message_rx = self
            .message_rx
            .write()
            .await
            .take()
            .ok_or_else(|| anyhow!("Inter-region manager already started"))?;

        // Start message processing loop
        let manager = self.clone();
        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                if let Err(e) = manager.process_message(message).await {
                    error!("Error processing inter-region message: {}", e);
                }
            }
        });

        // Start heartbeat monitoring
        let manager_for_heartbeat = self.clone();
        tokio::spawn(async move {
            manager_for_heartbeat.heartbeat_monitor().await;
        });

        // Start message queue processor
        let manager_for_queue = self.clone();
        tokio::spawn(async move {
            manager_for_queue.process_message_queue().await;
        });

        info!("Inter-region communication system started");
        Ok(())
    }

    /// Register a new region connection
    pub async fn register_region(
        &self,
        region_id: RegionId,
        region_name: String,
        endpoint: String,
    ) -> Result<()> {
        info!(
            "Registering region {} ({}) at {}",
            region_id, region_name, endpoint
        );

        let connection = RegionConnection {
            region_id,
            region_name: region_name.clone(),
            endpoint,
            last_heartbeat: self.current_timestamp(),
            status: RegionStatus::Online,
            connection_established: self.current_timestamp(),
        };

        self.connected_regions
            .write()
            .await
            .insert(region_id, connection);

        // Send welcome message to new region
        let welcome_message = self.create_message(
            InterRegionMessageType::StatusUpdate,
            Some(region_id),
            LLSDValue::Map({
                let mut map = HashMap::new();
                map.insert(
                    "action".to_string(),
                    LLSDValue::String("welcome".to_string()),
                );
                map.insert(
                    "message".to_string(),
                    LLSDValue::String("Welcome to the grid".to_string()),
                );
                map
            }),
            MessagePriority::Normal,
            false,
        );

        self.send_message(welcome_message).await?;

        info!("Region {} registered successfully", region_name);
        Ok(())
    }

    /// Unregister a region connection
    pub async fn unregister_region(&self, region_id: RegionId) -> Result<()> {
        if let Some(connection) = self.connected_regions.write().await.remove(&region_id) {
            info!(
                "Unregistered region {} ({})",
                region_id, connection.region_name
            );

            // Send goodbye message to other regions
            let goodbye_message = self.create_message(
                InterRegionMessageType::StatusUpdate,
                None, // Broadcast
                LLSDValue::Map({
                    let mut map = HashMap::new();
                    map.insert(
                        "action".to_string(),
                        LLSDValue::String("region_offline".to_string()),
                    );
                    map.insert(
                        "region_id".to_string(),
                        LLSDValue::Integer(region_id.0 as i32),
                    );
                    map.insert(
                        "region_name".to_string(),
                        LLSDValue::String(connection.region_name),
                    );
                    map
                }),
                MessagePriority::Normal,
                false,
            );

            self.send_message(goodbye_message).await?;
        }

        Ok(())
    }

    /// Send a message to another region
    pub async fn send_message(&self, message: InterRegionMessage) -> Result<()> {
        info!(
            "Sending inter-region message {} of type {:?} to {:?}",
            message.message_id, message.message_type, message.target_region
        );

        // Add to outgoing queue for processing
        self.outgoing_queue.write().await.push(message);
        Ok(())
    }

    /// Send a message and wait for response
    pub async fn send_message_with_response(
        &self,
        mut message: InterRegionMessage,
        timeout: Duration,
    ) -> Result<InterRegionResponse> {
        message.requires_response = true;

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        self.pending_responses
            .write()
            .await
            .insert(message.message_id.clone(), response_tx);

        self.send_message(message).await?;

        // Wait for response with timeout
        match tokio::time::timeout(timeout, response_rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(anyhow!("Response channel closed")),
            Err(_) => Err(anyhow!("Response timeout")),
        }
    }

    /// Broadcast a message to all connected regions
    pub async fn broadcast_message(
        &self,
        message_type: InterRegionMessageType,
        payload: LLSDValue,
        priority: MessagePriority,
    ) -> Result<()> {
        let message = self.create_message(message_type, None, payload, priority, false);
        self.send_message(message).await
    }

    /// Handle avatar crossing between regions
    pub async fn handle_avatar_crossing(
        &self,
        agent_id: Uuid,
        source_region: RegionId,
        target_region: RegionId,
        position: (f32, f32, f32),
    ) -> Result<()> {
        info!(
            "Handling avatar crossing for {} from {:?} to {:?}",
            agent_id, source_region, target_region
        );

        let crossing_data = LLSDValue::Map({
            let mut map = HashMap::new();
            map.insert(
                "agent_id".to_string(),
                LLSDValue::String(agent_id.to_string()),
            );
            map.insert(
                "source_region".to_string(),
                LLSDValue::Integer(source_region.0 as i32),
            );
            map.insert(
                "position".to_string(),
                LLSDValue::Array(vec![
                    LLSDValue::Real(position.0 as f64),
                    LLSDValue::Real(position.1 as f64),
                    LLSDValue::Real(position.2 as f64),
                ]),
            );
            map
        });

        let message = self.create_message(
            InterRegionMessageType::AvatarCrossing,
            Some(target_region),
            crossing_data,
            MessagePriority::High,
            true,
        );

        let response = self
            .send_message_with_response(message, Duration::from_secs(10))
            .await?;

        if response.success {
            info!("Avatar crossing confirmed by target region");
            Ok(())
        } else {
            Err(anyhow!(
                "Avatar crossing rejected: {}",
                response
                    .error_message
                    .unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// Get the status of all connected regions
    pub async fn get_region_status(&self) -> HashMap<RegionId, RegionConnection> {
        self.connected_regions.read().await.clone()
    }

    /// Process incoming messages
    async fn process_message(&self, message: InterRegionMessage) -> Result<()> {
        info!(
            "Processing inter-region message: {:?}",
            message.message_type
        );

        match message.message_type {
            InterRegionMessageType::AvatarCrossing => {
                self.handle_incoming_avatar_crossing(message).await?;
            }
            InterRegionMessageType::StatusUpdate => {
                self.handle_status_update(message).await?;
            }
            InterRegionMessageType::ChatRelay => {
                self.handle_chat_relay(message).await?;
            }
            InterRegionMessageType::GridAnnouncement => {
                self.handle_grid_announcement(message).await?;
            }
            _ => {
                warn!("Unhandled message type: {:?}", message.message_type);
            }
        }

        Ok(())
    }

    /// Handle incoming avatar crossing requests
    async fn handle_incoming_avatar_crossing(&self, message: InterRegionMessage) -> Result<()> {
        // Extract crossing data
        let data = match &message.payload {
            LLSDValue::Map(map) => map,
            _ => return Err(anyhow!("Invalid crossing data format")),
        };

        let agent_id = match data.get("agent_id") {
            Some(LLSDValue::String(id)) => Uuid::parse_str(id)?,
            _ => return Err(anyhow!("Missing or invalid agent_id")),
        };

        // Validate the crossing request
        // In a real implementation, this would check permissions, region capacity, etc.

        let response = InterRegionResponse {
            response_id: Uuid::new_v4().to_string(),
            original_message_id: message.message_id,
            success: true,
            payload: Some(LLSDValue::Map({
                let mut map = HashMap::new();
                map.insert("accepted".to_string(), LLSDValue::Boolean(true));
                map.insert(
                    "agent_id".to_string(),
                    LLSDValue::String(agent_id.to_string()),
                );
                map
            })),
            error_message: None,
            timestamp: self.current_timestamp(),
        };

        // Send response if required
        if message.requires_response {
            self.send_response(response).await?;
        }

        Ok(())
    }

    /// Handle status updates
    async fn handle_status_update(&self, message: InterRegionMessage) -> Result<()> {
        info!("Received status update: {:?}", message.payload);
        Ok(())
    }

    /// Handle chat relay messages
    async fn handle_chat_relay(&self, message: InterRegionMessage) -> Result<()> {
        info!("Received chat relay: {:?}", message.payload);
        Ok(())
    }

    /// Handle grid announcements
    async fn handle_grid_announcement(&self, message: InterRegionMessage) -> Result<()> {
        info!("Received grid announcement: {:?}", message.payload);
        Ok(())
    }

    /// Send a response to a message
    async fn send_response(&self, response: InterRegionResponse) -> Result<()> {
        // In a real implementation, this would send the response over the network
        // For now, we'll just handle it locally if there's a pending response

        if let Some(sender) = self
            .pending_responses
            .write()
            .await
            .remove(&response.original_message_id)
        {
            let _ = sender.send(response);
        }

        Ok(())
    }

    /// Create a new inter-region message
    pub fn create_message(
        &self,
        message_type: InterRegionMessageType,
        target_region: Option<RegionId>,
        payload: LLSDValue,
        priority: MessagePriority,
        requires_response: bool,
    ) -> InterRegionMessage {
        InterRegionMessage {
            message_id: Uuid::new_v4().to_string(),
            message_type,
            source_region: RegionId(1), // TODO: Get actual source region
            target_region,
            payload,
            timestamp: self.current_timestamp(),
            priority,
            requires_response,
        }
    }

    /// Process the outgoing message queue
    async fn process_message_queue(&self) {
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            interval.tick().await;

            let messages = {
                let mut queue = self.outgoing_queue.write().await;
                if queue.is_empty() {
                    continue;
                }

                // Sort by priority and take up to 10 messages
                queue.sort_by(|a, b| b.priority.cmp(&a.priority));
                let count = queue.len().min(10);
                queue.drain(..count).collect::<Vec<_>>()
            };

            for message in messages {
                if let Err(e) = self.deliver_message(message).await {
                    error!("Failed to deliver message: {}", e);
                }
            }
        }
    }

    /// Deliver a message to its target
    async fn deliver_message(&self, message: InterRegionMessage) -> Result<()> {
        if let Some(target_region) = message.target_region {
            // Send to specific region
            let regions = self.connected_regions.read().await;
            if let Some(connection) = regions.get(&target_region) {
                // In a real implementation, this would send over the network
                info!(
                    "Delivering message to region {} at {}",
                    target_region, connection.endpoint
                );

                // Simulate message delivery by sending to our own processor
                if let Err(e) = self.message_tx.send(message) {
                    error!("Failed to send message to processor: {}", e);
                }
            } else {
                warn!("Target region {:?} not connected", target_region);
            }
        } else {
            // Broadcast to all regions
            let regions = self.connected_regions.read().await;
            for (region_id, connection) in regions.iter() {
                info!(
                    "Broadcasting message to region {} at {}",
                    region_id, connection.endpoint
                );

                // Create a copy for each region
                let mut message_copy = message.clone();
                message_copy.target_region = Some(*region_id);

                // Simulate broadcast by sending to our own processor
                if let Err(e) = self.message_tx.send(message_copy) {
                    error!("Failed to send broadcast message to processor: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Monitor region heartbeats
    async fn heartbeat_monitor(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            let current_time = self.current_timestamp();
            let mut regions = self.connected_regions.write().await;

            for (region_id, connection) in regions.iter_mut() {
                let time_since_heartbeat = current_time.saturating_sub(connection.last_heartbeat);

                if time_since_heartbeat > 120 {
                    // 2 minutes
                    if connection.status != RegionStatus::Offline {
                        warn!("Region {} appears to be offline", region_id);
                        connection.status = RegionStatus::Offline;
                    }
                } else if time_since_heartbeat > 60 {
                    // 1 minute
                    if connection.status == RegionStatus::Online {
                        warn!("Region {} connection degraded", region_id);
                        connection.status = RegionStatus::Degraded;
                    }
                }
            }
        }
    }

    /// Get current timestamp
    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

// Implement Clone for InterRegionManager (needed for spawning tasks)
impl Clone for InterRegionManager {
    fn clone(&self) -> Self {
        // Create a new channel for the clone
        let (message_tx, _) = mpsc::unbounded_channel();

        Self {
            connected_regions: RwLock::new(HashMap::new()),
            outgoing_queue: RwLock::new(Vec::new()),
            pending_responses: RwLock::new(HashMap::new()),
            region_manager: self.region_manager.clone(),
            state_manager: self.state_manager.clone(),
            message_tx,
            message_rx: RwLock::new(None), // Clone doesn't get a receiver
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ffi::physics::PhysicsBridge,
        region::{terrain::TerrainConfig, RegionConfig},
    };

    #[tokio::test]
    async fn test_inter_region_manager_creation() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));

        let inter_region_manager = InterRegionManager::new(region_manager, state_manager);

        // Test region registration
        inter_region_manager
            .register_region(
                RegionId(1),
                "Test Region".to_string(),
                "http://localhost:8080".to_string(),
            )
            .await?;

        let regions = inter_region_manager.get_region_status().await;
        assert_eq!(regions.len(), 1);
        assert!(regions.contains_key(&RegionId(1)));

        Ok(())
    }

    #[tokio::test]
    async fn test_message_creation_and_sending() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));

        let inter_region_manager = InterRegionManager::new(region_manager, state_manager);

        let message = inter_region_manager.create_message(
            InterRegionMessageType::StatusUpdate,
            Some(RegionId(2)),
            LLSDValue::Map(HashMap::new()),
            MessagePriority::Normal,
            false,
        );

        assert_eq!(message.target_region, Some(RegionId(2)));
        assert_eq!(message.message_type, InterRegionMessageType::StatusUpdate);
        assert_eq!(message.priority, MessagePriority::Normal);

        inter_region_manager.send_message(message).await?;

        Ok(())
    }
}
