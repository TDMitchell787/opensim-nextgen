use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraRig {
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub up: [f32; 3],
    pub focal_length_mm: f32,
    pub sensor_width_mm: f32,
    pub sensor_height_mm: f32,
    pub f_stop: f32,
    pub focus_distance: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            position: [128.0, 118.0, 27.0],
            look_at: [128.0, 128.0, 25.0],
            up: [0.0, 0.0, 1.0],
            focal_length_mm: 50.0,
            sensor_width_mm: 36.0,
            sensor_height_mm: 24.0,
            f_stop: 5.6,
            focus_distance: 10.0,
        }
    }
}

impl CameraRig {
    pub fn fov_horizontal(&self) -> f32 {
        2.0 * (self.sensor_width_mm / (2.0 * self.focal_length_mm)).atan()
    }

    pub fn fov_vertical(&self) -> f32 {
        2.0 * (self.sensor_height_mm / (2.0 * self.focal_length_mm)).atan()
    }

    pub fn fov_horizontal_deg(&self) -> f32 {
        self.fov_horizontal().to_degrees()
    }

    pub fn fov_vertical_deg(&self) -> f32 {
        self.fov_vertical().to_degrees()
    }

    pub fn circle_of_confusion_mm(&self) -> f32 {
        self.sensor_width_mm / 1500.0
    }

    pub fn hyperfocal_distance(&self) -> f32 {
        let coc = self.circle_of_confusion_mm();
        let f_mm = self.focal_length_mm;
        let h_mm = (f_mm * f_mm) / (self.f_stop * coc) + f_mm;
        h_mm / 1000.0
    }

    pub fn dof_near(&self) -> f32 {
        let h = self.hyperfocal_distance();
        let s = self.focus_distance;
        if s >= h {
            return 0.0;
        }
        (h * s) / (h + (s - self.focal_length_mm / 1000.0))
    }

    pub fn dof_far(&self) -> f32 {
        let h = self.hyperfocal_distance();
        let s = self.focus_distance;
        if s >= h {
            return f32::INFINITY;
        }
        let denom = h - (s - self.focal_length_mm / 1000.0);
        if denom <= 0.0 {
            return f32::INFINITY;
        }
        (h * s) / denom
    }

    pub fn aperture_diameter_mm(&self) -> f32 {
        self.focal_length_mm / self.f_stop
    }

    pub fn aperture_radius_m(&self) -> f32 {
        self.aperture_diameter_mm() / 2000.0
    }

    pub fn has_dof_effect(&self) -> bool {
        self.f_stop < 16.0
    }

    pub fn forward(&self) -> [f32; 3] {
        let dx = self.look_at[0] - self.position[0];
        let dy = self.look_at[1] - self.position[1];
        let dz = self.look_at[2] - self.position[2];
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-8);
        [dx / len, dy / len, dz / len]
    }

    pub fn right(&self) -> [f32; 3] {
        let fwd = self.forward();
        cross(fwd, self.up)
    }

    pub fn camera_up(&self) -> [f32; 3] {
        let fwd = self.forward();
        let r = self.right();
        cross(r, fwd)
    }

    pub fn generate_ray(&self, u: f32, v: f32, aspect_ratio: f32) -> Ray {
        let half_width = (self.fov_horizontal() / 2.0).tan();
        let half_height = half_width / aspect_ratio;

        let fwd = self.forward();
        let r = normalize(self.right());
        let cup = self.camera_up();

        let pixel_x = (2.0 * u - 1.0) * half_width;
        let pixel_y = (2.0 * v - 1.0) * half_height;

        let dir = normalize([
            fwd[0] + pixel_x * r[0] + pixel_y * cup[0],
            fwd[1] + pixel_x * r[1] + pixel_y * cup[1],
            fwd[2] + pixel_x * r[2] + pixel_y * cup[2],
        ]);

        Ray {
            origin: self.position,
            direction: dir,
        }
    }

    pub fn generate_dof_ray(
        &self,
        u: f32,
        v: f32,
        aspect_ratio: f32,
        lens_u: f32,
        lens_v: f32,
    ) -> Ray {
        let primary = self.generate_ray(u, v, aspect_ratio);

        let focus_t = self.focus_distance / dot(primary.direction, self.forward()).max(1e-8);
        let focus_point = [
            primary.origin[0] + primary.direction[0] * focus_t,
            primary.origin[1] + primary.direction[1] * focus_t,
            primary.origin[2] + primary.direction[2] * focus_t,
        ];

        let r = normalize(self.right());
        let cup = self.camera_up();
        let aperture_r = self.aperture_radius_m();

        let (disk_x, disk_y) = concentric_disk_sample(lens_u, lens_v);
        let offset_x = disk_x * aperture_r;
        let offset_y = disk_y * aperture_r;

        let new_origin = [
            self.position[0] + r[0] * offset_x + cup[0] * offset_y,
            self.position[1] + r[1] * offset_x + cup[1] * offset_y,
            self.position[2] + r[2] * offset_x + cup[2] * offset_y,
        ];

        let new_dir = normalize([
            focus_point[0] - new_origin[0],
            focus_point[1] - new_origin[1],
            focus_point[2] - new_origin[2],
        ]);

        Ray {
            origin: new_origin,
            direction: new_dir,
        }
    }
}

