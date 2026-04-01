use super::camera::{Ray, dot, sub, normalize, scale, reflect};
use super::lighting::{LightingRig, LuxorLight};
use super::geometry::{HitRecord, BVHNode, intersect_terrain};
use super::scene_capture::{CapturedPrim, SceneGeometry};

#[derive(Debug, Clone, Copy)]
pub struct ShadingParams {
    pub ambient_weight: f32,
    pub diffuse_weight: f32,
    pub specular_weight: f32,
    pub specular_power: f32,
}

impl ShadingParams {
    pub fn from_shininess(shininess: u8) -> Self {
        match shininess {
            0 => ShadingParams {
                ambient_weight: 1.0,
                diffuse_weight: 1.0,
                specular_weight: 0.0,
                specular_power: 1.0,
            },
            1 => ShadingParams {
                ambient_weight: 1.0,
                diffuse_weight: 1.0,
                specular_weight: 0.3,
                specular_power: 16.0,
            },
            2 => ShadingParams {
                ambient_weight: 1.0,
                diffuse_weight: 1.0,
                specular_weight: 0.6,
                specular_power: 32.0,
            },
            _ => ShadingParams {
                ambient_weight: 1.0,
                diffuse_weight: 1.0,
                specular_weight: 0.9,
                specular_power: 64.0,
            },
        }
    }
}

pub fn shade_hit(
    hit: &HitRecord,
    ray: &Ray,
    prim: &CapturedPrim,
    scene: &SceneGeometry,
    bvh: Option<&BVHNode>,
    lighting: &LightingRig,
) -> [f32; 3] {
    if prim.fullbright {
        return [prim.color[0], prim.color[1], prim.color[2]];
    }

    let params = ShadingParams::from_shininess(prim.shininess);
    let base_color = [prim.color[0], prim.color[1], prim.color[2]];

    let ambient = [
        base_color[0] * lighting.ambient_color[0] * lighting.ambient_intensity * params.ambient_weight,
        base_color[1] * lighting.ambient_color[1] * lighting.ambient_intensity * params.ambient_weight,
        base_color[2] * lighting.ambient_color[2] * lighting.ambient_intensity * params.ambient_weight,
    ];

    let mut total = ambient;

    for light in &lighting.lights {
        let (intensity, light_color, dir_to_light) = light.evaluate_at(hit.position);

        if intensity < 0.001 {
            continue;
        }

        let in_shadow = is_in_shadow(hit, light, scene, bvh);
        if in_shadow {
            continue;
        }

        let n_dot_l = dot(hit.normal, dir_to_light).max(0.0);
        let diffuse = [
            base_color[0] * light_color[0] * intensity * n_dot_l * params.diffuse_weight * 0.001,
            base_color[1] * light_color[1] * intensity * n_dot_l * params.diffuse_weight * 0.001,
            base_color[2] * light_color[2] * intensity * n_dot_l * params.diffuse_weight * 0.001,
        ];

        total[0] += diffuse[0];
        total[1] += diffuse[1];
        total[2] += diffuse[2];

        if params.specular_weight > 0.0 {
            let view_dir = normalize(sub(ray.origin, hit.position));
            let half_vec = normalize([
                dir_to_light[0] + view_dir[0],
                dir_to_light[1] + view_dir[1],
                dir_to_light[2] + view_dir[2],
            ]);
            let n_dot_h = dot(hit.normal, half_vec).max(0.0);
            let spec = n_dot_h.powf(params.specular_power) * params.specular_weight * intensity * 0.001;

            total[0] += light_color[0] * spec;
            total[1] += light_color[1] * spec;
            total[2] += light_color[2] * spec;
        }
    }

    [
        total[0].clamp(0.0, 1.0),
        total[1].clamp(0.0, 1.0),
        total[2].clamp(0.0, 1.0),
    ]
}

pub fn shade_terrain(
    hit: &HitRecord,
    ray: &Ray,
    scene: &SceneGeometry,
    bvh: Option<&BVHNode>,
    lighting: &LightingRig,
) -> [f32; 3] {
    let height = hit.position[2];
    let base_color = terrain_color(height);

    let params = ShadingParams {
        ambient_weight: 1.0,
        diffuse_weight: 1.0,
        specular_weight: 0.05,
        specular_power: 8.0,
    };

    let ambient = [
        base_color[0] * lighting.ambient_color[0] * lighting.ambient_intensity * params.ambient_weight,
        base_color[1] * lighting.ambient_color[1] * lighting.ambient_intensity * params.ambient_weight,
        base_color[2] * lighting.ambient_color[2] * lighting.ambient_intensity * params.ambient_weight,
    ];

    let mut total = ambient;

    for light in &lighting.lights {
        let (intensity, light_color, dir_to_light) = light.evaluate_at(hit.position);
        if intensity < 0.001 { continue; }

        let in_shadow = is_in_shadow(hit, light, scene, bvh);
        if in_shadow { continue; }

        let n_dot_l = dot(hit.normal, dir_to_light).max(0.0);
        let diffuse_scale = intensity * n_dot_l * params.diffuse_weight * 0.001;

        total[0] += base_color[0] * light_color[0] * diffuse_scale;
        total[1] += base_color[1] * light_color[1] * diffuse_scale;
        total[2] += base_color[2] * light_color[2] * diffuse_scale;
    }

    [
        total[0].clamp(0.0, 1.0),
        total[1].clamp(0.0, 1.0),
        total[2].clamp(0.0, 1.0),
    ]
}

