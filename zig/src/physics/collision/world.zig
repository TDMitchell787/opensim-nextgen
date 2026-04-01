const std = @import("std");
const v3 = @import("vec3.zig");
const q = @import("quat.zig");
const shapes = @import("shapes.zig");
const gjk_mod = @import("gjk.zig");
const epa_mod = @import("epa.zig");

const Vec3 = v3.Vec3;
const Quat = q.Quat;
const CollisionBody = shapes.CollisionBody;
const Contact = shapes.Contact;
const AABB = shapes.AABB;

const MAX_CONTACTS_PER_STEP = 256;

pub const CollisionWorld = struct {
    bodies: std.AutoHashMap(u32, CollisionBody),
    terrain_heights: ?[]f32,
    terrain_width: u32,
    terrain_depth: u32,
    contacts: [MAX_CONTACTS_PER_STEP]Contact,
    contact_count: u32,
    region_id_hi: u64,
    region_id_lo: u64,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, region_hi: u64, region_lo: u64) CollisionWorld {
        return .{
            .bodies = std.AutoHashMap(u32, CollisionBody).init(allocator),
            .terrain_heights = null,
            .terrain_width = 0,
            .terrain_depth = 0,
            .contacts = undefined,
            .contact_count = 0,
            .region_id_hi = region_hi,
            .region_id_lo = region_lo,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *CollisionWorld) void {
        self.bodies.deinit();
        if (self.terrain_heights) |h| {
            self.allocator.free(h);
            self.terrain_heights = null;
        }
    }

    pub fn addBody(self: *CollisionWorld, body: CollisionBody) void {
        self.bodies.put(body.local_id, body) catch {};
    }

    pub fn removeBody(self: *CollisionWorld, local_id: u32) void {
        _ = self.bodies.remove(local_id);
    }

    pub fn updateTransform(self: *CollisionWorld, local_id: u32, pos: Vec3, rot: Quat) void {
        if (self.bodies.getPtr(local_id)) |body| {
            body.position = pos;
            body.rotation = rot;
        }
    }

    pub fn setTerrain(self: *CollisionWorld, heights: [*]const f32, width: u32, depth: u32) bool {
        const count = @as(usize, width) * @as(usize, depth);
        if (self.terrain_heights) |old| {
            self.allocator.free(old);
        }
        const buf = self.allocator.alloc(f32, count) catch return false;
        @memcpy(buf, heights[0..count]);
        self.terrain_heights = buf;
        self.terrain_width = width;
        self.terrain_depth = depth;
        return true;
    }

    pub fn getTerrainHeight(self: *const CollisionWorld, x: f32, y: f32) f32 {
        const h = self.terrain_heights orelse return 0;
        if (self.terrain_width == 0 or self.terrain_depth == 0) return 0;
        const fx = std.math.clamp(x, 0, @as(f32, @floatFromInt(self.terrain_width - 1)));
        const fy = std.math.clamp(y, 0, @as(f32, @floatFromInt(self.terrain_depth - 1)));
        const ix: u32 = @intFromFloat(fx);
        const iy: u32 = @intFromFloat(fy);
        const ix1 = @min(ix + 1, self.terrain_width - 1);
        const iy1 = @min(iy + 1, self.terrain_depth - 1);
        const dx = fx - @as(f32, @floatFromInt(ix));
        const dy = fy - @as(f32, @floatFromInt(iy));
        const w = self.terrain_width;
        const h00 = h[@as(usize, iy) * @as(usize, w) + @as(usize, ix)];
        const h10 = h[@as(usize, iy) * @as(usize, w) + @as(usize, ix1)];
        const h01 = h[@as(usize, iy1) * @as(usize, w) + @as(usize, ix)];
        const h11 = h[@as(usize, iy1) * @as(usize, w) + @as(usize, ix1)];
        return h00 * (1 - dx) * (1 - dy) + h10 * dx * (1 - dy) + h01 * (1 - dx) * dy + h11 * dx * dy;
    }

    pub fn step(self: *CollisionWorld) u32 {
        self.contact_count = 0;

        var avatars: [64]u32 = undefined;
        var avatar_count: u32 = 0;
        var prims: [2048]u32 = undefined;
        var prim_count: u32 = 0;

        var it = self.bodies.iterator();
        while (it.next()) |entry| {
            if (entry.value_ptr.is_avatar) {
                if (avatar_count < 64) {
                    avatars[avatar_count] = entry.key_ptr.*;
                    avatar_count += 1;
                }
            } else {
                if (prim_count < 2048) {
                    prims[prim_count] = entry.key_ptr.*;
                    prim_count += 1;
                }
            }
        }

        var ai: u32 = 0;
        while (ai < avatar_count) : (ai += 1) {
            const avatar_id = avatars[ai];
            const avatar_ptr = self.bodies.getPtr(avatar_id) orelse continue;

            self.testAvatarTerrain(avatar_ptr);

            var pi: u32 = 0;
            while (pi < prim_count) : (pi += 1) {
                const prim_id = prims[pi];
                const prim_ptr = self.bodies.getPtr(prim_id) orelse continue;

                if (prim_ptr.flags & CollisionBody.FLAG_PHANTOM != 0) continue;

                const avatar_aabb = shapes.computeAABB(avatar_ptr);
                const prim_aabb = shapes.computeAABB(prim_ptr);
                if (!avatar_aabb.overlaps(prim_aabb)) continue;

                const gjk_result = gjk_mod.gjk(avatar_ptr, prim_ptr);
                if (!gjk_result.intersecting) continue;

                if (epa_mod.epa(avatar_ptr, prim_ptr, &gjk_result.simplex)) |contact| {
                    if (self.contact_count < MAX_CONTACTS_PER_STEP) {
                        self.contacts[self.contact_count] = contact;
                        self.contact_count += 1;
                    }
                }
            }
        }

        return self.contact_count;
    }

    fn testAvatarTerrain(self: *CollisionWorld, avatar: *const CollisionBody) void {
        const terrain_h = self.getTerrainHeight(avatar.position.x, avatar.position.y);
        const capsule = switch (avatar.shape) {
            .capsule => |c| c,
            else => return,
        };
        const foot_z = avatar.position.z - capsule.half_height - capsule.radius;
        if (foot_z < terrain_h) {
            const depth = terrain_h - foot_z;
            if (self.contact_count < MAX_CONTACTS_PER_STEP) {
                self.contacts[self.contact_count] = .{
                    .point = .{ .x = avatar.position.x, .y = avatar.position.y, .z = terrain_h },
                    .normal = .{ .x = 0, .y = 0, .z = 1 },
                    .depth = depth,
                    .body_id_a = avatar.local_id,
                    .body_id_b = 0,
                };
                self.contact_count += 1;
            }
        }
    }

    pub fn getContact(self: *const CollisionWorld, index: u32) ?Contact {
        if (index >= self.contact_count) return null;
        return self.contacts[index];
    }

    pub const RayHit = struct {
        body_id: u32,
        t: f32,
        point: Vec3,
        normal: Vec3,
    };

    pub fn castRayDown(self: *const CollisionWorld, origin: Vec3, max_dist: f32) ?RayHit {
        var best: ?RayHit = null;
        var best_t: f32 = max_dist;

        const ray_end_z = origin.z - max_dist;
        const ray_aabb = AABB{
            .min = .{ .x = origin.x - 0.01, .y = origin.y - 0.01, .z = ray_end_z },
            .max = .{ .x = origin.x + 0.01, .y = origin.y + 0.01, .z = origin.z },
        };

        var it = self.bodies.iterator();
        while (it.next()) |entry| {
            const body = entry.value_ptr;
            if (body.is_avatar) continue;
            if (body.flags & CollisionBody.FLAG_PHANTOM != 0) continue;

            const prim_aabb = shapes.computeAABB(body);
            if (!ray_aabb.overlaps(prim_aabb)) continue;

            if (rayVsBody(origin, max_dist, body)) |hit| {
                if (hit.t < best_t) {
                    best_t = hit.t;
                    best = .{
                        .body_id = body.local_id,
                        .t = hit.t,
                        .point = hit.point,
                        .normal = hit.normal,
                    };
                }
            }
        }

        return best;
    }

    const ShapeHit = struct { t: f32, point: Vec3, normal: Vec3 };

    fn rayVsBody(origin: Vec3, max_dist: f32, body: *const CollisionBody) ?ShapeHit {
        switch (body.shape) {
            .box => |b| return rayVsOBB(origin, max_dist, body.position, body.rotation, b.half_extents),
            .sphere => |s| return rayVsSphere(origin, max_dist, body.position, s.radius),
            .cylinder => |cyl| return rayVsCylinder(origin, max_dist, body.position, body.rotation, cyl.radius, cyl.half_height),
            .capsule => |c| return rayVsCapsule(origin, max_dist, body.position, c.radius, c.half_height),
            else => {
                const aabb = shapes.computeAABB(body);
                return rayVsAABB(origin, max_dist, aabb);
            },
        }
    }

    fn rayVsOBB(origin: Vec3, max_dist: f32, pos: Vec3, rot: Quat, half: Vec3) ?ShapeHit {
        const local_origin = q.inverseRotateVec3(rot, v3.sub(origin, pos));
        const local_dir = q.inverseRotateVec3(rot, Vec3{ .x = 0, .y = 0, .z = -1 });

        var t_min: f32 = 0;
        var t_max: f32 = max_dist;
        var hit_normal_local = Vec3{ .x = 0, .y = 0, .z = 1 };

        const axes = [3]f32{ local_origin.x, local_origin.y, local_origin.z };
        const dirs = [3]f32{ local_dir.x, local_dir.y, local_dir.z };
        const halves = [3]f32{ half.x, half.y, half.z };
        const normals = [3]Vec3{
            .{ .x = 1, .y = 0, .z = 0 },
            .{ .x = 0, .y = 1, .z = 0 },
            .{ .x = 0, .y = 0, .z = 1 },
        };

        for (0..3) |i| {
            if (@abs(dirs[i]) < 1e-8) {
                if (axes[i] < -halves[i] or axes[i] > halves[i]) return null;
            } else {
                const inv_d = 1.0 / dirs[i];
                var t1 = (-halves[i] - axes[i]) * inv_d;
                var t2 = (halves[i] - axes[i]) * inv_d;
                var face_normal = v3.negate(normals[i]);
                if (t1 > t2) {
                    const tmp = t1;
                    t1 = t2;
                    t2 = tmp;
                    face_normal = normals[i];
                }
                if (t1 > t_min) {
                    t_min = t1;
                    hit_normal_local = face_normal;
                }
                if (t2 < t_max) t_max = t2;
                if (t_min > t_max) return null;
            }
        }

        if (t_min < 0.001 or t_min > max_dist) return null;

        const local_hit = v3.add(local_origin, v3.scale(local_dir, t_min));
        const world_hit = v3.add(pos, q.rotateVec3(rot, local_hit));
        const world_normal = q.rotateVec3(rot, hit_normal_local);

        return .{ .t = t_min, .point = world_hit, .normal = world_normal };
    }

    fn rayVsAABB(origin: Vec3, max_dist: f32, aabb: AABB) ?ShapeHit {
        const dir_z: f32 = -1.0;
        if (origin.x < aabb.min.x or origin.x > aabb.max.x) return null;
        if (origin.y < aabb.min.y or origin.y > aabb.max.y) return null;

        const t1 = (aabb.max.z - origin.z) / dir_z;
        const t2 = (aabb.min.z - origin.z) / dir_z;
        const t_near = @min(t1, t2);
        const t_far = @max(t1, t2);
        if (t_near > max_dist or t_far < 0) return null;
        const t = if (t_near >= 0) t_near else t_far;
        if (t < 0.001 or t > max_dist) return null;

        return .{
            .t = t,
            .point = .{ .x = origin.x, .y = origin.y, .z = origin.z - t },
            .normal = .{ .x = 0, .y = 0, .z = 1 },
        };
    }

    fn rayVsSphere(origin: Vec3, max_dist: f32, center: Vec3, radius: f32) ?ShapeHit {
        const oc = v3.sub(origin, center);
        const dir = Vec3{ .x = 0, .y = 0, .z = -1 };
        const a = v3.dot(dir, dir);
        const b = 2.0 * v3.dot(oc, dir);
        const c = v3.dot(oc, oc) - radius * radius;
        const disc = b * b - 4.0 * a * c;
        if (disc < 0) return null;
        const sq = @sqrt(disc);
        const t = (-b - sq) / (2.0 * a);
        if (t < 0.001 or t > max_dist) return null;
        const hit = v3.add(origin, v3.scale(dir, t));
        const n = v3.normalize(v3.sub(hit, center));
        return .{ .t = t, .point = hit, .normal = n };
    }

    fn rayVsCylinder(origin: Vec3, max_dist: f32, pos: Vec3, rot: Quat, radius: f32, hh: f32) ?ShapeHit {
        _ = rot;
        const oc = v3.sub(origin, pos);
        const a = 1.0;
        const b_val = -2.0 * oc.z;
        _ = b_val;
        const dx = oc.x;
        const dy = oc.y;
        const a2 = dx * dx + dy * dy;
        if (a2 > (radius + 0.01) * (radius + 0.01)) return null;
        _ = a;

        const t_top = (hh - oc.z);
        if (t_top >= 0 and t_top <= max_dist) {
            const hx = origin.x;
            const hy = origin.y;
            const dist_sq = (hx - pos.x) * (hx - pos.x) + (hy - pos.y) * (hy - pos.y);
            if (dist_sq <= radius * radius) {
                return .{
                    .t = t_top,
                    .point = .{ .x = origin.x, .y = origin.y, .z = pos.z + hh },
                    .normal = .{ .x = 0, .y = 0, .z = 1 },
                };
            }
        }
        return null;
    }

    fn rayVsCapsule(origin: Vec3, max_dist: f32, pos: Vec3, radius: f32, hh: f32) ?ShapeHit {
        _ = hh;
        return rayVsSphere(origin, max_dist, pos, radius);
    }
};

