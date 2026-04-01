//! Service Traits for Grid Mode Support
//!
//! These traits define the interface for all OpenSim services, allowing
//! both local (in-process) and remote (HTTP) implementations.
//!
//! Architecture:
//! - Standalone mode: Uses Local*Service implementations (direct database access)
//! - Grid mode: Uses Remote*Service implementations (HTTP calls to ROBUST)
//!
//! Reference: opensim-master/OpenSim/Services/Interfaces/

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Avatar Service Trait
// Reference: OpenSim/Services/Interfaces/IAvatarService.cs
// ============================================================================

/// Avatar appearance and data service interface
#[async_trait]
pub trait AvatarServiceTrait: Send + Sync {
    /// Get avatar data by principal ID
    async fn get_avatar(&self, principal_id: Uuid) -> Result<AvatarData>;

    /// Set avatar data for principal
    async fn set_avatar(&self, principal_id: Uuid, data: &AvatarData) -> Result<bool>;

    /// Reset avatar to default appearance
    async fn reset_avatar(&self, principal_id: Uuid) -> Result<bool>;

    /// Remove all avatar items (used for cleanup)
    async fn remove_items(&self, principal_id: Uuid, names: &[String]) -> Result<bool>;
}

/// Avatar data matching OpenSim's AvatarData class
#[derive(Debug, Clone, Default)]
pub struct AvatarData {
    pub avatar_type: i32,
    pub data: HashMap<String, String>,
}

// ============================================================================
// Grid Service Trait
// Reference: OpenSim/Services/Interfaces/IGridService.cs
// ============================================================================

/// Grid and region management service interface
#[async_trait]
pub trait GridServiceTrait: Send + Sync {
    /// Register a region with the grid
    async fn register_region(&self, region: &RegionInfo) -> Result<bool>;

    /// Deregister a region from the grid
    async fn deregister_region(&self, region_id: Uuid) -> Result<bool>;

    /// Get region by UUID
    async fn get_region_by_uuid(&self, scope_id: Uuid, region_id: Uuid) -> Result<Option<RegionInfo>>;

    /// Get region by name
    async fn get_region_by_name(&self, scope_id: Uuid, name: &str) -> Result<Option<RegionInfo>>;

    /// Get region by position (grid coordinates)
    async fn get_region_by_position(&self, scope_id: Uuid, x: u32, y: u32) -> Result<Option<RegionInfo>>;

    /// Get neighboring regions within range
    async fn get_neighbours(&self, scope_id: Uuid, region_id: Uuid, range: u32) -> Result<Vec<RegionInfo>>;

    /// Get default regions (fallback for login)
    async fn get_default_regions(&self, scope_id: Uuid) -> Result<Vec<RegionInfo>>;

    /// Get all regions (with optional flags filter)
    async fn get_regions(&self, scope_id: Uuid, flags: u32) -> Result<Vec<RegionInfo>>;
}

/// Region information matching OpenSim's GridRegion class
#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub region_id: Uuid,
    pub region_name: String,
    pub region_loc_x: u32,
    pub region_loc_y: u32,
    pub region_size_x: u32,
    pub region_size_y: u32,
    pub server_ip: String,
    pub server_port: u16,
    pub server_uri: String,
    pub region_flags: u32,
    pub scope_id: Uuid,
    pub owner_id: Uuid,
    pub estate_id: u32,
}

impl Default for RegionInfo {
    fn default() -> Self {
        Self {
            region_id: Uuid::nil(),
            region_name: String::new(),
            region_loc_x: 1000,
            region_loc_y: 1000,
            region_size_x: 256,
            region_size_y: 256,
            server_ip: "127.0.0.1".to_string(),
            server_port: 9000,
            server_uri: String::new(),
            region_flags: 0,
            scope_id: Uuid::nil(),
            owner_id: Uuid::nil(),
            estate_id: 1,
        }
    }
}

// ============================================================================
// User Account Service Trait
// Reference: OpenSim/Services/Interfaces/IUserAccountService.cs
// ============================================================================

/// User account management service interface
#[async_trait]
pub trait UserAccountServiceTrait: Send + Sync {
    /// Get user account by UUID
    async fn get_user_account(&self, scope_id: Uuid, user_id: Uuid) -> Result<Option<UserAccount>>;

    /// Get user account by name
    async fn get_user_account_by_name(&self, scope_id: Uuid, first: &str, last: &str) -> Result<Option<UserAccount>>;

    /// Get user account by email
    async fn get_user_account_by_email(&self, scope_id: Uuid, email: &str) -> Result<Option<UserAccount>>;

