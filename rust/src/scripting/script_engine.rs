//! Script execution engine for LSL scripts

use std::{collections::HashMap, sync::Arc};
use anyhow::{anyhow, Result};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use super::{LSLValue, LSLEvent, ScriptContext, lsl_functions::LSLFunctions};
use crate::{
    region::RegionManager,
    asset::AssetManager,
    network::grid_events::GridEventManager,
    scripting::executor::ScriptAction,
};

/// Compiled script representation
#[derive(Debug, Clone)]
pub struct CompiledScript {
    pub script_id: Uuid,
    pub bytecode: Vec<u8>,
    pub functions: HashMap<String, usize>,
    pub states: HashMap<String, usize>,
    pub events: HashMap<String, Vec<String>>, // state -> events
    pub global_variables: HashMap<String, LSLValue>,
}

/// Script execution environment
pub struct ScriptEngine {
    /// LSL function executor
    lsl_functions: Arc<LSLFunctions>,
    /// Compiled scripts cache
    compiled_scripts: Arc<tokio::sync::RwLock<HashMap<Uuid, CompiledScript>>>,
    /// Execution statistics
    execution_stats: Arc<tokio::sync::RwLock<ExecutionStats>>,
}

/// Script execution statistics
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub scripts_compiled: u64,
    pub compilation_errors: u64,
    pub events_executed: u64,
    pub execution_errors: u64,
    pub total_execution_time_ms: f64,
    pub peak_memory_usage: usize,
}

impl ScriptEngine {
    pub async fn new() -> Result<Self> {
        let region_manager = Arc::new(crate::region::RegionManager::new(
            Arc::new(crate::ffi::physics::PhysicsBridge::new()?),
            Arc::new(crate::state::StateManager::new()?),
        ));
        let database = Arc::new(crate::database::DatabaseManager::new("sqlite::memory:").await?);
        let cache_config = crate::asset::cache::CacheConfig::default();
        let cache = Arc::new(crate::asset::cache::AssetCache::new(cache_config).await?);
        let cdn = Arc::new(crate::asset::cdn::CdnManager::new(Default::default()).await?);
        let storage: Arc<dyn crate::asset::storage::StorageBackend> = Arc::new(crate::asset::storage::FileSystemStorage::new("./assets".into())?);
        let config = crate::asset::AssetManagerConfig::default();
        let asset_manager = Arc::new(crate::asset::AssetManager::new(database, cache, cdn, storage, config).await?);

        let action_queue: Arc<parking_lot::Mutex<Vec<(Uuid, ScriptAction)>>> =
            Arc::new(parking_lot::Mutex::new(Vec::new()));
        let lsl_functions = Arc::new(LSLFunctions::new(
            region_manager,
            asset_manager,
            None,
            action_queue,
        ));

        Ok(Self {
            lsl_functions,
            compiled_scripts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            execution_stats: Arc::new(tokio::sync::RwLock::new(ExecutionStats::default())),
        })
    }

    pub fn new_with_managers(
        region_manager: Arc<RegionManager>,
        asset_manager: Arc<AssetManager>,
        grid_event_manager: Option<Arc<GridEventManager>>,
    ) -> Self {
        let action_queue: Arc<parking_lot::Mutex<Vec<(Uuid, ScriptAction)>>> =
            Arc::new(parking_lot::Mutex::new(Vec::new()));
        let lsl_functions = Arc::new(LSLFunctions::new(
            region_manager,
            asset_manager,
            grid_event_manager,
            action_queue,
        ));

        Self {
            lsl_functions,
            compiled_scripts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            execution_stats: Arc::new(tokio::sync::RwLock::new(ExecutionStats::default())),
        }
    }

    /// Compile an LSL script
    pub async fn compile_script(&self, source_code: &str) -> Result<Uuid> {
        let script_id = Uuid::new_v4();
        info!("Compiling script {} from source", script_id);

        // Simple compilation - in a real implementation, this would parse LSL syntax
        let compiled = self.compile_lsl_source(script_id, source_code).await?;

        // Store compiled script
        self.compiled_scripts.write().await.insert(script_id, compiled);

        // Update statistics
        {
            let mut stats = self.execution_stats.write().await;
            stats.scripts_compiled += 1;
        }

        info!("Script {} compiled successfully", script_id);
        Ok(script_id)
    }

