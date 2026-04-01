//! Metrics collection for OpenSim
//! 
//! Collects and stores system performance metrics, network statistics,
//! and custom application metrics.

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::debug;

use super::SystemMetrics;

/// Metrics collector for gathering system performance data
#[derive(Debug)]
pub struct MetricsCollector {
    /// Historical metrics data
    metrics_history: Arc<RwLock<VecDeque<SystemMetrics>>>,
    /// Custom metrics storage
    custom_metrics: Arc<RwLock<HashMap<String, CustomMetric>>>,
    /// Maximum number of historical metrics to retain
    max_history_size: usize,
    /// Last collection time
    last_collection: Arc<RwLock<Instant>>,
    /// System startup time for uptime calculation
    startup_time: Instant,
    /// Request counters for metrics
    request_count: Arc<RwLock<u64>>,
    error_count: Arc<RwLock<u64>>,
    cache_hits: Arc<RwLock<u64>>,
    cache_misses: Arc<RwLock<u64>>,
    response_times: Arc<RwLock<VecDeque<Duration>>>,
}

/// Custom metric with tags and values
#[derive(Debug, Clone)]
pub struct CustomMetric {
    /// Metric name
    pub name: String,
    /// Current value
    pub value: f64,
    /// Tags for categorization
    pub tags: HashMap<String, String>,
    /// Last update time
    pub last_update: Instant,
    /// Metric type
    pub metric_type: MetricType,
}

/// Type of metric
#[derive(Debug, Clone)]
pub enum MetricType {
    /// Counter that only increases
    Counter,
    /// Gauge that can go up or down
    Gauge,
    /// Histogram for distribution analysis
    Histogram,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(max_history_size: usize) -> Result<Self> {
        Ok(Self {
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
            max_history_size,
            last_collection: Arc::new(RwLock::new(Instant::now())),
            startup_time: Instant::now(),
            request_count: Arc::new(RwLock::new(0)),
            error_count: Arc::new(RwLock::new(0)),
            cache_hits: Arc::new(RwLock::new(0)),
            cache_misses: Arc::new(RwLock::new(0)),
            response_times: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
        })
    }

    /// Record a request for metrics tracking
    pub async fn record_request(&self) {
        *self.request_count.write().await += 1;
    }

    /// Record an error for metrics tracking
    pub async fn record_error(&self) {
        *self.error_count.write().await += 1;
    }

    /// Record a cache hit
    pub async fn record_cache_hit(&self) {
        *self.cache_hits.write().await += 1;
    }

    /// Record a cache miss
    pub async fn record_cache_miss(&self) {
        *self.cache_misses.write().await += 1;
    }

    /// Record a response time
    pub async fn record_response_time(&self, duration: Duration) {
        let mut times = self.response_times.write().await;
        times.push_back(duration);
        while times.len() > 1000 {
            times.pop_front();
        }
    }

    /// Collect current system metrics
    pub async fn collect_system_metrics(&self) -> Result<()> {
        let metrics = self.gather_system_metrics().await?;
        
        let mut history = self.metrics_history.write().await;
        history.push_back(metrics);
        
        // Maintain history size limit
        while history.len() > self.max_history_size {
            history.pop_front();
        }
        
        *self.last_collection.write().await = Instant::now();
        
        debug!("Collected system metrics");
        Ok(())
    }

    /// Gather current system metrics
    async fn gather_system_metrics(&self) -> Result<SystemMetrics> {
        // In a real implementation, this would gather actual system metrics
        // For now, we'll use placeholder values
        
        let cpu_usage = self.get_cpu_usage().await?;
        let memory_usage = self.get_memory_usage().await?;
        let network_connections = self.get_network_connections().await?;
        let active_regions = self.get_active_regions().await?;
        let physics_fps = self.get_physics_fps().await?;
        let asset_cache_hit_rate = self.get_asset_cache_hit_rate().await?;
        let avg_response_time = self.get_avg_response_time().await?;
        let error_rate = self.get_error_rate().await?;
        let uptime = self.get_uptime().await?;

        Ok(SystemMetrics {
            cpu_usage,
            memory_usage,
            network_connections,
            active_regions,
            physics_fps,
            asset_cache_hit_rate,
            avg_response_time,
            error_rate,
            uptime,
        })
    }