pub const WorldManager = struct {
    worlds: std.AutoHashMap(u128, *CollisionWorld),
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) WorldManager {
        return .{
            .worlds = std.AutoHashMap(u128, *CollisionWorld).init(allocator),
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *WorldManager) void {
        var it = self.worlds.iterator();
        while (it.next()) |entry| {
            entry.value_ptr.*.deinit();
            self.allocator.destroy(entry.value_ptr.*);
        }
        self.worlds.deinit();
    }

    pub fn createWorld(self: *WorldManager, region_hi: u64, region_lo: u64) ?*CollisionWorld {
        const key = makeKey(region_hi, region_lo);
        if (self.worlds.get(key)) |existing| return existing;
        const world = self.allocator.create(CollisionWorld) catch return null;
        world.* = CollisionWorld.init(self.allocator, region_hi, region_lo);
        self.worlds.put(key, world) catch {
            world.deinit();
            self.allocator.destroy(world);
            return null;
        };
        return world;
    }

    pub fn destroyWorld(self: *WorldManager, region_hi: u64, region_lo: u64) void {
        const key = makeKey(region_hi, region_lo);
        if (self.worlds.fetchRemove(key)) |kv| {
            kv.value.deinit();
            self.allocator.destroy(kv.value);
        }
    }

    pub fn getWorld(self: *WorldManager, region_hi: u64, region_lo: u64) ?*CollisionWorld {
        return self.worlds.get(makeKey(region_hi, region_lo));
    }

    fn makeKey(hi: u64, lo: u64) u128 {
        return (@as(u128, hi) << 64) | @as(u128, lo);
    }
};