    /// Store/update user account
    async fn store_user_account(&self, data: &UserAccount) -> Result<bool>;

    /// Get accounts matching query
    async fn get_user_accounts(&self, scope_id: Uuid, query: &str) -> Result<Vec<UserAccount>>;
}

/// User account data matching OpenSim's UserAccount class
#[derive(Debug, Clone)]
pub struct UserAccount {
    pub principal_id: Uuid,
    pub scope_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub service_urls: HashMap<String, String>,
    pub created: i64,
    pub user_level: i32,
    pub user_flags: i32,
    pub user_title: String,
}

impl Default for UserAccount {
    fn default() -> Self {
        Self {
            principal_id: Uuid::nil(),
            scope_id: Uuid::nil(),
            first_name: String::new(),
            last_name: String::new(),
            email: String::new(),
            service_urls: HashMap::new(),
            created: 0,
            user_level: 0,
            user_flags: 0,
            user_title: String::new(),
        }
    }
}

// ============================================================================
// Authentication Service Trait
// Reference: OpenSim/Services/Interfaces/IAuthenticationService.cs
// ============================================================================

/// Authentication service interface
#[async_trait]
pub trait AuthenticationServiceTrait: Send + Sync {
    /// Authenticate user with password
    async fn authenticate(&self, principal_id: Uuid, password: &str, lifetime: i32) -> Result<Option<String>>;

    /// Verify authentication token
    async fn verify(&self, principal_id: Uuid, token: &str, lifetime: i32) -> Result<bool>;

    /// Release/invalidate authentication token
    async fn release(&self, principal_id: Uuid, token: &str) -> Result<bool>;

    /// Set password for user
    async fn set_password(&self, principal_id: Uuid, password: &str) -> Result<bool>;

    /// Get authentication info
    async fn get_authentication(&self, principal_id: Uuid) -> Result<Option<AuthInfo>>;
}

/// Authentication info
#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub principal_id: Uuid,
    pub password_hash: String,
    pub password_salt: String,
    pub web_login_key: String,
    pub account_type: String,
}

// ============================================================================
// Asset Service Trait
// Reference: OpenSim/Services/Interfaces/IAssetService.cs
// ============================================================================

/// Asset storage and retrieval service interface
#[async_trait]
pub trait AssetServiceTrait: Send + Sync {
    /// Get asset by ID
    async fn get(&self, id: &str) -> Result<Option<AssetBase>>;

    /// Get asset metadata only (no data)
    async fn get_metadata(&self, id: &str) -> Result<Option<AssetMetadata>>;

    /// Get asset data only (no metadata)
    async fn get_data(&self, id: &str) -> Result<Option<Vec<u8>>>;

    /// Store asset
    async fn store(&self, asset: &AssetBase) -> Result<String>;

    /// Delete asset
    async fn delete(&self, id: &str) -> Result<bool>;

    /// Check if asset exists
    async fn asset_exists(&self, id: &str) -> Result<bool>;
}

/// Asset data matching OpenSim's AssetBase class
#[derive(Debug, Clone)]
pub struct AssetBase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_type: i8,
    pub local: bool,
    pub temporary: bool,
    pub data: Vec<u8>,
    pub creator_id: String,
    pub flags: i32,
}

/// Asset metadata (without data)
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_type: i8,
    pub local: bool,
    pub temporary: bool,
    pub creator_id: String,
    pub flags: i32,
    pub created_date: i64,
}

// ============================================================================
// Inventory Service Trait
// Reference: OpenSim/Services/Interfaces/IInventoryService.cs
// ============================================================================

/// Inventory management service interface
#[async_trait]
pub trait InventoryServiceTrait: Send + Sync {
    /// Get inventory folder by ID
    async fn get_folder(&self, folder_id: Uuid) -> Result<Option<InventoryFolder>>;

    /// Get root folder for user
    async fn get_root_folder(&self, principal_id: Uuid) -> Result<Option<InventoryFolder>>;

    /// Get folder content (items and subfolders)
    async fn get_folder_content(&self, principal_id: Uuid, folder_id: Uuid) -> Result<InventoryCollection>;

    /// Create folder
    async fn create_folder(&self, folder: &InventoryFolder) -> Result<bool>;

    /// Update folder
    async fn update_folder(&self, folder: &InventoryFolder) -> Result<bool>;

    /// Delete folders
    async fn delete_folders(&self, principal_id: Uuid, folder_ids: &[Uuid]) -> Result<bool>;

    /// Get inventory item by ID
    async fn get_item(&self, item_id: Uuid) -> Result<Option<InventoryItem>>;

    /// Add item to inventory
    async fn add_item(&self, item: &InventoryItem) -> Result<bool>;

