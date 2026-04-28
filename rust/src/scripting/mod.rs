//! LSL (Linden Scripting Language) interpreter and execution environment

pub mod engine_manager;
pub mod executor;
pub mod listen_manager;
pub mod lsl_constants;
pub mod lsl_functions;
pub mod lsl_interpreter;
pub mod lsl_types;
pub mod ossl_functions;
pub mod persistence;
pub mod sandbox;
pub mod script_engine;
pub mod sensor_manager;
pub mod sl_functions;
pub mod state_machine;
pub mod timer_manager;
pub mod yengine_module;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    asset::AssetManager,
    region::{RegionId, RegionManager},
    state::StateManager,
};

pub use engine_manager::{
    EngineCompatibility, ScriptEngineConfig, ScriptEngineManager, ScriptEngineType,
};
pub use lsl_functions::*;
pub use lsl_interpreter::LSLInterpreter;
pub use lsl_types::*;
pub use sandbox::ScriptSandbox;
pub use script_engine::ScriptEngine;

/// Script execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptState {
    /// Script is not running
    Stopped,
    /// Script is running normally
    Running,
    /// Script is paused/suspended
    Suspended,
    /// Script encountered an error
    Error(String),
    /// Script is being compiled
    Compiling,
    /// Script compilation failed
    CompilationFailed(String),
}

/// Script metadata and information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub script_id: Uuid,
    pub name: String,
    pub source_code: String,
    pub owner_id: Uuid,
    pub object_id: Uuid,
    pub region_id: RegionId,
    pub state: ScriptState,
    pub created_at: u64,
    pub last_modified: u64,
    pub last_executed: Option<u64>,
    pub execution_count: u64,
    pub memory_usage: usize,
    pub max_memory: usize,
    pub execution_time_ms: f64,
    pub max_execution_time_ms: f64,
}

/// Script execution context
#[derive(Debug, Clone)]
pub struct ScriptContext {
    pub script_id: Uuid,
    pub object_id: Uuid,
    pub region_id: RegionId,
    pub owner_id: Uuid,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub velocity: (f32, f32, f32),
    pub variables: HashMap<String, LSLValue>,
    pub timers: HashMap<String, u64>,
    pub listeners: HashMap<String, LSLListener>,
    pub object_name: String,
    pub object_description: String,
    pub region_handle: u64,
    pub region_name: String,
    pub script_name: String,
    pub floating_text: Option<FloatingText>,
    pub inventory: Vec<InventoryItem>,
    pub pending_http_requests: HashMap<Uuid, PendingHttpRequest>,
    pub active_sensor: Option<SensorRequest>,
    pub detected_objects: Vec<DetectedObject>,
    pub permissions: u32,
    pub permission_key: Uuid,
    pub link_number: i32,
    pub linkset_data: HashMap<String, String>,
    pub script_start_time: std::time::Instant,
    pub start_parameter: i32,
    pub scale: (f32, f32, f32),
    pub sitting_avatar_id: Uuid,
    pub link_count: i32,
    pub link_names: Vec<(i32, String)>,
    pub link_scales: Vec<(i32, (f32, f32, f32))>,
    pub min_event_delay: f64,
    pub flags: u32,
    pub terrain_height: f32,
    pub base_mask: u32,
    pub owner_mask: u32,
    pub group_mask: u32,
    pub everyone_mask: u32,
    pub next_owner_mask: u32,
}

/// Floating text display above an object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingText {
    pub text: String,
    pub color: (f32, f32, f32),
    pub alpha: f32,
}

/// Inventory item in a script's object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub name: String,
    pub asset_id: Uuid,
    pub inv_type: i32,
    pub asset_type: i32,
    pub permissions: u32,
}

/// Pending HTTP request
#[derive(Debug, Clone)]
pub struct PendingHttpRequest {
    pub request_id: Uuid,
    pub url: String,
    pub method: String,
    pub mimetype: String,
    pub body: String,
    pub timestamp: std::time::SystemTime,
}

/// Sensor request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorRequest {
    pub name: Option<String>,
    pub id: Option<Uuid>,
    pub sensor_type: i32,
    pub range: f32,
    pub arc: f32,
    pub repeat_interval: Option<f32>,
}

