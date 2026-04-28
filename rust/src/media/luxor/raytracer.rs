use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::camera::{dot, normalize, CameraRig, Ray};
use super::geometry::{intersect_terrain, BVHNode, HitRecord};
use super::lighting::LightingRig;
use super::scene_capture::SceneGeometry;
use super::shading::{shade_hit, shade_terrain, sky_color};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RenderQuality {
    Draft,
    Standard,
    High,
    Ultra,
}

impl RenderQuality {
    pub fn samples_per_pixel(&self) -> u32 {
        match self {
            RenderQuality::Draft => 1,
            RenderQuality::Standard => 4,
            RenderQuality::High => 16,
            RenderQuality::Ultra => 64,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "draft" | "fast" | "preview" => Some(RenderQuality::Draft),
            "standard" | "normal" | "default" => Some(RenderQuality::Standard),
            "high" | "hq" | "quality" => Some(RenderQuality::High),
            "ultra" | "max" | "best" => Some(RenderQuality::Ultra),
            _ => None,
        }
    }
}

pub struct LuxorRaytracer {
    pub camera: CameraRig,
    pub lighting: LightingRig,
    pub quality: RenderQuality,
}

impl LuxorRaytracer {
    pub fn new(camera: CameraRig, lighting: LightingRig, quality: RenderQuality) -> Self {
        Self {
            camera,
            lighting,
            quality,
        }
    }

    pub fn render(&self, scene: &SceneGeometry, width: u32, height: u32) -> Vec<u8> {
        let spp = self.quality.samples_per_pixel();
        let aspect = width as f32 / height as f32;
        let use_dof = self.camera.has_dof_effect() && spp >= 4;

        let bvh = if !scene.prims.is_empty() {
            let mut indices: Vec<usize> = (0..scene.prims.len()).collect();
            Some(BVHNode::build(&scene.prims, &mut indices))
        } else {
            None
        };

        let lighting = self.lighting.with_sun();

        let scanlines: Vec<Vec<[f32; 3]>> = (0..height)
            .into_par_iter()
            .map(|y| {
                let mut rng_state: u32 = y.wrapping_mul(2654435761).wrapping_add(1);
                let mut row = Vec::with_capacity(width as usize);

                for x in 0..width {
                    let mut color_accum = [0.0f32; 3];

                    for s in 0..spp {
                        let (jx, jy) = if spp > 1 {
                            rng_state = xorshift32(rng_state);
                            let jx = (rng_state as f32 / u32::MAX as f32) - 0.5;
                            rng_state = xorshift32(rng_state);
                            let jy = (rng_state as f32 / u32::MAX as f32) - 0.5;
                            (jx, jy)
                        } else {
                            (0.0, 0.0)
                        };

                        let u = (x as f32 + 0.5 + jx) / width as f32;
                        let v = 1.0 - (y as f32 + 0.5 + jy) / height as f32;

                        let ray = if use_dof {
                            rng_state = xorshift32(rng_state);
                            let lu = rng_state as f32 / u32::MAX as f32;
                            rng_state = xorshift32(rng_state);
                            let lv = rng_state as f32 / u32::MAX as f32;
                            self.camera.generate_dof_ray(u, v, aspect, lu, lv)
                        } else {
                            self.camera.generate_ray(u, v, aspect)
                        };

                        let sample = self.trace_ray(&ray, scene, bvh.as_ref(), &lighting);
                        color_accum[0] += sample[0];
                        color_accum[1] += sample[1];
                        color_accum[2] += sample[2];
                    }

                    let inv_spp = 1.0 / spp as f32;
                    row.push([
                        color_accum[0] * inv_spp,
                        color_accum[1] * inv_spp,
                        color_accum[2] * inv_spp,
                    ]);
                }
                row
            })
            .collect();

        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for row in &scanlines {
            for color in row {
                pixels.push((color[0].clamp(0.0, 1.0) * 255.0) as u8);
                pixels.push((color[1].clamp(0.0, 1.0) * 255.0) as u8);
                pixels.push((color[2].clamp(0.0, 1.0) * 255.0) as u8);
                pixels.push(255);
            }
        }

        pixels
    }

