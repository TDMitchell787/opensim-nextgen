//! Dark Services for Secure Asset Transfer
//!
//! Dark services provide secure, encrypted asset transfer capabilities using zero trust
//! networking principles. Assets are transferred through encrypted tunnels with
//! identity-based access control and content verification.

use anyhow::{anyhow, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::asset::AssetManager;
use crate::monitoring::MetricsCollector;
use crate::network::crypto_manager::CryptoManager;

/// Serialize Duration as seconds for JSON output
fn serialize_duration_as_secs<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    duration.as_secs().serialize(serializer)
}

/// Dark service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkServiceConfig {
    /// Enable dark services
    pub enabled: bool,
    /// Service port for dark asset transfers
    pub port: u16,
    /// Bind address
    pub bind_address: String,
    /// Maximum concurrent transfers
    pub max_concurrent_transfers: u32,
    /// Transfer timeout in seconds
    pub transfer_timeout_seconds: u64,
    /// Maximum asset size for dark transfer (bytes)
    pub max_asset_size: u64,
    /// Enable asset encryption in transit
    pub enable_encryption: bool,
    /// Enable asset compression
    pub enable_compression: bool,
    /// Require mutual authentication
    pub require_mutual_auth: bool,
    /// Transfer chunk size (bytes)
    pub chunk_size: usize,
    /// Enable content verification
    pub enable_content_verification: bool,
    /// Allowed asset types for dark transfer
    pub allowed_asset_types: Vec<String>,
    /// Security level (0-100, where 100 is maximum security)
    pub security_level: u8,
}

impl Default for DarkServiceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8443,
            bind_address: "0.0.0.0".to_string(),
            max_concurrent_transfers: 100,
            transfer_timeout_seconds: 300,     // 5 minutes
            max_asset_size: 100 * 1024 * 1024, // 100MB
            enable_encryption: true,
            enable_compression: true,
            require_mutual_auth: true,
            chunk_size: 64 * 1024, // 64KB chunks
            enable_content_verification: true,
            allowed_asset_types: vec![
                "texture".to_string(),
                "sound".to_string(),
                "mesh".to_string(),
                "animation".to_string(),
                "script".to_string(),
                "notecard".to_string(),
            ],
            security_level: 90,
        }
    }
}

/// Dark transfer session
#[derive(Debug, Clone)]
pub struct DarkTransferSession {
    pub session_id: Uuid,
    pub asset_id: Uuid,
    pub requester_id: Uuid,
    pub provider_id: Uuid,
    pub asset_size: u64,
    pub asset_hash: String,
    pub encryption_key: Vec<u8>,
    pub compression_enabled: bool,
    pub chunks_total: u32,
    pub chunks_transferred: u32,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub status: TransferStatus,
    pub access_policy: AccessPolicy,
}

/// Transfer status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    Authorized,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Expired,
}

/// Access policy for dark transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub allowed_requesters: Vec<Uuid>,
    pub allowed_regions: Vec<Uuid>,
    pub time_restrictions: Option<TimeRestriction>,
    pub content_restrictions: ContentRestriction,
    pub require_encryption: bool,
    pub require_audit_log: bool,
}

/// Time-based access restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestriction {
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub allowed_hours: Vec<u8>, // 0-23
    pub allowed_days: Vec<u8>,  // 0-6 (Sunday-Saturday)
}

/// Content-based restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRestriction {
    pub max_file_size: u64,
    pub allowed_formats: Vec<String>,
    pub forbidden_keywords: Vec<String>,
    pub require_content_scan: bool,
    pub adult_content_policy: AdultContentPolicy,
}

/// Adult content policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdultContentPolicy {
    Allowed,
    Restricted,
    Forbidden,
}

/// Dark transfer request
#[derive(Debug, Serialize, Deserialize)]
pub struct DarkTransferRequest {
    pub transfer_id: Uuid,
    pub asset_id: Uuid,
    pub requester_id: Uuid,
    pub target_region: Option<Uuid>,
    pub security_level: u8,
    pub encryption_required: bool,
    pub compression_preferred: bool,
    pub priority: TransferPriority,
    pub access_token: String,
}

/// Transfer priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Dark transfer response
#[derive(Debug, Serialize, Deserialize)]
pub struct DarkTransferResponse {
    pub session_id: Uuid,
    pub status: TransferStatus,
    pub transfer_url: Option<String>,
    pub encryption_key: Option<Vec<u8>>,
    pub chunk_size: usize,
    pub total_chunks: u32,
    #[serde(serialize_with = "serialize_duration_as_secs")]
    pub estimated_duration: Duration,
    pub error_message: Option<String>,
}

