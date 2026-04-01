//! WebSocket protocol support for real-time client communication
//! 
//! This module provides comprehensive WebSocket support for OpenSim Next, enabling:
//! - Real-time bidirectional communication with Second Life viewers
//! - Web-based client interfaces 
//! - Binary and text message protocols
//! - Authentication and session management
//! - Message routing and handling

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant},
};
use tokio::{
    sync::{RwLock, mpsc, broadcast},
    time::timeout,
};
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade, Message, CloseFrame},
        State, ConnectInfo, Query,
    },
    response::Response,
    routing::get,
    Router,
    http::{StatusCode, HeaderMap},
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::{anyhow, Result};
use tracing::{info, warn, error, debug, trace};

use crate::{
    monitoring::MonitoringSystem,
    network::{
        llsd::{LLSDFormat, LLSDMessage, LLSDMessageHandler},
        session::{Session, SessionManager},
        security::SecurityManager,
        client::{ClientMessage, ViewerInfo},
    },
    region::RegionManager,
    state::StateManager,
    asset::AssetManager,
    database::user_accounts::UserAccountDatabase,
};

/// WebSocket server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub port: u16,
    pub max_connections: usize,
    pub message_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub max_message_size: usize,
    pub compression_enabled: bool,
    pub rate_limit_per_second: u32,
    pub authentication_required: bool,
    pub cors_enabled: bool,
    pub allowed_origins: Vec<String>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 9001,
            max_connections: 1000,
            message_timeout_ms: 30000,
            heartbeat_interval_ms: 30000,
            max_message_size: 1024 * 1024, // 1MB
            compression_enabled: true,
            rate_limit_per_second: 100,
            authentication_required: true,
            cors_enabled: true,
            allowed_origins: vec![
                "https://viewer.opensim.org".to_string(),
                "https://web.opensim.org".to_string(),
                "http://localhost:3000".to_string(),
            ],
        }
    }
}

/// WebSocket message types for OpenSim protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessageType {
    // Authentication and session
    Auth { token: Option<String>, session_id: Option<String> },
    AuthResponse { success: bool, session_id: Option<String>, error: Option<String> },

    // Client-server communication
    LLSD { data: String, format: String }, // LLSD message wrapper
    LLSDResponse { data: String, format: String, success: bool },

    // Real-time updates
    AvatarUpdate { agent_id: String, position: [f64; 3], rotation: [f64; 4] },
    ObjectUpdate { object_id: String, position: [f64; 3], rotation: [f64; 4] },
    ChatMessage { from: String, message: String, channel: i32 },

    // Region and world
    RegionUpdate { region_id: String, stats: RegionStats },
    TeleportRequest { region: String, position: [f64; 3] },
    TeleportResponse { success: bool, region: Option<String>, error: Option<String> },

    // Asset management
    AssetRequest { asset_id: String, asset_type: String },
    AssetResponse { asset_id: String, data: Option<String>, success: bool },

    // System messages
    Heartbeat,
    Pong,
    Error { code: u32, message: String },
    Disconnect { reason: String },

    // Instance Manager - Control Commands
    InstanceControl {
        instance_id: String,
        command: String,
        parameters: Option<serde_json::Value>,
    },
    InstanceControlResponse {
        instance_id: String,
        success: bool,
        message: String,
        data: Option<serde_json::Value>,
        duration_ms: u64,
    },

    // Instance Manager - Status Updates
    InstanceStatusUpdate {
        instance_id: String,
        status: String,
        metrics: Option<InstanceMetricsUpdate>,
        health: Option<InstanceHealthUpdate>,
    },
    InstanceList {
        instances: Vec<InstanceInfoMessage>,
    },

    // Instance Manager - Console Streaming
    ConsoleCommand {
        instance_id: String,
        command: String,
    },
    ConsoleOutput {
        instance_id: String,
        content: String,
        output_type: String,
        timestamp: u64,
    },

    // Instance Manager - User Management
    UserManagement {
        instance_id: String,
        action: String,
        user_id: Option<String>,
        parameters: Option<serde_json::Value>,
    },
    UserManagementResponse {
        instance_id: String,
        success: bool,
        message: String,
        data: Option<serde_json::Value>,
    },

    // Instance Manager - Region Control
    RegionControl {
        instance_id: String,
        action: String,
        region_id: Option<String>,
        parameters: Option<serde_json::Value>,
    },
    RegionControlResponse {
        instance_id: String,
        success: bool,
        message: String,
        data: Option<serde_json::Value>,
    },

    // Instance Manager - Subscriptions
    Subscribe {
        instance_id: Option<String>,
        channels: Vec<String>,
    },
    Unsubscribe {
        instance_id: Option<String>,
        channels: Vec<String>,
    },
    SubscriptionConfirmed {
        channels: Vec<String>,
    },

    // Instance Manager - Batch Operations
    BatchCommand {
        instance_ids: Vec<String>,
        command: String,
        parameters: Option<serde_json::Value>,
    },
    BatchCommandResponse {
        results: Vec<BatchResultMessage>,
        total_duration_ms: u64,
    },

    // Instance Manager - Controller Events
    InstanceAnnounced {
        instance_id: String,
        name: String,
        mode: String,
        host: String,
        ports: serde_json::Value,
        capabilities: Vec<String>,
    },
    InstanceDeparted {
        instance_id: String,
        reason: String,
    },
    ProcessOutput {
        instance_id: String,
        stream: String,
        line: String,
        timestamp: u64,
    },

    // Archive Operations (IAR/OAR)
    ArchiveSubscribe {
        job_id: Option<String>,
    },
    ArchiveUnsubscribe {
        job_id: Option<String>,
    },
    ArchiveProgress {
        job_id: String,
        job_type: String,
        status: String,
        progress: f64,
        message: Option<String>,
        items_processed: Option<u64>,
        items_total: Option<u64>,
        elapsed_ms: u64,
    },
    ArchiveCompleted {
        job_id: String,
        job_type: String,
        success: bool,
        message: String,
        result: Option<serde_json::Value>,
        elapsed_ms: u64,
    },
    ArchiveFailed {
        job_id: String,
        job_type: String,
        error: String,
        elapsed_ms: u64,
    },
}