fn terrain_color(height: f32) -> [f32; 3] {
    if height < 0.0 {
        [0.15, 0.25, 0.45]
    } else if height < 5.0 {
        [0.76, 0.7, 0.5]
    } else if height < 20.0 {
        let t = (height - 5.0) / 15.0;
        [
            0.76 * (1.0 - t) + 0.3 * t,
            0.7 * (1.0 - t) + 0.55 * t,
            0.5 * (1.0 - t) + 0.2 * t,
        ]
    } else if height < 50.0 {
        [0.3, 0.55, 0.2]
    } else if height < 100.0 {
        let t = (height - 50.0) / 50.0;
        [
            0.3 * (1.0 - t) + 0.5 * t,
            0.55 * (1.0 - t) + 0.45 * t,
            0.2 * (1.0 - t) + 0.35 * t,
        ]
    } else {
        [0.6, 0.6, 0.6]
    }
}

fn is_in_shadow(
    hit: &HitRecord,
    light: &LuxorLight,
    scene: &SceneGeometry,
    bvh: Option<&BVHNode>,
) -> bool {
    let (shadow_origin, max_dist) = light.shadow_origin(hit.position);
    let dir_to_light = match light.light_type {
        super::lighting::LightType::Directional => scale(light.direction, -1.0),
        _ => {
            let to_l = sub(light.position, hit.position);
            normalize(to_l)
        }
    };

    let shadow_ray = Ray {
        origin: shadow_origin,
        direction: dir_to_light,
    };

    if let Some(bvh) = bvh {
        if let Some(shadow_hit) = bvh.intersect(&shadow_ray, &scene.prims, max_dist) {
            if !shadow_hit.is_terrain {
                let prim = &scene.prims[shadow_hit.prim_index];
                if prim.alpha < 0.5 {
                    return false;
                }
                return true;
            }
        }
    }

    if let Some(ref terrain) = scene.terrain {
        if let Some(_) = intersect_terrain(&shadow_ray, terrain, max_dist) {
            return true;
        }
    }

    false
}

pub fn sky_color(ray: &Ray, lighting: &LightingRig) -> [f32; 3] {
    let up_factor = ray.direction[2].clamp(0.0, 1.0);
    let t = up_factor;

    let sun_dir = normalize(scale(lighting.sun_direction, -1.0));
    let sun_dot = dot(ray.direction, sun_dir).max(0.0);

    let sun_glow = if sun_dot > 0.995 {
        let s = ((sun_dot - 0.995) / 0.005).min(1.0);
        [
            lighting.sun_color[0] * s * 3.0,
            lighting.sun_color[1] * s * 3.0,
            lighting.sun_color[2] * s * 3.0,
        ]
    } else if sun_dot > 0.95 {
        let s = ((sun_dot - 0.95) / 0.045).min(1.0);
        [
            lighting.sun_color[0] * s * 0.3,
            lighting.sun_color[1] * s * 0.3,
            lighting.sun_color[2] * s * 0.3,
        ]
    } else {
        [0.0; 3]
    };

    let sky = [
        lighting.sky_color_horizon[0] * (1.0 - t) + lighting.sky_color_top[0] * t + sun_glow[0],
        lighting.sky_color_horizon[1] * (1.0 - t) + lighting.sky_color_top[1] * t + sun_glow[1],
        lighting.sky_color_horizon[2] * (1.0 - t) + lighting.sky_color_top[2] * t + sun_glow[2],
    ];

    [sky[0].min(1.0), sky[1].min(1.0), sky[2].min(1.0)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::scene_capture::PrimShape;

    #[test]
    fn test_terrain_color_bands() {
        let underwater = terrain_color(-5.0);
        assert!(underwater[2] > underwater[0], "Underwater should be blueish");

        let beach = terrain_color(2.0);
        assert!(beach[0] > beach[2], "Beach should be sandy/warm");

        let grass = terrain_color(30.0);
        assert!(grass[1] > grass[0] && grass[1] > grass[2], "Mid terrain should be green");

        let rock = terrain_color(120.0);
        assert!((rock[0] - rock[1]).abs() < 0.1, "High terrain should be gray");
    }

    #[test]
    fn test_sky_gradient() {
        let rig = LightingRig::default();
        let up_ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
        };
        let horizon_ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };

        let sky_up = sky_color(&up_ray, &rig);
        let sky_horiz = sky_color(&horizon_ray, &rig);
        assert!(sky_up[2] > sky_horiz[2] || (sky_up[2] - sky_horiz[2]).abs() < 0.3,
            "Sky should be bluer up top");
    }

    #[test]
    fn test_fullbright_bypass() {
        let hit = HitRecord {
            t: 1.0,
            position: [5.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            prim_index: 0,
            is_terrain: false,
        };
        let ray = Ray {
            origin: [0.0, 0.0, 10.0],
            direction: [0.0, 0.0, -1.0],
        };
        let prim = CapturedPrim {
            local_id: 1,
            uuid: uuid::Uuid::nil(),
            position: [5.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            shape: PrimShape::Box,
            color: [1.0, 0.5, 0.0, 1.0],
            fullbright: true,
            glow: 0.0,
            alpha: 1.0,
            shininess: 0,
            profile_curve: 1,
            path_curve: 16,
            profile_hollow: 0,
            parent_id: 0,
        };
        let scene = SceneGeometry::empty(uuid::Uuid::nil());
        let rig = LightingRig::default();

        let color = shade_hit(&hit, &ray, &prim, &scene, None, &rig);
        assert!((color[0] - 1.0).abs() < 0.01, "Fullbright should return exact color");
        assert!((color[1] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_shading_params_shininess() {
        let p0 = ShadingParams::from_shininess(0);
        assert!(p0.specular_weight < 0.01);

        let p3 = ShadingParams::from_shininess(3);
        assert!(p3.specular_weight > 0.5);
        assert!(p3.specular_power > 30.0);
    }
}
