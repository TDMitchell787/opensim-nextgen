const std = @import("std");
const v3 = @import("vec3.zig");
const shapes = @import("shapes.zig");

const Vec3 = v3.Vec3;
const CollisionBody = shapes.CollisionBody;

const MAX_ITERATIONS = 64;
const TOLERANCE = 1e-6;

pub const Simplex = struct {
    points: [4]Vec3 = undefined,
    count: u32 = 0,

    pub fn push(self: *Simplex, p: Vec3) void {
        if (self.count < 4) {
            self.points[self.count] = p;
            self.count += 1;
        }
    }

    pub fn last(self: *const Simplex) Vec3 {
        return self.points[self.count - 1];
    }
};

fn minkowskiSupport(a: *const CollisionBody, b: *const CollisionBody, dir: Vec3) Vec3 {
    const sa = shapes.supportPoint(a, dir);
    const sb = shapes.supportPoint(b, v3.negate(dir));
    return v3.sub(sa, sb);
}

fn doSimplex2(simplex: *Simplex, dir: *Vec3) bool {
    const a = simplex.points[1];
    const b = simplex.points[0];
    const ab = v3.sub(b, a);
    const ao = v3.negate(a);

    if (v3.dot(ab, ao) > 0) {
        dir.* = tripleProduct(ab, ao, ab);
        if (v3.lengthSq(dir.*) < TOLERANCE) {
            dir.* = v3.cross(ab, .{ .x = 0, .y = 0, .z = 1 });
            if (v3.lengthSq(dir.*) < TOLERANCE)
                dir.* = v3.cross(ab, .{ .x = 0, .y = 1, .z = 0 });
        }
    } else {
        simplex.points[0] = a;
        simplex.count = 1;
        dir.* = ao;
    }
    return false;
}

fn doSimplex3(simplex: *Simplex, dir: *Vec3) bool {
    const a = simplex.points[2];
    const b = simplex.points[1];
    const c = simplex.points[0];
    const ab = v3.sub(b, a);
    const ac = v3.sub(c, a);
    const ao = v3.negate(a);
    const abc = v3.cross(ab, ac);

    if (v3.dot(v3.cross(abc, ac), ao) > 0) {
        if (v3.dot(ac, ao) > 0) {
            simplex.points[0] = c;
            simplex.points[1] = a;
            simplex.count = 2;
            dir.* = tripleProduct(ac, ao, ac);
        } else {
            simplex.points[0] = b;
            simplex.points[1] = a;
            simplex.count = 2;
            return doSimplex2(simplex, dir);
        }
    } else {
        if (v3.dot(v3.cross(ab, abc), ao) > 0) {
            simplex.points[0] = b;
            simplex.points[1] = a;
            simplex.count = 2;
            return doSimplex2(simplex, dir);
        } else {
            if (v3.dot(abc, ao) > 0) {
                dir.* = abc;
            } else {
                const tmp = simplex.points[0];
                simplex.points[0] = simplex.points[1];
                simplex.points[1] = tmp;
                dir.* = v3.negate(abc);
            }
        }
    }
    return false;
}

fn doSimplex4(simplex: *Simplex, dir: *Vec3) bool {
    const a = simplex.points[3];
    const b = simplex.points[2];
    const c = simplex.points[1];
    const d = simplex.points[0];
    const ab = v3.sub(b, a);
    const ac = v3.sub(c, a);
    const ad = v3.sub(d, a);
    const ao = v3.negate(a);

    const abc = v3.cross(ab, ac);
    const acd = v3.cross(ac, ad);
    const adb = v3.cross(ad, ab);

    if (v3.dot(abc, ao) > 0) {
        simplex.points[0] = c;
        simplex.points[1] = b;
        simplex.points[2] = a;
        simplex.count = 3;
        dir.* = abc;
        return false;
    }
    if (v3.dot(acd, ao) > 0) {
        simplex.points[0] = d;
        simplex.points[1] = c;
        simplex.points[2] = a;
        simplex.count = 3;
        dir.* = acd;
        return false;
    }
    if (v3.dot(adb, ao) > 0) {
        simplex.points[0] = b;
        simplex.points[1] = d;
        simplex.points[2] = a;
        simplex.count = 3;
        dir.* = adb;
        return false;
    }

    return true;
}

fn doSimplex(simplex: *Simplex, dir: *Vec3) bool {
    return switch (simplex.count) {
        2 => doSimplex2(simplex, dir),
        3 => doSimplex3(simplex, dir),
        4 => doSimplex4(simplex, dir),
        else => false,
    };
}

