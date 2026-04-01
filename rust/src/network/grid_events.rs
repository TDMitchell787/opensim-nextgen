//! Grid-wide event and messaging system for OpenSim

use std::{collections::HashMap, sync::Arc, time::Duration};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::{
    network::{
        inter_region::{InterRegionManager, InterRegionMessageType, MessagePriority},
        llsd::LLSDValue,
    },
    region::RegionId,
};

/// Types of grid-wide events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GridEventType {
    /// User logged in or out
    UserStatusChange,
    /// Economy transaction occurred
    EconomyTransaction,
    /// Land ownership changed
    LandOwnershipChange,
    /// Group membership changed
    GroupMembershipChange,
    /// Object created, modified, or deleted
    ObjectManagement,
    /// Chat message (group, region, or global)
    ChatMessage,
    /// Friend request or status change
    FriendStatusChange,
    /// Teleport request or completion
    TeleportEvent,
    /// Region status change (online, offline, etc.)
    RegionStatusChange,
    /// Script execution event
    ScriptEvent,
    /// Asset uploaded or modified
    AssetEvent,
    /// Grid maintenance notification
    MaintenanceNotification,
    /// Security alert
    SecurityAlert,
}

/// Grid event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridEvent {
    pub event_id: String,
    pub event_type: GridEventType,
    pub source_region: Option<RegionId>,
    pub source_user: Option<Uuid>,
    pub target_regions: Vec<RegionId>, // Empty means broadcast to all
    pub target_users: Vec<Uuid>, // Empty means all users
    pub payload: LLSDValue,
    pub timestamp: u64,
    pub priority: EventPriority,
    pub persistent: bool, // Whether to store for offline users
    pub expiry: Option<u64>, // When the event expires (for persistent events)
}

/// Event priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
    Emergency = 4,
}

/// Event subscription information
#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub subscriber_id: String,
    pub event_types: Vec<GridEventType>,
    pub regions: Vec<RegionId>, // Empty means all regions
    pub users: Vec<Uuid>, // Empty means all users
    pub callback: String, // Endpoint or method to call
}

/// Event statistics
#[derive(Debug, Clone)]
pub struct EventStats {
    pub total_events_sent: u64,
    pub events_by_type: HashMap<GridEventType, u64>,
    pub active_subscriptions: usize,
    pub persistent_events_stored: usize,
    pub failed_deliveries: u64,
}

/// Manages grid-wide events and messaging
pub struct GridEventManager {
    /// Event subscriptions
    subscriptions: RwLock<HashMap<String, EventSubscription>>,
    /// Persistent events for offline users
    persistent_events: RwLock<HashMap<Uuid, Vec<GridEvent>>>,
    /// Event statistics
    stats: RwLock<EventStats>,
    /// Event broadcast channel
    event_tx: broadcast::Sender<GridEvent>,
    /// Inter-region manager for communication
    inter_region_manager: Arc<InterRegionManager>,
    /// Event processing queue
    event_queue: RwLock<Vec<GridEvent>>,
}

