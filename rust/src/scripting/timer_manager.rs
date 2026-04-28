use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

const DEFAULT_MIN_TIMER_INTERVAL: f64 = 0.5;

struct TimerEntry {
    script_id: Uuid,
    interval: f64,
    next_fire: Instant,
}

pub struct TimerManager {
    timers: HashMap<Uuid, TimerEntry>,
    min_interval: f64,
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            min_interval: DEFAULT_MIN_TIMER_INTERVAL,
        }
    }

    pub fn with_min_interval(mut self, interval: f64) -> Self {
        self.min_interval = interval;
        self
    }

    pub fn set_timer(&mut self, script_id: Uuid, interval: f64) {
        if interval <= 0.0 {
            self.timers.remove(&script_id);
            return;
        }

        let actual_interval = interval.max(self.min_interval);

        let now = Instant::now();
        self.timers.insert(
            script_id,
            TimerEntry {
                script_id,
                interval: actual_interval,
                next_fire: now + std::time::Duration::from_secs_f64(actual_interval),
            },
        );
    }

    pub fn stop_timer(&mut self, script_id: Uuid) {
        self.timers.remove(&script_id);
    }

    pub fn check_timers(&mut self) -> Vec<Uuid> {
        let now = Instant::now();
        let mut fired = Vec::new();

        for entry in self.timers.values_mut() {
            if now >= entry.next_fire {
                fired.push(entry.script_id);
                entry.next_fire = now + std::time::Duration::from_secs_f64(entry.interval);
            }
        }

        fired
    }

    pub fn remove_all_for_script(&mut self, script_id: Uuid) {
        self.timers.remove(&script_id);
    }

    pub fn has_timer(&self, script_id: Uuid) -> bool {
        self.timers.contains_key(&script_id)
    }

    pub fn timer_count(&self) -> usize {
        self.timers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_set_and_check_timer() {
        let mut mgr = TimerManager::new().with_min_interval(0.01);
        let script_id = Uuid::new_v4();

        mgr.set_timer(script_id, 0.02);
        assert!(mgr.has_timer(script_id));

        let fired = mgr.check_timers();
        assert!(fired.is_empty());

        sleep(Duration::from_millis(30));
        let fired = mgr.check_timers();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0], script_id);
    }

    #[test]
    fn test_stop_timer() {
        let mut mgr = TimerManager::new();
        let script_id = Uuid::new_v4();

        mgr.set_timer(script_id, 1.0);
        assert!(mgr.has_timer(script_id));

        mgr.stop_timer(script_id);
        assert!(!mgr.has_timer(script_id));
    }

    #[test]
    fn test_zero_interval_removes_timer() {
        let mut mgr = TimerManager::new();
        let script_id = Uuid::new_v4();

        mgr.set_timer(script_id, 1.0);
        assert!(mgr.has_timer(script_id));

        mgr.set_timer(script_id, 0.0);
        assert!(!mgr.has_timer(script_id));
    }

    #[test]
    fn test_min_interval_clamping() {
        let mut mgr = TimerManager::new().with_min_interval(0.5);
        let script_id = Uuid::new_v4();

        mgr.set_timer(script_id, 0.01);
        assert!(mgr.has_timer(script_id));

        let fired = mgr.check_timers();
        assert!(fired.is_empty());
    }
}
