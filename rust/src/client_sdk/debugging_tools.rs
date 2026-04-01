//! Comprehensive debugging tools for OpenSim client SDKs

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

/// Debugging tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub enabled: bool,
    pub log_level: DebugLogLevel,
    pub capture_network_traffic: bool,
    pub capture_request_response: bool,
    pub capture_performance_metrics: bool,
    pub max_log_entries: usize,
    pub output_directory: PathBuf,
    pub real_time_monitoring: bool,
    pub websocket_port: Option<u16>,
    pub export_formats: Vec<ExportFormat>,
    pub filters: DebugFilters,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: DebugLogLevel::Info,
            capture_network_traffic: true,
            capture_request_response: true,
            capture_performance_metrics: true,
            max_log_entries: 10000,
            output_directory: PathBuf::from("./debug-output"),
            real_time_monitoring: true,
            websocket_port: Some(8092),
            export_formats: vec![ExportFormat::Json, ExportFormat::Har],
            filters: DebugFilters::default(),
        }
    }
}

/// Debug log levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum DebugLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Export formats for debug data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Har,     // HTTP Archive format
    Csv,
    Xml,
    Pcap,    // Packet capture format
    Flamegraph,
}

/// Debug filters for selective capturing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugFilters {
    pub endpoints: Option<Vec<String>>,
    pub status_codes: Option<Vec<u16>>,
    pub methods: Option<Vec<String>>,
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub languages: Option<Vec<TargetLanguage>>,
    pub exclude_patterns: Vec<String>,
    pub include_patterns: Vec<String>,
}

impl Default for DebugFilters {
    fn default() -> Self {
        Self {
            endpoints: None,
            status_codes: None,
            methods: None,
            min_duration_ms: None,
            max_duration_ms: None,
            languages: None,
            exclude_patterns: vec![],
            include_patterns: vec![],
        }
    }
}

/// Debug session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub id: String,
    pub name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub language: TargetLanguage,
    pub client_version: String,
    pub config: DebugConfig,
    pub metadata: HashMap<String, String>,
}

/// Network request/response capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCapture {
    pub id: String,
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub method: String,
    pub url: String,
    pub request: RequestCapture,
    pub response: Option<ResponseCapture>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
    pub metadata: CaptureMetadata,
}

/// HTTP request capture details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestCapture {
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub body_size: u64,
    pub content_type: Option<String>,
    pub query_parameters: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
}

/// HTTP response capture details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCapture {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub body_size: u64,
    pub content_type: Option<String>,
    pub cookies: HashMap<String, String>,
}

/// Additional capture metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMetadata {
    pub client_language: TargetLanguage,
    pub client_version: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub ssl_info: Option<SslInfo>,
    pub performance_timing: PerformanceTiming,
}

/// SSL/TLS connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslInfo {
    pub protocol: String,
    pub cipher_suite: String,
    pub certificate_info: Option<String>,
    pub handshake_time_ms: u64,
}

/// Detailed performance timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTiming {
    pub dns_lookup_ms: Option<u64>,
    pub tcp_connect_ms: Option<u64>,
    pub ssl_handshake_ms: Option<u64>,
    pub request_send_ms: u64,
    pub response_receive_ms: u64,
    pub total_time_ms: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Debug event for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEvent {
    pub id: String,
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: DebugEventType,
    pub severity: DebugLogLevel,
    pub message: String,
    pub data: serde_json::Value,
    pub stack_trace: Option<String>,
}

/// Types of debug events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugEventType {
    NetworkRequest,
    NetworkResponse,
    Error,
    Warning,
    Performance,
    Authentication,
    Validation,
    Retry,
    Timeout,
    Custom { event_name: String },
}

/// Performance metrics for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_mb: Option<f64>,
    pub network_latency_ms: Option<u64>,
    pub active_connections: u32,
    pub request_queue_size: u32,
    pub cache_hit_rate: Option<f64>,
    pub error_rate: f64,
    pub throughput_requests_per_second: f64,
}

/// Debug analyzer for identifying issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugAnalysis {
    pub session_id: String,
    pub analysis_timestamp: chrono::DateTime<chrono::Utc>,
    pub issues_found: Vec<DebugIssue>,
    pub recommendations: Vec<Recommendation>,
    pub performance_summary: AnalysisPerformanceSummary,
    pub error_patterns: Vec<ErrorPattern>,
}

