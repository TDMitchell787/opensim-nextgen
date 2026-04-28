//! Performance profiling and analysis tools

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::metrics::MetricsRegistry;

/// Profiling sample representing a function call or operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSample {
    pub name: String,
    pub duration_ns: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub thread_id: String,
    pub stack_trace: Vec<String>,
    pub tags: HashMap<String, String>,
}

/// Aggregated profiling statistics for a function or operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStats {
    pub name: String,
    pub total_calls: u64,
    pub total_duration_ns: u64,
    pub min_duration_ns: u64,
    pub max_duration_ns: u64,
    pub avg_duration_ns: u64,
    pub p50_duration_ns: u64,
    pub p95_duration_ns: u64,
    pub p99_duration_ns: u64,
    pub samples_per_second: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    pub enabled: bool,
    pub sample_rate: f64, // 0.0 to 1.0
    pub max_samples: usize,
    pub max_stack_depth: usize,
    pub collection_interval: Duration,
    pub export_interval: Duration,
    pub flame_graph_enabled: bool,
    pub memory_profiling: bool,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate: 0.1, // 10% sampling
            max_samples: 100_000,
            max_stack_depth: 32,
            collection_interval: Duration::from_millis(100),
            export_interval: Duration::from_secs(60),
            flame_graph_enabled: false,
            memory_profiling: false,
        }
    }
}

/// Performance profiler
pub struct Profiler {
    config: ProfilingConfig,
    samples: Arc<RwLock<Vec<ProfileSample>>>,
    aggregated_stats: Arc<RwLock<HashMap<String, ProfileStats>>>,
    metrics_registry: Arc<MetricsRegistry>,
    active_spans: Arc<RwLock<HashMap<String, Instant>>>,
}

impl Profiler {
    /// Create a new profiler
    pub fn new(config: ProfilingConfig, metrics_registry: Arc<MetricsRegistry>) -> Self {
        Self {
            config,
            samples: Arc::new(RwLock::new(Vec::new())),
            aggregated_stats: Arc::new(RwLock::new(HashMap::new())),
            metrics_registry,
            active_spans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start profiling with periodic aggregation
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            debug!("Profiler disabled, not starting");
            return Ok(());
        }

        info!("Starting performance profiler");

        // Register profiling metrics
        self.register_profiling_metrics().await?;

        // Start aggregation task
        self.start_aggregation_task().await;

        // Start export task
        self.start_export_task().await;

        Ok(())
    }

    /// Begin profiling a named operation
    pub async fn begin_span(&self, name: &str) -> Result<ProfileSpan> {
        if !self.config.enabled {
            return Ok(ProfileSpan::disabled());
        }

        // Sample based on configured rate
        if rand::random::<f64>() > self.config.sample_rate {
            return Ok(ProfileSpan::disabled());
        }

        let span_id = format!("{}:{}", name, uuid::Uuid::new_v4());
        let start_time = Instant::now();

        self.active_spans
            .write()
            .await
            .insert(span_id.clone(), start_time);

        Ok(ProfileSpan {
            id: span_id,
            name: name.to_string(),
            start_time,
            profiler: Some(Arc::new(self.clone())),
            tags: HashMap::new(),
        })
    }

    /// Record a completed profile sample
    pub async fn record_sample(&self, sample: ProfileSample) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut samples = self.samples.write().await;

        // Enforce sample limit
        if samples.len() >= self.config.max_samples {
            // Remove oldest 10% of samples
            let remove_count = self.config.max_samples / 10;
            samples.drain(0..remove_count);
        }

        samples.push(sample);

