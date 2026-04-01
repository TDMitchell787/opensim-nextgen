//! Group System for OpenSim Next Social Features
//! 
//! Provides comprehensive group management including group creation, membership,
//! roles, permissions, activities, and group-based communication.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Group system manager
pub struct GroupSystem {
    database: Arc<DatabaseManager>,
    config: SocialConfig,
    active_groups: Arc<RwLock<HashMap<Uuid, Group>>>,
    group_memberships: Arc<RwLock<HashMap<Uuid, Vec<GroupMembership>>>>,
    group_invitations: Arc<RwLock<HashMap<Uuid, Vec<GroupInvitation>>>>,
    group_activities: Arc<RwLock<HashMap<Uuid, Vec<GroupActivity>>>>,
}

/// Virtual world group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub group_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub group_type: GroupType,
    pub visibility: GroupVisibility,
    pub membership_policy: GroupMembershipPolicy,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub member_count: u32,
    pub max_members: Option<u32>,
    pub tags: Vec<String>,
    pub group_image: Option<String>,
    pub group_charter: Option<String>,
    pub settings: GroupSettings,
    pub statistics: GroupStatistics,
}

/// Types of groups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GroupType {
    Public,
    Private,
    Secret,
    Professional,
    Educational,
    Gaming,
    Social,
    Business,
    Hobby,
}

/// Group visibility settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupVisibility {
    Public,        // Anyone can see and join
    Discoverable,  // Anyone can see, invite/apply to join
    Invite,        // Anyone can see, invite only
    Hidden,        // Only members can see
}

/// Group membership policies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupMembershipPolicy {
    Open,          // Anyone can join
    Application,   // Must apply and be approved
    Invitation,    // Invitation only
    Restricted,    // Owner approval required
}

/// Group settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    pub allow_member_invite: bool,
    pub allow_member_post: bool,
    pub allow_member_events: bool,
    pub moderated_posts: bool,
    pub require_approval_for_posts: bool,
    pub enable_group_chat: bool,
    pub enable_group_voice: bool,
    pub enable_group_land: bool,
    pub group_fee: Option<i64>,
    pub show_in_search: bool,
}

/// Group membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMembership {
    pub membership_id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: GroupRole,
    pub membership_status: GroupMembershipStatus,
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub contribution_points: u32,
    pub titles: Vec<String>,
    pub permissions: GroupPermissions,
}

/// Group roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GroupRole {
    Owner,
    Officer,
    Moderator,
    Member,
    Guest,
}

/// Group membership status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupMembershipStatus {
    Active,
    Inactive,
    Suspended,
    Banned,
    PendingApproval,
}

/// Group permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPermissions {
    pub can_invite_members: bool,
    pub can_remove_members: bool,
    pub can_send_notices: bool,
    pub can_start_meetings: bool,
    pub can_moderate_chat: bool,
    pub can_manage_land: bool,
    pub can_edit_group_info: bool,
    pub can_assign_roles: bool,
}

/// Group invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInvitation {
    pub invitation_id: Uuid,
    pub group_id: Uuid,
    pub inviter_id: Uuid,
    pub invitee_id: Uuid,
    pub message: Option<String>,
    pub status: GroupInvitationStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Group invitation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupInvitationStatus {
    Pending,
    Accepted,
    Declined,
    Cancelled,
    Expired,
}

/// Group activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupActivity {
    pub activity_id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub activity_type: GroupActivityType,
    pub content: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub visibility: VisibilityLevel,
    pub metadata: HashMap<String, String>,
}

/// Types of group activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupActivityType {
    MemberJoined,
    MemberLeft,
    MemberPromoted,
    MemberDemoted,
    PostCreated,
    EventCreated,
    MeetingStarted,
    NoticePosted,
    SettingsChanged,
    NameChanged,
}

