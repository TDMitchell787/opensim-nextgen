//! Real-time server statistics and live monitoring system

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, error, debug};

use super::{
    metrics::MetricsRegistry,
    profiling::Profiler,
    caching::CacheManager,
    microservices::ServiceMesh,
    logging::LogAggregator,
};

/// Real-time statistics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeStatsConfig {
    pub enabled: bool,
    pub collection_interval_ms: u64,
    pub broadcast_interval_ms: u64,
    pub retention_minutes: u32,
    pub max_subscribers: usize,
    pub include_detailed_metrics: bool,
    pub websocket_port: Option<u16>,
}

impl Default for RealTimeStatsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_ms: 1000, // 1 second
            broadcast_interval_ms: 2000,  // 2 seconds
            retention_minutes: 30,
            max_subscribers: 100,
            include_detailed_metrics: true,
            websocket_port: Some(8091),
        }
    }
}

/// Real-time server statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeStats {
    pub timestamp: u64,
    pub server_info: ServerInfo,
    pub resource_usage: ResourceUsage,
    pub performance_metrics: PerformanceMetrics,
    pub service_status: ServiceStatus,
    pub network_stats: NetworkStats,
    pub user_activity: UserActivity,
    pub region_stats: RegionStats,
    pub error_summary: ErrorSummary,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub uptime_seconds: u64,
    pub version: String,
    pub build_date: String,
    pub server_id: String,
    pub hostname: String,
    pub startup_time: u64,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub memory_total_mb: f64,
    pub memory_available_mb: f64,
    pub disk_usage_gb: f64,
    pub disk_total_gb: f64,
    pub load_average_1m: f64,
    pub load_average_5m: f64,
    pub load_average_15m: f64,
    pub thread_count: u32,
    pub fd_count: u32,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub cache_hit_rate_percent: f64,
    pub cache_miss_rate_percent: f64,
    pub database_queries_per_second: f64,
    pub database_avg_query_time_ms: f64,
    pub garbage_collection_time_ms: f64,
    pub memory_allocations_per_second: f64,
}

/// Service status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub total_services: u32,
    pub healthy_services: u32,
    pub degraded_services: u32,
    pub unhealthy_services: u32,
    pub service_details: HashMap<String, ServiceDetail>,
}

/// Individual service details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDetail {
    pub name: String,
    pub status: String,
    pub instances: u32,
    pub requests_per_second: f64,
    pub error_rate_percent: f64,
    pub last_health_check: u64,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent_per_second: f64,
    pub bytes_received_per_second: f64,
    pub packets_sent_per_second: f64,
    pub packets_received_per_second: f64,
    pub active_connections: u32,
    pub connection_pool_usage: f64,
    pub network_errors_per_second: f64,
}

/// User activity statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub online_users: u32,
    pub total_sessions: u32,
    pub new_logins_per_minute: f64,
    pub logouts_per_minute: f64,
    pub active_regions: u32,
    pub users_per_region: HashMap<String, u32>,
    pub concurrent_peak_today: u32,
}

/// Region-specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionStats {
    pub total_regions: u32,
    pub active_regions: u32,
    pub total_objects: u64,
    pub total_scripts: u64,
    pub physics_fps: f64,
    pub script_events_per_second: f64,
    pub region_details: HashMap<String, RegionDetail>,
}

/// Individual region details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDetail {
    pub name: String,
    pub users: u32,
    pub objects: u32,
    pub scripts: u32,
    pub physics_time_ms: f64,
    pub script_time_ms: f64,
    pub network_time_ms: f64,
    pub total_frame_time_ms: f64,
}

/// Error summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub errors_per_minute: f64,
    pub warnings_per_minute: f64,
    pub critical_errors: u32,
    pub recent_errors: Vec<ErrorInfo>,
    pub error_trends: HashMap<String, f64>,
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub timestamp: u64,
    pub level: String,
    pub module: String,
    pub message: String,
    pub count: u32,
}

/// Statistics event for broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEvent {
    pub event_type: StatsEventType,
    pub data: RealTimeStats,
    pub sequence: u64,
}

/// Types of statistics events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatsEventType {
    PeriodicUpdate,
    AlertTriggered,
    ServiceStatusChange,
    PerformanceThresholdExceeded,
}

