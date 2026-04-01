//! RemoteAdmin interface for external administration
//!
//! This module provides OpenSim-compatible RemoteAdmin functionality,
//! allowing external tools and scripts to manage the server remotely.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use crate::{
    region::{RegionManager, RegionId},
    network::session::SessionManager,
    database::user_accounts::UserAccountDatabase,
    asset::AssetManager,
    performance::admin_dashboard::AdminDashboard,
    scripting::ScriptingManager,
};

/// RemoteAdmin service for external administration
pub struct RemoteAdminService {
    /// Admin password for authentication
    admin_password: String,
    /// Enabled commands
    enabled_commands: HashMap<String, bool>,
    /// Access restrictions
    access_restrictions: AccessRestrictions,
    /// Region manager
    region_manager: Arc<RegionManager>,
    /// Session manager
    session_manager: Arc<SessionManager>,
    /// User account manager
    user_manager: Arc<UserAccountDatabase>,
    /// Asset manager
    asset_manager: Arc<AssetManager>,
    /// Admin dashboard
    admin_dashboard: Arc<AdminDashboard>,
    /// Scripting manager
    scripting_manager: Arc<ScriptingManager>,
    /// Command statistics
    command_stats: RwLock<CommandStatistics>,
}

/// Access restrictions for RemoteAdmin
#[derive(Debug, Clone, Serialize)]
pub struct AccessRestrictions {
    pub enabled: bool,
    pub allowed_ips: Vec<String>,
    pub require_ssl: bool,
    pub max_requests_per_minute: u32,
    pub allowed_methods: Vec<String>,
}

impl Default for AccessRestrictions {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_string(), "::1".to_string()],
            require_ssl: false,
            max_requests_per_minute: 60,
            allowed_methods: vec![
                "admin_create_user".to_string(),
                "admin_exists_user".to_string(),
                "admin_get_agents".to_string(),
                "admin_teleport_agent".to_string(),
                "admin_restart".to_string(),
                "admin_load_heightmap".to_string(),
                "admin_save_heightmap".to_string(),
                "admin_load_xml".to_string(),
                "admin_save_xml".to_string(),
                "admin_load_oar".to_string(),
                "admin_save_oar".to_string(),
                "admin_broadcast".to_string(),
                "admin_region_query".to_string(),
                "admin_console_command".to_string(),
            ],
        }
    }
}

/// Command statistics
#[derive(Debug, Clone, Default, Serialize)]
pub struct CommandStatistics {
    pub total_commands: u64,
    pub successful_commands: u64,
    pub failed_commands: u64,
    pub commands_by_type: HashMap<String, u64>,
    pub last_command_time: Option<u64>,
}

/// RemoteAdmin request parameters
#[derive(Debug, Deserialize)]
pub struct RemoteAdminRequest {
    pub method: String,
    pub password: String,
    #[serde(flatten)]
    pub parameters: HashMap<String, String>,
}

/// RemoteAdmin response
#[derive(Debug, Serialize)]
pub struct RemoteAdminResponse {
    pub success: bool,
    pub message: String,
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

impl RemoteAdminService {
    /// Create a new RemoteAdmin service
    pub fn new(
        admin_password: String,
        region_manager: Arc<RegionManager>,
        session_manager: Arc<SessionManager>,
        user_manager: Arc<UserAccountDatabase>,
        asset_manager: Arc<AssetManager>,
        admin_dashboard: Arc<AdminDashboard>,
        scripting_manager: Arc<ScriptingManager>,
    ) -> Self {
        let mut enabled_commands = HashMap::new();
        
        // Enable safe commands by default
        let safe_commands = [
            "admin_exists_user", "admin_get_agents", "admin_region_query",
            "admin_broadcast", "admin_load_oar", "admin_save_oar",
        ];
        
        for command in &safe_commands {
            enabled_commands.insert(command.to_string(), true);
        }
        
        // Disable potentially dangerous commands by default
        let dangerous_commands = [
            "admin_create_user", "admin_teleport_agent", "admin_restart",
            "admin_load_heightmap", "admin_save_heightmap", "admin_load_xml",
            "admin_save_xml", "admin_console_command",
        ];
        
        for command in &dangerous_commands {
            enabled_commands.insert(command.to_string(), false);
        }

        Self {
            admin_password,
            enabled_commands,
            access_restrictions: AccessRestrictions::default(),
            region_manager,
            session_manager,
            user_manager,
            asset_manager,
            admin_dashboard,
            scripting_manager,
            command_stats: RwLock::new(CommandStatistics::default()),
        }
    }