impl GridEventManager {
    /// Create a new grid event manager
    pub fn new(inter_region_manager: Arc<InterRegionManager>) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        
        Self {
            subscriptions: RwLock::new(HashMap::new()),
            persistent_events: RwLock::new(HashMap::new()),
            stats: RwLock::new(EventStats {
                total_events_sent: 0,
                events_by_type: HashMap::new(),
                active_subscriptions: 0,
                persistent_events_stored: 0,
                failed_deliveries: 0,
            }),
            event_tx,
            inter_region_manager,
            event_queue: RwLock::new(Vec::new()),
        }
    }

    /// Start the grid event system
    pub async fn start(&self) -> Result<()> {
        info!("Starting grid event system");

        // Start event processing loop
        let manager = self.clone();
        tokio::spawn(async move {
            manager.event_processing_loop().await;
        });

        // Start persistent event cleanup
        let manager_for_cleanup = self.clone();
        tokio::spawn(async move {
            manager_for_cleanup.cleanup_expired_events().await;
        });

        info!("Grid event system started");
        Ok(())
    }

    /// Subscribe to grid events
    pub async fn subscribe(
        &self,
        subscriber_id: String,
        event_types: Vec<GridEventType>,
        regions: Vec<RegionId>,
        users: Vec<Uuid>,
        callback: String,
    ) -> Result<()> {
        info!("Adding event subscription for {}", subscriber_id);

        let subscription = EventSubscription {
            subscriber_id: subscriber_id.clone(),
            event_types,
            regions,
            users,
            callback,
        };

        self.subscriptions.write().await.insert(subscriber_id, subscription);
        
        // Update stats
        self.stats.write().await.active_subscriptions = self.subscriptions.read().await.len();

        Ok(())
    }

    /// Unsubscribe from grid events
    pub async fn unsubscribe(&self, subscriber_id: &str) -> Result<()> {
        info!("Removing event subscription for {}", subscriber_id);

        self.subscriptions.write().await.remove(subscriber_id);
        
        // Update stats
        self.stats.write().await.active_subscriptions = self.subscriptions.read().await.len();

        Ok(())
    }

    /// Publish a grid event
    pub async fn publish_event(&self, event: GridEvent) -> Result<()> {
        info!("Publishing grid event: {:?} with ID {}", event.event_type, event.event_id);

        // Add to processing queue
        self.event_queue.write().await.push(event);

        Ok(())
    }

    /// Publish a user status change event
    pub async fn publish_user_status_change(
        &self,
        user_id: Uuid,
        status: &str,
        region_id: Option<RegionId>,
    ) -> Result<()> {
        let event = GridEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: GridEventType::UserStatusChange,
            source_region: region_id,
            source_user: Some(user_id),
            target_regions: Vec::new(), // Broadcast to all
            target_users: Vec::new(), // All users
            payload: LLSDValue::Map({
                let mut map = HashMap::new();
                map.insert("user_id".to_string(), LLSDValue::String(user_id.to_string()));
                map.insert("status".to_string(), LLSDValue::String(status.to_string()));
                if let Some(region) = region_id {
                    map.insert("region_id".to_string(), LLSDValue::Integer(region.0 as i32));
                }
                map.insert("timestamp".to_string(), LLSDValue::Integer(self.current_timestamp() as i32));
                map
            }),
            timestamp: self.current_timestamp(),
            priority: EventPriority::Normal,
            persistent: false,
            expiry: None,
        };

        self.publish_event(event).await
    }

    /// Publish a chat message event
    pub async fn publish_chat_message(
        &self,
        sender_id: Uuid,
        message: &str,
        channel: i32,
        region_id: RegionId,
        position: Option<(f32, f32, f32)>,
    ) -> Result<()> {
        let event = GridEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: GridEventType::ChatMessage,
            source_region: Some(region_id),
            source_user: Some(sender_id),
            target_regions: vec![region_id], // Only to same region for local chat
            target_users: Vec::new(),
            payload: LLSDValue::Map({
                let mut map = HashMap::new();
                map.insert("sender_id".to_string(), LLSDValue::String(sender_id.to_string()));
                map.insert("message".to_string(), LLSDValue::String(message.to_string()));
                map.insert("channel".to_string(), LLSDValue::Integer(channel));
                map.insert("region_id".to_string(), LLSDValue::Integer(region_id.0 as i32));
                
                if let Some(pos) = position {
                    map.insert("position".to_string(), LLSDValue::Array(vec![
                        LLSDValue::Real(pos.0 as f64),
                        LLSDValue::Real(pos.1 as f64),
                        LLSDValue::Real(pos.2 as f64),
                    ]));
                }
                
                map.insert("timestamp".to_string(), LLSDValue::Integer(self.current_timestamp() as i32));
                map
            }),
            timestamp: self.current_timestamp(),
            priority: EventPriority::Normal,
            persistent: false,
            expiry: None,
        };

        self.publish_event(event).await
    }

    /// Publish an economy transaction event
    pub async fn publish_economy_transaction(
        &self,
        from_user: Uuid,
        to_user: Uuid,
        amount: i32,
        description: &str,
        region_id: Option<RegionId>,
    ) -> Result<()> {
        let event = GridEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: GridEventType::EconomyTransaction,
            source_region: region_id,
            source_user: Some(from_user),
            target_regions: Vec::new(),
            target_users: vec![from_user, to_user], // Only notify involved users
            payload: LLSDValue::Map({
                let mut map = HashMap::new();
                map.insert("from_user".to_string(), LLSDValue::String(from_user.to_string()));
                map.insert("to_user".to_string(), LLSDValue::String(to_user.to_string()));
                map.insert("amount".to_string(), LLSDValue::Integer(amount));
                map.insert("description".to_string(), LLSDValue::String(description.to_string()));
                map.insert("timestamp".to_string(), LLSDValue::Integer(self.current_timestamp() as i32));
                map
            }),
            timestamp: self.current_timestamp(),
            priority: EventPriority::High,
            persistent: true, // Keep for offline users
            expiry: Some(self.current_timestamp() + 86400 * 7), // 7 days
        };

        self.publish_event(event).await
    }

    /// Publish a region status change event
    pub async fn publish_region_status_change(
        &self,
        region_id: RegionId,
        status: &str,
        additional_info: Option<HashMap<String, LLSDValue>>,
    ) -> Result<()> {
        let mut payload = HashMap::new();
        payload.insert("region_id".to_string(), LLSDValue::Integer(region_id.0 as i32));
        payload.insert("status".to_string(), LLSDValue::String(status.to_string()));
        payload.insert("timestamp".to_string(), LLSDValue::Integer(self.current_timestamp() as i32));
        
        if let Some(info) = additional_info {
            payload.extend(info);
        }

        let event = GridEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: GridEventType::RegionStatusChange,
            source_region: Some(region_id),
            source_user: None,
            target_regions: Vec::new(), // Broadcast to all
            target_users: Vec::new(),
            payload: LLSDValue::Map(payload),
            timestamp: self.current_timestamp(),
            priority: EventPriority::High,
            persistent: false,
            expiry: None,
        };

        self.publish_event(event).await
    }

    /// Get events for a specific user (including persistent events)
    pub async fn get_user_events(&self, user_id: Uuid) -> Vec<GridEvent> {
        self.persistent_events.read().await
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear events for a user (when they come online)
    pub async fn clear_user_events(&self, user_id: Uuid) -> usize {
        self.persistent_events.write().await
            .remove(&user_id)
            .map(|events| events.len())
            .unwrap_or(0)
    }

    /// Get event statistics
    pub async fn get_stats(&self) -> EventStats {
        self.stats.read().await.clone()
    }

    /// Process events from the queue
    async fn event_processing_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        
        loop {
            interval.tick().await;
            
            let events = {
                let mut queue = self.event_queue.write().await;
                if queue.is_empty() {
                    continue;
                }
                
                // Sort by priority and take up to 20 events
                queue.sort_by(|a, b| b.priority.cmp(&a.priority));
                let count = queue.len().min(20);
                queue.drain(..count).collect::<Vec<_>>()
            };

            for event in events {
                if let Err(e) = self.process_event(event).await {
                    error!("Failed to process event: {}", e);
                    self.stats.write().await.failed_deliveries += 1;
                }
            }
        }
    }

    /// Process a single event
    async fn process_event(&self, event: GridEvent) -> Result<()> {
        info!("Processing event {} of type {:?}", event.event_id, event.event_type);

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_events_sent += 1;
            *stats.events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
        }

        // Store persistent events
        if event.persistent {
            self.store_persistent_event(&event).await;
        }

        // Send to subscribers
        self.notify_subscribers(&event).await?;

        // Send via inter-region communication if needed
        if !event.target_regions.is_empty() || event.target_regions.is_empty() {
            self.send_via_inter_region(&event).await?;
        }

        // Broadcast on local channel
        if let Err(e) = self.event_tx.send(event) {
            warn!("Failed to broadcast event locally: {}", e);
        }

        Ok(())
    }

    /// Store a persistent event for offline users
    async fn store_persistent_event(&self, event: &GridEvent) {
        if event.target_users.is_empty() {
            // If no specific users, don't store (broadcast events aren't persistent per user)
            return;
        }

        let mut persistent = self.persistent_events.write().await;
        
        for user_id in &event.target_users {
            persistent.entry(*user_id).or_default().push(event.clone());
        }

        // Update stats
        self.stats.write().await.persistent_events_stored = 
            persistent.values().map(|events| events.len()).sum();
    }

    /// Notify subscribers about an event
    async fn notify_subscribers(&self, event: &GridEvent) -> Result<()> {
        let subscriptions = self.subscriptions.read().await;
        
        for subscription in subscriptions.values() {
            if self.matches_subscription(event, subscription) {
                // In a real implementation, this would call the subscriber's callback
                info!(
                    "Notifying subscriber {} about event {} via {}",
                    subscription.subscriber_id, event.event_id, subscription.callback
                );
            }
        }

        Ok(())
    }

    /// Check if an event matches a subscription
    fn matches_subscription(&self, event: &GridEvent, subscription: &EventSubscription) -> bool {
        // Check event type
        if !subscription.event_types.contains(&event.event_type) {
            return false;
        }

        // Check regions
        if !subscription.regions.is_empty() {
            if let Some(source_region) = event.source_region {
                if !subscription.regions.contains(&source_region) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check users
        if !subscription.users.is_empty() {
            if let Some(source_user) = event.source_user {
                if !subscription.users.contains(&source_user) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Send event via inter-region communication
    async fn send_via_inter_region(&self, event: &GridEvent) -> Result<()> {
        let payload = LLSDValue::Map({
            let mut map = HashMap::new();
            map.insert("event_id".to_string(), LLSDValue::String(event.event_id.clone()));
            map.insert("event_type".to_string(), LLSDValue::String(format!("{:?}", event.event_type)));
            map.insert("payload".to_string(), event.payload.clone());
            map.insert("timestamp".to_string(), LLSDValue::Integer(event.timestamp as i32));
            map.insert("priority".to_string(), LLSDValue::Integer(event.priority as i32));
            map
        });

        if event.target_regions.is_empty() {
            // Broadcast to all regions
            self.inter_region_manager.broadcast_message(
                InterRegionMessageType::GridAnnouncement,
                payload,
                match event.priority {
                    EventPriority::Low => MessagePriority::Low,
                    EventPriority::Normal => MessagePriority::Normal,
                    EventPriority::High | EventPriority::Critical | EventPriority::Emergency => MessagePriority::High,
                },
            ).await?;
        } else {
            // Send to specific regions
            for region_id in &event.target_regions {
                let message = self.inter_region_manager.create_message(
                    InterRegionMessageType::GridAnnouncement,
                    Some(*region_id),
                    payload.clone(),
                    match event.priority {
                        EventPriority::Low => MessagePriority::Low,
                        EventPriority::Normal => MessagePriority::Normal,
                        EventPriority::High | EventPriority::Critical | EventPriority::Emergency => MessagePriority::High,
                    },
                    false,
                );
                
                self.inter_region_manager.send_message(message).await?;
            }
        }

        Ok(())
    }

    /// Clean up expired persistent events
    async fn cleanup_expired_events(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour
        
        loop {
            interval.tick().await;
            
            let current_time = self.current_timestamp();
            let mut persistent = self.persistent_events.write().await;
            let mut cleaned_count = 0;
            
            for events in persistent.values_mut() {
                let original_len = events.len();
                events.retain(|event| {
                    if let Some(expiry) = event.expiry {
                        expiry > current_time
                    } else {
                        true // No expiry, keep forever
                    }
                });
                cleaned_count += original_len - events.len();
            }
            
            // Remove empty user entries
            persistent.retain(|_, events| !events.is_empty());
            
            if cleaned_count > 0 {
                info!("Cleaned up {} expired persistent events", cleaned_count);
                
                // Update stats
                self.stats.write().await.persistent_events_stored = 
                    persistent.values().map(|events| events.len()).sum();
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

// Implement Clone for GridEventManager (needed for spawning tasks)
impl Clone for GridEventManager {
    fn clone(&self) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        
        Self {
            subscriptions: RwLock::new(HashMap::new()),
            persistent_events: RwLock::new(HashMap::new()),
            stats: RwLock::new(EventStats {
                total_events_sent: 0,
                events_by_type: HashMap::new(),
                active_subscriptions: 0,
                persistent_events_stored: 0,
                failed_deliveries: 0,
            }),
            event_tx,
            inter_region_manager: self.inter_region_manager.clone(),
            event_queue: RwLock::new(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ffi::physics::PhysicsBridge,
        state::StateManager,
        region::RegionManager,
        network::inter_region::InterRegionManager,
    };

    #[tokio::test]
    async fn test_grid_event_manager_creation() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        let inter_region_manager = Arc::new(InterRegionManager::new(region_manager, state_manager));
        
        let grid_event_manager = GridEventManager::new(inter_region_manager);
        
        // Test subscription
        grid_event_manager.subscribe(
            "test_subscriber".to_string(),
            vec![GridEventType::UserStatusChange],
            vec![RegionId(1)],
            Vec::new(),
            "http://localhost:8080/events".to_string(),
        ).await?;
        
        let stats = grid_event_manager.get_stats().await;
        assert_eq!(stats.active_subscriptions, 1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_event_publishing() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        let inter_region_manager = Arc::new(InterRegionManager::new(region_manager, state_manager));
        
        let grid_event_manager = GridEventManager::new(inter_region_manager);
        
        let user_id = Uuid::new_v4();
        
        // Test user status change event
        grid_event_manager.publish_user_status_change(
            user_id,
            "online",
            Some(RegionId(1)),
        ).await?;
        
        // Test chat message event
        grid_event_manager.publish_chat_message(
            user_id,
            "Hello, world!",
            0,
            RegionId(1),
            Some((128.0, 128.0, 21.0)),
        ).await?;
        
        Ok(())
    }
}