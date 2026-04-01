//! Authentication and Authorization for OpenSim Next REST API

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use uuid::Uuid;
use sha2::{Digest, Sha256};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use tracing::{error, info, warn};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,         // Subject (user ID)
    pub username: String,    // Username
    pub user_level: u32,     // User permission level
    pub exp: usize,          // Expiration time
    pub iat: usize,          // Issued at
    pub iss: String,         // Issuer
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub user_level: u32,
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub ip_address: String,
    pub user_agent: String,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: u32,
    pub require_email_verification: bool,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub session_timeout_minutes: u32,
    pub allow_multiple_sessions: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key-change-in-production".to_string(),
            jwt_expiry_hours: 24,
            require_email_verification: false,
            max_failed_attempts: 5,
            lockout_duration_minutes: 15,
            session_timeout_minutes: 60,
            allow_multiple_sessions: true,
        }
    }
}

/// User credentials for login
#[derive(Debug, Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u32,
    pub user: UserInfo,
}

/// User information (public)
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub user_level: u32,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

/// Failed login attempt tracking
#[derive(Debug, Clone)]
pub struct FailedAttempt {
    pub count: u32,
    pub last_attempt: chrono::DateTime<chrono::Utc>,
    pub locked_until: Option<chrono::DateTime<chrono::Utc>>,
}

/// Authentication service
pub struct AuthenticationService {
    config: AuthConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    failed_attempts: Arc<RwLock<HashMap<String, FailedAttempt>>>,
    sessions: Arc<RwLock<HashMap<String, UserSession>>>,
}

impl AuthenticationService {
    pub fn new(config: AuthConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        
        Self {
            config,
            encoding_key,
            decoding_key,
            failed_attempts: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Authenticate user with username/password
    pub async fn authenticate(&self, credentials: LoginCredentials, ip_address: String, user_agent: String) -> Result<LoginResponse> {
        // Check if IP is locked out
        if self.is_locked_out(&ip_address).await {
            return Err(anyhow!("Account temporarily locked due to too many failed attempts"));
        }
        
        // Verify credentials (this would typically query a database)
        let user = self.verify_credentials(&credentials.username, &credentials.password).await?;
        
        // Clear failed attempts on successful login
        self.clear_failed_attempts(&ip_address).await;
        
        // Generate JWT tokens
        let access_token = self.generate_access_token(&user)?;
        let refresh_token = self.generate_refresh_token(&user)?;
        
        // Create session
        let session = UserSession {
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            user_level: user.user_level,
            session_id: Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            ip_address,
            user_agent,
        };
        
        // Store session
        self.sessions.write().await.insert(session.session_id.clone(), session);
        
        let expires_in = if credentials.remember_me.unwrap_or(false) {
            self.config.jwt_expiry_hours * 3600 * 7 // 7 days for remember me
        } else {
            self.config.jwt_expiry_hours * 3600
        };
        
        Ok(LoginResponse {
            access_token,
            refresh_token,
            expires_in,
            user,
        })
    }
    
    /// Verify JWT token and return user session
    pub async fn verify_token(&self, token: &str) -> Result<UserSession> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )?;
        
        let user_id = Uuid::parse_str(&token_data.claims.sub)?;
        
        // Find active session
        let sessions = self.sessions.read().await;
        for session in sessions.values() {
            if session.user_id == user_id {
                // Check session timeout
                let now = chrono::Utc::now();
                let timeout = chrono::Duration::minutes(self.config.session_timeout_minutes as i64);
                
                if now.signed_duration_since(session.last_activity) > timeout {
                    return Err(anyhow!("Session expired"));
                }
                
                return Ok(session.clone());
            }
        }
        
        Err(anyhow!("Session not found"))
    }
    
