use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::session::SessionManager;

use super::events::{EventHandler, SceneEvent};
use super::services::ServiceRegistry;
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};

pub trait IEstateModule: Send + Sync + 'static {
    fn get_estate_id(&self) -> u32;
    fn get_estate_name(&self) -> String;
    fn is_banned(&self, agent_id: Uuid) -> bool;
    fn is_manager(&self, agent_id: Uuid) -> bool;
    fn get_estate_owner(&self) -> Uuid;
}

struct EstateSettings {
    estate_id: u32,
    estate_name: String,
    estate_owner: Uuid,
    parent_estate_id: u32,
    estate_flags: u32,
    sun_position: f32,
    use_fixed_sun: bool,
    covenant_id: Uuid,
    ban_list: HashSet<Uuid>,
    access_list: HashSet<Uuid>,
    manager_list: HashSet<Uuid>,
}

impl Default for EstateSettings {
    fn default() -> Self {
        Self {
            estate_id: 100,
            estate_name: "My Estate".to_string(),
            estate_owner: Uuid::nil(),
            parent_estate_id: 1,
            estate_flags: 0,
            sun_position: 6.0,
            use_fixed_sun: false,
            covenant_id: Uuid::nil(),
            ban_list: HashSet::new(),
            access_list: HashSet::new(),
            manager_list: HashSet::new(),
        }
    }
}

pub struct EstateManagementModule {
    settings: Arc<RwLock<EstateSettings>>,
    region_uuid: Option<Uuid>,
    socket: Option<Arc<tokio::net::UdpSocket>>,
    session_manager: Option<Arc<SessionManager>>,
    db: Option<Arc<DatabaseConnection>>,
    service_registry: Option<Arc<RwLock<ServiceRegistry>>>,
}

impl EstateManagementModule {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(RwLock::new(EstateSettings::default())),
            region_uuid: None,
            socket: None,
            session_manager: None,
            db: None,
            service_registry: None,
        }
    }

    async fn load_estate_settings(&self, db: &DatabaseConnection, region_uuid: &Uuid) {
        match db {
            DatabaseConnection::PostgreSQL(pool) => {
                let query = "SELECT \"EstateID\" FROM estate_map WHERE \"RegionID\" = $1";
                if let Ok(Some(row)) = sqlx::query(query)
                    .bind(region_uuid.to_string())
                    .fetch_optional(pool)
                    .await
                {
                    use sqlx::Row;
                    let estate_id: i32 = row.get("EstateID");
                    {
                        let mut settings = self.settings.write();
                        settings.estate_id = estate_id as u32;
                    }

                    let settings_query =
                        "SELECT \"EstateName\", \"EstateOwner\", \"ParentEstateID\", \
                        \"FixedSun\", \"AllowVoice\", \"AllowDirectTeleport\", \
                        \"DenyAnonymous\", \"ResetHomeOnTeleport\", \"SunPosition\" \
                        FROM estate_settings WHERE \"EstateID\" = $1";
                    if let Ok(Some(srow)) = sqlx::query(settings_query)
                        .bind(estate_id)
                        .fetch_optional(pool)
                        .await
                    {
                        let mut settings = self.settings.write();
                        settings.estate_name =
                            srow.try_get::<String, _>("EstateName").unwrap_or_default();
                        let owner_str: String = srow.try_get("EstateOwner").unwrap_or_default();
                        settings.estate_owner = owner_str.parse().unwrap_or(Uuid::nil());
                        settings.parent_estate_id =
                            srow.try_get::<i32, _>("ParentEstateID").unwrap_or(1) as u32;
                        settings.sun_position =
                            srow.try_get::<f64, _>("SunPosition").unwrap_or(6.0) as f32;
                        let fixed_sun: i32 = srow.try_get("FixedSun").unwrap_or(0);
                        settings.use_fixed_sun = fixed_sun != 0;

                        let mut flags: u32 = 0;
                        if srow.try_get::<i32, _>("AllowVoice").unwrap_or(0) != 0 {
                            flags |= 1 << 28;
                        }
                        if srow.try_get::<i32, _>("AllowDirectTeleport").unwrap_or(0) != 0 {
                            flags |= 1 << 26;
                        }
                        if srow.try_get::<i32, _>("DenyAnonymous").unwrap_or(0) != 0 {
                            flags |= 1 << 10;
                        }
                        if srow.try_get::<i32, _>("ResetHomeOnTeleport").unwrap_or(0) != 0 {
                            flags |= 1 << 4;
                        }
                        settings.estate_flags = flags;
                    }

                    self.load_estate_lists(db, estate_id as u32).await;
                    info!(
                        "[ESTATE] Loaded estate {} (id={})",
                        self.settings.read().estate_name,
                        estate_id
                    );
                }
            }
            DatabaseConnection::MySQL(_pool) => {
                info!("[ESTATE] MySQL estate loading not yet implemented");
            }
        }
    }

    async fn load_estate_lists(&self, db: &DatabaseConnection, estate_id: u32) {
        match db {
            DatabaseConnection::PostgreSQL(pool) => {
                if let Ok(rows) =
                    sqlx::query("SELECT \"bannedUUID\" FROM estateban WHERE \"EstateID\" = $1")
                        .bind(estate_id as i32)
                        .fetch_all(pool)
                        .await
                {
                    use sqlx::Row;
                    let mut settings = self.settings.write();
                    for row in rows {
                        let uuid_str: String = row.get("bannedUUID");
                        if let Ok(uuid) = uuid_str.parse::<Uuid>() {
                            settings.ban_list.insert(uuid);
                        }
                    }
                }

                if let Ok(rows) =
                    sqlx::query("SELECT uuid FROM estate_managers WHERE \"EstateID\" = $1")
                        .bind(estate_id as i32)
                        .fetch_all(pool)
                        .await
                {
                    use sqlx::Row;
                    let mut settings = self.settings.write();
                    for row in rows {
                        let uuid_str: String = row.get("uuid");
                        if let Ok(uuid) = uuid_str.parse::<Uuid>() {
                            settings.manager_list.insert(uuid);
                        }
                    }
                }

                if let Ok(rows) =
                    sqlx::query("SELECT uuid FROM estate_users WHERE \"EstateID\" = $1")
                        .bind(estate_id as i32)
                        .fetch_all(pool)
                        .await
                {
                    use sqlx::Row;
                    let mut settings = self.settings.write();
                    for row in rows {
                        let uuid_str: String = row.get("uuid");
                        if let Ok(uuid) = uuid_str.parse::<Uuid>() {
                            settings.access_list.insert(uuid);
                        }
                    }
                }
            }
            DatabaseConnection::MySQL(_pool) => {}
        }
    }
}

