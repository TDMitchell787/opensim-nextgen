//! Monitoring and metrics collection for OpenSim
//!
//! This module provides comprehensive monitoring, metrics collection,
//! performance tracking, and health checks for the OpenSim server.

pub mod health;
pub mod metrics;
pub mod profiling;

use anyhow::Result;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{error, info};

// Re-export commonly used types
pub use self::health::HealthChecker;
pub use self::metrics::MetricsCollector;
pub use self::profiling::{PerformanceEvent, PerformanceProfile, PerformanceProfiler};

/// Main monitoring system for OpenSim
pub struct MonitoringSystem {
    /// Metrics collector for performance data
    metrics: Arc<MetricsCollector>,
    /// Health checker for system status
    health: Arc<HealthChecker>,
    /// Performance profiler for detailed analysis
    profiler: Arc<PerformanceProfiler>,
    /// System start time
    start_time: Instant,
    /// Configuration
    config: MonitoringConfig,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval: u64,
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Performance profiling sample rate (0.0 to 1.0)
    pub profiling_sample_rate: f64,
    /// Maximum number of metrics to retain
    pub max_metrics_history: usize,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_health_checks: true,
            enable_profiling: false,
            metrics_interval: 60,       // 1 minute
            health_check_interval: 30,  // 30 seconds
            profiling_sample_rate: 0.1, // 10% sampling
            max_metrics_history: 1000,
        }
    }
}

/// System health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System has warnings
    Warning,
    /// System is unhealthy
    Unhealthy,
    /// System status is unknown
    Unknown,
}

/// System metrics snapshot
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Network connections
    pub network_connections: usize,
    /// Active regions
    pub active_regions: usize,
    /// Physics simulation rate
    pub physics_fps: f64,
    /// Asset cache hit rate
    pub asset_cache_hit_rate: f64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Error rate
    pub error_rate: f64,
    /// System uptime
    pub uptime: Duration,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new(config: MonitoringConfig) -> Result<Self> {
        Ok(Self {
            metrics: Arc::new(MetricsCollector::new(config.max_metrics_history)?),
            health: Arc::new(HealthChecker::new()?),
            profiler: Arc::new(PerformanceProfiler::new(config.profiling_sample_rate)?),
            start_time: Instant::now(),
            config,
        })
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<()> {
        info!("Starting monitoring system");

        if self.config.enable_metrics {
            self.start_metrics_collection().await?;
        }

        if self.config.enable_health_checks {
            self.start_health_checks().await?;
        }

        if self.config.enable_profiling {
            self.start_profiling().await?;
        }

        info!("Monitoring system started successfully");
        Ok(())
    }

    /// Start metrics collection
    async fn start_metrics_collection(&self) -> Result<()> {
        let metrics = self.metrics.clone();
        let interval = Duration::from_secs(self.config.metrics_interval);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if let Err(e) = metrics.collect_system_metrics().await {
                    error!("Failed to collect metrics: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Start health checks
    async fn start_health_checks(&self) -> Result<()> {
        let health = self.health.clone();
        let interval = Duration::from_secs(self.config.health_check_interval);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if let Err(e) = health.run_health_checks().await {
                    error!("Health check failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Start performance profiling
    async fn start_profiling(&self) -> Result<()> {
        let profiler = self.profiler.clone();

        tokio::spawn(async move {
            if let Err(e) = profiler.start_profiling().await {
                error!("Failed to start profiling: {}", e);
            }
        });

        Ok(())
    }

    /// Get current system metrics
    pub async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        let mut metrics = self.metrics.get_current_metrics().await?;
        // Override with real uptime from server start time
        metrics.uptime = self.get_uptime();
        // TODO: Override with real region count and connection count from region manager
        Ok(metrics)
    }

    /// Get the metrics collector
    pub fn get_metrics_collector(&self) -> Arc<MetricsCollector> {
        self.metrics.clone()
    }

    /// Get system health status
    pub async fn get_health_status(&self) -> Result<HealthStatus> {
        self.health.get_overall_status().await
    }

    /// Get performance profile
    pub async fn get_performance_profile(&self) -> Result<Option<profiling::PerformanceProfile>> {
        self.profiler.get_profile("overall").await
    }

    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub async fn record_client_connection(&self) {
        // TODO: Implement metric collection
    }

    pub async fn record_client_disconnection(&self) {
        // TODO: Implement metric collection
    }

    /// Record a custom metric
    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        tags: HashMap<String, String>,
    ) -> Result<()> {
        self.metrics.record_custom_metric(name, value, tags).await
    }

    /// Record a performance event
    pub async fn record_performance_event(&self, event: profiling::PerformanceEvent) -> Result<()> {
        self.profiler.record_event(event).await
    }

    /// Get monitoring statistics
    pub async fn get_stats(&self) -> Result<MonitoringStats> {
        let metrics_count = self.metrics.get_metrics_count().await;
        let health_status = self.health.get_overall_status().await?;
        let profiling_enabled = self.config.enable_profiling;

        Ok(MonitoringStats {
            metrics_count,
            health_status,
            profiling_enabled,
            uptime: self.start_time.elapsed(),
        })
    }

    /// Stop the monitoring system
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping monitoring system");

        if self.config.enable_profiling {
            self.profiler.stop_profiling().await?;
        }

        info!("Monitoring system stopped");
        Ok(())
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> Result<String> {
        self.metrics.export_prometheus().await
    }
}

/// Monitoring statistics
#[derive(Debug, Clone)]
pub struct MonitoringStats {
    /// Number of metrics collected
    pub metrics_count: usize,
    /// Current health status
    pub health_status: HealthStatus,
    /// Whether profiling is enabled
    pub profiling_enabled: bool,
    /// System uptime
    pub uptime: Duration,
}

impl Default for MonitoringSystem {
    fn default() -> Self {
        Self::new(MonitoringConfig::default()).expect("Failed to create MonitoringSystem")
    }
}
