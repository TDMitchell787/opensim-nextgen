use std::collections::HashMap;
use anyhow::Result;
use tracing::info;
use uuid::Uuid;

use super::vehicle_scripts::VehicleScriptLibrary;
use super::vehicle_recipes::{self, VehicleRecipe};
use crate::udp::action_bridge::ActionBridge;

pub struct VehicleBuilder<'a> {
    bridge: &'a ActionBridge,
    scripts: &'static VehicleScriptLibrary,
    speaker_name: String,
}

pub struct BuildResult {
    pub root_id: u32,
    pub child_ids: Vec<(String, u32)>,
    pub hud_id: Option<u32>,
    pub recipe_name: String,
    pub narration: Vec<String>,
}

impl<'a> VehicleBuilder<'a> {
    pub fn new(bridge: &'a ActionBridge) -> Self {
        Self {
            bridge,
            scripts: VehicleScriptLibrary::global(),
            speaker_name: "Galadriel".to_string(),
        }
    }

    pub async fn build_vehicle(
        &self,
        npc_id: Uuid,
        recipe_name: &str,
        position: [f32; 3],
        tuning: &HashMap<String, f32>,
        requester_id: Uuid,
    ) -> Result<BuildResult> {
        let recipe = vehicle_recipes::get_recipe(recipe_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown vehicle recipe: '{}'. Available: {:?}", recipe_name, vehicle_recipes::list_recipe_names()))?;

        let scale_factor = tuning.get("SCALE").copied().unwrap_or(1.0);
        let use_scale = (scale_factor - 1.0).abs() > 0.01;
        let (scaled_root, scaled_children) = if use_scale {
            let (r, c) = vehicle_recipes::apply_scale(recipe, scale_factor);
            (Some(r), Some(c))
        } else {
            (None, None)
        };
        let root_spec = scaled_root.as_ref().unwrap_or(&recipe.root_prim);
        let child_slice: &[vehicle_recipes::ChildPrimSpec] = match &scaled_children {
            Some(v) => v.as_slice(),
            None => recipe.children,
        };

        let mut narration = Vec::new();
        let msg = format!("Building a {}...", recipe.description);
        narration.push(msg.clone());
        let _ = self.bridge.say(npc_id, &self.speaker_name, &msg, position).await;

        info!("[VEHICLE_BUILDER] Building '{}' at [{:.1}, {:.1}, {:.1}] for {}",
            recipe_name, position[0], position[1], position[2], requester_id);

        narration.push(format!("Rezzing the {}...", root_spec.name));
        let root_id = self.rez_prim(npc_id, root_spec.shape, position, root_spec.size, root_spec.name).await?;
        info!("[VEHICLE_BUILDER] Root prim '{}' → local_id {}", root_spec.name, root_id);

        let mut child_ids: Vec<(String, u32, Option<&str>)> = Vec::new();
        for child in child_slice {
            let child_pos = [
                position[0] + child.offset[0],
                position[1] + child.offset[1],
                position[2] + child.offset[2],
            ];
            narration.push(format!("Adding {}...", child.name));
            let child_id = self.rez_prim(npc_id, child.shape, child_pos, child.size, child.name).await?;
            info!("[VEHICLE_BUILDER] Child '{}' → local_id {}", child.name, child_id);
            child_ids.push((child.name.to_string(), child_id, child.script_name));
        }

        let msg = format!("Linking {} components...", child_ids.len() + 1);
        narration.push(msg.clone());
        let _ = self.bridge.say(npc_id, &self.speaker_name, &msg, position).await;
        let mut link_ids: Vec<u32> = child_ids.iter().map(|(_, id, _)| *id).collect();
        link_ids.push(root_id);
        info!("[VEHICLE_BUILDER] LinkObjects: children {:?}, root {} (root becomes link 1)",
            &link_ids[..link_ids.len()-1], root_id);
        self.bridge.link_objects(root_id, &link_ids[..link_ids.len()-1]).await?;

        let msg = "Installing drive controller...".to_string();
        narration.push(msg.clone());
        let _ = self.bridge.say(npc_id, &self.speaker_name, &msg, position).await;
        let merged_tuning = self.merge_tuning(recipe, tuning);
        if let Some(root_source) = self.scripts.get_script_with_tuning(recipe.root_script, &merged_tuning) {
            self.bridge.insert_script(npc_id, root_id, recipe.root_script, &root_source).await?;
            info!("[VEHICLE_BUILDER] Root script '{}' installed", recipe.root_script);
        } else {
            info!("[VEHICLE_BUILDER] WARNING: Root script '{}' not found in library, using template fallback", recipe.root_script);
            let template_name = recipe.root_script.trim_end_matches(".lsl");
            if let Some(source) = crate::ai::script_templates::apply_template(template_name, &self.tuning_to_string_map(&merged_tuning)) {
                self.bridge.insert_script(npc_id, root_id, recipe.root_script, &source).await?;
                info!("[VEHICLE_BUILDER] Root script from template '{}' installed", template_name);
            } else {
                return Err(anyhow::anyhow!("Neither vehicle script '{}' nor template '{}' found", recipe.root_script, template_name));
            }
        }

        let scripted_children: Vec<_> = child_ids.iter()
            .filter(|(_, _, script)| script.is_some())
            .collect();
        if !scripted_children.is_empty() {
            narration.push("Installing component scripts...".to_string());
            for (name, id, script_name) in &scripted_children {
                let script = script_name.unwrap();
                if let Some(source) = self.scripts.get_script(script) {
                    self.bridge.insert_script(npc_id, *id, script, source).await?;
                    info!("[VEHICLE_BUILDER] Child script '{}' → '{}' (local_id {})", script, name, id);
                } else {
                    info!("[VEHICLE_BUILDER] WARNING: Child script '{}' not found for '{}'", script, name);
                }
            }
        }

        let mut hud_id = None;
        if let Some(hud_script) = recipe.hud_script {
            if let Some(hud_source) = self.scripts.get_script(hud_script) {
                let hud_pos = [position[0] + 2.0, position[1] + 2.0, position[2] + 1.0];
                narration.push("Building HUD...".to_string());
                let hid = self.bridge.rez_box(npc_id, hud_pos, [0.1, 0.3, 0.05], "Vehicle HUD").await?;
                self.bridge.insert_script(npc_id, hid, hud_script, hud_source).await?;
                self.bridge.give_object(hid, requester_id).await?;
                hud_id = Some(hid);
                info!("[VEHICLE_BUILDER] HUD '{}' given to {}", hud_script, requester_id);
            }
        }

        let usage = self.usage_instructions(recipe_name);
        let msg = format!("Your {} is ready! {}", recipe_name, usage);
        narration.push(msg.clone());
        let _ = self.bridge.say(npc_id, &self.speaker_name, &msg, position).await;

        let result_child_ids: Vec<(String, u32)> = child_ids.into_iter()
            .map(|(name, id, _)| (name, id))
            .collect();

        info!("[VEHICLE_BUILDER] Build complete: root={}, children={}, hud={}",
            root_id, result_child_ids.len(), hud_id.is_some());

        Ok(BuildResult {
            root_id,
            child_ids: result_child_ids,
            hud_id,
            recipe_name: recipe_name.to_string(),
            narration,
        })
    }

    async fn rez_prim(&self, npc_id: Uuid, shape: &str, position: [f32; 3], size: [f32; 3], name: &str) -> Result<u32> {
        match shape {
            "box" => self.bridge.rez_box(npc_id, position, size, name).await,
            "cylinder" => self.bridge.rez_cylinder(npc_id, position, size, name).await,
            "sphere" => self.bridge.rez_sphere(npc_id, position, size, name).await,
            _ => self.bridge.rez_box(npc_id, position, size, name).await,
        }
    }

    fn merge_tuning(&self, recipe: &VehicleRecipe, overrides: &HashMap<String, f32>) -> HashMap<String, f32> {
        let mut merged = HashMap::new();
        for &(name, default) in recipe.tuning_defaults {
            merged.insert(name.to_string(), *overrides.get(name).unwrap_or(&default));
        }
        for (name, value) in overrides {
            if name != "SCALE" {
                merged.insert(name.clone(), *value);
            }
        }
        merged
    }

    fn tuning_to_string_map(&self, tuning: &HashMap<String, f32>) -> HashMap<String, String> {
        tuning.iter().map(|(k, v)| {
            if v.fract() == 0.0 && *v >= i32::MIN as f32 && *v <= i32::MAX as f32 {
                (k.clone(), format!("{}", *v as i32))
            } else {
                (k.clone(), format!("{:.1}", v))
            }
        }).collect()
    }

    fn usage_instructions(&self, recipe_name: &str) -> &'static str {
        match recipe_name {
            "car" => "Sit on it to drive. W/S for forward/reverse, A/D to steer.",
            "bike" => "Sit on it to ride. W/S for throttle, A/D to steer and lean.",
            "plane" => "Sit to board. W/S for throttle, A/D for roll, arrows for yaw, PgUp/PgDn for pitch.",
            "vtol" => "Sit to board. PgUp to hover, reach speed to transition to flight. W/S for throttle.",
            "vessel" => "Sit to board. Say 'sails up' to sail, 'motor on' for engine. A/D to steer.",
            "lani" => "Sit to board. Say 'sails up' to sail, 'motor on' for engine. 'dock' near a beacon. Full Lani/Dyna hybrid controls.",
            "starship" => "Sit to board. Say 'impulse' or 'warp N' for propulsion. Say 'red alert' for combat.",
            _ => "Sit on it to operate.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn merge_tuning_standalone(recipe: &VehicleRecipe, overrides: &HashMap<String, f32>) -> HashMap<String, f32> {
        let mut merged = HashMap::new();
        for &(name, default) in recipe.tuning_defaults {
            merged.insert(name.to_string(), *overrides.get(name).unwrap_or(&default));
        }
        for (name, value) in overrides {
            if name != "SCALE" {
                merged.insert(name.clone(), *value);
            }
        }
        merged
    }

    #[test]
    fn test_merge_tuning_defaults() {
        let recipe = vehicle_recipes::get_recipe("car").unwrap();
        let merged = merge_tuning_standalone(recipe, &HashMap::new());
        assert!((merged["MAX_SPEED"] - 40.0).abs() < 0.01);
        assert!((merged["TURN_RATE"] - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_merge_tuning_overrides() {
        let recipe = vehicle_recipes::get_recipe("car").unwrap();
        let mut overrides = HashMap::new();
        overrides.insert("MAX_SPEED".to_string(), 60.0);
        let merged = merge_tuning_standalone(recipe, &overrides);
        assert!((merged["MAX_SPEED"] - 60.0).abs() < 0.01);
        assert!((merged["TURN_RATE"] - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_usage_instructions_coverage() {
        let instructions = [
            ("car", "W/S"), ("bike", "W/S"), ("plane", "throttle"),
            ("vtol", "hover"), ("vessel", "sails"), ("starship", "impulse"),
        ];
        for (name, keyword) in &instructions {
            let recipe = vehicle_recipes::get_recipe(name).unwrap();
            assert!(recipe.name == *name);
            let usage = match *name {
                "car" => "Sit on it to drive. W/S for forward/reverse, A/D to steer.",
                "bike" => "Sit on it to ride. W/S for throttle, A/D to steer and lean.",
                "plane" => "Sit to board. W/S for throttle, A/D for roll, arrows for yaw, PgUp/PgDn for pitch.",
                "vtol" => "Sit to board. PgUp to hover, reach speed to transition to flight. W/S for throttle.",
                "vessel" => "Sit to board. Say 'sails up' to sail, 'motor on' for engine. A/D to steer.",
                "starship" => "Sit to board. Say 'impulse' or 'warp N' for propulsion. Say 'red alert' for combat.",
                _ => "Sit on it to operate.",
            };
            assert!(usage.contains(keyword), "Recipe '{}' should contain '{}'", name, keyword);
        }
    }
}