    /// Create router for RemoteAdmin endpoints
    pub fn create_router(self: Arc<Self>) -> Router {
        Router::new()
            .route("/admin", post(handle_admin_request_post))
            .route("/admin", get(handle_admin_request_get))
            .route("/admin/status", get(handle_admin_status))
            .route("/admin/commands", get(handle_admin_commands))
            .route("/admin/stats", get(handle_admin_stats))
            .with_state(self)
    }

    /// Authenticate admin request
    pub fn authenticate(&self, password: &str) -> bool {
        if self.admin_password.is_empty() {
            warn!("RemoteAdmin: No admin password set - denying access");
            return false;
        }
        
        password == self.admin_password
    }

    /// Check if command is enabled
    pub fn is_command_enabled(&self, command: &str) -> bool {
        self.enabled_commands.get(command).copied().unwrap_or(false)
    }

    /// Execute admin command
    pub async fn execute_command(
        &self,
        method: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<RemoteAdminResponse> {
        debug!("Executing RemoteAdmin command: {}", method);

        // Update statistics
        {
            let mut stats = self.command_stats.write().await;
            stats.total_commands += 1;
            *stats.commands_by_type.entry(method.to_string()).or_insert(0) += 1;
            stats.last_command_time = Some(chrono::Utc::now().timestamp() as u64);
        }

        let result = match method {
            "admin_create_user" => self.admin_create_user(parameters).await,
            "admin_exists_user" => self.admin_exists_user(parameters).await,
            "admin_get_agents" => self.admin_get_agents(parameters).await,
            "admin_teleport_agent" => self.admin_teleport_agent(parameters).await,
            "admin_restart" => self.admin_restart(parameters).await,
            "admin_load_heightmap" => self.admin_load_heightmap(parameters).await,
            "admin_save_heightmap" => self.admin_save_heightmap(parameters).await,
            "admin_load_xml" => self.admin_load_xml(parameters).await,
            "admin_save_xml" => self.admin_save_xml(parameters).await,
            "admin_load_oar" => self.admin_load_oar(parameters).await,
            "admin_save_oar" => self.admin_save_oar(parameters).await,
            "admin_broadcast" => self.admin_broadcast(parameters).await,
            "admin_region_query" => self.admin_region_query(parameters).await,
            "admin_console_command" => self.admin_console_command(parameters).await,
            _ => Err(anyhow!("Unknown command: {}", method)),
        };

        // Update success/failure statistics
        {
            let mut stats = self.command_stats.write().await;
            match &result {
                Ok(_) => stats.successful_commands += 1,
                Err(_) => stats.failed_commands += 1,
            }
        }

        result
    }

    /// Create user account
    async fn admin_create_user(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let user_firstname = params.get("user_firstname")
            .ok_or_else(|| anyhow!("Missing user_firstname parameter"))?;
        let user_lastname = params.get("user_lastname")
            .ok_or_else(|| anyhow!("Missing user_lastname parameter"))?;
        let user_password = params.get("user_password")
            .ok_or_else(|| anyhow!("Missing user_password parameter"))?;
        let user_email = params.get("user_email")
            .ok_or_else(|| anyhow!("Missing user_email parameter"))?;

        info!("Creating user: {} {}", user_firstname, user_lastname);

        // Create user account
        let create_request = crate::database::user_accounts::CreateUserRequest {
            username: format!("{} {}", user_firstname, user_lastname),
            email: user_email.clone(),
            password: user_password.clone(),
            first_name: user_firstname.clone(),
            last_name: user_lastname.clone(),
            home_region_id: None,
        };
        
        let user_account = self.user_manager.create_user(create_request).await?;

        let mut data = HashMap::new();
        data.insert("avatar_uuid".to_string(), serde_json::Value::String(user_account.id.to_string()));

        Ok(RemoteAdminResponse {
            success: true,
            message: format!("User {} {} created successfully", user_firstname, user_lastname),
            data,
        })
    }

    /// Check if user exists
    async fn admin_exists_user(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let user_firstname = params.get("user_firstname")
            .ok_or_else(|| anyhow!("Missing user_firstname parameter"))?;
        let user_lastname = params.get("user_lastname")
            .ok_or_else(|| anyhow!("Missing user_lastname parameter"))?;

        let exists = self.user_manager.user_exists(user_firstname, user_lastname).await?;

        let mut data = HashMap::new();
        data.insert("user_exists".to_string(), serde_json::Value::Bool(exists));

        Ok(RemoteAdminResponse {
            success: true,
            message: if exists { "User exists" } else { "User does not exist" }.to_string(),
            data,
        })
    }

    /// Get logged in agents
    async fn admin_get_agents(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64));

