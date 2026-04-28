//! Health checking for OpenSim
//!
//! Monitors system components and provides health status information.

use anyhow::Result;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

use super::HealthStatus;

/// Health checker for monitoring system components
pub struct HealthChecker {
    /// Health checks for different components
    health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    /// Overall health status
    overall_status: Arc<RwLock<HealthStatus>>,
    /// Last health check time
    last_check: Arc<RwLock<Instant>>,
}

/// Individual health check
pub struct HealthCheck {
    /// Component name
    pub name: String,
    /// Current status
    pub status: HealthStatus,
    /// Last check time
    pub last_check: Instant,
    /// Check interval
    pub interval: Duration,
    /// Error message if unhealthy
    pub error_message: Option<String>,
    /// Check function
    pub check_fn: HealthCheckFunction,
}

/// Health check function type
pub type HealthCheckFunction = Box<dyn Fn() -> Result<HealthStatus> + Send + Sync>;

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Result<Self> {
        Ok(Self {
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            overall_status: Arc::new(RwLock::new(HealthStatus::Unknown)),
            last_check: Arc::new(RwLock::new(Instant::now())),
        })
    }

    /// Add a health check for a component
    pub async fn add_health_check(
        &self,
        name: &str,
        interval: Duration,
        check_fn: HealthCheckFunction,
    ) -> Result<()> {
        let health_check = HealthCheck {
            name: name.to_string(),
            status: HealthStatus::Unknown,
            last_check: Instant::now(),
            interval,
            error_message: None,
            check_fn,
        };

        self.health_checks
            .write()
            .await
            .insert(name.to_string(), health_check);
        debug!("Added health check for component: {}", name);
        Ok(())
    }

    /// Run all health checks
    pub async fn run_health_checks(&self) -> Result<()> {
        let mut checks = self.health_checks.write().await;
        let now = Instant::now();
        let mut overall_status = HealthStatus::Healthy;

        for (name, check) in checks.iter_mut() {
            // Check if it's time to run this health check
            if now.duration_since(check.last_check) >= check.interval {
                match (check.check_fn)() {
                    Ok(status) => {
                        check.status = status;
                        check.error_message = None;
                        debug!("Health check for {}: {:?}", name, check.status);
                    }
                    Err(e) => {
                        check.status = HealthStatus::Unhealthy;
                        check.error_message = Some(e.to_string());
                        error!("Health check for {} failed: {}", name, e);
                    }
                }
                check.last_check = now;
            }

            // Update overall status
            match check.status {
                HealthStatus::Unhealthy => {
                    overall_status = HealthStatus::Unhealthy;
                }
                HealthStatus::Warning => {
                    if overall_status != HealthStatus::Unhealthy {
                        overall_status = HealthStatus::Warning;
                    }
                }
                _ => {}
            }
        }

        *self.overall_status.write().await = overall_status.clone();
        *self.last_check.write().await = now;

        debug!(
            "Health checks completed, overall status: {:?}",
            overall_status
        );
        Ok(())
    }

    /// Get overall health status
    pub async fn get_overall_status(&self) -> Result<HealthStatus> {
        Ok(self.overall_status.read().await.clone())
    }

    /// Get health status for a specific component
    pub async fn get_component_status(&self, component_name: &str) -> Result<Option<HealthStatus>> {
        let checks = self.health_checks.read().await;
        Ok(checks.get(component_name).map(|check| check.status.clone()))
    }

    /// Get detailed health information
    pub async fn get_health_details(&self) -> Result<HealthDetails> {
        let checks = self.health_checks.read().await;
        let overall_status = self.overall_status.read().await.clone();
        let last_check = *self.last_check.read().await;

        let mut components = Vec::new();
        for (name, check) in checks.iter() {
            components.push(ComponentHealth {
                name: name.clone(),
                status: check.status.clone(),
                last_check: check.last_check,
                error_message: check.error_message.clone(),
            });
        }

        Ok(HealthDetails {
            overall_status,
            last_check,
            components,
        })
    }

    /// Remove a health check
    pub async fn remove_health_check(&self, component_name: &str) -> Result<()> {
        self.health_checks.write().await.remove(component_name);
        debug!("Removed health check for component: {}", component_name);
        Ok(())
    }

    /// Force a health check for a specific component
    pub async fn force_health_check(&self, component_name: &str) -> Result<()> {
        let mut checks = self.health_checks.write().await;

        if let Some(check) = checks.get_mut(component_name) {
            match (check.check_fn)() {
                Ok(status) => {
                    check.status = status;
                    check.error_message = None;
                    check.last_check = Instant::now();
                    debug!(
                        "Forced health check for {}: {:?}",
                        component_name, check.status
                    );
                }
                Err(e) => {
                    check.status = HealthStatus::Unhealthy;
                    check.error_message = Some(e.to_string());
                    check.last_check = Instant::now();
                    error!("Forced health check for {} failed: {}", component_name, e);
                }
            }
        } else {
            warn!("Component {} not found for health check", component_name);
        }

        Ok(())
    }
}

/// Detailed health information
#[derive(Debug, Clone)]
pub struct HealthDetails {
    /// Overall system health status
    pub overall_status: HealthStatus,
    /// Last time health checks were run
    pub last_check: Instant,
    /// Health status of individual components
    pub components: Vec<ComponentHealth>,
}

/// Component health information
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Last check time
    pub last_check: Instant,
    /// Error message if unhealthy
    pub error_message: Option<String>,
}

impl Default for HealthChecker {
    fn default() -> Self {
        // If health checker creation fails, create a minimal fallback
        match Self::new() {
            Ok(checker) => checker,
            Err(e) => {
                tracing::error!(
                    "Failed to create HealthChecker: {}. Using fallback configuration.",
                    e
                );
                // Create a basic health checker that always reports degraded status
                HealthChecker {
                    health_checks: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    last_check: Arc::new(RwLock::new(std::time::Instant::now())),
                    overall_status: Arc::new(RwLock::new(HealthStatus::Warning)),
                }
            }
        }
    }
}
