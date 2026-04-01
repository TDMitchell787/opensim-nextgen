//! Advanced performance monitoring and metrics collection

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Metric types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
    Timer,
}

/// Individual metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: u64,
    pub labels: HashMap<String, String>,
}

/// Histogram bucket for latency measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Histogram metric data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramData {
    pub buckets: Vec<HistogramBucket>,
    pub count: u64,
    pub sum: f64,
}

/// Summary quantile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryQuantile {
    pub quantile: f64,
    pub value: f64,
}

/// Summary metric data  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryData {
    pub quantiles: Vec<SummaryQuantile>,
    pub count: u64,
    pub sum: f64,
}

/// Metric definition and current value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub help: String,
    pub metric_type: MetricType,
    pub labels: HashMap<String, String>,
    pub value: f64,
    pub histogram_data: Option<HistogramData>,
    pub summary_data: Option<SummaryData>,
    pub last_updated: u64,
}

/// Performance metric registry
pub struct MetricsRegistry {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
    histogram_buckets: Vec<f64>,
    summary_quantiles: Vec<f64>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        // Default histogram buckets (in milliseconds for latency)
        let histogram_buckets = vec![
            0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
        ];
        
        // Default summary quantiles
        let summary_quantiles = vec![0.5, 0.9, 0.95, 0.99, 0.999];
        
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            histogram_buckets,
            summary_quantiles,
        }
    }

    /// Register a new counter metric
    pub async fn register_counter(&self, name: &str, help: &str, labels: HashMap<String, String>) -> Result<()> {
        self.register_metric(name, help, MetricType::Counter, labels).await
    }

    /// Register a new gauge metric
    pub async fn register_gauge(&self, name: &str, help: &str, labels: HashMap<String, String>) -> Result<()> {
        self.register_metric(name, help, MetricType::Gauge, labels).await
    }

    /// Register a new histogram metric
    pub async fn register_histogram(&self, name: &str, help: &str, labels: HashMap<String, String>) -> Result<()> {
        let mut metric = Metric {
            name: name.to_string(),
            help: help.to_string(),
            metric_type: MetricType::Histogram,
            labels,
            value: 0.0,
            histogram_data: Some(HistogramData {
                buckets: self.histogram_buckets.iter().map(|&upper_bound| HistogramBucket {
                    upper_bound,
                    count: 0,
                }).collect(),
                count: 0,
                sum: 0.0,
            }),
            summary_data: None,
            last_updated: current_timestamp(),
        };

        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), metric);
        
        debug!("Registered histogram metric: {}", name);
        Ok(())
    }

    /// Register a new summary metric
    pub async fn register_summary(&self, name: &str, help: &str, labels: HashMap<String, String>) -> Result<()> {
        let mut metric = Metric {
            name: name.to_string(),
            help: help.to_string(),
            metric_type: MetricType::Summary,
            labels,
            value: 0.0,
            histogram_data: None,
            summary_data: Some(SummaryData {
                quantiles: self.summary_quantiles.iter().map(|&quantile| SummaryQuantile {
                    quantile,
                    value: 0.0,
                }).collect(),
                count: 0,
                sum: 0.0,
            }),
            last_updated: current_timestamp(),
        };

        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), metric);
        
        debug!("Registered summary metric: {}", name);
        Ok(())
    }

    /// Increment a counter metric
    pub async fn increment_counter(&self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(name) {
            if metric.metric_type != MetricType::Counter {
                return Err(anyhow!("Metric {} is not a counter", name));
            }
            metric.value += value;
            metric.last_updated = current_timestamp();
        } else {
            return Err(anyhow!("Metric {} not found", name));
        }
        Ok(())
    }

    /// Set a gauge metric value
    pub async fn set_gauge(&self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(name) {
            if metric.metric_type != MetricType::Gauge {
                return Err(anyhow!("Metric {} is not a gauge", name));
            }
            metric.value = value;
            metric.last_updated = current_timestamp();
        } else {
            return Err(anyhow!("Metric {} not found", name));
        }
        Ok(())
    }

    /// Observe a value in a histogram
    pub async fn observe_histogram(&self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(name) {
            if metric.metric_type != MetricType::Histogram {
                return Err(anyhow!("Metric {} is not a histogram", name));
            }
            
            if let Some(ref mut histogram) = metric.histogram_data {
                histogram.count += 1;
                histogram.sum += value;
                
                // Update buckets
                for bucket in &mut histogram.buckets {
                    if value <= bucket.upper_bound {
                        bucket.count += 1;
                    }
                }
            }
            
            metric.last_updated = current_timestamp();
        } else {
            return Err(anyhow!("Metric {} not found", name));
        }
        Ok(())
    }

    /// Observe a value in a summary (simplified implementation)
    pub async fn observe_summary(&self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(name) {
            if metric.metric_type != MetricType::Summary {
                return Err(anyhow!("Metric {} is not a summary", name));
            }
            
            if let Some(ref mut summary) = metric.summary_data {
                summary.count += 1;
                summary.sum += value;
                // Note: This is a simplified implementation
                // Real quantile calculation would require a more sophisticated algorithm
            }
            
            metric.last_updated = current_timestamp();
        } else {
            return Err(anyhow!("Metric {} not found", name));
        }
        Ok(())
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.values().cloned().collect())
    }

    /// Get a specific metric
    pub async fn get_metric(&self, name: &str) -> Result<Option<Metric>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(name).cloned())
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> Result<String> {
        let metrics = self.metrics.read().await;
        let mut output = String::new();
        
        for metric in metrics.values() {
            // Add help text
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
            output.push_str(&format!("# TYPE {} {}\n", metric.name, metric_type_to_prometheus(&metric.metric_type)));
            
            match metric.metric_type {
                MetricType::Counter | MetricType::Gauge => {
                    let labels = format_labels(&metric.labels);
                    output.push_str(&format!("{}{} {}\n", metric.name, labels, metric.value));
                }
                MetricType::Histogram => {
                    if let Some(ref histogram) = metric.histogram_data {
                        let base_labels = format_labels(&metric.labels);
                        
                        // Export buckets
                        for bucket in &histogram.buckets {
                            let mut bucket_labels = metric.labels.clone();
                            bucket_labels.insert("le".to_string(), bucket.upper_bound.to_string());
                            let labels = format_labels(&bucket_labels);
                            output.push_str(&format!("{}_bucket{} {}\n", metric.name, labels, bucket.count));
                        }
                        
                        // Export +Inf bucket
                        let mut inf_labels = metric.labels.clone();
                        inf_labels.insert("le".to_string(), "+Inf".to_string());
                        let inf_labels_str = format_labels(&inf_labels);
                        output.push_str(&format!("{}_bucket{} {}\n", metric.name, inf_labels_str, histogram.count));
                        
                        // Export count and sum
                        output.push_str(&format!("{}_count{} {}\n", metric.name, base_labels, histogram.count));
                        output.push_str(&format!("{}_sum{} {}\n", metric.name, base_labels, histogram.sum));
                    }
                }
                MetricType::Summary => {
                    if let Some(ref summary) = metric.summary_data {
                        let base_labels = format_labels(&metric.labels);
                        
                        // Export quantiles
                        for quantile in &summary.quantiles {
                            let mut quantile_labels = metric.labels.clone();
                            quantile_labels.insert("quantile".to_string(), quantile.quantile.to_string());
                            let labels = format_labels(&quantile_labels);
                            output.push_str(&format!("{}{} {}\n", metric.name, labels, quantile.value));
                        }
                        
                        // Export count and sum
                        output.push_str(&format!("{}_count{} {}\n", metric.name, base_labels, summary.count));
                        output.push_str(&format!("{}_sum{} {}\n", metric.name, base_labels, summary.sum));
                    }
                }
                MetricType::Timer => {
                    // Timers are exported as gauges
                    let labels = format_labels(&metric.labels);
                    output.push_str(&format!("{}{} {}\n", metric.name, labels, metric.value));
                }
            }
            
            output.push('\n');
        }
        
        Ok(output)
    }

    /// Clear all metrics
    pub async fn clear(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
        info!("Cleared all metrics");
        Ok(())
    }

    async fn register_metric(&self, name: &str, help: &str, metric_type: MetricType, labels: HashMap<String, String>) -> Result<()> {
        let metric = Metric {
            name: name.to_string(),
            help: help.to_string(),
            metric_type,
            labels,
            value: 0.0,
            histogram_data: None,
            summary_data: None,
            last_updated: current_timestamp(),
        };

        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), metric);
        
        debug!("Registered metric: {}", name);
        Ok(())
    }
}