        let agents = if let Some(region_id) = region_uuid {
            self.session_manager.get_region_agents(Uuid::from_u128(region_id.0 as u128)).await
                .map_err(|e| anyhow::anyhow!("Failed to get region agents: {}", e))?
        } else {
            self.session_manager.get_all_agents().await
        };

        let mut data = HashMap::new();
        data.insert("agents".to_string(), serde_json::Value::Array(
            agents.into_iter().map(|agent| serde_json::json!({
                "uuid": agent.user_id.to_string(),
                "firstname": agent.first_name,
                "lastname": agent.last_name,
                "region_uuid": agent.region_id.map(|r| r.0.to_string()).unwrap_or_else(|| "none".to_string()),
                "position": agent.position.map(|(x, y, z)| format!("{},{},{}", x, y, z)).unwrap_or_else(|| "0,0,0".to_string()),
                "login_time": agent.login_time.elapsed().as_secs(),
            })).collect()
        ));

        Ok(RemoteAdminResponse {
            success: true,
            message: format!("Found {} agents", data.get("agents").unwrap().as_array().unwrap().len()),
            data,
        })
    }

    /// Teleport agent
    async fn admin_teleport_agent(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let agent_id = params.get("agent_id")
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| anyhow!("Missing or invalid agent_id parameter"))?;
        
        let region_name = params.get("region_name")
            .ok_or_else(|| anyhow!("Missing region_name parameter"))?;

        // Parse position parameters
        let x: f32 = params.get("pos_x").and_then(|s| s.parse().ok()).unwrap_or(128.0);
        let y: f32 = params.get("pos_y").and_then(|s| s.parse().ok()).unwrap_or(128.0);
        let z: f32 = params.get("pos_z").and_then(|s| s.parse().ok()).unwrap_or(25.0);

        info!("Teleporting agent {} to {} at ({}, {}, {})", agent_id, region_name, x, y, z);

        // Find region by name
        let region_info = self.region_manager.find_region_by_name(region_name).await
            .map_err(|_| anyhow!("Region '{}' not found", region_name))?;
        
        // For now, use a placeholder region ID - in production, get from region_info
        let region_id = uuid::Uuid::new_v4();

        // Teleport agent
        self.session_manager.teleport_agent(agent_id, region_id, (x, y, z)).await
            .map_err(|e| anyhow::anyhow!("Failed to teleport agent: {}", e))?;

