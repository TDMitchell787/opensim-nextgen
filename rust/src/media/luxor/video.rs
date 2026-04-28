use super::camera::CameraRig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraKeyframe {
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub focal_length_mm: f32,
    pub f_stop: f32,
    pub time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoJob {
    pub keyframes: Vec<CameraKeyframe>,
    pub fps: u32,
    pub duration_secs: f32,
    pub path_type: PathType,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathType {
    Linear,
    CatmullRom,
    Orbit,
}

impl PathType {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "linear" | "straight" => Some(PathType::Linear),
            "catmullrom" | "catmull-rom" | "catmull_rom" | "smooth" | "spline" => {
                Some(PathType::CatmullRom)
            }
            "orbit" | "circle" | "rotate" => Some(PathType::Orbit),
            _ => None,
        }
    }
}

impl VideoJob {
    pub fn total_frames(&self) -> u32 {
        (self.duration_secs * self.fps as f32).ceil() as u32
    }

    pub fn orbit(center: [f32; 3], radius: f32, height: f32, duration_secs: f32, fps: u32) -> Self {
        let steps = 8;
        let mut keyframes = Vec::with_capacity(steps + 1);

        for i in 0..=steps {
            let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
            let x = center[0] + radius * angle.cos();
            let y = center[1] + radius * angle.sin();
            let z = center[2] + height;

            keyframes.push(CameraKeyframe {
                position: [x, y, z],
                look_at: center,
                focal_length_mm: 35.0,
                f_stop: 4.0,
                time: i as f32 / steps as f32 * duration_secs,
            });
        }

        Self {
            keyframes,
            fps,
            duration_secs,
            path_type: PathType::Orbit,
        }
    }

    pub fn dolly(
        start: [f32; 3],
        end: [f32; 3],
        look_at: [f32; 3],
        duration_secs: f32,
        fps: u32,
    ) -> Self {
        let keyframes = vec![
            CameraKeyframe {
                position: start,
                look_at,
                focal_length_mm: 50.0,
                f_stop: 4.0,
                time: 0.0,
            },
            CameraKeyframe {
                position: end,
                look_at,
                focal_length_mm: 50.0,
                f_stop: 4.0,
                time: duration_secs,
            },
        ];

        Self {
            keyframes,
            fps,
            duration_secs,
            path_type: PathType::Linear,
        }
    }

    pub fn interpolate_cameras(&self) -> Vec<CameraRig> {
        let total = self.total_frames();
        if total == 0 || self.keyframes.is_empty() {
            return vec![CameraRig::default()];
        }

        let mut cameras = Vec::with_capacity(total as usize);

        for frame in 0..total {
            let t = frame as f32 / total as f32 * self.duration_secs;
            let camera = self.camera_at_time(t);
            cameras.push(camera);
        }

        cameras
    }

    fn camera_at_time(&self, t: f32) -> CameraRig {
        if self.keyframes.len() == 1 {
            return keyframe_to_rig(&self.keyframes[0]);
        }

        let mut i1 = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time <= t {
                i1 = i;
            }
        }
        let i2 = (i1 + 1).min(self.keyframes.len() - 1);

        if i1 == i2 {
            return keyframe_to_rig(&self.keyframes[i1]);
        }