impl IEstateModule for EstateManagementModule {
    fn get_estate_id(&self) -> u32 {
        self.settings.read().estate_id
    }
    fn get_estate_name(&self) -> String {
        self.settings.read().estate_name.clone()
    }
    fn is_banned(&self, agent_id: Uuid) -> bool {
        self.settings.read().ban_list.contains(&agent_id)
    }
    fn is_manager(&self, agent_id: Uuid) -> bool {
        let s = self.settings.read();
        s.manager_list.contains(&agent_id) || s.estate_owner == agent_id
    }
    fn get_estate_owner(&self) -> Uuid {
        self.settings.read().estate_owner
    }
}

const ESTATE_OWNER_MESSAGE_REPLY_ID: u32 = 0xFFFF0105; // Low 261

fn build_estate_owner_message(method: &str, invoice: &Uuid, params: &[String]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(256);
    packet.push(0x40);
    packet.extend_from_slice(&0u32.to_be_bytes());
    packet.push(0);
    packet.extend_from_slice(&[0xFF, 0xFF]);
    packet.extend_from_slice(&0x0105u16.to_be_bytes());

    // AgentData
    packet.extend_from_slice(Uuid::nil().as_bytes()); // AgentID
    packet.extend_from_slice(Uuid::nil().as_bytes()); // SessionID
    packet.extend_from_slice(Uuid::nil().as_bytes()); // TransactionID

    // MethodData
    let method_bytes = method.as_bytes();
    packet.push(method_bytes.len() as u8 + 1);
    packet.extend_from_slice(method_bytes);
    packet.push(0);

    packet.extend_from_slice(invoice.as_bytes());

    // ParamList
    packet.push(params.len() as u8);
    for p in params {
        let p_bytes = p.as_bytes();
        packet.push(p_bytes.len() as u8 + 1);
        packet.extend_from_slice(p_bytes);
        packet.push(0);
    }

    packet
}

