use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::info;
use uuid::Uuid;

use super::events::{EventHandler, PermissionAction, SceneEvent};
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionType {
    RezObject,
    EditObject,
    DeleteObject,
    MoveObject,
    CopyObject,
    TakeObject,
    ModifyTerrain,
    RunScript,
    Fly,
    CreateLandmark,
    SetHome,
}

pub trait IPermissionsModule: Send + Sync + 'static {
    fn can_do(&self, agent_id: &Uuid, target_id: &Uuid, action: PermissionType) -> bool;
    fn is_god(&self, agent_id: &Uuid) -> bool;
    fn is_estate_manager(&self, agent_id: &Uuid) -> bool;
}

struct PermissionsConfig {
    bypass_permissions: bool,
    god_users: HashSet<Uuid>,
    estate_managers: HashSet<Uuid>,
    region_owner: Option<Uuid>,
    allow_grid_gods: bool,
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        Self {
            bypass_permissions: false,
            god_users: HashSet::new(),
            estate_managers: HashSet::new(),
            region_owner: None,
            allow_grid_gods: true,
        }
    }
}

pub struct PermissionsModule {
    config: PermissionsConfig,
    scene_objects:
        Option<Arc<RwLock<std::collections::HashMap<u32, crate::udp::server::SceneObject>>>>,
}

impl PermissionsModule {
    pub fn new() -> Self {
        Self {
            config: PermissionsConfig::default(),
            scene_objects: None,
        }
    }

    fn check_permission(&self, agent_id: &Uuid, target_id: &Uuid, action: PermissionType) -> bool {
        if self.config.bypass_permissions {
            return true;
        }

        if self.is_god(agent_id) {
            return true;
        }

        if self.is_estate_manager(agent_id) {
            return true;
        }

        if let Some(owner) = &self.config.region_owner {
            if agent_id == owner {
                return true;
            }
        }

        match action {
            PermissionType::Fly | PermissionType::CreateLandmark => true,

            PermissionType::RezObject | PermissionType::ModifyTerrain | PermissionType::SetHome => {
                self.config.region_owner.as_ref() == Some(agent_id)
                    || self.is_estate_manager(agent_id)
            }

            PermissionType::EditObject
            | PermissionType::DeleteObject
            | PermissionType::MoveObject
            | PermissionType::CopyObject
            | PermissionType::TakeObject => self.is_object_owner(agent_id, target_id),

            PermissionType::RunScript => true,
        }
    }

    fn is_object_owner(&self, agent_id: &Uuid, object_uuid: &Uuid) -> bool {
        if let Some(objects) = &self.scene_objects {
            let objs = objects.read();
            for obj in objs.values() {
                if obj.uuid == *object_uuid {
                    return obj.owner_id == *agent_id;
                }
            }
        }
        false
    }
}

impl IPermissionsModule for PermissionsModule {
    fn can_do(&self, agent_id: &Uuid, target_id: &Uuid, action: PermissionType) -> bool {
        self.check_permission(agent_id, target_id, action)
    }

    fn is_god(&self, agent_id: &Uuid) -> bool {
        self.config.god_users.contains(agent_id)
            || (self.config.allow_grid_gods && self.config.region_owner.as_ref() == Some(agent_id))
    }

    fn is_estate_manager(&self, agent_id: &Uuid) -> bool {
        self.config.estate_managers.contains(agent_id)
    }
}

