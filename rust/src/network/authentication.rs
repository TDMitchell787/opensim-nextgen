use md5;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Production-ready authentication manager with MD5 password verification and rate limiting
#[derive(Debug, Clone)]
pub struct AuthenticationManager {
    /// Rate limiter for login attempts
    rate_limiter: Arc<RwLock<RateLimiter>>,
    /// Password verification engine
    password_verifier: Arc<PasswordVerifier>,
    /// Security configuration
    config: AuthenticationConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Maximum login attempts per IP per time window
    pub max_attempts_per_ip: u32,
    /// Time window for rate limiting (in seconds)
    pub rate_limit_window: u64,
    /// Temporary ban duration for exceeding rate limit (in seconds)
    pub temp_ban_duration: u64,
    /// Enable detailed security logging
    pub enable_security_logging: bool,
    /// Minimum password length
    pub min_password_length: usize,
    /// Maximum password length
    pub max_password_length: usize,
    /// Enable MD5 hash verification
    pub enable_md5_verification: bool,
    /// Enable timing attack protection
    pub enable_timing_protection: bool,
}

/// Rate limiter for login attempts
#[derive(Debug)]
pub struct RateLimiter {
    /// IP address -> attempt tracking
    attempts: HashMap<IpAddr, AttemptTracker>,
    /// Temporarily banned IPs
    banned_ips: HashMap<IpAddr, Instant>,
    /// Configuration
    config: AuthenticationConfig,
}

/// Attempt tracking per IP
#[derive(Debug, Clone)]
pub struct AttemptTracker {
    /// Number of attempts in current window
    attempts: u32,
    /// Window start time
    window_start: Instant,
    /// Last attempt time
    last_attempt: Instant,
    /// Whether this IP is temporarily banned
    is_banned: bool,
}

/// Password verification engine
#[derive(Debug)]
pub struct PasswordVerifier {
    config: AuthenticationConfig,
}

/// Authentication result
#[derive(Debug, Clone, PartialEq)]
pub enum AuthenticationResult {
    /// Authentication successful
    Success,
    /// Invalid credentials
    InvalidCredentials,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// IP temporarily banned
    IpBanned,
    /// Invalid password format
    InvalidPasswordFormat,
    /// Password too weak
    PasswordTooWeak,
}

/// Authentication errors
#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthenticationError {
    #[error("Invalid credentials provided")]
    InvalidCredentials,
    #[error("Rate limit exceeded for IP: {ip}")]
    RateLimitExceeded { ip: IpAddr },
    #[error("IP temporarily banned: {ip}")]
    IpBanned { ip: IpAddr },
    #[error("Invalid password format")]
    InvalidPasswordFormat,
    #[error("Password does not meet security requirements")]
    PasswordTooWeak,
    #[error("Authentication service unavailable")]
    ServiceUnavailable,
    #[error("MD5 hash verification failed")]
    Md5VerificationFailed,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            max_attempts_per_ip: 10, // 10 attempts per window
            rate_limit_window: 600,  // 10 minute window
            temp_ban_duration: 1800, // 30 minute ban
            enable_security_logging: true,
            min_password_length: 8,
            max_password_length: 128,
            enable_md5_verification: true,
            enable_timing_protection: true,
        }
    }
}