/// WebSocket message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub id: String,
    pub timestamp: u64,
    pub message: WebSocketMessageType,
    pub sequence: Option<u64>,
}

/// Statistics for WebSocket connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketStats {
    pub total_connections: u64,
    pub active_connections: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u64,
    pub timeouts: u64,
}

/// Individual WebSocket connection handler
pub struct WebSocketConnection {
    pub id: Uuid,
    pub addr: SocketAddr,
    pub session: Arc<RwLock<Option<Session>>>,
    pub authenticated: Arc<RwLock<bool>>,
    pub last_activity: Arc<RwLock<Instant>>,
    pub viewer_info: Arc<RwLock<Option<ViewerInfo>>>,
    message_tx: mpsc::UnboundedSender<WebSocketMessage>,
    stats: Arc<RwLock<WebSocketStats>>,
    sequence_counter: Arc<AtomicU64>,
}

impl WebSocketConnection {
    pub fn new(id: Uuid, addr: SocketAddr) -> (Self, mpsc::UnboundedReceiver<WebSocketMessage>) {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        let connection = Self {
            id,
            addr,
            session: Arc::new(RwLock::new(None)),
            authenticated: Arc::new(RwLock::new(false)),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            viewer_info: Arc::new(RwLock::new(None)),
            message_tx,
            stats: Arc::new(RwLock::new(WebSocketStats {
                total_connections: 1,
                active_connections: 1,
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                errors: 0,
                timeouts: 0,
            })),
            sequence_counter: Arc::new(AtomicU64::new(0)),
        };
        
        (connection, message_rx)
    }
    
    pub async fn send_message(&self, message: WebSocketMessage) -> Result<()> {
        self.message_tx.send(message)
            .map_err(|e| anyhow!("Failed to send WebSocket message: {}", e))?;
        
        let mut stats = self.stats.write().await;
        stats.messages_sent += 1;
        
        Ok(())
    }
    
