//! Dark Services Client
//! Command-line tool for secure asset transfer using dark services

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;
use uuid::Uuid;

/// Deserialize Duration from seconds
fn deserialize_duration_from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

#[derive(Parser)]
#[command(name = "dark-services-client")]
#[command(about = "OpenSim Next Dark Services Client")]
#[command(version = "1.0.0")]
struct Cli {
    /// Dark services server URL
    #[arg(long, default_value = "https://localhost:8443")]
    server: String,
    
    /// Authentication token
    #[arg(long, env = "OPENSIM_DARK_TOKEN")]
    token: Option<String>,
    
    /// Client certificate for mutual TLS
    #[arg(long)]
    client_cert: Option<PathBuf>,
    
    /// Client private key for mutual TLS
    #[arg(long)]
    client_key: Option<PathBuf>,
    
    /// CA certificate for server verification
    #[arg(long)]
    ca_cert: Option<PathBuf>,
    
    /// Disable TLS certificate verification (dangerous!)
    #[arg(long)]
    insecure: bool,
    
    /// Output format (json, table)
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
    /// Request a secure asset transfer
    Transfer {
        /// Asset ID to transfer
        asset_id: Uuid,
        /// Requester ID
        #[arg(long)]
        requester_id: Uuid,
        /// Target region (optional)
        #[arg(long)]
        target_region: Option<Uuid>,
        /// Security level (0-100)
        #[arg(long, default_value_t = 90)]
        security_level: u8,
        /// Require encryption
        #[arg(long)]
        encryption: bool,
        /// Prefer compression
        #[arg(long)]
        compression: bool,
        /// Transfer priority
        #[arg(long, default_value = "normal")]
        priority: String,
    },
    /// Get transfer session status
    Status {
        /// Session ID
        session_id: Uuid,
    },
    /// Download asset chunks from transfer session
    Download {
        /// Session ID
        session_id: Uuid,
        /// Output directory
        #[arg(long, default_value = "./downloads")]
        output_dir: PathBuf,
        /// Verify chunks
        #[arg(long)]
        verify: bool,
    },
    /// Cancel a transfer session
    Cancel {
        /// Session ID
        session_id: Uuid,
    },
    /// List active transfer sessions
    Sessions,
    /// Set access policy for an asset
    SetPolicy {
        /// Asset ID
        asset_id: Uuid,
        /// Policy configuration file (JSON)
        #[arg(long)]
        policy_file: PathBuf,
    },
    /// Get access policy for an asset
    GetPolicy {
        /// Asset ID
        asset_id: Uuid,
    },
    /// Get dark services metrics
    Metrics,
    /// Test dark services connectivity
    Ping,
    /// Generate secure keys for testing
    GenerateKeys {
        /// Number of keys to generate
        #[arg(long, default_value_t = 1)]
        count: u32,
        /// Key length in bytes
        #[arg(long, default_value_t = 32)]
        length: usize,
        /// Output format (hex, base64)
        #[arg(long, default_value = "hex")]
        output_format: String,
    },
}

/// Dark transfer request
#[derive(Debug, Serialize)]
struct DarkTransferRequest {
    asset_id: Uuid,
    requester_id: Uuid,
    target_region: Option<Uuid>,
    security_level: u8,
    encryption_required: bool,
    compression_preferred: bool,
    priority: String,
    access_token: String,
}

/// Dark transfer response
#[derive(Debug, Deserialize, Serialize)]
struct DarkTransferResponse {
    session_id: Uuid,
    status: String,
    transfer_url: Option<String>,
    encryption_key: Option<Vec<u8>>,
    chunk_size: usize,
    total_chunks: u32,
    #[serde(deserialize_with = "deserialize_duration_from_secs")]
    estimated_duration: Duration,
    error_message: Option<String>,
}

/// Transfer session status
#[derive(Debug, Deserialize, Serialize)]
struct TransferSessionStatus {
    session_id: Uuid,
    status: String,
    progress: f32,
    chunks_transferred: u32,
    chunks_total: u32,
    bytes_transferred: u64,
    transfer_rate: f64,
    estimated_completion: Option<SystemTime>,
}

/// Asset chunk
#[derive(Debug, Deserialize)]
struct AssetChunk {
    chunk_id: u32,
    session_id: Uuid,
    data: Vec<u8>,
    checksum: String,
    is_encrypted: bool,
    is_compressed: bool,
}