    /// Execute an event in a script
    pub async fn execute_event(
        &self,
        script_id: Uuid,
        event: &LSLEvent,
        context: &ScriptContext,
    ) -> Result<()> {
        debug!("Executing event {:?} in script {}", event, script_id);

        let compiled_script = {
            let scripts = self.compiled_scripts.read().await;
            scripts.get(&script_id).cloned()
                .ok_or_else(|| anyhow!("Script {} not found or not compiled", script_id))?
        };

        let start_time = std::time::Instant::now();

        // Execute the event
        match self.execute_event_internal(&compiled_script, event, context).await {
            Ok(_) => {
                let execution_time = start_time.elapsed().as_millis() as f64;
                
                // Update statistics
                {
                    let mut stats = self.execution_stats.write().await;
                    stats.events_executed += 1;
                    stats.total_execution_time_ms += execution_time;
                }

                debug!("Event executed successfully in {}ms", execution_time);
                Ok(())
            }
            Err(e) => {
                let mut stats = self.execution_stats.write().await;
                stats.execution_errors += 1;
                
                error!("Event execution failed: {}", e);
                Err(e)
            }
        }
    }

    /// Get script engine statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        self.execution_stats.read().await.clone()
    }

    /// Compile LSL source code to bytecode
    async fn compile_lsl_source(&self, script_id: Uuid, source_code: &str) -> Result<CompiledScript> {
        debug!("Compiling LSL source for script {}", script_id);

        // This is a simplified compilation process
        // A real LSL compiler would parse the syntax tree and generate bytecode
        
        let mut functions = HashMap::new();
        let mut states = HashMap::new();
        let mut events = HashMap::new();
        let mut global_variables = HashMap::new();

        // Parse the source code for basic structure
        let lines: Vec<&str> = source_code.lines().collect();
        let mut current_state = "default".to_string();
        let mut current_events = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Detect state declarations
            if trimmed.starts_with("state ") {
                if !current_events.is_empty() {
                    events.insert(current_state.clone(), current_events.clone());
                    current_events.clear();
                }
                
                let state_name = trimmed[6..].trim_end_matches(" {").trim();
                current_state = state_name.to_string();
                states.insert(current_state.clone(), line_num);
                debug!("Found state: {}", current_state);
            }
            
            // Detect event handlers
            else if trimmed.contains("()") && trimmed.contains("{") {
                let event_name = trimmed.split("(").next().unwrap().trim();
                if is_lsl_event(event_name) {
                    current_events.push(event_name.to_string());
                    debug!("Found event: {} in state {}", event_name, current_state);
                }
            }
            
            // Detect function declarations
            else if is_function_declaration(trimmed) {
                let func_name = extract_function_name(trimmed);
                if let Some(name) = func_name {
                    debug!("Found function: {}", name);
                    functions.insert(name, line_num);
                }
            }
            
            // Detect global variable declarations
            else if is_global_variable(trimmed) {
                if let Some((var_name, var_value)) = extract_global_variable(trimmed) {
                    global_variables.insert(var_name, var_value);
                }
            }
        }

        // Add remaining events
        if !current_events.is_empty() {
            events.insert(current_state, current_events);
        }

        // Ensure default state exists
        if !states.contains_key("default") {
            states.insert("default".to_string(), 0);
        }

        // Create bytecode (simplified - just store the source for now)
        let bytecode = source_code.as_bytes().to_vec();

        Ok(CompiledScript {
            script_id,
            bytecode,
            functions,
            states,
            events,
            global_variables,
        })
    }

    /// Execute an event in a compiled script
    async fn execute_event_internal(
        &self,
        compiled_script: &CompiledScript,
        event: &LSLEvent,
        context: &ScriptContext,
    ) -> Result<()> {
        let event_name = get_event_name(event);
        debug!("Executing event: {}", event_name);

        // Check if the current state has this event handler
        let current_state = context.variables.get("$state")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "default".to_string());

        let empty_events = Vec::new();
        let state_events = compiled_script.events.get(&current_state)
            .unwrap_or(&empty_events);

        if !state_events.contains(&event_name) {
            debug!("Event {} not handled in state {}", event_name, current_state);
            return Ok(());
        }

        // Execute the event handler
        match event {
            LSLEvent::StateEntry => {
                self.execute_state_entry(compiled_script, context).await?;
            }
            LSLEvent::StateExit => {
                self.execute_state_exit(compiled_script, context).await?;
            }
            LSLEvent::Touch(touch_data) => {
                self.execute_touch_event(compiled_script, context, touch_data).await?;
            }
            LSLEvent::Timer(timer_name) => {
                self.execute_timer_event(compiled_script, context, timer_name).await?;
            }
            LSLEvent::Listen(listen_data) => {
                self.execute_listen_event(compiled_script, context, listen_data).await?;
            }
            LSLEvent::Collision(collision_data) => {
                self.execute_collision_event(compiled_script, context, collision_data).await?;
            }
            _ => {
                debug!("Event type {:?} not yet implemented", event);
            }
        }

        Ok(())
    }

    /// Execute state_entry event
    async fn execute_state_entry(
        &self,
        compiled_script: &CompiledScript,
        context: &ScriptContext,
    ) -> Result<()> {
        debug!("Executing state_entry for script {}", compiled_script.script_id);
        
        // In a real implementation, this would execute the actual bytecode
        // For now, we'll simulate execution by calling some LSL functions
        
        // Example: execute llSay(0, "Script started!");
        let mut mutable_context = context.clone();
        let args = vec![
            LSLValue::Integer(0),
            LSLValue::String("Script started!".to_string()),
        ];
        
        let _ = self.lsl_functions.execute_function("llSay", &args, &mut mutable_context).await;
        
        Ok(())
    }

    /// Execute state_exit event
    async fn execute_state_exit(
        &self,
        compiled_script: &CompiledScript,
        context: &ScriptContext,
    ) -> Result<()> {
        debug!("Executing state_exit for script {}", compiled_script.script_id);
        
        // Example: execute llSay(0, "Script stopping!");
        let mut mutable_context = context.clone();
        let args = vec![
            LSLValue::Integer(0),
            LSLValue::String("Script stopping!".to_string()),
        ];
        
        let _ = self.lsl_functions.execute_function("llSay", &args, &mut mutable_context).await;
        
        Ok(())
    }

    /// Execute touch event
    async fn execute_touch_event(
        &self,
        compiled_script: &CompiledScript,
        context: &ScriptContext,
        touch_data: &super::TouchEventData,
    ) -> Result<()> {
        debug!("Executing touch event for script {}", compiled_script.script_id);
        
        // Example: execute llSay(0, "Touched by " + toucher_name);
        let mut mutable_context = context.clone();
        let args = vec![
            LSLValue::Integer(0),
            LSLValue::String(format!("Touched by {}", touch_data.toucher_name)),
        ];
        
        let _ = self.lsl_functions.execute_function("llSay", &args, &mut mutable_context).await;
        
        Ok(())
    }

    /// Execute timer event
    async fn execute_timer_event(
        &self,
        compiled_script: &CompiledScript,
        context: &ScriptContext,
        timer_name: &str,
    ) -> Result<()> {
        debug!("Executing timer event {} for script {}", timer_name, compiled_script.script_id);
        
        // Example: execute llSay(0, "Timer fired!");
        let mut mutable_context = context.clone();
        let args = vec![
            LSLValue::Integer(0),
            LSLValue::String("Timer fired!".to_string()),
        ];
        
        let _ = self.lsl_functions.execute_function("llSay", &args, &mut mutable_context).await;
        
        Ok(())
    }

    /// Execute listen event
    async fn execute_listen_event(
        &self,
        compiled_script: &CompiledScript,
        context: &ScriptContext,
        listen_data: &super::ListenEventData,
    ) -> Result<()> {
        debug!("Executing listen event for script {}", compiled_script.script_id);
        
        // Example: execute llSay(0, "Heard: " + message);
        let mut mutable_context = context.clone();
        let args = vec![
            LSLValue::Integer(0),
            LSLValue::String(format!("Heard: {}", listen_data.message)),
        ];
        
        let _ = self.lsl_functions.execute_function("llSay", &args, &mut mutable_context).await;
        
        Ok(())
    }

    /// Execute collision event
    async fn execute_collision_event(
        &self,
        compiled_script: &CompiledScript,
        context: &ScriptContext,
        collision_data: &super::CollisionEventData,
    ) -> Result<()> {
        debug!("Executing collision event for script {}", compiled_script.script_id);
        
        // Example: execute llSay(0, "Collision detected!");
        let mut mutable_context = context.clone();
        let args = vec![
            LSLValue::Integer(0),
            LSLValue::String(format!("Collision with {} objects!", collision_data.detected_objects.len())),
        ];
        
        let _ = self.lsl_functions.execute_function("llSay", &args, &mut mutable_context).await;
        
        Ok(())
    }
}

