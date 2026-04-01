use std::collections::VecDeque;
use uuid::Uuid;

use super::LSLValue;
use super::executor::{ExecutionResult, ScriptExecutor, ScriptInstance};

const MAX_EVENT_QUEUE_SIZE: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScriptEventType {
    StateEntry,
    StateExit,
    Touch,
    TouchStart,
    TouchEnd,
    Timer,
    Control,
    Listen,
    Collision,
    CollisionStart,
    CollisionEnd,
    LandCollision,
    LandCollisionStart,
    LandCollisionEnd,
    Sensor,
    NoSensor,
    AtTarget,
    NotAtTarget,
    AtRotTarget,
    NotAtRotTarget,
    Money,
    Email,
    HttpRequest,
    HttpResponse,
    RunTimePermissions,
    Changed,
    Attach,
    Dataserver,
    MovingStart,
    MovingEnd,
    ObjectRez,
    RemoteData,
    LinkMessage,
    OnRez,
}

impl ScriptEventType {
    pub fn priority(&self) -> u8 {
        match self {
            ScriptEventType::StateEntry => 0,
            ScriptEventType::StateExit => 1,
            ScriptEventType::Timer => 2,
            ScriptEventType::Control => 3,
            ScriptEventType::Listen => 4,
            ScriptEventType::Touch | ScriptEventType::TouchStart | ScriptEventType::TouchEnd => 5,
            ScriptEventType::Collision | ScriptEventType::CollisionStart | ScriptEventType::CollisionEnd => 6,
            ScriptEventType::Sensor => 7,
            ScriptEventType::AtTarget | ScriptEventType::NotAtTarget => 8,
            ScriptEventType::AtRotTarget | ScriptEventType::NotAtRotTarget => 8,
            ScriptEventType::Money => 9,
            ScriptEventType::HttpResponse => 10,
            ScriptEventType::Dataserver => 11,
            ScriptEventType::LinkMessage => 12,
            ScriptEventType::Changed => 13,
            ScriptEventType::OnRez => 14,
            ScriptEventType::NoSensor => 15,
            _ => 14,
        }
    }

