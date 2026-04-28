//! Login sequence handlers for Second Life viewer protocol
//!
//! This module implements the complete login flow including:
//! - Login request validation
//! - User authentication
//! - Session establishment
//! - Avatar creation and placement
//! - Login response generation

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use rand;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    asset::AssetManager,
    database::user_accounts::{UserAccount, UserAccountDatabase},
    login_state::{LoginError, LoginState, LoginStateMachine, LoginStateManager},
    network::{
        llsd::LLSDValue,
        security::SecurityManager,
        session::{Session, SessionManager},
    },
    region::{
        avatar::{appearance::Appearance, Avatar},
        RegionManager,
    },
    state::StateManager,
};
use sqlx::Row;

/// Login request message from viewer (Second Life viewer protocol compliant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub method: String,        // "login_to_simulator"
    pub first: String,         // First name
    pub last: String,          // Last name
    pub passwd: String,        // Password hash ($1$ MD5 hash or plaintext)
    pub start: String,         // Start location ("home", "last", or region name)
    pub channel: String,       // Viewer channel (e.g., "Second Life Release")
    pub version: String,       // Viewer version (e.g., "6.4.21.550742")
    pub platform: String,      // Platform (Windows, Mac, Linux)
    pub mac: String,           // MAC address (often empty in modern viewers)
    pub id0: String,           // Hardware ID (machine identifier)
    pub agree_to_tos: bool,    // Terms of Service agreement
    pub read_critical: bool,   // Critical message acknowledgment
    pub viewer_digest: String, // Viewer hash for verification
    pub options: Vec<String>,  // Login options (e.g., ["inventory-root", "inventory-skeleton"])

    // Additional SL viewer protocol fields
    pub address_size: Option<u32>,      // Address size (32/64 bit)
    pub extended_errors: Option<bool>,  // Request extended error information
    pub host_id: Option<String>,        // Host identifier
    pub mfa_hash: Option<String>,       // Multi-factor authentication hash
    pub token: Option<String>,          // Authentication token
    pub request_creds: Option<bool>,    // Request credentials in response
    pub skipoptional: Option<bool>,     // Skip optional login data
    pub inventory_host: Option<String>, // Inventory service host
    pub want_to_login: Option<bool>,    // Explicit login intent
}

/// Login response message to viewer (Second Life viewer protocol compliant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub login: String,   // "true" or "false"
    pub message: String, // Login message or error
    pub reason: String,  // Success reason or error details

    // Session information (only if successful)
    pub session_id: Option<Uuid>,
    pub secure_session_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,

    // Region information (only if successful)
    pub sim_ip: Option<String>,
    pub sim_port: Option<u16>,
    pub region_x: Option<u32>,
    pub region_y: Option<u32>,
    pub region_size_x: Option<u32>,
    pub region_size_y: Option<u32>,

    // Avatar information (only if successful)
    pub look_at: Option<[f32; 3]>,
    pub agent_access: Option<String>, // "M", "PG", "A" (Mature, PG, Adult)
    pub agent_access_max: Option<String>, // Maximum access level
    pub start_location: Option<String>,

    // Inventory and assets (only if successful)
    pub inventory_root: Option<Vec<InventoryFolder>>,
    pub inventory_skeleton: Option<Vec<InventoryItem>>,
    pub inventory_lib_root: Option<Vec<InventoryFolder>>,
    pub inventory_lib_owner: Option<Vec<InventoryItem>>,

    // Additional login data
    pub login_flags: Option<Vec<LoginFlag>>,
    pub global_textures: Option<Vec<GlobalTexture>>,
    pub event_categories: Option<Vec<EventCategory>>,
    pub event_notifications: Option<Vec<EventNotification>>,
    pub classified_categories: Option<Vec<ClassifiedCategory>>,
    pub ui_config: Option<Vec<UIConfig>>,
    pub max_agent_groups: Option<u32>,
    pub map_server_url: Option<String>,
    pub search_token: Option<String>,
    pub currency: Option<Currency>,
    pub stipend_since_login: Option<String>,
    pub gendered: Option<String>,
    pub ever_logged_in: Option<String>,
    pub seconds_since_epoch: Option<u64>,

    // Additional SL viewer protocol fields
    pub circuit_code: Option<u32>, // Circuit code for UDP messaging
    pub sim_port_udp: Option<u16>, // UDP port for region connection
    pub inventory_host: Option<String>, // Inventory service host
    pub seed_capability: Option<String>, // Seed capability URL
    pub capabilities: Option<Vec<Capability>>, // Service capabilities
    pub home: Option<HomeLocation>, // Home location information
    pub buddy_list: Option<Vec<BuddyListEntry>>, // Friends list
    pub gestures: Option<Vec<GestureEntry>>, // Active gestures
    pub region_info: Option<RegionLoginInfo>, // Detailed region information
    pub tutorial_setting: Option<Vec<TutorialSetting>>, // Tutorial progress
    pub initial_outfit: Option<Vec<InitialOutfitFolder>>, // Initial outfit folders
    pub login_response_config: Option<LoginResponseConfig>, // Response configuration
}

/// Inventory folder structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryFolder {
    pub folder_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub type_default: i32,
    pub version: i32,
}

/// Inventory item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_id: Uuid,
    pub parent_id: Uuid,
    pub asset_id: Uuid,
    pub permissions: ItemPermissions,
    pub asset_type: i32,
    pub inv_type: i32,
    pub flags: u32,
    pub sale_info: SaleInfo,
    pub name: String,
    pub desc: String,
    pub creation_date: u32,
}

/// Item permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemPermissions {
    pub base_mask: u32,
    pub owner_mask: u32,
    pub group_mask: u32,
    pub everyone_mask: u32,
    pub next_owner_mask: u32,
    pub creator_id: Uuid,
    pub owner_id: Uuid,
    pub last_owner_id: Uuid,
    pub group_id: Uuid,
}

/// Sale information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleInfo {
    pub sale_type: i32,
    pub sale_price: i32,
}

/// Login flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginFlag {
    pub key: String,
    pub value: LLSDValue,
}

/// Global textures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalTexture {
    pub cloud_texture_id: Uuid,
    pub moon_texture_id: Uuid,
    pub sun_texture_id: Uuid,
}

/// Event categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCategory {
    pub category_id: u32,
    pub category_name: String,
}

/// Event notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventNotification {
    pub key: String,
    pub value: String,
}

/// Classified categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedCategory {
    pub category_id: u32,
    pub category_name: String,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub key: String,
    pub value: String,
}

/// Currency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    pub local_id: String,
}

/// Service capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub url: String,
}

/// Home location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeLocation {
    pub region_handle: u64,
    pub position: [f32; 3],
    pub look_at: [f32; 3],
}

/// Buddy list entry (friends list)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuddyListEntry {
    pub buddy_id: Uuid,
    pub buddy_rights_given: u32,
    pub buddy_rights_has: u32,
}

/// Gesture entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureEntry {
    pub item_id: Uuid,
    pub asset_id: Uuid,
}

/// Detailed region information for login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionLoginInfo {
    pub region_handle: u64,
    pub seed_capability: String,
    pub sim_access: String,
    pub agent_movement_complete: bool,
}

/// Tutorial setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialSetting {
    pub tutorial_url: String,
}

