//! Friend System for OpenSim Next Social Features
//! 
//! Provides comprehensive friend management including friend requests, acceptance,
//! blocking, online status tracking, and friend-based privacy controls.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Friend system manager
pub struct FriendSystem {
    database: Arc<DatabaseManager>,
    config: SocialConfig,
    active_friendships: Arc<RwLock<HashMap<Uuid, Vec<Friendship>>>>,
    friend_requests: Arc<RwLock<HashMap<Uuid, Vec<FriendRequest>>>>,
    blocked_users: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    rate_limiter: Arc<RwLock<HashMap<Uuid, FriendRequestRateLimit>>>,
}

/// Friendship relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friendship {
    pub friendship_id: Uuid,
    pub user_id: Uuid,
    pub friend_id: Uuid,
    pub friendship_status: FriendshipStatus,
    pub friendship_type: FriendshipType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Friendship status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FriendshipStatus {
    Active,
    Blocked,
    Muted,
    Archived,
}

/// Types of friendships
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FriendshipType {
    Regular,
    BestFriend,
    Family,
    Colleague,
    Acquaintance,
}

/// Friend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequest {
    pub request_id: Uuid,
    pub requester_id: Uuid,
    pub target_id: Uuid,
    pub message: Option<String>,
    pub status: FriendRequestStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Friend request status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FriendRequestStatus {
    Pending,
    Accepted,
    Declined,
    Cancelled,
    Expired,
}

/// Friend request rate limiting
#[derive(Debug, Clone)]
struct FriendRequestRateLimit {
    user_id: Uuid,
    requests_this_hour: u32,
    last_reset: std::time::Instant,
}

/// Friend list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendListResponse {
    pub friends: Vec<FriendInfo>,
    pub total_count: u32,
    pub online_count: u32,
    pub mutual_friends: HashMap<Uuid, u32>,
}

/// Friend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendInfo {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_image: Option<String>,
    pub online_status: OnlineStatus,
    pub friendship: Friendship,
    pub mutual_friend_count: u32,
    pub last_online: Option<DateTime<Utc>>,
}

/// Friend activity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendActivity {
    pub user_id: Uuid,
    pub activity_type: FriendActivityType,
    pub content: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub visibility: VisibilityLevel,
}

/// Types of friend activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FriendActivityType {
    StatusUpdate,
    LocationChange,
    GroupJoin,
    Achievement,
    ContentShare,
}

/// Friend search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendSearchCriteria {
    pub query: Option<String>,
    pub online_only: bool,
    pub friendship_types: Vec<FriendshipType>,
    pub sort_by: FriendSortOption,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Friend sort options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FriendSortOption {
    Name,
    OnlineStatus,
    LastOnline,
    FriendshipDate,
    MutualFriends,
}