var global_world_manager: ?*WorldManager = null;

pub fn getGlobalWorldManager() ?*WorldManager {
    return global_world_manager;
}

pub fn initGlobalWorldManager(allocator: std.mem.Allocator) bool {
    if (global_world_manager != null) return true;
    const wm = allocator.create(WorldManager) catch return false;
    wm.* = WorldManager.init(allocator);
    global_world_manager = wm;
    return true;
}

pub fn deinitGlobalWorldManager(allocator: std.mem.Allocator) void {
    if (global_world_manager) |wm| {
        wm.deinit();
        allocator.destroy(wm);
        global_world_manager = null;
    }
}

test "collision world basic" {
    const allocator = std.testing.allocator;
    var world = CollisionWorld.init(allocator, 0, 1);
    defer world.deinit();

    const platform = CollisionBody{
        .local_id = 100,
        .position = .{ .x = 128, .y = 128, .z = 50 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .box = .{ .half_extents = .{ .x = 5, .y = 5, .z = 0.25 } } },
        .flags = CollisionBody.FLAG_SOLID,
        .is_avatar = false,
    };
    world.addBody(platform);

    const avatar = CollisionBody{
        .local_id = 200,
        .position = .{ .x = 128, .y = 128, .z = 50.8 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .capsule = .{ .radius = 0.37, .half_height = 0.53 } },
        .flags = CollisionBody.FLAG_AVATAR,
        .is_avatar = true,
    };
    world.addBody(avatar);

    const num = world.step();
    try std.testing.expect(num >= 1);

    const contact = world.getContact(0);
    try std.testing.expect(contact != null);
}

