// OpenSim Next - Distributed Tracing Implementation
// Phase 30: Advanced distributed tracing for virtual world operations

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing::{instrument, info, warn, error, debug, Span};
use reqwest::Client;

/// Distributed tracing manager for OpenSim Next
#[derive(Debug, Clone)]
pub struct DistributedTracingManager {
    inner: Arc<DistributedTracingInner>,
}

#[derive(Debug)]
struct DistributedTracingInner {
    active_traces: RwLock<HashMap<Uuid, DistributedTrace>>,
    trace_queue: RwLock<Vec<CompletedDistributedTrace>>,
    config: TracingConfig,
    exporter: Arc<TraceExporter>,
    sender: mpsc::UnboundedSender<TraceEvent>,
}

/// Configuration for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub service_name: String,
    pub service_version: String,
    pub jaeger_endpoint: Option<String>,
    pub zipkin_endpoint: Option<String>,
    pub otlp_endpoint: Option<String>,
    pub sampling_rate: f64,
    pub max_spans_per_trace: usize,
    pub trace_timeout_seconds: u64,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub enable_logging: bool,
    pub enable_metrics: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "opensim-next".to_string(),
            service_version: "30.0.0".to_string(),
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            zipkin_endpoint: None,
            otlp_endpoint: None,
            sampling_rate: 0.1, // 10% sampling
            max_spans_per_trace: 1000,
            trace_timeout_seconds: 300, // 5 minutes
            batch_size: 100,
            batch_timeout_ms: 5000,
            enable_logging: true,
            enable_metrics: true,
        }
    }
}

/// Distributed trace encompassing multiple services and regions
#[derive(Debug, Clone)]
pub struct DistributedTrace {
    pub trace_id: Uuid,
    pub service_name: String,
    pub operation_name: String,
    pub start_time: Instant,
    pub spans: HashMap<Uuid, DistributedSpan>,
    pub trace_state: TraceState,
    pub baggage: HashMap<String, String>,
    pub resource_attributes: HashMap<String, String>,
}

/// Individual span within a distributed trace
#[derive(Debug, Clone)]
pub struct DistributedSpan {
    pub span_id: Uuid,
    pub trace_id: Uuid,
    pub parent_span_id: Option<Uuid>,
    pub operation_name: String,
    pub service_name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub status: SpanStatus,
    pub kind: SpanKind,
    pub attributes: HashMap<String, AttributeValue>,
    pub events: Vec<SpanEvent>,
    pub links: Vec<SpanLink>,
    pub instrumentation_scope: InstrumentationScope,
}

/// Status of a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error { code: ErrorCode, message: String },
}

/// Error codes for span status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    Ok = 1,
    Cancelled = 2,
    Unknown = 3,
    InvalidArgument = 4,
    DeadlineExceeded = 5,
    NotFound = 6,
    AlreadyExists = 7,
    PermissionDenied = 8,
    ResourceExhausted = 9,
    FailedPrecondition = 10,
    Aborted = 11,
    OutOfRange = 12,
    Unimplemented = 13,
    Internal = 14,
    Unavailable = 15,
    DataLoss = 16,
    Unauthenticated = 17,
}

/// Span kind indicating the role of the span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanKind {
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

/// Attribute values for spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Int(i64),
    Double(f64),
    StringArray(Vec<String>),
    BoolArray(Vec<bool>),
    IntArray(Vec<i64>),
    DoubleArray(Vec<f64>),
}

/// Event within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: u64,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Link to another span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLink {
    pub trace_id: Uuid,
    pub span_id: Uuid,
    pub trace_state: TraceState,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Trace state for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceState {
    pub entries: HashMap<String, String>,
}

/// Instrumentation scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationScope {
    pub name: String,
    pub version: Option<String>,
    pub schema_url: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Completed distributed trace ready for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedDistributedTrace {
    pub trace_id: Uuid,
    pub service_name: String,
    pub operation_name: String,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    pub duration_ms: u64,
    pub spans: Vec<CompletedDistributedSpan>,
    pub resource_attributes: HashMap<String, AttributeValue>,
    pub status: TraceStatus,
    pub span_count: usize,
    pub error_count: usize,
}