impl FriendSystem {
    /// Create new friend system
    pub fn new(database: Arc<DatabaseManager>, config: SocialConfig) -> Self {
        Self {
            database,
            config,
            active_friendships: Arc::new(RwLock::new(HashMap::new())),
            friend_requests: Arc::new(RwLock::new(HashMap::new())),
            blocked_users: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize friend system
    pub async fn initialize(&self) -> SocialResult<()> {
        info!("Initializing friend system");

        // Create database tables
        self.create_tables().await?;

        // Load active friendships into cache
        self.load_active_friendships().await?;

        info!("Friend system initialized successfully");
        Ok(())
    }

    /// Send friend request
    pub async fn send_friend_request(
        &self,
        requester_id: Uuid,
        target_id: Uuid,
        message: Option<String>,
    ) -> SocialResult<FriendRequest> {
        info!("Sending friend request from {} to {}", requester_id, target_id);

        // Validate request
        self.validate_friend_request(requester_id, target_id).await?;

        // Check rate limits
        self.check_friend_request_rate_limit(requester_id).await?;

        // Check if users are already friends
        if self.are_friends(requester_id, target_id).await? {
            return Err(SocialError::FriendshipExists);
        }

        // Check if target user allows friend requests
        if !self.allows_friend_requests(target_id).await? {
            return Err(SocialError::AccessDenied {
                reason: "User does not accept friend requests".to_string(),
            });
        }

        // Check if user is blocked
        if self.is_blocked(target_id, requester_id).await? {
            return Err(SocialError::AccessDenied {
                reason: "You are blocked by this user".to_string(),
            });
        }

        // Create friend request
        let friend_request = FriendRequest {
            request_id: Uuid::new_v4(),
            requester_id,
            target_id,
            message,
            status: FriendRequestStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(30),
        };

        // Save to database
        self.save_friend_request(&friend_request).await?;

        // Add to cache
        {
            let mut requests = self.friend_requests.write().await;
            requests.entry(target_id).or_insert_with(Vec::new).push(friend_request.clone());
        }

        // Update rate limiter
        self.update_friend_request_rate_limit(requester_id).await;

        info!("Friend request sent successfully: {}", friend_request.request_id);
        Ok(friend_request)
    }

    /// Accept friend request
    pub async fn accept_friend_request(&self, request_id: Uuid, target_id: Uuid) -> SocialResult<Friendship> {
        info!("Accepting friend request {} by user {}", request_id, target_id);

        // Get and validate friend request
        let friend_request = self.get_friend_request(request_id).await?;
        
        if friend_request.target_id != target_id {
            return Err(SocialError::AccessDenied {
                reason: "Only the target user can accept this friend request".to_string(),
            });
        }

        if friend_request.status != FriendRequestStatus::Pending {
            return Err(SocialError::InvalidInvitation {
                reason: "Friend request is not pending".to_string(),
            });
        }

        if friend_request.expires_at < Utc::now() {
            return Err(SocialError::InvalidInvitation {
                reason: "Friend request has expired".to_string(),
            });
        }

        // Create friendship
        let friendship = Friendship {
            friendship_id: Uuid::new_v4(),
            user_id: friend_request.requester_id,
            friend_id: friend_request.target_id,
            friendship_status: FriendshipStatus::Active,
            friendship_type: FriendshipType::Regular,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        // Save friendship (bidirectional)
        self.save_friendship(&friendship).await?;
        
        // Create reverse friendship
        let reverse_friendship = Friendship {
            friendship_id: Uuid::new_v4(),
            user_id: friend_request.target_id,
            friend_id: friend_request.requester_id,
            friendship_status: FriendshipStatus::Active,
            friendship_type: FriendshipType::Regular,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        self.save_friendship(&reverse_friendship).await?;

        // Update friend request status
        let mut updated_request = friend_request;
        updated_request.status = FriendRequestStatus::Accepted;
        self.save_friend_request(&updated_request).await?;

        // Update caches
        {
            let mut friendships = self.active_friendships.write().await;
            friendships.entry(friendship.user_id).or_insert_with(Vec::new).push(friendship.clone());
            friendships.entry(reverse_friendship.user_id).or_insert_with(Vec::new).push(reverse_friendship);
        }

        // Remove from pending requests
        {
            let mut requests = self.friend_requests.write().await;
            if let Some(user_requests) = requests.get_mut(&target_id) {
                user_requests.retain(|r| r.request_id != request_id);
            }
        }

        info!("Friend request accepted successfully");
        Ok(friendship)
    }

    /// Decline friend request
    pub async fn decline_friend_request(&self, request_id: Uuid, target_id: Uuid) -> SocialResult<()> {
        info!("Declining friend request {} by user {}", request_id, target_id);

        // Get and validate friend request
        let mut friend_request = self.get_friend_request(request_id).await?;
        
        if friend_request.target_id != target_id {
            return Err(SocialError::AccessDenied {
                reason: "Only the target user can decline this friend request".to_string(),
            });
        }

        if friend_request.status != FriendRequestStatus::Pending {
            return Err(SocialError::InvalidInvitation {
                reason: "Friend request is not pending".to_string(),
            });
        }

        // Update request status
        friend_request.status = FriendRequestStatus::Declined;
        self.save_friend_request(&friend_request).await?;

        // Remove from pending requests
        {
            let mut requests = self.friend_requests.write().await;
            if let Some(user_requests) = requests.get_mut(&target_id) {
                user_requests.retain(|r| r.request_id != request_id);
            }
        }

        info!("Friend request declined successfully");
        Ok(())
    }

    /// Remove friend
    pub async fn remove_friend(&self, user_id: Uuid, friend_id: Uuid) -> SocialResult<()> {
        info!("Removing friendship between {} and {}", user_id, friend_id);

        // Verify friendship exists
        if !self.are_friends(user_id, friend_id).await? {
            return Err(SocialError::AccessDenied {
                reason: "Friendship does not exist".to_string(),
            });
        }

        // Remove both directions of friendship
        self.delete_friendship(user_id, friend_id).await?;
        self.delete_friendship(friend_id, user_id).await?;

        // Update caches
        {
            let mut friendships = self.active_friendships.write().await;
            if let Some(user_friends) = friendships.get_mut(&user_id) {
                user_friends.retain(|f| f.friend_id != friend_id);
            }
            if let Some(friend_friends) = friendships.get_mut(&friend_id) {
                friend_friends.retain(|f| f.friend_id != user_id);
            }
        }

        info!("Friendship removed successfully");
        Ok(())
    }

    /// Block user
    pub async fn block_user(&self, user_id: Uuid, blocked_id: Uuid) -> SocialResult<()> {
        info!("User {} blocking user {}", user_id, blocked_id);

        if user_id == blocked_id {
            return Err(SocialError::ValidationError {
                message: "Cannot block yourself".to_string(),
            });
        }

        // Remove friendship if exists
        if self.are_friends(user_id, blocked_id).await? {
            self.remove_friend(user_id, blocked_id).await?;
        }

        // Add to blocked list
        self.save_blocked_user(user_id, blocked_id).await?;

        // Update cache
        {
            let mut blocked = self.blocked_users.write().await;
            blocked.entry(user_id).or_insert_with(Vec::new).push(blocked_id);
        }

        info!("User blocked successfully");
        Ok(())
    }

    /// Unblock user
    pub async fn unblock_user(&self, user_id: Uuid, blocked_id: Uuid) -> SocialResult<()> {
        info!("User {} unblocking user {}", user_id, blocked_id);

        // Remove from blocked list
        self.delete_blocked_user(user_id, blocked_id).await?;

        // Update cache
        {
            let mut blocked = self.blocked_users.write().await;
            if let Some(user_blocked) = blocked.get_mut(&user_id) {
                user_blocked.retain(|&id| id != blocked_id);
            }
        }

        info!("User unblocked successfully");
        Ok(())
    }

    /// Get friend list
    pub async fn get_friend_list(&self, user_id: Uuid) -> SocialResult<FriendListResponse> {
        debug!("Getting friend list for user {}", user_id);

        let friendships = self.active_friendships.read().await;
        let user_friends = friendships.get(&user_id).cloned().unwrap_or_default();
        drop(friendships);

        let mut friends = Vec::new();
        let mut online_count = 0;
        let mut mutual_friends = HashMap::new();

        for friendship in user_friends {
            if friendship.friendship_status == FriendshipStatus::Active {
                let display_name = self.get_user_display_name(friendship.friend_id).await;
                let friend_info = FriendInfo {
                    user_id: friendship.friend_id,
                    display_name,
                    avatar_image: None,
                    online_status: OnlineStatus::Offline,
                    friendship,
                    mutual_friend_count: 0,
                    last_online: None,
                };

                if matches!(friend_info.online_status, OnlineStatus::Online) {
                    online_count += 1;
                }

                friends.push(friend_info);
            }
        }

        let response = FriendListResponse {
            total_count: friends.len() as u32,
            online_count,
            mutual_friends,
            friends,
        };

        debug!("Friend list retrieved: {} friends, {} online", response.total_count, response.online_count);
        Ok(response)
    }

    /// Get pending friend requests
    pub async fn get_pending_requests(&self, user_id: Uuid) -> SocialResult<Vec<FriendRequest>> {
        debug!("Getting pending friend requests for user {}", user_id);

        let requests = self.friend_requests.read().await;
        let user_requests = requests.get(&user_id).cloned().unwrap_or_default();

        let pending_requests: Vec<FriendRequest> = user_requests
            .into_iter()
            .filter(|r| r.status == FriendRequestStatus::Pending && r.expires_at > Utc::now())
            .collect();

        debug!("Found {} pending friend requests", pending_requests.len());
        Ok(pending_requests)
    }

    /// Get blocked users
    pub async fn get_blocked_users(&self, user_id: Uuid) -> SocialResult<Vec<Uuid>> {
        debug!("Getting blocked users for user {}", user_id);

        let blocked = self.blocked_users.read().await;
        let user_blocked = blocked.get(&user_id).cloned().unwrap_or_default();

        debug!("Found {} blocked users", user_blocked.len());
        Ok(user_blocked)
    }

    /// Search friends
    pub async fn search_friends(&self, user_id: Uuid, criteria: FriendSearchCriteria) -> SocialResult<Vec<FriendInfo>> {
        debug!("Searching friends for user {} with criteria: {:?}", user_id, criteria);

        let friend_list = self.get_friend_list(user_id).await?;
        let mut results = friend_list.friends;

        // Apply filters
        if let Some(query) = &criteria.query {
            let query_lower = query.to_lowercase();
            results.retain(|f| f.display_name.to_lowercase().contains(&query_lower));
        }

        if criteria.online_only {
            results.retain(|f| matches!(f.online_status, OnlineStatus::Online));
        }

        if !criteria.friendship_types.is_empty() {
            results.retain(|f| criteria.friendship_types.contains(&f.friendship.friendship_type));
        }

        // Apply sorting
        match criteria.sort_by {
            FriendSortOption::Name => {
                results.sort_by(|a, b| a.display_name.cmp(&b.display_name));
            }
            FriendSortOption::OnlineStatus => {
                results.sort_by(|a, b| {
                    let a_priority = match a.online_status {
                        OnlineStatus::Online => 0,
                        OnlineStatus::Away => 1,
                        OnlineStatus::Busy => 2,
                        OnlineStatus::Invisible => 3,
                        OnlineStatus::Offline => 4,
                    };
                    let b_priority = match b.online_status {
                        OnlineStatus::Online => 0,
                        OnlineStatus::Away => 1,
                        OnlineStatus::Busy => 2,
                        OnlineStatus::Invisible => 3,
                        OnlineStatus::Offline => 4,
                    };
                    a_priority.cmp(&b_priority)
                });
            }
            FriendSortOption::FriendshipDate => {
                results.sort_by(|a, b| b.friendship.created_at.cmp(&a.friendship.created_at));
            }
            FriendSortOption::MutualFriends => {
                results.sort_by(|a, b| b.mutual_friend_count.cmp(&a.mutual_friend_count));
            }
            FriendSortOption::LastOnline => {
                results.sort_by(|a, b| {
                    match (a.last_online.as_ref(), b.last_online.as_ref()) {
                        (Some(a_time), Some(b_time)) => b_time.cmp(a_time),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
            }
        }

        // Apply pagination
        if let Some(offset) = criteria.offset {
            if offset as usize >= results.len() {
                results.clear();
            } else {
                results = results.into_iter().skip(offset as usize).collect();
            }
        }

        if let Some(limit) = criteria.limit {
            results.truncate(limit as usize);
        }

        debug!("Friend search returned {} results", results.len());
        Ok(results)
    }

    /// Check if two users are friends
    pub async fn are_friends(&self, user_id: Uuid, friend_id: Uuid) -> SocialResult<bool> {
        let friendships = self.active_friendships.read().await;
        
        if let Some(user_friends) = friendships.get(&user_id) {
            for friendship in user_friends {
                if friendship.friend_id == friend_id && friendship.friendship_status == FriendshipStatus::Active {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if user is blocked by another user
    pub async fn is_blocked(&self, user_id: Uuid, potential_blocker: Uuid) -> SocialResult<bool> {
        let blocked = self.blocked_users.read().await;
        
        if let Some(user_blocked) = blocked.get(&potential_blocker) {
            return Ok(user_blocked.contains(&user_id));
        }

        Ok(false)
    }

    // Private helper methods

    async fn validate_friend_request(&self, requester_id: Uuid, target_id: Uuid) -> SocialResult<()> {
        if requester_id == target_id {
            return Err(SocialError::ValidationError {
                message: "Cannot send friend request to yourself".to_string(),
            });
        }

        // Check if there's already a pending request
        let requests = self.friend_requests.read().await;
        if let Some(target_requests) = requests.get(&target_id) {
            for request in target_requests {
                if request.requester_id == requester_id && request.status == FriendRequestStatus::Pending {
                    return Err(SocialError::ValidationError {
                        message: "Friend request already pending".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    async fn check_friend_request_rate_limit(&self, user_id: Uuid) -> SocialResult<()> {
        let mut rate_limiter = self.rate_limiter.write().await;
        let now = std::time::Instant::now();

        let user_limit = rate_limiter.entry(user_id).or_insert_with(|| FriendRequestRateLimit {
            user_id,
            requests_this_hour: 0,
            last_reset: now,
        });

        // Reset if an hour has passed
        if now.duration_since(user_limit.last_reset).as_secs() >= 3600 {
            user_limit.requests_this_hour = 0;
            user_limit.last_reset = now;
        }

        if user_limit.requests_this_hour >= self.config.friend_request_rate_limit {
            return Err(SocialError::RateLimitExceeded { user_id });
        }

        Ok(())
    }

    async fn update_friend_request_rate_limit(&self, user_id: Uuid) {
        let mut rate_limiter = self.rate_limiter.write().await;
        if let Some(user_limit) = rate_limiter.get_mut(&user_id) {
            user_limit.requests_this_hour += 1;
        }
    }

    async fn allows_friend_requests(&self, _user_id: Uuid) -> SocialResult<bool> {
        // Would check user's privacy settings
        Ok(true)
    }

    async fn get_friend_request(&self, request_id: Uuid) -> SocialResult<FriendRequest> {
        // Implementation would query database
        Err(SocialError::ValidationError {
            message: "Friend request not found".to_string(),
        })
    }

    // Database operations (placeholder implementations)

    async fn create_tables(&self) -> SocialResult<()> {
        // Would create friendship, friend_request, and blocked_user tables
        Ok(())
    }

    async fn load_active_friendships(&self) -> SocialResult<()> {
        // Would load all active friendships from database into cache
        Ok(())
    }

    async fn save_friend_request(&self, _request: &FriendRequest) -> SocialResult<()> {
        // Would save friend request to database
        Ok(())
    }

    async fn save_friendship(&self, _friendship: &Friendship) -> SocialResult<()> {
        // Would save friendship to database
        Ok(())
    }

    async fn delete_friendship(&self, _user_id: Uuid, _friend_id: Uuid) -> SocialResult<()> {
        // Would delete friendship from database
        Ok(())
    }

    async fn save_blocked_user(&self, _user_id: Uuid, _blocked_id: Uuid) -> SocialResult<()> {
        // Would save blocked user relationship to database
        Ok(())
    }

    async fn delete_blocked_user(&self, _user_id: Uuid, _blocked_id: Uuid) -> SocialResult<()> {
        // Would delete blocked user relationship from database
        Ok(())
    }

    async fn get_user_display_name(&self, user_id: Uuid) -> String {
        if let Ok(pool) = self.database.legacy_pool() {
            let row_result = sqlx::query(
                "SELECT FirstName, LastName FROM UserAccounts WHERE PrincipalID = $1"
            )
            .bind(user_id.to_string())
            .fetch_optional(pool)
            .await;

            if let Ok(Some(row)) = row_result {
                let first_name: String = row.try_get("FirstName").unwrap_or_else(|_| "Unknown".to_string());
                let last_name: String = row.try_get("LastName").unwrap_or_else(|_| "User".to_string());
                return format!("{} {}", first_name, last_name);
            }
        }
        format!("User {}", &user_id.to_string()[..8])
    }
}

impl Default for FriendshipType {
    fn default() -> Self {
        Self::Regular
    }
}

impl Default for FriendSortOption {
    fn default() -> Self {
        Self::Name
    }
}

impl Default for FriendSearchCriteria {
    fn default() -> Self {
        Self {
            query: None,
            online_only: false,
            friendship_types: Vec::new(),
            sort_by: FriendSortOption::default(),
            limit: Some(50),
            offset: None,
        }
    }
}