/// Initial outfit folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialOutfitFolder {
    pub folder_name: String,
    pub gender: String,
}

/// Login response configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponseConfig {
    pub default_economy_info: bool,
    pub voice_server_type: String,
}

/// Login handler that processes login requests and generates responses
pub struct LoginHandler {
    user_account_database: Arc<UserAccountDatabase>,
    session_manager: Arc<SessionManager>,
    region_manager: Arc<RegionManager>,
    state_manager: Arc<StateManager>,
    asset_manager: Arc<AssetManager>,
    security_manager: Arc<SecurityManager>,
    login_state_manager: Arc<RwLock<LoginStateManager>>,
}

impl LoginHandler {
    /// Creates a new login handler
    pub fn new(
        user_account_database: Arc<UserAccountDatabase>,
        session_manager: Arc<SessionManager>,
        region_manager: Arc<RegionManager>,
        state_manager: Arc<StateManager>,
        asset_manager: Arc<AssetManager>,
        security_manager: Arc<SecurityManager>,
    ) -> Self {
        Self {
            user_account_database,
            session_manager,
            region_manager,
            state_manager,
            asset_manager,
            security_manager,
            login_state_manager: Arc::new(RwLock::new(LoginStateManager::new())),
        }
    }

    /// Processes a login request and returns a login response
    pub async fn handle_login_request(
        &self,
        request: LoginRequest,
        client_ip: std::net::SocketAddr,
    ) -> Result<LoginResponse> {
        info!(
            "Processing login request for user: {} {} from {}",
            request.first, request.last, client_ip
        );

        // Create new login session with state machine
        let session_id = Uuid::new_v4();
        let mut state_machine = {
            let mut state_manager = self.login_state_manager.write().await;
            state_manager.create_session(session_id).clone()
        };

        // Transition to login authentication state
        if let Err(e) = state_machine.transition_to(LoginState::LoginAuthInit) {
            error!("Failed to transition to LoginAuthInit state: {}", e);
            return Ok(self.create_error_response(
                "Login failed",
                "Internal server error during login initialization",
            ));
        }

        // Validate request format
        if let Err(e) = self.validate_login_request(&request).await {
            warn!("Login request validation failed: {}", e);
            state_machine
                .transition_to(LoginState::Failed(LoginError::InvalidCredentials))
                .ok();
            return Ok(self
                .create_error_response("Login failed", &format!("Invalid login request: {}", e)));
        }

        // Transition to XML-RPC login state
        if let Err(e) = state_machine.transition_to(LoginState::XmlrpcLogin) {
            error!("Failed to transition to XmlrpcLogin state: {}", e);
            return Ok(self.create_error_response(
                "Login failed",
                "Internal server error during login processing",
            ));
        }

        // Authenticate user
        let user_account = match self.authenticate_user(&request).await {
            Ok(account) => {
                // Transition to processing response state
                if let Err(e) = state_machine.transition_to(LoginState::LoginProcessResponse) {
                    error!("Failed to transition to LoginProcessResponse state: {}", e);
                    return Ok(self.create_error_response(
                        "Login failed",
                        "Internal server error during authentication processing",
                    ));
                }
                account
            }
            Err(e) => {
                warn!(
                    "Authentication failed for {} {}: {}",
                    request.first, request.last, e
                );
                state_machine
                    .transition_to(LoginState::Failed(LoginError::XmlrpcAuthFailed(
                        e.to_string(),
                    )))
                    .ok();
                return Ok(self.create_error_response(
                    "Login failed",
                    "Authentication failed. Please check your username and password.",
                ));
            }
        };

        // Check for existing session
        if self
            .session_manager
            .has_active_session(&user_account.id.to_string())
            .await
        {
            warn!(
                "User {} {} already has an active session",
                request.first, request.last
            );
            state_machine
                .transition_to(LoginState::Failed(LoginError::NetworkFailed(
                    "Already logged in".to_string(),
                )))
                .ok();
            return Ok(self.create_error_response(
                "Login failed",
                "You are already logged in. Please try again in a few moments.",
            ));
        }

        // Transition to world initialization
        if let Err(e) = state_machine.transition_to(LoginState::WorldInit) {
            error!("Failed to transition to WorldInit state: {}", e);
            return Ok(self.create_error_response(
                "Login failed",
                "Internal server error during world initialization",
            ));
        }

        // Create new session
        let (session_id_actual, secure_session_id) = self
            .create_user_session(&user_account, &request, client_ip)
            .await?;

        // Set agent ID in state machine
        state_machine.set_agent_id(user_account.id);

        // Find appropriate region for login
        let region_info = self.determine_login_region(&request, &user_account).await?;

        // Transition to agent send state
        if let Err(e) = state_machine.transition_to(LoginState::AgentSend) {
            error!("Failed to transition to AgentSend state: {}", e);
            return Ok(self.create_error_response(
                "Login failed",
                "Internal server error during agent initialization",
            ));
        }

        // Create or load avatar
        let avatar = self
            .create_or_load_avatar(&user_account, &region_info)
            .await?;

        // Transition to started state
        if let Err(e) = state_machine.transition_to(LoginState::Started) {
            error!("Failed to transition to Started state: {}", e);
            return Ok(self.create_error_response(
                "Login failed",
                "Internal server error during login completion",
            ));
        }

        // Update state manager with completed session
        {
            let mut state_manager = self.login_state_manager.write().await;
            if let Some(stored_state) = state_manager.get_session_mut(&session_id) {
                *stored_state = state_machine.clone();
            }
        }

        // Generate circuit code once for this login session
        let circuit_code = rand::random::<u32>();
        info!(
            "🎯 Generated coordinated circuit code: {} for user: {} {}",
            circuit_code, user_account.first_name, user_account.last_name
        );

        // Generate successful login response
        let response = self
            .create_success_response(
                &user_account,
                session_id_actual,
                secure_session_id,
                &region_info,
                &avatar,
                &request,
                circuit_code,
            )
            .await?;

        info!(
            "Login successful for user: {} {} (Agent: {})",
            request.first, request.last, user_account.id
        );

        Ok(response)
    }

    /// Validates the login request format and required fields
    async fn validate_login_request(&self, request: &LoginRequest) -> Result<()> {
        if request.method != "login_to_simulator" {
            return Err(anyhow!("Invalid login method: {}", request.method));
        }

        if request.first.trim().is_empty() || request.last.trim().is_empty() {
            return Err(anyhow!("First and last names are required"));
        }

        if request.passwd.is_empty() {
            return Err(anyhow!("Password is required"));
        }

        if !request.agree_to_tos {
            return Err(anyhow!("You must agree to the Terms of Service"));
        }

        // Validate viewer information
        if request.channel.is_empty() || request.version.is_empty() {
            return Err(anyhow!("Viewer information is required"));
        }

        Ok(())
    }