/// Check if a string is an LSL event name
fn is_lsl_event(name: &str) -> bool {
    matches!(name, 
        "state_entry" | "state_exit" | "touch" | "touch_start" | "touch_end" |
        "timer" | "listen" | "collision" | "collision_start" | "collision_end" |
        "land_collision" | "land_collision_start" | "land_collision_end" |
        "at_target" | "not_at_target" | "at_rot_target" | "not_at_rot_target" |
        "money" | "email" | "http_request" | "http_response" | "run_time_permissions" |
        "changed" | "attach" | "dataserver" | "moving_start" | "moving_end" |
        "object_rez" | "remote_data" | "control" | "sensor" | "no_sensor" |
        "link_message"
    )
}

/// Check if a line is a function declaration
fn is_function_declaration(line: &str) -> bool {
    // Simple check for function patterns
    line.contains("(") && line.contains(")") && line.contains("{") && 
    !line.trim_start().starts_with("//") &&
    !is_lsl_event(line.split("(").next().unwrap_or("").trim())
}

/// Extract function name from declaration
fn extract_function_name(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if part.contains("(") {
            let func_name = part.split("(").next()?;
            return Some(func_name.to_string());
        }
    }
    None
}

/// Check if a line is a global variable declaration
fn is_global_variable(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.starts_with("//") && 
    !trimmed.contains("(") &&
    !trimmed.contains("{") &&
    (trimmed.starts_with("integer ") || 
     trimmed.starts_with("float ") ||
     trimmed.starts_with("string ") ||
     trimmed.starts_with("key ") ||
     trimmed.starts_with("vector ") ||
     trimmed.starts_with("rotation ") ||
     trimmed.starts_with("list "))
}