/// Access policy
#[derive(Debug, Serialize, Deserialize)]
struct AccessPolicy {
    allowed_requesters: Vec<Uuid>,
    allowed_regions: Vec<Uuid>,
    time_restrictions: Option<TimeRestriction>,
    content_restrictions: ContentRestriction,
    require_encryption: bool,
    require_audit_log: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TimeRestriction {
    start_time: SystemTime,
    end_time: SystemTime,
    allowed_hours: Vec<u8>,
    allowed_days: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentRestriction {
    max_file_size: u64,
    allowed_formats: Vec<String>,
    forbidden_keywords: Vec<String>,
    require_content_scan: bool,
    adult_content_policy: String,
}

/// Dark services metrics
#[derive(Debug, Deserialize, Serialize)]
struct DarkServiceMetrics {
    total_requests: u64,
    successful_transfers: u64,
    failed_transfers: u64,
    bytes_transferred: u64,
    active_sessions: u64,
    #[serde(deserialize_with = "deserialize_duration_from_secs")]
    average_transfer_time: Duration,
    security_violations: u64,
    policy_denials: u64,
}

struct DarkServicesClient {
    client: Client,
    base_url: String,
    token: Option<String>,
    verbose: bool,
}

impl DarkServicesClient {
    fn new(server: String, token: Option<String>, verbose: bool, insecure: bool) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("OpenSim-Dark-Client/1.0");
        
        if insecure {
            client_builder = client_builder
                .danger_accept_invalid_certs(true);
        }
        
        let client = client_builder.build()?;
        
        Ok(Self {
            client,
            base_url: server,
            token,
            verbose,
        })
    }
    
    async fn request_transfer(&self, request: DarkTransferRequest) -> Result<DarkTransferResponse> {
        let url = format!("{}/api/v1/dark/transfer", self.base_url);
        
        if self.verbose {
            println!("Requesting dark transfer: {}", serde_json::to_string_pretty(&request)?);
        }
        
        let mut req_builder = self.client.post(&url).json(&request);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Transfer request failed: HTTP {}", response.status()));
        }
        
        let transfer_response: DarkTransferResponse = response.json().await?;
        
        if self.verbose {
            println!("Transfer response: {}", serde_json::to_string_pretty(&transfer_response)?);
        }
        
        Ok(transfer_response)
    }
    
    async fn get_session_status(&self, session_id: Uuid) -> Result<TransferSessionStatus> {
        let url = format!("{}/api/v1/dark/sessions/{}/status", self.base_url, session_id);
        
        let mut req_builder = self.client.get(&url);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Status request failed: HTTP {}", response.status()));
        }
        
        Ok(response.json().await?)
    }
    
    async fn download_chunks(&self, session_id: Uuid, output_dir: &PathBuf, verify: bool) -> Result<()> {
        let url = format!("{}/api/v1/dark/sessions/{}/chunks", self.base_url, session_id);
        
        let mut req_builder = self.client.get(&url);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Download request failed: HTTP {}", response.status()));
        }
        
        let chunks: Vec<AssetChunk> = response.json().await?;
        
        // Create output directory
        fs::create_dir_all(output_dir).await?;
        
        println!("Downloading {} chunks to {}", chunks.len(), output_dir.display());
        
        for chunk in chunks {
            let chunk_file = output_dir.join(format!("chunk_{:04}.bin", chunk.chunk_id));
            
            if verify {
                // Verify chunk integrity
                let computed_checksum = self.compute_checksum(&chunk.data);
                if computed_checksum != chunk.checksum {
                    return Err(anyhow!("Chunk {} integrity check failed", chunk.chunk_id));
                }
            }
            
            fs::write(&chunk_file, &chunk.data).await?;
            
            if self.verbose {
                println!("Downloaded chunk {} ({} bytes) -> {}", 
                        chunk.chunk_id, chunk.data.len(), chunk_file.display());
            }
        }
        
        println!("✅ Download completed successfully");
        Ok(())
    }
    
    async fn cancel_transfer(&self, session_id: Uuid) -> Result<()> {
        let url = format!("{}/api/v1/dark/sessions/{}/cancel", self.base_url, session_id);
        
        let mut req_builder = self.client.post(&url);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Cancel request failed: HTTP {}", response.status()));
        }
        
        println!("✅ Transfer session {} cancelled", session_id);
        Ok(())
    }
    
    async fn list_sessions(&self) -> Result<Vec<TransferSessionStatus>> {
        let url = format!("{}/api/v1/dark/sessions", self.base_url);
        
        let mut req_builder = self.client.get(&url);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Sessions request failed: HTTP {}", response.status()));
        }
        
        Ok(response.json().await?)
    }
    
    async fn set_access_policy(&self, asset_id: Uuid, policy: AccessPolicy) -> Result<()> {
        let url = format!("{}/api/v1/dark/policies/{}", self.base_url, asset_id);
        
        let mut req_builder = self.client.put(&url).json(&policy);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Set policy request failed: HTTP {}", response.status()));
        }
        
        println!("✅ Access policy set for asset {}", asset_id);
        Ok(())
    }
    
    async fn get_access_policy(&self, asset_id: Uuid) -> Result<AccessPolicy> {
        let url = format!("{}/api/v1/dark/policies/{}", self.base_url, asset_id);
        
        let mut req_builder = self.client.get(&url);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Get policy request failed: HTTP {}", response.status()));
        }
        
        Ok(response.json().await?)
    }
    
    async fn get_metrics(&self) -> Result<DarkServiceMetrics> {
        let url = format!("{}/api/v1/dark/metrics", self.base_url);
        
        let mut req_builder = self.client.get(&url);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Metrics request failed: HTTP {}", response.status()));
        }
        
        Ok(response.json().await?)
    }
    
    async fn ping(&self) -> Result<()> {
        let url = format!("{}/api/v1/dark/ping", self.base_url);
        
        let mut req_builder = self.client.get(&url);
        
        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let start = std::time::Instant::now();
        let response = req_builder.send().await?;
        let duration = start.elapsed();
        
        if response.status().is_success() {
            println!("✅ Dark services reachable ({}ms)", duration.as_millis());
            Ok(())
        } else {
            Err(anyhow!("Ping failed: HTTP {}", response.status()))
        }
    }
    
    fn compute_checksum(&self, data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    fn generate_secure_keys(&self, count: u32, length: usize, format: &str) -> Result<()> {
        use rand::RngCore;
        
        println!("Generating {} secure keys ({} bytes each):", count, length);
        
        for i in 0..count {
            let mut key = vec![0u8; length];
            rand::thread_rng().fill_bytes(&mut key);
            
            let formatted_key = match format {
                "hex" => hex::encode(&key),
                "base64" => base64::encode(&key),
                _ => return Err(anyhow!("Invalid format: {} (use 'hex' or 'base64')", format)),
            };
            
            println!("Key {}: {}", i + 1, formatted_key);
        }
        
        Ok(())
    }
}

