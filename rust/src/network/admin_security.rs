//! Enhanced security for OpenSim Next Admin API
//! 
//! Provides comprehensive security features including rate limiting,
//! IP restrictions, audit logging, and advanced authentication.

use anyhow::Result;
use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Security configuration for admin API
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum requests per minute per IP
    pub rate_limit_per_minute: u32,
    /// Rate limit window duration
    pub rate_limit_window: Duration,
    /// Maximum failed attempts before temporary ban
    pub max_failed_attempts: u32,
    /// Ban duration for repeated failures
    pub ban_duration: Duration,
    /// Allowed IP addresses (empty = allow all)
    pub allowed_ips: Vec<IpAddr>,
    /// Blocked IP addresses
    pub blocked_ips: Vec<IpAddr>,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Require HTTPS for admin operations
    pub require_https: bool,
    /// Session timeout duration
    pub session_timeout: Duration,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit_per_minute: 60,
            rate_limit_window: Duration::from_secs(60),
            max_failed_attempts: 5,
            ban_duration: Duration::from_secs(300), // 5 minutes
            allowed_ips: Vec::new(), // Empty = allow all
            blocked_ips: Vec::new(),
            enable_audit_logging: true,
            require_https: false, // Disabled for development
            session_timeout: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Rate limiting tracker for IP addresses
#[derive(Debug, Clone)]
struct RateLimitTracker {
    requests: Vec<Instant>,
    failed_attempts: u32,
    banned_until: Option<Instant>,
    last_activity: Instant,
}

impl RateLimitTracker {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            failed_attempts: 0,
            banned_until: None,
            last_activity: Instant::now(),
        }
    }
    
    /// Check if IP is currently banned
    fn is_banned(&self) -> bool {
        if let Some(banned_until) = self.banned_until {
            Instant::now() < banned_until
        } else {
            false
        }
    }
    
    /// Add a request and check rate limit
    fn add_request(&mut self, window: Duration, limit: u32) -> bool {
        let now = Instant::now();
        self.last_activity = now;
        
        // Remove old requests outside the window
        self.requests.retain(|&request_time| now.duration_since(request_time) < window);
        
        // Check rate limit
        if self.requests.len() >= limit as usize {
            false
        } else {
            self.requests.push(now);
            true
        }
    }
    
    /// Record failed attempt
    fn record_failure(&mut self, max_attempts: u32, ban_duration: Duration) {
        self.failed_attempts += 1;
        self.last_activity = Instant::now();
        
        if self.failed_attempts >= max_attempts {
            self.banned_until = Some(Instant::now() + ban_duration);
            warn!("IP banned due to {} failed attempts", self.failed_attempts);
        }
    }
    
    /// Reset failed attempts on successful authentication
    fn reset_failures(&mut self) {
        self.failed_attempts = 0;
        self.banned_until = None;
    }
}

/// Audit log entry for admin operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub operation: String,
    pub target: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub request_data: Option<serde_json::Value>,
    pub session_id: Option<String>,
}

/// Enhanced security manager for admin API
pub struct AdminSecurityManager {
    config: SecurityConfig,
    rate_limiters: Arc<RwLock<HashMap<IpAddr, RateLimitTracker>>>,
    audit_log: Arc<RwLock<Vec<AuditLogEntry>>>,
}

impl AdminSecurityManager {
    /// Create new security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Check if request is allowed (rate limiting, IP restrictions, etc.)
    pub async fn check_request_allowed(&self, ip: IpAddr, headers: &HeaderMap) -> Result<(), SecurityError> {
        // Check if IP is explicitly blocked
        if self.config.blocked_ips.contains(&ip) {
            warn!("Blocked IP attempted access: {}", ip);
            return Err(SecurityError::IpBlocked);
        }
        
        // Check if IP is in allowed list (if configured)
        if !self.config.allowed_ips.is_empty() && !self.config.allowed_ips.contains(&ip) {
            warn!("Unauthorized IP attempted access: {}", ip);
            return Err(SecurityError::IpNotAllowed);
        }
        
        // Check HTTPS requirement
        if self.config.require_https {
            if let Some(scheme) = headers.get("x-forwarded-proto") {
                if scheme != "https" {
                    return Err(SecurityError::HttpsRequired);
                }
            } else {
                // In development, we might not have the header
                warn!("HTTPS required but no x-forwarded-proto header found");
            }
        }
        
        // Check rate limiting and bans
        let mut limiters = self.rate_limiters.write().await;
        let tracker = limiters.entry(ip).or_insert_with(RateLimitTracker::new);
        
        // Check if IP is currently banned
        if tracker.is_banned() {
            return Err(SecurityError::IpBanned);
        }
        
        // Check rate limit
        if !tracker.add_request(self.config.rate_limit_window, self.config.rate_limit_per_minute) {
            warn!("Rate limit exceeded for IP: {}", ip);
            return Err(SecurityError::RateLimitExceeded);
        }
        
        Ok(())
    }
    
