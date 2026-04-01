//! Security Manager for OpenSim Next
//!
//! Provides comprehensive security services including rate limiting, IP filtering,
//! intrusion detection, authentication, and security policy enforcement.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::database::{DatabaseConnection, UserAccountDatabase};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Maximum requests per minute per IP
    pub max_requests_per_minute: u32,
    /// Maximum requests per hour per IP
    pub max_requests_per_hour: u32,
    /// Enable IP filtering
    pub enable_ip_filtering: bool,
    /// Enable intrusion detection
    pub enable_intrusion_detection: bool,
    /// Maximum failed authentication attempts
    pub max_failed_auth_attempts: u32,
    /// Lockout duration in seconds
    pub lockout_duration_seconds: u64,
    /// Enable security logging
    pub enable_security_logging: bool,
    /// Enable automatic threat mitigation
    pub enable_auto_mitigation: bool,
    /// Whitelist of allowed IPs
    pub ip_whitelist: Vec<String>,
    /// Blacklist of blocked IPs
    pub ip_blacklist: Vec<String>,
    /// Require authentication for all requests
    pub require_authentication: bool,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Max UDP packets per second per IP before throttling
    pub max_udp_packets_per_second: u32,
    /// Max circuit auth failures before lockout
    pub circuit_failure_lockout: u32,
    /// Max UDP packets per minute per IP (env: OPENSIM_UDP_RATE_LIMIT)
    pub max_udp_packets_per_minute: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_rate_limiting: true,
            max_requests_per_minute: 100,
            max_requests_per_hour: 1000,
            enable_ip_filtering: true,
            enable_intrusion_detection: true,
            max_failed_auth_attempts: 5,
            lockout_duration_seconds: 300, // 5 minutes
            enable_security_logging: true,
            enable_auto_mitigation: true,
            ip_whitelist: vec!["127.0.0.1".to_string(), "::1".to_string()],
            ip_blacklist: Vec::new(),
            require_authentication: true,
            session_timeout_seconds: 3600, // 1 hour
            max_udp_packets_per_second: 500,
            circuit_failure_lockout: 3,
            max_udp_packets_per_minute: std::env::var("OPENSIM_UDP_RATE_LIMIT")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(30000),
        }
    }
}

/// Rate limiting information per IP
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub ip: IpAddr,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub last_request_time: Instant,
    pub minute_reset_time: Instant,
    pub hour_reset_time: Instant,
    pub is_rate_limited: bool,
    pub rate_limit_expires: Option<Instant>,
}

/// Security threat information
#[derive(Debug, Clone)]
pub struct SecurityThreat {
    pub threat_id: Uuid,
    pub ip: IpAddr,
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub detected_at: SystemTime,
    pub description: String,
    pub mitigation_action: Option<MitigationAction>,
    pub resolved: bool,
}

/// Types of security threats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    BruteForceAttack,
    RateLimitExceeded,
    SuspiciousActivity,
    MalformedRequest,
    UnauthorizedAccess,
    DenialOfService,
    DataExfiltration,
    SQLInjection,
    XSSAttempt,
    PathTraversal,
}

/// Threat severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Mitigation actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitigationAction {
    Block,
    RateLimit,
    Monitor,
    Alert,
    Quarantine,
}

/// Authentication session
#[derive(Debug, Clone)]
pub struct SecuritySession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub ip: IpAddr,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub last_activity: SystemTime,
    pub permissions: Vec<String>,
    pub is_active: bool,
}

/// Failed authentication attempt
#[derive(Debug, Clone)]
pub struct FailedAuthAttempt {
    pub ip: IpAddr,
    pub user_id: Option<Uuid>,
    pub attempt_time: SystemTime,
    pub failure_reason: String,
}

/// IP lockout information
#[derive(Debug, Clone)]
pub struct IpLockout {
    pub ip: IpAddr,
    pub locked_at: SystemTime,
    pub lockout_expires: SystemTime,
    pub failed_attempts: u32,
    pub reason: String,
}

/// Security statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub rate_limited_requests: u64,
    pub failed_auth_attempts: u64,
    pub active_sessions: u64,
    pub detected_threats: u64,
    pub mitigated_threats: u64,
    pub locked_ips: u64,
}

/// UDP security statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpSecurityStats {
    pub total_packets: u64,
    pub total_dropped: u64,
    pub tracked_ips: usize,
    pub blocked_ips: usize,
}

/// Fast-path UDP packet counter per IP (lock-free)
pub struct UdpIpCounter {
    pub packets_this_second: AtomicU64,
    pub packets_this_minute: AtomicU64,
    pub second_epoch: AtomicU64,
    pub minute_epoch: AtomicU64,
    pub is_blocked: AtomicBool,
    pub circuit_failures: AtomicU64,
}