    /// Generate access token
    fn generate_access_token(&self, user: &UserInfo) -> Result<String> {
        let now = chrono::Utc::now();
        let expiry = now + chrono::Duration::hours(self.config.jwt_expiry_hours as i64);
        
        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            user_level: user.user_level,
            exp: expiry.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: "opensim-next".to_string(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to generate token: {}", e))
    }
    
    /// Generate refresh token (longer lived)
    fn generate_refresh_token(&self, user: &UserInfo) -> Result<String> {
        let now = chrono::Utc::now();
        let expiry = now + chrono::Duration::days(30); // 30 days for refresh token
        
        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            user_level: user.user_level,
            exp: expiry.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: "opensim-next-refresh".to_string(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to generate refresh token: {}", e))
    }
    
    /// Verify user credentials (mock implementation)
    async fn verify_credentials(&self, username: &str, password: &str) -> Result<UserInfo> {
        // This would typically query a database
        // For demo purposes, accept "admin/admin"
        if username == "admin" && password == "admin" {
            Ok(UserInfo {
                id: Uuid::new_v4(),
                username: username.to_string(),
                email: "admin@opensim.local".to_string(),
                first_name: "Admin".to_string(),
                last_name: "User".to_string(),
                user_level: 100, // Admin level
                last_login: Some(chrono::Utc::now()),
            })
        } else {
            Err(anyhow!("Invalid credentials"))
        }
    }
    
    /// Record failed login attempt
    pub async fn record_failed_attempt(&self, ip_address: &str) {
        let mut attempts = self.failed_attempts.write().await;
        let now = chrono::Utc::now();
        
        match attempts.get_mut(ip_address) {
            Some(attempt) => {
                attempt.count += 1;
                attempt.last_attempt = now;
                
                if attempt.count >= self.config.max_failed_attempts {
                    attempt.locked_until = Some(now + chrono::Duration::minutes(self.config.lockout_duration_minutes as i64));
                }
            }
            None => {
                attempts.insert(ip_address.to_string(), FailedAttempt {
                    count: 1,
                    last_attempt: now,
                    locked_until: None,
                });
            }
        }
    }
    
    /// Check if IP address is locked out
    async fn is_locked_out(&self, ip_address: &str) -> bool {
        let attempts = self.failed_attempts.read().await;
        
        if let Some(attempt) = attempts.get(ip_address) {
            if let Some(locked_until) = attempt.locked_until {
                return chrono::Utc::now() < locked_until;
            }
        }
        
        false
    }
    
    /// Clear failed attempts for IP address
    async fn clear_failed_attempts(&self, ip_address: &str) {
        let mut attempts = self.failed_attempts.write().await;
        attempts.remove(ip_address);
    }
    
    /// Logout user session
    pub async fn logout(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }
    
    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = chrono::Utc::now();
            Ok(())
        } else {
            Err(anyhow!("Session not found"))
        }
    }
    
    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Vec<UserSession> {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|session| session.user_id == user_id)
            .cloned()
            .collect()
    }
    
    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let now = chrono::Utc::now();
        let timeout = chrono::Duration::minutes(self.config.session_timeout_minutes as i64);
        
        sessions.retain(|_, session| {
            now.signed_duration_since(session.last_activity) <= timeout
        });
    }
    
    /// Hash password (for user registration/password changes)
    pub fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Verify password hash
    pub fn verify_password_hash(password: &str, hash: &str) -> bool {
        let computed_hash = Self::hash_password(password);
        computed_hash == hash
    }
}

/// Authentication extractor for Axum handlers
pub struct AuthenticatedUser(pub UserSession);

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts.headers
            .get("authorization")
            .and_then(|header| header.to_str().ok())
            .ok_or(AuthError::MissingToken)?;
        
        // Parse Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;
        
        // Note: In a real implementation, you'd inject the AuthenticationService
        // and verify the token here. For now, this is a placeholder.
        
        Err(AuthError::InvalidToken)
    }
}

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    Expired,
    InsufficientPermissions,
    LockedOut,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authentication token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authentication token"),
            AuthError::Expired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions"),
            AuthError::LockedOut => (StatusCode::LOCKED, "Account temporarily locked"),
        };
        
        let response = json!({
            "success": false,
            "error": message,
            "timestamp": chrono::Utc::now()
        });
        
        (status, Json(response)).into_response()
    }
}

/// Permission levels
pub mod permissions {
    pub const GUEST: u32 = 0;
    pub const USER: u32 = 10;
    pub const MODERATOR: u32 = 50;
    pub const ADMIN: u32 = 100;
    
    pub fn has_permission(user_level: u32, required_level: u32) -> bool {
        user_level >= required_level
    }
}

