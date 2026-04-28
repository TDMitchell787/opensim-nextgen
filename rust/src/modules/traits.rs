use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::session::SessionManager;
use crate::udp::server::{AvatarMovementState, SceneObject};

use super::events::EventBus;
use super::services::ServiceRegistry;

pub struct SceneContext {
    pub region_uuid: Uuid,
    pub region_handle: u64,
    pub region_name: String,
    pub grid_x: u32,
    pub grid_y: u32,
    pub avatar_states: Arc<RwLock<HashMap<Uuid, AvatarMovementState>>>,
    pub scene_objects: Arc<RwLock<HashMap<u32, SceneObject>>>,
    pub session_manager: Arc<SessionManager>,
    pub db: Option<Arc<DatabaseConnection>>,
    pub socket: Arc<tokio::net::UdpSocket>,
    pub event_bus: Arc<EventBus>,
    pub service_registry: Arc<RwLock<ServiceRegistry>>,
}

pub struct ModuleConfig {
    pub section_name: String,
    pub params: HashMap<String, String>,
}

impl ModuleConfig {
    pub fn new(section_name: &str) -> Self {
        Self {
            section_name: section_name.to_string(),
            params: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.params
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn get_f32(&self, key: &str, default: f32) -> f32 {
        self.params
            .get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.params
            .get(key)
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "yes" | "1"))
            .unwrap_or(default)
    }
}

#[async_trait]
pub trait RegionModule: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn replaceable_interface(&self) -> Option<&'static str> {
        None
    }

    async fn initialize(&mut self, config: &ModuleConfig) -> Result<()>;

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()>;

    async fn remove_region(&mut self, scene: &SceneContext) -> Result<()> {
        let _ = scene;
        Ok(())
    }

    async fn region_loaded(&mut self, scene: &SceneContext) -> Result<()> {
        let _ = scene;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[async_trait]
pub trait SharedRegionModule: RegionModule {
    async fn post_initialize(&mut self, scene: &SceneContext) -> Result<()> {
        let _ = scene;
        Ok(())
    }
}

pub trait NonSharedRegionModule: RegionModule {}
