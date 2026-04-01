//! Instance Controller
//!
//! Executes commands on instances and manages their lifecycle.
//! Handles start, stop, restart, and other control operations.

use anyhow::{anyhow, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use super::registry::InstanceRegistry;
use super::types::{
    BatchResult, BatchStatus, CommandResult, ConsoleEntry, ConsoleOutputType,
    InstanceCommand, InstanceStatus, RegionControlAction, UserManagementAction,
};

/// Controller for executing commands on instances
pub struct InstanceController {
    registry: Arc<InstanceRegistry>,
    http_client: Client,
    command_timeout: Duration,
    event_tx: broadcast::Sender<ControllerEvent>,
}

/// Events emitted by the controller
#[derive(Debug, Clone)]
pub enum ControllerEvent {
    CommandStarted {
        instance_id: String,
        command: String,
    },
    CommandCompleted {
        instance_id: String,
        command: String,
        success: bool,
        duration_ms: u64,
    },
    ConsoleOutput(ConsoleEntry),
    StatusChanged {
        instance_id: String,
        old_status: InstanceStatus,
        new_status: InstanceStatus,
    },
}

impl InstanceController {
    /// Create a new instance controller
    pub fn new(registry: Arc<InstanceRegistry>) -> Self {
        let command_timeout = Duration::from_millis(
            registry.controller_config().command_timeout_ms
        );

        let http_client = Client::builder()
            .timeout(command_timeout)
            .build()
            .expect("Failed to create HTTP client");

        let (event_tx, _) = broadcast::channel(1000);

        Self {
            registry,
            http_client,
            command_timeout,
            event_tx,
        }
    }

    /// Subscribe to controller events
    pub fn subscribe(&self) -> broadcast::Receiver<ControllerEvent> {
        self.event_tx.subscribe()
    }

    /// Execute a command on a specific instance
    pub async fn execute_command(
        &self,
        instance_id: &str,
        command: InstanceCommand,
    ) -> Result<CommandResult> {
        let config = self.registry
            .get_instance_config(instance_id)
            .await
            .ok_or_else(|| anyhow!("Instance not found: {}", instance_id))?;

        let command_name = format!("{:?}", command);
        let start = Instant::now();

        let _ = self.event_tx.send(ControllerEvent::CommandStarted {
            instance_id: instance_id.to_string(),
            command: command_name.clone(),
        });

        info!("Executing command {:?} on instance {}", command, instance_id);

        let result = match command {
            InstanceCommand::GetStatus => self.get_status(&config).await,
            InstanceCommand::GetMetrics => self.get_metrics(&config).await,
            InstanceCommand::Start => self.start_instance(&config).await,
            InstanceCommand::Stop => self.stop_instance(&config, false).await,
            InstanceCommand::Shutdown => self.stop_instance(&config, true).await,
            InstanceCommand::ForceShutdown => self.force_shutdown(&config).await,
            InstanceCommand::Restart => self.restart_instance(&config).await,
            InstanceCommand::Reload => self.reload_config(&config).await,
            InstanceCommand::Backup => self.backup_instance(&config).await,
            InstanceCommand::GetLogs => self.get_logs(&config).await,
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        let mut result = result.unwrap_or_else(|e| CommandResult::failure(e.to_string()));
        result.duration_ms = duration_ms;

        let _ = self.event_tx.send(ControllerEvent::CommandCompleted {
            instance_id: instance_id.to_string(),
            command: command_name,
            success: result.success,
            duration_ms,
        });

        Ok(result)
    }

    /// Execute a command on multiple instances
    pub async fn broadcast_command(
        &self,
        command: InstanceCommand,
        instance_ids: Option<Vec<String>>,
    ) -> Result<HashMap<String, BatchResult>> {
        let target_ids = match instance_ids {
            Some(ids) => ids,
            None => self.registry.get_instance_ids().await,
        };

        let mut results = HashMap::new();

        for instance_id in target_ids {
            let start = Instant::now();

            let result = self.execute_command(&instance_id, command.clone()).await;
            let duration_ms = start.elapsed().as_millis() as u64;

            let batch_result = match result {
                Ok(cmd_result) => BatchResult {
                    instance_id: instance_id.clone(),
                    status: if cmd_result.success {
                        BatchStatus::Success
                    } else {
                        BatchStatus::Failed
                    },
                    message: cmd_result.message,
                    data: cmd_result.data,
                    duration_ms,
                },
                Err(e) => BatchResult {
                    instance_id: instance_id.clone(),
                    status: BatchStatus::Failed,
                    message: e.to_string(),
                    data: None,
                    duration_ms,
                },
            };

            results.insert(instance_id, batch_result);
        }

        Ok(results)
    }

    /// Execute a console command on an instance
    pub async fn execute_console_command(
        &self,
        instance_id: &str,
        command: &str,
    ) -> Result<CommandResult> {
        let config = self.registry
            .get_instance_config(instance_id)
            .await
            .ok_or_else(|| anyhow!("Instance not found: {}", instance_id))?;

        info!("Executing console command on {}: {}", instance_id, command);

        let url = format!("{}/api/system/command", config.admin_url());

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.get_api_key()))
            .json(&serde_json::json!({ "command": command }))
            .send()
            .await
            .map_err(|e| anyhow!("Console command request failed: {}", e))?;

        if !response.status().is_success() {
            return Ok(CommandResult::failure(format!(
                "Console command failed with status: {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse console response: {}", e))?;

        let output = body.get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let _ = self.event_tx.send(ControllerEvent::ConsoleOutput(ConsoleEntry {
            instance_id: instance_id.to_string(),
            content: output.to_string(),
            output_type: ConsoleOutputType::Stdout,
            timestamp: chrono::Utc::now(),
        }));

        Ok(CommandResult::success_with_data(
            "Console command executed",
            serde_json::json!({ "output": output }),
        ))
    }

    /// Execute a user management action
    pub async fn execute_user_action(
        &self,
        instance_id: &str,
        action: UserManagementAction,
    ) -> Result<CommandResult> {
        let config = self.registry
            .get_instance_config(instance_id)
            .await
            .ok_or_else(|| anyhow!("Instance not found: {}", instance_id))?;

        let (method, endpoint, body) = match &action {
            UserManagementAction::List { limit, offset } => {
                let url = format!(
                    "/admin/users?limit={}&offset={}",
                    limit.unwrap_or(100),
                    offset.unwrap_or(0)
                );
                ("GET", url, None)
            }
            UserManagementAction::Get { user_id } => {
                ("GET", format!("/admin/users/{}", user_id), None)
            }
            UserManagementAction::Create { user } => {
                ("POST", "/admin/users".to_string(), Some(user.clone()))
            }
            UserManagementAction::Update { user_id, updates } => {
                ("PUT", format!("/admin/users/{}", user_id), Some(updates.clone()))
            }
            UserManagementAction::Delete { user_id } => {
                ("DELETE", format!("/admin/users/{}", user_id), None)
            }
            UserManagementAction::ResetPassword { user_id, new_password } => {
                ("PUT", format!("/admin/users/{}/password", user_id),
                 Some(serde_json::json!({ "password": new_password })))
            }
            UserManagementAction::SetLevel { user_id, level } => {
                ("PUT", format!("/admin/users/{}/level", user_id),
                 Some(serde_json::json!({ "level": level })))
            }
            UserManagementAction::Kick { user_id, reason } => {
                ("POST", format!("/admin/users/{}/kick", user_id),
                 Some(serde_json::json!({ "reason": reason })))
            }
            UserManagementAction::Ban { user_id, reason, duration_hours } => {
                ("POST", format!("/admin/users/{}/ban", user_id),
                 Some(serde_json::json!({ "reason": reason, "duration_hours": duration_hours })))
            }
        };

        let url = format!("{}{}", config.admin_url(), endpoint);

        let mut request = match method {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            _ => return Err(anyhow!("Unknown HTTP method: {}", method)),
        };

        request = request.header("Authorization", format!("Bearer {}", config.get_api_key()));

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await
            .map_err(|e| anyhow!("User management request failed: {}", e))?;

        if !response.status().is_success() {
            return Ok(CommandResult::failure(format!(
                "User management action failed with status: {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);

        Ok(CommandResult::success_with_data("User action completed", data))
    }

    /// Execute a region control action
    pub async fn execute_region_action(
        &self,
        instance_id: &str,
        action: RegionControlAction,
    ) -> Result<CommandResult> {
        let config = self.registry
            .get_instance_config(instance_id)
            .await
            .ok_or_else(|| anyhow!("Instance not found: {}", instance_id))?;

        let (method, endpoint, body) = match &action {
            RegionControlAction::List => {
                ("GET", "/admin/regions".to_string(), None)
            }
            RegionControlAction::Get { region_id } => {
                ("GET", format!("/admin/regions/{}", region_id), None)
            }
            RegionControlAction::Start { region_id } => {
                ("POST", format!("/admin/regions/{}/start", region_id), None)
            }
            RegionControlAction::Stop { region_id } => {
                ("POST", format!("/admin/regions/{}/stop", region_id), None)
            }
            RegionControlAction::Restart { region_id } => {
                ("POST", format!("/admin/regions/{}/restart", region_id), None)
            }
            RegionControlAction::Backup { region_id } => {
                ("POST", format!("/admin/regions/{}/backup", region_id), None)
            }
            RegionControlAction::LoadOar { region_id, oar_path } => {
                ("POST", format!("/admin/regions/{}/load-oar", region_id),
                 Some(serde_json::json!({ "oar_path": oar_path })))
            }
            RegionControlAction::SaveOar { region_id, oar_path } => {
                ("POST", format!("/admin/regions/{}/save-oar", region_id),
                 Some(serde_json::json!({ "oar_path": oar_path })))
            }
            RegionControlAction::TeleportAll { region_id, target_region } => {
                ("POST", format!("/admin/regions/{}/teleport-all", region_id),
                 Some(serde_json::json!({ "target_region": target_region })))
            }
        };

        let url = format!("{}{}", config.admin_url(), endpoint);

        let mut request = match method {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            _ => return Err(anyhow!("Unknown HTTP method: {}", method)),
        };

        request = request.header("Authorization", format!("Bearer {}", config.get_api_key()));

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await
            .map_err(|e| anyhow!("Region control request failed: {}", e))?;

        if !response.status().is_success() {
            return Ok(CommandResult::failure(format!(
                "Region control action failed with status: {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);

        Ok(CommandResult::success_with_data("Region action completed", data))
    }

    // Private helper methods

    async fn get_status(&self, config: &super::config_loader::InstanceConfig) -> Result<CommandResult> {
        let url = format!("{}/api/status", config.admin_url());

        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.get_api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(CommandResult::failure(format!("Status check failed: {}", response.status())));
        }

        let data: serde_json::Value = response.json().await?;
        Ok(CommandResult::success_with_data("Status retrieved", data))
    }

    async fn get_metrics(&self, config: &super::config_loader::InstanceConfig) -> Result<CommandResult> {
        let url = format!("{}/metrics", config.metrics_url());

        let response = self.http_client
            .get(&url)
            .header("api_key", config.get_api_key())
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(CommandResult::failure(format!("Metrics fetch failed: {}", response.status())));
        }

        let text = response.text().await?;
        Ok(CommandResult::success_with_data("Metrics retrieved", serde_json::json!({ "metrics": text })))
    }

    async fn start_instance(&self, config: &super::config_loader::InstanceConfig) -> Result<CommandResult> {
        warn!("Start command requires external process management - not directly supported");
        Ok(CommandResult::failure("Start command requires external process manager (systemd, docker, etc.)"))
    }

    async fn stop_instance(&self, config: &super::config_loader::InstanceConfig, graceful: bool) -> Result<CommandResult> {
        let url = format!("{}/api/system/shutdown", config.admin_url());

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.get_api_key()))
            .json(&serde_json::json!({ "graceful": graceful }))
            .send()
            .await?;

        if response.status().is_success() {
            self.registry.update_status(&config.id, InstanceStatus::Stopping).await?;
            Ok(CommandResult::success(if graceful { "Graceful shutdown initiated" } else { "Shutdown initiated" }))
        } else {
            Ok(CommandResult::failure(format!("Shutdown failed: {}", response.status())))
        }
    }

    async fn force_shutdown(&self, config: &super::config_loader::InstanceConfig) -> Result<CommandResult> {
        let url = format!("{}/api/system/shutdown", config.admin_url());

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.get_api_key()))
            .json(&serde_json::json!({ "graceful": false, "force": true }))
            .send()
            .await?;

        if response.status().is_success() {
            self.registry.update_status(&config.id, InstanceStatus::Stopping).await?;
            Ok(CommandResult::success("Force shutdown initiated"))
        } else {
            Ok(CommandResult::failure(format!("Force shutdown failed: {}", response.status())))
        }
    }

    async fn restart_instance(&self, config: &super::config_loader::InstanceConfig) -> Result<CommandResult> {
        let url = format!("{}/api/system/restart", config.admin_url());

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.get_api_key()))
            .send()
            .await?;

        if response.status().is_success() {
            self.registry.update_status(&config.id, InstanceStatus::Starting).await?;
            Ok(CommandResult::success("Restart initiated"))
        } else {
            Ok(CommandResult::failure(format!("Restart failed: {}", response.status())))
        }
    }

    async fn reload_config(&self, config: &super::config_loader::InstanceConfig) -> Result<CommandResult> {
        let url = format!("{}/api/config/reload", config.admin_url());

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.get_api_key()))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(CommandResult::success("Configuration reloaded"))
        } else {
            Ok(CommandResult::failure(format!("Config reload failed: {}", response.status())))
        }
    }

    async fn backup_instance(&self, config: &super::config_loader::InstanceConfig) -> Result<CommandResult> {
        let url = format!("{}/api/system/backup", config.admin_url());

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.get_api_key()))
            .send()
            .await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            Ok(CommandResult::success_with_data("Backup created", data))
        } else {
            Ok(CommandResult::failure(format!("Backup failed: {}", response.status())))
        }
    }

    async fn get_logs(&self, config: &super::config_loader::InstanceConfig) -> Result<CommandResult> {
        let url = format!("{}/api/logs?limit=100", config.admin_url());

        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.get_api_key()))
            .send()
            .await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            Ok(CommandResult::success_with_data("Logs retrieved", data))
        } else {
            Ok(CommandResult::failure(format!("Get logs failed: {}", response.status())))
        }
    }
}