    /// Authenticate API key
    pub async fn authenticate_api_key(&self, api_key: &str, ip: IpAddr) -> Result<(), SecurityError> {
        let expected_key = std::env::var("OPENSIM_API_KEY")
            .unwrap_or_else(|_| "default-key-change-me".to_string());
        
        if api_key != expected_key {
            // Record failed attempt
            let mut limiters = self.rate_limiters.write().await;
            let tracker = limiters.entry(ip).or_insert_with(RateLimitTracker::new);
            tracker.record_failure(self.config.max_failed_attempts, self.config.ban_duration);
            
            return Err(SecurityError::InvalidApiKey);
        }
        
        // Clear failed attempts on successful authentication
        let mut limiters = self.rate_limiters.write().await;
        if let Some(tracker) = limiters.get_mut(&ip) {
            tracker.reset_failures();
        }
        
        Ok(())
    }
    
    /// Log admin operation for audit trail
    pub async fn log_operation(
        &self,
        ip: String,
        user_agent: Option<String>,
        operation: String,
        target: Option<String>,
        success: bool,
        error_message: Option<String>,
        request_data: Option<serde_json::Value>,
    ) {
        if !self.config.enable_audit_logging {
            return;
        }
        
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            ip_address: ip,
            user_agent,
            operation,
            target,
            success,
            error_message,
            request_data,
            session_id: None, // TODO: Implement session tracking
        };
        
        // Log to tracing for immediate visibility
        if success {
            info!("Admin operation: {} from {} - SUCCESS", entry.operation, entry.ip_address);
        } else {
            warn!("Admin operation: {} from {} - FAILED: {}", 
                  entry.operation, entry.ip_address, 
                  entry.error_message.as_deref().unwrap_or("Unknown error"));
        }
        
        // Store in audit log
        let mut log = self.audit_log.write().await;
        log.push(entry);
        
        // Keep only last 10,000 entries to prevent memory bloat
        if log.len() > 10_000 {
            log.drain(0..1_000); // Remove oldest 1,000 entries
        }
    }
    
    /// Get recent audit log entries
    pub async fn get_audit_log(&self, limit: Option<usize>) -> Vec<AuditLogEntry> {
        let log = self.audit_log.read().await;
        let limit = limit.unwrap_or(100).min(1000); // Max 1000 entries
        
        log.iter()
            .rev() // Most recent first
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Get security statistics
    pub async fn get_security_stats(&self) -> SecurityStats {
        let limiters = self.rate_limiters.read().await;
        let log = self.audit_log.read().await;
        
        let total_ips = limiters.len();
        let banned_ips = limiters.values().filter(|t| t.is_banned()).count();
        let total_requests = limiters.values().map(|t| t.requests.len()).sum::<usize>();
        
        let recent_operations = log.iter()
            .filter(|entry| {
                let now = Utc::now();
                now.signed_duration_since(entry.timestamp).num_hours() < 24
            })
            .count();
        
        let failed_operations = log.iter()
            .filter(|entry| {
                let now = Utc::now();
                !entry.success && now.signed_duration_since(entry.timestamp).num_hours() < 24
            })
            .count();
        
        SecurityStats {
            total_ips_tracked: total_ips,
            currently_banned_ips: banned_ips,
            total_requests_last_hour: total_requests,
            operations_last_24h: recent_operations,
            failed_operations_last_24h: failed_operations,
            audit_log_entries: log.len(),
            rate_limit_per_minute: self.config.rate_limit_per_minute,
            max_failed_attempts: self.config.max_failed_attempts,
        }
    }
    
    /// Clean up old rate limit data
    pub async fn cleanup_old_data(&self) {
        let mut limiters = self.rate_limiters.write().await;
        let now = Instant::now();
        
        // Remove entries older than 1 hour
        limiters.retain(|_, tracker| {
            now.duration_since(tracker.last_activity) < Duration::from_secs(3600)
        });
        
        info!("Cleaned up rate limit data, {} IPs remaining", limiters.len());
    }
}

