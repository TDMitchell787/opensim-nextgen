//! Comprehensive logging and monitoring system

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Write,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Log levels for structured logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<Level> for LogLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
    pub module: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub thread_id: String,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub console_output: bool,
    pub file_output: bool,
    pub json_format: bool,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
    pub max_files: u32,
    pub include_source_location: bool,
    pub include_thread_info: bool,
    pub buffer_size: usize,
    pub flush_interval_ms: u64,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            console_output: true,
            file_output: true,
            json_format: false,
            file_path: Some("logs/opensim.log".to_string()),
            max_file_size_mb: 100,
            max_files: 10,
            include_source_location: true,
            include_thread_info: true,
            buffer_size: 8192,
            flush_interval_ms: 1000,
        }
    }
}

/// Log aggregator for collecting and analyzing logs
pub struct LogAggregator {
    config: LoggingConfig,
    log_buffer: Arc<RwLock<Vec<LogEntry>>>,
    log_stats: Arc<RwLock<LogStatistics>>,
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
}

/// Log statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogStatistics {
    pub total_logs: u64,
    pub logs_by_level: HashMap<LogLevel, u64>,
    pub logs_by_module: HashMap<String, u64>,
    pub error_rate_per_minute: f64,
    pub warning_rate_per_minute: f64,
    pub last_error: Option<LogEntry>,
    pub last_warning: Option<LogEntry>,
    pub uptime_seconds: u64,
    pub start_time: u64,
}

/// Alert rule for log monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub window_minutes: u32,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub last_triggered: Option<chrono::DateTime<chrono::Utc>>,
}

/// Alert conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    ErrorRateExceeds,
    WarningRateExceeds,
    LogVolumeExceeds,
    SpecificErrorPattern { pattern: String },
    ModuleErrorRate { module: String },
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Generated alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub context: HashMap<String, serde_json::Value>,
}

impl LogAggregator {
    /// Create a new log aggregator
    pub fn new(config: LoggingConfig) -> Self {
        let stats = LogStatistics {
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ..Default::default()
        };

        Self {
            config,
            log_buffer: Arc::new(RwLock::new(Vec::new())),
            log_stats: Arc::new(RwLock::new(stats)),
            alert_rules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize the logging system
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing comprehensive logging system");

        // Set up tracing subscriber
        let filter = match self.config.level {
            LogLevel::Trace => EnvFilter::new("trace"),
            LogLevel::Debug => EnvFilter::new("debug"),
            LogLevel::Info => EnvFilter::new("info"),
            LogLevel::Warn => EnvFilter::new("warn"),
            LogLevel::Error => EnvFilter::new("error"),
        };

        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(true).with_thread_ids(true));

        subscriber.init();

        // Set up default alert rules
        self.setup_default_alert_rules().await?;

        // Start log processing
        self.start_log_processing().await?;

        info!("Logging system initialized successfully");
        Ok(())
    }

    /// Record a log entry
    pub async fn record_log(&self, entry: LogEntry) -> Result<()> {
        // Add to buffer
        let mut buffer = self.log_buffer.write().await;
        buffer.push(entry.clone());

        // Enforce buffer size limit
        if buffer.len() > self.config.buffer_size {
            let drain_count = buffer.len() / 2;
            buffer.drain(0..drain_count); // Remove older half
        }

        // Update statistics
        let mut stats = self.log_stats.write().await;
        stats.total_logs += 1;
        *stats.logs_by_level.entry(entry.level.clone()).or_insert(0) += 1;
        *stats
            .logs_by_module
            .entry(entry.module.clone())
            .or_insert(0) += 1;

        // Update uptime
        stats.uptime_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - stats.start_time;

        // Track last error/warning
        match entry.level {
            LogLevel::Error => {
                stats.last_error = Some(entry.clone());
            }
            LogLevel::Warn => {
                stats.last_warning = Some(entry.clone());
            }
            _ => {}
        }

        // Check alert rules
        self.check_alert_rules(&entry).await?;

        Ok(())
    }

    /// Add an alert rule
    pub async fn add_alert_rule(&self, rule: AlertRule) -> Result<()> {
        info!("Adding alert rule: {}", rule.name);
        self.alert_rules.write().await.push(rule);
        Ok(())
    }

