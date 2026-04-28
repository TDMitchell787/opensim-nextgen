pub mod camera;
pub mod geometry;
pub mod gpu;
pub mod hud_protocol;
pub mod lighting;
pub mod post_process;
pub mod raytracer;
pub mod scene_capture;
pub mod shading;
pub mod video;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use camera::{CameraPreset, CameraRig};
use lighting::{LightingPreset, LightingRig};
use post_process::PostEffect;
use raytracer::{LuxorRaytracer, RenderQuality};
use scene_capture::{CapturedPrim, SceneGeometry};
use video::VideoJob;

pub const LUXOR_CHANNEL: i32 = -15500;
pub const LUXOR_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ScreenSize {
    SD,
    HD,
    FullHD,
    QHD,
    UHD4K,
    Cinema,
    Square,
    Portrait,
    Poster,
    Banner,
    Custom(u32, u32),
}

impl ScreenSize {
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            ScreenSize::SD => (640, 480),
            ScreenSize::HD => (1280, 720),
            ScreenSize::FullHD => (1920, 1080),
            ScreenSize::QHD => (2560, 1440),
            ScreenSize::UHD4K => (3840, 2160),
            ScreenSize::Cinema => (2560, 1080),
            ScreenSize::Square => (1080, 1080),
            ScreenSize::Portrait => (1080, 1920),
            ScreenSize::Poster => (2480, 3508),
            ScreenSize::Banner => (3840, 1080),
            ScreenSize::Custom(w, h) => (*w, *h),
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        let (w, h) = self.resolution();
        w as f32 / h as f32
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "sd" | "480p" => Some(ScreenSize::SD),
            "hd" | "720p" => Some(ScreenSize::HD),
            "fullhd" | "1080p" | "fhd" => Some(ScreenSize::FullHD),
            "qhd" | "1440p" | "2k" => Some(ScreenSize::QHD),
            "4k" | "uhd" | "uhd4k" | "2160p" => Some(ScreenSize::UHD4K),
            "cinema" | "ultrawide" | "cinemascope" => Some(ScreenSize::Cinema),
            "square" | "1x1" => Some(ScreenSize::Square),
            "portrait" | "vertical" | "9x16" => Some(ScreenSize::Portrait),
            "poster" | "a4" | "print" => Some(ScreenSize::Poster),
            "banner" | "header" | "32x9" => Some(ScreenSize::Banner),
            _ => {
                if let Some((w_str, h_str)) = name.split_once('x') {
                    if let (Ok(w), Ok(h)) =
                        (w_str.trim().parse::<u32>(), h_str.trim().parse::<u32>())
                    {
                        if w > 0 && h > 0 && w <= 7680 && h <= 7680 {
                            return Some(ScreenSize::Custom(w, h));
                        }
                    }
                }
                None
            }
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            ScreenSize::SD => "SD (640x480)".into(),
            ScreenSize::HD => "HD (1280x720)".into(),
            ScreenSize::FullHD => "Full HD (1920x1080)".into(),
            ScreenSize::QHD => "QHD (2560x1440)".into(),
            ScreenSize::UHD4K => "4K UHD (3840x2160)".into(),
            ScreenSize::Cinema => "Cinema (2560x1080)".into(),
            ScreenSize::Square => "Square (1080x1080)".into(),
            ScreenSize::Portrait => "Portrait (1080x1920)".into(),
            ScreenSize::Poster => "Poster A4 (2480x3508)".into(),
            ScreenSize::Banner => "Banner (3840x1080)".into(),
            ScreenSize::Custom(w, h) => format!("Custom ({}x{})", w, h),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    pub size: ScreenSize,
    pub quality: RenderQuality,
    pub effects: Vec<PostEffect>,
    pub format: OutputFormat,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            size: ScreenSize::FullHD,
            quality: RenderQuality::Standard,
            effects: Vec::new(),
            format: OutputFormat::Png,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    Png,
    Jpeg,
    Bmp,
}

