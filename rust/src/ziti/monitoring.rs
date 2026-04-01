//! OpenZiti Advanced Monitoring and Analytics
//!
//! Provides comprehensive monitoring, analytics, and observability for zero trust networking.
//! Features real-time network analytics, security monitoring, performance analysis, and
//! business intelligence integration for enterprise-grade zero trust networks.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime};
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use tokio::time;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use super::config::ZitiConfig;

/// Network connection analytics data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    pub connection_id: String,
    pub service_name: String,
    pub identity_name: String,
    pub started_at: SystemTime,
    pub last_activity: SystemTime,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub latency_samples: VecDeque<Duration>,
    pub security_violations: u32,
    pub policy_checks: u32,
    pub error_count: u32,
}

/// Security analytics and threat detection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityAnalytics {
    pub identity_id: String,
    pub threat_score: f64,
    pub anomaly_score: f64,
    pub violation_count: u32,
    pub last_violation: Option<SystemTime>,
    pub connection_patterns: Vec<String>,
    pub suspicious_activities: Vec<String>,
}

/// Network performance analytics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_latency: Duration,
    pub peak_latency: Duration,
    pub throughput_bps: f64,
    pub connection_success_rate: f64,
    pub service_availability: f64,
    pub resource_utilization: f64,
}

/// Real-time network analytics engine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkAnalytics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_data_transferred: u64,
    pub services_count: u32,
    pub identities_count: u32,
    pub policy_evaluations: u64,
    pub security_events: u64,
    pub performance_score: f64,
}

/// Advanced alerting system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub condition: String,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub last_triggered: Option<SystemTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Advanced monitoring system for OpenZiti networks
pub struct ZitiAdvancedMonitoring {
    config: ZitiConfig,
    
    // Core metrics storage
    connection_metrics: Arc<RwLock<HashMap<String, ConnectionMetrics>>>,
    security_analytics: Arc<RwLock<HashMap<String, SecurityAnalytics>>>,
    performance_history: Arc<RwLock<VecDeque<PerformanceMetrics>>>,
    
    // Analytics engines
    network_analytics: Arc<RwLock<NetworkAnalytics>>,
    alert_rules: Arc<RwLock<HashMap<Uuid, AlertRule>>>,
    
    // System state
    is_initialized: bool,
    monitoring_interval: Duration,
    retention_period: Duration,
}

impl ZitiAdvancedMonitoring {
    pub fn new(config: &ZitiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            connection_metrics: Arc::new(RwLock::new(HashMap::new())),
            security_analytics: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            network_analytics: Arc::new(RwLock::new(NetworkAnalytics {
                total_connections: 0,
                active_connections: 0,
                total_data_transferred: 0,
                services_count: 0,
                identities_count: 0,
                policy_evaluations: 0,
                security_events: 0,
                performance_score: 100.0,
            })),
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            is_initialized: false,
            monitoring_interval: Duration::from_secs(30),
            retention_period: Duration::from_secs(86400 * 7), // 7 days
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.is_initialized = true;
        self.setup_default_alert_rules().await?;
        tracing::info!("OpenZiti advanced monitoring initialized with analytics engine");
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        if !self.is_initialized {
            return Err(anyhow!("Monitoring not initialized".to_string()));
        }

        // Start background analytics processing
        let analytics = Arc::clone(&self.network_analytics);
        let connections = Arc::clone(&self.connection_metrics);
        let security = Arc::clone(&self.security_analytics);
        let performance = Arc::clone(&self.performance_history);
        let interval = self.monitoring_interval;

        tokio::spawn(async move {
            let mut interval_timer = time::interval(interval);
            loop {
                interval_timer.tick().await;
                if let Err(e) = Self::process_analytics(
                    &analytics, &connections, &security, &performance
                ).await {
                    tracing::error!("Analytics processing error: {}", e);
                }
            }
        });

        tracing::info!("OpenZiti advanced monitoring started with real-time analytics");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("OpenZiti advanced monitoring stopped");
        Ok(())
    }

    /// Record new connection with comprehensive metrics
    pub async fn record_connection(
        &mut self, 
        connection_id: &str,
        service_name: &str,
        identity_name: &str
    ) -> Result<()> {
        let metrics = ConnectionMetrics {
            connection_id: connection_id.to_string(),
            service_name: service_name.to_string(),
            identity_name: identity_name.to_string(),
            started_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            bytes_sent: 0,
            bytes_received: 0,
            latency_samples: VecDeque::new(),
            security_violations: 0,
            policy_checks: 0,
            error_count: 0,
        };

        self.connection_metrics.write().unwrap().insert(connection_id.to_string(), metrics);
        
        // Update network analytics
        {
            let mut analytics = self.network_analytics.write().unwrap();
            analytics.total_connections += 1;
            analytics.active_connections += 1;
        }

        tracing::debug!("Recorded new connection: {} for service: {}", connection_id, service_name);
        Ok(())
    }