impl UdpIpCounter {
    fn new(now_secs: u64) -> Self {
        Self {
            packets_this_second: AtomicU64::new(0),
            packets_this_minute: AtomicU64::new(0),
            second_epoch: AtomicU64::new(now_secs),
            minute_epoch: AtomicU64::new(now_secs),
            is_blocked: AtomicBool::new(false),
            circuit_failures: AtomicU64::new(0),
        }
    }
}

/// Security manager
pub struct SecurityManager {
    config: SecurityConfig,
    rate_limits: Arc<RwLock<HashMap<IpAddr, RateLimitInfo>>>,
    security_sessions: Arc<RwLock<HashMap<Uuid, SecuritySession>>>,
    failed_auth_attempts: Arc<RwLock<HashMap<IpAddr, Vec<FailedAuthAttempt>>>>,
    ip_lockouts: Arc<RwLock<HashMap<IpAddr, IpLockout>>>,
    detected_threats: Arc<RwLock<Vec<SecurityThreat>>>,
    security_stats: Arc<RwLock<SecurityStats>>,
    user_database: Option<Arc<UserAccountDatabase>>,
    db_connection: Option<Arc<DatabaseConnection>>,
    udp_counters: Arc<DashMap<IpAddr, UdpIpCounter>>,
    udp_blocked_ips: Arc<DashMap<IpAddr, Instant>>,
    total_udp_packets: AtomicU64,
    total_udp_dropped: AtomicU64,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Result<Self> {
        let config = SecurityConfig::default();

        let manager = Self {
            config,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            security_sessions: Arc::new(RwLock::new(HashMap::new())),
            failed_auth_attempts: Arc::new(RwLock::new(HashMap::new())),
            ip_lockouts: Arc::new(RwLock::new(HashMap::new())),
            detected_threats: Arc::new(RwLock::new(Vec::new())),
            security_stats: Arc::new(RwLock::new(SecurityStats {
                total_requests: 0,
                blocked_requests: 0,
                rate_limited_requests: 0,
                failed_auth_attempts: 0,
                active_sessions: 0,
                detected_threats: 0,
                mitigated_threats: 0,
                locked_ips: 0,
            })),
            user_database: None,
            db_connection: None,
            udp_counters: Arc::new(DashMap::new()),
            udp_blocked_ips: Arc::new(DashMap::new()),
            total_udp_packets: AtomicU64::new(0),
            total_udp_dropped: AtomicU64::new(0),
        };

        info!("SecurityManager initialized with default configuration");
        Ok(manager)
    }

    /// Create security manager with custom configuration
    pub fn with_config(config: SecurityConfig) -> Result<Self> {
        let manager = Self {
            config,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            security_sessions: Arc::new(RwLock::new(HashMap::new())),
            failed_auth_attempts: Arc::new(RwLock::new(HashMap::new())),
            ip_lockouts: Arc::new(RwLock::new(HashMap::new())),
            detected_threats: Arc::new(RwLock::new(Vec::new())),
            security_stats: Arc::new(RwLock::new(SecurityStats {
                total_requests: 0,
                blocked_requests: 0,
                rate_limited_requests: 0,
                failed_auth_attempts: 0,
                active_sessions: 0,
                detected_threats: 0,
                mitigated_threats: 0,
                locked_ips: 0,
            })),
            user_database: None,
            db_connection: None,
            udp_counters: Arc::new(DashMap::new()),
            udp_blocked_ips: Arc::new(DashMap::new()),
            total_udp_packets: AtomicU64::new(0),
            total_udp_dropped: AtomicU64::new(0),
        };

        info!("SecurityManager initialized with custom configuration");
        Ok(manager)
    }

    /// Set the database connection for authentication
    pub fn set_database(&mut self, connection: Arc<DatabaseConnection>) {
        self.db_connection = Some(connection);
        info!("SecurityManager database connection configured");
    }

    /// Set the user database for authentication
    pub fn set_user_database(&mut self, user_db: Arc<UserAccountDatabase>) {
        self.user_database = Some(user_db);
        info!("SecurityManager user database configured");
    }