    /// Authenticates user credentials against the database
    async fn authenticate_user(&self, request: &LoginRequest) -> Result<UserAccount> {
        let username = format!("{} {}", request.first, request.last);

        // EADS: Use OpenSim-compatible authentication with PostgreSQL
        match self
            .user_account_database
            .authenticate_user_opensim(&request.first, &request.last, &request.passwd)
            .await
        {
            Ok(Some(user_account)) => {
                // Check if account is active (using user_level >= 0 as active indicator)
                if user_account.user_level < 0 {
                    return Err(anyhow!("Account is disabled"));
                }
                debug!("User authentication successful for: {}", username);
                return Ok(user_account);
            }
            Ok(None) => {
                info!("User not found via database authentication, trying direct SQLite fallback");
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("stub mode") {
                    info!("Database in stub mode, using direct SQLite authentication");
                } else {
                    info!(
                        "Database authentication failed: {}, trying direct SQLite fallback",
                        e
                    );
                }
            }
        }

        // EADS: No SQLite fallback - PostgreSQL authentication required
        Err(anyhow!(
            "Authentication failed: user not found or invalid credentials"
        ))
    }

    // EADS: SQLite authentication removed - PostgreSQL only

    /// Creates a new user session
    async fn create_user_session(
        &self,
        user_account: &UserAccount,
        request: &LoginRequest,
        client_ip: std::net::SocketAddr,
    ) -> Result<(Uuid, Uuid)> {
        let session_id = Uuid::new_v4();
        let secure_session_id = Uuid::new_v4();

        let mut session = Session::new(user_account.id.to_string(), client_ip);
        session.session_id = session_id.to_string();
        session.agent_id = user_account.id;
        session.first_name = user_account.first_name.clone();
        session.last_name = user_account.last_name.clone();
        session.viewer_info = Some(crate::network::client::ViewerInfo {
            name: request.channel.clone(),
            version: request.version.clone(),
            platform: request.platform.clone(),
            channel: request.channel.clone(),
        });

        // Store session
        self.session_manager
            .create_session(session)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;

        debug!(
            "Created session {} for user {}",
            session_id, user_account.id
        );
        Ok((session_id, secure_session_id))
    }

    /// Determines which region the user should log into
    async fn determine_login_region(
        &self,
        request: &LoginRequest,
        user_account: &UserAccount,
    ) -> Result<RegionInfo> {
        let region_info = match request.start.as_str() {
            "home" => {
                // Use user's home region if set, otherwise default
                match self.get_user_home_region(user_account).await {
                    Ok(region) => region,
                    Err(_) => self.get_default_region(),
                }
            }
            "last" => {
                // Use user's last location if available, otherwise home/default
                match self.get_user_last_region(user_account).await {
                    Ok(region) => region,
                    Err(_) => match self.get_user_home_region(user_account).await {
                        Ok(region) => region,
                        Err(_) => self.get_default_region(),
                    },
                }
            }
            region_name => {
                // Try to find the specified region
                match self.find_region_by_name(region_name).await {
                    Ok(region) => region,
                    Err(_) => {
                        // If specified region not found, use auto-assignment
                        match self.auto_assign_region(user_account).await {
                            Ok(region) => region,
                            Err(_) => self.get_default_region(),
                        }
                    }
                }
            }
        };

        debug!(
            "Login region determined: {} for user {}",
            region_info.name, user_account.id
        );

        Ok(region_info)
    }

    /// Creates or loads avatar for the user
    async fn create_or_load_avatar(
        &self,
        user_account: &UserAccount,
        region_info: &RegionInfo,
    ) -> Result<Avatar> {
        // Try to load existing avatar
        if let Ok(existing_avatar) = self
            .region_manager
            .get_avatar(&user_account.id.to_string())
            .await
        {
            debug!("Loaded existing avatar for user {}", user_account.id);
            return Ok(existing_avatar);
        }

        // Create new avatar
        let mut avatar = Avatar::new(
            user_account.id,
            user_account.first_name.clone(),
            user_account.last_name.clone(),
        );

        // Load user appearance from preferences or use default
        avatar.appearance = self
            .get_user_appearance(user_account)
            .await
            .unwrap_or_else(|_| {
                // Default to appropriate gender based on user preferences or fallback
                match user_account.first_name.to_lowercase().as_str() {
                    // Common female names - use female appearance
                    name if [
                        "alice", "anna", "beth", "carol", "diana", "emma", "fiona", "grace",
                        "helen", "iris", "jane", "kate", "lisa", "mary", "nina", "olivia", "paula",
                        "quinn", "rose", "sara", "tina", "una", "vera", "wendy",
                    ]
                    .contains(&name) =>
                    {
                        Appearance::default_female()
                    }
                    // Common male names - use male appearance
                    name if [
                        "adam", "bob", "carl", "dave", "eric", "frank", "greg", "henry", "ivan",
                        "jack", "kyle", "luke", "mike", "nick", "owen", "paul", "quincy", "rick",
                        "steve", "tom", "ulrich", "victor", "will", "xavier",
                    ]
                    .contains(&name) =>
                    {
                        Appearance::default_male()
                    }
                    // Default fallback based on account creation preferences or female default
                    _ => Appearance::default_female(),
                }
            });

        // Set initial position in region based on region info and user's last known position
        let (start_x, start_y, start_z) = self.calculate_spawn_position(user_account, region_info);
        avatar.set_position(start_x, start_y, start_z);
        avatar.set_look_at(1.0, 0.0, 0.0); // Looking east

        // Add avatar to region
        self.region_manager.add_avatar(avatar.clone()).await?;

        info!(
            "Created new avatar for user {} in region {}",
            user_account.id, region_info.name
        );

        Ok(avatar)
    }

    /// Creates a successful login response
    async fn create_success_response(
        &self,
        user_account: &UserAccount,
        session_id: Uuid,
        secure_session_id: Uuid,
        region_info: &RegionInfo,
        avatar: &Avatar,
        request: &LoginRequest,
        circuit_code: u32,
    ) -> Result<LoginResponse> {
        let inventory = self.get_user_inventory(user_account).await?;
        let global_textures = self.get_global_textures().await?;

        // Use the coordinated circuit code passed from the main handler
        info!(
            "🔧 Using coordinated circuit code: {} in create_success_response",
            circuit_code
        );

        // Create capabilities for this session
        let capabilities = self
            .create_session_capabilities(session_id, user_account)
            .await?;

        // Create home location information
        let home_location = self.create_home_location(user_account, region_info).await?;

        Ok(LoginResponse {
            login: "true".to_string(),
            message: "Welcome to OpenSim Next!".to_string(),
            reason: "successful login".to_string(),

            // Session information
            session_id: Some(session_id),
            secure_session_id: Some(secure_session_id),
            agent_id: Some(user_account.id),
            first_name: Some(user_account.first_name.clone()),
            last_name: Some(user_account.last_name.clone()),

            // Region information
            sim_ip: Some(region_info.ip.clone()),
            sim_port: Some(region_info.port),
            region_x: Some(region_info.x),
            region_y: Some(region_info.y),
            region_size_x: Some(region_info.size_x),
            region_size_y: Some(region_info.size_y),

            // Avatar information
            look_at: Some(avatar.look_at),
            agent_access: Some("M".to_string()), // Mature access
            agent_access_max: Some("A".to_string()), // Adult access
            start_location: Some(request.start.clone()),

            // Inventory
            inventory_root: Some(inventory.root_folders),
            inventory_skeleton: Some(inventory.items),
            inventory_lib_root: Some(vec![]), // Library folders (empty for now)
            inventory_lib_owner: Some(vec![]), // Library items (empty for now)

            // Additional data
            login_flags: Some(self.get_login_flags().await),
            global_textures: Some(global_textures),
            event_categories: Some(vec![]),
            event_notifications: Some(vec![]),
            classified_categories: Some(vec![]),
            ui_config: Some(vec![]),
            max_agent_groups: Some(42),
            map_server_url: Some("http://map.opensim.local/".to_string()),
            search_token: Some(Uuid::new_v4().to_string()),
            currency: Some(Currency {
                local_id: "L$".to_string(),
            }),
            stipend_since_login: Some("0".to_string()),
            gendered: Some("Y".to_string()),
            ever_logged_in: Some("Y".to_string()),
            seconds_since_epoch: Some(Utc::now().timestamp() as u64),

            // SL viewer protocol fields
            circuit_code: Some(circuit_code),
            sim_port_udp: Some(region_info.port + 1), // UDP port typically +1 from HTTP
            inventory_host: Some(format!("{}:{}", region_info.ip, region_info.port)),
            seed_capability: Some(format!(
                "http://{}:{}/cap/{}",
                region_info.ip, region_info.port, session_id
            )),
            capabilities: Some(capabilities),
            home: Some(home_location),
            buddy_list: Some(self.get_user_friends(user_account).await?),
            gestures: Some(self.get_user_gestures(user_account).await?),
            region_info: Some(RegionLoginInfo {
                // CRITICAL: region_handle uses METER coordinates, not grid coordinates
                // Grid (1000, 1000) → Meters (256000, 256000)
                region_handle: (((region_info.x as u64) * 256) << 32)
                    | ((region_info.y as u64) * 256),
                seed_capability: format!(
                    "http://{}:{}/cap/{}",
                    region_info.ip, region_info.port, session_id
                ),
                sim_access: "M".to_string(), // Mature
                agent_movement_complete: false,
            }),
            tutorial_setting: Some(vec![]),
            initial_outfit: Some(self.get_initial_outfits().await),
            login_response_config: Some(LoginResponseConfig {
                default_economy_info: true,
                voice_server_type: "vivox".to_string(),
            }),
        })
    }