/// LSL event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LSLEvent {
    StateEntry,
    StateExit,
    Touch(TouchEventData),
    Timer(String),
    Listen(ListenEventData),
    Collision(CollisionEventData),
    LandCollision(CollisionEventData),
    AtTarget(AtTargetEventData),
    NotAtTarget,
    AtRotTarget(AtRotTargetEventData),
    NotAtRotTarget,
    MoneyTransfer(MoneyEventData),
    Email(EmailEventData),
    HttpRequest(HttpRequestEventData),
    HttpResponse(HttpResponseEventData),
    RunTimePermissions(u32),
    Changed(u32),
    Attach(Uuid),
    Dataserver(DataserverEventData),
    MovingStart,
    MovingEnd,
    ObjectRez(Uuid),
    Remote(RemoteEventData),
    Control(ControlEventData),
    SensorDetect(Vec<DetectedObject>),
    NoSensor,
    LinkMessage(LinkMessageEventData),
}

/// Touch event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TouchEventData {
    pub toucher_id: Uuid,
    pub toucher_name: String,
    pub touch_position: (f32, f32, f32),
    pub touch_normal: (f32, f32, f32),
    pub touch_binormal: (f32, f32, f32),
    pub touch_face: i32,
    pub touch_uv: (f32, f32),
    pub touch_st: (f32, f32),
}

/// Listen event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListenEventData {
    pub channel: i32,
    pub name: String,
    pub id: Uuid,
    pub message: String,
}

/// Collision event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollisionEventData {
    pub detected_objects: Vec<DetectedObject>,
}

/// Detected object information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectedObject {
    pub key: Uuid,
    pub name: String,
    pub owner: Uuid,
    pub group: Uuid,
    pub position: (f32, f32, f32),
    pub velocity: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub link_number: i32,
    pub object_type: i32,
    pub touch_face: i32,
    pub touch_uv: (f32, f32),
    pub touch_st: (f32, f32),
    pub touch_normal: (f32, f32, f32),
    pub touch_binormal: (f32, f32, f32),
    pub touch_position: (f32, f32, f32),
}

/// At target event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtTargetEventData {
    pub target_number: i32,
    pub target_position: (f32, f32, f32),
    pub our_position: (f32, f32, f32),
}

/// At rotation target event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtRotTargetEventData {
    pub target_number: i32,
    pub target_rotation: (f32, f32, f32, f32),
    pub our_rotation: (f32, f32, f32, f32),
}

/// Money transfer event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoneyEventData {
    pub id: Uuid,
    pub amount: i32,
}

/// Email event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmailEventData {
    pub time: String,
    pub address: String,
    pub subject: String,
    pub message: String,
    pub num_left: i32,
}

/// HTTP request event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpRequestEventData {
    pub request_id: String,
    pub method: String,
    pub body: String,
}

/// HTTP response event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpResponseEventData {
    pub request_id: String,
    pub status: i32,
    pub metadata: Vec<String>,
    pub body: String,
}

/// Dataserver event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataserverEventData {
    pub query_id: String,
    pub data: String,
}

/// Remote event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteEventData {
    pub channel: i32,
    pub message_id: String,
    pub sender: String,
    pub idata: i32,
    pub sdata: String,
}

/// Control event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlEventData {
    pub id: Uuid,
    pub held: u32,
    pub change: u32,
}

/// Link message event data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkMessageEventData {
    pub sender_num: i32,
    pub num: i32,
    pub str: String,
    pub id: String,
}

/// LSL listener for chat/communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSLListener {
    pub handle: i32,
    pub channel: i32,
    pub name: String,
    pub id: Option<Uuid>,
    pub message: Option<String>,
    pub active: bool,
}

/// Script execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptStats {
    pub total_scripts: usize,
    pub running_scripts: usize,
    pub suspended_scripts: usize,
    pub error_scripts: usize,
    pub total_execution_time_ms: f64,
    pub average_execution_time_ms: f64,
    pub total_memory_usage: usize,
    pub peak_memory_usage: usize,
    pub events_processed: u64,
    pub functions_called: u64,
}

