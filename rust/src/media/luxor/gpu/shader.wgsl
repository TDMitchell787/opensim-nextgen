struct Uniforms {
    cam_position: vec3<f32>,
    cam_fov_h: f32,
    cam_forward: vec3<f32>,
    cam_fov_v: f32,
    cam_right: vec3<f32>,
    cam_aperture_r: f32,
    cam_up: vec3<f32>,
    cam_focus_dist: f32,

    width: u32,
    height: u32,
    spp: u32,
    num_prims: u32,

    num_lights: u32,
    num_bvh_nodes: u32,
    has_terrain: u32,
    terrain_side: u32,

    terrain_origin: vec3<f32>,
    max_t: f32,

    ambient_color: vec3<f32>,
    ambient_intensity: f32,

    use_dof: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct GpuPrim {
    position: vec3<f32>,
    shape: u32,
    rotation: vec4<f32>,
    scale: vec3<f32>,
    shininess: u32,
    color: vec4<f32>,
    fullbright: u32,
    glow: f32,
    alpha: f32,
    _pad: u32,
};

struct GpuBvhNode {
    bounds_min: vec3<f32>,
    left_or_first: i32,
    bounds_max: vec3<f32>,
    count_or_right: i32,
    miss_link: i32,
    is_leaf: u32,
    _pad0: u32,
    _pad1: u32,
};

struct GpuLight {
    position: vec3<f32>,
    light_type: u32,
    direction: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    radius: f32,
    spot_angle: f32,
    soft_edge: f32,
    _pad0: f32,
    _pad1: f32,
};

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

struct HitRecord {
    t: f32,
    position: vec3<f32>,
    normal: vec3<f32>,
    prim_index: i32,
    is_terrain: u32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var<storage, read> prims: array<GpuPrim>;
@group(0) @binding(2) var<storage, read> bvh: array<GpuBvhNode>;
@group(0) @binding(3) var<storage, read> prim_indices: array<u32>;
@group(0) @binding(4) var<storage, read> lights: array<GpuLight>;
@group(0) @binding(5) var<storage, read> terrain: array<f32>;
@group(0) @binding(6) var<storage, read_write> output: array<u32>;

fn xorshift32(state: u32) -> u32 {
    var s = state;
    s = s ^ (s << 13u);
    s = s ^ (s >> 17u);
    s = s ^ (s << 5u);
    return s;
}

fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let qxyz = q.xyz;
    let qw = q.w;
    let t = 2.0 * cross(qxyz, v);
    return v + qw * t + cross(qxyz, t);
}

fn quat_inverse(q: vec4<f32>) -> vec4<f32> {
    let len_sq = dot(q, q);
    if len_sq < 1e-10 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return vec4<f32>(-q.x / len_sq, -q.y / len_sq, -q.z / len_sq, q.w / len_sq);
}

fn generate_ray(pixel_u: f32, pixel_v: f32) -> Ray {
    let half_w = tan(u.cam_fov_h * 0.5);
    let half_h = half_w / (f32(u.width) / f32(u.height));

    let px = (2.0 * pixel_u - 1.0) * half_w;
    let py = (2.0 * pixel_v - 1.0) * half_h;

    let dir = normalize(u.cam_forward + px * u.cam_right + py * u.cam_up);
    return Ray(u.cam_position, dir);
}

fn generate_dof_ray(pixel_u: f32, pixel_v: f32, rng: ptr<function, u32>) -> Ray {
    let primary = generate_ray(pixel_u, pixel_v);

    let focus_t = u.cam_focus_dist / max(dot(primary.direction, u.cam_forward), 1e-8);
    let focus_point = primary.origin + primary.direction * focus_t;

    *rng = xorshift32(*rng);
    let lu = f32(*rng) / 4294967295.0;
    *rng = xorshift32(*rng);
    let lv = f32(*rng) / 4294967295.0;

    let sx = 2.0 * lu - 1.0;
    let sy = 2.0 * lv - 1.0;
    var disk_x = 0.0;
    var disk_y = 0.0;
    if abs(sx) > 1e-6 || abs(sy) > 1e-6 {
        if abs(sx) > abs(sy) {
            let theta = 0.785398 * (sy / sx);
            disk_x = sx * cos(theta);
            disk_y = sx * sin(theta);
        } else {
            let theta = 1.570796 - 0.785398 * (sx / sy);
            disk_x = sy * cos(theta);
            disk_y = sy * sin(theta);
        }
    }

    let offset_x = disk_x * u.cam_aperture_r;
    let offset_y = disk_y * u.cam_aperture_r;

    let new_origin = u.cam_position + u.cam_right * offset_x + u.cam_up * offset_y;
    let new_dir = normalize(focus_point - new_origin);
    return Ray(new_origin, new_dir);
}

fn intersect_aabb(ray: Ray, bmin: vec3<f32>, bmax: vec3<f32>) -> f32 {
    let inv_dir = vec3<f32>(
        select(1e8 * sign(ray.direction.x), 1.0 / ray.direction.x, abs(ray.direction.x) > 1e-8),
        select(1e8 * sign(ray.direction.y), 1.0 / ray.direction.y, abs(ray.direction.y) > 1e-8),
        select(1e8 * sign(ray.direction.z), 1.0 / ray.direction.z, abs(ray.direction.z) > 1e-8),
    );

    let t1 = (bmin - ray.origin) * inv_dir;
    let t2 = (bmax - ray.origin) * inv_dir;
    let tmin_v = min(t1, t2);
    let tmax_v = max(t1, t2);
    let tmin = max(max(tmin_v.x, tmin_v.y), tmin_v.z);
    let tmax = min(min(tmax_v.x, tmax_v.y), tmax_v.z);

    if tmax >= max(tmin, 0.0) {
        return max(tmin, 0.0);
    }
    return -1.0;
}

fn intersect_box_shape(ray: Ray, prim: GpuPrim) -> HitRecord {
    let inv_rot = quat_inverse(prim.rotation);
    let local_o = quat_rotate(inv_rot, ray.origin - prim.position);
    let local_d = quat_rotate(inv_rot, ray.direction);

    let half = prim.scale * 0.5;

    let inv_d = vec3<f32>(
        select(1e8 * sign(local_d.x), 1.0 / local_d.x, abs(local_d.x) > 1e-8),
        select(1e8 * sign(local_d.y), 1.0 / local_d.y, abs(local_d.y) > 1e-8),
        select(1e8 * sign(local_d.z), 1.0 / local_d.z, abs(local_d.z) > 1e-8),
    );

    let t1 = (-half - local_o) * inv_d;
    let t2 = (half - local_o) * inv_d;
    let ta = min(t1, t2);
    let tb = max(t1, t2);

    var tmin = max(max(ta.x, ta.y), ta.z);
    let tmax = min(min(tb.x, tb.y), tb.z);

    var hit: HitRecord;
    hit.t = -1.0;

    if tmax < max(tmin, 0.001) {
        return hit;
    }

    var t = tmin;
    if t < 0.001 { t = tmax; }
    if t < 0.001 { return hit; }

    var local_n = vec3<f32>(0.0);
    if abs(tmin - ta.x) < 1e-5 { local_n = vec3<f32>(select(1.0, -1.0, local_d.x > 0.0), 0.0, 0.0); }
    else if abs(tmin - ta.y) < 1e-5 { local_n = vec3<f32>(0.0, select(1.0, -1.0, local_d.y > 0.0), 0.0); }
    else { local_n = vec3<f32>(0.0, 0.0, select(1.0, -1.0, local_d.z > 0.0)); }

    hit.t = t;
    hit.position = ray.origin + ray.direction * t;
    hit.normal = normalize(quat_rotate(prim.rotation, local_n));
    hit.prim_index = -1;
    hit.is_terrain = 0u;
    return hit;
}

fn intersect_sphere_shape(ray: Ray, prim: GpuPrim) -> HitRecord {
    let radius = min(prim.scale.x, min(prim.scale.y, prim.scale.z)) * 0.5;
    let oc = ray.origin - prim.position;

    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    var hit: HitRecord;
    hit.t = -1.0;

    if discriminant < 0.0 { return hit; }

    let sqrt_d = sqrt(discriminant);
    var t = (-b - sqrt_d) / (2.0 * a);
    if t < 0.001 { t = (-b + sqrt_d) / (2.0 * a); }
    if t < 0.001 { return hit; }

    hit.t = t;
    hit.position = ray.origin + ray.direction * t;
    hit.normal = normalize(hit.position - prim.position);
    hit.prim_index = -1;
    hit.is_terrain = 0u;
    return hit;
}

fn intersect_cylinder_shape(ray: Ray, prim: GpuPrim) -> HitRecord {
    let inv_rot = quat_inverse(prim.rotation);
    let local_o = quat_rotate(inv_rot, ray.origin - prim.position);
    let local_d = quat_rotate(inv_rot, ray.direction);

    let rx = prim.scale.x * 0.5;
    let ry = prim.scale.y * 0.5;
    let half_h = prim.scale.z * 0.5;

    let a = (local_d.x / rx) * (local_d.x / rx) + (local_d.y / ry) * (local_d.y / ry);
    let b = 2.0 * ((local_o.x * local_d.x) / (rx * rx) + (local_o.y * local_d.y) / (ry * ry));
    let c = (local_o.x / rx) * (local_o.x / rx) + (local_o.y / ry) * (local_o.y / ry) - 1.0;
    let disc = b * b - 4.0 * a * c;

    var hit: HitRecord;
    hit.t = -1.0;
    var best_t = 1e10;
    var best_n = vec3<f32>(0.0);

    if disc >= 0.0 {
        let sqrt_d = sqrt(disc);
        let t_cands = array<f32, 2>((-b - sqrt_d) / (2.0 * a), (-b + sqrt_d) / (2.0 * a));
        for (var i = 0; i < 2; i++) {
            let tc = t_cands[i];
            if tc > 0.001 && tc < best_t {
                let p = local_o + local_d * tc;
                if p.z >= -half_h && p.z <= half_h {
                    best_t = tc;
                    best_n = normalize(vec3<f32>(p.x / (rx * rx), p.y / (ry * ry), 0.0));
                }
            }
        }
    }

    for (var cap = 0; cap < 2; cap++) {
        let cap_z = select(-half_h, half_h, cap == 1);
        if abs(local_d.z) > 1e-8 {
            let t_cap = (cap_z - local_o.z) / local_d.z;
            if t_cap > 0.001 && t_cap < best_t {
                let px = local_o.x + local_d.x * t_cap;
                let py = local_o.y + local_d.y * t_cap;
                if (px / rx) * (px / rx) + (py / ry) * (py / ry) <= 1.0 {
                    best_t = t_cap;
                    best_n = vec3<f32>(0.0, 0.0, select(-1.0, 1.0, cap_z > 0.0));
                }
            }
        }
    }

    if best_t > 1e9 { return hit; }

    hit.t = best_t;
    hit.position = ray.origin + ray.direction * best_t;
    hit.normal = normalize(quat_rotate(prim.rotation, best_n));
    hit.prim_index = -1;
    hit.is_terrain = 0u;
    return hit;
}

fn intersect_torus_approx(ray: Ray, prim: GpuPrim) -> HitRecord {
    let major_r = max(prim.scale.x, prim.scale.y) * 0.35;
    let minor_r = prim.scale.z * 0.25;
    let center = prim.position;
    let oc = ray.origin - center;
    let d = ray.direction;

    let r2 = major_r * major_r;
    let s2 = minor_r * minor_r;
    let od = dot(oc, d);
    let oo = dot(oc, oc);
    let dd = dot(d, d);
    let k = oo - r2 - s2;

    let a4 = dd * dd;
    let a3 = 4.0 * dd * od;
    let a2 = 2.0 * dd * k + 4.0 * od * od + 4.0 * r2 * d.z * d.z;
    let a1 = 4.0 * k * od + 8.0 * r2 * oc.z * d.z;
    let a0 = k * k - 4.0 * r2 * (s2 - oc.z * oc.z);

    var hit: HitRecord;
    hit.t = -1.0;
    var best_t = u.max_t + 1.0;

    let steps = 32u;
    let step_size = u.max_t / f32(steps);
    var prev_val = a4 * 0.001 * 0.001 * 0.001 * 0.001 + a3 * 0.001 * 0.001 * 0.001 + a2 * 0.001 * 0.001 + a1 * 0.001 + a0;

    for (var i = 1u; i <= steps; i++) {
        let t_sample = 0.001 + f32(i) * step_size;
        let t2 = t_sample * t_sample;
        let val = a4 * t2 * t2 + a3 * t_sample * t2 + a2 * t2 + a1 * t_sample + a0;

        if prev_val * val < 0.0 {
            var lo = t_sample - step_size;
            var hi = t_sample;
            for (var j = 0; j < 16; j++) {
                let mid = (lo + hi) * 0.5;
                let m2 = mid * mid;
                let mid_val = a4 * m2 * m2 + a3 * mid * m2 + a2 * m2 + a1 * mid + a0;
                let lo2 = lo * lo;
                let lo_val = a4 * lo2 * lo2 + a3 * lo * lo2 + a2 * lo2 + a1 * lo + a0;
                if mid_val * lo_val < 0.0 { hi = mid; } else { lo = mid; }
            }
            let root = (lo + hi) * 0.5;
            if root > 0.001 && root < best_t {
                best_t = root;
                break;
            }
        }
        prev_val = val;
    }

    if best_t > u.max_t { return hit; }

    let pos = ray.origin + ray.direction * best_t;
    let pc = pos - center;
    let proj_len = sqrt(pc.x * pc.x + pc.y * pc.y);
    var on_ring = vec3<f32>(major_r, 0.0, 0.0);
    if proj_len > 1e-6 {
        on_ring = vec3<f32>(pc.x / proj_len * major_r, pc.y / proj_len * major_r, 0.0);
    }

    hit.t = best_t;
    hit.position = pos;
    hit.normal = normalize(pc - on_ring);
    hit.prim_index = -1;
    hit.is_terrain = 0u;
    return hit;
}

fn ray_triangle(origin: vec3<f32>, dir: vec3<f32>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32 {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = cross(dir, edge2);
    let a = dot(edge1, h);
    if abs(a) < 1e-8 { return -1.0; }
    let f = 1.0 / a;
    let s = origin - v0;
    let uu = f * dot(s, h);
    if uu < 0.0 || uu > 1.0 { return -1.0; }
    let q = cross(s, edge1);
    let vv = f * dot(dir, q);
    if vv < 0.0 || uu + vv > 1.0 { return -1.0; }
    let t = f * dot(edge2, q);
    if t > 0.001 { return t; }
    return -1.0;
}

fn intersect_prism_shape(ray: Ray, prim: GpuPrim) -> HitRecord {
    let inv_rot = quat_inverse(prim.rotation);
    let local_o = quat_rotate(inv_rot, ray.origin - prim.position);
    let local_d = quat_rotate(inv_rot, ray.direction);

    let hx = prim.scale.x * 0.5;
    let hy = prim.scale.y * 0.5;
    let hz = prim.scale.z * 0.5;

    let v0 = vec3<f32>(0.0, hy, hz);
    let v1 = vec3<f32>(-hx, -hy, hz);
    let v2 = vec3<f32>(hx, -hy, hz);
    let v3 = vec3<f32>(0.0, hy, -hz);
    let v4 = vec3<f32>(-hx, -hy, -hz);
    let v5 = vec3<f32>(hx, -hy, -hz);

    var hit: HitRecord;
    hit.t = -1.0;

    var final_n = vec3<f32>(0.0, 0.0, 1.0);
    var best_t = 1e10;

    var t0 = ray_triangle(local_o, local_d, v0, v1, v2);
    if t0 > 0.001 && t0 < best_t { best_t = t0; final_n = vec3<f32>(0.0, 0.0, 1.0); }
    t0 = ray_triangle(local_o, local_d, v3, v5, v4);
    if t0 > 0.001 && t0 < best_t { best_t = t0; final_n = vec3<f32>(0.0, 0.0, -1.0); }
    t0 = ray_triangle(local_o, local_d, v0, v3, v4);
    if t0 > 0.001 && t0 < best_t { best_t = t0; final_n = vec3<f32>(-0.866, 0.5, 0.0); }
    t0 = ray_triangle(local_o, local_d, v0, v4, v1);
    if t0 > 0.001 && t0 < best_t { best_t = t0; final_n = vec3<f32>(-0.866, 0.5, 0.0); }
    t0 = ray_triangle(local_o, local_d, v0, v2, v5);
    if t0 > 0.001 && t0 < best_t { best_t = t0; final_n = vec3<f32>(0.866, 0.5, 0.0); }
    t0 = ray_triangle(local_o, local_d, v0, v5, v3);
    if t0 > 0.001 && t0 < best_t { best_t = t0; final_n = vec3<f32>(0.866, 0.5, 0.0); }
    t0 = ray_triangle(local_o, local_d, v1, v4, v5);
    if t0 > 0.001 && t0 < best_t { best_t = t0; final_n = vec3<f32>(0.0, -1.0, 0.0); }
    t0 = ray_triangle(local_o, local_d, v1, v5, v2);
    if t0 > 0.001 && t0 < best_t { best_t = t0; final_n = vec3<f32>(0.0, -1.0, 0.0); }

    if best_t > 1e9 { return hit; }

    hit.t = best_t;
    hit.position = ray.origin + ray.direction * best_t;
    hit.normal = normalize(quat_rotate(prim.rotation, final_n));
    hit.prim_index = -1;
    hit.is_terrain = 0u;
    return hit;
}

fn intersect_prim(ray: Ray, prim: GpuPrim, max_t_val: f32) -> HitRecord {
    var hit: HitRecord;
    hit.t = -1.0;

    switch prim.shape {
        case 0u: { hit = intersect_box_shape(ray, prim); }
        case 6u: { hit = intersect_box_shape(ray, prim); }
        case 1u: { hit = intersect_sphere_shape(ray, prim); }
        case 2u: { hit = intersect_cylinder_shape(ray, prim); }
        case 3u: { hit = intersect_torus_approx(ray, prim); }
        case 5u: { hit = intersect_torus_approx(ray, prim); }
        case 4u: { hit = intersect_prism_shape(ray, prim); }
        default: { hit = intersect_box_shape(ray, prim); }
    }

    if hit.t > max_t_val { hit.t = -1.0; }
    return hit;
}

fn traverse_bvh(ray: Ray, max_t_val: f32) -> HitRecord {
    var closest: HitRecord;
    closest.t = -1.0;
    var best_t = max_t_val;

    if u.num_prims == 0u { return closest; }

    var stack: array<i32, 32>;
    var sp = 0;
    stack[0] = 0;
    sp = 1;

    while sp > 0 {
        sp -= 1;
        let node_idx = stack[sp];
        if node_idx < 0 { continue; }

        let node = bvh[node_idx];
        let box_t = intersect_aabb(ray, node.bounds_min, node.bounds_max);
        if box_t < 0.0 || box_t > best_t { continue; }

        if node.is_leaf == 1u {
            let first = node.left_or_first;
            let count = node.count_or_right;
            for (var i = 0; i < count; i++) {
                let prim_idx = prim_indices[first + i];
                let prim = prims[prim_idx];
                let hit = intersect_prim(ray, prim, best_t);
                if hit.t > 0.001 && hit.t < best_t {
                    closest = hit;
                    closest.prim_index = i32(prim_idx);
                    best_t = hit.t;
                }
            }
        } else {
            let left_idx = node.left_or_first;
            let right_idx = node.count_or_right;

            let left_t = intersect_aabb(ray, bvh[left_idx].bounds_min, bvh[left_idx].bounds_max);
            let right_t = intersect_aabb(ray, bvh[right_idx].bounds_min, bvh[right_idx].bounds_max);

            if left_t >= 0.0 && right_t >= 0.0 {
                if left_t <= right_t {
                    if sp < 31 { stack[sp] = right_idx; sp += 1; }
                    if sp < 31 { stack[sp] = left_idx; sp += 1; }
                } else {
                    if sp < 31 { stack[sp] = left_idx; sp += 1; }
                    if sp < 31 { stack[sp] = right_idx; sp += 1; }
                }
            } else if left_t >= 0.0 {
                if sp < 31 { stack[sp] = left_idx; sp += 1; }
            } else if right_t >= 0.0 {
                if sp < 31 { stack[sp] = right_idx; sp += 1; }
            }
        }
    }

    return closest;
}

fn terrain_height_at(x: f32, y: f32) -> f32 {
    if u.has_terrain == 0u { return 0.0; }
    let lx = x - u.terrain_origin.x;
    let ly = y - u.terrain_origin.y;
    let side = f32(u.terrain_side);

    if lx < 0.0 || ly < 0.0 || lx >= side || ly >= side { return 0.0; }

    let ix = u32(min(lx, side - 2.0));
    let iy = u32(min(ly, side - 2.0));
    let fx = lx - f32(ix);
    let fy = ly - f32(iy);

    let h00 = terrain[iy * u.terrain_side + ix];
    let h10 = terrain[iy * u.terrain_side + ix + 1u];
    let h01 = terrain[(iy + 1u) * u.terrain_side + ix];
    let h11 = terrain[(iy + 1u) * u.terrain_side + ix + 1u];

    let h0 = h00 + (h10 - h00) * fx;
    let h1 = h01 + (h11 - h01) * fx;
    return h0 + (h1 - h0) * fy;
}

fn terrain_normal_at(x: f32, y: f32) -> vec3<f32> {
    let hc = terrain_height_at(x, y);
    let hx = terrain_height_at(x + 1.0, y);
    let hy = terrain_height_at(x, y + 1.0);
    let dx = vec3<f32>(1.0, 0.0, hx - hc);
    let dy = vec3<f32>(0.0, 1.0, hy - hc);
    return normalize(cross(dx, dy));
}

fn intersect_terrain_march(ray: Ray, max_t_val: f32) -> HitRecord {
    var hit: HitRecord;
    hit.t = -1.0;

    if u.has_terrain == 0u { return hit; }

    let step = 0.5;
    let max_steps = min(u32(max_t_val / step), 2000u);

    var t = 0.1;
    var prev_pos = ray.origin + ray.direction * t;
    var prev_h = terrain_height_at(prev_pos.x, prev_pos.y);

    let side = f32(u.terrain_side);

    for (var i = 0u; i < max_steps; i++) {
        t += step;
        if t > max_t_val { break; }

        let pos = ray.origin + ray.direction * t;
        let h = terrain_height_at(pos.x, pos.y);

        let lx = pos.x - u.terrain_origin.x;
        let ly = pos.y - u.terrain_origin.y;
        if lx < 0.0 || ly < 0.0 || lx >= side || ly >= side {
            prev_pos = pos;
            prev_h = h;
            continue;
        }

        if prev_pos.z >= prev_h && pos.z < h {
            var lo = t - step;
            var hi = t;
            for (var j = 0; j < 16; j++) {
                let mid = (lo + hi) * 0.5;
                let mid_pos = ray.origin + ray.direction * mid;
                let mid_h = terrain_height_at(mid_pos.x, mid_pos.y);
                if mid_pos.z > mid_h { lo = mid; } else { hi = mid; }
            }

            let hit_t = (lo + hi) * 0.5;
            let hit_pos = ray.origin + ray.direction * hit_t;

            hit.t = hit_t;
            hit.position = hit_pos;
            hit.normal = terrain_normal_at(hit_pos.x, hit_pos.y);
            hit.prim_index = -1;
            hit.is_terrain = 1u;
            return hit;
        }

        prev_pos = pos;
        prev_h = h;
    }

    return hit;
}

fn terrain_color(height: f32) -> vec3<f32> {
    if height < 0.5 { return vec3<f32>(0.7, 0.65, 0.5); }
    if height < 15.0 { return vec3<f32>(0.65, 0.6, 0.4); }
    if height < 30.0 { return vec3<f32>(0.3, 0.55, 0.2); }
    if height < 60.0 { return vec3<f32>(0.2, 0.4, 0.15); }
    if height < 100.0 { return vec3<f32>(0.5, 0.45, 0.35); }
    return vec3<f32>(0.9, 0.9, 0.92);
}

fn sky_color_fn(ray: Ray) -> vec3<f32> {
    let t = ray.direction.z * 0.5 + 0.5;
    let horizon = vec3<f32>(0.85, 0.9, 0.95);
    let zenith = vec3<f32>(0.35, 0.55, 0.85);
    return mix(horizon, zenith, clamp(t, 0.0, 1.0));
}

fn evaluate_light(light: GpuLight, hit_pos: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    var light_dir: vec3<f32>;
    var attenuation = 1.0;

    switch light.light_type {
        case 2u: {
            light_dir = normalize(-light.direction);
        }
        default: {
            light_dir = light.position - hit_pos;
            let dist = length(light_dir);
            light_dir = light_dir / max(dist, 1e-6);
            attenuation = 1.0 / (1.0 + dist * dist * 0.01);

            if light.radius < 9999.0 {
                attenuation *= max(0.0, 1.0 - dist / light.radius);
            }

            if light.light_type == 1u {
                let spot_cos = dot(-light_dir, normalize(light.direction));
                let angle_cos = cos(radians(light.spot_angle * 0.5));
                if spot_cos < angle_cos { return vec3<f32>(0.0); }
                let soft_cos = cos(radians(max(0.0, light.spot_angle * 0.5 - light.soft_edge)));
                attenuation *= smoothstep(angle_cos, soft_cos, spot_cos);
            }
        }
    }

    let ndl = max(dot(normal, light_dir), 0.0);
    return light.color * light.intensity * ndl * attenuation;
}

fn is_in_shadow(hit_pos: vec3<f32>, light_dir: vec3<f32>, max_dist: f32) -> bool {
    let shadow_ray = Ray(hit_pos + light_dir * 0.01, light_dir);
    let shadow_hit = traverse_bvh(shadow_ray, max_dist);
    return shadow_hit.t > 0.0;
}

fn shade_prim(hit: HitRecord, ray: Ray, prim: GpuPrim) -> vec3<f32> {
    let base_color = prim.color.rgb;

    if prim.fullbright == 1u {
        return base_color;
    }

    var total_light = u.ambient_color * u.ambient_intensity;

    for (var i = 0u; i < u.num_lights; i++) {
        let light = lights[i];
        let contribution = evaluate_light(light, hit.position, hit.normal);

        if length(contribution) > 0.001 {
            var light_dir: vec3<f32>;
            var shadow_dist = 500.0;

            if light.light_type == 2u {
                light_dir = normalize(-light.direction);
            } else {
                light_dir = normalize(light.position - hit.position);
                shadow_dist = length(light.position - hit.position);
            }

            if !is_in_shadow(hit.position, light_dir, shadow_dist) {
                total_light += contribution;

                if prim.shininess > 0u {
                    let view_dir = normalize(ray.origin - hit.position);
                    let half_vec = normalize(light_dir + view_dir);
                    let spec = pow(max(dot(hit.normal, half_vec), 0.0), f32(prim.shininess) * 32.0 + 8.0);
                    total_light += light.color * spec * 0.3;
                }
            }
        }
    }

    return base_color * total_light;
}

fn shade_terrain_hit(hit: HitRecord, ray: Ray) -> vec3<f32> {
    let base_color = terrain_color(hit.position.z);

    var total_light = u.ambient_color * u.ambient_intensity;

    for (var i = 0u; i < u.num_lights; i++) {
        let light = lights[i];
        let contribution = evaluate_light(light, hit.position, hit.normal);

        if length(contribution) > 0.001 {
            var light_dir: vec3<f32>;
            var shadow_dist = 500.0;

            if light.light_type == 2u {
                light_dir = normalize(-light.direction);
            } else {
                light_dir = normalize(light.position - hit.position);
                shadow_dist = length(light.position - hit.position);
            }

            if !is_in_shadow(hit.position, light_dir, shadow_dist) {
                total_light += contribution;
            }
        }
    }

    return base_color * total_light;
}

fn trace_ray(ray: Ray) -> vec3<f32> {
    let prim_hit = traverse_bvh(ray, u.max_t);
    let terrain_hit = intersect_terrain_march(ray, u.max_t);

    var use_prim = prim_hit.t > 0.0;
    var use_terrain = terrain_hit.t > 0.0;

    if use_prim && use_terrain {
        if prim_hit.t <= terrain_hit.t {
            use_terrain = false;
        } else {
            use_prim = false;
        }
    }

    if use_terrain {
        return shade_terrain_hit(terrain_hit, ray);
    }

    if use_prim {
        let prim = prims[prim_hit.prim_index];

        if prim.alpha < 0.01 {
            return sky_color_fn(ray);
        }

        var surface_color = shade_prim(prim_hit, ray, prim);

        if prim.alpha < 0.99 {
            let bg = sky_color_fn(ray);
            let a = prim.alpha;
            surface_color = surface_color * a + bg * (1.0 - a);
        }

        if prim.glow > 0.0 {
            let glow_boost = prim.glow * 0.5;
            surface_color = min(surface_color + vec3<f32>(glow_boost), vec3<f32>(1.0));
        }

        return surface_color;
    }

    return sky_color_fn(ray);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;

    if x >= u.width || y >= u.height { return; }

    var rng = y * 2654435761u + x * 1013904223u + 1u;

    var color_accum = vec3<f32>(0.0);
    let spp = u.spp;

    for (var s = 0u; s < spp; s++) {
        var jx = 0.0;
        var jy = 0.0;
        if spp > 1u {
            rng = xorshift32(rng);
            jx = f32(rng) / 4294967295.0 - 0.5;
            rng = xorshift32(rng);
            jy = f32(rng) / 4294967295.0 - 0.5;
        }

        let pu = (f32(x) + 0.5 + jx) / f32(u.width);
        let pv = 1.0 - (f32(y) + 0.5 + jy) / f32(u.height);

        var ray: Ray;
        if u.use_dof == 1u {
            ray = generate_dof_ray(pu, pv, &rng);
        } else {
            ray = generate_ray(pu, pv);
        }

        color_accum += trace_ray(ray);
    }

    let inv_spp = 1.0 / f32(spp);
    let final_color = clamp(color_accum * inv_spp, vec3<f32>(0.0), vec3<f32>(1.0));

    let r = u32(final_color.r * 255.0);
    let g = u32(final_color.g * 255.0);
    let b = u32(final_color.b * 255.0);
    let a = 255u;

    let packed = r | (g << 8u) | (b << 16u) | (a << 24u);
    output[y * u.width + x] = packed;
}