    /// Creates an error login response
    fn create_error_response(&self, message: &str, reason: &str) -> LoginResponse {
        LoginResponse {
            login: "false".to_string(),
            message: message.to_string(),
            reason: reason.to_string(),

            // All optional fields are None for error responses
            session_id: None,
            secure_session_id: None,
            agent_id: None,
            first_name: None,
            last_name: None,
            sim_ip: None,
            sim_port: None,
            region_x: None,
            region_y: None,
            region_size_x: None,
            region_size_y: None,
            look_at: None,
            agent_access: None,
            agent_access_max: None,
            start_location: None,
            inventory_root: None,
            inventory_skeleton: None,
            inventory_lib_root: None,
            inventory_lib_owner: None,
            login_flags: None,
            global_textures: None,
            event_categories: None,
            event_notifications: None,
            classified_categories: None,
            ui_config: None,
            max_agent_groups: None,
            map_server_url: None,
            search_token: None,
            currency: None,
            stipend_since_login: None,
            gendered: None,
            ever_logged_in: None,
            seconds_since_epoch: None,

            // SL viewer protocol fields (all None for errors)
            circuit_code: None,
            sim_port_udp: None,
            inventory_host: None,
            seed_capability: None,
            capabilities: None,
            home: None,
            buddy_list: None,
            gestures: None,
            region_info: None,
            tutorial_setting: None,
            initial_outfit: None,
            login_response_config: None,
        }
    }

    // Helper methods for region and inventory management

    async fn get_user_home_region(&self, user_account: &UserAccount) -> Result<RegionInfo> {
        // Check if user has a home region set in database
        if let Some(home_region_id) = user_account.home_region_id {
            // Convert to RegionId and lookup region
            let region_id = crate::region::RegionId(home_region_id.as_u128() as u64);
            match self.region_manager.get_region_info(region_id).await {
                Ok(region_info) => {
                    return Ok(RegionInfo {
                        name: region_info.name,
                        ip: "127.0.0.1".to_string(), // Use local IP for now
                        port: 9000,
                        x: region_info.x,
                        y: region_info.y,
                        size_x: region_info.size_x,
                        size_y: region_info.size_y,
                    });
                }
                Err(_) => {
                    warn!(
                        "User's home region {} not found, using default",
                        home_region_id
                    );
                }
            }
        }

        Err(anyhow!("Home region not set or not found"))
    }

    async fn get_user_last_region(&self, user_account: &UserAccount) -> Result<RegionInfo> {
        // For now, use home position from user account if available
        if user_account.home_position_x != 0.0 || user_account.home_position_y != 0.0 {
            // Calculate which region this position would be in
            // OpenSim regions are typically 256x256, positioned on a grid
            let region_x = (user_account.home_position_x / 256.0) as u32 * 256;
            let region_y = (user_account.home_position_y / 256.0) as u32 * 256;

            return Ok(RegionInfo {
                name: format!("Region at ({}, {})", region_x, region_y),
                ip: "127.0.0.1".to_string(),
                port: 9000,
                x: region_x,
                y: region_y,
                size_x: 256,
                size_y: 256,
            });
        }

        Err(anyhow!("Last location not available"))
    }

    async fn find_region_by_name(&self, region_name: &str) -> Result<RegionInfo> {
        // Try to find region by name in the region manager
        match self.region_manager.find_region_by_name(region_name).await {
            Ok(region_info) => Ok(RegionInfo {
                name: region_info.name,
                ip: "127.0.0.1".to_string(),
                port: 9000,
                x: region_info.x,
                y: region_info.y,
                size_x: region_info.size_x,
                size_y: region_info.size_y,
            }),
            Err(_) => Err(anyhow!("Region '{}' not found", region_name)),
        }
    }

    fn get_default_region(&self) -> RegionInfo {
        RegionInfo {
            name: "Default Region".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 9000,
            x: 1000,
            y: 1000,
            size_x: 256,
            size_y: 256,
        }
    }

    /// Automatically assign a suitable region for login based on load balancing
    async fn auto_assign_region(&self, _user_account: &UserAccount) -> Result<RegionInfo> {
        // Get all available regions and find the least loaded one
        match self.region_manager.get_least_loaded_region().await {
            Ok(region_info) => Ok(RegionInfo {
                name: region_info.name,
                ip: "127.0.0.1".to_string(),
                port: 9000,
                x: region_info.x,
                y: region_info.y,
                size_x: region_info.size_x,
                size_y: region_info.size_y,
            }),
            Err(_) => {
                // Fallback to default region
                info!("No regions available for load balancing, using default region");
                Ok(self.get_default_region())
            }
        }
    }

    /// Calculate optimal spawn position for avatar in the region
    fn calculate_spawn_position(
        &self,
        user_account: &UserAccount,
        region_info: &RegionInfo,
    ) -> (f32, f32, f32) {
        // Use user's home position if it's within the current region
        let region_min_x = region_info.x as f32;
        let region_max_x = region_min_x + region_info.size_x as f32;
        let region_min_y = region_info.y as f32;
        let region_max_y = region_min_y + region_info.size_y as f32;

        // Check if user's home position is within this region
        if user_account.home_position_x >= region_min_x
            && user_account.home_position_x < region_max_x
            && user_account.home_position_y >= region_min_y
            && user_account.home_position_y < region_max_y
        {
            // Use the user's home position
            return (
                user_account.home_position_x,
                user_account.home_position_y,
                user_account.home_position_z.max(22.0), // Ensure minimum height
            );
        }

        // Otherwise, calculate a safe spawn position
        let center_x = region_min_x + (region_info.size_x as f32 / 2.0);
        let center_y = region_min_y + (region_info.size_y as f32 / 2.0);
        let spawn_height = 25.0; // Safe height above ground

        // Add small random offset to avoid spawning multiple users in exact same spot
        let offset_x = (rand::random::<f32>() - 0.5) * 10.0; // ±5 meters
        let offset_y = (rand::random::<f32>() - 0.5) * 10.0; // ±5 meters

        (center_x + offset_x, center_y + offset_y, spawn_height)
    }

