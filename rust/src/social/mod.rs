//! Advanced Social Features for OpenSim Next
//!
//! Provides comprehensive social networking capabilities including community management,
//! friend systems, group functionality, communication tools, and moderation features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod community;
pub mod friends;
pub mod groups;
pub mod manager;
pub mod messaging;
pub mod moderation;
pub mod notifications;

// Re-export main components
pub use community::*;
pub use friends::*;
pub use groups::*;
pub use manager::*;
pub use messaging::*;
pub use moderation::*;
pub use notifications::*;

/// Social feature error types
#[derive(Debug, thiserror::Error)]
pub enum SocialError {
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: Uuid },

    #[error("Group not found: {group_id}")]
    GroupNotFound { group_id: Uuid },

    #[error("Community not found: {community_id}")]
    CommunityNotFound { community_id: Uuid },

    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },

    #[error("Friendship already exists")]
    FriendshipExists,

    #[error("Group membership limit exceeded")]
    GroupMembershipLimitExceeded,

    #[error("Invalid invitation: {reason}")]
    InvalidInvitation { reason: String },

    #[error("Moderation action failed: {reason}")]
    ModerationFailed { reason: String },

    #[error("Database error: {source}")]
    DatabaseError { source: anyhow::Error },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Rate limit exceeded for user {user_id}")]
    RateLimitExceeded { user_id: Uuid },
}

/// Social system result type
pub type SocialResult<T> = Result<T, SocialError>;

/// User social profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSocialProfile {
    pub user_id: Uuid,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_image: Option<String>,
    pub status_message: Option<String>,
    pub online_status: OnlineStatus,
    pub privacy_settings: PrivacySettings,
    pub social_statistics: SocialStatistics,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User online status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnlineStatus {
    Online,
    Away,
    Busy,
    Invisible,
    Offline,
}

/// Privacy settings for user social profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub profile_visibility: VisibilityLevel,
    pub friend_list_visibility: VisibilityLevel,
    pub group_membership_visibility: VisibilityLevel,
    pub online_status_visibility: VisibilityLevel,
    pub allow_friend_requests: bool,
    pub allow_group_invitations: bool,
    pub allow_direct_messages: MessagePermission,
}

/// Visibility levels for profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisibilityLevel {
    Public,
    Friends,
    Private,
}

/// Message permission levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePermission {
    Everyone,
    Friends,
    Nobody,
}

/// Social statistics for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialStatistics {
    pub friend_count: u32,
    pub group_membership_count: u32,
    pub community_membership_count: u32,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub posts_created: u64,
    pub comments_made: u64,
    pub likes_given: u64,
    pub likes_received: u64,
    pub reputation_score: i32,
}

/// Social activity types for tracking and notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialActivityType {
    FriendRequest,
    FriendAccepted,
    GroupInvitation,
    GroupJoined,
    GroupLeft,
    CommunityJoined,
    CommunityLeft,
    MessageSent,
    MessageReceived,
    PostCreated,
    PostLiked,
    CommentMade,
    CommentLiked,
    StatusUpdated,
    ProfileUpdated,
}

/// Social activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialActivity {
    pub activity_id: Uuid,
    pub user_id: Uuid,
    pub activity_type: SocialActivityType,
    pub target_user_id: Option<Uuid>,
    pub target_group_id: Option<Uuid>,
    pub target_community_id: Option<Uuid>,
    pub content: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

/// Social feed item for activity streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialFeedItem {
    pub feed_id: Uuid,
    pub user_id: Uuid,
    pub activity: SocialActivity,
    pub visibility: VisibilityLevel,
    pub interactions: FeedInteractions,
    pub created_at: DateTime<Utc>,
}

/// Interactions on social feed items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedInteractions {
    pub likes: Vec<Uuid>,
    pub comments: Vec<FeedComment>,
    pub shares: Vec<Uuid>,
    pub reactions: HashMap<String, Vec<Uuid>>, // emoji -> user_ids
}