    pub async fn update_activity(&self) {
        *self.last_activity.write().await = Instant::now();
    }
    
    pub async fn set_session(&self, session: Session) {
        *self.session.write().await = Some(session);
        *self.authenticated.write().await = true;
    }
    
    pub async fn is_authenticated(&self) -> bool {
        *self.authenticated.read().await
    }
    
    pub async fn get_session(&self) -> Option<Session> {
        self.session.read().await.clone()
    }
    
    pub async fn get_stats(&self) -> WebSocketStats {
        self.stats.read().await.clone()
    }
    
    fn next_sequence(&self) -> u64 {
        self.sequence_counter.fetch_add(1, Ordering::SeqCst)
    }
}

/// WebSocket server for OpenSim client communication
pub struct WebSocketServer {
    config: WebSocketConfig,
    connections: Arc<RwLock<HashMap<Uuid, Arc<WebSocketConnection>>>>,
    session_manager: Arc<SessionManager>,
    security_manager: Arc<SecurityManager>,
    region_manager: Arc<RegionManager>,
    state_manager: Arc<StateManager>,
    asset_manager: Arc<AssetManager>,
    user_account_database: Arc<UserAccountDatabase>,
    monitoring: Arc<MonitoringSystem>,
    llsd_handler: LLSDMessageHandler,
    event_broadcaster: broadcast::Sender<WebSocketMessage>,
    global_stats: Arc<RwLock<WebSocketStats>>,
}