    async fn get_user_inventory(&self, user_account: &UserAccount) -> Result<UserInventory> {
        // Load user's inventory from database
        debug!("Loading inventory for user: {}", user_account.id);

        // For now, we'll create a basic inventory structure if it doesn't exist
        // In a full implementation, this would load from the inventory database
        let root_folder = InventoryFolder {
            folder_id: user_account.id, // Use user ID as root folder ID
            parent_id: None,
            name: "My Inventory".to_string(),
            type_default: 8, // Root folder type
            version: 1,
        };

        // Create essential subfolders
        let clothing_folder = InventoryFolder {
            folder_id: Uuid::new_v4(),
            parent_id: Some(user_account.id),
            name: "Clothing".to_string(),
            type_default: 5, // Clothing folder type
            version: 1,
        };

        let objects_folder = InventoryFolder {
            folder_id: Uuid::new_v4(),
            parent_id: Some(user_account.id),
            name: "Objects".to_string(),
            type_default: 6, // Objects folder type
            version: 1,
        };

        let textures_folder = InventoryFolder {
            folder_id: Uuid::new_v4(),
            parent_id: Some(user_account.id),
            name: "Textures".to_string(),
            type_default: 0, // Texture folder type
            version: 1,
        };

        let sounds_folder = InventoryFolder {
            folder_id: Uuid::new_v4(),
            parent_id: Some(user_account.id),
            name: "Sounds".to_string(),
            type_default: 1, // Sound folder type
            version: 1,
        };

        let scripts_folder = InventoryFolder {
            folder_id: Uuid::new_v4(),
            parent_id: Some(user_account.id),
            name: "Scripts".to_string(),
            type_default: 10, // LSL Text folder type
            version: 1,
        };

        let landmarks_folder = InventoryFolder {
            folder_id: Uuid::new_v4(),
            parent_id: Some(user_account.id),
            name: "Landmarks".to_string(),
            type_default: 3, // Landmark folder type
            version: 1,
        };

        // Create default items for new users
        let mut items = Vec::new();

        // Add default shape item
        items.push(InventoryItem {
            item_id: Uuid::new_v4(),
            parent_id: clothing_folder.folder_id,
            asset_id: Uuid::parse_str("66c41e39-38f9-f75a-024e-585989bfaba9").unwrap_or_default(),
            permissions: ItemPermissions {
                base_mask: 0x7FFFFFFF,
                owner_mask: 0x7FFFFFFF,
                group_mask: 0,
                everyone_mask: 0,
                next_owner_mask: 0x7FFFFFFF,
                creator_id: Uuid::nil(),
                owner_id: user_account.id,
                last_owner_id: Uuid::nil(),
                group_id: Uuid::nil(),
            },
            asset_type: 13, // Bodypart
            inv_type: 18,   // Wearable
            flags: 0,
            sale_info: SaleInfo {
                sale_type: 0,
                sale_price: 0,
            },
            name: "Default Avatar Shape".to_string(),
            desc: "Default avatar body shape".to_string(),
            creation_date: chrono::Utc::now().timestamp() as u32,
        });

        // Add default skin item
        items.push(InventoryItem {
            item_id: Uuid::new_v4(),
            parent_id: clothing_folder.folder_id,
            asset_id: Uuid::parse_str("77c41e39-38f9-f75a-024e-585989bfab73").unwrap_or_default(),
            permissions: ItemPermissions {
                base_mask: 0x7FFFFFFF,
                owner_mask: 0x7FFFFFFF,
                group_mask: 0,
                everyone_mask: 0,
                next_owner_mask: 0x7FFFFFFF,
                creator_id: Uuid::nil(),
                owner_id: user_account.id,
                last_owner_id: Uuid::nil(),
                group_id: Uuid::nil(),
            },
            asset_type: 13, // Bodypart
            inv_type: 18,   // Wearable
            flags: 0,
            sale_info: SaleInfo {
                sale_type: 0,
                sale_price: 0,
            },
            name: "Default Skin".to_string(),
            desc: "Default avatar skin".to_string(),
            creation_date: chrono::Utc::now().timestamp() as u32,
        });

        // Add default hair item
        items.push(InventoryItem {
            item_id: Uuid::new_v4(),
            parent_id: clothing_folder.folder_id,
            asset_id: Uuid::parse_str("88c41e39-38f9-f75a-024e-585989bfab84").unwrap_or_default(),
            permissions: ItemPermissions {
                base_mask: 0x7FFFFFFF,
                owner_mask: 0x7FFFFFFF,
                group_mask: 0,
                everyone_mask: 0,
                next_owner_mask: 0x7FFFFFFF,
                creator_id: Uuid::nil(),
                owner_id: user_account.id,
                last_owner_id: Uuid::nil(),
                group_id: Uuid::nil(),
            },
            asset_type: 13, // Bodypart
            inv_type: 18,   // Wearable
            flags: 0,
            sale_info: SaleInfo {
                sale_type: 0,
                sale_price: 0,
            },
            name: "Default Hair".to_string(),
            desc: "Default avatar hair".to_string(),
            creation_date: chrono::Utc::now().timestamp() as u32,
        });

        Ok(UserInventory {
            root_folders: vec![
                root_folder,
                clothing_folder,
                objects_folder,
                textures_folder,
                sounds_folder,
                scripts_folder,
                landmarks_folder,
            ],
            items,
        })
    }

    async fn get_global_textures(&self) -> Result<Vec<GlobalTexture>> {
        // Default textures for sky, moon, sun
        Ok(vec![GlobalTexture {
            cloud_texture_id: Uuid::parse_str("dc4b9f0b-d008-45c6-96a4-01dd947ac621")?,
            moon_texture_id: Uuid::parse_str("ec4b9f0b-d008-45c6-96a4-01dd947ac621")?,
            sun_texture_id: Uuid::parse_str("cce0f112-878f-4586-a2e2-a8f104bba271")?,
        }])
    }

    async fn get_login_flags(&self) -> Vec<LoginFlag> {
        vec![
            LoginFlag {
                key: "stipend_since_login".to_string(),
                value: LLSDValue::String("N".to_string()),
            },
            LoginFlag {
                key: "gendered".to_string(),
                value: LLSDValue::String("Y".to_string()),
            },
            LoginFlag {
                key: "ever_logged_in".to_string(),
                value: LLSDValue::String("Y".to_string()),
            },
        ]
    }

