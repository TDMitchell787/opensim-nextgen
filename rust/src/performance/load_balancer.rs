//! Load balancing system for multiple regions and servers

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

use crate::region::RegionManager;

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LoadBalancingStrategy {
    /// Round-robin selection
    RoundRobin,
    /// Least connections first
    LeastConnections,
    /// Weighted round-robin based on server capacity
    WeightedRoundRobin,
    /// Least CPU/memory usage
    LeastResourceUsage,
    /// Geographic proximity
    Geographic,
    /// Custom algorithm
    Custom(String),
}

/// Server health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Server performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub server_id: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub active_connections: u32,
    pub active_regions: u32,
    pub active_avatars: u32,
    pub bandwidth_usage: u64,
    pub latency_ms: f32,
    pub health_status: ServerHealth,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Server instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub weight: f32,
    pub max_regions: u32,
    pub max_connections: u32,
    pub geographic_location: Option<String>,
    pub capabilities: Vec<String>,
    pub metrics: ServerMetrics,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Region assignment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAssignment {
    pub region_id: Uuid,
    pub server_id: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub priority: u8,
    pub sticky: bool, // If true, try to keep users on this server
}

/// Load balancing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingDecision {
    pub target_server_id: String,
    pub strategy_used: LoadBalancingStrategy,
    pub decision_factors: HashMap<String, f32>,
    pub confidence_score: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Load balancer for distributing regions and users across multiple servers
pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    servers: Arc<RwLock<HashMap<String, ServerInstance>>>,
    region_assignments: Arc<RwLock<HashMap<Uuid, RegionAssignment>>>,
    round_robin_index: Arc<RwLock<usize>>,
    health_check_interval: Duration,
    rebalance_threshold: f32,
    region_manager: Arc<RegionManager>,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(strategy: LoadBalancingStrategy, region_manager: Arc<RegionManager>) -> Self {
        Self {
            strategy,
            servers: Arc::new(RwLock::new(HashMap::new())),
            region_assignments: Arc::new(RwLock::new(HashMap::new())),
            round_robin_index: Arc::new(RwLock::new(0)),
            health_check_interval: Duration::from_secs(30),
            rebalance_threshold: 0.8, // 80% threshold for rebalancing
            region_manager,
        }
    }

    /// Register a new server instance
    pub async fn register_server(&self, server: ServerInstance) -> Result<()> {
        info!(
            "Registering server: {} at {}:{}",
            server.id, server.address, server.port
        );

        let mut servers = self.servers.write().await;
        servers.insert(server.id.clone(), server);

        info!(
            "Server registered successfully. Total servers: {}",
            servers.len()
        );
        Ok(())
    }

    /// Unregister a server instance
    pub async fn unregister_server(&self, server_id: &str) -> Result<()> {
        warn!("Unregistering server: {}", server_id);

        // First, migrate all regions from this server
        self.migrate_regions_from_server(server_id).await?;

        // Remove from server list
        let mut servers = self.servers.write().await;
        servers.remove(server_id);

        info!("Server unregistered successfully: {}", server_id);
        Ok(())
    }

    /// Update server metrics
    pub async fn update_server_metrics(
        &self,
        server_id: &str,
        metrics: ServerMetrics,
    ) -> Result<()> {
        debug!("Updating metrics for server: {}", server_id);

        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(server_id) {
            server.metrics = metrics;

            // Check if server needs attention
            if server.metrics.health_status == ServerHealth::Unhealthy {
                warn!("Server {} is unhealthy, may need intervention", server_id);
            }
        } else {
            return Err(anyhow!("Server {} not found", server_id));
        }

        Ok(())
    }

    /// Select the best server for a new region
    pub async fn select_server_for_region(&self, region_id: Uuid) -> Result<LoadBalancingDecision> {
        debug!("Selecting server for region: {}", region_id);

        let servers = self.servers.read().await;
        let healthy_servers: Vec<&ServerInstance> = servers
            .values()
            .filter(|s| s.metrics.health_status == ServerHealth::Healthy)
            .collect();

        if healthy_servers.is_empty() {
            return Err(anyhow!("No healthy servers available"));
        }

        let decision = match &self.strategy {
            LoadBalancingStrategy::RoundRobin => self.select_round_robin(&healthy_servers).await,
            LoadBalancingStrategy::LeastConnections => {
                self.select_least_connections(&healthy_servers).await
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.select_weighted_round_robin(&healthy_servers).await
            }
            LoadBalancingStrategy::LeastResourceUsage => {
                self.select_least_resource_usage(&healthy_servers).await
            }
            LoadBalancingStrategy::Geographic => {
                self.select_geographic(&healthy_servers, region_id).await
            }
            LoadBalancingStrategy::Custom(algorithm) => {
                self.select_custom(&healthy_servers, algorithm).await
            }
        }?;

        // Record the assignment
        let assignment = RegionAssignment {
            region_id,
            server_id: decision.target_server_id.clone(),
            assigned_at: chrono::Utc::now(),
            priority: 1,
            sticky: false,
        };

        self.region_assignments
            .write()
            .await
            .insert(region_id, assignment);

        info!(
            "Selected server {} for region {} using strategy {:?}",
            decision.target_server_id, region_id, decision.strategy_used
        );

        Ok(decision)
    }

    /// Rebalance regions across servers
    pub async fn rebalance_regions(&self) -> Result<Vec<RegionAssignment>> {
        info!("Starting region rebalancing");

        let servers = self.servers.read().await;
        let assignments = self.region_assignments.read().await;

        let mut rebalance_actions = Vec::new();

        // Calculate server loads
        let mut server_loads: HashMap<String, f32> = HashMap::new();
        for server in servers.values() {
            let load = self.calculate_server_load(server);
            server_loads.insert(server.id.clone(), load);
        }

        // Find overloaded servers
        for (server_id, load) in &server_loads {
            if *load > self.rebalance_threshold {
                warn!("Server {} is overloaded (load: {:.2})", server_id, load);

                // Find regions to migrate
                let regions_to_migrate: Vec<_> = assignments
                    .values()
                    .filter(|a| &a.server_id == server_id && !a.sticky)
                    .take(1) // Start with migrating one region
                    .cloned()
                    .collect();

                for assignment in regions_to_migrate {
                    // Find a better server
                    if let Ok(decision) = self.select_server_for_region(assignment.region_id).await
                    {
                        if decision.target_server_id != assignment.server_id {
                            let new_assignment = RegionAssignment {
                                region_id: assignment.region_id,
                                server_id: decision.target_server_id,
                                assigned_at: chrono::Utc::now(),
                                priority: assignment.priority,
                                sticky: assignment.sticky,
                            };

                            rebalance_actions.push(new_assignment);
                        }
                    }
                }
            }
        }

        // Apply rebalancing actions
        if !rebalance_actions.is_empty() {
            info!("Applying {} rebalancing actions", rebalance_actions.len());
            let mut assignments_mut = self.region_assignments.write().await;
            for action in &rebalance_actions {
                assignments_mut.insert(action.region_id, action.clone());
            }
        }

        info!(
            "Region rebalancing completed with {} actions",
            rebalance_actions.len()
        );
        Ok(rebalance_actions)
    }

    /// Get current load balancing statistics
    pub async fn get_statistics(&self) -> Result<LoadBalancingStatistics> {
        let servers = self.servers.read().await;
        let assignments = self.region_assignments.read().await;

        let total_servers = servers.len();
        let healthy_servers = servers
            .values()
            .filter(|s| s.metrics.health_status == ServerHealth::Healthy)
            .count();

        let total_regions = assignments.len();
        let total_connections: u32 = servers.values().map(|s| s.metrics.active_connections).sum();

        let average_cpu: f32 = if total_servers > 0 {
            servers.values().map(|s| s.metrics.cpu_usage).sum::<f32>() / total_servers as f32
        } else {
            0.0
        };

        let average_memory: f32 = if total_servers > 0 {
            servers
                .values()
                .map(|s| s.metrics.memory_usage)
                .sum::<f32>()
                / total_servers as f32
        } else {
            0.0
        };

        // Calculate load distribution
        let mut server_loads = HashMap::new();
        for server in servers.values() {
            let load = self.calculate_server_load(server);
            server_loads.insert(server.id.clone(), load);
        }

        Ok(LoadBalancingStatistics {
            total_servers,
            healthy_servers,
            total_regions,
            total_connections,
            average_cpu_usage: average_cpu,
            average_memory_usage: average_memory,
            server_loads,
            strategy: self.strategy.clone(),
            last_rebalance: chrono::Utc::now(), // Would track actual last rebalance time
        })
    }

    /// Start health monitoring and automatic rebalancing
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("Starting load balancer monitoring");

        let self_clone = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self_clone.health_check_interval);

            loop {
                interval.tick().await;

                // Perform health checks
                if let Err(e) = self_clone.perform_health_checks().await {
                    error!("Health check failed: {}", e);
                }

                // Perform automatic rebalancing
                if let Err(e) = self_clone.rebalance_regions().await {
                    error!("Automatic rebalancing failed: {}", e);
                }
            }
        });

        Ok(())
    }

    // Private helper methods

    async fn select_round_robin(
        &self,
        servers: &[&ServerInstance],
    ) -> Result<LoadBalancingDecision> {
        let mut index = self.round_robin_index.write().await;
        *index = (*index + 1) % servers.len();

        let selected_server = servers[*index];

        Ok(LoadBalancingDecision {
            target_server_id: selected_server.id.clone(),
            strategy_used: LoadBalancingStrategy::RoundRobin,
            decision_factors: HashMap::from([("index".to_string(), *index as f32)]),
            confidence_score: 1.0,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn select_least_connections(
        &self,
        servers: &[&ServerInstance],
    ) -> Result<LoadBalancingDecision> {
        let selected_server = servers
            .iter()
            .min_by_key(|s| s.metrics.active_connections)
            .ok_or_else(|| anyhow!("No servers available"))?;

        let mut factors = HashMap::new();
        factors.insert(
            "connections".to_string(),
            selected_server.metrics.active_connections as f32,
        );

        Ok(LoadBalancingDecision {
            target_server_id: selected_server.id.clone(),
            strategy_used: LoadBalancingStrategy::LeastConnections,
            decision_factors: factors,
            confidence_score: 0.9,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn select_weighted_round_robin(
        &self,
        servers: &[&ServerInstance],
    ) -> Result<LoadBalancingDecision> {
        // Calculate cumulative weights
        let total_weight: f32 = servers.iter().map(|s| s.weight).sum();
        let random_value = rand::random::<f32>() * total_weight;

        let mut cumulative_weight = 0.0;
        for server in servers {
            cumulative_weight += server.weight;
            if random_value <= cumulative_weight {
                let mut factors = HashMap::new();
                factors.insert("weight".to_string(), server.weight);
                factors.insert("total_weight".to_string(), total_weight);

                return Ok(LoadBalancingDecision {
                    target_server_id: server.id.clone(),
                    strategy_used: LoadBalancingStrategy::WeightedRoundRobin,
                    decision_factors: factors,
                    confidence_score: 0.8,
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        // Fallback to first server
        let server = servers[0];
        Ok(LoadBalancingDecision {
            target_server_id: server.id.clone(),
            strategy_used: LoadBalancingStrategy::WeightedRoundRobin,
            decision_factors: HashMap::new(),
            confidence_score: 0.5,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn select_least_resource_usage(
        &self,
        servers: &[&ServerInstance],
    ) -> Result<LoadBalancingDecision> {
        let selected_server = servers
            .iter()
            .min_by(|a, b| {
                let load_a = self.calculate_server_load(a);
                let load_b = self.calculate_server_load(b);
                load_a
                    .partial_cmp(&load_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No servers available"))?;

        let load = self.calculate_server_load(selected_server);
        let mut factors = HashMap::new();
        factors.insert("cpu_usage".to_string(), selected_server.metrics.cpu_usage);
        factors.insert(
            "memory_usage".to_string(),
            selected_server.metrics.memory_usage,
        );
        factors.insert("calculated_load".to_string(), load);

        Ok(LoadBalancingDecision {
            target_server_id: selected_server.id.clone(),
            strategy_used: LoadBalancingStrategy::LeastResourceUsage,
            decision_factors: factors,
            confidence_score: 0.95,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn select_geographic(
        &self,
        servers: &[&ServerInstance],
        region_id: Uuid,
    ) -> Result<LoadBalancingDecision> {
        // Calculate geographic proximity scores for each server
        let region_location = self.get_region_location(region_id).await;

        let mut server_scores: Vec<(f32, &ServerInstance)> = servers
            .iter()
            .map(|server| {
                let proximity_score = self.calculate_geographic_proximity(&region_location, server);
                (proximity_score, *server)
            })
            .collect();

        // Sort by proximity score (higher is better)
        server_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let (score, selected_server) = server_scores
            .first()
            .ok_or_else(|| anyhow!("No servers available"))?;

        let mut factors = HashMap::new();
        factors.insert("proximity_score".to_string(), *score);
        if let Some(location) = &selected_server.geographic_location {
            factors.insert("server_location".to_string(), location.len() as f32);
        }
        factors.insert("region_location".to_string(), region_location.len() as f32);

        Ok(LoadBalancingDecision {
            target_server_id: selected_server.id.clone(),
            strategy_used: LoadBalancingStrategy::Geographic,
            decision_factors: factors,
            confidence_score: 0.85,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn select_custom(
        &self,
        servers: &[&ServerInstance],
        algorithm: &str,
    ) -> Result<LoadBalancingDecision> {
        match algorithm {
            "hybrid" => self.select_hybrid_algorithm(servers).await,
            "latency_optimized" => self.select_latency_optimized(servers).await,
            "capacity_aware" => self.select_capacity_aware(servers).await,
            _ => {
                warn!(
                    "Unknown custom algorithm '{}', falling back to least resource usage",
                    algorithm
                );
                self.select_least_resource_usage(servers).await
            }
        }
    }

    fn calculate_server_load(&self, server: &ServerInstance) -> f32 {
        // Weighted load calculation
        let cpu_weight = 0.4;
        let memory_weight = 0.3;
        let connection_weight = 0.2;
        let region_weight = 0.1;

        let cpu_load = server.metrics.cpu_usage / 100.0;
        let memory_load = server.metrics.memory_usage / 100.0;
        let connection_load =
            server.metrics.active_connections as f32 / server.max_connections as f32;
        let region_load = server.metrics.active_regions as f32 / server.max_regions as f32;

        cpu_weight * cpu_load
            + memory_weight * memory_load
            + connection_weight * connection_load
            + region_weight * region_load
    }

    async fn migrate_regions_from_server(&self, server_id: &str) -> Result<()> {
        info!("Migrating regions from server: {}", server_id);

        let assignments = self.region_assignments.read().await;
        let regions_to_migrate: Vec<_> = assignments
            .values()
            .filter(|a| a.server_id == server_id)
            .cloned()
            .collect();

        drop(assignments); // Release read lock

        for assignment in regions_to_migrate {
            // Select a new server for this region
            if let Ok(decision) = self.select_server_for_region(assignment.region_id).await {
                info!(
                    "Migrating region {} from {} to {}",
                    assignment.region_id, server_id, decision.target_server_id
                );

                // Update assignment
                let new_assignment = RegionAssignment {
                    region_id: assignment.region_id,
                    server_id: decision.target_server_id,
                    assigned_at: chrono::Utc::now(),
                    priority: assignment.priority,
                    sticky: assignment.sticky,
                };

                self.region_assignments
                    .write()
                    .await
                    .insert(assignment.region_id, new_assignment);
            }
        }

        Ok(())
    }

    async fn perform_health_checks(&self) -> Result<()> {
        let mut servers = self.servers.write().await;

        for server in servers.values_mut() {
            // Perform actual health check
            match self.ping_server(server).await {
                Ok(health_status) => {
                    server.metrics.health_status = health_status;
                    server.metrics.last_updated = chrono::Utc::now();
                    debug!("Health check passed for server {}", server.id);
                }
                Err(e) => {
                    warn!("Health check failed for server {}: {}", server.id, e);
                    server.metrics.health_status = ServerHealth::Unhealthy;

                    // Mark server offline if health checks continue to fail
                    let last_update = server.metrics.last_updated;
                    let now = chrono::Utc::now();
                    let age = now.signed_duration_since(last_update);

                    if age > chrono::Duration::minutes(5) {
                        server.metrics.health_status = ServerHealth::Offline;
                        error!(
                            "Server {} marked as offline due to prolonged health check failures",
                            server.id
                        );
                    }
                }
            }
        }

        Ok(())
    }

    async fn get_region_location(&self, region_id: Uuid) -> String {
        // Get region location from region manager
        match self
            .region_manager
            .get_region_location(crate::region::RegionId(region_id.as_u128() as u64))
            .await
        {
            Some((x, y)) => format!("grid:{}:{}", x, y),
            None => {
                debug!("Could not determine location for region {}", region_id);
                "unknown".to_string()
            }
        }
    }

    fn calculate_geographic_proximity(
        &self,
        region_location: &str,
        server: &ServerInstance,
    ) -> f32 {
        match (&server.geographic_location, region_location) {
            (Some(server_location), region_loc) if !region_loc.is_empty() => {
                // Simple proximity calculation based on location strings
                // In production, this would use actual geographic coordinates and distance calculations
                if server_location == region_loc {
                    1.0 // Perfect match
                } else if server_location.contains(region_loc)
                    || region_loc.contains(server_location)
                {
                    0.7 // Partial match (e.g., same country/continent)
                } else {
                    0.3 // Different locations
                }
            }
            (Some(_), _) => 0.5, // Server has location but region location is unknown
            (None, _) => 0.1,    // Server has no location preference
        }
    }

    async fn select_hybrid_algorithm(
        &self,
        servers: &[&ServerInstance],
    ) -> Result<LoadBalancingDecision> {
        // Hybrid algorithm combines multiple factors
        let mut server_scores: Vec<(f32, &ServerInstance)> = servers
            .iter()
            .map(|server| {
                let load_score = 1.0 - self.calculate_server_load(server); // Invert load (lower load = higher score)
                let health_score = match server.metrics.health_status {
                    ServerHealth::Healthy => 1.0,
                    ServerHealth::Degraded => 0.5,
                    ServerHealth::Unhealthy => 0.1,
                    ServerHealth::Offline => 0.0,
                };
                let latency_score = 1.0 - (server.metrics.latency_ms / 1000.0).min(1.0); // Normalize to 0-1

                // Weighted combination
                let total_score = (load_score * 0.4) + (health_score * 0.4) + (latency_score * 0.2);
                (total_score, *server)
            })
            .collect();

        server_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let (score, selected_server) = server_scores
            .first()
            .ok_or_else(|| anyhow!("No servers available"))?;

        let mut factors = HashMap::new();
        factors.insert("hybrid_score".to_string(), *score);
        factors.insert(
            "server_load".to_string(),
            self.calculate_server_load(selected_server),
        );
        factors.insert("latency_ms".to_string(), selected_server.metrics.latency_ms);

        Ok(LoadBalancingDecision {
            target_server_id: selected_server.id.clone(),
            strategy_used: LoadBalancingStrategy::Custom("hybrid".to_string()),
            decision_factors: factors,
            confidence_score: 0.9,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn select_latency_optimized(
        &self,
        servers: &[&ServerInstance],
    ) -> Result<LoadBalancingDecision> {
        let selected_server = servers
            .iter()
            .filter(|s| s.metrics.health_status == ServerHealth::Healthy)
            .min_by(|a, b| {
                a.metrics
                    .latency_ms
                    .partial_cmp(&b.metrics.latency_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No healthy servers available"))?;

        let mut factors = HashMap::new();
        factors.insert("latency_ms".to_string(), selected_server.metrics.latency_ms);

        Ok(LoadBalancingDecision {
            target_server_id: selected_server.id.clone(),
            strategy_used: LoadBalancingStrategy::Custom("latency_optimized".to_string()),
            decision_factors: factors,
            confidence_score: 0.95,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn select_capacity_aware(
        &self,
        servers: &[&ServerInstance],
    ) -> Result<LoadBalancingDecision> {
        let selected_server = servers
            .iter()
            .filter(|s| s.metrics.health_status == ServerHealth::Healthy)
            .max_by(|a, b| {
                let capacity_a = (a.max_connections - a.metrics.active_connections) as f32
                    + (a.max_regions - a.metrics.active_regions) as f32;
                let capacity_b = (b.max_connections - b.metrics.active_connections) as f32
                    + (b.max_regions - b.metrics.active_regions) as f32;
                capacity_a
                    .partial_cmp(&capacity_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No healthy servers available"))?;

        let available_capacity = (selected_server.max_connections
            - selected_server.metrics.active_connections) as f32
            + (selected_server.max_regions - selected_server.metrics.active_regions) as f32;

        let mut factors = HashMap::new();
        factors.insert("available_capacity".to_string(), available_capacity);
        factors.insert(
            "active_connections".to_string(),
            selected_server.metrics.active_connections as f32,
        );
        factors.insert(
            "active_regions".to_string(),
            selected_server.metrics.active_regions as f32,
        );

        Ok(LoadBalancingDecision {
            target_server_id: selected_server.id.clone(),
            strategy_used: LoadBalancingStrategy::Custom("capacity_aware".to_string()),
            decision_factors: factors,
            confidence_score: 0.88,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn ping_server(&self, server: &ServerInstance) -> Result<ServerHealth> {
        // Perform actual HTTP health check
        let health_url = format!("http://{}:{}/health", server.address, server.port);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        match client.get(&health_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Could parse response body for detailed health info
                    Ok(ServerHealth::Healthy)
                } else {
                    warn!(
                        "Server {} returned non-success status: {}",
                        server.id,
                        response.status()
                    );
                    Ok(ServerHealth::Degraded)
                }
            }
            Err(e) => {
                warn!("Failed to ping server {}: {}", server.id, e);
                Ok(ServerHealth::Unhealthy)
            }
        }
    }
}

impl Clone for LoadBalancer {
    fn clone(&self) -> Self {
        Self {
            strategy: self.strategy.clone(),
            servers: self.servers.clone(),
            region_assignments: self.region_assignments.clone(),
            round_robin_index: self.round_robin_index.clone(),
            health_check_interval: self.health_check_interval,
            rebalance_threshold: self.rebalance_threshold,
            region_manager: self.region_manager.clone(),
        }
    }
}

/// Load balancing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStatistics {
    pub total_servers: usize,
    pub healthy_servers: usize,
    pub total_regions: usize,
    pub total_connections: u32,
    pub average_cpu_usage: f32,
    pub average_memory_usage: f32,
    pub server_loads: HashMap<String, f32>,
    pub strategy: LoadBalancingStrategy,
    pub last_rebalance: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::physics::PhysicsBridge;
    use crate::state::StateManager;

    fn create_test_server(id: &str, cpu: f32, memory: f32, connections: u32) -> ServerInstance {
        ServerInstance {
            id: id.to_string(),
            address: "127.0.0.1".to_string(),
            port: 9000,
            weight: 1.0,
            max_regions: 10,
            max_connections: 1000,
            geographic_location: None,
            capabilities: vec!["regions".to_string()],
            metrics: ServerMetrics {
                server_id: id.to_string(),
                cpu_usage: cpu,
                memory_usage: memory,
                active_connections: connections,
                active_regions: 0,
                active_avatars: 0,
                bandwidth_usage: 0,
                latency_ms: 10.0,
                health_status: ServerHealth::Healthy,
                last_updated: chrono::Utc::now(),
            },
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_load_balancer_creation() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager));

        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin, region_manager);

        let stats = lb.get_statistics().await?;
        assert_eq!(stats.total_servers, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_server_registration() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager));

        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin, region_manager);

        let server = create_test_server("test-server-1", 50.0, 60.0, 100);
        lb.register_server(server).await?;

        let stats = lb.get_statistics().await?;
        assert_eq!(stats.total_servers, 1);
        assert_eq!(stats.healthy_servers, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_load_balancing_strategies() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager));

        let lb = LoadBalancer::new(LoadBalancingStrategy::LeastConnections, region_manager);

        // Register servers with different loads
        let server1 = create_test_server("server-1", 30.0, 40.0, 50);
        let server2 = create_test_server("server-2", 60.0, 70.0, 200);
        let server3 = create_test_server("server-3", 20.0, 30.0, 25);

        lb.register_server(server1).await?;
        lb.register_server(server2).await?;
        lb.register_server(server3).await?;

        // Test least connections strategy
        let region_id = Uuid::new_v4();
        let decision = lb.select_server_for_region(region_id).await?;

        // Should select server-3 (least connections: 25)
        assert_eq!(decision.target_server_id, "server-3");

        Ok(())
    }
}
