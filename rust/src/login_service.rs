use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Response},
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::auth::PasswordAuth;
use crate::capabilities::{CapabilitiesConfig, CapabilitiesManager};
use crate::caps::CapsManager;
use crate::config::login::{LoginConfig, LoginConfigManager};
use crate::database::multi_backend::DatabaseConnection;
use crate::database::user_accounts::UserAccountDatabase;
use crate::database::DatabaseManager;
use crate::inventory::LoginInventoryService;
use crate::login_session::CircuitCodeRegistry;
use crate::login_stage_tracker::LoginStageTracker;
use crate::services::factory::ServiceContainer;
use crate::services::AvatarService;
use crate::session::{LoginSession, SessionManager};
use crate::udp::UdpServer;
use crate::xmlrpc::{LoginRequest, LoginResponse, XmlRpcParser, XmlRpcResponseGenerator};

#[derive(Clone)]
pub struct CurrencyState {
    pub database_manager: Option<Arc<DatabaseManager>>,
}

pub struct LoginService {
    config_manager: LoginConfigManager,
    session_manager: Arc<SessionManager>,
    user_database: Arc<UserAccountDatabase>,
    avatar_service: Arc<AvatarService>,
    inventory_service: Arc<std::sync::Mutex<LoginInventoryService>>,
    capabilities_manager: Arc<std::sync::Mutex<CapabilitiesManager>>,
    caps_manager: Option<Arc<CapsManager>>,
    udp_server: Option<Arc<UdpServer>>,
    circuit_registry: Option<Arc<CircuitCodeRegistry>>,
    stage_tracker: Option<Arc<LoginStageTracker>>,
    db_pool: Option<Arc<sqlx::PgPool>>,
    service_container: Option<Arc<ServiceContainer>>,
    hg_service_urls: std::collections::HashMap<String, String>,
    security_manager: Option<Arc<crate::network::security::SecurityManager>>,
    uas_service: Option<Arc<dyn crate::services::traits::UserAgentServiceTrait>>,
    gatekeeper_service: Option<Arc<dyn crate::services::traits::GatekeeperServiceTrait>>,
    readiness_tracker: Option<Arc<crate::readiness::ReadinessTracker>>,
}

impl LoginService {
    pub fn new(
        config_manager: LoginConfigManager,
        session_manager: Arc<SessionManager>,
        user_database: Arc<UserAccountDatabase>,
        avatar_service: Arc<AvatarService>,
    ) -> Self {
        let summary = config_manager.get_summary();
        info!(
            "Initializing Login Service with config summary: {:?}",
            summary
        );

        // Create capabilities manager with config from login config
        let config = config_manager.config();
        let capabilities_config = CapabilitiesConfig {
            base_url: config.capabilities.base_url.clone(),
            timeout_seconds: config.capabilities.timeout_seconds,
            max_capabilities: 100,
            enable_essential: config.capabilities.enabled,
            enable_uploads: config.capabilities.enabled,
            enable_advanced: false,
            custom_handlers: std::collections::HashMap::new(),
        };

        Self {
            config_manager,
            session_manager,
            user_database,
            avatar_service,
            inventory_service: Arc::new(std::sync::Mutex::new(LoginInventoryService::new())),
            capabilities_manager: Arc::new(std::sync::Mutex::new(CapabilitiesManager::new(
                capabilities_config,
            ))),
            caps_manager: None,
            udp_server: None,
            circuit_registry: None,
            stage_tracker: None,
            db_pool: None,
            service_container: None,
            hg_service_urls: std::collections::HashMap::new(),
            security_manager: None,
            uas_service: None,
            gatekeeper_service: None,
            readiness_tracker: None,
        }
    }

    /// Create login service with default configuration
    pub fn with_defaults(
        session_manager: Arc<SessionManager>,
        user_database: Arc<UserAccountDatabase>,
        avatar_service: Arc<AvatarService>,
    ) -> Self {
        let config_manager = LoginConfigManager::new();
        Self::new(
            config_manager,
            session_manager,
            user_database,
            avatar_service,
        )
    }

    /// Create login service from configuration file
    pub fn from_config_file(
        config_file: &str,
        session_manager: Arc<SessionManager>,
        user_database: Arc<UserAccountDatabase>,
        avatar_service: Arc<AvatarService>,
    ) -> Result<Self> {
        let config_manager = LoginConfigManager::load_from_file(config_file)?;
        config_manager.validate()?;
        Ok(Self::new(
            config_manager,
            session_manager,
            user_database,
            avatar_service,
        ))
    }

    /// Create login service from environment variables
    pub fn from_environment(
        session_manager: Arc<SessionManager>,
        user_database: Arc<UserAccountDatabase>,
        avatar_service: Arc<AvatarService>,
    ) -> Result<Self> {
        let config_manager = LoginConfigManager::load_from_env()?;
        config_manager.validate()?;
        Ok(Self::new(
            config_manager,
            session_manager,
            user_database,
            avatar_service,
        ))
    }

    /// Set the circuit code registry for XMLRPC->UDP session coordination
    pub fn set_circuit_registry(&mut self, circuit_registry: Arc<CircuitCodeRegistry>) {
        self.circuit_registry = Some(circuit_registry);
    }

    /// Set the login stage tracker for diagnostic visibility
    pub fn set_stage_tracker(&mut self, stage_tracker: Arc<LoginStageTracker>) {
        self.stage_tracker = Some(stage_tracker);
    }

    /// Set the PostgreSQL database pool for inventory queries
    pub fn set_db_pool(&mut self, db_pool: Arc<sqlx::PgPool>) {
        self.db_pool = Some(db_pool);
    }

    pub fn set_service_container(&mut self, container: Arc<ServiceContainer>) {
        self.service_container = Some(container);
    }

    pub fn set_hg_service_urls(&mut self, urls: std::collections::HashMap<String, String>) {
        self.hg_service_urls = urls;
    }

    pub fn set_security_manager(&mut self, sm: Arc<crate::network::security::SecurityManager>) {
        self.security_manager = Some(sm);
    }

    pub fn set_readiness_tracker(&mut self, tracker: Arc<crate::readiness::ReadinessTracker>) {
        self.readiness_tracker = Some(tracker);
    }

    pub fn set_uas_service(
        &mut self,
        uas: Arc<dyn crate::services::traits::UserAgentServiceTrait>,
    ) {
        self.uas_service = Some(uas);
    }

    pub fn set_gatekeeper_service(
        &mut self,
        gk: Arc<dyn crate::services::traits::GatekeeperServiceTrait>,
    ) {
        self.gatekeeper_service = Some(gk);
    }

    pub async fn start_udp_server(
        &mut self,
        db_connection: Option<Arc<DatabaseConnection>>,
    ) -> Result<()> {
        let config = self.config_manager.config();
        info!("Starting UDP server on port {}", config.server.udp_port);

        let stage_tracker = self
            .stage_tracker
            .clone()
            .unwrap_or_else(|| Arc::new(LoginStageTracker::new()));

        let default_region_uuid = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001")
            .expect("Valid default region UUID");
        let region_x: u32 = (config.region.region_x as u32) / 256;
        let region_y: u32 = (config.region.region_y as u32) / 256;
        let mut udp_server = UdpServer::new(
            &format!("{}:{}", "0.0.0.0", config.server.udp_port),
            self.session_manager.clone(),
            stage_tracker,
            default_region_uuid,
            ((region_x as u64 * 256) << 32) | (region_y as u64 * 256),
            region_x,
            region_y,
            config.region.region_size_x as u32,
            config.region.region_size_y as u32,
            20.0,
            config.region.default_region_name.clone(),
        )
        .await?;

        if let Some(db_conn) = db_connection {
            info!("[TERRAIN] Initializing UDP server with database connection");
            udp_server = udp_server.with_database(db_conn);

            if let Err(e) = udp_server.initialize_terrain().await {
                warn!("[TERRAIN] Failed to initialize default terrain: {}", e);
            } else {
                info!("[TERRAIN] Default region terrain initialized successfully");
            }
        } else {
            warn!("[TERRAIN] No database connection provided, terrain system disabled");
        }

        let udp_server = Arc::new(udp_server);
        self.udp_server = Some(udp_server.clone());

        // Start UDP server in background task
        let udp_server_clone = udp_server.clone();
        tokio::spawn(async move {
            if let Err(e) = udp_server_clone.run().await {
                error!("UDP server error: {}", e);
            }
        });

        info!("UDP server started successfully");
        Ok(())
    }

