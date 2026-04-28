use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::udp::server::SceneObject;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimShape {
    Box,
    Sphere,
    Cylinder,
    Torus,
    Prism,
    Ring,
    Tube,
    Terrain,
}

#[derive(Debug, Clone)]
pub struct CapturedPrim {
    pub local_id: u32,
    pub uuid: Uuid,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub shape: PrimShape,
    pub color: [f32; 4],
    pub fullbright: bool,
    pub glow: f32,
    pub alpha: f32,
    pub shininess: u8,
    pub profile_curve: u8,
    pub path_curve: u8,
    pub profile_hollow: u16,
    pub parent_id: u32,
}

#[derive(Debug, Clone)]
pub struct TerrainData {
    pub heightmap: Vec<f32>,
    pub side: usize,
    pub region_origin: [f32; 3],
}

impl TerrainData {
    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        let lx = x - self.region_origin[0];
        let ly = y - self.region_origin[1];

        if lx < 0.0 || ly < 0.0 || lx >= self.side as f32 || ly >= self.side as f32 {
            return 0.0;
        }

        let ix = (lx as usize).min(self.side - 2);
        let iy = (ly as usize).min(self.side - 2);
        let fx = lx - ix as f32;
        let fy = ly - iy as f32;

        let h00 = self
            .heightmap
            .get(iy * self.side + ix)
            .copied()
            .unwrap_or(0.0);
        let h10 = self
            .heightmap
            .get(iy * self.side + ix + 1)
            .copied()
            .unwrap_or(0.0);
        let h01 = self
            .heightmap
            .get((iy + 1) * self.side + ix)
            .copied()
            .unwrap_or(0.0);
        let h11 = self
            .heightmap
            .get((iy + 1) * self.side + ix + 1)
            .copied()
            .unwrap_or(0.0);

        let h0 = h00 + (h10 - h00) * fx;
        let h1 = h01 + (h11 - h01) * fx;
        h0 + (h1 - h0) * fy
    }

    pub fn normal_at(&self, x: f32, y: f32) -> [f32; 3] {
        let step = 1.0;
        let hc = self.height_at(x, y);
        let hx = self.height_at(x + step, y);
        let hy = self.height_at(x, y + step);

        let dx = [step, 0.0, hx - hc];
        let dy = [0.0, step, hy - hc];

        let n = super::camera::cross(dx, dy);
        super::camera::normalize(n)
    }
}

#[derive(Debug, Clone)]
pub struct SceneGeometry {
    pub prims: Vec<CapturedPrim>,
    pub terrain: Option<TerrainData>,
    pub region_id: Uuid,
}

impl SceneGeometry {
    pub fn empty(region_id: Uuid) -> Self {
        Self {
            prims: Vec::new(),
            terrain: None,
            region_id,
        }
    }
}

pub fn capture_scene(
    scene_objects: &Arc<parking_lot::RwLock<HashMap<u32, SceneObject>>>,
    region_id: Uuid,
    terrain_heightmap: Option<Vec<f32>>,
) -> SceneGeometry {
    let prims: Vec<CapturedPrim> = {
        let objects = scene_objects.read();
        objects
            .values()
            .filter(|obj| obj.pcode == 9 && obj.attachment_point == 0)
            .map(|obj| {
                let (color, fullbright, glow, alpha, shininess) =
                    extract_te_properties(&obj.texture_entry);
                let shape = classify_shape(obj.profile_curve, obj.path_curve);

                CapturedPrim {
                    local_id: obj.local_id,
                    uuid: obj.uuid,
                    position: obj.position,
                    rotation: obj.rotation,
                    scale: obj.scale,
                    shape,
                    color,
                    fullbright,
                    glow,
                    alpha,
                    shininess,
                    profile_curve: obj.profile_curve,
                    path_curve: obj.path_curve,
                    profile_hollow: obj.profile_hollow,
                    parent_id: obj.parent_id,
                }
            })
            .collect()
    };

    let terrain = terrain_heightmap.map(|hm| {
        let side = (hm.len() as f64).sqrt() as usize;
        TerrainData {
            heightmap: hm,
            side: if side > 0 { side } else { 256 },
            region_origin: [0.0, 0.0, 0.0],
        }
    });

    info!(
        "[LUXOR] Captured scene: {} prims, terrain={}",
        prims.len(),
        terrain.is_some()
    );

    SceneGeometry {
        prims,
        terrain,
        region_id,
    }
}

