use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::modules::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};
use crate::modules::events::{EventBus, SceneEvent};
use crate::modules::services::ServiceRegistry;

use super::executor::tree_walk::TreeWalkExecutor;
use super::executor::bytecode::BytecodeExecutor;
use super::executor::cranelift_jit::CraneliftExecutor;
use super::executor::{CompiledScript, ExecutionResult, ScriptExecutor, ScriptInstance};
use super::state_machine::{ScriptStateMachine, ScriptEvent, ScriptEventType};
use super::listen_manager::ListenManager;
use super::timer_manager::TimerManager;
use super::sensor_manager::SensorManager;
use super::LSLValue;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionBackend {
    TreeWalk,
    Bytecode,
    Cranelift,
}

impl ExecutionBackend {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bytecode" => ExecutionBackend::Bytecode,
            "cranelift" | "jit" => ExecutionBackend::Cranelift,
            _ => ExecutionBackend::TreeWalk,
        }
    }
}

pub struct YEngineConfig {
    pub enabled: bool,
    pub backend: ExecutionBackend,
    pub max_scripts_per_region: usize,
    pub default_heap_limit: usize,
    pub event_queue_size: usize,
    pub timeslice_ms: u64,
    pub enable_ossl: bool,
    pub ossl_threat_level: String,
    pub save_state_interval: u64,
    pub min_timer_interval: f64,
}

impl Default for YEngineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: ExecutionBackend::TreeWalk,
            max_scripts_per_region: 10000,
            default_heap_limit: 65536,
            event_queue_size: 50,
            timeslice_ms: 5,
            enable_ossl: true,
            ossl_threat_level: "VeryLow".to_string(),
            save_state_interval: 300,
            min_timer_interval: 0.5,
        }
    }
}

pub struct YEngineModule {
    config: YEngineConfig,
    executor: Option<Arc<dyn ScriptExecutor>>,
    scripts: Arc<RwLock<HashMap<Uuid, ScriptStateMachine>>>,
    listen_manager: Arc<RwLock<ListenManager>>,
    timer_manager: Arc<RwLock<TimerManager>>,
    sensor_manager: Arc<RwLock<SensorManager>>,
    region_uuid: Option<Uuid>,
    granted_perms: Arc<RwLock<HashMap<Uuid, (u32, Uuid)>>>,
}