    /// Update item
    async fn update_item(&self, item: &InventoryItem) -> Result<bool>;

    /// Delete items
    async fn delete_items(&self, principal_id: Uuid, item_ids: &[Uuid]) -> Result<bool>;

    /// Get user inventory skeleton (folder structure)
    async fn get_inventory_skeleton(&self, principal_id: Uuid) -> Result<Vec<InventoryFolder>>;

    /// Move items to new folders — items is Vec of (item_id, new_folder_id)
    async fn move_items(&self, principal_id: Uuid, items: &[(Uuid, Uuid)]) -> Result<bool>;

    /// Move a folder to a new parent
    async fn move_folder(&self, principal_id: Uuid, folder_id: Uuid, new_parent_id: Uuid) -> Result<bool>;

    /// Purge folder contents (items + subfolders)
    async fn purge_folder(&self, principal_id: Uuid, folder_id: Uuid) -> Result<bool>;

    /// Get active gestures for user (assettype=21, flags=1)
    async fn get_active_gestures(&self, principal_id: Uuid) -> Result<Vec<InventoryItem>>;

    /// Get content for multiple folders at once
    async fn get_multiple_folders_content(&self, principal_id: Uuid, folder_ids: &[Uuid]) -> Result<Vec<InventoryCollection>>;

    /// Get combined current permissions for all items owned by principal with given asset ID
    async fn get_asset_permissions(&self, principal_id: Uuid, asset_id: Uuid) -> Result<i32>;
}

/// Inventory folder
#[derive(Debug, Clone)]
pub struct InventoryFolder {
    pub folder_id: Uuid,
    pub parent_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub folder_type: i32,
    pub version: i32,
}

/// Inventory item
#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub item_id: Uuid,
    pub asset_id: Uuid,
    pub folder_id: Uuid,
    pub owner_id: Uuid,
    pub creator_id: Uuid,
    pub creator_data: String,
    pub name: String,
    pub description: String,
    pub asset_type: i32,
    pub inv_type: i32,
    pub flags: u32,
    pub creation_date: i64,
    pub base_permissions: u32,
    pub current_permissions: u32,
    pub everyone_permissions: u32,
    pub next_permissions: u32,
    pub group_permissions: u32,
    pub group_id: Uuid,
    pub group_owned: bool,
    pub sale_price: i32,
    pub sale_type: i32,
}

/// Inventory collection (folder content)
#[derive(Debug, Clone, Default)]
pub struct InventoryCollection {
    pub folders: Vec<InventoryFolder>,
    pub items: Vec<InventoryItem>,
}

// ============================================================================
// Presence Service Trait
// Reference: OpenSim/Services/Interfaces/IPresenceService.cs
// ============================================================================

/// Online presence tracking service interface
#[async_trait]
pub trait PresenceServiceTrait: Send + Sync {
    /// Login agent to region
    async fn login_agent(&self, user_id: Uuid, session_id: Uuid, secure_session_id: Uuid, region_id: Uuid) -> Result<bool>;

    /// Logout agent
    async fn logout_agent(&self, session_id: Uuid) -> Result<bool>;

    /// Report agent in region (heartbeat)
    async fn report_agent(&self, session_id: Uuid, region_id: Uuid) -> Result<bool>;

    /// Get agent session info
    async fn get_agent(&self, session_id: Uuid) -> Result<Option<PresenceInfo>>;

    /// Get all sessions for user
    async fn get_agents(&self, user_ids: &[Uuid]) -> Result<Vec<PresenceInfo>>;
}

/// Presence/session information
#[derive(Debug, Clone)]
pub struct PresenceInfo {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub secure_session_id: Uuid,
    pub region_id: Uuid,
    pub online: bool,
    pub login_time: i64,
    pub logout_time: i64,
}

// ============================================================================
// GridUser Service Trait
// Reference: OpenSim/Services/Interfaces/IGridUserService.cs
// ============================================================================

#[derive(Debug, Clone)]
pub struct GridUserInfo {
    pub user_id: String,
    pub home_region_id: Uuid,
    pub home_position: String,
    pub home_look_at: String,
    pub last_region_id: Uuid,
    pub last_position: String,
    pub last_look_at: String,
    pub online: bool,
    pub login: String,
    pub logout: String,
}

impl Default for GridUserInfo {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            home_region_id: Uuid::nil(),
            home_position: "<0,0,0>".to_string(),
            home_look_at: "<0,0,0>".to_string(),
            last_region_id: Uuid::nil(),
            last_position: "<0,0,0>".to_string(),
            last_look_at: "<0,0,0>".to_string(),
            online: false,
            login: "0".to_string(),
            logout: "0".to_string(),
        }
    }
}

