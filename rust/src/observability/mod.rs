// OpenSim Next - Advanced Observability & Analytics Platform
// Phase 30: Comprehensive monitoring, tracing, and analytics for virtual world operations

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing::{instrument, info, warn, error, debug, span, Level};
use metrics::{counter, gauge, histogram, describe_counter, describe_gauge, describe_histogram};

pub mod tracing_manager;
pub mod analytics_engine;
pub mod performance_profiler;
pub mod user_behavior_analytics;
pub mod virtual_world_metrics;
pub mod distributed_tracing;
pub mod apm_integration;

/// Core observability manager for OpenSim Next
#[derive(Debug, Clone)]
pub struct ObservabilityManager {
    inner: Arc<ObservabilityManagerInner>,
}

#[derive(Debug)]
struct ObservabilityManagerInner {
    metrics_collector: RwLock<MetricsCollector>,
    trace_manager: RwLock<TraceManager>,
    analytics_engine: RwLock<AnalyticsEngine>,
    performance_profiler: RwLock<PerformanceProfiler>,
    config: ObservabilityConfig,
}

/// Configuration for observability features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub enable_distributed_tracing: bool,
    pub enable_performance_profiling: bool,
    pub enable_user_analytics: bool,
    pub enable_real_time_metrics: bool,
    pub trace_sampling_rate: f64,
    pub metrics_retention_days: u32,
    pub analytics_batch_size: usize,
    pub export_endpoint: Option<String>,
    pub jaeger_endpoint: Option<String>,
    pub prometheus_endpoint: String,
    pub grafana_endpoint: Option<String>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enable_distributed_tracing: true,
            enable_performance_profiling: true,
            enable_user_analytics: true,
            enable_real_time_metrics: true,
            trace_sampling_rate: 0.1, // 10% sampling
            metrics_retention_days: 30,
            analytics_batch_size: 1000,
            export_endpoint: None,
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            prometheus_endpoint: "http://localhost:9090".to_string(),
            grafana_endpoint: Some("http://localhost:3000".to_string()),
        }
    }
}

/// Metrics collector for virtual world operations
#[derive(Debug)]
pub struct MetricsCollector {
    active_metrics: HashMap<String, MetricValue>,
    counters: HashMap<String, u64>,
    gauges: HashMap<String, f64>,
    histograms: HashMap<String, Vec<f64>>,
    start_time: Instant,
}

/// Different types of metric values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary { sum: f64, count: u64 },
}

/// Trace manager for distributed tracing
#[derive(Debug)]
pub struct TraceManager {
    active_traces: HashMap<Uuid, TraceSpan>,
    completed_traces: Vec<CompletedTrace>,
    sampling_rate: f64,
}

/// Trace span for tracking operations
#[derive(Debug, Clone)]
pub struct TraceSpan {
    pub trace_id: Uuid,
    pub span_id: Uuid,
    pub parent_span_id: Option<Uuid>,
    pub operation_name: String,
    pub start_time: Instant,
    pub tags: HashMap<String, String>,
    pub logs: Vec<TraceLog>,
}

/// Completed trace with timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTrace {
    pub trace_id: Uuid,
    pub spans: Vec<CompletedSpan>,
    pub total_duration_ms: u64,
    pub service_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Completed span within a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSpan {
    pub span_id: Uuid,
    pub parent_span_id: Option<Uuid>,
    pub operation_name: String,
    pub start_time_ms: u64,
    pub duration_ms: u64,
    pub tags: HashMap<String, String>,
    pub logs: Vec<TraceLog>,
    pub status: SpanStatus,
}

/// Status of a completed span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error { message: String },
    Timeout,
    Cancelled,
}

/// Log entry within a trace span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceLog {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// Analytics engine for user behavior and system performance
#[derive(Debug)]
pub struct AnalyticsEngine {
    user_sessions: HashMap<Uuid, UserSession>,
    world_statistics: WorldStatistics,
    performance_metrics: PerformanceMetrics,
    batch_events: Vec<AnalyticsEvent>,
}

/// User session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub region_visits: Vec<RegionVisit>,
    pub social_interactions: u32,
    pub economic_transactions: u32,
    pub client_type: ClientType,
}

/// Region visit tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionVisit {
    pub region_id: Uuid,
    pub region_name: String,
    pub enter_time: chrono::DateTime<chrono::Utc>,
    pub leave_time: Option<chrono::DateTime<chrono::Utc>>,
    pub avatar_position_samples: Vec<(f64, f64, f64)>,
}