    /// Get current CPU usage percentage
    async fn get_cpu_usage(&self) -> Result<f64> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let output = Command::new("ps")
                .args(["-o", "%cpu", "-p", &std::process::id().to_string()])
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    if let Ok(cpu) = line.trim().parse::<f64>() {
                        return Ok(cpu);
                    }
                }
            }
            Ok(0.0)
        }
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(stat) = fs::read_to_string(format!("/proc/{}/stat", std::process::id())) {
                let parts: Vec<&str> = stat.split_whitespace().collect();
                if parts.len() > 14 {
                    let utime: u64 = parts[13].parse().unwrap_or(0);
                    let stime: u64 = parts[14].parse().unwrap_or(0);
                    let total_time = utime + stime;
                    let uptime = self.startup_time.elapsed().as_secs();
                    if uptime > 0 {
                        return Ok((total_time as f64 / uptime as f64) * 100.0 / 100.0);
                    }
                }
            }
            Ok(0.0)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Ok(0.0)
        }
    }

    /// Get current memory usage in bytes
    async fn get_memory_usage(&self) -> Result<u64> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let output = Command::new("ps")
                .args(["-o", "rss=", "-p", &std::process::id().to_string()])
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(rss_kb) = stdout.trim().parse::<u64>() {
                    return Ok(rss_kb * 1024);
                }
            }
            Ok(0)
        }
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string(format!("/proc/{}/status", std::process::id())) {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(kb) = parts[1].parse::<u64>() {
                                return Ok(kb * 1024);
                            }
                        }
                    }
                }
            }
            Ok(0)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Ok(0)
        }
    }

    /// Get number of network connections
    async fn get_network_connections(&self) -> Result<usize> {
        // Return 0 for now since no actual connections are being tracked
        // In a real system, this would count active TCP connections
        Ok(0)
    }

    /// Get number of active regions
    async fn get_active_regions(&self) -> Result<usize> {
        // Return 1 as configured in main.rs (default region)
        // In a real system, this would count active region instances
        Ok(1)
    }

    /// Get physics simulation FPS
    async fn get_physics_fps(&self) -> Result<f64> {
        // Placeholder implementation
        // In a real system, this would measure physics step timing
        Ok(60.0)
    }

    /// Get asset cache hit rate
    async fn get_asset_cache_hit_rate(&self) -> Result<f64> {
        let hits = *self.cache_hits.read().await;
        let misses = *self.cache_misses.read().await;
        let total = hits + misses;

        if total == 0 {
            Ok(1.0)
        } else {
            Ok(hits as f64 / total as f64)
        }
    }

    /// Get average response time
    async fn get_avg_response_time(&self) -> Result<Duration> {
        let times = self.response_times.read().await;

        if times.is_empty() {
            Ok(Duration::from_millis(0))
        } else {
            let total_nanos: u128 = times.iter().map(|d| d.as_nanos()).sum();
            let avg_nanos = total_nanos / times.len() as u128;
            Ok(Duration::from_nanos(avg_nanos as u64))
        }
    }

    /// Get error rate
    async fn get_error_rate(&self) -> Result<f64> {
        let errors = *self.error_count.read().await;
        let requests = *self.request_count.read().await;

        if requests == 0 {
            Ok(0.0)
        } else {
            Ok(errors as f64 / requests as f64)
        }
    }

    /// Get system uptime
    async fn get_uptime(&self) -> Result<Duration> {
        Ok(self.startup_time.elapsed())
    }

    /// Record a custom metric
    pub async fn record_custom_metric(
        &self,
        name: &str,
        value: f64,
        tags: HashMap<String, String>,
    ) -> Result<()> {
        let metric = CustomMetric {
            name: name.to_string(),
            value,
            tags,
            last_update: Instant::now(),
            metric_type: MetricType::Gauge,
        };

        self.custom_metrics.write().await.insert(name.to_string(), metric);
        debug!("Recorded custom metric: {} = {}", name, value);
        Ok(())
    }

    /// Increment a counter metric
    pub async fn increment_counter(&self, name: &str, tags: HashMap<String, String>) -> Result<()> {
        let mut metrics = self.custom_metrics.write().await;
        
        if let Some(metric) = metrics.get_mut(name) {
            metric.value += 1.0;
            metric.last_update = Instant::now();
        } else {
            let metric = CustomMetric {
                name: name.to_string(),
                value: 1.0,
                tags,
                last_update: Instant::now(),
                metric_type: MetricType::Counter,
            };
            metrics.insert(name.to_string(), metric);
        }

        debug!("Incremented counter: {}", name);
        Ok(())
    }

    /// Record a gauge metric - EADS fix for compilation errors
    pub async fn record_gauge(&self, name: &str, value: f64, tags: HashMap<String, String>) -> Result<()> {
        self.record_custom_metric(name, value, tags).await
    }

    /// Record a histogram metric - EADS fix for compilation errors
    pub async fn record_histogram(&self, name: &str, value: f64, tags: HashMap<String, String>) -> Result<()> {
        // For now, treat histograms as gauges - in a full implementation, 
        // this would track distribution statistics
        self.record_custom_metric(name, value, tags).await
    }

    /// Get current system metrics
    pub async fn get_current_metrics(&self) -> Result<SystemMetrics> {
        self.gather_system_metrics().await
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self) -> Vec<SystemMetrics> {
        self.metrics_history.read().await.clone().into()
    }

    /// Get number of metrics collected
    pub async fn get_metrics_count(&self) -> usize {
        self.metrics_history.read().await.len()
    }

    /// Get custom metrics
    pub async fn get_custom_metrics(&self) -> HashMap<String, CustomMetric> {
        self.custom_metrics.read().await.clone()
    }

    /// Clear old metrics
    pub async fn clear_old_metrics(&self, older_than: Duration) -> Result<()> {
        let mut history = self.metrics_history.write().await;
        let cutoff_time = older_than;
        
        history.retain(|metric| metric.uptime < cutoff_time);
        
        debug!("Cleared metrics older than {:?}", older_than);
        Ok(())
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> Result<String> {
        let mut output = String::new();
        
        // Export system metrics
        let current_metrics = self.get_current_metrics().await?;
        output.push_str(&format!("# HELP opensim_cpu_usage CPU usage percentage\n"));
        output.push_str(&format!("# TYPE opensim_cpu_usage gauge\n"));
        output.push_str(&format!("opensim_cpu_usage {}\n", current_metrics.cpu_usage));
        
        output.push_str(&format!("# HELP opensim_memory_usage Memory usage in bytes\n"));
        output.push_str(&format!("# TYPE opensim_memory_usage gauge\n"));
        output.push_str(&format!("opensim_memory_usage {}\n", current_metrics.memory_usage));
        
        output.push_str(&format!("# HELP opensim_network_connections Number of network connections\n"));
        output.push_str(&format!("# TYPE opensim_network_connections gauge\n"));
        output.push_str(&format!("opensim_network_connections {}\n", current_metrics.network_connections));
        
        // Export custom metrics
        let custom_metrics = self.get_custom_metrics().await;
        for (name, metric) in custom_metrics {
            let metric_type = match metric.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
            };
            
            output.push_str(&format!("# HELP opensim_{} Custom metric {}\n", name, name));
            output.push_str(&format!("# TYPE opensim_{} {}\n", name, metric_type));
            
            let tags_str = if metric.tags.is_empty() {
                String::new()
            } else {
                let tag_pairs: Vec<String> = metric.tags
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", tag_pairs.join(","))
            };
            
            output.push_str(&format!("opensim_{}{} {}\n", name, tags_str, metric.value));
        }
        
        Ok(output)
    }

    /// Record AI avatar interaction metrics
    pub async fn record_ai_interaction(&self, avatar_id: uuid::Uuid, processing_time_ms: u64) {
        let mut tags = HashMap::new();
        tags.insert("avatar_id".to_string(), avatar_id.to_string());
        
        let _ = self.record_custom_metric("ai_avatar_interactions_total", 1.0, tags.clone()).await;
        let _ = self.record_custom_metric("ai_avatar_processing_time_ms", processing_time_ms as f64, tags).await;
    }

    /// Record NPC behavior generation metrics
    pub async fn record_npc_behavior_generation(&self, npc_id: uuid::Uuid, processing_time_ms: u64) {
        let mut tags = HashMap::new();
        tags.insert("npc_id".to_string(), npc_id.to_string());
        
        let _ = self.record_custom_metric("ai_npc_behavior_generations_total", 1.0, tags.clone()).await;
        let _ = self.record_custom_metric("ai_npc_processing_time_ms", processing_time_ms as f64, tags).await;
    }

    /// Record content generation queued
    pub async fn record_content_generation_queued(&self, job_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("job_id".to_string(), job_id.to_string());
        
        let _ = self.record_custom_metric("ai_content_generation_queued_total", 1.0, tags).await;
    }

    /// Record content generation completed
    pub async fn record_content_generation_completed(&self, job_id: uuid::Uuid, generation_time_ms: u64) {
        let mut tags = HashMap::new();
        tags.insert("job_id".to_string(), job_id.to_string());
        
        let _ = self.record_custom_metric("ai_content_generation_completed_total", 1.0, tags.clone()).await;
        let _ = self.record_custom_metric("ai_content_generation_time_ms", generation_time_ms as f64, tags).await;
    }

    /// Record VR session started
    pub async fn record_vr_session_started(&self, user_id: uuid::Uuid, device_type: &str) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());
        tags.insert("device_type".to_string(), device_type.to_string());
        
        let _ = self.record_custom_metric("vr_sessions_started_total", 1.0, tags).await;
    }

    /// Record VR session ended
    pub async fn record_vr_session_ended(&self, user_id: uuid::Uuid, duration_seconds: u64) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());
        
        let _ = self.record_custom_metric("vr_sessions_ended_total", 1.0, tags.clone()).await;
        let _ = self.record_custom_metric("vr_session_duration_seconds", duration_seconds as f64, tags).await;
    }

    /// Record VR frame processed
    pub async fn record_vr_frame_processed(&self, session_id: uuid::Uuid, frame_time_ms: f32) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());
        
        let _ = self.record_custom_metric("vr_frames_processed_total", 1.0, tags.clone()).await;
        let _ = self.record_custom_metric("vr_frame_time_ms", frame_time_ms as f64, tags).await;
    }

    /// Record VR frame rendered
    pub async fn record_vr_frame_rendered(&self, session_id: uuid::Uuid, render_time_ms: f32) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());
        
        let _ = self.record_custom_metric("vr_frames_rendered_total", 1.0, tags.clone()).await;
        let _ = self.record_custom_metric("vr_render_time_ms", render_time_ms as f64, tags).await;
    }

    /// Record OpenXR runtime initialized
    pub async fn record_openxr_runtime_initialized(&self) {
        let _ = self.record_custom_metric("openxr_runtime_initializations_total", 1.0, HashMap::new()).await;
    }

    /// Record OpenXR session created
    pub async fn record_openxr_session_created(&self, session_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());
        
        let _ = self.record_custom_metric("openxr_sessions_created_total", 1.0, tags).await;
    }

    /// Record OpenXR session destroyed
    pub async fn record_openxr_session_destroyed(&self, session_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());
        
        let _ = self.record_custom_metric("openxr_sessions_destroyed_total", 1.0, tags).await;
    }

    /// Record OpenXR frame submitted
    pub async fn record_openxr_frame_submitted(&self, session_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());
        
        let _ = self.record_custom_metric("openxr_frames_submitted_total", 1.0, tags).await;
    }

    /// Record haptic user initialized
    pub async fn record_haptic_user_initialized(&self, user_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());
        
        let _ = self.record_custom_metric("haptic_users_initialized_total", 1.0, tags).await;
    }

    /// Record haptic user cleanup
    pub async fn record_haptic_user_cleanup(&self, user_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());
        
        let _ = self.record_custom_metric("haptic_users_cleanup_total", 1.0, tags).await;
    }

    /// Record haptic frame generated
    pub async fn record_haptic_frame_generated(&self, user_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());
        
        let _ = self.record_custom_metric("haptic_frames_generated_total", 1.0, tags).await;
    }

    /// Record spatial audio session created
    pub async fn record_spatial_audio_session_created(&self, session_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());
        
        let _ = self.record_custom_metric("spatial_audio_sessions_created_total", 1.0, tags).await;
    }

    /// Record spatial audio session destroyed
    pub async fn record_spatial_audio_session_destroyed(&self, session_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());
        
        let _ = self.record_custom_metric("spatial_audio_sessions_destroyed_total", 1.0, tags).await;
    }

    /// Record spatial audio frame processed
    pub async fn record_spatial_audio_frame_processed(&self, session_id: uuid::Uuid) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());
        
        let _ = self.record_custom_metric("spatial_audio_frames_processed_total", 1.0, tags).await;
    }
} 