/// Status of the overall trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceStatus {
    Ok,
    Error { message: String, error_spans: Vec<Uuid> },
    Timeout,
    Partial { missing_spans: usize },
}

/// Completed span for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedDistributedSpan {
    pub span_id: Uuid,
    pub trace_id: Uuid,
    pub parent_span_id: Option<Uuid>,
    pub operation_name: String,
    pub service_name: String,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    pub duration_ns: u64,
    pub status: SpanStatus,
    pub kind: SpanKind,
    pub attributes: HashMap<String, AttributeValue>,
    pub events: Vec<SpanEvent>,
    pub links: Vec<SpanLink>,
    pub instrumentation_scope: InstrumentationScope,
}

/// Events for trace processing
#[derive(Debug)]
pub enum TraceEvent {
    SpanStart(DistributedSpan),
    SpanEnd(Uuid, SpanStatus),
    SpanEvent(Uuid, SpanEvent),
    SpanSetAttribute(Uuid, String, AttributeValue),
    TraceComplete(Uuid),
    Export(Vec<CompletedDistributedTrace>),
}

/// Trace exporter for sending traces to external systems
#[derive(Debug)]
pub struct TraceExporter {
    client: Client,
    config: TracingConfig,
}

/// Jaeger trace format for export
#[derive(Debug, Serialize)]
struct JaegerTrace {
    #[serde(rename = "traceID")]
    trace_id: String,
    spans: Vec<JaegerSpan>,
    processes: HashMap<String, JaegerProcess>,
}

#[derive(Debug, Serialize)]
struct JaegerSpan {
    #[serde(rename = "traceID")]
    trace_id: String,
    #[serde(rename = "spanID")]
    span_id: String,
    #[serde(rename = "parentSpanID")]
    parent_span_id: Option<String>,
    #[serde(rename = "operationName")]
    operation_name: String,
    #[serde(rename = "startTime")]
    start_time: u64,
    duration: u64,
    tags: Vec<JaegerTag>,
    logs: Vec<JaegerLog>,
    #[serde(rename = "processID")]
    process_id: String,
}

#[derive(Debug, Serialize)]
struct JaegerTag {
    key: String,
    #[serde(rename = "type")]
    tag_type: String,
    value: String,
}

#[derive(Debug, Serialize)]
struct JaegerLog {
    timestamp: u64,
    fields: Vec<JaegerTag>,
}

#[derive(Debug, Serialize)]
struct JaegerProcess {
    #[serde(rename = "serviceName")]
    service_name: String,
    tags: Vec<JaegerTag>,
}

impl DistributedTracingManager {
    /// Create a new distributed tracing manager
    pub fn new(config: TracingConfig) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let exporter = Arc::new(TraceExporter::new(config.clone()));
        
        let inner = Arc::new(DistributedTracingInner {
            active_traces: RwLock::new(HashMap::new()),
            trace_queue: RwLock::new(Vec::new()),
            config: config.clone(),
            exporter: exporter.clone(),
            sender,
        });
        
        let manager = Self { inner };
        
        // Start trace processing task
        let inner_clone = manager.inner.clone();
        tokio::spawn(async move {
            Self::process_trace_events(inner_clone, receiver).await;
        });
        
        // Start export batch task
        let inner_clone = manager.inner.clone();
        tokio::spawn(async move {
            Self::export_batch_task(inner_clone).await;
        });
        