    fn trace_ray(
        &self,
        ray: &Ray,
        scene: &SceneGeometry,
        bvh: Option<&BVHNode>,
        lighting: &LightingRig,
    ) -> [f32; 3] {
        let max_t = 500.0;

        let prim_hit = if let Some(bvh) = bvh {
            bvh.intersect(ray, &scene.prims, max_t)
        } else {
            None
        };

        let terrain_hit = if let Some(ref terrain) = scene.terrain {
            intersect_terrain(ray, terrain, max_t)
        } else {
            None
        };

        let closest = match (prim_hit, terrain_hit) {
            (Some(ph), Some(th)) => {
                if ph.t <= th.t {
                    Some(ph)
                } else {
                    Some(th)
                }
            }
            (Some(ph), None) => Some(ph),
            (None, Some(th)) => Some(th),
            (None, None) => None,
        };

        match closest {
            Some(hit) if hit.is_terrain => shade_terrain(&hit, ray, scene, bvh, lighting),
            Some(hit) => {
                let prim = &scene.prims[hit.prim_index];

                if prim.alpha < 0.01 {
                    return sky_color(ray, lighting);
                }

                let surface_color = shade_hit(&hit, ray, prim, scene, bvh, lighting);

                if prim.alpha < 0.99 {
                    let bg = sky_color(ray, lighting);
                    let a = prim.alpha;
                    return [
                        surface_color[0] * a + bg[0] * (1.0 - a),
                        surface_color[1] * a + bg[1] * (1.0 - a),
                        surface_color[2] * a + bg[2] * (1.0 - a),
                    ];
                }

                if prim.glow > 0.0 {
                    let glow_boost = prim.glow * 0.5;
                    return [
                        (surface_color[0] + glow_boost).min(1.0),
                        (surface_color[1] + glow_boost).min(1.0),
                        (surface_color[2] + glow_boost).min(1.0),
                    ];
                }

                surface_color
            }
            None => sky_color(ray, lighting),
        }
    }
}

fn xorshift32(mut state: u32) -> u32 {
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    state
}

#[cfg(test)]
mod tests {
    use super::super::lighting::LightingPreset;
    use super::super::scene_capture::{CapturedPrim, PrimShape, SceneGeometry};
    use super::*;

    #[test]
    fn test_render_empty_scene() {
        let cam = CameraRig::default();
        let lighting = LightingPreset::Studio3Point.build_rig([128.0, 128.0, 25.0], 8.0);
        let rt = LuxorRaytracer::new(cam, lighting, RenderQuality::Draft);
        let scene = SceneGeometry::empty(uuid::Uuid::nil());

        let pixels = rt.render(&scene, 64, 48);
        assert_eq!(pixels.len(), 64 * 48 * 4);

        let mut has_nonblack = false;
        for i in (0..pixels.len()).step_by(4) {
            if pixels[i] > 0 || pixels[i + 1] > 0 || pixels[i + 2] > 0 {
                has_nonblack = true;
                break;
            }
        }
        assert!(has_nonblack, "Empty scene should still have sky color");
    }

    #[test]
    fn test_render_with_prim() {
        let cam = CameraRig {
            position: [0.0, 0.0, 0.0],
            look_at: [5.0, 0.0, 0.0],
            ..CameraRig::default()
        };
        let lighting = LightingPreset::Flat.build_rig([5.0, 0.0, 0.0], 3.0);
        let rt = LuxorRaytracer::new(cam, lighting, RenderQuality::Draft);

        let scene = SceneGeometry {
            prims: vec![CapturedPrim {
                local_id: 1,
                uuid: uuid::Uuid::nil(),
                position: [5.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [2.0, 2.0, 2.0],
                shape: PrimShape::Box,
                color: [1.0, 0.0, 0.0, 1.0],
                fullbright: false,
                glow: 0.0,
                alpha: 1.0,
                shininess: 0,
                profile_curve: 1,
                path_curve: 16,
                profile_hollow: 0,
                parent_id: 0,
            }],
            terrain: None,
            region_id: uuid::Uuid::nil(),
        };

        let pixels = rt.render(&scene, 32, 24);
        assert_eq!(pixels.len(), 32 * 24 * 4);

        let center_idx = (12 * 32 + 16) * 4;
        let r = pixels[center_idx];
        assert!(
            r > 20,
            "Center pixel should have red (box is red), got r={}",
            r
        );
    }

    #[test]
    fn test_quality_levels() {
        assert_eq!(RenderQuality::Draft.samples_per_pixel(), 1);
        assert_eq!(RenderQuality::Standard.samples_per_pixel(), 4);
        assert_eq!(RenderQuality::High.samples_per_pixel(), 16);
        assert_eq!(RenderQuality::Ultra.samples_per_pixel(), 64);
    }

    #[test]
    fn test_quality_from_name() {
        assert_eq!(
            RenderQuality::from_name("draft"),
            Some(RenderQuality::Draft)
        );
        assert_eq!(
            RenderQuality::from_name("ultra"),
            Some(RenderQuality::Ultra)
        );
        assert_eq!(RenderQuality::from_name("invalid"), None);
    }

    #[test]
    fn test_xorshift() {
        let mut state = 12345u32;
        let mut values = Vec::new();
        for _ in 0..100 {
            state = xorshift32(state);
            values.push(state);
        }
        let unique: std::collections::HashSet<u32> = values.iter().copied().collect();
        assert!(
            unique.len() > 90,
            "PRNG should produce mostly unique values"
        );
    }
}