impl GridUserInfo {
    pub fn to_key_value_pairs(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("UserID".to_string(), self.user_id.clone());
        m.insert("HomeRegionID".to_string(), self.home_region_id.to_string());
        m.insert("HomePosition".to_string(), self.home_position.clone());
        m.insert("HomeLookAt".to_string(), self.home_look_at.clone());
        m.insert("LastRegionID".to_string(), self.last_region_id.to_string());
        m.insert("LastPosition".to_string(), self.last_position.clone());
        m.insert("LastLookAt".to_string(), self.last_look_at.clone());
        m.insert("Online".to_string(), if self.online { "True" } else { "False" }.to_string());
        m.insert("Login".to_string(), self.login.clone());
        m.insert("Logout".to_string(), self.logout.clone());
        m
    }
}

#[async_trait]
pub trait GridUserServiceTrait: Send + Sync {
    async fn logged_in(&self, user_id: &str) -> Result<Option<GridUserInfo>>;
    async fn logged_out(&self, user_id: &str, region_id: Uuid, position: &str, look_at: &str) -> Result<bool>;
    async fn set_home(&self, user_id: &str, home_id: Uuid, position: &str, look_at: &str) -> Result<bool>;
    async fn set_last_position(&self, user_id: &str, region_id: Uuid, position: &str, look_at: &str) -> Result<bool>;
    async fn get_grid_user_info(&self, user_id: &str) -> Result<Option<GridUserInfo>>;
    async fn get_grid_user_infos(&self, user_ids: &[String]) -> Result<Vec<GridUserInfo>>;
}

// ============================================================================
// Agent Preferences Service Trait
// Reference: OpenSim/Services/Interfaces/IAgentPreferencesService.cs
// ============================================================================

#[derive(Debug, Clone)]
pub struct AgentPrefs {
    pub principal_id: Uuid,
    pub access_prefs: String,
    pub hover_height: f64,
    pub language: String,
    pub language_is_public: bool,
    pub perm_everyone: i32,
    pub perm_group: i32,
    pub perm_next_owner: i32,
}

impl Default for AgentPrefs {
    fn default() -> Self {
        Self {
            principal_id: Uuid::nil(),
            access_prefs: "M".to_string(),
            hover_height: 0.0,
            language: "en-us".to_string(),
            language_is_public: true,
            perm_everyone: 0,
            perm_group: 0,
            perm_next_owner: 532480,
        }
    }
}

impl AgentPrefs {
    pub fn to_key_value_pairs(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("PrincipalID".to_string(), self.principal_id.to_string());
        m.insert("AccessPrefs".to_string(), self.access_prefs.clone());
        m.insert("HoverHeight".to_string(), self.hover_height.to_string());
        m.insert("Language".to_string(), self.language.clone());
        m.insert("LanguageIsPublic".to_string(), if self.language_is_public { "1" } else { "0" }.to_string());
        m.insert("PermEveryone".to_string(), self.perm_everyone.to_string());
        m.insert("PermGroup".to_string(), self.perm_group.to_string());
        m.insert("PermNextOwner".to_string(), self.perm_next_owner.to_string());
        m
    }
}

#[async_trait]
pub trait AgentPrefsServiceTrait: Send + Sync {
    async fn get_agent_preferences(&self, principal_id: Uuid) -> Result<Option<AgentPrefs>>;
    async fn store_agent_preferences(&self, prefs: &AgentPrefs) -> Result<bool>;
    async fn get_agent_lang(&self, principal_id: Uuid) -> Result<String>;
}

// ============================================================================
// HGFriends Service Trait
// Reference: OpenSim/Services/Interfaces/IHypergridServices.cs
// ============================================================================

#[async_trait]
pub trait HGFriendsServiceTrait: Send + Sync {
    async fn get_friend_perms(&self, principal_id: Uuid, friend_id: &str) -> Result<i32>;
    async fn new_friendship(&self, principal_id: Uuid, friend_id: &str, secret: &str, verified: bool) -> Result<bool>;
    async fn delete_friendship(&self, principal_id: Uuid, friend_id: &str, secret: &str) -> Result<bool>;
    async fn validate_friendship_offered(&self, principal_id: Uuid, friend_id: &str) -> Result<bool>;
    async fn status_notification(&self, friends: &[String], user_id: Uuid, online: bool) -> Result<Vec<String>>;
}

// ============================================================================
// MuteList Service Trait
// Reference: OpenSim/Services/Interfaces/IMuteListService.cs
// ============================================================================