impl OutputFormat {
    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Bmp => "bmp",
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "jpg" | "jpeg" => OutputFormat::Jpeg,
            "bmp" => OutputFormat::Bmp,
            _ => OutputFormat::Png,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotRequest {
    pub camera: CameraRig,
    pub lighting: LightingRig,
    pub output: OutputSettings,
    pub region_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct VideoRequest {
    pub job: VideoJob,
    pub lighting: LightingRig,
    pub output: OutputSettings,
    pub region_id: Uuid,
    pub name: String,
}

pub struct LuxorDirector {
    pub output_base: PathBuf,
    pub ffmpeg_path: String,
    pub gpu_renderer: Option<gpu::GpuRenderer>,
}

impl Clone for LuxorDirector {
    fn clone(&self) -> Self {
        Self {
            output_base: self.output_base.clone(),
            ffmpeg_path: self.ffmpeg_path.clone(),
            gpu_renderer: None,
        }
    }
}

impl LuxorDirector {
    pub fn new(output_base: PathBuf) -> Self {
        let gpu_renderer = gpu::GpuRenderer::try_new();
        if let Some(ref gpu) = gpu_renderer {
            info!("[LUXOR] GPU acceleration enabled: {}", gpu.adapter_name());
        } else {
            info!("[LUXOR] No GPU available — using CPU rayon backend");
        }
        Self {
            output_base,
            ffmpeg_path: "/usr/local/bin/ffmpeg".to_string(),
            gpu_renderer,
        }
    }

    pub fn ensure_directories(&self) -> Result<()> {
        for subdir in &["images", "video", "temp"] {
            std::fs::create_dir_all(self.output_base.join(subdir))?;
        }
        Ok(())
    }

    fn render_pixels_gpu(
        &self,
        scene: &SceneGeometry,
        camera: &CameraRig,
        lighting: &LightingRig,
        quality: RenderQuality,
        width: u32,
        height: u32,
    ) -> Option<Vec<u8>> {
        if let Some(ref gpu) = self.gpu_renderer {
            match gpu.render(scene, camera, lighting, quality, width, height) {
                Ok(pixels) => return Some(pixels),
                Err(e) => {
                    warn!("[LUXOR] GPU render failed, falling back to CPU: {}", e);
                }
            }
        }
        None
    }

    fn render_pixels_cpu(
        &self,
        scene: &SceneGeometry,
        camera: &CameraRig,
        lighting: &LightingRig,
        quality: RenderQuality,
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        let rt = LuxorRaytracer::new(camera.clone(), lighting.clone(), quality);
        rt.render(scene, width, height)
    }

    fn render_pixels(
        &self,
        scene: &SceneGeometry,
        camera: &CameraRig,
        lighting: &LightingRig,
        quality: RenderQuality,
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        self.render_pixels_gpu(scene, camera, lighting, quality, width, height)
            .unwrap_or_else(|| {
                self.render_pixels_cpu(scene, camera, lighting, quality, width, height)
            })
    }

    pub fn render_snapshot(
        &self,
        scene: &SceneGeometry,
        request: &SnapshotRequest,
    ) -> Result<PathBuf> {
        self.ensure_directories()?;

        let (width, height) = request.output.size.resolution();
        let backend = if self.gpu_renderer.is_some() {
            "GPU"
        } else {
            "CPU"
        };
        info!(
            "[LUXOR] Rendering snapshot '{}' at {}x{} ({:?}) via {}",
            request.name, width, height, request.output.quality, backend
        );

        let mut img = self.render_pixels(
            scene,
            &request.camera,
            &request.lighting,
            request.output.quality,
            width,
            height,
        );

        for effect in &request.output.effects {
            post_process::apply_effect(&mut img, effect, width, height);
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let ext = request.output.format.extension();
        let filename = format!("luxor_{}_{}.{}", request.name, timestamp, ext);
        let output_path = self.output_base.join("images").join(&filename);

        let img_buf = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, img)
            .ok_or_else(|| anyhow!("Failed to create image buffer"))?;

        match request.output.format {
            OutputFormat::Png => img_buf.save_with_format(&output_path, image::ImageFormat::Png)?,
            OutputFormat::Jpeg => {
                let rgb = image::DynamicImage::ImageRgba8(img_buf).to_rgb8();
                rgb.save_with_format(&output_path, image::ImageFormat::Jpeg)?;
            }
            OutputFormat::Bmp => img_buf.save_with_format(&output_path, image::ImageFormat::Bmp)?,
        }

        info!(
            "[LUXOR] Snapshot saved: {} ({} bytes)",
            output_path.display(),
            std::fs::metadata(&output_path)
                .map(|m| m.len())
                .unwrap_or(0)
        );

        Ok(output_path)
    }

    pub fn render_video_frames(
        &self,
        scene: &SceneGeometry,
        request: &VideoRequest,
        frames_dir: &std::path::Path,
    ) -> Result<()> {
        let (width, height) = request.output.size.resolution();
        let total_frames = request.job.total_frames();
        let interpolated_cameras = request.job.interpolate_cameras();

        for (frame_idx, cam) in interpolated_cameras.iter().enumerate() {
            let mut img = self.render_pixels(
                scene,
                cam,
                &request.lighting,
                request.output.quality,
                width,
                height,
            );

            for effect in &request.output.effects {
                post_process::apply_effect(&mut img, effect, width, height);
            }

            let img_buf =
                image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, img)
                    .ok_or_else(|| anyhow!("Failed to create frame buffer"))?;

            let frame_path = frames_dir.join(format!("frame_{:04}.png", frame_idx));
            img_buf.save_with_format(&frame_path, image::ImageFormat::Png)?;

            if frame_idx % 30 == 0 {
                info!("[LUXOR] Rendered frame {}/{}", frame_idx + 1, total_frames);
            }
        }
        Ok(())
    }

    pub async fn render_video(
        &self,
        scene: &SceneGeometry,
        request: &VideoRequest,
    ) -> Result<PathBuf> {
        self.ensure_directories()?;

        let (width, height) = request.output.size.resolution();
        let fps = request.job.fps;
        let total_frames = request.job.total_frames();

        let backend = if self.gpu_renderer.is_some() {
            "GPU"
        } else {
            "CPU"
        };
        info!(
            "[LUXOR] Rendering video '{}' — {} frames at {}x{} {}fps ({:?}) via {}",
            request.name, total_frames, width, height, fps, request.output.quality, backend
        );

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let frames_dir = self
            .output_base
            .join("temp")
            .join(format!("luxor_frames_{}", timestamp));
        std::fs::create_dir_all(&frames_dir)?;

        let director = self.clone();
        let scene_clone = scene.clone();
        let request_clone = request.clone();
        let frames_dir_clone = frames_dir.clone();
        tokio::task::spawn_blocking(move || {
            director.render_video_frames(&scene_clone, &request_clone, &frames_dir_clone)
        })
        .await
        .map_err(|e| anyhow!("Frame render task panicked: {}", e))??;

        let video_filename = format!("luxor_{}_{}.mp4", request.name, timestamp);
        let video_path = self.output_base.join("video").join(&video_filename);

        crate::media::encoder::encode_video(&self.ffmpeg_path, &frames_dir, &video_path, fps)
            .await?;

        if let Err(e) = std::fs::remove_dir_all(&frames_dir) {
            warn!("[LUXOR] Failed to clean temp frames: {}", e);
        }

        info!(
            "[LUXOR] Video saved: {} ({} bytes)",
            video_path.display(),
            std::fs::metadata(&video_path).map(|m| m.len()).unwrap_or(0)
        );

        Ok(video_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_sizes() {
        assert_eq!(ScreenSize::SD.resolution(), (640, 480));
        assert_eq!(ScreenSize::FullHD.resolution(), (1920, 1080));
        assert_eq!(ScreenSize::UHD4K.resolution(), (3840, 2160));
        assert_eq!(ScreenSize::Cinema.resolution(), (2560, 1080));
        assert_eq!(ScreenSize::Custom(800, 600).resolution(), (800, 600));
    }

    #[test]
    fn test_screen_size_from_name() {
        assert_eq!(ScreenSize::from_name("4k"), Some(ScreenSize::UHD4K));
        assert_eq!(ScreenSize::from_name("1080p"), Some(ScreenSize::FullHD));
        assert_eq!(ScreenSize::from_name("square"), Some(ScreenSize::Square));
        assert_eq!(
            ScreenSize::from_name("800x600"),
            Some(ScreenSize::Custom(800, 600))
        );
        assert_eq!(ScreenSize::from_name("invalid"), None);
    }

    #[test]
    fn test_aspect_ratios() {
        let ratio_16_9 = ScreenSize::FullHD.aspect_ratio();
        assert!((ratio_16_9 - 16.0 / 9.0).abs() < 0.01);

        let ratio_1_1 = ScreenSize::Square.aspect_ratio();
        assert!((ratio_1_1 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_output_format() {
        assert_eq!(OutputFormat::Png.extension(), "png");
        assert_eq!(OutputFormat::from_name("jpg"), OutputFormat::Jpeg);
        assert_eq!(OutputFormat::from_name("unknown"), OutputFormat::Png);
    }
}