    pub async fn handle_login_request(&self, request_body: String) -> Result<Response<Body>> {
        debug!("🔍 XMLRPC TRACE: Starting handle_login_request");
        debug!(
            "🔍 XMLRPC TRACE: Request body length: {} bytes",
            request_body.len()
        );

        // Parse XMLRPC request
        debug!("🔍 XMLRPC TRACE: About to call XmlRpcParser::parse_login_request");
        let login_request = match XmlRpcParser::parse_login_request(&request_body) {
            Ok(request) => {
                debug!("🔍 XMLRPC TRACE: Successfully parsed login request");
                request
            }
            Err(e) => {
                warn!("🔍 XMLRPC TRACE: Failed to parse login request: {}", e);
                let response = LoginResponse::failure("parse", "Invalid login request format");
                let xml = XmlRpcResponseGenerator::generate_login_response(&response)?;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/xml")
                    .body(Body::from(xml))?);
            }
        };

        info!(
            "Login attempt: first='{}' last='{}' (bytes: {:02x?} {:02x?})",
            login_request.first,
            login_request.last,
            login_request.first.as_bytes(),
            login_request.last.as_bytes()
        );
        debug!("🔍 XMLRPC TRACE: About to call authenticate_user");

        // Authenticate user (with brute-force protection)
        match self.authenticate_user(&login_request).await {
            Ok(session) => {
                info!(
                    "Successful login for {} {} (circuit: {})",
                    session.first_name, session.last_name, session.circuit_code
                );

                debug!(
                    "[Phase 68] Login IDs: agent={} session={}",
                    session.agent_id, session.session_id
                );

                let config = self.config_manager.config();

                // Load or create avatar appearance
                // Matches: OpenSim LLLoginService.cs:588-592
                debug!(
                    "🔍 TRACE: Loading avatar appearance for agent_id: {}",
                    session.agent_id
                );
                match self
                    .avatar_service
                    .get_or_create_default_appearance(session.agent_id)
                    .await
                {
                    Ok(avatar) => {
                        info!(
                            "[AVATAR SERVICE]: Avatar appearance loaded for {} (Serial: {})",
                            session.agent_id,
                            avatar.data.get("Serial").unwrap_or(&"unknown".to_string())
                        );
                        // Avatar data will be used in Phase 64.4/64.5 for UDP ObjectUpdate packets
                    }
                    Err(e) => {
                        warn!("[AVATAR SERVICE]: Failed to load avatar appearance: {}", e);
                        // Continue with login - avatar will use defaults
                    }
                }
                debug!("🔍 TRACE: Avatar appearance processing completed");

                // Get user inventory for login response
                // Phase 83.2: Query PostgreSQL database for actual folder UUIDs
                debug!("🔍 TRACE: About to get inventory for login response");
                if let Some(ref db_pool) = self.db_pool {
                    match crate::database::default_inventory::ensure_user_has_inventory(
                        db_pool,
                        session.agent_id,
                    )
                    .await
                    {
                        Ok(true) => info!(
                            "[INVENTORY] Created missing default inventory for agent {}",
                            session.agent_id
                        ),
                        Ok(false) => {}
                        Err(e) => warn!(
                            "[INVENTORY] ensure_user_has_inventory failed for {}: {}",
                            session.agent_id, e
                        ),
                    }
                }
                let inventory = if let Some(ref db_pool) = self.db_pool {
                    info!(
                        "[INVENTORY] Querying PostgreSQL for inventory folders for agent {}",
                        session.agent_id
                    );
                    match self
                        .query_inventory_from_database(db_pool, session.agent_id)
                        .await
                    {
                        Ok(inv) => {
                            info!(
                                "[INVENTORY] Loaded {} skeleton folders from DATABASE for agent {}",
                                inv.folder_count(),
                                session.agent_id
                            );
                            for folder in &inv.inventory_skeleton {
                                debug!(
                                    "[INVENTORY]   folder: {} type={} id={} parent={}",
                                    folder.name,
                                    folder.type_default,
                                    folder.folder_id,
                                    folder.parent_id
                                );
                            }
                            if let Some(root) = inv.inventory_root.first() {
                                debug!("[INVENTORY]   ROOT: {} id={}", root.name, root.folder_id);
                            }
                            inv
                        }
                        Err(e) => {
                            warn!("[INVENTORY] Database query failed ({}), falling back to generated inventory", e);
                            let mut inv_service = self.inventory_service.lock().unwrap();
                            inv_service.get_login_inventory(session.agent_id)
                        }
                    }
                } else {
                    warn!("[INVENTORY] No db_pool available, using generated inventory (will have wrong folder UUIDs!)");
                    let mut inv_service = self.inventory_service.lock().unwrap();
                    inv_service.get_login_inventory(session.agent_id)
                };
                info!(
                    "[INVENTORY] Inventory skeleton ready with {} folders",
                    inventory.inventory_skeleton.len()
                );

                // Create capabilities for the agent
                debug!("🔍 TRACE: About to lock capabilities manager mutex");
                let seed_capability_url = {
                    debug!("🔍 TRACE: Attempting capabilities_manager.lock()...");
                    let mut caps_manager = self.capabilities_manager.lock().unwrap();
                    debug!("🔍 TRACE: Successfully locked capabilities manager, calling create_agent_capabilities");
                    match caps_manager
                        .create_agent_capabilities(session.agent_id, session.session_id)
                    {
                        Ok(seed_url) => {
                            debug!(
                                "🔍 TRACE: create_agent_capabilities succeeded: {}",
                                seed_url
                            );
                            seed_url
                        }
                        Err(e) => {
                            warn!("🔍 TRACE: create_agent_capabilities failed: {}", e);
                            // Use default seed capability URL as fallback
                            format!("{}/cap/{}", config.server.base_url, session.agent_id)
                        }
                    }
                };
                debug!("🔍 TRACE: Capabilities generation completed");

                let final_seed_capability_url = if let Some(caps_manager) = &self.caps_manager {
                    debug!("🔍 TRACE: Registering session with CapsManager");
                    let caps_session_id = caps_manager
                        .create_session(session.agent_id.to_string(), session.circuit_code, None)
                        .await;
                    debug!("🔍 TRACE: CapsManager session created: {}", caps_session_id);
                    // Use CapsManager's base URL structure for seed capability
                    format!("{}/cap/{}", config.server.base_url, caps_session_id)
                } else {
                    seed_capability_url
                };

                debug!(
                    "🔍 TRACE: Creating LoginResponse::success with seed capability: {}",
                    final_seed_capability_url
                );
                let mut response = LoginResponse::success(
                    &session,
                    &config.server.base_url,
                    &inventory,
                    &final_seed_capability_url,
                );
                if !self.hg_service_urls.is_empty() {
                    response.service_urls = self.hg_service_urls.clone();
                }
                if let Ok(url) = std::env::var("OPENSIM_DESTINATION_GUIDE_URL") {
                    response.destination_guide_url = Some(url);
                }
                if let Ok(url) = std::env::var("OPENSIM_CURRENCY_URL") {
                    response.currency_base_uri = Some(url);
                }
                debug!("🔍 TRACE: Calling XmlRpcResponseGenerator::generate_login_response");
                let xml = XmlRpcResponseGenerator::generate_login_response(&response)?;
                info!(
                    "[LOGIN] XMLRPC response ready ({} bytes, {} folders, circuit={})",
                    xml.len(),
                    inventory.inventory_skeleton.len(),
                    session.circuit_code
                );

                if let Some(tracker) = &self.stage_tracker {
                    let session_id_str = session.session_id.to_string();
                    tracker
                        .create_session(&session_id_str, session.circuit_code)
                        .await;
                    tracker
                        .mark_passed(
                            &session_id_str,
                            crate::login_stage_tracker::LoginStage::XmlRpcSent,
                            Some(format!(
                                "circuit_code={} agent={}",
                                session.circuit_code, session.agent_id
                            )),
                        )
                        .await;
                }

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/xml")
                    .header("Content-Length", xml.len().to_string())
                    .body(Body::from(xml))?)
            }
            Err(e) => {
                warn!(
                    "Login failed for {} {}: {}",
                    login_request.first, login_request.last, e
                );
                let response = LoginResponse::failure("key", "Invalid username or password");
                let xml = XmlRpcResponseGenerator::generate_login_response(&response)?;

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/xml")
                    .body(Body::from(xml))?)
            }
        }
    }

    async fn authenticate_user(&self, request: &LoginRequest) -> Result<LoginSession> {
        debug!(
            "🔍 XMLRPC TRACE: Starting authenticate_user for {} {}",
            request.first, request.last
        );

        // Authenticate user with password from database
        debug!("🔍 XMLRPC TRACE: About to call user_database.authenticate_user_opensim");
        let user = self
            .user_database
            .authenticate_user_opensim(&request.first, &request.last, &request.passwd)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        debug!(
            "🔍 XMLRPC TRACE: Password verification successful for user {}",
            user.username
        );

        // Create session
        debug!("🔍 XMLRPC TRACE: About to create session");
        let config = self.config_manager.config();
        let session = self.session_manager.create_session(
            user.id,
            request.first.clone(),
            request.last.clone(),
            self.extract_sim_ip(),
            config.server.udp_port,
        )?;

        // CRITICAL FIX: Register XMLRPC session in CircuitCodeRegistry for UDP coordination
        if let Some(ref circuit_registry) = self.circuit_registry {
            let xmlrpc_session = crate::login_session::LoginSession {
                circuit_code: session.circuit_code,
                session_id: session.session_id.to_string(),
                agent_id: user.id.to_string(),
                first_name: request.first.clone(),
                last_name: request.last.clone(),
                created_at: std::time::Instant::now(),
                is_xmlrpc_session: true, // Mark as XMLRPC session
            };
            circuit_registry.register_login(xmlrpc_session).await;
            debug!("🔗 XMLRPC session registered in CircuitCodeRegistry for UDP coordination: circuit={}", session.circuit_code);
        } else {
            warn!("⚠️ CircuitCodeRegistry not available - XMLRPC->UDP coordination disabled");
        }

        debug!("🔍 XMLRPC TRACE: Session created successfully, returning session");
        Ok(session)
    }

    fn extract_sim_ip(&self) -> String {
        let config = self.config_manager.config();
        // Use configured sim_ip if available, otherwise extract from base_url
        if !config.server.sim_ip.is_empty() {
            return config.server.sim_ip.clone();
        }

        // Extract IP from base_url or use default
        if let Ok(url) = url::Url::parse(&config.server.base_url) {
            if let Some(host) = url.host_str() {
                return host.to_string();
            }
        }
        "127.0.0.1".to_string()
    }

    pub fn get_session_manager(&self) -> Arc<SessionManager> {
        self.session_manager.clone()
    }

    pub fn get_active_session_count(&self) -> usize {
        self.session_manager.get_active_session_count()
    }

    pub async fn cleanup_expired_sessions(&self) -> usize {
        self.session_manager.cleanup_expired_sessions()
    }

    /// Set the CapsManager for session coordination
    pub fn set_caps_manager(&mut self, caps_manager: Arc<CapsManager>) {
        self.caps_manager = Some(caps_manager);
    }

    pub fn get_inventory_cache_stats(&self) -> Result<crate::inventory::InventoryCacheStats> {
        let inv_service = self
            .inventory_service
            .lock()
            .map_err(|_| anyhow!("Failed to lock inventory service"))?;
        Ok(inv_service.cache_stats())
    }

    pub fn clear_inventory_cache(&self, user_id: Uuid) -> Result<()> {
        let mut inv_service = self
            .inventory_service
            .lock()
            .map_err(|_| anyhow!("Failed to lock inventory service"))?;
        inv_service.clear_cache(user_id);
        Ok(())
    }

    async fn query_inventory_from_database(
        &self,
        db_pool: &sqlx::PgPool,
        agent_id: Uuid,
    ) -> Result<crate::inventory::LoginInventoryResponse> {
        use sqlx::Row;

        debug!(
            "[INVENTORY] Phase 121: Loading inventory from database for agent {}",
            agent_id
        );

        let db_folders =
            crate::inventory::folders::lookup_database_folder_uuids(db_pool, agent_id).await;

        if let Some(ref folders) = db_folders {
            info!(
                "[INVENTORY] Found {} folder UUIDs in database",
                folders.len()
            );
        } else {
            debug!("[INVENTORY] No folders in database, will use generated UUIDs");
        }

        let mut inventory =
            crate::inventory::UserInventory::create_with_db_folders(agent_id, db_folders.as_ref());

        let all_db_folders = sqlx::query(
            "SELECT folderid, agentid, parentfolderid, foldername, type, version FROM inventoryfolders WHERE agentid = $1"
        )
        .bind(agent_id)
        .fetch_all(db_pool)
        .await
        .unwrap_or_default();

        let suitcase_id: Option<Uuid> = all_db_folders.iter().find_map(|row| {
            let ft: i32 = row.try_get::<i32, _>("type").unwrap_or(-1);
            if ft == 100 {
                row.try_get("folderid").ok()
            } else {
                None
            }
        });

        let mut added_folders = 0u32;
        for row in &all_db_folders {
            let folder_id: Uuid = match row.try_get("folderid") {
                Ok(id) => id,
                Err(_) => continue,
            };
            if inventory.folders.contains_key(&folder_id) {
                continue;
            }
            let parent_id: Uuid = row.try_get("parentfolderid").unwrap_or(Uuid::nil());
            let folder_type_i: i32 = row.try_get::<i32, _>("type").unwrap_or(-1);
            if folder_type_i == 100 {
                continue;
            }
            if let Some(sc_id) = suitcase_id {
                if parent_id == sc_id {
                    continue;
                }
            }
            let name: String = row.try_get("foldername").unwrap_or_default();
            let folder_type: i32 = row.try_get::<i32, _>("type").unwrap_or(-1);
            let version: i32 = row.try_get::<i32, _>("version").unwrap_or(1);

            let inv_folder_type = match folder_type {
                0 => crate::inventory::InventoryFolderType::Texture,
                1 => crate::inventory::InventoryFolderType::Sound,
                2 => crate::inventory::InventoryFolderType::CallingCard,
                3 => crate::inventory::InventoryFolderType::Landmark,
                5 => crate::inventory::InventoryFolderType::Clothing,
                6 => crate::inventory::InventoryFolderType::Object,
                7 => crate::inventory::InventoryFolderType::Notecard,
                8 => crate::inventory::InventoryFolderType::Root,
                10 => crate::inventory::InventoryFolderType::LSLText,
                13 => crate::inventory::InventoryFolderType::BodyPart,
                14 => crate::inventory::InventoryFolderType::Trash,
                15 => crate::inventory::InventoryFolderType::Snapshot,
                16 => crate::inventory::InventoryFolderType::LostAndFound,
                20 => crate::inventory::InventoryFolderType::Animation,
                21 => crate::inventory::InventoryFolderType::Gesture,
                23 => crate::inventory::InventoryFolderType::Favorites,
                46 => crate::inventory::InventoryFolderType::CurrentOutfit,
                47 => crate::inventory::InventoryFolderType::Outfit,
                48 => crate::inventory::InventoryFolderType::MyOutfits,
                49 => crate::inventory::InventoryFolderType::Mesh,
                50 => crate::inventory::InventoryFolderType::Inbox,
                51 => crate::inventory::InventoryFolderType::Outbox,
                53 => crate::inventory::InventoryFolderType::MarketplaceListings,
                56 => crate::inventory::InventoryFolderType::Settings,
                57 => crate::inventory::InventoryFolderType::Material,
                _ => crate::inventory::InventoryFolderType::Object,
            };

            let folder = crate::inventory::InventoryFolder {
                id: folder_id,
                parent_id: if parent_id.is_nil() {
                    None
                } else {
                    Some(parent_id)
                },
                owner_id: agent_id,
                name,
                folder_type: inv_folder_type,
                version: version as u32,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            inventory.add_folder(folder);
            added_folders += 1;
        }
        if added_folders > 0 {
            info!(
                "[INVENTORY] Added {} user-created folders to login skeleton",
                added_folders
            );
        }

        let cof_id = inventory
            .folders
            .values()
            .find(|f| f.folder_type == crate::inventory::InventoryFolderType::CurrentOutfit)
            .map(|f| f.id);

        if let Some(cof_id) = cof_id {
            let cof_items = sqlx::query(
                r#"SELECT inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                    inventoryname, inventorydescription, creationdate, creatorid,
                    groupid, groupowned, saleprice, saletype, flags,
                    inventorybasepermissions, inventorycurrentpermissions,
                    inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions
                FROM inventoryitems WHERE parentfolderid = $1"#
            )
            .bind(cof_id)
            .fetch_all(db_pool)
            .await
            .unwrap_or_default();

            if !cof_items.is_empty() {
                inventory.items.retain(|_, item| item.folder_id != cof_id);
                for row in &cof_items {
                    if let Ok(item) = Self::inventory_item_from_row(row, agent_id) {
                        inventory.add_item(item);
                    }
                }
                info!(
                    "[INVENTORY] Loaded {} COF items from database (replaced hardcoded defaults)",
                    cof_items.len()
                );
            }
        }

        let bodypart_id = inventory
            .folders
            .values()
            .find(|f| f.folder_type == crate::inventory::InventoryFolderType::BodyPart)
            .map(|f| f.id);
        let clothing_id = inventory
            .folders
            .values()
            .find(|f| f.folder_type == crate::inventory::InventoryFolderType::Clothing)
            .map(|f| f.id);

        for folder_id in [bodypart_id, clothing_id].iter().flatten() {
            let db_items = sqlx::query(
                r#"SELECT inventoryid, avatarid, parentfolderid, assetid, assettype, invtype,
                    inventoryname, inventorydescription, creationdate, creatorid,
                    groupid, groupowned, saleprice, saletype, flags,
                    inventorybasepermissions, inventorycurrentpermissions,
                    inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions
                FROM inventoryitems WHERE parentfolderid = $1"#
            )
            .bind(folder_id)
            .fetch_all(db_pool)
            .await
            .unwrap_or_default();

            if !db_items.is_empty() {
                inventory
                    .items
                    .retain(|_, item| item.folder_id != *folder_id);
                for row in &db_items {
                    if let Ok(item) = Self::inventory_item_from_row(row, agent_id) {
                        inventory.add_item(item);
                    }
                }
                info!(
                    "[INVENTORY] Loaded {} items from {} folder",
                    db_items.len(),
                    if *folder_id == bodypart_id.unwrap_or(Uuid::nil()) {
                        "Body Parts"
                    } else {
                        "Clothing"
                    }
                );
            }
        }

        info!(
            "[INVENTORY] Final login inventory: {} folders, {} items",
            inventory.folders.len(),
            inventory.items.len()
        );

        Ok(inventory.to_login_response())
    }

    fn inventory_item_from_row(
        row: &sqlx::postgres::PgRow,
        agent_id: Uuid,
    ) -> Result<crate::inventory::InventoryItem> {
        use sqlx::Row;

        let id: Uuid = row.try_get("inventoryid")?;
        let asset_id: Uuid = row.try_get("assetid").unwrap_or(Uuid::nil());
        let folder_id: Uuid = row.try_get("parentfolderid").unwrap_or(Uuid::nil());
        let name: String = row.try_get("inventoryname").unwrap_or_default();
        let description: String = row.try_get("inventorydescription").unwrap_or_default();
        let asset_type: i32 = row.try_get("assettype").unwrap_or(0);
        let inv_type: i32 = row.try_get("invtype").unwrap_or(0);
        let creator_id: String = row
            .try_get("creatorid")
            .unwrap_or_else(|_| agent_id.to_string());
        let flags: i32 = row.try_get("flags").unwrap_or(0);
        let base_perms: i32 = row
            .try_get("inventorybasepermissions")
            .unwrap_or(0x7FFFFFFF);
        let current_perms: i32 = row
            .try_get("inventorycurrentpermissions")
            .unwrap_or(0x7FFFFFFF);
        let everyone_perms: i32 = row.try_get("inventoryeveryonepermissions").unwrap_or(0);
        let group_perms: i32 = row.try_get("inventorygrouppermissions").unwrap_or(0);
        let next_perms: i32 = row
            .try_get("inventorynextpermissions")
            .unwrap_or(0x7FFFFFFF);
        let sale_price: i32 = row.try_get("saleprice").unwrap_or(0);
        let sale_type: i32 = row.try_get("saletype").unwrap_or(0);
        let group_id: Option<Uuid> = row.try_get("groupid").ok();
        let group_owned: i32 = row.try_get("groupowned").unwrap_or(0);

        let inv_asset_type = match asset_type {
            0 => crate::inventory::InventoryAssetType::Texture,
            1 => crate::inventory::InventoryAssetType::Sound,
            2 => crate::inventory::InventoryAssetType::CallingCard,
            3 => crate::inventory::InventoryAssetType::Landmark,
            5 => crate::inventory::InventoryAssetType::Clothing,
            6 => crate::inventory::InventoryAssetType::Object,
            7 => crate::inventory::InventoryAssetType::Notecard,
            10 => crate::inventory::InventoryAssetType::LSLText,
            13 => crate::inventory::InventoryAssetType::Bodypart,
            20 => crate::inventory::InventoryAssetType::Animation,
            21 => crate::inventory::InventoryAssetType::Gesture,
            24 => crate::inventory::InventoryAssetType::Link,
            49 => crate::inventory::InventoryAssetType::Mesh,
            _ => crate::inventory::InventoryAssetType::Object,
        };

        let inv_inventory_type = match inv_type {
            0 => crate::inventory::InventoryAssetType::Texture,
            1 => crate::inventory::InventoryAssetType::Sound,
            2 => crate::inventory::InventoryAssetType::CallingCard,
            3 => crate::inventory::InventoryAssetType::Landmark,
            5 => crate::inventory::InventoryAssetType::Clothing,
            6 => crate::inventory::InventoryAssetType::Object,
            7 => crate::inventory::InventoryAssetType::Notecard,
            10 => crate::inventory::InventoryAssetType::LSLText,
            13 => crate::inventory::InventoryAssetType::Bodypart,
            18 => crate::inventory::InventoryAssetType::Wearable,
            20 => crate::inventory::InventoryAssetType::Animation,
            21 => crate::inventory::InventoryAssetType::Gesture,
            24 => crate::inventory::InventoryAssetType::Link,
            49 => crate::inventory::InventoryAssetType::Mesh,
            _ => crate::inventory::InventoryAssetType::Object,
        };

        let creator_uuid = Uuid::parse_str(&creator_id).unwrap_or(agent_id);
        let now = chrono::Utc::now();

        let mut item = crate::inventory::InventoryItem::new_with_id(
            id,
            asset_id,
            folder_id,
            agent_id,
            creator_uuid,
            name,
            description,
            inv_asset_type,
            flags as u32,
        );
        item.inventory_type = inv_inventory_type;
        item.permissions = crate::inventory::InventoryPermissions {
            base_perms: base_perms as u32,
            owner_perms: current_perms as u32,
            group_perms: group_perms as u32,
            everyone_perms: everyone_perms as u32,
            next_perms: next_perms as u32,
        };
        item.sale_price = sale_price;
        item.sale_type = sale_type as u8;
        item.group_id = group_id;
        item.group_owned = group_owned != 0;

        Ok(item)
    }
}