impl WebSocketServer {
    pub fn new(
        config: WebSocketConfig,
        session_manager: Arc<SessionManager>,
        security_manager: Arc<SecurityManager>,
        region_manager: Arc<RegionManager>,
        state_manager: Arc<StateManager>,
        asset_manager: Arc<AssetManager>,
        user_account_database: Arc<UserAccountDatabase>,
        monitoring: Arc<MonitoringSystem>,
    ) -> Self {
        let (event_broadcaster, _) = broadcast::channel(1000);
        
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            session_manager,
            security_manager,
            region_manager,
            state_manager,
            asset_manager,
            user_account_database,
            monitoring,
            llsd_handler: LLSDMessageHandler::default(),
            event_broadcaster,
            global_stats: Arc::new(RwLock::new(WebSocketStats {
                total_connections: 0,
                active_connections: 0,
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                errors: 0,
                timeouts: 0,
            })),
        }
    }
    
    /// Start the WebSocket server
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("WebSocket server is disabled");
            return Ok(());
        }
        
        info!("Starting WebSocket server on port {}", self.config.port);
        
        // Start cleanup task for inactive connections
        self.start_cleanup_task().await;
        
        // Start heartbeat task
        self.start_heartbeat_task().await;
        
        // Create router with WebSocket endpoint
        let app = self.create_router().await;
        
        // Start server
        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| anyhow!("Failed to bind WebSocket server to {}: {}", addr, e))?;
        
        info!("WebSocket server listening on {}", addr);
        info!("WebSocket endpoint: ws://{}/ws", addr);
        
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>()
        ).await
            .map_err(|e| anyhow!("WebSocket server error: {}", e))?;
        
        Ok(())
    }
    
    async fn create_router(&self) -> Router {
        let server_state = Arc::new(self.clone());
        
        Router::new()
            .route("/ws", get({
                let state = server_state.clone();
                move |ws, info, query, headers| {
                    websocket_handler(ws, State(state), info, query, headers)
                }
            }))
            .route("/", get(|| async { "OpenSim WebSocket Server" }))
            .route("/health", get(|| async { "OK" }))
            .route("/stats", get({
                let state = server_state.clone();
                move || websocket_stats_handler(State(state))
            }))
    }
    
    async fn start_cleanup_task(&self) {
        let connections = self.connections.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let mut to_remove = Vec::new();
                {
                    let conns = connections.read().await;
                    for (id, conn) in conns.iter() {
                        let last_activity = *conn.last_activity.read().await;
                        if last_activity.elapsed() > Duration::from_millis(config.heartbeat_interval_ms * 3) {
                            to_remove.push(*id);
                        }
                    }
                }
                
                if !to_remove.is_empty() {
                    let mut conns = connections.write().await;
                    for id in to_remove {
                        if let Some(conn) = conns.remove(&id) {
                            info!("Removing inactive WebSocket connection: {}", id);
                        }
                    }
                }
            }
        });
    }
    
    async fn start_heartbeat_task(&self) {
        let connections = self.connections.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(config.heartbeat_interval_ms));
            
            loop {
                interval.tick().await;
                
                let conns = connections.read().await;
                for (_, conn) in conns.iter() {
                    let heartbeat_msg = WebSocketMessage {
                        id: Uuid::new_v4().to_string(),
                        timestamp: chrono::Utc::now().timestamp() as u64,
                        message: WebSocketMessageType::Heartbeat,
                        sequence: Some(conn.next_sequence()),
                    };
                    
                    if let Err(e) = conn.send_message(heartbeat_msg).await {
                        debug!("Failed to send heartbeat to {}: {}", conn.id, e);
                    }
                }
            }
        });
    }
    
    pub async fn handle_websocket_connection(
        &self,
        socket: WebSocket,
        addr: SocketAddr,
        _query: Option<WebSocketQuery>,
        _headers: HeaderMap,
    ) {
        let connection_id = Uuid::new_v4();
        info!("New WebSocket client connected: {} from {}", connection_id, addr);
        
        // Update global stats
        {
            let mut stats = self.global_stats.write().await;
            stats.total_connections += 1;
            stats.active_connections += 1;
        }
        
        // Check connection limit
        {
            let current_count = self.connections.read().await.len();
            if current_count >= self.config.max_connections {
                warn!("WebSocket connection limit reached, rejecting {}", addr);
                return;
            }
        }
        
        let (connection, message_rx) = WebSocketConnection::new(connection_id, addr);
        let connection_arc = Arc::new(connection);
        
        // Add to active connections
        self.connections.write().await.insert(connection_id, connection_arc.clone());
        
        let (mut sender, mut receiver) = socket.split();
        
        // Send welcome message
        let welcome_msg = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            message: WebSocketMessageType::AuthResponse {
                success: false,
                session_id: None,
                error: Some("Authentication required".to_string()),
            },
            sequence: Some(connection_arc.next_sequence()),
        };
        
        if let Ok(json) = serde_json::to_string(&welcome_msg) {
            let _ = sender.send(Message::Text(json)).await;
        }
        
        // Spawn task to handle outgoing messages
        let sender_task = {
            let connection = connection_arc.clone();
            let mut message_receiver = message_rx;
            
            tokio::spawn(async move {
                while let Some(msg) = message_receiver.recv().await {
                    match serde_json::to_string(&msg) {
                        Ok(json) => {
                            let message_size = json.len();
                            match sender.send(Message::Text(json)).await {
                                Ok(_) => {
                                    let mut stats = connection.stats.write().await;
                                    stats.bytes_sent += message_size as u64;
                                }
                                Err(e) => {
                                    debug!("Failed to send WebSocket message: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize WebSocket message: {}", e);
                            let mut stats = connection.stats.write().await;
                            stats.errors += 1;
                        }
                    }
                }
            })
        };
        
        // Spawn task to handle incoming messages
        let receiver_task = {
            let connection = connection_arc.clone();
            let server = self.clone();
            
            tokio::spawn(async move {
                while let Some(msg) = receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            connection.update_activity().await;
                            let message_size = text.len();
                            
                            {
                                let mut stats = connection.stats.write().await;
                                stats.messages_received += 1;
                                stats.bytes_received += message_size as u64;
                            }
                            
                            if let Err(e) = server.handle_text_message(&connection, text).await {
                                error!("Error handling WebSocket text message: {}", e);
                                let mut stats = connection.stats.write().await;
                                stats.errors += 1;
                            }
                        }
                        Ok(Message::Binary(data)) => {
                            connection.update_activity().await;
                            let message_size = data.len();
                            
                            {
                                let mut stats = connection.stats.write().await;
                                stats.messages_received += 1;
                                stats.bytes_received += message_size as u64;
                            }
                            
                            if let Err(e) = server.handle_binary_message(&connection, data).await {
                                error!("Error handling WebSocket binary message: {}", e);
                                let mut stats = connection.stats.write().await;
                                stats.errors += 1;
                            }
                        }
                        Ok(Message::Ping(data)) => {
                            debug!("WebSocket ping received from {}", connection.id);
                            // Pong is automatically sent by axum
                        }
                        Ok(Message::Pong(_)) => {
                            connection.update_activity().await;
                            debug!("WebSocket pong received from {}", connection.id);
                        }
                        Ok(Message::Close(frame)) => {
                            info!("WebSocket client {} closed connection: {:?}", connection.id, frame);
                            break;
                        }
                        Err(e) => {
                            warn!("WebSocket error from {}: {}", connection.id, e);
                            let mut stats = connection.stats.write().await;
                            stats.errors += 1;
                            break;
                        }
                    }
                }
            })
        };
        
        // Wait for either task to complete
        tokio::select! {
            _ = sender_task => {},
            _ = receiver_task => {},
        }
        
        // Clean up connection
        self.connections.write().await.remove(&connection_id);
        
        // Update global stats
        {
            let mut stats = self.global_stats.write().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
        }
        
        info!("WebSocket client {} disconnected", connection_id);
    }
    
    async fn handle_text_message(&self, connection: &Arc<WebSocketConnection>, text: String) -> Result<()> {
        trace!("Received WebSocket text message: {}", text);
        
        let message: WebSocketMessage = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Failed to parse WebSocket message: {}", e))?;
        
        self.process_websocket_message(connection, message).await
    }
    
    async fn handle_binary_message(&self, connection: &Arc<WebSocketConnection>, data: Vec<u8>) -> Result<()> {
        trace!("Received WebSocket binary message: {} bytes", data.len());
        
        // Try to parse as LLSD binary format
        match LLSDMessage::from_bytes(&data, LLSDFormat::Binary) {
            Ok(llsd_msg) => {
                self.handle_llsd_message(connection, llsd_msg).await
            }
            Err(e) => {
                warn!("Failed to parse binary message as LLSD: {}", e);
                Err(anyhow!("Invalid binary message format"))
            }
        }
    }
    
    async fn process_websocket_message(&self, connection: &Arc<WebSocketConnection>, message: WebSocketMessage) -> Result<()> {
        match message.message {
            WebSocketMessageType::Auth { token, session_id } => {
                self.handle_auth_message(connection, token, session_id).await
            }
            WebSocketMessageType::LLSD { data, format } => {
                self.handle_llsd_wrapper(connection, data, format).await
            }
            WebSocketMessageType::AvatarUpdate { agent_id, position, rotation } => {
                self.handle_avatar_update(connection, agent_id, position, rotation).await
            }
            WebSocketMessageType::ChatMessage { from, message: chat_msg, channel } => {
                self.handle_chat_message(connection, from, chat_msg, channel).await
            }
            WebSocketMessageType::TeleportRequest { region, position } => {
                self.handle_teleport_request(connection, region, position).await
            }
            WebSocketMessageType::AssetRequest { asset_id, asset_type } => {
                self.handle_asset_request(connection, asset_id, asset_type).await
            }
            WebSocketMessageType::Pong => {
                // Heartbeat response
                connection.update_activity().await;
                Ok(())
            }
            _ => {
                warn!("Unhandled WebSocket message type: {:?}", message.message);
                Ok(())
            }
        }
    }
    
    async fn handle_auth_message(&self, connection: &Arc<WebSocketConnection>, token: Option<String>, session_id: Option<String>) -> Result<()> {
        let success = if let Some(token) = token {
            // Look up session by token (treating token as session_id for now)
            if let Some(session) = self.session_manager.get_session(&token).await {
                connection.set_session(session).await;
                true
            } else {
                false
            }
        } else if let Some(session_id) = session_id {
            // Look up existing session
            if let Some(session) = self.session_manager.get_session(&session_id).await {
                connection.set_session(session).await;
                true
            } else {
                false
            }
        } else {
            false
        };
        
        let response = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            message: WebSocketMessageType::AuthResponse {
                success,
                session_id: if success { connection.get_session().await.map(|s| s.session_id) } else { None },
                error: if !success { Some("Authentication failed".to_string()) } else { None },
            },
            sequence: Some(connection.next_sequence()),
        };
        
        connection.send_message(response).await
    }
    
    async fn handle_llsd_wrapper(&self, connection: &Arc<WebSocketConnection>, data: String, format: String) -> Result<()> {
        if !connection.is_authenticated().await {
            return Err(anyhow!("Authentication required for LLSD messages"));
        }
        
        let llsd_format = match format.as_str() {
            "xml" => LLSDFormat::Xml,
            "binary" => LLSDFormat::Binary,
            _ => return Err(anyhow!("Unsupported LLSD format: {}", format)),
        };
        
        let llsd_msg = LLSDMessage::from_bytes(data.as_bytes(), llsd_format)?;
        self.handle_llsd_message(connection, llsd_msg).await
    }
    
    async fn handle_llsd_message(&self, connection: &Arc<WebSocketConnection>, llsd_msg: LLSDMessage) -> Result<()> {
        if !connection.is_authenticated().await {
            return Err(anyhow!("Authentication required for LLSD messages"));
        }
        
        let session = if let Some(session) = connection.session.read().await.clone() {
            Arc::new(RwLock::new(session))
        } else {
            return Err(anyhow!("No session available for LLSD message"));
        };
        
        match self.llsd_handler.handle_message(
            llsd_msg,
            session,
            self.security_manager.clone(),
            self.session_manager.clone(),
            self.region_manager.clone(),
            self.state_manager.clone(),
            self.asset_manager.clone(),
            self.user_account_database.clone(),
        ).await {
            Ok(Some(response)) => {
                let response_msg = WebSocketMessage {
                    id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    message: WebSocketMessageType::LLSDResponse {
                        data: String::from_utf8(response.to_bytes(LLSDFormat::Xml)?)?,
                        format: "xml".to_string(),
                        success: true,
                    },
                    sequence: Some(connection.next_sequence()),
                };
                
                connection.send_message(response_msg).await
            }
            Ok(None) => Ok(()),
            Err(e) => {
                error!("LLSD message handling error: {}", e);
                
                let error_msg = WebSocketMessage {
                    id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    message: WebSocketMessageType::Error {
                        code: 500,
                        message: format!("LLSD processing error: {}", e),
                    },
                    sequence: Some(connection.next_sequence()),
                };
                
                connection.send_message(error_msg).await
            }
        }
    }
    
    async fn handle_avatar_update(&self, _connection: &Arc<WebSocketConnection>, _agent_id: String, _position: [f64; 3], _rotation: [f64; 4]) -> Result<()> {
        // Handle avatar position/rotation updates
        // This would integrate with the region manager to update avatar state
        // and broadcast to other clients in the same region
        Ok(())
    }
    
    async fn handle_chat_message(&self, _connection: &Arc<WebSocketConnection>, _from: String, _message: String, _channel: i32) -> Result<()> {
        // Handle chat messages
        // This would broadcast to other clients in range
        Ok(())
    }
    
    async fn handle_teleport_request(&self, connection: &Arc<WebSocketConnection>, region: String, position: [f64; 3]) -> Result<()> {
        // Handle teleport requests
        let response = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            message: WebSocketMessageType::TeleportResponse {
                success: true,
                region: Some(region),
                error: None,
            },
            sequence: Some(connection.next_sequence()),
        };
        
        connection.send_message(response).await
    }
    
    async fn handle_asset_request(&self, connection: &Arc<WebSocketConnection>, asset_id: String, asset_type: String) -> Result<()> {
        // Handle asset requests
        let response = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            message: WebSocketMessageType::AssetResponse {
                asset_id: asset_id.clone(),
                data: None, // Would load from asset manager
                success: false, // Placeholder
            },
            sequence: Some(connection.next_sequence()),
        };
        
        connection.send_message(response).await
    }
    
    pub async fn get_stats(&self) -> WebSocketStats {
        self.global_stats.read().await.clone()
    }
    
    pub async fn get_connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
    
    pub async fn broadcast_message(&self, message: WebSocketMessage) -> Result<()> {
        let connections = self.connections.read().await;
        let mut errors = 0;
        
        for (_, conn) in connections.iter() {
            if let Err(_) = conn.send_message(message.clone()).await {
                errors += 1;
            }
        }
        
        if errors > 0 {
            warn!("Failed to send broadcast message to {} connections", errors);
        }
        
        Ok(())
    }

    /// Get the WebSocket server port
    pub fn get_port(&self) -> u16 {
        self.config.port
    }
}

