//! Network layer for OpenSim
//!
//! This module handles network communication, client connections, and protocol management.

pub mod admin_api;
pub mod admin_security;
pub mod ai_api;
pub mod auth;
pub mod authentication;
pub mod client;
pub mod console_api;
pub mod crypto_manager;
pub mod dark_services;
pub mod distributed;
pub mod fwdfe_api;
pub mod grid_events;
pub mod handlers;
pub mod hypergrid;
pub mod inter_region;
pub mod llsd;
pub mod loopback;
pub mod protocol;
pub mod rate_limiting;
pub mod region_crossing;
pub mod remote_admin;
pub mod rest_api;
pub mod security;
pub mod session;
pub mod skill_api;
pub mod terminal_commands;
pub mod web_client;
pub mod websocket;
pub mod ziti_manager;
pub mod ziti_policies;

use anyhow::Result;
use bytes::{BufMut, BytesMut};
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::net::TcpListener;
use tokio_util::codec::{Decoder, Encoder};
use tracing::{error, info};

use crate::{
    database::user_accounts::UserAccountDatabase,
    monitoring::MonitoringSystem,
    network::{
        auth::AuthenticationService,
        client::{ClientConnection, ClientManager},
        rest_api::{RestApiConfig, RestApiService},
        security::SecurityManager,
        session::SessionManager,
        websocket::{WebSocketConfig, WebSocketServer},
    },
    region::RegionManager,
    state::StateManager,
};

// Message codec for JSON serialization
pub struct JsonCodec;

impl Decoder for JsonCodec {
    type Item = protocol::NetworkMessage;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        if src.len() < 4 + length {
            return Ok(None);
        }

        let json_bytes = src.split_to(4 + length);
        let json = std::str::from_utf8(&json_bytes[4..])?;

        Ok(Some(serde_json::from_str(json)?))
    }
}

impl Encoder<protocol::NetworkMessage> for JsonCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: protocol::NetworkMessage, dst: &mut BytesMut) -> Result<()> {
        let json = serde_json::to_string(&item)?;
        let json_bytes = json.as_bytes();
        let length = json_bytes.len();

        dst.reserve(4 + length);
        dst.put_u32(length as u32);
        dst.extend_from_slice(json_bytes);

        Ok(())
    }
}

/// Network manager for handling client connections
pub struct NetworkManager {
    client_manager: Arc<ClientManager>,
    session_manager: Arc<SessionManager>,
    security_manager: Arc<SecurityManager>,
    region_manager: Arc<RegionManager>,
    region_crossing_manager: Arc<region_crossing::RegionCrossingManager>,
    inter_region_manager: Arc<inter_region::InterRegionManager>,
    websocket_server: Option<Arc<WebSocketServer>>,
    rest_api_service: Option<Arc<RestApiService>>,
    monitoring: Arc<MonitoringSystem>,
    state_manager: Arc<StateManager>,
    asset_manager: Arc<crate::asset::AssetManager>,
    user_account_database: Arc<UserAccountDatabase>,
}

/// Network configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct NetworkConfig {
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Enable SSL/TLS encryption
    pub enable_ssl: bool,
    /// SSL certificate path
    pub ssl_cert_path: Option<String>,
    /// SSL private key path
    pub ssl_key_path: Option<String>,
    /// Rate limiting configuration
    pub rate_limit_requests_per_minute: u32,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// WebSocket configuration
    pub websocket: WebSocketConfig,
    /// REST API configuration
    pub rest_api: RestApiConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            connection_timeout: 30,
            heartbeat_interval: 5,
            enable_ssl: false,
            ssl_cert_path: None,
            ssl_key_path: None,
            rate_limit_requests_per_minute: 1000,
            max_message_size: 1024 * 1024, // 1MB
            websocket: WebSocketConfig::default(),
            rest_api: RestApiConfig::default(),
        }
    }
}

