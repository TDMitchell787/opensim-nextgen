use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

use super::process_manager::ProcessManager;
use super::registry::InstanceRegistry;
use super::types::InstanceStatus;

#[derive(Debug, Deserialize)]
pub struct AnnouncementPorts {
    pub login: Option<u16>,
    pub admin: Option<u16>,
    pub metrics: Option<u16>,
    pub websocket: Option<u16>,
    pub udp_start: Option<u16>,
    pub udp_end: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct InstanceAnnouncement {
    pub instance_id: String,
    pub service_mode: String,
    pub ports: AnnouncementPorts,
    pub region_count: u32,
    pub capabilities: Vec<String>,
    pub version: String,
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub instance_id: String,
    pub status: String,
    pub active_users: u32,
    pub active_regions: u32,
    pub uptime_seconds: u64,
    pub cpu_usage: f64,
    pub memory_usage_mb: u64,
}

#[derive(Debug, Serialize)]
pub struct AnnounceResponse {
    pub success: bool,
    pub message: String,
}

pub struct ControllerState {
    pub registry: Arc<InstanceRegistry>,
    pub process_manager: Arc<ProcessManager>,
    pub controller_port: u16,
}

pub async fn handle_announce(
    State(state): State<Arc<ControllerState>>,
    Json(announcement): Json<InstanceAnnouncement>,
) -> (StatusCode, Json<AnnounceResponse>) {
    info!(
        "Instance announcement: {} (mode={}, host={}, regions={})",
        announcement.instance_id,
        announcement.service_mode,
        announcement.host,
        announcement.region_count
    );

    let instance_exists = state.registry.has_instance(&announcement.instance_id).await;

    if instance_exists {
        if let Err(e) = state
            .registry
            .update_status(&announcement.instance_id, InstanceStatus::Running)
            .await
        {
            warn!("Failed to update instance status: {}", e);
        }
        if let Err(e) = state
            .registry
            .update_connected(&announcement.instance_id, true)
            .await
        {
            warn!("Failed to update instance connection: {}", e);
        }
        if let Err(e) = state
            .registry
            .update_version(&announcement.instance_id, announcement.version.clone())
            .await
        {
            warn!("Failed to update instance version: {}", e);
        }
    } else {
        use super::config_loader::InstanceConfig;
        use super::types::Environment;

        let config = InstanceConfig {
            id: announcement.instance_id.clone(),
            name: announcement.instance_id.clone(),
            description: format!("{} instance", announcement.service_mode),
            host: announcement.host.clone(),
            websocket_port: announcement.ports.websocket.unwrap_or(9001),
            admin_port: announcement.ports.admin.unwrap_or(9200),
            metrics_port: announcement.ports.metrics.unwrap_or(9100),
            http_port: announcement.ports.login.unwrap_or(9000),
            udp_port: announcement.ports.udp_start.unwrap_or(9000),
            api_key: "announced".to_string(),
            environment: Environment::Development,
            auto_connect: false,
            tags: vec![announcement.service_mode.clone()],
            authentication: Default::default(),
            tls: Default::default(),
        };

        if let Err(e) = state.registry.add_instance(config).await {
            warn!("Failed to register announced instance: {}", e);
        }
        if let Err(e) = state
            .registry
            .update_status(&announcement.instance_id, InstanceStatus::Running)
            .await
        {
            warn!("Failed to set Running status: {}", e);
        }
        if let Err(e) = state
            .registry
            .update_connected(&announcement.instance_id, true)
            .await
        {
            warn!("Failed to update connection: {}", e);
        }
    }

    (
        StatusCode::OK,
        Json(AnnounceResponse {
            success: true,
            message: format!("Instance {} registered", announcement.instance_id),
        }),
    )
}

pub async fn handle_heartbeat(
    State(state): State<Arc<ControllerState>>,
    Json(heartbeat): Json<HeartbeatRequest>,
) -> (StatusCode, Json<AnnounceResponse>) {
    let metrics = super::types::InstanceMetrics {
        cpu_usage: heartbeat.cpu_usage,
        memory_usage_mb: heartbeat.memory_usage_mb,
        active_users: heartbeat.active_users,
        active_regions: heartbeat.active_regions,
        uptime_seconds: heartbeat.uptime_seconds,
        ..Default::default()
    };

    if let Err(e) = state
        .registry
        .update_metrics(&heartbeat.instance_id, metrics)
        .await
    {
        warn!(
            "Failed to update metrics for {}: {}",
            heartbeat.instance_id, e
        );
        return (
            StatusCode::NOT_FOUND,
            Json(AnnounceResponse {
                success: false,
                message: format!("Instance {} not found", heartbeat.instance_id),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(AnnounceResponse {
            success: true,
            message: "Heartbeat received".to_string(),
        }),
    )
}

pub async fn handle_list_instances(
    State(state): State<Arc<ControllerState>>,
) -> Json<serde_json::Value> {
    let instances = state.registry.get_all_instances().await;
    Json(serde_json::json!({
        "instances": instances,
        "count": instances.len(),
    }))
}

pub async fn handle_list_running() -> Json<serde_json::Value> {
    let instances_base = super::discovery::resolve_instances_base_dir();
    let running = super::discovery::scan_all_running_instances(&instances_base);
    Json(serde_json::json!({
        "running": running,
        "count": running.len(),
    }))
}

pub async fn handle_list_instance_dirs(
    State(state): State<Arc<ControllerState>>,
) -> Json<serde_json::Value> {
    let dirs = state.process_manager.scan_directories();
    Json(serde_json::json!({
        "directories": dirs,
        "count": dirs.len(),
    }))
}

pub async fn handle_health(State(state): State<Arc<ControllerState>>) -> Json<serde_json::Value> {
    let instances = state.registry.get_all_instances().await;
    let running = instances
        .iter()
        .filter(|i| i.status == InstanceStatus::Running)
        .count();
    Json(serde_json::json!({
        "status": "healthy",
        "mode": "controller",
        "port": state.controller_port,
        "instances_total": instances.len(),
        "instances_running": running,
    }))
}

pub async fn handle_api_health(
    State(state): State<Arc<ControllerState>>,
) -> Json<serde_json::Value> {
    let instances = state.registry.get_all_instances().await;
    let running_count = instances
        .iter()
        .filter(|i| i.status == InstanceStatus::Running)
        .count();
    let instance_id = instances.first().map(|i| i.id.clone()).unwrap_or_default();
    Json(serde_json::json!({
        "status": if running_count > 0 { "healthy" } else { "degraded" },
        "instance_id": instance_id,
        "instances_running": running_count,
        "instances_total": instances.len(),
    }))
}

pub async fn handle_api_info(State(state): State<Arc<ControllerState>>) -> Json<serde_json::Value> {
    let instances = state.registry.get_all_instances().await;
    let running: Vec<_> = instances
        .iter()
        .filter(|i| i.status == InstanceStatus::Running)
        .collect();

    let mut total_users: u32 = 0;
    let mut total_regions: u32 = 0;
    let mut total_cpu: f64 = 0.0;
    let mut total_memory: u64 = 0;
    let mut max_uptime: u64 = 0;

    for inst in &running {
        if let Some(m) = &inst.metrics {
            total_users += m.active_users;
            total_regions += m.active_regions;
            total_cpu += m.cpu_usage;
            total_memory += m.memory_usage_mb * 1024 * 1024;
            if m.uptime_seconds > max_uptime {
                max_uptime = m.uptime_seconds;
            }
        }
    }

    if max_uptime == 0 {
        let discovery_base = super::discovery::resolve_instances_base_dir();
        let disc = super::discovery::scan_all_running_instances(&discovery_base);
        if let Some(first) = disc.first() {
            if let Ok(started) = chrono::DateTime::parse_from_rfc3339(&first.started_at) {
                let elapsed = chrono::Utc::now().signed_duration_since(started);
                max_uptime = elapsed.num_seconds().max(0) as u64;
            }
        }
    }

    let disc_base = super::discovery::resolve_instances_base_dir();
    let disc_instances = super::discovery::scan_all_running_instances(&disc_base);
    let region_count = disc_instances
        .iter()
        .filter(|i| i.service_mode != "robust")
        .count();
    if total_regions == 0 {
        total_regions = region_count as u32;
    }

    Json(serde_json::json!({
        "active_connections": total_users,
        "active_regions": total_regions,
        "uptime": max_uptime,
        "cpu_usage": total_cpu,
        "memory_usage": total_memory,
        "instances_running": running.len(),
        "grid_name": disc_instances.first().map(|i| i.grid_name.as_str()).unwrap_or("OpenSim Next"),
        "version": disc_instances.first().map(|i| i.version.as_str()).unwrap_or("2.3.0"),
    }))
}