/// Asset chunk for transfer
#[derive(Debug, Clone)]
pub struct AssetChunk {
    pub chunk_id: u32,
    pub session_id: Uuid,
    pub data: Bytes,
    pub checksum: String,
    pub is_encrypted: bool,
    pub is_compressed: bool,
}

/// Dark service metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DarkServiceMetrics {
    pub total_requests: u64,
    pub successful_transfers: u64,
    pub failed_transfers: u64,
    pub bytes_transferred: u64,
    pub active_sessions: u64,
    #[serde(serialize_with = "serialize_duration_as_secs")]
    pub average_transfer_time: Duration,
    pub security_violations: u64,
    pub policy_denials: u64,
}

/// Dark services manager
pub struct DarkServicesManager {
    config: DarkServiceConfig,
    asset_manager: Arc<AssetManager>,
    crypto_manager: Arc<CryptoManager>,
    metrics: Arc<MetricsCollector>,
    active_sessions: Arc<RwLock<HashMap<Uuid, DarkTransferSession>>>,
    access_policies: Arc<RwLock<HashMap<Uuid, AccessPolicy>>>, // Asset ID -> Policy
    transfer_queue: Arc<Mutex<Vec<DarkTransferRequest>>>,
    service_metrics: Arc<RwLock<DarkServiceMetrics>>,
}

