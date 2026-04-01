const std = @import("std");
const interface = @import("../interface.zig");
const Vec3 = interface.Vec3;
const Quat = interface.Quat;
const PhysicsBody = interface.PhysicsBody;
const PhysicsEngine = interface.PhysicsEngine;

const TERRAIN_SIZE = 256;
const TERMINAL_VELOCITY: f32 = 50.0;
const DEFAULT_WATER_LEVEL: f32 = 20.0;

const POSBody = struct {
    position: Vec3,
    velocity: Vec3,
    rotation: Quat,
    is_physical: bool,
    buoyancy: f32,
    is_flying: bool,
    on_ground: bool,
};

pub const POSEngine = struct {
    alloc: std.mem.Allocator,
    bodies: std.AutoHashMap(u64, *POSBody),
    next_id: u64,
    gravity: Vec3,
    water_level: f32,
    terrain: [TERRAIN_SIZE * TERRAIN_SIZE]f32,
    has_terrain: bool,

    pub fn create(alloc: std.mem.Allocator) ?*PhysicsEngine {
        const self = alloc.create(POSEngine) catch return null;
        self.* = .{
            .alloc = alloc,
            .bodies = std.AutoHashMap(u64, *POSBody).init(alloc),
            .next_id = 1,
            .gravity = Vec3{ .x = 0, .y = 0, .z = -9.81 },
            .water_level = DEFAULT_WATER_LEVEL,
            .terrain = [_]f32{0} ** (TERRAIN_SIZE * TERRAIN_SIZE),
            .has_terrain = false,
        };
        const engine = alloc.create(PhysicsEngine) catch {
            alloc.destroy(self);
            return null;
        };
        engine.* = .{
            .initFn = initFn,
            .deinitFn = deinitFn,
            .stepFn = stepFn,
            .createBodyFn = createBodyFn,
            .destroyBodyFn = destroyBodyFn,
            .getPositionFn = getPositionFn,
            .setPositionFn = setPositionFn,
            .getVelocityFn = getVelocityFn,
            .setVelocityFn = setVelocityFn,
            .getRotationFn = getRotationFn,
            .setRotationFn = setRotationFn,
            .setGravityFn = setGravityFn,
            .createTerrainFn = createTerrainFn,
            .getTerrainHeightFn = getTerrainHeightExFn,
            .setBodyFlyingFn = setBodyFlyingFn,
            .isBodyOnGroundFn = isBodyOnGroundFn,
            .raycastFn = raycastFn,
            .name = "POS",
            .engine_type = .pos,
            .impl_ptr = @ptrCast(self),
        };
        return engine;
    }

    fn getSelf(engine: *PhysicsEngine) *POSEngine {
        return @ptrCast(@alignCast(engine.impl_ptr));
    }

    fn getTerrainHeight(self: *POSEngine, x: f32, y: f32) f32 {
        if (!self.has_terrain) return 0;
        const ix = @as(u32, @intFromFloat(@max(0, @min(@as(f32, TERRAIN_SIZE - 2), x))));
        const iy = @as(u32, @intFromFloat(@max(0, @min(@as(f32, TERRAIN_SIZE - 2), y))));
        const fx = x - @as(f32, @floatFromInt(ix));
        const fy = y - @as(f32, @floatFromInt(iy));
        const h00 = self.terrain[iy * TERRAIN_SIZE + ix];
        const h10 = self.terrain[iy * TERRAIN_SIZE + ix + 1];
        const h01 = self.terrain[(iy + 1) * TERRAIN_SIZE + ix];
        const h11 = self.terrain[(iy + 1) * TERRAIN_SIZE + ix + 1];
        return h00 * (1 - fx) * (1 - fy) + h10 * fx * (1 - fy) + h01 * (1 - fx) * fy + h11 * fx * fy;
    }

    fn clampVelocity(v: f32) f32 {
        if (v > TERMINAL_VELOCITY) return TERMINAL_VELOCITY;
        if (v < -TERMINAL_VELOCITY) return -TERMINAL_VELOCITY;
        return v;
    }

    fn initFn(_: *PhysicsEngine) bool {
        return true;
    }

    fn deinitFn(engine: *PhysicsEngine) void {
        const self = getSelf(engine);
        var iter = self.bodies.iterator();
        while (iter.next()) |entry| {
            self.alloc.destroy(entry.value_ptr.*);
        }
        self.bodies.deinit();
    }

    fn stepFn(engine: *PhysicsEngine, dt: f32) void {
        const self = getSelf(engine);
        var iter = self.bodies.iterator();
        while (iter.next()) |entry| {
            const body = entry.value_ptr.*;
            if (!body.is_physical) continue;

            if (!body.is_flying) {
                body.velocity.x += self.gravity.x * dt;
                body.velocity.y += self.gravity.y * dt;
                body.velocity.z += self.gravity.z * dt;

                if (body.position.z < self.water_level) {
                    body.velocity.z += body.buoyancy * (-self.gravity.z) * dt;
                }
            }

            body.velocity.x = clampVelocity(body.velocity.x);
            body.velocity.y = clampVelocity(body.velocity.y);
            body.velocity.z = clampVelocity(body.velocity.z);

            body.position.x += body.velocity.x * dt;
            body.position.y += body.velocity.y * dt;
            body.position.z += body.velocity.z * dt;

            body.position.x = @max(0, @min(255.9, body.position.x));
            body.position.y = @max(0, @min(255.9, body.position.y));

            const terrain_h = self.getTerrainHeight(body.position.x, body.position.y);
            if (body.position.z < terrain_h) {
                body.position.z = terrain_h;
                body.velocity.z = 0;
                body.on_ground = true;
            } else {
                body.on_ground = body.position.z <= terrain_h + 0.1;
            }
        }
    }

    fn createBodyFn(engine: *PhysicsEngine) ?*PhysicsBody {
        const self = getSelf(engine);
        const pb_impl = self.alloc.create(POSBody) catch return null;
        pb_impl.* = .{
            .position = Vec3{ .x = 128, .y = 128, .z = 25 },
            .velocity = Vec3{ .x = 0, .y = 0, .z = 0 },
            .rotation = Quat{ .x = 0, .y = 0, .z = 0, .w = 1 },
            .is_physical = true,
            .buoyancy = 1.0,
            .is_flying = false,
            .on_ground = true,
        };
        const id = self.next_id;
        self.next_id += 1;
        self.bodies.put(id, pb_impl) catch {
            self.alloc.destroy(pb_impl);
            return null;
        };
        const pb = self.alloc.create(PhysicsBody) catch {
            _ = self.bodies.fetchRemove(id);
            self.alloc.destroy(pb_impl);
            return null;
        };
        pb.* = .{
            .engine = engine,
            .impl_data = @ptrCast(pb_impl),
        };
        return pb;
    }

    fn destroyBodyFn(engine: *PhysicsEngine, body: *PhysicsBody) void {
        const self = getSelf(engine);
        const pb: *POSBody = @ptrCast(@alignCast(body.impl_data.?));
        var iter = self.bodies.iterator();
        while (iter.next()) |entry| {
            if (entry.value_ptr.* == pb) {
                _ = self.bodies.fetchRemove(entry.key_ptr.*);
                break;
            }
        }
        self.alloc.destroy(pb);
        self.alloc.destroy(body);
    }

    fn getBody(body: *PhysicsBody) *POSBody {
        return @ptrCast(@alignCast(body.impl_data.?));
    }

    fn getPositionFn(body: *PhysicsBody) Vec3 {
        return getBody(body).position;
    }

    fn setPositionFn(body: *PhysicsBody, pos: Vec3) void {
        getBody(body).position = pos;
    }

    fn getVelocityFn(body: *PhysicsBody) Vec3 {
        return getBody(body).velocity;
    }

    fn setVelocityFn(body: *PhysicsBody, vel: Vec3) void {
        getBody(body).velocity = vel;
    }

    fn getRotationFn(body: *PhysicsBody) Quat {
        return getBody(body).rotation;
    }

    fn setRotationFn(body: *PhysicsBody, rot: Quat) void {
        getBody(body).rotation = rot;
    }

    fn setGravityFn(engine: *PhysicsEngine, gravity: Vec3) void {
        getSelf(engine).gravity = gravity;
    }

    fn createTerrainFn(engine: *PhysicsEngine, heights: [*]const f32, w: u32, h: u32) bool {
        const self = getSelf(engine);
        if (w != TERRAIN_SIZE or h != TERRAIN_SIZE) return false;
        const total = TERRAIN_SIZE * TERRAIN_SIZE;
        @memcpy(self.terrain[0..total], heights[0..total]);
        self.has_terrain = true;
        return true;
    }

    fn getTerrainHeightExFn(engine: *PhysicsEngine, x: f32, y: f32) f32 {
        return getSelf(engine).getTerrainHeight(x, y);
    }

    fn setBodyFlyingFn(body: *PhysicsBody, flying: bool) void {
        getBody(body).is_flying = flying;
    }

    fn isBodyOnGroundFn(body: *PhysicsBody) bool {
        return getBody(body).on_ground;
    }

    fn raycastFn(_: *PhysicsEngine, _: Vec3, _: Vec3) interface.RaycastResult {
        return interface.RaycastResult.miss();
    }
};
