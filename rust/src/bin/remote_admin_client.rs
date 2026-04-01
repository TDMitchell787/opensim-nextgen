//! OpenSim Next RemoteAdmin Client
//! Command-line tool for remote administration of OpenSim Next servers

use std::collections::HashMap;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "remote-admin-client")]
#[command(about = "OpenSim Next RemoteAdmin Client")]
#[command(version = "1.0.0")]
struct Cli {
    /// Server hostname
    #[arg(long, default_value = "localhost")]
    host: String,
    
    /// Server port
    #[arg(long, default_value_t = 9000)]
    port: u16,
    
    /// Admin password
    #[arg(long, env = "OPENSIM_ADMIN_PASSWORD")]
    password: String,
    
    /// Use HTTPS
    #[arg(long)]
    ssl: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new user account
    CreateUser {
        /// First name
        firstname: String,
        /// Last name
        lastname: String,
        /// User password
        password: String,
        /// Email address
        email: String,
    },
    /// Check if a user exists
    UserExists {
        /// First name
        firstname: String,
        /// Last name
        lastname: String,
    },
    /// Get list of logged in agents
    GetAgents {
        /// Specific region UUID
        #[arg(long)]
        region_uuid: Option<Uuid>,
    },
    /// Teleport an agent to a region
    Teleport {
        /// Agent UUID
        agent_id: Uuid,
        /// Destination region name
        region_name: String,
        /// X coordinate
        #[arg(long, default_value_t = 128.0)]
        x: f32,
        /// Y coordinate
        #[arg(long, default_value_t = 128.0)]
        y: f32,
        /// Z coordinate
        #[arg(long, default_value_t = 25.0)]
        z: f32,
    },
    /// Restart region(s)
    Restart {
        /// Specific region UUID (all regions if not specified)
        #[arg(long)]
        region_uuid: Option<Uuid>,
    },
    /// Load an OAR file into a region
    LoadOar {
        /// Region UUID
        region_uuid: Uuid,
        /// OAR filename
        filename: String,
    },
    /// Save a region to an OAR file
    SaveOar {
        /// Region UUID
        region_uuid: Uuid,
        /// OAR filename
        filename: String,
    },
    /// Broadcast a message to users
    Broadcast {
        /// Message to broadcast
        message: String,
        /// Specific region UUID
        #[arg(long)]
        region_uuid: Option<Uuid>,
    },
    /// Query region information
    QueryRegions {
        /// Specific region UUID
        #[arg(long)]
        region_uuid: Option<Uuid>,
    },
    /// Execute a console command
    Console {
        /// Console command to execute
        command: String,
    },
    /// Get RemoteAdmin status
    Status,
}

#[derive(Debug, Serialize)]
struct RemoteAdminRequest {
    method: String,
    password: String,
    #[serde(flatten)]
    parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RemoteAdminResponse {
    success: bool,
    message: String,
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    status: String,
    enabled_commands: Vec<String>,
    statistics: HashMap<String, serde_json::Value>,
}

struct RemoteAdminClient {
    client: Client,
    base_url: String,
    password: String,
}

impl RemoteAdminClient {
    fn new(host: &str, port: u16, password: String, ssl: bool) -> Self {
        let protocol = if ssl { "https" } else { "http" };
        let base_url = format!("{}://{}:{}/admin", protocol, host, port);
        
        Self {
            client: Client::new(),
            base_url,
            password,
        }
    }
    
    async fn execute_command(
        &self,
        method: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<RemoteAdminResponse> {
        let request = RemoteAdminRequest {
            method: method.to_string(),
            password: self.password.clone(),
            parameters,
        };
        
        let response = self.client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            
            return Err(match status.as_u16() {
                401 => anyhow!("Authentication failed - check password"),
                403 => anyhow!("Command '{}' is disabled", method),
                500 => {
                    if let Ok(error_data) = serde_json::from_str::<RemoteAdminResponse>(&error_text) {
                        anyhow!("Server error: {}", error_data.message)
                    } else {
                        anyhow!("Server error: HTTP {}", status.as_u16())
                    }
                },
                _ => anyhow!("HTTP error: {}", status.as_u16()),
            });
        }
        
        let result: RemoteAdminResponse = response.json().await?;
        Ok(result)
    }
    
    async fn get_status(&self) -> Result<StatusResponse> {
        let response = self.client
            .get(&format!("{}/status", self.base_url))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to get status: HTTP {}", response.status().as_u16()));
        }
        
        let status: StatusResponse = response.json().await?;
        Ok(status)
    }
    