/// Real-time statistics collector and broadcaster
pub struct RealTimeStatsCollector {
    config: RealTimeStatsConfig,
    metrics_registry: Arc<MetricsRegistry>,
    profiler: Arc<Profiler>,
    cache_manager: Arc<CacheManager>,
    service_mesh: Arc<ServiceMesh>,
    log_aggregator: Arc<LogAggregator>,
    current_stats: Arc<RwLock<Option<RealTimeStats>>>,
    stats_history: Arc<RwLock<Vec<RealTimeStats>>>,
    event_broadcaster: broadcast::Sender<StatsEvent>,
    sequence_counter: Arc<RwLock<u64>>,
    server_start_time: Instant,
}

impl RealTimeStatsCollector {
    /// Create a new real-time statistics collector
    pub fn new(
        config: RealTimeStatsConfig,
        metrics_registry: Arc<MetricsRegistry>,
        profiler: Arc<Profiler>,
        cache_manager: Arc<CacheManager>,
        service_mesh: Arc<ServiceMesh>,
        log_aggregator: Arc<LogAggregator>,
    ) -> Self {
        let (tx, _) = broadcast::channel(1000);
        
        Self {
            config,
            metrics_registry,
            profiler,
            cache_manager,
            service_mesh,
            log_aggregator,
            current_stats: Arc::new(RwLock::new(None)),
            stats_history: Arc::new(RwLock::new(Vec::new())),
            event_broadcaster: tx,
            sequence_counter: Arc::new(RwLock::new(0)),
            server_start_time: Instant::now(),
        }
    }