/// Identified debug issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugIssue {
    pub id: String,
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub affected_requests: Vec<String>,
    pub first_occurrence: chrono::DateTime<chrono::Utc>,
    pub last_occurrence: chrono::DateTime<chrono::Utc>,
    pub frequency: u32,
    pub suggested_fix: Option<String>,
}

/// Types of issues that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    SlowResponse,
    HighErrorRate,
    AuthenticationFailure,
    NetworkTimeout,
    MalformedRequest,
    InvalidResponse,
    MemoryLeak,
    ConnectionLeak,
    RateLimitExceeded,
    SslError,
    Custom { issue_name: String },
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Debug recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub priority: IssueSeverity,
    pub effort_level: EffortLevel,
    pub implementation_steps: Vec<String>,
    pub expected_impact: String,
}

/// Categories of recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Performance,
    Reliability,
    Security,
    CodeQuality,
    Configuration,
    Monitoring,
}

/// Implementation effort level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,     // < 1 hour
    Medium,  // 1-8 hours
    High,    // 1-3 days
    Complex, // > 3 days
}

/// Performance analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPerformanceSummary {
    pub total_requests: u32,
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub slowest_endpoints: Vec<EndpointPerformance>,
    pub most_error_prone_endpoints: Vec<EndpointPerformance>,
}

/// Endpoint performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPerformance {
    pub endpoint: String,
    pub method: String,
    pub request_count: u32,
    pub average_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub p95_response_time_ms: f64,
}

/// Error pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub pattern_id: String,
    pub error_type: String,
    pub frequency: u32,
    pub affected_endpoints: Vec<String>,
    pub common_causes: Vec<String>,
    pub resolution_suggestions: Vec<String>,
}

/// Debug session tracker
pub struct DebugSessionTracker {
    sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
    network_captures: Arc<RwLock<Vec<NetworkCapture>>>,
    debug_events: Arc<RwLock<Vec<DebugEvent>>>,
    performance_metrics: Arc<RwLock<Vec<PerformanceMetrics>>>,
    config: DebugConfig,
}

impl DebugSessionTracker {
    /// Create a new debug session tracker
    pub fn new(config: DebugConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            network_captures: Arc::new(RwLock::new(Vec::new())),
            debug_events: Arc::new(RwLock::new(Vec::new())),
            performance_metrics: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Start a new debug session
    pub async fn start_session(&self, name: &str, language: TargetLanguage, client_version: &str) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        let session = DebugSession {
            id: session_id.clone(),
            name: name.to_string(),
            started_at: chrono::Utc::now(),
            ended_at: None,
            language,
            client_version: client_version.to_string(),
            config: self.config.clone(),
            metadata: HashMap::new(),
        };

        self.sessions.write().await.insert(session_id.clone(), session);
        
        info!("Started debug session: {} ({})", name, session_id);
        Ok(session_id)
    }

