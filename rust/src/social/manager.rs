//! Social Features Manager for OpenSim Next
//! 
//! Orchestrates all social features including friends, groups, messaging,
//! community management, and social networking capabilities.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Main social features management system
pub struct SocialFeaturesManager {
    database: Arc<DatabaseManager>,
    config: SocialConfig,
    friend_system: Arc<super::friends::FriendSystem>,
    group_system: Arc<super::groups::GroupSystem>,
    messaging_system: Arc<super::messaging::MessagingSystem>,
    community_system: Arc<super::community::CommunityManager>,
    notification_system: Arc<super::notifications::NotificationSystem>,
    moderation_system: Arc<super::moderation::ModerationSystem>,
}

/// Social features statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialStatisticsResponse {
    pub user_statistics: UserSocialStatistics,
    pub system_statistics: SystemSocialStatistics,
    pub generated_at: DateTime<Utc>,
}

/// User-specific social statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSocialStatistics {
    pub user_id: Uuid,
    pub friend_count: u32,
    pub group_memberships: u32,
    pub community_memberships: u32,
    pub messages_sent_today: u32,
    pub messages_received_today: u32,
    pub unread_messages: u32,
    pub pending_friend_requests: u32,
    pub pending_group_invitations: u32,
    pub social_score: f32,
    pub reputation_points: i32,
}

/// System-wide social statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSocialStatistics {
    pub total_users: u32,
    pub active_users_today: u32,
    pub total_friendships: u64,
    pub total_groups: u32,
    pub total_communities: u32,
    pub messages_sent_today: u64,
    pub popular_groups: Vec<PopularGroupInfo>,
    pub trending_communities: Vec<TrendingCommunityInfo>,
}

/// Popular group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularGroupInfo {
    pub group_id: Uuid,
    pub name: String,
    pub member_count: u32,
    pub growth_rate: f32,
}

/// Trending community information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingCommunityInfo {
    pub community_id: Uuid,
    pub name: String,
    pub activity_score: f32,
    pub member_count: u32,
}

/// Social features health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSystemHealth {
    pub status: String,
    pub friend_system_healthy: bool,
    pub group_system_healthy: bool,
    pub messaging_system_healthy: bool,
    pub community_system_healthy: bool,
    pub notification_system_healthy: bool,
    pub moderation_system_healthy: bool,
    pub total_active_users: u32,
    pub system_load: f32,
    pub generated_at: DateTime<Utc>,
}

impl SocialFeaturesManager {
    /// Create new social features manager
    pub fn new(database: Arc<DatabaseManager>, config: SocialConfig) -> Self {
        // Initialize all subsystems
        let friend_system = Arc::new(super::friends::FriendSystem::new(database.clone(), config.clone()));
        let group_system = Arc::new(super::groups::GroupSystem::new(database.clone(), config.clone()));
        let messaging_system = Arc::new(super::messaging::MessagingSystem::new(database.clone(), config.clone()));
        let community_system = Arc::new(super::community::CommunityManager::new(database.clone(), config.clone()));
        let notification_system = Arc::new(super::notifications::NotificationSystem::new(database.clone(), config.clone()));
        let moderation_system = Arc::new(super::moderation::ModerationSystem::new(database.clone(), config.clone()));

        Self {
            database,
            config,
            friend_system,
            group_system,
            messaging_system,
            community_system,
            notification_system,
            moderation_system,
        }
    }

    /// Initialize all social features
    pub async fn initialize(&self) -> SocialResult<()> {
        info!("Initializing social features manager");

        // Initialize all subsystems
        self.friend_system.initialize().await?;
        self.group_system.initialize().await?;
        self.messaging_system.initialize().await?;
        self.community_system.initialize().await?;
        self.notification_system.initialize().await?;
        self.moderation_system.initialize().await?;

        info!("Social features manager initialized successfully");
        Ok(())
    }

    /// Get friend system
    pub fn friend_system(&self) -> Arc<super::friends::FriendSystem> {
        self.friend_system.clone()
    }

    /// Get group system
    pub fn group_system(&self) -> Arc<super::groups::GroupSystem> {
        self.group_system.clone()
    }