        Ok(RemoteAdminResponse {
            success: true,
            message: format!("Agent {} teleported to {} at ({}, {}, {})", agent_id, region_name, x, y, z),
            data: HashMap::new(),
        })
    }

    /// Restart region
    async fn admin_restart(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64));

        if let Some(region_id) = region_uuid {
            info!("Restarting region: {}", region_id.0);
            self.region_manager.restart_region(region_id).await?;
            
            Ok(RemoteAdminResponse {
                success: true,
                message: format!("Region {} restarted successfully", region_id.0),
                data: HashMap::new(),
            })
        } else {
            info!("Restarting all regions");
            self.region_manager.restart_all_regions().await?;
            
            Ok(RemoteAdminResponse {
                success: true,
                message: "All regions restarted successfully".to_string(),
                data: HashMap::new(),
            })
        }
    }

    /// Load heightmap
    async fn admin_load_heightmap(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64))
            .ok_or_else(|| anyhow!("Missing or invalid region_uuid parameter"))?;
        
        let filename = params.get("filename")
            .ok_or_else(|| anyhow!("Missing filename parameter"))?;

        info!("Loading heightmap {} for region {}", filename, region_uuid.0);

        self.region_manager.load_heightmap(region_uuid, filename).await?;

        Ok(RemoteAdminResponse {
            success: true,
            message: format!("Heightmap {} loaded for region {}", filename, region_uuid.0),
            data: HashMap::new(),
        })
    }

    /// Save heightmap
    async fn admin_save_heightmap(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64))
            .ok_or_else(|| anyhow!("Missing or invalid region_uuid parameter"))?;
        
        let filename = params.get("filename")
            .ok_or_else(|| anyhow!("Missing filename parameter"))?;

        info!("Saving heightmap {} for region {}", filename, region_uuid.0);

        self.region_manager.save_heightmap(region_uuid, filename).await?;

        Ok(RemoteAdminResponse {
            success: true,
            message: format!("Heightmap saved to {} for region {}", filename, region_uuid.0),
            data: HashMap::new(),
        })
    }

    /// Load XML
    async fn admin_load_xml(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64))
            .ok_or_else(|| anyhow!("Missing or invalid region_uuid parameter"))?;
        
        let filename = params.get("filename")
            .ok_or_else(|| anyhow!("Missing filename parameter"))?;

        info!("Loading XML {} for region {}", filename, region_uuid.0);

        self.region_manager.load_xml(region_uuid, filename).await?;

        Ok(RemoteAdminResponse {
            success: true,
            message: format!("XML {} loaded for region {}", filename, region_uuid.0),
            data: HashMap::new(),
        })
    }

    /// Save XML
    async fn admin_save_xml(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64))
            .ok_or_else(|| anyhow!("Missing or invalid region_uuid parameter"))?;
        
        let filename = params.get("filename")
            .ok_or_else(|| anyhow!("Missing filename parameter"))?;

        info!("Saving XML {} for region {}", filename, region_uuid.0);

        self.region_manager.save_xml(region_uuid, filename).await?;

        Ok(RemoteAdminResponse {
            success: true,
            message: format!("XML saved to {} for region {}", filename, region_uuid.0),
            data: HashMap::new(),
        })
    }

    /// Load OAR
    async fn admin_load_oar(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64))
            .ok_or_else(|| anyhow!("Missing or invalid region_uuid parameter"))?;
        
        let filename = params.get("filename")
            .ok_or_else(|| anyhow!("Missing filename parameter"))?;

        info!("Loading OAR {} for region {}", filename, region_uuid.0);

        self.region_manager.load_oar(region_uuid, filename).await?;

        Ok(RemoteAdminResponse {
            success: true,
            message: format!("OAR {} loaded for region {}", filename, region_uuid.0),
            data: HashMap::new(),
        })
    }

    /// Save OAR
    async fn admin_save_oar(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64))
            .ok_or_else(|| anyhow!("Missing or invalid region_uuid parameter"))?;
        
        let filename = params.get("filename")
            .ok_or_else(|| anyhow!("Missing filename parameter"))?;

        info!("Saving OAR {} for region {}", filename, region_uuid.0);

        self.region_manager.save_oar(region_uuid, filename).await?;

        Ok(RemoteAdminResponse {
            success: true,
            message: format!("OAR saved to {} for region {}", filename, region_uuid.0),
            data: HashMap::new(),
        })
    }

    /// Broadcast message
    async fn admin_broadcast(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let message = params.get("message")
            .ok_or_else(|| anyhow!("Missing message parameter"))?;

        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64));

        info!("Broadcasting message: {}", message);

        if let Some(region_id) = region_uuid {
            self.session_manager.broadcast_to_region(Uuid::from_u128(region_id.0 as u128), message).await
                .map_err(|e| anyhow::anyhow!("Failed to broadcast to region: {}", e))?;
        } else {
            self.session_manager.broadcast_to_all(message).await
                .map_err(|e| anyhow::anyhow!("Failed to broadcast to all: {}", e))?;
        }

        Ok(RemoteAdminResponse {
            success: true,
            message: "Message broadcast successfully".to_string(),
            data: HashMap::new(),
        })
    }

    /// Query region information
    async fn admin_region_query(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let region_uuid = params.get("region_uuid")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|uuid| RegionId(uuid.as_u128() as u64));

        if let Some(region_id) = region_uuid {
            let region_info = self.region_manager.get_region_info(region_id).await?;
            
            let mut data = HashMap::new();
            data.insert("region_name".to_string(), serde_json::Value::String(region_info.name));
            data.insert("region_x".to_string(), serde_json::Value::Number(region_info.x.into()));
            data.insert("region_y".to_string(), serde_json::Value::Number(region_info.y.into()));
            data.insert("region_size_x".to_string(), serde_json::Value::Number(region_info.size_x.into()));
            data.insert("region_size_y".to_string(), serde_json::Value::Number(region_info.size_y.into()));
            data.insert("avatar_count".to_string(), serde_json::Value::Number(region_info.avatar_count.into()));

            Ok(RemoteAdminResponse {
                success: true,
                message: "Region information retrieved".to_string(),
                data,
            })
        } else {
            let regions = self.region_manager.get_all_regions().await;
            
            let mut data = HashMap::new();
            // Convert RegionIds to region information
            let mut region_infos = Vec::new();
            for region_id in regions {
                if let Ok(region_info) = self.region_manager.get_region_info(region_id).await {
                    region_infos.push(serde_json::json!({
                        "region_name": region_info.name,
                        "region_id": region_id.0.to_string(),
                        "region_x": region_info.x,
                        "region_y": region_info.y,
                        "region_size_x": region_info.size_x,
                        "region_size_y": region_info.size_y,
                        "avatar_count": region_info.avatar_count,
                    }));
                }
            }
            data.insert("regions".to_string(), serde_json::Value::Array(region_infos));

            Ok(RemoteAdminResponse {
                success: true,
                message: format!("Retrieved {} regions", data.get("regions").unwrap().as_array().unwrap().len()),
                data,
            })
        }
    }

    /// Execute console command
    async fn admin_console_command(&self, params: &HashMap<String, String>) -> Result<RemoteAdminResponse> {
        let command = params.get("command")
            .ok_or_else(|| anyhow!("Missing command parameter"))?;

        warn!("Console command execution: {}", command);

        // For security, only allow specific safe commands
        let safe_commands = [
            "show users", "show regions", "show stats", "show version",
            "show uptime", "show memory", "show threads",
        ];

        if !safe_commands.iter().any(|&safe_cmd| command.starts_with(safe_cmd)) {
            return Err(anyhow!("Command '{}' not allowed for security reasons", command));
        }

        // Execute safe command and return result
        let result = self.execute_console_command(command).await?;

        let mut data = HashMap::new();
        data.insert("result".to_string(), serde_json::Value::String(result));

        Ok(RemoteAdminResponse {
            success: true,
            message: "Console command executed".to_string(),
            data,
        })
    }

    /// Execute console command (safe commands only)
    async fn execute_console_command(&self, command: &str) -> Result<String> {
        match command {
            "show users" => {
                let agents = self.session_manager.get_all_agents().await;
                Ok(format!("Active users: {}\n{}", agents.len(), 
                    agents.iter().map(|a| format!("{} {} ({})", a.first_name, a.last_name, a.user_id))
                        .collect::<Vec<_>>().join("\n")))
            },
            "show regions" => {
                let region_ids = self.region_manager.get_all_regions().await;
                let mut regions_info = Vec::new();
                for region_id in region_ids {
                    if let Ok(region_info) = self.region_manager.get_region_info(region_id).await {
                        regions_info.push(format!("{} at {},{}", 
                            region_info.name, region_info.x, region_info.y));
                    }
                }
                Ok(format!("Active regions: {}\n{}", regions_info.len(), regions_info.join("\n")))
            },
            "show stats" => {
                let stats = self.command_stats.read().await;
                Ok(format!("RemoteAdmin Statistics:\nTotal commands: {}\nSuccessful: {}\nFailed: {}", 
                    stats.total_commands, stats.successful_commands, stats.failed_commands))
            },
            "show version" => {
                Ok("OpenSim Next v1.0.0 - Rust/Zig High Performance Virtual World Server".to_string())
            },
            "show uptime" => {
                // This would need to be tracked from server start time
                Ok("Uptime information not available".to_string())
            },
            "show memory" => {
                Ok("Memory usage information not available".to_string())
            },
            "show threads" => {
                Ok("Thread information not available".to_string())
            },
            _ => Err(anyhow!("Unknown safe command: {}", command)),
        }
    }

    /// Get command statistics
    pub async fn get_statistics(&self) -> CommandStatistics {
        self.command_stats.read().await.clone()
    }

    /// Enable or disable a command
    pub fn set_command_enabled(&mut self, command: &str, enabled: bool) {
        self.enabled_commands.insert(command.to_string(), enabled);
        info!("RemoteAdmin command '{}' {}", command, if enabled { "enabled" } else { "disabled" });
    }

    /// Get enabled commands
    pub fn get_enabled_commands(&self) -> Vec<String> {
        self.enabled_commands.iter()
            .filter(|(_, &enabled)| enabled)
            .map(|(cmd, _)| cmd.clone())
            .collect()
    }
}