        info!("Distributed tracing manager initialized with service: {}", config.service_name);
        manager
    }
    
    /// Start a new distributed trace
    #[instrument(skip(self))]
    pub async fn start_trace(&self, operation_name: &str) -> Result<Uuid, TracingError> {
        let trace_id = Uuid::new_v4();
        
        let trace = DistributedTrace {
            trace_id,
            service_name: self.inner.config.service_name.clone(),
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
            spans: HashMap::new(),
            trace_state: TraceState { entries: HashMap::new() },
            baggage: HashMap::new(),
            resource_attributes: self.create_resource_attributes(),
        };
        
        let mut traces = self.inner.active_traces.write().await;
        traces.insert(trace_id, trace);
        
        debug!("Started distributed trace: {} for operation: {}", trace_id, operation_name);
        Ok(trace_id)
    }
    
    /// Start a new span within a trace
    #[instrument(skip(self))]
    pub async fn start_span(&self, trace_id: Uuid, operation_name: &str, parent_span_id: Option<Uuid>) -> Result<Uuid, TracingError> {
        let span_id = Uuid::new_v4();
        
        let span = DistributedSpan {
            span_id,
            trace_id,
            parent_span_id,
            operation_name: operation_name.to_string(),
            service_name: self.inner.config.service_name.clone(),
            start_time: Instant::now(),
            end_time: None,
            status: SpanStatus::Unset,
            kind: SpanKind::Internal,
            attributes: HashMap::new(),
            events: Vec::new(),
            links: Vec::new(),
            instrumentation_scope: InstrumentationScope {
                name: "opensim-next-tracer".to_string(),
                version: Some(self.inner.config.service_version.clone()),
                schema_url: None,
                attributes: HashMap::new(),
            },
        };
        
        // Add span to trace
        let mut traces = self.inner.active_traces.write().await;
        if let Some(trace) = traces.get_mut(&trace_id) {
            trace.spans.insert(span_id, span.clone());
        } else {
            return Err(TracingError::TraceNotFound(trace_id));
        }
        
        // Send event
        if let Err(_) = self.inner.sender.send(TraceEvent::SpanStart(span)) {
            warn!("Failed to send span start event");
        }
        
        debug!("Started span: {} in trace: {} for operation: {}", span_id, trace_id, operation_name);
        Ok(span_id)
    }
    
    /// End a span
    #[instrument(skip(self))]
    pub async fn end_span(&self, trace_id: Uuid, span_id: Uuid, status: SpanStatus) -> Result<(), TracingError> {
        let mut traces = self.inner.active_traces.write().await;
        if let Some(trace) = traces.get_mut(&trace_id) {
            if let Some(span) = trace.spans.get_mut(&span_id) {
                span.end_time = Some(Instant::now());
                span.status = status.clone();
            } else {
                return Err(TracingError::SpanNotFound(span_id));
            }
        } else {
            return Err(TracingError::TraceNotFound(trace_id));
        }
        
        // Send event
        if let Err(_) = self.inner.sender.send(TraceEvent::SpanEnd(span_id, status)) {
            warn!("Failed to send span end event");
        }
        
        debug!("Ended span: {} in trace: {}", span_id, trace_id);
        Ok(())
    }
    
    /// Add an attribute to a span
    #[instrument(skip(self))]
    pub async fn set_span_attribute(&self, trace_id: Uuid, span_id: Uuid, key: &str, value: AttributeValue) -> Result<(), TracingError> {
        let mut traces = self.inner.active_traces.write().await;
        if let Some(trace) = traces.get_mut(&trace_id) {
            if let Some(span) = trace.spans.get_mut(&span_id) {
                span.attributes.insert(key.to_string(), value.clone());
            } else {
                return Err(TracingError::SpanNotFound(span_id));
            }
        } else {
            return Err(TracingError::TraceNotFound(trace_id));
        }
        
        // Send event
        if let Err(_) = self.inner.sender.send(TraceEvent::SpanSetAttribute(span_id, key.to_string(), value)) {
            warn!("Failed to send span attribute event");
        }
        
        Ok(())
    }
    
    /// Add an event to a span
    #[instrument(skip(self))]
    pub async fn add_span_event(&self, trace_id: Uuid, span_id: Uuid, event: SpanEvent) -> Result<(), TracingError> {
        let mut traces = self.inner.active_traces.write().await;
        if let Some(trace) = traces.get_mut(&trace_id) {
            if let Some(span) = trace.spans.get_mut(&span_id) {
                span.events.push(event.clone());
            } else {
                return Err(TracingError::SpanNotFound(span_id));
            }
        } else {
            return Err(TracingError::TraceNotFound(trace_id));
        }
        
        // Send event
        if let Err(_) = self.inner.sender.send(TraceEvent::SpanEvent(span_id, event)) {
            warn!("Failed to send span event");
        }
        
        Ok(())
    }
    
    /// Complete a trace
    #[instrument(skip(self))]
    pub async fn complete_trace(&self, trace_id: Uuid) -> Result<(), TracingError> {
        let mut traces = self.inner.active_traces.write().await;
        if let Some(trace) = traces.remove(&trace_id) {
            let completed_trace = self.convert_to_completed_trace(trace).await;
            
            let mut queue = self.inner.trace_queue.write().await;
            queue.push(completed_trace);
        } else {
            return Err(TracingError::TraceNotFound(trace_id));
        }
        
        // Send completion event
        if let Err(_) = self.inner.sender.send(TraceEvent::TraceComplete(trace_id)) {
            warn!("Failed to send trace complete event");
        }
        
        debug!("Completed trace: {}", trace_id);
        Ok(())
    }
    
    /// Create resource attributes for the service
    fn create_resource_attributes(&self) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        attributes.insert("service.name".to_string(), self.inner.config.service_name.clone());
        attributes.insert("service.version".to_string(), self.inner.config.service_version.clone());
        attributes.insert("telemetry.sdk.name".to_string(), "opensim-next-tracing".to_string());
        attributes.insert("telemetry.sdk.version".to_string(), "30.0.0".to_string());
        attributes.insert("telemetry.sdk.language".to_string(), "rust".to_string());
        attributes
    }
    
    /// Convert active trace to completed trace
    async fn convert_to_completed_trace(&self, trace: DistributedTrace) -> CompletedDistributedTrace {
        let end_time = Instant::now();
        let duration = end_time.duration_since(trace.start_time);
        
        let mut completed_spans = Vec::new();
        let mut error_count = 0;
        
        for span in trace.spans.values() {
            let span_end_time = span.end_time.unwrap_or(end_time);
            let span_duration = span_end_time.duration_since(span.start_time);
            
            if matches!(span.status, SpanStatus::Error { .. }) {
                error_count += 1;
            }
            
            let completed_span = CompletedDistributedSpan {
                span_id: span.span_id,
                trace_id: span.trace_id,
                parent_span_id: span.parent_span_id,
                operation_name: span.operation_name.clone(),
                service_name: span.service_name.clone(),
                start_time_unix_nano: self.instant_to_unix_nano(span.start_time),
                end_time_unix_nano: self.instant_to_unix_nano(span_end_time),
                duration_ns: span_duration.as_nanos() as u64,
                status: span.status.clone(),
                kind: span.kind.clone(),
                attributes: span.attributes.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                events: span.events.clone(),
                links: span.links.clone(),
                instrumentation_scope: span.instrumentation_scope.clone(),
            };
            
            completed_spans.push(completed_span);
        }
        
        let status = if error_count > 0 {
            TraceStatus::Error {
                message: format!("{} spans had errors", error_count),
                error_spans: completed_spans.iter()
                    .filter(|s| matches!(s.status, SpanStatus::Error { .. }))
                    .map(|s| s.span_id)
                    .collect(),
            }
        } else {
            TraceStatus::Ok
        };
        
        CompletedDistributedTrace {
            trace_id: trace.trace_id,
            service_name: trace.service_name,
            operation_name: trace.operation_name,
            start_time_unix_nano: self.instant_to_unix_nano(trace.start_time),
            end_time_unix_nano: self.instant_to_unix_nano(end_time),
            duration_ms: duration.as_millis() as u64,
            spans: completed_spans,
            resource_attributes: trace.resource_attributes.iter()
                .map(|(k, v)| (k.clone(), AttributeValue::String(v.clone())))
                .collect(),
            status,
            span_count: trace.spans.len(),
            error_count,
        }
    }
    
    /// Convert Instant to Unix nanoseconds
    fn instant_to_unix_nano(&self, instant: Instant) -> u64 {
        let system_time = SystemTime::now();
        let since_epoch = system_time.duration_since(UNIX_EPOCH).unwrap_or_default();
        since_epoch.as_nanos() as u64
    }
    
    /// Process trace events
    async fn process_trace_events(inner: Arc<DistributedTracingInner>, mut receiver: mpsc::UnboundedReceiver<TraceEvent>) {
        while let Some(event) = receiver.recv().await {
            match event {
                TraceEvent::SpanStart(span) => {
                    debug!("Processing span start: {}", span.span_id);
                }
                TraceEvent::SpanEnd(span_id, status) => {
                    debug!("Processing span end: {} with status: {:?}", span_id, status);
                }
                TraceEvent::SpanEvent(span_id, event) => {
                    debug!("Processing span event: {} for span: {}", event.name, span_id);
                }
                TraceEvent::SpanSetAttribute(span_id, key, value) => {
                    debug!("Processing span attribute: {}={:?} for span: {}", key, value, span_id);
                }
                TraceEvent::TraceComplete(trace_id) => {
                    debug!("Processing trace completion: {}", trace_id);
                }
                TraceEvent::Export(traces) => {
                    if let Err(e) = inner.exporter.export_traces(traces).await {
                        error!("Failed to export traces: {}", e);
                    }
                }
            }
        }
    }
    
    /// Export batch task
    async fn export_batch_task(inner: Arc<DistributedTracingInner>) {
        let mut interval = tokio::time::interval(Duration::from_millis(inner.config.batch_timeout_ms));
        
        loop {
            interval.tick().await;
            
            let mut queue = inner.trace_queue.write().await;
            if queue.len() >= inner.config.batch_size {
                let batch: Vec<_> = queue.drain(..inner.config.batch_size).collect();
                drop(queue);
                
                if let Err(e) = inner.exporter.export_traces(batch).await {
                    error!("Failed to export trace batch: {}", e);
                }
            }
        }
    }
}