        Ok(())
    }

    /// Get current profiling statistics
    pub async fn get_stats(&self) -> Result<Vec<ProfileStats>> {
        let stats = self.aggregated_stats.read().await;
        Ok(stats.values().cloned().collect())
    }

    /// Get profiling statistics for a specific function
    pub async fn get_function_stats(&self, name: &str) -> Result<Option<ProfileStats>> {
        let stats = self.aggregated_stats.read().await;
        Ok(stats.get(name).cloned())
    }

    /// Get raw samples (for debugging)
    pub async fn get_samples(&self, limit: Option<usize>) -> Result<Vec<ProfileSample>> {
        let samples = self.samples.read().await;
        let limit = limit.unwrap_or(1000);

        Ok(samples.iter().rev().take(limit).cloned().collect())
    }

    /// Generate a flame graph representation
    pub async fn generate_flame_graph(&self) -> Result<String> {
        if !self.config.flame_graph_enabled {
            return Err(anyhow!("Flame graph generation is disabled"));
        }

        let samples = self.samples.read().await;
        let mut flame_graph_data = HashMap::new();

        for sample in samples.iter() {
            let stack_key = sample.stack_trace.join(";");
            *flame_graph_data.entry(stack_key).or_insert(0u64) += sample.duration_ns;
        }

        // Convert to flame graph format
        let mut flame_graph = String::new();
        for (stack, duration) in flame_graph_data {
            flame_graph.push_str(&format!("{} {}\n", stack, duration));
        }

        Ok(flame_graph)
    }

    /// Export profiling data
    pub async fn export_data(&self) -> Result<ProfilingExport> {
        let stats = self.get_stats().await?;
        let samples = self.get_samples(Some(10000)).await?; // Last 10k samples

        Ok(ProfilingExport {
            stats,
            recent_samples: samples,
            config: self.config.clone(),
            export_timestamp: chrono::Utc::now(),
        })
    }

    /// Clear all profiling data
    pub async fn clear(&self) -> Result<()> {
        self.samples.write().await.clear();
        self.aggregated_stats.write().await.clear();
        self.active_spans.write().await.clear();

        info!("Cleared all profiling data");
        Ok(())
    }

    async fn register_profiling_metrics(&self) -> Result<()> {
        let labels = HashMap::new();

        self.metrics_registry
            .register_gauge(
                "profiler_active_samples",
                "Number of active profiling samples",
                labels.clone(),
            )
            .await?;

        self.metrics_registry
            .register_gauge(
                "profiler_active_spans",
                "Number of active profiling spans",
                labels.clone(),
            )
            .await?;

        self.metrics_registry
            .register_counter(
                "profiler_samples_total",
                "Total number of profiling samples recorded",
                labels.clone(),
            )
            .await?;

        self.metrics_registry
            .register_histogram(
                "profiler_sample_duration_ms",
                "Duration of profiled operations in milliseconds",
                labels.clone(),
            )
            .await?;

        Ok(())
    }

    async fn start_aggregation_task(&self) {
        let samples = self.samples.clone();
        let stats = self.aggregated_stats.clone();
        let metrics = self.metrics_registry.clone();
        let interval = self.config.collection_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if let Err(e) = Self::aggregate_samples(&samples, &stats, &metrics).await {
                    error!("Failed to aggregate profiling samples: {}", e);
                }
            }
        });
    }

    async fn start_export_task(&self) {
        let profiler = Arc::new(self.clone());
        let interval = self.config.export_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                match profiler.export_data().await {
                    Ok(export) => {
                        debug!(
                            "Exported profiling data: {} stats, {} samples",
                            export.stats.len(),
                            export.recent_samples.len()
                        );
                    }
                    Err(e) => {
                        error!("Failed to export profiling data: {}", e);
                    }
                }
            }
        });
    }

    async fn aggregate_samples(
        samples: &Arc<RwLock<Vec<ProfileSample>>>,
        stats: &Arc<RwLock<HashMap<String, ProfileStats>>>,
        metrics: &Arc<MetricsRegistry>,
    ) -> Result<()> {
        let samples_guard = samples.read().await;
        let mut stats_guard = stats.write().await;

        // Update metrics
        let _ = metrics
            .set_gauge("profiler_active_samples", samples_guard.len() as f64)
            .await;

        // Group samples by function name
        let mut function_samples: HashMap<String, Vec<&ProfileSample>> = HashMap::new();
        for sample in samples_guard.iter() {
            function_samples
                .entry(sample.name.clone())
                .or_default()
                .push(sample);
        }

        // Aggregate statistics for each function
        for (function_name, samples) in function_samples {
            let durations: Vec<u64> = samples.iter().map(|s| s.duration_ns).collect();

            if durations.is_empty() {
                continue;
            }

            let total_calls = samples.len() as u64;
            let total_duration: u64 = durations.iter().sum();
            let min_duration = *durations.iter().min().unwrap();
            let max_duration = *durations.iter().max().unwrap();
            let avg_duration = total_duration / total_calls;

            // Calculate percentiles
            let mut sorted_durations = durations.clone();
            sorted_durations.sort_unstable();

            let p50_idx = (sorted_durations.len() as f64 * 0.5) as usize;
            let p95_idx = (sorted_durations.len() as f64 * 0.95) as usize;
            let p99_idx = (sorted_durations.len() as f64 * 0.99) as usize;

            let p50_duration = sorted_durations
                .get(p50_idx)
                .copied()
                .unwrap_or(avg_duration);
            let p95_duration = sorted_durations
                .get(p95_idx)
                .copied()
                .unwrap_or(max_duration);
            let p99_duration = sorted_durations
                .get(p99_idx)
                .copied()
                .unwrap_or(max_duration);

            // Calculate samples per second (simplified)
            let samples_per_second = total_calls as f64 / 60.0; // Approximate over last minute

            let function_stats = ProfileStats {
                name: function_name.clone(),
                total_calls,
                total_duration_ns: total_duration,
                min_duration_ns: min_duration,
                max_duration_ns: max_duration,
                avg_duration_ns: avg_duration,
                p50_duration_ns: p50_duration,
                p95_duration_ns: p95_duration,
                p99_duration_ns: p99_duration,
                samples_per_second,
                last_updated: chrono::Utc::now(),
            };

            stats_guard.insert(function_name, function_stats);

            // Record in metrics
            let duration_ms = avg_duration as f64 / 1_000_000.0;
            let _ = metrics
                .observe_histogram("profiler_sample_duration_ms", duration_ms)
                .await;
        }

        Ok(())
    }
}