    /// Record connection closure with analytics
    pub async fn record_connection_closed(&mut self, connection_id: &str) -> Result<()> {
        if let Some(metrics) = self.connection_metrics.write().unwrap().remove(connection_id) {
            // Update network analytics
            {
                let mut analytics = self.network_analytics.write().unwrap();
                analytics.active_connections = analytics.active_connections.saturating_sub(1);
                analytics.total_data_transferred += metrics.bytes_sent + metrics.bytes_received;
            }

            // Archive connection data for historical analysis
            self.archive_connection_metrics(&metrics).await?;
        }

        tracing::debug!("Recorded connection closure: {}", connection_id);
        Ok(())
    }

    /// Record data transmission with latency tracking
    pub async fn record_data_sent(
        &mut self, 
        connection_id: &str, 
        bytes: usize,
        latency: Option<Duration>
    ) -> Result<()> {
        if let Some(metrics) = self.connection_metrics.write().unwrap().get_mut(connection_id) {
            metrics.bytes_sent += bytes as u64;
            metrics.last_activity = SystemTime::now();
            
            if let Some(lat) = latency {
                metrics.latency_samples.push_back(lat);
                // Keep only recent samples for performance
                if metrics.latency_samples.len() > 100 {
                    metrics.latency_samples.pop_front();
                }
            }
        }
        Ok(())
    }

    /// Record data reception with analytics
    pub async fn record_data_received(
        &mut self, 
        connection_id: &str, 
        bytes: usize,
        latency: Option<Duration>
    ) -> Result<()> {
        if let Some(metrics) = self.connection_metrics.write().unwrap().get_mut(connection_id) {
            metrics.bytes_received += bytes as u64;
            metrics.last_activity = SystemTime::now();
            
            if let Some(lat) = latency {
                metrics.latency_samples.push_back(lat);
                if metrics.latency_samples.len() > 100 {
                    metrics.latency_samples.pop_front();
                }
            }
        }
        Ok(())
    }

    /// Record security event for threat analysis
    pub async fn record_security_event(
        &mut self,
        identity_id: &str,
        event_type: &str,
        severity: AlertSeverity
    ) -> Result<()> {
        {
            let mut analytics = self.network_analytics.write().unwrap();
            analytics.security_events += 1;
        }

        // Update security analytics
        let mut security_map = self.security_analytics.write().unwrap();
        let security_analytics = security_map.entry(identity_id.to_string()).or_insert_with(|| {
            SecurityAnalytics {
                identity_id: identity_id.to_string(),
                threat_score: 0.0,
                anomaly_score: 0.0,
                violation_count: 0,
                last_violation: None,
                connection_patterns: Vec::new(),
                suspicious_activities: Vec::new(),
            }
        });

        security_analytics.violation_count += 1;
        security_analytics.last_violation = Some(SystemTime::now());
        security_analytics.suspicious_activities.push(event_type.to_string());
        
        // Calculate threat score based on severity
        let threat_increase = match severity {
            AlertSeverity::Critical => 25.0,
            AlertSeverity::High => 15.0,
            AlertSeverity::Medium => 10.0,
            AlertSeverity::Low => 5.0,
            AlertSeverity::Info => 0.0,
        };
        
        security_analytics.threat_score = (security_analytics.threat_score + threat_increase).min(100.0);

        tracing::warn!("Security event recorded: {} for identity: {}", event_type, identity_id);
        Ok(())
    }

    /// Get comprehensive network analytics
    pub async fn get_network_analytics(&self) -> Result<NetworkAnalytics> {
        Ok(self.network_analytics.read().unwrap().clone())
    }

