use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use anyhow::{Result, anyhow};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub struct ZitiConfig {
    pub enabled: bool,
    pub identity_path: String,
    pub controller_url: String,
    pub tunnel_binary: String,
    pub keepalive_interval_secs: u64,
    pub restart_delay_secs: u64,
    pub max_restart_attempts: u32,
}

impl ZitiConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: std::env::var("OPENSIM_ZITI_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            identity_path: std::env::var("OPENSIM_ZITI_IDENTITY")
                .unwrap_or_default(),
            controller_url: std::env::var("OPENSIM_ZITI_CONTROLLER")
                .unwrap_or_default(),
            tunnel_binary: std::env::var("OPENSIM_ZITI_TUNNEL_BIN")
                .unwrap_or_else(|_| "ziti-edge-tunnel".to_string()),
            keepalive_interval_secs: std::env::var("OPENSIM_ZITI_KEEPALIVE")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(15),
            restart_delay_secs: 5,
            max_restart_attempts: 3,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ZitiStatus {
    pub enabled: bool,
    pub running: bool,
    pub identity_loaded: bool,
    pub controller_url: String,
    pub restart_count: u32,
    pub uptime_secs: u64,
}

pub struct ZitiManager {
    config: ZitiConfig,
    child: Arc<Mutex<Option<Child>>>,
    running: Arc<AtomicBool>,
    restart_count: Arc<std::sync::atomic::AtomicU32>,
    started_at: Arc<Mutex<Option<std::time::Instant>>>,
}

impl ZitiManager {
    pub fn new(config: ZitiConfig) -> Self {
        Self {
            config,
            child: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
            restart_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            started_at: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_env() -> Self {
        Self::new(ZitiConfig::from_env())
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("[ZITI] OpenZiti integration disabled");
            return Ok(());
        }

        if self.config.identity_path.is_empty() {
            return Err(anyhow!("OPENSIM_ZITI_IDENTITY not set"));
        }

        info!("[ZITI] Starting OpenZiti tunnel with identity: {}", self.config.identity_path);

        let child = Command::new(&self.config.tunnel_binary)
            .arg("run")
            .arg("-i")
            .arg(&self.config.identity_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start ziti-edge-tunnel: {}. Is it installed?", e))?;

        *self.child.lock().await = Some(child);
        self.running.store(true, Ordering::Relaxed);
        *self.started_at.lock().await = Some(std::time::Instant::now());

        info!("[ZITI] OpenZiti tunnel started successfully");

        let running = self.running.clone();
        let child_ref = self.child.clone();
        let restart_count = self.restart_count.clone();
        let max_restarts = self.config.max_restart_attempts;
        let restart_delay = self.config.restart_delay_secs;
        let tunnel_binary = self.config.tunnel_binary.clone();
        let identity_path = self.config.identity_path.clone();
        let started_at = self.started_at.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;

                if !running.load(Ordering::Relaxed) {
                    break;
                }

                let mut child_lock = child_ref.lock().await;
                if let Some(ref mut child) = *child_lock {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            warn!("[ZITI] Tunnel process exited with status: {}", status);
                            let count = restart_count.fetch_add(1, Ordering::Relaxed) + 1;

                            if count > max_restarts {
                                error!("[ZITI] Max restart attempts ({}) exceeded, giving up", max_restarts);
                                running.store(false, Ordering::Relaxed);
                                *child_lock = None;
                                break;
                            }

                            info!("[ZITI] Restarting tunnel (attempt {}/{})", count, max_restarts);
                            tokio::time::sleep(Duration::from_secs(restart_delay)).await;

                            match Command::new(&tunnel_binary)
                                .arg("run")
                                .arg("-i")
                                .arg(&identity_path)
                                .stdout(std::process::Stdio::piped())
                                .stderr(std::process::Stdio::piped())
                                .spawn()
                            {
                                Ok(new_child) => {
                                    *child_lock = Some(new_child);
                                    *started_at.lock().await = Some(std::time::Instant::now());
                                    info!("[ZITI] Tunnel restarted successfully");
                                }
                                Err(e) => {
                                    error!("[ZITI] Failed to restart tunnel: {}", e);
                                    running.store(false, Ordering::Relaxed);
                                    *child_lock = None;
                                    break;
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            warn!("[ZITI] Error checking tunnel status: {}", e);
                        }
                    }
                }
            }
        });

        let running_ka = self.running.clone();
        let keepalive_interval = self.config.keepalive_interval_secs;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(keepalive_interval));
            loop {
                interval.tick().await;
                if !running_ka.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);

        let mut child_lock = self.child.lock().await;
        if let Some(ref mut child) = *child_lock {
            info!("[ZITI] Stopping OpenZiti tunnel");
            child.kill().await.ok();
            child.wait().await.ok();
            *child_lock = None;
            info!("[ZITI] OpenZiti tunnel stopped");
        }

        Ok(())
    }

    pub async fn get_status(&self) -> ZitiStatus {
        let running = self.running.load(Ordering::Relaxed);
        let uptime = if let Some(started) = *self.started_at.lock().await {
            started.elapsed().as_secs()
        } else {
            0
        };

        ZitiStatus {
            enabled: self.config.enabled,
            running,
            identity_loaded: !self.config.identity_path.is_empty(),
            controller_url: self.config.controller_url.clone(),
            restart_count: self.restart_count.load(Ordering::Relaxed),
            uptime_secs: uptime,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Drop for ZitiManager {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
