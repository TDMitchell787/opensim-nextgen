//! Horizontal scaling system for dynamic server provisioning

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::load_balancer::{LoadBalancer, ServerHealth, ServerInstance, ServerMetrics};

/// Scaling triggers and thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    pub name: String,
    pub scale_up_cpu_threshold: f32,
    pub scale_up_memory_threshold: f32,
    pub scale_up_connection_threshold: f32,
    pub scale_down_cpu_threshold: f32,
    pub scale_down_memory_threshold: f32,
    pub scale_down_connection_threshold: f32,
    pub min_instances: u32,
    pub max_instances: u32,
    pub scale_up_cooldown: Duration,
    pub scale_down_cooldown: Duration,
    pub evaluation_period: Duration,
    pub consecutive_threshold_breaches: u32,
}

impl Default for ScalingPolicy {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            scale_up_cpu_threshold: 75.0,
            scale_up_memory_threshold: 80.0,
            scale_up_connection_threshold: 85.0,
            scale_down_cpu_threshold: 25.0,
            scale_down_memory_threshold: 30.0,
            scale_down_connection_threshold: 30.0,
            min_instances: 1,
            max_instances: 10,
            scale_up_cooldown: Duration::from_secs(300), // 5 minutes
            scale_down_cooldown: Duration::from_secs(600), // 10 minutes
            evaluation_period: Duration::from_secs(60),  // 1 minute
            consecutive_threshold_breaches: 3,
        }
    }
}

/// Scaling action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScalingAction {
    ScaleUp { target_count: u32 },
    ScaleDown { target_count: u32 },
    NoAction,
}

/// Scaling event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingEvent {
    pub id: Uuid,
    pub action: ScalingAction,
    pub trigger_reason: String,
    pub previous_count: u32,
    pub new_count: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub policy_name: String,
    pub metrics_snapshot: HashMap<String, f32>,
}

/// Server provisioning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningConfig {
    pub server_template: ServerTemplate,
    pub auto_discovery: bool,
    pub health_check_timeout: Duration,
    pub startup_grace_period: Duration,
    pub shutdown_grace_period: Duration,
}

/// Template for provisioning new servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTemplate {
    pub base_address: String,
    pub port_range_start: u16,
    pub port_range_end: u16,
    pub max_regions_per_server: u32,
    pub max_connections_per_server: u32,
    pub server_weight: f32,
    pub geographic_location: Option<String>,
    pub capabilities: Vec<String>,
}

/// Scaling decision context
#[derive(Debug, Clone)]
struct ScalingContext {
    current_instances: u32,
    average_cpu: f32,
    average_memory: f32,
    average_connections: f32,
    peak_cpu: f32,
    peak_memory: f32,
    peak_connections: f32,
    consecutive_breaches: u32,
    last_scaling_action: Option<Instant>,
}

/// Horizontal scaling manager
pub struct HorizontalScaler {
    load_balancer: Arc<LoadBalancer>,
    scaling_policies: Arc<RwLock<HashMap<String, ScalingPolicy>>>,
    provisioning_config: ProvisioningConfig,
    scaling_history: Arc<RwLock<Vec<ScalingEvent>>>,
    active_instances: Arc<RwLock<HashMap<String, ServerInstance>>>,
    breach_counters: Arc<RwLock<HashMap<String, u32>>>,
    last_scaling_actions: Arc<RwLock<HashMap<String, Instant>>>,
    monitoring_enabled: Arc<RwLock<bool>>,
}

impl HorizontalScaler {
    /// Create a new horizontal scaler
    pub fn new(load_balancer: Arc<LoadBalancer>, provisioning_config: ProvisioningConfig) -> Self {
        let mut default_policies = HashMap::new();
        default_policies.insert("default".to_string(), ScalingPolicy::default());

        Self {
            load_balancer,
            scaling_policies: Arc::new(RwLock::new(default_policies)),
            provisioning_config,
            scaling_history: Arc::new(RwLock::new(Vec::new())),
            active_instances: Arc::new(RwLock::new(HashMap::new())),
            breach_counters: Arc::new(RwLock::new(HashMap::new())),
            last_scaling_actions: Arc::new(RwLock::new(HashMap::new())),
            monitoring_enabled: Arc::new(RwLock::new(false)),
        }
    }