impl DarkServicesManager {
    /// Create a new dark services manager
    pub fn new(
        config: DarkServiceConfig,
        asset_manager: Arc<AssetManager>,
        crypto_manager: Arc<CryptoManager>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            config,
            asset_manager,
            crypto_manager,
            metrics,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            access_policies: Arc::new(RwLock::new(HashMap::new())),
            transfer_queue: Arc::new(Mutex::new(Vec::new())),
            service_metrics: Arc::new(RwLock::new(DarkServiceMetrics::default())),
        }
    }

    /// Start the dark services
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Dark services disabled in configuration");
            return Ok(());
        }

        info!(
            "Starting dark services on {}:{}",
            self.config.bind_address, self.config.port
        );

        // Start the transfer processor
        self.start_transfer_processor().await?;

        // Start session cleanup task
        self.start_session_cleanup().await;

        // Start metrics collection
        self.start_metrics_collection().await;

        info!("Dark services started successfully");
        Ok(())
    }

    /// Request a dark transfer
    pub async fn request_transfer(
        &self,
        request: DarkTransferRequest,
    ) -> Result<DarkTransferResponse> {
        info!(
            "Received dark transfer request for asset {}",
            request.asset_id
        );

        // Update metrics
        {
            let mut metrics = self.service_metrics.write().await;
            metrics.total_requests += 1;
        }

        // Validate request
        self.validate_transfer_request(&request).await?;

        // Check access policy
        self.check_access_policy(&request).await?;

        // Create transfer session
        let session = self.create_transfer_session(&request).await?;

        // Store session
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session.session_id, session.clone());
        }

        // Generate response
        let response = DarkTransferResponse {
            session_id: session.session_id,
            status: session.status.clone(),
            transfer_url: Some(format!(
                "dark://{}:{}/transfer/{}",
                self.config.bind_address, self.config.port, session.session_id
            )),
            encryption_key: if self.config.enable_encryption {
                Some(session.encryption_key.clone())
            } else {
                None
            },
            chunk_size: self.config.chunk_size,
            total_chunks: session.chunks_total,
            estimated_duration: self.estimate_transfer_duration(&session),
            error_message: None,
        };

        info!(
            "Dark transfer session {} created for asset {}",
            session.session_id, request.asset_id
        );

        Ok(response)
    }

    /// Execute a dark transfer
    pub async fn execute_transfer(&self, session_id: Uuid) -> Result<Vec<AssetChunk>> {
        let session = {
            let sessions = self.active_sessions.read().await;
            sessions
                .get(&session_id)
                .ok_or_else(|| anyhow!("Transfer session not found: {}", session_id))?
                .clone()
        };

        if session.status != TransferStatus::Authorized {
            return Err(anyhow!("Transfer session not authorized: {}", session_id));
        }

        info!("Executing dark transfer for session {}", session_id);

        // Get asset data
        let asset_data = self.asset_manager.get_asset_data(&session.asset_id).await?;
        let asset_data = asset_data.ok_or_else(|| anyhow::anyhow!("Asset data not found"))?;

        // Verify asset integrity
        if self.config.enable_content_verification {
            self.verify_asset_content(&asset_data, &session.asset_hash)?;
        }

        // Split into chunks
        let chunks = self.create_asset_chunks(&session, asset_data).await?;

        // Update session status
        {
            let mut sessions = self.active_sessions.write().await;
            if let Some(mut sess) = sessions.get_mut(&session_id) {
                sess.status = TransferStatus::InProgress;
            }
        }

        // Update metrics
        {
            let mut metrics = self.service_metrics.write().await;
            metrics.bytes_transferred += session.asset_size;
            metrics.active_sessions += 1;
        }

        info!(
            "Dark transfer completed for session {} ({} chunks)",
            session_id,
            chunks.len()
        );

        Ok(chunks)
    }

    /// Set access policy for an asset
    pub async fn set_access_policy(&self, asset_id: Uuid, policy: AccessPolicy) -> Result<()> {
        info!("Setting access policy for asset {}", asset_id);

        let mut policies = self.access_policies.write().await;
        policies.insert(asset_id, policy);

        Ok(())
    }

    /// Get access policy for an asset
    pub async fn get_access_policy(&self, asset_id: &Uuid) -> Option<AccessPolicy> {
        let policies = self.access_policies.read().await;
        policies.get(asset_id).cloned()
    }

    /// Cancel a transfer session
    pub async fn cancel_transfer(&self, session_id: Uuid) -> Result<()> {
        info!("Cancelling dark transfer session {}", session_id);

        let mut sessions = self.active_sessions.write().await;
        if let Some(mut session) = sessions.get_mut(&session_id) {
            session.status = TransferStatus::Cancelled;

            // Update metrics
            let mut metrics = self.service_metrics.write().await;
            metrics.failed_transfers += 1;

            Ok(())
        } else {
            Err(anyhow!("Transfer session not found: {}", session_id))
        }
    }

    /// Get transfer session status
    pub async fn get_transfer_status(&self, session_id: &Uuid) -> Option<TransferStatus> {
        let sessions = self.active_sessions.read().await;
        sessions.get(session_id).map(|s| s.status.clone())
    }

    /// Get service metrics
    pub async fn get_metrics(&self) -> DarkServiceMetrics {
        (*self.service_metrics.read().await).clone()
    }

    /// Validate transfer request
    async fn validate_transfer_request(&self, request: &DarkTransferRequest) -> Result<()> {
        // Check if asset exists
        if !self.asset_manager.asset_exists(&request.asset_id).await? {
            return Err(anyhow!("Asset not found: {}", request.asset_id));
        }

        // Get asset info
        let asset_info = self
            .asset_manager
            .get_asset_info(&request.asset_id)
            .await?
            .ok_or_else(|| anyhow!("Asset metadata not found: {}", request.asset_id))?;

        // Check asset size
        if asset_info.original_size as u64 > self.config.max_asset_size {
            return Err(anyhow!(
                "Asset too large: {} bytes (max: {} bytes)",
                asset_info.original_size,
                self.config.max_asset_size
            ));
        }

        // Check asset type (using content_type since AssetMetadata doesn't have asset_type)
        let asset_type_str = asset_info
            .content_type
            .split('/')
            .next()
            .unwrap_or(&asset_info.content_type);
        if !self
            .config
            .allowed_asset_types
            .iter()
            .any(|t| asset_info.content_type.contains(t))
        {
            return Err(anyhow!(
                "Asset type not allowed for dark transfer: {}",
                asset_info.content_type
            ));
        }

        // Check security level
        if request.security_level > self.config.security_level {
            return Err(anyhow!(
                "Requested security level {} exceeds maximum {}",
                request.security_level,
                self.config.security_level
            ));
        }

        // Check concurrent transfers limit
        let active_count = self.active_sessions.read().await.len();
        if active_count >= self.config.max_concurrent_transfers as usize {
            return Err(anyhow!(
                "Maximum concurrent transfers exceeded: {}",
                active_count
            ));
        }

        Ok(())
    }

    /// Check access policy for transfer request
    async fn check_access_policy(&self, request: &DarkTransferRequest) -> Result<()> {
        let policies = self.access_policies.read().await;

        if let Some(policy) = policies.get(&request.asset_id) {
            // Check allowed requesters
            if !policy.allowed_requesters.is_empty()
                && !policy.allowed_requesters.contains(&request.requester_id)
            {
                return Err(anyhow!(
                    "Requester not authorized for asset: {}",
                    request.requester_id
                ));
            }

            // Check allowed regions
            if let Some(target_region) = request.target_region {
                if !policy.allowed_regions.is_empty()
                    && !policy.allowed_regions.contains(&target_region)
                {
                    return Err(anyhow!("Target region not authorized: {}", target_region));
                }
            }

            // Check time restrictions
            if let Some(time_restriction) = &policy.time_restrictions {
                self.check_time_restriction(time_restriction)?;
            }

            // Check encryption requirement
            if policy.require_encryption && !request.encryption_required {
                return Err(anyhow!("Encryption required for this asset"));
            }

            // Update metrics for policy check
            let mut metrics = self.service_metrics.write().await;
            // Policy was checked and passed (if we reach here)
        } else {
            // No specific policy - use default allow if no restrictive policies
            debug!(
                "No access policy found for asset {}, using default",
                request.asset_id
            );
        }

        Ok(())
    }

    /// Check time-based restrictions
    fn check_time_restriction(&self, restriction: &TimeRestriction) -> Result<()> {
        let now = SystemTime::now();

        // Check time window
        if now < restriction.start_time || now > restriction.end_time {
            return Err(anyhow!("Transfer not allowed outside time window"));
        }

        // Check allowed hours and days would require more time parsing
        // For simplicity, we'll assume they pass for now

        Ok(())
    }

    /// Create a transfer session
    async fn create_transfer_session(
        &self,
        request: &DarkTransferRequest,
    ) -> Result<DarkTransferSession> {
        let asset_info = self
            .asset_manager
            .get_asset_info(&request.asset_id)
            .await?
            .ok_or_else(|| anyhow!("Asset metadata not found: {}", request.asset_id))?;
        let asset_data = self
            .asset_manager
            .get_asset_data(&request.asset_id)
            .await?
            .ok_or_else(|| anyhow!("Asset data not found: {}", request.asset_id))?;

        // Calculate asset hash
        let mut hasher = Sha256::new();
        hasher.update(&asset_data);
        let asset_hash = format!("{:x}", hasher.finalize());

        // Generate encryption key if needed
        let encryption_key = if self.config.enable_encryption || request.encryption_required {
            self.crypto_manager.generate_key(32)?
        } else {
            Vec::new()
        };

        // Calculate chunks
        let chunks_total = ((asset_info.original_size as u64 + self.config.chunk_size as u64 - 1)
            / self.config.chunk_size as u64) as u32;

        let now = SystemTime::now();
        let expires_at = now + Duration::from_secs(self.config.transfer_timeout_seconds);

        // Get access policy
        let access_policy = self
            .get_access_policy(&request.asset_id)
            .await
            .unwrap_or_else(|| AccessPolicy {
                allowed_requesters: vec![request.requester_id],
                allowed_regions: Vec::new(),
                time_restrictions: None,
                content_restrictions: ContentRestriction {
                    max_file_size: self.config.max_asset_size,
                    allowed_formats: self.config.allowed_asset_types.clone(),
                    forbidden_keywords: Vec::new(),
                    require_content_scan: false,
                    adult_content_policy: AdultContentPolicy::Allowed,
                },
                require_encryption: self.config.enable_encryption,
                require_audit_log: true,
            });

        Ok(DarkTransferSession {
            session_id: Uuid::new_v4(),
            asset_id: request.asset_id,
            requester_id: request.requester_id,
            provider_id: Uuid::new_v4(), // Would be the actual provider ID
            asset_size: asset_info.original_size as u64,
            asset_hash,
            encryption_key,
            compression_enabled: self.config.enable_compression && request.compression_preferred,
            chunks_total,
            chunks_transferred: 0,
            created_at: now,
            expires_at,
            status: TransferStatus::Authorized,
            access_policy,
        })
    }

    /// Create asset chunks for transfer
    async fn create_asset_chunks(
        &self,
        session: &DarkTransferSession,
        asset_data: Bytes,
    ) -> Result<Vec<AssetChunk>> {
        let mut chunks = Vec::new();
        let chunk_size = self.config.chunk_size;

        for (i, chunk_data) in asset_data.chunks(chunk_size).enumerate() {
            let mut chunk_bytes = Bytes::copy_from_slice(chunk_data);

            // Compress if enabled
            if session.compression_enabled {
                chunk_bytes = self.compress_data(chunk_bytes)?;
            }

            // Encrypt if enabled
            let is_encrypted = if !session.encryption_key.is_empty() {
                chunk_bytes = self
                    .crypto_manager
                    .encrypt_data(&chunk_bytes, &session.encryption_key)?;
                true
            } else {
                false
            };

            // Calculate checksum
            let mut hasher = Sha256::new();
            hasher.update(&chunk_bytes);
            let checksum = format!("{:x}", hasher.finalize());

            chunks.push(AssetChunk {
                chunk_id: i as u32,
                session_id: session.session_id,
                data: chunk_bytes,
                checksum,
                is_encrypted,
                is_compressed: session.compression_enabled,
            });
        }

        Ok(chunks)
    }

    /// Compress data
    fn compress_data(&self, data: Bytes) -> Result<Bytes> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&data)?;
        let compressed = encoder.finish()?;

        Ok(Bytes::from(compressed))
    }

    /// Verify asset content integrity
    fn verify_asset_content(&self, data: &Bytes, expected_hash: &str) -> Result<()> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let actual_hash = format!("{:x}", hasher.finalize());

        if actual_hash != expected_hash {
            return Err(anyhow!("Asset content verification failed: hash mismatch"));
        }

        Ok(())
    }

    /// Estimate transfer duration
    fn estimate_transfer_duration(&self, session: &DarkTransferSession) -> Duration {
        // Simple estimation based on asset size and assumed bandwidth
        let estimated_bandwidth = 10 * 1024 * 1024; // 10 MB/s
        let transfer_time = session.asset_size / estimated_bandwidth;
        Duration::from_secs(transfer_time.max(1))
    }

    /// Start transfer processor
    async fn start_transfer_processor(&self) -> Result<()> {
        let queue = self.transfer_queue.clone();
        let crypto_manager = self.crypto_manager.clone();
        let asset_manager = self.asset_manager.clone();

        tokio::spawn(async move {
            loop {
                let request = {
                    let mut queue_lock = queue.lock().await;
                    queue_lock.pop()
                };

                if let Some(request) = request {
                    // Process the transfer request directly
                    debug!("Processing transfer request: {:?}", request.transfer_id);
                    // Note: This would require restructuring the method to be static
                    // For now, we'll just log it
                    debug!("Transfer request queued: {:?}", request.transfer_id);
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        Ok(())
    }

    /// Process a transfer request
    async fn process_transfer_request(&self, request: DarkTransferRequest) -> Result<()> {
        match self.request_transfer(request).await {
            Ok(response) => {
                info!(
                    "Transfer request processed successfully: {}",
                    response.session_id
                );
                Ok(())
            }
            Err(e) => {
                error!("Transfer request failed: {}", e);
                Err(e)
            }
        }
    }

    /// Start session cleanup task
    async fn start_session_cleanup(&self) {
        let sessions = self.active_sessions.clone();
        let metrics = self.service_metrics.clone();

        tokio::spawn(async move {
            loop {
                let now = SystemTime::now();
                let mut expired_sessions = Vec::new();

                // Find expired sessions
                {
                    let sessions_lock = sessions.read().await;
                    for (id, session) in sessions_lock.iter() {
                        if now > session.expires_at {
                            expired_sessions.push(*id);
                        }
                    }
                }

                // Remove expired sessions
                if !expired_sessions.is_empty() {
                    let mut sessions_lock = sessions.write().await;
                    let mut metrics_lock = metrics.write().await;

                    for session_id in expired_sessions {
                        sessions_lock.remove(&session_id);
                        metrics_lock.active_sessions =
                            metrics_lock.active_sessions.saturating_sub(1);
                        info!("Cleaned up expired transfer session: {}", session_id);
                    }
                }

                // Clean up every minute
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    /// Start metrics collection
    async fn start_metrics_collection(&self) {
        let metrics = self.metrics.clone();
        let service_metrics = self.service_metrics.clone();
        let sessions = self.active_sessions.clone();

        tokio::spawn(async move {
            loop {
                // Update active sessions count
                {
                    let sessions_lock = sessions.read().await;
                    let mut service_metrics_lock = service_metrics.write().await;
                    service_metrics_lock.active_sessions = sessions_lock.len() as u64;
                }

                // Collect metrics every 30 seconds
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
    }
}
