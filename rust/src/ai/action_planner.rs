use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use super::npc_avatar::{NPCAction, NPCResponse};
use crate::ai::ml_integration::llm_client::LocalLLMClient;
use crate::media::luxor::camera::{CameraPreset, CameraRig};
use crate::media::luxor::lighting::LightingPreset;
use crate::media::luxor::post_process::PostEffect;
use crate::media::luxor::raytracer::RenderQuality;
use crate::media::luxor::scene_capture::capture_scene;
use crate::media::luxor::video::VideoJob;
use crate::media::luxor::{
    LuxorDirector, OutputFormat, OutputSettings, ScreenSize, SnapshotRequest, VideoRequest,
};
use crate::mesh::encoder::{
    generate_box_mesh, generate_cylinder_mesh, generate_sphere_mesh, generate_torus_mesh,
};
use crate::udp::action_bridge::ActionBridge;

pub struct ActionPlanner {
    bridge: ActionBridge,
    llm_client: Option<Arc<LocalLLMClient>>,
}

impl ActionPlanner {
    pub fn new(bridge: ActionBridge) -> Self {
        Self {
            bridge,
            llm_client: None,
        }
    }

    pub fn with_llm(bridge: ActionBridge, llm_client: Option<Arc<LocalLLMClient>>) -> Self {
        Self { bridge, llm_client }
    }

    fn remap_id(llm_id: u32, created_ids: &[(String, u32)]) -> u32 {
        if llm_id >= 1 && (llm_id as usize) <= created_ids.len() {
            let real_id = created_ids[(llm_id as usize) - 1].1;
            info!(
                "[ACTION_PLANNER] Remapped LLM id {} → real local_id {} (\"{}\")",
                llm_id,
                real_id,
                created_ids[(llm_id as usize) - 1].0
            );
            real_id
        } else {
            llm_id
        }
    }

    pub async fn execute_response(
        &self,
        npc_id: Uuid,
        response: &NPCResponse,
    ) -> Result<(Vec<(String, u32)>, Vec<String>)> {
        self.execute_response_with_session(npc_id, response, None)
            .await
    }