    /// Create session capabilities for Second Life viewer protocol
    async fn create_session_capabilities(
        &self,
        session_id: Uuid,
        user_account: &UserAccount,
    ) -> Result<Vec<Capability>> {
        // Essential capabilities that SL viewers expect
        let base_url = format!("http://127.0.0.1:9000/cap/{}", session_id);

        Ok(vec![
            Capability {
                name: "AvatarPickerSearch".to_string(),
                url: format!("{}/AvatarPickerSearch", base_url),
            },
            Capability {
                name: "ChatSessionRequest".to_string(),
                url: format!("{}/ChatSessionRequest", base_url),
            },
            Capability {
                name: "CopyInventoryFromNotecard".to_string(),
                url: format!("{}/CopyInventoryFromNotecard", base_url),
            },
            Capability {
                name: "CreateInventoryCategory".to_string(),
                url: format!("{}/CreateInventoryCategory", base_url),
            },
            Capability {
                name: "CreateInventoryItem".to_string(),
                url: format!("{}/CreateInventoryItem", base_url),
            },
            Capability {
                name: "EstateChangeInfo".to_string(),
                url: format!("{}/EstateChangeInfo", base_url),
            },
            Capability {
                name: "EventQueueGet".to_string(),
                url: format!("{}/EventQueueGet", base_url),
            },
            Capability {
                name: "FetchInventory2".to_string(),
                url: format!("{}/FetchInventory2", base_url),
            },
            Capability {
                name: "FetchLib2".to_string(),
                url: format!("{}/FetchLib2", base_url),
            },
            Capability {
                name: "FetchLibDescendents2".to_string(),
                url: format!("{}/FetchLibDescendents2", base_url),
            },
            Capability {
                name: "GetTexture".to_string(),
                url: format!("{}/GetTexture", base_url),
            },
            Capability {
                name: "GroupProposalBallot".to_string(),
                url: format!("{}/GroupProposalBallot", base_url),
            },
            Capability {
                name: "HomeLocation".to_string(),
                url: format!("{}/HomeLocation", base_url),
            },
            Capability {
                name: "LandResources".to_string(),
                url: format!("{}/LandResources", base_url),
            },
            Capability {
                name: "MapLayer".to_string(),
                url: format!("{}/MapLayer", base_url),
            },
            Capability {
                name: "MapLayerGod".to_string(),
                url: format!("{}/MapLayerGod", base_url),
            },
            Capability {
                name: "NewFileAgentInventory".to_string(),
                url: format!("{}/NewFileAgentInventory", base_url),
            },
            Capability {
                name: "ParcelPropertiesUpdate".to_string(),
                url: format!("{}/ParcelPropertiesUpdate", base_url),
            },
            Capability {
                name: "ParcelVoiceInfoRequest".to_string(),
                url: format!("{}/ParcelVoiceInfoRequest", base_url),
            },
            Capability {
                name: "ProvisionVoiceAccountRequest".to_string(),
                url: format!("{}/ProvisionVoiceAccountRequest", base_url),
            },
            Capability {
                name: "RemoteParcelRequest".to_string(),
                url: format!("{}/RemoteParcelRequest", base_url),
            },
            Capability {
                name: "RequestTextureDownload".to_string(),
                url: format!("{}/RequestTextureDownload", base_url),
            },
            Capability {
                name: "SearchStatRequest".to_string(),
                url: format!("{}/SearchStatRequest", base_url),
            },
            Capability {
                name: "SearchStatTracking".to_string(),
                url: format!("{}/SearchStatTracking", base_url),
            },
            Capability {
                name: "SendPostcard".to_string(),
                url: format!("{}/SendPostcard", base_url),
            },
            Capability {
                name: "SendUserReport".to_string(),
                url: format!("{}/SendUserReport", base_url),
            },
            Capability {
                name: "SendUserReportWithScreenshot".to_string(),
                url: format!("{}/SendUserReportWithScreenshot", base_url),
            },
            Capability {
                name: "ServerReleaseNotes".to_string(),
                url: format!("{}/ServerReleaseNotes", base_url),
            },
            Capability {
                name: "StartGroupProposal".to_string(),
                url: format!("{}/StartGroupProposal", base_url),
            },
            Capability {
                name: "TextureStats".to_string(),
                url: format!("{}/TextureStats", base_url),
            },
            Capability {
                name: "UntrustedSimulatorMessage".to_string(),
                url: format!("{}/UntrustedSimulatorMessage", base_url),
            },
            Capability {
                name: "UpdateAvatarAppearance".to_string(),
                url: format!("{}/UpdateAvatarAppearance", base_url),
            },
            Capability {
                name: "UpdateGestureAgentInventory".to_string(),
                url: format!("{}/UpdateGestureAgentInventory", base_url),
            },
            Capability {
                name: "UpdateGestureTaskInventory".to_string(),
                url: format!("{}/UpdateGestureTaskInventory", base_url),
            },
            Capability {
                name: "UpdateNotecardAgentInventory".to_string(),
                url: format!("{}/UpdateNotecardAgentInventory", base_url),
            },
            Capability {
                name: "UpdateNotecardTaskInventory".to_string(),
                url: format!("{}/UpdateNotecardTaskInventory", base_url),
            },
            Capability {
                name: "UpdateScriptAgent".to_string(),
                url: format!("{}/UpdateScriptAgent", base_url),
            },
            Capability {
                name: "UpdateScriptTask".to_string(),
                url: format!("{}/UpdateScriptTask", base_url),
            },
            Capability {
                name: "UploadBakedTexture".to_string(),
                url: format!("{}/UploadBakedTexture", base_url),
            },
            Capability {
                name: "ViewerAsset".to_string(),
                url: format!("{}/ViewerAsset", base_url),
            },
            Capability {
                name: "ViewerStats".to_string(),
                url: format!("{}/ViewerStats", base_url),
            },
        ])
    }

    /// Create home location information for the user
    async fn create_home_location(
        &self,
        user_account: &UserAccount,
        region_info: &RegionInfo,
    ) -> Result<HomeLocation> {
        // CRITICAL: region_handle uses METER coordinates, not grid coordinates
        // Grid (1000, 1000) → Meters (256000, 256000)
        let region_handle = (((region_info.x as u64) * 256) << 32) | ((region_info.y as u64) * 256);

        Ok(HomeLocation {
            region_handle,
            position: [
                user_account.home_position_x,
                user_account.home_position_y,
                user_account.home_position_z,
            ],
            look_at: [1.0, 0.0, 0.0], // Default look direction
        })
    }

    /// Get user's friends list
    async fn get_user_friends(&self, user_account: &UserAccount) -> Result<Vec<BuddyListEntry>> {
        // Load user's friends from database
        debug!("Loading friends list for user: {}", user_account.id);

        // In a full implementation, this would query a friends/relationships table
        // For now, we'll create a basic friends list structure

        // This would typically be:
        // SELECT friend_id, rights_given, rights_received
        // FROM user_friends
        // WHERE user_id = $1 AND status = 'accepted'

        let mut friends = Vec::new();

        // For demonstration, add some default system contacts if this is a new user
        // In production, this would be loaded from database

        // Add system administrator as friend (if exists)
        if user_account.user_level >= 0 {
            // Active user
            friends.push(BuddyListEntry {
                buddy_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                    .unwrap_or_default(),
                buddy_rights_given: 0x1F, // Basic friend rights
                buddy_rights_has: 0x1F,   // Reciprocal rights
            });
        }

        // Add welcome bot as friend for new users
        if user_account.user_flags == 0 {
            // New user indicator
            friends.push(BuddyListEntry {
                buddy_id: Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                    .unwrap_or_default(),
                buddy_rights_given: 0x0F, // Limited rights for bot
                buddy_rights_has: 0x01,   // Bot can see online status
            });
        }

        debug!(
            "Loaded {} friends for user {}",
            friends.len(),
            user_account.id
        );
        Ok(friends)
    }

