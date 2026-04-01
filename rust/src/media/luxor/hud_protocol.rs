use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use super::{ScreenSize, OutputFormat, OutputSettings, SnapshotRequest, VideoRequest, LUXOR_CHANNEL};
use super::camera::{CameraRig, CameraPreset};
use super::lighting::{LightingRig, LightingPreset};
use super::raytracer::RenderQuality;
use super::post_process::PostEffect;
use super::video::{VideoJob, PathType, CameraKeyframe};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuxorCommand {
    pub cmd: String,
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub quality: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub effects: Option<Vec<String>>,
    #[serde(default)]
    pub focal: Option<f32>,
    #[serde(default)]
    pub fstop: Option<f32>,
    #[serde(default)]
    pub focus: Option<f32>,
    #[serde(default)]
    pub pos: Option<[f32; 3]>,
    #[serde(default)]
    pub lookat: Option<[f32; 3]>,
    #[serde(default)]
    pub slot: Option<usize>,
    #[serde(default)]
    pub light_type: Option<String>,
    #[serde(default)]
    pub color: Option<[f32; 3]>,
    #[serde(default)]
    pub intensity: Option<f32>,
    #[serde(default)]
    pub radius: Option<f32>,
    #[serde(default)]
    pub spot_angle: Option<f32>,
    #[serde(default)]
    pub fps: Option<u32>,
    #[serde(default)]
    pub duration: Option<f32>,
    #[serde(default)]
    pub path_type: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub lighting: Option<String>,
}

#[derive(Debug)]
pub enum LuxorAction {
    Snapshot(SnapshotRequest),
    Preview(SnapshotRequest),
    SetCamera(CameraRig),
    SetLight { slot: usize, light: super::lighting::LuxorLight },
    SetEffects(Vec<PostEffect>),
    RecordStart(VideoRequest),
    RecordStop,
    AddWaypoint(CameraKeyframe),
    Status,
}

pub struct HudState {
    pub camera: CameraRig,
    pub lighting: LightingRig,
    pub quality: RenderQuality,
    pub effects: Vec<PostEffect>,
    pub size: ScreenSize,
    pub format: OutputFormat,
    pub waypoints: Vec<CameraKeyframe>,
    pub recording: bool,
    pub recording_fps: u32,
    pub recording_duration: f32,
    pub recording_path_type: PathType,
    pub recording_name: String,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            camera: CameraRig::default(),
            lighting: LightingPreset::GoldenHour.build_rig([128.0, 128.0, 25.0], 10.0),
            quality: RenderQuality::Standard,
            effects: Vec::new(),
            size: ScreenSize::FullHD,
            format: OutputFormat::Png,
            waypoints: Vec::new(),
            recording: false,
            recording_fps: 30,
            recording_duration: 10.0,
            recording_path_type: PathType::CatmullRom,
            recording_name: "luxor_video".to_string(),
        }
    }
}

