const std = @import("std");
const interface = @import("../interface.zig");
const Vec3 = interface.Vec3;
const Quat = interface.Quat;
const PhysicsBody = interface.PhysicsBody;
const PhysicsEngine = interface.PhysicsEngine;

const BSVector3 = extern struct { x: f32, y: f32, z: f32 };
const BSQuaternion = extern struct { x: f32, y: f32, z: f32, w: f32 };

extern "C" fn Initialize2(maxPos: BSVector3, maxCollisions: *c_int, collisionArray: *?*anyopaque, maxUpdates: *c_int, updateArray: *?*anyopaque) ?*anyopaque;
extern "C" fn Shutdown2(world: *anyopaque) void;
extern "C" fn PhysicsStep2(world: *anyopaque, timeStep: f32, maxSubSteps: c_int, fixedTimeStep: f32, updatedEntityCount: *c_int, collidersCount: *c_int) c_int;
extern "C" fn CreateBodyFromShape2(world: *anyopaque, shape: *anyopaque, id: u32, pos: BSVector3, rot: BSQuaternion) ?*anyopaque;
extern "C" fn CreateGroundPlaneShape2() ?*anyopaque;
extern "C" fn CreateTerrainShape2(id: u32, size: u32, heightMap: [*]const f32, minHeight: f32, maxHeight: f32) ?*anyopaque;
extern "C" fn BuildNativeShape2(world: *anyopaque, shapeData: *anyopaque) ?*anyopaque;
extern "C" fn BuildCapsuleShape2(world: *anyopaque, radius: f32, height: f32, upAxis: c_int) ?*anyopaque;
extern "C" fn DestroyObject2(world: *anyopaque, body: *anyopaque) bool;
extern "C" fn GetPosition2(body: *anyopaque) BSVector3;
extern "C" fn SetTranslation2(body: *anyopaque, pos: BSVector3, rot: BSQuaternion) bool;
extern "C" fn GetLinearVelocity2(body: *anyopaque) BSVector3;
extern "C" fn SetLinearVelocity2(body: *anyopaque, vel: BSVector3) bool;
extern "C" fn SetGravity2(body: *anyopaque, gravity: BSVector3) bool;
extern "C" fn GetOrientation2(body: *anyopaque) BSQuaternion;
extern "C" fn Activate2(body: *anyopaque) bool;
extern "C" fn AddObjectToWorld2(world: *anyopaque, body: *anyopaque) bool;
extern "C" fn ClearForces2(body: *anyopaque) void;
extern "C" fn CreateHullShape2(world: *anyopaque, hullCount: c_int, hulls: [*]const f32) ?*anyopaque;
extern "C" fn CreateMeshShape2(world: *anyopaque, indicesCount: c_int, indices: [*]const c_int, verticesCount: c_int, vertices: [*]const f32) ?*anyopaque;
extern "C" fn DeleteCollisionShape2(world: *anyopaque, shape: *anyopaque) bool;
extern "C" fn UpdateSingleAabb2(world: *anyopaque, body: *anyopaque) void;
extern "C" fn ForceActivationState2(body: *anyopaque, state: c_int) void;
extern "C" fn SetMassProps2(body: *anyopaque, mass: f32, inertia: BSVector3) void;
extern "C" fn SetCollisionFlags2(body: *anyopaque, flags: u32) u32;

extern "C" fn RayTest2_safe(
    world: *anyopaque,
    fx: f32, fy: f32, fz: f32,
    tx: f32, ty: f32, tz: f32,
    fg: u32, fm: u32,
    out_id: *u32, out_fraction: *f32,
    out_nx: *f32, out_ny: *f32, out_nz: *f32,
    out_px: *f32, out_py: *f32, out_pz: *f32,
) void;

const BulletBody = struct {
    bs_body: *anyopaque,
    bs_shape: *anyopaque,
    id: u32,
    is_flying: bool,
    on_ground: bool,
};

const TERRAIN_SIZE: u32 = 256;