// Grid info handler for viewer compatibility
pub async fn handle_grid_info() -> Response<Body> {
    let port = std::env::var("OPENSIM_LOGIN_PORT").unwrap_or_else(|_| "9000".to_string());
    let home_uri =
        std::env::var("OPENSIM_HOME_URI").unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));
    let gatekeeper_uri = std::env::var("OPENSIM_GATEKEEPER_URI")
        .unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));
    let grid_name =
        std::env::var("OPENSIM_GRID_NAME").unwrap_or_else(|_| "OpenSim Next Grid".to_string());
    let grid_nick =
        std::env::var("OPENSIM_GRID_NICK").unwrap_or_else(|_| "opensim-next".to_string());

    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<gridinfo>
  <gridname>{grid_name}</gridname>
  <gridnick>{grid_nick}</gridnick>
  <login>{home_uri}/</login>
  <welcome>{home_uri}/welcome</welcome>
  <economy>{home_uri}/</economy>
  <about>{home_uri}/welcome</about>
  <register>{home_uri}/</register>
  <help>{home_uri}/</help>
  <password>{home_uri}/</password>
  <gatekeeper>{gatekeeper_uri}/</gatekeeper>
  <uas>{gatekeeper_uri}/</uas>
  <platform>OpenSim</platform>
  <version>2.1.0</version>
  <splashimage>{home_uri}/splash.png</splashimage>