/// Client connection
struct Client {
    addr: SocketAddr,
    connected_at: Instant,
    last_activity: Instant,
    message_count: u32,
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(
        monitoring: Arc<MonitoringSystem>,
        session_manager: Arc<SessionManager>,
        region_manager: Arc<RegionManager>,
        state_manager: Arc<StateManager>,
        asset_manager: Arc<crate::asset::AssetManager>,
        user_account_database: Arc<UserAccountDatabase>,
    ) -> Result<Self> {
        let config = Arc::new(NetworkConfig::default());
        let region_crossing_manager = Arc::new(region_crossing::RegionCrossingManager::new(
            region_manager.clone(),
            state_manager.clone(),
        ));

        let inter_region_manager = Arc::new(inter_region::InterRegionManager::new(
            region_manager.clone(),
            state_manager.clone(),
        ));

        let security_manager = Arc::new(SecurityManager::new()?);

        // Create WebSocket server if enabled
        let websocket_server = if config.websocket.enabled {
            Some(Arc::new(WebSocketServer::new(
                config.websocket.clone(),
                session_manager.clone(),
                security_manager.clone(),
                region_manager.clone(),
                state_manager.clone(),
                asset_manager.clone(),
                user_account_database.clone(),
                monitoring.clone(),
            )))
        } else {
            None
        };

        // Create REST API service if enabled
        let rest_api_service = if config.rest_api.enabled {
            let auth_service = Arc::new(AuthenticationService::new(
                crate::network::auth::AuthConfig::default(),
            ));

            let database_url =
                std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://opensim.db".to_string());

            let _orig_skip = std::env::var("OPENSIM_SERVICE_MODE").ok();
            std::env::set_var("OPENSIM_SERVICE_MODE", "grid");
            let database_manager = match crate::database::DatabaseManager::new(&database_url).await
            {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    error!("Failed to create database manager for REST API: {}", e);
                    if let Some(orig) = _orig_skip {
                        std::env::set_var("OPENSIM_SERVICE_MODE", orig);
                    } else {
                        std::env::remove_var("OPENSIM_SERVICE_MODE");
                    }
                    return Err(e);
                }
            };
            if let Some(orig) = _orig_skip {
                std::env::set_var("OPENSIM_SERVICE_MODE", orig);
            } else {
                std::env::remove_var("OPENSIM_SERVICE_MODE");
            }

            Some(Arc::new(RestApiService::new(
                config.rest_api.clone(),
                asset_manager.clone(),
                database_manager,
                auth_service,
                region_manager.clone(),
                monitoring.get_metrics_collector(),
            )))
        } else {
            None
        };