    /// Get user's active gestures
    async fn get_user_gestures(&self, user_account: &UserAccount) -> Result<Vec<GestureEntry>> {
        // Load user's active gestures from database
        debug!("Loading active gestures for user: {}", user_account.id);

        // In a full implementation, this would query a user_gestures table
        // For now, we'll provide some default gestures

        // This would typically be:
        // SELECT item_id, asset_id FROM user_gestures
        // WHERE user_id = $1 AND is_active = true

        let mut gestures = Vec::new();

        // Add some default gestures for all users
        // These would typically be loaded from the user's inventory

        // Default wave gesture
        gestures.push(GestureEntry {
            item_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap_or_default(),
            asset_id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap_or_default(),
        });

        // Default hello gesture
        gestures.push(GestureEntry {
            item_id: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap_or_default(),
            asset_id: Uuid::parse_str("44444444-4444-4444-4444-444444444444").unwrap_or_default(),
        });

        // Default dance gesture for users with positive standing
        if user_account.user_level > 0 {
            gestures.push(GestureEntry {
                item_id: Uuid::parse_str("55555555-5555-5555-5555-555555555555")
                    .unwrap_or_default(),
                asset_id: Uuid::parse_str("66666666-6666-6666-6666-666666666666")
                    .unwrap_or_default(),
            });
        }

        debug!(
            "Loaded {} active gestures for user {}",
            gestures.len(),
            user_account.id
        );
        Ok(gestures)
    }

    /// Get initial outfit configurations
    async fn get_initial_outfits(&self) -> Vec<InitialOutfitFolder> {
        vec![
            InitialOutfitFolder {
                folder_name: "Female Shape & Outfit".to_string(),
                gender: "female".to_string(),
            },
            InitialOutfitFolder {
                folder_name: "Male Shape & Outfit".to_string(),
                gender: "male".to_string(),
            },
        ]
    }

    /// Get user appearance from database or preferences
    async fn get_user_appearance(&self, user_account: &UserAccount) -> Result<Appearance> {
        // Try to load saved appearance from database first
        // For now, we'll check if the user has appearance preferences stored
        debug!("Loading appearance for user: {}", user_account.id);

        // This would typically query a user preferences table or avatar appearance table
        // For now, we'll create a basic appearance based on user's stored preferences
        // In a full implementation, this would load from a dedicated appearances table

        // Check if user has gender preference stored (this would come from registration)
        let is_male = user_account
            .first_name
            .to_lowercase()
            .chars()
            .next()
            .map(|c| c as u32 % 2 == 0) // Simple heuristic for demo
            .unwrap_or(false);

        if is_male {
            Ok(Appearance::default_male())
        } else {
            Ok(Appearance::default_female())
        }
    }
}

/// Region information for login
#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub x: u32,
    pub y: u32,
    pub size_x: u32,
    pub size_y: u32,
}

/// User inventory structure
#[derive(Debug, Clone)]
pub struct UserInventory {
    pub root_folders: Vec<InventoryFolder>,
    pub items: Vec<InventoryItem>,
}

/// Simple HTTP-based login server for Second Life viewer compatibility
pub struct LoginServer {
    user_db: Arc<UserAccountDatabase>,
    port: u16,
}

impl LoginServer {
    pub fn new(user_db: Arc<UserAccountDatabase>, port: u16) -> Self {
        Self { user_db, port }
    }

    pub async fn start(&self) -> Result<()> {
        use axum::{extract::Form, response::Json, routing::post, Router};
        use serde_json::json;
        use std::collections::HashMap;

        let user_db = self.user_db.clone();

        let app = Router::new()
            .route("/", post(handle_login))
            .route("/login", post(handle_login))
            .route("/xmlrpc", post(handle_login))
            // Legacy viewer path aliases (Cool Viewer, Singularity, etc.)
            .route("/xmlrpc.php", post(handle_login))
            .route("/login.cgi", post(handle_login))
            .route("/cgi-bin/login.cgi", post(handle_login))
            .with_state(user_db);

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Starting login server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

// EADS: SQLite authentication completely removed - PostgreSQL only

async fn handle_login(
    axum::extract::State(user_db): axum::extract::State<Arc<UserAccountDatabase>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::{Json, Response};
    use serde_json::json;

    // Check Content-Type to determine if this is XML-RPC or form data
    // Support various viewer implementations (Cool Viewer, Singularity, Firestorm, etc.)
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let xml_body = String::from_utf8_lossy(&body);
    let is_xmlrpc = content_type.contains("text/xml")
        || content_type.contains("application/xml")
        || xml_body.trim().starts_with("<?xml")
        || xml_body.trim().starts_with("<methodCall");

    let (first, last, passwd) = if is_xmlrpc {
        // Handle XML-RPC request from Second Life viewer
        let xml_body = String::from_utf8_lossy(&body);
        info!("Received XML-RPC login request: {}", xml_body);

        // Parse XML-RPC parameters (simple extraction for OpenSim compatibility)
        let first = extract_xml_param(&xml_body, "first").unwrap_or_default();
        let last = extract_xml_param(&xml_body, "last").unwrap_or_default();
        let passwd = extract_xml_param(&xml_body, "passwd").unwrap_or_default();

        (first, last, passwd)
    } else {
        // Handle form data request
        let form_data = String::from_utf8_lossy(&body);
        let parsed_form: std::collections::HashMap<String, String> =
            serde_urlencoded::from_str(&form_data).unwrap_or_default();

        let first = parsed_form.get("first").unwrap_or(&"".to_string()).clone();
        let last = parsed_form.get("last").unwrap_or(&"".to_string()).clone();
        let passwd = parsed_form.get("passwd").unwrap_or(&"".to_string()).clone();

        (first, last, passwd)
    };

    info!("Login attempt: {} {}", first, last);

    // Try to authenticate with fallback
    let username = format!("{} {}", first, last);

    // EADS: PostgreSQL-only authentication
    let user_result = user_db
        .authenticate_user_opensim(&first, &last, &passwd)
        .await;

    let result = match user_result {
        Ok(Some(user)) => {
            info!("Login successful for: {}", username);

            // FIXED: Use viewer-compatible circuit code (Cool Viewer sends 16776960)
            let circuit_code = 16776960u32; // 0x00FF0100 - Cool Viewer protocol constant
            info!(
                "🎯 Using viewer-compatible circuit code: {} for user: {}",
                circuit_code, username
            );

            json!({
                "login": "true",
                "message": "Welcome to OpenSim Next!",
                "reason": "success",
                "session_id": uuid::Uuid::new_v4().to_string(),
                "secure_session_id": uuid::Uuid::new_v4().to_string(),
                "agent_id": user.id.to_string(),
                "first_name": user.first_name,
                "last_name": user.last_name,
                "sim_ip": "127.0.0.1",
                "sim_port": 9000,
                "circuit_code": circuit_code,
                "region_x": 256000,
                "region_y": 256000,
                "region_size_x": 256,
                "region_size_y": 256,
                "start_location": "last",
                "look_at": [128.0, 128.0, 25.0],
                "seed_capability": format!("http://localhost:8080/cap/{}", uuid::Uuid::new_v4()),
                "inventory_host": "localhost:8080"
            })
        }
        Ok(None) => {
            warn!("Login failed - invalid credentials: {}", username);
            json!({
                "login": "false",
                "message": "Invalid username or password",
                "reason": "Authentication failed"
            })
        }
        Err(e) => {
            error!("Login error: {}", e);
            json!({
                "login": "false",
                "message": "Server error during login",
                "reason": format!("Error: {}", e)
            })
        }
    };

    // Return XML-RPC response if XML request, JSON if form request
    if is_xmlrpc {
        // Return XML-RPC response for Second Life viewers
        let xml_response = format_xml_rpc_response(&result);
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/xml")
            .body(xml_response.into())
            .unwrap()
    } else {
        // Return JSON response for form requests
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&result).unwrap().into())
            .unwrap()
    }
}