/// Client type identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientType {
    SecondLifeViewer { viewer_name: String, version: String },
    WebBrowser { browser_name: String, version: String },
    Mobile { platform: String, app_version: String },
    Api { client_name: String },
}

/// World-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStatistics {
    pub total_users_online: u32,
    pub total_regions_active: u32,
    pub total_objects_in_world: u64,
    pub physics_bodies_active: u64,
    pub websocket_connections: u32,
    pub database_connections: u32,
    pub asset_requests_per_second: f64,
    pub region_crossings_per_minute: f64,
    pub social_messages_per_minute: f64,
    pub economic_transactions_per_hour: f64,
}

/// Performance metrics for the virtual world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub memory_usage_percent: f64,
    pub disk_io_read_mb_per_sec: f64,
    pub disk_io_write_mb_per_sec: f64,
    pub network_in_mb_per_sec: f64,
    pub network_out_mb_per_sec: f64,
    pub database_query_time_ms_avg: f64,
    pub redis_response_time_ms_avg: f64,
    pub physics_frame_time_ms: f64,
    pub websocket_latency_ms_avg: f64,
}

/// Analytics event for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_id: Uuid,
    pub event_type: AnalyticsEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub region_id: Option<Uuid>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of analytics events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsEventType {
    UserLogin,
    UserLogout,
    RegionEntry,
    RegionExit,
    SocialInteraction { interaction_type: String },
    EconomicTransaction { transaction_type: String, amount: f64 },
    ObjectCreation,
    ObjectDeletion,
    AssetUpload { asset_type: String, size_bytes: u64 },
    PhysicsCollision,
    ScriptExecution { script_name: String, execution_time_ms: u64 },
    PerformanceAlert { alert_type: String, severity: AlertSeverity },
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Performance profiler for deep performance analysis
#[derive(Debug)]
pub struct PerformanceProfiler {
    active_profiles: HashMap<String, ProfileSession>,
    completed_profiles: Vec<CompletedProfile>,
    profiling_enabled: bool,
}

/// Active profiling session
#[derive(Debug)]
pub struct ProfileSession {
    pub profile_id: String,
    pub start_time: Instant,
    pub samples: Vec<ProfileSample>,
    pub target_function: String,
}

/// Performance profile sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSample {
    pub timestamp_ms: u64,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub stack_trace: Vec<String>,
    pub duration_ns: u64,
}

/// Completed performance profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedProfile {
    pub profile_id: String,
    pub function_name: String,
    pub total_duration_ms: u64,
    pub total_samples: usize,
    pub hotspots: Vec<ProfileHotspot>,
    pub memory_allocations: Vec<MemoryAllocation>,
    pub recommendations: Vec<String>,
}

/// Performance hotspot identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileHotspot {
    pub function_name: String,
    pub exclusive_time_ms: u64,
    pub inclusive_time_ms: u64,
    pub call_count: u64,
    pub percentage_of_total: f64,
}

/// Memory allocation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    pub allocation_site: String,
    pub size_bytes: u64,
    pub count: u64,
    pub peak_usage_bytes: u64,
}

impl ObservabilityManager {
    /// Create a new observability manager
    pub fn new(config: ObservabilityConfig) -> Self {
        info!("Initializing OpenSim Next Observability Manager");
        
        // Initialize metrics descriptions
        Self::initialize_metrics();
        
        let inner = ObservabilityManagerInner {
            metrics_collector: RwLock::new(MetricsCollector::new()),
            trace_manager: RwLock::new(TraceManager::new(config.trace_sampling_rate)),
            analytics_engine: RwLock::new(AnalyticsEngine::new()),
            performance_profiler: RwLock::new(PerformanceProfiler::new()),
            config,
        };
        
        Self {
            inner: Arc::new(inner),
        }
    }
    
    /// Initialize Prometheus metrics descriptions
    fn initialize_metrics() {
        // Core virtual world metrics
        describe_counter!("opensim_user_logins_total", "Total number of user logins");
        describe_counter!("opensim_user_logouts_total", "Total number of user logouts");
        describe_gauge!("opensim_users_online", "Number of users currently online");
        describe_gauge!("opensim_regions_active", "Number of active regions");
        describe_gauge!("opensim_objects_total", "Total number of objects in world");
        
        // Performance metrics
        describe_histogram!("opensim_request_duration_seconds", "Request duration in seconds");
        describe_histogram!("opensim_database_query_duration_seconds", "Database query duration");
        describe_histogram!("opensim_physics_frame_time_seconds", "Physics frame time");
        describe_gauge!("opensim_memory_usage_bytes", "Memory usage in bytes");
        describe_gauge!("opensim_cpu_usage_ratio", "CPU usage ratio");
        
        // Network metrics
        describe_counter!("opensim_websocket_connections_total", "Total WebSocket connections");
        describe_counter!("opensim_asset_requests_total", "Total asset requests");
        describe_histogram!("opensim_websocket_latency_seconds", "WebSocket latency");
        
        // Social metrics
        describe_counter!("opensim_social_messages_total", "Total social messages sent");
        describe_counter!("opensim_friend_requests_total", "Total friend requests");
        describe_gauge!("opensim_groups_active", "Number of active groups");
        
        // Economic metrics
        describe_counter!("opensim_transactions_total", "Total economic transactions");
        describe_histogram!("opensim_transaction_amount", "Economic transaction amounts");
        describe_gauge!("opensim_currency_in_circulation", "Total currency in circulation");
    }
    