pub const BulletEngine = struct {
    alloc: std.mem.Allocator,
    world: ?*anyopaque,
    next_id: u32,
    max_collisions: c_int,
    max_updates: c_int,
    collision_array: ?*anyopaque,
    update_array: ?*anyopaque,
    bodies: std.AutoHashMap(u32, BulletBody),
    terrain: [TERRAIN_SIZE * TERRAIN_SIZE]f32,
    terrain_shape: ?*anyopaque,
    terrain_body: ?*anyopaque,
    has_terrain: bool,

    pub fn create(alloc: std.mem.Allocator) ?*PhysicsEngine {
        const self = alloc.create(BulletEngine) catch return null;
        self.* = .{
            .alloc = alloc,
            .world = null,
            .next_id = 1,
            .max_collisions = 4096,
            .max_updates = 4096,
            .collision_array = null,
            .update_array = null,
            .bodies = std.AutoHashMap(u32, BulletBody).init(alloc),
            .terrain = [_]f32{0} ** (TERRAIN_SIZE * TERRAIN_SIZE),
            .terrain_shape = null,
            .terrain_body = null,
            .has_terrain = false,
        };
        const max_pos = BSVector3{ .x = 256, .y = 256, .z = 4096 };
        self.world = Initialize2(
            max_pos,
            &self.max_collisions,
            &self.collision_array,
            &self.max_updates,
            &self.update_array,
        );
        if (self.world == null) {
            self.bodies.deinit();
            alloc.destroy(self);
            return null;
        }

        const engine = alloc.create(PhysicsEngine) catch {
            Shutdown2(self.world.?);
            self.bodies.deinit();
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
            .name = "BulletSim",
            .engine_type = .bullet,
            .impl_ptr = @ptrCast(self),
        };
        return engine;
    }

    fn getSelf(engine: *PhysicsEngine) *BulletEngine {
        return @ptrCast(@alignCast(engine.impl_ptr));
    }

    fn initFn(_: *PhysicsEngine) bool {
        return true;
    }

    fn deinitFn(engine: *PhysicsEngine) void {
        const self = getSelf(engine);
        var iter = self.bodies.iterator();
        while (iter.next()) |entry| {
            _ = DestroyObject2(self.world.?, entry.value_ptr.bs_body);
        }
        self.bodies.deinit();
        if (self.world) |w| Shutdown2(w);
    }

    fn stepFn(engine: *PhysicsEngine, dt: f32) void {
        const self = getSelf(engine);
        if (self.world == null) return;
        var updated: c_int = 0;
        var colliders: c_int = 0;
        _ = PhysicsStep2(self.world.?, dt, 10, dt / 10.0, &updated, &colliders);
    }

    fn createBodyFn(engine: *PhysicsEngine) ?*PhysicsBody {
        const self = getSelf(engine);
        if (self.world == null) return null;
        const shape = BuildCapsuleShape2(self.world.?, 0.37, 1.6, 2) orelse return null;
        const id = self.next_id;
        self.next_id += 1;
        const pos = BSVector3{ .x = 128, .y = 128, .z = 25 };
        const rot = BSQuaternion{ .x = 0, .y = 0, .z = 0, .w = 1 };
        const bs_body = CreateBodyFromShape2(self.world.?, shape, id, pos, rot) orelse return null;
        _ = Activate2(bs_body);
        _ = SetGravity2(bs_body, BSVector3{ .x = 0, .y = 0, .z = -9.81 });

        self.bodies.put(id, .{ .bs_body = bs_body, .bs_shape = shape, .id = id, .is_flying = false, .on_ground = true }) catch {
            _ = DestroyObject2(self.world.?, bs_body);
            return null;
        };
        const pb = self.alloc.create(PhysicsBody) catch {
            _ = DestroyObject2(self.world.?, bs_body);
            _ = self.bodies.fetchRemove(id);
            return null;
        };
        pb.* = .{
            .engine = engine,
            .impl_data = @ptrCast(bs_body),
        };
        return pb;
    }

    fn destroyBodyFn(engine: *PhysicsEngine, body: *PhysicsBody) void {
        const self = getSelf(engine);
        if (self.world == null) return;
        const bs_body: *anyopaque = body.impl_data orelse return;
        var iter = self.bodies.iterator();
        while (iter.next()) |entry| {
            if (entry.value_ptr.bs_body == bs_body) {
                _ = DestroyObject2(self.world.?, bs_body);
                _ = self.bodies.fetchRemove(entry.key_ptr.*);
                break;
            }
        }
        self.alloc.destroy(body);
    }

    fn getPositionFn(body: *PhysicsBody) Vec3 {
        const bs: *anyopaque = body.impl_data orelse return Vec3{ .x = 0, .y = 0, .z = 0 };
        const p = GetPosition2(bs);
        return Vec3{ .x = p.x, .y = p.y, .z = p.z };
    }

    fn setPositionFn(body: *PhysicsBody, pos: Vec3) void {
        const bs: *anyopaque = body.impl_data orelse return;
        const q = GetOrientation2(bs);
        _ = SetTranslation2(bs, BSVector3{ .x = pos.x, .y = pos.y, .z = pos.z }, q);
    }

    fn getVelocityFn(body: *PhysicsBody) Vec3 {
        const bs: *anyopaque = body.impl_data orelse return Vec3{ .x = 0, .y = 0, .z = 0 };
        const v = GetLinearVelocity2(bs);
        return Vec3{ .x = v.x, .y = v.y, .z = v.z };
    }

    fn setVelocityFn(body: *PhysicsBody, vel: Vec3) void {
        const bs: *anyopaque = body.impl_data orelse return;
        _ = SetLinearVelocity2(bs, BSVector3{ .x = vel.x, .y = vel.y, .z = vel.z });
    }

    fn getRotationFn(body: *PhysicsBody) Quat {
        const bs: *anyopaque = body.impl_data orelse return Quat{ .x = 0, .y = 0, .z = 0, .w = 1 };
        const q = GetOrientation2(bs);
        return Quat{ .x = q.x, .y = q.y, .z = q.z, .w = q.w };
    }

    fn setRotationFn(body: *PhysicsBody, rot: Quat) void {
        const bs: *anyopaque = body.impl_data orelse return;
        const p = GetPosition2(bs);
        _ = SetTranslation2(bs, p, BSQuaternion{ .x = rot.x, .y = rot.y, .z = rot.z, .w = rot.w });
    }

    fn setGravityFn(engine: *PhysicsEngine, gravity: Vec3) void {
        const self = getSelf(engine);
        var iter = self.bodies.iterator();
        while (iter.next()) |entry| {
            _ = SetGravity2(entry.value_ptr.bs_body, BSVector3{ .x = gravity.x, .y = gravity.y, .z = gravity.z });
        }
    }

    fn createTerrainFn(engine: *PhysicsEngine, heights: [*]const f32, w: u32, h: u32) bool {
        const self = getSelf(engine);
        if (w != TERRAIN_SIZE or h != TERRAIN_SIZE) return false;
        const total = TERRAIN_SIZE * TERRAIN_SIZE;
        @memcpy(self.terrain[0..total], heights[0..total]);
        self.has_terrain = true;
        return true;
    }

    fn getTerrainHeightFn(engine: *PhysicsEngine, x: f32, y: f32) f32 {
        const self = getSelf(engine);
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

    fn setBodyFlyingFn(body: *PhysicsBody, flying: bool) void {
        const bs: *anyopaque = body.impl_data orelse return;
        const eng = body.engine orelse return;
        const self = getSelf(eng);
        var iter = self.bodies.iterator();
        while (iter.next()) |entry| {
            if (entry.value_ptr.bs_body == bs) {
                entry.value_ptr.is_flying = flying;
                if (flying) {
                    _ = SetGravity2(bs, BSVector3{ .x = 0, .y = 0, .z = 0 });
                } else {
                    _ = SetGravity2(bs, BSVector3{ .x = 0, .y = 0, .z = -9.81 });
                }
                break;
            }
        }
    }

    fn isBodyOnGroundFn(body: *PhysicsBody) bool {
        const bs: *anyopaque = body.impl_data orelse return true;
        const eng = body.engine orelse return true;
        const self = getSelf(eng);
        var iter = self.bodies.iterator();
        while (iter.next()) |entry| {
            if (entry.value_ptr.bs_body == bs) {
                return entry.value_ptr.on_ground;
            }
        }
        return true;
    }

    fn raycastFn(engine: *PhysicsEngine, from: Vec3, to: Vec3) interface.RaycastResult {
        const self_eng = getSelf(engine);
        const world = self_eng.world orelse return interface.RaycastResult.miss();

        var hit_id: u32 = 0;
        var hit_fraction: f32 = 0;
        var hit_nx: f32 = 0;
        var hit_ny: f32 = 0;
        var hit_nz: f32 = 0;
        var hit_px: f32 = 0;
        var hit_py: f32 = 0;
        var hit_pz: f32 = 0;

        RayTest2_safe(
            world,
            from.x, from.y, from.z,
            to.x, to.y, to.z,
            0xFFFF, 0xFFFF,
            &hit_id, &hit_fraction,
            &hit_nx, &hit_ny, &hit_nz,
            &hit_px, &hit_py, &hit_pz,
        );

        const pn_sum = @abs(hit_nx) + @abs(hit_ny) + @abs(hit_nz) + @abs(hit_px) + @abs(hit_py) + @abs(hit_pz);
        if (pn_sum > 0.001 or hit_id != 0) {
            if (pn_sum > 0.001) {
                return interface.RaycastResult{
                    .hit = true,
                    .point = Vec3{ .x = hit_px, .y = hit_py, .z = hit_pz },
                    .normal = Vec3{ .x = hit_nx, .y = hit_ny, .z = hit_nz },
                    .fraction = hit_fraction,
                    .body_id = hit_id,
                };
            }
        }

        // Fallback: ray-terrain intersection via heightmap march
        if (!self_eng.has_terrain) return interface.RaycastResult.miss();

        const dir = Vec3{
            .x = to.x - from.x,
            .y = to.y - from.y,
            .z = to.z - from.z,
        };
        const len = @sqrt(dir.x * dir.x + dir.y * dir.y + dir.z * dir.z);
        if (len < 0.001) return interface.RaycastResult.miss();

        const inv_len = 1.0 / len;
        const d = Vec3{ .x = dir.x * inv_len, .y = dir.y * inv_len, .z = dir.z * inv_len };

        const step_size: f32 = 0.5;
        const max_steps: u32 = @intFromFloat(@min(len / step_size + 1.0, 2048.0));
        var prev_above = true;
        var prev_t: f32 = 0;

        var i: u32 = 0;
        while (i <= max_steps) : (i += 1) {
            const t = @as(f32, @floatFromInt(i)) * step_size;
            const px = from.x + d.x * t;
            const py = from.y + d.y * t;
            const pz = from.z + d.z * t;

            if (px < 0 or px >= @as(f32, TERRAIN_SIZE) or py < 0 or py >= @as(f32, TERRAIN_SIZE)) {
                prev_t = t;
                continue;
            }

            const th = getTerrainHeightFn(engine, px, py);
            const above = pz > th;

            if (!above and prev_above and i > 0) {
                const mid_t = (prev_t + t) * 0.5;
                const hx = from.x + d.x * mid_t;
                const hy = from.y + d.y * mid_t;
                const hz_terrain = getTerrainHeightFn(engine, hx, hy);
                return interface.RaycastResult{
                    .hit = true,
                    .point = Vec3{ .x = hx, .y = hy, .z = hz_terrain },
                    .normal = Vec3{ .x = 0, .y = 0, .z = 1 },
                    .fraction = mid_t / len,
                    .body_id = 0,
                };
            }
            prev_above = above;
            prev_t = t;
        }

        return interface.RaycastResult.miss();
    }
};
