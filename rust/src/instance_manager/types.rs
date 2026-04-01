//! Instance Manager Types
//!
//! Shared types for instance management, control, and monitoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Instance operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    Discovered,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
    Maintenance,
    Unknown,
    Disconnected,
}

impl Default for InstanceStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Environment type for an instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

/// Authentication method for instance connections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    ApiKey,
    Jwt,
    Mtls,
}

impl Default for AuthMethod {
    fn default() -> Self {
        Self::ApiKey
    }
}

/// Commands that can be sent to control an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceCommand {
    Start,
    Stop,
    Restart,
    Shutdown,
    ForceShutdown,
    Reload,
    Backup,
    GetStatus,
    GetLogs,
    GetMetrics,
}

/// Real-time metrics from an instance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub active_users: u32,
    pub active_regions: u32,
    pub network_tx_bytes: u64,
    pub network_rx_bytes: u64,
    pub db_connections: u32,
    pub websocket_connections: u32,
    pub request_rate_per_sec: f64,
    pub error_rate_per_sec: f64,
    pub uptime_seconds: u64,
}

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl Default for HealthState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Health status for an individual component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthState,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// Complete health status for an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall: HealthState,
    pub components: HashMap<String, ComponentHealth>,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: u64,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            overall: HealthState::Unknown,
            components: HashMap::new(),
            last_check: Utc::now(),
            response_time_ms: 0,
        }
    }
}

/// Console output type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsoleOutputType {
    Stdout,
    Stderr,
    Info,
    Warning,
    Error,
    Debug,
    Command,
}

/// Console entry (input or output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    pub instance_id: String,
    pub content: String,
    pub output_type: ConsoleOutputType,
    pub timestamp: DateTime<Utc>,
}

/// User management actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UserManagementAction {
    List { limit: Option<u32>, offset: Option<u32> },
    Get { user_id: String },
    Create { user: serde_json::Value },
    Update { user_id: String, updates: serde_json::Value },
    Delete { user_id: String },
    ResetPassword { user_id: String, new_password: String },
    SetLevel { user_id: String, level: i32 },
    Kick { user_id: String, reason: String },
    Ban { user_id: String, reason: String, duration_hours: Option<u32> },
}

/// Region control actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RegionControlAction {
    List,
    Get { region_id: String },
    Start { region_id: String },
    Stop { region_id: String },
    Restart { region_id: String },
    Backup { region_id: String },
    LoadOar { region_id: String, oar_path: String },
    SaveOar { region_id: String, oar_path: String },
    TeleportAll { region_id: String, target_region: String },
}

/// Subscription channels for real-time updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionChannel {
    StatusUpdates,
    Metrics,
    Logs,
    Console,
    Alerts,
    UserActivity,
    RegionActivity,
}

/// Batch operation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
    Timeout,
}

/// Result of a batch operation on a single instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub instance_id: String,
    pub status: BatchStatus,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub duration_ms: u64,
}

/// Result of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub duration_ms: u64,
}

impl CommandResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            duration_ms: 0,
        }
    }

    pub fn success_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            duration_ms: 0,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            duration_ms: 0,
        }
    }
}

/// Runtime information about a connected instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub environment: Environment,
    pub status: InstanceStatus,
    pub metrics: Option<InstanceMetrics>,
    pub health: Option<HealthStatus>,
    pub version: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub connected: bool,
    pub tags: Vec<String>,
}