/// Extract global variable name and value
fn extract_global_variable(line: &str) -> Option<(String, LSLValue)> {
    let trimmed = line.trim().trim_end_matches(";");
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    
    if parts.len() >= 2 {
        let var_type = parts[0];
        let var_name = parts[1];
        
        // Extract initial value if present
        let value = if parts.len() >= 4 && parts[2] == "=" {
            let val_str = parts[3..].join(" ");
            match var_type {
                "integer" => LSLValue::Integer(val_str.parse().unwrap_or(0)),
                "float" => LSLValue::Float(val_str.parse().unwrap_or(0.0)),
                "string" => LSLValue::String(val_str.trim_matches('"').to_string()),
                "key" => LSLValue::Key(uuid::Uuid::parse_str(&val_str).unwrap_or(uuid::Uuid::nil())),
                _ => LSLValue::String(val_str),
            }
        } else {
            // Default values
            match var_type {
                "integer" => LSLValue::Integer(0),
                "float" => LSLValue::Float(0.0),
                "string" => LSLValue::String(String::new()),
                "key" => LSLValue::Key(uuid::Uuid::nil()),
                "vector" => LSLValue::Vector(super::LSLVector::zero()),
                "rotation" => LSLValue::Rotation(super::LSLRotation::identity()),
                "list" => LSLValue::List(Vec::new()),
                _ => LSLValue::String(String::new()),
            }
        };
        
        Some((var_name.to_string(), value))
    } else {
        None
    }
}

