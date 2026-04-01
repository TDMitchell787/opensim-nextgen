//! Performance profiling tools for OpenSim client SDKs

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use super::{
    generator::TargetLanguage,
    api_schema::APISchema,
};

/// Performance profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerConfig {
    pub enabled: bool,
    pub sampling_interval_ms: u64,
    pub max_samples: usize,
    pub profile_memory: bool,
    pub profile_network: bool,
    pub profile_cpu: bool,
    pub profile_method_calls: bool,
    pub output_directory: PathBuf,
    pub export_formats: Vec<ProfileExportFormat>,
    pub real_time_monitoring: bool,
    pub aggregation_window_seconds: u64,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_interval_ms: 100, // 100ms sampling
            max_samples: 10000,
            profile_memory: true,
            profile_network: true,
            profile_cpu: true,
            profile_method_calls: true,
            output_directory: PathBuf::from("./performance-profiles"),
            export_formats: vec![
                ProfileExportFormat::Json,
                ProfileExportFormat::Flamegraph,
                ProfileExportFormat::Csv,
            ],
            real_time_monitoring: true,
            aggregation_window_seconds: 60,
        }
    }
}

/// Export formats for profiling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileExportFormat {
    Json,
    Csv,
    Flamegraph,
    Chrome, // Chrome DevTools format
    Pprof,  // Google pprof format
    Html,
}

/// Performance sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub session_id: String,
    pub method_name: String,
    pub language: TargetLanguage,
    pub duration_ms: u64,
    pub memory_usage: MemoryUsage,
    pub network_stats: NetworkStats,
    pub cpu_stats: CpuStats,
    pub custom_metrics: HashMap<String, f64>,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub heap_used_mb: f64,
    pub heap_total_mb: f64,
    pub heap_peak_mb: f64,
    pub allocations_count: u64,
    pub deallocations_count: u64,
    pub gc_collections: u32,
    pub gc_time_ms: u64,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub requests_count: u32,
    pub requests_failed: u32,
    pub average_latency_ms: f64,
    pub connection_count: u32,
    pub dns_lookup_time_ms: Option<u64>,
    pub ssl_handshake_time_ms: Option<u64>,
}

/// CPU usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    pub cpu_usage_percent: f64,
    pub user_time_ms: u64,
    pub system_time_ms: u64,
    pub thread_count: u32,
    pub context_switches: u64,
}

/// Method call profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodProfile {
    pub method_name: String,
    pub call_count: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub p50_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub p99_duration_ms: f64,
    pub error_count: u64,
    pub error_rate_percent: f64,
}

/// Performance benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub language: TargetLanguage,
    pub iterations: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
    pub throughput_ops_per_second: f64,
    pub memory_usage: MemoryUsage,
    pub percentiles: HashMap<String, f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Performance comparison between languages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageComparison {
    pub operation: String,
    pub results: HashMap<TargetLanguage, BenchmarkResult>,
    pub winner: TargetLanguage,
    pub performance_ratios: HashMap<TargetLanguage, f64>,
}

/// Real-time performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub active_sessions: u32,
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub network_throughput_mbps: f64,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub suggested_actions: Vec<String>,
}

/// Types of performance alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighLatency,
    HighMemoryUsage,
    HighCpuUsage,
    HighErrorRate,
    LowThroughput,
    MemoryLeak,
    SlowMethod,
    NetworkTimeout,
    ResourceExhaustion,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Performance profiler
pub struct PerformanceProfiler {
    config: ProfilerConfig,
    samples: Arc<RwLock<Vec<PerformanceSample>>>,
    method_profiles: Arc<RwLock<HashMap<String, MethodProfile>>>,
    benchmark_results: Arc<RwLock<Vec<BenchmarkResult>>>,
    real_time_metrics: Arc<RwLock<Vec<RealTimeMetrics>>>,
    active_sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
}