</gridinfo>"#
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml")
        .body(Body::from(xml))
        .unwrap()
}

pub async fn handle_grid_info_json() -> Response<Body> {
    let port = std::env::var("OPENSIM_LOGIN_PORT").unwrap_or_else(|_| "9000".to_string());
    let home_uri =
        std::env::var("OPENSIM_HOME_URI").unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));
    let gatekeeper_uri = std::env::var("OPENSIM_GATEKEEPER_URI")
        .unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));
    let grid_name =
        std::env::var("OPENSIM_GRID_NAME").unwrap_or_else(|_| "OpenSim Next Grid".to_string());
    let grid_nick =
        std::env::var("OPENSIM_GRID_NICK").unwrap_or_else(|_| "opensim-next".to_string());

    let json = format!(
        r#"{{"gridname":"{}","gridnick":"{}","login":"{}/","welcome":"{}/welcome","economy":"{}/","about":"{}/welcome","register":"{}/","help":"{}/","password":"{}/","gatekeeper":"{}/","uas":"{}/","platform":"OpenSim","version":"2.1.0"}}"#,
        grid_name.replace('"', "\\\""),
        grid_nick.replace('"', "\\\""),
        home_uri,
        home_uri,
        home_uri,
        home_uri,
        home_uri,
        home_uri,
        home_uri,
        gatekeeper_uri,
        gatekeeper_uri
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json))
        .unwrap()
}

