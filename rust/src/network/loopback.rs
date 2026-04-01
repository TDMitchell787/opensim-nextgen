//! Loopback connectors for local server communication
//! 
//! This module provides loopback connectors that enable proper local
//! communication between OpenSim Next components and external clients.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{RwLock, mpsc};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};

/// Loopback connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopbackConfig {
    /// Enable loopback connectors
    pub enabled: bool,
    /// Local interface to bind to
    pub interface: IpAddr,
    /// Port range for dynamic allocation
    pub port_range: (u16, u16),
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for LoopbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interface: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port_range: (10000, 20000),
            max_connections: 1000,
            timeout_seconds: 30,
        }
    }
}

/// Loopback connector service types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServiceType {
    Web,
    WebSocket,
    API,
    SLViewer,
    Hypergrid,
    Database,
    Monitor,
}

/// Connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub service_type: ServiceType,
    pub local_addr: SocketAddr,
    pub remote_addr: Option<SocketAddr>,
    pub protocol: Protocol,
    pub status: ConnectionStatus,
}

#[derive(Debug, Clone)]
pub enum Protocol {
    TCP,
    UDP,
    WebSocket,
    HTTP,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Listening,
    Connected,
    Disconnected,
    Error(String),
}

/// Loopback connector manager
pub struct LoopbackConnector {
    config: LoopbackConfig,
    connections: Arc<RwLock<HashMap<ServiceType, Vec<ConnectionInfo>>>>,
    listeners: Arc<RwLock<HashMap<ServiceType, TcpListener>>>,
    event_sender: mpsc::UnboundedSender<LoopbackEvent>,
    event_receiver: RwLock<Option<mpsc::UnboundedReceiver<LoopbackEvent>>>,
}

#[derive(Debug, Clone)]
pub enum LoopbackEvent {
    ServiceStarted(ServiceType, SocketAddr),
    ServiceStopped(ServiceType),
    ConnectionEstablished(ServiceType, SocketAddr),
    ConnectionLost(ServiceType, SocketAddr),
    Error(ServiceType, String),
}