/// Get event name from LSLEvent
fn get_event_name(event: &LSLEvent) -> String {
    match event {
        LSLEvent::StateEntry => "state_entry".to_string(),
        LSLEvent::StateExit => "state_exit".to_string(),
        LSLEvent::Touch(_) => "touch".to_string(),
        LSLEvent::Timer(_) => "timer".to_string(),
        LSLEvent::Listen(_) => "listen".to_string(),
        LSLEvent::Collision(_) => "collision".to_string(),
        LSLEvent::LandCollision(_) => "land_collision".to_string(),
        LSLEvent::AtTarget(_) => "at_target".to_string(),
        LSLEvent::NotAtTarget => "not_at_target".to_string(),
        LSLEvent::AtRotTarget(_) => "at_rot_target".to_string(),
        LSLEvent::NotAtRotTarget => "not_at_rot_target".to_string(),
        LSLEvent::MoneyTransfer(_) => "money".to_string(),
        LSLEvent::Email(_) => "email".to_string(),
        LSLEvent::HttpRequest(_) => "http_request".to_string(),
        LSLEvent::HttpResponse(_) => "http_response".to_string(),
        LSLEvent::RunTimePermissions(_) => "run_time_permissions".to_string(),
        LSLEvent::Changed(_) => "changed".to_string(),
        LSLEvent::Attach(_) => "attach".to_string(),
        LSLEvent::Dataserver(_) => "dataserver".to_string(),
        LSLEvent::MovingStart => "moving_start".to_string(),
        LSLEvent::MovingEnd => "moving_end".to_string(),
        LSLEvent::ObjectRez(_) => "object_rez".to_string(),
        LSLEvent::Remote(_) => "remote_data".to_string(),
        LSLEvent::Control(_) => "control".to_string(),
        LSLEvent::SensorDetect(_) => "sensor".to_string(),
        LSLEvent::NoSensor => "no_sensor".to_string(),
        LSLEvent::LinkMessage(_) => "link_message".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_script_engine_creation() -> Result<()> {
        let engine = ScriptEngine::new().await?;
        let stats = engine.get_stats().await;
        
        assert_eq!(stats.scripts_compiled, 0);
        assert_eq!(stats.events_executed, 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_script_compilation() -> Result<()> {
        let engine = ScriptEngine::new().await?;
        
        let script_source = r#"
            default {
                state_entry() {
                    llSay(0, "Hello, world!");
                }
                
                touch_start(integer total_number) {
                    llSay(0, "Touched!");
                }
            }
        "#;
        
        let script_id = engine.compile_script(script_source).await?;
        assert_ne!(script_id, Uuid::nil());
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.scripts_compiled, 1);
        
        Ok(())
    }

    #[test]
    fn test_lsl_event_detection() {
        assert!(is_lsl_event("state_entry"));
        assert!(is_lsl_event("touch"));
        assert!(is_lsl_event("timer"));
        assert!(!is_lsl_event("my_function"));
        assert!(!is_lsl_event("random_name"));
    }

    #[test]
    fn test_function_declaration_detection() {
        assert!(is_function_declaration("my_function() {"));
        assert!(is_function_declaration("integer calculate(float x, float y) {"));
        assert!(!is_function_declaration("state_entry() {"));
        assert!(!is_function_declaration("// this is a comment"));
    }

    #[test]
    fn test_global_variable_detection() {
        assert!(is_global_variable("integer myVar = 42;"));
        assert!(is_global_variable("string message;"));
        assert!(is_global_variable("float pi = 3.14159;"));
        assert!(!is_global_variable("state_entry() {"));
        assert!(!is_global_variable("// comment"));
    }
}