use serde::{Serialize, Deserialize};
use super::camera::{dot, sub, normalize, scale, add};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LightType {
    Point,
    Spot,
    Directional,
    Area,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuxorLight {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub light_type: LightType,
    pub radius: f32,
    pub spot_angle: f32,
    pub soft_edge: f32,
}

impl LuxorLight {
    pub fn point(position: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        Self {
            position,
            direction: [0.0, 0.0, -1.0],
            color,
            intensity,
            light_type: LightType::Point,
            radius: 20.0,
            spot_angle: 180.0,
            soft_edge: 0.0,
        }
    }

    pub fn spot(position: [f32; 3], direction: [f32; 3], color: [f32; 3], intensity: f32, angle: f32) -> Self {
        Self {
            position,
            direction: normalize(direction),
            color,
            intensity,
            light_type: LightType::Spot,
            radius: 30.0,
            spot_angle: angle,
            soft_edge: 5.0,
        }
    }

    pub fn directional(direction: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            direction: normalize(direction),
            color,
            intensity,
            light_type: LightType::Directional,
            radius: f32::INFINITY,
            spot_angle: 180.0,
            soft_edge: 0.0,
        }
    }

    pub fn evaluate_at(&self, point: [f32; 3]) -> (f32, [f32; 3], [f32; 3]) {
        match self.light_type {
            LightType::Directional => {
                let dir_to_light = scale(self.direction, -1.0);
                (self.intensity, self.color, dir_to_light)
            }
            LightType::Point => {
                let to_light = sub(self.position, point);
                let dist = super::camera::length(to_light);
                if dist > self.radius || dist < 1e-6 {
                    return (0.0, [0.0; 3], [0.0; 3]);
                }
                let dir_to_light = scale(to_light, 1.0 / dist);
                let falloff = 1.0 / (1.0 + dist * dist * 0.01);
                let range_atten = (1.0 - (dist / self.radius).powi(2)).max(0.0);
                let final_intensity = self.intensity * falloff * range_atten;
                (final_intensity, self.color, dir_to_light)
            }
            LightType::Spot => {
                let to_light = sub(self.position, point);
                let dist = super::camera::length(to_light);
                if dist > self.radius || dist < 1e-6 {
                    return (0.0, [0.0; 3], [0.0; 3]);
                }
                let dir_to_light = scale(to_light, 1.0 / dist);
                let cos_angle = dot(scale(dir_to_light, -1.0), self.direction);
                let half_angle_rad = (self.spot_angle / 2.0).to_radians();
                let cos_half = half_angle_rad.cos();
                let soft_rad = (self.soft_edge).to_radians();
                let cos_outer = (half_angle_rad + soft_rad).cos();

                if cos_angle < cos_outer {
                    return (0.0, [0.0; 3], [0.0; 3]);
                }

                let spot_factor = if cos_angle >= cos_half {
                    1.0
                } else {
                    ((cos_angle - cos_outer) / (cos_half - cos_outer)).max(0.0)
                };

                let falloff = 1.0 / (1.0 + dist * dist * 0.01);
                let range_atten = (1.0 - (dist / self.radius).powi(2)).max(0.0);
                let final_intensity = self.intensity * falloff * range_atten * spot_factor;
                (final_intensity, self.color, dir_to_light)
            }
            LightType::Area => {
                let to_light = sub(self.position, point);
                let dist = super::camera::length(to_light);
                if dist > self.radius || dist < 1e-6 {
                    return (0.0, [0.0; 3], [0.0; 3]);
                }
                let dir_to_light = scale(to_light, 1.0 / dist);
                let falloff = 1.0 / (1.0 + dist * 0.05);
                let range_atten = (1.0 - (dist / self.radius).powi(2)).max(0.0);
                let final_intensity = self.intensity * falloff * range_atten;
                (final_intensity, self.color, dir_to_light)
            }
        }
    }

    pub fn shadow_origin(&self, hit_point: [f32; 3]) -> ([f32; 3], f32) {
        match self.light_type {
            LightType::Directional => {
                let dir = scale(self.direction, -1.0);
                let origin = add(hit_point, scale(dir, 0.01));
                (origin, 1000.0)
            }
            _ => {
                let to_light = sub(self.position, hit_point);
                let dist = super::camera::length(to_light);
                let origin = add(hit_point, scale(normalize(to_light), 0.01));
                (origin, dist - 0.02)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingRig {
    pub lights: Vec<LuxorLight>,
    pub ambient_color: [f32; 3],
    pub ambient_intensity: f32,
    pub sky_color_top: [f32; 3],
    pub sky_color_horizon: [f32; 3],
    pub sun_direction: [f32; 3],
    pub sun_color: [f32; 3],
    pub sun_intensity: f32,
}

impl Default for LightingRig {
    fn default() -> Self {
        Self {
            lights: Vec::new(),
            ambient_color: [0.15, 0.18, 0.22],
            ambient_intensity: 0.3,
            sky_color_top: [0.4, 0.6, 1.0],
            sky_color_horizon: [0.7, 0.8, 0.95],
            sun_direction: normalize([-0.5, -0.3, -0.8]),
            sun_color: [1.0, 0.95, 0.85],
            sun_intensity: 0.8,
        }
    }
}

impl LightingRig {
    pub fn with_sun(&self) -> Self {
        let mut rig = self.clone();
        rig.lights.push(LuxorLight::directional(
            rig.sun_direction,
            rig.sun_color,
            rig.sun_intensity,
        ));
        rig
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LightingPreset {
    Studio3Point,
    Rembrandt,
    Butterfly,
    Noir,
    GoldenHour,
    Moonlight,
    Split,
    Flat,
    Backlit,
    Neon,
}

impl LightingPreset {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "studio" | "studio_3point" | "3point" | "three_point" => Some(LightingPreset::Studio3Point),
            "rembrandt" => Some(LightingPreset::Rembrandt),
            "butterfly" | "paramount" => Some(LightingPreset::Butterfly),
            "noir" | "dramatic" | "hard" => Some(LightingPreset::Noir),
            "golden_hour" | "golden" | "sunset" | "warm" => Some(LightingPreset::GoldenHour),
            "moonlight" | "moon" | "night" | "cool" => Some(LightingPreset::Moonlight),
            "split" => Some(LightingPreset::Split),
            "flat" | "even" | "diffuse" => Some(LightingPreset::Flat),
            "backlit" | "silhouette" | "rim" => Some(LightingPreset::Backlit),
            "neon" | "cyberpunk" | "vapor" => Some(LightingPreset::Neon),
            _ => None,
        }
    }

    pub fn build_rig(&self, subject: [f32; 3], distance: f32) -> LightingRig {
        let d = distance;
        let s = subject;

        match self {
            LightingPreset::Studio3Point => LightingRig {
                lights: vec![
                    LuxorLight::point(
                        [s[0] - d * 0.7, s[1] + d * 0.7, s[2] + d * 0.5],
                        [1.0, 0.98, 0.95], 800.0,
                    ),
                    LuxorLight::point(
                        [s[0] + d * 0.6, s[1] + d * 0.4, s[2] + d * 0.3],
                        [0.85, 0.9, 1.0], 400.0,
                    ),
                    LuxorLight::point(
                        [s[0], s[1] - d * 0.8, s[2] + d * 0.6],
                        [1.0, 1.0, 1.0], 300.0,
                    ),
                ],
                ambient_color: [0.12, 0.14, 0.18],
                ambient_intensity: 0.25,
                sky_color_top: [0.3, 0.4, 0.6],
                sky_color_horizon: [0.5, 0.55, 0.65],
                sun_direction: normalize([-0.5, -0.3, -0.8]),
                sun_color: [1.0, 0.95, 0.9],
                sun_intensity: 0.3,
            },

            LightingPreset::Rembrandt => LightingRig {
                lights: vec![
                    LuxorLight::spot(
                        [s[0] - d * 0.7, s[1] + d * 0.7, s[2] + d * 0.5],
                        normalize(sub(s, [s[0] - d * 0.7, s[1] + d * 0.7, s[2] + d * 0.5])),
                        [1.0, 0.92, 0.8], 900.0, 45.0,
                    ),
                    LuxorLight::point(
                        [s[0] + d * 0.5, s[1] + d * 0.5, s[2] + d * 0.2],
                        [0.7, 0.75, 0.9], 250.0,
                    ),
                    LuxorLight::point(
                        [s[0], s[1] - d * 0.8, s[2] + d * 0.6],
                        [1.0, 1.0, 1.0], 350.0,
                    ),
                ],
                ambient_color: [0.08, 0.08, 0.1],
                ambient_intensity: 0.15,
                sky_color_top: [0.2, 0.25, 0.35],
                sky_color_horizon: [0.4, 0.4, 0.45],
                sun_direction: normalize([-0.6, 0.4, -0.7]),
                sun_color: [1.0, 0.9, 0.75],
                sun_intensity: 0.2,
            },

            LightingPreset::Butterfly => LightingRig {
                lights: vec![
                    LuxorLight::point(
                        [s[0], s[1] + d * 0.3, s[2] + d],
                        [1.0, 1.0, 0.95], 1000.0,
                    ),
                    LuxorLight::point(
                        [s[0], s[1] + d * 0.3, s[2] - d * 0.3],
                        [0.9, 0.9, 1.0], 200.0,
                    ),
                ],
                ambient_color: [0.15, 0.15, 0.18],
                ambient_intensity: 0.2,
                sky_color_top: [0.35, 0.4, 0.6],
                sky_color_horizon: [0.6, 0.65, 0.75],
                sun_direction: normalize([0.0, 0.3, -1.0]),
                sun_color: [1.0, 1.0, 0.95],
                sun_intensity: 0.3,
            },

            LightingPreset::Noir => LightingRig {
                lights: vec![
                    LuxorLight::spot(
                        [s[0] - d, s[1], s[2] + d * 0.3],
                        normalize(sub(s, [s[0] - d, s[1], s[2] + d * 0.3])),
                        [1.0, 1.0, 1.0], 1200.0, 30.0,
                    ),
                ],
                ambient_color: [0.03, 0.03, 0.05],
                ambient_intensity: 0.08,
                sky_color_top: [0.05, 0.05, 0.1],
                sky_color_horizon: [0.1, 0.1, 0.15],
                sun_direction: normalize([-1.0, 0.0, -0.3]),
                sun_color: [1.0, 1.0, 1.0],
                sun_intensity: 0.1,
            },

            LightingPreset::GoldenHour => LightingRig {
                lights: vec![
                    LuxorLight::directional(
                        normalize([-1.0, 0.0, -0.15]),
                        [1.0, 0.7, 0.3], 900.0,
                    ),
                    LuxorLight::point(
                        [s[0], s[1], s[2] + d * 1.5],
                        [0.5, 0.6, 0.9], 200.0,
                    ),
                    LuxorLight::point(
                        [s[0] + d * 0.8, s[1] - d * 0.5, s[2] + d * 0.3],
                        [1.0, 0.75, 0.4], 400.0,
                    ),
                ],
                ambient_color: [0.2, 0.15, 0.08],
                ambient_intensity: 0.3,
                sky_color_top: [0.3, 0.4, 0.7],
                sky_color_horizon: [1.0, 0.7, 0.3],
                sun_direction: normalize([-1.0, 0.0, -0.15]),
                sun_color: [1.0, 0.7, 0.3],
                sun_intensity: 0.9,
            },

            LightingPreset::Moonlight => LightingRig {
                lights: vec![
                    LuxorLight::directional(
                        normalize([0.3, -0.5, -0.7]),
                        [0.6, 0.65, 0.9], 500.0,
                    ),
                    LuxorLight::point(
                        [s[0] - d, s[1] + d, s[2] + d * 0.5],
                        [0.4, 0.45, 0.7], 150.0,
                    ),
                ],
                ambient_color: [0.04, 0.05, 0.1],
                ambient_intensity: 0.12,
                sky_color_top: [0.02, 0.03, 0.08],
                sky_color_horizon: [0.05, 0.06, 0.12],
                sun_direction: normalize([0.3, -0.5, -0.7]),
                sun_color: [0.6, 0.65, 0.9],
                sun_intensity: 0.5,
            },

            LightingPreset::Split => LightingRig {
                lights: vec![
                    LuxorLight::point(
                        [s[0] - d, s[1], s[2] + d * 0.3],
                        [1.0, 0.95, 0.9], 800.0,
                    ),
                    LuxorLight::point(
                        [s[0] + d, s[1], s[2] + d * 0.3],
                        [0.6, 0.65, 0.8], 200.0,
                    ),
                ],
                ambient_color: [0.06, 0.06, 0.08],
                ambient_intensity: 0.1,
                sky_color_top: [0.15, 0.18, 0.3],
                sky_color_horizon: [0.3, 0.32, 0.4],
                sun_direction: normalize([-1.0, 0.0, -0.3]),
                sun_color: [1.0, 0.95, 0.9],
                sun_intensity: 0.15,
            },

            LightingPreset::Flat => LightingRig {
                lights: vec![
                    LuxorLight::point(
                        [s[0], s[1] + d, s[2] + d * 0.5],
                        [1.0, 1.0, 1.0], 500.0,
                    ),
                    LuxorLight::point(
                        [s[0] - d * 0.7, s[1] - d * 0.3, s[2] + d * 0.3],
                        [0.95, 0.95, 1.0], 400.0,
                    ),
                    LuxorLight::point(
                        [s[0] + d * 0.7, s[1] - d * 0.3, s[2] + d * 0.3],
                        [0.95, 0.95, 1.0], 400.0,
                    ),
                ],
                ambient_color: [0.2, 0.2, 0.22],
                ambient_intensity: 0.4,
                sky_color_top: [0.5, 0.6, 0.8],
                sky_color_horizon: [0.7, 0.75, 0.85],
                sun_direction: normalize([0.0, 0.0, -1.0]),
                sun_color: [1.0, 1.0, 1.0],
                sun_intensity: 0.4,
            },

            LightingPreset::Backlit => LightingRig {
                lights: vec![
                    LuxorLight::point(
                        [s[0], s[1] - d * 1.2, s[2] + d * 0.5],
                        [1.0, 0.95, 0.85], 1000.0,
                    ),
                    LuxorLight::point(
                        [s[0] - d * 0.5, s[1] - d * 0.8, s[2] + d * 0.4],
                        [1.0, 1.0, 1.0], 600.0,
                    ),
                    LuxorLight::point(
                        [s[0] + d * 0.5, s[1] - d * 0.8, s[2] + d * 0.4],
                        [1.0, 1.0, 1.0], 600.0,
                    ),
                    LuxorLight::point(
                        [s[0], s[1] + d * 0.5, s[2] + d * 0.2],
                        [0.5, 0.55, 0.7], 100.0,
                    ),
                ],
                ambient_color: [0.05, 0.06, 0.08],
                ambient_intensity: 0.1,
                sky_color_top: [0.3, 0.4, 0.6],
                sky_color_horizon: [0.8, 0.75, 0.6],
                sun_direction: normalize([0.0, -1.0, -0.3]),
                sun_color: [1.0, 0.95, 0.85],
                sun_intensity: 0.8,
            },

            LightingPreset::Neon => LightingRig {
                lights: vec![
                    LuxorLight::point(
                        [s[0] - d * 0.8, s[1], s[2] + d * 0.3],
                        [1.0, 0.1, 0.6], 700.0,
                    ),
                    LuxorLight::point(
                        [s[0] + d * 0.8, s[1], s[2] + d * 0.3],
                        [0.1, 0.5, 1.0], 700.0,
                    ),
                    LuxorLight::point(
                        [s[0], s[1] + d * 0.5, s[2] + d * 0.8],
                        [0.0, 1.0, 0.6], 300.0,
                    ),
                ],
                ambient_color: [0.02, 0.02, 0.05],
                ambient_intensity: 0.08,
                sky_color_top: [0.03, 0.02, 0.08],
                sky_color_horizon: [0.08, 0.05, 0.12],
                sun_direction: normalize([0.0, 0.0, -1.0]),
                sun_color: [0.3, 0.3, 0.4],
                sun_intensity: 0.05,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_presets_build() {
        let subject = [128.0, 128.0, 25.0];
        let presets = [
            LightingPreset::Studio3Point,
            LightingPreset::Rembrandt,
            LightingPreset::Butterfly,
            LightingPreset::Noir,
            LightingPreset::GoldenHour,
            LightingPreset::Moonlight,
            LightingPreset::Split,
            LightingPreset::Flat,
            LightingPreset::Backlit,
            LightingPreset::Neon,
        ];
        for preset in &presets {
            let rig = preset.build_rig(subject, 8.0);
            assert!(!rig.lights.is_empty(), "Preset {:?} should have lights", preset);
            for light in &rig.lights {
                assert!(light.intensity > 0.0, "Light intensity should be positive");
            }
        }
    }

    #[test]
    fn test_preset_from_name() {
        assert_eq!(LightingPreset::from_name("studio"), Some(LightingPreset::Studio3Point));
        assert_eq!(LightingPreset::from_name("golden_hour"), Some(LightingPreset::GoldenHour));
        assert_eq!(LightingPreset::from_name("cyberpunk"), Some(LightingPreset::Neon));
        assert_eq!(LightingPreset::from_name("invalid"), None);
    }

    #[test]
    fn test_point_light_falloff() {
        let light = LuxorLight::point([0.0, 0.0, 10.0], [1.0, 1.0, 1.0], 500.0);
        let (i_near, _, _) = light.evaluate_at([0.0, 0.0, 9.0]);
        let (i_far, _, _) = light.evaluate_at([0.0, 0.0, 0.0]);
        assert!(i_near > i_far, "Near point should be brighter: {} vs {}", i_near, i_far);
    }

    #[test]
    fn test_directional_constant() {
        let light = LuxorLight::directional([0.0, 0.0, -1.0], [1.0, 1.0, 1.0], 500.0);
        let (i1, _, d1) = light.evaluate_at([0.0, 0.0, 0.0]);
        let (i2, _, d2) = light.evaluate_at([100.0, 100.0, 100.0]);
        assert!((i1 - i2).abs() < 1e-3, "Directional light should be constant intensity");
        assert!((d1[2] - d2[2]).abs() < 1e-6, "Direction should be constant");
    }

    #[test]
    fn test_spot_cone() {
        let light = LuxorLight::spot(
            [0.0, 0.0, 10.0],
            [0.0, 0.0, -1.0],
            [1.0, 1.0, 1.0],
            500.0,
            30.0,
        );
        let (i_center, _, _) = light.evaluate_at([0.0, 0.0, 0.0]);
        let (i_outside, _, _) = light.evaluate_at([20.0, 0.0, 0.0]);
        assert!(i_center > 0.0, "Center should be lit");
        assert!(i_outside < i_center * 0.01, "Outside cone should be dark: {}", i_outside);
    }

    #[test]
    fn test_noir_is_single_light() {
        let rig = LightingPreset::Noir.build_rig([128.0, 128.0, 25.0], 8.0);
        assert_eq!(rig.lights.len(), 1, "Noir should have single hard light");
        assert!(rig.ambient_intensity < 0.1, "Noir should have very low ambient");
    }

    #[test]
    fn test_golden_hour_warm_tones() {
        let rig = LightingPreset::GoldenHour.build_rig([128.0, 128.0, 25.0], 8.0);
        assert!(rig.sun_color[0] > rig.sun_color[2], "Golden hour sun should be warm (R > B)");
        assert!(rig.ambient_color[0] > rig.ambient_color[2], "Golden hour ambient should be warm");
    }
}