fn format_output<T: Serialize>(data: &T, format: &str) -> Result<String> {
    match format {
        "json" => Ok(serde_json::to_string_pretty(data)?),
        "table" => {
            // For simplicity, just return JSON for now
            // In a real implementation, you'd format as a table
            Ok(serde_json::to_string_pretty(data)?)
        }
        _ => Err(anyhow!("Unsupported output format: {}", format)),
    }
}

fn parse_transfer_priority(priority: &str) -> Result<String> {
    match priority.to_lowercase().as_str() {
        "low" | "normal" | "high" | "critical" => Ok(priority.to_lowercase()),
        _ => Err(anyhow!("Invalid priority: {} (use low, normal, high, or critical)", priority)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let client = DarkServicesClient::new(
        cli.server,
        cli.token,
        cli.verbose,
        cli.insecure,
    )?;
    
    match cli.command {
        Commands::Transfer { 
            asset_id, 
            requester_id, 
            target_region, 
            security_level, 
            encryption, 
            compression, 
            priority 
        } => {
            let transfer_priority = parse_transfer_priority(&priority)?;
            let access_token = client.token.clone().unwrap_or_else(|| "default-token".to_string());
            
            let request = DarkTransferRequest {
                asset_id,
                requester_id,
                target_region,
                security_level,
                encryption_required: encryption,
                compression_preferred: compression,
                priority: transfer_priority,
                access_token,
            };
            
            let response = client.request_transfer(request).await?;
            println!("{}", format_output(&response, &cli.format)?);
        }
        
        Commands::Status { session_id } => {
            let status = client.get_session_status(session_id).await?;
            println!("{}", format_output(&status, &cli.format)?);
        }
        
        Commands::Download { session_id, output_dir, verify } => {
            client.download_chunks(session_id, &output_dir, verify).await?;
        }
        
        Commands::Cancel { session_id } => {
            client.cancel_transfer(session_id).await?;
        }
        
        Commands::Sessions => {
            let sessions = client.list_sessions().await?;
            println!("{}", format_output(&sessions, &cli.format)?);
        }
        
        Commands::SetPolicy { asset_id, policy_file } => {
            let policy_json = fs::read_to_string(&policy_file).await?;
            let policy: AccessPolicy = serde_json::from_str(&policy_json)?;
            client.set_access_policy(asset_id, policy).await?;
        }
        
        Commands::GetPolicy { asset_id } => {
            let policy = client.get_access_policy(asset_id).await?;
            println!("{}", format_output(&policy, &cli.format)?);
        }
        
        Commands::Metrics => {
            let metrics = client.get_metrics().await?;
            println!("{}", format_output(&metrics, &cli.format)?);
        }
        
        Commands::Ping => {
            client.ping().await?;
        }
        
        Commands::GenerateKeys { count, length, output_format } => {
            client.generate_secure_keys(count, length, &output_format)?;
        }
    }
    
    Ok(())
}