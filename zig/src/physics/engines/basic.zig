const std = @import("std");
const interface = @import("../interface.zig");
const Vec3 = interface.Vec3;
const Quat = interface.Quat;
const PhysicsBody = interface.PhysicsBody;
const PhysicsEngine = interface.PhysicsEngine;

const BasicBody = struct {
    position: Vec3,
    velocity: Vec3,
    rotation: Quat,
    is_physical: bool,
    is_flying: bool,
    on_ground: bool,
};

pub const BasicEngine = struct {
    alloc: std.mem.Allocator,
    bodies: std.AutoHashMap(u64, *BasicBody),
    next_id: u64,
    gravity: Vec3,

    pub fn create(alloc: std.mem.Allocator) ?*PhysicsEngine {
        const self = alloc.create(BasicEngine) catch return null;
        self.* = .{
            .alloc = alloc,
            .bodies = std.AutoHashMap(u64, *BasicBody).init(alloc),
            .next_id = 1,
            .gravity = Vec3{ .x = 0, .y = 0, .z = -9.81 },
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
            .getTerrainHeightFn = getTerrainHeightFn,
            .setBodyFlyingFn = setBodyFlyingFn,
            .isBodyOnGroundFn = isBodyOnGroundFn,
            .raycastFn = raycastFn,
            .name = "BasicPhysics",
            .engine_type = .basic,
            .impl_ptr = @ptrCast(self),
        };
        return engine;
    }

    fn getSelf(engine: *PhysicsEngine) *BasicEngine {
        return @ptrCast(@alignCast(engine.impl_ptr));
    }

    fn initFn(engine: *PhysicsEngine) bool {
        _ = engine;
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
            }
            body.position.x += body.velocity.x * dt;
            body.position.y += body.velocity.y * dt;
            body.position.z += body.velocity.z * dt;
            if (body.position.z < 0) {
                body.position.z = 0;
                body.velocity.z = 0;
                body.on_ground = true;
            } else {
                body.on_ground = false;
            }
        }
    }

    fn createBodyFn(engine: *PhysicsEngine) ?*PhysicsBody {
        const self = getSelf(engine);
        const bb = self.alloc.create(BasicBody) catch return null;
        bb.* = .{
            .position = Vec3{ .x = 128, .y = 128, .z = 25 },
            .velocity = Vec3{ .x = 0, .y = 0, .z = 0 },
            .rotation = Quat{ .x = 0, .y = 0, .z = 0, .w = 1 },
            .is_physical = true,
            .is_flying = false,
            .on_ground = true,
        };
        const id = self.next_id;
        self.next_id += 1;
        self.bodies.put(id, bb) catch {
            self.alloc.destroy(bb);
            return null;
        };
        const pb = self.alloc.create(PhysicsBody) catch {
            _ = self.bodies.fetchRemove(id);
            self.alloc.destroy(bb);
            return null;
        };
        pb.* = .{
            .engine = engine,
            .impl_data = @ptrCast(bb),
        };
        return pb;
    }

    fn destroyBodyFn(engine: *PhysicsEngine, body: *PhysicsBody) void {
        const self = getSelf(engine);
        const bb: *BasicBody = @ptrCast(@alignCast(body.impl_data.?));
        var iter = self.bodies.iterator();
        while (iter.next()) |entry| {
            if (entry.value_ptr.* == bb) {
                _ = self.bodies.fetchRemove(entry.key_ptr.*);
                break;
            }
        }
        self.alloc.destroy(bb);
        self.alloc.destroy(body);
    }

    fn getBody(body: *PhysicsBody) *BasicBody {
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

    fn createTerrainFn(_: *PhysicsEngine, _: [*]const f32, _: u32, _: u32) bool {
        return true;
    }

    fn getTerrainHeightFn(_: *PhysicsEngine, _: f32, _: f32) f32 {
        return 0.0;
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