fn classify_shape(profile_curve: u8, path_curve: u8) -> PrimShape {
    let profile_type = profile_curve & 0x0F;
    match (profile_type, path_curve) {
        (1, 16) => PrimShape::Box,
        (0, 16) => PrimShape::Cylinder,
        (5, 16) => PrimShape::Cylinder,
        (0, 32) => PrimShape::Sphere,
        (5, 32) => PrimShape::Sphere,
        (3, 16) => PrimShape::Prism,
        (0, 48) | (48, 32) => PrimShape::Torus,
        (1, 32) => PrimShape::Tube,
        (3, 32) => PrimShape::Ring,
        _ => PrimShape::Box,
    }
}

fn extract_te_properties(te: &[u8]) -> ([f32; 4], bool, f32, f32, u8) {
    if te.len() < 16 {
        return ([0.8, 0.8, 0.8, 1.0], false, 0.0, 1.0, 0);
    }

    let mut color = [1.0f32; 4];
    let mut fullbright = false;
    let mut glow = 0.0f32;
    let mut shininess: u8 = 0;

    let mut pos = 16;

    while pos < te.len() {
        if te[pos] == 0 {
            pos += 1;
            break;
        }
        let mut _face_bits: u64 = 0;
        loop {
            if pos >= te.len() {
                return (color, fullbright, glow, color[3], shininess);
            }
            let b = te[pos];
            _face_bits = (_face_bits << 7) | (b as u64 & 0x7F);
            pos += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        if pos + 16 > te.len() {
            break;
        }
        pos += 16;
    }

    if pos + 4 <= te.len() {
        let r_inv = te.get(pos).copied().unwrap_or(255);
        let g_inv = te.get(pos + 1).copied().unwrap_or(255);
        let b_inv = te.get(pos + 2).copied().unwrap_or(255);
        let a_inv = te.get(pos + 3).copied().unwrap_or(255);

        color[0] = (255 - r_inv) as f32 / 255.0;
        color[1] = (255 - g_inv) as f32 / 255.0;
        color[2] = (255 - b_inv) as f32 / 255.0;
        color[3] = (255 - a_inv) as f32 / 255.0;
    }

    while pos < te.len() {
        if te[pos] == 0 {
            pos += 1;
            break;
        }
        let mut _face_bits: u64 = 0;
        loop {
            if pos >= te.len() {
                return (color, fullbright, glow, color[3], shininess);
            }
            let b = te[pos];
            _face_bits = (_face_bits << 7) | (b as u64 & 0x7F);
            pos += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        if pos + 4 > te.len() {
            break;
        }
        pos += 4;
    }

    if pos + 2 <= te.len() {
        let _tex_s = te.get(pos).copied().unwrap_or(0);
        pos += 1;
        let _tex_t = te.get(pos).copied().unwrap_or(0);
        pos += 1;
    }

    while pos < te.len() {
        if te[pos] == 0 {
            pos += 1;
            break;
        }
        let mut _face_bits: u64 = 0;
        loop {
            if pos >= te.len() {
                return (color, fullbright, glow, color[3], shininess);
            }
            let b = te[pos];
            _face_bits = (_face_bits << 7) | (b as u64 & 0x7F);
            pos += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        if pos + 2 > te.len() {
            break;
        }
        pos += 2;
    }

    while pos < te.len() {
        if te[pos] == 0 {
            pos += 1;
            break;
        }
        let mut _face_bits: u64 = 0;
        loop {
            if pos >= te.len() {
                return (color, fullbright, glow, color[3], shininess);
            }
            let b = te[pos];
            _face_bits = (_face_bits << 7) | (b as u64 & 0x7F);
            pos += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        if pos + 2 > te.len() {
            break;
        }
        pos += 2;
    }

    if pos + 1 <= te.len() {
        let media_byte = te.get(pos).copied().unwrap_or(0);
        shininess = (media_byte >> 6) & 0x03;
        fullbright = (media_byte & 0x20) != 0;
        pos += 1;
    }

    while pos < te.len() {
        if te[pos] == 0 {
            pos += 1;
            break;
        }
        let mut _face_bits: u64 = 0;
        loop {
            if pos >= te.len() {
                return (color, fullbright, glow, color[3], shininess);
            }
            let b = te[pos];
            _face_bits = (_face_bits << 7) | (b as u64 & 0x7F);
            pos += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        if pos + 1 > te.len() {
            break;
        }
        pos += 1;
    }

    if pos + 1 <= te.len() {
        let glow_byte = te.get(pos).copied().unwrap_or(0);
        glow = glow_byte as f32 / 255.0;
    }

    (color, fullbright, glow, color[3], shininess)
}

pub fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let qx = q[0];
    let qy = q[1];
    let qz = q[2];
    let qw = q[3];

    let ix = qw * v[0] + qy * v[2] - qz * v[1];
    let iy = qw * v[1] + qz * v[0] - qx * v[2];
    let iz = qw * v[2] + qx * v[1] - qy * v[0];
    let iw = -qx * v[0] - qy * v[1] - qz * v[2];

    [
        ix * qw + iw * (-qx) + iy * (-qz) - iz * (-qy),
        iy * qw + iw * (-qy) + iz * (-qx) - ix * (-qz),
        iz * qw + iw * (-qz) + ix * (-qy) - iy * (-qx),
    ]
}

pub fn quat_inverse(q: [f32; 4]) -> [f32; 4] {
    let len_sq = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    if len_sq < 1e-10 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [
        -q[0] / len_sq,
        -q[1] / len_sq,
        -q[2] / len_sq,
        q[3] / len_sq,
    ]
}

pub fn world_to_local(point: [f32; 3], prim: &CapturedPrim) -> [f32; 3] {
    let translated = super::camera::sub(point, prim.position);
    let inv_rot = quat_inverse(prim.rotation);
    let rotated = quat_rotate(inv_rot, translated);
    [
        rotated[0] / prim.scale[0].max(1e-6),
        rotated[1] / prim.scale[1].max(1e-6),
        rotated[2] / prim.scale[2].max(1e-6),
    ]
}

pub fn local_normal_to_world(normal: [f32; 3], prim: &CapturedPrim) -> [f32; 3] {
    let scaled = [
        normal[0] / prim.scale[0].max(1e-6),
        normal[1] / prim.scale[1].max(1e-6),
        normal[2] / prim.scale[2].max(1e-6),
    ];
    let rotated = quat_rotate(prim.rotation, scaled);
    super::camera::normalize(rotated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_shapes() {
        assert_eq!(classify_shape(1, 16), PrimShape::Box);
        assert_eq!(classify_shape(0, 16), PrimShape::Cylinder);
        assert_eq!(classify_shape(0, 32), PrimShape::Sphere);
        assert_eq!(classify_shape(3, 16), PrimShape::Prism);
    }

    #[test]
    fn test_terrain_bilinear() {
        let terrain = TerrainData {
            heightmap: vec![10.0, 20.0, 30.0, 40.0],
            side: 2,
            region_origin: [0.0, 0.0, 0.0],
        };
        let h_corner = terrain.height_at(0.0, 0.0);
        assert!((h_corner - 10.0).abs() < 0.01);
        let h_mid = terrain.height_at(0.5, 0.5);
        assert!(h_mid > 10.0 && h_mid < 40.0);
    }

    #[test]
    fn test_quat_identity() {
        let q = [0.0f32, 0.0, 0.0, 1.0];
        let v = [1.0, 2.0, 3.0];
        let result = quat_rotate(q, v);
        assert!((result[0] - 1.0).abs() < 1e-5);
        assert!((result[1] - 2.0).abs() < 1e-5);
        assert!((result[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_quat_inverse_roundtrip() {
        let q = [0.0, 0.707, 0.0, 0.707f32];
        let v = [1.0, 0.0, 0.0];
        let rotated = quat_rotate(q, v);
        let inv = quat_inverse(q);
        let back = quat_rotate(inv, rotated);
        assert!((back[0] - v[0]).abs() < 0.01);
        assert!((back[1] - v[1]).abs() < 0.01);
        assert!((back[2] - v[2]).abs() < 0.01);
    }

    #[test]
    fn test_te_default_color() {
        let (color, _, _, _, _) = extract_te_properties(&[]);
        assert!((color[0] - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_terrain_normal() {
        let hm: Vec<f32> = (0..256 * 256).map(|_| 21.0).collect();
        let terrain = TerrainData {
            heightmap: hm,
            side: 256,
            region_origin: [0.0, 0.0, 0.0],
        };
        let n = terrain.normal_at(128.0, 128.0);
        assert!(
            (n[2] - 1.0).abs() < 0.01,
            "Flat terrain normal should point up: {:?}",
            n
        );
    }
}