// Axum handler function for the login endpoint
pub async fn handle_login_post(
    State(login_service): State<Arc<LoginService>>,
    headers: HeaderMap,
    body: String,
) -> Result<Response<Body>, StatusCode> {
    if let Some(ref tracker) = login_service.readiness_tracker {
        if !tracker.is_login_ready() {
            let pending: Vec<&str> = tracker
                .status_breakdown()
                .iter()
                .filter(|(_, ready)| !ready)
                .map(|(name, _)| *name)
                .collect();
            warn!(
                "[LOGIN] Rejected login — server not ready. Pending: {:?}",
                pending
            );
            let xml = format!(
                r#"<?xml version="1.0"?>
<methodResponse><params><param><value><struct>
  <member><name>login</name><value><string>false</string></value></member>
  <member><name>reason</name><value><string>presence</string></value></member>
  <member><name>message</name><value><string>Region is starting up ({} services pending). Please wait 30 seconds and try again.</string></value></member>
</struct></value></param></params></methodResponse>"#,
                pending.len()
            );
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/xml")
                .body(Body::from(xml))
                .unwrap());
        }
    }

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let is_valid_content_type = content_type.is_empty()
        || content_type.contains("text/xml")
        || content_type.contains("application/xml")
        || content_type.contains("application/x-www-form-urlencoded")
        || body.trim().starts_with("<?xml")
        || body.trim().starts_with("<methodCall");

    if !is_valid_content_type {
        warn!(
            "Unexpected content type for login request: {} (body starts with: {})",
            content_type,
            body.chars().take(50).collect::<String>()
        );
    }

    if let Some(method_name) = extract_xmlrpc_method_name(&body) {
        let gk_methods = ["link_region", "get_region"];
        if gk_methods.contains(&method_name.as_str()) {
            if let Some(ref gk) = login_service.gatekeeper_service {
                info!(
                    "[HG GK] XmlRpc method '{}' received at root path — dispatching to gatekeeper",
                    method_name
                );
                let xml = handle_gatekeeper_xmlrpc_at_root(&body, &method_name, gk).await;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/xml")
                    .body(Body::from(xml))
                    .unwrap());
            } else {
                warn!(
                    "[HG GK] XmlRpc method '{}' received but no gatekeeper service configured",
                    method_name
                );
            }
        }

        let uas_methods = [
            "verify_agent",
            "verify_client",
            "get_home_region",
            "get_server_urls",
            "logout_agent",
            "get_uui",
            "get_uuid",
            "status_notification",
            "get_online_friends",
            "agent_is_coming_home",
            "get_user_info",
            "avatar_properties_request",
        ];
        if uas_methods.contains(&method_name.as_str()) {
            if let Some(ref uas) = login_service.uas_service {
                info!("[HG UAS] XmlRpc method '{}' received at root path — dispatching to UAS handler", method_name);
                let xml = handle_uas_xmlrpc_at_root(&body, &method_name, uas).await;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/xml")
                    .body(Body::from(xml))
                    .unwrap());
            } else {
                warn!(
                    "[HG UAS] XmlRpc method '{}' received but no UAS service configured",
                    method_name
                );
            }
        }

        if method_name == "get_grid_info" {
            info!("[HG] XmlRpc 'get_grid_info' received at root path — returning grid info");
            let port = std::env::var("OPENSIM_LOGIN_PORT").unwrap_or_else(|_| "9000".to_string());
            let home_uri = std::env::var("OPENSIM_HOME_URI")
                .unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));
            let gatekeeper_uri = std::env::var("OPENSIM_GATEKEEPER_URI")
                .unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));
            let grid_name = std::env::var("OPENSIM_GRID_NAME")
                .unwrap_or_else(|_| "OpenSim Next Grid".to_string());
            let grid_nick =
                std::env::var("OPENSIM_GRID_NICK").unwrap_or_else(|_| "opensim-next".to_string());

            let mut grid_info = std::collections::HashMap::new();
            grid_info.insert(
                "gridname".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(grid_name),
            );
            grid_info.insert(
                "gridnick".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(grid_nick),
            );
            grid_info.insert(
                "login".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(format!("{}/", home_uri)),
            );
            grid_info.insert(
                "welcome".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(format!(
                    "{}/welcome",
                    home_uri
                )),
            );
            grid_info.insert(
                "economy".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(format!("{}/", home_uri)),
            );
            grid_info.insert(
                "about".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(format!(
                    "{}/welcome",
                    home_uri
                )),
            );
            grid_info.insert(
                "register".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(format!("{}/", home_uri)),
            );
            grid_info.insert(
                "help".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(format!("{}/", home_uri)),
            );
            grid_info.insert(
                "password".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(format!("{}/", home_uri)),
            );
            grid_info.insert(
                "gatekeeper".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(format!(
                    "{}/",
                    gatekeeper_uri
                )),
            );
            grid_info.insert(
                "uas".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String(format!(
                    "{}/",
                    gatekeeper_uri
                )),
            );
            grid_info.insert(
                "platform".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String("OpenSim".to_string()),
            );
            grid_info.insert(
                "version".to_string(),
                crate::services::robust::xmlrpc::XmlRpcValue::String("2.1.0".to_string()),
            );

            let xml = crate::services::robust::xmlrpc::build_xmlrpc_response(
                &crate::services::robust::xmlrpc::XmlRpcValue::Struct(grid_info),
            );
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/xml")
                .body(Body::from(xml))
                .unwrap());
        }

        if method_name != "login_to_simulator" {
            warn!(
                "[HG] Unrecognized XMLRPC method '{}' at root path (body_len={}, body_preview={})",
                method_name,
                body.len(),
                &body[..std::cmp::min(body.len(), 500)]
            );
        }
    }

    match login_service.handle_login_request(body).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Login service error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn extract_xmlrpc_method_name(body: &str) -> Option<String> {
    let start = body.find("<methodName>")?;
    let after_tag = &body[start + 12..];
    let end = after_tag.find("</methodName>")?;
    Some(after_tag[..end].trim().to_string())
}