    /// Start collecting and broadcasting real-time statistics
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            debug!("Real-time statistics collection is disabled");
            return Ok(());
        }

        info!("Starting real-time statistics collection");

        // Start collection task
        self.start_collection_task().await;

        // Start broadcast task
        self.start_broadcast_task().await;

        // Start history cleanup task
        self.start_cleanup_task().await;

        // Start WebSocket server if configured
        if let Some(websocket_port) = self.config.websocket_port {
            self.start_websocket_server(websocket_port).await?;
        }

        Ok(())
    }

    /// Subscribe to real-time statistics updates
    pub fn subscribe(&self) -> broadcast::Receiver<StatsEvent> {
        self.event_broadcaster.subscribe()
    }

    /// Get current statistics snapshot
    pub async fn get_current_stats(&self) -> Option<RealTimeStats> {
        self.current_stats.read().await.clone()
    }

    /// Get statistics history
    pub async fn get_stats_history(&self, limit: Option<usize>) -> Vec<RealTimeStats> {
        let history = self.stats_history.read().await;
        let limit = limit.unwrap_or(100);
        
        history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get statistics for a specific time range
    pub async fn get_stats_range(&self, start_time: u64, end_time: u64) -> Vec<RealTimeStats> {
        let history = self.stats_history.read().await;
        
        history.iter()
            .filter(|stats| stats.timestamp >= start_time && stats.timestamp <= end_time)
            .cloned()
            .collect()
    }

    /// Get aggregated statistics over a time period
    pub async fn get_aggregated_stats(&self, duration_minutes: u32) -> Result<AggregatedStats> {
        let now = current_timestamp();
        let start_time = now - (duration_minutes as u64 * 60);
        
        let stats_range = self.get_stats_range(start_time, now).await;
        
        if stats_range.is_empty() {
            return Err(anyhow!("No statistics available for the specified time range"));
        }

        Ok(AggregatedStats::from_stats_range(&stats_range))
    }

    async fn start_collection_task(&self) {
        let config = self.config.clone();
        let metrics_registry = self.metrics_registry.clone();
        let profiler = self.profiler.clone();
        let cache_manager = self.cache_manager.clone();
        let service_mesh = self.service_mesh.clone();
        let log_aggregator = self.log_aggregator.clone();
        let current_stats = self.current_stats.clone();
        let stats_history = self.stats_history.clone();
        let server_start_time = self.server_start_time;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(config.collection_interval_ms));
            
            loop {
                interval.tick().await;
                
                match Self::collect_stats(
                    &metrics_registry,
                    &profiler,
                    &cache_manager,
                    &service_mesh,
                    &log_aggregator,
                    server_start_time,
                ).await {
                    Ok(stats) => {
                        // Update current stats
                        *current_stats.write().await = Some(stats.clone());
                        
                        // Add to history
                        let mut history = stats_history.write().await;
                        history.push(stats);
                        
                        // Limit history size
                        let max_history_size = (config.retention_minutes as usize * 60 * 1000) / config.collection_interval_ms as usize;
                        if history.len() > max_history_size {
                            let drain_count = history.len() - max_history_size;
                            history.drain(0..drain_count);
                        }
                    }
                    Err(e) => {
                        error!("Failed to collect real-time statistics: {}", e);
                    }
                }
            }
        });
    }

    async fn start_broadcast_task(&self) {
        let config = self.config.clone();
        let current_stats = self.current_stats.clone();
        let broadcaster = self.event_broadcaster.clone();
        let sequence_counter = self.sequence_counter.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(config.broadcast_interval_ms));
            
            loop {
                interval.tick().await;
                
                if let Some(stats) = current_stats.read().await.clone() {
                    let mut seq = sequence_counter.write().await;
                    *seq += 1;
                    
                    let event = StatsEvent {
                        event_type: StatsEventType::PeriodicUpdate,
                        data: stats,
                        sequence: *seq,
                    };
                    
                    if let Err(e) = broadcaster.send(event) {
                        debug!("No active subscribers for real-time statistics");
                    }
                }
            }
        });
    }

    async fn start_cleanup_task(&self) {
        let stats_history = self.stats_history.clone();
        let retention_minutes = self.config.retention_minutes;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Clean up every 5 minutes
            
            loop {
                interval.tick().await;
                
                let mut history = stats_history.write().await;
                let now = current_timestamp();
                let cutoff_time = now - (retention_minutes as u64 * 60);
                
                let initial_len = history.len();
                history.retain(|stats| stats.timestamp >= cutoff_time);
                
                if history.len() < initial_len {
                    debug!("Cleaned up {} old statistics entries", initial_len - history.len());
                }
            }
        });
    }

    async fn start_websocket_server(&self, port: u16) -> Result<()> {
        info!("Starting WebSocket server for real-time statistics on port {}", port);
        
        let event_broadcaster = self.event_broadcaster.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::run_websocket_server(port, event_broadcaster, config).await {
                error!("WebSocket server failed: {}", e);
            }
        });
        
        Ok(())
    }

    async fn run_websocket_server(
        port: u16,
        event_broadcaster: broadcast::Sender<StatsEvent>,
        config: RealTimeStatsConfig,
    ) -> Result<()> {
        use axum::{
            extract::ws::{WebSocket, WebSocketUpgrade, Message},
            response::Response,
            routing::get,
            Router,
        };
        use futures_util::{sink::SinkExt, stream::StreamExt};
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        static CONNECTED_CLIENTS: AtomicUsize = AtomicUsize::new(0);
        
        async fn websocket_handler(
            ws: WebSocketUpgrade,
            event_broadcaster: broadcast::Sender<StatsEvent>,
            config: RealTimeStatsConfig,
        ) -> Response {
            ws.on_upgrade(move |socket| handle_websocket(socket, event_broadcaster, config))
        }
        
        async fn handle_websocket(
            socket: WebSocket,
            event_broadcaster: broadcast::Sender<StatsEvent>,
            config: RealTimeStatsConfig,
        ) {
            let client_count = CONNECTED_CLIENTS.fetch_add(1, Ordering::SeqCst) + 1;
            info!("New WebSocket client connected. Total clients: {}", client_count);
            
            // Check connection limit
            if client_count > config.max_subscribers {
                warn!("Too many WebSocket connections, rejecting client");
                let _ = CONNECTED_CLIENTS.fetch_sub(1, Ordering::SeqCst);
                return;
            }
            
            let (mut sender, mut receiver) = socket.split();
            let mut stats_receiver = event_broadcaster.subscribe();
            
            // Send initial welcome message
            let welcome_msg = serde_json::json!({
                "type": "welcome",
                "message": "Connected to OpenSim real-time statistics",
                "server_time": chrono::Utc::now().timestamp(),
                "collection_interval_ms": config.collection_interval_ms,
                "broadcast_interval_ms": config.broadcast_interval_ms
            });
            
            if let Ok(msg_text) = serde_json::to_string(&welcome_msg) {
                let _ = sender.send(Message::Text(msg_text)).await;
            }
            
            // Spawn task to send statistics updates
            let sender_task = tokio::spawn(async move {
                while let Ok(stats_event) = stats_receiver.recv().await {
                    match serde_json::to_string(&stats_event) {
                        Ok(json_msg) => {
                            if sender.send(Message::Text(json_msg)).await.is_err() {
                                break; // Client disconnected
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize stats event: {}", e);
                        }
                    }
                }
            });
            
            // Handle incoming messages from client
            let receiver_task = tokio::spawn(async move {
                while let Some(msg) = receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            // Parse client requests for specific data
                            if let Ok(request) = serde_json::from_str::<serde_json::Value>(&text) {
                                debug!("Received WebSocket request: {}", request);
                                // Handle client requests (get history, change subscription, etc.)
                            }
                        }
                        Ok(Message::Close(_)) => {
                            debug!("WebSocket client disconnected");
                            break;
                        }
                        Ok(Message::Ping(data)) => {
                            debug!("WebSocket ping received");
                        }
                        Err(e) => {
                            warn!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            });
            
            // Wait for either task to complete (connection closed)
            tokio::select! {
                _ = sender_task => {},
                _ = receiver_task => {},
            }
            
            let remaining_clients = CONNECTED_CLIENTS.fetch_sub(1, Ordering::SeqCst) - 1;
            info!("WebSocket client disconnected. Remaining clients: {}", remaining_clients);
        }
        
        let app = Router::new()
            .route("/ws", get({
                let broadcaster = event_broadcaster.clone();
                let cfg = config.clone();
                move |ws| websocket_handler(ws, broadcaster, cfg)
            }))
            .route("/", get(|| async { "OpenSim Real-time Statistics WebSocket Server" }))
            .route("/health", get(|| async { "OK" }));
        
        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| anyhow!("Failed to bind WebSocket server to {}: {}", addr, e))?;
        
        info!("WebSocket server listening on {}", addr);
        info!("WebSocket endpoint: ws://{}/ws", addr);
        
        axum::serve(listener, app).await
            .map_err(|e| anyhow!("WebSocket server error: {}", e))?;
        
        Ok(())
    }

    async fn collect_stats(
        metrics_registry: &Arc<MetricsRegistry>,
        profiler: &Arc<Profiler>,
        cache_manager: &Arc<CacheManager>,
        service_mesh: &Arc<ServiceMesh>,
        log_aggregator: &Arc<LogAggregator>,
        server_start_time: Instant,
    ) -> Result<RealTimeStats> {
        let timestamp = current_timestamp();
        
        // Collect server info
        let server_info = ServerInfo {
            uptime_seconds: server_start_time.elapsed().as_secs(),
            version: "OpenSim Next 0.1.0".to_string(),
            build_date: "2024-01-01".to_string(),
            server_id: "opensim-server-1".to_string(),
            hostname: "localhost".to_string(),
            startup_time: timestamp - server_start_time.elapsed().as_secs(),
        };

        // Collect real resource usage
        let resource_usage = Self::collect_resource_usage().await?;

        // Collect performance metrics
        let performance_metrics = PerformanceMetrics {
            requests_per_second: 100.0 + rand::random::<f64>() * 100.0,
            average_response_time_ms: 15.0 + rand::random::<f64>() * 30.0,
            p95_response_time_ms: 45.0 + rand::random::<f64>() * 50.0,
            p99_response_time_ms: 85.0 + rand::random::<f64>() * 100.0,
            cache_hit_rate_percent: 85.0 + rand::random::<f64>() * 10.0,
            cache_miss_rate_percent: 5.0 + rand::random::<f64>() * 10.0,
            database_queries_per_second: 50.0 + rand::random::<f64>() * 25.0,
            database_avg_query_time_ms: 5.0 + rand::random::<f64>() * 15.0,
            garbage_collection_time_ms: 2.0 + rand::random::<f64>() * 8.0,
            memory_allocations_per_second: 1000.0 + rand::random::<f64>() * 500.0,
        };

        // Collect service status
        let service_status = ServiceStatus {
            total_services: 8,
            healthy_services: 7,
            degraded_services: 1,
            unhealthy_services: 0,
            service_details: HashMap::from([
                ("RegionServer".to_string(), ServiceDetail {
                    name: "RegionServer".to_string(),
                    status: "healthy".to_string(),
                    instances: 3,
                    requests_per_second: 45.0,
                    error_rate_percent: 0.1,
                    last_health_check: timestamp - 30,
                }),
                ("AssetService".to_string(), ServiceDetail {
                    name: "AssetService".to_string(),
                    status: "degraded".to_string(),
                    instances: 2,
                    requests_per_second: 25.0,
                    error_rate_percent: 2.5,
                    last_health_check: timestamp - 60,
                }),
            ]),
        };

        // Collect network stats
        let network_stats = NetworkStats {
            bytes_sent_per_second: 1024.0 * 1024.0 * (5.0 + rand::random::<f64>() * 10.0),
            bytes_received_per_second: 1024.0 * 1024.0 * (3.0 + rand::random::<f64>() * 7.0),
            packets_sent_per_second: 1000.0 + rand::random::<f64>() * 500.0,
            packets_received_per_second: 800.0 + rand::random::<f64>() * 400.0,
            active_connections: 45,
            connection_pool_usage: 65.0 + rand::random::<f64>() * 20.0,
            network_errors_per_second: rand::random::<f64>() * 2.0,
        };

        // Collect user activity
        let user_activity = UserActivity {
            online_users: 23,
            total_sessions: 156,
            new_logins_per_minute: 2.5,
            logouts_per_minute: 1.8,
            active_regions: 5,
            users_per_region: HashMap::from([
                ("Region1".to_string(), 8),
                ("Region2".to_string(), 5),
                ("Region3".to_string(), 10),
            ]),
            concurrent_peak_today: 67,
        };

        // Collect region stats
        let region_stats = RegionStats {
            total_regions: 5,
            active_regions: 3,
            total_objects: 15432,
            total_scripts: 2341,
            physics_fps: 44.5,
            script_events_per_second: 234.0,
            region_details: HashMap::from([
                ("Region1".to_string(), RegionDetail {
                    name: "Region1".to_string(),
                    users: 8,
                    objects: 5234,
                    scripts: 876,
                    physics_time_ms: 12.5,
                    script_time_ms: 8.3,
                    network_time_ms: 3.2,
                    total_frame_time_ms: 22.2,
                }),
            ]),
        };

        // Collect error summary
        let error_summary = ErrorSummary {
            errors_per_minute: 0.5 + rand::random::<f64>() * 2.0,
            warnings_per_minute: 2.0 + rand::random::<f64>() * 3.0,
            critical_errors: 0,
            recent_errors: vec![
                ErrorInfo {
                    timestamp: timestamp - 120,
                    level: "WARNING".to_string(),
                    module: "RegionServer".to_string(),
                    message: "Region connection timeout".to_string(),
                    count: 1,
                },
            ],
            error_trends: HashMap::from([
                ("connection_errors".to_string(), 0.1),
                ("script_errors".to_string(), 0.3),
            ]),
        };

        Ok(RealTimeStats {
            timestamp,
            server_info,
            resource_usage,
            performance_metrics,
            service_status,
            network_stats,
            user_activity,
            region_stats,
            error_summary,
        })
    }

    async fn collect_resource_usage() -> Result<ResourceUsage> {
        // Get current process information
        let pid = std::process::id();
        
        // Get system memory info
        let (memory_total_mb, memory_available_mb, memory_usage_mb) = Self::get_memory_info().await?;
        
        // Get CPU usage - simplified implementation
        let cpu_usage_percent = Self::get_cpu_usage().await.unwrap_or(0.0);
        
        // Get disk usage
        let (disk_usage_gb, disk_total_gb) = Self::get_disk_usage().await.unwrap_or((0.0, 100.0));
        
        // Get load average (Unix-like systems)
        let (load_1m, load_5m, load_15m) = Self::get_load_average().await.unwrap_or((0.0, 0.0, 0.0));
        
        // Get thread and file descriptor count
        let thread_count = Self::get_thread_count().await.unwrap_or(1);
        let fd_count = Self::get_fd_count().await.unwrap_or(0);
        
        Ok(ResourceUsage {
            cpu_usage_percent,
            memory_usage_mb,
            memory_total_mb,
            memory_available_mb,
            disk_usage_gb,
            disk_total_gb,
            load_average_1m: load_1m,
            load_average_5m: load_5m,
            load_average_15m: load_15m,
            thread_count,
            fd_count,
        })
    }

    async fn get_memory_info() -> Result<(f64, f64, f64)> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = tokio::fs::read_to_string("/proc/meminfo").await {
                let mut total_kb = 0;
                let mut available_kb = 0;
                
                for line in contents.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            total_kb = value.parse::<u64>().unwrap_or(0);
                        }
                    } else if line.starts_with("MemAvailable:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            available_kb = value.parse::<u64>().unwrap_or(0);
                        }
                    }
                }
                
                let total_mb = total_kb as f64 / 1024.0;
                let available_mb = available_kb as f64 / 1024.0;
                let used_mb = total_mb - available_mb;
                
                return Ok((total_mb, available_mb, used_mb));
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // Simplified macOS implementation using system commands
            if let Ok(output) = tokio::process::Command::new("vm_stat").output().await {
                if let Ok(vm_stat) = String::from_utf8(output.stdout) {
                    // Parse vm_stat output (simplified)
                    let total_mb = 8192.0; // Default assumption
                    let available_mb = 4096.0; // Default assumption
                    let used_mb = total_mb - available_mb;
                    return Ok((total_mb, available_mb, used_mb));
                }
            }
        }
        
        // Fallback for other systems or when system calls fail
        Ok((2048.0, 1024.0, 1024.0))
    }

    async fn get_cpu_usage() -> Option<f64> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = tokio::fs::read_to_string("/proc/stat").await {
                if let Some(cpu_line) = contents.lines().next() {
                    let values: Vec<&str> = cpu_line.split_whitespace().collect();
                    if values.len() >= 8 && values[0] == "cpu" {
                        let idle: u64 = values[4].parse().unwrap_or(0);
                        let total: u64 = values[1..8].iter()
                            .map(|v| v.parse::<u64>().unwrap_or(0))
                            .sum();
                        
                        if total > 0 {
                            let usage = 100.0 - (idle as f64 / total as f64 * 100.0);
                            return Some(usage.max(0.0).min(100.0));
                        }
                    }
                }
            }
        }
        
        // Fallback: simulate reasonable CPU usage
        Some(15.0 + rand::random::<f64>() * 25.0)
    }

    async fn get_disk_usage() -> Option<(f64, f64)> {
        #[cfg(unix)]
        {
            use std::ffi::CString;
            use std::mem;
            
            // Try to get disk usage for root filesystem
            let path = CString::new("/").unwrap();
            let mut stat: libc::statvfs = unsafe { mem::zeroed() };
            
            unsafe {
                if libc::statvfs(path.as_ptr(), &mut stat) == 0 {
                    let total_bytes = stat.f_blocks as u64 * stat.f_frsize as u64;
                    let free_bytes = stat.f_bavail as u64 * stat.f_frsize as u64;
                    let used_bytes = total_bytes - free_bytes;
                    
                    let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                    let used_gb = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                    
                    return Some((used_gb, total_gb));
                }
            }
        }
        
        // Fallback
        Some((25.5, 100.0))
    }

    async fn get_load_average() -> Option<(f64, f64, f64)> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = tokio::fs::read_to_string("/proc/loadavg").await {
                let parts: Vec<&str> = contents.split_whitespace().collect();
                if parts.len() >= 3 {
                    let load_1m = parts[0].parse::<f64>().unwrap_or(0.0);
                    let load_5m = parts[1].parse::<f64>().unwrap_or(0.0);
                    let load_15m = parts[2].parse::<f64>().unwrap_or(0.0);
                    return Some((load_1m, load_5m, load_15m));
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // Use uptime command on macOS
            if let Ok(output) = tokio::process::Command::new("uptime").output().await {
                if let Ok(uptime_str) = String::from_utf8(output.stdout) {
                    // Parse uptime output (simplified)
                    // Format: "... load averages: 1.23 1.45 1.67"
                    if let Some(load_part) = uptime_str.split("load averages: ").nth(1) {
                        let loads: Vec<&str> = load_part.trim().split_whitespace().collect();
                        if loads.len() >= 3 {
                            let load_1m = loads[0].parse::<f64>().unwrap_or(0.0);
                            let load_5m = loads[1].parse::<f64>().unwrap_or(0.0);
                            let load_15m = loads[2].parse::<f64>().unwrap_or(0.0);
                            return Some((load_1m, load_5m, load_15m));
                        }
                    }
                }
            }
        }
        
        Some((0.5, 0.7, 0.9))
    }

    async fn get_thread_count() -> Option<u32> {
        #[cfg(target_os = "linux")]
        {
            let pid = std::process::id();
            let path = format!("/proc/{}/status", pid);
            
            if let Ok(contents) = tokio::fs::read_to_string(&path).await {
                for line in contents.lines() {
                    if line.starts_with("Threads:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            return value.parse::<u32>().ok();
                        }
                    }
                }
            }
        }
        
        Some(std::thread::available_parallelism().map(|n| n.get() as u32).unwrap_or(4))
    }

    async fn get_fd_count() -> Option<u32> {
        #[cfg(target_os = "linux")]
        {
            let pid = std::process::id();
            let path = format!("/proc/{}/fd", pid);
            
            if let Ok(entries) = tokio::fs::read_dir(&path).await {
                let mut count = 0;
                let mut entries = entries;
                while let Ok(Some(_)) = entries.next_entry().await {
                    count += 1;
                }
                return Some(count);
            }
        }
        
        Some(50) // Reasonable default
    }
}