impl YEngineModule {
    pub fn new() -> Self {
        Self {
            config: YEngineConfig::default(),
            executor: None,
            scripts: Arc::new(RwLock::new(HashMap::new())),
            listen_manager: Arc::new(RwLock::new(ListenManager::new())),
            timer_manager: Arc::new(RwLock::new(TimerManager::new())),
            sensor_manager: Arc::new(RwLock::new(SensorManager::new())),
            region_uuid: None,
            granted_perms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn initialize_default(&mut self) {
        self.executor = Some(Arc::new(TreeWalkExecutor::new()));
        self.timer_manager = Arc::new(RwLock::new(
            TimerManager::new().with_min_interval(self.config.min_timer_interval)
        ));
    }

    pub fn rez_script(
        &self,
        script_id: Uuid,
        source: &str,
        start_param: i32,
    ) -> Result<()> {
        let executor = self.executor.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No executor configured"))?;

        let compiled = executor.compile(source, script_id)?;
        let instance = ScriptInstance::new(script_id, compiled, self.config.default_heap_limit);
        let mut sm = ScriptStateMachine::new(script_id, instance);

        let mut args = vec![];
        if start_param != 0 {
            args.push(LSLValue::Integer(start_param));
        }

        sm.post_event(ScriptEvent {
            event_type: ScriptEventType::StateEntry,
            args: vec![],
        });

        if start_param != 0 {
            sm.post_event(ScriptEvent {
                event_type: ScriptEventType::OnRez,
                args: vec![LSLValue::Integer(start_param)],
            });
        }

        {
            let mut scripts = self.scripts.write();
            if scripts.len() >= self.config.max_scripts_per_region {
                return Err(anyhow::anyhow!("Maximum scripts per region exceeded"));
            }
            scripts.insert(script_id, sm);
        }

        info!("Rezzed script {}", script_id);
        Ok(())
    }

    pub fn stop_script(&self, script_id: Uuid) -> Result<()> {
        let mut scripts = self.scripts.write();
        if let Some(sm) = scripts.get_mut(&script_id) {
            sm.instance.running = false;
            self.listen_manager.write().remove_all_for_script(script_id);
            self.timer_manager.write().remove_all_for_script(script_id);
            self.sensor_manager.write().remove_all_for_script(script_id);
            info!("Stopped script {}", script_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Script not found: {}", script_id))
        }
    }

    pub fn reset_script(&self, script_id: Uuid) -> Result<()> {
        let executor = self.executor.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No executor configured"))?;

        let mut scripts = self.scripts.write();
        if let Some(sm) = scripts.get_mut(&script_id) {
            self.listen_manager.write().remove_all_for_script(script_id);
            self.timer_manager.write().remove_all_for_script(script_id);
            self.sensor_manager.write().remove_all_for_script(script_id);
            sm.reset(executor.as_ref());
            info!("Reset script {}", script_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Script not found: {}", script_id))
        }
    }

    pub fn set_script_running(&self, script_id: Uuid, running: bool) -> Result<()> {
        let mut scripts = self.scripts.write();
        if let Some(sm) = scripts.get_mut(&script_id) {
            sm.instance.running = running;
            if !running {
                self.listen_manager.write().remove_all_for_script(script_id);
                self.timer_manager.write().remove_all_for_script(script_id);
                self.sensor_manager.write().remove_all_for_script(script_id);
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Script not found: {}", script_id))
        }
    }

    pub fn get_script_running(&self, script_id: Uuid) -> bool {
        self.scripts.read().get(&script_id)
            .map(|sm| sm.is_running())
            .unwrap_or(false)
    }

    pub fn set_script_context(&self, script_id: Uuid, ctx: super::executor::ObjectContext) {
        let mut scripts = self.scripts.write();
        if let Some(sm) = scripts.get_mut(&script_id) {
            sm.instance.context = ctx;
        }
    }

    pub fn set_detect_params(&self, script_id: Uuid, params: Vec<super::executor::DetectInfo>) {
        let mut scripts = self.scripts.write();
        if let Some(sm) = scripts.get_mut(&script_id) {
            sm.instance.context.detect_params = params;
        }
    }

    pub fn grant_permissions(&self, script_id: Uuid, perms: u32, granter: Uuid) {
        self.granted_perms.write().insert(script_id, (perms, granter));
        let mut scripts = self.scripts.write();
        if let Some(sm) = scripts.get_mut(&script_id) {
            sm.instance.context.granted_perms = perms;
            sm.instance.context.perm_granter = granter;
        }
    }

    pub fn get_granted_perms(&self, script_id: Uuid) -> (u32, Uuid) {
        self.granted_perms.read().get(&script_id).copied().unwrap_or((0, Uuid::nil()))
    }

    pub fn update_sitting_avatar(&self, script_ids: &[Uuid], avatar_id: Uuid) {
        let mut scripts = self.scripts.write();
        for sid in script_ids {
            if let Some(sm) = scripts.get_mut(sid) {
                sm.instance.context.sitting_avatar_id = avatar_id;
            }
        }
    }

    pub fn post_event(&self, script_id: Uuid, event: ScriptEvent) {
        let mut scripts = self.scripts.write();
        if let Some(sm) = scripts.get_mut(&script_id) {
            sm.post_event(event);
        }
    }

    pub fn post_event_all(&self, event_type: ScriptEventType, args: Vec<LSLValue>) {
        let mut scripts = self.scripts.write();
        for sm in scripts.values_mut() {
            sm.post_event(ScriptEvent {
                event_type: event_type.clone(),
                args: args.clone(),
            });
        }
    }

    pub fn post_event_to_scripts(&self, script_ids: &[Uuid], event_type: ScriptEventType, args: Vec<LSLValue>) {
        let mut scripts = self.scripts.write();
        for sid in script_ids {
            if let Some(sm) = scripts.get_mut(sid) {
                sm.post_event(ScriptEvent {
                    event_type: event_type.clone(),
                    args: args.clone(),
                });
            }
        }
    }

    pub fn post_event_to_scripts_with_detect(&self, script_ids: &[Uuid], event_type: ScriptEventType, args: Vec<LSLValue>, detect: Vec<super::executor::DetectInfo>) {
        let mut scripts = self.scripts.write();
        for sid in script_ids {
            if let Some(sm) = scripts.get_mut(sid) {
                sm.instance.context.detect_params = detect.clone();
                sm.post_event(ScriptEvent {
                    event_type: event_type.clone(),
                    args: args.clone(),
                });
            }
        }
    }

    pub fn process_scripts(&self) -> Vec<(Uuid, super::executor::ScriptAction)> {
        let executor = match &self.executor {
            Some(e) => e.clone(),
            None => return Vec::new(),
        };

        let timer_fires = self.timer_manager.write().check_timers();
        for script_id in timer_fires {
            let mut scripts = self.scripts.write();
            if let Some(sm) = scripts.get_mut(&script_id) {
                sm.post_event(ScriptEvent {
                    event_type: ScriptEventType::Timer,
                    args: vec![],
                });
            }
        }

        let sensor_fires = self.sensor_manager.write().check_sensors();
        for (script_id, _params) in sensor_fires {
            let mut scripts = self.scripts.write();
            if let Some(sm) = scripts.get_mut(&script_id) {
                sm.post_event(ScriptEvent {
                    event_type: ScriptEventType::Sensor,
                    args: vec![LSLValue::Integer(0)],
                });
            }
        }

        let mut all_actions = Vec::new();
        let mut scripts = self.scripts.write();
        let script_ids: Vec<Uuid> = scripts.keys().copied().collect();
        let perms_snapshot = self.granted_perms.read().clone();
        let has_pending: Vec<_> = script_ids.iter().filter(|id| {
            scripts.get(id).map_or(false, |sm| sm.event_queue_len() > 0 || !sm.instance.pending_actions.is_empty())
        }).cloned().collect();
        if !has_pending.is_empty() {
            info!("[PROCESS] {} scripts with pending events/actions: {:?}", has_pending.len(), has_pending);
        }
        for script_id in script_ids {
            if let Some(sm) = scripts.get_mut(&script_id) {
                if let Some(&(p, g)) = perms_snapshot.get(&script_id) {
                    sm.instance.context.granted_perms = p;
                    sm.instance.context.perm_granter = g;
                }
                sm.process_next_event(executor.as_ref());
                let actions: Vec<_> = sm.instance.pending_actions.drain(..).collect();
                for action in actions {
                    match &action {
                        super::executor::ScriptAction::SetTimerEvent { interval } => {
                            if *interval > 0.0 {
                                self.timer_manager.write().set_timer(script_id, *interval);
                            } else {
                                self.timer_manager.write().stop_timer(script_id);
                            }
                        }
                        super::executor::ScriptAction::Listen { channel, name, id, msg } => {
                            self.listen_manager.write().add_listener(
                                script_id, *channel, name,
                                uuid::Uuid::parse_str(id).unwrap_or_default(), msg,
                            );
                        }
                        _ => {
                            all_actions.push((script_id, action));
                        }
                    }
                }
            }
        }
        all_actions
    }

    pub fn deliver_chat(
        &self,
        channel: i32,
        sender_name: &str,
        sender_id: Uuid,
        message: &str,
    ) {
        let listeners = {
            let mgr = self.listen_manager.read();
            mgr.get_matching_listeners(channel, sender_name, sender_id, message)
                .iter()
                .map(|l| (l.script_id, l.channel))
                .collect::<Vec<_>>()
        };

        let mut scripts = self.scripts.write();
        for (script_id, ch) in listeners {
            if let Some(sm) = scripts.get_mut(&script_id) {
                sm.post_event(ScriptEvent {
                    event_type: ScriptEventType::Listen,
                    args: vec![
                        LSLValue::Integer(ch),
                        LSLValue::String(sender_name.to_string()),
                        LSLValue::Key(sender_id),
                        LSLValue::String(message.to_string()),
                    ],
                });
            }
        }
    }

    pub fn script_count(&self) -> usize {
        self.scripts.read().len()
    }

    pub fn running_script_count(&self) -> usize {
        self.scripts.read().values().filter(|sm| sm.is_running()).count()
    }

    pub fn listen_manager(&self) -> Arc<RwLock<ListenManager>> {
        self.listen_manager.clone()
    }

    pub fn timer_manager(&self) -> Arc<RwLock<TimerManager>> {
        self.timer_manager.clone()
    }

    pub fn sensor_manager(&self) -> Arc<RwLock<SensorManager>> {
        self.sensor_manager.clone()
    }
}

#[async_trait]
impl RegionModule for YEngineModule {
    fn name(&self) -> &'static str {
        "YEngine"
    }

    fn replaceable_interface(&self) -> Option<&'static str> {
        Some("IScriptEngine")
    }

    async fn initialize(&mut self, config: &ModuleConfig) -> Result<()> {
        self.config.enabled = config.get_bool("Enabled", true);
        if !self.config.enabled {
            info!("YEngine disabled by configuration");
            return Ok(());
        }

        self.config.backend = ExecutionBackend::from_str(
            &config.get_or("ExecutionBackend", "TreeWalk")
        );
        self.config.max_scripts_per_region = config.get_or("MaxScriptsPerRegion", "10000")
            .parse().unwrap_or(10000);
        self.config.default_heap_limit = config.get_or("DefaultHeapLimit", "65536")
            .parse().unwrap_or(65536);
        self.config.event_queue_size = config.get_or("EventQueueSize", "50")
            .parse().unwrap_or(50);
        self.config.timeslice_ms = config.get_or("TimesliceMs", "5")
            .parse().unwrap_or(5);
        self.config.enable_ossl = config.get_bool("EnableOSSL", true);
        self.config.ossl_threat_level = config.get_or("OSSLThreatLevel", "VeryLow");
        self.config.save_state_interval = config.get_or("SaveStateInterval", "300")
            .parse().unwrap_or(300);
        self.config.min_timer_interval = config.get_or("MinTimerInterval", "0.5")
            .parse().unwrap_or(0.5);

        let executor: Arc<dyn ScriptExecutor> = match self.config.backend {
            ExecutionBackend::TreeWalk => Arc::new(TreeWalkExecutor::new()),
            ExecutionBackend::Bytecode => {
                info!("Using Bytecode VM execution backend");
                Arc::new(BytecodeExecutor::new())
            }
            ExecutionBackend::Cranelift => {
                info!("Using Cranelift JIT execution backend");
                Arc::new(CraneliftExecutor::new())
            }
        };

        self.executor = Some(executor);
        self.timer_manager = Arc::new(RwLock::new(
            TimerManager::new().with_min_interval(self.config.min_timer_interval)
        ));

        info!(
            "YEngine initialized: backend={:?}, max_scripts={}, heap_limit={}, ossl={}",
            self.config.backend,
            self.config.max_scripts_per_region,
            self.config.default_heap_limit,
            self.config.enable_ossl,
        );

        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = Some(scene.region_uuid);
        info!("YEngine added to region {} ({})", scene.region_name, scene.region_uuid);
        Ok(())
    }

    async fn remove_region(&mut self, scene: &SceneContext) -> Result<()> {
        let script_count = self.script_count();
        if script_count > 0 {
            info!("YEngine removing {} scripts from region {}", script_count, scene.region_name);
            self.scripts.write().clear();
            self.listen_manager.write().remove_all_for_script(Uuid::nil());
        }
        self.region_uuid = None;
        Ok(())
    }

    async fn region_loaded(&mut self, scene: &SceneContext) -> Result<()> {
        info!("YEngine region loaded: {} - ready for scripts", scene.region_name);
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
impl SharedRegionModule for YEngineModule {
    async fn post_initialize(&mut self, _scene: &SceneContext) -> Result<()> {
        info!("YEngine post-initialized");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rez_and_run_script() {
        let mut module = YEngineModule::new();
        module.executor = Some(Arc::new(TreeWalkExecutor::new()));

        let script_id = Uuid::new_v4();
        let source = r#"
integer count = 0;
default
{
    state_entry()
    {
        count = count + 1;
    }
}
"#;
        module.rez_script(script_id, source, 0).unwrap();
        assert!(module.get_script_running(script_id));

        module.process_scripts();

        let scripts = module.scripts.read();
        let sm = scripts.get(&script_id).unwrap();
        assert_eq!(sm.instance.global_vars.get("count"), Some(&LSLValue::Integer(1)));
    }

    #[test]
    fn test_stop_script() {
        let mut module = YEngineModule::new();
        module.executor = Some(Arc::new(TreeWalkExecutor::new()));

        let script_id = Uuid::new_v4();
        let source = r#"
default
{
    state_entry()
    {
    }
}
"#;
        module.rez_script(script_id, source, 0).unwrap();
        assert!(module.get_script_running(script_id));

        module.stop_script(script_id).unwrap();
        assert!(!module.get_script_running(script_id));
    }

    #[test]
    fn test_reset_script() {
        let mut module = YEngineModule::new();
        module.executor = Some(Arc::new(TreeWalkExecutor::new()));

        let script_id = Uuid::new_v4();
        let source = r#"
integer x = 0;
default
{
    state_entry()
    {
        x = 42;
    }
}
"#;
        module.rez_script(script_id, source, 0).unwrap();
        module.process_scripts();

        module.reset_script(script_id).unwrap();

        let scripts = module.scripts.read();
        let sm = scripts.get(&script_id).unwrap();
        assert_eq!(sm.instance.current_state, "default");
    }

    #[test]
    fn test_deliver_chat() {
        let mut module = YEngineModule::new();
        module.executor = Some(Arc::new(TreeWalkExecutor::new()));

        let script_id = Uuid::new_v4();
        let source = r#"
integer heard = 0;
default
{
    state_entry()
    {
    }
    listen(integer channel, string name, key id, string message)
    {
        heard = 1;
    }
}
"#;
        module.rez_script(script_id, source, 0).unwrap();
        module.process_scripts();

        module.listen_manager.write().add_listener(script_id, 0, "", Uuid::nil(), "");

        module.deliver_chat(0, "Test User", Uuid::new_v4(), "Hello");

        module.process_scripts();

        let scripts = module.scripts.read();
        let sm = scripts.get(&script_id).unwrap();
        assert_eq!(sm.instance.global_vars.get("heard"), Some(&LSLValue::Integer(1)));
    }

    #[test]
    fn test_max_scripts_limit() {
        let mut module = YEngineModule::new();
        module.executor = Some(Arc::new(TreeWalkExecutor::new()));
        module.config.max_scripts_per_region = 2;

        let source = r#"
default
{
    state_entry()
    {
    }
}
"#;

        module.rez_script(Uuid::new_v4(), source, 0).unwrap();
        module.rez_script(Uuid::new_v4(), source, 0).unwrap();
        assert!(module.rez_script(Uuid::new_v4(), source, 0).is_err());
    }
}
