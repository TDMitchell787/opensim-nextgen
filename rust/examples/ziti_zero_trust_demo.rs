//! OpenZiti Zero Trust Networking Demo for OpenSim Next
//!
//! Demonstrates how to use OpenZiti for secure, encrypted, zero-trust networking
//! in a virtual world environment.

use anyhow::Result;
use opensim_next::ziti::{
    config::ZitiConfig,
    integration::{OpenSimComponentType, OpenSimZitiConfig, OpenSimZitiIntegration},
    ZitiNetworkManager,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔒 OpenZiti Zero Trust Networking Demo for OpenSim Next");
    println!("======================================================");

    // Demo 1: Basic OpenZiti Network Manager
    demo_basic_ziti_manager().await?;

    // Demo 2: OpenSim Integration with Zero Trust
    demo_opensim_integration().await?;

    // Demo 3: Secure Service Communication
    demo_secure_service_communication().await?;

    // Demo 4: Zero Trust Policies
    demo_zero_trust_policies().await?;

    // Demo 5: Performance Monitoring
    demo_performance_monitoring().await?;

    println!("\n✅ All OpenZiti zero trust networking demos completed successfully!");
    Ok(())
}

/// Demo 1: Basic OpenZiti Network Manager functionality
async fn demo_basic_ziti_manager() -> Result<()> {
    println!("\n📡 Demo 1: Basic OpenZiti Network Manager");
    println!("-----------------------------------------");

    // Create configuration
    let config = ZitiConfig::default().with_opensim_services();

    // Create and initialize network manager
    let mut network_manager = ZitiNetworkManager::new(config)?;

    println!("🔧 Initializing OpenZiti network manager...");
    network_manager.initialize().await?;

    println!("🌐 Connecting to zero trust network...");
    network_manager.connect().await?;

    // Get network capabilities
    let capabilities = opensim_next::ziti::ZitiCapabilities::default();
    println!("🛡️  Zero Trust Capabilities:");
    println!("   - Encryption: {}", capabilities.supports_encryption);
    println!(
        "   - Dark Services: {}",
        capabilities.supports_dark_services
    );
    println!(
        "   - Policy Enforcement: {}",
        capabilities.supports_policy_enforcement
    );
    println!(
        "   - Identity Verification: {}",
        capabilities.supports_identity_verification
    );
    println!("   - Max Connections: {}", capabilities.max_connections);

    // Get network statistics
    let stats = network_manager.get_network_stats().await?;
    println!("📊 Network Statistics:");
    for (key, value) in stats {
        println!("   - {}: {}", key, value);
    }

    println!("📡 Basic network manager demo completed");
    Ok(())
}

/// Demo 2: OpenSim Integration with Zero Trust
async fn demo_opensim_integration() -> Result<()> {
    println!("\n🏗️  Demo 2: OpenSim Integration with Zero Trust");
    println!("----------------------------------------------");

    // Create OpenSim-specific configuration
    let opensim_config = OpenSimZitiConfig::default();

    println!("🔧 Creating OpenSim-OpenZiti integration...");
    let mut integration = OpenSimZitiIntegration::new(opensim_config).await?;

    println!("🚀 Initializing integration with default services...");
    integration.initialize().await?;

    // Show integration status
    println!(
        "✅ Integration Status: {}",
        if integration.is_enabled() {
            "Enabled"
        } else {
            "Disabled"
        }
    );

    // Get integration statistics
    let stats = integration.get_statistics().await?;
    println!("📊 Integration Statistics:");
    for (key, value) in stats {
        println!("   - {}: {}", key, value);
    }

    println!("🏗️  OpenSim integration demo completed");
    Ok(())
}