/// Group statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStatistics {
    pub total_members: u32,
    pub active_members_30d: u32,
    pub posts_count: u64,
    pub events_count: u64,
    pub meetings_count: u64,
    pub notices_count: u64,
    pub growth_rate: f32,
    pub engagement_score: f32,
}

/// Request to create a new group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub group_type: GroupType,
    pub visibility: GroupVisibility,
    pub membership_policy: GroupMembershipPolicy,
    pub max_members: Option<u32>,
    pub tags: Vec<String>,
    pub group_charter: Option<String>,
    pub settings: GroupSettings,
}

/// Request to update group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<GroupVisibility>,
    pub membership_policy: Option<GroupMembershipPolicy>,
    pub max_members: Option<u32>,
    pub tags: Option<Vec<String>>,
    pub group_charter: Option<String>,
    pub settings: Option<GroupSettings>,
}

/// Group search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSearchCriteria {
    pub query: Option<String>,
    pub group_types: Vec<GroupType>,
    pub visibility_filter: Option<GroupVisibility>,
    pub membership_policy_filter: Option<GroupMembershipPolicy>,
    pub tags: Vec<String>,
    pub member_count_min: Option<u32>,
    pub member_count_max: Option<u32>,
    pub sort_by: GroupSortOption,
    pub sort_order: SortOrder,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Group sort options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupSortOption {
    Name,
    MemberCount,
    Created,
    LastActivity,
    Popularity,
    Relevance,
}

/// Group member list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberListResponse {
    pub members: Vec<GroupMemberInfo>,
    pub total_count: u32,
    pub online_count: u32,
    pub role_distribution: HashMap<GroupRole, u32>,
}

/// Group member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberInfo {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_image: Option<String>,
    pub membership: GroupMembership,
    pub online_status: OnlineStatus,
    pub last_seen: Option<DateTime<Utc>>,
}

