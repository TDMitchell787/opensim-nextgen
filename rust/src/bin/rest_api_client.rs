//! OpenSim Next REST API Client
//! Command-line tool for interacting with the REST API

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, Args};
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::fs as async_fs;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "rest-api-client")]
#[command(about = "OpenSim Next REST API Client")]
#[command(version = "1.0.0")]
struct Cli {
    /// API server URL
    #[arg(long, default_value = "http://localhost:8080")]
    server: String,
    
    /// API version
    #[arg(long, default_value = "v1")]
    version: String,
    
    /// Authentication token
    #[arg(long, env = "OPENSIM_API_TOKEN")]
    token: Option<String>,
    
    /// Username for authentication
    #[arg(long, env = "OPENSIM_USERNAME")]
    username: Option<String>,
    
    /// Password for authentication
    #[arg(long, env = "OPENSIM_PASSWORD")]
    password: Option<String>,
    
    /// Output format (json, table, csv)
    #[arg(long, default_value = "json")]
    format: String,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        action: AuthCommands,
    },
    /// Asset management commands
    Assets {
        #[command(subcommand)]
        action: AssetCommands,
    },
    /// Inventory management commands
    Inventory {
        #[command(subcommand)]
        action: InventoryCommands,
    },
    /// User management commands
    Users {
        #[command(subcommand)]
        action: UserCommands,
    },
    /// Region management commands
    Regions {
        #[command(subcommand)]
        action: RegionCommands,
    },
    /// Statistics and monitoring commands
    Stats {
        #[command(subcommand)]
        action: StatsCommands,
    },
    /// API information and health
    Info,
    /// Health check
    Health,
    /// Version information
    Version,
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Login and get access token
    Login,
    /// Logout and invalidate token
    Logout,
    /// Refresh access token
    Refresh,
    /// Validate current token
    Validate,
}