        let kf1 = &self.keyframes[i1];
        let kf2 = &self.keyframes[i2];
        let segment_t = if (kf2.time - kf1.time).abs() > 1e-6 {
            ((t - kf1.time) / (kf2.time - kf1.time)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        match self.path_type {
            PathType::Linear => interpolate_linear(kf1, kf2, segment_t),
            PathType::CatmullRom | PathType::Orbit => {
                let i0 = if i1 > 0 { i1 - 1 } else { i1 };
                let i3 = (i2 + 1).min(self.keyframes.len() - 1);
                interpolate_catmull_rom(
                    &self.keyframes[i0],
                    kf1,
                    kf2,
                    &self.keyframes[i3],
                    segment_t,
                )
            }
        }
    }
}

fn keyframe_to_rig(kf: &CameraKeyframe) -> CameraRig {
    CameraRig {
        position: kf.position,
        look_at: kf.look_at,
        focal_length_mm: kf.focal_length_mm,
        f_stop: kf.f_stop,
        ..CameraRig::default()
    }
}

fn interpolate_linear(kf1: &CameraKeyframe, kf2: &CameraKeyframe, t: f32) -> CameraRig {
    CameraRig {
        position: lerp3(kf1.position, kf2.position, t),
        look_at: lerp3(kf1.look_at, kf2.look_at, t),
        focal_length_mm: kf1.focal_length_mm + (kf2.focal_length_mm - kf1.focal_length_mm) * t,
        f_stop: kf1.f_stop + (kf2.f_stop - kf1.f_stop) * t,
        ..CameraRig::default()
    }
}

fn interpolate_catmull_rom(
    kf0: &CameraKeyframe,
    kf1: &CameraKeyframe,
    kf2: &CameraKeyframe,
    kf3: &CameraKeyframe,
    t: f32,
) -> CameraRig {
    CameraRig {
        position: catmull_rom_3(kf0.position, kf1.position, kf2.position, kf3.position, t),
        look_at: catmull_rom_3(kf0.look_at, kf1.look_at, kf2.look_at, kf3.look_at, t),
        focal_length_mm: catmull_rom_1(
            kf0.focal_length_mm,
            kf1.focal_length_mm,
            kf2.focal_length_mm,
            kf3.focal_length_mm,
            t,
        ),
        f_stop: catmull_rom_1(kf0.f_stop, kf1.f_stop, kf2.f_stop, kf3.f_stop, t),
        ..CameraRig::default()
    }
}

fn catmull_rom_1(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

fn catmull_rom_3(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], t: f32) -> [f32; 3] {
    [
        catmull_rom_1(p0[0], p1[0], p2[0], p3[0], t),
        catmull_rom_1(p0[1], p1[1], p2[1], p3[1], t),
        catmull_rom_1(p0[2], p1[2], p2[2], p3[2], t),
    ]
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orbit_keyframes() {
        let job = VideoJob::orbit([128.0, 128.0, 25.0], 10.0, 5.0, 10.0, 30);
        assert_eq!(job.keyframes.len(), 9);
        assert_eq!(job.total_frames(), 300);
        assert_eq!(job.fps, 30);
    }

    #[test]
    fn test_dolly_keyframes() {
        let job = VideoJob::dolly(
            [118.0, 128.0, 27.0],
            [138.0, 128.0, 27.0],
            [128.0, 128.0, 25.0],
            5.0,
            24,
        );
        assert_eq!(job.keyframes.len(), 2);
        assert_eq!(job.total_frames(), 120);
    }

    #[test]
    fn test_interpolate_linear() {
        let job = VideoJob::dolly(
            [0.0, 0.0, 10.0],
            [20.0, 0.0, 10.0],
            [10.0, 10.0, 0.0],
            2.0,
            30,
        );

        let cameras = job.interpolate_cameras();
        assert_eq!(cameras.len(), 60);

        assert!((cameras[0].position[0] - 0.0).abs() < 0.5);
        assert!((cameras[59].position[0] - 20.0).abs() < 1.0);
    }

    #[test]
    fn test_catmull_rom_passes_through() {
        let val = catmull_rom_1(0.0, 1.0, 2.0, 3.0, 0.0);
        assert!((val - 1.0).abs() < 0.01, "t=0 should give p1, got {}", val);

        let val2 = catmull_rom_1(0.0, 1.0, 2.0, 3.0, 1.0);
        assert!(
            (val2 - 2.0).abs() < 0.01,
            "t=1 should give p2, got {}",
            val2
        );
    }

    #[test]
    fn test_orbit_smooth() {
        let job = VideoJob::orbit([128.0, 128.0, 25.0], 10.0, 5.0, 5.0, 24);
        let cameras = job.interpolate_cameras();

        for i in 1..cameras.len() {
            let dx = cameras[i].position[0] - cameras[i - 1].position[0];
            let dy = cameras[i].position[1] - cameras[i - 1].position[1];
            let dz = cameras[i].position[2] - cameras[i - 1].position[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            assert!(dist < 5.0, "Frame {} jump too large: {}", i, dist);
        }
    }

    #[test]
    fn test_path_type_from_name() {
        assert_eq!(PathType::from_name("orbit"), Some(PathType::Orbit));
        assert_eq!(PathType::from_name("linear"), Some(PathType::Linear));
        assert_eq!(
            PathType::from_name("catmull-rom"),
            Some(PathType::CatmullRom)
        );
        assert_eq!(PathType::from_name("invalid"), None);
    }

    #[test]
    fn test_single_keyframe() {
        let job = VideoJob {
            keyframes: vec![CameraKeyframe {
                position: [10.0, 20.0, 30.0],
                look_at: [0.0, 0.0, 0.0],
                focal_length_mm: 50.0,
                f_stop: 4.0,
                time: 0.0,
            }],
            fps: 24,
            duration_secs: 1.0,
            path_type: PathType::Linear,
        };

        let cameras = job.interpolate_cameras();
        assert!(!cameras.is_empty());
        assert!((cameras[0].position[0] - 10.0).abs() < 0.01);
    }
}
