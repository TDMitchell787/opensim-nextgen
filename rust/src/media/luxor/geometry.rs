use super::camera::{cross, dot, length, normalize, sub, Ray};
use super::scene_capture::{
    local_normal_to_world, quat_inverse, quat_rotate, world_to_local, CapturedPrim, PrimShape,
    TerrainData,
};

#[derive(Debug, Clone, Copy)]
pub struct HitRecord {
    pub t: f32,
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub prim_index: usize,
    pub is_terrain: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl AABB {
    pub fn empty() -> Self {
        Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        }
    }

    pub fn from_prim(prim: &CapturedPrim) -> Self {
        let half = [
            prim.scale[0] * 0.5,
            prim.scale[1] * 0.5,
            prim.scale[2] * 0.5,
        ];

        let corners = [
            [-half[0], -half[1], -half[2]],
            [half[0], -half[1], -half[2]],
            [-half[0], half[1], -half[2]],
            [half[0], half[1], -half[2]],
            [-half[0], -half[1], half[2]],
            [half[0], -half[1], half[2]],
            [-half[0], half[1], half[2]],
            [half[0], half[1], half[2]],
        ];

        let mut aabb = AABB::empty();
        for corner in &corners {
            let world = quat_rotate(prim.rotation, *corner);
            let pos = [
                prim.position[0] + world[0],
                prim.position[1] + world[1],
                prim.position[2] + world[2],
            ];
            aabb.expand_point(pos);
        }
        aabb
    }

    pub fn expand_point(&mut self, p: [f32; 3]) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(p[i]);
            self.max[i] = self.max[i].max(p[i]);
        }
    }

    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    pub fn surface_area(&self) -> f32 {
        let dx = (self.max[0] - self.min[0]).max(0.0);
        let dy = (self.max[1] - self.min[1]).max(0.0);
        let dz = (self.max[2] - self.min[2]).max(0.0);
        2.0 * (dx * dy + dy * dz + dz * dx)
    }

    pub fn intersect_ray(&self, ray: &Ray) -> Option<f32> {
        let inv_dir = [
            if ray.direction[0].abs() > 1e-8 {
                1.0 / ray.direction[0]
            } else {
                1e8_f32.copysign(ray.direction[0])
            },
            if ray.direction[1].abs() > 1e-8 {
                1.0 / ray.direction[1]
            } else {
                1e8_f32.copysign(ray.direction[1])
            },
            if ray.direction[2].abs() > 1e-8 {
                1.0 / ray.direction[2]
            } else {
                1e8_f32.copysign(ray.direction[2])
            },
        ];

        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;

        for i in 0..3 {
            let t1 = (self.min[i] - ray.origin[i]) * inv_dir[i];
            let t2 = (self.max[i] - ray.origin[i]) * inv_dir[i];
            let ta = t1.min(t2);
            let tb = t1.max(t2);
            tmin = tmin.max(ta);
            tmax = tmax.min(tb);
        }

        if tmax >= tmin.max(0.0) {
            Some(tmin.max(0.0))
        } else {
            None
        }
    }
}

pub enum BVHNode {
    Leaf {
        prim_indices: Vec<usize>,
        bounds: AABB,
    },
    Internal {
        left: Box<BVHNode>,
        right: Box<BVHNode>,
        bounds: AABB,
    },
}

