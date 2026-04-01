const std = @import("std");
const interface = @import("../interface.zig");
const v3 = @import("vec3.zig");
const q = @import("quat.zig");

pub const Vec3 = interface.Vec3;
pub const Quat = interface.Quat;

pub const ShapeType = enum(u8) {
    capsule = 0,
    box = 1,
    sphere = 2,
    cylinder = 3,
    heightfield = 4,
    trimesh = 5,
    convex_hull = 6,
};

pub const Capsule = struct {
    radius: f32,
    half_height: f32,
};

pub const Box = struct {
    half_extents: Vec3,
};

pub const Sphere = struct {
    radius: f32,
};

pub const Cylinder = struct {
    radius: f32,
    half_height: f32,
};

pub const Heightfield = struct {
    heights: [*]const f32,
    width: u32,
    depth: u32,
    scale_x: f32,
    scale_z: f32,
};

pub const TriMesh = struct {
    vertices: [*]const f32,
    vertex_count: u32,
    indices: [*]const u32,
    index_count: u32,
};

pub const ConvexHull = struct {
    vertices: [*]const f32,
    vertex_count: u32,
};

pub const ShapeData = union(ShapeType) {
    capsule: Capsule,
    box: Box,
    sphere: Sphere,
    cylinder: Cylinder,
    heightfield: Heightfield,
    trimesh: TriMesh,
    convex_hull: ConvexHull,
};

pub const CollisionBody = struct {
    local_id: u32,
    position: Vec3,
    rotation: Quat,
    shape: ShapeData,
    flags: u32,
    is_avatar: bool,

    pub const FLAG_SOLID: u32 = 0x02;
    pub const FLAG_AVATAR: u32 = 0x04;
    pub const FLAG_PHANTOM: u32 = 0x1000;
};

pub const Contact = struct {
    point: Vec3,
    normal: Vec3,
    depth: f32,
    body_id_a: u32,
    body_id_b: u32,
};

pub const AABB = struct {
    min: Vec3,
    max: Vec3,

    pub fn overlaps(a: AABB, b: AABB) bool {
        return a.min.x <= b.max.x and a.max.x >= b.min.x and
            a.min.y <= b.max.y and a.max.y >= b.min.y and
            a.min.z <= b.max.z and a.max.z >= b.min.z;
    }

    pub fn center(self: AABB) Vec3 {
        return v3.scale(v3.add(self.min, self.max), 0.5);
    }

    pub fn halfSize(self: AABB) Vec3 {
        return v3.scale(v3.sub(self.max, self.min), 0.5);
    }
};

