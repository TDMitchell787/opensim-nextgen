//! Community and developer portal infrastructure for OpenSim
//!
//! This module provides comprehensive community features including:
//! - Developer portal with documentation and resources
//! - Community forums and discussions
//! - Knowledge base and FAQ system
//! - User authentication and profiles
//! - Content management and moderation tools

pub mod developer_portal;
pub mod forums;
pub mod knowledge_base;
pub mod user_management;
pub mod content_moderation;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Community platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityConfig {
    pub portal_name: String,
    pub base_url: String,
    pub admin_email: String,
    pub features: CommunityFeatures,
    pub authentication: AuthConfig,
    pub moderation: ModerationConfig,
}

impl Default for CommunityConfig {
    fn default() -> Self {
        Self {
            portal_name: "OpenSim Developer Portal".to_string(),
            base_url: "https://developers.opensim.org".to_string(),
            admin_email: "admin@opensim.org".to_string(),
            features: CommunityFeatures::default(),
            authentication: AuthConfig::default(),
            moderation: ModerationConfig::default(),
        }
    }
}

/// Community platform features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityFeatures {
    pub forums_enabled: bool,
    pub knowledge_base_enabled: bool,
    pub user_profiles_enabled: bool,
    pub code_sharing_enabled: bool,
    pub events_enabled: bool,
    pub announcements_enabled: bool,
    pub search_enabled: bool,
    pub analytics_enabled: bool,
}

impl Default for CommunityFeatures {
    fn default() -> Self {
        Self {
            forums_enabled: true,
            knowledge_base_enabled: true,
            user_profiles_enabled: true,
            code_sharing_enabled: true,
            events_enabled: true,
            announcements_enabled: true,
            search_enabled: true,
            analytics_enabled: true,
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub github_oauth_enabled: bool,
    pub discord_oauth_enabled: bool,
    pub email_registration_enabled: bool,
    pub require_email_verification: bool,
    pub session_timeout_hours: u32,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            github_oauth_enabled: true,
            discord_oauth_enabled: true,
            email_registration_enabled: true,
            require_email_verification: true,
            session_timeout_hours: 24,
        }
    }
}

/// Content moderation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationConfig {
    pub auto_moderation_enabled: bool,
    pub spam_detection_enabled: bool,
    pub profanity_filter_enabled: bool,
    pub require_approval_for_new_users: bool,
    pub max_posts_per_hour: u32,
}

impl Default for ModerationConfig {
    fn default() -> Self {
        Self {
            auto_moderation_enabled: true,
            spam_detection_enabled: true,
            profanity_filter_enabled: true,
            require_approval_for_new_users: false,
            max_posts_per_hour: 10,
        }
    }
}

/// Main community platform manager
pub struct CommunityPlatform {
    config: CommunityConfig,
    developer_portal: Arc<RwLock<developer_portal::DeveloperPortal>>,
    forums: Arc<RwLock<forums::ForumSystem>>,
    knowledge_base: Arc<RwLock<knowledge_base::KnowledgeBase>>,
    user_manager: Arc<RwLock<user_management::UserManager>>,
    content_moderator: Arc<RwLock<content_moderation::ContentModerator>>,
}

impl CommunityPlatform {
    /// Create new community platform instance
    pub async fn new(config: CommunityConfig) -> Result<Self> {
        let developer_portal = Arc::new(RwLock::new(
            developer_portal::DeveloperPortal::new(config.clone()).await?
        ));
        
        let forums = Arc::new(RwLock::new(
            forums::ForumSystem::new(config.clone()).await?
        ));
        
        let knowledge_base = Arc::new(RwLock::new(
            knowledge_base::KnowledgeBase::new(config.clone()).await?
        ));
        
        let user_manager = Arc::new(RwLock::new(
            user_management::UserManager::new(config.clone()).await?
        ));
        
        let content_moderator = Arc::new(RwLock::new(
            content_moderation::ContentModerator::new(config.clone()).await?
        ));

        Ok(Self {
            config,
            developer_portal,
            forums,
            knowledge_base,
            user_manager,
            content_moderator,
        })
    }

    /// Initialize the community platform
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing OpenSim Community Platform");

        // Initialize all subsystems
        self.developer_portal.write().await.initialize().await?;
        self.forums.write().await.initialize().await?;
        self.knowledge_base.write().await.initialize().await?;
        self.user_manager.write().await.initialize().await?;
        self.content_moderator.write().await.initialize().await?;

        tracing::info!("Community platform initialized successfully");
        Ok(())
    }

    /// Get platform health status
    pub async fn health_check(&self) -> Result<CommunityHealthStatus> {
        let portal_health = self.developer_portal.read().await.health_check().await?;
        let forums_health = self.forums.read().await.health_check().await?;
        let kb_health = self.knowledge_base.read().await.health_check().await?;
        let user_health = self.user_manager.read().await.health_check().await?;
        let moderation_health = self.content_moderator.read().await.health_check().await?;

        Ok(CommunityHealthStatus {
            overall_status: if portal_health.is_healthy() && 
                              forums_health.is_healthy() && 
                              kb_health.is_healthy() && 
                              user_health.is_healthy() && 
                              moderation_health.is_healthy() {
                "healthy".to_string()
            } else {
                "degraded".to_string()
            },
            portal_status: portal_health,
            forums_status: forums_health,
            knowledge_base_status: kb_health,
            user_management_status: user_health,
            moderation_status: moderation_health,
            uptime_seconds: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Get platform statistics
    pub async fn get_platform_stats(&self) -> Result<CommunityStats> {
        let portal_stats = self.developer_portal.read().await.get_stats().await?;
        let forum_stats = self.forums.read().await.get_stats().await?;
        let kb_stats = self.knowledge_base.read().await.get_stats().await?;
        let user_stats = self.user_manager.read().await.get_stats().await?;

        Ok(CommunityStats {
            total_users: user_stats.total_users,
            active_users_24h: user_stats.active_users_24h,
            total_forum_posts: forum_stats.total_posts,
            total_forum_topics: forum_stats.total_topics,
            total_kb_articles: kb_stats.total_articles,
            total_portal_views: portal_stats.total_views,
            new_users_today: user_stats.new_users_today,
            posts_today: forum_stats.posts_today,
        })
    }
}

/// Community platform health status
#[derive(Debug, Serialize, Deserialize)]
pub struct CommunityHealthStatus {
    pub overall_status: String,
    pub portal_status: ComponentHealth,
    pub forums_status: ComponentHealth,
    pub knowledge_base_status: ComponentHealth,
    pub user_management_status: ComponentHealth,
    pub moderation_status: ComponentHealth,
    pub uptime_seconds: u64,
}

/// Individual component health status
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub response_time_ms: u64,
    pub last_error: Option<String>,
}

impl ComponentHealth {
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            response_time_ms: 0,
            last_error: None,
        }
    }

    pub fn unhealthy(error: String) -> Self {
        Self {
            status: "unhealthy".to_string(),
            response_time_ms: 0,
            last_error: Some(error),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.status == "healthy"
    }
}

/// Community platform statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CommunityStats {
    pub total_users: u64,
    pub active_users_24h: u64,
    pub total_forum_posts: u64,
    pub total_forum_topics: u64,
    pub total_kb_articles: u64,
    pub total_portal_views: u64,
    pub new_users_today: u64,
    pub posts_today: u64,
}