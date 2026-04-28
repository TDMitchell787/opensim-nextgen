//! Instance Registry
//!
//! Tracks all known instances and their current status.
//! Provides a central point for instance discovery and lookup.

use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::config_loader::{InstanceConfig, InstancesConfig};
use super::types::{Environment, HealthStatus, InstanceInfo, InstanceMetrics, InstanceStatus};

/// Central registry for tracking all known instances
pub struct InstanceRegistry {
    instances: Arc<RwLock<HashMap<String, RegisteredInstance>>>,
    config: InstancesConfig,
}

/// Internal representation of a registered instance
#[derive(Debug, Clone)]
pub struct RegisteredInstance {
    pub config: InstanceConfig,
    pub info: InstanceInfo,
    pub connection_attempts: u32,
    pub last_error: Option<String>,
}

impl InstanceRegistry {
    /// Create a new instance registry from configuration
    pub fn new(config: InstancesConfig) -> Self {
        let mut instances = HashMap::new();

        for instance_config in &config.instances {
            let info = InstanceInfo {
                id: instance_config.id.clone(),
                name: instance_config.name.clone(),
                description: instance_config.description.clone(),
                host: instance_config.get_host(),
                environment: instance_config.environment,
                status: InstanceStatus::Unknown,
                metrics: None,
                health: None,
                version: None,
                last_seen: Utc::now(),
                connected: false,
                tags: instance_config.tags.clone(),
            };

            instances.insert(
                instance_config.id.clone(),
                RegisteredInstance {
                    config: instance_config.clone(),
                    info,
                    connection_attempts: 0,
                    last_error: None,
                },
            );
        }

        info!(
            "Instance registry initialized with {} instance(s)",
            instances.len()
        );

        Self {
            instances: Arc::new(RwLock::new(instances)),
            config,
        }
    }

    /// Get the controller configuration
    pub fn controller_config(&self) -> &super::config_loader::ControllerConfig {
        &self.config.controller
    }

    /// Get all registered instance IDs
    pub async fn get_instance_ids(&self) -> Vec<String> {
        let instances = self.instances.read().await;
        instances.keys().cloned().collect()
    }

    /// Get all registered instances
    pub async fn get_all_instances(&self) -> Vec<InstanceInfo> {
        let instances = self.instances.read().await;
        instances.values().map(|r| r.info.clone()).collect()
    }

    /// Get instances that should auto-connect
    pub async fn get_auto_connect_instances(&self) -> Vec<String> {
        let instances = self.instances.read().await;
        instances
            .iter()
            .filter(|(_, r)| r.config.auto_connect)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get instance info by ID
    pub async fn get_instance(&self, id: &str) -> Option<InstanceInfo> {
        let instances = self.instances.read().await;
        instances.get(id).map(|r| r.info.clone())
    }

    /// Get instance configuration by ID
    pub async fn get_instance_config(&self, id: &str) -> Option<InstanceConfig> {
        let instances = self.instances.read().await;
        instances.get(id).map(|r| r.config.clone())
    }

    /// Check if an instance exists
    pub async fn has_instance(&self, id: &str) -> bool {
        let instances = self.instances.read().await;
        instances.contains_key(id)
    }

    /// Update instance status
    pub async fn update_status(&self, id: &str, status: InstanceStatus) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(id)
            .ok_or_else(|| anyhow!("Instance not found: {}", id))?;

        instance.info.status = status;
        instance.info.last_seen = Utc::now();

        debug!("Instance {} status updated to {:?}", id, status);
        Ok(())
    }

    /// Update instance connection state
    pub async fn update_connected(&self, id: &str, connected: bool) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(id)
            .ok_or_else(|| anyhow!("Instance not found: {}", id))?;

        instance.info.connected = connected;
        instance.info.last_seen = Utc::now();

        if connected {
            instance.connection_attempts = 0;
            instance.last_error = None;
            instance.info.status = InstanceStatus::Running;
        } else {
            instance.info.status = InstanceStatus::Disconnected;
        }