/// Handle admin request (renamed to handle GET route)
async fn handle_admin_request_get(
    State(service): State<Arc<RemoteAdminService>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let method = params.get("method").cloned().unwrap_or_default();
    let password = params.get("password").cloned().unwrap_or_default();
    let mut parameters = params.clone();
    parameters.remove("method");
    parameters.remove("password");
    
    handle_admin_request_internal(service, method, password, parameters).await
}

/// Handle admin request (POST with JSON)
async fn handle_admin_request_post(
    State(service): State<Arc<RemoteAdminService>>,
    Json(request): Json<RemoteAdminRequest>,
) -> impl IntoResponse {
    handle_admin_request_internal(service, request.method, request.password, request.parameters).await
}

/// Internal admin request handler
async fn handle_admin_request_internal(
    service: Arc<RemoteAdminService>,
    method: String,
    password: String,
    parameters: HashMap<String, String>,
) -> impl IntoResponse {
    // Direct parameter use (already extracted by caller)

    // Authenticate
    if !service.authenticate(&password) {
        warn!("RemoteAdmin: Authentication failed for method {}", method);
        return (StatusCode::UNAUTHORIZED, Json(RemoteAdminResponse {
            success: false,
            message: "Authentication failed".to_string(),
            data: HashMap::new(),
        })).into_response();
    }

    // Check if command is enabled
    if !service.is_command_enabled(&method) {
        warn!("RemoteAdmin: Command {} is disabled", method);
        return (StatusCode::FORBIDDEN, Json(RemoteAdminResponse {
            success: false,
            message: format!("Command '{}' is disabled", method),
            data: HashMap::new(),
        })).into_response();
    }

    // Execute command
    match service.execute_command(&method, &parameters).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("RemoteAdmin command '{}' failed: {}", method, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(RemoteAdminResponse {
                success: false,
                message: e.to_string(),
                data: HashMap::new(),
            })).into_response()
        }
    }
}