/// Scripting system manager
pub struct ScriptingManager {
    /// Script engine for execution
    script_engine: Arc<ScriptEngine>,
    /// Script engine manager for multi-engine support
    engine_manager: Arc<engine_manager::ScriptEngineManager>,
    /// Active scripts
    scripts: RwLock<HashMap<Uuid, ScriptInfo>>,
    /// Script contexts
    contexts: RwLock<HashMap<Uuid, ScriptContext>>,
    /// Event queue
    event_queue: RwLock<Vec<(Uuid, LSLEvent)>>,
    /// Statistics
    stats: RwLock<ScriptStats>,
    /// Region manager
    region_manager: Arc<RegionManager>,
    /// State manager
    state_manager: Arc<StateManager>,
    /// Asset manager
    asset_manager: Arc<AssetManager>,
    /// Event processing channel
    event_tx: mpsc::UnboundedSender<(Uuid, LSLEvent)>,
    event_rx: RwLock<Option<mpsc::UnboundedReceiver<(Uuid, LSLEvent)>>>,
}

impl ScriptingManager {
    pub fn new_with_engine(
        script_engine: Arc<ScriptEngine>,
        region_manager: Arc<RegionManager>,
        state_manager: Arc<StateManager>,
        asset_manager: Arc<AssetManager>,
    ) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let script_engines_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("bin")
            .join("ScriptEngines");

        let engine_config = engine_manager::ScriptEngineConfig::default();
        let engine_manager = Arc::new(engine_manager::ScriptEngineManager::new(
            engine_config,
            script_engines_path,
            script_engine.clone(),
        )?);