fn tripleProduct(a: Vec3, b: Vec3, c: Vec3) Vec3 {
    return v3.sub(v3.scale(b, v3.dot(c, a)), v3.scale(a, v3.dot(c, b)));
}

pub const GjkResult = struct {
    intersecting: bool,
    simplex: Simplex,
};

pub fn gjk(a: *const CollisionBody, b: *const CollisionBody) GjkResult {
    var dir = v3.sub(b.position, a.position);
    if (v3.lengthSq(dir) < TOLERANCE) {
        dir = .{ .x = 1, .y = 0, .z = 0 };
    }

    var simplex = Simplex{};
    simplex.push(minkowskiSupport(a, b, dir));
    dir = v3.negate(simplex.last());

    var iter: u32 = 0;
    while (iter < MAX_ITERATIONS) : (iter += 1) {
        const new_point = minkowskiSupport(a, b, dir);
        if (v3.dot(new_point, dir) < 0) {
            return .{ .intersecting = false, .simplex = simplex };
        }
        simplex.push(new_point);
        if (doSimplex(&simplex, &dir)) {
            return .{ .intersecting = true, .simplex = simplex };
        }
        if (v3.lengthSq(dir) < TOLERANCE) {
            return .{ .intersecting = true, .simplex = simplex };
        }
    }
    return .{ .intersecting = false, .simplex = simplex };
}

test "gjk separated spheres" {
    const a = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 0, .y = 0, .z = 0 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .sphere = .{ .radius = 1.0 } },
        .flags = 0,
        .is_avatar = false,
    };
    const b = CollisionBody{
        .local_id = 2,
        .position = .{ .x = 5, .y = 0, .z = 0 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .sphere = .{ .radius = 1.0 } },
        .flags = 0,
        .is_avatar = false,
    };
    const result = gjk(&a, &b);
    try std.testing.expect(!result.intersecting);
}

test "gjk overlapping spheres" {
    const a = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 0, .y = 0, .z = 0 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .sphere = .{ .radius = 1.0 } },
        .flags = 0,
        .is_avatar = false,
    };
    const b = CollisionBody{
        .local_id = 2,
        .position = .{ .x = 1.0, .y = 0, .z = 0 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .sphere = .{ .radius = 1.0 } },
        .flags = 0,
        .is_avatar = false,
    };
    const result = gjk(&a, &b);
    try std.testing.expect(result.intersecting);
}

test "gjk capsule on box" {
    const platform = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 128, .y = 128, .z = 50 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .box = .{ .half_extents = .{ .x = 5, .y = 5, .z = 0.25 } } },
        .flags = 0,
        .is_avatar = false,
    };
    const avatar = CollisionBody{
        .local_id = 2,
        .position = .{ .x = 128, .y = 128, .z = 50.8 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .capsule = .{ .radius = 0.37, .half_height = 0.53 } },
        .flags = 0,
        .is_avatar = true,
    };
    const result = gjk(&platform, &avatar);
    try std.testing.expect(result.intersecting);
}

test "gjk capsule above box" {
    const platform = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 128, .y = 128, .z = 50 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .box = .{ .half_extents = .{ .x = 5, .y = 5, .z = 0.25 } } },
        .flags = 0,
        .is_avatar = false,
    };
    const avatar = CollisionBody{
        .local_id = 2,
        .position = .{ .x = 128, .y = 128, .z = 55 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .capsule = .{ .radius = 0.37, .half_height = 0.53 } },
        .flags = 0,
        .is_avatar = true,
    };
    const result = gjk(&platform, &avatar);
    try std.testing.expect(!result.intersecting);
}

test "gjk rotated box vs capsule" {
    const q_mod = @import("quat.zig");
    const box_body = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 10, .y = 10, .z = 50 },
        .rotation = q_mod.fromAxisAngle(v3.up, std.math.pi / 4.0),
        .shape = .{ .box = .{ .half_extents = .{ .x = 3, .y = 3, .z = 0.25 } } },
        .flags = 0,
        .is_avatar = false,
    };
    const avatar = CollisionBody{
        .local_id = 2,
        .position = .{ .x = 10, .y = 10, .z = 50.5 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .capsule = .{ .radius = 0.37, .half_height = 0.53 } },
        .flags = 0,
        .is_avatar = true,
    };
    const result = gjk(&box_body, &avatar);
    try std.testing.expect(result.intersecting);
}