        Ok(Self {
            client_manager: Arc::new(ClientManager::new(
                monitoring.clone(),
                session_manager.clone(),
                config,
            )),
            session_manager,
            security_manager,
            region_manager,
            region_crossing_manager,
            inter_region_manager,
            websocket_server,
            rest_api_service,
            monitoring,
            state_manager,
            asset_manager,
            user_account_database,
        })
    }

    /// Start the network server
    pub async fn start(&self, address: &str) -> Result<()> {
        // Start inter-region communication system
        self.inter_region_manager.start().await?;

        // Start WebSocket server if configured
        if let Some(websocket_server) = &self.websocket_server {
            let ws_server = websocket_server.clone();
            tokio::spawn(async move {
                if let Err(e) = ws_server.start().await {
                    error!("WebSocket server error: {}", e);
                }
            });
            info!(
                "WebSocket server started on port {}",
                websocket_server.get_port()
            );
        }

        // Start REST API service if configured
        if let Some(rest_api_service) = &self.rest_api_service {
            let api_service = rest_api_service.clone();
            tokio::spawn(async move {
                if let Err(e) = api_service.start_server().await {
                    error!("REST API server error: {}", e);
                }
            });
            info!(
                "REST API server started on port {}",
                rest_api_service.get_port()
            );
        }

        let listener = TcpListener::bind(address).await?;
        info!("NetworkManager listening on {}", address);

        loop {
            let (stream, addr) = listener.accept().await?;
            info!("Accepted connection from {}", addr);

            let client_manager = self.client_manager.clone();
            let session_manager = self.session_manager.clone();
            let security_manager = self.security_manager.clone();
            let region_manager = self.region_manager.clone();
            let monitoring = self.monitoring.clone();
            let state_manager = self.state_manager.clone();
            let asset_manager = self.asset_manager.clone();
            let user_account_database = self.user_account_database.clone();

            tokio::spawn(async move {
                let (client_connection, message_rx, _shutdown_rx) = ClientConnection::new(
                    stream,
                    addr,
                    security_manager,
                    session_manager,
                    region_manager,
                    monitoring,
                    state_manager,
                    asset_manager,
                    user_account_database,
                );

                let client_id = client_connection.id;
                client_manager
                    .add_client(client_id, client_connection.message_tx.clone())
                    .await;

                if let Err(e) = client_connection.handle_connection(message_rx).await {
                    error!("Error handling client connection: {}", e);
                }

                client_manager.remove_client(&client_id).await;
            });
        }
    }

    /// Gracefully shutdown the network manager by disconnecting all clients
    pub async fn shutdown(&self) -> Result<()> {
        self.client_manager
            .broadcast(crate::network::client::ClientMessage::Shutdown)
            .await;
        Ok(())
    }

    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkStats {
        let active_connections = self.client_manager.get_client_count().await;
        let total_messages = 0;

        NetworkStats {
            active_connections,
            max_connections: self.client_manager.config.max_connections,
            total_messages,
            uptime: self.monitoring.get_uptime(),
        }
    }

    /// Get WebSocket server reference
    pub fn websocket_server(&self) -> Option<&Arc<WebSocketServer>> {
        self.websocket_server.as_ref()
    }

    /// Get WebSocket statistics
    pub async fn get_websocket_stats(&self) -> Option<websocket::WebSocketStats> {
        if let Some(ws_server) = &self.websocket_server {
            Some(ws_server.get_stats().await)
        } else {
            None
        }
    }

    /// Broadcast a WebSocket message to all connected clients
    pub async fn broadcast_websocket_message(
        &self,
        message: websocket::WebSocketMessage,
    ) -> Result<()> {
        if let Some(ws_server) = &self.websocket_server {
            ws_server.broadcast_message(message).await?;
        }
        Ok(())
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    /// Number of active connections
    pub active_connections: usize,
    /// Maximum number of connections
    pub max_connections: usize,
    /// Total number of messages
    pub total_messages: u32,
    /// Uptime of the network manager
    pub uptime: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::physics::PhysicsBridge;
    use crate::state::StateManager;

    #[tokio::test]
    #[ignore]
    async fn test_network_manager_creation() {
        let monitoring = Arc::new(
            MonitoringSystem::new(crate::monitoring::MonitoringConfig::default()).unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(Duration::from_secs(600)));
        let physics_bridge = Arc::new(PhysicsBridge::new().unwrap());
        let state_manager = Arc::new(StateManager::new().unwrap());
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        // Create test dependencies for AssetManager
        use crate::asset::{
            cache::{AssetCache, CacheConfig},
            cdn::{CdnConfig, CdnManager},
            storage::StorageBackend,
            AssetManagerConfig,
        };
        use crate::database::DatabaseManager;

        let database = Arc::new(
            DatabaseManager::new("postgresql://opensim:password123@localhost/opensim_pg")
                .await
                .unwrap(),
        );
        let cache_config = CacheConfig {
            memory_cache_size: 100,
            memory_ttl_seconds: 3600,
            redis_ttl_seconds: 7200,
            redis_url: None,
            enable_compression: false,
            compression_threshold: 1024,
        };
        let cache = Arc::new(AssetCache::new(cache_config).await.unwrap());
        use crate::asset::cdn::CdnProvider;
        use std::collections::HashMap;
        let cdn_config = CdnConfig {
            provider: CdnProvider::Generic,
            base_url: "http://localhost:8080".to_string(),
            api_key: None,
            provider_config: HashMap::new(),
            default_ttl: 3600,
            auto_distribute: false,
            regions: vec![],
        };
        let cdn = Arc::new(CdnManager::new(cdn_config).await.unwrap());
        let storage: Arc<dyn StorageBackend> = Arc::new(
            crate::asset::storage::FileSystemStorage::new(std::path::PathBuf::from(
                "./test_assets",
            ))
            .unwrap(),
        );
        let config = AssetManagerConfig::default();

        let asset_manager = Arc::new(
            crate::asset::AssetManager::new(database.clone(), cache, cdn, storage, config)
                .await
                .unwrap(),
        );

        let user_account_database = database.user_accounts();

        let manager = NetworkManager::new(
            monitoring,
            session_manager,
            region_manager,
            state_manager,
            asset_manager,
            user_account_database,
        )
        .await
        .unwrap();
        let stats = manager.get_stats().await;
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.max_connections, 1000);
    }
}