/// Authorization middleware for specific permission levels
pub struct RequirePermission {
    pub level: u32,
}

impl RequirePermission {
    pub fn new(level: u32) -> Self {
        Self { level }
    }
    
    pub fn admin() -> Self {
        Self::new(permissions::ADMIN)
    }
    
    pub fn moderator() -> Self {
        Self::new(permissions::MODERATOR)
    }
    
    pub fn user() -> Self {
        Self::new(permissions::USER)
    }
}

// Note: RequirePermission should be used as a constructor for creating
// specific permission guards, not as a direct extractor.
// Use RequireAdmin, RequireModerator, RequireUser instead for axum handlers.

/// Admin permission extractor
pub struct RequireAdmin(pub UserSession);

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthenticatedUser(session) = AuthenticatedUser::from_request_parts(parts, state).await?;
        
        if !permissions::has_permission(session.user_level, permissions::ADMIN) {
            return Err(AuthError::InsufficientPermissions);
        }
        
        Ok(RequireAdmin(session))
    }
}

/// Moderator permission extractor
pub struct RequireModerator(pub UserSession);

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequireModerator
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthenticatedUser(session) = AuthenticatedUser::from_request_parts(parts, state).await?;
        
        if !permissions::has_permission(session.user_level, permissions::MODERATOR) {
            return Err(AuthError::InsufficientPermissions);
        }
        
        Ok(RequireModerator(session))
    }
}

/// Admin API authentication - simplified for OpenSim Robust compatibility
/// Uses API key authentication for admin operations
pub async fn admin_auth_middleware(
    headers: HeaderMap,
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    // Extract API key from headers
    let api_key = match headers.get("X-API-Key") {
        Some(key) => match key.to_str() {
            Ok(key_str) => key_str,
            Err(_) => {
                warn!("Invalid API key format in headers");
                return Err(StatusCode::BAD_REQUEST);
            }
        },
        None => {
            warn!("Missing X-API-Key header for admin endpoint");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    // Get expected API key from environment or default
    let expected_api_key = std::env::var("OPENSIM_API_KEY")
        .unwrap_or_else(|_| "default-key-change-me".to_string());
    
    // Validate API key
    if api_key != expected_api_key {
        warn!("Invalid API key provided for admin endpoint");
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    info!("Admin API key validated successfully");
    
    // API key is valid, proceed with request
    Ok(next.run(request).await)
}

/// Simple admin authentication middleware using Axum's from_fn
pub async fn require_admin_auth_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let headers = request.headers();
    
    // Validate admin authentication
    match validate_admin_auth(headers).await {
        Ok(_) => {
            info!("Admin authentication successful");
            Ok(next.run(request).await)
        }
        Err(status) => {
            warn!("Admin authentication failed: {:?}", status);
            Err(status)
        }
    }
}

/// Validate admin authentication from request headers
async fn validate_admin_auth(headers: &HeaderMap) -> Result<(), StatusCode> {
    // Extract API key from headers
    let api_key = headers
        .get("X-API-Key")
        .and_then(|key| key.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Get expected API key from environment or default
    let expected_api_key = std::env::var("OPENSIM_API_KEY")
        .unwrap_or_else(|_| "default-key-change-me".to_string());
    
    // Validate API key
    if api_key != expected_api_key {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // TODO: Add additional validation like:
    // - Check if user has admin privileges in database
    // - Validate session tokens
    // - Check IP restrictions
    // - Rate limiting
    
    Ok(())
}

/// Create standardized authentication error response
fn create_auth_error_response(status: StatusCode) -> axum::response::Response {
    let (status_code, message) = match status {
        StatusCode::UNAUTHORIZED => (StatusCode::UNAUTHORIZED, "Authentication required. Provide valid X-API-Key header."),
        StatusCode::FORBIDDEN => (StatusCode::FORBIDDEN, "Insufficient privileges for admin operations."),
        StatusCode::BAD_REQUEST => (StatusCode::BAD_REQUEST, "Invalid authentication format."),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication error."),
    };
    
    let body = json!({
        "success": false,
        "error": "authentication_failed",
        "message": message,
        "required_headers": ["X-API-Key"],
        "admin_level_required": 200
    });
    
    (status_code, Json(body)).into_response()
}