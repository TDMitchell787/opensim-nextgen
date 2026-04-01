use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use tracing::info;

use super::events::EventBus;
use super::services::ServiceRegistry;
use super::traits::{ModuleConfig, SceneContext, SharedRegionModule};

struct RegisteredModule {
    module: Box<dyn SharedRegionModule>,
    config: ModuleConfig,
    initialized: bool,
    added_to_region: bool,
}

pub struct ModuleRegistry {
    modules: Vec<RegisteredModule>,
    event_bus: Arc<EventBus>,
    service_registry: Arc<RwLock<ServiceRegistry>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            event_bus: Arc::new(EventBus::new()),
            service_registry: Arc::new(RwLock::new(ServiceRegistry::new())),
        }
    }

    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    pub fn service_registry(&self) -> Arc<RwLock<ServiceRegistry>> {
        self.service_registry.clone()
    }

    pub fn register_shared(
        &mut self,
        module: Box<dyn SharedRegionModule>,
        config: ModuleConfig,
    ) {
        let name = module.name();
        info!("[MODULES] Registered shared module: {}", name);
        self.modules.push(RegisteredModule {
            module,
            config,
            initialized: false,
            added_to_region: false,
        });
    }

    pub async fn initialize_all(&mut self) -> Result<()> {
        info!("[MODULES] Initializing {} modules...", self.modules.len());
        for entry in self.modules.iter_mut() {
            if !entry.initialized {
                let name = entry.module.name();
                info!("[MODULES]   Initializing: {}", name);
                entry.module.initialize(&entry.config).await?;
                entry.initialized = true;
            }
        }
        info!("[MODULES] All modules initialized");
        Ok(())
    }

    pub async fn add_region_all(&mut self, scene: &SceneContext) -> Result<()> {
        info!(
            "[MODULES] Adding region '{}' to {} modules...",
            scene.region_name,
            self.modules.len()
        );
        for entry in self.modules.iter_mut() {
            if entry.initialized && !entry.added_to_region {
                let name = entry.module.name();
                info!("[MODULES]   AddRegion: {}", name);
                entry.module.add_region(scene).await?;
                entry.added_to_region = true;
            }
        }
        Ok(())
    }

    pub async fn post_initialize_all(&mut self, scene: &SceneContext) -> Result<()> {
        info!("[MODULES] PostInitialize for {} modules...", self.modules.len());
        for entry in self.modules.iter_mut() {
            if entry.initialized && entry.added_to_region {
                let name = entry.module.name();
                info!("[MODULES]   PostInitialize: {}", name);
                entry.module.post_initialize(scene).await?;
            }
        }
        Ok(())
    }

    pub async fn region_loaded_all(&mut self, scene: &SceneContext) -> Result<()> {
        info!("[MODULES] RegionLoaded for {} modules...", self.modules.len());
        for entry in self.modules.iter_mut() {
            if entry.initialized && entry.added_to_region {
                let name = entry.module.name();
                info!("[MODULES]   RegionLoaded: {}", name);
                entry.module.region_loaded(scene).await?;
            }
        }
        info!("[MODULES] All modules loaded for region '{}'", scene.region_name);
        Ok(())
    }

    pub async fn remove_region_all(&mut self, scene: &SceneContext) -> Result<()> {
        info!("[MODULES] Removing region '{}' from modules...", scene.region_name);
        for entry in self.modules.iter_mut().rev() {
            if entry.added_to_region {
                let name = entry.module.name();
                info!("[MODULES]   RemoveRegion: {}", name);
                entry.module.remove_region(scene).await?;
                entry.added_to_region = false;
            }
        }
        Ok(())
    }

    pub async fn close_all(&mut self) -> Result<()> {
        info!("[MODULES] Closing {} modules...", self.modules.len());
        for entry in self.modules.iter_mut().rev() {
            if entry.initialized {
                let name = entry.module.name();
                info!("[MODULES]   Closing: {}", name);
                entry.module.close().await?;
                entry.initialized = false;
            }
        }
        info!("[MODULES] All modules closed");
        Ok(())
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn module_names(&self) -> Vec<&'static str> {
        self.modules.iter().map(|e| e.module.name()).collect()
    }

    pub fn get_service<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.service_registry.read().get::<T>()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
