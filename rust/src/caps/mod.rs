use crate::baking::cache::{BakedTextureCache, SharedBakedTextureCache};
use crate::login_stage_tracker::LoginStageTracker;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

pub mod event_queue;
pub mod handlers;
pub mod router;

pub use event_queue::*;
pub use handlers::*;
pub use router::*;

#[derive(Debug, Clone)]
pub struct PendingUpload {
    pub name: String,
    pub description: String,
    pub asset_type: String,
    pub inventory_type: String,
    pub folder_id: String,
    pub task_id: Option<String>,
    pub is_script_running: bool,
}

#[derive(Debug, Clone)]
pub struct CapsSession {
    pub session_id: String,
    pub agent_id: String,
    pub circuit_code: u32,
    pub region_uuid: String,
    pub capabilities: HashMap<String, String>,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub udp_circuit_ready: bool,
    pub sent_seeds: bool,
    pub region_handshake_reply_received: bool,
    pub initial_data_sent: bool,
    pub initial_data_ready_time: Option<Instant>,
    pub pending_uploads: HashMap<String, PendingUpload>,
}

impl CapsSession {
    pub fn new(session_id: String, agent_id: String, circuit_code: u32) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            agent_id,
            circuit_code,
            region_uuid: String::new(),
            capabilities: HashMap::new(),
            created_at: now,
            last_activity: now,
            udp_circuit_ready: false,
            sent_seeds: false,
            region_handshake_reply_received: false,
            initial_data_sent: false,
            initial_data_ready_time: None,
            pending_uploads: HashMap::new(),
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
}

pub struct CapsManager {
    sessions: Arc<RwLock<HashMap<String, CapsSession>>>,
    circuit_code_map: Arc<RwLock<HashMap<u32, String>>>,
    eqg_uuid_map: Arc<RwLock<HashMap<String, String>>>, // eqg_uuid -> session_id
    event_queue: Arc<EventQueueManager>,
    baked_texture_cache: SharedBakedTextureCache,
    pub base_url: String,
    session_timeout: Duration,
    pub default_region_uuid: String,
}

impl Clone for CapsManager {
    fn clone(&self) -> Self {
        Self {
            sessions: self.sessions.clone(),
            circuit_code_map: self.circuit_code_map.clone(),
            eqg_uuid_map: self.eqg_uuid_map.clone(),
            event_queue: self.event_queue.clone(),
            baked_texture_cache: self.baked_texture_cache.clone(),
            base_url: self.base_url.clone(),
            session_timeout: self.session_timeout,
            default_region_uuid: self.default_region_uuid.clone(),
        }
    }
}

impl std::fmt::Debug for CapsManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapsManager")
            .field("base_url", &self.base_url)
            .field("session_timeout", &self.session_timeout)
            .finish()
    }
}