        info!(
            "Instance {} connection state: {}",
            id,
            if connected {
                "connected"
            } else {
                "disconnected"
            }
        );
        Ok(())
    }

    /// Update instance metrics
    pub async fn update_metrics(&self, id: &str, metrics: InstanceMetrics) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(id)
            .ok_or_else(|| anyhow!("Instance not found: {}", id))?;

        instance.info.metrics = Some(metrics);
        instance.info.last_seen = Utc::now();

        debug!("Instance {} metrics updated", id);
        Ok(())
    }

    /// Update instance health status
    pub async fn update_health(&self, id: &str, health: HealthStatus) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(id)
            .ok_or_else(|| anyhow!("Instance not found: {}", id))?;

        instance.info.health = Some(health);
        instance.info.last_seen = Utc::now();

        debug!("Instance {} health updated", id);
        Ok(())
    }

    /// Update instance version
    pub async fn update_version(&self, id: &str, version: String) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(id)
            .ok_or_else(|| anyhow!("Instance not found: {}", id))?;

        instance.info.version = Some(version);
        Ok(())
    }

    /// Record a connection attempt failure
    pub async fn record_connection_failure(&self, id: &str, error: &str) -> Result<u32> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(id)
            .ok_or_else(|| anyhow!("Instance not found: {}", id))?;

        instance.connection_attempts += 1;
        instance.last_error = Some(error.to_string());
        instance.info.status = InstanceStatus::Error;

        warn!(
            "Instance {} connection attempt {} failed: {}",
            id, instance.connection_attempts, error
        );

        Ok(instance.connection_attempts)
    }

    /// Get connection attempt count for an instance
    pub async fn get_connection_attempts(&self, id: &str) -> Option<u32> {
        let instances = self.instances.read().await;
        instances.get(id).map(|r| r.connection_attempts)
    }

    /// Reset connection attempts for an instance
    pub async fn reset_connection_attempts(&self, id: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(id)
            .ok_or_else(|| anyhow!("Instance not found: {}", id))?;

        instance.connection_attempts = 0;
        instance.last_error = None;
        Ok(())
    }

    /// Add a new instance at runtime
    pub async fn add_instance(&self, config: InstanceConfig) -> Result<()> {
        let mut instances = self.instances.write().await;

        if instances.contains_key(&config.id) {
            return Err(anyhow!("Instance already exists: {}", config.id));
        }

        let info = InstanceInfo {
            id: config.id.clone(),
            name: config.name.clone(),
            description: config.description.clone(),
            host: config.get_host(),
            environment: config.environment,
            status: InstanceStatus::Unknown,
            metrics: None,
            health: None,
            version: None,
            last_seen: Utc::now(),
            connected: false,
            tags: config.tags.clone(),
        };

        info!("Adding new instance: {} ({})", config.name, config.id);

        instances.insert(
            config.id.clone(),
            RegisteredInstance {
                config,
                info,
                connection_attempts: 0,
                last_error: None,
            },
        );

        Ok(())
    }

    /// Remove an instance at runtime
    pub async fn remove_instance(&self, id: &str) -> Result<InstanceInfo> {
        let mut instances = self.instances.write().await;

        let removed = instances
            .remove(id)
            .ok_or_else(|| anyhow!("Instance not found: {}", id))?;

        info!("Removed instance: {} ({})", removed.info.name, id);
        Ok(removed.info)
    }

    /// Get instances filtered by environment
    pub async fn get_instances_by_environment(&self, env: Environment) -> Vec<InstanceInfo> {
        let instances = self.instances.read().await;
        instances
            .values()
            .filter(|r| r.info.environment == env)
            .map(|r| r.info.clone())
            .collect()
    }

    /// Get instances filtered by tag
    pub async fn get_instances_by_tag(&self, tag: &str) -> Vec<InstanceInfo> {
        let instances = self.instances.read().await;
        instances
            .values()
            .filter(|r| r.info.tags.contains(&tag.to_string()))
            .map(|r| r.info.clone())
            .collect()
    }

    /// Get all connected instances
    pub async fn get_connected_instances(&self) -> Vec<InstanceInfo> {
        let instances = self.instances.read().await;
        instances
            .values()
            .filter(|r| r.info.connected)
            .map(|r| r.info.clone())
            .collect()
    }

    /// Get instance count
    pub async fn count(&self) -> usize {
        let instances = self.instances.read().await;
        instances.len()
    }

    /// Get connected instance count
    pub async fn connected_count(&self) -> usize {
        let instances = self.instances.read().await;
        instances.values().filter(|r| r.info.connected).count()
    }
}

#[cfg(test)]
mod tests {
    use super::super::config_loader::ControllerConfig;
    use super::*;

    fn create_test_config() -> InstancesConfig {
        InstancesConfig {
            controller: ControllerConfig::default(),
            instances: vec![InstanceConfig {
                id: "test-1".to_string(),
                name: "Test Instance 1".to_string(),
                description: "Test".to_string(),
                host: "localhost".to_string(),
                websocket_port: 9001,
                admin_port: 9200,
                metrics_port: 9100,
                http_port: 9000,
                udp_port: 9000,
                api_key: "test-key".to_string(),
                environment: Environment::Development,
                auto_connect: true,
                tags: vec!["test".to_string()],
                authentication: Default::default(),
                tls: Default::default(),
            }],
        }
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let config = create_test_config();
        let registry = InstanceRegistry::new(config);

        assert_eq!(registry.count().await, 1);
        assert!(registry.has_instance("test-1").await);
    }

    #[tokio::test]
    async fn test_update_status() {
        let config = create_test_config();
        let registry = InstanceRegistry::new(config);

        registry
            .update_status("test-1", InstanceStatus::Running)
            .await
            .unwrap();

        let instance = registry.get_instance("test-1").await.unwrap();
        assert_eq!(instance.status, InstanceStatus::Running);
    }

    #[tokio::test]
    async fn test_connection_tracking() {
        let config = create_test_config();
        let registry = InstanceRegistry::new(config);

        registry
            .record_connection_failure("test-1", "Test error")
            .await
            .unwrap();
        assert_eq!(registry.get_connection_attempts("test-1").await, Some(1));

        registry.update_connected("test-1", true).await.unwrap();
        assert_eq!(registry.get_connection_attempts("test-1").await, Some(0));
    }
}