    /// Remove an alert rule
    pub async fn remove_alert_rule(&self, rule_id: &str) -> Result<bool> {
        let mut rules = self.alert_rules.write().await;
        let initial_len = rules.len();
        rules.retain(|r| r.id != rule_id);
        Ok(rules.len() < initial_len)
    }

    /// Get current log statistics
    pub async fn get_statistics(&self) -> Result<LogStatistics> {
        let stats = self.log_stats.read().await;
        Ok(stats.clone())
    }

    /// Search logs with filters
    pub async fn search_logs(
        &self,
        level_filter: Option<LogLevel>,
        module_filter: Option<String>,
        message_filter: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<LogEntry>> {
        let buffer = self.log_buffer.read().await;
        let limit = limit.unwrap_or(1000);

        let filtered: Vec<_> = buffer
            .iter()
            .rev() // Most recent first
            .filter(|entry| {
                if let Some(ref level) = level_filter {
                    if entry.level != *level {
                        return false;
                    }
                }
                if let Some(ref module) = module_filter {
                    if !entry.module.contains(module) {
                        return false;
                    }
                }
                if let Some(ref message) = message_filter {
                    if !entry.message.contains(message) {
                        return false;
                    }
                }
                true
            })
            .take(limit)
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Export logs in various formats
    pub async fn export_logs(&self, format: LogExportFormat) -> Result<String> {
        let buffer = self.log_buffer.read().await;

        match format {
            LogExportFormat::Json => serde_json::to_string_pretty(&*buffer)
                .map_err(|e| anyhow!("Failed to serialize logs: {}", e)),
            LogExportFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("timestamp,level,module,message,thread_id\n");

                for entry in buffer.iter() {
                    csv.push_str(&format!(
                        "{},{:?},{},{},{}\n",
                        entry.timestamp,
                        entry.level,
                        entry.module,
                        entry.message.replace(',', ";"),
                        entry.thread_id
                    ));
                }

                Ok(csv)
            }
            LogExportFormat::Logfmt => {
                let mut logfmt = String::new();

                for entry in buffer.iter() {
                    logfmt.push_str(&format!(
                        "ts={} level={:?} module={} msg=\"{}\" thread={}\n",
                        entry.timestamp, entry.level, entry.module, entry.message, entry.thread_id
                    ));
                }

                Ok(logfmt)
            }
        }
    }

    /// Clear log buffer
    pub async fn clear_logs(&self) -> Result<()> {
        let mut buffer = self.log_buffer.write().await;
        let cleared_count = buffer.len();
        buffer.clear();

        info!("Cleared {} log entries", cleared_count);
        Ok(())
    }

    async fn setup_default_alert_rules(&self) -> Result<()> {
        let default_rules = vec![
            AlertRule {
                id: "high_error_rate".to_string(),
                name: "High Error Rate".to_string(),
                condition: AlertCondition::ErrorRateExceeds,
                threshold: 10.0, // 10 errors per minute
                window_minutes: 5,
                severity: AlertSeverity::High,
                enabled: true,
                last_triggered: None,
            },
            AlertRule {
                id: "high_warning_rate".to_string(),
                name: "High Warning Rate".to_string(),
                condition: AlertCondition::WarningRateExceeds,
                threshold: 50.0, // 50 warnings per minute
                window_minutes: 5,
                severity: AlertSeverity::Medium,
                enabled: true,
                last_triggered: None,
            },
            AlertRule {
                id: "database_errors".to_string(),
                name: "Database Connection Errors".to_string(),
                condition: AlertCondition::SpecificErrorPattern {
                    pattern: "database".to_string(),
                },
                threshold: 3.0, // 3 database errors
                window_minutes: 1,
                severity: AlertSeverity::Critical,
                enabled: true,
                last_triggered: None,
            },
        ];

        let mut rules = self.alert_rules.write().await;
        rules.extend(default_rules);

        Ok(())
    }

    async fn start_log_processing(&self) -> Result<()> {
        let stats = self.log_stats.clone();
        let buffer = self.log_buffer.clone();
        let flush_interval = self.config.flush_interval_ms;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(flush_interval));

            loop {
                interval.tick().await;

                // Calculate rates
                let buffer_guard = buffer.read().await;
                let mut stats_guard = stats.write().await;

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let one_minute_ago = now - 60;

                let recent_errors = buffer_guard
                    .iter()
                    .filter(|entry| {
                        entry.timestamp >= one_minute_ago && entry.level == LogLevel::Error
                    })
                    .count();

                let recent_warnings = buffer_guard
                    .iter()
                    .filter(|entry| {
                        entry.timestamp >= one_minute_ago && entry.level == LogLevel::Warn
                    })
                    .count();

                stats_guard.error_rate_per_minute = recent_errors as f64;
                stats_guard.warning_rate_per_minute = recent_warnings as f64;

                debug!(
                    "Log processing: {} errors/min, {} warnings/min",
                    recent_errors, recent_warnings
                );
            }
        });

        Ok(())
    }

    async fn check_alert_rules(&self, entry: &LogEntry) -> Result<()> {
        let rules = self.alert_rules.read().await;

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            let should_trigger = match &rule.condition {
                AlertCondition::ErrorRateExceeds => {
                    entry.level == LogLevel::Error
                        && self
                            .check_rate_threshold(
                                LogLevel::Error,
                                rule.threshold,
                                rule.window_minutes,
                            )
                            .await?
                }
                AlertCondition::WarningRateExceeds => {
                    entry.level == LogLevel::Warn
                        && self
                            .check_rate_threshold(
                                LogLevel::Warn,
                                rule.threshold,
                                rule.window_minutes,
                            )
                            .await?
                }
                AlertCondition::SpecificErrorPattern { pattern } => {
                    entry.level == LogLevel::Error
                        && entry
                            .message
                            .to_lowercase()
                            .contains(&pattern.to_lowercase())
                }
                AlertCondition::ModuleErrorRate { module } => {
                    entry.level == LogLevel::Error
                        && entry.module == *module
                        && self
                            .check_module_rate_threshold(
                                module,
                                rule.threshold,
                                rule.window_minutes,
                            )
                            .await?
                }
                AlertCondition::LogVolumeExceeds => {
                    self.check_volume_threshold(rule.threshold, rule.window_minutes)
                        .await?
                }
            };

            if should_trigger {
                self.trigger_alert(rule, entry).await?;
            }
        }

        Ok(())
    }

    async fn check_rate_threshold(
        &self,
        level: LogLevel,
        threshold: f64,
        window_minutes: u32,
    ) -> Result<bool> {
        let buffer = self.log_buffer.read().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now - (window_minutes as u64 * 60);

        let count = buffer
            .iter()
            .filter(|entry| entry.timestamp >= window_start && entry.level == level)
            .count();

        let rate_per_minute = count as f64 / window_minutes as f64;
        Ok(rate_per_minute >= threshold)
    }

    async fn check_module_rate_threshold(
        &self,
        module: &str,
        threshold: f64,
        window_minutes: u32,
    ) -> Result<bool> {
        let buffer = self.log_buffer.read().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now - (window_minutes as u64 * 60);

        let count = buffer
            .iter()
            .filter(|entry| {
                entry.timestamp >= window_start
                    && entry.level == LogLevel::Error
                    && entry.module == module
            })
            .count();

        let rate_per_minute = count as f64 / window_minutes as f64;
        Ok(rate_per_minute >= threshold)
    }

    async fn check_volume_threshold(&self, threshold: f64, window_minutes: u32) -> Result<bool> {
        let buffer = self.log_buffer.read().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now - (window_minutes as u64 * 60);

        let count = buffer
            .iter()
            .filter(|entry| entry.timestamp >= window_start)
            .count();

        let rate_per_minute = count as f64 / window_minutes as f64;
        Ok(rate_per_minute >= threshold)
    }

    async fn trigger_alert(&self, rule: &AlertRule, entry: &LogEntry) -> Result<()> {
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            severity: rule.severity.clone(),
            message: format!("Alert: {} triggered by log: {}", rule.name, entry.message),
            triggered_at: chrono::Utc::now(),
            resolved_at: None,
            context: HashMap::from([
                (
                    "module".to_string(),
                    serde_json::Value::String(entry.module.clone()),
                ),
                (
                    "level".to_string(),
                    serde_json::Value::String(format!("{:?}", entry.level)),
                ),
                (
                    "thread_id".to_string(),
                    serde_json::Value::String(entry.thread_id.clone()),
                ),
            ]),
        };

        match rule.severity {
            AlertSeverity::Critical => error!("CRITICAL ALERT: {}", alert.message),
            AlertSeverity::High => error!("HIGH ALERT: {}", alert.message),
            AlertSeverity::Medium => warn!("MEDIUM ALERT: {}", alert.message),
            AlertSeverity::Low => info!("LOW ALERT: {}", alert.message),
        }

        // In a real implementation, this would send the alert to external systems
        // (email, Slack, PagerDuty, etc.)

        Ok(())
    }
}