fn concentric_disk_sample(u: f32, v: f32) -> (f32, f32) {
    let sx = 2.0 * u - 1.0;
    let sy = 2.0 * v - 1.0;

    if sx == 0.0 && sy == 0.0 {
        return (0.0, 0.0);
    }

    let (r, theta) = if sx.abs() > sy.abs() {
        (sx, std::f32::consts::FRAC_PI_4 * (sy / sx))
    } else {
        (sy, std::f32::consts::FRAC_PI_2 - std::f32::consts::FRAC_PI_4 * (sx / sy))
    };

    (r * theta.cos(), r * theta.sin())
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CameraPreset {
    Wide,
    Normal,
    Portrait,
    Telephoto,
    Macro,
    Cinematic,
    Drone,
    Security,
}

impl CameraPreset {
    pub fn apply(&self, rig: &mut CameraRig) {
        match self {
            CameraPreset::Wide => {
                rig.focal_length_mm = 24.0;
                rig.f_stop = 8.0;
            }
            CameraPreset::Normal => {
                rig.focal_length_mm = 50.0;
                rig.f_stop = 5.6;
            }
            CameraPreset::Portrait => {
                rig.focal_length_mm = 85.0;
                rig.f_stop = 1.8;
            }
            CameraPreset::Telephoto => {
                rig.focal_length_mm = 200.0;
                rig.f_stop = 4.0;
            }
            CameraPreset::Macro => {
                rig.focal_length_mm = 100.0;
                rig.f_stop = 2.8;
            }
            CameraPreset::Cinematic => {
                rig.focal_length_mm = 35.0;
                rig.f_stop = 2.0;
            }
            CameraPreset::Drone => {
                rig.focal_length_mm = 14.0;
                rig.f_stop = 5.6;
            }
            CameraPreset::Security => {
                rig.focal_length_mm = 28.0;
                rig.f_stop = 11.0;
            }
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "wide" | "landscape" | "architecture" => Some(CameraPreset::Wide),
            "normal" | "standard" | "natural" => Some(CameraPreset::Normal),
            "portrait" | "bokeh" => Some(CameraPreset::Portrait),
            "telephoto" | "tele" | "zoom" => Some(CameraPreset::Telephoto),
            "macro" | "closeup" | "close-up" => Some(CameraPreset::Macro),
            "cinematic" | "film" | "movie" => Some(CameraPreset::Cinematic),
            "drone" | "aerial" | "bird" => Some(CameraPreset::Drone),
            "security" | "surveillance" | "cctv" => Some(CameraPreset::Security),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

impl Ray {
    pub fn at(&self, t: f32) -> [f32; 3] {
        [
            self.origin[0] + self.direction[0] * t,
            self.origin[1] + self.direction[1] * t,
            self.origin[2] + self.direction[2] * t,
        ]
    }
}

pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-8);
    [v[0] / len, v[1] / len, v[2] / len]
}

pub fn length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

pub fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

pub fn mul_comp(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

pub fn reflect(incident: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let d = 2.0 * dot(incident, normal);
    sub(incident, scale(normal, d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_camera() {
        let cam = CameraRig::default();
        assert_eq!(cam.focal_length_mm, 50.0);
        assert_eq!(cam.sensor_width_mm, 36.0);
        let fov = cam.fov_horizontal_deg();
        assert!((fov - 39.6).abs() < 1.0, "50mm on 36mm sensor should be ~39.6° FOV, got {}", fov);
    }

    #[test]
    fn test_wide_fov() {
        let mut cam = CameraRig::default();
        CameraPreset::Wide.apply(&mut cam);
        let fov = cam.fov_horizontal_deg();
        assert!(fov > 70.0 && fov < 90.0, "24mm should give ~73° FOV, got {}", fov);
    }

    #[test]
    fn test_telephoto_fov() {
        let mut cam = CameraRig::default();
        CameraPreset::Telephoto.apply(&mut cam);
        let fov = cam.fov_horizontal_deg();
        assert!(fov > 8.0 && fov < 14.0, "200mm should give ~10° FOV, got {}", fov);
    }

    #[test]
    fn test_ray_generation() {
        let cam = CameraRig::default();
        let ray = cam.generate_ray(0.5, 0.5, 16.0 / 9.0);
        let fwd = cam.forward();
        let d = dot(ray.direction, fwd);
        assert!(d > 0.99, "Center ray should be aligned with forward, dot={}", d);
    }

    #[test]
    fn test_dof_shallow() {
        let mut cam = CameraRig::default();
        CameraPreset::Portrait.apply(&mut cam);
        cam.focus_distance = 3.0;
        let near = cam.dof_near();
        let far = cam.dof_far();
        assert!(near > 0.0 && near < cam.focus_distance, "near={}", near);
        assert!(far > cam.focus_distance, "far={}", far);
        let dof_range = far - near;
        assert!(dof_range < 2.0, "f/1.8 at 3m should have <2m DoF range, got {}", dof_range);
    }

    #[test]
    fn test_dof_deep() {
        let mut cam = CameraRig::default();
        CameraPreset::Security.apply(&mut cam);
        cam.focus_distance = 50.0;
        assert!(cam.f_stop >= 11.0);
    }

    #[test]
    fn test_presets_all() {
        for name in &["wide", "normal", "portrait", "telephoto", "macro", "cinematic", "drone", "security"] {
            assert!(CameraPreset::from_name(name).is_some(), "Preset '{}' should parse", name);
        }
    }

    #[test]
    fn test_ray_at() {
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        let p = ray.at(5.0);
        assert!((p[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_math_ops() {
        assert!((dot([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])).abs() < 1e-6);
        assert!((dot([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]) - 1.0).abs() < 1e-6);
        let c = cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((c[2] - 1.0).abs() < 1e-6);
        let n = normalize([3.0, 4.0, 0.0]);
        assert!((length(n) - 1.0).abs() < 1e-6);
    }
}