#[async_trait]
impl RegionModule for PermissionsModule {
    fn name(&self) -> &'static str {
        "PermissionsModule"
    }

    fn replaceable_interface(&self) -> Option<&'static str> {
        Some("IPermissionsModule")
    }

    async fn initialize(&mut self, config: &ModuleConfig) -> Result<()> {
        self.config.bypass_permissions = config.get_bool("bypass_permissions", false);
        self.config.allow_grid_gods = config.get_bool("allow_grid_gods", true);

        if let Some(owner_str) = config.get("region_owner") {
            if let Ok(uuid) = Uuid::parse_str(owner_str) {
                self.config.region_owner = Some(uuid);
            }
        }

        if let Some(gods_str) = config.get("god_users") {
            for id_str in gods_str.split(',') {
                if let Ok(uuid) = Uuid::parse_str(id_str.trim()) {
                    self.config.god_users.insert(uuid);
                }
            }
        }

        if let Some(em_str) = config.get("estate_managers") {
            for id_str in em_str.split(',') {
                if let Ok(uuid) = Uuid::parse_str(id_str.trim()) {
                    self.config.estate_managers.insert(uuid);
                }
            }
        }

        info!(
            "[PERMISSIONS MODULE] Initialized: bypass={}, gods={}, estate_managers={}",
            self.config.bypass_permissions,
            self.config.god_users.len(),
            self.config.estate_managers.len(),
        );
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.scene_objects = Some(scene.scene_objects.clone());

        let handler = Arc::new(PermissionCheckHandler {
            bypass: self.config.bypass_permissions,
            god_users: self.config.god_users.clone(),
            estate_managers: self.config.estate_managers.clone(),
            region_owner: self.config.region_owner,
            allow_grid_gods: self.config.allow_grid_gods,
            scene_objects: scene.scene_objects.clone(),
        });

        scene.event_bus.subscribe(
            SceneEvent::OnPermissionCheck {
                agent_id: Uuid::nil(),
                target_id: Uuid::nil(),
                action: PermissionAction::Fly,
                result: Arc::new(RwLock::new(super::events::PermissionCheckResult {
                    allowed: false,
                    reason: None,
                })),
            },
            handler,
            200,
        );

        scene
            .service_registry
            .write()
            .register::<PermissionsModule>(Arc::new(PermissionsModule {
                config: PermissionsConfig {
                    bypass_permissions: self.config.bypass_permissions,
                    god_users: self.config.god_users.clone(),
                    estate_managers: self.config.estate_managers.clone(),
                    region_owner: self.config.region_owner,
                    allow_grid_gods: self.config.allow_grid_gods,
                },
                scene_objects: self.scene_objects.clone(),
            }));

        info!(
            "[PERMISSIONS MODULE] Added to region '{}'",
            scene.region_name
        );
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
impl SharedRegionModule for PermissionsModule {}

struct PermissionCheckHandler {
    bypass: bool,
    god_users: HashSet<Uuid>,
    estate_managers: HashSet<Uuid>,
    region_owner: Option<Uuid>,
    allow_grid_gods: bool,
    scene_objects: Arc<RwLock<std::collections::HashMap<u32, crate::udp::server::SceneObject>>>,
}

impl PermissionCheckHandler {
    fn is_god(&self, agent_id: &Uuid) -> bool {
        self.god_users.contains(agent_id)
            || (self.allow_grid_gods && self.region_owner.as_ref() == Some(agent_id))
    }

    fn is_estate_manager(&self, agent_id: &Uuid) -> bool {
        self.estate_managers.contains(agent_id)
    }

    fn is_object_owner(&self, agent_id: &Uuid, object_uuid: &Uuid) -> bool {
        let objs = self.scene_objects.read();
        for obj in objs.values() {
            if obj.uuid == *object_uuid {
                return obj.owner_id == *agent_id;
            }
        }
        false
    }
}

#[async_trait]
impl EventHandler for PermissionCheckHandler {
    async fn handle_event(&self, event: &SceneEvent, _scene: &SceneContext) -> Result<()> {
        if let SceneEvent::OnPermissionCheck {
            agent_id,
            target_id,
            action,
            result,
        } = event
        {
            if self.bypass {
                let mut r = result.write();
                r.allowed = true;
                return Ok(());
            }

            if self.is_god(agent_id) || self.is_estate_manager(agent_id) {
                let mut r = result.write();
                r.allowed = true;
                return Ok(());
            }

            if let Some(owner) = &self.region_owner {
                if agent_id == owner {
                    let mut r = result.write();
                    r.allowed = true;
                    return Ok(());
                }
            }

            let allowed = match action {
                PermissionAction::Fly | PermissionAction::CreateLandmark => true,
                PermissionAction::RezObject
                | PermissionAction::ModifyTerrain
                | PermissionAction::SetHome => {
                    self.region_owner.as_ref() == Some(agent_id) || self.is_estate_manager(agent_id)
                }
                PermissionAction::EditObject
                | PermissionAction::DeleteObject
                | PermissionAction::MoveObject
                | PermissionAction::CopyObject
                | PermissionAction::TakeObject => self.is_object_owner(agent_id, target_id),
                PermissionAction::RunScript => true,
            };

            let mut r = result.write();
            r.allowed = allowed;
            if !allowed {
                r.reason = Some(format!(
                    "Agent {} denied {:?} on {}",
                    agent_id, action, target_id
                ));
            }
        }
        Ok(())
    }
}