// Simple XML parameter extraction for OpenSim XML-RPC compatibility
fn extract_xml_param(xml: &str, param_name: &str) -> Option<String> {
    // Look for <name>param_name</name> followed by <value><string>value</string></value>
    let name_pattern = format!("<name>{}</name>", param_name);
    if let Some(name_pos) = xml.find(&name_pattern) {
        if let Some(value_start) = xml[name_pos..].find("<string>") {
            let value_pos = name_pos + value_start + 8; // 8 = len("<string>")
            if let Some(value_end) = xml[value_pos..].find("</string>") {
                return Some(xml[value_pos..value_pos + value_end].to_string());
            }
        }
    }
    None
}

// Format XML-RPC response for Second Life viewers
fn format_xml_rpc_response(json_result: &serde_json::Value) -> String {
    if json_result.get("login").and_then(|v| v.as_str()) == Some("true") {
        // Success response with complete inventory data
        format!(
            r#"<?xml version="1.0"?>
<methodResponse>
<params>
<param>
<value>
<struct>
<member><name>login</name><value><string>true</string></value></member>
<member><name>session_id</name><value><string>{}</string></value></member>
<member><name>agent_id</name><value><string>{}</string></value></member>
<member><name>first_name</name><value><string>{}</string></value></member>
<member><name>last_name</name><value><string>{}</string></value></member>
<member><name>sim_ip</name><value><string>127.0.0.1</string></value></member>
<member><name>sim_port</name><value><i4>9000</i4></value></member>
<member><name>region_x</name><value><i4>256000</i4></value></member>
<member><name>region_y</name><value><i4>256000</i4></value></member>
<member><name>look_at</name><value><string>[r1,r1,r0]</string></value></member>
<member><name>agent_access</name><value><string>M</string></value></member>
<member><name>start_location</name><value><string>home</string></value></member>
<member><name>circuit_code</name><value><i4>{}</i4></value></member>
<member><name>secure_session_id</name><value><string>{}</string></value></member>
<member><name>inventory-root</name><value><array><data>
<value><struct>
<member><name>folder_id</name><value><string>00000000-0000-0000-0000-000000000001</string></value></member>
<member><name>parent_id</name><value><string>00000000-0000-0000-0000-000000000000</string></value></member>
<member><name>name</name><value><string>My Inventory</string></value></member>
<member><name>type_default</name><value><i4>8</i4></value></member>
<member><name>version</name><value><i4>1</i4></value></member>
</struct></value>
</data></array></value></member>
<member><name>inventory-skeleton</name><value><array><data>
<value><struct>
<member><name>folder_id</name><value><string>00000000-0000-0000-0000-000000000002</string></value></member>
<member><name>parent_id</name><value><string>00000000-0000-0000-0000-000000000001</string></value></member>
<member><name>name</name><value><string>Textures</string></value></member>
<member><name>type_default</name><value><i4>0</i4></value></member>
<member><name>version</name><value><i4>1</i4></value></member>
</struct></value>
<value><struct>
<member><name>folder_id</name><value><string>00000000-0000-0000-0000-000000000003</string></value></member>
<member><name>parent_id</name><value><string>00000000-0000-0000-0000-000000000001</string></value></member>
<member><name>name</name><value><string>Sounds</string></value></member>
<member><name>type_default</name><value><i4>1</i4></value></member>
<member><name>version</name><value><i4>1</i4></value></member>
</struct></value>
<value><struct>
<member><name>folder_id</name><value><string>00000000-0000-0000-0000-000000000004</string></value></member>
<member><name>parent_id</name><value><string>00000000-0000-0000-0000-000000000001</string></value></member>
<member><name>name</name><value><string>Calling Cards</string></value></member>
<member><name>type_default</name><value><i4>2</i4></value></member>
<member><name>version</name><value><i4>1</i4></value></member>
</struct></value>
<value><struct>
<member><name>folder_id</name><value><string>00000000-0000-0000-0000-000000000005</string></value></member>
<member><name>parent_id</name><value><string>00000000-0000-0000-0000-000000000001</string></value></member>
<member><name>name</name><value><string>Landmarks</string></value></member>
<member><name>type_default</name><value><i4>3</i4></value></member>
<member><name>version</name><value><i4>1</i4></value></member>
</struct></value>
</data></array></value></member>
<member><name>inventory-lib-root</name><value><array><data>
<value><struct>
<member><name>folder_id</name><value><string>00000000-0000-0000-0000-000000000010</string></value></member>
<member><name>parent_id</name><value><string>00000000-0000-0000-0000-000000000000</string></value></member>
<member><name>name</name><value><string>Library</string></value></member>
<member><name>type_default</name><value><i4>8</i4></value></member>
<member><name>version</name><value><i4>1</i4></value></member>
</struct></value>
</data></array></value></member>
<member><name>initial-outfit</name><value><array><data>
<value><struct>
<member><name>folder_name</name><value><string>Default Outfit</string></value></member>
<member><name>gender</name><value><string>both</string></value></member>
</struct></value>
</data></array></value></member>
<member><name>seed_capability</name><value><string>http://127.0.0.1:9000/CAPS/{}</string></value></member>
</struct>
</value>
</param>
</params>
</methodResponse>"#,
            json_result
                .get("session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("00000000-0000-0000-0000-000000000000"),
            json_result
                .get("agent_id")
                .and_then(|v| v.as_str())
                .unwrap_or("00000000-0000-0000-0000-000000000000"),
            json_result
                .get("first_name")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            json_result
                .get("last_name")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            json_result
                .get("circuit_code")
                .and_then(|v| v.as_u64())
                .unwrap_or(123456),
            json_result
                .get("secure_session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("00000000-0000-0000-0000-000000000000"),
            json_result
                .get("session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("00000000-0000-0000-0000-000000000000")
        )
    } else {
        // Error response
        format!(
            r#"<?xml version="1.0"?>
<methodResponse>
<params>
<param>
<value>
<struct>
<member><name>login</name><value><string>false</string></value></member>
<member><name>message</name><value><string>{}</string></value></member>
<member><name>reason</name><value><string>{}</string></value></member>
</struct>
</value>
</param>
</params>
</methodResponse>"#,
            json_result
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Login failed"),
            json_result
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("Authentication failed")
        )
    }
}