#[derive(Subcommand)]
enum AssetCommands {
    /// List assets
    List {
        /// Search query
        #[arg(long)]
        query: Option<String>,
        /// Asset type filter
        #[arg(long)]
        asset_type: Option<String>,
        /// Creator ID filter
        #[arg(long)]
        creator_id: Option<Uuid>,
        /// Public assets only
        #[arg(long)]
        public: bool,
        /// Page number
        #[arg(long, default_value_t = 1)]
        page: u32,
        /// Items per page
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    /// Get asset information
    Get {
        /// Asset ID
        id: Uuid,
    },
    /// Upload asset
    Upload {
        /// Asset file path
        file: PathBuf,
        /// Asset name
        #[arg(long)]
        name: String,
        /// Asset description
        #[arg(long)]
        description: Option<String>,
        /// Asset type
        #[arg(long)]
        asset_type: String,
        /// Make public
        #[arg(long)]
        public: bool,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// Download asset data
    Download {
        /// Asset ID
        id: Uuid,
        /// Output file path
        #[arg(long)]
        output: PathBuf,
    },
    /// Update asset metadata
    Update {
        /// Asset ID
        id: Uuid,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// Make public/private
        #[arg(long)]
        public: Option<bool>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// Delete asset
    Delete {
        /// Asset ID
        id: Uuid,
        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },
    /// Search assets
    Search {
        /// Search query
        query: String,
        /// Asset type filter
        #[arg(long)]
        asset_type: Option<String>,
        /// Tags filter
        #[arg(long)]
        tags: Option<String>,
        /// Page number
        #[arg(long, default_value_t = 1)]
        page: u32,
        /// Items per page
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
}

#[derive(Subcommand)]
enum InventoryCommands {
    /// List user inventory items
    Items {
        /// User ID
        user_id: Uuid,
        /// Page number
        #[arg(long, default_value_t = 1)]
        page: u32,
        /// Items per page
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    /// List user inventory folders
    Folders {
        /// User ID
        user_id: Uuid,
    },
    /// Get inventory item details
    GetItem {
        /// Item ID
        id: Uuid,
    },
    /// Get inventory folder details
    GetFolder {
        /// Folder ID
        id: Uuid,
    },
    /// Create inventory item
    CreateItem {
        /// User ID
        user_id: Uuid,
        /// Item name
        #[arg(long)]
        name: String,
        /// Item description
        #[arg(long)]
        description: Option<String>,
        /// Folder ID
        #[arg(long)]
        folder_id: Uuid,
        /// Asset ID
        #[arg(long)]
        asset_id: Option<Uuid>,
        /// Item type
        #[arg(long)]
        item_type: String,
    },
    /// Create inventory folder
    CreateFolder {
        /// User ID
        user_id: Uuid,
        /// Folder name
        #[arg(long)]
        name: String,
        /// Parent folder ID
        #[arg(long)]
        parent_id: Option<Uuid>,
        /// Folder type
        #[arg(long)]
        folder_type: String,
    },
}

#[derive(Subcommand)]
enum UserCommands {
    /// List users
    List {
        /// Page number
        #[arg(long, default_value_t = 1)]
        page: u32,
        /// Items per page
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    /// Get user details
    Get {
        /// User ID
        id: Uuid,
    },
    /// Get user profile
    Profile {
        /// User ID
        id: Uuid,
    },
    /// Create user
    Create {
        /// Username
        #[arg(long)]
        username: String,
        /// Email
        #[arg(long)]
        email: String,
        /// Password
        #[arg(long)]
        password: String,
        /// First name
        #[arg(long)]
        first_name: String,
        /// Last name
        #[arg(long)]
        last_name: String,
    },
    /// Update user
    Update {
        /// User ID
        id: Uuid,
        /// New email
        #[arg(long)]
        email: Option<String>,
        /// New first name
        #[arg(long)]
        first_name: Option<String>,
        /// New last name
        #[arg(long)]
        last_name: Option<String>,
        /// New display name
        #[arg(long)]
        display_name: Option<String>,
    },
    /// Delete user
    Delete {
        /// User ID
        id: Uuid,
        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
enum RegionCommands {
    /// List regions
    List {
        /// Page number
        #[arg(long, default_value_t = 1)]
        page: u32,
        /// Items per page
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    /// Get region details
    Get {
        /// Region ID
        id: Uuid,
    },
    /// Create region
    Create {
        /// Region name
        #[arg(long)]
        name: String,
        /// Location X coordinate
        #[arg(long)]
        location_x: u32,
        /// Location Y coordinate
        #[arg(long)]
        location_y: u32,
        /// Size X (default 256)
        #[arg(long)]
        size_x: Option<u32>,
        /// Size Y (default 256)
        #[arg(long)]
        size_y: Option<u32>,
        /// Estate ID
        #[arg(long)]
        estate_id: u32,
    },
    /// Update region
    Update {
        /// Region ID
        id: Uuid,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New location X
        #[arg(long)]
        location_x: Option<u32>,
        /// New location Y
        #[arg(long)]
        location_y: Option<u32>,
    },
    /// Delete region
    Delete {
        /// Region ID
        id: Uuid,
        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },
    /// Restart region
    Restart {
        /// Region ID
        id: Uuid,
    },
    /// List agents in region
    Agents {
        /// Region ID
        id: Uuid,
    },
    /// List objects in region
    Objects {
        /// Region ID
        id: Uuid,
        /// Page number
        #[arg(long, default_value_t = 1)]
        page: u32,
        /// Items per page
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
}

#[derive(Subcommand)]
enum StatsCommands {
    /// Get overview statistics
    Overview,
    /// Get asset statistics
    Assets,
    /// Get user statistics
    Users,
    /// Get region statistics
    Regions,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    timestamp: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    remember_me: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u32,
    user: Value,
}

struct RestApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
    verbose: bool,
}

impl RestApiClient {
    fn new(server: String, version: String, token: Option<String>, verbose: bool) -> Self {
        let base_url = format!("{}/{}", server.trim_end_matches('/'), version);
        
        Self {
            client: Client::new(),
            base_url,
            token,
            verbose,
        }
    }
    
    async fn login(&mut self, username: String, password: String) -> Result<LoginResponse> {
        let request = LoginRequest {
            username,
            password,
            remember_me: Some(false),
        };
        
        let response = self.client
            .post(&format!("{}/auth/login", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Login failed: HTTP {}", response.status()));
        }
        
        let api_response: ApiResponse<LoginResponse> = response.json().await?;
        
        if !api_response.success {
            return Err(anyhow!("Login failed: {}", api_response.error.unwrap_or_default()));
        }
        
        let login_data = api_response.data.unwrap();
        self.token = Some(login_data.access_token.clone());
        
        Ok(login_data)
    }
    
    async fn make_request<T>(&self, method: &str, path: &str, body: Option<Value>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, path);
        
        if self.verbose {
            println!("Making {} request to: {}", method, url);
            if let Some(ref body) = body {
                println!("Request body: {}", serde_json::to_string_pretty(body)?);
            }
        }
        
        let mut request_builder = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };
        
        // Add authentication header if token is available
        if let Some(ref token) = self.token {
            request_builder = request_builder.bearer_auth(token);
        }
        
        // Add JSON body if provided
        if let Some(body) = body {
            request_builder = request_builder.json(&body);
        }
        
        let response = request_builder.send().await?;
        
        if self.verbose {
            println!("Response status: {}", response.status());
        }
        
        let status = response.status();
        let response_text = response.text().await?;
        
        if self.verbose {
            println!("Response body: {}", response_text);
        }
        
        if !status.is_success() {
            return Err(anyhow!("HTTP error {}: {}", status, response_text));
        }
        
        let api_response: ApiResponse<T> = serde_json::from_str(&response_text)?;
        
        if !api_response.success {
            return Err(anyhow!("API error: {}", api_response.error.unwrap_or_default()));
        }
        
        api_response.data.ok_or_else(|| anyhow!("No data in response"))
    }
    
    async fn upload_file(&self, path: &str, file_path: &PathBuf, metadata: Value) -> Result<Value> {
        let url = format!("{}/{}", self.base_url, path);
        
        if self.verbose {
            println!("Uploading file to: {}", url);
            println!("File: {}", file_path.display());
            println!("Metadata: {}", serde_json::to_string_pretty(&metadata)?);
        }
        
        // Read file data
        let file_data = async_fs::read(file_path).await?;
        let file_name = file_path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("Invalid file name"))?;
        
        // Create multipart form
        let mut form = multipart::Form::new()
            .part("metadata", multipart::Part::text(metadata.to_string()));
        
        form = form.part("file", multipart::Part::bytes(file_data)
            .file_name(file_name.to_string()));
        
        let mut request_builder = self.client.post(&url).multipart(form);
        
        // Add authentication header if token is available
        if let Some(ref token) = self.token {
            request_builder = request_builder.bearer_auth(token);
        }
        
        let response = request_builder.send().await?;
        
        if self.verbose {
            println!("Upload response status: {}", response.status());
        }
        
        let status = response.status();
        let response_text = response.text().await?;
        
        if self.verbose {
            println!("Upload response body: {}", response_text);
        }
        
        if !status.is_success() {
            return Err(anyhow!("Upload failed: HTTP {}: {}", status, response_text));
        }
        
        let api_response: ApiResponse<Value> = serde_json::from_str(&response_text)?;
        
        if !api_response.success {
            return Err(anyhow!("Upload failed: {}", api_response.error.unwrap_or_default()));
        }
        
        api_response.data.ok_or_else(|| anyhow!("No data in upload response"))
    }
}

fn format_output(data: &Value, format: &str) -> Result<String> {
    match format {
        "json" => Ok(serde_json::to_string_pretty(data)?),
        "table" | "csv" => {
            // For demo purposes, just return JSON
            // In a real implementation, you'd format as a table or CSV
            Ok(serde_json::to_string_pretty(data)?)
        }
        _ => Err(anyhow!("Unsupported output format: {}", format)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let mut client = RestApiClient::new(
        cli.server,
        cli.version,
        cli.token,
        cli.verbose,
    );
    
    // Handle authentication if username/password provided
    if let (Some(username), Some(password)) = (cli.username, cli.password) {
        if client.token.is_none() {
            println!("Logging in as {}...", username);
            let login_response = client.login(username, password).await?;
            println!("✅ Login successful");
            println!("Access token expires in {} seconds", login_response.expires_in);
        }
    }
    
    match cli.command {
        Commands::Info => {
            let data: Value = client.make_request("GET", "", None).await?;
            println!("{}", format_output(&data, &cli.format)?);
        }
        
        Commands::Health => {
            let data: Value = client.make_request("GET", "health", None).await?;
            println!("{}", format_output(&data, &cli.format)?);
        }
        
        Commands::Version => {
            let data: Value = client.make_request("GET", "version", None).await?;
            println!("{}", format_output(&data, &cli.format)?);
        }
        
        Commands::Auth { action } => {
            match action {
                AuthCommands::Login => {
                    println!("Use --username and --password for login");
                }
                AuthCommands::Logout => {
                    let _: Value = client.make_request("POST", "auth/logout", None).await?;
                    println!("✅ Logged out successfully");
                }
                AuthCommands::Refresh => {
                    let data: Value = client.make_request("POST", "auth/refresh", None).await?;
                    println!("{}", format_output(&data, &cli.format)?);
                }
                AuthCommands::Validate => {
                    let data: Value = client.make_request("GET", "auth/validate", None).await?;
                    println!("{}", format_output(&data, &cli.format)?);
                }
            }
        }
        
        Commands::Assets { action } => {
            match action {
                AssetCommands::List { query, asset_type, creator_id, public, page, limit } => {
                    let mut params = Vec::new();
                    if let Some(q) = query { params.push(format!("query={}", q)); }
                    if let Some(t) = asset_type { params.push(format!("asset_type={}", t)); }
                    if let Some(c) = creator_id { params.push(format!("creator_id={}", c)); }
                    if public { params.push("is_public=true".to_string()); }
                    params.push(format!("page={}", page));
                    params.push(format!("limit={}", limit));
                    
                    let query_string = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
                    let data: Value = client.make_request("GET", &format!("assets{}", query_string), None).await?;
                    println!("{}", format_output(&data, &cli.format)?);
                }
                
                AssetCommands::Get { id } => {
                    let data: Value = client.make_request("GET", &format!("assets/{}", id), None).await?;
                    println!("{}", format_output(&data, &cli.format)?);
                }
                
                AssetCommands::Upload { file, name, description, asset_type, public, tags } => {
                    if !file.exists() {
                        return Err(anyhow!("File not found: {}", file.display()));
                    }
                    
                    let metadata = json!({
                        "name": name,
                        "description": description,
                        "asset_type": asset_type,
                        "is_public": public,
                        "tags": tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect::<Vec<String>>())
                    });
                    
                    let data = client.upload_file("assets", &file, metadata).await?;
                    println!("✅ Asset uploaded successfully");
                    println!("{}", format_output(&data, &cli.format)?);
                }
                
                AssetCommands::Download { id, output } => {
                    // This would download the asset data
                    println!("Downloading asset {} to {}", id, output.display());
                    println!("⚠️  Download functionality not yet implemented");
                }
                
                AssetCommands::Update { id, name, description, public, tags } => {
                    let mut updates = json!({});
                    if let Some(n) = name { updates["name"] = json!(n); }
                    if let Some(d) = description { updates["description"] = json!(d); }
                    if let Some(p) = public { updates["is_public"] = json!(p); }
                    if let Some(t) = tags { 
                        updates["tags"] = json!(t.split(',').map(|s| s.trim()).collect::<Vec<_>>());
                    }
                    
                    let data: Value = client.make_request("PUT", &format!("assets/{}", id), Some(updates)).await?;
                    println!("✅ Asset updated successfully");
                    println!("{}", format_output(&data, &cli.format)?);
                }
                
                AssetCommands::Delete { id, confirm } => {
                    if !confirm {
                        return Err(anyhow!("Use --confirm to confirm deletion"));
                    }
                    
                    let _: Value = client.make_request("DELETE", &format!("assets/{}", id), None).await?;
                    println!("✅ Asset deleted successfully");
                }
                
                AssetCommands::Search { query, asset_type, tags, page, limit } => {
                    let mut params = vec![format!("query={}", query)];
                    if let Some(t) = asset_type { params.push(format!("asset_type={}", t)); }
                    if let Some(t) = tags { params.push(format!("tags={}", t)); }
                    params.push(format!("page={}", page));
                    params.push(format!("limit={}", limit));
                    
                    let query_string = format!("?{}", params.join("&"));
                    let data: Value = client.make_request("GET", &format!("assets/search{}", query_string), None).await?;
                    println!("{}", format_output(&data, &cli.format)?);
                }
            }
        }
        
        // Add other command handlers here...
        _ => {
            println!("⚠️  Command not yet implemented");
        }
    }
    
    Ok(())
}