/// Timer utility for measuring execution time
pub struct Timer {
    start_time: Instant,
    metric_name: String,
    registry: Arc<MetricsRegistry>,
}

impl Timer {
    pub fn new(metric_name: String, registry: Arc<MetricsRegistry>) -> Self {
        Self {
            start_time: Instant::now(),
            metric_name,
            registry,
        }
    }

    /// Stop the timer and record the duration
    pub async fn stop(self) -> Result<Duration> {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_millis() as f64;
        
        // Record in histogram if it exists
        if let Ok(Some(metric)) = self.registry.get_metric(&self.metric_name).await {
            if metric.metric_type == MetricType::Histogram {
                self.registry.observe_histogram(&self.metric_name, duration_ms).await?;
            } else if metric.metric_type == MetricType::Summary {
                self.registry.observe_summary(&self.metric_name, duration_ms).await?;
            }
        }
        
        Ok(duration)
    }
}

/// Performance collector for system-level metrics
pub struct PerformanceCollector {
    registry: Arc<MetricsRegistry>,
    collection_interval: Duration,
    enabled: Arc<RwLock<bool>>,
}

impl PerformanceCollector {
    pub fn new(registry: Arc<MetricsRegistry>) -> Self {
        Self {
            registry,
            collection_interval: Duration::from_secs(15),
            enabled: Arc::new(RwLock::new(false)),
        }
    }