    /// Record a metric value
    #[instrument(skip(self))]
    pub async fn record_metric(&self, name: &str, value: MetricValue) {
        let mut collector = self.inner.metrics_collector.write().await;
        collector.record_metric(name, value).await;
    }
    
    /// Start a new trace span
    #[instrument(skip(self))]
    pub async fn start_trace(&self, operation_name: &str, parent_span_id: Option<Uuid>) -> Uuid {
        let mut trace_manager = self.inner.trace_manager.write().await;
        trace_manager.start_span(operation_name, parent_span_id).await
    }
    
    /// Complete a trace span
    #[instrument(skip(self))]
    pub async fn complete_trace(&self, span_id: Uuid, status: SpanStatus) {
        let mut trace_manager = self.inner.trace_manager.write().await;
        trace_manager.complete_span(span_id, status).await;
    }
    
    /// Record an analytics event
    #[instrument(skip(self))]
    pub async fn record_event(&self, event: AnalyticsEvent) {
        let mut analytics = self.inner.analytics_engine.write().await;
        analytics.record_event(event).await;
    }
    
    /// Start performance profiling
    #[instrument(skip(self))]
    pub async fn start_profiling(&self, profile_id: &str, target_function: &str) {
        if !self.inner.config.enable_performance_profiling {
            return;
        }
        
        let mut profiler = self.inner.performance_profiler.write().await;
        profiler.start_profiling(profile_id, target_function).await;
    }
    
    /// Get current system metrics
    #[instrument(skip(self))]
    pub async fn get_system_metrics(&self) -> WorldStatistics {
        let analytics = self.inner.analytics_engine.read().await;
        analytics.get_world_statistics().await
    }
    
    /// Export metrics to external systems
    #[instrument(skip(self))]
    pub async fn export_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(endpoint) = &self.inner.config.export_endpoint {
            let metrics = self.collect_all_metrics().await;
            self.send_to_external_system(endpoint, &metrics).await?;
        }
        Ok(())
    }
    
    /// Collect all metrics for export
    async fn collect_all_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut all_metrics = HashMap::new();
        
        // Collect metrics
        let collector = self.inner.metrics_collector.read().await;
        for (name, value) in &collector.active_metrics {
            all_metrics.insert(name.clone(), serde_json::to_value(value).unwrap_or_default());
        }
        
        // Collect world statistics
        let analytics = self.inner.analytics_engine.read().await;
        let world_stats = analytics.get_world_statistics().await;
        all_metrics.insert("world_statistics".to_string(), serde_json::to_value(world_stats).unwrap_or_default());
        
        all_metrics
    }
    
    /// Send metrics to external monitoring system
    async fn send_to_external_system(&self, endpoint: &str, metrics: &HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let response = client
            .post(endpoint)
            .json(metrics)
            .send()
            .await?;
            
        if response.status().is_success() {
            debug!("Successfully exported metrics to {}", endpoint);
        } else {
            warn!("Failed to export metrics to {}: {}", endpoint, response.status());
        }
        
        Ok(())
    }
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            active_metrics: HashMap::new(),
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            start_time: Instant::now(),
        }
    }
    
    async fn record_metric(&mut self, name: &str, value: MetricValue) {
        match &value {
            MetricValue::Counter(count) => {
                counter!(name, *count);
                *self.counters.entry(name.to_string()).or_insert(0) += count;
            }
            MetricValue::Gauge(gauge_value) => {
                gauge!(name, *gauge_value);
                self.gauges.insert(name.to_string(), *gauge_value);
            }
            MetricValue::Histogram(values) => {
                for val in values {
                    histogram!(name, *val);
                }
                self.histograms.entry(name.to_string()).or_insert_with(Vec::new).extend(values);
            }
            MetricValue::Summary { sum, count } => {
                if *count > 0 {
                    let avg = sum / (*count as f64);
                    gauge!(format!("{}_avg", name), avg);
                    counter!(format!("{}_count", name), *count);
                }
            }
        }
        
        self.active_metrics.insert(name.to_string(), value);
    }
}