impl HudState {
    pub fn parse_command(&mut self, json_str: &str) -> Result<LuxorAction, String> {
        let cmd: LuxorCommand = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        match cmd.cmd.to_lowercase().as_str() {
            "snapshot" | "snap" | "photo" | "capture" => {
                self.apply_settings(&cmd);
                let request = self.build_snapshot_request(
                    cmd.name.as_deref().unwrap_or("snapshot"),
                );
                Ok(LuxorAction::Snapshot(request))
            }
            "preview" => {
                self.apply_settings(&cmd);
                let mut request = self.build_snapshot_request("preview");
                request.output.size = ScreenSize::SD;
                request.output.quality = RenderQuality::Draft;
                Ok(LuxorAction::Preview(request))
            }
            "set_camera" | "camera" => {
                if let Some(preset_name) = &cmd.preset {
                    if let Some(preset) = CameraPreset::from_name(preset_name) {
                        preset.apply(&mut self.camera);
                    }
                }
                if let Some(focal) = cmd.focal {
                    self.camera.focal_length_mm = focal;
                }
                if let Some(fstop) = cmd.fstop {
                    self.camera.f_stop = fstop;
                }
                if let Some(focus) = cmd.focus {
                    self.camera.focus_distance = focus;
                }
                if let Some(pos) = cmd.pos {
                    self.camera.position = pos;
                }
                if let Some(lookat) = cmd.lookat {
                    self.camera.look_at = lookat;
                }
                info!("[LUXOR HUD] Camera updated: focal={}mm f/{} focus={}m",
                    self.camera.focal_length_mm, self.camera.f_stop, self.camera.focus_distance);
                Ok(LuxorAction::SetCamera(self.camera.clone()))
            }
            "set_light" | "light" => {
                let slot = cmd.slot.unwrap_or(0);
                let light = super::lighting::LuxorLight {
                    position: cmd.pos.unwrap_or([128.0, 128.0, 30.0]),
                    direction: [0.0, 0.0, -1.0],
                    color: cmd.color.unwrap_or([1.0, 1.0, 1.0]),
                    intensity: cmd.intensity.unwrap_or(500.0),
                    light_type: match cmd.light_type.as_deref() {
                        Some("spot") => super::lighting::LightType::Spot,
                        Some("directional") | Some("sun") => super::lighting::LightType::Directional,
                        Some("area") => super::lighting::LightType::Area,
                        _ => super::lighting::LightType::Point,
                    },
                    radius: cmd.radius.unwrap_or(20.0),
                    spot_angle: cmd.spot_angle.unwrap_or(45.0),
                    soft_edge: 0.1,
                };
                if slot < self.lighting.lights.len() {
                    self.lighting.lights[slot] = light.clone();
                } else {
                    self.lighting.lights.push(light.clone());
                }
                info!("[LUXOR HUD] Light slot {} updated: {:?} intensity={}",
                    slot, light.light_type, light.intensity);
                Ok(LuxorAction::SetLight { slot, light })
            }
            "set_lighting" | "lighting_preset" => {
                if let Some(preset_name) = &cmd.preset.as_ref().or(cmd.lighting.as_ref()) {
                    let center = self.camera.look_at;
                    if let Some(preset) = LightingPreset::from_name(preset_name) {
                        self.lighting = preset.build_rig(center, 10.0);
                        info!("[LUXOR HUD] Lighting preset '{}' applied", preset_name);
                    } else {
                        return Err(format!("Unknown lighting preset: {}", preset_name));
                    }
                }
                Ok(LuxorAction::SetCamera(self.camera.clone()))
            }
            "set_effect" | "set_effects" | "effects" => {
                self.effects.clear();
                if let Some(names) = &cmd.effects {
                    for name in names {
                        if let Some(effect) = PostEffect::from_name(name) {
                            self.effects.push(effect);
                        } else {
                            warn!("[LUXOR HUD] Unknown effect: {}", name);
                        }
                    }
                }
                info!("[LUXOR HUD] Effects set: {:?}", self.effects);
                Ok(LuxorAction::SetEffects(self.effects.clone()))
            }
            "record_start" | "record" | "video" => {
                self.apply_settings(&cmd);
                self.recording_fps = cmd.fps.unwrap_or(30);
                self.recording_duration = cmd.duration.unwrap_or(10.0);
                if let Some(pt) = cmd.path_type.as_deref().and_then(PathType::from_name) {
                    self.recording_path_type = pt;
                }
                if let Some(name) = &cmd.name {
                    self.recording_name = name.clone();
                }
                self.recording = true;

                let job = if self.waypoints.len() >= 2 {
                    VideoJob {
                        keyframes: self.waypoints.clone(),
                        fps: self.recording_fps,
                        duration_secs: self.recording_duration,
                        path_type: self.recording_path_type,
                    }
                } else {
                    VideoJob::orbit(
                        self.camera.look_at,
                        10.0,
                        5.0,
                        self.recording_duration,
                        self.recording_fps,
                    )
                };

                let request = VideoRequest {
                    job,
                    lighting: self.lighting.clone(),
                    output: OutputSettings {
                        size: self.size,
                        quality: self.quality,
                        effects: self.effects.clone(),
                        format: OutputFormat::Png,
                    },
                    region_id: uuid::Uuid::nil(),
                    name: self.recording_name.clone(),
                };

                info!("[LUXOR HUD] Recording started: {} frames at {}fps, {:?} path",
                    request.job.total_frames(), self.recording_fps, self.recording_path_type);

                Ok(LuxorAction::RecordStart(request))
            }
            "record_stop" | "stop" => {
                self.recording = false;
                self.waypoints.clear();
                info!("[LUXOR HUD] Recording stopped");
                Ok(LuxorAction::RecordStop)
            }
            "add_waypoint" | "waypoint" | "keyframe" => {
                let pos = cmd.pos.unwrap_or(self.camera.position);
                let lookat = cmd.lookat.unwrap_or(self.camera.look_at);
                let focal = cmd.focal.unwrap_or(self.camera.focal_length_mm);
                let fstop = cmd.fstop.unwrap_or(self.camera.f_stop);

                let time = if self.waypoints.is_empty() {
                    0.0
                } else {
                    let last_time = self.waypoints.last().map(|kf| kf.time).unwrap_or(0.0);
                    last_time + cmd.duration.unwrap_or(2.0)
                };

                let kf = CameraKeyframe {
                    position: pos,
                    look_at: lookat,
                    focal_length_mm: focal,
                    f_stop: fstop,
                    time,
                };

                self.waypoints.push(kf.clone());
                info!("[LUXOR HUD] Waypoint #{} added at t={:.1}s pos={:?}",
                    self.waypoints.len(), time, pos);

                Ok(LuxorAction::AddWaypoint(kf))
            }
            "status" | "info" => {
                Ok(LuxorAction::Status)
            }
            "reset" | "clear" => {
                *self = HudState::default();
                info!("[LUXOR HUD] State reset to defaults");
                Ok(LuxorAction::Status)
            }
            other => {
                Err(format!("Unknown Luxor command: '{}'. Valid: snapshot, preview, set_camera, \
                    set_light, set_lighting, set_effect, record_start, record_stop, \
                    add_waypoint, status, reset", other))
            }
        }
    }

