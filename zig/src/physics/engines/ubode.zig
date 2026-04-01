const std = @import("std");
const c = @cImport({
    @cInclude("ode/ode.h");
});
const interface = @import("../interface.zig");
const Vec3 = interface.Vec3;
const Quat = interface.Quat;
const PhysicsBody = interface.PhysicsBody;
const PhysicsEngine = interface.PhysicsEngine;

var ode_initialized: bool = false;

const UbOdeBody = struct {
    body_id: c.dBodyID,
    geom_id: c.dGeomID,
    is_avatar: bool,
    is_flying: bool,
    on_ground: bool,
};

const UBODE_TERRAIN_SIZE = 256;

pub const UbOdeEngine = struct {
    alloc: std.mem.Allocator,
    world_id: c.dWorldID,
    space_id: c.dSpaceID,
    contact_group: c.dJointGroupID,
    pending_destruction: std.AutoHashMap(*PhysicsBody, void),
    simulation_active: bool,
    terrain_data: ?c.dHeightfieldDataID,
    terrain_geom: ?c.dGeomID,
    terrain_heights: [UBODE_TERRAIN_SIZE * UBODE_TERRAIN_SIZE]f32,
    has_terrain: bool,

    pub fn create(alloc: std.mem.Allocator) ?*PhysicsEngine {
        if (!ode_initialized) {
            if (c.dInitODE2(0) == 0) return null;
            ode_initialized = true;
        }
        const self = alloc.create(UbOdeEngine) catch return null;
        self.* = .{
            .alloc = alloc,
            .world_id = c.dWorldCreate(),
            .space_id = c.dHashSpaceCreate(null),
            .contact_group = c.dJointGroupCreate(0),
            .pending_destruction = std.AutoHashMap(*PhysicsBody, void).init(alloc),
            .simulation_active = false,
            .terrain_data = null,
            .terrain_geom = null,
            .terrain_heights = [_]f32{0} ** (UBODE_TERRAIN_SIZE * UBODE_TERRAIN_SIZE),
            .has_terrain = false,
        };
        c.dWorldSetGravity(self.world_id, 0, 0, -9.81);
        c.dWorldSetCFM(self.world_id, 0.0001);
        c.dWorldSetERP(self.world_id, 0.6);
        c.dWorldSetQuickStepNumIterations(self.world_id, 10);
        c.dWorldSetContactMaxCorrectingVel(self.world_id, 30.0);
        c.dWorldSetContactSurfaceLayer(self.world_id, 0.001);
        c.dWorldSetAutoDisableFlag(self.world_id, 1);
        c.dWorldSetAutoDisableLinearThreshold(self.world_id, 0.01);
        c.dWorldSetAutoDisableAngularThreshold(self.world_id, 0.01);
        c.dWorldSetAutoDisableSteps(self.world_id, 20);
        _ = c.dCreatePlane(self.space_id, 0, 0, 1, 0);

        const engine = alloc.create(PhysicsEngine) catch {
            self.cleanup();
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
            .name = "ubODE",
            .engine_type = .ubode,
            .impl_ptr = @ptrCast(self),
        };
        return engine;
    }

    fn cleanup(self: *UbOdeEngine) void {
        if (self.terrain_geom) |tg| c.dGeomDestroy(tg);
        if (self.terrain_data) |td| c.dGeomHeightfieldDataDestroy(td);
        c.dJointGroupDestroy(self.contact_group);
        c.dSpaceDestroy(self.space_id);
        c.dWorldDestroy(self.world_id);
    }

    fn getSelf(engine: *PhysicsEngine) *UbOdeEngine {
        return @ptrCast(@alignCast(engine.impl_ptr));
    }

    fn processPending(self: *UbOdeEngine) void {
        var iter = self.pending_destruction.iterator();
        while (iter.next()) |entry| {
            const body = entry.key_ptr.*;
            const ob: *UbOdeBody = @ptrCast(@alignCast(body.impl_data.?));
            cleanupJointsForBody(ob.body_id);
            if (ob.geom_id != null) c.dGeomDestroy(ob.geom_id);
            if (ob.body_id != null) c.dBodyDestroy(ob.body_id);
            self.alloc.destroy(ob);
            self.alloc.destroy(body);
        }
        self.pending_destruction.clearRetainingCapacity();
    }

    fn initFn(_: *PhysicsEngine) bool {
        return true;
    }

    fn deinitFn(engine: *PhysicsEngine) void {
        const self = getSelf(engine);
        self.processPending();
        self.pending_destruction.deinit();
        self.cleanup();
    }

    fn stepFn(engine: *PhysicsEngine, dt: f32) void {
        const self = getSelf(engine);
        self.simulation_active = true;
        const callback: c.dNearCallback = nearCallback;
        c.dSpaceCollide(self.space_id, self, callback);
        _ = c.dWorldQuickStep(self.world_id, dt);
        c.dJointGroupEmpty(self.contact_group);
        self.simulation_active = false;
        self.processPending();
    }

    fn createBodyFn(engine: *PhysicsEngine) ?*PhysicsBody {
        const self = getSelf(engine);
        const ob = self.alloc.create(UbOdeBody) catch return null;
        ob.body_id = c.dBodyCreate(self.world_id);
        if (ob.body_id == null) {
            self.alloc.destroy(ob);
            return null;
        }
        ob.is_flying = false;
        ob.on_ground = true;
        ob.geom_id = c.dCreateCapsule(self.space_id, 0.37, 1.6);
        if (ob.geom_id == null) {
            c.dBodyDestroy(ob.body_id);
            self.alloc.destroy(ob);
            return null;
        }
        ob.is_avatar = true;
        c.dGeomSetBody(ob.geom_id, ob.body_id);
        c.dBodySetPosition(ob.body_id, 128.0, 128.0, 25.0);
        var mass: c.dMass = undefined;
        c.dMassSetCapsule(&mass, 80.0, 3, 0.37, 1.6);
        c.dBodySetMass(ob.body_id, &mass);

        const pb = self.alloc.create(PhysicsBody) catch {
            c.dGeomDestroy(ob.geom_id);
            c.dBodyDestroy(ob.body_id);
            self.alloc.destroy(ob);
            return null;
        };
        pb.* = .{
            .engine = engine,
            .impl_data = @ptrCast(ob),
        };
        return pb;
    }

    fn destroyBodyFn(engine: *PhysicsEngine, body: *PhysicsBody) void {
        const self = getSelf(engine);
        if (self.simulation_active) {
            self.pending_destruction.put(body, {}) catch {};
            return;
        }
        const ob: *UbOdeBody = @ptrCast(@alignCast(body.impl_data.?));
        cleanupJointsForBody(ob.body_id);
        if (ob.geom_id != null) c.dGeomDestroy(ob.geom_id);
        if (ob.body_id != null) c.dBodyDestroy(ob.body_id);
        self.alloc.destroy(ob);
        self.alloc.destroy(body);
    }

    fn getUbOdeBody(body: *PhysicsBody) *UbOdeBody {
        return @ptrCast(@alignCast(body.impl_data.?));
    }

    fn getPositionFn(body: *PhysicsBody) Vec3 {
        const ob = getUbOdeBody(body);
        const pos = c.dBodyGetPosition(ob.body_id);
        return Vec3{ .x = @floatCast(pos[0]), .y = @floatCast(pos[1]), .z = @floatCast(pos[2]) };
    }

    fn setPositionFn(body: *PhysicsBody, pos: Vec3) void {
        c.dBodySetPosition(getUbOdeBody(body).body_id, pos.x, pos.y, pos.z);
    }

    fn getVelocityFn(body: *PhysicsBody) Vec3 {
        const vel = c.dBodyGetLinearVel(getUbOdeBody(body).body_id);
        return Vec3{ .x = @floatCast(vel[0]), .y = @floatCast(vel[1]), .z = @floatCast(vel[2]) };
    }

    fn setVelocityFn(body: *PhysicsBody, vel: Vec3) void {
        c.dBodySetLinearVel(getUbOdeBody(body).body_id, vel.x, vel.y, vel.z);
    }

    fn getRotationFn(body: *PhysicsBody) Quat {
        const ob = getUbOdeBody(body);
        const r = c.dBodyGetRotation(ob.body_id);
        const trace = r[0] + r[5] + r[10];
        if (trace > 0) {
            const s = @sqrt(trace + 1.0) * 2.0;
            return Quat{ .w = @floatCast(s * 0.25), .x = @floatCast((r[9] - r[6]) / s), .y = @floatCast((r[2] - r[8]) / s), .z = @floatCast((r[4] - r[1]) / s) };
        } else if (r[0] > r[5] and r[0] > r[10]) {
            const s = @sqrt(1.0 + r[0] - r[5] - r[10]) * 2.0;
            return Quat{ .w = @floatCast((r[9] - r[6]) / s), .x = @floatCast(s * 0.25), .y = @floatCast((r[1] + r[4]) / s), .z = @floatCast((r[2] + r[8]) / s) };
        } else if (r[5] > r[10]) {
            const s = @sqrt(1.0 + r[5] - r[0] - r[10]) * 2.0;
            return Quat{ .w = @floatCast((r[2] - r[8]) / s), .x = @floatCast((r[1] + r[4]) / s), .y = @floatCast(s * 0.25), .z = @floatCast((r[6] + r[9]) / s) };
        } else {
            const s = @sqrt(1.0 + r[10] - r[0] - r[5]) * 2.0;
            return Quat{ .w = @floatCast((r[4] - r[1]) / s), .x = @floatCast((r[2] + r[8]) / s), .y = @floatCast((r[6] + r[9]) / s), .z = @floatCast(s * 0.25) };
        }
    }

    fn setRotationFn(body: *PhysicsBody, rot: Quat) void {
        const ob = getUbOdeBody(body);
        const x: f64 = rot.x;
        const y: f64 = rot.y;
        const z: f64 = rot.z;
        const w: f64 = rot.w;
        var mat: [12]c.dReal = undefined;
        mat[0] = 1.0 - 2.0 * (y * y + z * z);
        mat[1] = 2.0 * (x * y - z * w);
        mat[2] = 2.0 * (x * z + y * w);
        mat[3] = 0;
        mat[4] = 2.0 * (x * y + z * w);
        mat[5] = 1.0 - 2.0 * (x * x + z * z);
        mat[6] = 2.0 * (y * z - x * w);
        mat[7] = 0;
        mat[8] = 2.0 * (x * z - y * w);
        mat[9] = 2.0 * (y * z + x * w);
        mat[10] = 1.0 - 2.0 * (x * x + y * y);
        mat[11] = 0;
        c.dBodySetRotation(ob.body_id, &mat);
    }

    fn setGravityFn(engine: *PhysicsEngine, gravity: Vec3) void {
        c.dWorldSetGravity(getSelf(engine).world_id, gravity.x, gravity.y, gravity.z);
    }

    fn createTerrainFn(engine: *PhysicsEngine, heights: [*]const f32, w: u32, h: u32) bool {
        const self = getSelf(engine);
        if (self.terrain_geom) |tg| {
            c.dGeomDestroy(tg);
            self.terrain_geom = null;
        }
        if (self.terrain_data) |td| {
            c.dGeomHeightfieldDataDestroy(td);
            self.terrain_data = null;
        }
        const td = c.dGeomHeightfieldDataCreate();
        if (td == null) return false;
        c.dGeomHeightfieldDataBuildSingle(
            td,
            @ptrCast(heights),
            0,
            @floatFromInt(w),
            @floatFromInt(h),
            @intCast(w),
            @intCast(h),
            1.0,
            0.0,
            0.0,
            0,
        );
        c.dGeomHeightfieldDataSetBounds(td, -100.0, 10000.0);
        const tg = c.dCreateHeightfield(self.space_id, td, 1);
        if (tg == null) {
            c.dGeomHeightfieldDataDestroy(td);
            return false;
        }
        c.dGeomSetPosition(tg, @as(f64, @floatFromInt(w)) / 2.0, @as(f64, @floatFromInt(h)) / 2.0, 0);
        self.terrain_data = td;
        self.terrain_geom = tg;
        if (w == UBODE_TERRAIN_SIZE and h == UBODE_TERRAIN_SIZE) {
            const total = UBODE_TERRAIN_SIZE * UBODE_TERRAIN_SIZE;
            @memcpy(self.terrain_heights[0..total], heights[0..total]);
            self.has_terrain = true;
        }
        return true;
    }

    fn sampleTerrainHeight(self: *UbOdeEngine, x: f32, y: f32) f32 {
        if (!self.has_terrain) return 0.0;
        const ix = @as(u32, @intFromFloat(@max(0, @min(@as(f32, UBODE_TERRAIN_SIZE - 2), x))));
        const iy = @as(u32, @intFromFloat(@max(0, @min(@as(f32, UBODE_TERRAIN_SIZE - 2), y))));
        const fx = x - @as(f32, @floatFromInt(ix));
        const fy = y - @as(f32, @floatFromInt(iy));
        const h00 = self.terrain_heights[iy * UBODE_TERRAIN_SIZE + ix];
        const h10 = self.terrain_heights[iy * UBODE_TERRAIN_SIZE + ix + 1];
        const h01 = self.terrain_heights[(iy + 1) * UBODE_TERRAIN_SIZE + ix];
        const h11 = self.terrain_heights[(iy + 1) * UBODE_TERRAIN_SIZE + ix + 1];
        return h00 * (1 - fx) * (1 - fy) + h10 * fx * (1 - fy) + h01 * (1 - fx) * fy + h11 * fx * fy;
    }

    fn getTerrainHeightFn(engine: *PhysicsEngine, x: f32, y: f32) f32 {
        return getSelf(engine).sampleTerrainHeight(x, y);
    }

    fn setBodyFlyingFn(body: *PhysicsBody, flying: bool) void {
        const ob = getUbOdeBody(body);
        ob.is_flying = flying;
        if (ob.body_id != null) {
            c.dBodySetGravityMode(ob.body_id, if (flying) @as(c_int, 0) else @as(c_int, 1));
        }
    }

    fn isBodyOnGroundFn(body: *PhysicsBody) bool {
        return getUbOdeBody(body).on_ground;
    }

    fn raycastFn(_: *PhysicsEngine, _: Vec3, _: Vec3) interface.RaycastResult {
        return interface.RaycastResult.miss();
    }
};