    /// Create security manager with database connection
    pub async fn with_database(config: SecurityConfig, connection: Arc<DatabaseConnection>) -> Result<Self> {
        let user_database = Arc::new(UserAccountDatabase::new(connection.clone()).await?);

        let manager = Self {
            config,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            security_sessions: Arc::new(RwLock::new(HashMap::new())),
            failed_auth_attempts: Arc::new(RwLock::new(HashMap::new())),
            ip_lockouts: Arc::new(RwLock::new(HashMap::new())),
            detected_threats: Arc::new(RwLock::new(Vec::new())),
            security_stats: Arc::new(RwLock::new(SecurityStats {
                total_requests: 0,
                blocked_requests: 0,
                rate_limited_requests: 0,
                failed_auth_attempts: 0,
                active_sessions: 0,
                detected_threats: 0,
                mitigated_threats: 0,
                locked_ips: 0,
            })),
            user_database: Some(user_database),
            db_connection: Some(connection),
            udp_counters: Arc::new(DashMap::new()),
            udp_blocked_ips: Arc::new(DashMap::new()),
            total_udp_packets: AtomicU64::new(0),
            total_udp_dropped: AtomicU64::new(0),
        };

        info!("SecurityManager initialized with database connection");
        Ok(manager)
    }
    
    /// Check if a request is allowed from the given IP
    pub async fn check_request_allowed(&self, ip: IpAddr) -> Result<bool> {
        // Update total requests counter
        {
            let mut stats = self.security_stats.write().await;
            stats.total_requests += 1;
        }
        
        // Check IP lockout
        if self.is_ip_locked(ip).await? {
            let mut stats = self.security_stats.write().await;
            stats.blocked_requests += 1;
            return Ok(false);
        }
        
        // Check IP filtering
        if self.config.enable_ip_filtering && !self.is_ip_allowed(ip).await? {
            let mut stats = self.security_stats.write().await;
            stats.blocked_requests += 1;
            return Ok(false);
        }
        
        // Check rate limiting
        if self.config.enable_rate_limiting && !self.check_rate_limit(ip).await? {
            let mut stats = self.security_stats.write().await;
            stats.rate_limited_requests += 1;
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Check if IP is locked out
    pub async fn is_ip_locked(&self, ip: IpAddr) -> Result<bool> {
        let lockouts = self.ip_lockouts.read().await;
        
        if let Some(lockout) = lockouts.get(&ip) {
            let now = SystemTime::now();
            if now < lockout.lockout_expires {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Check if IP is allowed (whitelist/blacklist)
    pub async fn is_ip_allowed(&self, ip: IpAddr) -> Result<bool> {
        let ip_str = ip.to_string();
        
        // Check blacklist first
        if self.config.ip_blacklist.contains(&ip_str) {
            return Ok(false);
        }
        
        // If whitelist is not empty, check if IP is whitelisted
        if !self.config.ip_whitelist.is_empty() {
            return Ok(self.config.ip_whitelist.contains(&ip_str));
        }
        
        // If no whitelist, allow by default (unless blacklisted)
        Ok(true)
    }
    
    /// Check rate limit for IP
    pub async fn check_rate_limit(&self, ip: IpAddr) -> Result<bool> {
        let now = Instant::now();
        let mut rate_limits = self.rate_limits.write().await;
        
        let rate_info = rate_limits.entry(ip).or_insert(RateLimitInfo {
            ip,
            requests_per_minute: 0,
            requests_per_hour: 0,
            last_request_time: now,
            minute_reset_time: now + Duration::from_secs(60),
            hour_reset_time: now + Duration::from_secs(3600),
            is_rate_limited: false,
            rate_limit_expires: None,
        });
        
        // Reset counters if time windows have passed
        if now > rate_info.minute_reset_time {
            rate_info.requests_per_minute = 0;
            rate_info.minute_reset_time = now + Duration::from_secs(60);
        }
        
        if now > rate_info.hour_reset_time {
            rate_info.requests_per_hour = 0;
            rate_info.hour_reset_time = now + Duration::from_secs(3600);
        }
        
        // Check if rate limit has expired
        if let Some(expires) = rate_info.rate_limit_expires {
            if now > expires {
                rate_info.is_rate_limited = false;
                rate_info.rate_limit_expires = None;
            }
        }
        
        // Check current rate limits
        if rate_info.requests_per_minute >= self.config.max_requests_per_minute {
            rate_info.is_rate_limited = true;
            rate_info.rate_limit_expires = Some(rate_info.minute_reset_time);
            
            // Log rate limit violation
            warn!("Rate limit exceeded for IP {}: {} requests per minute", ip, rate_info.requests_per_minute);
            
            // Detect as potential threat
            self.detect_threat(ip, ThreatType::RateLimitExceeded, ThreatSeverity::Medium,
                             format!("IP {} exceeded rate limit: {} requests per minute", ip, rate_info.requests_per_minute)).await?;
            
            return Ok(false);
        }
        
        if rate_info.requests_per_hour >= self.config.max_requests_per_hour {
            rate_info.is_rate_limited = true;
            rate_info.rate_limit_expires = Some(rate_info.hour_reset_time);
            
            warn!("Hourly rate limit exceeded for IP {}: {} requests per hour", ip, rate_info.requests_per_hour);
            
            self.detect_threat(ip, ThreatType::RateLimitExceeded, ThreatSeverity::High,
                             format!("IP {} exceeded hourly rate limit: {} requests per hour", ip, rate_info.requests_per_hour)).await?;
            
            return Ok(false);
        }
        
        // Update counters
        rate_info.requests_per_minute += 1;
        rate_info.requests_per_hour += 1;
        rate_info.last_request_time = now;
        
        Ok(true)
    }
    
    /// Authenticate a user with username and password
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<String> {
        if username.is_empty() || password.is_empty() {
            return Err(anyhow!("Username and password cannot be empty"));
        }

        if let Some(ref user_db) = self.user_database {
            if username.contains(' ') {
                let parts: Vec<&str> = username.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let first_name = parts[0];
                    let last_name = parts[1];

                    match user_db.authenticate_user_opensim(first_name, last_name, password).await {
                        Ok(Some(user)) => {
                            let token = self.generate_auth_token(&user.id);
                            debug!("Authenticated user {} {} via database", first_name, last_name);
                            return Ok(token);
                        }
                        Ok(None) => {
                            debug!("Authentication failed for {} {}: invalid credentials", first_name, last_name);
                            return Err(anyhow!("Invalid username or password"));
                        }
                        Err(e) => {
                            warn!("Database authentication error: {}", e);
                        }
                    }
                }
            }

            match user_db.authenticate_user(username, password).await {
                Ok(Some(user)) => {
                    let token = self.generate_auth_token(&user.id);
                    debug!("Authenticated user {} via database", username);
                    return Ok(token);
                }
                Ok(None) => {
                    debug!("Authentication failed for {}: invalid credentials", username);
                    return Err(anyhow!("Invalid username or password"));
                }
                Err(e) => {
                    warn!("Database authentication error for {}: {}", username, e);
                }
            }
        }

        if let Some(ref db_conn) = self.db_connection {
            match self.authenticate_via_direct_query(db_conn, username, password).await {
                Ok(token) => return Ok(token),
                Err(e) => {
                    debug!("Direct database authentication failed: {}", e);
                }
            }
        }

        Err(anyhow!("Invalid username or password"))
    }

    /// Generate a secure authentication token
    fn generate_auth_token(&self, user_id: &Uuid) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();

        let mut hasher = Sha256::new();
        hasher.update(user_id.as_bytes());
        hasher.update(&random_bytes);
        hasher.update(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .to_le_bytes(),
        );
        let hash = hasher.finalize();

        format!("auth_{:x}", hash)
    }

    /// Authenticate via direct database query
    async fn authenticate_via_direct_query(
        &self,
        connection: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> Result<String> {
        let (first_name, last_name) = if username.contains(' ') {
            let parts: Vec<&str> = username.splitn(2, ' ').collect();
            if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                (username, "Resident")
            }
        } else {
            (username, "Resident")
        };

        match connection {
            DatabaseConnection::PostgreSQL(pool) => {
                let row = sqlx::query(
                    "SELECT ua.principalid, a.passwordhash, a.passwordsalt
                     FROM useraccounts ua
                     JOIN auth a ON ua.principalid = a.uuid
                     WHERE LOWER(ua.firstname) = LOWER($1) AND LOWER(ua.lastname) = LOWER($2)",
                )
                .bind(first_name)
                .bind(last_name)
                .fetch_optional(pool)
                .await?;

                if let Some(row) = row {
                    use sqlx::Row;
                    let user_id: Uuid = row.try_get("principalid")?;
                    let stored_hash: String = row.try_get("passwordhash")?;

                    let viewer_hash = if password.starts_with("$1$") {
                        password.strip_prefix("$1$").unwrap_or(password)
                    } else {
                        password
                    };

                    if viewer_hash == stored_hash {
                        return Ok(self.generate_auth_token(&user_id));
                    }
                }
                Err(anyhow!("Authentication failed"))
            }
            DatabaseConnection::MySQL(pool) => {
                let row = sqlx::query(
                    "SELECT ua.PrincipalID, a.passwordHash, a.passwordSalt
                     FROM UserAccounts ua
                     JOIN auth a ON ua.PrincipalID = a.UUID
                     WHERE LOWER(ua.FirstName) = LOWER(?) AND LOWER(ua.LastName) = LOWER(?)",
                )
                .bind(first_name)
                .bind(last_name)
                .fetch_optional(pool)
                .await?;

                if let Some(row) = row {
                    use sqlx::Row;
                    let user_id_str: String = row.try_get("PrincipalID")?;
                    let user_id = Uuid::parse_str(&user_id_str)?;
                    let stored_hash: String = row.try_get("passwordHash")?;

                    let viewer_hash = if password.starts_with("$1$") {
                        password.strip_prefix("$1$").unwrap_or(password)
                    } else {
                        password
                    };

                    if viewer_hash == stored_hash {
                        return Ok(self.generate_auth_token(&user_id));
                    }
                }
                Err(anyhow!("Authentication failed"))
            }
        }
    }

    /// Record a failed authentication attempt
    pub async fn record_failed_auth(&self, ip: IpAddr, user_id: Option<Uuid>, reason: String) -> Result<()> {
        let now = SystemTime::now();
        let mut failed_attempts = self.failed_auth_attempts.write().await;
        
        let attempts = failed_attempts.entry(ip).or_insert_with(Vec::new);
        attempts.push(FailedAuthAttempt {
            ip,
            user_id,
            attempt_time: now,
            failure_reason: reason.clone(),
        });
        
        // Clean up old attempts (older than 1 hour)
        let cutoff_time = now - Duration::from_secs(3600);
        attempts.retain(|attempt| attempt.attempt_time > cutoff_time);
        
        // Check if we need to lock the IP
        if attempts.len() >= self.config.max_failed_auth_attempts as usize {
            self.lock_ip(ip, format!("Too many failed authentication attempts: {}", attempts.len())).await?;
            
            // Clear attempts after locking
            attempts.clear();
            
            // Detect as brute force attack
            self.detect_threat(ip, ThreatType::BruteForceAttack, ThreatSeverity::High,
                             format!("Brute force attack detected from IP {}: {} failed attempts", ip, attempts.len())).await?;
        }
        
        // Update statistics
        let mut stats = self.security_stats.write().await;
        stats.failed_auth_attempts += 1;
        
        info!("Recorded failed authentication attempt from IP {}: {}", ip, reason);
        Ok(())
    }
    
    /// Lock an IP address
    pub async fn lock_ip(&self, ip: IpAddr, reason: String) -> Result<()> {
        let now = SystemTime::now();
        let lockout_expires = now + Duration::from_secs(self.config.lockout_duration_seconds);
        
        let lockout = IpLockout {
            ip,
            locked_at: now,
            lockout_expires,
            failed_attempts: 0,
            reason: reason.clone(),
        };
        
        let mut lockouts = self.ip_lockouts.write().await;
        lockouts.insert(ip, lockout);
        
        // Update statistics
        let mut stats = self.security_stats.write().await;
        stats.locked_ips += 1;
        
        warn!("Locked IP {} for {} seconds: {}", ip, self.config.lockout_duration_seconds, reason);
        Ok(())
    }
    
    /// Unlock an IP address
    pub async fn unlock_ip(&self, ip: IpAddr) -> Result<bool> {
        let mut lockouts = self.ip_lockouts.write().await;
        
        if lockouts.remove(&ip).is_some() {
            let mut stats = self.security_stats.write().await;
            stats.locked_ips = stats.locked_ips.saturating_sub(1);
            
            info!("Unlocked IP {}", ip);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Create a new security session
    pub async fn create_session(&self, user_id: Uuid, ip: IpAddr, permissions: Vec<String>) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let now = SystemTime::now();
        let expires_at = now + Duration::from_secs(self.config.session_timeout_seconds);
        
        let session = SecuritySession {
            session_id,
            user_id,
            ip,
            created_at: now,
            expires_at,
            last_activity: now,
            permissions,
            is_active: true,
        };
        
        let mut sessions = self.security_sessions.write().await;
        sessions.insert(session_id, session);
        
        // Update statistics
        let mut stats = self.security_stats.write().await;
        stats.active_sessions += 1;
        
        debug!("Created security session {} for user {} from IP {}", session_id, user_id, ip);
        Ok(session_id)
    }
    
    /// Validate a security session
    pub async fn validate_session(&self, session_id: &Uuid) -> Result<bool> {
        let mut sessions = self.security_sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            let now = SystemTime::now();
            
            // Check if session has expired
            if now > session.expires_at {
                session.is_active = false;
                return Ok(false);
            }
            
            // Update last activity
            session.last_activity = now;
            
            Ok(session.is_active)
        } else {
            Ok(false)
        }
    }
    
    /// Invalidate a security session
    pub async fn invalidate_session(&self, session_id: &Uuid) -> Result<bool> {
        let mut sessions = self.security_sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.is_active = false;
            
            // Update statistics
            let mut stats = self.security_stats.write().await;
            stats.active_sessions = stats.active_sessions.saturating_sub(1);
            
            debug!("Invalidated security session {}", session_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Detect a security threat
    pub async fn detect_threat(&self, ip: IpAddr, threat_type: ThreatType, severity: ThreatSeverity, description: String) -> Result<()> {
        let threat_id = Uuid::new_v4();
        let now = SystemTime::now();
        
        let threat = SecurityThreat {
            threat_id,
            ip,
            threat_type: threat_type.clone(),
            severity: severity.clone(),
            detected_at: now,
            description: description.clone(),
            mitigation_action: None,
            resolved: false,
        };
        
        let mut threats = self.detected_threats.write().await;
        threats.push(threat);
        
        // Update statistics
        let mut stats = self.security_stats.write().await;
        stats.detected_threats += 1;
        
        // Auto-mitigation if enabled
        if self.config.enable_auto_mitigation {
            let mitigation_action = self.determine_mitigation_action(&threat_type, &severity);
            self.apply_mitigation(ip, &mitigation_action).await?;
            
            // Update the threat with mitigation action
            if let Some(threat) = threats.last_mut() {
                threat.mitigation_action = Some(mitigation_action);
            }
            
            stats.mitigated_threats += 1;
        }
        
        error!("Security threat detected: {:?} from IP {} - {}", threat_type, ip, description);
        Ok(())
    }
    
    /// Determine appropriate mitigation action for a threat
    fn determine_mitigation_action(&self, threat_type: &ThreatType, severity: &ThreatSeverity) -> MitigationAction {
        match (threat_type, severity) {
            (ThreatType::BruteForceAttack, ThreatSeverity::High | ThreatSeverity::Critical) => MitigationAction::Block,
            (ThreatType::DenialOfService, _) => MitigationAction::Block,
            (ThreatType::RateLimitExceeded, ThreatSeverity::High | ThreatSeverity::Critical) => MitigationAction::RateLimit,
            (ThreatType::UnauthorizedAccess, ThreatSeverity::High | ThreatSeverity::Critical) => MitigationAction::Block,
            (ThreatType::DataExfiltration, _) => MitigationAction::Block,
            (ThreatType::SQLInjection | ThreatType::XSSAttempt | ThreatType::PathTraversal, _) => MitigationAction::Block,
            _ => MitigationAction::Monitor,
        }
    }
    
    /// Apply mitigation action
    async fn apply_mitigation(&self, ip: IpAddr, action: &MitigationAction) -> Result<()> {
        match action {
            MitigationAction::Block => {
                self.lock_ip(ip, "Automatic threat mitigation".to_string()).await?;
                info!("Applied mitigation: Blocked IP {}", ip);
            }
            MitigationAction::RateLimit => {
                // Rate limiting is already handled in check_rate_limit
                info!("Applied mitigation: Rate limited IP {}", ip);
            }
            MitigationAction::Monitor => {
                info!("Applied mitigation: Monitoring IP {}", ip);
            }
            MitigationAction::Alert => {
                info!("Applied mitigation: Alert generated for IP {}", ip);
            }
            MitigationAction::Quarantine => {
                // For now, treat quarantine as a block
                self.lock_ip(ip, "Quarantine - threat mitigation".to_string()).await?;
                info!("Applied mitigation: Quarantined IP {}", ip);
            }
        }
        
        Ok(())
    }
    
    /// Get security statistics
    pub async fn get_security_stats(&self) -> SecurityStats {
        let stats = self.security_stats.read().await;
        stats.clone()
    }
    
    /// Get active security sessions
    pub async fn get_active_sessions(&self) -> Vec<SecuritySession> {
        let sessions = self.security_sessions.read().await;
        sessions.values().filter(|s| s.is_active).cloned().collect()
    }
    
    /// Get detected threats
    pub async fn get_detected_threats(&self) -> Vec<SecurityThreat> {
        let threats = self.detected_threats.read().await;
        threats.clone()
    }
    
    /// Fast-path check for UDP packets — lock-free, sub-microsecond.
    /// Returns true if the packet should be processed, false if dropped.
    pub fn check_udp_packet_allowed(&self, ip: IpAddr) -> bool {
        self.total_udp_packets.fetch_add(1, Ordering::Relaxed);

        if let Some(blocked_until) = self.udp_blocked_ips.get(&ip) {
            if Instant::now() < *blocked_until.value() {
                self.total_udp_dropped.fetch_add(1, Ordering::Relaxed);
                return false;
            } else {
                self.udp_blocked_ips.remove(&ip);
            }
        }

        if self.config.ip_whitelist.contains(&ip.to_string()) {
            return true;
        }
        if self.config.ip_blacklist.contains(&ip.to_string()) {
            self.total_udp_dropped.fetch_add(1, Ordering::Relaxed);
            return false;
        }

        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let counter = self.udp_counters
            .entry(ip)
            .or_insert_with(|| UdpIpCounter::new(now_secs));

        let stored_sec = counter.second_epoch.load(Ordering::Relaxed);
        if now_secs != stored_sec {
            counter.packets_this_second.store(0, Ordering::Relaxed);
            counter.second_epoch.store(now_secs, Ordering::Relaxed);
        }

        let stored_min = counter.minute_epoch.load(Ordering::Relaxed);
        if now_secs - stored_min >= 60 {
            counter.packets_this_minute.store(0, Ordering::Relaxed);
            counter.minute_epoch.store(now_secs, Ordering::Relaxed);
        }

        let pps = counter.packets_this_second.fetch_add(1, Ordering::Relaxed) + 1;
        let ppm = counter.packets_this_minute.fetch_add(1, Ordering::Relaxed) + 1;

        if pps > self.config.max_udp_packets_per_second as u64 {
            self.total_udp_dropped.fetch_add(1, Ordering::Relaxed);
            if !counter.is_blocked.swap(true, Ordering::Relaxed) {
                warn!("[SECURITY] UDP rate limit: {} sent {} pps (limit {})",
                      ip, pps, self.config.max_udp_packets_per_second);
                self.udp_blocked_ips.insert(ip, Instant::now() + Duration::from_secs(10));
            }
            return false;
        }

        if ppm > self.config.max_udp_packets_per_minute as u64 {
            self.total_udp_dropped.fetch_add(1, Ordering::Relaxed);
            if !counter.is_blocked.swap(true, Ordering::Relaxed) {
                warn!("[SECURITY] UDP minute rate limit: {} sent {} ppm (limit {})",
                      ip, ppm, self.config.max_udp_packets_per_minute);
                self.udp_blocked_ips.insert(ip, Instant::now() + Duration::from_secs(60));
            }
            return false;
        }

        if counter.is_blocked.load(Ordering::Relaxed) {
            counter.is_blocked.store(false, Ordering::Relaxed);
        }

        true
    }

    /// Record a circuit authentication failure from an IP
    pub fn record_circuit_failure(&self, ip: IpAddr) {
        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let counter = self.udp_counters
            .entry(ip)
            .or_insert_with(|| UdpIpCounter::new(now_secs));

        let failures = counter.circuit_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= self.config.circuit_failure_lockout as u64 {
            warn!("[SECURITY] Circuit brute-force: {} had {} failures, blocking for 5 min", ip, failures);
            self.udp_blocked_ips.insert(ip, Instant::now() + Duration::from_secs(300));
            counter.circuit_failures.store(0, Ordering::Relaxed);
        }
    }

    /// Get UDP security statistics
    pub fn get_udp_stats(&self) -> UdpSecurityStats {
        UdpSecurityStats {
            total_packets: self.total_udp_packets.load(Ordering::Relaxed),
            total_dropped: self.total_udp_dropped.load(Ordering::Relaxed),
            tracked_ips: self.udp_counters.len(),
            blocked_ips: self.udp_blocked_ips.len(),
        }
    }

    /// Add an IP to the blacklist (runtime)
    pub fn add_to_blacklist(&self, ip: IpAddr) {
        self.udp_blocked_ips.insert(ip, Instant::now() + Duration::from_secs(86400));
        warn!("[SECURITY] Manually blacklisted IP {} for 24h", ip);
    }

    /// Remove an IP from the blocked list (runtime)
    pub fn remove_from_blocked(&self, ip: IpAddr) -> bool {
        self.udp_blocked_ips.remove(&ip).is_some()
    }

    /// Get list of currently locked/blocked IPs
    pub fn get_blocked_ip_list(&self) -> Vec<(IpAddr, Instant)> {
        self.udp_blocked_ips.iter()
            .map(|entry| (*entry.key(), *entry.value()))
            .collect()
    }

    /// Cleanup expired UDP counters and blocks
    pub fn cleanup_udp_data(&self) {
        let now = Instant::now();
        self.udp_blocked_ips.retain(|_, expires| now < *expires);

        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.udp_counters.retain(|_, counter| {
            now_secs - counter.minute_epoch.load(Ordering::Relaxed) < 300
        });
    }

    /// Clean up expired sessions and lockouts
    pub async fn cleanup_expired_data(&self) -> Result<()> {
        let now = SystemTime::now();
        let mut cleanup_count = 0;
        
        // Clean up expired sessions
        {
            let mut sessions = self.security_sessions.write().await;
            let initial_count = sessions.len();
            sessions.retain(|_, session| {
                if now > session.expires_at {
                    false
                } else {
                    true
                }
            });
            cleanup_count += initial_count - sessions.len();
        }
        
        // Clean up expired lockouts
        {
            let mut lockouts = self.ip_lockouts.write().await;
            let initial_count = lockouts.len();
            lockouts.retain(|_, lockout| now < lockout.lockout_expires);
            cleanup_count += initial_count - lockouts.len();
        }
        
        // Clean up old failed attempts
        {
            let mut failed_attempts = self.failed_auth_attempts.write().await;
            let cutoff_time = now - Duration::from_secs(3600); // 1 hour
            
            for attempts in failed_attempts.values_mut() {
                let initial_count = attempts.len();
                attempts.retain(|attempt| attempt.attempt_time > cutoff_time);
                cleanup_count += initial_count - attempts.len();
            }
            
            // Remove empty entries
            failed_attempts.retain(|_, attempts| !attempts.is_empty());
        }
        
        if cleanup_count > 0 {
            debug!("Cleaned up {} expired security records", cleanup_count);
        }
        
        Ok(())
    }
    
    /// Start automatic cleanup task
    pub async fn start_cleanup_task(&self) -> Result<()> {
        let rate_limits = self.rate_limits.clone();
        let security_sessions = self.security_sessions.clone();
        let failed_auth_attempts = self.failed_auth_attempts.clone();
        let ip_lockouts = self.ip_lockouts.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                let now = SystemTime::now();
                let now_instant = Instant::now();
                
                // Clean up expired sessions
                {
                    let mut sessions = security_sessions.write().await;
                    sessions.retain(|_, session| now <= session.expires_at);
                }
                
                // Clean up expired lockouts
                {
                    let mut lockouts = ip_lockouts.write().await;
                    lockouts.retain(|_, lockout| now < lockout.lockout_expires);
                }
                
                // Clean up old failed attempts
                {
                    let mut failed_attempts = failed_auth_attempts.write().await;
                    let cutoff_time = now - Duration::from_secs(3600);
                    
                    for attempts in failed_attempts.values_mut() {
                        attempts.retain(|attempt| attempt.attempt_time > cutoff_time);
                    }
                    
                    failed_attempts.retain(|_, attempts| !attempts.is_empty());
                }
                
                // Clean up old rate limit entries
                {
                    let mut rate_limits = rate_limits.write().await;
                    rate_limits.retain(|_, info| {
                        // Keep entries that are still within the time window or are rate limited
                        now_instant < info.hour_reset_time || info.is_rate_limited
                    });
                }
            }
        });
        
        info!("Started automatic security cleanup task");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[tokio::test]
    async fn test_security_manager_creation() {
        let manager = SecurityManager::new().unwrap();
        let stats = manager.get_security_stats().await;
        
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.blocked_requests, 0);
        assert_eq!(stats.active_sessions, 0);
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let mut config = SecurityConfig::default();
        config.max_requests_per_minute = 2;
        
        let manager = SecurityManager::with_config(config).unwrap();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        
        // First two requests should be allowed
        assert!(manager.check_rate_limit(ip).await.unwrap());
        assert!(manager.check_rate_limit(ip).await.unwrap());
        
        // Third request should be rate limited
        assert!(!manager.check_rate_limit(ip).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_session_management() {
        let manager = SecurityManager::new().unwrap();
        let user_id = Uuid::new_v4();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let permissions = vec!["read".to_string(), "write".to_string()];
        
        // Create session
        let session_id = manager.create_session(user_id, ip, permissions).await.unwrap();
        
        // Validate session
        assert!(manager.validate_session(&session_id).await.unwrap());
        
        // Invalidate session
        assert!(manager.invalidate_session(&session_id).await.unwrap());
        
        // Session should no longer be valid
        assert!(!manager.validate_session(&session_id).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_ip_locking() {
        let manager = SecurityManager::new().unwrap();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        
        // IP should not be locked initially
        assert!(!manager.is_ip_locked(ip).await.unwrap());
        
        // Lock the IP
        manager.lock_ip(ip, "Test lockout".to_string()).await.unwrap();
        
        // IP should now be locked
        assert!(manager.is_ip_locked(ip).await.unwrap());
        
        // Unlock the IP
        assert!(manager.unlock_ip(ip).await.unwrap());
        
        // IP should no longer be locked
        assert!(!manager.is_ip_locked(ip).await.unwrap());
    }
} 