impl Clone for WebSocketServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            connections: self.connections.clone(),
            session_manager: self.session_manager.clone(),
            security_manager: self.security_manager.clone(),
            region_manager: self.region_manager.clone(),
            state_manager: self.state_manager.clone(),
            asset_manager: self.asset_manager.clone(),
            user_account_database: self.user_account_database.clone(),
            monitoring: self.monitoring.clone(),
            llsd_handler: LLSDMessageHandler::default(),
            event_broadcaster: self.event_broadcaster.clone(),
            global_stats: self.global_stats.clone(),
        }
    }
}

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    pub token: Option<String>,
    pub session_id: Option<String>,
    pub viewer: Option<String>,
    pub version: Option<String>,
}

/// Region statistics for WebSocket updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionStats {
    pub name: String,
    pub users: u32,
    pub objects: u32,
    pub scripts: u32,
    pub physics_fps: f64,
}

/// Instance metrics update for WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMetricsUpdate {
    pub cpu_usage: f64,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub active_users: u32,
    pub active_regions: u32,
    pub network_tx_bytes: u64,
    pub network_rx_bytes: u64,
    pub db_connections: u32,
    pub websocket_connections: u32,
    pub request_rate_per_sec: f64,
    pub error_rate_per_sec: f64,
    pub uptime_seconds: u64,
}

