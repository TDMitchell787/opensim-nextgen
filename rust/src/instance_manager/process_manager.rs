use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn, error};

use super::types::{ConsoleEntry, ConsoleOutputType};

const MAX_CONSOLE_BUFFER: usize = 2000;

#[derive(Debug, Clone)]
pub enum ProcessEvent {
    Spawned { id: String, pid: u32 },
    Exited { id: String, pid: u32, exit_code: Option<i32> },
    StdoutLine { id: String, line: String },
    StderrLine { id: String, line: String },
}

pub struct ManagedProcess {
    pub id: String,
    pub pid: u32,
    pub instance_dir: PathBuf,
    pub service_mode: String,
    pub started_at: DateTime<Utc>,
    pub console_buffer: VecDeque<ConsoleEntry>,
    kill_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredInstance {
    pub id: String,
    pub name: String,
    pub path: String,
    pub service_mode: String,
    pub login_port: u16,
    pub robust_port: u16,
    pub region_count: u32,
    pub database_url: String,
    pub hypergrid_enabled: bool,
}

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<String, ManagedProcess>>>,
    binary_path: PathBuf,
    instances_base_dir: PathBuf,
    event_tx: broadcast::Sender<ProcessEvent>,
}

impl ProcessManager {
    pub fn new(binary_path: PathBuf, instances_base_dir: PathBuf) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            binary_path,
            instances_base_dir,
            event_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ProcessEvent> {
        self.event_tx.subscribe()
    }