impl TraceExporter {
    fn new(config: TracingConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
    
    async fn export_traces(&self, traces: Vec<CompletedDistributedTrace>) -> Result<(), TracingError> {
        if let Some(jaeger_endpoint) = &self.config.jaeger_endpoint {
            self.export_to_jaeger(jaeger_endpoint, &traces).await?;
        }
        
        if let Some(zipkin_endpoint) = &self.config.zipkin_endpoint {
            self.export_to_zipkin(zipkin_endpoint, &traces).await?;
        }
        
        if let Some(otlp_endpoint) = &self.config.otlp_endpoint {
            self.export_to_otlp(otlp_endpoint, &traces).await?;
        }
        
        info!("Exported {} traces successfully", traces.len());
        Ok(())
    }
    
    async fn export_to_jaeger(&self, endpoint: &str, traces: &[CompletedDistributedTrace]) -> Result<(), TracingError> {
        let jaeger_traces: Vec<JaegerTrace> = traces.iter()
            .map(|trace| self.convert_to_jaeger_trace(trace))
            .collect();
        
        let response = self.client
            .post(endpoint)
            .json(&jaeger_traces)
            .send()
            .await
            .map_err(|e| TracingError::ExportError(e.to_string()))?;
            
        if !response.status().is_success() {
            return Err(TracingError::ExportError(format!("Jaeger export failed: {}", response.status())));
        }
        
        debug!("Successfully exported {} traces to Jaeger", traces.len());
        Ok(())
    }
    
    async fn export_to_zipkin(&self, endpoint: &str, traces: &[CompletedDistributedTrace]) -> Result<(), TracingError> {
        // Zipkin export implementation would go here
        debug!("Zipkin export not yet implemented");
        Ok(())
    }
    
    async fn export_to_otlp(&self, endpoint: &str, traces: &[CompletedDistributedTrace]) -> Result<(), TracingError> {
        // OTLP export implementation would go here
        debug!("OTLP export not yet implemented");
        Ok(())
    }
    
    fn convert_to_jaeger_trace(&self, trace: &CompletedDistributedTrace) -> JaegerTrace {
        let trace_id_hex = format!("{:x}", trace.trace_id.as_u128());
        
        let spans: Vec<JaegerSpan> = trace.spans.iter()
            .map(|span| self.convert_to_jaeger_span(span))
            .collect();
        
        let mut processes = HashMap::new();
        processes.insert("p1".to_string(), JaegerProcess {
            service_name: trace.service_name.clone(),
            tags: vec![
                JaegerTag {
                    key: "version".to_string(),
                    tag_type: "string".to_string(),
                    value: self.config.service_version.clone(),
                },
            ],
        });
        
        JaegerTrace {
            trace_id: trace_id_hex,
            spans,
            processes,
        }
    }
    
    fn convert_to_jaeger_span(&self, span: &CompletedDistributedSpan) -> JaegerSpan {
        let span_id_hex = format!("{:x}", span.span_id.as_u128() & 0xFFFFFFFFFFFFFFFF);
        let trace_id_hex = format!("{:x}", span.trace_id.as_u128());
        let parent_span_id_hex = span.parent_span_id.map(|id| format!("{:x}", id.as_u128() & 0xFFFFFFFFFFFFFFFF));
        
        let tags: Vec<JaegerTag> = span.attributes.iter()
            .map(|(key, value)| JaegerTag {
                key: key.clone(),
                tag_type: self.attribute_value_to_jaeger_type(value),
                value: self.attribute_value_to_string(value),
            })
            .collect();
        
        let logs: Vec<JaegerLog> = span.events.iter()
            .map(|event| JaegerLog {
                timestamp: event.timestamp,
                fields: vec![JaegerTag {
                    key: "event".to_string(),
                    tag_type: "string".to_string(),
                    value: event.name.clone(),
                }],
            })
            .collect();
        
        JaegerSpan {
            trace_id: trace_id_hex,
            span_id: span_id_hex,
            parent_span_id: parent_span_id_hex,
            operation_name: span.operation_name.clone(),
            start_time: span.start_time_unix_nano / 1000, // Jaeger expects microseconds
            duration: span.duration_ns / 1000, // Jaeger expects microseconds
            tags,
            logs,
            process_id: "p1".to_string(),
        }
    }
    
    fn attribute_value_to_jaeger_type(&self, value: &AttributeValue) -> String {
        match value {
            AttributeValue::String(_) => "string".to_string(),
            AttributeValue::Bool(_) => "bool".to_string(),
            AttributeValue::Int(_) => "number".to_string(),
            AttributeValue::Double(_) => "number".to_string(),
            _ => "string".to_string(),
        }
    }
    
    fn attribute_value_to_string(&self, value: &AttributeValue) -> String {
        match value {
            AttributeValue::String(s) => s.clone(),
            AttributeValue::Bool(b) => b.to_string(),
            AttributeValue::Int(i) => i.to_string(),
            AttributeValue::Double(d) => d.to_string(),
            AttributeValue::StringArray(arr) => arr.join(","),
            AttributeValue::BoolArray(arr) => arr.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","),
            AttributeValue::IntArray(arr) => arr.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","),
            AttributeValue::DoubleArray(arr) => arr.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(","),
        }
    }
}

/// Errors that can occur during tracing operations
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Trace not found: {0}")]
    TraceNotFound(Uuid),
    #[error("Span not found: {0}")]
    SpanNotFound(Uuid),
    #[error("Export error: {0}")]
    ExportError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Sampling error: {0}")]
    SamplingError(String),
}

impl Default for TraceState {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}