#[derive(Debug, Clone)]
pub struct MuteData {
    pub agent_id: String,
    pub mute_id: String,
    pub mute_name: String,
    pub mute_type: i32,
    pub mute_flags: i32,
    pub stamp: i32,
}

#[async_trait]
pub trait MuteListServiceTrait: Send + Sync {
    async fn get_mutes(&self, agent_id: &str) -> Result<Vec<MuteData>>;
    async fn update_mute(&self, mute: &MuteData) -> Result<bool>;
    async fn remove_mute(&self, agent_id: &str, mute_id: &str, mute_name: &str) -> Result<bool>;
}

// ============================================================================
// Estate Service Trait
// Reference: OpenSim/Services/Interfaces/IEstateDataService.cs
// ============================================================================

#[derive(Debug, Clone)]
pub struct EstateBan {
    pub banned_user_id: String,
    pub banned_ip: String,
    pub banned_ip_mask: String,
    pub ban_time: i32,
}

#[derive(Debug, Clone)]
pub struct EstateSettings {
    pub estate_id: i32,
    pub estate_name: String,
    pub estate_owner: String,
    pub parent_estate_id: i32,
    pub abuse_email_to_estate_owner: bool,
    pub deny_anonymous: bool,
    pub reset_home_on_teleport: bool,
    pub fixed_sun: bool,
    pub deny_transacted: bool,
    pub block_dwell: bool,
    pub deny_identified: bool,
    pub allow_voice: bool,
    pub use_global_time: bool,
    pub price_per_meter: i32,
    pub tax_free: bool,
    pub allow_direct_teleport: bool,
    pub redirect_grid_x: i32,
    pub redirect_grid_y: i32,
    pub sun_position: f64,
    pub estate_skip_scripts: bool,
    pub billable_factor: f64,
    pub public_access: bool,
    pub abuse_email: String,
    pub deny_minors: bool,
    pub allow_landmark: bool,
    pub allow_parcel_changes: bool,
    pub allow_set_home: bool,
    pub allow_environment_override: bool,
    pub estate_managers: Vec<String>,
    pub estate_users: Vec<String>,
    pub estate_groups: Vec<String>,
    pub estate_bans: Vec<EstateBan>,
}

impl Default for EstateSettings {
    fn default() -> Self {
        Self {
            estate_id: 0,
            estate_name: "My Estate".to_string(),
            estate_owner: "00000000-0000-0000-0000-000000000000".to_string(),
            parent_estate_id: 0,
            abuse_email_to_estate_owner: true,
            deny_anonymous: false,
            reset_home_on_teleport: false,
            fixed_sun: false,
            deny_transacted: false,
            block_dwell: false,
            deny_identified: false,
            allow_voice: true,
            use_global_time: true,
            price_per_meter: 1,
            tax_free: false,
            allow_direct_teleport: true,
            redirect_grid_x: 0,
            redirect_grid_y: 0,
            sun_position: 0.0,
            estate_skip_scripts: false,
            billable_factor: 1.0,
            public_access: true,
            abuse_email: String::new(),
            deny_minors: false,
            allow_landmark: true,
            allow_parcel_changes: true,
            allow_set_home: true,
            allow_environment_override: false,
            estate_managers: Vec::new(),
            estate_users: Vec::new(),
            estate_groups: Vec::new(),
            estate_bans: Vec::new(),
        }
    }
}