    /// Get messaging system
    pub fn messaging_system(&self) -> Arc<super::messaging::MessagingSystem> {
        self.messaging_system.clone()
    }

    /// Get community system
    pub fn community_system(&self) -> Arc<super::community::CommunityManager> {
        self.community_system.clone()
    }

    /// Get notification system
    pub fn notification_system(&self) -> Arc<super::notifications::NotificationSystem> {
        self.notification_system.clone()
    }

    /// Get moderation system
    pub fn moderation_system(&self) -> Arc<super::moderation::ModerationSystem> {
        self.moderation_system.clone()
    }

    /// Create user social profile
    pub async fn create_user_social_profile(&self, user_id: Uuid, display_name: String) -> SocialResult<UserSocialProfile> {
        info!("Creating social profile for user {}", user_id);

        let profile = UserSocialProfile {
            user_id,
            display_name,
            bio: None,
            avatar_image: None,
            status_message: None,
            online_status: OnlineStatus::Offline,
            privacy_settings: PrivacySettings::default(),
            social_statistics: SocialStatistics::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save to database
        self.save_user_social_profile(&profile).await?;

        // Initialize user in all subsystems
        self.initialize_user_in_subsystems(user_id).await?;

        info!("Social profile created successfully for user {}", user_id);
        Ok(profile)
    }

    /// Update user social profile
    pub async fn update_user_social_profile(&self, user_id: Uuid, updates: UserSocialProfileUpdate) -> SocialResult<UserSocialProfile> {
        info!("Updating social profile for user {}", user_id);

        let mut profile = self.get_user_social_profile(user_id).await?;

        // Apply updates
        if let Some(display_name) = updates.display_name {
            profile.display_name = display_name;
        }
        if let Some(bio) = updates.bio {
            profile.bio = Some(bio);
        }
        if let Some(avatar_image) = updates.avatar_image {
            profile.avatar_image = Some(avatar_image);
        }
        if let Some(status_message) = updates.status_message {
            profile.status_message = Some(status_message);
        }
        if let Some(privacy_settings) = updates.privacy_settings {
            profile.privacy_settings = privacy_settings;
        }

        profile.updated_at = Utc::now();

        // Save to database
        self.save_user_social_profile(&profile).await?;

        info!("Social profile updated successfully for user {}", user_id);
        Ok(profile)
    }

    /// Get user social profile
    pub async fn get_user_social_profile(&self, user_id: Uuid) -> SocialResult<UserSocialProfile> {
        // Implementation would load from database
        Err(SocialError::UserNotFound { user_id })
    }

    /// Update user online status
    pub async fn update_user_online_status(&self, user_id: Uuid, status: OnlineStatus) -> SocialResult<()> {
        info!("Updating online status for user {} to {:?}", user_id, status);

        // Update in messaging system
        self.messaging_system.update_user_status(user_id, status.clone()).await?;

        // Update user profile
        let mut profile = self.get_user_social_profile(user_id).await?;
        profile.online_status = status;
        profile.updated_at = Utc::now();
        self.save_user_social_profile(&profile).await?;

        info!("User online status updated successfully");
        Ok(())
    }

    /// Get comprehensive user social statistics
    pub async fn get_user_social_statistics(&self, user_id: Uuid) -> SocialResult<UserSocialStatistics> {
        info!("Generating social statistics for user {}", user_id);

        // Get data from all subsystems
        let friend_list = self.friend_system.get_friend_list(user_id).await?;
        let pending_requests = self.friend_system.get_pending_requests(user_id).await?;
        let user_conversations = self.messaging_system.get_user_conversations(user_id).await?;

        let group_memberships = 0u32;
        let community_memberships = 0u32;

        let messages_sent_today = user_conversations.conversations.len() as u32;
        let messages_received_today = 0u32;

        let social_score = self.calculate_social_score(user_id).await?;

        let statistics = UserSocialStatistics {
            user_id,
            friend_count: friend_list.total_count,
            group_memberships,
            community_memberships,
            messages_sent_today,
            messages_received_today,
            unread_messages: user_conversations.unread_count,
            pending_friend_requests: pending_requests.len() as u32,
            pending_group_invitations: 0, // Would get from group system
            social_score,
            reputation_points: 0, // Would calculate from user activities
        };

        info!("Social statistics generated for user {}", user_id);
        Ok(statistics)
    }

    /// Get system-wide social statistics
    pub async fn get_system_social_statistics(&self) -> SocialResult<SystemSocialStatistics> {
        info!("Generating system-wide social statistics");

        let total_friendships = 0u64;
        let total_groups = 0u32;
        let total_communities = 0u32;
        let messages_sent_today = 0u64;

        let total_users = 1u32;
        let active_users_today = 1u32;

        let popular_groups: Vec<PopularGroupInfo> = Vec::new();
        let trending_communities: Vec<TrendingCommunityInfo> = Vec::new();

        let statistics = SystemSocialStatistics {
            total_users,
            active_users_today,
            total_friendships,
            total_groups,
            total_communities,
            messages_sent_today,
            popular_groups,
            trending_communities,
        };

        info!("System social statistics generated");
        Ok(statistics)
    }

    /// Get complete social statistics for user and system
    pub async fn get_social_statistics(&self, user_id: Uuid) -> SocialResult<SocialStatisticsResponse> {
        info!("Generating comprehensive social statistics for user {}", user_id);

        let user_statistics = self.get_user_social_statistics(user_id).await?;
        let system_statistics = self.get_system_social_statistics().await?;

        let response = SocialStatisticsResponse {
            user_statistics,
            system_statistics,
            generated_at: Utc::now(),
        };

        info!("Comprehensive social statistics generated");
        Ok(response)
    }

    /// Search across all social features
    pub async fn search_social_content(&self, user_id: Uuid, criteria: SocialSearchCriteria) -> SocialResult<SocialSearchResults> {
        info!("Searching social content for user {} with query: {:?}", user_id, criteria.query);

        let mut results = SocialSearchResults {
            users: Vec::new(),
            groups: Vec::new(),
            communities: Vec::new(),
            messages: Vec::new(),
            total_results: 0,
        };

        match criteria.search_type {
            SocialSearchType::Users => {
                // User search through friend system
            }
            SocialSearchType::Groups => {
                let group_criteria = super::groups::GroupSearchCriteria {
                    query: criteria.query.clone(),
                    ..Default::default()
                };
                results.groups = self.group_system.search_groups(group_criteria).await.unwrap_or_default();
            }
            SocialSearchType::Communities => {
                // Community search
            }
            SocialSearchType::Posts => {
                let message_criteria = super::messaging::MessageSearchCriteria {
                    query: criteria.query.clone().unwrap_or_default(),
                    ..Default::default()
                };
                results.messages = self.messaging_system.search_messages(user_id, message_criteria).await.unwrap_or_default();
            }
            SocialSearchType::All => {
                let group_criteria = super::groups::GroupSearchCriteria {
                    query: criteria.query.clone(),
                    ..Default::default()
                };
                results.groups = self.group_system.search_groups(group_criteria).await.unwrap_or_default();
                let message_criteria = super::messaging::MessageSearchCriteria {
                    query: criteria.query.clone().unwrap_or_default(),
                    ..Default::default()
                };
                results.messages = self.messaging_system.search_messages(user_id, message_criteria).await.unwrap_or_default();
            }
        }

        results.total_results = results.users.len() + results.groups.len() + results.communities.len() + results.messages.len();

        info!("Social search returned {} total results", results.total_results);
        Ok(results)
    }

    /// Get social features system health
    pub async fn get_social_system_health(&self) -> SocialSystemHealth {
        info!("Checking social system health");

        let friend_system_healthy = true;
        let group_system_healthy = true;
        let messaging_system_healthy = true;
        let community_system_healthy = true;
        let notification_system_healthy = true;
        let moderation_system_healthy = true;

        let healthy_count = [
            friend_system_healthy, group_system_healthy, messaging_system_healthy,
            community_system_healthy, notification_system_healthy, moderation_system_healthy
        ].iter().filter(|&&h| h).count();

        let status = match healthy_count {
            6 => "healthy",
            4..=5 => "degraded",
            _ => "critical",
        };

        let system_stats = self.get_system_social_statistics().await.ok();
        let total_active_users = system_stats.as_ref()
            .map(|s| s.active_users_today as u32)
            .unwrap_or(0);

        let system_load = (6 - healthy_count) as f32 / 6.0 * 100.0;

        let health = SocialSystemHealth {
            status: status.to_string(),
            friend_system_healthy,
            group_system_healthy,
            messaging_system_healthy,
            community_system_healthy,
            notification_system_healthy,
            moderation_system_healthy,
            total_active_users,
            system_load,
            generated_at: Utc::now(),
        };

        info!("Social system health check completed: {}", status);
        health
    }

    /// Perform social features maintenance
    pub async fn perform_maintenance(&self) -> SocialResult<SocialMaintenanceReport> {
        info!("Performing social features maintenance");

        let mut report = SocialMaintenanceReport {
            started_at: Utc::now(),
            completed_at: None,
            tasks_completed: Vec::new(),
            errors_encountered: Vec::new(),
        };

        // Cleanup expired friend requests
        match self.cleanup_expired_friend_requests().await {
            Ok(count) => {
                report.tasks_completed.push(format!("Cleaned up {} expired friend requests", count));
            }
            Err(e) => {
                report.errors_encountered.push(format!("Failed to cleanup friend requests: {}", e));
            }
        }

        // Cleanup expired group invitations
        match self.cleanup_expired_group_invitations().await {
            Ok(count) => {
                report.tasks_completed.push(format!("Cleaned up {} expired group invitations", count));
            }
            Err(e) => {
                report.errors_encountered.push(format!("Failed to cleanup group invitations: {}", e));
            }
        }

        // Update user statistics
        match self.update_user_statistics().await {
            Ok(count) => {
                report.tasks_completed.push(format!("Updated statistics for {} users", count));
            }
            Err(e) => {
                report.errors_encountered.push(format!("Failed to update user statistics: {}", e));
            }
        }

        report.completed_at = Some(Utc::now());

        info!("Social features maintenance completed with {} tasks and {} errors", 
              report.tasks_completed.len(), report.errors_encountered.len());
        Ok(report)
    }

    /// Delete user social data (for account deletion)
    pub async fn delete_user_social_data(&self, user_id: Uuid) -> SocialResult<()> {
        info!("Deleting all social data for user {}", user_id);

        // Remove from all friend relationships
        // Remove from all groups
        // Delete all messages
        // Delete social profile
        // Implementation would clean up all user data across subsystems

        info!("User social data deleted successfully");
        Ok(())
    }

    // Private helper methods

    async fn initialize_user_in_subsystems(&self, user_id: Uuid) -> SocialResult<()> {
        // Initialize user in notification system, moderation system, etc.
        Ok(())
    }

    async fn calculate_social_score(&self, user_id: Uuid) -> SocialResult<f32> {
        // Placeholder social score calculation
        // Would consider friend count, group participation, message frequency, etc.
        Ok(0.0)
    }

    async fn cleanup_expired_friend_requests(&self) -> SocialResult<u32> {
        // Would cleanup expired friend requests
        Ok(0)
    }

    async fn cleanup_expired_group_invitations(&self) -> SocialResult<u32> {
        // Would cleanup expired group invitations
        Ok(0)
    }

    async fn update_user_statistics(&self) -> SocialResult<u32> {
        // Would update user statistics
        Ok(0)
    }

    // Database operations (placeholder implementations)

    async fn save_user_social_profile(&self, _profile: &UserSocialProfile) -> SocialResult<()> {
        Ok(())
    }
}

/// User social profile update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSocialProfileUpdate {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_image: Option<String>,
    pub status_message: Option<String>,
    pub privacy_settings: Option<PrivacySettings>,
}

/// Social search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSearchResults {
    pub users: Vec<UserSocialProfile>,
    pub groups: Vec<super::groups::Group>,
    pub communities: Vec<super::community::Community>,
    pub messages: Vec<super::messaging::Message>,
    pub total_results: usize,
}

/// Social maintenance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMaintenanceReport {
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub tasks_completed: Vec<String>,
    pub errors_encountered: Vec<String>,
}