pub fn computeAABB(body: *const CollisionBody) AABB {
    switch (body.shape) {
        .sphere => |s| {
            const r = Vec3{ .x = s.radius, .y = s.radius, .z = s.radius };
            return .{ .min = v3.sub(body.position, r), .max = v3.add(body.position, r) };
        },
        .capsule => |c| {
            const ext = c.radius + c.half_height;
            const r = Vec3{ .x = c.radius, .y = c.radius, .z = ext };
            return .{ .min = v3.sub(body.position, r), .max = v3.add(body.position, r) };
        },
        .box => |b| {
            const mat = q.toMat3(body.rotation);
            const ax = q.mat3MulVec3(mat, .{ .x = b.half_extents.x, .y = 0, .z = 0 });
            const ay = q.mat3MulVec3(mat, .{ .x = 0, .y = b.half_extents.y, .z = 0 });
            const az = q.mat3MulVec3(mat, .{ .x = 0, .y = 0, .z = b.half_extents.z });
            const ext = Vec3{
                .x = @abs(ax.x) + @abs(ay.x) + @abs(az.x),
                .y = @abs(ax.y) + @abs(ay.y) + @abs(az.y),
                .z = @abs(ax.z) + @abs(ay.z) + @abs(az.z),
            };
            return .{ .min = v3.sub(body.position, ext), .max = v3.add(body.position, ext) };
        },
        .cylinder => |cyl| {
            const ext = @max(cyl.radius, cyl.half_height);
            const r = Vec3{ .x = ext, .y = ext, .z = ext };
            return .{ .min = v3.sub(body.position, r), .max = v3.add(body.position, r) };
        },
        .heightfield => {
            return .{
                .min = .{ .x = 0, .y = 0, .z = -1000 },
                .max = .{ .x = 256, .y = 256, .z = 4096 },
            };
        },
        .trimesh => |tm| {
            var bmin = Vec3{ .x = std.math.inf(f32), .y = std.math.inf(f32), .z = std.math.inf(f32) };
            var bmax = Vec3{ .x = -std.math.inf(f32), .y = -std.math.inf(f32), .z = -std.math.inf(f32) };
            var i: u32 = 0;
            while (i < tm.vertex_count) : (i += 1) {
                const lv = Vec3{ .x = tm.vertices[i * 3], .y = tm.vertices[i * 3 + 1], .z = tm.vertices[i * 3 + 2] };
                const wv = v3.add(body.position, q.rotateVec3(body.rotation, lv));
                bmin = v3.min3(bmin, wv);
                bmax = v3.max3(bmax, wv);
            }
            return .{ .min = bmin, .max = bmax };
        },
        .convex_hull => |ch| {
            var bmin = Vec3{ .x = std.math.inf(f32), .y = std.math.inf(f32), .z = std.math.inf(f32) };
            var bmax = Vec3{ .x = -std.math.inf(f32), .y = -std.math.inf(f32), .z = -std.math.inf(f32) };
            var i: u32 = 0;
            while (i < ch.vertex_count) : (i += 1) {
                const lv = Vec3{ .x = ch.vertices[i * 3], .y = ch.vertices[i * 3 + 1], .z = ch.vertices[i * 3 + 2] };
                const wv = v3.add(body.position, q.rotateVec3(body.rotation, lv));
                bmin = v3.min3(bmin, wv);
                bmax = v3.max3(bmax, wv);
            }
            return .{ .min = bmin, .max = bmax };
        },
    }
}

pub fn supportPoint(body: *const CollisionBody, dir_world: Vec3) Vec3 {
    const dir_local = q.inverseRotateVec3(body.rotation, dir_world);
    const local_support = supportPointLocal(body.shape, dir_local);
    return v3.add(body.position, q.rotateVec3(body.rotation, local_support));
}

fn supportPointLocal(shape: ShapeData, dir: Vec3) Vec3 {
    switch (shape) {
        .sphere => |s| {
            const n = v3.normalize(dir);
            return v3.scale(n, s.radius);
        },
        .capsule => |c| {
            const n = v3.normalize(dir);
            const tip: Vec3 = if (dir.z >= 0) .{ .x = 0, .y = 0, .z = c.half_height } else .{ .x = 0, .y = 0, .z = -c.half_height };
            return v3.add(tip, v3.scale(n, c.radius));
        },
        .box => |b| {
            return .{
                .x = if (dir.x >= 0) b.half_extents.x else -b.half_extents.x,
                .y = if (dir.y >= 0) b.half_extents.y else -b.half_extents.y,
                .z = if (dir.z >= 0) b.half_extents.z else -b.half_extents.z,
            };
        },
        .cylinder => |cyl| {
            const horiz_len = @sqrt(dir.x * dir.x + dir.y * dir.y);
            var sx: f32 = 0;
            var sy: f32 = 0;
            if (horiz_len > 1e-8) {
                sx = dir.x / horiz_len * cyl.radius;
                sy = dir.y / horiz_len * cyl.radius;
            }
            const sz: f32 = if (dir.z >= 0) cyl.half_height else -cyl.half_height;
            return .{ .x = sx, .y = sy, .z = sz };
        },
        .convex_hull => |ch| {
            var best_dot: f32 = -std.math.inf(f32);
            var best = Vec3{ .x = 0, .y = 0, .z = 0 };
            var i: u32 = 0;
            while (i < ch.vertex_count) : (i += 1) {
                const vert = Vec3{ .x = ch.vertices[i * 3], .y = ch.vertices[i * 3 + 1], .z = ch.vertices[i * 3 + 2] };
                const d = v3.dot(vert, dir);
                if (d > best_dot) {
                    best_dot = d;
                    best = vert;
                }
            }
            return best;
        },
        .trimesh => |tm| {
            var best_dot: f32 = -std.math.inf(f32);
            var best = Vec3{ .x = 0, .y = 0, .z = 0 };
            var i: u32 = 0;
            while (i < tm.vertex_count) : (i += 1) {
                const vert = Vec3{ .x = tm.vertices[i * 3], .y = tm.vertices[i * 3 + 1], .z = tm.vertices[i * 3 + 2] };
                const d = v3.dot(vert, dir);
                if (d > best_dot) {
                    best_dot = d;
                    best = vert;
                }
            }
            return best;
        },
        .heightfield => {
            return v3.zero;
        },
    }
}