impl EstateSettings {
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("EstateID".to_string(), self.estate_id.to_string());
        m.insert("EstateName".to_string(), self.estate_name.clone());
        m.insert("EstateOwner".to_string(), self.estate_owner.clone());
        m.insert("ParentEstateID".to_string(), self.parent_estate_id.to_string());
        m.insert("AbuseEmailToEstateOwner".to_string(), self.abuse_email_to_estate_owner.to_string());
        m.insert("DenyAnonymous".to_string(), self.deny_anonymous.to_string());
        m.insert("ResetHomeOnTeleport".to_string(), self.reset_home_on_teleport.to_string());
        m.insert("FixedSun".to_string(), self.fixed_sun.to_string());
        m.insert("DenyTransacted".to_string(), self.deny_transacted.to_string());
        m.insert("BlockDwell".to_string(), self.block_dwell.to_string());
        m.insert("DenyIdentified".to_string(), self.deny_identified.to_string());
        m.insert("AllowVoice".to_string(), self.allow_voice.to_string());
        m.insert("UseGlobalTime".to_string(), self.use_global_time.to_string());
        m.insert("PricePerMeter".to_string(), self.price_per_meter.to_string());
        m.insert("TaxFree".to_string(), self.tax_free.to_string());
        m.insert("AllowDirectTeleport".to_string(), self.allow_direct_teleport.to_string());
        m.insert("RedirectGridX".to_string(), self.redirect_grid_x.to_string());
        m.insert("RedirectGridY".to_string(), self.redirect_grid_y.to_string());
        m.insert("SunPosition".to_string(), self.sun_position.to_string());
        m.insert("EstateSkipScripts".to_string(), self.estate_skip_scripts.to_string());
        m.insert("BillableFactor".to_string(), self.billable_factor.to_string());
        m.insert("PublicAccess".to_string(), self.public_access.to_string());
        m.insert("AbuseEmail".to_string(), self.abuse_email.clone());
        m.insert("DenyMinors".to_string(), self.deny_minors.to_string());
        m.insert("AllowLandmark".to_string(), self.allow_landmark.to_string());
        m.insert("AllowParcelChanges".to_string(), self.allow_parcel_changes.to_string());
        m.insert("AllowSetHome".to_string(), self.allow_set_home.to_string());
        m.insert("AllowEnvironmentOverride".to_string(), self.allow_environment_override.to_string());
        m.insert("EstateManagers".to_string(), self.estate_managers.join(","));
        m.insert("EstateAccess".to_string(), self.estate_users.join(","));
        m.insert("EstateGroups".to_string(), self.estate_groups.join(","));
        m
    }
}

#[async_trait]
pub trait EstateServiceTrait: Send + Sync {
    async fn load_estate_by_region(&self, region_id: &str, create: bool) -> Result<Option<EstateSettings>>;
    async fn load_estate_by_id(&self, estate_id: i32) -> Result<Option<EstateSettings>>;
    async fn store_estate_settings(&self, settings: &EstateSettings) -> Result<bool>;
    async fn link_region(&self, region_id: &str, estate_id: i32) -> Result<bool>;
    async fn get_regions(&self, estate_id: i32) -> Result<Vec<String>>;
    async fn get_estates_by_name(&self, name: &str) -> Result<Vec<i32>>;
    async fn get_estates_by_owner(&self, owner_id: &str) -> Result<Vec<i32>>;
    async fn get_estates_all(&self) -> Result<Vec<i32>>;
}

// ============================================================================
// Map Image Service Trait
// Reference: OpenSim/Services/Interfaces/IMapImageService.cs
// ============================================================================

#[async_trait]
pub trait MapImageServiceTrait: Send + Sync {
    async fn add_map_tile(&self, x: i32, y: i32, data: &[u8], scope_id: &str) -> Result<bool>;
    async fn remove_map_tile(&self, x: i32, y: i32, scope_id: &str) -> Result<bool>;
    async fn get_map_tile(&self, filename: &str, scope_id: &str) -> Result<Option<Vec<u8>>>;
}

// ============================================================================
// Service Mode Configuration
// ============================================================================

/// Service mode - determines whether to use local or remote implementations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceMode {
    /// Standalone mode - all services run locally with direct database access
    Standalone,
    /// Grid mode - services connect to remote ROBUST server
    Grid,
    /// Hybrid mode - some services local, some remote
    Hybrid,
}

impl Default for ServiceMode {
    fn default() -> Self {
        ServiceMode::Standalone
    }
}

/// Service configuration for grid/standalone mode
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub mode: ServiceMode,
    pub grid_server_uri: Option<String>,
    pub asset_server_uri: Option<String>,
    pub inventory_server_uri: Option<String>,
    pub user_account_server_uri: Option<String>,
    pub presence_server_uri: Option<String>,
    pub avatar_server_uri: Option<String>,
    pub authentication_server_uri: Option<String>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            mode: ServiceMode::Standalone,
            grid_server_uri: None,
            asset_server_uri: None,
            inventory_server_uri: None,
            user_account_server_uri: None,
            presence_server_uri: None,
            avatar_server_uri: None,
            authentication_server_uri: None,
        }
    }
}

// ============================================================================
// Hypergrid Service Types
// Reference: OpenSim/Services/Interfaces/IHypergridServices.cs
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct HGRegionInfo {
    pub region_id: Uuid,
    pub region_handle: u64,
    pub external_name: String,
    pub image_url: String,
    pub size_x: u32,
    pub size_y: u32,
    pub http_port: u16,
    pub server_uri: String,
    pub region_name: String,
    pub region_loc_x: u32,
    pub region_loc_y: u32,
    pub hostname: String,
    pub internal_port: u16,
}

#[derive(Debug, Clone)]
pub struct TravelingAgentData {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub grid_external_name: String,
    pub service_token: String,
    pub client_ip: String,
    pub my_ip_address: String,
}