impl GroupSystem {
    /// Create new group system
    pub fn new(database: Arc<DatabaseManager>, config: SocialConfig) -> Self {
        Self {
            database,
            config,
            active_groups: Arc::new(RwLock::new(HashMap::new())),
            group_memberships: Arc::new(RwLock::new(HashMap::new())),
            group_invitations: Arc::new(RwLock::new(HashMap::new())),
            group_activities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize group system
    pub async fn initialize(&self) -> SocialResult<()> {
        info!("Initializing group system");

        // Create database tables
        self.create_tables().await?;

        // Load active groups and memberships
        self.load_active_groups().await?;

        info!("Group system initialized successfully");
        Ok(())
    }

    /// Create a new group
    pub async fn create_group(&self, owner_id: Uuid, request: CreateGroupRequest) -> SocialResult<Group> {
        info!("Creating new group '{}' by user {}", request.name, owner_id);

        // Validate group creation request
        self.validate_group_creation(&request, owner_id).await?;

        // Create group
        let group = Group {
            group_id: Uuid::new_v4(),
            name: request.name,
            description: request.description,
            group_type: request.group_type,
            visibility: request.visibility,
            membership_policy: request.membership_policy,
            owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            member_count: 1, // Owner is first member
            max_members: request.max_members,
            tags: request.tags,
            group_image: None,
            group_charter: request.group_charter,
            settings: request.settings,
            statistics: GroupStatistics::default(),
        };

        // Save group to database
        self.save_group(&group).await?;

        // Create owner membership
        let owner_membership = GroupMembership {
            membership_id: Uuid::new_v4(),
            group_id: group.group_id,
            user_id: owner_id,
            role: GroupRole::Owner,
            membership_status: GroupMembershipStatus::Active,
            joined_at: Utc::now(),
            last_active: Utc::now(),
            contribution_points: 0,
            titles: vec!["Founder".to_string()],
            permissions: GroupPermissions::owner_permissions(),
        };

        self.save_group_membership(&owner_membership).await?;

        // Update caches
        {
            let mut groups = self.active_groups.write().await;
            groups.insert(group.group_id, group.clone());
        }

        {
            let mut memberships = self.group_memberships.write().await;
            memberships.entry(owner_id).or_insert_with(Vec::new).push(owner_membership);
        }

        // Record activity
        self.record_group_activity(group.group_id, owner_id, GroupActivityType::MemberJoined, None).await?;

        info!("Group created successfully: {}", group.group_id);
        Ok(group)
    }

    /// Update group information
    pub async fn update_group(&self, group_id: Uuid, user_id: Uuid, request: UpdateGroupRequest) -> SocialResult<Group> {
        info!("Updating group {} by user {}", group_id, user_id);

        // Get group and verify permissions
        let mut group = self.get_group(group_id).await?;
        self.verify_group_permission(group_id, user_id, GroupPermissionType::EditGroupInfo).await?;

        // Apply updates
        if let Some(name) = request.name {
            group.name = name;
        }
        if let Some(description) = request.description {
            group.description = Some(description);
        }
        if let Some(visibility) = request.visibility {
            group.visibility = visibility;
        }
        if let Some(membership_policy) = request.membership_policy {
            group.membership_policy = membership_policy;
        }
        if let Some(max_members) = request.max_members {
            group.max_members = Some(max_members);
        }
        if let Some(tags) = request.tags {
            group.tags = tags;
        }
        if let Some(group_charter) = request.group_charter {
            group.group_charter = Some(group_charter);
        }
        if let Some(settings) = request.settings {
            group.settings = settings;
        }

        group.updated_at = Utc::now();

        // Save to database
        self.save_group(&group).await?;

        // Update cache
        {
            let mut groups = self.active_groups.write().await;
            groups.insert(group_id, group.clone());
        }

        // Record activity
        self.record_group_activity(group_id, user_id, GroupActivityType::SettingsChanged, None).await?;

        info!("Group updated successfully");
        Ok(group)
    }

    /// Join a group
    pub async fn join_group(&self, group_id: Uuid, user_id: Uuid) -> SocialResult<GroupMembership> {
        info!("User {} joining group {}", user_id, group_id);

        let group = self.get_group(group_id).await?;

        // Check if user is already a member
        if self.is_group_member(group_id, user_id).await? {
            return Err(SocialError::ValidationError {
                message: "User is already a member of this group".to_string(),
            });
        }

        // Check membership limits
        self.validate_group_join(&group, user_id).await?;

        // Handle different membership policies
        match group.membership_policy {
            GroupMembershipPolicy::Open => {
                self.add_group_member(group_id, user_id, GroupRole::Member).await
            }
            GroupMembershipPolicy::Application => {
                // Would create membership application
                Err(SocialError::ValidationError {
                    message: "Group requires application for membership".to_string(),
                })
            }
            GroupMembershipPolicy::Invitation => {
                Err(SocialError::AccessDenied {
                    reason: "Group is invitation only".to_string(),
                })
            }
            GroupMembershipPolicy::Restricted => {
                Err(SocialError::AccessDenied {
                    reason: "Group membership is restricted".to_string(),
                })
            }
        }
    }

    /// Leave a group
    pub async fn leave_group(&self, group_id: Uuid, user_id: Uuid) -> SocialResult<()> {
        info!("User {} leaving group {}", user_id, group_id);

        // Verify membership
        if !self.is_group_member(group_id, user_id).await? {
            return Err(SocialError::AccessDenied {
                reason: "User is not a member of this group".to_string(),
            });
        }

        // Check if user is owner
        let group = self.get_group(group_id).await?;
        if group.owner_id == user_id {
            return Err(SocialError::ValidationError {
                message: "Group owner cannot leave group. Transfer ownership first.".to_string(),
            });
        }

        // Remove membership
        self.remove_group_member(group_id, user_id).await?;

        // Record activity
        self.record_group_activity(group_id, user_id, GroupActivityType::MemberLeft, None).await?;

        info!("User left group successfully");
        Ok(())
    }

    /// Send group invitation
    pub async fn send_group_invitation(
        &self,
        group_id: Uuid,
        inviter_id: Uuid,
        invitee_id: Uuid,
        message: Option<String>,
    ) -> SocialResult<GroupInvitation> {
        info!("Sending group invitation from {} to {} for group {}", inviter_id, invitee_id, group_id);

        // Verify permissions
        self.verify_group_permission(group_id, inviter_id, GroupPermissionType::InviteMembers).await?;

        // Check if target user is already a member
        if self.is_group_member(group_id, invitee_id).await? {
            return Err(SocialError::ValidationError {
                message: "User is already a member of this group".to_string(),
            });
        }

        // Create invitation
        let invitation = GroupInvitation {
            invitation_id: Uuid::new_v4(),
            group_id,
            inviter_id,
            invitee_id,
            message,
            status: GroupInvitationStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(7),
        };

        // Save invitation
        self.save_group_invitation(&invitation).await?;

        // Update cache
        {
            let mut invitations = self.group_invitations.write().await;
            invitations.entry(invitee_id).or_insert_with(Vec::new).push(invitation.clone());
        }

        info!("Group invitation sent successfully");
        Ok(invitation)
    }

    /// Accept group invitation
    pub async fn accept_group_invitation(&self, invitation_id: Uuid, user_id: Uuid) -> SocialResult<GroupMembership> {
        info!("User {} accepting group invitation {}", user_id, invitation_id);

        // Get and validate invitation
        let invitation = self.get_group_invitation(invitation_id).await?;
        
        if invitation.invitee_id != user_id {
            return Err(SocialError::AccessDenied {
                reason: "Only the invitee can accept this invitation".to_string(),
            });
        }

        if invitation.status != GroupInvitationStatus::Pending {
            return Err(SocialError::InvalidInvitation {
                reason: "Invitation is not pending".to_string(),
            });
        }

        if invitation.expires_at < Utc::now() {
            return Err(SocialError::InvalidInvitation {
                reason: "Invitation has expired".to_string(),
            });
        }

        // Add user to group
        let membership = self.add_group_member(invitation.group_id, user_id, GroupRole::Member).await?;

        // Update invitation status
        let mut updated_invitation = invitation;
        updated_invitation.status = GroupInvitationStatus::Accepted;
        self.save_group_invitation(&updated_invitation).await?;

        // Remove from pending invitations cache
        {
            let mut invitations = self.group_invitations.write().await;
            if let Some(user_invitations) = invitations.get_mut(&user_id) {
                user_invitations.retain(|i| i.invitation_id != invitation_id);
            }
        }

        info!("Group invitation accepted successfully");
        Ok(membership)
    }

    /// Get group information
    pub async fn get_group(&self, group_id: Uuid) -> SocialResult<Group> {
        let groups = self.active_groups.read().await;
        
        if let Some(group) = groups.get(&group_id) {
            Ok(group.clone())
        } else {
            Err(SocialError::GroupNotFound { group_id })
        }
    }

    /// Get group members
    pub async fn get_group_members(&self, group_id: Uuid, requester_id: Uuid) -> SocialResult<GroupMemberListResponse> {
        debug!("Getting members for group {}", group_id);

        // Verify group exists and user has permission to view members
        let _group = self.get_group(group_id).await?;
        
        // For now, allow all members to see member list
        // In production, this would check visibility settings

        let memberships = self.group_memberships.read().await;
        let mut members = Vec::new();
        let mut online_count = 0;
        let mut role_distribution = HashMap::new();

        // Collect all memberships for this group
        let active_memberships: Vec<_> = memberships.values()
            .flat_map(|user_memberships| user_memberships.iter())
            .filter(|m| m.group_id == group_id && m.membership_status == GroupMembershipStatus::Active)
            .cloned()
            .collect();
        drop(memberships);

        for membership in active_memberships {
            let display_name = self.get_user_display_name(membership.user_id).await;
            let member_info = GroupMemberInfo {
                user_id: membership.user_id,
                display_name,
                avatar_image: None,
                membership: membership.clone(),
                online_status: OnlineStatus::Offline,
                last_seen: None,
            };

            if matches!(member_info.online_status, OnlineStatus::Online) {
                online_count += 1;
            }

            *role_distribution.entry(membership.role.clone()).or_insert(0) += 1;
            members.push(member_info);
        }

        let response = GroupMemberListResponse {
            total_count: members.len() as u32,
            online_count,
            role_distribution,
            members,
        };

        debug!("Group member list retrieved: {} members", response.total_count);
        Ok(response)
    }

    /// Search groups
    pub async fn search_groups(&self, criteria: GroupSearchCriteria) -> SocialResult<Vec<Group>> {
        debug!("Searching groups with criteria: {:?}", criteria);

        let groups = self.active_groups.read().await;
        let mut results: Vec<Group> = groups.values().cloned().collect();

        // Apply filters
        if let Some(query) = &criteria.query {
            let query_lower = query.to_lowercase();
            results.retain(|g| {
                g.name.to_lowercase().contains(&query_lower) ||
                g.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
            });
        }

        if !criteria.group_types.is_empty() {
            results.retain(|g| criteria.group_types.contains(&g.group_type));
        }

        if let Some(visibility) = &criteria.visibility_filter {
            results.retain(|g| g.visibility == *visibility);
        }

        if let Some(policy) = &criteria.membership_policy_filter {
            results.retain(|g| g.membership_policy == *policy);
        }

        if let Some(min_members) = criteria.member_count_min {
            results.retain(|g| g.member_count >= min_members);
        }

        if let Some(max_members) = criteria.member_count_max {
            results.retain(|g| g.member_count <= max_members);
        }

        // Apply sorting
        match criteria.sort_by {
            GroupSortOption::Name => {
                results.sort_by(|a, b| match criteria.sort_order {
                    SortOrder::Ascending => a.name.cmp(&b.name),
                    SortOrder::Descending => b.name.cmp(&a.name),
                });
            }
            GroupSortOption::MemberCount => {
                results.sort_by(|a, b| match criteria.sort_order {
                    SortOrder::Ascending => a.member_count.cmp(&b.member_count),
                    SortOrder::Descending => b.member_count.cmp(&a.member_count),
                });
            }
            GroupSortOption::Created => {
                results.sort_by(|a, b| match criteria.sort_order {
                    SortOrder::Ascending => a.created_at.cmp(&b.created_at),
                    SortOrder::Descending => b.created_at.cmp(&a.created_at),
                });
            }
            _ => {} // Other sort options not implemented
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

        debug!("Group search returned {} results", results.len());
        Ok(results)
    }

    /// Check if user is a member of group
    pub async fn is_group_member(&self, group_id: Uuid, user_id: Uuid) -> SocialResult<bool> {
        let memberships = self.group_memberships.read().await;
        
        if let Some(user_memberships) = memberships.get(&user_id) {
            for membership in user_memberships {
                if membership.group_id == group_id && membership.membership_status == GroupMembershipStatus::Active {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    // Private helper methods

    async fn validate_group_creation(&self, request: &CreateGroupRequest, user_id: Uuid) -> SocialResult<()> {
        if request.name.trim().is_empty() {
            return Err(SocialError::ValidationError {
                message: "Group name cannot be empty".to_string(),
            });
        }

        if request.name.len() > 50 {
            return Err(SocialError::ValidationError {
                message: "Group name too long (max 50 characters)".to_string(),
            });
        }

        // Check user's group limit
        let user_group_count = self.get_user_group_count(user_id).await?;
        if user_group_count >= self.config.max_groups_per_user {
            return Err(SocialError::GroupMembershipLimitExceeded);
        }

        Ok(())
    }

    async fn validate_group_join(&self, group: &Group, user_id: Uuid) -> SocialResult<()> {
        // Check group member limit
        if let Some(max_members) = group.max_members {
            if group.member_count >= max_members {
                return Err(SocialError::ValidationError {
                    message: "Group has reached maximum member limit".to_string(),
                });
            }
        }

        // Check user's group limit
        let user_group_count = self.get_user_group_count(user_id).await?;
        if user_group_count >= self.config.max_groups_per_user {
            return Err(SocialError::GroupMembershipLimitExceeded);
        }

        Ok(())
    }

    async fn add_group_member(&self, group_id: Uuid, user_id: Uuid, role: GroupRole) -> SocialResult<GroupMembership> {
        let membership = GroupMembership {
            membership_id: Uuid::new_v4(),
            group_id,
            user_id,
            role,
            membership_status: GroupMembershipStatus::Active,
            joined_at: Utc::now(),
            last_active: Utc::now(),
            contribution_points: 0,
            titles: Vec::new(),
            permissions: GroupPermissions::member_permissions(),
        };

        // Save to database
        self.save_group_membership(&membership).await?;

        // Update caches
        {
            let mut memberships = self.group_memberships.write().await;
            memberships.entry(user_id).or_insert_with(Vec::new).push(membership.clone());
        }

        // Update group member count
        {
            let mut groups = self.active_groups.write().await;
            if let Some(group) = groups.get_mut(&group_id) {
                group.member_count += 1;
            }
        }

        // Record activity
        self.record_group_activity(group_id, user_id, GroupActivityType::MemberJoined, None).await?;

        Ok(membership)
    }

    async fn remove_group_member(&self, group_id: Uuid, user_id: Uuid) -> SocialResult<()> {
        // Remove from database
        self.delete_group_membership(group_id, user_id).await?;

        // Update caches
        {
            let mut memberships = self.group_memberships.write().await;
            if let Some(user_memberships) = memberships.get_mut(&user_id) {
                user_memberships.retain(|m| m.group_id != group_id);
            }
        }

        // Update group member count
        {
            let mut groups = self.active_groups.write().await;
            if let Some(group) = groups.get_mut(&group_id) {
                group.member_count = group.member_count.saturating_sub(1);
            }
        }

        Ok(())
    }

    async fn verify_group_permission(&self, group_id: Uuid, user_id: Uuid, permission: GroupPermissionType) -> SocialResult<()> {
        let memberships = self.group_memberships.read().await;
        
        if let Some(user_memberships) = memberships.get(&user_id) {
            for membership in user_memberships {
                if membership.group_id == group_id && membership.membership_status == GroupMembershipStatus::Active {
                    if self.has_permission(&membership.permissions, permission) {
                        return Ok(());
                    }
                }
            }
        }

        Err(SocialError::AccessDenied {
            reason: "Insufficient permissions for this group action".to_string(),
        })
    }

    fn has_permission(&self, permissions: &GroupPermissions, permission_type: GroupPermissionType) -> bool {
        match permission_type {
            GroupPermissionType::InviteMembers => permissions.can_invite_members,
            GroupPermissionType::RemoveMembers => permissions.can_remove_members,
            GroupPermissionType::EditGroupInfo => permissions.can_edit_group_info,
            GroupPermissionType::ManageLand => permissions.can_manage_land,
            GroupPermissionType::SendNotices => permissions.can_send_notices,
            GroupPermissionType::StartMeetings => permissions.can_start_meetings,
            GroupPermissionType::ModerateChat => permissions.can_moderate_chat,
            GroupPermissionType::AssignRoles => permissions.can_assign_roles,
        }
    }

    async fn get_user_group_count(&self, user_id: Uuid) -> SocialResult<u32> {
        let memberships = self.group_memberships.read().await;
        
        if let Some(user_memberships) = memberships.get(&user_id) {
            let active_count = user_memberships
                .iter()
                .filter(|m| m.membership_status == GroupMembershipStatus::Active)
                .count();
            Ok(active_count as u32)
        } else {
            Ok(0)
        }
    }

    async fn record_group_activity(&self, group_id: Uuid, user_id: Uuid, activity_type: GroupActivityType, content: Option<String>) -> SocialResult<()> {
        let activity = GroupActivity {
            activity_id: Uuid::new_v4(),
            group_id,
            user_id,
            activity_type,
            content,
            timestamp: Utc::now(),
            visibility: VisibilityLevel::Public,
            metadata: HashMap::new(),
        };

        // Save to database
        self.save_group_activity(&activity).await?;

        // Update cache
        {
            let mut activities = self.group_activities.write().await;
            activities.entry(group_id).or_insert_with(Vec::new).push(activity);
        }

        Ok(())
    }

    // Database operations (placeholder implementations)

    async fn create_tables(&self) -> SocialResult<()> {
        Ok(())
    }

    async fn load_active_groups(&self) -> SocialResult<()> {
        Ok(())
    }

    async fn save_group(&self, _group: &Group) -> SocialResult<()> {
        Ok(())
    }

    async fn save_group_membership(&self, _membership: &GroupMembership) -> SocialResult<()> {
        Ok(())
    }

    async fn delete_group_membership(&self, _group_id: Uuid, _user_id: Uuid) -> SocialResult<()> {
        Ok(())
    }

    async fn save_group_invitation(&self, _invitation: &GroupInvitation) -> SocialResult<()> {
        Ok(())
    }

    async fn get_group_invitation(&self, _invitation_id: Uuid) -> SocialResult<GroupInvitation> {
        Err(SocialError::ValidationError {
            message: "Invitation not found".to_string(),
        })
    }

    async fn save_group_activity(&self, _activity: &GroupActivity) -> SocialResult<()> {
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

/// Group permission types
#[derive(Debug, Clone, Copy)]
pub enum GroupPermissionType {
    InviteMembers,
    RemoveMembers,
    EditGroupInfo,
    ManageLand,
    SendNotices,
    StartMeetings,
    ModerateChat,
    AssignRoles,
}

impl GroupPermissions {
    /// Create permissions for group owner
    pub fn owner_permissions() -> Self {
        Self {
            can_invite_members: true,
            can_remove_members: true,
            can_send_notices: true,
            can_start_meetings: true,
            can_moderate_chat: true,
            can_manage_land: true,
            can_edit_group_info: true,
            can_assign_roles: true,
        }
    }

    /// Create permissions for regular group member
    pub fn member_permissions() -> Self {
        Self {
            can_invite_members: false,
            can_remove_members: false,
            can_send_notices: false,
            can_start_meetings: false,
            can_moderate_chat: false,
            can_manage_land: false,
            can_edit_group_info: false,
            can_assign_roles: false,
        }
    }
}

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            allow_member_invite: true,
            allow_member_post: true,
            allow_member_events: true,
            moderated_posts: false,
            require_approval_for_posts: false,
            enable_group_chat: true,
            enable_group_voice: false,
            enable_group_land: false,
            group_fee: None,
            show_in_search: true,
        }
    }
}

impl Default for GroupStatistics {
    fn default() -> Self {
        Self {
            total_members: 0,
            active_members_30d: 0,
            posts_count: 0,
            events_count: 0,
            meetings_count: 0,
            notices_count: 0,
            growth_rate: 0.0,
            engagement_score: 0.0,
        }
    }
}

impl Default for GroupSortOption {
    fn default() -> Self {
        Self::Name
    }
}

impl Default for GroupSearchCriteria {
    fn default() -> Self {
        Self {
            query: None,
            group_types: Vec::new(),
            visibility_filter: None,
            membership_policy_filter: None,
            tags: Vec::new(),
            member_count_min: None,
            member_count_max: None,
            sort_by: GroupSortOption::default(),
            sort_order: SortOrder::default(),
            limit: Some(20),
            offset: None,
        }
    }
}