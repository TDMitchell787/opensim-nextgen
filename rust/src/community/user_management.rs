//! User management system for OpenSim community platform
//!
//! Provides comprehensive user management including:
//! - User registration and authentication
//! - Profile management
//! - Role-based permissions
//! - Activity tracking
//! - OAuth integration

use super::{CommunityConfig, ComponentHealth};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use std::sync::Arc;

/// User management system
pub struct UserManager {
    config: CommunityConfig,
    users: Arc<RwLock<HashMap<String, User>>>,
    sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    oauth_providers: Arc<RwLock<HashMap<String, OAuthProvider>>>,
    activity_tracker: Arc<RwLock<ActivityTracker>>,
}

impl UserManager {
    /// Create new user manager
    pub async fn new(config: CommunityConfig) -> Result<Self> {
        let users = Arc::new(RwLock::new(HashMap::new()));
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let oauth_providers = Arc::new(RwLock::new(HashMap::new()));
        let activity_tracker = Arc::new(RwLock::new(ActivityTracker::new()));

        Ok(Self {
            config,
            users,
            sessions,
            oauth_providers,
            activity_tracker,
        })
    }

    /// Initialize user management system
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing user management system");

        // Setup OAuth providers
        self.setup_oauth_providers().await?;
        
        // Create default admin user if needed
        self.create_default_admin().await?;
        
        // Initialize activity tracking
        self.activity_tracker.write().await.initialize().await?;

        tracing::info!("User management system initialized successfully");
        Ok(())
    }

    /// Setup OAuth providers
    async fn setup_oauth_providers(&self) -> Result<()> {
        let mut providers = self.oauth_providers.write().await;

        if self.config.authentication.github_oauth_enabled {
            providers.insert("github".to_string(), OAuthProvider {
                id: "github".to_string(),
                name: "GitHub".to_string(),
                client_id: std::env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
                client_secret: std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
                authorize_url: "https://github.com/login/oauth/authorize".to_string(),
                token_url: "https://github.com/login/oauth/access_token".to_string(),
                user_info_url: "https://api.github.com/user".to_string(),
                scopes: vec!["user:email".to_string()],
                enabled: true,
            });
        }

        if self.config.authentication.discord_oauth_enabled {
            providers.insert("discord".to_string(), OAuthProvider {
                id: "discord".to_string(),
                name: "Discord".to_string(),
                client_id: std::env::var("DISCORD_CLIENT_ID").unwrap_or_default(),
                client_secret: std::env::var("DISCORD_CLIENT_SECRET").unwrap_or_default(),
                authorize_url: "https://discord.com/api/oauth2/authorize".to_string(),
                token_url: "https://discord.com/api/oauth2/token".to_string(),
                user_info_url: "https://discord.com/api/users/@me".to_string(),
                scopes: vec!["identify".to_string(), "email".to_string()],
                enabled: true,
            });
        }

        Ok(())
    }

    /// Create default admin user
    async fn create_default_admin(&self) -> Result<()> {
        let mut users = self.users.write().await;
        
        if users.is_empty() {
            let admin_user = User {
                id: "admin".to_string(),
                username: "admin".to_string(),
                email: self.config.admin_email.clone(),
                display_name: "Administrator".to_string(),
                role: UserRole::Admin,
                status: UserStatus::Active,
                created_at: get_current_timestamp(),
                last_active_at: get_current_timestamp(),
                profile: UserProfile {
                    bio: Some("OpenSim Administrator".to_string()),
                    avatar_url: None,
                    website_url: None,
                    github_username: None,
                    discord_username: None,
                    location: None,
                    timezone: None,
                },
                preferences: UserPreferences {
                    email_notifications: true,
                    forum_notifications: true,
                    marketing_emails: false,
                    theme: "system".to_string(),
                    language: "en".to_string(),
                },
                stats: UserStats {
                    forum_posts: 0,
                    forum_topics: 0,
                    kb_contributions: 0,
                    reputation_score: 1000,
                    badges: vec!["Admin".to_string(), "Founder".to_string()],
                },
            };

            users.insert("admin".to_string(), admin_user);
            tracing::info!("Created default admin user");
        }

        Ok(())
    }

    /// Register new user
    pub async fn register_user(&self, registration: UserRegistration) -> Result<User> {
        let user_id = generate_user_id();
        
        // Check if username or email already exists
        let users = self.users.read().await;
        for user in users.values() {
            if user.username == registration.username {
                return Err(anyhow::anyhow!("Username already exists"));
            }
            if user.email == registration.email {
                return Err(anyhow::anyhow!("Email already exists"));
            }
        }
        drop(users);

        let user = User {
            id: user_id.clone(),
            username: registration.username,
            email: registration.email,
            display_name: registration.display_name,
            role: UserRole::Member,
            status: if self.config.authentication.require_email_verification {
                UserStatus::PendingVerification
            } else {
                UserStatus::Active
            },
            created_at: get_current_timestamp(),
            last_active_at: get_current_timestamp(),
            profile: UserProfile {
                bio: None,
                avatar_url: None,
                website_url: None,
                github_username: None,
                discord_username: None,
                location: None,
                timezone: None,
            },
            preferences: UserPreferences {
                email_notifications: true,
                forum_notifications: true,
                marketing_emails: false,
                theme: "system".to_string(),
                language: "en".to_string(),
            },
            stats: UserStats {
                forum_posts: 0,
                forum_topics: 0,
                kb_contributions: 0,
                reputation_score: 0,
                badges: Vec::new(),
            },
        };

        // Add user to storage
        self.users.write().await.insert(user_id.clone(), user.clone());

        // Track registration event
        self.activity_tracker.write().await.track_event(
            user_id,
            ActivityEvent::UserRegistered,
            None,
        ).await?;

        Ok(user)
    }

    /// Authenticate user
    pub async fn authenticate_user(&self, credentials: UserCredentials) -> Result<UserSession> {
        // For demo purposes, we'll create a simple session
        // In production, this would validate credentials properly
        let session_id = generate_session_id();
        let session = UserSession {
            id: session_id.clone(),
            user_id: credentials.username.clone(), // In real implementation, would lookup user_id
            created_at: get_current_timestamp(),
            expires_at: get_current_timestamp() + (self.config.authentication.session_timeout_hours as u64 * 3600),
            last_accessed_at: get_current_timestamp(),
            ip_address: credentials.ip_address.clone(),
            user_agent: credentials.user_agent.clone(),
        };

        self.sessions.write().await.insert(session_id.clone(), session.clone());

        // Track login event
        self.activity_tracker.write().await.track_event(
            credentials.username,
            ActivityEvent::UserLogin,
            credentials.ip_address.as_ref().map(|ip| ip.clone()).or_else(|| Some(String::new())),
        ).await?;

        Ok(session)
    }

    /// Get user statistics
    pub async fn get_stats(&self) -> Result<UserManagementStats> {
        let users = self.users.read().await;
        let sessions = self.sessions.read().await;

        let total_users = users.len() as u64;
        let active_users_24h = sessions.values()
            .filter(|s| s.last_accessed_at >= get_current_timestamp() - 86400)
            .count() as u64;

        // Calculate new users today
        let today_start = get_today_start_timestamp();
        let new_users_today = users.values()
            .filter(|u| u.created_at >= today_start)
            .count() as u64;

        Ok(UserManagementStats {
            total_users,
            active_users_24h,
            new_users_today,
            total_sessions: sessions.len() as u64,
            verified_users: users.values().filter(|u| u.status == UserStatus::Active).count() as u64,
            pending_verification: users.values().filter(|u| u.status == UserStatus::PendingVerification).count() as u64,
        })
    }

    /// Get user management health status
    pub async fn health_check(&self) -> Result<ComponentHealth> {
        let start_time = SystemTime::now();
        
        // Test all components
        let _users_count = self.users.read().await.len();
        let _sessions_count = self.sessions.read().await.len();
        let _providers_count = self.oauth_providers.read().await.len();
        
        let response_time = start_time.elapsed().unwrap().as_millis() as u64;
        
        Ok(ComponentHealth {
            status: "healthy".to_string(),
            response_time_ms: response_time,
            last_error: None,
        })
    }
}