    fn apply_settings(&mut self, cmd: &LuxorCommand) {
        if let Some(preset_name) = &cmd.preset {
            if let Some(preset) = CameraPreset::from_name(preset_name) {
                preset.apply(&mut self.camera);
            }
        }
        if let Some(size_name) = &cmd.size {
            if let Some(size) = ScreenSize::from_name(size_name) {
                self.size = size;
            }
        }
        if let Some(quality_name) = &cmd.quality {
            if let Some(q) = RenderQuality::from_name(quality_name) {
                self.quality = q;
            }
        }
        if let Some(format_name) = &cmd.format {
            self.format = OutputFormat::from_name(format_name);
        }
        if let Some(names) = &cmd.effects {
            self.effects.clear();
            for name in names {
                if let Some(effect) = PostEffect::from_name(name) {
                    self.effects.push(effect);
                }
            }
        }
        if let Some(lighting_name) = &cmd.lighting {
            if let Some(preset) = LightingPreset::from_name(lighting_name) {
                self.lighting = preset.build_rig(self.camera.look_at, 10.0);
            }
        }
        if let Some(focal) = cmd.focal {
            self.camera.focal_length_mm = focal;
        }
        if let Some(fstop) = cmd.fstop {
            self.camera.f_stop = fstop;
        }
        if let Some(focus) = cmd.focus {
            self.camera.focus_distance = focus;
        }
        if let Some(pos) = cmd.pos {
            self.camera.position = pos;
        }
        if let Some(lookat) = cmd.lookat {
            self.camera.look_at = lookat;
        }
    }

    fn build_snapshot_request(&self, name: &str) -> SnapshotRequest {
        SnapshotRequest {
            camera: self.camera.clone(),
            lighting: self.lighting.clone(),
            output: OutputSettings {
                size: self.size,
                quality: self.quality,
                effects: self.effects.clone(),
                format: self.format,
            },
            region_id: uuid::Uuid::nil(),
            name: name.to_string(),
        }
    }

    pub fn status_text(&self) -> String {
        let (w, h) = self.size.resolution();
        format!(
            "Luxor Camera v{}\n\
             Camera: {}mm f/{} focus={}m\n\
             Pos: [{:.1},{:.1},{:.1}] → [{:.1},{:.1},{:.1}]\n\
             Size: {}x{} ({}) Quality: {:?}\n\
             Effects: {}\n\
             Waypoints: {} Recording: {}",
            super::LUXOR_VERSION,
            self.camera.focal_length_mm,
            self.camera.f_stop,
            self.camera.focus_distance,
            self.camera.position[0], self.camera.position[1], self.camera.position[2],
            self.camera.look_at[0], self.camera.look_at[1], self.camera.look_at[2],
            w, h, self.size.display_name(),
            self.quality,
            if self.effects.is_empty() { "none".to_string() }
            else { format!("{:?}", self.effects) },
            self.waypoints.len(),
            if self.recording { "YES" } else { "no" },
        )
    }
}