impl LoopbackConnector {
    /// Create a new loopback connector
    pub fn new(config: LoopbackConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            listeners: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: RwLock::new(Some(event_receiver)),
        }
    }

    /// Start loopback connector service
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Loopback connectors disabled in configuration");
            return Ok(());
        }

        info!("Starting loopback connectors on {}", self.config.interface);

        // Start core services
        self.start_web_service().await?;
        self.start_websocket_service().await?;
        self.start_api_service().await?;
        self.start_sl_viewer_service().await?;
        self.start_hypergrid_service().await?;

        // Start event processing
        self.start_event_processor().await;

        info!("All loopback connectors started successfully");
        Ok(())
    }

    /// Start web service loopback
    async fn start_web_service(&self) -> Result<()> {
        let addr = SocketAddr::new(self.config.interface, 8080);
        
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                info!("Web service listening on {}", addr);
                
                let mut listeners = self.listeners.write().await;
                listeners.insert(ServiceType::Web, listener);
                
                let connection = ConnectionInfo {
                    service_type: ServiceType::Web,
                    local_addr: addr,
                    remote_addr: None,
                    protocol: Protocol::HTTP,
                    status: ConnectionStatus::Listening,
                };
                
                let mut connections = self.connections.write().await;
                connections.entry(ServiceType::Web).or_insert_with(Vec::new).push(connection);
                
                let _ = self.event_sender.send(LoopbackEvent::ServiceStarted(ServiceType::Web, addr));
                Ok(())
            }
            Err(e) => {
                error!("Failed to bind web service to {}: {}", addr, e);
                let _ = self.event_sender.send(LoopbackEvent::Error(ServiceType::Web, e.to_string()));
                Err(e.into())
            }
        }
    }

    /// Start WebSocket service loopback
    async fn start_websocket_service(&self) -> Result<()> {
        let addr = SocketAddr::new(self.config.interface, 9001);
        
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                info!("WebSocket service listening on {}", addr);
                
                let mut listeners = self.listeners.write().await;
                listeners.insert(ServiceType::WebSocket, listener);
                
                let connection = ConnectionInfo {
                    service_type: ServiceType::WebSocket,
                    local_addr: addr,
                    remote_addr: None,
                    protocol: Protocol::WebSocket,
                    status: ConnectionStatus::Listening,
                };
                
                let mut connections = self.connections.write().await;
                connections.entry(ServiceType::WebSocket).or_insert_with(Vec::new).push(connection);
                
                let _ = self.event_sender.send(LoopbackEvent::ServiceStarted(ServiceType::WebSocket, addr));
                Ok(())
            }
            Err(e) => {
                error!("Failed to bind WebSocket service to {}: {}", addr, e);
                let _ = self.event_sender.send(LoopbackEvent::Error(ServiceType::WebSocket, e.to_string()));
                Err(e.into())
            }
        }
    }

    /// Start API service loopback
    async fn start_api_service(&self) -> Result<()> {
        let addr = SocketAddr::new(self.config.interface, 9100);
        
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                info!("API service listening on {}", addr);
                
                let mut listeners = self.listeners.write().await;
                listeners.insert(ServiceType::API, listener);
                
                let connection = ConnectionInfo {
                    service_type: ServiceType::API,
                    local_addr: addr,
                    remote_addr: None,
                    protocol: Protocol::HTTP,
                    status: ConnectionStatus::Listening,
                };
                
                let mut connections = self.connections.write().await;
                connections.entry(ServiceType::API).or_insert_with(Vec::new).push(connection);
                
                let _ = self.event_sender.send(LoopbackEvent::ServiceStarted(ServiceType::API, addr));
                Ok(())
            }
            Err(e) => {
                error!("Failed to bind API service to {}: {}", addr, e);
                let _ = self.event_sender.send(LoopbackEvent::Error(ServiceType::API, e.to_string()));
                Err(e.into())
            }
        }
    }

    /// Start Second Life viewer service loopback
    async fn start_sl_viewer_service(&self) -> Result<()> {
        let addr = SocketAddr::new(self.config.interface, 9000);
        
        match UdpSocket::bind(addr).await {
            Ok(_socket) => {
                info!("SL Viewer service listening on {} (UDP)", addr);
                
                let connection = ConnectionInfo {
                    service_type: ServiceType::SLViewer,
                    local_addr: addr,
                    remote_addr: None,
                    protocol: Protocol::UDP,
                    status: ConnectionStatus::Listening,
                };
                
                let mut connections = self.connections.write().await;
                connections.entry(ServiceType::SLViewer).or_insert_with(Vec::new).push(connection);
                
                let _ = self.event_sender.send(LoopbackEvent::ServiceStarted(ServiceType::SLViewer, addr));
                Ok(())
            }
            Err(e) => {
                error!("Failed to bind SL Viewer service to {}: {}", addr, e);
                let _ = self.event_sender.send(LoopbackEvent::Error(ServiceType::SLViewer, e.to_string()));
                Err(e.into())
            }
        }
    }

    /// Start Hypergrid service loopback
    async fn start_hypergrid_service(&self) -> Result<()> {
        let addr = SocketAddr::new(self.config.interface, 8002);
        
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                info!("Hypergrid service listening on {}", addr);
                
                let mut listeners = self.listeners.write().await;
                listeners.insert(ServiceType::Hypergrid, listener);
                
                let connection = ConnectionInfo {
                    service_type: ServiceType::Hypergrid,
                    local_addr: addr,
                    remote_addr: None,
                    protocol: Protocol::HTTP,
                    status: ConnectionStatus::Listening,
                };
                
                let mut connections = self.connections.write().await;
                connections.entry(ServiceType::Hypergrid).or_insert_with(Vec::new).push(connection);
                
                let _ = self.event_sender.send(LoopbackEvent::ServiceStarted(ServiceType::Hypergrid, addr));
                Ok(())
            }
            Err(e) => {
                error!("Failed to bind Hypergrid service to {}: {}", addr, e);
                let _ = self.event_sender.send(LoopbackEvent::Error(ServiceType::Hypergrid, e.to_string()));
                Err(e.into())
            }
        }
    }

    /// Start event processor
    async fn start_event_processor(&self) {
        let mut receiver = self.event_receiver.write().await.take();
        if let Some(mut rx) = receiver {
            let connections = self.connections.clone();
            
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match &event {
                        LoopbackEvent::ServiceStarted(service_type, addr) => {
                            debug!("Service {:?} started on {}", service_type, addr);
                        }
                        LoopbackEvent::ServiceStopped(service_type) => {
                            debug!("Service {:?} stopped", service_type);
                            let mut conns = connections.write().await;
                            if let Some(service_connections) = conns.get_mut(service_type) {
                                for conn in service_connections.iter_mut() {
                                    conn.status = ConnectionStatus::Disconnected;
                                }
                            }
                        }
                        LoopbackEvent::ConnectionEstablished(service_type, addr) => {
                            debug!("Connection established for {:?} from {}", service_type, addr);
                        }
                        LoopbackEvent::ConnectionLost(service_type, addr) => {
                            debug!("Connection lost for {:?} from {}", service_type, addr);
                        }
                        LoopbackEvent::Error(service_type, error) => {
                            error!("Error in {:?}: {}", service_type, error);
                        }
                    }
                }
            });
        }
    }

    /// Get all active connections
    pub async fn get_connections(&self) -> HashMap<ServiceType, Vec<ConnectionInfo>> {
        self.connections.read().await.clone()
    }

    /// Get connection status for a specific service
    pub async fn get_service_status(&self, service_type: &ServiceType) -> Option<Vec<ConnectionInfo>> {
        let connections = self.connections.read().await;
        connections.get(service_type).cloned()
    }

    /// Check if all services are healthy
    pub async fn health_check(&self) -> bool {
        let connections = self.connections.read().await;
        
        let required_services = vec![
            ServiceType::Web,
            ServiceType::WebSocket,
            ServiceType::API,
            ServiceType::SLViewer,
        ];

        for service in required_services {
            if let Some(service_connections) = connections.get(&service) {
                let healthy = service_connections.iter().any(|conn| {
                    matches!(conn.status, ConnectionStatus::Listening | ConnectionStatus::Connected)
                });
                if !healthy {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Stop all loopback connectors
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping all loopback connectors");

        let mut listeners = self.listeners.write().await;
        listeners.clear();

        let mut connections = self.connections.write().await;
        for (service_type, service_connections) in connections.iter_mut() {
            for conn in service_connections.iter_mut() {
                conn.status = ConnectionStatus::Disconnected;
            }
            let _ = self.event_sender.send(LoopbackEvent::ServiceStopped(service_type.clone()));
        }

        info!("All loopback connectors stopped");
        Ok(())
    }

    /// Get loopback connector statistics
    pub async fn get_stats(&self) -> LoopbackStats {
        let connections = self.connections.read().await;
        
        let mut total_connections = 0;
        let mut listening_services = 0;
        let mut active_connections = 0;
        let mut error_count = 0;

        for (_, service_connections) in connections.iter() {
            total_connections += service_connections.len();
            
            for conn in service_connections.iter() {
                match &conn.status {
                    ConnectionStatus::Listening => listening_services += 1,
                    ConnectionStatus::Connected => active_connections += 1,
                    ConnectionStatus::Error(_) => error_count += 1,
                    _ => {}
                }
            }
        }

        LoopbackStats {
            total_connections,
            listening_services,
            active_connections,
            error_count,
            health_status: if error_count == 0 && listening_services > 0 {
                "Healthy".to_string()
            } else if error_count > 0 {
                "Degraded".to_string()
            } else {
                "Offline".to_string()
            },
        }
    }
}

/// Loopback connector statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopbackStats {
    pub total_connections: usize,
    pub listening_services: usize,
    pub active_connections: usize,
    pub error_count: usize,
    pub health_status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_loopback_connector() {
        let config = LoopbackConfig::default();
        let connector = LoopbackConnector::new(config);
        
        // Test starting connectors
        assert!(connector.start().await.is_ok());
        
        // Test health check
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert!(connector.health_check().await);
        
        // Test getting connections
        let connections = connector.get_connections().await;
        assert!(!connections.is_empty());
        
        // Test stopping connectors
        assert!(connector.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_service_binding() {
        let config = LoopbackConfig::default();
        let connector = LoopbackConnector::new(config);
        
        // Start services
        assert!(connector.start_web_service().await.is_ok());
        assert!(connector.start_api_service().await.is_ok());
        
        // Check connections
        let web_status = connector.get_service_status(&ServiceType::Web).await;
        assert!(web_status.is_some());
        
        let api_status = connector.get_service_status(&ServiceType::API).await;
        assert!(api_status.is_some());
    }
}