    /// End a debug session
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.ended_at = Some(chrono::Utc::now());
            info!("Ended debug session: {}", session_id);
        }
        Ok(())
    }

    /// Capture a network request/response
    pub async fn capture_network_activity(
        &self,
        session_id: &str,
        method: &str,
        url: &str,
        request: RequestCapture,
        response: Option<ResponseCapture>,
        duration_ms: Option<u64>,
        error: Option<String>,
    ) -> Result<()> {
        if !self.config.capture_network_traffic {
            return Ok(());
        }

        let capture_id = uuid::Uuid::new_v4().to_string();
        
        // Get session info for metadata
        let session = self.sessions.read().await
            .get(session_id)
            .cloned()
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        let metadata = CaptureMetadata {
            client_language: session.language,
            client_version: session.client_version,
            user_agent: request.headers.get("User-Agent").cloned(),
            ip_address: None, // Would be populated in real implementation
            ssl_info: None,   // Would be populated for HTTPS requests
            performance_timing: PerformanceTiming {
                dns_lookup_ms: None,
                tcp_connect_ms: None,
                ssl_handshake_ms: None,
                request_send_ms: duration_ms.unwrap_or(0) / 4,
                response_receive_ms: duration_ms.unwrap_or(0) * 3 / 4,
                total_time_ms: duration_ms.unwrap_or(0),
                bytes_sent: request.body_size,
                bytes_received: response.as_ref().map(|r| r.body_size).unwrap_or(0),
            },
        };

        let capture = NetworkCapture {
            id: capture_id,
            session_id: session_id.to_string(),
            timestamp: chrono::Utc::now(),
            method: method.to_string(),
            url: url.to_string(),
            request,
            response,
            duration_ms,
            error,
            metadata,
        };

        let mut captures = self.network_captures.write().await;
        captures.push(capture);

        // Limit the number of captures
        if captures.len() > self.config.max_log_entries {
            captures.drain(0..captures.len() - self.config.max_log_entries);
        }

        Ok(())
    }

    /// Log a debug event
    pub async fn log_event(
        &self,
        session_id: &str,
        event_type: DebugEventType,
        severity: DebugLogLevel,
        message: &str,
        data: serde_json::Value,
        stack_trace: Option<String>,
    ) -> Result<()> {
        if severity < self.config.log_level {
            return Ok(());
        }

        let event = DebugEvent {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            timestamp: chrono::Utc::now(),
            event_type,
            severity,
            message: message.to_string(),
            data,
            stack_trace,
        };

        let mut events = self.debug_events.write().await;
        events.push(event);

        // Limit the number of events
        if events.len() > self.config.max_log_entries {
            events.drain(0..events.len() - self.config.max_log_entries);
        }

        Ok(())
    }

    /// Record performance metrics
    pub async fn record_performance_metrics(&self, session_id: &str, metrics: PerformanceMetrics) -> Result<()> {
        if !self.config.capture_performance_metrics {
            return Ok(());
        }

        let mut perf_metrics = self.performance_metrics.write().await;
        perf_metrics.push(metrics);

        // Limit the number of metrics
        if perf_metrics.len() > self.config.max_log_entries {
            perf_metrics.drain(0..perf_metrics.len() - self.config.max_log_entries);
        }

        Ok(())
    }

    /// Get network captures for a session
    pub async fn get_network_captures(&self, session_id: &str) -> Vec<NetworkCapture> {
        self.network_captures.read().await
            .iter()
            .filter(|capture| capture.session_id == session_id)
            .cloned()
            .collect()
    }

    /// Get debug events for a session
    pub async fn get_debug_events(&self, session_id: &str) -> Vec<DebugEvent> {
        self.debug_events.read().await
            .iter()
            .filter(|event| event.session_id == session_id)
            .cloned()
            .collect()
    }

    /// Get performance metrics for a session
    pub async fn get_performance_metrics(&self, session_id: &str) -> Vec<PerformanceMetrics> {
        self.performance_metrics.read().await
            .iter()
            .filter(|metrics| metrics.session_id == session_id)
            .cloned()
            .collect()
    }

    /// Analyze debug session and identify issues
    pub async fn analyze_session(&self, session_id: &str) -> Result<DebugAnalysis> {
        let captures = self.get_network_captures(session_id).await;
        let events = self.get_debug_events(session_id).await;
        let metrics = self.get_performance_metrics(session_id).await;

        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Analyze slow responses
        let slow_captures: Vec<_> = captures.iter()
            .filter(|c| c.duration_ms.unwrap_or(0) > 5000) // > 5 seconds
            .collect();

        if !slow_captures.is_empty() {
            issues.push(DebugIssue {
                id: uuid::Uuid::new_v4().to_string(),
                issue_type: IssueType::SlowResponse,
                severity: IssueSeverity::Medium,
                description: format!("Found {} slow responses (>5s)", slow_captures.len()),
                affected_requests: slow_captures.iter().map(|c| c.id.clone()).collect(),
                first_occurrence: slow_captures.iter().map(|c| c.timestamp).min().unwrap(),
                last_occurrence: slow_captures.iter().map(|c| c.timestamp).max().unwrap(),
                frequency: slow_captures.len() as u32,
                suggested_fix: Some("Consider implementing request timeout and retry logic".to_string()),
            });

            recommendations.push(Recommendation {
                id: uuid::Uuid::new_v4().to_string(),
                category: RecommendationCategory::Performance,
                title: "Implement Request Timeouts".to_string(),
                description: "Add appropriate timeout values to prevent long-running requests".to_string(),
                priority: IssueSeverity::Medium,
                effort_level: EffortLevel::Low,
                implementation_steps: vec![
                    "Set reasonable timeout values for HTTP requests".to_string(),
                    "Implement retry logic with exponential backoff".to_string(),
                    "Add circuit breaker pattern for failing endpoints".to_string(),
                ],
                expected_impact: "Improved user experience and resource utilization".to_string(),
            });
        }

        // Analyze error rates
        let error_captures: Vec<_> = captures.iter()
            .filter(|c| c.response.as_ref().map(|r| r.status_code >= 400).unwrap_or(false) || c.error.is_some())
            .collect();

        let error_rate = if captures.is_empty() {
            0.0
        } else {
            error_captures.len() as f64 / captures.len() as f64 * 100.0
        };

        if error_rate > 10.0 {
            issues.push(DebugIssue {
                id: uuid::Uuid::new_v4().to_string(),
                issue_type: IssueType::HighErrorRate,
                severity: if error_rate > 50.0 { IssueSeverity::Critical } else { IssueSeverity::High },
                description: format!("High error rate: {:.1}%", error_rate),
                affected_requests: error_captures.iter().map(|c| c.id.clone()).collect(),
                first_occurrence: error_captures.iter().map(|c| c.timestamp).min().unwrap_or(chrono::Utc::now()),
                last_occurrence: error_captures.iter().map(|c| c.timestamp).max().unwrap_or(chrono::Utc::now()),
                frequency: error_captures.len() as u32,
                suggested_fix: Some("Review error handling and implement proper retry logic".to_string()),
            });
        }

        // Calculate performance summary
        let response_times: Vec<u64> = captures.iter()
            .filter_map(|c| c.duration_ms)
            .collect();

        let performance_summary = AnalysisPerformanceSummary {
            total_requests: captures.len() as u32,
            average_response_time_ms: if response_times.is_empty() {
                0.0
            } else {
                response_times.iter().sum::<u64>() as f64 / response_times.len() as f64
            },
            p95_response_time_ms: self.calculate_percentile(&response_times, 0.95),
            p99_response_time_ms: self.calculate_percentile(&response_times, 0.99),
            error_rate_percent: error_rate,
            slowest_endpoints: self.analyze_endpoint_performance(&captures, true),
            most_error_prone_endpoints: self.analyze_endpoint_performance(&captures, false),
        };

        // Analyze error patterns
        let error_patterns = self.analyze_error_patterns(&captures, &events);

        Ok(DebugAnalysis {
            session_id: session_id.to_string(),
            analysis_timestamp: chrono::Utc::now(),
            issues_found: issues,
            recommendations,
            performance_summary,
            error_patterns,
        })
    }

    /// Export debug data in specified format
    pub async fn export_debug_data(&self, session_id: &str, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Json => self.export_json(session_id).await,
            ExportFormat::Har => self.export_har(session_id).await,
            ExportFormat::Csv => self.export_csv(session_id).await,
            _ => Err(anyhow!("Export format not yet implemented: {:?}", format)),
        }
    }

    async fn export_json(&self, session_id: &str) -> Result<String> {
        let session = self.sessions.read().await
            .get(session_id)
            .cloned()
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        let captures = self.get_network_captures(session_id).await;
        let events = self.get_debug_events(session_id).await;
        let metrics = self.get_performance_metrics(session_id).await;

        let export_data = serde_json::json!({
            "session": session,
            "network_captures": captures,
            "debug_events": events,
            "performance_metrics": metrics,
            "exported_at": chrono::Utc::now()
        });

        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    async fn export_har(&self, session_id: &str) -> Result<String> {
        let captures = self.get_network_captures(session_id).await;
        
        // Convert to HAR format
        let har_entries: Vec<_> = captures.iter().map(|capture| {
            serde_json::json!({
                "startedDateTime": capture.timestamp.to_rfc3339(),
                "time": capture.duration_ms.unwrap_or(0),
                "request": {
                    "method": capture.method,
                    "url": capture.url,
                    "headers": capture.request.headers,
                    "bodySize": capture.request.body_size
                },
                "response": capture.response.as_ref().map(|resp| serde_json::json!({
                    "status": resp.status_code,
                    "statusText": resp.status_text,
                    "headers": resp.headers,
                    "bodySize": resp.body_size
                }))
            })
        }).collect();

        let har = serde_json::json!({
            "log": {
                "version": "1.2",
                "creator": {
                    "name": "OpenSim Debug Tools",
                    "version": "1.0.0"
                },
                "entries": har_entries
            }
        });

        Ok(serde_json::to_string_pretty(&har)?)
    }

    async fn export_csv(&self, session_id: &str) -> Result<String> {
        let captures = self.get_network_captures(session_id).await;
        
        let mut csv = String::new();
        csv.push_str("timestamp,method,url,status_code,duration_ms,request_size,response_size,error\n");
        
        for capture in captures {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                capture.timestamp.to_rfc3339(),
                capture.method,
                capture.url,
                capture.response.as_ref().map(|r| r.status_code.to_string()).unwrap_or_default(),
                capture.duration_ms.unwrap_or(0),
                capture.request.body_size,
                capture.response.as_ref().map(|r| r.body_size).unwrap_or(0),
                capture.error.as_deref().unwrap_or("")
            ));
        }
        
        Ok(csv)
    }

    fn calculate_percentile(&self, values: &[u64], percentile: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        
        let index = (values.len() as f64 * percentile) as usize;
        sorted.get(index.min(sorted.len() - 1)).copied().unwrap_or(0) as f64
    }

    fn analyze_endpoint_performance(&self, captures: &[NetworkCapture], by_slowness: bool) -> Vec<EndpointPerformance> {
        let mut endpoint_stats: HashMap<String, (Vec<u64>, u32, u32)> = HashMap::new();

        for capture in captures {
            let key = format!("{} {}", capture.method, capture.url);
            let entry = endpoint_stats.entry(key).or_insert((Vec::new(), 0, 0));
            
            if let Some(duration) = capture.duration_ms {
                entry.0.push(duration);
            }
            entry.1 += 1; // Total requests
            
            if capture.response.as_ref().map(|r| r.status_code >= 400).unwrap_or(false) || capture.error.is_some() {
                entry.2 += 1; // Error count
            }
        }

        let mut performances: Vec<_> = endpoint_stats.into_iter()
            .map(|(endpoint, (durations, total, errors))| {
                let avg_duration = if durations.is_empty() {
                    0.0
                } else {
                    durations.iter().sum::<u64>() as f64 / durations.len() as f64
                };
                
                let p95_duration = self.calculate_percentile(&durations, 0.95);
                let error_rate = if total == 0 { 0.0 } else { errors as f64 / total as f64 * 100.0 };
                
                let parts: Vec<&str> = endpoint.splitn(2, ' ').collect();
                EndpointPerformance {
                    method: parts.get(0).unwrap_or(&"").to_string(),
                    endpoint: parts.get(1).unwrap_or(&endpoint).to_string(),
                    request_count: total,
                    average_response_time_ms: avg_duration,
                    error_rate_percent: error_rate,
                    p95_response_time_ms: p95_duration,
                }
            })
            .collect();

        if by_slowness {
            performances.sort_by(|a, b| b.average_response_time_ms.partial_cmp(&a.average_response_time_ms).unwrap());
        } else {
            performances.sort_by(|a, b| b.error_rate_percent.partial_cmp(&a.error_rate_percent).unwrap());
        }

        performances.into_iter().take(10).collect()
    }

    fn analyze_error_patterns(&self, captures: &[NetworkCapture], events: &[DebugEvent]) -> Vec<ErrorPattern> {
        let mut patterns = HashMap::new();

        // Analyze network errors
        for capture in captures {
            if let Some(ref error) = capture.error {
                let pattern_key = error.clone();
                let entry = patterns.entry(pattern_key).or_insert(ErrorPattern {
                    pattern_id: uuid::Uuid::new_v4().to_string(),
                    error_type: error.clone(),
                    frequency: 0,
                    affected_endpoints: Vec::new(),
                    common_causes: Vec::new(),
                    resolution_suggestions: Vec::new(),
                });
                
                entry.frequency += 1;
                entry.affected_endpoints.push(capture.url.clone());
            }
        }

        // Analyze debug events
        for event in events {
            if matches!(event.event_type, DebugEventType::Error) {
                let pattern_key = event.message.clone();
                let entry = patterns.entry(pattern_key).or_insert(ErrorPattern {
                    pattern_id: uuid::Uuid::new_v4().to_string(),
                    error_type: event.message.clone(),
                    frequency: 0,
                    affected_endpoints: Vec::new(),
                    common_causes: Vec::new(),
                    resolution_suggestions: Vec::new(),
                });
                
                entry.frequency += 1;
            }
        }

        patterns.into_values().collect()
    }

    /// Clear debug data for a session
    pub async fn clear_session_data(&self, session_id: &str) -> Result<()> {
        let mut captures = self.network_captures.write().await;
        captures.retain(|c| c.session_id != session_id);

        let mut events = self.debug_events.write().await;
        events.retain(|e| e.session_id != session_id);

        let mut metrics = self.performance_metrics.write().await;
        metrics.retain(|m| m.session_id != session_id);

        info!("Cleared debug data for session: {}", session_id);
        Ok(())
    }
}