    /// Start collecting performance metrics
    pub async fn start(&self) -> Result<()> {
        info!("Starting performance metrics collection");
        
        *self.enabled.write().await = true;
        
        // Register system metrics
        self.register_system_metrics().await?;
        
        // Start collection loop
        let registry = self.registry.clone();
        let enabled = self.enabled.clone();
        let interval = self.collection_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            while *enabled.read().await {
                interval_timer.tick().await;
                
                if let Err(e) = Self::collect_system_metrics(&registry).await {
                    error!("Failed to collect system metrics: {}", e);
                }
            }
        });
        
        Ok(())
    }

    /// Stop collecting metrics
    pub async fn stop(&self) {
        info!("Stopping performance metrics collection");
        *self.enabled.write().await = false;
    }

    async fn register_system_metrics(&self) -> Result<()> {
        let labels = HashMap::new();
        
        // CPU metrics
        self.registry.register_gauge("system_cpu_usage_percent", "CPU usage percentage", labels.clone()).await?;
        self.registry.register_gauge("system_load_average_1m", "1-minute load average", labels.clone()).await?;
        
        // Memory metrics
        self.registry.register_gauge("system_memory_total_bytes", "Total system memory", labels.clone()).await?;
        self.registry.register_gauge("system_memory_used_bytes", "Used system memory", labels.clone()).await?;
        self.registry.register_gauge("system_memory_available_bytes", "Available system memory", labels.clone()).await?;
        
        // Process metrics
        self.registry.register_gauge("process_memory_rss_bytes", "Process RSS memory", labels.clone()).await?;
        self.registry.register_gauge("process_memory_vms_bytes", "Process VMS memory", labels.clone()).await?;
        self.registry.register_counter("process_cpu_seconds_total", "Process CPU time", labels.clone()).await?;
        
        // Network metrics
        self.registry.register_counter("network_bytes_sent_total", "Network bytes sent", labels.clone()).await?;
        self.registry.register_counter("network_bytes_received_total", "Network bytes received", labels.clone()).await?;
        self.registry.register_counter("network_packets_sent_total", "Network packets sent", labels.clone()).await?;
        self.registry.register_counter("network_packets_received_total", "Network packets received", labels.clone()).await?;
        
        // OpenSim specific metrics
        self.registry.register_gauge("opensim_active_regions", "Number of active regions", labels.clone()).await?;
        self.registry.register_gauge("opensim_active_avatars", "Number of active avatars", labels.clone()).await?;
        self.registry.register_gauge("opensim_active_connections", "Number of active connections", labels.clone()).await?;
        self.registry.register_counter("opensim_events_processed_total", "Total events processed", labels.clone()).await?;
        
        Ok(())
    }

    async fn collect_system_metrics(registry: &Arc<MetricsRegistry>) -> Result<()> {
        // In a real implementation, these would collect actual system metrics
        // For now, we'll use placeholder values
        
        // Simulated CPU usage
        let cpu_usage = rand::random::<f64>() * 100.0;
        registry.set_gauge("system_cpu_usage_percent", cpu_usage).await?;
        
        // Simulated memory metrics
        let total_memory = 8_000_000_000.0; // 8GB
        let used_memory = total_memory * (0.3 + rand::random::<f64>() * 0.4); // 30-70%
        let available_memory = total_memory - used_memory;
        
        registry.set_gauge("system_memory_total_bytes", total_memory).await?;
        registry.set_gauge("system_memory_used_bytes", used_memory).await?;
        registry.set_gauge("system_memory_available_bytes", available_memory).await?;
        
        // Simulated process metrics
        let process_memory = 100_000_000.0 + rand::random::<f64>() * 50_000_000.0; // 100-150MB
        registry.set_gauge("process_memory_rss_bytes", process_memory).await?;
        
        debug!("Collected system metrics");
        Ok(())
    }
}

