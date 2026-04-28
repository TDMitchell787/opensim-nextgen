use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct NpcBehaviorConfig {
    #[serde(default = "default_behavior")]
    pub behavior: String,
    #[serde(default = "default_wander_radius")]
    pub wander_radius: f32,
    #[serde(default = "default_wander_pause")]
    pub wander_pause: [f32; 2],
    #[serde(default)]
    pub patrol_points: Vec<[f32; 3]>,
    #[serde(default = "default_patrol_pause")]
    pub patrol_pause: f32,
    #[serde(default = "default_true")]
    pub patrol_loop: bool,
    #[serde(default)]
    pub patrol_say: Vec<String>,
    #[serde(default = "default_greet_radius")]
    pub greet_radius: f32,
    #[serde(default)]
    pub greet_message: String,
    #[serde(default = "default_true")]
    pub face_speaker: bool,
    #[serde(default = "default_walk_speed")]
    pub walk_speed: String,
    #[serde(default = "default_arrival_threshold")]
    pub arrival_threshold: f32,
    #[serde(default = "default_follow_distance")]
    pub follow_distance: f32,
    #[serde(default = "default_proximity_sleep_radius")]
    pub proximity_sleep_radius: f32,
}

fn default_behavior() -> String {
    "idle".to_string()
}
fn default_wander_radius() -> f32 {
    10.0
}
fn default_wander_pause() -> [f32; 2] {
    [15.0, 30.0]
}
fn default_patrol_pause() -> f32 {
    5.0
}
fn default_true() -> bool {
    true
}
fn default_greet_radius() -> f32 {
    15.0
}
fn default_walk_speed() -> String {
    "walk".to_string()
}
fn default_arrival_threshold() -> f32 {
    0.5
}
fn default_follow_distance() -> f32 {
    3.0
}
fn default_proximity_sleep_radius() -> f32 {
    50.0
}

impl NpcBehaviorConfig {
    pub fn speed_value(&self) -> f32 {
        match self.walk_speed.as_str() {
            "run" => 4.096,
            _ => 2.458,
        }
    }
}

#[derive(Debug, Deserialize)]
struct NpcConfigFile {
    npc: HashMap<String, NpcBehaviorConfig>,
}

pub fn load_npc_configs(
    roster: &[crate::ai::npc_avatar::NPCAvatar],
) -> HashMap<Uuid, NpcBehaviorConfig> {
    let config_path = std::path::Path::new("rust/npc_behaviors.toml");
    let file_configs: HashMap<String, NpcBehaviorConfig> = if config_path.exists() {
        match std::fs::read_to_string(config_path) {
            Ok(contents) => match toml::from_str::<NpcConfigFile>(&contents) {
                Ok(cfg) => {
                    tracing::info!(
                        "[BEHAVIOR] Loaded {} NPC configs from npc_behaviors.toml",
                        cfg.npc.len()
                    );
                    cfg.npc
                }
                Err(e) => {
                    tracing::warn!(
                        "[BEHAVIOR] Failed to parse npc_behaviors.toml: {} - using defaults",
                        e
                    );
                    HashMap::new()
                }
            },
            Err(e) => {
                tracing::warn!(
                    "[BEHAVIOR] Failed to read npc_behaviors.toml: {} - using defaults",
                    e
                );
                HashMap::new()
            }
        }
    } else {
        tracing::info!("[BEHAVIOR] No npc_behaviors.toml found - using role-based defaults");
        HashMap::new()
    };

    let mut result = HashMap::new();
    for npc in roster {
        let key = format!(
            "{}_{}",
            npc.first_name.to_lowercase(),
            npc.last_name.to_lowercase()
        );
        let config = file_configs
            .get(&key)
            .cloned()
            .unwrap_or_else(|| default_for_role(&npc.role));
        result.insert(npc.agent_id, config);
    }
    result
}

fn default_for_role(role: &crate::ai::npc_avatar::NPCRole) -> NpcBehaviorConfig {
    use crate::ai::npc_avatar::NPCRole;
    match role {
        NPCRole::Builder => NpcBehaviorConfig {
            behavior: "wander".to_string(),
            wander_radius: 8.0,
            wander_pause: [20.0, 40.0],
            greet_radius: 10.0,
            greet_message: "Hi there! Need something built?".to_string(),
            face_speaker: true,
            ..default_config()
        },
        NPCRole::Clothier => NpcBehaviorConfig {
            behavior: "idle".to_string(),
            greet_radius: 12.0,
            greet_message: "Welcome! I'm Zara - let me know if you need fashion advice!".to_string(),
            face_speaker: true,
            ..default_config()
        },
        NPCRole::Scripter => NpcBehaviorConfig {
            behavior: "wander".to_string(),
            wander_radius: 6.0,
            wander_pause: [25.0, 45.0],
            greet_radius: 10.0,
            greet_message: "Hey! I'm Reed, the scripting helper. Ask me about LSL!".to_string(),
            face_speaker: true,
            ..default_config()
        },
        NPCRole::Landscaper => NpcBehaviorConfig {
            behavior: "wander".to_string(),
            wander_radius: 12.0,
            wander_pause: [15.0, 30.0],
            greet_radius: 15.0,
            greet_message: "Hello! I'm Terra. Want to discuss terrain or landscaping?".to_string(),
            face_speaker: true,
            ..default_config()
        },
        NPCRole::Guide => NpcBehaviorConfig {
            behavior: "patrol".to_string(),
            patrol_points: vec![
                [128.0, 130.0, 25.0], [140.0, 130.0, 25.0],
                [140.0, 140.0, 25.0], [128.0, 140.0, 25.0],
            ],
            patrol_pause: 8.0,
            patrol_loop: true,
            patrol_say: vec![
                "This area is great for building!".to_string(),
                "Over here you can find the clothier and scripter.".to_string(),
                "Beautiful view from this corner!".to_string(),
                "Check out the landscaping in this area.".to_string(),
            ],
            greet_radius: 20.0,
            greet_message: "Welcome! I'm Nova, your guide. Aria builds, Zara does fashion, Reed scripts, Terra landscapes!".to_string(),
            face_speaker: true,
            ..default_config()
        },
        NPCRole::Director | NPCRole::Media => NpcBehaviorConfig {
            behavior: "idle".to_string(),
            greet_radius: 0.0,
            ..default_config()
        },
    }
}

fn default_config() -> NpcBehaviorConfig {
    NpcBehaviorConfig {
        behavior: default_behavior(),
        wander_radius: default_wander_radius(),
        wander_pause: default_wander_pause(),
        patrol_points: Vec::new(),
        patrol_pause: default_patrol_pause(),
        patrol_loop: default_true(),
        patrol_say: Vec::new(),
        greet_radius: default_greet_radius(),
        greet_message: String::new(),
        face_speaker: default_true(),
        walk_speed: default_walk_speed(),
        arrival_threshold: default_arrival_threshold(),
        follow_distance: default_follow_distance(),
        proximity_sleep_radius: default_proximity_sleep_radius(),
    }
}