impl AuthenticationManager {
    /// Create new authentication manager
    pub fn new(config: AuthenticationConfig) -> Self {
        Self {
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new(config.clone()))),
            password_verifier: Arc::new(PasswordVerifier::new(config.clone())),
            config,
        }
    }

    /// Authenticate user with production-ready security
    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
        stored_password_hash: &str,
        client_ip: IpAddr,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        // Check rate limiting first
        match self.check_rate_limit(client_ip).await {
            Ok(()) => {}
            Err(e) => return Err(e),
        }

        // Record the attempt
        self.record_attempt(client_ip).await;

        // Verify password with timing attack protection
        let verification_result = if self.config.enable_timing_protection {
            self.verify_password_with_timing_protection(password, stored_password_hash)
                .await
        } else {
            self.verify_password(password, stored_password_hash).await
        };

        match verification_result {
            Ok(AuthenticationResult::Success) => {
                // Reset attempts on successful authentication
                self.reset_attempts(client_ip).await;

                if self.config.enable_security_logging {
                    info!(
                        "Successful authentication for user: {} from IP: {}",
                        username, client_ip
                    );
                }

                Ok(AuthenticationResult::Success)
            }
            Ok(result) => {
                if self.config.enable_security_logging {
                    warn!(
                        "Failed authentication for user: {} from IP: {} - {:?}",
                        username, client_ip, result
                    );
                }
                Ok(result)
            }
            Err(e) => {
                if self.config.enable_security_logging {
                    error!(
                        "Authentication error for user: {} from IP: {} - {}",
                        username, client_ip, e
                    );
                }
                Err(e)
            }
        }
    }

    /// Check if IP is rate limited
    async fn check_rate_limit(&self, ip: IpAddr) -> Result<(), AuthenticationError> {
        let rate_limiter = self.rate_limiter.read().await;
        rate_limiter.check_rate_limit(ip)
    }

    /// Record authentication attempt
    async fn record_attempt(&self, ip: IpAddr) {
        let mut rate_limiter = self.rate_limiter.write().await;
        rate_limiter.record_attempt(ip);
    }

    /// Reset attempts for IP on successful authentication
    async fn reset_attempts(&self, ip: IpAddr) {
        let mut rate_limiter = self.rate_limiter.write().await;
        rate_limiter.reset_attempts(ip);
    }

    /// Verify password
    async fn verify_password(
        &self,
        password: &str,
        stored_hash: &str,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        self.password_verifier
            .verify_password(password, stored_hash)
    }

    /// Verify password with timing attack protection
    async fn verify_password_with_timing_protection(
        &self,
        password: &str,
        stored_hash: &str,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        // Always perform the same amount of work regardless of input
        let start_time = Instant::now();

        let result = self
            .password_verifier
            .verify_password(password, stored_hash);

        // Ensure minimum computation time to prevent timing attacks
        let min_duration = Duration::from_millis(100);
        let elapsed = start_time.elapsed();

        if elapsed < min_duration {
            tokio::time::sleep(min_duration - elapsed).await;
        }

        result
    }

    /// Get authentication statistics
    pub async fn get_statistics(&self) -> AuthenticationStatistics {
        let rate_limiter = self.rate_limiter.read().await;
        rate_limiter.get_statistics()
    }

    /// Clean up expired bans and old attempt records
    pub async fn cleanup_expired(&self) {
        let mut rate_limiter = self.rate_limiter.write().await;
        rate_limiter.cleanup_expired();
    }

    /// Check if IP is banned
    pub async fn is_ip_banned(&self, ip: IpAddr) -> bool {
        let rate_limiter = self.rate_limiter.read().await;
        rate_limiter.is_ip_banned(ip)
    }

    /// Manually ban IP (for abuse prevention)
    pub async fn ban_ip(&self, ip: IpAddr, duration: Duration) {
        let mut rate_limiter = self.rate_limiter.write().await;
        rate_limiter.ban_ip(ip, duration);
    }
}

impl RateLimiter {
    fn new(config: AuthenticationConfig) -> Self {
        Self {
            attempts: HashMap::new(),
            banned_ips: HashMap::new(),
            config,
        }
    }

    fn check_rate_limit(&self, ip: IpAddr) -> Result<(), AuthenticationError> {
        // Check if IP is banned
        if let Some(ban_time) = self.banned_ips.get(&ip) {
            let ban_duration = Duration::from_secs(self.config.temp_ban_duration);
            if ban_time.elapsed() < ban_duration {
                return Err(AuthenticationError::IpBanned { ip });
            }
        }

        // Check rate limit
        if let Some(tracker) = self.attempts.get(&ip) {
            let window_duration = Duration::from_secs(self.config.rate_limit_window);

            // If within the window and exceeding limits
            if tracker.window_start.elapsed() < window_duration {
                if tracker.attempts >= self.config.max_attempts_per_ip {
                    return Err(AuthenticationError::RateLimitExceeded { ip });
                }
            }
        }

        Ok(())
    }