/// User structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: u64,
    pub last_active_at: u64,
    pub profile: UserProfile,
    pub preferences: UserPreferences,
    pub stats: UserStats,
}

/// User role enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Moderator,
    Developer,
    Member,
    Guest,
}

/// User status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Active,
    PendingVerification,
    Suspended,
    Banned,
    Deleted,
}

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website_url: Option<String>,
    pub github_username: Option<String>,
    pub discord_username: Option<String>,
    pub location: Option<String>,
    pub timezone: Option<String>,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub email_notifications: bool,
    pub forum_notifications: bool,
    pub marketing_emails: bool,
    pub theme: String,
    pub language: String,
}

/// User statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub forum_posts: u64,
    pub forum_topics: u64,
    pub kb_contributions: u64,
    pub reputation_score: i64,
    pub badges: Vec<String>,
}

/// User registration request
#[derive(Debug, Deserialize)]
pub struct UserRegistration {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

/// User credentials for authentication
#[derive(Debug, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub last_accessed_at: u64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// OAuth provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProvider {
    pub id: String,
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub authorize_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub scopes: Vec<String>,
    pub enabled: bool,
}

/// User management statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct UserManagementStats {
    pub total_users: u64,
    pub active_users_24h: u64,
    pub new_users_today: u64,
    pub total_sessions: u64,
    pub verified_users: u64,
    pub pending_verification: u64,
}

/// Activity tracking system
pub struct ActivityTracker {
    events: Vec<ActivityEventRecord>,
}

impl ActivityTracker {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize activity tracking
        Ok(())
    }

    pub async fn track_event(&mut self, user_id: String, event: ActivityEvent, metadata: Option<String>) -> Result<()> {
        let record = ActivityEventRecord {
            id: generate_event_id(),
            user_id,
            event,
            metadata,
            timestamp: get_current_timestamp(),
        };

        self.events.push(record);
        Ok(())
    }
}

/// Activity event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityEvent {
    UserRegistered,
    UserLogin,
    UserLogout,
    ProfileUpdated,
    ForumPost,
    ForumTopic,
    KnowledgeBaseContribution,
}

/// Activity event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEventRecord {
    pub id: String,
    pub user_id: String,
    pub event: ActivityEvent,
    pub metadata: Option<String>,
    pub timestamp: u64,
}

/// Utility functions
fn generate_user_id() -> String {
    format!("user_{}", get_current_timestamp())
}

fn generate_session_id() -> String {
    format!("session_{}", get_current_timestamp())
}

fn generate_event_id() -> String {
    format!("event_{}", get_current_timestamp())
}

fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn get_today_start_timestamp() -> u64 {
    let now = get_current_timestamp();
    now - (now % 86400)
}