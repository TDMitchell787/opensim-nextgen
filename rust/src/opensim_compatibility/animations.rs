//! Avatar Animations - Standard animation UUID mappings
//!
//! This module provides loading and lookup of standard Second Life/OpenSim
//! animation UUIDs from the avataranimations.xml data file.

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

static GLOBAL_ANIMATIONS: OnceLock<Arc<RwLock<AnimationManager>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub uuid: Uuid,
    pub state: Option<String>,
}

#[derive(Debug)]
pub struct AnimationManager {
    animations_by_name: HashMap<String, Animation>,
    animations_by_uuid: HashMap<Uuid, Animation>,
    loaded: bool,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations_by_name: HashMap::new(),
            animations_by_uuid: HashMap::new(),
            loaded: false,
        }
    }

    pub fn load_from_file(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow!("Animation file not found: {:?}", path));
        }

        let content = std::fs::read_to_string(path)?;
        self.parse_animations_xml(&content)?;
        self.loaded = true;

        info!(
            "Loaded {} animations from {:?}",
            self.animations_by_name.len(),
            path
        );
        Ok(())
    }

    fn parse_animations_xml(&mut self, content: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with("<animation ") {
                continue;
            }

            if let Some(animation) = self.parse_animation_element(line) {
                self.animations_by_name
                    .insert(animation.name.clone(), animation.clone());
                self.animations_by_uuid.insert(animation.uuid, animation);
            }
        }

        Ok(())
    }

    fn parse_animation_element(&self, line: &str) -> Option<Animation> {
        let name = self.extract_attribute(line, "name")?;
        let state_str = self.extract_attribute(line, "state");

        let uuid_start = line.find('>')?;
        let uuid_end = line.find("</animation>")?;
        let uuid_str = line[uuid_start + 1..uuid_end].trim();

        let uuid = Uuid::parse_str(uuid_str).ok()?;

        let state = state_str.filter(|s| !s.is_empty());

        Some(Animation { name, uuid, state })
    }

    fn extract_attribute(&self, line: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        let start = line.find(&pattern)? + pattern.len();
        let rest = &line[start..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Animation> {
        self.animations_by_name
            .get(&name.to_uppercase())
            .or_else(|| self.animations_by_name.get(name))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&Animation> {
        self.animations_by_uuid.get(uuid)
    }

    pub fn get_uuid_by_name(&self, name: &str) -> Option<Uuid> {
        self.get_by_name(name).map(|a| a.uuid)
    }

    pub fn get_name_by_uuid(&self, uuid: &Uuid) -> Option<&str> {
        self.get_by_uuid(uuid).map(|a| a.name.as_str())
    }

    pub fn get_state_animation(&self, state: &str) -> Option<&Animation> {
        self.animations_by_name
            .values()
            .find(|a| a.state.as_deref() == Some(state))
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn animation_count(&self) -> usize {
        self.animations_by_name.len()
    }

    pub fn all_animations(&self) -> impl Iterator<Item = &Animation> {
        self.animations_by_name.values()
    }
}

pub fn init_global_animations(bin_path: &Path) -> Result<()> {
    let manager = GLOBAL_ANIMATIONS.get_or_init(|| Arc::new(RwLock::new(AnimationManager::new())));

    let mut guard = manager.write();
    if guard.is_loaded() {
        return Ok(());
    }

    let animations_path = bin_path.join("data/avataranimations.xml");
    guard.load_from_file(&animations_path)
}

pub fn get_global_animation_manager() -> Option<Arc<RwLock<AnimationManager>>> {
    GLOBAL_ANIMATIONS.get().cloned()
}

pub fn get_animation_uuid(name: &str) -> Option<Uuid> {
    let manager = GLOBAL_ANIMATIONS.get()?;
    let guard = manager.read();
    guard.get_uuid_by_name(name)
}

pub fn get_animation_name(uuid: &Uuid) -> Option<String> {
    let manager = GLOBAL_ANIMATIONS.get()?;
    let guard = manager.read();
    guard.get_name_by_uuid(uuid).map(|s| s.to_string())
}

pub fn get_state_animation_uuid(state: &str) -> Option<Uuid> {
    let manager = GLOBAL_ANIMATIONS.get()?;
    let guard = manager.read();
    guard.get_state_animation(state).map(|a| a.uuid)
}

pub mod default_animations {
    use uuid::Uuid;

    pub const STAND: &str = "2408fe9e-df1d-1d7d-f4ff-1384fa7b350f";
    pub const WALK: &str = "6ed24bd8-91aa-4b12-ccc7-c97c857ab4e0";
    pub const RUN: &str = "05ddbff8-aaa9-92a1-2b74-8fe77a29b445";
    pub const FLY: &str = "aec4610c-757f-bc4e-c092-c6e9caf18daf";
    pub const SIT: &str = "1a5fe8ac-a804-8a5d-7cbd-56bd83184568";
    pub const SIT_GROUND: &str = "1c7600d6-661f-b87b-efe2-d7421eb93c86";
    pub const CROUCH: &str = "201f3fdf-cb1f-dbec-201f-7333e328ae7c";
    pub const CROUCHWALK: &str = "47f5f6fb-22e5-ae44-f871-73aaaf4a6022";
    pub const FALLDOWN: &str = "666307d9-a860-572d-6fd4-c3ab8865c094";
    pub const HOVER: &str = "4ae8016b-31b9-03bb-c401-b1ea941db41d";
    pub const HOVER_UP: &str = "62c5de58-cb33-5743-3d07-9e4cd4352864";
    pub const HOVER_DOWN: &str = "20f063ea-8306-2562-0b07-5c853b37b31e";
    pub const JUMP: &str = "2305bd75-1ca9-b03b-1faa-b176b8a8c49e";
    pub const PREJUMP: &str = "7a4e87fe-de39-6fcb-6223-024b00893244";
    pub const LAND: &str = "7a17b059-12b2-41b1-570a-186368b6aa6f";
    pub const SOFT_LAND: &str = "f4f00d6e-b9fe-9292-f4cb-0ae06ea58d57";
    pub const STANDUP: &str = "3da1d753-028a-5446-24f3-9c9b856d9422";
    pub const TURNLEFT: &str = "56e0ba0d-4a9f-7f27-6117-32f2ebbf6135";
    pub const TURNRIGHT: &str = "2d6daa51-3192-6794-8e2e-a15f8338ec30";
    pub const AWAY: &str = "fd037134-85d4-f241-72c6-4f42164fedee";
    pub const BUSY: &str = "efcf670c-2d18-8128-973a-034ebc806b67";

    pub fn stand_uuid() -> Uuid {
        Uuid::parse_str(STAND).unwrap()
    }

    pub fn walk_uuid() -> Uuid {
        Uuid::parse_str(WALK).unwrap()
    }

    pub fn run_uuid() -> Uuid {
        Uuid::parse_str(RUN).unwrap()
    }

    pub fn fly_uuid() -> Uuid {
        Uuid::parse_str(FLY).unwrap()
    }

    pub fn sit_uuid() -> Uuid {
        Uuid::parse_str(SIT).unwrap()
    }

    pub fn crouch_uuid() -> Uuid {
        Uuid::parse_str(CROUCH).unwrap()
    }

    pub fn hover_uuid() -> Uuid {
        Uuid::parse_str(HOVER).unwrap()
    }

    pub fn jump_uuid() -> Uuid {
        Uuid::parse_str(JUMP).unwrap()
    }

    pub fn away_uuid() -> Uuid {
        Uuid::parse_str(AWAY).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_xml() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"<?xml version="1.0" encoding="iso-8859-1"?>"#).unwrap();
        writeln!(file, r#"<animations>"#).unwrap();
        writeln!(file, r#"  <animation name="STAND" state="Standing">2408fe9e-df1d-1d7d-f4ff-1384fa7b350f</animation>"#).unwrap();
        writeln!(file, r#"  <animation name="WALK" state="Walking">6ed24bd8-91aa-4b12-ccc7-c97c857ab4e0</animation>"#).unwrap();
        writeln!(file, r#"  <animation name="RUN" state="Running">05ddbff8-aaa9-92a1-2b74-8fe77a29b445</animation>"#).unwrap();
        writeln!(file, r#"  <animation name="DANCE1" state="">b68a3d7c-de9e-fc87-eec8-543d787e5b0d</animation>"#).unwrap();
        writeln!(file, r#"</animations>"#).unwrap();
        file
    }

    #[test]
    fn test_load_animations() {
        let file = create_test_xml();
        let mut manager = AnimationManager::new();
        manager.load_from_file(file.path()).unwrap();

        assert!(manager.is_loaded());
        assert_eq!(manager.animation_count(), 4);
    }

    #[test]
    fn test_get_by_name() {
        let file = create_test_xml();
        let mut manager = AnimationManager::new();
        manager.load_from_file(file.path()).unwrap();

        let stand = manager.get_by_name("STAND").unwrap();
        assert_eq!(stand.name, "STAND");
        assert_eq!(stand.state, Some("Standing".to_string()));
    }

    #[test]
    fn test_get_uuid_by_name() {
        let file = create_test_xml();
        let mut manager = AnimationManager::new();
        manager.load_from_file(file.path()).unwrap();

        let uuid = manager.get_uuid_by_name("WALK").unwrap();
        assert_eq!(uuid.to_string(), "6ed24bd8-91aa-4b12-ccc7-c97c857ab4e0");
    }

    #[test]
    fn test_get_by_uuid() {
        let file = create_test_xml();
        let mut manager = AnimationManager::new();
        manager.load_from_file(file.path()).unwrap();

        let uuid = Uuid::parse_str("05ddbff8-aaa9-92a1-2b74-8fe77a29b445").unwrap();
        let anim = manager.get_by_uuid(&uuid).unwrap();
        assert_eq!(anim.name, "RUN");
    }

    #[test]
    fn test_get_state_animation() {
        let file = create_test_xml();
        let mut manager = AnimationManager::new();
        manager.load_from_file(file.path()).unwrap();

        let anim = manager.get_state_animation("Walking").unwrap();
        assert_eq!(anim.name, "WALK");
    }

    #[test]
    fn test_empty_state() {
        let file = create_test_xml();
        let mut manager = AnimationManager::new();
        manager.load_from_file(file.path()).unwrap();

        let dance = manager.get_by_name("DANCE1").unwrap();
        assert!(dance.state.is_none());
    }

    #[test]
    fn test_default_animation_uuids() {
        assert_eq!(
            default_animations::STAND,
            "2408fe9e-df1d-1d7d-f4ff-1384fa7b350f"
        );
        assert_eq!(
            default_animations::WALK,
            "6ed24bd8-91aa-4b12-ccc7-c97c857ab4e0"
        );

        let stand_uuid = default_animations::stand_uuid();
        assert_eq!(
            stand_uuid.to_string(),
            "2408fe9e-df1d-1d7d-f4ff-1384fa7b350f"
        );
    }
}
