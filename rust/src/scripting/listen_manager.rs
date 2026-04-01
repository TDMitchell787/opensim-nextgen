use std::collections::HashMap;
use uuid::Uuid;

const MAX_LISTENERS_PER_SCRIPT: usize = 65;

#[derive(Debug, Clone)]
pub struct ListenEntry {
    pub handle: i32,
    pub script_id: Uuid,
    pub channel: i32,
    pub name_filter: String,
    pub key_filter: Uuid,
    pub msg_filter: String,
    pub active: bool,
}

pub struct ListenManager {
    listeners: HashMap<i32, ListenEntry>,
    next_handle: i32,
    script_listener_count: HashMap<Uuid, usize>,
}

impl ListenManager {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            next_handle: 1,
            script_listener_count: HashMap::new(),
        }
    }

    pub fn add_listener(
        &mut self,
        script_id: Uuid,
        channel: i32,
        name_filter: &str,
        key_filter: Uuid,
        msg_filter: &str,
    ) -> i32 {
        let count = self.script_listener_count.entry(script_id).or_insert(0);
        if *count >= MAX_LISTENERS_PER_SCRIPT {
            return -1;
        }

        let handle = self.next_handle;
        self.next_handle += 1;

        self.listeners.insert(handle, ListenEntry {
            handle,
            script_id,
            channel,
            name_filter: name_filter.to_string(),
            key_filter,
            msg_filter: msg_filter.to_string(),
            active: true,
        });

        *count += 1;
        handle
    }

    pub fn remove_listener(&mut self, handle: i32) {
        if let Some(entry) = self.listeners.remove(&handle) {
            if let Some(count) = self.script_listener_count.get_mut(&entry.script_id) {
                *count = count.saturating_sub(1);
            }
        }
    }

    pub fn set_active(&mut self, handle: i32, active: bool) {
        if let Some(entry) = self.listeners.get_mut(&handle) {
            entry.active = active;
        }
    }

    pub fn remove_all_for_script(&mut self, script_id: Uuid) {
        self.listeners.retain(|_, entry| entry.script_id != script_id);
        self.script_listener_count.remove(&script_id);
    }

    pub fn get_matching_listeners(
        &self,
        channel: i32,
        sender_name: &str,
        sender_id: Uuid,
        message: &str,
    ) -> Vec<&ListenEntry> {
        self.listeners.values().filter(|entry| {
            if !entry.active {
                return false;
            }
            if entry.channel != channel {
                return false;
            }
            if !entry.name_filter.is_empty() && entry.name_filter != sender_name {
                return false;
            }
            if !entry.key_filter.is_nil() && entry.key_filter != sender_id {
                return false;
            }
            if !entry.msg_filter.is_empty() && entry.msg_filter != message {
                return false;
            }
            true
        }).collect()
    }

    pub fn listener_count_for_script(&self, script_id: Uuid) -> usize {
        *self.script_listener_count.get(&script_id).unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_match_listener() {
        let mut mgr = ListenManager::new();
        let script_id = Uuid::new_v4();

        let handle = mgr.add_listener(script_id, 0, "", Uuid::nil(), "");
        assert!(handle > 0);

        let matches = mgr.get_matching_listeners(0, "Test User", Uuid::new_v4(), "Hello");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].handle, handle);
    }

    #[test]
    fn test_name_filter() {
        let mut mgr = ListenManager::new();
        let script_id = Uuid::new_v4();

        mgr.add_listener(script_id, 0, "Alice", Uuid::nil(), "");

        let matches = mgr.get_matching_listeners(0, "Bob", Uuid::new_v4(), "Hi");
        assert_eq!(matches.len(), 0);

        let matches = mgr.get_matching_listeners(0, "Alice", Uuid::new_v4(), "Hi");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_channel_filter() {
        let mut mgr = ListenManager::new();
        let script_id = Uuid::new_v4();

        mgr.add_listener(script_id, 42, "", Uuid::nil(), "");

        let matches = mgr.get_matching_listeners(0, "Test", Uuid::new_v4(), "msg");
        assert_eq!(matches.len(), 0);

        let matches = mgr.get_matching_listeners(42, "Test", Uuid::new_v4(), "msg");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_max_listeners_per_script() {
        let mut mgr = ListenManager::new();
        let script_id = Uuid::new_v4();

        for i in 0..MAX_LISTENERS_PER_SCRIPT {
            let handle = mgr.add_listener(script_id, i as i32, "", Uuid::nil(), "");
            assert!(handle > 0, "Listener {} should succeed", i);
        }

        let handle = mgr.add_listener(script_id, 999, "", Uuid::nil(), "");
        assert_eq!(handle, -1);
    }

    #[test]
    fn test_remove_listener() {
        let mut mgr = ListenManager::new();
        let script_id = Uuid::new_v4();

        let handle = mgr.add_listener(script_id, 0, "", Uuid::nil(), "");
        assert_eq!(mgr.listener_count_for_script(script_id), 1);

        mgr.remove_listener(handle);
        assert_eq!(mgr.listener_count_for_script(script_id), 0);

        let matches = mgr.get_matching_listeners(0, "Test", Uuid::new_v4(), "msg");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_deactivate_listener() {
        let mut mgr = ListenManager::new();
        let script_id = Uuid::new_v4();

        let handle = mgr.add_listener(script_id, 0, "", Uuid::nil(), "");

        mgr.set_active(handle, false);
        let matches = mgr.get_matching_listeners(0, "Test", Uuid::new_v4(), "msg");
        assert_eq!(matches.len(), 0);

        mgr.set_active(handle, true);
        let matches = mgr.get_matching_listeners(0, "Test", Uuid::new_v4(), "msg");
        assert_eq!(matches.len(), 1);
    }
}