test "aabb overlap" {
    const a = AABB{ .min = .{ .x = 0, .y = 0, .z = 0 }, .max = .{ .x = 2, .y = 2, .z = 2 } };
    const b = AABB{ .min = .{ .x = 1, .y = 1, .z = 1 }, .max = .{ .x = 3, .y = 3, .z = 3 } };
    const c = AABB{ .min = .{ .x = 5, .y = 5, .z = 5 }, .max = .{ .x = 6, .y = 6, .z = 6 } };
    try std.testing.expect(a.overlaps(b));
    try std.testing.expect(!a.overlaps(c));
}

test "support point sphere" {
    const body = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 0, .y = 0, .z = 0 },
        .rotation = q.identity,
        .shape = .{ .sphere = .{ .radius = 2.0 } },
        .flags = 0,
        .is_avatar = false,
    };
    const sp = supportPoint(&body, .{ .x = 1, .y = 0, .z = 0 });
    try std.testing.expectApproxEqAbs(sp.x, 2.0, 1e-5);
    try std.testing.expectApproxEqAbs(sp.y, 0.0, 1e-5);
    try std.testing.expectApproxEqAbs(sp.z, 0.0, 1e-5);
}

test "support point box identity" {
    const body = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 0, .y = 0, .z = 0 },
        .rotation = q.identity,
        .shape = .{ .box = .{ .half_extents = .{ .x = 1, .y = 2, .z = 3 } } },
        .flags = 0,
        .is_avatar = false,
    };
    const sp = supportPoint(&body, .{ .x = 1, .y = 1, .z = 1 });
    try std.testing.expectApproxEqAbs(sp.x, 1.0, 1e-5);
    try std.testing.expectApproxEqAbs(sp.y, 2.0, 1e-5);
    try std.testing.expectApproxEqAbs(sp.z, 3.0, 1e-5);
}

test "support point capsule up" {
    const body = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 0, .y = 0, .z = 5 },
        .rotation = q.identity,
        .shape = .{ .capsule = .{ .radius = 0.37, .half_height = 0.53 } },
        .flags = 0,
        .is_avatar = true,
    };
    const sp = supportPoint(&body, v3.up);
    try std.testing.expectApproxEqAbs(sp.z, 5.0 + 0.53 + 0.37, 1e-4);
}

test "aabb rotated box" {
    const body = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 0, .y = 0, .z = 0 },
        .rotation = q.fromAxisAngle(v3.up, std.math.pi / 4.0),
        .shape = .{ .box = .{ .half_extents = .{ .x = 1, .y = 0, .z = 0 } } },
        .flags = 0,
        .is_avatar = false,
    };
    const aabb = computeAABB(&body);
    const diag = @sqrt(2.0) / 2.0;
    try std.testing.expectApproxEqAbs(aabb.max.x, diag, 0.01);
    try std.testing.expectApproxEqAbs(aabb.max.y, diag, 0.01);
}
