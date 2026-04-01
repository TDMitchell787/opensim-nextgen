use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningInstanceInfo {
    pub instance_id: String,
    pub pid: u32,
    pub controller_port: u16,
    pub login_port: u16,
    pub robust_port: u16,
    pub service_mode: String,
    pub grid_name: String,
    pub started_at: String,
    pub version: String,
    pub host: String,
}

impl RunningInstanceInfo {
    pub fn new(
        instance_id: String,
        service_mode: String,
        controller_port: u16,
        login_port: u16,
        robust_port: u16,
    ) -> Self {
        Self {
            instance_id,
            pid: std::process::id(),
            controller_port,
            login_port,
            robust_port,
            service_mode,
            grid_name: std::env::var("OPENSIM_GRID_NAME").unwrap_or_default(),
            started_at: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            host: "localhost".to_string(),
        }
    }
}

fn discovery_filename(mode: &str) -> &str {
    if mode == "robust" {
        ".running-robust.json"
    } else {
        ".running.json"
    }
}

pub fn write_discovery_file(instance_dir: &Path, mode: &str, info: &RunningInstanceInfo) -> std::io::Result<()> {
    let filename = discovery_filename(mode);
    let target = instance_dir.join(filename);
    let tmp = instance_dir.join(format!(".running-{}.tmp", std::process::id()));

    let json = serde_json::to_string_pretty(info)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, &target)?;

    info!("Discovery file written: {}", target.display());
    Ok(())
}

pub fn remove_discovery_file(instance_dir: &Path, mode: &str) {
    let filename = discovery_filename(mode);
    let target = instance_dir.join(filename);
    if target.exists() {
        if let Err(e) = std::fs::remove_file(&target) {
            warn!("Failed to remove discovery file {}: {}", target.display(), e);
        } else {
            info!("Discovery file removed: {}", target.display());
        }
    }
}

pub fn is_pid_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

pub fn cleanup_stale_discovery_file(path: &Path) -> bool {
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let info: RunningInstanceInfo = match serde_json::from_str(&data) {
        Ok(i) => i,
        Err(_) => {
            warn!("Invalid discovery file {}, removing", path.display());
            let _ = std::fs::remove_file(path);
            return true;
        }
    };

    if !is_pid_alive(info.pid) {
        info!("Cleaning stale discovery file {} (PID {} dead)", path.display(), info.pid);
        let _ = std::fs::remove_file(path);
        return true;
    }
    false
}

pub fn scan_all_running_instances(instances_base_dir: &Path) -> Vec<RunningInstanceInfo> {
    let mut results = Vec::new();

    let entries = match std::fs::read_dir(instances_base_dir) {
        Ok(e) => e,
        Err(e) => {
            debug!("Cannot read instances dir {}: {}", instances_base_dir.display(), e);
            return results;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        for filename in &[".running.json", ".running-robust.json"] {
            let discovery_path = path.join(filename);
            if !discovery_path.exists() {
                continue;
            }

            if cleanup_stale_discovery_file(&discovery_path) {
                continue;
            }

            match std::fs::read_to_string(&discovery_path) {
                Ok(data) => {
                    if let Ok(info) = serde_json::from_str::<RunningInstanceInfo>(&data) {
                        results.push(info);
                    }
                }
                Err(e) => {
                    warn!("Failed to read {}: {}", discovery_path.display(), e);
                }
            }
        }
    }

    results
}

pub fn find_available_controller_port(instances_base_dir: &Path, range_start: u16, range_end: u16) -> u16 {
    let running = scan_all_running_instances(instances_base_dir);
    let claimed: std::collections::HashSet<u16> = running
        .iter()
        .filter(|i| i.controller_port > 0)
        .map(|i| i.controller_port)
        .collect();

    for port in range_start..=range_end {
        if !claimed.contains(&port) {
            if port_is_available(port) {
                return port;
            }
        }
    }

    range_start
}

fn port_is_available(port: u16) -> bool {
    std::net::TcpListener::bind(("0.0.0.0", port)).is_ok()
}

pub fn resolve_instance_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OPENSIM_INSTANCE_DIR") {
        PathBuf::from(dir)
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

pub fn resolve_instances_base_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OPENSIM_INSTANCE_DIR") {
        let p = PathBuf::from(&dir);
        p.parent().unwrap_or(&p).to_path_buf()
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        cwd.join("Instances")
    }
}
