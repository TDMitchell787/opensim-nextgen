use bytemuck::{Pod, Zeroable};

use super::super::camera::CameraRig;
use super::super::lighting::{LightingRig, LightType};
use super::super::raytracer::RenderQuality;
use super::super::scene_capture::{SceneGeometry, CapturedPrim, PrimShape, TerrainData};
use super::super::geometry::{BVHNode, AABB};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuPrim {
    pub position: [f32; 3],
    pub shape: u32,
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub shininess: u32,
    pub color: [f32; 4],
    pub fullbright: u32,
    pub glow: f32,
    pub alpha: f32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuBvhNode {
    pub bounds_min: [f32; 3],
    pub left_or_first: i32,
    pub bounds_max: [f32; 3],
    pub count_or_right: i32,
    pub miss_link: i32,
    pub is_leaf: u32,
    pub _pad: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuLight {
    pub position: [f32; 3],
    pub light_type: u32,
    pub direction: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    pub radius: f32,
    pub spot_angle: f32,
    pub soft_edge: f32,
    pub _pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuUniforms {
    pub cam_position: [f32; 3],
    pub cam_fov_h: f32,
    pub cam_forward: [f32; 3],
    pub cam_fov_v: f32,
    pub cam_right: [f32; 3],
    pub cam_aperture_r: f32,
    pub cam_up: [f32; 3],
    pub cam_focus_dist: f32,

    pub width: u32,
    pub height: u32,
    pub spp: u32,
    pub num_prims: u32,

    pub num_lights: u32,
    pub num_bvh_nodes: u32,
    pub has_terrain: u32,
    pub terrain_side: u32,

    pub terrain_origin: [f32; 3],
    pub max_t: f32,

    pub ambient_color: [f32; 3],
    pub ambient_intensity: f32,

    pub use_dof: u32,
    pub _pad: [u32; 3],
}

pub struct PackedScene {
    pub uniforms_bytes: Vec<u8>,
    pub prims_bytes: Vec<u8>,
    pub bvh_bytes: Vec<u8>,
    pub indices_bytes: Vec<u8>,
    pub lights_bytes: Vec<u8>,
    pub terrain_bytes: Vec<u8>,
}

pub fn pack_scene(
    scene: &SceneGeometry,
    camera: &CameraRig,
    lighting: &LightingRig,
    quality: RenderQuality,
    width: u32,
    height: u32,
) -> PackedScene {
    let gpu_prims = pack_prims(&scene.prims);
    let (gpu_bvh, gpu_indices) = if !scene.prims.is_empty() {
        let mut indices: Vec<usize> = (0..scene.prims.len()).collect();
        let bvh = BVHNode::build(&scene.prims, &mut indices);
        flatten_bvh(&bvh)
    } else {
        (vec![GpuBvhNode::zeroed()], vec![0u32])
    };
    let gpu_lights = pack_lights(lighting);
    let spp = quality.samples_per_pixel();
    let use_dof = camera.has_dof_effect() && spp >= 4;

    let lighting_with_sun = lighting.with_sun();
    let uniforms = pack_uniforms(camera, &lighting_with_sun, scene, width, height, spp, use_dof);

    let terrain_bytes = if let Some(ref terrain) = scene.terrain {
        bytemuck::cast_slice(&terrain.heightmap).to_vec()
    } else {
        vec![0u8; 4]
    };

    PackedScene {
        uniforms_bytes: bytemuck::bytes_of(&uniforms).to_vec(),
        prims_bytes: bytemuck::cast_slice(&gpu_prims).to_vec(),
        bvh_bytes: bytemuck::cast_slice(&gpu_bvh).to_vec(),
        indices_bytes: bytemuck::cast_slice(&gpu_indices).to_vec(),
        lights_bytes: bytemuck::cast_slice(&gpu_lights).to_vec(),
        terrain_bytes,
    }
}

fn shape_to_u32(shape: PrimShape) -> u32 {
    match shape {
        PrimShape::Box => 0,
        PrimShape::Sphere => 1,
        PrimShape::Cylinder => 2,
        PrimShape::Torus => 3,
        PrimShape::Prism => 4,
        PrimShape::Ring => 5,
        PrimShape::Tube => 6,
        PrimShape::Terrain => 7,
    }
}

fn light_type_to_u32(lt: LightType) -> u32 {
    match lt {
        LightType::Point => 0,
        LightType::Spot => 1,
        LightType::Directional => 2,
        LightType::Area => 3,
    }
}

pub fn pack_prims(prims: &[CapturedPrim]) -> Vec<GpuPrim> {
    if prims.is_empty() {
        return vec![GpuPrim::zeroed()];
    }
    prims.iter().map(|p| GpuPrim {
        position: p.position,
        shape: shape_to_u32(p.shape),
        rotation: p.rotation,
        scale: p.scale,
        shininess: p.shininess as u32,
        color: p.color,
        fullbright: if p.fullbright { 1 } else { 0 },
        glow: p.glow,
        alpha: p.alpha,
        _pad: 0,
    }).collect()
}

pub fn pack_lights(lighting: &LightingRig) -> Vec<GpuLight> {
    let lighting_with_sun = lighting.with_sun();
    if lighting_with_sun.lights.is_empty() {
        return vec![GpuLight::zeroed()];
    }
    lighting_with_sun.lights.iter().map(|l| GpuLight {
        position: l.position,
        light_type: light_type_to_u32(l.light_type),
        direction: l.direction,
        intensity: l.intensity,
        color: l.color,
        radius: if l.radius.is_finite() { l.radius } else { 10000.0 },
        spot_angle: l.spot_angle,
        soft_edge: l.soft_edge,
        _pad: [0.0; 2],
    }).collect()
}

pub fn pack_uniforms(
    camera: &CameraRig,
    lighting: &LightingRig,
    scene: &SceneGeometry,
    width: u32,
    height: u32,
    spp: u32,
    use_dof: bool,
) -> GpuUniforms {
    let fwd = camera.forward();
    let right = super::super::camera::normalize(camera.right());
    let cup = camera.camera_up();

    let (has_terrain, terrain_side, terrain_origin) = if let Some(ref t) = scene.terrain {
        (1u32, t.side as u32, t.region_origin)
    } else {
        (0u32, 0u32, [0.0; 3])
    };

    GpuUniforms {
        cam_position: camera.position,
        cam_fov_h: camera.fov_horizontal(),
        cam_forward: fwd,
        cam_fov_v: camera.fov_vertical(),
        cam_right: right,
        cam_aperture_r: camera.aperture_radius_m(),
        cam_up: cup,
        cam_focus_dist: camera.focus_distance,

        width,
        height,
        spp,
        num_prims: scene.prims.len() as u32,

        num_lights: lighting.lights.len() as u32,
        num_bvh_nodes: 0,
        has_terrain,
        terrain_side,

        terrain_origin,
        max_t: 500.0,

        ambient_color: lighting.ambient_color,
        ambient_intensity: lighting.ambient_intensity,

        use_dof: if use_dof { 1 } else { 0 },
        _pad: [0; 3],
    }
}

pub fn flatten_bvh(root: &BVHNode) -> (Vec<GpuBvhNode>, Vec<u32>) {
    let mut nodes = Vec::new();
    let mut prim_indices = Vec::new();
    flatten_recursive(root, &mut nodes, &mut prim_indices);
    if nodes.is_empty() {
        nodes.push(GpuBvhNode::zeroed());
    }
    if prim_indices.is_empty() {
        prim_indices.push(0);
    }
    (nodes, prim_indices)
}

fn flatten_recursive(
    node: &BVHNode,
    nodes: &mut Vec<GpuBvhNode>,
    prim_indices: &mut Vec<u32>,
) -> usize {
    let my_idx = nodes.len();
    nodes.push(GpuBvhNode::zeroed());

    match node {
        BVHNode::Leaf { prim_indices: leaf_indices, bounds } => {
            let first = prim_indices.len() as i32;
            let count = leaf_indices.len() as i32;
            for &idx in leaf_indices {
                prim_indices.push(idx as u32);
            }
            nodes[my_idx] = GpuBvhNode {
                bounds_min: bounds.min,
                left_or_first: first,
                bounds_max: bounds.max,
                count_or_right: count,
                miss_link: -1,
                is_leaf: 1,
                _pad: [0; 2],
            };
        }
        BVHNode::Internal { left, right, bounds } => {
            let left_idx = flatten_recursive(left, nodes, prim_indices);
            let right_idx = flatten_recursive(right, nodes, prim_indices);
            nodes[my_idx] = GpuBvhNode {
                bounds_min: bounds.min,
                left_or_first: left_idx as i32,
                bounds_max: bounds.max,
                count_or_right: right_idx as i32,
                miss_link: -1,
                is_leaf: 0,
                _pad: [0; 2],
            };
        }
    }

    my_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_sizes() {
        assert_eq!(std::mem::size_of::<GpuPrim>(), 80, "GpuPrim must be 80 bytes");
        assert_eq!(std::mem::size_of::<GpuBvhNode>(), 48, "GpuBvhNode must be 48 bytes");
        assert_eq!(std::mem::size_of::<GpuLight>(), 64, "GpuLight must be 64 bytes");
        assert_eq!(std::mem::size_of::<GpuUniforms>(), 144, "GpuUniforms must be 144 bytes");
    }

    #[test]
    fn test_struct_alignment() {
        assert_eq!(std::mem::size_of::<GpuPrim>() % 16, 0);
        assert_eq!(std::mem::size_of::<GpuBvhNode>() % 16, 0);
        assert_eq!(std::mem::size_of::<GpuLight>() % 16, 0);
        assert_eq!(std::mem::size_of::<GpuUniforms>() % 16, 0);
    }

    #[test]
    fn test_shape_mapping() {
        assert_eq!(shape_to_u32(PrimShape::Box), 0);
        assert_eq!(shape_to_u32(PrimShape::Sphere), 1);
        assert_eq!(shape_to_u32(PrimShape::Cylinder), 2);
        assert_eq!(shape_to_u32(PrimShape::Torus), 3);
        assert_eq!(shape_to_u32(PrimShape::Prism), 4);
    }

    #[test]
    fn test_pack_empty_scene() {
        let scene = SceneGeometry::empty(uuid::Uuid::nil());
        let camera = CameraRig::default();
        let lighting = super::super::super::lighting::LightingPreset::Flat
            .build_rig([128.0, 128.0, 25.0], 8.0);
        let packed = pack_scene(&scene, &camera, &lighting, RenderQuality::Draft, 64, 48);
        assert!(!packed.uniforms_bytes.is_empty());
        assert!(!packed.prims_bytes.is_empty());
        assert!(!packed.bvh_bytes.is_empty());
    }

    #[test]
    fn test_flatten_bvh_simple() {
        let prims = vec![
            CapturedPrim {
                local_id: 1, uuid: uuid::Uuid::nil(),
                position: [5.0, 0.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [2.0, 2.0, 2.0], shape: PrimShape::Box,
                color: [1.0, 0.0, 0.0, 1.0], fullbright: false, glow: 0.0,
                alpha: 1.0, shininess: 0, profile_curve: 1, path_curve: 16,
                profile_hollow: 0, parent_id: 0,
            },
            CapturedPrim {
                local_id: 2, uuid: uuid::Uuid::nil(),
                position: [15.0, 0.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [2.0, 2.0, 2.0], shape: PrimShape::Sphere,
                color: [0.0, 1.0, 0.0, 1.0], fullbright: false, glow: 0.0,
                alpha: 1.0, shininess: 0, profile_curve: 0, path_curve: 32,
                profile_hollow: 0, parent_id: 0,
            },
        ];
        let mut indices: Vec<usize> = (0..prims.len()).collect();
        let bvh = BVHNode::build(&prims, &mut indices);
        let (flat_nodes, flat_indices) = flatten_bvh(&bvh);
        assert!(!flat_nodes.is_empty());
        assert!(!flat_indices.is_empty());
    }
}