#[async_trait]
impl RegionModule for EstateManagementModule {
    fn name(&self) -> &'static str {
        "EstateManagementModule"
    }
    fn replaceable_interface(&self) -> Option<&'static str> {
        Some("IEstateModule")
    }

    async fn initialize(&mut self, _config: &ModuleConfig) -> Result<()> {
        info!("[ESTATE MODULE] Initialized");
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = Some(scene.region_uuid);
        self.socket = Some(scene.socket.clone());
        self.session_manager = Some(scene.session_manager.clone());
        self.db = scene.db.clone();
        self.service_registry = Some(scene.service_registry.clone());

        if let Some(ref db) = scene.db {
            self.load_estate_settings(db, &scene.region_uuid).await;
        }

        let handler = Arc::new(EstateEventHandler {
            socket: scene.socket.clone(),
            settings: self.settings.clone(),
        });
        scene.event_bus.subscribe(
            SceneEvent::OnEstateOwnerMessage {
                agent_id: Uuid::nil(),
                method: String::new(),
                params: Vec::new(),
                dest: "0.0.0.0:0".parse().unwrap(),
            },
            handler,
            100,
        );

        scene
            .service_registry
            .write()
            .register::<EstateManagementModule>(Arc::new(EstateManagementModule {
                settings: self.settings.clone(),
                region_uuid: self.region_uuid,
                socket: self.socket.clone(),
                session_manager: self.session_manager.clone(),
                db: self.db.clone(),
                service_registry: self.service_registry.clone(),
            }));

        info!("[ESTATE MODULE] Added to region {:?}", scene.region_name);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[async_trait]
impl SharedRegionModule for EstateManagementModule {}

struct EstateEventHandler {
    socket: Arc<tokio::net::UdpSocket>,
    settings: Arc<RwLock<EstateSettings>>,
}

#[async_trait]
impl EventHandler for EstateEventHandler {
    async fn handle_event(&self, event: &SceneEvent, _scene: &SceneContext) -> Result<()> {
        if let SceneEvent::OnEstateOwnerMessage {
            agent_id,
            method,
            params,
            dest,
        } = event
        {
            match method.as_str() {
                "getinfo" => {
                    let (reply_params, estate_name) = {
                        let s = self.settings.read();
                        let params = vec![
                            s.estate_name.clone(),
                            s.estate_owner.to_string(),
                            s.estate_id.to_string(),
                            s.estate_flags.to_string(),
                            s.sun_position.to_string(),
                            s.parent_estate_id.to_string(),
                            s.covenant_id.to_string(),
                            "0".to_string(),
                            if s.use_fixed_sun { "1" } else { "0" }.to_string(),
                        ];
                        (params, s.estate_name.clone())
                    };
                    let packet =
                        build_estate_owner_message("estateupdateinfo", &Uuid::nil(), &reply_params);
                    let _ = self.socket.send_to(&packet, dest).await;
                    info!(
                        "[ESTATE MODULE] Sent estate info for '{}' to {}",
                        estate_name, dest
                    );
                }
                "setregioninfo" => {
                    info!("[ESTATE MODULE] Agent {} updating region info", agent_id);
                }
                "estate_access_delta" => {
                    if params.len() >= 2 {
                        info!(
                            "[ESTATE MODULE] Agent {} modifying estate access: {:?}",
                            agent_id, params
                        );
                    }
                }
                "restart" => {
                    warn!(
                        "[ESTATE MODULE] Agent {} requested region restart",
                        agent_id
                    );
                }
                "kickestate" => {
                    info!("[ESTATE MODULE] Agent {} kicked user from estate", agent_id);
                }
                "estatechangecovenantid" => {
                    if let Some(covenant_str) = params.first() {
                        if let Ok(cov_id) = covenant_str.parse::<Uuid>() {
                            self.settings.write().covenant_id = cov_id;
                            info!("[ESTATE MODULE] Covenant updated to {}", cov_id);
                        }
                    }
                }
                _ => {
                    info!(
                        "[ESTATE MODULE] Unhandled method '{}' from {}",
                        method, agent_id
                    );
                }
            }
        }
        Ok(())
    }
}