    pub fn event_name(&self) -> &'static str {
        match self {
            ScriptEventType::StateEntry => "state_entry",
            ScriptEventType::StateExit => "state_exit",
            ScriptEventType::Touch => "touch",
            ScriptEventType::TouchStart => "touch_start",
            ScriptEventType::TouchEnd => "touch_end",
            ScriptEventType::Timer => "timer",
            ScriptEventType::Control => "control",
            ScriptEventType::Listen => "listen",
            ScriptEventType::Collision => "collision",
            ScriptEventType::CollisionStart => "collision_start",
            ScriptEventType::CollisionEnd => "collision_end",
            ScriptEventType::LandCollision => "land_collision",
            ScriptEventType::LandCollisionStart => "land_collision_start",
            ScriptEventType::LandCollisionEnd => "land_collision_end",
            ScriptEventType::Sensor => "sensor",
            ScriptEventType::NoSensor => "no_sensor",
            ScriptEventType::AtTarget => "at_target",
            ScriptEventType::NotAtTarget => "not_at_target",
            ScriptEventType::AtRotTarget => "at_rot_target",
            ScriptEventType::NotAtRotTarget => "not_at_rot_target",
            ScriptEventType::Money => "money",
            ScriptEventType::Email => "email",
            ScriptEventType::HttpRequest => "http_request",
            ScriptEventType::HttpResponse => "http_response",
            ScriptEventType::RunTimePermissions => "run_time_permissions",
            ScriptEventType::Changed => "changed",
            ScriptEventType::Attach => "attach",
            ScriptEventType::Dataserver => "dataserver",
            ScriptEventType::MovingStart => "moving_start",
            ScriptEventType::MovingEnd => "moving_end",
            ScriptEventType::ObjectRez => "object_rez",
            ScriptEventType::RemoteData => "remote_data",
            ScriptEventType::LinkMessage => "link_message",
            ScriptEventType::OnRez => "on_rez",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptEvent {
    pub event_type: ScriptEventType,
    pub args: Vec<LSLValue>,
}

pub struct ScriptStateMachine {
    pub script_id: Uuid,
    pub instance: ScriptInstance,
    event_queue: VecDeque<ScriptEvent>,
    pub min_event_delay: f64,
    pub last_event_time: std::time::Instant,
    timer_queued: bool,
}

impl ScriptStateMachine {
    pub fn new(script_id: Uuid, instance: ScriptInstance) -> Self {
        Self {
            script_id,
            instance,
            event_queue: VecDeque::new(),
            min_event_delay: 0.0,
            last_event_time: std::time::Instant::now(),
            timer_queued: false,
        }
    }

    pub fn post_event(&mut self, event: ScriptEvent) {
        if !self.instance.running {
            return;
        }

        if self.event_queue.len() >= MAX_EVENT_QUEUE_SIZE {
            return;
        }

        if event.event_type == ScriptEventType::Timer {
            if self.timer_queued {
                return;
            }
            self.timer_queued = true;
        }

        let insert_pos = self.event_queue.iter().position(|e| {
            event.event_type.priority() < e.event_type.priority()
        }).unwrap_or(self.event_queue.len());

        self.event_queue.insert(insert_pos, event);
    }

    pub fn process_next_event(&mut self, executor: &dyn ScriptExecutor) -> Option<ExecutionResult> {
        if !self.instance.running {
            return None;
        }

        if self.min_event_delay > 0.0 {
            let elapsed = self.last_event_time.elapsed().as_secs_f64();
            if elapsed < self.min_event_delay {
                return None;
            }
        }

        let event = self.event_queue.pop_front()?;

        if event.event_type == ScriptEventType::Timer {
            self.timer_queued = false;
        }

        self.last_event_time = std::time::Instant::now();

        let event_name = event.event_type.event_name();
        tracing::debug!("[SCRIPT] Executing '{}' on {} (running={}, queue_left={})", event_name, self.script_id, self.instance.running, self.event_queue.len());
        match executor.execute_event(&mut self.instance, event_name, &event.args) {
            Ok(ExecutionResult::StateChange(new_state)) => {
                tracing::info!("[SCRIPT] Event '{}' on {} triggered state change to '{}'", event_name, self.script_id, new_state);
                self.change_state(&new_state, executor);
                Some(ExecutionResult::StateChange(new_state))
            }
            Ok(ExecutionResult::Error(ref msg)) => {
                tracing::warn!("[SCRIPT] Event '{}' on {} execution error: {}", event_name, self.script_id, msg);
                Some(ExecutionResult::Error(msg.clone()))
            }
            Ok(result) => {
                tracing::debug!("[SCRIPT] Event '{}' on {} completed: actions={}, running={}", event_name, self.script_id, self.instance.pending_actions.len(), self.instance.running);
                Some(result)
            }
            Err(e) => {
                tracing::warn!("[SCRIPT] Event '{}' CRASHED script {}: {}", event_name, self.script_id, e);
                self.instance.running = false;
                Some(ExecutionResult::Error(e.to_string()))
            }
        }
    }

    fn change_state(&mut self, new_state: &str, executor: &dyn ScriptExecutor) {
        let _ = executor.execute_event(&mut self.instance, "state_exit", &[]);

        self.event_queue.clear();
        self.timer_queued = false;

        self.instance.current_state = new_state.to_string();

        let _ = executor.execute_event(&mut self.instance, "state_entry", &[]);
    }

    pub fn event_queue_len(&self) -> usize {
        self.event_queue.len()
    }

    pub fn is_running(&self) -> bool {
        self.instance.running
    }

    pub fn reset(&mut self, executor: &dyn ScriptExecutor) {
        self.event_queue.clear();
        self.timer_queued = false;
        self.instance.current_state = "default".to_string();
        self.instance.heap_used = 0;

        for (name, type_name, _) in &self.instance.compiled.globals.clone() {
            self.instance.global_vars.insert(name.clone(), LSLValue::type_default(type_name));
        }

        self.instance.running = true;
        match executor.execute_event(&mut self.instance, "state_entry", &[]) {
            Ok(result) => {
                tracing::info!("[SCRIPT RESET] state_entry executed OK: {:?}, pending_actions={}", result, self.instance.pending_actions.len());
            }
            Err(e) => {
                tracing::warn!("[SCRIPT RESET] state_entry FAILED: {}", e);
                self.instance.running = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::executor::tree_walk::TreeWalkExecutor;

    #[test]
    fn test_event_queue_priority() {
        let executor = TreeWalkExecutor::new();
        let source = r#"
default
{
    state_entry()
    {
    }
}
"#;
        let compiled = executor.compile(source, Uuid::new_v4()).unwrap();
        let instance = ScriptInstance::new(Uuid::new_v4(), compiled, 65536);
        let mut sm = ScriptStateMachine::new(Uuid::new_v4(), instance);

        sm.post_event(ScriptEvent {
            event_type: ScriptEventType::Listen,
            args: vec![],
        });
        sm.post_event(ScriptEvent {
            event_type: ScriptEventType::Timer,
            args: vec![],
        });
        sm.post_event(ScriptEvent {
            event_type: ScriptEventType::StateEntry,
            args: vec![],
        });

        assert_eq!(sm.event_queue[0].event_type, ScriptEventType::StateEntry);
        assert_eq!(sm.event_queue[1].event_type, ScriptEventType::Timer);
        assert_eq!(sm.event_queue[2].event_type, ScriptEventType::Listen);
    }

    #[test]
    fn test_timer_coalescing() {
        let executor = TreeWalkExecutor::new();
        let source = r#"
default
{
    state_entry()
    {
    }
}
"#;
        let compiled = executor.compile(source, Uuid::new_v4()).unwrap();
        let instance = ScriptInstance::new(Uuid::new_v4(), compiled, 65536);
        let mut sm = ScriptStateMachine::new(Uuid::new_v4(), instance);

        sm.post_event(ScriptEvent {
            event_type: ScriptEventType::Timer,
            args: vec![],
        });
        sm.post_event(ScriptEvent {
            event_type: ScriptEventType::Timer,
            args: vec![],
        });

        assert_eq!(sm.event_queue_len(), 1);
    }

    #[test]
    fn test_max_queue_size() {
        let executor = TreeWalkExecutor::new();
        let source = r#"
default
{
    state_entry()
    {
    }
}
"#;
        let compiled = executor.compile(source, Uuid::new_v4()).unwrap();
        let instance = ScriptInstance::new(Uuid::new_v4(), compiled, 65536);
        let mut sm = ScriptStateMachine::new(Uuid::new_v4(), instance);

        for _ in 0..100 {
            sm.post_event(ScriptEvent {
                event_type: ScriptEventType::Listen,
                args: vec![],
            });
        }

        assert_eq!(sm.event_queue_len(), MAX_EVENT_QUEUE_SIZE);
    }

    #[test]
    fn test_process_event() {
        let executor = TreeWalkExecutor::new();
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
        let compiled = executor.compile(source, Uuid::new_v4()).unwrap();
        let instance = ScriptInstance::new(Uuid::new_v4(), compiled, 65536);
        let mut sm = ScriptStateMachine::new(Uuid::new_v4(), instance);

        sm.post_event(ScriptEvent {
            event_type: ScriptEventType::StateEntry,
            args: vec![],
        });

        let result = sm.process_next_event(&executor);
        assert!(result.is_some());
        assert_eq!(sm.instance.global_vars.get("count"), Some(&LSLValue::Integer(1)));
    }
}
