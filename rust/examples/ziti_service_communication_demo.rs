//! OpenZiti Secure Service-to-Service Communication Demo
//!
//! Demonstrates secure, encrypted communication between virtual world services
//! using OpenZiti zero trust networking.

use anyhow::Result;
use opensim_next::physics::PhysicsResult;
use opensim_next::ziti::{
    config::ZitiConfig,
    services::{
        MessagePriority, ServiceCommunicationHandler, ServiceCommunicationStats,
        ServiceConnectionEvent, ServiceConnectionType, ServiceDiscoveryInfo,
        ServiceHandlerCapabilities, ServiceMessage, ServiceMessageType, ZitiServiceManager,
    },
    ZitiNetworkManager,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Example service handler for region server
pub struct RegionServiceHandler {
    service_name: String,
}

impl RegionServiceHandler {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

#[async_trait::async_trait]
impl ServiceCommunicationHandler for RegionServiceHandler {
    async fn handle_message(
        &self,
        message: ServiceMessage,
    ) -> PhysicsResult<Option<ServiceMessage>> {
        println!(
            "🏗️  {} received {} message from {}: {} bytes",
            self.service_name,
            format!("{:?}", message.message_type),
            message.source_service,
            message.payload.len()
        );

        match message.message_type {
            ServiceMessageType::Request => {
                // Simulate processing request
                let response_payload = format!(
                    "Response from {} to request {}",
                    self.service_name, message.message_id
                );

                let response = ServiceMessage {
                    message_id: uuid::Uuid::new_v4().to_string(),
                    source_service: self.service_name.clone(),
                    target_service: message.source_service,
                    message_type: ServiceMessageType::Response,
                    payload: response_payload.into_bytes(),
                    headers: HashMap::new(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    priority: message.priority,
                    encryption_required: true,
                    authentication_token: None,
                    correlation_id: Some(message.message_id),
                    expires_at: None,
                };

                Ok(Some(response))
            }
            ServiceMessageType::Heartbeat => {
                println!(
                    "💓 {} received heartbeat from {}",
                    self.service_name, message.source_service
                );
                Ok(None)
            }
            ServiceMessageType::Event => {
                let event_data = String::from_utf8_lossy(&message.payload);
                println!("📢 {} received event: {}", self.service_name, event_data);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    async fn handle_connection_event(&self, event: ServiceConnectionEvent) -> PhysicsResult<()> {
        match event {
            ServiceConnectionEvent::Connected {
                connection_id,
                service_id,
            } => {
                println!(
                    "🔗 {} connected to service {} (connection: {})",
                    self.service_name, service_id, connection_id
                );
            }
            ServiceConnectionEvent::Disconnected {
                connection_id,
                reason,
            } => {
                println!(
                    "🔌 {} disconnected from connection {} (reason: {})",
                    self.service_name, connection_id, reason
                );
            }
            ServiceConnectionEvent::Error {
                connection_id,
                error,
            } => {
                println!(
                    "❌ {} connection error on {}: {}",
                    self.service_name, connection_id, error
                );
            }
            ServiceConnectionEvent::DataReceived {
                connection_id,
                data_size,
            } => {
                println!(
                    "📦 {} received {} bytes on connection {}",
                    self.service_name, data_size, connection_id
                );
            }
            ServiceConnectionEvent::AuthenticationRequired {
                connection_id,
                challenge,
            } => {
                println!(
                    "🔐 {} requires authentication for connection {} (challenge: {})",
                    self.service_name, connection_id, challenge
                );
            }
        }
        Ok(())
    }

    fn get_capabilities(&self) -> ServiceHandlerCapabilities {
        ServiceHandlerCapabilities {
            supported_message_types: vec![
                ServiceMessageType::Request,
                ServiceMessageType::Response,
                ServiceMessageType::Event,
                ServiceMessageType::Heartbeat,
            ],
            max_message_size: 1024 * 1024, // 1MB
            supports_encryption: true,
            supports_authentication: true,
            supports_compression: true,
            timeout_seconds: 30,
        }
    }
}

/// Example service handler for asset server
pub struct AssetServiceHandler {
    service_name: String,
}

impl AssetServiceHandler {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

#[async_trait::async_trait]
impl ServiceCommunicationHandler for AssetServiceHandler {
    async fn handle_message(
        &self,
        message: ServiceMessage,
    ) -> PhysicsResult<Option<ServiceMessage>> {
        println!(
            "💾 {} received {} message from {}: {} bytes",
            self.service_name,
            format!("{:?}", message.message_type),
            message.source_service,
            message.payload.len()
        );

        match message.message_type {
            ServiceMessageType::Request => {
                let request_data = String::from_utf8_lossy(&message.payload);
                if request_data.contains("asset_id") {
                    // Simulate asset retrieval
                    let asset_data = b"Mock asset data - texture, sound, or mesh content";

                    let response = ServiceMessage {
                        message_id: uuid::Uuid::new_v4().to_string(),
                        source_service: self.service_name.clone(),
                        target_service: message.source_service,
                        message_type: ServiceMessageType::DataTransfer,
                        payload: asset_data.to_vec(),
                        headers: {
                            let mut headers = HashMap::new();
                            headers.insert(
                                "content-type".to_string(),
                                "application/octet-stream".to_string(),
                            );
                            headers.insert("asset-type".to_string(), "texture".to_string());
                            headers
                        },
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        priority: MessagePriority::Normal,
                        encryption_required: true,
                        authentication_token: None,
                        correlation_id: Some(message.message_id),
                        expires_at: None,
                    };

                    println!(
                        "📤 {} sending asset data: {} bytes",
                        self.service_name,
                        asset_data.len()
                    );
                    Ok(Some(response))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    async fn handle_connection_event(&self, event: ServiceConnectionEvent) -> PhysicsResult<()> {
        match event {
            ServiceConnectionEvent::Connected {
                connection_id,
                service_id,
            } => {
                println!(
                    "🔗 {} connected to service {} (connection: {})",
                    self.service_name, service_id, connection_id
                );
            }
            _ => {}
        }
        Ok(())
    }

    fn get_capabilities(&self) -> ServiceHandlerCapabilities {
        ServiceHandlerCapabilities {
            supported_message_types: vec![
                ServiceMessageType::Request,
                ServiceMessageType::DataTransfer,
            ],
            max_message_size: 10 * 1024 * 1024, // 10MB for large assets
            supports_encryption: true,
            supports_authentication: true,
            supports_compression: true,
            timeout_seconds: 60,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🌐 OpenZiti Secure Service-to-Service Communication Demo");
    println!("=======================================================");

    // Demo 1: Initialize Service Manager
    demo_service_manager_initialization().await?;

    // Demo 2: Service Registration and Discovery
    demo_service_registration_discovery().await?;

    // Demo 3: Secure Service-to-Service Communication
    demo_secure_service_communication().await?;

    // Demo 4: Message Routing and Handling
    demo_message_routing().await?;

    // Demo 5: Service Health Monitoring
    demo_service_health_monitoring().await?;

    // Demo 6: Service Statistics and Analytics
    demo_service_statistics().await?;

    println!("\n✅ All OpenZiti secure service communication demos completed successfully!");
    Ok(())
}

/// Demo 1: Initialize Service Manager
async fn demo_service_manager_initialization() -> Result<()> {
    println!("\n🔧 Demo 1: Service Manager Initialization");
    println!("-----------------------------------------");

    // Create OpenZiti configuration
    let config = ZitiConfig::default().with_opensim_services();

    // Create and initialize service manager
    let mut service_manager = ZitiServiceManager::new(&config)?;

    println!("🚀 Initializing service manager with secure communication...");
    service_manager.initialize().await?;

    // Get initial services
    let services = service_manager.get_services().await;
    println!(
        "📋 Service manager initialized with {} default services",
        services.len()
    );

    for service in &services {
        println!(
            "   - {}: {} ({}:{})",
            service.name,
            format!("{:?}", service.service_type),
            service.endpoint_address,
            service.port
        );
    }

    println!("🔧 Service manager initialization completed");
    Ok(())
}

/// Demo 2: Service Registration and Discovery
async fn demo_service_registration_discovery() -> Result<()> {
    println!("\n🔍 Demo 2: Service Registration and Discovery");
    println!("--------------------------------------------");

    let config = ZitiConfig::default();
    let service_manager = ZitiServiceManager::new(&config)?;

    // Register service handlers
    let region_handler = Arc::new(RegionServiceHandler::new("region-server".to_string()));
    let asset_handler = Arc::new(AssetServiceHandler::new("asset-server".to_string()));

    service_manager
        .register_handler("region-server", region_handler)
        .await?;
    service_manager
        .register_handler("asset-server", asset_handler)
        .await?;

    println!("📝 Registered communication handlers for services");

    // Discover services
    println!("🔍 Discovering available services...");
    let discovered_services = service_manager.discover_services(None).await?;

    println!("📋 Discovered {} services:", discovered_services.len());
    for service in &discovered_services {
        println!(
            "   - {} ({}): {} - Health: {:?}, Load: {:.2}",
            service.service_name,
            format!("{:?}", service.service_type),
            service.endpoint_address,
            service.health_status,
            service.load_factor
        );
    }

    // Discover specific service types
    println!("🔍 Discovering region services...");
    let region_services = service_manager
        .discover_services(Some(opensim_next::ziti::ZitiServiceType::Region))
        .await?;

    println!("🏗️  Found {} region services", region_services.len());

    println!("🔍 Service discovery completed");
    Ok(())
}

/// Demo 3: Secure Service-to-Service Communication
async fn demo_secure_service_communication() -> Result<()> {
    println!("\n🔐 Demo 3: Secure Service-to-Service Communication");
    println!("------------------------------------------------");

    let config = ZitiConfig::default();
    let service_manager = ZitiServiceManager::new(&config)?;

    // Establish secure connections between services
    println!("🔗 Establishing secure connections...");

    let connection_types = vec![
        (
            "region-server",
            "asset-server",
            ServiceConnectionType::Direct,
        ),
        (
            "region-server",
            "auth-server",
            ServiceConnectionType::Proxied,
        ),
        (
            "asset-server",
            "inventory-server",
            ServiceConnectionType::P2P,
        ),
    ];

    let mut connection_ids = Vec::new();

    for (source, target, conn_type) in connection_types {
        match service_manager
            .connect_services(source, target, conn_type)
            .await
        {
            Ok(connection_id) => {
                println!(
                    "   ✅ Connected {} to {} ({})",
                    source, target, connection_id
                );
                connection_ids.push(connection_id);
            }
            Err(e) => {
                println!("   ❌ Failed to connect {} to {}: {}", source, target, e);
            }
        }
    }

    println!("🔐 Established {} secure connections", connection_ids.len());

    // Get active connections
    let active_connections = service_manager.get_active_connections().await;
    println!("📊 Active connections: {}", active_connections.len());

    for connection in &active_connections {
        println!(
            "   - {} → {} ({:?}): {} bytes transferred",
            connection.source_service,
            connection.target_service,
            connection.connection_type,
            connection.bytes_sent + connection.bytes_received
        );
    }

    println!("🔐 Secure service communication setup completed");
    Ok(())
}

/// Demo 4: Message Routing and Handling
async fn demo_message_routing() -> Result<()> {
    println!("\n📨 Demo 4: Message Routing and Handling");
    println!("--------------------------------------");

    let config = ZitiConfig::default();
    let service_manager = ZitiServiceManager::new(&config)?;

    // Register handlers
    let region_handler = Arc::new(RegionServiceHandler::new("region-server".to_string()));
    let asset_handler = Arc::new(AssetServiceHandler::new("asset-server".to_string()));

    service_manager
        .register_handler("region-server", region_handler)
        .await?;
    service_manager
        .register_handler("asset-server", asset_handler)
        .await?;

    println!("📤 Sending various types of messages...");

    // Send different types of messages
    let messages = vec![
        (
            "region-server",
            "asset-server",
            ServiceMessageType::Request,
            "asset_id=12345",
            MessagePriority::High,
        ),
        (
            "asset-server",
            "region-server",
            ServiceMessageType::Event,
            "Asset cache updated",
            MessagePriority::Normal,
        ),
        (
            "region-server",
            "auth-server",
            ServiceMessageType::Request,
            "authenticate user",
            MessagePriority::Critical,
        ),
        (
            "messaging-server",
            "region-server",
            ServiceMessageType::Heartbeat,
            "ping",
            MessagePriority::Low,
        ),
    ];

    for (source, target, msg_type, content, priority) in messages {
        let message_id = service_manager
            .send_service_message(
                source,
                target,
                msg_type,
                content.as_bytes().to_vec(),
                priority,
            )
            .await?;

        println!(
            "   📬 Sent {} message {} from {} to {}: \"{}\"",
            format!("{:?}", msg_type),
            message_id,
            source,
            target,
            content
        );

        // Small delay to show message processing
        sleep(Duration::from_millis(500)).await;
    }

    // Allow time for message processing
    println!("⏳ Allowing time for message processing...");
    sleep(Duration::from_secs(2)).await;

    println!("📨 Message routing and handling completed");
    Ok(())
}

/// Demo 5: Service Health Monitoring
async fn demo_service_health_monitoring() -> Result<()> {
    println!("\n💊 Demo 5: Service Health Monitoring");
    println!("-----------------------------------");

    let config = ZitiConfig::default();
    let service_manager = ZitiServiceManager::new(&config)?;

    println!("📊 Checking service health status...");

    let hosted_services = service_manager.get_hosted_services().await;

    println!("🏥 Service Health Report:");
    for service in &hosted_services {
        let status_emoji = match service.health_status {
            opensim_next::ziti::services::ServiceHealthStatus::Healthy => "✅",
            opensim_next::ziti::services::ServiceHealthStatus::Degraded => "⚠️",
            opensim_next::ziti::services::ServiceHealthStatus::Unhealthy => "❌",
            opensim_next::ziti::services::ServiceHealthStatus::Unknown => "❓",
        };

        println!(
            "   {} {} ({:?}): {:?} - {} clients, {} bytes transferred",
            status_emoji,
            service.service_id,
            service.service_type,
            service.health_status,
            service.client_count,
            service.bytes_transferred
        );

        println!(
            "      Last heartbeat: {} seconds ago",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                - service.last_heartbeat
        );
    }

    // Simulate health check
    println!("🔄 Running health checks...");
    sleep(Duration::from_secs(1)).await;

    println!("💊 Service health monitoring completed");
    Ok(())
}

/// Demo 6: Service Statistics and Analytics
async fn demo_service_statistics() -> Result<()> {
    println!("\n📈 Demo 6: Service Statistics and Analytics");
    println!("------------------------------------------");

    let config = ZitiConfig::default();
    let service_manager = ZitiServiceManager::new(&config)?;

    println!("📊 Generating service communication statistics...");

    let services = service_manager.get_services().await;

    for service in &services {
        let stats = service_manager
            .get_communication_stats(&service.service_id)
            .await?;

        println!("📋 {} Statistics:", service.name);
        println!("   Total Connections: {}", stats.total_connections);
        println!("   Active Connections: {}", stats.active_connections);
        println!("   Messages Sent: {}", stats.messages_sent);
        println!("   Messages Received: {}", stats.messages_received);
        println!("   Bytes Transferred: {} bytes", stats.bytes_transferred);
        println!("   Failed Connections: {}", stats.failed_connections);
        println!("   Average Latency: {:.2} ms", stats.average_latency_ms);
        println!("   Error Rate: {:.2}%", stats.error_rate * 100.0);
        println!("   Uptime: {} seconds", stats.uptime_seconds);
        println!();
    }

    // Overall network statistics
    println!("🌐 Overall Network Statistics:");
    let total_services = services.len();
    let active_connections = service_manager.get_active_connections().await;
    let total_connections = active_connections.len();

    println!("   Total Services: {}", total_services);
    println!("   Total Active Connections: {}", total_connections);
    println!("   Network Security: Zero Trust (OpenZiti)");
    println!("   Encryption: Always Enabled");
    println!("   Authentication: Policy-Based");

    println!("📈 Service statistics and analytics completed");
    Ok(())
}

/// Helper function to demonstrate zero trust security benefits
fn demonstrate_zero_trust_security() {
    println!("\n🛡️  Zero Trust Security Benefits for Virtual World Services:");
    println!("=========================================================");
    println!("🔒 Service-to-Service Security:");
    println!("   • Every service connection is authenticated");
    println!("   • All communication is encrypted by default");
    println!("   • Policy-based access control");
    println!("   • No implicit trust between services");

    println!("\n🌐 Dark Services:");
    println!("   • Services are invisible until authorized");
    println!("   • No network scanning or service discovery attacks");
    println!("   • Micro-segmentation at the service level");
    println!("   • Each service has its own security perimeter");

    println!("\n🔍 Service Discovery:");
    println!("   • Authenticated service discovery only");
    println!("   • Services announce capabilities securely");
    println!("   • Dynamic service registration and health monitoring");
    println!("   • Load balancing based on real-time metrics");

    println!("\n📊 Monitoring and Analytics:");
    println!("   • Real-time connection monitoring");
    println!("   • Service health and performance tracking");
    println!("   • Security event logging and alerting");
    println!("   • Comprehensive communication statistics");

    println!("\n🚀 Virtual World Specific Benefits:");
    println!("   • Secure region-to-region communication");
    println!("   • Protected asset transfer and caching");
    println!("   • Encrypted user authentication flows");
    println!("   • Secure inventory and messaging services");
    println!("   • Zero trust network for all virtual world components");
}
