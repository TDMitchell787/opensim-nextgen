//! Script Engine Manager for XEngine/YEngine compatibility
//!
//! This module provides a compatibility layer that allows OpenSim Next to work
//! with existing XEngine and YEngine compiled scripts while also supporting
//! the native Rust LSL implementation.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::{LSLEvent, ScriptContext, ScriptInfo, ScriptState};
use crate::opensim_compatibility::modules::{LoadedModule, ModuleLoader, ModuleType};

/// Supported script engine types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ScriptEngineType {
    /// Native Rust LSL implementation (default, best performance)
    Native,
    /// XEngine compatibility (legacy, deprecated in OpenSim)
    XEngine,
    /// YEngine compatibility (current OpenSim default)
    YEngine,
    /// External engine loaded via addon module
    External(String),
}

/// Script engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptEngineConfig {
    pub engine_type: ScriptEngineType,
    pub max_scripts_per_region: usize,
    pub max_execution_time_ms: f64,
    pub max_memory_per_script: usize,
    pub enable_debugging: bool,
    pub script_timeout_seconds: u64,
    pub compile_timeout_seconds: u64,
    pub thread_pool_size: usize,
    pub enable_http_requests: bool,
    pub enable_email: bool,
    pub enable_sensor: bool,
    pub enable_dataserver: bool,
    pub xengine_dll_path: Option<PathBuf>,
    pub yengine_dll_path: Option<PathBuf>,
}

impl Default for ScriptEngineConfig {
    fn default() -> Self {
        Self {
            engine_type: ScriptEngineType::Native,
            max_scripts_per_region: 1000,
            max_execution_time_ms: 100.0,
            max_memory_per_script: 1024 * 1024, // 1MB
            enable_debugging: true,
            script_timeout_seconds: 30,
            compile_timeout_seconds: 60,
            thread_pool_size: 4,
            enable_http_requests: true,
            enable_email: true,
            enable_sensor: true,
            enable_dataserver: true,
            xengine_dll_path: None,
            yengine_dll_path: None,
        }
    }
}

/// Script engine compatibility information
#[derive(Debug, Clone)]
pub struct EngineCompatibility {
    pub supports_compilation: bool,
    pub supports_runtime_switching: bool,
    pub supports_state_migration: bool,
    pub max_script_size: usize,
    pub supported_lsl_version: String,
    pub performance_rating: u8, // 1-10, higher is better
}

/// Script engine manager handles multiple engine types
pub struct ScriptEngineManager {
    /// Current engine configuration
    config: RwLock<ScriptEngineConfig>,
    /// Module loader for external engines
    module_loader: Arc<ModuleLoader>,
    /// Available engines
    available_engines: RwLock<HashMap<ScriptEngineType, EngineCompatibility>>,
    /// Active scripts by engine type
    active_scripts: RwLock<HashMap<ScriptEngineType, Vec<Uuid>>>,
    /// Script engine instances
    engine_instances: RwLock<HashMap<ScriptEngineType, Box<dyn ScriptEngineInstance>>>,
    /// Native script engine
    native_engine: Arc<super::ScriptEngine>,
}

/// Trait for script engine implementations
#[async_trait::async_trait]
pub trait ScriptEngineInstance: Send + Sync {
    /// Compile a script
    async fn compile_script(&self, source: &str, script_id: Uuid) -> Result<Vec<u8>>;

    /// Execute an event in a script
    async fn execute_event(
        &self,
        script_id: Uuid,
        event: &LSLEvent,
        context: &ScriptContext,
    ) -> Result<()>;

    /// Load compiled script
    async fn load_compiled_script(&self, script_id: Uuid, bytecode: &[u8]) -> Result<()>;

    /// Unload script
    async fn unload_script(&self, script_id: Uuid) -> Result<()>;

    /// Get engine statistics
    async fn get_stats(&self) -> Result<EngineStats>;

    /// Check if engine is healthy
    async fn health_check(&self) -> Result<bool>;
}

/// Engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub scripts_loaded: usize,
    pub scripts_running: usize,
    pub total_executions: u64,
    pub total_compilation_time_ms: f64,
    pub total_execution_time_ms: f64,
    pub memory_usage_bytes: usize,
    pub errors_count: u64,
}

impl ScriptEngineManager {
    /// Create a new script engine manager
    pub fn new(
        config: ScriptEngineConfig,
        script_engines_path: PathBuf,
        native_engine: Arc<super::ScriptEngine>,
    ) -> Result<Self> {
        let addon_modules_path = script_engines_path.join("addon-modules");
        let module_loader = Arc::new(ModuleLoader::new(addon_modules_path)?);

        Ok(Self {
            config: RwLock::new(config),
            module_loader,
            available_engines: RwLock::new(HashMap::new()),
            active_scripts: RwLock::new(HashMap::new()),
            engine_instances: RwLock::new(HashMap::new()),
            native_engine,
        })
    }

    /// Initialize the script engine manager
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing script engine manager");

        // Register native engine
        self.register_native_engine().await?;

        // Scan for external engines
        self.scan_external_engines().await?;

        // Initialize XEngine/YEngine compatibility if available
        self.initialize_legacy_engines().await?;

        info!("Script engine manager initialized successfully");
        Ok(())
    }

    /// Register the native Rust LSL engine
    async fn register_native_engine(&self) -> Result<()> {
        let compatibility = EngineCompatibility {
            supports_compilation: true,
            supports_runtime_switching: true,
            supports_state_migration: true,
            max_script_size: 1024 * 1024, // 1MB
            supported_lsl_version: "2.0".to_string(),
            performance_rating: 10, // Best performance
        };

        self.available_engines
            .write()
            .await
            .insert(ScriptEngineType::Native, compatibility);

        // Create native engine instance wrapper
        let native_instance = NativeEngineWrapper::new(self.native_engine.clone());
        self.engine_instances
            .write()
            .await
            .insert(ScriptEngineType::Native, Box::new(native_instance));

        info!("Native Rust LSL engine registered");
        Ok(())
    }

    /// Scan for external script engines
    async fn scan_external_engines(&self) -> Result<()> {
        // Get script engines first, then scan
        let script_engines = self
            .module_loader
            .get_modules_by_type(ModuleType::ScriptEngine);
        let script_engines_count = script_engines.len();

        for engine_module in script_engines {
            self.register_external_engine(engine_module).await?;
        }

        info!("Scanned {} external script engines", script_engines_count);
        Ok(())
    }

    /// Register an external script engine
    async fn register_external_engine(&self, module: &LoadedModule) -> Result<()> {
        let engine_type = ScriptEngineType::External(module.name.clone());

        let compatibility = EngineCompatibility {
            supports_compilation: true,
            supports_runtime_switching: false, // Depends on implementation
            supports_state_migration: false,   // Requires custom implementation
            max_script_size: 512 * 1024,       // 512KB typical
            supported_lsl_version: "1.0".to_string(),
            performance_rating: 5, // Medium performance
        };

        self.available_engines
            .write()
            .await
            .insert(engine_type, compatibility);

        info!("Registered external script engine: {}", module.name);
        Ok(())
    }

    /// Initialize legacy XEngine/YEngine compatibility
    async fn initialize_legacy_engines(&self) -> Result<()> {
        let config = self.config.read().await;

        // Initialize XEngine compatibility if available
        if let Some(xengine_path) = &config.xengine_dll_path {
            if xengine_path.exists() {
                self.initialize_xengine_compatibility(xengine_path).await?;
            }
        }

        // Initialize YEngine compatibility if available
        if let Some(yengine_path) = &config.yengine_dll_path {
            if yengine_path.exists() {
                self.initialize_yengine_compatibility(yengine_path).await?;
            }
        }

        Ok(())
    }

    /// Initialize XEngine compatibility layer
    async fn initialize_xengine_compatibility(&self, dll_path: &PathBuf) -> Result<()> {
        info!(
            "Initializing XEngine compatibility with DLL: {}",
            dll_path.display()
        );

        let compatibility = EngineCompatibility {
            supports_compilation: true,
            supports_runtime_switching: false,
            supports_state_migration: true,
            max_script_size: 256 * 1024, // 256KB
            supported_lsl_version: "1.0".to_string(),
            performance_rating: 3, // Lower performance (legacy)
        };

        self.available_engines
            .write()
            .await
            .insert(ScriptEngineType::XEngine, compatibility);

        // Create XEngine wrapper instance
        let xengine_instance = XEngineWrapper::new(dll_path.clone())?;
        self.engine_instances
            .write()
            .await
            .insert(ScriptEngineType::XEngine, Box::new(xengine_instance));

        info!("XEngine compatibility initialized");
        Ok(())
    }

    /// Initialize YEngine compatibility layer
    async fn initialize_yengine_compatibility(&self, dll_path: &PathBuf) -> Result<()> {
        info!(
            "Initializing YEngine compatibility with DLL: {}",
            dll_path.display()
        );

        let compatibility = EngineCompatibility {
            supports_compilation: true,
            supports_runtime_switching: false,
            supports_state_migration: true,
            max_script_size: 512 * 1024, // 512KB
            supported_lsl_version: "1.0".to_string(),
            performance_rating: 6, // Medium performance
        };

        self.available_engines
            .write()
            .await
            .insert(ScriptEngineType::YEngine, compatibility);

        // Create YEngine wrapper instance
        let yengine_instance = YEngineWrapper::new(dll_path.clone())?;
        self.engine_instances
            .write()
            .await
            .insert(ScriptEngineType::YEngine, Box::new(yengine_instance));

        info!("YEngine compatibility initialized");
        Ok(())
    }

    /// Get available engine types
    pub async fn get_available_engines(&self) -> Vec<ScriptEngineType> {
        self.available_engines
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }

    /// Get engine compatibility information
    pub async fn get_engine_compatibility(
        &self,
        engine_type: &ScriptEngineType,
    ) -> Option<EngineCompatibility> {
        self.available_engines
            .read()
            .await
            .get(engine_type)
            .cloned()
    }

    /// Switch script engine for a region
    pub async fn switch_engine(&self, new_engine_type: ScriptEngineType) -> Result<()> {
        let mut config = self.config.write().await;
        let old_engine_type = config.engine_type.clone();

        if old_engine_type == new_engine_type {
            return Ok(()); // Already using this engine
        }

        info!(
            "Switching script engine from {:?} to {:?}",
            old_engine_type, new_engine_type
        );

        // Check if target engine is available
        if !self
            .available_engines
            .read()
            .await
            .contains_key(&new_engine_type)
        {
            return Err(anyhow!(
                "Script engine {:?} is not available",
                new_engine_type
            ));
        }

        // Migrate active scripts if supported
        self.migrate_scripts(&old_engine_type, &new_engine_type)
            .await?;

        // Update configuration
        config.engine_type = new_engine_type;

        info!("Script engine switch completed successfully");
        Ok(())
    }

    /// Migrate scripts between engines
    async fn migrate_scripts(
        &self,
        from_engine: &ScriptEngineType,
        to_engine: &ScriptEngineType,
    ) -> Result<()> {
        let active_scripts = self.active_scripts.read().await;

        if let Some(script_ids) = active_scripts.get(from_engine) {
            info!(
                "Migrating {} scripts from {:?} to {:?}",
                script_ids.len(),
                from_engine,
                to_engine
            );

            // For now, just log the migration
            // In a full implementation, you would:
            // 1. Save script states from old engine
            // 2. Load scripts into new engine
            // 3. Restore script states

            for script_id in script_ids {
                debug!("Migrating script {}", script_id);
                // Migration logic would go here
            }
        }

        Ok(())
    }

    /// Compile a script using the configured engine
    pub async fn compile_script(&self, source: &str, script_id: Uuid) -> Result<Vec<u8>> {
        let config = self.config.read().await;
        let engine_type = config.engine_type.clone();
        drop(config);

        let engines = self.engine_instances.read().await;
        if let Some(engine) = engines.get(&engine_type) {
            engine.compile_script(source, script_id).await
        } else {
            Err(anyhow!("Script engine {:?} is not available", engine_type))
        }
    }

    /// Execute an event using the configured engine
    pub async fn execute_event(
        &self,
        script_id: Uuid,
        event: &LSLEvent,
        context: &ScriptContext,
    ) -> Result<()> {
        let config = self.config.read().await;
        let engine_type = config.engine_type.clone();
        drop(config);

        let engines = self.engine_instances.read().await;
        if let Some(engine) = engines.get(&engine_type) {
            engine.execute_event(script_id, event, context).await
        } else {
            Err(anyhow!("Script engine {:?} is not available", engine_type))
        }
    }

    /// Get combined statistics from all engines
    pub async fn get_combined_stats(&self) -> Result<HashMap<ScriptEngineType, EngineStats>> {
        let mut stats = HashMap::new();
        let engines = self.engine_instances.read().await;

        for (engine_type, engine) in engines.iter() {
            if let Ok(engine_stats) = engine.get_stats().await {
                stats.insert(engine_type.clone(), engine_stats);
            }
        }

        Ok(stats)
    }

    /// Get current engine configuration
    pub async fn get_config(&self) -> ScriptEngineConfig {
        self.config.read().await.clone()
    }

    /// Update engine configuration
    pub async fn update_config(&self, new_config: ScriptEngineConfig) -> Result<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }
}