    async fn create_user(&self, firstname: &str, lastname: &str, password: &str, email: &str) -> Result<RemoteAdminResponse> {
        let mut params = HashMap::new();
        params.insert("user_firstname".to_string(), json!(firstname));
        params.insert("user_lastname".to_string(), json!(lastname));
        params.insert("user_password".to_string(), json!(password));
        params.insert("user_email".to_string(), json!(email));
        
        self.execute_command("admin_create_user", params).await
    }
    
    async fn user_exists(&self, firstname: &str, lastname: &str) -> Result<bool> {
        let mut params = HashMap::new();
        params.insert("user_firstname".to_string(), json!(firstname));
        params.insert("user_lastname".to_string(), json!(lastname));
        
        let result = self.execute_command("admin_exists_user", params).await?;
        Ok(result.data.get("user_exists").and_then(|v| v.as_bool()).unwrap_or(false))
    }
    
    async fn get_agents(&self, region_uuid: Option<Uuid>) -> Result<Vec<serde_json::Value>> {
        let mut params = HashMap::new();
        if let Some(uuid) = region_uuid {
            params.insert("region_uuid".to_string(), json!(uuid.to_string()));
        }
        
        let result = self.execute_command("admin_get_agents", params).await?;
        Ok(result.data.get("agents").and_then(|v| v.as_array()).cloned().unwrap_or_default())
    }
    
    async fn teleport_agent(&self, agent_id: Uuid, region_name: &str, x: f32, y: f32, z: f32) -> Result<RemoteAdminResponse> {
        let mut params = HashMap::new();
        params.insert("agent_id".to_string(), json!(agent_id.to_string()));
        params.insert("region_name".to_string(), json!(region_name));
        params.insert("pos_x".to_string(), json!(x.to_string()));
        params.insert("pos_y".to_string(), json!(y.to_string()));
        params.insert("pos_z".to_string(), json!(z.to_string()));
        
        self.execute_command("admin_teleport_agent", params).await
    }
    
    async fn restart_region(&self, region_uuid: Option<Uuid>) -> Result<RemoteAdminResponse> {
        let mut params = HashMap::new();
        if let Some(uuid) = region_uuid {
            params.insert("region_uuid".to_string(), json!(uuid.to_string()));
        }
        
        self.execute_command("admin_restart", params).await
    }
    
    async fn load_oar(&self, region_uuid: Uuid, filename: &str) -> Result<RemoteAdminResponse> {
        let mut params = HashMap::new();
        params.insert("region_uuid".to_string(), json!(region_uuid.to_string()));
        params.insert("filename".to_string(), json!(filename));
        
        self.execute_command("admin_load_oar", params).await
    }
    
    async fn save_oar(&self, region_uuid: Uuid, filename: &str) -> Result<RemoteAdminResponse> {
        let mut params = HashMap::new();
        params.insert("region_uuid".to_string(), json!(region_uuid.to_string()));
        params.insert("filename".to_string(), json!(filename));
        
        self.execute_command("admin_save_oar", params).await
    }
    
    async fn broadcast_message(&self, message: &str, region_uuid: Option<Uuid>) -> Result<RemoteAdminResponse> {
        let mut params = HashMap::new();
        params.insert("message".to_string(), json!(message));
        if let Some(uuid) = region_uuid {
            params.insert("region_uuid".to_string(), json!(uuid.to_string()));
        }
        
        self.execute_command("admin_broadcast", params).await
    }
    
    async fn query_regions(&self, region_uuid: Option<Uuid>) -> Result<RemoteAdminResponse> {
        let mut params = HashMap::new();
        if let Some(uuid) = region_uuid {
            params.insert("region_uuid".to_string(), json!(uuid.to_string()));
        }
        
        self.execute_command("admin_region_query", params).await
    }
    