impl CapsManager {
    pub fn new(base_url: String) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            circuit_code_map: Arc::new(RwLock::new(HashMap::new())),
            eqg_uuid_map: Arc::new(RwLock::new(HashMap::new())),
            event_queue: Arc::new(EventQueueManager::new()),
            baked_texture_cache: Arc::new(BakedTextureCache::new(1000)),
            base_url,
            session_timeout: Duration::from_secs(300),
            default_region_uuid: String::new(),
        }
    }

    pub fn with_default_region_uuid(mut self, uuid: String) -> Self {
        self.default_region_uuid = uuid;
        self
    }

    pub fn get_baked_texture_cache(&self) -> SharedBakedTextureCache {
        self.baked_texture_cache.clone()
    }

    pub fn store_baked_texture(&self, texture_id: Uuid, data: Vec<u8>) {
        self.baked_texture_cache.store_texture(texture_id, data);
    }

    pub fn get_baked_texture(&self, texture_id: &Uuid) -> Option<Vec<u8>> {
        self.baked_texture_cache.get_texture(texture_id)
    }

    pub async fn create_session(
        &self,
        agent_id: String,
        circuit_code: u32,
        region_uuid: Option<String>,
    ) -> String {
        let session_id = Uuid::new_v4().to_string();
        let mut session = CapsSession::new(session_id.clone(), agent_id.clone(), circuit_code);
        session.region_uuid = region_uuid.unwrap_or_else(|| self.default_region_uuid.clone());

        // Generate capability URLs
        let cap_base = format!("{}/cap/{}", self.base_url, session_id);

        // EventQueueGet uses OpenSim-compatible /CE/{uuid} format
        let eqg_uuid = Uuid::new_v4().to_string();
        session.capabilities.insert(
            "EventQueueGet".to_string(),
            format!("{}/CE/{}", self.base_url, eqg_uuid),
        );
        session.capabilities.insert(
            "FetchInventory2".to_string(),
            format!("{}/FetchInventory2", cap_base),
        );
        session.capabilities.insert(
            "FetchInventoryDescendents2".to_string(),
            format!("{}/FetchInventoryDescendents2", cap_base),
        );
        session.capabilities.insert(
            "FetchLibDescendents2".to_string(),
            format!("{}/FetchLibDescendents2", cap_base),
        );
        session
            .capabilities
            .insert("FetchLib2".to_string(), format!("{}/FetchLib2", cap_base));
        session.capabilities.insert(
            "UpdateAvatarAppearance".to_string(),
            format!("{}/UpdateAvatarAppearance", cap_base),
        );
        session
            .capabilities
            .insert("GetTexture".to_string(), format!("{}/GetTexture", cap_base));
        session.capabilities.insert(
            "ViewerStats".to_string(),
            format!("{}/ViewerStats", cap_base),
        );
        session.capabilities.insert(
            "UpdateAgentInformation".to_string(),
            format!("{}/UpdateAgentInformation", cap_base),
        );
        session.capabilities.insert(
            "UpdateAgentLanguage".to_string(),
            format!("{}/UpdateAgentLanguage", cap_base),
        );
        session.capabilities.insert(
            "AgentPreferences".to_string(),
            format!("{}/AgentPreferences", cap_base),
        );
        session.capabilities.insert(
            "HomeLocation".to_string(),
            format!("{}/HomeLocation", cap_base),
        );
        session.capabilities.insert(
            "GetDisplayNames".to_string(),
            format!("{}/GetDisplayNames", cap_base),
        );
        session.capabilities.insert(
            "SetDisplayName".to_string(),
            format!("{}/SetDisplayName", cap_base),
        );
        session.capabilities.insert(
            "CreateInventoryCategory".to_string(),
            format!("{}/CreateInventoryCategory", cap_base),
        );
        session.capabilities.insert(
            "NewFileAgentInventory".to_string(),
            format!("{}/NewFileAgentInventory", cap_base),
        );
        session.capabilities.insert(
            "UpdateNotecardAgentInventory".to_string(),
            format!("{}/UpdateNotecardAgentInventory", cap_base),
        );
        session.capabilities.insert(
            "UpdateScriptAgentInventory".to_string(),
            format!("{}/UpdateScriptAgentInventory", cap_base),
        );
        session.capabilities.insert(
            "UpdateScriptTask".to_string(),
            format!("{}/UpdateScriptTask", cap_base),
        );
        session.capabilities.insert(
            "UpdateNotecardTaskInventory".to_string(),
            format!("{}/UpdateNotecardTaskInventory", cap_base),
        );
        session.capabilities.insert(
            "ParcelPropertiesUpdate".to_string(),
            format!("{}/ParcelPropertiesUpdate", cap_base),
        );
        session
            .capabilities
            .insert("MapLayer".to_string(), format!("{}/MapLayer", cap_base));
        session.capabilities.insert(
            "SimulatorFeatures".to_string(),
            format!("{}/SimulatorFeatures", cap_base),
        );
        session.capabilities.insert(
            "EnvironmentSettings".to_string(),
            format!("{}/EnvironmentSettings", cap_base),
        );
        session.capabilities.insert(
            "ExtEnvironment".to_string(),
            format!("{}/ExtEnvironment", cap_base),
        );
        session.capabilities.insert(
            "UploadBakedTexture".to_string(),
            format!("{}/UploadBakedTexture", cap_base),
        );
        session
            .capabilities
            .insert("GetMesh".to_string(), format!("{}/GetMesh", cap_base));
        session
            .capabilities
            .insert("GetMesh2".to_string(), format!("{}/GetMesh2", cap_base));
        session.capabilities.insert(
            "ViewerAsset".to_string(),
            format!("{}/ViewerAsset", cap_base),
        );
        session.capabilities.insert(
            "MeshUploadFlag".to_string(),
            format!("{}/MeshUploadFlag", cap_base),
        );
        session.capabilities.insert(
            "GetObjectCost".to_string(),
            format!("{}/GetObjectCost", cap_base),
        );
        session.capabilities.insert(
            "GetObjectPhysicsData".to_string(),
            format!("{}/GetObjectPhysicsData", cap_base),
        );
        session.capabilities.insert(
            "ResourceCostSelected".to_string(),
            format!("{}/ResourceCostSelected", cap_base),
        );
        session.capabilities.insert(
            "ProvisionVoiceAccountRequest".to_string(),
            format!("{}/ProvisionVoiceAccountRequest", cap_base),
        );
        session.capabilities.insert(
            "ParcelVoiceInfoRequest".to_string(),
            format!("{}/ParcelVoiceInfoRequest", cap_base),
        );
        session.capabilities.insert(
            "AgentProfile".to_string(),
            format!("{}/AgentProfile", cap_base),
        );
        session.capabilities.insert(
            "RenderMaterials".to_string(),
            format!("{}/RenderMaterials", cap_base),
        );
        session.capabilities.insert(
            "ModifyMaterialParams".to_string(),
            format!("{}/ModifyMaterialParams", cap_base),
        );
        session.capabilities.insert(
            "ObjectMedia".to_string(),
            format!("{}/ObjectMedia", cap_base),
        );
        session.capabilities.insert(
            "ObjectMediaNavigate".to_string(),
            format!("{}/ObjectMediaNavigate", cap_base),
        );
        session.capabilities.insert(
            "SearchStatRequest".to_string(),
            format!("{}/SearchStatRequest", cap_base),
        );
        session.capabilities.insert(
            "LandResources".to_string(),
            format!("{}/LandResources", cap_base),
        );
        session.capabilities.insert(
            "AvatarPickerSearch".to_string(),
            format!("{}/AvatarPickerSearch", cap_base),
        );
        session.capabilities.insert(
            "DispatchRegionInfo".to_string(),
            format!("{}/DispatchRegionInfo", cap_base),
        );
        session.capabilities.insert(
            "ProductInfoRequest".to_string(),
            format!("{}/ProductInfoRequest", cap_base),
        );
        session.capabilities.insert(
            "ServerReleaseNotes".to_string(),
            format!("{}/ServerReleaseNotes", cap_base),
        );
        session.capabilities.insert(
            "CopyInventoryFromNotecard".to_string(),
            format!("{}/CopyInventoryFromNotecard", cap_base),
        );
        session.capabilities.insert(
            "UpdateGestureAgentInventory".to_string(),
            format!("{}/UpdateGestureAgentInventory", cap_base),
        );
        session.capabilities.insert(
            "UpdateGestureTaskInventory".to_string(),
            format!("{}/UpdateGestureTaskInventory", cap_base),
        );
        session
            .capabilities
            .insert("LSLSyntax".to_string(), format!("{}/LSLSyntax", cap_base));
        session.capabilities.insert(
            "ScriptResourceSummary".to_string(),
            format!("{}/ScriptResourceSummary", cap_base),
        );
        session.capabilities.insert(
            "ScriptResourceDetails".to_string(),
            format!("{}/ScriptResourceDetails", cap_base),
        );
        session.capabilities.insert(
            "NewFileAgentInventoryVariablePrice".to_string(),
            format!("{}/NewFileAgentInventoryVariablePrice", cap_base),
        );

        // Initialize event queue for this session
        self.event_queue.create_session(session_id.clone()).await;

        // NOTE: Login events are now sent from udp/server.rs after CompleteAgentMovement
        // Sending them here creates duplicates with wrong Port 9001
        // self.send_login_events(&session_id).await;

        // Store session and circuit_code mapping
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }
        {
            let mut circuit_map = self.circuit_code_map.write().await;
            circuit_map.insert(circuit_code, session_id.clone());
        }
        {
            let mut eqg_map = self.eqg_uuid_map.write().await;
            eqg_map.insert(eqg_uuid.clone(), session_id.clone());
        }

        info!(
            "Created CAPS session {} for agent {} with circuit_code {} and EQG UUID {}",
            session_id, agent_id, circuit_code, eqg_uuid
        );
        session_id
    }

    pub async fn get_session(&self, session_id: &str) -> Option<CapsSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn get_session_by_circuit_code(&self, circuit_code: u32) -> Option<CapsSession> {
        let circuit_map = self.circuit_code_map.read().await;
        if let Some(session_id) = circuit_map.get(&circuit_code) {
            let sessions = self.sessions.read().await;
            sessions.get(session_id).cloned()
        } else {
            None
        }
    }

    pub async fn get_session_id_for_agent(&self, agent_id: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        for (sid, session) in sessions.iter() {
            if session.agent_id == agent_id {
                return Some(sid.clone());
            }
        }
        None
    }

    pub async fn get_session_by_eqg_uuid(&self, eqg_uuid: &str) -> Option<CapsSession> {
        let eqg_map = self.eqg_uuid_map.read().await;
        if let Some(session_id) = eqg_map.get(eqg_uuid) {
            let sessions = self.sessions.read().await;
            sessions.get(session_id).cloned()
        } else {
            None
        }
    }

    pub async fn get_capabilities(&self, session_id: &str) -> Option<HashMap<String, String>> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|session| session.capabilities.clone())
    }

    pub async fn update_session_activity(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.update_activity();
        }
    }

    pub async fn mark_udp_circuit_ready(&self, circuit_code: u32) {
        let circuit_map = self.circuit_code_map.read().await;
        if let Some(session_id) = circuit_map.get(&circuit_code) {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.udp_circuit_ready = true;
                let agent_id = session.agent_id.clone();
                info!(
                    "[CAPS] ✅ UDP circuit marked ready for session {} (circuit_code: {})",
                    session_id, circuit_code
                );
                drop(sessions);

                // Trigger EventQueue to send pending login events if viewer is waiting
                self.event_queue
                    .notify_udp_circuit_ready(session_id, &agent_id)
                    .await;
            }
        }
    }

    pub async fn mark_sent_seeds(&self, session_id: &str) -> Option<(bool, u32, String)> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.sent_seeds = true;
            info!(
                "[CAPS] 🌱 SentSeeds marked for session {} (circuit_code: {})",
                session_id, session.circuit_code
            );

            let both_ready = session.sent_seeds
                && session.region_handshake_reply_received
                && !session.initial_data_sent;
            if both_ready && session.initial_data_ready_time.is_none() {
                session.initial_data_ready_time = Some(Instant::now());
                info!("[CAPS] ⏱️ Both conditions met! Initial data will be sent after delay (session: {})", session_id);
            }

            Some((both_ready, session.circuit_code, session.agent_id.clone()))
        } else {
            None
        }
    }

    pub async fn mark_region_handshake_reply_received(
        &self,
        circuit_code: u32,
    ) -> Option<(bool, String, String)> {
        let circuit_map = self.circuit_code_map.read().await;
        if let Some(session_id) = circuit_map.get(&circuit_code) {
            let session_id = session_id.clone();
            drop(circuit_map);

            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                session.region_handshake_reply_received = true;
                info!(
                    "[CAPS] 📬 RegionHandshakeReply received for session {} (circuit_code: {})",
                    session_id, circuit_code
                );

                let both_ready = session.sent_seeds
                    && session.region_handshake_reply_received
                    && !session.initial_data_sent;
                if both_ready && session.initial_data_ready_time.is_none() {
                    session.initial_data_ready_time = Some(Instant::now());
                    info!("[CAPS] ⏱️ Both conditions met! Initial data will be sent after delay (session: {})", session_id);
                }

                Some((both_ready, session_id.clone(), session.agent_id.clone()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn reset_session_for_teleport(&self, circuit_code: u32) {
        let circuit_map = self.circuit_code_map.read().await;
        if let Some(session_id) = circuit_map.get(&circuit_code) {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.sent_seeds = false;
                session.region_handshake_reply_received = false;
                session.initial_data_sent = false;
                session.initial_data_ready_time = None;
                info!(
                    "[CAPS] Reset session {} flags for cross-region teleport (circuit_code: {})",
                    session_id, circuit_code
                );
            }
        }
    }

    pub async fn mark_initial_data_sent(&self, circuit_code: u32) -> bool {
        let circuit_map = self.circuit_code_map.read().await;
        if let Some(session_id) = circuit_map.get(&circuit_code) {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                if session.initial_data_sent {
                    info!(
                        "[CAPS] ⚠️ Initial data already sent for session {} - skipping duplicate",
                        session_id
                    );
                    return false;
                }
                session.initial_data_sent = true;
                info!(
                    "[CAPS] ✅ Initial data marked as sent for session {} (circuit_code: {})",
                    session_id, circuit_code
                );
                return true;
            }
        }
        false
    }

    pub async fn check_initial_data_readiness(
        &self,
        circuit_code: u32,
    ) -> Option<(bool, Duration)> {
        let circuit_map = self.circuit_code_map.read().await;
        if let Some(session_id) = circuit_map.get(&circuit_code) {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(session_id) {
                let ready = session.sent_seeds
                    && session.region_handshake_reply_received
                    && !session.initial_data_sent;
                let elapsed = session
                    .initial_data_ready_time
                    .map(|t| t.elapsed())
                    .unwrap_or(Duration::ZERO);
                return Some((ready, elapsed));
            }
        }
        None
    }

    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let expired_sessions: Vec<(String, u32)> = sessions
            .iter()
            .filter(|(_, session)| session.is_expired(self.session_timeout))
            .map(|(id, session)| (id.clone(), session.circuit_code))
            .collect();

        for (session_id, circuit_code) in expired_sessions {
            sessions.remove(&session_id);
            self.event_queue.remove_session(&session_id).await;

            let mut circuit_map = self.circuit_code_map.write().await;
            circuit_map.remove(&circuit_code);

            info!(
                "Removed expired CAPS session: {} (circuit_code: {})",
                session_id, circuit_code
            );
        }
    }

    // Phase 67.8: This function is DEPRECATED - DO NOT USE for fresh login
    // EnableSimulator and TeleportFinish events tell the viewer to connect to a NEW region,
    // which triggers UseCircuitCode and causes infinite reconnection loops during fresh login.
    // Use send_delayed_login_events() from EventQueueManager instead for login-appropriate events.
    #[allow(dead_code)]
    pub async fn send_login_events(&self, _session_id: &str) {
        warn!("[Phase 67.8] send_login_events() called but disabled - EnableSimulator/TeleportFinish cause login loops");
        // No events sent - this function should not be used for fresh login
    }

    pub fn get_event_queue(&self) -> Arc<EventQueueManager> {
        self.event_queue.clone()
    }
}