fn nearCallback(data: ?*anyopaque, o1: c.dGeomID, o2: c.dGeomID) callconv(.c) void {
    const MAX_CONTACTS = 12;
    var contacts: [MAX_CONTACTS]c.dContactGeom = undefined;
    const numc = @as(usize, @intCast(c.dCollide(o1, o2, MAX_CONTACTS, &contacts, @sizeOf(c.dContactGeom))));
    if (numc == 0) return;
    const self: *UbOdeEngine = @ptrCast(@alignCast(data.?));
    for (contacts[0..numc]) |contact_geom| {
        var jc = c.dContact{
            .surface = .{
                .mode = c.dContactBounce | c.dContactSoftCFM | c.dContactSoftERP,
                .mu = 250.0,
                .bounce = 0.2,
                .bounce_vel = 0.5,
                .soft_cfm = 0.0001,
                .soft_erp = 0.6,
            },
            .geom = contact_geom,
        };
        const joint = c.dJointCreateContact(self.world_id, self.contact_group, &jc);
        c.dJointAttach(joint, c.dGeomGetBody(o1), c.dGeomGetBody(o2));
    }
}

fn cleanupJointsForBody(body_id: c.dBodyID) void {
    if (body_id == null) return;
    var num_joints = c.dBodyGetNumJoints(body_id);
    var i: c_int = 0;
    while (i < num_joints) : (i += 1) {
        const joint = c.dBodyGetJoint(body_id, i);
        if (joint != null) {
            c.dJointDestroy(joint);
            num_joints = c.dBodyGetNumJoints(body_id);
            i -= 1;
        }
    }
}
