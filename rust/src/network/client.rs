use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, RwLock},
};
use uuid::Uuid;
use tracing::{debug, info, warn, error};

use crate::{
    monitoring::MonitoringSystem,
    network::{
        llsd::{LLSDFormat, LLSDMessage, LLSDMessageHandler},
        security::SecurityManager,
        session::{Session, SessionManager},
        NetworkConfig,
    },
    region::RegionManager,
    state::StateManager,
    database::user_accounts::UserAccountDatabase,
};

/// Information about the client's viewer.
#[derive(Debug, Clone)]
pub struct ViewerInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub channel: String,
}

/// Handles a client connection.
pub struct ClientConnection {
    pub id: Uuid,
    pub addr: SocketAddr,
    pub stream: Arc<tokio::sync::Mutex<TcpStream>>,
    pub session: Arc<RwLock<Session>>,
    llsd_handler: LLSDMessageHandler,
    security_manager: Arc<SecurityManager>,
    region_manager: Arc<RegionManager>,
    session_manager: Arc<SessionManager>,
    monitoring: Arc<MonitoringSystem>,
    state_manager: Arc<StateManager>,
    asset_manager: Arc<crate::asset::AssetManager>,
    user_account_database: Arc<UserAccountDatabase>,
    pub message_tx: mpsc::UnboundedSender<ClientMessage>,
    pub shutdown_tx: mpsc::Sender<()>,
}

/// Represents messages sent to or from a client.
#[derive(Debug, Clone)]
pub enum ClientMessage {
    Incoming(LLSDMessage),
    Outgoing(LLSDMessage),
    Shutdown,
}

impl ClientConnection {
    /// Creates a new `ClientConnection`.
    pub fn new(
        stream: TcpStream,
        addr: SocketAddr,
        security_manager: Arc<SecurityManager>,
        session_manager: Arc<SessionManager>,
        region_manager: Arc<RegionManager>,
        monitoring: Arc<MonitoringSystem>,
        state_manager: Arc<StateManager>,
        asset_manager: Arc<crate::asset::AssetManager>,
        user_account_database: Arc<UserAccountDatabase>,
    ) -> (Self, mpsc::UnboundedReceiver<ClientMessage>, mpsc::Receiver<()>) {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        let session = Session::new("".to_string(), addr);

        let conn = Self {
            id: Uuid::new_v4(),
            addr,
            stream: Arc::new(tokio::sync::Mutex::new(stream)),
            session: Arc::new(RwLock::new(session)),
            llsd_handler: LLSDMessageHandler::default(),
            security_manager,
            region_manager,
            session_manager,
            monitoring,
            state_manager,
            asset_manager,
            user_account_database,
            message_tx,
            shutdown_tx,
        };

        (conn, message_rx, shutdown_rx)
    }