pub fn is_luxor_channel(channel: i32) -> bool {
    channel == LUXOR_CHANNEL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_snapshot() {
        let mut state = HudState::default();
        let cmd = r#"{"cmd": "snapshot", "preset": "portrait", "size": "4K", "effects": ["vignette", "warm"]}"#;
        let result = state.parse_command(cmd);
        assert!(result.is_ok());
        if let Ok(LuxorAction::Snapshot(req)) = result {
            assert_eq!(req.output.size.resolution(), (3840, 2160));
            assert_eq!(req.output.effects.len(), 2);
            assert!((req.camera.focal_length_mm - 85.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_parse_set_camera() {
        let mut state = HudState::default();
        let cmd = r#"{"cmd": "set_camera", "focal": 85, "fstop": 1.8, "focus": 5.0}"#;
        let result = state.parse_command(cmd);
        assert!(result.is_ok());
        assert!((state.camera.focal_length_mm - 85.0).abs() < 0.1);
        assert!((state.camera.f_stop - 1.8).abs() < 0.1);
        assert!((state.camera.focus_distance - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_parse_set_light() {
        let mut state = HudState::default();
        let cmd = r#"{"cmd": "set_light", "slot": 0, "light_type": "spot", "pos": [130,128,27], "color": [1,0.9,0.8], "intensity": 800}"#;
        let result = state.parse_command(cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_record_start() {
        let mut state = HudState::default();
        let cmd = r#"{"cmd": "record_start", "fps": 30, "size": "1080p", "path_type": "orbit", "duration": 10}"#;
        let result = state.parse_command(cmd);
        assert!(result.is_ok());
        assert!(state.recording);
        if let Ok(LuxorAction::RecordStart(req)) = result {
            assert_eq!(req.job.fps, 30);
            assert_eq!(req.job.duration_secs, 10.0);
        }
    }

    #[test]
    fn test_parse_add_waypoint() {
        let mut state = HudState::default();
        let cmd1 = r#"{"cmd": "add_waypoint", "pos": [128,128,30], "lookat": [128,128,25], "focal": 50}"#;
        let r1 = state.parse_command(cmd1);
        assert!(r1.is_ok());
        assert_eq!(state.waypoints.len(), 1);

        let cmd2 = r#"{"cmd": "add_waypoint", "pos": [138,128,30], "lookat": [128,128,25]}"#;
        let r2 = state.parse_command(cmd2);
        assert!(r2.is_ok());
        assert_eq!(state.waypoints.len(), 2);
        assert!(state.waypoints[1].time > 0.0);
    }

    #[test]
    fn test_parse_set_effects() {
        let mut state = HudState::default();
        let cmd = r#"{"cmd": "set_effect", "effects": ["noir", "film_grain", "letterbox"]}"#;
        let result = state.parse_command(cmd);
        assert!(result.is_ok());
        assert_eq!(state.effects.len(), 3);
    }

    #[test]
    fn test_parse_preview() {
        let mut state = HudState::default();
        let cmd = r#"{"cmd": "preview"}"#;
        let result = state.parse_command(cmd);
        assert!(result.is_ok());
        if let Ok(LuxorAction::Preview(req)) = result {
            assert_eq!(req.output.size.resolution(), (640, 480));
        }
    }

    #[test]
    fn test_parse_invalid_command() {
        let mut state = HudState::default();
        let cmd = r#"{"cmd": "explode"}"#;
        let result = state.parse_command(cmd);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json() {
        let mut state = HudState::default();
        let result = state.parse_command("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_status_text() {
        let state = HudState::default();
        let text = state.status_text();
        assert!(text.contains("Luxor Camera"));
        assert!(text.contains("Recording: no"));
    }

    #[test]
    fn test_reset() {
        let mut state = HudState::default();
        state.camera.focal_length_mm = 200.0;
        state.recording = true;
        state.waypoints.push(CameraKeyframe {
            position: [0.0; 3], look_at: [0.0; 3],
            focal_length_mm: 50.0, f_stop: 4.0, time: 0.0,
        });

        let result = state.parse_command(r#"{"cmd": "reset"}"#);
        assert!(result.is_ok());
        assert!((state.camera.focal_length_mm - 50.0).abs() < 0.1);
        assert!(!state.recording);
        assert!(state.waypoints.is_empty());
    }

    #[test]
    fn test_is_luxor_channel() {
        assert!(is_luxor_channel(-15500));
        assert!(!is_luxor_channel(-15400));
        assert!(!is_luxor_channel(0));
    }

    #[test]
    fn test_record_with_waypoints() {
        let mut state = HudState::default();
        state.parse_command(r#"{"cmd": "add_waypoint", "pos": [118,128,30], "lookat": [128,128,25]}"#).unwrap();
        state.parse_command(r#"{"cmd": "add_waypoint", "pos": [138,128,30], "lookat": [128,128,25]}"#).unwrap();
        state.parse_command(r#"{"cmd": "add_waypoint", "pos": [138,138,30], "lookat": [128,128,25]}"#).unwrap();

        let result = state.parse_command(r#"{"cmd": "record_start", "fps": 24, "duration": 5}"#);
        assert!(result.is_ok());
        if let Ok(LuxorAction::RecordStart(req)) = result {
            assert_eq!(req.job.keyframes.len(), 3);
            assert_eq!(req.job.fps, 24);
        }
    }
}