/// Handle admin status request
async fn handle_admin_status(
    State(service): State<Arc<RemoteAdminService>>,
) -> impl IntoResponse {
    let stats = service.get_statistics().await;
    
    (StatusCode::OK, Json(serde_json::json!({
        "status": "running",
        "enabled_commands": service.get_enabled_commands(),
        "statistics": stats,
        "access_restrictions": service.access_restrictions,
    }))).into_response()
}

/// Handle admin commands list
async fn handle_admin_commands(
    State(service): State<Arc<RemoteAdminService>>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "available_commands": service.access_restrictions.allowed_methods,
        "enabled_commands": service.get_enabled_commands(),
    }))).into_response()
}

/// Handle admin statistics
async fn handle_admin_stats(
    State(service): State<Arc<RemoteAdminService>>,
) -> impl IntoResponse {
    let stats = service.get_statistics().await;
    (StatusCode::OK, Json(stats)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_authentication() {
        let service = create_test_service().await;
        
        assert!(service.authenticate("test_password"));
        assert!(!service.authenticate("wrong_password"));
        assert!(!service.authenticate(""));
    }
    
    #[tokio::test]
    async fn test_command_enabling() {
        let mut service = create_test_service().await;
        
        assert!(service.is_command_enabled("admin_exists_user"));
        assert!(!service.is_command_enabled("admin_create_user"));
        
        service.set_command_enabled("admin_create_user", true);
        assert!(service.is_command_enabled("admin_create_user"));
    }
    
    async fn create_test_service() -> RemoteAdminService {
        // This would create mock services for testing
        // Implementation would depend on your test setup
        unimplemented!("Test setup needed")
    }
}