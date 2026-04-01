use std::path::PathBuf;
use anyhow::Result;
use uuid::Uuid;
use tracing::info;

use super::{MediaDirector, MediaRecipe, RecipeCamera, RecipeLight, RenderJob, RenderSettings, OutputMode, SceneExportObject};
use crate::ai::npc_avatar::CinemaLight;
use crate::ai::cinematography;

#[derive(Debug, Clone)]
pub struct PhotoComposition {
    pub subject_position: [f32; 3],
    pub camera_angle: String,
    pub composition_rule: String,
    pub lighting_preset: String,
    pub depth_of_field: f32,
    pub region_id: Uuid,
    pub name: String,
}

impl MediaDirector {
    pub async fn compose_photo(
        &self,
        comp: &PhotoComposition,
        scene_objects: Vec<SceneExportObject>,
        terrain: Option<Vec<f32>>,
    ) -> Result<PathBuf> {
        self.ensure_directories()?;

        let camera_pos = compute_camera_position(
            comp.subject_position,
            &comp.camera_angle,
            &comp.composition_rule,
        );
        let lights = cinematography::generate_lighting_preset(
            &comp.lighting_preset,
            comp.subject_position,
            8.0,
        );
        let f_stop = depth_of_field_to_fstop(comp.depth_of_field);

        let job_dir = self.output_base.join("temp").join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&job_dir)?;
        let meshes_dir = job_dir.join("meshes");
        std::fs::create_dir_all(&meshes_dir)?;

        let render_job = RenderJob {
            job_id: Uuid::new_v4(),
            region_id: comp.region_id,
            scene_objects,
            waypoints: vec![],
            lights: lights.clone(),
            terrain_heightmap: terrain,
            output_name: comp.name.clone(),
            settings: RenderSettings::default(),
            audio_preset: None,
        };
        super::scene_exporter::export_scene(&render_job, &meshes_dir)?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let output_path = self.output_base.join("images")
            .join(format!("{}_still_0_{}.png", comp.name, timestamp));

        let script = super::render_script::generate_photo_script(
            &meshes_dir,
            camera_pos,
            comp.subject_position,
            60.0,
            f_stop,
            &lights,
            &output_path,
            (3840, 2160),
            256,
        );
        let script_path = job_dir.join("photo.py");
        std::fs::write(&script_path, &script)?;

        super::renderer::run_blender(&self.blender_path, &script_path).await?;

        let recipe = MediaRecipe {
            recipe_name: comp.name.clone(),
            media_type: "photo".to_string(),
            camera: RecipeCamera {
                position: camera_pos,
                focus: comp.subject_position,
                fov: 60.0,
                f_stop,
            },
            lighting: lights.iter().map(|l| RecipeLight {
                light_type: l.name.clone(),
                position: l.position,
                color: l.color,
                intensity: l.intensity,
                radius: l.radius,
            }).collect(),
            composition: comp.composition_rule.clone(),
            region_id: comp.region_id.to_string(),
            post_process: None,
            print_layout: None,
        };
        self.save_recipe(&recipe)?;

        if let Err(e) = std::fs::remove_dir_all(&job_dir) {
            tracing::warn!("[MEDIA] Failed to clean temp: {}", e);
        }

        info!("[MEDIA] Photo composed: {}", output_path.display());
        Ok(output_path)
    }
}

fn compute_camera_position(subject: [f32; 3], angle: &str, composition: &str) -> [f32; 3] {
    let distance = 10.0;
    let offset = match composition {
        "rule_of_thirds" => 2.0,
        "golden_ratio" => 1.618,
        "leading_lines" => 3.0,
        _ => 0.0,
    };

    match angle {
        "low_angle" => [
            subject[0] - distance + offset,
            subject[1] + offset,
            subject[2] - 3.0,
        ],
        "high_angle" => [
            subject[0] - distance + offset,
            subject[1] + offset,
            subject[2] + 8.0,
        ],
        "bird_eye" => [
            subject[0] + offset,
            subject[1] + offset,
            subject[2] + 20.0,
        ],
        "dutch_tilt" => [
            subject[0] - distance * 0.7 + offset,
            subject[1] + distance * 0.7,
            subject[2] + 2.0,
        ],
        _ => [
            subject[0] - distance + offset,
            subject[1] + offset,
            subject[2] + 1.5,
        ],
    }
}

fn depth_of_field_to_fstop(dof: f32) -> f32 {
    let clamped = dof.clamp(0.0, 1.0);
    1.4 + (16.0 - 1.4) * (1.0 - clamped)
}