    /// Get security analytics for specific identity
    pub async fn get_security_analytics(&self, identity_id: &str) -> Result<Option<SecurityAnalytics>> {
        Ok(self.security_analytics.read().unwrap().get(identity_id).cloned())
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> Result<Option<PerformanceMetrics>> {
        Ok(self.performance_history.read().unwrap().back().cloned())
    }

    /// Get all connection metrics
    pub async fn get_connection_metrics(&self) -> Result<HashMap<String, ConnectionMetrics>> {
        Ok(self.connection_metrics.read().unwrap().clone())
    }

    /// Create custom alert rule
    pub async fn create_alert_rule(
        &mut self,
        name: String,
        condition: String,
        threshold: f64,
        severity: AlertSeverity
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let rule = AlertRule {
            id,
            name,
            condition,
            threshold,
            severity,
            enabled: true,
            last_triggered: None,
        };

        self.alert_rules.write().unwrap().insert(id, rule);
        tracing::info!("Created alert rule: {}", id);
        Ok(id)
    }

    /// Setup default alert rules
    async fn setup_default_alert_rules(&mut self) -> Result<()> {
        self.create_alert_rule(
            "High Connection Count".to_string(),
            "active_connections > threshold".to_string(),
            1000.0,
            AlertSeverity::High
        ).await?;

        self.create_alert_rule(
            "High Latency".to_string(),
            "average_latency > threshold".to_string(),
            1000.0, // 1 second in milliseconds
            AlertSeverity::Medium
        ).await?;

        self.create_alert_rule(
            "Security Threat Detected".to_string(),
            "threat_score > threshold".to_string(),
            75.0,
            AlertSeverity::Critical
        ).await?;

        Ok(())
    }

    /// Process real-time analytics
    async fn process_analytics(
        analytics: &Arc<RwLock<NetworkAnalytics>>,
        connections: &Arc<RwLock<HashMap<String, ConnectionMetrics>>>,
        security: &Arc<RwLock<HashMap<String, SecurityAnalytics>>>,
        performance: &Arc<RwLock<VecDeque<PerformanceMetrics>>>
    ) -> Result<()> {
        let connections_map = connections.read().unwrap();
        let security_map = security.read().unwrap();

        // Calculate performance metrics
        let mut total_latency = Duration::ZERO;
        let mut peak_latency = Duration::ZERO;
        let mut total_throughput = 0.0;
        let mut sample_count = 0;

        for metrics in connections_map.values() {
            for &latency in &metrics.latency_samples {
                total_latency += latency;
                peak_latency = peak_latency.max(latency);
                sample_count += 1;
            }
            
            let connection_duration = metrics.last_activity
                .duration_since(metrics.started_at)
                .unwrap_or(Duration::from_secs(1));
            
            let total_bytes = metrics.bytes_sent + metrics.bytes_received;
            total_throughput += total_bytes as f64 / connection_duration.as_secs_f64();
        }

        let average_latency = if sample_count > 0 {
            total_latency / sample_count as u32
        } else {
            Duration::ZERO
        };

        // Calculate performance score
        let latency_score = (1000.0 - average_latency.as_millis() as f64).max(0.0).min(100.0);
        let security_score = 100.0 - security_map.values()
            .map(|s| s.threat_score)
            .fold(0.0, f64::max);
        
        let performance_score = (latency_score + security_score) / 2.0;

        // Update analytics
        {
            let mut analytics = analytics.write().unwrap();
            analytics.services_count = connections_map.values()
                .map(|c| c.service_name.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len() as u32;
            
            analytics.identities_count = connections_map.values()
                .map(|c| c.identity_name.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len() as u32;
            
            analytics.performance_score = performance_score;
        }

        // Store performance history
        let perf_metrics = PerformanceMetrics {
            average_latency,
            peak_latency,
            throughput_bps: total_throughput,
            connection_success_rate: 95.0, // TODO: Calculate based on actual data
            service_availability: 99.9,    // TODO: Calculate based on actual data
            resource_utilization: 65.0,    // TODO: Calculate based on actual data
        };

        {
            let mut perf_history = performance.write().unwrap();
            perf_history.push_back(perf_metrics);
            
            // Keep only recent history
            while perf_history.len() > 288 { // 24 hours at 5-minute intervals
                perf_history.pop_front();
            }
        }

        Ok(())
    }

    /// Archive connection metrics for historical analysis
    async fn archive_connection_metrics(&self, _metrics: &ConnectionMetrics) -> Result<()> {
        // TODO: Implement database storage for historical metrics
        tracing::debug!("Archiving connection metrics (TODO: implement database storage)");
        Ok(())
    }

    /// Legacy compatibility method for get_statistics
    pub async fn get_statistics(&self) -> Result<HashMap<String, u64>> {
        let analytics = self.network_analytics.read().unwrap();
        let mut stats = HashMap::new();
        
        stats.insert("total_connections".to_string(), analytics.total_connections);
        stats.insert("active_connections".to_string(), analytics.active_connections);
        stats.insert("total_data_transferred".to_string(), analytics.total_data_transferred);
        stats.insert("services_count".to_string(), analytics.services_count as u64);
        stats.insert("identities_count".to_string(), analytics.identities_count as u64);
        stats.insert("security_events".to_string(), analytics.security_events);
        stats.insert("performance_score".to_string(), analytics.performance_score as u64);
        
        Ok(stats)
    }
}

// Legacy compatibility alias
pub type ZitiMonitoring = ZitiAdvancedMonitoring;