/// Comment on a social feed item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedComment {
    pub comment_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub likes: Vec<Uuid>,
}

/// Social recommendation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    FriendSuggestion,
    GroupSuggestion,
    CommunitySuggestion,
    ContentSuggestion,
    EventSuggestion,
}

/// Social recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialRecommendation {
    pub recommendation_id: Uuid,
    pub user_id: Uuid,
    pub recommendation_type: RecommendationType,
    pub target_id: Uuid,
    pub confidence_score: f32,
    pub reason: String,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Social search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSearchCriteria {
    pub query: Option<String>,
    pub search_type: SocialSearchType,
    pub filters: SocialSearchFilters,
    pub sort_by: SocialSortOption,
    pub sort_order: SortOrder,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Types of social searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialSearchType {
    Users,
    Groups,
    Communities,
    Posts,
    All,
}

/// Filters for social searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSearchFilters {
    pub online_only: bool,
    pub friends_only: bool,
    pub location_filter: Option<String>,
    pub interests: Vec<String>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

/// Sort options for social search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialSortOption {
    Relevance,
    Recent,
    Popular,
    Alphabetical,
    Distance,
    MutualFriends,
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Social configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialConfig {
    pub max_friends_per_user: u32,
    pub max_groups_per_user: u32,
    pub max_communities_per_user: u32,
    pub enable_recommendations: bool,
    pub enable_activity_feeds: bool,
    pub enable_social_search: bool,
    pub message_rate_limit: u32,
    pub friend_request_rate_limit: u32,
    pub auto_cleanup_inactive_days: u32,
    pub moderation_enabled: bool,
    pub content_filtering_enabled: bool,
}

impl Default for SocialConfig {
    fn default() -> Self {
        Self {
            max_friends_per_user: 1000,
            max_groups_per_user: 100,
            max_communities_per_user: 50,
            enable_recommendations: true,
            enable_activity_feeds: true,
            enable_social_search: true,
            message_rate_limit: 60,        // messages per minute
            friend_request_rate_limit: 10, // requests per hour
            auto_cleanup_inactive_days: 365,
            moderation_enabled: true,
            content_filtering_enabled: true,
        }
    }
}

impl Default for OnlineStatus {
    fn default() -> Self {
        Self::Offline
    }
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            profile_visibility: VisibilityLevel::Public,
            friend_list_visibility: VisibilityLevel::Friends,
            group_membership_visibility: VisibilityLevel::Friends,
            online_status_visibility: VisibilityLevel::Friends,
            allow_friend_requests: true,
            allow_group_invitations: true,
            allow_direct_messages: MessagePermission::Friends,
        }
    }
}

impl Default for SocialStatistics {
    fn default() -> Self {
        Self {
            friend_count: 0,
            group_membership_count: 0,
            community_membership_count: 0,
            messages_sent: 0,
            messages_received: 0,
            posts_created: 0,
            comments_made: 0,
            likes_given: 0,
            likes_received: 0,
            reputation_score: 0,
        }
    }
}

impl Default for FeedInteractions {
    fn default() -> Self {
        Self {
            likes: Vec::new(),
            comments: Vec::new(),
            shares: Vec::new(),
            reactions: HashMap::new(),
        }
    }
}

impl Default for SocialSortOption {
    fn default() -> Self {
        Self::Recent
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Descending
    }
}

impl Default for SocialSearchFilters {
    fn default() -> Self {
        Self {
            online_only: false,
            friends_only: false,
            location_filter: None,
            interests: Vec::new(),
            date_range: None,
        }
    }
}

impl Default for SocialSearchCriteria {
    fn default() -> Self {
        Self {
            query: None,
            search_type: SocialSearchType::All,
            filters: SocialSearchFilters::default(),
            sort_by: SocialSortOption::default(),
            sort_order: SortOrder::default(),
            limit: Some(20),
            offset: None,
        }
    }
}
