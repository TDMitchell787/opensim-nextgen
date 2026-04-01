const std = @import("std");
const interface = @import("../interface.zig");
const v3 = @import("vec3.zig");

pub const Quat = interface.Quat;
pub const Vec3 = interface.Vec3;

pub const identity = Quat{ .x = 0, .y = 0, .z = 0, .w = 1 };

pub fn multiply(a: Quat, b: Quat) Quat {
    return .{
        .x = a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        .y = a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        .z = a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        .w = a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    };
}

pub fn conjugate(q: Quat) Quat {
    return .{ .x = -q.x, .y = -q.y, .z = -q.z, .w = q.w };
}

pub fn normalize(q: Quat) Quat {
    const len = @sqrt(q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w);
    if (len < 1e-12) return identity;
    const inv = 1.0 / len;
    return .{ .x = q.x * inv, .y = q.y * inv, .z = q.z * inv, .w = q.w * inv };
}

pub fn rotateVec3(q: Quat, v: Vec3) Vec3 {
    const qv = Vec3{ .x = q.x, .y = q.y, .z = q.z };
    const uv = v3.cross(qv, v);
    const uuv = v3.cross(qv, uv);
    return v3.add(v, v3.add(v3.scale(uv, 2.0 * q.w), v3.scale(uuv, 2.0)));
}

pub fn inverseRotateVec3(q: Quat, v: Vec3) Vec3 {
    return rotateVec3(conjugate(q), v);
}

pub fn fromAxisAngle(axis: Vec3, angle: f32) Quat {
    const ha = angle * 0.5;
    const s = @sin(ha);
    const n = v3.normalize(axis);
    return .{ .x = n.x * s, .y = n.y * s, .z = n.z * s, .w = @cos(ha) };
}

pub const Mat3 = struct {
    m: [9]f32,

    pub fn col(self: Mat3, c: usize) Vec3 {
        return .{ .x = self.m[c * 3], .y = self.m[c * 3 + 1], .z = self.m[c * 3 + 2] };
    }
};

pub fn toMat3(q: Quat) Mat3 {
    const xx = q.x * q.x;
    const yy = q.y * q.y;
    const zz = q.z * q.z;
    const xy = q.x * q.y;
    const xz = q.x * q.z;
    const yz = q.y * q.z;
    const wx = q.w * q.x;
    const wy = q.w * q.y;
    const wz = q.w * q.z;
    return .{ .m = .{
        1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy),
        2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx),
        2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy),
    } };
}

pub fn mat3MulVec3(m: Mat3, v: Vec3) Vec3 {
    return .{
        .x = m.m[0] * v.x + m.m[3] * v.y + m.m[6] * v.z,
        .y = m.m[1] * v.x + m.m[4] * v.y + m.m[7] * v.z,
        .z = m.m[2] * v.x + m.m[5] * v.y + m.m[8] * v.z,
    };
}

pub fn mat3Transpose(m: Mat3) Mat3 {
    return .{ .m = .{
        m.m[0], m.m[3], m.m[6],
        m.m[1], m.m[4], m.m[7],
        m.m[2], m.m[5], m.m[8],
    } };
}

test "quat identity rotation" {
    const v = Vec3{ .x = 1, .y = 2, .z = 3 };
    const r = rotateVec3(identity, v);
    try std.testing.expectApproxEqAbs(r.x, 1.0, 1e-5);
    try std.testing.expectApproxEqAbs(r.y, 2.0, 1e-5);
    try std.testing.expectApproxEqAbs(r.z, 3.0, 1e-5);
}

test "quat 90deg Z rotation" {
    const q = fromAxisAngle(v3.up, std.math.pi / 2.0);
    const v = Vec3{ .x = 1, .y = 0, .z = 0 };
    const r = rotateVec3(q, v);
    try std.testing.expectApproxEqAbs(r.x, 0.0, 1e-5);
    try std.testing.expectApproxEqAbs(r.y, 1.0, 1e-5);
    try std.testing.expectApproxEqAbs(r.z, 0.0, 1e-5);
}

test "quat inverse rotation roundtrip" {
    const q = fromAxisAngle(.{ .x = 1, .y = 1, .z = 0 }, 0.7);
    const v = Vec3{ .x = 3, .y = -1, .z = 5 };
    const rotated = rotateVec3(q, v);
    const back = inverseRotateVec3(q, rotated);
    try std.testing.expectApproxEqAbs(back.x, v.x, 1e-4);
    try std.testing.expectApproxEqAbs(back.y, v.y, 1e-4);
    try std.testing.expectApproxEqAbs(back.z, v.z, 1e-4);
}

test "quat multiply associativity" {
    const a = fromAxisAngle(v3.up, 0.3);
    const b = fromAxisAngle(v3.right, 0.5);
    const c = fromAxisAngle(v3.forward, 0.7);
    const ab_c = multiply(multiply(a, b), c);
    const a_bc = multiply(a, multiply(b, c));
    try std.testing.expectApproxEqAbs(ab_c.x, a_bc.x, 1e-5);
    try std.testing.expectApproxEqAbs(ab_c.y, a_bc.y, 1e-5);
    try std.testing.expectApproxEqAbs(ab_c.z, a_bc.z, 1e-5);
    try std.testing.expectApproxEqAbs(ab_c.w, a_bc.w, 1e-5);
}

test "quat toMat3 matches rotateVec3" {
    const q = fromAxisAngle(.{ .x = 0.5, .y = 0.3, .z = 0.8 }, 1.2);
    const v = Vec3{ .x = 2, .y = -3, .z = 1 };
    const r1 = rotateVec3(q, v);
    const m = toMat3(q);
    const r2 = mat3MulVec3(m, v);
    try std.testing.expectApproxEqAbs(r1.x, r2.x, 1e-4);
    try std.testing.expectApproxEqAbs(r1.y, r2.y, 1e-4);
    try std.testing.expectApproxEqAbs(r1.z, r2.z, 1e-4);
}