/// Debug proxy for intercepting network traffic
pub struct DebugProxy {
    port: u16,
    target_host: String,
    target_port: u16,
    session_tracker: Arc<DebugSessionTracker>,
    running: Arc<RwLock<bool>>,
}

impl DebugProxy {
    pub fn new(
        port: u16,
        target_host: String,
        target_port: u16,
        session_tracker: Arc<DebugSessionTracker>,
    ) -> Self {
        Self {
            port,
            target_host,
            target_port,
            session_tracker,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        *self.running.write().await = true;
        info!("Debug proxy started on port {} -> {}:{}", self.port, self.target_host, self.target_port);
        
        // In a full implementation, this would start an HTTP proxy server
        // that intercepts and logs all traffic
        
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        *self.running.write().await = false;
        info!("Debug proxy stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debug_session_creation() -> Result<()> {
        let config = DebugConfig::default();
        let tracker = DebugSessionTracker::new(config);
        
        let session_id = tracker.start_session("test_session", TargetLanguage::Rust, "1.0.0").await?;
        assert!(!session_id.is_empty());
        
        tracker.end_session(&session_id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_network_capture() -> Result<()> {
        let config = DebugConfig::default();
        let tracker = DebugSessionTracker::new(config);
        
        let session_id = tracker.start_session("test_session", TargetLanguage::Rust, "1.0.0").await?;
        
        let request = RequestCapture {
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: Some(r#"{"test": "data"}"#.to_string()),
            body_size: 15,
            content_type: Some("application/json".to_string()),
            query_parameters: HashMap::new(),
            cookies: HashMap::new(),
        };

        let response = ResponseCapture {
            status_code: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: Some(r#"{"result": "success"}"#.to_string()),
            body_size: 20,
            content_type: Some("application/json".to_string()),
            cookies: HashMap::new(),
        };

        tracker.capture_network_activity(
            &session_id,
            "POST",
            "https://api.example.com/test",
            request,
            Some(response),
            Some(150),
            None,
        ).await?;

        let captures = tracker.get_network_captures(&session_id).await;
        assert_eq!(captures.len(), 1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_debug_analysis() -> Result<()> {
        let config = DebugConfig::default();
        let tracker = DebugSessionTracker::new(config);
        
        let session_id = tracker.start_session("test_session", TargetLanguage::Rust, "1.0.0").await?;
        
        // Add some test data
        let request = RequestCapture {
            headers: HashMap::new(),
            body: None,
            body_size: 0,
            content_type: None,
            query_parameters: HashMap::new(),
            cookies: HashMap::new(),
        };

        // Slow response
        tracker.capture_network_activity(
            &session_id,
            "GET",
            "https://api.example.com/slow",
            request.clone(),
            Some(ResponseCapture {
                status_code: 200,
                status_text: "OK".to_string(),
                headers: HashMap::new(),
                body: None,
                body_size: 0,
                content_type: None,
                cookies: HashMap::new(),
            }),
            Some(6000), // 6 seconds - should trigger slow response detection
            None,
        ).await?;

        let analysis = tracker.analyze_session(&session_id).await?;
        assert!(!analysis.issues_found.is_empty());
        assert!(!analysis.recommendations.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_export_formats() -> Result<()> {
        let config = DebugConfig::default();
        let tracker = DebugSessionTracker::new(config);
        
        let session_id = tracker.start_session("test_session", TargetLanguage::Rust, "1.0.0").await?;
        
        let json_export = tracker.export_debug_data(&session_id, ExportFormat::Json).await?;
        assert!(json_export.contains("session"));
        
        let csv_export = tracker.export_debug_data(&session_id, ExportFormat::Csv).await?;
        assert!(csv_export.contains("timestamp,method,url"));
        
        Ok(())
    }
}