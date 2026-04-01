const std = @import("std");
const math = std.math;
const interface = @import("../interface.zig");

pub const Vec3 = interface.Vec3;

pub fn add(a: Vec3, b: Vec3) Vec3 {
    return .{ .x = a.x + b.x, .y = a.y + b.y, .z = a.z + b.z };
}

pub fn sub(a: Vec3, b: Vec3) Vec3 {
    return .{ .x = a.x - b.x, .y = a.y - b.y, .z = a.z - b.z };
}

pub fn scale(v: Vec3, s: f32) Vec3 {
    return .{ .x = v.x * s, .y = v.y * s, .z = v.z * s };
}

pub fn negate(v: Vec3) Vec3 {
    return .{ .x = -v.x, .y = -v.y, .z = -v.z };
}

pub fn dot(a: Vec3, b: Vec3) f32 {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

pub fn cross(a: Vec3, b: Vec3) Vec3 {
    return .{
        .x = a.y * b.z - a.z * b.y,
        .y = a.z * b.x - a.x * b.z,
        .z = a.x * b.y - a.y * b.x,
    };
}

pub fn lengthSq(v: Vec3) f32 {
    return dot(v, v);
}

pub fn length(v: Vec3) f32 {
    return @sqrt(lengthSq(v));
}

pub fn normalize(v: Vec3) Vec3 {
    const len = length(v);
    if (len < 1e-12) return .{ .x = 0, .y = 0, .z = 0 };
    return scale(v, 1.0 / len);
}

pub fn lerp(a: Vec3, b: Vec3, t: f32) Vec3 {
    return add(scale(a, 1.0 - t), scale(b, t));
}

pub fn distanceSq(a: Vec3, b: Vec3) f32 {
    return lengthSq(sub(a, b));
}

pub fn distance(a: Vec3, b: Vec3) f32 {
    return @sqrt(distanceSq(a, b));
}

pub fn min3(a: Vec3, b: Vec3) Vec3 {
    return .{
        .x = @min(a.x, b.x),
        .y = @min(a.y, b.y),
        .z = @min(a.z, b.z),
    };
}

pub fn max3(a: Vec3, b: Vec3) Vec3 {
    return .{
        .x = @max(a.x, b.x),
        .y = @max(a.y, b.y),
        .z = @max(a.z, b.z),
    };
}

pub fn abs3(v: Vec3) Vec3 {
    return .{
        .x = @abs(v.x),
        .y = @abs(v.y),
        .z = @abs(v.z),
    };
}

pub const zero = Vec3{ .x = 0, .y = 0, .z = 0 };
pub const up = Vec3{ .x = 0, .y = 0, .z = 1 };
pub const right = Vec3{ .x = 1, .y = 0, .z = 0 };
pub const forward = Vec3{ .x = 0, .y = 1, .z = 0 };

test "vec3 basics" {
    const a = Vec3{ .x = 1, .y = 2, .z = 3 };
    const b = Vec3{ .x = 4, .y = 5, .z = 6 };

    const s = add(a, b);
    try std.testing.expectApproxEqAbs(s.x, 5.0, 1e-6);
    try std.testing.expectApproxEqAbs(s.y, 7.0, 1e-6);
    try std.testing.expectApproxEqAbs(s.z, 9.0, 1e-6);

    const d = sub(b, a);
    try std.testing.expectApproxEqAbs(d.x, 3.0, 1e-6);
    try std.testing.expectApproxEqAbs(d.y, 3.0, 1e-6);
    try std.testing.expectApproxEqAbs(d.z, 3.0, 1e-6);

    try std.testing.expectApproxEqAbs(dot(a, b), 32.0, 1e-6);

    const c = cross(a, b);
    try std.testing.expectApproxEqAbs(c.x, -3.0, 1e-6);
    try std.testing.expectApproxEqAbs(c.y, 6.0, 1e-6);
    try std.testing.expectApproxEqAbs(c.z, -3.0, 1e-6);
}

test "vec3 normalize" {
    const v = Vec3{ .x = 3, .y = 0, .z = 4 };
    const n = normalize(v);
    try std.testing.expectApproxEqAbs(length(n), 1.0, 1e-5);
    try std.testing.expectApproxEqAbs(n.x, 0.6, 1e-5);
    try std.testing.expectApproxEqAbs(n.z, 0.8, 1e-5);

    const z = normalize(zero);
    try std.testing.expectApproxEqAbs(length(z), 0.0, 1e-6);
}

test "vec3 lerp" {
    const a = Vec3{ .x = 0, .y = 0, .z = 0 };
    const b = Vec3{ .x = 10, .y = 20, .z = 30 };
    const mid = lerp(a, b, 0.5);
    try std.testing.expectApproxEqAbs(mid.x, 5.0, 1e-6);
    try std.testing.expectApproxEqAbs(mid.y, 10.0, 1e-6);
    try std.testing.expectApproxEqAbs(mid.z, 15.0, 1e-6);
}