/// Demo 3: Secure Service Communication
async fn demo_secure_service_communication() -> Result<()> {
    println!("\n🔐 Demo 3: Secure Service Communication");
    println!("--------------------------------------");

    let opensim_config = OpenSimZitiConfig::default();
    let integration = OpenSimZitiIntegration::new(opensim_config).await?;

    // Simulate creating secure connections for different components
    let components = vec![
        (OpenSimComponentType::RegionCommunication, "region-service"),
        (OpenSimComponentType::AssetTransfer, "asset-service"),
        (OpenSimComponentType::UserAuthentication, "auth-service"),
        (OpenSimComponentType::Administration, "admin-service"),
        (OpenSimComponentType::WebSocketService, "websocket-service"),
    ];

    for (component_type, service_name) in components {
        println!("🔗 Creating secure connection for {:?}...", component_type);

        // Create secure connection
        match integration
            .create_secure_connection(
                component_type.clone(),
                service_name,
                Some("opensim-demo-user"),
            )
            .await
        {
            Ok(connection_id) => {
                println!("   ✅ Connection established: {}", connection_id);

                // Simulate sending secure data
                let demo_data = format!("Hello from {:?}!", component_type).into_bytes();
                match integration
                    .send_secure_data(&connection_id, &demo_data, component_type)
                    .await
                {
                    Ok(bytes_sent) => {
                        println!("   📤 Sent {} bytes securely", bytes_sent);
                    }
                    Err(e) => {
                        println!("   ❌ Failed to send data: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Connection failed: {}", e);
            }
        }

        // Small delay between connections
        sleep(Duration::from_millis(100)).await;
    }

    println!("🔐 Secure service communication demo completed");
    Ok(())
}

/// Demo 4: Zero Trust Policies
async fn demo_zero_trust_policies() -> Result<()> {
    println!("\n🛡️  Demo 4: Zero Trust Policies");
    println!("------------------------------");

    let config = ZitiConfig::default();
    let network_manager = ZitiNetworkManager::new(config)?;

    // Simulate policy checks
    let test_cases = vec![
        ("admin-user", "admin-service", "Administrator access"),
        ("regular-user", "asset-service", "User asset access"),
        (
            "region-server",
            "region-communication",
            "Region-to-region communication",
        ),
        ("web-client", "websocket-service", "Web client connection"),
        ("guest-user", "admin-service", "Unauthorized admin access"),
    ];

    for (identity, service, description) in test_cases {
        println!("🔍 Checking access: {}", description);
        println!("   Identity: {} → Service: {}", identity, service);

        match network_manager.check_access(identity, service).await {
            Ok(has_access) => {
                if has_access {
                    println!("   ✅ Access granted");
                } else {
                    println!("   ❌ Access denied");
                }
            }
            Err(e) => {
                println!("   ⚠️  Policy check failed: {}", e);
            }
        }

        sleep(Duration::from_millis(50)).await;
    }

    println!("🛡️  Zero trust policies demo completed");
    Ok(())
}

/// Demo 5: Performance Monitoring
async fn demo_performance_monitoring() -> Result<()> {
    println!("\n📈 Demo 5: Performance Monitoring");
    println!("---------------------------------");

    let config = ZitiConfig::default();
    let network_manager = ZitiNetworkManager::new(config)?;

    println!("📊 Simulating network activity for monitoring...");

    // Simulate some network activity
    for i in 1..=5 {
        println!("   🔄 Activity round {}/5", i);

        // Get current statistics
        match network_manager.get_network_stats().await {
            Ok(stats) => {
                println!("   📊 Current stats:");
                for (key, value) in stats.iter().take(5) {
                    // Show first 5 stats
                    println!("      - {}: {}", key, value);
                }
            }
            Err(e) => {
                println!("   ⚠️  Failed to get stats: {}", e);
            }
        }

        // Simulate processing time
        sleep(Duration::from_millis(500)).await;
    }

    // Final statistics summary
    println!("📈 Final Network Performance Summary:");
    match network_manager.get_network_stats().await {
        Ok(stats) => {
            let connections = stats.get("connections_total").unwrap_or(&0);
            let bytes_sent = stats.get("bytes_sent").unwrap_or(&0);
            let bytes_received = stats.get("bytes_received").unwrap_or(&0);

            println!("   🔗 Total Connections: {}", connections);
            println!("   📤 Bytes Sent: {}", bytes_sent);
            println!("   📥 Bytes Received: {}", bytes_received);
            println!("   🚀 Zero Trust Network: Active");
        }
        Err(e) => {
            println!("   ❌ Failed to get final stats: {}", e);
        }
    }

    println!("📈 Performance monitoring demo completed");
    Ok(())
}

/// Helper function to demonstrate network capabilities
fn demonstrate_zero_trust_benefits() {
    println!("\n🌟 Zero Trust Networking Benefits for Virtual Worlds:");
    println!("====================================================");
    println!("🔒 Application-Embedded Security:");
    println!("   • No VPNs or firewalls required");
    println!("   • Security travels with the application");
    println!("   • Encrypted by default, always");

    println!("\n🕵️  Identity-Based Access:");
    println!("   • Every connection requires authentication");
    println!("   • Granular access control policies");
    println!("   • Continuous verification");

    println!("\n🌐 Dark Services:");
    println!("   • Services are invisible until authorized");
    println!("   • No network scanning or discovery attacks");
    println!("   • Micro-segmentation by default");

    println!("\n🚀 Performance Benefits:");
    println!("   • Direct peer-to-peer connections");
    println!("   • Intelligent routing and failover");
    println!("   • Reduced latency for virtual world traffic");

    println!("\n🛡️  Virtual World Specific Security:");
    println!("   • Secure asset transfer and caching");
    println!("   • Protected region-to-region communication");
    println!("   • Encrypted user authentication");
    println!("   • Secure administrative access");
    println!("   • Protected WebSocket connections for web clients");
}