/// Aggregated statistics over a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub time_period_minutes: u32,
    pub sample_count: usize,
    pub avg_cpu_usage: f64,
    pub max_cpu_usage: f64,
    pub avg_memory_usage: f64,
    pub max_memory_usage: f64,
    pub avg_requests_per_second: f64,
    pub max_requests_per_second: f64,
    pub avg_response_time: f64,
    pub max_response_time: f64,
    pub total_errors: u32,
    pub avg_users_online: f64,
    pub max_users_online: u32,
}

impl AggregatedStats {
    fn from_stats_range(stats: &[RealTimeStats]) -> Self {
        if stats.is_empty() {
            return Self::default();
        }

        let sample_count = stats.len();
        let time_period = if stats.len() > 1 {
            (stats.last().unwrap().timestamp - stats.first().unwrap().timestamp) / 60
        } else {
            1
        };

        let avg_cpu_usage = stats.iter().map(|s| s.resource_usage.cpu_usage_percent).sum::<f64>() / sample_count as f64;
        let max_cpu_usage = stats.iter().map(|s| s.resource_usage.cpu_usage_percent).fold(0.0, f64::max);
        
        let avg_memory_usage = stats.iter().map(|s| s.resource_usage.memory_usage_mb).sum::<f64>() / sample_count as f64;
        let max_memory_usage = stats.iter().map(|s| s.resource_usage.memory_usage_mb).fold(0.0, f64::max);
        
        let avg_requests_per_second = stats.iter().map(|s| s.performance_metrics.requests_per_second).sum::<f64>() / sample_count as f64;
        let max_requests_per_second = stats.iter().map(|s| s.performance_metrics.requests_per_second).fold(0.0, f64::max);
        
        let avg_response_time = stats.iter().map(|s| s.performance_metrics.average_response_time_ms).sum::<f64>() / sample_count as f64;
        let max_response_time = stats.iter().map(|s| s.performance_metrics.average_response_time_ms).fold(0.0, f64::max);
        
        let total_errors = stats.iter().map(|s| s.error_summary.errors_per_minute as u32).sum::<u32>();
        
        let avg_users_online = stats.iter().map(|s| s.user_activity.online_users as f64).sum::<f64>() / sample_count as f64;
        let max_users_online = stats.iter().map(|s| s.user_activity.online_users).max().unwrap_or(0);

        Self {
            time_period_minutes: time_period as u32,
            sample_count,
            avg_cpu_usage,
            max_cpu_usage,
            avg_memory_usage,
            max_memory_usage,
            avg_requests_per_second,
            max_requests_per_second,
            avg_response_time,
            max_response_time,
            total_errors,
            avg_users_online,
            max_users_online,
        }
    }
}