#[derive(Clone)]
pub struct CapsHandlerState {
    pub caps_manager: Arc<CapsManager>,
    pub db_pool: Arc<sqlx::PgPool>,
    pub stage_tracker: Arc<LoginStageTracker>,
    pub avatar_factory: Option<Arc<crate::avatar::factory::AvatarFactory>>,
    pub voice_module: Option<Arc<dyn crate::modules::voice::VoiceHandler>>,
    pub scene_objects: Option<
        Arc<parking_lot::RwLock<std::collections::HashMap<u32, crate::udp::server::SceneObject>>>,
    >,
    pub yengine: Option<Arc<parking_lot::RwLock<crate::scripting::yengine_module::YEngineModule>>>,
    pub parcels: Option<Arc<parking_lot::RwLock<Vec<crate::modules::land::Parcel>>>>,
    pub asset_fetcher: Option<Arc<crate::asset::AssetFetcher>>,
}

impl std::fmt::Debug for CapsHandlerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapsHandlerState")
            .field("caps_manager", &self.caps_manager)
            .field("db_pool", &"PgPool")
            .field("stage_tracker", &"LoginStageTracker")
            .field("avatar_factory", &self.avatar_factory.is_some())
            .field("voice_module", &self.voice_module.is_some())
            .field("scene_objects", &self.scene_objects.is_some())
            .field("yengine", &self.yengine.is_some())
            .field("parcels", &self.parcels.is_some())
            .field("asset_fetcher", &self.asset_fetcher.is_some())
            .finish()
    }
}