/// Session information for profiling
#[derive(Debug, Clone)]
struct SessionInfo {
    session_id: String,
    language: TargetLanguage,
    started_at: Instant,
    method_calls: HashMap<String, Vec<Duration>>,
    network_requests: u64,
    errors: u64,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new(config: ProfilerConfig) -> Self {
        Self {
            config,
            samples: Arc::new(RwLock::new(Vec::new())),
            method_profiles: Arc::new(RwLock::new(HashMap::new())),
            benchmark_results: Arc::new(RwLock::new(Vec::new())),
            real_time_metrics: Arc::new(RwLock::new(Vec::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start profiling a session
    pub async fn start_session(&self, session_id: &str, language: TargetLanguage) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let session_info = SessionInfo {
            session_id: session_id.to_string(),
            language,
            started_at: Instant::now(),
            method_calls: HashMap::new(),
            network_requests: 0,
            errors: 0,
        };

        self.active_sessions.write().await.insert(session_id.to_string(), session_info);
        info!("Started profiling session: {} ({})", session_id, language);
        Ok(())
    }

    /// End profiling session
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        if let Some(session) = self.active_sessions.write().await.remove(session_id) {
            self.finalize_session_profile(session).await?;
            info!("Ended profiling session: {}", session_id);
        }
        Ok(())
    }

    /// Record method call performance
    pub async fn record_method_call(
        &self,
        session_id: &str,
        method_name: &str,
        duration: Duration,
        memory_usage: MemoryUsage,
        success: bool,
    ) -> Result<()> {
        if !self.config.enabled || !self.config.profile_method_calls {
            return Ok(());
        }

        // Update session info
        if let Some(session) = self.active_sessions.write().await.get_mut(session_id) {
            session.method_calls
                .entry(method_name.to_string())
                .or_insert_with(Vec::new)
                .push(duration);

            if !success {
                session.errors += 1;
            }
        }

        // Create performance sample
        let sample = PerformanceSample {
            timestamp: chrono::Utc::now(),
            session_id: session_id.to_string(),
            method_name: method_name.to_string(),
            language: self.get_session_language(session_id).await?,
            duration_ms: duration.as_millis() as u64,
            memory_usage,
            network_stats: NetworkStats {
                bytes_sent: 0,
                bytes_received: 0,
                requests_count: 0,
                requests_failed: 0,
                average_latency_ms: 0.0,
                connection_count: 0,
                dns_lookup_time_ms: None,
                ssl_handshake_time_ms: None,
            },
            cpu_stats: CpuStats {
                cpu_usage_percent: 0.0,
                user_time_ms: 0,
                system_time_ms: 0,
                thread_count: 0,
                context_switches: 0,
            },
            custom_metrics: HashMap::new(),
        };

        // Store sample
        let mut samples = self.samples.write().await;
        samples.push(sample);

        // Limit samples
        if samples.len() > self.config.max_samples {
            samples.drain(0..samples.len() - self.config.max_samples);
        }

        // Update method profile
        self.update_method_profile(method_name, duration, success).await?;

        // Check for performance alerts
        self.check_performance_alerts(method_name, duration).await?;

        Ok(())
    }

    /// Record network request performance
    pub async fn record_network_request(
        &self,
        session_id: &str,
        duration: Duration,
        bytes_sent: u64,
        bytes_received: u64,
        success: bool,
    ) -> Result<()> {
        if !self.config.enabled || !self.config.profile_network {
            return Ok(());
        }

        if let Some(session) = self.active_sessions.write().await.get_mut(session_id) {
            session.network_requests += 1;
            if !success {
                session.errors += 1;
            }
        }

        // Record network statistics for analysis
        debug!(
            "Network request: session={}, duration={:?}, bytes_sent={}, bytes_received={}, success={}",
            session_id, duration, bytes_sent, bytes_received, success
        );

        Ok(())
    }

    /// Run performance benchmark
    pub async fn run_benchmark(
        &self,
        name: &str,
        language: TargetLanguage,
        iterations: u64,
        benchmark_fn: impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>,
    ) -> Result<BenchmarkResult> {
        info!("Running benchmark: {} ({:?}) - {} iterations", name, language, iterations);

        let start_time = Instant::now();
        let mut durations = Vec::new();
        let mut errors = 0;

        let initial_memory = self.get_current_memory_usage().await?;

        for i in 0..iterations {
            let iteration_start = Instant::now();
            
            match benchmark_fn().await {
                Ok(_) => {
                    durations.push(iteration_start.elapsed());
                }
                Err(e) => {
                    errors += 1;
                    warn!("Benchmark iteration {} failed: {}", i, e);
                }
            }
        }

        let total_duration = start_time.elapsed();
        let final_memory = self.get_current_memory_usage().await?;

        let average_duration_ms = if durations.is_empty() {
            0.0
        } else {
            durations.iter().sum::<Duration>().as_millis() as f64 / durations.len() as f64
        };

        let throughput = if total_duration.as_secs_f64() > 0.0 {
            iterations as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        // Calculate percentiles
        let mut percentiles = HashMap::new();
        if !durations.is_empty() {
            let mut sorted_durations = durations.clone();
            sorted_durations.sort();
            
            percentiles.insert("p50".to_string(), 
                self.calculate_percentile(&sorted_durations, 0.5));
            percentiles.insert("p95".to_string(), 
                self.calculate_percentile(&sorted_durations, 0.95));
            percentiles.insert("p99".to_string(), 
                self.calculate_percentile(&sorted_durations, 0.99));
        }

        let result = BenchmarkResult {
            name: name.to_string(),
            language,
            iterations,
            total_duration_ms: total_duration.as_millis() as u64,
            average_duration_ms,
            throughput_ops_per_second: throughput,
            memory_usage: MemoryUsage {
                heap_used_mb: final_memory.heap_used_mb - initial_memory.heap_used_mb,
                heap_total_mb: final_memory.heap_total_mb,
                heap_peak_mb: final_memory.heap_peak_mb.max(initial_memory.heap_peak_mb),
                allocations_count: final_memory.allocations_count - initial_memory.allocations_count,
                deallocations_count: final_memory.deallocations_count - initial_memory.deallocations_count,
                gc_collections: final_memory.gc_collections - initial_memory.gc_collections,
                gc_time_ms: final_memory.gc_time_ms - initial_memory.gc_time_ms,
            },
            percentiles,
            timestamp: chrono::Utc::now(),
        };

        // Store benchmark result
        self.benchmark_results.write().await.push(result.clone());

        info!(
            "Benchmark completed: {} - {:.2}ms avg, {:.2} ops/sec, {} errors",
            name, average_duration_ms, throughput, errors
        );

        Ok(result)
    }

    /// Compare performance across languages
    pub async fn compare_languages(
        &self,
        operation: &str,
        languages: &[TargetLanguage],
        iterations: u64,
        benchmark_fn: impl Fn(TargetLanguage) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>,
    ) -> Result<LanguageComparison> {
        let mut results = HashMap::new();

        for &language in languages {
            let result = self.run_benchmark(
                operation,
                language,
                iterations,
                || benchmark_fn(language),
            ).await?;
            results.insert(language, result);
        }

        // Determine winner (fastest average response time)
        let winner = results.iter()
            .min_by(|(_, a), (_, b)| a.average_duration_ms.partial_cmp(&b.average_duration_ms).unwrap())
            .map(|(lang, _)| *lang)
            .unwrap_or(languages[0]);

        // Calculate performance ratios
        let baseline_time = results[&winner].average_duration_ms;
        let performance_ratios: HashMap<TargetLanguage, f64> = results.iter()
            .map(|(lang, result)| (*lang, result.average_duration_ms / baseline_time))
            .collect();

        Ok(LanguageComparison {
            operation: operation.to_string(),
            results,
            winner,
            performance_ratios,
        })
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> Result<String> {
        let samples = self.samples.read().await;
        let method_profiles = self.method_profiles.read().await;
        let benchmark_results = self.benchmark_results.read().await;
        let alerts = self.alerts.read().await;

        let mut report = String::new();
        report.push_str("# Performance Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().to_rfc3339()));

        // Summary statistics
        report.push_str("## Summary\n\n");
        report.push_str(&format!("- Total Samples: {}\n", samples.len()));
        report.push_str(&format!("- Method Profiles: {}\n", method_profiles.len()));
        report.push_str(&format!("- Benchmark Results: {}\n", benchmark_results.len()));
        report.push_str(&format!("- Active Alerts: {}\n", alerts.len()));

        // Method performance
        if !method_profiles.is_empty() {
            report.push_str("\n## Method Performance\n\n");
            report.push_str("| Method | Calls | Avg (ms) | P95 (ms) | Error Rate |\n");
            report.push_str("|--------|-------|----------|----------|------------|\n");

            let mut sorted_methods: Vec<_> = method_profiles.values().collect();
            sorted_methods.sort_by(|a, b| b.average_duration_ms.partial_cmp(&a.average_duration_ms).unwrap());

            for profile in sorted_methods.iter().take(10) {
                report.push_str(&format!(
                    "| {} | {} | {:.1} | {:.1} | {:.1}% |\n",
                    profile.method_name,
                    profile.call_count,
                    profile.average_duration_ms,
                    profile.p95_duration_ms,
                    profile.error_rate_percent
                ));
            }
        }

        // Benchmark results
        if !benchmark_results.is_empty() {
            report.push_str("\n## Benchmark Results\n\n");
            report.push_str("| Name | Language | Avg (ms) | Throughput (ops/sec) |\n");
            report.push_str("|------|----------|----------|----------------------|\n");

            for result in benchmark_results.iter() {
                report.push_str(&format!(
                    "| {} | {:?} | {:.1} | {:.1} |\n",
                    result.name,
                    result.language,
                    result.average_duration_ms,
                    result.throughput_ops_per_second
                ));
            }
        }

        // Active alerts
        if !alerts.is_empty() {
            report.push_str("\n## Performance Alerts\n\n");
            for alert in alerts.iter() {
                report.push_str(&format!(
                    "- **{:?}**: {} (Current: {:.1}, Threshold: {:.1})\n",
                    alert.severity,
                    alert.message,
                    alert.current_value,
                    alert.threshold_value
                ));
            }
        }

        Ok(report)
    }

    /// Export profiling data
    pub async fn export_data(&self, format: ProfileExportFormat) -> Result<String> {
        match format {
            ProfileExportFormat::Json => self.export_json().await,
            ProfileExportFormat::Csv => self.export_csv().await,
            ProfileExportFormat::Flamegraph => self.export_flamegraph().await,
            ProfileExportFormat::Chrome => self.export_chrome_format().await,
            _ => Err(anyhow!("Export format not yet implemented: {:?}", format)),
        }
    }

    // Helper methods
    async fn get_session_language(&self, session_id: &str) -> Result<TargetLanguage> {
        self.active_sessions.read().await
            .get(session_id)
            .map(|s| s.language)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))
    }

    async fn update_method_profile(&self, method_name: &str, duration: Duration, success: bool) -> Result<()> {
        let mut profiles = self.method_profiles.write().await;
        let profile = profiles.entry(method_name.to_string()).or_insert(MethodProfile {
            method_name: method_name.to_string(),
            call_count: 0,
            total_duration_ms: 0,
            average_duration_ms: 0.0,
            min_duration_ms: u64::MAX,
            max_duration_ms: 0,
            p50_duration_ms: 0.0,
            p95_duration_ms: 0.0,
            p99_duration_ms: 0.0,
            error_count: 0,
            error_rate_percent: 0.0,
        });

        let duration_ms = duration.as_millis() as u64;
        profile.call_count += 1;
        profile.total_duration_ms += duration_ms;
        profile.average_duration_ms = profile.total_duration_ms as f64 / profile.call_count as f64;
        profile.min_duration_ms = profile.min_duration_ms.min(duration_ms);
        profile.max_duration_ms = profile.max_duration_ms.max(duration_ms);

        if !success {
            profile.error_count += 1;
        }
        profile.error_rate_percent = (profile.error_count as f64 / profile.call_count as f64) * 100.0;

        Ok(())
    }

    async fn check_performance_alerts(&self, method_name: &str, duration: Duration) -> Result<()> {
        let duration_ms = duration.as_millis() as f64;

        // Check for slow method alert
        if duration_ms > 5000.0 { // 5 seconds threshold
            let alert = PerformanceAlert {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                alert_type: AlertType::SlowMethod,
                severity: AlertSeverity::Warning,
                message: format!("Method {} is running slowly", method_name),
                metric_name: "duration_ms".to_string(),
                current_value: duration_ms,
                threshold_value: 5000.0,
                suggested_actions: vec![
                    "Review method implementation for performance bottlenecks".to_string(),
                    "Consider adding caching or optimization".to_string(),
                    "Check for network or database issues".to_string(),
                ],
            };

            self.alerts.write().await.push(alert);
            warn!("Performance alert: Method {} took {:.1}ms", method_name, duration_ms);
        }

        Ok(())
    }

    async fn finalize_session_profile(&self, session: SessionInfo) -> Result<()> {
        let session_duration = session.started_at.elapsed();
        info!(
            "Session {} completed: duration={:?}, method_calls={}, network_requests={}, errors={}",
            session.session_id,
            session_duration,
            session.method_calls.len(),
            session.network_requests,
            session.errors
        );
        Ok(())
    }

    async fn get_current_memory_usage(&self) -> Result<MemoryUsage> {
        // In a real implementation, this would collect actual memory statistics
        // For now, return mock data
        Ok(MemoryUsage {
            heap_used_mb: 50.0,
            heap_total_mb: 100.0,
            heap_peak_mb: 75.0,
            allocations_count: 1000,
            deallocations_count: 800,
            gc_collections: 5,
            gc_time_ms: 50,
        })
    }

    fn calculate_percentile(&self, durations: &[Duration], percentile: f64) -> f64 {
        if durations.is_empty() {
            return 0.0;
        }

        let index = (durations.len() as f64 * percentile) as usize;
        durations.get(index.min(durations.len() - 1))
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0)
    }

    async fn export_json(&self) -> Result<String> {
        let data = serde_json::json!({
            "samples": *self.samples.read().await,
            "method_profiles": *self.method_profiles.read().await,
            "benchmark_results": *self.benchmark_results.read().await,
            "alerts": *self.alerts.read().await,
            "exported_at": chrono::Utc::now()
        });

        Ok(serde_json::to_string_pretty(&data)?)
    }

    async fn export_csv(&self) -> Result<String> {
        let samples = self.samples.read().await;
        let mut csv = String::new();
        csv.push_str("timestamp,session_id,method_name,language,duration_ms,memory_mb\n");

        for sample in samples.iter() {
            csv.push_str(&format!(
                "{},{},{},{:?},{},{}\n",
                sample.timestamp.to_rfc3339(),
                sample.session_id,
                sample.method_name,
                sample.language,
                sample.duration_ms,
                sample.memory_usage.heap_used_mb
            ));
        }

        Ok(csv)
    }

    async fn export_flamegraph(&self) -> Result<String> {
        // Generate flamegraph format for visualization
        let samples = self.samples.read().await;
        let mut flamegraph = String::new();

        for sample in samples.iter() {
            flamegraph.push_str(&format!(
                "{};{} {}\n",
                sample.method_name,
                sample.language,
                sample.duration_ms
            ));
        }

        Ok(flamegraph)
    }

    async fn export_chrome_format(&self) -> Result<String> {
        // Generate Chrome DevTools performance format
        let samples = self.samples.read().await;
        let mut events = Vec::new();

        for sample in samples.iter() {
            let event = serde_json::json!({
                "name": sample.method_name,
                "cat": "method",
                "ph": "X",
                "ts": sample.timestamp.timestamp_micros(),
                "dur": sample.duration_ms * 1000,
                "pid": 1,
                "tid": 1,
                "args": {
                    "language": format!("{:?}", sample.language),
                    "memory_mb": sample.memory_usage.heap_used_mb
                }
            });
            events.push(event);
        }

        let trace = serde_json::json!({
            "traceEvents": events,
            "displayTimeUnit": "ms"
        });

        Ok(serde_json::to_string_pretty(&trace)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profiler_creation() -> Result<()> {
        let config = ProfilerConfig::default();
        let profiler = PerformanceProfiler::new(config);

        let session_id = "test_session";
        profiler.start_session(session_id, TargetLanguage::Rust).await?;
        profiler.end_session(session_id).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_method_profiling() -> Result<()> {
        let config = ProfilerConfig::default();
        let profiler = PerformanceProfiler::new(config);

        let session_id = "test_session";
        profiler.start_session(session_id, TargetLanguage::Rust).await?;

        let memory_usage = MemoryUsage {
            heap_used_mb: 50.0,
            heap_total_mb: 100.0,
            heap_peak_mb: 75.0,
            allocations_count: 100,
            deallocations_count: 80,
            gc_collections: 1,
            gc_time_ms: 10,
        };

        profiler.record_method_call(
            session_id,
            "test_method",
            Duration::from_millis(100),
            memory_usage,
            true,
        ).await?;

        let profiles = profiler.method_profiles.read().await;
        assert!(profiles.contains_key("test_method"));

        profiler.end_session(session_id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_benchmark() -> Result<()> {
        let config = ProfilerConfig::default();
        let profiler = PerformanceProfiler::new(config);

        let result = profiler.run_benchmark(
            "test_benchmark",
            TargetLanguage::Rust,
            10,
            || Box::pin(async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(())
            }),
        ).await?;

        assert_eq!(result.name, "test_benchmark");
        assert_eq!(result.language, TargetLanguage::Rust);
        assert_eq!(result.iterations, 10);
        assert!(result.average_duration_ms > 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_report_generation() -> Result<()> {
        let config = ProfilerConfig::default();
        let profiler = PerformanceProfiler::new(config);

        let report = profiler.generate_report().await?;
        assert!(report.contains("Performance Report"));
        assert!(report.contains("Summary"));

        Ok(())
    }
}