#[derive(Debug, Clone)]
pub struct AgentCircuitData {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub secure_session_id: Uuid,
    pub circuit_code: u32,
    pub first_name: String,
    pub last_name: String,
    pub service_urls: HashMap<String, String>,
    pub service_session_id: String,
    pub start_pos: [f32; 3],
    pub appearance_serial: i32,
    pub client_ip: String,
    pub mac: String,
    pub id0: String,
    pub teleport_flags: u32,
    pub caps_path: String,
}

impl Default for AgentCircuitData {
    fn default() -> Self {
        Self {
            agent_id: Uuid::nil(),
            session_id: Uuid::nil(),
            secure_session_id: Uuid::nil(),
            circuit_code: 0,
            first_name: String::new(),
            last_name: String::new(),
            service_urls: HashMap::new(),
            service_session_id: String::new(),
            start_pos: [128.0, 128.0, 21.0],
            appearance_serial: 0,
            client_ip: String::new(),
            mac: String::new(),
            id0: String::new(),
            teleport_flags: 0,
            caps_path: String::new(),
        }
    }
}

// ============================================================================
// Gatekeeper Service Trait
// Reference: OpenSim/Services/Interfaces/IGatekeeperService.cs
// ============================================================================

#[async_trait]
pub trait GatekeeperServiceTrait: Send + Sync {
    async fn link_region(&self, region_name: &str) -> Result<Option<HGRegionInfo>>;

    async fn get_hyperlinkregion(
        &self,
        region_id: Uuid,
        agent_id: Uuid,
        agent_home_uri: &str,
    ) -> Result<Option<HGRegionInfo>>;

    async fn login_agent(
        &self,
        source: &HGRegionInfo,
        agent_data: &AgentCircuitData,
        destination: &HGRegionInfo,
    ) -> Result<(bool, String)>;
}

// ============================================================================
// User Agent Service Trait
// Reference: OpenSim/Services/Interfaces/IUserAgentService.cs
// ============================================================================

#[async_trait]
pub trait UserAgentServiceTrait: Send + Sync {
    async fn verify_agent(&self, session_id: Uuid, token: &str) -> Result<bool>;

    async fn verify_client(&self, session_id: Uuid, reported_ip: &str) -> Result<bool>;

