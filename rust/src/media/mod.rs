pub mod scene_exporter;
pub mod render_script;
pub mod renderer;
pub mod encoder;
pub mod post_process;
pub mod photographer;
pub mod print_compositor;
pub mod audio_renderer;
pub mod luxor;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use crate::ai::npc_avatar::{CameraWaypoint, CinemaLight};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputMode {
    Video,
    Stills,
    Both,
    Print,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    pub resolution: (u32, u32),
    pub fps: u32,
    pub frames_per_waypoint: u32,
    pub render_engine: String,
    pub samples: u32,
    pub output_mode: OutputMode,
    pub post_process: Option<String>,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            fps: 24,
            frames_per_waypoint: 30,
            render_engine: "EEVEE".to_string(),
            samples: 64,
            output_mode: OutputMode::Video,
            post_process: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneExportObject {
    pub uuid: Uuid,
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub shape_type: String,
    pub color: [f32; 4],
    pub mesh_asset_id: Option<Uuid>,
    pub profile_curve: u8,
    pub path_curve: u8,
    pub profile_begin: f32,
    pub profile_end: f32,
    pub profile_hollow: f32,
    pub path_begin: f32,
    pub path_end: f32,
}

#[derive(Debug, Clone)]
pub struct RenderJob {
    pub job_id: Uuid,
    pub region_id: Uuid,
    pub scene_objects: Vec<SceneExportObject>,
    pub waypoints: Vec<CameraWaypoint>,
    pub lights: Vec<CinemaLight>,
    pub terrain_heightmap: Option<Vec<f32>>,
    pub output_name: String,
    pub settings: RenderSettings,
    pub audio_preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRecipe {
    pub recipe_name: String,
    pub media_type: String,
    pub camera: RecipeCamera,
    pub lighting: Vec<RecipeLight>,
    pub composition: String,
    pub region_id: String,
    pub post_process: Option<String>,
    pub print_layout: Option<PrintLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCamera {
    pub position: [f32; 3],
    pub focus: [f32; 3],
    pub fov: f32,
    pub f_stop: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeLight {
    pub light_type: String,
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintLayout {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub font: String,
    pub font_size: f32,
    pub logo_path: Option<String>,
    pub border_width: f32,
    pub border_color: [f32; 3],
}

#[derive(Clone)]
pub struct MediaDirector {
    pub blender_path: String,
    pub gimp_path: String,
    pub ffmpeg_path: String,
    pub output_base: PathBuf,
    pub db_url: String,
}

impl MediaDirector {
    pub fn new(output_base: PathBuf, db_url: String) -> Self {
        Self {
            blender_path: "/Applications/Blender.app/Contents/MacOS/Blender".to_string(),
            gimp_path: "/Applications/GIMP-2.10.app/Contents/MacOS/gimp".to_string(),
            ffmpeg_path: "/usr/local/bin/ffmpeg".to_string(),
            output_base,
            db_url,
        }
    }

    pub fn ensure_directories(&self) -> Result<()> {
        for subdir in &["video", "images", "audio", "print", "projects", "temp"] {
            std::fs::create_dir_all(self.output_base.join(subdir))?;
        }
        Ok(())
    }

    pub async fn render_and_encode(&self, job: &RenderJob) -> Result<PathBuf> {
        self.ensure_directories()?;

        let job_dir = self.output_base.join("temp").join(job.job_id.to_string());
        std::fs::create_dir_all(&job_dir)?;
        let meshes_dir = job_dir.join("meshes");
        std::fs::create_dir_all(&meshes_dir)?;

        info!("[MEDIA] Starting render job '{}' ({} objects, {} waypoints)",
            job.output_name, job.scene_objects.len(), job.waypoints.len());

        scene_exporter::export_scene(job, &meshes_dir)?;

        let frames_dir = job_dir.join("frames");
        std::fs::create_dir_all(&frames_dir)?;

        let script = render_script::generate_render_script(job, &meshes_dir, &frames_dir)?;
        let script_path = job_dir.join("render.py");
        std::fs::write(&script_path, &script)?;

        renderer::run_blender(&self.blender_path, &script_path).await?;

        if let Some(ref filter_name) = job.settings.post_process {
            if Path::new(&self.gimp_path).exists() {
                post_process::apply_gimp_filter(&self.gimp_path, &frames_dir, filter_name).await?;
            } else {
                warn!("[MEDIA] GIMP not found at {}, skipping post-process", self.gimp_path);
            }
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

        let output_path = match job.settings.output_mode {
            OutputMode::Video | OutputMode::Both => {
                let video_path = self.output_base.join("video")
                    .join(format!("{}_{}.mp4", job.output_name, timestamp));
                encoder::encode_video(&self.ffmpeg_path, &frames_dir, &video_path, job.settings.fps).await?;

                if let Some(ref audio_preset) = job.audio_preset {
                    let audio_path = self.output_base.join("audio")
                        .join(format!("{}_{}.wav", job.output_name, timestamp));
                    let total_frames = job.waypoints.len() as u32 * job.settings.frames_per_waypoint;
                    let duration_secs = total_frames as f32 / job.settings.fps as f32;
                    audio_renderer::render_ambient_audio(audio_preset, duration_secs, &audio_path)?;

                    let muxed_path = self.output_base.join("video")
                        .join(format!("{}_{}_final.mp4", job.output_name, timestamp));
                    encoder::mux_audio(&self.ffmpeg_path, &video_path, &audio_path, &muxed_path).await?;
                    std::fs::rename(&muxed_path, &video_path)?;
                    info!("[MEDIA] Audio muxed into video: {}", video_path.display());
                }

                video_path
            }
            OutputMode::Stills => {
                let stills_dir = self.output_base.join("images");
                let mut last_path = stills_dir.clone();
                for entry in std::fs::read_dir(&frames_dir)? {
                    let entry = entry?;
                    let dest = stills_dir.join(format!("{}_{}_{}",
                        job.output_name, timestamp,
                        entry.file_name().to_string_lossy()));
                    std::fs::copy(entry.path(), &dest)?;
                    last_path = dest;
                }
                last_path
            }
            OutputMode::Print => {
                frames_dir.clone()
            }
        };

        if matches!(job.settings.output_mode, OutputMode::Both) {
            let stills_dir = self.output_base.join("images");
            for entry in std::fs::read_dir(&frames_dir)? {
                let entry = entry?;
                let dest = stills_dir.join(format!("{}_{}_{}",
                    job.output_name, timestamp,
                    entry.file_name().to_string_lossy()));
                std::fs::copy(entry.path(), &dest)?;
            }
        }

        let blend_path = self.output_base.join("projects")
            .join(format!("{}_{}.blend", job.output_name, timestamp));
        let blend_script = render_script::generate_save_blend_script(&blend_path);
        let blend_script_path = job_dir.join("save_blend.py");
        std::fs::write(&blend_script_path, &blend_script)?;
        let _ = renderer::run_blender(&self.blender_path, &blend_script_path).await;

        if let Err(e) = std::fs::remove_dir_all(&job_dir) {
            warn!("[MEDIA] Failed to clean temp dir: {}", e);
        }

        info!("[MEDIA] Render job '{}' complete: {}", job.output_name, output_path.display());
        Ok(output_path)
    }

    pub fn save_recipe(&self, recipe: &MediaRecipe) -> Result<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let path = self.output_base.join("projects")
            .join(format!("{}_recipe_{}.json", recipe.recipe_name, timestamp));
        let json = serde_json::to_string_pretty(recipe)?;
        std::fs::write(&path, &json)?;
        info!("[MEDIA] Recipe saved: {}", path.display());
        Ok(path)
    }

    pub fn load_recipe(&self, path: &Path) -> Result<MediaRecipe> {
        let json = std::fs::read_to_string(path)?;
        let recipe: MediaRecipe = serde_json::from_str(&json)?;
        Ok(recipe)
    }
}