/// Native engine wrapper implementing ScriptEngineInstance
struct NativeEngineWrapper {
    engine: Arc<super::ScriptEngine>,
}

impl NativeEngineWrapper {
    fn new(engine: Arc<super::ScriptEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait::async_trait]
impl ScriptEngineInstance for NativeEngineWrapper {
    async fn compile_script(&self, source: &str, _script_id: Uuid) -> Result<Vec<u8>> {
        // Use native engine compilation
        let _compiled_id = self.engine.compile_script(source).await?;

        // Return the source as bytecode for now (in real implementation,
        // this would be actual bytecode)
        Ok(source.as_bytes().to_vec())
    }

    async fn execute_event(
        &self,
        script_id: Uuid,
        event: &LSLEvent,
        context: &ScriptContext,
    ) -> Result<()> {
        self.engine.execute_event(script_id, event, context).await
    }

    async fn load_compiled_script(&self, _script_id: Uuid, _bytecode: &[u8]) -> Result<()> {
        // Native engine handles this internally
        Ok(())
    }

    async fn unload_script(&self, _script_id: Uuid) -> Result<()> {
        // Native engine handles this internally
        Ok(())
    }

    async fn get_stats(&self) -> Result<EngineStats> {
        let native_stats = self.engine.get_stats().await;

        Ok(EngineStats {
            scripts_loaded: 0,  // Would need to track this
            scripts_running: 0, // Would need to track this
            total_executions: native_stats.events_executed,
            total_compilation_time_ms: 0.0, // Not tracked in native stats
            total_execution_time_ms: native_stats.total_execution_time_ms,
            memory_usage_bytes: native_stats.peak_memory_usage,
            errors_count: native_stats.execution_errors,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        // Native engine is always healthy if it exists
        Ok(true)
    }
}

/// XEngine compatibility wrapper
struct XEngineWrapper {
    dll_path: PathBuf,
}

impl XEngineWrapper {
    fn new(dll_path: PathBuf) -> Result<Self> {
        // In a real implementation, you would load the XEngine DLL here
        // and set up the FFI bindings

        info!("XEngine wrapper created for DLL: {}", dll_path.display());
        Ok(Self { dll_path })
    }
}

#[async_trait::async_trait]
impl ScriptEngineInstance for XEngineWrapper {
    async fn compile_script(&self, source: &str, script_id: Uuid) -> Result<Vec<u8>> {
        // XEngine compilation would happen here via FFI
        info!(
            "XEngine compiling script {} from DLL: {}",
            script_id,
            self.dll_path.display()
        );

        // For now, return source as bytecode
        // In real implementation: call XEngine compilation functions
        Ok(source.as_bytes().to_vec())
    }

    async fn execute_event(
        &self,
        script_id: Uuid,
        event: &LSLEvent,
        context: &ScriptContext,
    ) -> Result<()> {
        // XEngine event execution would happen here via FFI
        debug!(
            "XEngine executing event {:?} for script {}",
            event, script_id
        );

        // For now, just log
        // In real implementation: call XEngine execution functions
        Ok(())
    }

    async fn load_compiled_script(&self, script_id: Uuid, bytecode: &[u8]) -> Result<()> {
        info!(
            "XEngine loading compiled script {} ({} bytes)",
            script_id,
            bytecode.len()
        );
        Ok(())
    }

    async fn unload_script(&self, script_id: Uuid) -> Result<()> {
        info!("XEngine unloading script {}", script_id);
        Ok(())
    }

    async fn get_stats(&self) -> Result<EngineStats> {
        Ok(EngineStats {
            scripts_loaded: 0,
            scripts_running: 0,
            total_executions: 0,
            total_compilation_time_ms: 0.0,
            total_execution_time_ms: 0.0,
            memory_usage_bytes: 0,
            errors_count: 0,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if XEngine DLL is still accessible
        Ok(self.dll_path.exists())
    }
}

/// YEngine compatibility wrapper
struct YEngineWrapper {
    dll_path: PathBuf,
}

impl YEngineWrapper {
    fn new(dll_path: PathBuf) -> Result<Self> {
        // In a real implementation, you would load the YEngine DLL here
        // and set up the FFI bindings

        info!("YEngine wrapper created for DLL: {}", dll_path.display());
        Ok(Self { dll_path })
    }
}

#[async_trait::async_trait]
impl ScriptEngineInstance for YEngineWrapper {
    async fn compile_script(&self, source: &str, script_id: Uuid) -> Result<Vec<u8>> {
        // YEngine compilation would happen here via FFI
        info!(
            "YEngine compiling script {} from DLL: {}",
            script_id,
            self.dll_path.display()
        );

        // For now, return source as bytecode
        // In real implementation: call YEngine compilation functions
        Ok(source.as_bytes().to_vec())
    }

    async fn execute_event(
        &self,
        script_id: Uuid,
        event: &LSLEvent,
        context: &ScriptContext,
    ) -> Result<()> {
        // YEngine event execution would happen here via FFI
        debug!(
            "YEngine executing event {:?} for script {}",
            event, script_id
        );

        // For now, just log
        // In real implementation: call YEngine execution functions
        Ok(())
    }

    async fn load_compiled_script(&self, script_id: Uuid, bytecode: &[u8]) -> Result<()> {
        info!(
            "YEngine loading compiled script {} ({} bytes)",
            script_id,
            bytecode.len()
        );
        Ok(())
    }

    async fn unload_script(&self, script_id: Uuid) -> Result<()> {
        info!("YEngine unloading script {}", script_id);
        Ok(())
    }

    async fn get_stats(&self) -> Result<EngineStats> {
        Ok(EngineStats {
            scripts_loaded: 0,
            scripts_running: 0,
            total_executions: 0,
            total_compilation_time_ms: 0.0,
            total_execution_time_ms: 0.0,
            memory_usage_bytes: 0,
            errors_count: 0,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if YEngine DLL is still accessible
        Ok(self.dll_path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn test_engine_manager_creation() -> Result<()> {
        let config = ScriptEngineConfig::default();
        let script_engines_path = PathBuf::from("./test_script_engines");
        let native_engine = Arc::new(super::super::ScriptEngine::new().await?);

        let manager = ScriptEngineManager::new(config, script_engines_path, native_engine)?;
        assert!(manager.get_available_engines().await.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_native_engine_registration() -> Result<()> {
        let config = ScriptEngineConfig::default();
        let script_engines_path = PathBuf::from("./test_script_engines");
        let native_engine = Arc::new(super::super::ScriptEngine::new().await?);

        let manager = ScriptEngineManager::new(config, script_engines_path, native_engine)?;
        manager.register_native_engine().await?;

        let engines = manager.get_available_engines().await;
        assert!(engines.contains(&ScriptEngineType::Native));

        Ok(())
    }
}