    fn record_attempt(&mut self, ip: IpAddr) {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.rate_limit_window);

        let tracker = self.attempts.entry(ip).or_insert_with(|| AttemptTracker {
            attempts: 0,
            window_start: now,
            last_attempt: now,
            is_banned: false,
        });

        // Reset window if expired
        if tracker.window_start.elapsed() >= window_duration {
            tracker.attempts = 0;
            tracker.window_start = now;
        }

        tracker.attempts += 1;
        tracker.last_attempt = now;

        // Check if we should ban this IP
        if tracker.attempts >= self.config.max_attempts_per_ip {
            self.banned_ips.insert(ip, now);
            tracker.is_banned = true;

            warn!(
                "IP {} temporarily banned for exceeding rate limit ({} attempts)",
                ip, tracker.attempts
            );
        }
    }

    fn reset_attempts(&mut self, ip: IpAddr) {
        self.attempts.remove(&ip);
        self.banned_ips.remove(&ip);
    }

    fn is_ip_banned(&self, ip: IpAddr) -> bool {
        if let Some(ban_time) = self.banned_ips.get(&ip) {
            let ban_duration = Duration::from_secs(self.config.temp_ban_duration);
            ban_time.elapsed() < ban_duration
        } else {
            false
        }
    }

    fn ban_ip(&mut self, ip: IpAddr, _duration: Duration) {
        self.banned_ips.insert(ip, Instant::now());

        if let Some(tracker) = self.attempts.get_mut(&ip) {
            tracker.is_banned = true;
        }
    }

    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let ban_duration = Duration::from_secs(self.config.temp_ban_duration);
        let window_duration = Duration::from_secs(self.config.rate_limit_window);

        // Remove expired bans
        self.banned_ips
            .retain(|_, ban_time| ban_time.elapsed() < ban_duration);

        // Remove old attempt records
        self.attempts.retain(|_, tracker| {
            tracker.window_start.elapsed() < window_duration
                || tracker.last_attempt.elapsed() < ban_duration
        });
    }

    fn get_statistics(&self) -> AuthenticationStatistics {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.rate_limit_window);

        let mut stats = AuthenticationStatistics::default();
        stats.total_tracked_ips = self.attempts.len();
        stats.currently_banned_ips = self.banned_ips.len();

        for tracker in self.attempts.values() {
            if tracker.window_start.elapsed() < window_duration {
                stats.active_rate_limited_ips += 1;
                stats.total_attempts_in_window += tracker.attempts;
            }
        }

        stats
    }
}

impl PasswordVerifier {
    fn new(config: AuthenticationConfig) -> Self {
        Self { config }
    }

    /// Verify password against stored hash with Cool Viewer compatibility
    fn verify_password(
        &self,
        password: &str,
        stored_hash: &str,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        // Validate password format
        if password.len() < self.config.min_password_length {
            return Ok(AuthenticationResult::PasswordTooWeak);
        }

        if password.len() > self.config.max_password_length {
            return Err(AuthenticationError::InvalidPasswordFormat);
        }

        // Handle MD5 hash format from Second Life viewers
        if self.config.enable_md5_verification && password.starts_with("$1$") {
            self.verify_md5_hash(password, stored_hash)
        } else {
            // Fallback to plaintext comparison for testing/development
            self.verify_plaintext(password, stored_hash)
        }
    }