    async fn console_command(&self, command: &str) -> Result<RemoteAdminResponse> {
        let mut params = HashMap::new();
        params.insert("command".to_string(), json!(command));
        
        self.execute_command("admin_console_command", params).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let client = RemoteAdminClient::new(&cli.host, cli.port, cli.password, cli.ssl);
    
    match cli.command {
        Commands::CreateUser { firstname, lastname, password, email } => {
            match client.create_user(&firstname, &lastname, &password, &email).await {
                Ok(result) => {
                    println!("✅ User created: {}", result.message);
                    if let Some(uuid) = result.data.get("avatar_uuid") {
                        println!("Avatar UUID: {}", uuid);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to create user: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::UserExists { firstname, lastname } => {
            match client.user_exists(&firstname, &lastname).await {
                Ok(exists) => {
                    println!("User {} {}: {}", firstname, lastname, 
                            if exists { "✅ EXISTS" } else { "❌ NOT FOUND" });
                }
                Err(e) => {
                    eprintln!("❌ Failed to check user existence: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::GetAgents { region_uuid } => {
            match client.get_agents(region_uuid).await {
                Ok(agents) => {
                    println!("📊 Found {} agents:", agents.len());
                    for agent in agents {
                        if let (Some(firstname), Some(lastname), Some(uuid)) = (
                            agent.get("firstname").and_then(|v| v.as_str()),
                            agent.get("lastname").and_then(|v| v.as_str()),
                            agent.get("uuid").and_then(|v| v.as_str()),
                        ) {
                            println!("  👤 {} {} ({})", firstname, lastname, uuid);
                            if let (Some(region_uuid), Some(position)) = (
                                agent.get("region_uuid").and_then(|v| v.as_str()),
                                agent.get("position").and_then(|v| v.as_str()),
                            ) {
                                println!("     📍 Region: {}, Position: {}", region_uuid, position);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to get agents: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Teleport { agent_id, region_name, x, y, z } => {
            match client.teleport_agent(agent_id, &region_name, x, y, z).await {
                Ok(result) => {
                    println!("✅ Teleport result: {}", result.message);
                }
                Err(e) => {
                    eprintln!("❌ Failed to teleport agent: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Restart { region_uuid } => {
            match client.restart_region(region_uuid).await {
                Ok(result) => {
                    println!("✅ Restart result: {}", result.message);
                }
                Err(e) => {
                    eprintln!("❌ Failed to restart: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::LoadOar { region_uuid, filename } => {
            match client.load_oar(region_uuid, &filename).await {
                Ok(result) => {
                    println!("✅ Load OAR result: {}", result.message);
                }
                Err(e) => {
                    eprintln!("❌ Failed to load OAR: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::SaveOar { region_uuid, filename } => {
            match client.save_oar(region_uuid, &filename).await {
                Ok(result) => {
                    println!("✅ Save OAR result: {}", result.message);
                }
                Err(e) => {
                    eprintln!("❌ Failed to save OAR: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Broadcast { message, region_uuid } => {
            match client.broadcast_message(&message, region_uuid).await {
                Ok(result) => {
                    println!("✅ Broadcast result: {}", result.message);
                }
                Err(e) => {
                    eprintln!("❌ Failed to broadcast: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::QueryRegions { region_uuid } => {
            match client.query_regions(region_uuid).await {
                Ok(result) => {
                    println!("✅ Region query result: {}", result.message);
                    
                    if let Some(regions) = result.data.get("regions").and_then(|v| v.as_array()) {
                        println!("🗺️  Regions:");
                        for region in regions {
                            if let (Some(name), Some(uuid), Some(x), Some(y)) = (
                                region.get("region_name").and_then(|v| v.as_str()),
                                region.get("region_uuid").and_then(|v| v.as_str()),
                                region.get("region_x").and_then(|v| v.as_u64()),
                                region.get("region_y").and_then(|v| v.as_u64()),
                            ) {
                                println!("  📍 {} ({})", name, uuid);
                                println!("     Location: {},{}", x, y);
                                if let (Some(size_x), Some(size_y)) = (
                                    region.get("region_size_x").and_then(|v| v.as_u64()),
                                    region.get("region_size_y").and_then(|v| v.as_u64()),
                                ) {
                                    println!("     Size: {}x{}", size_x, size_y);
                                }
                            }
                        }
                    } else if let Some(name) = result.data.get("region_name").and_then(|v| v.as_str()) {
                        println!("🗺️  Region: {}", name);
                        if let Some(uuid) = result.data.get("region_uuid").and_then(|v| v.as_str()) {
                            println!("   UUID: {}", uuid);
                        }
                        if let (Some(x), Some(y)) = (
                            result.data.get("region_x").and_then(|v| v.as_u64()),
                            result.data.get("region_y").and_then(|v| v.as_u64()),
                        ) {
                            println!("   Location: {},{}", x, y);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to query regions: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Console { command } => {
            match client.console_command(&command).await {
                Ok(result) => {
                    println!("💻 Console command result:");
                    if let Some(output) = result.data.get("result").and_then(|v| v.as_str()) {
                        println!("{}", output);
                    } else {
                        println!("{}", result.message);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to execute console command: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Status => {
            match client.get_status().await {
                Ok(status) => {
                    println!("📊 RemoteAdmin Status:");
                    println!("   Status: {}", status.status);
                    println!("   Enabled commands: {}", status.enabled_commands.len());
                    
                    if let Some(stats) = status.statistics.get("total_commands").and_then(|v| v.as_u64()) {
                        println!("   Statistics:");
                        println!("     Total commands: {}", stats);
                        
                        if let Some(successful) = status.statistics.get("successful_commands").and_then(|v| v.as_u64()) {
                            println!("     Successful: {}", successful);
                        }
                        
                        if let Some(failed) = status.statistics.get("failed_commands").and_then(|v| v.as_u64()) {
                            println!("     Failed: {}", failed);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to get status: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}