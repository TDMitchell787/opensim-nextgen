use anyhow::Result;
use std::path::PathBuf;
use tracing::info;
use uuid::Uuid;

use super::{
    MediaDirector, MediaRecipe, OutputMode, PrintLayout, RecipeCamera, RenderJob, RenderSettings,
    SceneExportObject,
};
use crate::ai::cinematography;
use crate::ai::npc_avatar::CinemaLight;

#[derive(Debug, Clone)]
pub struct PrintJob {
    pub name: String,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub subject_position: [f32; 3],
    pub region_id: Uuid,
    pub resolution: (u32, u32),
    pub lighting_preset: String,
    pub font: String,
    pub border_width: f32,
    pub border_color: [f32; 3],
}

impl Default for PrintJob {
    fn default() -> Self {
        Self {
            name: "poster".to_string(),
            title: None,
            subtitle: None,
            subject_position: [128.0, 128.0, 25.0],
            region_id: Uuid::nil(),
            resolution: (2480, 3508),
            lighting_preset: "studio".to_string(),
            font: "DejaVu Sans".to_string(),
            border_width: 0.0,
            border_color: [0.0, 0.0, 0.0],
        }
    }
}

impl MediaDirector {
    pub async fn compose_print(
        &self,
        print_job: &PrintJob,
        scene_objects: Vec<SceneExportObject>,
        terrain: Option<Vec<f32>>,
    ) -> Result<PathBuf> {
        self.ensure_directories()?;

        let job_dir = self
            .output_base
            .join("temp")
            .join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&job_dir)?;
        let meshes_dir = job_dir.join("meshes");
        std::fs::create_dir_all(&meshes_dir)?;

        let lights = cinematography::generate_lighting_preset(
            &print_job.lighting_preset,
            print_job.subject_position,
            8.0,
        );

        let render_job = RenderJob {
            job_id: Uuid::new_v4(),
            region_id: print_job.region_id,
            scene_objects,
            waypoints: vec![],
            lights: lights.clone(),
            terrain_heightmap: terrain,
            output_name: print_job.name.clone(),
            settings: RenderSettings {
                resolution: print_job.resolution,
                output_mode: OutputMode::Stills,
                render_engine: "CYCLES".to_string(),
                samples: 256,
                ..Default::default()
            },
            audio_preset: None,
        };
        super::scene_exporter::export_scene(&render_job, &meshes_dir)?;

        let render_image_path = job_dir.join("render_plate.png");
        let photo_script = super::render_script::generate_photo_script(
            &meshes_dir,
            [
                print_job.subject_position[0] - 12.0,
                print_job.subject_position[1],
                print_job.subject_position[2] + 5.0,
            ],
            print_job.subject_position,
            50.0,
            5.6,
            &lights,
            &render_image_path,
            print_job.resolution,
            256,
        );
        let photo_script_path = job_dir.join("render_plate.py");
        std::fs::write(&photo_script_path, &photo_script)?;
        super::renderer::run_blender(&self.blender_path, &photo_script_path).await?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let output_path = self
            .output_base
            .join("print")
            .join(format!("{}_poster_{}.png", print_job.name, timestamp));

        let compositor_script = super::render_script::generate_compositor_script(
            &render_image_path,
            print_job.title.as_deref(),
            print_job.subtitle.as_deref(),
            &output_path,
            print_job.resolution,
        );
        let comp_script_path = job_dir.join("compositor.py");
        std::fs::write(&comp_script_path, &compositor_script)?;
        super::renderer::run_blender(&self.blender_path, &comp_script_path).await?;

        let recipe = MediaRecipe {
            recipe_name: print_job.name.clone(),
            media_type: "print".to_string(),
            camera: RecipeCamera {
                position: [
                    print_job.subject_position[0] - 12.0,
                    print_job.subject_position[1],
                    print_job.subject_position[2] + 5.0,
                ],
                focus: print_job.subject_position,
                fov: 50.0,
                f_stop: 5.6,
            },
            lighting: vec![],
            composition: "centered".to_string(),
            region_id: print_job.region_id.to_string(),
            post_process: None,
            print_layout: Some(PrintLayout {
                title: print_job.title.clone(),
                subtitle: print_job.subtitle.clone(),
                font: print_job.font.clone(),
                font_size: 72.0,
                logo_path: None,
                border_width: print_job.border_width,
                border_color: print_job.border_color,
            }),
        };
        self.save_recipe(&recipe)?;

        if let Err(e) = std::fs::remove_dir_all(&job_dir) {
            tracing::warn!("[MEDIA] Failed to clean temp: {}", e);
        }

        info!(
            "[MEDIA] Print composition complete: {}",
            output_path.display()
        );
        Ok(output_path)
    }
}