impl CapsHandlerState {
    pub fn new(
        caps_manager: Arc<CapsManager>,
        db_pool: Arc<sqlx::PgPool>,
        stage_tracker: Arc<LoginStageTracker>,
    ) -> Self {
        Self {
            caps_manager,
            db_pool,
            stage_tracker,
            avatar_factory: None,
            voice_module: None,
            scene_objects: None,
            yengine: None,
            parcels: None,
            asset_fetcher: None,
        }
    }

    pub fn with_avatar_factory(
        mut self,
        factory: Arc<crate::avatar::factory::AvatarFactory>,
    ) -> Self {
        self.avatar_factory = Some(factory);
        self
    }

    pub fn with_voice_module(
        mut self,
        module: Arc<dyn crate::modules::voice::VoiceHandler>,
    ) -> Self {
        self.voice_module = Some(module);
        self
    }

    pub fn with_scene_objects(
        mut self,
        objects: Arc<
            parking_lot::RwLock<std::collections::HashMap<u32, crate::udp::server::SceneObject>>,
        >,
    ) -> Self {
        self.scene_objects = Some(objects);
        self
    }

    pub fn with_asset_fetcher(mut self, fetcher: Arc<crate::asset::AssetFetcher>) -> Self {
        self.asset_fetcher = Some(fetcher);
        self
    }
}
