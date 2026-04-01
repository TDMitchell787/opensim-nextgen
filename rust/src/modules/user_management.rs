use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::session::SessionManager;

use super::events::{EventHandler, SceneEvent};
use super::services::ServiceRegistry;
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};
use crate::udp::server::AvatarMovementState;

pub trait IUserManagement: Send + Sync + 'static {
    fn get_user_name(&self, user_id: Uuid) -> Option<(String, String)>;
    fn add_user(&self, user_id: Uuid, first: String, last: String);
    fn request_name(&self, user_id: Uuid);
}

pub struct UserManagementModule {
    name_cache: Arc<RwLock<HashMap<Uuid, (String, String)>>>,
    db: Option<Arc<DatabaseConnection>>,
    session_manager: Option<Arc<SessionManager>>,
    avatar_states: Option<Arc<RwLock<HashMap<Uuid, AvatarMovementState>>>>,
    service_registry: Option<Arc<RwLock<ServiceRegistry>>>,
}

impl UserManagementModule {
    pub fn new() -> Self {
        Self {
            name_cache: Arc::new(RwLock::new(HashMap::new())),
            db: None,
            session_manager: None,
            avatar_states: None,
            service_registry: None,
        }
    }

    async fn populate_cache_from_db(&self, db: &DatabaseConnection) {
        match db {
            DatabaseConnection::PostgreSQL(pool) => {
                let query = "SELECT \"PrincipalID\", \"FirstName\", \"LastName\" FROM useraccounts LIMIT 10000";
                if let Ok(rows) = sqlx::query(query).fetch_all(pool).await {
                    use sqlx::Row;
                    let mut cache = self.name_cache.write();
                    for row in rows {
                        let id_str: String = row.get("PrincipalID");
                        if let Ok(id) = id_str.parse::<Uuid>() {
                            let first: String = row.get("FirstName");
                            let last: String = row.get("LastName");
                            cache.insert(id, (first, last));
                        }
                    }
                    info!("[USER MGMT] Populated name cache with {} entries", cache.len());
                }
            }
            DatabaseConnection::MySQL(_pool) => {
                info!("[USER MGMT] MySQL name cache not yet implemented");
            }
        }
    }
}

impl IUserManagement for UserManagementModule {
    fn get_user_name(&self, user_id: Uuid) -> Option<(String, String)> {
        self.name_cache.read().get(&user_id).cloned()
    }

    fn add_user(&self, user_id: Uuid, first: String, last: String) {
        self.name_cache.write().insert(user_id, (first, last));
    }

    fn request_name(&self, _user_id: Uuid) {}
}

#[async_trait]
impl RegionModule for UserManagementModule {
    fn name(&self) -> &'static str { "UserManagementModule" }
    fn replaceable_interface(&self) -> Option<&'static str> { Some("IUserManagement") }

    async fn initialize(&mut self, _config: &ModuleConfig) -> Result<()> {
        info!("[USER MGMT MODULE] Initialized");
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.db = scene.db.clone();
        self.session_manager = Some(scene.session_manager.clone());
        self.avatar_states = Some(scene.avatar_states.clone());
        self.service_registry = Some(scene.service_registry.clone());

        if let Some(ref db) = scene.db {
            self.populate_cache_from_db(db).await;
        }

        let handler = Arc::new(UserMgmtEventHandler {
            name_cache: self.name_cache.clone(),
            session_manager: scene.session_manager.clone(),
        });
        scene.event_bus.subscribe(
            SceneEvent::OnNewClient {
                agent_id: Uuid::nil(),
                session_id: Uuid::nil(),
                addr: "0.0.0.0:0".parse().unwrap(),
            },
            handler,
            50,
        );

        scene.service_registry.write().register::<UserManagementModule>(
            Arc::new(UserManagementModule {
                name_cache: self.name_cache.clone(),
                db: self.db.clone(),
                session_manager: self.session_manager.clone(),
                avatar_states: self.avatar_states.clone(),
                service_registry: self.service_registry.clone(),
            }),
        );

        info!("[USER MGMT MODULE] Added to region {:?}", scene.region_name);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[async_trait]
impl SharedRegionModule for UserManagementModule {}

struct UserMgmtEventHandler {
    name_cache: Arc<RwLock<HashMap<Uuid, (String, String)>>>,
    session_manager: Arc<SessionManager>,
}

#[async_trait]
impl EventHandler for UserMgmtEventHandler {
    async fn handle_event(&self, event: &SceneEvent, _scene: &SceneContext) -> Result<()> {
        if let SceneEvent::OnNewClient { agent_id, .. } = event {
            if let Some(session) = self.session_manager.get_session_by_agent_id(*agent_id) {
                let mut cache = self.name_cache.write();
                cache.insert(*agent_id, (session.first_name.clone(), session.last_name.clone()));
            }
        }
        Ok(())
    }
}