    async fn get_home_region(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(HGRegionInfo, [f32; 3], [f32; 3])>>;

    async fn get_server_urls(&self, user_id: Uuid) -> Result<HashMap<String, String>>;

    async fn logout_agent(&self, user_id: Uuid, session_id: Uuid) -> Result<()>;

    async fn get_uui(&self, user_id: Uuid, target_user_id: Uuid) -> Result<String>;

    async fn get_uuid(&self, first: &str, last: &str) -> Result<Option<Uuid>>;

    async fn status_notification(
        &self,
        friends: &[String],
        user_id: Uuid,
        online: bool,
    ) -> Result<Vec<Uuid>>;

    async fn is_agent_coming_home(
        &self,
        session_id: Uuid,
        external_name: &str,
    ) -> Result<bool>;

    async fn login_agent_to_grid(
        &self,
        agent: &AgentCircuitData,
        gatekeeper: &HGRegionInfo,
        destination: &HGRegionInfo,
        from_login: bool,
    ) -> Result<(bool, String)>;

    async fn get_user_info(&self, user_id: Uuid) -> Result<HashMap<String, String>>;
}

// ============================================================================
// Authorization Service Trait
// Reference: OpenSim/Services/Interfaces/IAuthorizationService.cs
// ============================================================================

#[async_trait]
pub trait AuthorizationServiceTrait: Send + Sync {
    async fn is_authorized_for_region(
        &self,
        user_id: Uuid,
        first_name: &str,
        last_name: &str,
        region_id: Uuid,
    ) -> Result<(bool, String)>;
}

// ============================================================================
// Friends Service Trait (Local Grid)
// Reference: OpenSim/Services/Interfaces/IFriendsService.cs
// ============================================================================

#[derive(Debug, Clone)]
pub struct FriendInfo {
    pub principal_id: String,
    pub friend: String,
    pub my_flags: i32,
    pub their_flags: i32,
}

#[async_trait]
pub trait FriendsServiceTrait: Send + Sync {
    async fn get_friends(&self, principal_id: &str) -> Result<Vec<FriendInfo>>;
    async fn store_friend(&self, principal_id: &str, friend: &str, flags: i32) -> Result<bool>;
    async fn delete_friend(&self, principal_id: &str, friend: &str) -> Result<bool>;
}

// ============================================================================
// Land Service Trait
// Reference: OpenSim/Services/Interfaces/ILandService.cs
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct LandData {
    pub local_id: i32,
    pub global_id: Uuid,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub is_group_owned: bool,
    pub area: i32,
    pub landing_x: f32,
    pub landing_y: f32,
    pub landing_z: f32,
    pub region_id: Uuid,
    pub flags: u32,
    pub sale_price: i32,
    pub snapshot_id: Uuid,
    pub dwell: f32,
}

#[async_trait]
pub trait LandServiceTrait: Send + Sync {
    async fn get_land_data(
        &self,
        scope_id: Uuid,
        region_handle: u64,
        x: u32,
        y: u32,
    ) -> Result<Option<LandData>>;
}

// ============================================================================
// Offline IM Service Trait
// Reference: OpenSim/Addons/OfflineIM/
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct OfflineIM {
    pub id: i32,
    pub principal_id: String,
    pub from_id: String,
    pub message: String,
    pub timestamp: i32,
}

#[async_trait]
pub trait OfflineIMServiceTrait: Send + Sync {
    async fn get_messages(&self, principal_id: &str) -> Result<Vec<OfflineIM>>;
    async fn store_message(&self, im: &OfflineIM) -> Result<bool>;
    async fn delete_messages(&self, principal_id: &str) -> Result<bool>;
}

// ============================================================================
// Profiles Service Trait (JsonRpc)
// Reference: OpenSim/Services/Interfaces/IUserProfilesService.cs
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct UserProfileProperties {
    pub user_id: Uuid,
    pub partner_id: Uuid,
    pub profile_url: String,
    pub image_id: Uuid,
    pub about_text: String,
    pub first_life_image_id: Uuid,
    pub first_life_text: String,
    pub want_to_text: String,
    pub want_to_mask: i32,
    pub skills_text: String,
    pub skills_mask: i32,
    pub languages: String,
}

#[derive(Debug, Clone, Default)]
pub struct UserProfilePick {
    pub pick_id: Uuid,
    pub creator_id: Uuid,
    pub top_pick: bool,
    pub parcel_id: Uuid,
    pub name: String,
    pub description: String,
    pub snapshot_id: Uuid,
    pub user: String,
    pub original_name: String,
    pub sim_name: String,
    pub global_pos: String,
    pub sort_order: i32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct UserClassifiedAdd {
    pub classified_id: Uuid,
    pub creator_id: Uuid,
    pub creation_date: i32,
    pub expiration_date: i32,
    pub category: i32,
    pub name: String,
    pub description: String,
    pub parcel_id: Uuid,
    pub parent_estate: i32,
    pub snapshot_id: Uuid,
    pub sim_name: String,
    pub global_pos: String,
    pub parcel_name: String,
    pub flags: i32,
    pub listing_price: i32,
}

#[derive(Debug, Clone, Default)]
pub struct UserProfileNotes {
    pub user_id: Uuid,
    pub target_id: Uuid,
    pub notes: String,
}

#[derive(Debug, Clone, Default)]
pub struct UserPreferences {
    pub user_id: Uuid,
    pub im_via_email: bool,
    pub visible: bool,
    pub email: String,
}

#[async_trait]
pub trait ProfilesServiceTrait: Send + Sync {
    async fn get_classifieds(&self, creator_id: Uuid) -> Result<Vec<UserClassifiedAdd>>;
    async fn get_classified(&self, classified_id: Uuid) -> Result<Option<UserClassifiedAdd>>;
    async fn update_classified(&self, classified: &UserClassifiedAdd) -> Result<bool>;
    async fn delete_classified(&self, classified_id: Uuid) -> Result<bool>;

    async fn get_picks(&self, creator_id: Uuid) -> Result<Vec<UserProfilePick>>;
    async fn get_pick(&self, pick_id: Uuid) -> Result<Option<UserProfilePick>>;
    async fn update_pick(&self, pick: &UserProfilePick) -> Result<bool>;
    async fn delete_pick(&self, pick_id: Uuid) -> Result<bool>;

    async fn get_notes(&self, user_id: Uuid, target_id: Uuid) -> Result<Option<UserProfileNotes>>;
    async fn update_notes(&self, notes: &UserProfileNotes) -> Result<bool>;

    async fn get_properties(&self, user_id: Uuid) -> Result<Option<UserProfileProperties>>;
    async fn update_properties(&self, props: &UserProfileProperties) -> Result<bool>;

    async fn get_preferences(&self, user_id: Uuid) -> Result<Option<UserPreferences>>;
    async fn update_preferences(&self, prefs: &UserPreferences) -> Result<bool>;
}