/// Instance health update for WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceHealthUpdate {
    pub overall: String,
    pub components: std::collections::HashMap<String, ComponentHealthUpdate>,
    pub last_check: u64,
    pub response_time_ms: u64,
}

/// Component health update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthUpdate {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// Instance info message for WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfoMessage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub environment: String,
    pub status: String,
    pub metrics: Option<InstanceMetricsUpdate>,
    pub health: Option<InstanceHealthUpdate>,
    pub version: Option<String>,
    pub last_seen: u64,
    pub connected: bool,
    pub tags: Vec<String>,
}

/// Batch result message for WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResultMessage {
    pub instance_id: String,
    pub status: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub duration_ms: u64,
}

/// WebSocket handler for axum
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(server): State<Arc<WebSocketServer>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    query: Option<Query<WebSocketQuery>>,
    headers: HeaderMap,
) -> Response {
    let query_params = query.map(|q| q.0);
    let server_clone = server.clone();
    
    ws.on_upgrade(move |socket| async move {
        server_clone.handle_websocket_connection(socket, addr, query_params, headers).await
    })
}

/// WebSocket statistics handler
async fn websocket_stats_handler(State(server): State<Arc<WebSocketServer>>) -> Result<axum::Json<WebSocketStats>, StatusCode> {
    match server.get_stats().await {
        stats => Ok(axum::Json(stats)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_websocket_config() {
        let config = WebSocketConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 9001);
        assert!(config.max_connections > 0);
    }
    
    #[tokio::test]
    async fn test_websocket_connection() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let (conn, _rx) = WebSocketConnection::new(Uuid::new_v4(), addr);
        
        assert!(!conn.is_authenticated().await);
        assert!(conn.get_session().await.is_none());
    }
    
    #[tokio::test]
    async fn test_websocket_message_serialization() -> Result<()> {
        let msg = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: 1234567890,
            message: WebSocketMessageType::Heartbeat,
            sequence: Some(1),
        };
        
        let json = serde_json::to_string(&msg)?;
        let deserialized: WebSocketMessage = serde_json::from_str(&json)?;
        
        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.timestamp, deserialized.timestamp);
        
        Ok(())
    }
}