impl TraceManager {
    fn new(sampling_rate: f64) -> Self {
        Self {
            active_traces: HashMap::new(),
            completed_traces: Vec::new(),
            sampling_rate,
        }
    }
    
    async fn start_span(&mut self, operation_name: &str, parent_span_id: Option<Uuid>) -> Uuid {
        let span_id = Uuid::new_v4();
        let trace_id = parent_span_id.unwrap_or_else(Uuid::new_v4);
        
        let span = TraceSpan {
            trace_id,
            span_id,
            parent_span_id,
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
            tags: HashMap::new(),
            logs: Vec::new(),
        };
        
        self.active_traces.insert(span_id, span);
        span_id
    }
    
    async fn complete_span(&mut self, span_id: Uuid, status: SpanStatus) {
        if let Some(span) = self.active_traces.remove(&span_id) {
            let duration = span.start_time.elapsed();
            
            let completed_span = CompletedSpan {
                span_id: span.span_id,
                parent_span_id: span.parent_span_id,
                operation_name: span.operation_name,
                start_time_ms: span.start_time.elapsed().as_millis() as u64,
                duration_ms: duration.as_millis() as u64,
                tags: span.tags,
                logs: span.logs,
                status,
            };
            
            // Find or create completed trace
            if let Some(completed_trace) = self.completed_traces.iter_mut().find(|t| t.trace_id == span.trace_id) {
                completed_trace.spans.push(completed_span);
            } else {
                let completed_trace = CompletedTrace {
                    trace_id: span.trace_id,
                    spans: vec![completed_span],
                    total_duration_ms: duration.as_millis() as u64,
                    service_name: "opensim-next".to_string(),
                    timestamp: chrono::Utc::now(),
                };
                self.completed_traces.push(completed_trace);
            }
        }
    }
}

impl AnalyticsEngine {
    fn new() -> Self {
        Self {
            user_sessions: HashMap::new(),
            world_statistics: WorldStatistics::default(),
            performance_metrics: PerformanceMetrics::default(),
            batch_events: Vec::new(),
        }
    }
    
    async fn record_event(&mut self, event: AnalyticsEvent) {
        // Update real-time statistics based on event
        match &event.event_type {
            AnalyticsEventType::UserLogin => {
                self.world_statistics.total_users_online += 1;
            }
            AnalyticsEventType::UserLogout => {
                self.world_statistics.total_users_online = self.world_statistics.total_users_online.saturating_sub(1);
            }
            AnalyticsEventType::RegionEntry => {
                // Track region activity
            }
            AnalyticsEventType::EconomicTransaction { .. } => {
                self.world_statistics.economic_transactions_per_hour += 1.0;
            }
            _ => {}
        }
        
        self.batch_events.push(event);
    }
    
    async fn get_world_statistics(&self) -> WorldStatistics {
        self.world_statistics.clone()
    }
}

impl PerformanceProfiler {
    fn new() -> Self {
        Self {
            active_profiles: HashMap::new(),
            completed_profiles: Vec::new(),
            profiling_enabled: true,
        }
    }
    
    async fn start_profiling(&mut self, profile_id: &str, target_function: &str) {
        if !self.profiling_enabled {
            return;
        }
        
        let session = ProfileSession {
            profile_id: profile_id.to_string(),
            start_time: Instant::now(),
            samples: Vec::new(),
            target_function: target_function.to_string(),
        };
        
        self.active_profiles.insert(profile_id.to_string(), session);
    }
}

impl Default for WorldStatistics {
    fn default() -> Self {
        Self {
            total_users_online: 0,
            total_regions_active: 0,
            total_objects_in_world: 0,
            physics_bodies_active: 0,
            websocket_connections: 0,
            database_connections: 0,
            asset_requests_per_second: 0.0,
            region_crossings_per_minute: 0.0,
            social_messages_per_minute: 0.0,
            economic_transactions_per_hour: 0.0,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            memory_usage_percent: 0.0,
            disk_io_read_mb_per_sec: 0.0,
            disk_io_write_mb_per_sec: 0.0,
            network_in_mb_per_sec: 0.0,
            network_out_mb_per_sec: 0.0,
            database_query_time_ms_avg: 0.0,
            redis_response_time_ms_avg: 0.0,
            physics_frame_time_ms: 0.0,
            websocket_latency_ms_avg: 0.0,
        }
    }
}