    /// Verify MD5 hash (Second Life viewer format: "$1$hash")
    fn verify_md5_hash(
        &self,
        password: &str,
        stored_hash: &str,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        // Extract hash part after $1$ prefix
        let hash_part = password
            .strip_prefix("$1$")
            .ok_or(AuthenticationError::InvalidPasswordFormat)?;

        if hash_part.len() != 32 {
            return Err(AuthenticationError::InvalidPasswordFormat);
        }

        // Verify MD5 hash format (hex string)
        if !hash_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AuthenticationError::InvalidPasswordFormat);
        }

        // Compare with stored hash - this could be:
        // 1. A plaintext password that we hash and compare
        // 2. An already-hashed password that we compare directly

        if stored_hash.len() == 32 && stored_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            // Stored hash is already MD5, compare directly
            if constant_time_compare(hash_part.as_bytes(), stored_hash.as_bytes()) {
                Ok(AuthenticationResult::Success)
            } else {
                Ok(AuthenticationResult::InvalidCredentials)
            }
        } else {
            // Stored value is plaintext, hash it and compare
            let computed_hash = self.compute_md5_hash(stored_hash);
            if constant_time_compare(hash_part.as_bytes(), computed_hash.as_bytes()) {
                Ok(AuthenticationResult::Success)
            } else {
                Ok(AuthenticationResult::InvalidCredentials)
            }
        }
    }

    /// Verify plaintext password
    fn verify_plaintext(
        &self,
        password: &str,
        stored_password: &str,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        if constant_time_compare(password.as_bytes(), stored_password.as_bytes()) {
            Ok(AuthenticationResult::Success)
        } else {
            Ok(AuthenticationResult::InvalidCredentials)
        }
    }

    /// Compute MD5 hash of input
    fn compute_md5_hash(&self, input: &str) -> String {
        let result = md5::compute(input.as_bytes());
        format!("{:x}", result)
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Authentication statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AuthenticationStatistics {
    pub total_tracked_ips: usize,
    pub currently_banned_ips: usize,
    pub active_rate_limited_ips: usize,
    pub total_attempts_in_window: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_authentication_manager_creation() {
        let config = AuthenticationConfig::default();
        let auth_manager = AuthenticationManager::new(config);

        // Should be able to create without errors
        assert!(
            !auth_manager
                .is_ip_banned(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
                .await
        );
    }

    #[tokio::test]
    async fn test_md5_verification() {
        let config = AuthenticationConfig::default();
        let auth_manager = AuthenticationManager::new(config);

        // Test MD5 hash verification
        let password = "$1$482c811da5d5b4bc6d497ffa98491e38"; // MD5 of "password123"
        let stored_password = "password123";
        let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        let result = auth_manager
            .authenticate_user("testuser", password, stored_password, client_ip)
            .await;
        assert!(matches!(result, Ok(AuthenticationResult::Success)));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mut config = AuthenticationConfig::default();
        config.max_attempts_per_ip = 2; // Low limit for testing

        let auth_manager = AuthenticationManager::new(config);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));

        // First attempt should be allowed
        let result = auth_manager
            .authenticate_user("testuser", "wrongpassword", "correctpassword", client_ip)
            .await;
        assert!(matches!(
            result,
            Ok(AuthenticationResult::InvalidCredentials)
        ));

        // Second attempt should be allowed
        let result = auth_manager
            .authenticate_user("testuser", "wrongpassword", "correctpassword", client_ip)
            .await;
        assert!(matches!(
            result,
            Ok(AuthenticationResult::InvalidCredentials)
        ));

        // Third attempt should be rate limited or IP banned
        let result = auth_manager
            .authenticate_user("testuser", "wrongpassword", "correctpassword", client_ip)
            .await;
        assert!(matches!(
            result,
            Err(AuthenticationError::RateLimitExceeded { .. })
                | Err(AuthenticationError::IpBanned { .. })
        ));
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare(b"hello", b"hello"));
        assert!(!constant_time_compare(b"hello", b"world"));
        assert!(!constant_time_compare(b"hello", b"hello1"));
        assert!(!constant_time_compare(b"hello1", b"hello"));
    }

    #[test]
    fn test_md5_hash_computation() {
        let config = AuthenticationConfig::default();
        let verifier = PasswordVerifier::new(config);

        let hash = verifier.compute_md5_hash("password123");
        assert_eq!(hash, "482c811da5d5b4bc6d497ffa98491e38");
    }
}