test "collision world phantom skip" {
    const allocator = std.testing.allocator;
    var world = CollisionWorld.init(allocator, 0, 2);
    defer world.deinit();

    const phantom = CollisionBody{
        .local_id = 100,
        .position = .{ .x = 128, .y = 128, .z = 50 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .box = .{ .half_extents = .{ .x = 5, .y = 5, .z = 0.25 } } },
        .flags = CollisionBody.FLAG_PHANTOM,
        .is_avatar = false,
    };
    world.addBody(phantom);

    const avatar = CollisionBody{
        .local_id = 200,
        .position = .{ .x = 128, .y = 128, .z = 50.5 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .capsule = .{ .radius = 0.37, .half_height = 0.53 } },
        .flags = CollisionBody.FLAG_AVATAR,
        .is_avatar = true,
    };
    world.addBody(avatar);

    const num = world.step();
    try std.testing.expect(num == 0);
}

test "collision world terrain" {
    const allocator = std.testing.allocator;
    var world = CollisionWorld.init(allocator, 0, 3);
    defer world.deinit();

    var heights: [4]f32 = .{ 25.0, 25.0, 25.0, 25.0 };
    _ = world.setTerrain(&heights, 2, 2);

    const avatar = CollisionBody{
        .local_id = 200,
        .position = .{ .x = 0.5, .y = 0.5, .z = 25.5 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .capsule = .{ .radius = 0.37, .half_height = 0.53 } },
        .flags = CollisionBody.FLAG_AVATAR,
        .is_avatar = true,
    };
    world.addBody(avatar);

    const num = world.step();
    try std.testing.expect(num >= 1);
    const c = world.getContact(0);
    try std.testing.expect(c != null);
    if (c) |contact| {
        try std.testing.expectApproxEqAbs(contact.normal.z, 1.0, 1e-5);
        try std.testing.expect(contact.body_id_b == 0);
    }
}

test "world manager lifecycle" {
    const allocator = std.testing.allocator;
    var wm = WorldManager.init(allocator);
    defer wm.deinit();

    const w1 = wm.createWorld(0, 1);
    try std.testing.expect(w1 != null);

    const w2 = wm.createWorld(0, 2);
    try std.testing.expect(w2 != null);

    try std.testing.expect(wm.getWorld(0, 1) != null);
    try std.testing.expect(wm.getWorld(0, 99) == null);

    wm.destroyWorld(0, 1);
    try std.testing.expect(wm.getWorld(0, 1) == null);
}
