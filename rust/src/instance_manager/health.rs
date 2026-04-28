//! Health Checker
//!
//! Monitors the health and connectivity of all registered instances.
//! Runs periodic health checks and broadcasts status updates.

use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use super::registry::InstanceRegistry;
use super::types::{ComponentHealth, HealthState, HealthStatus, InstanceMetrics, InstanceStatus};

/// Health checker for monitoring instance health
pub struct HealthChecker {
    registry: Arc<InstanceRegistry>,
    http_client: Client,
    check_interval: Duration,
    timeout: Duration,
    event_tx: broadcast::Sender<HealthEvent>,
}

/// Events emitted by the health checker
#[derive(Debug, Clone)]
pub enum HealthEvent {
    HealthCheckStarted {
        instance_id: String,
    },
    HealthCheckCompleted {
        instance_id: String,
        status: HealthStatus,
        duration_ms: u64,
    },
    InstanceHealthChanged {
        instance_id: String,
        old_state: HealthState,
        new_state: HealthState,
    },
    InstanceUnreachable {
        instance_id: String,
        error: String,
    },
    MetricsUpdated {
        instance_id: String,
        metrics: InstanceMetrics,
    },
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(registry: Arc<InstanceRegistry>) -> Self {
        let config = registry.controller_config();

        let check_interval = Duration::from_millis(config.health_check_interval_ms);
        let timeout = Duration::from_millis(config.heartbeat_timeout_ms);

        let http_client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        let (event_tx, _) = broadcast::channel(1000);

        Self {
            registry,
            http_client,
            check_interval,
            timeout,
            event_tx,
        }
    }

    /// Subscribe to health events
    pub fn subscribe(&self) -> broadcast::Receiver<HealthEvent> {
        self.event_tx.subscribe()
    }

    /// Start the health check loop
    pub async fn start(&self) {
        info!(
            "Starting health checker with {}ms interval",
            self.check_interval.as_millis()
        );

        let mut ticker = interval(self.check_interval);

        loop {
            ticker.tick().await;
            self.check_all_instances().await;
        }
    }

    /// Run a single health check cycle for all instances
    pub async fn check_all_instances(&self) {
        let instance_ids = self.registry.get_instance_ids().await;

        debug!(
            "Running health checks for {} instance(s)",
            instance_ids.len()
        );

        for instance_id in instance_ids {
            if let Err(e) = self.check_instance(&instance_id).await {
                error!("Health check failed for {}: {}", instance_id, e);
            }
        }
    }