async fn handle_gatekeeper_xmlrpc_at_root(
    body: &str,
    method: &str,
    gk: &Arc<dyn crate::services::traits::GatekeeperServiceTrait>,
) -> String {
    use crate::services::robust::xmlrpc::{
        build_xmlrpc_fault, build_xmlrpc_response, parse_xmlrpc_call, XmlRpcValue,
    };

    let (_method, params) = match parse_xmlrpc_call(body) {
        Ok(result) => result,
        Err(e) => {
            warn!("[HG GK] Failed to parse XmlRpc: {}", e);
            return build_xmlrpc_fault(2, &format!("Invalid XmlRpc: {}", e));
        }
    };

    let param = params
        .first()
        .cloned()
        .unwrap_or(XmlRpcValue::Struct(std::collections::HashMap::new()));

    match method {
        "link_region" => {
            let region_name = param.get_str("region_name").unwrap_or("");
            match gk.link_region(region_name).await {
                Ok(Some(info)) => {
                    info!(
                        "[HG GK] link_region '{}' -> {} (handle={})",
                        region_name, info.region_id, info.region_handle
                    );
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("True".to_string()),
                    );
                    result.insert(
                        "uuid".to_string(),
                        XmlRpcValue::String(info.region_id.to_string()),
                    );
                    result.insert(
                        "handle".to_string(),
                        XmlRpcValue::String(info.region_handle.to_string()),
                    );
                    result.insert(
                        "size_x".to_string(),
                        XmlRpcValue::String(info.size_x.to_string()),
                    );
                    result.insert(
                        "size_y".to_string(),
                        XmlRpcValue::String(info.size_y.to_string()),
                    );
                    result.insert(
                        "region_image".to_string(),
                        XmlRpcValue::String(info.image_url.clone()),
                    );
                    result.insert(
                        "external_name".to_string(),
                        XmlRpcValue::String(info.external_name.clone()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    warn!("[HG GK] link_region '{}' -> not found", region_name);
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("False".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("Internal error: {}", e)),
            }
        }
        "get_region" => {
            let region_id = param
                .get_str("region_uuid")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let agent_id = param
                .get_str("agent_id")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let agent_home = param.get_str("agent_home_uri").unwrap_or("");

            match gk
                .get_hyperlinkregion(region_id, agent_id, agent_home)
                .await
            {
                Ok(Some(info)) => {
                    info!(
                        "[HG GK] get_region {} -> {} ({})",
                        region_id, info.region_name, info.server_uri
                    );
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("True".to_string()),
                    );
                    result.insert(
                        "uuid".to_string(),
                        XmlRpcValue::String(info.region_id.to_string()),
                    );
                    result.insert(
                        "x".to_string(),
                        XmlRpcValue::String(info.region_loc_x.to_string()),
                    );
                    result.insert(
                        "y".to_string(),
                        XmlRpcValue::String(info.region_loc_y.to_string()),
                    );
                    result.insert(
                        "size_x".to_string(),
                        XmlRpcValue::String(info.size_x.to_string()),
                    );
                    result.insert(
                        "size_y".to_string(),
                        XmlRpcValue::String(info.size_y.to_string()),
                    );
                    result.insert(
                        "region_name".to_string(),
                        XmlRpcValue::String(info.region_name.clone()),
                    );
                    result.insert(
                        "hostname".to_string(),
                        XmlRpcValue::String(info.hostname.clone()),
                    );
                    result.insert(
                        "http_port".to_string(),
                        XmlRpcValue::String(info.http_port.to_string()),
                    );
                    result.insert(
                        "internal_port".to_string(),
                        XmlRpcValue::String(info.internal_port.to_string()),
                    );
                    result.insert(
                        "server_uri".to_string(),
                        XmlRpcValue::String(info.server_uri.clone()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    warn!("[HG GK] get_region {} -> not found", region_id);
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("False".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("Internal error: {}", e)),
            }
        }
        _ => build_xmlrpc_fault(4, &format!("Unknown gatekeeper method: {}", method)),
    }
}

async fn handle_uas_xmlrpc_at_root(
    body: &str,
    method: &str,
    uas: &Arc<dyn crate::services::traits::UserAgentServiceTrait>,
) -> String {
    use crate::services::robust::xmlrpc::{
        build_xmlrpc_fault, build_xmlrpc_response, parse_xmlrpc_call, XmlRpcValue,
    };

    let (_method, params) = match parse_xmlrpc_call(body) {
        Ok(result) => result,
        Err(e) => {
            warn!("[HG UAS] Failed to parse XmlRpc: {}", e);
            return build_xmlrpc_fault(2, &format!("Invalid XmlRpc: {}", e));
        }
    };

    let param = params
        .first()
        .cloned()
        .unwrap_or(XmlRpcValue::Struct(std::collections::HashMap::new()));

    match method {
        "verify_agent" => {
            let session_id = param
                .get_str("sessionID")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let token = param.get_str("token").unwrap_or("");
            info!(
                "[HG UAS] verify_agent: session={}, token={}",
                session_id, token
            );

            match uas.verify_agent(session_id, token).await {
                Ok(valid) => {
                    info!("[HG UAS] verify_agent result: {}", valid);
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String(if valid { "True" } else { "False" }.to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => {
                    warn!("[HG UAS] verify_agent error: {}", e);
                    build_xmlrpc_fault(3, &format!("verify_agent error: {}", e))
                }
            }
        }

        "verify_client" => {
            let session_id = param
                .get_str("sessionID")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let token = param.get_str("token").unwrap_or("");
            info!("[HG UAS] verify_client: session={}", session_id);

            match uas.verify_client(session_id, token).await {
                Ok(valid) => {
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String(if valid { "True" } else { "False" }.to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("verify_client error: {}", e)),
            }
        }

        "get_home_region" => {
            let user_id = param
                .get_str("userID")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();

            match uas.get_home_region(user_id).await {
                Ok(Some((info, position, look_at))) => {
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("true".to_string()),
                    );
                    result.insert(
                        "uuid".to_string(),
                        XmlRpcValue::String(info.region_id.to_string()),
                    );
                    result.insert(
                        "handle".to_string(),
                        XmlRpcValue::String(info.region_handle.to_string()),
                    );
                    result.insert(
                        "region_name".to_string(),
                        XmlRpcValue::String(info.region_name),
                    );
                    result.insert(
                        "server_uri".to_string(),
                        XmlRpcValue::String(info.server_uri),
                    );
                    result.insert(
                        "position".to_string(),
                        XmlRpcValue::String(format!(
                            "<{},{},{}>",
                            position[0], position[1], position[2]
                        )),
                    );
                    result.insert(
                        "lookAt".to_string(),
                        XmlRpcValue::String(format!(
                            "<{},{},{}>",
                            look_at[0], look_at[1], look_at[2]
                        )),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("false".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("get_home_region error: {}", e)),
            }
        }

        "get_server_urls" => {
            let user_id = param
                .get_str("userID")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();

            match uas.get_server_urls(user_id).await {
                Ok(urls) => {
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("true".to_string()),
                    );
                    for (k, v) in urls {
                        result.insert(k, XmlRpcValue::String(v));
                    }
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("get_server_urls error: {}", e)),
            }
        }

        "logout_agent" => {
            let user_id = param
                .get_str("userID")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let session_id = param
                .get_str("sessionID")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();

            match uas.logout_agent(user_id, session_id).await {
                Ok(()) => {
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("true".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("logout_agent error: {}", e)),
            }
        }

        "agent_is_coming_home" => {
            let session_id = param
                .get_str("sessionID")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let external_name = param.get_str("externalName").unwrap_or("");

            match uas.is_agent_coming_home(session_id, external_name).await {
                Ok(coming_home) => {
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String(if coming_home { "True" } else { "False" }.to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("agent_is_coming_home error: {}", e)),
            }
        }

        "get_user_info" => {
            let user_id = param
                .get_str("userID")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();
            info!("[HG UAS] get_user_info: user_id={}", user_id);

            match uas.get_user_info(user_id).await {
                Ok(info) => {
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("success".to_string()),
                    );
                    for (k, v) in info {
                        result.insert(k, XmlRpcValue::String(v));
                    }
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => {
                    warn!("[HG UAS] get_user_info error: {}", e);
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("failure".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
            }
        }

        "avatar_properties_request" => {
            let avatar_id = param
                .get_str("avatar_id")
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default();
            info!(
                "[HG UAS] avatar_properties_request: avatar_id={}",
                avatar_id
            );

            match uas.get_user_info(avatar_id).await {
                Ok(info) => {
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("success".to_string()),
                    );
                    for (k, v) in info {
                        result.insert(k, XmlRpcValue::String(v));
                    }
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(_) => {
                    let mut result = std::collections::HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("failure".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
            }
        }

        _ => {
            warn!("[HG UAS] Unhandled UAS method at root: {}", method);
            build_xmlrpc_fault(4, &format!("Unknown method: {}", method))
        }
    }
}

// Handler for login page (for testing)
pub async fn handle_login_page() -> Html<&'static str> {
    Html(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>OpenSim Next Login Test</title>
</head>
<body>
    <h1>OpenSim Next Login Service</h1>
    <p>This is the XMLRPC login endpoint for Second Life viewers.</p>
    <p>To test login, configure your viewer to use this server's login URI.</p>
    <h2>Server Information</h2>
    <ul>
        <li>XMLRPC Login: POST to /login</li>
        <li>UDP Circuit: Port 9000</li>
        <li>Status: Active</li>
    </ul>
</body>
</html>
"#,
    )
}

pub async fn handle_splash_page() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSim Next - Revolutionary Virtual World Platform</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            background: linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #334155 100%);
            color: #f8fafc;
            overflow: hidden;
        }
        .container {
            text-align: center;
            padding: 2rem;
            max-width: 800px;
        }
        .logo-container {
            width: 140px;
            height: 140px;
            margin: 0 auto 2rem;
            background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            box-shadow: 0 0 60px rgba(59, 130, 246, 0.4);
            animation: pulse 3s ease-in-out infinite;
        }
        @keyframes pulse {
            0%, 100% { transform: scale(1); box-shadow: 0 0 60px rgba(59, 130, 246, 0.4); }
            50% { transform: scale(1.05); box-shadow: 0 0 80px rgba(59, 130, 246, 0.6); }
        }
        .logo-icon {
            font-size: 64px;
            color: white;
        }
        h1 {
            font-size: 3rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
            background: linear-gradient(90deg, #60a5fa, #a78bfa);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        .subtitle {
            font-size: 1.25rem;
            color: #94a3b8;
            margin-bottom: 3rem;
            font-weight: 300;
        }
        .features {
            display: flex;
            flex-wrap: wrap;
            justify-content: center;
            gap: 2rem;
            margin-top: 2rem;
        }
        .feature {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 0.5rem;
            min-width: 120px;
        }
        .feature-icon {
            width: 48px;
            height: 48px;
            border-radius: 12px;
            background: rgba(59, 130, 246, 0.2);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 24px;
        }
        .feature-text {
            font-size: 0.875rem;
            color: #cbd5e1;
            text-align: center;
        }
        .version {
            position: fixed;
            bottom: 1rem;
            color: #64748b;
            font-size: 0.75rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo-container">
            <span class="logo-icon">&#127760;</span>
        </div>
        <h1>OpenSim Next</h1>
        <p class="subtitle">Revolutionary Virtual World Platform</p>
        <div class="features">
            <div class="feature">
                <div class="feature-icon">&#9889;</div>
                <span class="feature-text">High<br>Performance</span>
            </div>
            <div class="feature">
                <div class="feature-icon">&#128274;</div>
                <span class="feature-text">Zero Trust<br>Security</span>
            </div>
            <div class="feature">
                <div class="feature-icon">&#128202;</div>
                <span class="feature-text">Real-time<br>Analytics</span>
            </div>
            <div class="feature">
                <div class="feature-icon">&#128187;</div>
                <span class="feature-text">Multi<br>Platform</span>
            </div>
        </div>
    </div>
    <div class="version">OpenSim Next v2.1.0 - Rust/Zig Architecture</div>
</body>
</html>"#,
    )
}

pub async fn handle_splash_image() -> Response<Body> {
    let png_data: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x80, 0x08, 0x02, 0x00, 0x00, 0x00, 0x4C,
        0x5C, 0xF6, 0x9C, 0x00, 0x00, 0x02, 0x00, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0xED, 0xD2,
        0x31, 0x0E, 0x00, 0x20, 0x08, 0x03, 0xD0, 0xF2, 0xFF, 0x3F, 0x8D, 0x0B, 0x0B, 0xA2, 0x24,
        0x4E, 0x12, 0x77, 0x9B, 0x60, 0x90, 0xB6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0xF8, 0x77, 0x01, 0xE9, 0x09, 0x7F, 0x31, 0x3B, 0x6C, 0x74, 0xBC,
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=86400")
        .body(Body::from(png_data.to_vec()))
        .unwrap()
}

fn extract_xmlrpc_string(body: &str, name: &str) -> Option<String> {
    let search = format!("<name>{}</name>", name);
    let pos = body.find(&search)?;
    let after = &body[pos + search.len()..];
    let val_start = after.find("<string>")? + 8;
    let val_end = after.find("</string>")?;
    Some(after[val_start..val_end].to_string())
}

fn extract_xmlrpc_int(body: &str, name: &str) -> Option<i64> {
    let search = format!("<name>{}</name>", name);
    let pos = body.find(&search)?;
    let after = &body[pos + search.len()..];
    if let Some(start) = after.find("<int>") {
        let val_start = start + 5;
        let val_end = after.find("</int>")?;
        after[val_start..val_end].parse().ok()
    } else if let Some(start) = after.find("<i4>") {
        let val_start = start + 4;
        let val_end = after.find("</i4>")?;
        after[val_start..val_end].parse().ok()
    } else {
        None
    }
}

async fn get_user_balance(db: &DatabaseManager, agent_id: &Uuid) -> i64 {
    use sqlx::Row;
    match db.legacy_pool() {
        Ok(pool) => {
            match sqlx::query(
                "SELECT COALESCE(balance, 0) as balance FROM currency_balances WHERE user_id = $1 AND currency_code = 'LS'"
            )
            .bind(agent_id)
            .fetch_optional(pool)
            .await
            {
                Ok(Some(row)) => row.try_get::<i64, _>("balance").unwrap_or(0),
                Ok(None) => {
                    if let Err(e) = sqlx::query(
                        "INSERT INTO currency_balances (user_id, currency_code, balance, reserved, available, version) VALUES ($1, 'LS', 1000, 0, 1000, 1) ON CONFLICT DO NOTHING"
                    )
                    .bind(agent_id)
                    .execute(pool)
                    .await
                    {
                        warn!("[CURRENCY] Failed to create starter balance: {}", e);
                    }
                    1000
                }
                Err(e) => {
                    warn!("[CURRENCY] Balance query failed: {}", e);
                    0
                }
            }
        }
        Err(e) => {
            warn!("[CURRENCY] No database pool: {}", e);
            0
        }
    }
}

pub async fn handle_currency_php(
    State(state): State<CurrencyState>,
    body: String,
) -> Response<Body> {
    info!("[CURRENCY] currency.php request ({} bytes)", body.len());

    let method = if body.contains("getCurrencyQuote") {
        "getCurrencyQuote"
    } else if body.contains("buyCurrency") {
        "buyCurrency"
    } else if body.contains("preflightBuyLandPrep") {
        "preflightBuyLandPrep"
    } else if body.contains("buyLandPrep") {
        "buyLandPrep"
    } else {
        "unknown"
    };

    info!("[CURRENCY] Method: {}", method);

    let agent_id = extract_xmlrpc_string(&body, "agentId").and_then(|s| Uuid::parse_str(&s).ok());

    let balance = if let (Some(ref db), Some(ref aid)) = (&state.database_manager, &agent_id) {
        get_user_balance(db, aid).await
    } else {
        0
    };

    let xml = match method {
        "getCurrencyQuote" => {
            let currency_buy = extract_xmlrpc_int(&body, "currencyBuy").unwrap_or(0);
            info!(
                "[CURRENCY] getCurrencyQuote: agent={:?} balance={} currencyBuy={}",
                agent_id, balance, currency_buy
            );
            format!(
                r#"<?xml version="1.0"?>
<methodResponse>
  <params>
    <param><value><struct>
      <member><name>success</name><value><boolean>1</boolean></value></member>
      <member><name>currency</name><value><struct>
        <member><name>estimatedCost</name><value><int>0</int></value></member>
        <member><name>currencyBuy</name><value><int>{}</int></value></member>
      </struct></value></member>
      <member><name>confirm</name><value><string></string></value></member>
    </struct></value></param>
  </params>
</methodResponse>"#,
                currency_buy
            )
        }
        "buyCurrency" => {
            let currency_buy = extract_xmlrpc_int(&body, "currencyBuy").unwrap_or(0);
            info!(
                "[CURRENCY] buyCurrency: agent={:?} amount={}",
                agent_id, currency_buy
            );

            if let (Some(ref db), Some(ref aid)) = (&state.database_manager, &agent_id) {
                if currency_buy > 0 {
                    if let Ok(pool) = db.legacy_pool() {
                        let _ = sqlx::query(
                            "UPDATE currency_balances SET balance = balance + $1, available = available + $1, version = version + 1, updated_at = now() WHERE user_id = $2 AND currency_code = 'LS'"
                        )
                        .bind(currency_buy)
                        .bind(aid)
                        .execute(pool)
                        .await;
                        info!("[CURRENCY] Granted {} L$ to {}", currency_buy, aid);
                    }
                }
            }

            r#"<?xml version="1.0"?>
<methodResponse>
  <params>
    <param><value><struct>
      <member><name>success</name><value><boolean>1</boolean></value></member>
    </struct></value></param>
  </params>
</methodResponse>"#
                .to_string()
        }
        "preflightBuyLandPrep" => {
            info!(
                "[CURRENCY] preflightBuyLandPrep: agent={:?} balance={}",
                agent_id, balance
            );
            format!(
                r#"<?xml version="1.0"?>
<methodResponse>
  <params>
    <param><value><struct>
      <member><name>success</name><value><boolean>1</boolean></value></member>
      <member><name>currency</name><value><struct>
        <member><name>estimatedCost</name><value><int>0</int></value></member>
      </struct></value></member>
      <member><name>membership</name><value><struct>
        <member><name>upgrade</name><value><boolean>0</boolean></value></member>
        <member><name>action</name><value><string></string></value></member>
        <member><name>levels</name><value><array><data></data></array></value></member>
      </struct></value></member>
      <member><name>landUse</name><value><struct>
        <member><name>upgrade</name><value><boolean>0</boolean></value></member>
        <member><name>action</name><value><string></string></value></member>
      </struct></value></member>
      <member><name>confirm</name><value><string></string></value></member>
    </struct></value></param>
  </params>
</methodResponse>"#
            )
        }
        _ => r#"<?xml version="1.0"?>
<methodResponse>
  <params>
    <param><value><struct>
      <member><name>success</name><value><boolean>0</boolean></value></member>
      <member><name>errorMessage</name><value><string>Unknown method</string></value></member>
    </struct></value></param>
  </params>
</methodResponse>"#
            .to_string(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/xml")
        .body(Body::from(xml))
        .unwrap()
}

pub async fn handle_landtool_php(state: State<CurrencyState>, body: String) -> Response<Body> {
    info!("[LANDTOOL] landtool.php request ({} bytes)", body.len());
    handle_currency_php(state, body).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseManager;

    #[tokio::test]
    #[ignore]
    async fn test_login_service_creation() {
        use crate::services::AvatarService;

        let config_manager = LoginConfigManager::new();
        let session_manager = Arc::new(SessionManager::new());

        let db_manager = Arc::new(
            DatabaseManager::new("postgresql://opensim:password123@localhost/opensim_pg")
                .await
                .unwrap(),
        );
        let user_database = db_manager.user_accounts();
        let avatar_service = Arc::new(AvatarService::new(db_manager.clone()));

        let login_service = LoginService::new(
            config_manager,
            session_manager,
            user_database,
            avatar_service,
        );

        assert_eq!(login_service.get_active_session_count(), 0);
    }

    #[tokio::test]
    async fn test_xml_parsing() {
        let xml = r#"<?xml version="1.0"?>
<methodCall>
  <methodName>login_to_simulator</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member>
            <name>first</name>
            <value><string>Test</string></value>
          </member>
          <member>
            <name>last</name>
            <value><string>User</string></value>
          </member>
          <member>
            <name>passwd</name>
            <value><string>$1$d8e8fca2dc0aa04b9c9e6c3456b74a4f</string></value>
          </member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>"#;

        let request = XmlRpcParser::parse_login_request(xml).unwrap();
        assert_eq!(request.first, "Test");
        assert_eq!(request.last, "User");
        assert!(request.passwd.starts_with("$1$"));
    }
}