impl BVHNode {
    pub fn build(prims: &[CapturedPrim], indices: &mut [usize]) -> Self {
        if indices.len() <= 4 {
            let mut bounds = AABB::empty();
            for &idx in indices.iter() {
                bounds = bounds.merge(&AABB::from_prim(&prims[idx]));
            }
            return BVHNode::Leaf {
                prim_indices: indices.to_vec(),
                bounds,
            };
        }

        let mut bounds = AABB::empty();
        let mut centroid_bounds = AABB::empty();
        for &idx in indices.iter() {
            let aabb = AABB::from_prim(&prims[idx]);
            bounds = bounds.merge(&aabb);
            centroid_bounds.expand_point(aabb.center());
        }

        let dx = centroid_bounds.max[0] - centroid_bounds.min[0];
        let dy = centroid_bounds.max[1] - centroid_bounds.min[1];
        let dz = centroid_bounds.max[2] - centroid_bounds.min[2];
        let axis = if dx >= dy && dx >= dz {
            0
        } else if dy >= dz {
            1
        } else {
            2
        };

        let n = indices.len();
        if n <= 8 {
            indices.sort_by(|&a, &b| {
                let ca = AABB::from_prim(&prims[a]).center()[axis];
                let cb = AABB::from_prim(&prims[b]).center()[axis];
                ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
            });
            let mid = n / 2;
            let (left_indices, right_indices) = indices.split_at_mut(mid);
            let left = BVHNode::build(prims, left_indices);
            let right = BVHNode::build(prims, right_indices);
            return BVHNode::Internal {
                left: Box::new(left),
                right: Box::new(right),
                bounds,
            };
        }

        const NUM_BUCKETS: usize = 12;
        let mut best_cost = f32::INFINITY;
        let mut best_split = n / 2;

        indices.sort_by(|&a, &b| {
            let ca = AABB::from_prim(&prims[a]).center()[axis];
            let cb = AABB::from_prim(&prims[b]).center()[axis];
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let bucket_size = (n + NUM_BUCKETS - 1) / NUM_BUCKETS;
        for i in 1..NUM_BUCKETS {
            let split = (i * bucket_size).min(n - 1).max(1);
            if split == 0 || split >= n {
                continue;
            }

            let mut left_bounds = AABB::empty();
            for &idx in &indices[..split] {
                left_bounds = left_bounds.merge(&AABB::from_prim(&prims[idx]));
            }

            let mut right_bounds = AABB::empty();
            for &idx in &indices[split..] {
                right_bounds = right_bounds.merge(&AABB::from_prim(&prims[idx]));
            }

            let cost = split as f32 * left_bounds.surface_area()
                + (n - split) as f32 * right_bounds.surface_area();

            if cost < best_cost {
                best_cost = cost;
                best_split = split;
            }
        }

        let best_split = best_split.max(1).min(n - 1);
        let (left_indices, right_indices) = indices.split_at_mut(best_split);
        let left = BVHNode::build(prims, left_indices);
        let right = BVHNode::build(prims, right_indices);

        BVHNode::Internal {
            left: Box::new(left),
            right: Box::new(right),
            bounds,
        }
    }

    pub fn intersect(&self, ray: &Ray, prims: &[CapturedPrim], max_t: f32) -> Option<HitRecord> {
        match self {
            BVHNode::Leaf {
                prim_indices,
                bounds,
            } => {
                if bounds.intersect_ray(ray).map_or(true, |t| t > max_t) {
                    return None;
                }
                let mut closest: Option<HitRecord> = None;
                let mut best_t = max_t;
                for &idx in prim_indices {
                    if let Some(hit) = intersect_prim(ray, &prims[idx], best_t) {
                        let mut h = hit;
                        h.prim_index = idx;
                        best_t = h.t;
                        closest = Some(h);
                    }
                }
                closest
            }
            BVHNode::Internal {
                left,
                right,
                bounds,
            } => {
                if bounds.intersect_ray(ray).map_or(true, |t| t > max_t) {
                    return None;
                }
                let left_t = match &**left {
                    BVHNode::Leaf { bounds, .. } | BVHNode::Internal { bounds, .. } => {
                        bounds.intersect_ray(ray).unwrap_or(f32::INFINITY)
                    }
                };
                let right_t = match &**right {
                    BVHNode::Leaf { bounds, .. } | BVHNode::Internal { bounds, .. } => {
                        bounds.intersect_ray(ray).unwrap_or(f32::INFINITY)
                    }
                };

                let (first, second) = if left_t <= right_t {
                    (left, right)
                } else {
                    (right, left)
                };

                let mut best_t = max_t;
                let mut closest = first.intersect(ray, prims, best_t);
                if let Some(ref h) = closest {
                    best_t = h.t;
                }

                let second_bounds_t = match &**second {
                    BVHNode::Leaf { bounds, .. } | BVHNode::Internal { bounds, .. } => {
                        bounds.intersect_ray(ray).unwrap_or(f32::INFINITY)
                    }
                };

                if second_bounds_t < best_t {
                    if let Some(hit) = second.intersect(ray, prims, best_t) {
                        closest = Some(hit);
                    }
                }

                closest
            }
        }
    }
}

pub fn intersect_prim(ray: &Ray, prim: &CapturedPrim, max_t: f32) -> Option<HitRecord> {
    let aabb = AABB::from_prim(prim);
    if aabb.intersect_ray(ray).map_or(true, |t| t > max_t) {
        return None;
    }

    match prim.shape {
        PrimShape::Box | PrimShape::Tube => intersect_box(ray, prim, max_t),
        PrimShape::Sphere => intersect_sphere(ray, prim, max_t),
        PrimShape::Cylinder => intersect_cylinder(ray, prim, max_t),
        PrimShape::Torus | PrimShape::Ring => intersect_torus(ray, prim, max_t),
        PrimShape::Prism => intersect_prism(ray, prim, max_t),
        PrimShape::Terrain => None,
    }
}

fn intersect_box(ray: &Ray, prim: &CapturedPrim, max_t: f32) -> Option<HitRecord> {
    let inv_rot = quat_inverse(prim.rotation);
    let local_origin = quat_rotate(inv_rot, sub(ray.origin, prim.position));
    let local_dir = quat_rotate(inv_rot, ray.direction);

    let half = [
        prim.scale[0] * 0.5,
        prim.scale[1] * 0.5,
        prim.scale[2] * 0.5,
    ];

    let inv_dir = [
        if local_dir[0].abs() > 1e-8 {
            1.0 / local_dir[0]
        } else {
            1e8_f32.copysign(local_dir[0])
        },
        if local_dir[1].abs() > 1e-8 {
            1.0 / local_dir[1]
        } else {
            1e8_f32.copysign(local_dir[1])
        },
        if local_dir[2].abs() > 1e-8 {
            1.0 / local_dir[2]
        } else {
            1e8_f32.copysign(local_dir[2])
        },
    ];

    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    let mut normal_axis = 0usize;
    let mut normal_sign = 1.0f32;

    for i in 0..3 {
        let t1 = (-half[i] - local_origin[i]) * inv_dir[i];
        let t2 = (half[i] - local_origin[i]) * inv_dir[i];
        let (ta, tb, sign) = if t1 < t2 {
            (t1, t2, -1.0)
        } else {
            (t2, t1, 1.0)
        };

        if ta > tmin {
            tmin = ta;
            normal_axis = i;
            normal_sign = sign;
        }
        tmax = tmax.min(tb);
    }

    if tmax < tmin.max(0.001) || tmin > max_t {
        return None;
    }

    let t = if tmin > 0.001 { tmin } else { tmax };
    if t < 0.001 || t > max_t {
        return None;
    }

    let mut local_normal = [0.0f32; 3];
    local_normal[normal_axis] = normal_sign;

    let world_normal = quat_rotate(prim.rotation, local_normal);
    let position = ray.at(t);

    Some(HitRecord {
        t,
        position,
        normal: normalize(world_normal),
        prim_index: 0,
        is_terrain: false,
    })
}

fn intersect_sphere(ray: &Ray, prim: &CapturedPrim, max_t: f32) -> Option<HitRecord> {
    let radius = (prim.scale[0].min(prim.scale[1]).min(prim.scale[2])) * 0.5;
    let oc = sub(ray.origin, prim.position);

    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_d = discriminant.sqrt();
    let mut t = (-b - sqrt_d) / (2.0 * a);
    if t < 0.001 {
        t = (-b + sqrt_d) / (2.0 * a);
    }
    if t < 0.001 || t > max_t {
        return None;
    }

    let position = ray.at(t);
    let normal = normalize(sub(position, prim.position));

    Some(HitRecord {
        t,
        position,
        normal,
        prim_index: 0,
        is_terrain: false,
    })
}

fn intersect_cylinder(ray: &Ray, prim: &CapturedPrim, max_t: f32) -> Option<HitRecord> {
    let inv_rot = quat_inverse(prim.rotation);
    let local_origin = quat_rotate(inv_rot, sub(ray.origin, prim.position));
    let local_dir = quat_rotate(inv_rot, ray.direction);

    let rx = prim.scale[0] * 0.5;
    let ry = prim.scale[1] * 0.5;
    let half_h = prim.scale[2] * 0.5;

    let a = (local_dir[0] / rx).powi(2) + (local_dir[1] / ry).powi(2);
    let b = 2.0
        * ((local_origin[0] * local_dir[0]) / (rx * rx)
            + (local_origin[1] * local_dir[1]) / (ry * ry));
    let c = (local_origin[0] / rx).powi(2) + (local_origin[1] / ry).powi(2) - 1.0;

    let discriminant = b * b - 4.0 * a * c;

    let mut best_t = max_t + 1.0;
    let mut best_normal = [0.0f32; 3];

    if discriminant >= 0.0 {
        let sqrt_d = discriminant.sqrt();
        for t_cand in [(-b - sqrt_d) / (2.0 * a), (-b + sqrt_d) / (2.0 * a)] {
            if t_cand > 0.001 && t_cand < best_t {
                let p = [
                    local_origin[0] + local_dir[0] * t_cand,
                    local_origin[1] + local_dir[1] * t_cand,
                    local_origin[2] + local_dir[2] * t_cand,
                ];
                if p[2] >= -half_h && p[2] <= half_h {
                    best_t = t_cand;
                    best_normal = normalize([p[0] / (rx * rx), p[1] / (ry * ry), 0.0]);
                }
            }
        }
    }

    for cap_z in [-half_h, half_h] {
        if local_dir[2].abs() > 1e-8 {
            let t_cap = (cap_z - local_origin[2]) / local_dir[2];
            if t_cap > 0.001 && t_cap < best_t {
                let px = local_origin[0] + local_dir[0] * t_cap;
                let py = local_origin[1] + local_dir[1] * t_cap;
                if (px / rx).powi(2) + (py / ry).powi(2) <= 1.0 {
                    best_t = t_cap;
                    best_normal = [0.0, 0.0, if cap_z > 0.0 { 1.0 } else { -1.0 }];
                }
            }
        }
    }

    if best_t > max_t {
        return None;
    }

    let world_normal = normalize(quat_rotate(prim.rotation, best_normal));
    let position = ray.at(best_t);

    Some(HitRecord {
        t: best_t,
        position,
        normal: world_normal,
        prim_index: 0,
        is_terrain: false,
    })
}

fn intersect_torus(ray: &Ray, prim: &CapturedPrim, max_t: f32) -> Option<HitRecord> {
    let major_r = (prim.scale[0].max(prim.scale[1])) * 0.35;
    let minor_r = prim.scale[2] * 0.25;

    let center = prim.position;
    let oc = sub(ray.origin, center);
    let d = ray.direction;

    let r2 = major_r * major_r;
    let s2 = minor_r * minor_r;

    let od = dot(oc, d);
    let oo = dot(oc, oc);
    let dd = dot(d, d);

    let k = oo - r2 - s2;

    let a4 = dd * dd;
    let a3 = 4.0 * dd * od;
    let a2 = 2.0 * dd * k + 4.0 * od * od + 4.0 * r2 * d[2] * d[2];
    let a1 = 4.0 * k * od + 8.0 * r2 * oc[2] * d[2];
    let a0 = k * k - 4.0 * r2 * (s2 - oc[2] * oc[2]);

    let roots = solve_quartic_numerical(a4, a3, a2, a1, a0, max_t);

    let mut best_t_val = max_t + 1.0;
    for t_val in roots {
        if t_val > 0.001 && t_val < best_t_val {
            best_t_val = t_val;
        }
    }

    if best_t_val > max_t {
        return None;
    }

    let position = ray.at(best_t_val);
    let pc = sub(position, center);
    let proj_len = (pc[0] * pc[0] + pc[1] * pc[1]).sqrt();
    let on_ring = if proj_len > 1e-6 {
        [pc[0] / proj_len * major_r, pc[1] / proj_len * major_r, 0.0]
    } else {
        [major_r, 0.0, 0.0]
    };
    let normal = normalize(sub(pc, on_ring));

    Some(HitRecord {
        t: best_t_val,
        position,
        normal,
        prim_index: 0,
        is_terrain: false,
    })
}

fn solve_quartic_numerical(a4: f32, a3: f32, a2: f32, a1: f32, a0: f32, max_t: f32) -> Vec<f32> {
    let mut roots = Vec::new();
    let steps = 64;
    let step = max_t / steps as f32;

    let eval = |t: f32| -> f32 { a4 * t * t * t * t + a3 * t * t * t + a2 * t * t + a1 * t + a0 };

    let mut prev_val = eval(0.001);
    for i in 1..=steps {
        let t = 0.001 + i as f32 * step;
        let val = eval(t);
        if prev_val * val < 0.0 {
            let mut lo = t - step;
            let mut hi = t;
            for _ in 0..20 {
                let mid = (lo + hi) * 0.5;
                let mid_val = eval(mid);
                if mid_val * eval(lo) < 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            roots.push((lo + hi) * 0.5);
            if roots.len() >= 4 {
                break;
            }
        }
        prev_val = val;
    }
    roots
}

fn intersect_prism(ray: &Ray, prim: &CapturedPrim, max_t: f32) -> Option<HitRecord> {
    let inv_rot = quat_inverse(prim.rotation);
    let local_origin = quat_rotate(inv_rot, sub(ray.origin, prim.position));
    let local_dir = quat_rotate(inv_rot, ray.direction);

    let hx = prim.scale[0] * 0.5;
    let hy = prim.scale[1] * 0.5;
    let hz = prim.scale[2] * 0.5;

    let verts = [
        [0.0, hy, hz],
        [-hx, -hy, hz],
        [hx, -hy, hz],
        [0.0, hy, -hz],
        [-hx, -hy, -hz],
        [hx, -hy, -hz],
    ];

    let faces: &[([usize; 3], [f32; 3])] = &[
        ([0, 1, 2], [0.0, 0.0, 1.0]),
        ([3, 5, 4], [0.0, 0.0, -1.0]),
        ([0, 3, 4], [-0.866, 0.5, 0.0]),
        ([0, 4, 1], [-0.866, 0.5, 0.0]),
        ([0, 2, 5], [0.866, 0.5, 0.0]),
        ([0, 5, 3], [0.866, 0.5, 0.0]),
        ([1, 4, 5], [0.0, -1.0, 0.0]),
        ([1, 5, 2], [0.0, -1.0, 0.0]),
    ];

    let mut best_t = max_t + 1.0;
    let mut best_normal = [0.0f32; 3];

    for (indices, face_normal) in faces {
        let v0 = verts[indices[0]];
        let v1 = verts[indices[1]];
        let v2 = verts[indices[2]];

        if let Some(t) = ray_triangle(local_origin, local_dir, v0, v1, v2) {
            if t > 0.001 && t < best_t {
                best_t = t;
                best_normal = *face_normal;
            }
        }
    }

    if best_t > max_t {
        return None;
    }

    let world_normal = normalize(quat_rotate(prim.rotation, best_normal));
    let position = ray.at(best_t);

    Some(HitRecord {
        t: best_t,
        position,
        normal: world_normal,
        prim_index: 0,
        is_terrain: false,
    })
}

fn ray_triangle(
    origin: [f32; 3],
    dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let edge1 = sub(v1, v0);
    let edge2 = sub(v2, v0);
    let h = cross(dir, edge2);
    let a = dot(edge1, h);

    if a.abs() < 1e-8 {
        return None;
    }

    let f = 1.0 / a;
    let s = sub(origin, v0);
    let u = f * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = cross(s, edge1);
    let v = f * dot(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * dot(edge2, q);
    if t > 0.001 {
        Some(t)
    } else {
        None
    }
}

pub fn intersect_terrain(ray: &Ray, terrain: &TerrainData, max_t: f32) -> Option<HitRecord> {
    let step = 0.5;
    let max_steps = (max_t / step).min(2000.0) as usize;

    let mut t = 0.1;
    let mut prev_pos = ray.at(t);
    let mut prev_h = terrain.height_at(prev_pos[0], prev_pos[1]);

    for _ in 0..max_steps {
        t += step;
        if t > max_t {
            break;
        }

        let pos = ray.at(t);
        let h = terrain.height_at(pos[0], pos[1]);

        if pos[0] < terrain.region_origin[0]
            || pos[1] < terrain.region_origin[1]
            || pos[0] >= terrain.region_origin[0] + terrain.side as f32
            || pos[1] >= terrain.region_origin[1] + terrain.side as f32
        {
            prev_pos = pos;
            prev_h = h;
            continue;
        }

        if prev_pos[2] >= prev_h && pos[2] < h {
            let mut lo = t - step;
            let mut hi = t;
            for _ in 0..16 {
                let mid = (lo + hi) * 0.5;
                let mid_pos = ray.at(mid);
                let mid_h = terrain.height_at(mid_pos[0], mid_pos[1]);
                if mid_pos[2] > mid_h {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }

            let hit_t = (lo + hi) * 0.5;
            let hit_pos = ray.at(hit_t);
            let normal = terrain.normal_at(hit_pos[0], hit_pos[1]);

            return Some(HitRecord {
                t: hit_t,
                position: hit_pos,
                normal,
                prim_index: 0,
                is_terrain: true,
            });
        }

        prev_pos = pos;
        prev_h = h;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::scene_capture::PrimShape;
    use super::*;

    fn make_test_prim(shape: PrimShape, position: [f32; 3], scale: [f32; 3]) -> CapturedPrim {
        CapturedPrim {
            local_id: 1,
            uuid: uuid::Uuid::nil(),
            position,
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale,
            shape,
            color: [1.0, 0.0, 0.0, 1.0],
            fullbright: false,
            glow: 0.0,
            alpha: 1.0,
            shininess: 0,
            profile_curve: 1,
            path_curve: 16,
            profile_hollow: 0,
            parent_id: 0,
        }
    }

    #[test]
    fn test_ray_box_hit() {
        let prim = make_test_prim(PrimShape::Box, [5.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        let hit = intersect_box(&ray, &prim, 100.0);
        assert!(hit.is_some(), "Ray should hit box");
        let h = hit.unwrap();
        assert!((h.t - 4.0).abs() < 0.1, "t should be ~4.0, got {}", h.t);
    }

    #[test]
    fn test_ray_box_miss() {
        let prim = make_test_prim(PrimShape::Box, [5.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 1.0, 0.0],
        };
        assert!(intersect_box(&ray, &prim, 100.0).is_none());
    }

    #[test]
    fn test_ray_sphere_hit() {
        let prim = make_test_prim(PrimShape::Sphere, [5.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        let hit = intersect_sphere(&ray, &prim, 100.0);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.t - 4.0).abs() < 0.1, "t should be ~4.0, got {}", h.t);
    }

    #[test]
    fn test_ray_cylinder_hit() {
        let prim = make_test_prim(PrimShape::Cylinder, [5.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        let hit = intersect_cylinder(&ray, &prim, 100.0);
        assert!(hit.is_some(), "Ray should hit cylinder");
    }

    #[test]
    fn test_ray_prism_hit() {
        let prim = make_test_prim(PrimShape::Prism, [5.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        let hit = intersect_prism(&ray, &prim, 100.0);
        assert!(hit.is_some(), "Ray should hit prism");
    }

    #[test]
    fn test_aabb_from_prim() {
        let prim = make_test_prim(PrimShape::Box, [10.0, 10.0, 10.0], [4.0, 4.0, 4.0]);
        let aabb = AABB::from_prim(&prim);
        assert!((aabb.min[0] - 8.0).abs() < 0.01);
        assert!((aabb.max[0] - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_bvh_build_and_intersect() {
        let prims = vec![
            make_test_prim(PrimShape::Box, [5.0, 0.0, 0.0], [2.0, 2.0, 2.0]),
            make_test_prim(PrimShape::Sphere, [10.0, 0.0, 0.0], [2.0, 2.0, 2.0]),
            make_test_prim(PrimShape::Box, [15.0, 0.0, 0.0], [2.0, 2.0, 2.0]),
        ];
        let mut indices: Vec<usize> = (0..prims.len()).collect();
        let bvh = BVHNode::build(&prims, &mut indices);

        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        let hit = bvh.intersect(&ray, &prims, 100.0);
        assert!(hit.is_some(), "Should hit first box");
        assert!((hit.unwrap().t - 4.0).abs() < 0.2);
    }

    #[test]
    fn test_terrain_intersection() {
        let hm: Vec<f32> = (0..256 * 256).map(|_| 21.0).collect();
        let terrain = TerrainData {
            heightmap: hm,
            side: 256,
            region_origin: [0.0, 0.0, 0.0],
        };
        let ray = Ray {
            origin: [128.0, 128.0, 50.0],
            direction: normalize([0.0, 0.0, -1.0]),
        };
        let hit = intersect_terrain(&ray, &terrain, 100.0);
        assert!(hit.is_some(), "Should hit flat terrain");
        let h = hit.unwrap();
        assert!(
            (h.position[2] - 21.0).abs() < 1.0,
            "Hit z should be ~21, got {}",
            h.position[2]
        );
        assert!(h.is_terrain);
    }

    #[test]
    fn test_ray_triangle() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let t = ray_triangle([0.2, 0.2, 1.0], [0.0, 0.0, -1.0], v0, v1, v2);
        assert!(t.is_some());
        assert!((t.unwrap() - 1.0).abs() < 0.01);
    }
}
