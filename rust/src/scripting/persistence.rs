use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

use super::lsl_types::{LSLRotation, LSLVector};
use super::LSLValue;

pub const DEFAULT_HEAP_LIMIT: usize = 65536;
pub const MAX_HEAP_LIMIT: usize = 1048576;

#[derive(Debug, Clone)]
pub struct ScriptStateData {
    pub script_id: Uuid,
    pub object_id: Uuid,
    pub item_id: Uuid,
    pub region_id: Uuid,
    pub current_state: String,
    pub running: bool,
    pub permissions_granted: u32,
    pub permissions_key: Uuid,
    pub min_event_delay: f64,
    pub global_vars: HashMap<String, LSLValue>,
}

impl ScriptStateData {
    pub fn new(script_id: Uuid, object_id: Uuid, item_id: Uuid, region_id: Uuid) -> Self {
        Self {
            script_id,
            object_id,
            item_id,
            region_id,
            current_state: "default".to_string(),
            running: true,
            permissions_granted: 0,
            permissions_key: Uuid::nil(),
            min_event_delay: 0.0,
            global_vars: HashMap::new(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(512);
        buf.extend_from_slice(self.script_id.as_bytes());
        buf.extend_from_slice(self.object_id.as_bytes());
        buf.extend_from_slice(self.item_id.as_bytes());
        buf.extend_from_slice(self.region_id.as_bytes());

        let state_bytes = self.current_state.as_bytes();
        buf.extend_from_slice(&(state_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(state_bytes);

        buf.push(if self.running { 1 } else { 0 });
        buf.extend_from_slice(&self.permissions_granted.to_le_bytes());
        buf.extend_from_slice(self.permissions_key.as_bytes());
        buf.extend_from_slice(&self.min_event_delay.to_le_bytes());

        buf.extend_from_slice(&(self.global_vars.len() as u32).to_le_bytes());
        for (name, value) in &self.global_vars {
            let name_bytes = name.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(name_bytes);
            serialize_lsl_value(&mut buf, value);
        }

        buf
    }

    pub fn deserialize(data: &[u8]) -> Option<Self> {
        if data.len() < 64 {
            return None;
        }
        let mut pos = 0;

        let script_id = Uuid::from_slice(&data[pos..pos + 16]).ok()?;
        pos += 16;
        let object_id = Uuid::from_slice(&data[pos..pos + 16]).ok()?;
        pos += 16;
        let item_id = Uuid::from_slice(&data[pos..pos + 16]).ok()?;
        pos += 16;
        let region_id = Uuid::from_slice(&data[pos..pos + 16]).ok()?;
        pos += 16;

        let state_len = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;
        let current_state = String::from_utf8(data[pos..pos + state_len].to_vec()).ok()?;
        pos += state_len;

        let running = data[pos] != 0;
        pos += 1;
        let permissions_granted = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
        pos += 4;
        let permissions_key = Uuid::from_slice(&data[pos..pos + 16]).ok()?;
        pos += 16;
        let min_event_delay = f64::from_le_bytes(data[pos..pos + 8].try_into().ok()?);
        pos += 8;

        let var_count = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;

        let mut global_vars = HashMap::new();
        for _ in 0..var_count {
            if pos + 4 > data.len() {
                break;
            }
            let name_len = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;
            if pos + name_len > data.len() {
                break;
            }
            let name = String::from_utf8(data[pos..pos + name_len].to_vec()).ok()?;
            pos += name_len;
            let (value, new_pos) = deserialize_lsl_value(data, pos)?;
            pos = new_pos;
            global_vars.insert(name, value);
        }

        Some(ScriptStateData {
            script_id,
            object_id,
            item_id,
            region_id,
            current_state,
            running,
            permissions_granted,
            permissions_key,
            min_event_delay,
            global_vars,
        })
    }
}

fn serialize_lsl_value(buf: &mut Vec<u8>, value: &LSLValue) {
    match value {
        LSLValue::Integer(n) => {
            buf.push(0);
            buf.extend_from_slice(&n.to_le_bytes());
        }
        LSLValue::Float(f) => {
            buf.push(1);
            buf.extend_from_slice(&(*f as f64).to_le_bytes());
        }
        LSLValue::String(s) => {
            buf.push(2);
            let bytes = s.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(bytes);
        }
        LSLValue::Key(k) => {
            buf.push(3);
            buf.extend_from_slice(k.as_bytes());
        }
        LSLValue::Vector(v) => {
            buf.push(4);
            buf.extend_from_slice(&(v.x as f64).to_le_bytes());
            buf.extend_from_slice(&(v.y as f64).to_le_bytes());
            buf.extend_from_slice(&(v.z as f64).to_le_bytes());
        }
        LSLValue::Rotation(r) => {
            buf.push(5);
            buf.extend_from_slice(&(r.x as f64).to_le_bytes());
            buf.extend_from_slice(&(r.y as f64).to_le_bytes());
            buf.extend_from_slice(&(r.z as f64).to_le_bytes());
            buf.extend_from_slice(&(r.s as f64).to_le_bytes());
        }
        LSLValue::List(items) => {
            buf.push(6);
            buf.extend_from_slice(&(items.len() as u32).to_le_bytes());
            for item in items {
                serialize_lsl_value(buf, item);
            }
        }
    }
}

fn deserialize_lsl_value(data: &[u8], pos: usize) -> Option<(LSLValue, usize)> {
    if pos >= data.len() {
        return None;
    }
    let tag = data[pos];
    let mut p = pos + 1;

    match tag {
        0 => {
            if p + 4 > data.len() {
                return None;
            }
            let n = i32::from_le_bytes(data[p..p + 4].try_into().ok()?);
            Some((LSLValue::Integer(n), p + 4))
        }
        1 => {
            if p + 8 > data.len() {
                return None;
            }
            let f = f64::from_le_bytes(data[p..p + 8].try_into().ok()?);
            Some((LSLValue::Float(f as f32), p + 8))
        }
        2 => {
            if p + 4 > data.len() {
                return None;
            }
            let len = u32::from_le_bytes(data[p..p + 4].try_into().ok()?) as usize;
            p += 4;
            if p + len > data.len() {
                return None;
            }
            let s = String::from_utf8(data[p..p + len].to_vec()).ok()?;
            Some((LSLValue::String(s), p + len))
        }
        3 => {
            if p + 16 > data.len() {
                return None;
            }
            let k = Uuid::from_slice(&data[p..p + 16]).ok()?;
            Some((LSLValue::Key(k), p + 16))
        }
        4 => {
            if p + 24 > data.len() {
                return None;
            }
            let x = f64::from_le_bytes(data[p..p + 8].try_into().ok()?);
            let y = f64::from_le_bytes(data[p + 8..p + 16].try_into().ok()?);
            let z = f64::from_le_bytes(data[p + 16..p + 24].try_into().ok()?);
            Some((
                LSLValue::Vector(LSLVector::new(x as f32, y as f32, z as f32)),
                p + 24,
            ))
        }
        5 => {
            if p + 32 > data.len() {
                return None;
            }
            let x = f64::from_le_bytes(data[p..p + 8].try_into().ok()?);
            let y = f64::from_le_bytes(data[p + 8..p + 16].try_into().ok()?);
            let z = f64::from_le_bytes(data[p + 16..p + 24].try_into().ok()?);
            let s = f64::from_le_bytes(data[p + 24..p + 32].try_into().ok()?);
            Some((
                LSLValue::Rotation(LSLRotation {
                    x: x as f32,
                    y: y as f32,
                    z: z as f32,
                    s: s as f32,
                }),
                p + 32,
            ))
        }
        6 => {
            if p + 4 > data.len() {
                return None;
            }
            let count = u32::from_le_bytes(data[p..p + 4].try_into().ok()?) as usize;
            p += 4;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let (item, new_p) = deserialize_lsl_value(data, p)?;
                p = new_p;
                items.push(item);
            }
            Some((LSLValue::List(items), p))
        }
        _ => None,
    }
}

pub fn compute_source_hash(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn compute_heap_size(value: &LSLValue) -> usize {
    match value {
        LSLValue::Integer(_) => 4,
        LSLValue::Float(_) => 8,
        LSLValue::String(s) => 24 + s.len(),
        LSLValue::Key(_) => 16,
        LSLValue::Vector(_) => 12,
        LSLValue::Rotation(_) => 16,
        LSLValue::List(items) => 24 + items.iter().map(compute_heap_size).sum::<usize>(),
        _ => 0,
    }
}

pub fn total_heap_usage(globals: &HashMap<String, LSLValue>) -> usize {
    globals
        .iter()
        .map(|(k, v)| k.len() + compute_heap_size(v))
        .sum()
}

pub fn check_heap_limit(globals: &HashMap<String, LSLValue>, limit: usize) -> bool {
    total_heap_usage(globals) <= limit
}

pub struct CompiledScriptCache {
    entries: HashMap<String, CacheEntry>,
}

struct CacheEntry {
    source_hash: String,
    backend: String,
    compiled_data: Vec<u8>,
}

impl CompiledScriptCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, source_hash: &str, backend: &str) -> Option<&[u8]> {
        let key = format!("{}:{}", source_hash, backend);
        self.entries.get(&key).map(|e| e.compiled_data.as_slice())
    }

    pub fn put(&mut self, source_hash: String, backend: String, compiled_data: Vec<u8>) {
        let key = format!("{}:{}", source_hash, backend);
        self.entries.insert(
            key,
            CacheEntry {
                source_hash,
                backend,
                compiled_data,
            },
        );
    }

    pub fn remove(&mut self, source_hash: &str, backend: &str) {
        let key = format!("{}:{}", source_hash, backend);
        self.entries.remove(&key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

pub struct ScriptStatePersistence {
    states: HashMap<Uuid, ScriptStateData>,
    save_interval_secs: u64,
}

impl ScriptStatePersistence {
    pub fn new(save_interval_secs: u64) -> Self {
        Self {
            states: HashMap::new(),
            save_interval_secs,
        }
    }

    pub fn save_state(&mut self, state: ScriptStateData) {
        self.states.insert(state.script_id, state);
    }

    pub fn load_state(&self, script_id: Uuid) -> Option<&ScriptStateData> {
        self.states.get(&script_id)
    }

    pub fn remove_state(&mut self, script_id: Uuid) {
        self.states.remove(&script_id);
    }

    pub fn get_all_for_region(&self, region_id: Uuid) -> Vec<&ScriptStateData> {
        self.states
            .values()
            .filter(|s| s.region_id == region_id)
            .collect()
    }

    pub fn get_all_for_object(&self, object_id: Uuid) -> Vec<&ScriptStateData> {
        self.states
            .values()
            .filter(|s| s.object_id == object_id)
            .collect()
    }

    pub fn save_interval(&self) -> u64 {
        self.save_interval_secs
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn serialize_all(&self) -> Vec<(Uuid, Vec<u8>)> {
        self.states
            .iter()
            .map(|(id, state)| (*id, state.serialize()))
            .collect()
    }

    pub fn deserialize_and_load(&mut self, entries: Vec<(Uuid, Vec<u8>)>) -> usize {
        let mut loaded = 0;
        for (_id, data) in entries {
            if let Some(state) = ScriptStateData::deserialize(&data) {
                self.states.insert(state.script_id, state);
                loaded += 1;
            }
        }
        loaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_state() {
        let mut state = ScriptStateData::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        state.current_state = "walking".to_string();
        state.running = true;
        state.permissions_granted = 0x1234;
        state
            .global_vars
            .insert("counter".to_string(), LSLValue::Integer(42));
        state
            .global_vars
            .insert("name".to_string(), LSLValue::String("test".to_string()));
        state.global_vars.insert(
            "pos".to_string(),
            LSLValue::Vector(LSLVector::new(1.0, 2.0, 3.0)),
        );

        let data = state.serialize();
        let restored = ScriptStateData::deserialize(&data).expect("deserialize failed");

        assert_eq!(restored.script_id, state.script_id);
        assert_eq!(restored.current_state, "walking");
        assert_eq!(restored.running, true);
        assert_eq!(restored.permissions_granted, 0x1234);
        assert_eq!(restored.global_vars.len(), 3);

        match restored.global_vars.get("counter") {
            Some(LSLValue::Integer(42)) => {}
            _ => panic!("counter mismatch"),
        }
    }

    #[test]
    fn test_serialize_list_value() {
        let mut state = ScriptStateData::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        state.global_vars.insert(
            "items".to_string(),
            LSLValue::List(vec![
                LSLValue::Integer(1),
                LSLValue::String("two".to_string()),
                LSLValue::Float(3.0),
            ]),
        );

        let data = state.serialize();
        let restored = ScriptStateData::deserialize(&data).unwrap();
        match restored.global_vars.get("items") {
            Some(LSLValue::List(l)) => assert_eq!(l.len(), 3),
            _ => panic!("list mismatch"),
        }
    }

    #[test]
    fn test_source_hash() {
        let hash1 = compute_source_hash("default { state_entry() { llSay(0, \"Hello\"); } }");
        let hash2 = compute_source_hash("default { state_entry() { llSay(0, \"Hello\"); } }");
        let hash3 = compute_source_hash("default { state_entry() { llSay(0, \"World\"); } }");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_heap_size() {
        assert_eq!(compute_heap_size(&LSLValue::Integer(0)), 4);
        assert_eq!(compute_heap_size(&LSLValue::Float(0.0)), 8);
        assert_eq!(
            compute_heap_size(&LSLValue::String("hello".to_string())),
            29
        );
        assert_eq!(compute_heap_size(&LSLValue::Key(Uuid::nil())), 16);
        assert_eq!(
            compute_heap_size(&LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0))),
            12
        );
        assert_eq!(
            compute_heap_size(&LSLValue::Rotation(LSLRotation {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                s: 1.0
            })),
            16
        );
    }

    #[test]
    fn test_heap_limit_check() {
        let mut globals = HashMap::new();
        globals.insert("x".to_string(), LSLValue::Integer(0));
        assert!(check_heap_limit(&globals, 100));
        assert!(check_heap_limit(&globals, DEFAULT_HEAP_LIMIT));

        globals.insert("big".to_string(), LSLValue::String("a".repeat(100000)));
        assert!(!check_heap_limit(&globals, DEFAULT_HEAP_LIMIT));
    }

    #[test]
    fn test_compiled_cache() {
        let mut cache = CompiledScriptCache::new();
        let hash = "abc123".to_string();
        cache.put(hash.clone(), "treewalk".to_string(), vec![1, 2, 3]);
        assert_eq!(cache.entry_count(), 1);

        let data = cache.get("abc123", "treewalk").unwrap();
        assert_eq!(data, &[1, 2, 3]);

        assert!(cache.get("abc123", "bytecode").is_none());

        cache.remove("abc123", "treewalk");
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_state_persistence() {
        let mut persistence = ScriptStatePersistence::new(300);
        let region_id = Uuid::new_v4();

        let state1 =
            ScriptStateData::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), region_id);
        let state2 =
            ScriptStateData::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), region_id);
        let other_region = ScriptStateData::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        let s1_id = state1.script_id;
        persistence.save_state(state1);
        persistence.save_state(state2);
        persistence.save_state(other_region);

        assert_eq!(persistence.state_count(), 3);
        assert_eq!(persistence.get_all_for_region(region_id).len(), 2);

        persistence.remove_state(s1_id);
        assert_eq!(persistence.state_count(), 2);
    }

    #[test]
    fn test_bulk_serialize_deserialize() {
        let mut persistence = ScriptStatePersistence::new(300);

        let mut state = ScriptStateData::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        state
            .global_vars
            .insert("val".to_string(), LSLValue::Integer(99));
        persistence.save_state(state);

        let serialized = persistence.serialize_all();
        assert_eq!(serialized.len(), 1);

        let mut new_persistence = ScriptStatePersistence::new(300);
        let loaded = new_persistence.deserialize_and_load(serialized);
        assert_eq!(loaded, 1);
        assert_eq!(new_persistence.state_count(), 1);
    }
}