impl Clone for Profiler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            samples: self.samples.clone(),
            aggregated_stats: self.aggregated_stats.clone(),
            metrics_registry: self.metrics_registry.clone(),
            active_spans: self.active_spans.clone(),
        }
    }
}

/// Profile span for measuring operation duration
pub struct ProfileSpan {
    id: String,
    name: String,
    start_time: Instant,
    profiler: Option<Arc<Profiler>>,
    tags: HashMap<String, String>,
}

impl ProfileSpan {
    fn disabled() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            start_time: Instant::now(),
            profiler: None,
            tags: HashMap::new(),
        }
    }

    /// Add a tag to this span
    pub fn tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    /// End the span and record the duration
    pub async fn end(self) -> Result<Duration> {
        let duration = self.start_time.elapsed();

        if let Some(profiler) = &self.profiler {
            // Remove from active spans
            profiler.active_spans.write().await.remove(&self.id);

            // Create sample
            let sample = ProfileSample {
                name: self.name.clone(),
                duration_ns: duration.as_nanos() as u64,
                start_time: system_time_as_nanos(self.start_time),
                end_time: system_time_as_nanos(self.start_time + duration),
                thread_id: format!("{:?}", std::thread::current().id()),
                stack_trace: vec![self.name], // Simplified stack trace
                tags: self.tags,
            };

            profiler.record_sample(sample).await?;

            // Update metrics
            let _ = profiler
                .metrics_registry
                .increment_counter("profiler_samples_total", 1.0)
                .await;
        }

        Ok(duration)
    }
}

/// Profiling data export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingExport {
    pub stats: Vec<ProfileStats>,
    pub recent_samples: Vec<ProfileSample>,
    pub config: ProfilingConfig,
    pub export_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Memory profiling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    pub total_allocated: u64,
    pub total_deallocated: u64,
    pub current_usage: u64,
    pub peak_usage: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub allocations_per_second: f64,
}

// Helper functions

fn system_time_as_nanos(instant: Instant) -> u64 {
    // This is a simplified conversion - in practice you'd want more accurate timing
    instant.elapsed().as_nanos() as u64
}

/// Convenience macro for profiling a block of code
#[macro_export]
macro_rules! profile {
    ($profiler:expr, $name:expr, $block:expr) => {{
        let span = $profiler.begin_span($name).await?;
        let result = $block;
        span.end().await?;
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::metrics::MetricsRegistry;

    #[tokio::test]
    async fn test_profiler_creation() -> Result<()> {
        let metrics = Arc::new(MetricsRegistry::new());
        let config = ProfilingConfig::default();
        let profiler = Profiler::new(config, metrics);

        profiler.start().await?;

        let stats = profiler.get_stats().await?;
        assert!(stats.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_profile_span() -> Result<()> {
        let metrics = Arc::new(MetricsRegistry::new());
        let config = ProfilingConfig {
            enabled: true,
            sample_rate: 1.0, // 100% sampling for test
            ..Default::default()
        };
        let profiler = Arc::new(Profiler::new(config, metrics));

        profiler.start().await?;

        let span = profiler.begin_span("test_function").await?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        let duration = span.end().await?;

        assert!(duration >= Duration::from_millis(10));

        // Wait for aggregation
        tokio::time::sleep(Duration::from_millis(200)).await;

        let stats = profiler.get_function_stats("test_function").await?;
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.total_calls, 1);
        assert!(stats.avg_duration_ns > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_profiler_sampling() -> Result<()> {
        let metrics = Arc::new(MetricsRegistry::new());
        let config = ProfilingConfig {
            enabled: true,
            sample_rate: 0.0, // 0% sampling
            ..Default::default()
        };
        let profiler = Arc::new(Profiler::new(config, metrics));

        profiler.start().await?;

        // This should be disabled due to 0% sampling
        let span = profiler.begin_span("test_function").await?;
        span.end().await?;

        let samples = profiler.get_samples(None).await?;
        assert!(samples.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_profiler_export() -> Result<()> {
        let metrics = Arc::new(MetricsRegistry::new());
        let config = ProfilingConfig {
            enabled: true,
            sample_rate: 1.0,
            ..Default::default()
        };
        let profiler = Arc::new(Profiler::new(config, metrics));

        profiler.start().await?;

        // Create some samples
        for i in 0..5 {
            let span = profiler.begin_span(&format!("test_function_{}", i)).await?;
            tokio::time::sleep(Duration::from_millis(1)).await;
            span.end().await?;
        }

        let export = profiler.export_data().await?;
        assert!(!export.recent_samples.is_empty());
        assert!(!export.stats.is_empty());

        Ok(())
    }
}