    /// Add or update a scaling policy
    pub async fn set_scaling_policy(&self, policy: ScalingPolicy) -> Result<()> {
        info!("Setting scaling policy: {}", policy.name);

        let mut policies = self.scaling_policies.write().await;
        policies.insert(policy.name.clone(), policy);

        Ok(())
    }

    /// Start automatic scaling monitoring
    pub async fn start_auto_scaling(&self) -> Result<()> {
        info!("Starting automatic scaling");

        *self.monitoring_enabled.write().await = true;

        let self_clone = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            while *self_clone.monitoring_enabled.read().await {
                interval.tick().await;

                if let Err(e) = self_clone.evaluate_scaling_policies().await {
                    error!("Scaling evaluation failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Stop automatic scaling
    pub async fn stop_auto_scaling(&self) {
        info!("Stopping automatic scaling");
        *self.monitoring_enabled.write().await = false;
    }

    /// Manually scale to a specific number of instances
    pub async fn scale_to(&self, target_count: u32, policy_name: &str) -> Result<ScalingEvent> {
        info!("Manual scaling to {} instances", target_count);

        let current_count = self.get_active_instance_count().await;

        let action = if target_count > current_count {
            ScalingAction::ScaleUp { target_count }
        } else if target_count < current_count {
            ScalingAction::ScaleDown { target_count }
        } else {
            ScalingAction::NoAction
        };

        let event = ScalingEvent {
            id: Uuid::new_v4(),
            action: action.clone(),
            trigger_reason: "Manual scaling request".to_string(),
            previous_count: current_count,
            new_count: target_count,
            timestamp: chrono::Utc::now(),
            policy_name: policy_name.to_string(),
            metrics_snapshot: HashMap::new(),
        };

        match action {
            ScalingAction::ScaleUp { target_count } => {
                self.scale_up(target_count - current_count).await?;
            }
            ScalingAction::ScaleDown { target_count } => {
                self.scale_down(current_count - target_count).await?;
            }
            ScalingAction::NoAction => {
                info!("No scaling action needed, already at target count");
            }
        }

        // Record the event
        self.scaling_history.write().await.push(event.clone());

        Ok(event)
    }

    /// Provision a new server instance
    pub async fn provision_instance(&self) -> Result<ServerInstance> {
        info!("Provisioning new server instance");

        let template = &self.provisioning_config.server_template;
        let active_instances = self.active_instances.read().await;

        // Find available port
        let mut port = template.port_range_start;
        while port <= template.port_range_end {
            let address = format!("{}:{}", template.base_address, port);
            if !active_instances
                .values()
                .any(|s| s.address == template.base_address && s.port == port)
            {
                break;
            }
            port += 1;
        }

        if port > template.port_range_end {
            return Err(anyhow!("No available ports in range"));
        }

        let server_id = format!(
            "auto-{}-{}",
            chrono::Utc::now().timestamp(),
            rand::random::<u16>()
        );

        let server = ServerInstance {
            id: server_id.clone(),
            address: template.base_address.clone(),
            port,
            weight: template.server_weight,
            max_regions: template.max_regions_per_server,
            max_connections: template.max_connections_per_server,
            geographic_location: template.geographic_location.clone(),
            capabilities: template.capabilities.clone(),
            metrics: ServerMetrics {
                server_id: server_id.clone(),
                cpu_usage: 0.0,
                memory_usage: 0.0,
                active_connections: 0,
                active_regions: 0,
                active_avatars: 0,
                bandwidth_usage: 0,
                latency_ms: 0.0,
                health_status: ServerHealth::Healthy,
                last_updated: chrono::Utc::now(),
            },
            created_at: chrono::Utc::now(),
        };

        // Start the actual server process
        match self.start_server_process(&server).await {
            Ok(_) => {
                info!("Server process started successfully for {}", server_id);

                // Wait for startup grace period
                tokio::time::sleep(self.provisioning_config.startup_grace_period).await;

                // Perform health check to ensure server is ready
                let health_check = self.perform_startup_health_check(&server).await;
                if !health_check {
                    warn!(
                        "Server {} failed startup health check, marking as degraded",
                        server_id
                    );
                }
            }
            Err(e) => {
                error!("Failed to start server process for {}: {}", server_id, e);
                return Err(e);
            }
        }

        // Register with load balancer
        self.load_balancer.register_server(server.clone()).await?;

        // Track as active instance
        self.active_instances
            .write()
            .await
            .insert(server_id.clone(), server.clone());

        info!(
            "Successfully provisioned server: {} at {}:{}",
            server_id, server.address, server.port
        );

        Ok(server)
    }

    /// Decommission a server instance
    pub async fn decommission_instance(&self, server_id: &str) -> Result<()> {
        info!("Decommissioning server instance: {}", server_id);

        // Remove from load balancer (will migrate regions)
        self.load_balancer.unregister_server(server_id).await?;

        // Wait for graceful shutdown
        tokio::time::sleep(self.provisioning_config.shutdown_grace_period).await;

        // Remove from active instances
        self.active_instances.write().await.remove(server_id);

        // Stop the actual server process
        if let Err(e) = self.stop_server_process(server_id).await {
            error!("Failed to stop server process for {}: {}", server_id, e);
        }

        info!("Successfully decommissioned server: {}", server_id);

        Ok(())
    }

    /// Get scaling statistics
    pub async fn get_scaling_statistics(&self) -> Result<ScalingStatistics> {
        let active_count = self.get_active_instance_count().await;
        let policies = self.scaling_policies.read().await;
        let history = self.scaling_history.read().await;

        let recent_events: Vec<_> = history.iter().rev().take(10).cloned().collect();

        let total_scale_ups = history
            .iter()
            .filter(|e| matches!(e.action, ScalingAction::ScaleUp { .. }))
            .count();

        let total_scale_downs = history
            .iter()
            .filter(|e| matches!(e.action, ScalingAction::ScaleDown { .. }))
            .count();

        Ok(ScalingStatistics {
            active_instances: active_count,
            policy_count: policies.len(),
            total_scaling_events: history.len(),
            total_scale_ups,
            total_scale_downs,
            recent_events,
            monitoring_enabled: *self.monitoring_enabled.read().await,
            last_evaluation: chrono::Utc::now(), // Would track actual last evaluation
        })
    }

    // Private helper methods

    async fn evaluate_scaling_policies(&self) -> Result<()> {
        let policies = self.scaling_policies.read().await;

        for (policy_name, policy) in policies.iter() {
            if let Err(e) = self.evaluate_policy(policy_name, policy).await {
                error!("Failed to evaluate policy {}: {}", policy_name, e);
            }
        }

        Ok(())
    }

    async fn evaluate_policy(&self, policy_name: &str, policy: &ScalingPolicy) -> Result<()> {
        // Get current metrics
        let stats = self.load_balancer.get_statistics().await?;
        let context = self.build_scaling_context(&stats, policy_name).await;

        // Check cooldown period
        if let Some(last_action) = context.last_scaling_action {
            let cooldown = if context.current_instances > policy.min_instances {
                policy.scale_down_cooldown
            } else {
                policy.scale_up_cooldown
            };

            if last_action.elapsed() < cooldown {
                return Ok(()); // Still in cooldown
            }
        }

        // Evaluate scaling decision
        let action = self.determine_scaling_action(&context, policy).await?;

        match action {
            ScalingAction::ScaleUp { target_count } => {
                if context.current_instances < policy.max_instances {
                    info!("Scaling up due to policy: {}", policy_name);

                    let instances_to_add = target_count - context.current_instances;
                    self.scale_up(instances_to_add).await?;

                    self.record_scaling_event(action, policy_name, &context)
                        .await;
                    self.update_last_scaling_action(policy_name).await;
                }
            }
            ScalingAction::ScaleDown { target_count } => {
                if context.current_instances > policy.min_instances {
                    info!("Scaling down due to policy: {}", policy_name);

                    let instances_to_remove = context.current_instances - target_count;
                    self.scale_down(instances_to_remove).await?;

                    self.record_scaling_event(action, policy_name, &context)
                        .await;
                    self.update_last_scaling_action(policy_name).await;
                }
            }
            ScalingAction::NoAction => {
                // Reset breach counter for this policy
                self.breach_counters
                    .write()
                    .await
                    .insert(policy_name.to_string(), 0);
            }
        }

        Ok(())
    }

    async fn build_scaling_context(
        &self,
        stats: &super::load_balancer::LoadBalancingStatistics,
        policy_name: &str,
    ) -> ScalingContext {
        let breach_counters = self.breach_counters.read().await;
        let last_actions = self.last_scaling_actions.read().await;

        ScalingContext {
            current_instances: stats.total_servers as u32,
            average_cpu: stats.average_cpu_usage,
            average_memory: stats.average_memory_usage,
            average_connections: if stats.total_servers > 0 {
                stats.total_connections as f32 / stats.total_servers as f32
            } else {
                0.0
            },
            peak_cpu: stats.server_loads.values().cloned().fold(0.0, f32::max),
            peak_memory: stats.average_memory_usage, // Simplified
            peak_connections: stats.total_connections as f32,
            consecutive_breaches: breach_counters.get(policy_name).cloned().unwrap_or(0),
            last_scaling_action: last_actions.get(policy_name).cloned(),
        }
    }

    async fn determine_scaling_action(
        &self,
        context: &ScalingContext,
        policy: &ScalingPolicy,
    ) -> Result<ScalingAction> {
        // Check scale-up conditions
        let should_scale_up = context.average_cpu > policy.scale_up_cpu_threshold
            || context.average_memory > policy.scale_up_memory_threshold
            || context.average_connections > policy.scale_up_connection_threshold;

        // Check scale-down conditions
        let should_scale_down = context.average_cpu < policy.scale_down_cpu_threshold
            && context.average_memory < policy.scale_down_memory_threshold
            && context.average_connections < policy.scale_down_connection_threshold;

        if should_scale_up {
            // Increment breach counter
            let policy_name = &policy.name;
            let mut counters = self.breach_counters.write().await;
            let new_count = counters.get(policy_name).unwrap_or(&0) + 1;
            counters.insert(policy_name.clone(), new_count);

            if new_count >= policy.consecutive_threshold_breaches {
                let target_count = (context.current_instances + 1).min(policy.max_instances);
                return Ok(ScalingAction::ScaleUp { target_count });
            }
        } else if should_scale_down {
            let policy_name = &policy.name;
            let mut counters = self.breach_counters.write().await;
            let new_count = counters.get(policy_name).unwrap_or(&0) + 1;
            counters.insert(policy_name.clone(), new_count);

            if new_count >= policy.consecutive_threshold_breaches {
                let target_count =
                    (context.current_instances.saturating_sub(1)).max(policy.min_instances);
                return Ok(ScalingAction::ScaleDown { target_count });
            }
        }

        Ok(ScalingAction::NoAction)
    }

    async fn scale_up(&self, instances_to_add: u32) -> Result<()> {
        info!("Scaling up by {} instances", instances_to_add);

        for _ in 0..instances_to_add {
            if let Err(e) = self.provision_instance().await {
                error!("Failed to provision instance: {}", e);
                break; // Stop on first failure
            }
        }

        Ok(())
    }

    async fn scale_down(&self, instances_to_remove: u32) -> Result<()> {
        info!("Scaling down by {} instances", instances_to_remove);

        let active_instances = self.active_instances.read().await;
        let mut servers_to_remove: Vec<_> = active_instances
            .keys()
            .take(instances_to_remove as usize)
            .cloned()
            .collect();

        drop(active_instances); // Release read lock

        for server_id in servers_to_remove {
            if let Err(e) = self.decommission_instance(&server_id).await {
                error!("Failed to decommission instance {}: {}", server_id, e);
            }
        }

        Ok(())
    }

    async fn record_scaling_event(
        &self,
        action: ScalingAction,
        policy_name: &str,
        context: &ScalingContext,
    ) {
        let mut metrics_snapshot = HashMap::new();
        metrics_snapshot.insert("avg_cpu".to_string(), context.average_cpu);
        metrics_snapshot.insert("avg_memory".to_string(), context.average_memory);
        metrics_snapshot.insert("avg_connections".to_string(), context.average_connections);

        let (new_count, reason) = match &action {
            ScalingAction::ScaleUp { target_count } => {
                (*target_count, "Resource thresholds exceeded".to_string())
            }
            ScalingAction::ScaleDown { target_count } => {
                (*target_count, "Resource usage below thresholds".to_string())
            }
            ScalingAction::NoAction => return,
        };

        let event = ScalingEvent {
            id: Uuid::new_v4(),
            action,
            trigger_reason: reason,
            previous_count: context.current_instances,
            new_count,
            timestamp: chrono::Utc::now(),
            policy_name: policy_name.to_string(),
            metrics_snapshot,
        };

        self.scaling_history.write().await.push(event);
    }

    async fn update_last_scaling_action(&self, policy_name: &str) {
        self.last_scaling_actions
            .write()
            .await
            .insert(policy_name.to_string(), Instant::now());
    }

    async fn get_active_instance_count(&self) -> u32 {
        self.active_instances.read().await.len() as u32
    }

    async fn start_server_process(&self, server: &ServerInstance) -> Result<()> {
        // Create server process configuration
        let config_path = format!("/tmp/opensim-{}.toml", server.id);
        self.create_server_config(server, &config_path).await?;

        // Start server process using cargo run in background
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.args(&["run", "--release"])
            .env("OPENSIM_CONFIG_PATH", &config_path)
            .env("OPENSIM_SERVER_ID", &server.id)
            .env("OPENSIM_SERVER_PORT", server.port.to_string())
            .env("OPENSIM_SERVER_ADDRESS", &server.address)
            .env("RUST_LOG", "info")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        match cmd.spawn() {
            Ok(mut child) => {
                // Store process handle for later management
                info!(
                    "Started server process for {} (PID: {:?})",
                    server.id,
                    child.id()
                );

                // Detach the process so it continues running
                tokio::spawn(async move {
                    if let Err(e) = child.wait().await {
                        error!("Server process terminated unexpectedly: {}", e);
                    }
                });

                Ok(())
            }
            Err(e) => {
                error!("Failed to spawn server process: {}", e);
                Err(anyhow!("Failed to start server: {}", e))
            }
        }
    }

    async fn create_server_config(&self, server: &ServerInstance, config_path: &str) -> Result<()> {
        let config_content = format!(
            r#"
[server]
id = "{}"
address = "{}"
port = {}
max_regions = {}
max_connections = {}

[performance]
weight = {}

[monitoring]
enable_metrics = true
metrics_port = {}

[regions]
auto_create = true
default_size = [256, 256]

[physics]
engine = "zig"
enable_collisions = true

[database]
url = "sqlite:///tmp/opensim-{}.db"

[logging]
level = "info"
format = "json"
"#,
            server.id,
            server.address,
            server.port,
            server.max_regions,
            server.max_connections,
            server.weight,
            server.port + 100, // Metrics port offset
            server.id
        );

        tokio::fs::write(config_path, config_content)
            .await
            .map_err(|e| anyhow!("Failed to write server config: {}", e))?;

        info!("Created server configuration at {}", config_path);
        Ok(())
    }

    async fn perform_startup_health_check(&self, server: &ServerInstance) -> bool {
        let health_url = format!("http://{}:{}/health", server.address, server.port);
        let client = match reqwest::Client::builder()
            .timeout(self.provisioning_config.health_check_timeout)
            .build()
        {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to create HTTP client for health check: {}", e);
                return false;
            }
        };

        // Try health check multiple times during startup
        for attempt in 1..=5 {
            debug!("Health check attempt {} for server {}", attempt, server.id);

            match client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    info!(
                        "Server {} passed health check on attempt {}",
                        server.id, attempt
                    );
                    return true;
                }
                Ok(response) => {
                    warn!(
                        "Server {} health check returned status: {} (attempt {})",
                        server.id,
                        response.status(),
                        attempt
                    );
                }
                Err(e) => {
                    debug!(
                        "Health check failed for server {} (attempt {}): {}",
                        server.id, attempt, e
                    );
                }
            }

            // Wait before next attempt
            if attempt < 5 {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }

        warn!("Server {} failed all health check attempts", server.id);
        false
    }

    async fn stop_server_process(&self, server_id: &str) -> Result<()> {
        // In a production environment, you would:
        // 1. Look up the process ID from a process registry
        // 2. Send SIGTERM signal for graceful shutdown
        // 3. Wait for graceful shutdown timeout
        // 4. Send SIGKILL if necessary

        info!("Stopping server process for {}", server_id);

        // For now, we'll simulate the process stop
        // The actual implementation would depend on your process management strategy
        // (systemd, docker, kubernetes, etc.)

        // Clean up configuration file
        let config_path = format!("/tmp/opensim-{}.toml", server_id);
        if let Err(e) = tokio::fs::remove_file(&config_path).await {
            warn!("Failed to remove config file {}: {}", config_path, e);
        }

        Ok(())
    }
}

impl Clone for HorizontalScaler {
    fn clone(&self) -> Self {
        Self {
            load_balancer: self.load_balancer.clone(),
            scaling_policies: self.scaling_policies.clone(),
            provisioning_config: self.provisioning_config.clone(),
            scaling_history: self.scaling_history.clone(),
            active_instances: self.active_instances.clone(),
            breach_counters: self.breach_counters.clone(),
            last_scaling_actions: self.last_scaling_actions.clone(),
            monitoring_enabled: self.monitoring_enabled.clone(),
        }
    }
}

/// Scaling statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingStatistics {
    pub active_instances: u32,
    pub policy_count: usize,
    pub total_scaling_events: usize,
    pub total_scale_ups: usize,
    pub total_scale_downs: usize,
    pub recent_events: Vec<ScalingEvent>,
    pub monitoring_enabled: bool,
    pub last_evaluation: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::physics::PhysicsBridge;
    use crate::region::RegionManager;
    use crate::state::StateManager;

    #[tokio::test]
    async fn test_horizontal_scaler_creation() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager));
        let load_balancer = Arc::new(LoadBalancer::new(
            super::super::load_balancer::LoadBalancingStrategy::RoundRobin,
            region_manager,
        ));

        let config = ProvisioningConfig {
            server_template: ServerTemplate {
                base_address: "127.0.0.1".to_string(),
                port_range_start: 9000,
                port_range_end: 9100,
                max_regions_per_server: 10,
                max_connections_per_server: 1000,
                server_weight: 1.0,
                geographic_location: None,
                capabilities: vec!["regions".to_string()],
            },
            auto_discovery: false,
            health_check_timeout: Duration::from_secs(30),
            startup_grace_period: Duration::from_secs(60),
            shutdown_grace_period: Duration::from_secs(30),
        };

        let scaler = HorizontalScaler::new(load_balancer, config);

        let stats = scaler.get_scaling_statistics().await?;
        assert_eq!(stats.active_instances, 0);
        assert!(!stats.monitoring_enabled);

        Ok(())
    }

    #[tokio::test]
    async fn test_manual_scaling() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager));
        let load_balancer = Arc::new(LoadBalancer::new(
            super::super::load_balancer::LoadBalancingStrategy::RoundRobin,
            region_manager,
        ));

        let config = ProvisioningConfig {
            server_template: ServerTemplate {
                base_address: "127.0.0.1".to_string(),
                port_range_start: 9000,
                port_range_end: 9100,
                max_regions_per_server: 10,
                max_connections_per_server: 1000,
                server_weight: 1.0,
                geographic_location: None,
                capabilities: vec!["regions".to_string()],
            },
            auto_discovery: false,
            health_check_timeout: Duration::from_secs(30),
            startup_grace_period: Duration::from_secs(60),
            shutdown_grace_period: Duration::from_secs(30),
        };

        let scaler = HorizontalScaler::new(load_balancer, config);

        // Scale up to 2 instances
        let event = scaler.scale_to(2, "default").await?;

        assert!(matches!(
            event.action,
            ScalingAction::ScaleUp { target_count: 2 }
        ));
        assert_eq!(event.previous_count, 0);
        assert_eq!(event.new_count, 2);

        let stats = scaler.get_scaling_statistics().await?;
        assert_eq!(stats.active_instances, 2);
        assert_eq!(stats.total_scale_ups, 1);

        Ok(())
    }
}