    pub async fn execute_response_with_session(
        &self,
        npc_id: Uuid,
        response: &NPCResponse,
        session_object_uuids: Option<Vec<Uuid>>,
    ) -> Result<(Vec<(String, u32)>, Vec<String>)> {
        let mut created_ids = Vec::new();
        let mut failures = Vec::new();

        for action in &response.actions {
            match action {
                NPCAction::RezBox {
                    position,
                    scale,
                    name,
                } => match self.bridge.rez_box(npc_id, *position, *scale, name).await {
                    Ok(local_id) => {
                        created_ids.push((name.clone(), local_id));
                    }
                    Err(e) => {
                        let msg = format!("Failed to create '{}': {}", name, e);
                        info!("[ACTION_PLANNER] {}", msg);
                        failures.push(msg);
                    }
                },
                NPCAction::RezCylinder {
                    position,
                    scale,
                    name,
                } => {
                    match self
                        .bridge
                        .rez_cylinder(npc_id, *position, *scale, name)
                        .await
                    {
                        Ok(local_id) => {
                            created_ids.push((name.clone(), local_id));
                        }
                        Err(e) => {
                            let msg = format!("Failed to create '{}': {}", name, e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::RezSphere {
                    position,
                    scale,
                    name,
                } => {
                    match self
                        .bridge
                        .rez_sphere(npc_id, *position, *scale, name)
                        .await
                    {
                        Ok(local_id) => {
                            created_ids.push((name.clone(), local_id));
                        }
                        Err(e) => {
                            let msg = format!("Failed to create '{}': {}", name, e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::RezTorus {
                    position,
                    scale,
                    name,
                } => match self.bridge.rez_torus(npc_id, *position, *scale, name).await {
                    Ok(local_id) => {
                        created_ids.push((name.clone(), local_id));
                    }
                    Err(e) => {
                        let msg = format!("Failed to create '{}': {}", name, e);
                        info!("[ACTION_PLANNER] {}", msg);
                        failures.push(msg);
                    }
                },
                NPCAction::RezTube {
                    position,
                    scale,
                    name,
                } => match self.bridge.rez_tube(npc_id, *position, *scale, name).await {
                    Ok(local_id) => {
                        created_ids.push((name.clone(), local_id));
                    }
                    Err(e) => {
                        let msg = format!("Failed to create '{}': {}", name, e);
                        info!("[ACTION_PLANNER] {}", msg);
                        failures.push(msg);
                    }
                },
                NPCAction::RezRing {
                    position,
                    scale,
                    name,
                } => match self.bridge.rez_ring(npc_id, *position, *scale, name).await {
                    Ok(local_id) => {
                        created_ids.push((name.clone(), local_id));
                    }
                    Err(e) => {
                        let msg = format!("Failed to create '{}': {}", name, e);
                        info!("[ACTION_PLANNER] {}", msg);
                        failures.push(msg);
                    }
                },
                NPCAction::RezPrism {
                    position,
                    scale,
                    name,
                } => match self.bridge.rez_prism(npc_id, *position, *scale, name).await {
                    Ok(local_id) => {
                        created_ids.push((name.clone(), local_id));
                    }
                    Err(e) => {
                        let msg = format!("Failed to create '{}': {}", name, e);
                        info!("[ACTION_PLANNER] {}", msg);
                        failures.push(msg);
                    }
                },
                NPCAction::SetPosition { local_id, position } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Err(e) = self.bridge.set_object_position(real_id, *position).await {
                        info!("[ACTION_PLANNER] Failed to set position: {}", e);
                    }
                }
                NPCAction::SetRotation { local_id, rotation } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Err(e) = self.bridge.set_object_rotation(real_id, *rotation).await {
                        info!("[ACTION_PLANNER] Failed to set rotation: {}", e);
                    }
                }
                NPCAction::SetScale { local_id, scale } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Err(e) = self.bridge.set_object_scale(real_id, *scale).await {
                        info!("[ACTION_PLANNER] Failed to set scale: {}", e);
                    }
                }
                NPCAction::SetColor { local_id, color } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Err(e) = self.bridge.set_object_color(real_id, *color).await {
                        info!("[ACTION_PLANNER] Failed to set color: {}", e);
                    }
                }
                NPCAction::SetTexture {
                    local_id,
                    texture_uuid,
                } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Ok(uuid) = Uuid::parse_str(texture_uuid) {
                        if let Err(e) = self.bridge.set_object_texture(real_id, uuid).await {
                            info!("[ACTION_PLANNER] Failed to set texture: {}", e);
                        }
                    } else {
                        info!("[ACTION_PLANNER] Invalid texture UUID: {}", texture_uuid);
                    }
                }
                NPCAction::SetName { local_id, name } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Err(e) = self.bridge.set_object_name(real_id, name).await {
                        info!("[ACTION_PLANNER] Failed to set name: {}", e);
                    }
                }
                NPCAction::LinkObjects { root_id, child_ids } => {
                    let real_root = Self::remap_id(*root_id, &created_ids);
                    let real_children: Vec<u32> = child_ids
                        .iter()
                        .map(|id| Self::remap_id(*id, &created_ids))
                        .collect();
                    info!("[ACTION_PLANNER] LinkObjects: LLM ids root={} children={:?} → real root={} children={:?}",
                        root_id, child_ids, real_root, real_children);
                    if let Err(e) = self.bridge.link_objects(real_root, &real_children).await {
                        info!("[ACTION_PLANNER] Failed to link objects: {}", e);
                    }
                }
                NPCAction::DeleteObject { local_id } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Err(e) = self.bridge.delete_object(real_id).await {
                        info!("[ACTION_PLANNER] Failed to delete object: {}", e);
                    }
                }
                NPCAction::InsertScript {
                    local_id,
                    script_name,
                    script_source,
                } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Err(e) = self
                        .bridge
                        .insert_script(npc_id, real_id, script_name, script_source)
                        .await
                    {
                        let msg = format!("Failed to insert script '{}': {}", script_name, e);
                        info!("[ACTION_PLANNER] {}", msg);
                        failures.push(msg);
                    }
                }
                NPCAction::InsertTemplateScript {
                    local_id,
                    template_name,
                    params,
                } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Some(source) =
                        crate::ai::script_templates::apply_template(template_name, params)
                    {
                        if let Err(e) = self
                            .bridge
                            .insert_script(npc_id, real_id, template_name, &source)
                            .await
                        {
                            let msg =
                                format!("Failed to insert template '{}': {}", template_name, e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    } else {
                        let msg = format!("Unknown script template: {}", template_name);
                        info!("[ACTION_PLANNER] {}", msg);
                        failures.push(msg);
                    }
                }
                NPCAction::UpdateScript {
                    local_id,
                    script_name,
                    script_source,
                } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    if let Err(e) = self
                        .bridge
                        .update_script(npc_id, real_id, script_name, script_source)
                        .await
                    {
                        info!("[ACTION_PLANNER] Failed to update script: {}", e);
                    }
                }
                NPCAction::GiveObject {
                    local_id,
                    target_agent_id,
                } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    match self.bridge.give_object(real_id, *target_agent_id).await {
                        Ok(msg) => info!("[ACTION_PLANNER] Give object: {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to give object: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::ExportOar {
                    region_id,
                    filename,
                } => {
                    let actual_region = if *region_id == Uuid::nil() {
                        self.bridge.default_region_uuid
                    } else {
                        *region_id
                    };
                    let export_uuids = if !created_ids.is_empty() {
                        let scene_objs = self.bridge.scene_objects_ref();
                        let uuids: Vec<Uuid> = created_ids
                            .iter()
                            .filter_map(|(_, lid)| scene_objs.read().get(lid).map(|o| o.uuid))
                            .collect();
                        if uuids.is_empty() {
                            session_object_uuids.clone()
                        } else {
                            Some(uuids)
                        }
                    } else {
                        session_object_uuids.clone()
                    };
                    match self
                        .bridge
                        .export_oar(actual_region, filename, export_uuids)
                        .await
                    {
                        Ok(msg) => info!("[ACTION_PLANNER] OAR export: {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to export OAR: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::RezMesh {
                    geometry_type,
                    position,
                    scale,
                    name,
                } => {
                    let gt = geometry_type.to_lowercase();
                    let blender_template = match gt.as_str() {
                        "table" | "chair" | "shelf" | "arch" | "staircase" | "stone"
                        | "stone_ring" | "boulder" | "column" => Some(gt.as_str().to_string()),
                        "path" | "walkway" | "cobblestone" | "cobblestone_path"
                        | "serpentine_path" => Some("path".to_string()),
                        _ => None,
                    };
                    if let Some(template) = blender_template {
                        info!("[ACTION_PLANNER] RezMesh '{}' matches Blender template '{}' — auto-converting", geometry_type, template);
                        let params = std::collections::HashMap::new();
                        match self
                            .bridge
                            .blender_generate(npc_id, &template, &params, name, *position)
                            .await
                        {
                            Ok(local_id) => {
                                created_ids.push((name.clone(), local_id));
                            }
                            Err(e) => {
                                let msg = format!(
                                    "Failed to generate '{}' (auto-template '{}'): {}",
                                    name, template, e
                                );
                                info!("[ACTION_PLANNER] {}", msg);
                                failures.push(msg);
                            }
                        }
                    } else {
                        let geometry = match geometry_type.as_str() {
                            "box" => generate_box_mesh(scale[0], scale[1], scale[2]),
                            "cylinder" => generate_cylinder_mesh(scale[0], scale[2], 24),
                            "sphere" => generate_sphere_mesh(scale[0], 16, 12),
                            "torus" => generate_torus_mesh(scale[0], scale[0] * 0.3, 24, 12),
                            _ => generate_box_mesh(scale[0], scale[1], scale[2]),
                        };
                        match self
                            .bridge
                            .rez_mesh(npc_id, geometry, *position, *scale, name)
                            .await
                        {
                            Ok(local_id) => {
                                created_ids.push((name.clone(), local_id));
                            }
                            Err(e) => {
                                let msg = format!("Failed to create mesh '{}': {}", name, e);
                                info!("[ACTION_PLANNER] {}", msg);
                                failures.push(msg);
                            }
                        }
                    }
                }
                NPCAction::ImportMesh {
                    file_path,
                    name,
                    position,
                } => {
                    match self
                        .bridge
                        .import_mesh(npc_id, file_path, name, *position)
                        .await
                    {
                        Ok(local_id) => {
                            created_ids.push((name.clone(), local_id));
                        }
                        Err(e) => {
                            let msg = format!("Failed to import mesh '{}': {}", name, e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::BlenderGenerate {
                    template,
                    params,
                    name,
                    position,
                } => {
                    match self
                        .bridge
                        .blender_generate(npc_id, template, params, name, *position)
                        .await
                    {
                        Ok(local_id) => {
                            created_ids.push((name.clone(), local_id));
                        }
                        Err(e) => {
                            let msg = format!(
                                "Failed to generate '{}' (template '{}'): {}",
                                name, template, e
                            );
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::CreateTShirt {
                    target_agent_id,
                    logo_path,
                    shirt_color,
                    front_offset_inches,
                    back_offset_inches,
                    sleeve_length,
                    fit,
                    collar,
                    name,
                } => {
                    match self
                        .bridge
                        .create_tshirt(
                            npc_id,
                            *target_agent_id,
                            logo_path,
                            *shirt_color,
                            *front_offset_inches,
                            *back_offset_inches,
                            *sleeve_length,
                            fit,
                            collar,
                            name,
                        )
                        .await
                    {
                        Ok(msg) => info!("[ACTION_PLANNER] Create T-shirt: {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to create T-shirt '{}': {}", name, e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::PackageObjectIntoPrim {
                    source_local_id,
                    container_local_id,
                } => {
                    let real_src = Self::remap_id(*source_local_id, &created_ids);
                    let real_dst = Self::remap_id(*container_local_id, &created_ids);
                    match self
                        .bridge
                        .package_object_into_prim(real_src, real_dst)
                        .await
                    {
                        Ok(msg) => info!("[ACTION_PLANNER] Package object: {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to package object: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::GiveToRequester { local_id } => {
                    let real_id = Self::remap_id(*local_id, &created_ids);
                    match self.bridge.give_object(real_id, npc_id).await {
                        Ok(msg) => {
                            info!("[ACTION_PLANNER] Give to requester: {}", msg);
                            if let Err(e) = self.bridge.delete_object(real_id).await {
                                info!("[ACTION_PLANNER] Failed to delete in-world object after give: {}", e);
                            }
                        }
                        Err(e) => {
                            let msg = format!("Failed to give to requester: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::DroneCinematography {
                    scene_name,
                    shot_type,
                    camera_waypoints,
                    lights,
                    lighting_preset,
                    subject_position,
                    speed,
                } => {
                    match self
                        .bridge
                        .setup_cinematography(
                            npc_id,
                            scene_name,
                            shot_type,
                            camera_waypoints.clone(),
                            lights.clone(),
                            lighting_preset.clone(),
                            *subject_position,
                            *speed,
                        )
                        .await
                    {
                        Ok(ids) => {
                            for (name, lid) in &ids {
                                created_ids.push((name.clone(), *lid));
                            }
                            info!(
                                "[ACTION_PLANNER] Cinematography scene '{}' set up: {} objects",
                                scene_name,
                                ids.len()
                            );
                        }
                        Err(e) => {
                            let msg =
                                format!("Failed to set up cinematography '{}': {}", scene_name, e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }

                    let region_id = self.bridge.default_region_uuid;
                    let terrain_hm = self.bridge.get_terrain_heightmap(region_id).await;
                    let scene =
                        capture_scene(self.bridge.scene_objects_ref(), region_id, terrain_hm);
                    info!(
                        "[LUXOR] Drone render: {} prims, terrain={} for '{}'",
                        scene.prims.len(),
                        scene.terrain.is_some(),
                        scene_name
                    );

                    let duration_secs = 10.0 / speed;
                    let fps = 30u32;
                    let video_job = match shot_type.to_lowercase().as_str() {
                        "orbit" | "circle" | "rotate" => {
                            VideoJob::orbit(*subject_position, 12.0, 4.0, duration_secs, fps)
                        }
                        "dolly" | "push" | "pull" | "flyby" => {
                            let start = [
                                subject_position[0] - 15.0,
                                subject_position[1],
                                subject_position[2] + 5.0,
                            ];
                            let end = [
                                subject_position[0] + 15.0,
                                subject_position[1],
                                subject_position[2] + 5.0,
                            ];
                            VideoJob::dolly(start, end, *subject_position, duration_secs, fps)
                        }
                        _ => VideoJob::orbit(*subject_position, 12.0, 4.0, duration_secs, fps),
                    };

                    let dist = 12.0;
                    let lighting_rig = lighting_preset
                        .as_ref()
                        .and_then(|l| LightingPreset::from_name(l))
                        .unwrap_or(LightingPreset::Studio3Point)
                        .build_rig(*subject_position, dist);

                    let output = OutputSettings {
                        size: ScreenSize::FullHD,
                        quality: RenderQuality::Standard,
                        effects: vec![],
                        format: OutputFormat::Png,
                    };

                    let request = VideoRequest {
                        job: video_job,
                        lighting: lighting_rig,
                        output,
                        region_id,
                        name: scene_name.clone(),
                    };

                    let director = LuxorDirector::new(PathBuf::from("Mediastorage"));
                    match director.render_video(&scene, &request).await {
                        Ok(path) => info!("[LUXOR] Drone video rendered: {}", path.display()),
                        Err(e) => {
                            let msg = format!("Luxor drone render failed: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::TerrainGenerate {
                    preset,
                    seed,
                    scale,
                    roughness,
                    water_level,
                    region_id,
                    grid_size,
                    grid_x,
                    grid_y,
                } => {
                    match self
                        .bridge
                        .generate_terrain(
                            npc_id,
                            preset,
                            *seed,
                            *scale,
                            *roughness,
                            *water_level,
                            region_id.as_deref(),
                            *grid_size,
                            *grid_x,
                            *grid_y,
                        )
                        .await
                    {
                        Ok(msg) => info!("[ACTION_PLANNER] {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to generate terrain: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::TerrainLoadR32 { file_path } => {
                    match self.bridge.load_terrain_r32(npc_id, file_path).await {
                        Ok(msg) => info!("[ACTION_PLANNER] {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to load terrain .r32: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::TerrainLoadImage {
                    file_path,
                    height_min,
                    height_max,
                } => {
                    match self
                        .bridge
                        .load_terrain_image(npc_id, file_path, *height_min, *height_max)
                        .await
                    {
                        Ok(msg) => info!("[ACTION_PLANNER] {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to load terrain from image: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::TerrainPreview {
                    preset,
                    seed,
                    scale,
                    roughness,
                    water_level,
                    region_id,
                    grid_size,
                    grid_x,
                    grid_y,
                } => {
                    match self
                        .bridge
                        .preview_terrain(
                            npc_id,
                            preset,
                            *seed,
                            *scale,
                            *roughness,
                            *water_level,
                            region_id.as_deref(),
                            *grid_size,
                            *grid_x,
                            *grid_y,
                        )
                        .await
                    {
                        Ok(msg) => info!("[ACTION_PLANNER] {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to preview terrain: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::TerrainApply { preview_id } => {
                    match self.bridge.apply_pending_terrain(npc_id, preview_id).await {
                        Ok(msg) => info!("[ACTION_PLANNER] {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to apply terrain: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::TerrainReject { preview_id } => {
                    match self.bridge.reject_pending_terrain(npc_id, preview_id).await {
                        Ok(msg) => info!("[ACTION_PLANNER] {}", msg),
                        Err(e) => {
                            let msg = format!("Failed to reject terrain: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::BuildVehicle {
                    recipe,
                    position,
                    tuning,
                } => {
                    let builder = crate::ai::vehicle_builder::VehicleBuilder::new(&self.bridge);
                    match builder
                        .build_vehicle(npc_id, recipe, *position, tuning, npc_id)
                        .await
                    {
                        Ok(result) => {
                            created_ids.push((result.recipe_name.clone(), result.root_id));
                            for (name, id) in &result.child_ids {
                                created_ids.push((name.clone(), *id));
                            }
                            if let Some(hid) = result.hud_id {
                                created_ids.push(("Vehicle HUD".to_string(), hid));
                            }
                            for line in &result.narration {
                                info!("[ACTION_PLANNER] Vehicle: {}", line);
                            }
                        }
                        Err(e) => {
                            let msg = format!("Failed to build vehicle '{}': {}", recipe, e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::ModifyVehicle { root_id, tuning } => {
                    let scripts = crate::ai::vehicle_scripts::VehicleScriptLibrary::global();
                    let recipes = crate::ai::vehicle_recipes::list_recipe_names();
                    let mut found = false;
                    for recipe_name in &recipes {
                        if let Some(recipe) = crate::ai::vehicle_recipes::get_recipe(recipe_name) {
                            if let Some(source) =
                                scripts.get_script_with_tuning(recipe.root_script, tuning)
                            {
                                match self
                                    .bridge
                                    .update_script(npc_id, *root_id, recipe.root_script, &source)
                                    .await
                                {
                                    Ok(_) => {
                                        info!("[ACTION_PLANNER] Vehicle tuning updated on local_id {} with {:?}", root_id, tuning);
                                        found = true;
                                    }
                                    Err(_) => continue,
                                }
                                break;
                            }
                        }
                    }
                    if !found {
                        info!("[ACTION_PLANNER] ModifyVehicle: trying all recipes as template fallback for local_id {}", root_id);
                        for recipe_name in &recipes {
                            if let Some(recipe) =
                                crate::ai::vehicle_recipes::get_recipe(recipe_name)
                            {
                                let tuning_str: std::collections::HashMap<String, String> = tuning
                                    .iter()
                                    .map(|(k, v)| (k.clone(), format!("{}", v)))
                                    .collect();
                                let template_name = recipe.root_script.trim_end_matches(".lsl");
                                if let Some(source) = crate::ai::script_templates::apply_template(
                                    template_name,
                                    &tuning_str,
                                ) {
                                    if self
                                        .bridge
                                        .update_script(
                                            npc_id,
                                            *root_id,
                                            recipe.root_script,
                                            &source,
                                        )
                                        .await
                                        .is_ok()
                                    {
                                        info!("[ACTION_PLANNER] Vehicle tuning via template '{}' on local_id {}", template_name, root_id);
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if !found {
                        let msg = format!(
                            "Could not modify vehicle at local_id {}: no matching script found",
                            root_id
                        );
                        info!("[ACTION_PLANNER] {}", msg);
                        failures.push(msg);
                    }
                }
                NPCAction::FindNearbyObject { name, radius } => {
                    let search_radius = radius.unwrap_or(30.0);
                    let center = self
                        .bridge
                        .get_avatar_position(npc_id)
                        .unwrap_or([128.0, 128.0, 25.0]);
                    let results =
                        self.bridge
                            .find_objects_near(center, search_radius, name.as_deref());
                    info!("[ACTION_PLANNER] FindNearby: center=[{:.1},{:.1},{:.1}] radius={:.0} filter={:?} → {} results",
                        center[0], center[1], center[2], search_radius, name, results.len());
                    for (local_id, obj_name, pos, is_linkset) in &results {
                        info!("[ACTION_PLANNER]   local_id={} name=\"{}\" pos=[{:.1},{:.1},{:.1}] linkset={}",
                            local_id, obj_name, pos[0], pos[1], pos[2], is_linkset);
                        created_ids.push((obj_name.clone(), *local_id));
                    }
                    if results.is_empty() {
                        let msg = format!(
                            "No objects found near [{:.0},{:.0},{:.0}] within {}m{}",
                            center[0],
                            center[1],
                            center[2],
                            search_radius,
                            name.as_ref()
                                .map(|n| format!(" matching '{}'", n))
                                .unwrap_or_default()
                        );
                        info!("[ACTION_PLANNER] {}", msg);
                        failures.push(msg);
                    }
                }
                NPCAction::ScanLinkset { root_id } => {
                    let real_id = Self::remap_id(*root_id, &created_ids);
                    match self.bridge.scan_linkset(real_id) {
                        Ok(links) => {
                            info!(
                                "[ACTION_PLANNER] ScanLinkset root_id={} → {} prims:",
                                real_id,
                                links.len()
                            );
                            for (link_num, local_id, name, scale) in &links {
                                info!("[ACTION_PLANNER]   Link {}: local_id={} name=\"{}\" scale=[{:.2},{:.2},{:.2}]",
                                    link_num, local_id, name, scale[0], scale[1], scale[2]);
                                created_ids.push((name.clone(), *local_id));
                            }
                        }
                        Err(e) => {
                            let msg = format!("Failed to scan linkset root_id={}: {}", real_id, e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::ScriptLinkset {
                    root_id,
                    script_map,
                    root_script,
                } => {
                    let real_id = Self::remap_id(*root_id, &created_ids);
                    let scripts = crate::ai::vehicle_scripts::VehicleScriptLibrary::global();
                    match self.bridge.scan_linkset(real_id) {
                        Ok(links) => {
                            info!(
                                "[ACTION_PLANNER] ScriptLinkset: {} prims, {} script mappings",
                                links.len(),
                                script_map.len()
                            );
                            if let Some(rs) = root_script {
                                if let Some(source) = scripts.get_script(rs) {
                                    match self.bridge.insert_script(npc_id, real_id, rs, source).await {
                                        Ok(_) => info!("[ACTION_PLANNER] Root script '{}' installed on local_id {}", rs, real_id),
                                        Err(e) => {
                                            let msg = format!("Failed to install root script '{}': {}", rs, e);
                                            info!("[ACTION_PLANNER] {}", msg);
                                            failures.push(msg);
                                        }
                                    }
                                } else {
                                    let msg = format!("Root script '{}' not found in library", rs);
                                    info!("[ACTION_PLANNER] {}", msg);
                                    failures.push(msg);
                                }
                            }
                            for (link_num, local_id, prim_name, _scale) in &links {
                                let name_lower = prim_name.to_lowercase();
                                for (match_name, script_name) in script_map.iter() {
                                    if name_lower.contains(&match_name.to_lowercase()) {
                                        if let Some(source) = scripts.get_script(script_name) {
                                            match self.bridge.insert_script(npc_id, *local_id, script_name, source).await {
                                                Ok(_) => info!("[ACTION_PLANNER] Script '{}' → link {} '{}' (local_id {})",
                                                    script_name, link_num, prim_name, local_id),
                                                Err(e) => {
                                                    let msg = format!("Failed to install '{}' in '{}': {}", script_name, prim_name, e);
                                                    info!("[ACTION_PLANNER] {}", msg);
                                                    failures.push(msg);
                                                }
                                            }
                                        } else {
                                            let msg = format!(
                                                "Script '{}' not found in library for prim '{}'",
                                                script_name, prim_name
                                            );
                                            info!("[ACTION_PLANNER] {}", msg);
                                            failures.push(msg);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let msg = format!(
                                "Failed to scan linkset for scripting root_id={}: {}",
                                real_id, e
                            );
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::AddToLinkset {
                    root_id,
                    new_prim_ids,
                } => {
                    let real_root = Self::remap_id(*root_id, &created_ids);
                    let real_ids: Vec<u32> = new_prim_ids
                        .iter()
                        .map(|id| Self::remap_id(*id, &created_ids))
                        .collect();
                    match self.bridge.add_to_linkset(real_root, &real_ids).await {
                        Ok(assignments) => {
                            for (local_id, link_num) in &assignments {
                                info!(
                                    "[ACTION_PLANNER] add_to_linkset: prim {} → link {} in root {}",
                                    local_id, link_num, real_root
                                );
                            }
                        }
                        Err(e) => {
                            let msg = format!("Failed add_to_linkset root={}: {}", real_root, e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::CreateBadge { .. }
                | NPCAction::ComposeFilm { .. }
                | NPCAction::ComposeMusic { .. }
                | NPCAction::ComposeAd { .. }
                | NPCAction::ComposePhoto { .. } => {
                    info!("[ACTION_PLANNER] Media/badge action — handled by skill module");
                }
                NPCAction::LuxorSnapshot {
                    preset,
                    size,
                    quality,
                    effects,
                    lighting,
                    subject_position,
                    name,
                } => {
                    let region_id = self.bridge.default_region_uuid;
                    let terrain_hm = self.bridge.get_terrain_heightmap(region_id).await;
                    let scene =
                        capture_scene(self.bridge.scene_objects_ref(), region_id, terrain_hm);
                    info!(
                        "[LUXOR] Scene captured: {} prims, terrain={} for snapshot '{}'",
                        scene.prims.len(),
                        scene.terrain.is_some(),
                        name
                    );

                    let mut camera = CameraRig::default();
                    camera.position = [
                        subject_position[0] - 5.0,
                        subject_position[1] - 5.0,
                        subject_position[2] + 3.0,
                    ];
                    camera.look_at = *subject_position;
                    if let Some(ref p) = preset {
                        if let Some(cp) = CameraPreset::from_name(p) {
                            cp.apply(&mut camera);
                        }
                    }

                    let dist = 10.0;
                    let lighting_rig = lighting
                        .as_ref()
                        .and_then(|l| LightingPreset::from_name(l))
                        .unwrap_or(LightingPreset::Studio3Point)
                        .build_rig(*subject_position, dist);

                    let render_quality = quality
                        .as_ref()
                        .and_then(|q| RenderQuality::from_name(q))
                        .unwrap_or(RenderQuality::Standard);
                    let screen_size = size
                        .as_ref()
                        .and_then(|s| ScreenSize::from_name(s))
                        .unwrap_or(ScreenSize::FullHD);
                    let post_effects: Vec<PostEffect> = effects
                        .iter()
                        .filter_map(|e| PostEffect::from_name(e))
                        .collect();

                    let output = OutputSettings {
                        size: screen_size,
                        quality: render_quality,
                        effects: post_effects,
                        format: OutputFormat::Png,
                    };

                    let request = SnapshotRequest {
                        camera,
                        lighting: lighting_rig,
                        output,
                        region_id,
                        name: name.clone(),
                    };

                    let director = LuxorDirector::new(PathBuf::from("Mediastorage"));
                    let scene_clone = scene.clone();
                    let request_clone = request.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        director.render_snapshot(&scene_clone, &request_clone)
                    })
                    .await;
                    match result {
                        Ok(Ok(path)) => info!("[LUXOR] Snapshot rendered: {}", path.display()),
                        Ok(Err(e)) => {
                            let msg = format!("Luxor snapshot failed: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                        Err(e) => {
                            let msg = format!("Luxor snapshot task panicked: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::LuxorVideo {
                    shot_type,
                    duration,
                    fps,
                    size,
                    quality,
                    effects,
                    lighting,
                    subject_position,
                    name,
                } => {
                    let region_id = self.bridge.default_region_uuid;
                    let terrain_hm = self.bridge.get_terrain_heightmap(region_id).await;
                    let scene =
                        capture_scene(self.bridge.scene_objects_ref(), region_id, terrain_hm);
                    info!(
                        "[LUXOR] Scene captured: {} prims, terrain={} for video '{}'",
                        scene.prims.len(),
                        scene.terrain.is_some(),
                        name
                    );

                    let video_job = match shot_type.to_lowercase().as_str() {
                        "orbit" | "circle" | "rotate" => {
                            VideoJob::orbit(*subject_position, 10.0, 3.0, *duration, *fps)
                        }
                        "dolly" | "push" | "pull" => {
                            let start = [
                                subject_position[0] - 10.0,
                                subject_position[1],
                                subject_position[2] + 2.0,
                            ];
                            let end = [
                                subject_position[0] + 10.0,
                                subject_position[1],
                                subject_position[2] + 2.0,
                            ];
                            VideoJob::dolly(start, end, *subject_position, *duration, *fps)
                        }
                        _ => VideoJob::orbit(*subject_position, 10.0, 3.0, *duration, *fps),
                    };

                    let dist = 10.0;
                    let lighting_rig = lighting
                        .as_ref()
                        .and_then(|l| LightingPreset::from_name(l))
                        .unwrap_or(LightingPreset::Studio3Point)
                        .build_rig(*subject_position, dist);

                    let render_quality = quality
                        .as_ref()
                        .and_then(|q| RenderQuality::from_name(q))
                        .unwrap_or(RenderQuality::Standard);
                    let screen_size = size
                        .as_ref()
                        .and_then(|s| ScreenSize::from_name(s))
                        .unwrap_or(ScreenSize::FullHD);
                    let post_effects: Vec<PostEffect> = effects
                        .iter()
                        .filter_map(|e| PostEffect::from_name(e))
                        .collect();

                    let output = OutputSettings {
                        size: screen_size,
                        quality: render_quality,
                        effects: post_effects,
                        format: OutputFormat::Png,
                    };

                    let request = VideoRequest {
                        job: video_job,
                        lighting: lighting_rig,
                        output,
                        region_id,
                        name: name.clone(),
                    };

                    let director = LuxorDirector::new(PathBuf::from("Mediastorage"));
                    match director.render_video(&scene, &request).await {
                        Ok(path) => info!("[LUXOR] Video rendered: {}", path.display()),
                        Err(e) => {
                            let msg = format!("Luxor video failed: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::ImportFloorplan {
                    image_path,
                    position,
                    wall_height,
                    scale,
                } => {
                    info!(
                        "[ACTION_PLANNER] ImportFloorplan: {} at {:?} (wall_h={:?}, scale={:?})",
                        image_path, position, wall_height, scale
                    );
                    match self
                        .execute_floorplan_build(
                            npc_id,
                            image_path,
                            position,
                            wall_height,
                            scale,
                            &mut created_ids,
                        )
                        .await
                    {
                        Ok(count) => info!("[ACTION_PLANNER] Floorplan built: {} prims", count),
                        Err(e) => {
                            let msg = format!("Floorplan build failed: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
                NPCAction::ImportElevation {
                    image_path,
                    position,
                } => {
                    info!(
                        "[ACTION_PLANNER] ImportElevation: {} at {:?}",
                        image_path, position
                    );
                }
                NPCAction::ImportBlueprint {
                    floorplan_path,
                    elevation_path,
                    position,
                    wall_height,
                    scale,
                } => {
                    info!(
                        "[ACTION_PLANNER] ImportBlueprint: floor={}, elev={:?} at {:?}",
                        floorplan_path, elevation_path, position
                    );
                    match self
                        .execute_floorplan_build(
                            npc_id,
                            floorplan_path,
                            position,
                            wall_height,
                            scale,
                            &mut created_ids,
                        )
                        .await
                    {
                        Ok(count) => info!(
                            "[ACTION_PLANNER] Blueprint floor plan built: {} prims",
                            count
                        ),
                        Err(e) => {
                            let msg = format!("Blueprint build failed: {}", e);
                            info!("[ACTION_PLANNER] {}", msg);
                            failures.push(msg);
                        }
                    }
                }
            }
        }

        if !created_ids.is_empty() {
            info!(
                "[ACTION_PLANNER] NPC created {} objects: {:?}",
                created_ids.len(),
                created_ids
            );
        }
        if !failures.is_empty() {
            info!(
                "[ACTION_PLANNER] {} action(s) failed: {:?}",
                failures.len(),
                failures
            );
        }

        Ok((created_ids, failures))
    }

    async fn execute_floorplan_build(
        &self,
        npc_id: Uuid,
        image_path: &str,
        position: &[f32; 3],
        wall_height: &Option<f32>,
        scale: &Option<f32>,
        created_ids: &mut Vec<(String, u32)>,
    ) -> Result<usize> {
        use crate::ai::image_to_build::{
            analyze_floorplan, detect_media_type, floorplan_to_actions, BuildConfig,
        };

        let llm = self.llm_client.as_ref().ok_or_else(|| {
            anyhow::anyhow!("No LLM client available for vision analysis — set provider in llm.ini")
        })?;

        let image_data = tokio::fs::read(image_path)
            .await
            .map_err(|e| anyhow::anyhow!("Cannot read floor plan image '{}': {}", image_path, e))?;

        let media_type = detect_media_type(image_path);
        let plan = analyze_floorplan(llm, &image_data, media_type)
            .await
            .map_err(|e| anyhow::anyhow!("Vision analysis failed: {}", e))?;

        info!("[ACTION_PLANNER] Floorplan extracted: {} walls, {} doors, {} windows, {} rooms ({}x{}m)",
            plan.walls.len(), plan.doors.len(), plan.windows.len(), plan.rooms.len(),
            plan.overall_width, plan.overall_depth);

        let config = BuildConfig {
            origin: *position,
            wall_height: wall_height.unwrap_or(3.0),
            default_scale: scale.unwrap_or(1.0),
            ..Default::default()
        };

        let build_actions = floorplan_to_actions(&plan, &config);
        let count = build_actions.len();
        let start_idx = created_ids.len();

        for action in &build_actions {
            match action {
                NPCAction::RezBox {
                    position: pos,
                    scale: sc,
                    name,
                } => match self.bridge.rez_box(npc_id, *pos, *sc, name).await {
                    Ok(local_id) => {
                        created_ids.push((name.clone(), local_id));
                    }
                    Err(e) => {
                        info!("[ACTION_PLANNER] Failed to rez {}: {}", name, e);
                    }
                },
                NPCAction::RezPrism {
                    position: pos,
                    scale: sc,
                    name,
                } => match self.bridge.rez_prism(npc_id, *pos, *sc, name).await {
                    Ok(local_id) => {
                        created_ids.push((name.clone(), local_id));
                    }
                    Err(e) => {
                        info!("[ACTION_PLANNER] Failed to rez {}: {}", name, e);
                    }
                },
                NPCAction::SetRotation { local_id, rotation } => {
                    let real_id = Self::remap_id(*local_id, created_ids);
                    let _ = self.bridge.set_object_rotation(real_id, *rotation).await;
                }
                _ => {}
            }
        }

        let floorplan_ids: Vec<(String, u32)> = created_ids[start_idx..].to_vec();
        if floorplan_ids.len() > 1 {
            let root_id = floorplan_ids[0].1;
            let child_ids: Vec<u32> = floorplan_ids[1..].iter().map(|(_, id)| *id).collect();
            if let Err(e) = self.bridge.link_objects(root_id, &child_ids).await {
                info!("[ACTION_PLANNER] Floorplan link failed: {}", e);
            } else {
                info!(
                    "[ACTION_PLANNER] Linked floorplan: root={} + {} children",
                    root_id,
                    child_ids.len()
                );
            }
        }

        Ok(count)
    }
}