// Helper functions

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn metric_type_to_prometheus(metric_type: &MetricType) -> &'static str {
    match metric_type {
        MetricType::Counter => "counter",
        MetricType::Gauge => "gauge",
        MetricType::Histogram => "histogram",
        MetricType::Summary => "summary",
        MetricType::Timer => "gauge",
    }
}

fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        return String::new();
    }
    
    let mut label_pairs: Vec<_> = labels.iter().collect();
    label_pairs.sort_by_key(|(k, _)| *k);
    
    let formatted: Vec<String> = label_pairs
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, v))
        .collect();
    
    format!("{{{}}}", formatted.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_registry() -> Result<()> {
        let registry = MetricsRegistry::new();
        let labels = HashMap::new();
        
        // Test counter
        registry.register_counter("test_counter", "Test counter", labels.clone()).await?;
        registry.increment_counter("test_counter", 5.0).await?;
        
        let metric = registry.get_metric("test_counter").await?;
        assert!(metric.is_some());
        assert_eq!(metric.unwrap().value, 5.0);
        
        // Test gauge
        registry.register_gauge("test_gauge", "Test gauge", labels.clone()).await?;
        registry.set_gauge("test_gauge", 42.0).await?;
        
        let metric = registry.get_metric("test_gauge").await?;
        assert!(metric.is_some());
        assert_eq!(metric.unwrap().value, 42.0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_histogram() -> Result<()> {
        let registry = MetricsRegistry::new();
        let labels = HashMap::new();
        
        registry.register_histogram("test_histogram", "Test histogram", labels).await?;
        
        // Observe some values
        registry.observe_histogram("test_histogram", 1.5).await?;
        registry.observe_histogram("test_histogram", 5.2).await?;
        registry.observe_histogram("test_histogram", 15.8).await?;
        
        let metric = registry.get_metric("test_histogram").await?;
        assert!(metric.is_some());
        
        let histogram_data = metric.unwrap().histogram_data;
        assert!(histogram_data.is_some());
        
        let histogram = histogram_data.unwrap();
        assert_eq!(histogram.count, 3);
        assert_eq!(histogram.sum, 22.5);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_timer() -> Result<()> {
        let registry = Arc::new(MetricsRegistry::new());
        let labels = HashMap::new();
        
        registry.register_histogram("test_timer", "Test timer", labels).await?;
        
        let timer = Timer::new("test_timer".to_string(), registry.clone());
        tokio::time::sleep(Duration::from_millis(10)).await;
        let duration = timer.stop().await?;
        
        assert!(duration >= Duration::from_millis(10));
        
        let metric = registry.get_metric("test_timer").await?;
        assert!(metric.is_some());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_prometheus_export() -> Result<()> {
        let registry = MetricsRegistry::new();
        let labels = HashMap::from([("service".to_string(), "test".to_string())]);
        
        registry.register_counter("test_counter", "Test counter", labels).await?;
        registry.increment_counter("test_counter", 42.0).await?;
        
        let prometheus_output = registry.export_prometheus().await?;
        
        assert!(prometheus_output.contains("# HELP test_counter Test counter"));
        assert!(prometheus_output.contains("# TYPE test_counter counter"));
        assert!(prometheus_output.contains("test_counter{service=\"test\"} 42"));
        
        Ok(())
    }
}