    /// Check health of a single instance
    pub async fn check_instance(&self, instance_id: &str) -> Result<HealthStatus> {
        let config = match self.registry.get_instance_config(instance_id).await {
            Some(c) => c,
            None => {
                warn!("Instance not found for health check: {}", instance_id);
                return Ok(HealthStatus::default());
            }
        };

        let current_health = self
            .registry
            .get_instance(instance_id)
            .await
            .and_then(|i| i.health)
            .map(|h| h.overall)
            .unwrap_or(HealthState::Unknown);

        let _ = self.event_tx.send(HealthEvent::HealthCheckStarted {
            instance_id: instance_id.to_string(),
        });

        let start = Instant::now();
        let mut components = HashMap::new();

        // Check health endpoint
        let health_result = self.check_health_endpoint(&config).await;
        let overall_state = match &health_result {
            Ok(state) => {
                components.insert(
                    "api".to_string(),
                    ComponentHealth {
                        name: "API Server".to_string(),
                        status: HealthState::Healthy,
                        message: None,
                        response_time_ms: Some(start.elapsed().as_millis() as u64),
                    },
                );
                *state
            }
            Err(e) => {
                let _ = self.event_tx.send(HealthEvent::InstanceUnreachable {
                    instance_id: instance_id.to_string(),
                    error: e.to_string(),
                });

                components.insert(
                    "api".to_string(),
                    ComponentHealth {
                        name: "API Server".to_string(),
                        status: HealthState::Unhealthy,
                        message: Some(e.to_string()),
                        response_time_ms: None,
                    },
                );

                self.registry
                    .update_status(instance_id, InstanceStatus::Error)
                    .await
                    .ok();

                HealthState::Unhealthy
            }
        };

        // Check metrics endpoint
        if overall_state == HealthState::Healthy {
            if let Ok(metrics) = self.fetch_metrics(&config).await {
                let _ = self.event_tx.send(HealthEvent::MetricsUpdated {
                    instance_id: instance_id.to_string(),
                    metrics: metrics.clone(),
                });

                self.registry
                    .update_metrics(instance_id, metrics)
                    .await
                    .ok();

                components.insert(
                    "metrics".to_string(),
                    ComponentHealth {
                        name: "Metrics".to_string(),
                        status: HealthState::Healthy,
                        message: None,
                        response_time_ms: None,
                    },
                );
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        let health_status = HealthStatus {
            overall: overall_state,
            components,
            last_check: chrono::Utc::now(),
            response_time_ms: duration_ms,
        };

        self.registry
            .update_health(instance_id, health_status.clone())
            .await
            .ok();

        // Emit health changed event if state changed
        if current_health != overall_state {
            let _ = self.event_tx.send(HealthEvent::InstanceHealthChanged {
                instance_id: instance_id.to_string(),
                old_state: current_health,
                new_state: overall_state,
            });

            match overall_state {
                HealthState::Healthy => {
                    self.registry
                        .update_status(instance_id, InstanceStatus::Running)
                        .await
                        .ok();
                }
                HealthState::Degraded => {
                    warn!("Instance {} health degraded", instance_id);
                }
                HealthState::Unhealthy => {
                    warn!("Instance {} is unhealthy", instance_id);
                    self.registry
                        .update_status(instance_id, InstanceStatus::Error)
                        .await
                        .ok();
                }
                HealthState::Unknown => {}
            }
        }

        let _ = self.event_tx.send(HealthEvent::HealthCheckCompleted {
            instance_id: instance_id.to_string(),
            status: health_status.clone(),
            duration_ms,
        });

        Ok(health_status)
    }

    /// Check the health endpoint
    async fn check_health_endpoint(
        &self,
        config: &super::config_loader::InstanceConfig,
    ) -> Result<HealthState> {
        let url = format!("{}/health", config.admin_url());

        let response = self
            .http_client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await?;

        if response.status().is_success() {
            let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));

            // Parse health status from response
            let status = body
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("healthy");

            match status.to_lowercase().as_str() {
                "healthy" | "ok" | "up" => Ok(HealthState::Healthy),
                "degraded" | "warning" => Ok(HealthState::Degraded),
                "unhealthy" | "down" | "error" => Ok(HealthState::Unhealthy),
                _ => Ok(HealthState::Unknown),
            }
        } else {
            Ok(HealthState::Unhealthy)
        }
    }

    /// Fetch metrics from the metrics endpoint
    async fn fetch_metrics(
        &self,
        config: &super::config_loader::InstanceConfig,
    ) -> Result<InstanceMetrics> {
        let url = format!("{}/api/stats", config.admin_url());

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.get_api_key()))
            .timeout(self.timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(InstanceMetrics::default());
        }

        let body: serde_json::Value = response.json().await?;

        Ok(InstanceMetrics {
            cpu_usage: body
                .get("cpu_usage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            memory_usage_mb: body
                .get("memory_usage_mb")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            memory_total_mb: body
                .get("memory_total_mb")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            active_users: body
                .get("active_users")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            active_regions: body
                .get("active_regions")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            network_tx_bytes: body
                .get("network_tx_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            network_rx_bytes: body
                .get("network_rx_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            db_connections: body
                .get("db_connections")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            websocket_connections: body
                .get("websocket_connections")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            request_rate_per_sec: body
                .get("request_rate_per_sec")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            error_rate_per_sec: body
                .get("error_rate_per_sec")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            uptime_seconds: body
                .get("uptime_seconds")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::config_loader::{ControllerConfig, InstanceConfig, InstancesConfig};
    use super::super::types::Environment;
    use super::*;

    fn create_test_registry() -> Arc<InstanceRegistry> {
        let config = InstancesConfig {
            controller: ControllerConfig {
                health_check_interval_ms: 1000,
                heartbeat_timeout_ms: 5000,
                ..Default::default()
            },
            instances: vec![InstanceConfig {
                id: "test-1".to_string(),
                name: "Test Instance".to_string(),
                description: "".to_string(),
                host: "localhost".to_string(),
                websocket_port: 9001,
                admin_port: 9200,
                metrics_port: 9100,
                http_port: 9000,
                udp_port: 9000,
                api_key: "test-key".to_string(),
                environment: Environment::Development,
                auto_connect: true,
                tags: vec![],
                authentication: Default::default(),
                tls: Default::default(),
            }],
        };

        Arc::new(InstanceRegistry::new(config))
    }

    #[tokio::test]
    async fn test_health_checker_creation() {
        let registry = create_test_registry();
        let checker = HealthChecker::new(registry);

        // Verify checker was created with correct interval
        assert_eq!(checker.check_interval.as_millis(), 1000);
    }
}