/// Log export formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogExportFormat {
    Json,
    Csv,
    Logfmt,
}

/// Log query builder for complex searches
pub struct LogQuery {
    level: Option<LogLevel>,
    module: Option<String>,
    message_contains: Option<String>,
    time_range: Option<(u64, u64)>,
    thread_id: Option<String>,
    limit: Option<usize>,
}

impl LogQuery {
    pub fn new() -> Self {
        Self {
            level: None,
            module: None,
            message_contains: None,
            time_range: None,
            thread_id: None,
            limit: None,
        }
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_module(mut self, module: &str) -> Self {
        self.module = Some(module.to_string());
        self
    }

    pub fn with_message_containing(mut self, text: &str) -> Self {
        self.message_contains = Some(text.to_string());
        self
    }

    pub fn with_time_range(mut self, start: u64, end: u64) -> Self {
        self.time_range = Some((start, end));
        self
    }

    pub fn with_thread_id(mut self, thread_id: &str) -> Self {
        self.thread_id = Some(thread_id.to_string());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub async fn execute(&self, aggregator: &LogAggregator) -> Result<Vec<LogEntry>> {
        let buffer = aggregator.log_buffer.read().await;
        let limit = self.limit.unwrap_or(1000);

        let filtered: Vec<_> = buffer
            .iter()
            .rev()
            .filter(|entry| {
                if let Some(ref level) = self.level {
                    if entry.level != *level {
                        return false;
                    }
                }
                if let Some(ref module) = self.module {
                    if !entry.module.contains(module) {
                        return false;
                    }
                }
                if let Some(ref message) = self.message_contains {
                    if !entry.message.contains(message) {
                        return false;
                    }
                }
                if let Some((start, end)) = self.time_range {
                    if entry.timestamp < start || entry.timestamp > end {
                        return false;
                    }
                }
                if let Some(ref thread_id) = self.thread_id {
                    if entry.thread_id != *thread_id {
                        return false;
                    }
                }
                true
            })
            .take(limit)
            .cloned()
            .collect();

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_aggregator() -> Result<()> {
        let config = LoggingConfig::default();
        let aggregator = LogAggregator::new(config);

        let entry = LogEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            level: LogLevel::Info,
            message: "Test log message".to_string(),
            module: "test".to_string(),
            file: None,
            line: None,
            thread_id: "main".to_string(),
            span_id: None,
            trace_id: None,
            fields: HashMap::new(),
        };

        aggregator.record_log(entry).await?;

        let stats = aggregator.get_statistics().await?;
        assert_eq!(stats.total_logs, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_log_search() -> Result<()> {
        let config = LoggingConfig::default();
        let aggregator = LogAggregator::new(config);

        // Add some test logs
        for i in 0..5 {
            let entry = LogEntry {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                level: if i % 2 == 0 {
                    LogLevel::Info
                } else {
                    LogLevel::Error
                },
                message: format!("Test message {}", i),
                module: "test".to_string(),
                file: None,
                line: None,
                thread_id: "main".to_string(),
                span_id: None,
                trace_id: None,
                fields: HashMap::new(),
            };
            aggregator.record_log(entry).await?;
        }

        // Search for error logs
        let errors = aggregator
            .search_logs(Some(LogLevel::Error), None, None, None)
            .await?;
        assert_eq!(errors.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_log_export() -> Result<()> {
        let config = LoggingConfig::default();
        let aggregator = LogAggregator::new(config);

        let entry = LogEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            level: LogLevel::Info,
            message: "Test export".to_string(),
            module: "test".to_string(),
            file: None,
            line: None,
            thread_id: "main".to_string(),
            span_id: None,
            trace_id: None,
            fields: HashMap::new(),
        };

        aggregator.record_log(entry).await?;

        let json_export = aggregator.export_logs(LogExportFormat::Json).await?;
        assert!(json_export.contains("Test export"));

        let csv_export = aggregator.export_logs(LogExportFormat::Csv).await?;
        assert!(csv_export.contains("timestamp,level,module,message,thread_id"));

        Ok(())
    }
}