    /// Handles the entire lifecycle of a client connection.
    pub async fn handle_connection(
        self,
        message_rx: mpsc::UnboundedReceiver<ClientMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Client connected: {} (session: {})", self.addr, self.id);
        self.monitoring.record_client_connection().await;

        let stream_clone = self.stream.clone();
        let message_tx_clone = self.message_tx.clone();

        let read_task = tokio::spawn(async move {
            if let Ok(mut stream_guard) = stream_clone.try_lock() {
                let peer_addr = stream_guard.peer_addr().map_or_else(|_| "unknown".to_string(), |a| a.to_string());
                let (mut read_stream, _) = split(&mut *stream_guard);
                let mut buffer = [0u8; 8192];
                loop {
                    match read_stream.read(&mut buffer).await {
                        Ok(0) => {
                            info!("Client {} disconnected.", peer_addr);
                            // Signal disconnection to the message processor
                            let _ = message_tx_clone.send(ClientMessage::Shutdown);
                            break;
                        }
                        Ok(n) => {
                            debug!("Read {} bytes from {}", n, peer_addr);
                            // For now, assume binary format until we can determine it
                            let format = LLSDFormat::Binary;
                            match LLSDMessage::from_bytes(&buffer[..n], format) {
                                Ok(msg) => {
                                    if message_tx_clone.send(ClientMessage::Incoming(msg)).is_err() {
                                        warn!("Failed to send incoming message for processing");
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to parse LLSD message: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Error reading from stream: {}", e);
                            break;
                        }
                    }
                }
            } else {
                error!("Could not get lock on TcpStream for reading");
            }
        });
        
        let write_stream_clone = self.stream.clone();
        let process_task = self.process_messages(message_rx, write_stream_clone);

        tokio::select! {
            _ = read_task => {},
            _ = process_task => {},
        }

        self.monitoring.record_client_disconnection().await;
        info!("Connection handler for {} finished.", self.addr);
        Ok(())
    }

    /// Processes incoming and outgoing messages for the client.
    async fn process_messages(
        &self,
        mut message_rx: mpsc::UnboundedReceiver<ClientMessage>,
        write_stream: Arc<tokio::sync::Mutex<TcpStream>>,
    ) {
        if let Ok(mut stream_guard) = write_stream.try_lock() {
            let (_, mut write_half) = split(&mut *stream_guard);

            while let Some(message) = message_rx.recv().await {
                match message {
                    ClientMessage::Incoming(msg) => {
                        let response = self.llsd_handler.handle_message(
                            msg, 
                            self.session.clone(), 
                            self.security_manager.clone(), 
                            self.session_manager.clone(),
                            self.region_manager.clone(),
                            self.state_manager.clone(),
                            self.asset_manager.clone(),
                            self.user_account_database.clone()
                        ).await;
                        if let Ok(Some(response_msg)) = response {
                            if self.message_tx.send(ClientMessage::Outgoing(response_msg)).is_err() {
                                warn!("Failed to queue outgoing message");
                            }
                        }
                    }
                    ClientMessage::Outgoing(msg) => {
                        let format = LLSDFormat::Xml; // Default to XML for responses
                        match msg.to_bytes(format) {
                            Ok(bytes) => {
                                if let Err(e) = write_half.write_all(&bytes).await {
                                    warn!("Failed to write to stream: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to serialize outgoing message: {}", e);
                            }
                        }
                    }
                    ClientMessage::Shutdown => {
                        info!("Shutting down client connection: {}", self.addr);
                        // Clean up session data
                        self.cleanup_session().await;
                        break;
                    }
                }
            }
        } else {
            error!("Could not get lock on TcpStream for writing");
        }
    }

    /// Cleans up session data when a client disconnects
    async fn cleanup_session(&self) {
        let session_guard = self.session.read().await;
        let session_id = session_guard.session_id.clone();
        let agent_id = session_guard.agent_id.clone();
        drop(session_guard);

        info!("Cleaning up session {} for agent {}", session_id, agent_id);

        // Remove session from session manager
        self.session_manager.remove_session(&session_id).await;

        // Remove avatar from region if logged in
        if let Err(e) = self.region_manager.remove_avatar(&agent_id.to_string()).await {
            warn!("Failed to remove avatar {} from region: {}", agent_id, e);
        }

        // Update monitoring metrics
        self.monitoring.record_client_disconnection().await;

        info!("Session cleanup completed for {}", session_id);
    }

    /// Gracefully shuts down the client connection
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initiating graceful shutdown for client {}", self.addr);
        
        // Send shutdown message to the message processor
        self.message_tx.send(ClientMessage::Shutdown)?;
        
        // Send shutdown signal
        self.shutdown_tx.send(()).await?;
        
        Ok(())
    }
}

/// Manages all active client connections.
#[derive(Clone)]
pub struct ClientManager {
    clients: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<ClientMessage>>>>,
    pub session_manager: Arc<SessionManager>,
    pub monitoring: Arc<MonitoringSystem>,
    pub config: Arc<NetworkConfig>,
}

impl ClientManager {
    /// Creates a new `ClientManager`.
    pub fn new(
        monitoring: Arc<MonitoringSystem>,
        session_manager: Arc<SessionManager>,
        config: Arc<NetworkConfig>,
    ) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            session_manager,
            monitoring,
            config,
        }
    }

    /// Adds a new client to the manager.
    pub async fn add_client(&self, client_id: Uuid, message_tx: mpsc::UnboundedSender<ClientMessage>) {
        self.clients.write().await.insert(client_id, message_tx);
    }

    /// Removes a client from the manager.
    pub async fn remove_client(&self, client_id: &Uuid) {
        self.clients.write().await.remove(client_id);
    }

    /// Gets the number of connected clients.
    pub async fn get_client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Broadcast a message to all clients.
    pub async fn broadcast(&self, message: ClientMessage) {
        let clients = self.clients.read().await;
        for client_tx in clients.values() {
            if let Err(e) = client_tx.send(message.clone()) {
                error!("Failed to broadcast message: {}", e);
            }
        }
    }

    /// Retrieves all active client sessions.
    pub async fn get_client_sessions(&self) -> Vec<Session> {
        self.session_manager.get_all_sessions().await
    }
} 