    pub fn scan_directories(&self) -> Vec<DiscoveredInstance> {
        let base = &self.instances_base_dir;
        let mut results = Vec::new();

        let entries = match std::fs::read_dir(base) {
            Ok(e) => e,
            Err(e) => {
                warn!("Cannot scan instances directory {}: {}", base.display(), e);
                return results;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if name == "template" || name.starts_with('.') {
                continue;
            }

            let env_file = path.join(".env");
            if !env_file.exists() {
                continue;
            }

            let env = Self::parse_env_file(&env_file);

            let region_count = Self::count_regions(&path);

            let id = name.to_lowercase().replace(' ', "-");
            results.push(DiscoveredInstance {
                id,
                name: name.clone(),
                path: path.to_string_lossy().to_string(),
                service_mode: env.get("OPENSIM_SERVICE_MODE").cloned().unwrap_or_else(|| "standalone".to_string()),
                login_port: env.get("OPENSIM_LOGIN_PORT").and_then(|v| v.parse().ok()).unwrap_or(9000),
                robust_port: env.get("OPENSIM_ROBUST_PORT").and_then(|v| v.parse().ok()).unwrap_or(8003),
                region_count,
                database_url: env.get("DATABASE_URL").cloned().unwrap_or_default(),
                hypergrid_enabled: env.get("OPENSIM_HYPERGRID_ENABLED").map(|v| v == "true").unwrap_or(false),
            });
        }

        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }

    pub async fn spawn_instance(&self, id: &str, instance_dir: &Path, controller_url: &str) -> anyhow::Result<u32> {
        {
            let procs = self.processes.read().await;
            if procs.contains_key(id) {
                anyhow::bail!("Instance {} is already running", id);
            }
        }

        let env_file = instance_dir.join(".env");
        let env = Self::parse_env_file(&env_file);

        let service_mode = env.get("OPENSIM_SERVICE_MODE").cloned().unwrap_or_else(|| "standalone".to_string());

        let binary = if self.binary_path.as_os_str().is_empty() {
            std::env::current_exe().unwrap_or_else(|_| PathBuf::from("opensim-next"))
        } else {
            self.binary_path.clone()
        };

        let mut cmd = Command::new(&binary);
        cmd.arg("start");

        cmd.env("OPENSIM_INSTANCE_DIR", instance_dir);
        cmd.env("OPENSIM_CONTROLLER_URL", controller_url);

        if let Ok(dyld) = std::env::var("DYLD_LIBRARY_PATH") {
            cmd.env("DYLD_LIBRARY_PATH", &dyld);
        }
        if let Ok(ld) = std::env::var("LD_LIBRARY_PATH") {
            cmd.env("LD_LIBRARY_PATH", &ld);
        }
        if let Ok(rust_log) = std::env::var("RUST_LOG") {
            cmd.env("RUST_LOG", &rust_log);
        }

        for (key, value) in &env {
            cmd.env(key, value);
        }

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        info!("Spawning instance {} from {} (mode={})", id, instance_dir.display(), service_mode);

        let mut child = cmd.spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn instance {}: {}", id, e))?;

        let pid = child.id().unwrap_or(0);
        info!("Instance {} spawned with PID {}", id, pid);

        let (kill_tx, kill_rx) = tokio::sync::oneshot::channel();

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let managed = ManagedProcess {
            id: id.to_string(),
            pid,
            instance_dir: instance_dir.to_path_buf(),
            service_mode: service_mode.clone(),
            started_at: Utc::now(),
            console_buffer: VecDeque::with_capacity(MAX_CONSOLE_BUFFER),
            kill_tx: Some(kill_tx),
        };

        {
            let mut procs = self.processes.write().await;
            procs.insert(id.to_string(), managed);
        }

        let _ = self.event_tx.send(ProcessEvent::Spawned { id: id.to_string(), pid });

        if let Some(stdout) = stdout {
            let id_clone = id.to_string();
            let event_tx = self.event_tx.clone();
            let procs = self.processes.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = event_tx.send(ProcessEvent::StdoutLine { id: id_clone.clone(), line: line.clone() });
                    let mut procs = procs.write().await;
                    if let Some(proc) = procs.get_mut(&id_clone) {
                        if proc.console_buffer.len() >= MAX_CONSOLE_BUFFER {
                            proc.console_buffer.pop_front();
                        }
                        proc.console_buffer.push_back(ConsoleEntry {
                            instance_id: id_clone.clone(),
                            content: line,
                            output_type: ConsoleOutputType::Stdout,
                            timestamp: Utc::now(),
                        });
                    }
                }
            });
        }

        if let Some(stderr) = stderr {
            let id_clone = id.to_string();
            let event_tx = self.event_tx.clone();
            let procs = self.processes.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = event_tx.send(ProcessEvent::StderrLine { id: id_clone.clone(), line: line.clone() });
                    let mut procs = procs.write().await;
                    if let Some(proc) = procs.get_mut(&id_clone) {
                        if proc.console_buffer.len() >= MAX_CONSOLE_BUFFER {
                            proc.console_buffer.pop_front();
                        }
                        proc.console_buffer.push_back(ConsoleEntry {
                            instance_id: id_clone.clone(),
                            content: line,
                            output_type: ConsoleOutputType::Stderr,
                            timestamp: Utc::now(),
                        });
                    }
                }
            });
        }

        let id_clone = id.to_string();
        let event_tx = self.event_tx.clone();
        let procs = self.processes.clone();
        tokio::spawn(async move {
            tokio::select! {
                status = child.wait() => {
                    let exit_code = status.ok().and_then(|s| s.code());
                    info!("Instance {} (PID {}) exited with code {:?}", id_clone, pid, exit_code);
                    let _ = event_tx.send(ProcessEvent::Exited { id: id_clone.clone(), pid, exit_code });
                    let mut procs = procs.write().await;
                    procs.remove(&id_clone);
                }
                _ = kill_rx => {
                    info!("Sending SIGTERM to instance {} (PID {})", id_clone, pid);
                    let _ = child.kill().await;
                    let _ = event_tx.send(ProcessEvent::Exited { id: id_clone.clone(), pid, exit_code: None });
                    let mut procs = procs.write().await;
                    procs.remove(&id_clone);
                }
            }
        });

        Ok(pid)
    }

    pub async fn stop_instance(&self, id: &str, graceful: bool) -> anyhow::Result<()> {
        let mut procs = self.processes.write().await;
        let proc = procs.get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Instance {} is not running", id))?;

        info!("Stopping instance {} (PID {}, graceful={})", id, proc.pid, graceful);

        if let Some(kill_tx) = proc.kill_tx.take() {
            let _ = kill_tx.send(());
        }

        if graceful {
            drop(procs);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            let procs = self.processes.read().await;
            if procs.contains_key(id) {
                warn!("Instance {} did not exit gracefully, force killing", id);
            }
        }

        Ok(())
    }

    pub async fn restart_instance(&self, id: &str, instance_dir: &Path, controller_url: &str) -> anyhow::Result<u32> {
        if self.is_running(id).await {
            self.stop_instance(id, true).await?;
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        self.spawn_instance(id, instance_dir, controller_url).await
    }

    pub async fn is_running(&self, id: &str) -> bool {
        let procs = self.processes.read().await;
        procs.contains_key(id)
    }

    pub async fn get_console(&self, id: &str, limit: usize) -> Vec<ConsoleEntry> {
        let procs = self.processes.read().await;
        match procs.get(id) {
            Some(proc) => {
                let len = proc.console_buffer.len();
                let skip = if len > limit { len - limit } else { 0 };
                proc.console_buffer.iter().skip(skip).cloned().collect()
            }
            None => Vec::new(),
        }
    }

    pub async fn list_running(&self) -> Vec<(String, u32, DateTime<Utc>)> {
        let procs = self.processes.read().await;
        procs.values().map(|p| (p.id.clone(), p.pid, p.started_at)).collect()
    }

    fn parse_env_file(path: &Path) -> HashMap<String, String> {
        let mut env = HashMap::new();
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return env,
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(idx) = trimmed.find('=') {
                let key = trimmed[..idx].trim().to_string();
                let value = trimmed[idx + 1..].trim().to_string();
                env.insert(key, value);
            }
        }
        env
    }

    fn count_regions(instance_dir: &Path) -> u32 {
        let regions_ini = instance_dir.join("Regions").join("Regions.ini");
        if !regions_ini.exists() {
            return 0;
        }
        match std::fs::read_to_string(&regions_ini) {
            Ok(content) => content.lines()
                .filter(|l| l.trim().starts_with('['))
                .count() as u32,
            Err(_) => 0,
        }
    }
}