        Ok(Self {
            script_engine,
            engine_manager,
            scripts: RwLock::new(HashMap::new()),
            contexts: RwLock::new(HashMap::new()),
            event_queue: RwLock::new(Vec::new()),
            stats: RwLock::new(ScriptStats {
                total_scripts: 0,
                running_scripts: 0,
                suspended_scripts: 0,
                error_scripts: 0,
                total_execution_time_ms: 0.0,
                average_execution_time_ms: 0.0,
                total_memory_usage: 0,
                peak_memory_usage: 0,
                events_processed: 0,
                functions_called: 0,
            }),
            region_manager,
            state_manager,
            asset_manager,
            event_tx,
            event_rx: RwLock::new(Some(event_rx)),
        })
    }

    pub async fn new(
        region_manager: Arc<RegionManager>,
        state_manager: Arc<StateManager>,
        asset_manager: Arc<AssetManager>,
    ) -> Result<Self> {
        let script_engine = Arc::new(ScriptEngine::new().await?);
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Create script engine manager with ScriptEngines directory
        let script_engines_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("bin")
            .join("ScriptEngines");

        let engine_config = engine_manager::ScriptEngineConfig::default();
        let engine_manager = Arc::new(engine_manager::ScriptEngineManager::new(
            engine_config,
            script_engines_path,
            script_engine.clone(),
        )?);

        Ok(Self {
            script_engine,
            engine_manager,
            scripts: RwLock::new(HashMap::new()),
            contexts: RwLock::new(HashMap::new()),
            event_queue: RwLock::new(Vec::new()),
            stats: RwLock::new(ScriptStats {
                total_scripts: 0,
                running_scripts: 0,
                suspended_scripts: 0,
                error_scripts: 0,
                total_execution_time_ms: 0.0,
                average_execution_time_ms: 0.0,
                total_memory_usage: 0,
                peak_memory_usage: 0,
                events_processed: 0,
                functions_called: 0,
            }),
            region_manager,
            state_manager,
            asset_manager,
            event_tx,
            event_rx: RwLock::new(Some(event_rx)),
        })
    }

    /// Start the scripting system
    pub async fn start(&self) -> Result<()> {
        info!("Starting scripting system");

        // Initialize script engine manager
        self.engine_manager.initialize().await?;
        info!("Script engine manager initialized");

        // Take the receiver out of the option
        let mut event_rx = self
            .event_rx
            .write()
            .await
            .take()
            .ok_or_else(|| anyhow!("Scripting manager already started"))?;

        // Start event processing loop
        let manager = self.clone();
        tokio::spawn(async move {
            while let Some((script_id, event)) = event_rx.recv().await {
                if let Err(e) = manager.process_script_event(script_id, event).await {
                    error!("Error processing script event: {}", e);
                }
            }
        });

        // Start script maintenance loop
        let manager_for_maintenance = self.clone();
        tokio::spawn(async move {
            manager_for_maintenance.maintenance_loop().await;
        });

        info!("Scripting system started with multi-engine support");
        Ok(())
    }

    /// Compile and load a script
    pub async fn load_script(
        &self,
        name: String,
        source_code: String,
        owner_id: Uuid,
        object_id: Uuid,
        region_id: RegionId,
        position: (f32, f32, f32),
    ) -> Result<Uuid> {
        let script_id = Uuid::new_v4();
        info!("Loading script {} ({})", name, script_id);

        // Create script info
        let current_time = Self::current_timestamp();
        let mut script_info = ScriptInfo {
            script_id,
            name: name.clone(),
            source_code: source_code.clone(),
            owner_id,
            object_id,
            region_id,
            state: ScriptState::Compiling,
            created_at: current_time,
            last_modified: current_time,
            last_executed: None,
            execution_count: 0,
            memory_usage: 0,
            max_memory: 1024 * 1024, // 1MB default limit
            execution_time_ms: 0.0,
            max_execution_time_ms: 100.0, // 100ms default limit
        };

        // Create script context
        let context = ScriptContext {
            script_id,
            object_id,
            region_id,
            owner_id,
            position,
            rotation: (0.0, 0.0, 0.0, 1.0),
            velocity: (0.0, 0.0, 0.0),
            variables: HashMap::new(),
            timers: HashMap::new(),
            listeners: HashMap::new(),
            object_name: "Object".to_string(),
            object_description: String::new(),
            region_handle: 0,
            region_name: "Region".to_string(),
            script_name: name.clone(),
            floating_text: None,
            inventory: Vec::new(),
            pending_http_requests: HashMap::new(),
            active_sensor: None,
            detected_objects: Vec::new(),
            permissions: 0,
            permission_key: Uuid::nil(),
            link_number: 0,
            linkset_data: HashMap::new(),
            script_start_time: std::time::Instant::now(),
            start_parameter: 0,
            scale: (1.0, 1.0, 1.0),
            sitting_avatar_id: Uuid::nil(),
            link_count: 1,
            link_names: Vec::new(),
            link_scales: Vec::new(),
            min_event_delay: 0.0,
            flags: 0,
            terrain_height: 25.0,
            base_mask: 0x7FFFFFFF,
            owner_mask: 0x7FFFFFFF,
            group_mask: 0,
            everyone_mask: 0,
            next_owner_mask: 0x7FFFFFFF,
        };

        // Compile the script
        match self.script_engine.compile_script(&source_code).await {
            Ok(_) => {
                script_info.state = ScriptState::Stopped;
                info!("Script {} compiled successfully", name);
            }
            Err(e) => {
                script_info.state = ScriptState::CompilationFailed(e.to_string());
                error!("Script {} compilation failed: {}", name, e);
            }
        }

        // Store script and context
        self.scripts.write().await.insert(script_id, script_info);
        self.contexts.write().await.insert(script_id, context);

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_scripts += 1;
        }

        Ok(script_id)
    }

    /// Start a script
    pub async fn start_script(&self, script_id: Uuid) -> Result<()> {
        let mut scripts = self.scripts.write().await;
        let script = scripts
            .get_mut(&script_id)
            .ok_or_else(|| anyhow!("Script not found: {}", script_id))?;

        if script.state != ScriptState::Stopped && script.state != ScriptState::Suspended {
            return Err(anyhow!(
                "Script {} is not in a startable state: {:?}",
                script_id,
                script.state
            ));
        }

        script.state = ScriptState::Running;
        info!("Started script: {}", script.name);

        // Update statistics
        drop(scripts);
        self.update_script_stats().await;

        // Send state_entry event
        self.queue_event(script_id, LSLEvent::StateEntry).await?;

        Ok(())
    }

    /// Stop a script
    pub async fn stop_script(&self, script_id: Uuid) -> Result<()> {
        let mut scripts = self.scripts.write().await;
        let script = scripts
            .get_mut(&script_id)
            .ok_or_else(|| anyhow!("Script not found: {}", script_id))?;

        if script.state == ScriptState::Running {
            // Send state_exit event before stopping
            drop(scripts);
            self.queue_event(script_id, LSLEvent::StateExit).await?;

            let mut scripts_lock = self.scripts.write().await;
            let script = scripts_lock.get_mut(&script_id).unwrap();
            script.state = ScriptState::Stopped;
            info!("Stopped script: {}", script.name);
        }

        // Update statistics (no need to drop, lock will be released automatically)
        self.update_script_stats().await;

        Ok(())
    }

    /// Suspend a script
    pub async fn suspend_script(&self, script_id: Uuid) -> Result<()> {
        let mut scripts = self.scripts.write().await;
        let script = scripts
            .get_mut(&script_id)
            .ok_or_else(|| anyhow!("Script not found: {}", script_id))?;

        if script.state == ScriptState::Running {
            script.state = ScriptState::Suspended;
            info!("Suspended script: {}", script.name);
        }

        // Update statistics
        drop(scripts);
        self.update_script_stats().await;

        Ok(())
    }

    /// Remove a script
    pub async fn remove_script(&self, script_id: Uuid) -> Result<()> {
        // Stop the script first
        let _ = self.stop_script(script_id).await;

        // Remove from storage
        let script = self.scripts.write().await.remove(&script_id);
        self.contexts.write().await.remove(&script_id);

        if let Some(script) = script {
            info!("Removed script: {}", script.name);

            // Update statistics
            self.update_script_stats().await;
        }

        Ok(())
    }

    /// Queue an event for a script
    pub async fn queue_event(&self, script_id: Uuid, event: LSLEvent) -> Result<()> {
        debug!("Queuing event {:?} for script {}", event, script_id);

        if let Err(_) = self.event_tx.send((script_id, event)) {
            return Err(anyhow!("Failed to queue event - channel closed"));
        }

        Ok(())
    }

    /// Get script information
    pub async fn get_script(&self, script_id: Uuid) -> Option<ScriptInfo> {
        self.scripts.read().await.get(&script_id).cloned()
    }

    /// Get all scripts for an object
    pub async fn get_object_scripts(&self, object_id: Uuid) -> Vec<ScriptInfo> {
        self.scripts
            .read()
            .await
            .values()
            .filter(|script| script.object_id == object_id)
            .cloned()
            .collect()
    }

    /// Get scripting statistics
    pub async fn get_stats(&self) -> ScriptStats {
        self.stats.read().await.clone()
    }

    /// Get script engine manager for advanced engine operations
    pub fn get_engine_manager(&self) -> Arc<engine_manager::ScriptEngineManager> {
        self.engine_manager.clone()
    }

    /// Get available script engines
    pub async fn get_available_engines(&self) -> Vec<engine_manager::ScriptEngineType> {
        self.engine_manager.get_available_engines().await
    }

    /// Switch script engine
    pub async fn switch_engine(&self, engine_type: engine_manager::ScriptEngineType) -> Result<()> {
        self.engine_manager.switch_engine(engine_type).await
    }

    /// Get engine compatibility information
    pub async fn get_engine_compatibility(
        &self,
        engine_type: &engine_manager::ScriptEngineType,
    ) -> Option<engine_manager::EngineCompatibility> {
        self.engine_manager
            .get_engine_compatibility(engine_type)
            .await
    }

    /// Get combined engine statistics
    pub async fn get_engine_stats(
        &self,
    ) -> Result<HashMap<engine_manager::ScriptEngineType, engine_manager::EngineStats>> {
        self.engine_manager.get_combined_stats().await
    }

    /// Get current engine configuration
    pub async fn get_engine_config(&self) -> engine_manager::ScriptEngineConfig {
        self.engine_manager.get_config().await
    }

    /// Process a script event
    async fn process_script_event(&self, script_id: Uuid, event: LSLEvent) -> Result<()> {
        debug!("Processing event {:?} for script {}", event, script_id);

        // Check if script exists and is running
        let script_info = {
            let scripts = self.scripts.read().await;
            match scripts.get(&script_id) {
                Some(script) if script.state == ScriptState::Running => script.clone(),
                Some(script) => {
                    debug!(
                        "Skipping event for script {} in state {:?}",
                        script_id, script.state
                    );
                    return Ok(());
                }
                None => {
                    warn!("Event for unknown script: {}", script_id);
                    return Ok(());
                }
            }
        };

        // Get script context
        let context = self
            .contexts
            .read()
            .await
            .get(&script_id)
            .cloned()
            .ok_or_else(|| anyhow!("Script context not found: {}", script_id))?;

        // Execute the event in the script engine
        let start_time = std::time::Instant::now();
        match self
            .script_engine
            .execute_event(script_id, &event, &context)
            .await
        {
            Ok(_) => {
                let execution_time = start_time.elapsed().as_millis() as f64;
                debug!("Event executed successfully in {}ms", execution_time);

                // Update script statistics
                {
                    let mut scripts = self.scripts.write().await;
                    if let Some(script) = scripts.get_mut(&script_id) {
                        script.execution_count += 1;
                        script.execution_time_ms += execution_time;
                        script.last_executed = Some(Self::current_timestamp());
                    }
                }

                // Update global statistics
                {
                    let mut stats = self.stats.write().await;
                    stats.events_processed += 1;
                    stats.total_execution_time_ms += execution_time;
                    stats.average_execution_time_ms =
                        stats.total_execution_time_ms / stats.events_processed as f64;
                }
            }
            Err(e) => {
                error!("Script {} event execution failed: {}", script_info.name, e);

                // Mark script as error state
                {
                    let mut scripts = self.scripts.write().await;
                    if let Some(script) = scripts.get_mut(&script_id) {
                        script.state = ScriptState::Error(e.to_string());
                    }
                }

                self.update_script_stats().await;
            }
        }

        Ok(())
    }

    /// Update script statistics
    async fn update_script_stats(&self) {
        let scripts = self.scripts.read().await;
        let mut stats = self.stats.write().await;

        stats.total_scripts = scripts.len();
        stats.running_scripts = scripts
            .values()
            .filter(|s| s.state == ScriptState::Running)
            .count();
        stats.suspended_scripts = scripts
            .values()
            .filter(|s| s.state == ScriptState::Suspended)
            .count();
        stats.error_scripts = scripts
            .values()
            .filter(|s| matches!(s.state, ScriptState::Error(_)))
            .count();
        stats.total_memory_usage = scripts.values().map(|s| s.memory_usage).sum();
        stats.peak_memory_usage = stats.peak_memory_usage.max(stats.total_memory_usage);
    }

    /// Maintenance loop for scripts
    async fn maintenance_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            // Update statistics
            self.update_script_stats().await;

            // Process any cleanup tasks
            self.cleanup_expired_timers().await;
            self.cleanup_inactive_listeners().await;

            debug!("Script maintenance completed");
        }
    }

    /// Clean up expired timers
    async fn cleanup_expired_timers(&self) {
        let current_time = Self::current_timestamp();
        let mut contexts = self.contexts.write().await;

        for (script_id, context) in contexts.iter_mut() {
            let expired_timers: Vec<String> = context
                .timers
                .iter()
                .filter(|(_, &expiry)| expiry <= current_time)
                .map(|(name, _)| name.clone())
                .collect();

            for timer_name in expired_timers {
                context.timers.remove(&timer_name);

                // Queue timer event
                if let Err(e) = self
                    .event_tx
                    .send((*script_id, LSLEvent::Timer(timer_name)))
                {
                    error!("Failed to queue timer event: {}", e);
                }
            }
        }
    }

    /// Clean up inactive listeners
    async fn cleanup_inactive_listeners(&self) {
        let mut contexts = self.contexts.write().await;

        for context in contexts.values_mut() {
            context.listeners.retain(|_, listener| listener.active);
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

// Implement Clone for ScriptingManager
impl Clone for ScriptingManager {
    fn clone(&self) -> Self {
        let (event_tx, _) = mpsc::unbounded_channel();

        Self {
            script_engine: self.script_engine.clone(),
            engine_manager: self.engine_manager.clone(),
            scripts: RwLock::new(HashMap::new()),
            contexts: RwLock::new(HashMap::new()),
            event_queue: RwLock::new(Vec::new()),
            stats: RwLock::new(ScriptStats {
                total_scripts: 0,
                running_scripts: 0,
                suspended_scripts: 0,
                error_scripts: 0,
                total_execution_time_ms: 0.0,
                average_execution_time_ms: 0.0,
                total_memory_usage: 0,
                peak_memory_usage: 0,
                events_processed: 0,
                functions_called: 0,
            }),
            region_manager: self.region_manager.clone(),
            state_manager: self.state_manager.clone(),
            asset_manager: self.asset_manager.clone(),
            event_tx,
            event_rx: RwLock::new(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ffi::physics::PhysicsBridge,
        region::{terrain::TerrainConfig, RegionConfig},
    };

    #[tokio::test]
    async fn test_scripting_manager_creation() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        // Create test dependencies for AssetManager
        use crate::asset::{
            cache::{AssetCache, CacheConfig},
            cdn::{CdnConfig, CdnManager},
            storage::StorageBackend,
            AssetManagerConfig,
        };
        use crate::database::DatabaseManager;

        let database = Arc::new(DatabaseManager::new("sqlite://test_scripting.db").await?);
        let cache_config = CacheConfig {
            memory_cache_size: 100,
            memory_ttl_seconds: 3600,
            redis_ttl_seconds: 7200,
            redis_url: None,
            enable_compression: false,
            compression_threshold: 1024,
        };
        let cache = Arc::new(AssetCache::new(cache_config).await?);
        use crate::asset::cdn::CdnProvider;
        use std::collections::HashMap;
        let cdn_config = CdnConfig {
            provider: CdnProvider::Generic,
            base_url: "http://localhost:8080".to_string(),
            api_key: None,
            provider_config: HashMap::new(),
            default_ttl: 3600,
            auto_distribute: false,
            regions: vec![],
        };
        let cdn = Arc::new(CdnManager::new(cdn_config).await?);
        let storage: Arc<dyn StorageBackend> =
            Arc::new(crate::asset::storage::FileSystemStorage::new(
                std::path::PathBuf::from("./test_assets"),
            )?);
        let config = AssetManagerConfig::default();

        let asset_manager =
            Arc::new(AssetManager::new(database, cache, cdn, storage, config).await?);

        let scripting_manager =
            ScriptingManager::new(region_manager, state_manager, asset_manager).await?;

        let stats = scripting_manager.get_stats().await;
        assert_eq!(stats.total_scripts, 0);
        assert_eq!(stats.running_scripts, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_script_loading() -> Result<()> {
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        // Create test dependencies for AssetManager
        use crate::asset::{
            cache::{AssetCache, CacheConfig},
            cdn::{CdnConfig, CdnManager},
            storage::StorageBackend,
            AssetManagerConfig,
        };
        use crate::database::DatabaseManager;

        let database = Arc::new(DatabaseManager::new("sqlite://test_scripting.db").await?);
        let cache_config = CacheConfig {
            memory_cache_size: 100,
            memory_ttl_seconds: 3600,
            redis_ttl_seconds: 7200,
            redis_url: None,
            enable_compression: false,
            compression_threshold: 1024,
        };
        let cache = Arc::new(AssetCache::new(cache_config).await?);
        use crate::asset::cdn::CdnProvider;
        use std::collections::HashMap;
        let cdn_config = CdnConfig {
            provider: CdnProvider::Generic,
            base_url: "http://localhost:8080".to_string(),
            api_key: None,
            provider_config: HashMap::new(),
            default_ttl: 3600,
            auto_distribute: false,
            regions: vec![],
        };
        let cdn = Arc::new(CdnManager::new(cdn_config).await?);
        let storage: Arc<dyn StorageBackend> =
            Arc::new(crate::asset::storage::FileSystemStorage::new(
                std::path::PathBuf::from("./test_assets"),
            )?);
        let config = AssetManagerConfig::default();

        let asset_manager =
            Arc::new(AssetManager::new(database, cache, cdn, storage, config).await?);

        let scripting_manager =
            ScriptingManager::new(region_manager, state_manager, asset_manager).await?;

        let script_source = r#"
            default {
                state_entry() {
                    llSay(0, "Hello, world!");
                }
            }
        "#;

        let script_id = scripting_manager
            .load_script(
                "test_script".to_string(),
                script_source.to_string(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                crate::region::RegionId(1),
                (128.0, 128.0, 21.0),
            )
            .await?;

        let script = scripting_manager.get_script(script_id).await;
        assert!(script.is_some());

        let script = script.unwrap();
        assert_eq!(script.name, "test_script");
        assert!(matches!(
            script.state,
            ScriptState::Stopped | ScriptState::CompilationFailed(_)
        ));

        Ok(())
    }
}