/// Security error types
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("IP address is blocked")]
    IpBlocked,
    #[error("IP address not in allowed list")]
    IpNotAllowed,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("IP address is temporarily banned")]
    IpBanned,
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("HTTPS required for admin operations")]
    HttpsRequired,
}

impl SecurityError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            SecurityError::IpBlocked | SecurityError::IpNotAllowed => StatusCode::FORBIDDEN,
            SecurityError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            SecurityError::IpBanned => StatusCode::FORBIDDEN,
            SecurityError::InvalidApiKey => StatusCode::UNAUTHORIZED,
            SecurityError::HttpsRequired => StatusCode::UPGRADE_REQUIRED,
        }
    }
}

/// Security statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityStats {
    pub total_ips_tracked: usize,
    pub currently_banned_ips: usize,
    pub total_requests_last_hour: usize,
    pub operations_last_24h: usize,
    pub failed_operations_last_24h: usize,
    pub audit_log_entries: usize,
    pub rate_limit_per_minute: u32,
    pub max_failed_attempts: u32,
}

/// Helper function to extract IP address from request
pub fn extract_ip_from_headers(headers: &HeaderMap) -> Option<IpAddr> {
    // Check X-Forwarded-For header first (for proxy/load balancer)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }
    
    // Check X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let config = SecurityConfig {
            rate_limit_per_minute: 2,
            rate_limit_window: Duration::from_secs(60),
            ..Default::default()
        };
        
        let security = AdminSecurityManager::new(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let headers = HeaderMap::new();
        
        // First two requests should succeed
        assert!(security.check_request_allowed(ip, &headers).await.is_ok());
        assert!(security.check_request_allowed(ip, &headers).await.is_ok());
        
        // Third request should fail (rate limit exceeded)
        assert!(security.check_request_allowed(ip, &headers).await.is_err());
    }
    
    #[tokio::test]
    async fn test_ip_blocking() {
        let blocked_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let config = SecurityConfig {
            blocked_ips: vec![blocked_ip],
            ..Default::default()
        };
        
        let security = AdminSecurityManager::new(config);
        let headers = HeaderMap::new();
        
        // Blocked IP should be rejected
        assert!(security.check_request_allowed(blocked_ip, &headers).await.is_err());
        
        // Other IP should be allowed
        let allowed_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        assert!(security.check_request_allowed(allowed_ip, &headers).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_api_key_authentication() {
        let config = SecurityConfig::default();
        let security = AdminSecurityManager::new(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        // Set expected API key
        std::env::set_var("OPENSIM_API_KEY", "test-key-123");
        
        // Valid API key should succeed
        assert!(security.authenticate_api_key("test-key-123", ip).await.is_ok());
        
        // Invalid API key should fail
        assert!(security.authenticate_api_key("wrong-key", ip).await.is_err());
    }
    
    #[tokio::test]
    async fn test_audit_logging() {
        let config = SecurityConfig::default();
        let security = AdminSecurityManager::new(config);
        
        // Log some operations
        security.log_operation(
            "127.0.0.1".to_string(),
            Some("test-agent".to_string()),
            "create_user".to_string(),
            Some("test user".to_string()),
            true,
            None,
            None,
        ).await;
        
        security.log_operation(
            "127.0.0.1".to_string(),
            Some("test-agent".to_string()),
            "delete_user".to_string(),
            Some("test user".to_string()),
            false,
            Some("User not found".to_string()),
            None,
        ).await;
        
        // Check audit log
        let log = security.get_audit_log(Some(10)).await;
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].operation, "delete_user"); // Most recent first
        assert!(!log[0].success);
        assert_eq!(log[1].operation, "create_user");
        assert!(log[1].success);
    }
}