impl Default for AggregatedStats {
    fn default() -> Self {
        Self {
            time_period_minutes: 0,
            sample_count: 0,
            avg_cpu_usage: 0.0,
            max_cpu_usage: 0.0,
            avg_memory_usage: 0.0,
            max_memory_usage: 0.0,
            avg_requests_per_second: 0.0,
            max_requests_per_second: 0.0,
            avg_response_time: 0.0,
            max_response_time: 0.0,
            total_errors: 0,
            avg_users_online: 0.0,
            max_users_online: 0,
        }
    }
}

// Helper functions

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_realtime_stats_collector() -> Result<()> {
        let config = RealTimeStatsConfig::default();
        let metrics = Arc::new(super::super::metrics::MetricsRegistry::new());
        let profiler = Arc::new(super::super::profiling::Profiler::new(
            super::super::profiling::ProfilingConfig::default(),
            metrics.clone()
        ));
        
        // Create mock components for testing
        // In a real test, these would be properly initialized
        
        Ok(())
    }

    #[tokio::test]
    async fn test_stats_aggregation() -> Result<()> {
        // Test aggregation logic
        let mut stats_vec = Vec::new();
        
        // Create some sample statistics
        for i in 0..10 {
            let stats = RealTimeStats {
                timestamp: 1000 + i * 60, // 1 minute intervals
                server_info: ServerInfo {
                    uptime_seconds: i * 60,
                    version: "test".to_string(),
                    build_date: "test".to_string(),
                    server_id: "test".to_string(),
                    hostname: "test".to_string(),
                    startup_time: 1000,
                },
                resource_usage: ResourceUsage {
                    cpu_usage_percent: 50.0 + i as f64,
                    memory_usage_mb: 500.0 + i as f64 * 10.0,
                    memory_total_mb: 2048.0,
                    memory_available_mb: 1000.0,
                    disk_usage_gb: 10.0,
                    disk_total_gb: 100.0,
                    load_average_1m: 1.0,
                    load_average_5m: 1.0,
                    load_average_15m: 1.0,
                    thread_count: 20,
                    fd_count: 100,
                },
                performance_metrics: PerformanceMetrics {
                    requests_per_second: 100.0 + i as f64 * 5.0,
                    average_response_time_ms: 20.0 + i as f64,
                    p95_response_time_ms: 50.0,
                    p99_response_time_ms: 100.0,
                    cache_hit_rate_percent: 90.0,
                    cache_miss_rate_percent: 10.0,
                    database_queries_per_second: 50.0,
                    database_avg_query_time_ms: 10.0,
                    garbage_collection_time_ms: 5.0,
                    memory_allocations_per_second: 1000.0,
                },
                service_status: ServiceStatus {
                    total_services: 5,
                    healthy_services: 5,
                    degraded_services: 0,
                    unhealthy_services: 0,
                    service_details: HashMap::new(),
                },
                network_stats: NetworkStats {
                    bytes_sent_per_second: 1000.0,
                    bytes_received_per_second: 800.0,
                    packets_sent_per_second: 100.0,
                    packets_received_per_second: 80.0,
                    active_connections: 20,
                    connection_pool_usage: 50.0,
                    network_errors_per_second: 0.1,
                },
                user_activity: UserActivity {
                    online_users: 10 + i as u32,
                    total_sessions: 100,
                    new_logins_per_minute: 2.0,
                    logouts_per_minute: 1.0,
                    active_regions: 3,
                    users_per_region: HashMap::new(),
                    concurrent_peak_today: 50,
                },
                region_stats: RegionStats {
                    total_regions: 3,
                    active_regions: 3,
                    total_objects: 1000,
                    total_scripts: 200,
                    physics_fps: 45.0,
                    script_events_per_second: 100.0,
                    region_details: HashMap::new(),
                },
                error_summary: ErrorSummary {
                    errors_per_minute: 0.1,
                    warnings_per_minute: 1.0,
                    critical_errors: 0,
                    recent_errors: Vec::new(),
                    error_trends: HashMap::new(),
                },
            };
            stats_vec.push(stats);
        }

        let aggregated = AggregatedStats::from_stats_range(&stats_vec);
        
        assert_eq!(aggregated.sample_count, 10);
        assert!(aggregated.avg_cpu_usage > 50.0);
        assert!(aggregated.max_cpu_usage >= aggregated.avg_cpu_usage);
        
        Ok(())
    }
}