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

const OdeBody = struct {
    body_id: c.dBodyID,
    geom_id: c.dGeomID,
    is_flying: bool,
    on_ground: bool,
};

pub const OdeEngine = struct {
    alloc: std.mem.Allocator,
    world_id: c.dWorldID,
    space_id: c.dSpaceID,
    contact_group: c.dJointGroupID,
    pending_destruction: std.AutoHashMap(*PhysicsBody, void),
    simulation_active: bool,

    pub fn create(alloc: std.mem.Allocator) ?*PhysicsEngine {
        if (!ode_initialized) {
            if (c.dInitODE2(0) == 0) return null;
            ode_initialized = true;
        }
        const self = alloc.create(OdeEngine) catch return null;
        self.* = .{
            .alloc = alloc,
            .world_id = c.dWorldCreate(),
            .space_id = c.dHashSpaceCreate(null),
            .contact_group = c.dJointGroupCreate(0),
            .pending_destruction = std.AutoHashMap(*PhysicsBody, void).init(alloc),
            .simulation_active = false,
        };
        c.dWorldSetGravity(self.world_id, 0, 0, -9.81);
        c.dWorldSetCFM(self.world_id, 1e-5);
        c.dWorldSetERP(self.world_id, 0.2);
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
            .name = "ODE",
            .engine_type = .ode,
            .impl_ptr = @ptrCast(self),
        };
        return engine;
    }

    fn cleanup(self: *OdeEngine) void {
        c.dJointGroupDestroy(self.contact_group);
        c.dSpaceDestroy(self.space_id);
        c.dWorldDestroy(self.world_id);
    }

    fn getSelf(engine: *PhysicsEngine) *OdeEngine {
        return @ptrCast(@alignCast(engine.impl_ptr));
    }

    fn processPending(self: *OdeEngine) void {
        var iter = self.pending_destruction.iterator();
        while (iter.next()) |entry| {
            const body = entry.key_ptr.*;
            const ob: *OdeBody = @ptrCast(@alignCast(body.impl_data.?));
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
        const ob = self.alloc.create(OdeBody) catch return null;
        ob.body_id = c.dBodyCreate(self.world_id);
        if (ob.body_id == null) {
            self.alloc.destroy(ob);
            return null;
        }
        ob.is_flying = false;
        ob.on_ground = true;
        ob.geom_id = c.dCreateBox(self.space_id, 1.0, 1.0, 1.0);
        if (ob.geom_id == null) {
            c.dBodyDestroy(ob.body_id);
            self.alloc.destroy(ob);
            return null;
        }
        c.dGeomSetBody(ob.geom_id, ob.body_id);
        c.dBodySetPosition(ob.body_id, 128.0, 128.0, 25.0);
        c.dBodySetGravityMode(ob.body_id, 0);
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
        const ob: *OdeBody = @ptrCast(@alignCast(body.impl_data.?));
        cleanupJointsForBody(ob.body_id);
        if (ob.geom_id != null) c.dGeomDestroy(ob.geom_id);
        if (ob.body_id != null) c.dBodyDestroy(ob.body_id);
        self.alloc.destroy(ob);
        self.alloc.destroy(body);
    }

    fn getOdeBody(body: *PhysicsBody) *OdeBody {
        return @ptrCast(@alignCast(body.impl_data.?));
    }

    fn getPositionFn(body: *PhysicsBody) Vec3 {
        const ob = getOdeBody(body);
        const pos = c.dBodyGetPosition(ob.body_id);
        return Vec3{
            .x = @floatCast(pos[0]),
            .y = @floatCast(pos[1]),
            .z = @floatCast(pos[2]),
        };
    }

    fn setPositionFn(body: *PhysicsBody, pos: Vec3) void {
        const ob = getOdeBody(body);
        c.dBodySetPosition(ob.body_id, pos.x, pos.y, pos.z);
    }

    fn getVelocityFn(body: *PhysicsBody) Vec3 {
        const ob = getOdeBody(body);
        const vel = c.dBodyGetLinearVel(ob.body_id);
        return Vec3{
            .x = @floatCast(vel[0]),
            .y = @floatCast(vel[1]),
            .z = @floatCast(vel[2]),
        };
    }

    fn setVelocityFn(body: *PhysicsBody, vel: Vec3) void {
        const ob = getOdeBody(body);
        c.dBodySetLinearVel(ob.body_id, vel.x, vel.y, vel.z);
    }

    fn getRotationFn(body: *PhysicsBody) Quat {
        const ob = getOdeBody(body);
        const r = c.dBodyGetRotation(ob.body_id);
        const trace = r[0] + r[5] + r[10];
        if (trace > 0) {
            const s = @sqrt(trace + 1.0) * 2.0;
            return Quat{
                .w = @floatCast(s * 0.25),
                .x = @floatCast((r[9] - r[6]) / s),
                .y = @floatCast((r[2] - r[8]) / s),
                .z = @floatCast((r[4] - r[1]) / s),
            };
        } else if (r[0] > r[5] and r[0] > r[10]) {
            const s = @sqrt(1.0 + r[0] - r[5] - r[10]) * 2.0;
            return Quat{
                .w = @floatCast((r[9] - r[6]) / s),
                .x = @floatCast(s * 0.25),
                .y = @floatCast((r[1] + r[4]) / s),
                .z = @floatCast((r[2] + r[8]) / s),
            };
        } else if (r[5] > r[10]) {
            const s = @sqrt(1.0 + r[5] - r[0] - r[10]) * 2.0;
            return Quat{
                .w = @floatCast((r[2] - r[8]) / s),
                .x = @floatCast((r[1] + r[4]) / s),
                .y = @floatCast(s * 0.25),
                .z = @floatCast((r[6] + r[9]) / s),
            };
        } else {
            const s = @sqrt(1.0 + r[10] - r[0] - r[5]) * 2.0;
            return Quat{
                .w = @floatCast((r[4] - r[1]) / s),
                .x = @floatCast((r[2] + r[8]) / s),
                .y = @floatCast((r[6] + r[9]) / s),
                .z = @floatCast(s * 0.25),
            };
        }
    }

    fn setRotationFn(body: *PhysicsBody, rot: Quat) void {
        const ob = getOdeBody(body);
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
        const self = getSelf(engine);
        c.dWorldSetGravity(self.world_id, gravity.x, gravity.y, gravity.z);
    }

    fn createTerrainFn(engine: *PhysicsEngine, heights: [*]const f32, w: u32, h: u32) bool {
        _ = heights;
        _ = w;
        _ = h;
        _ = engine;
        return true;
    }

    fn getTerrainHeightFn(_: *PhysicsEngine, _: f32, _: f32) f32 {
        return 0.0;
    }

    fn setBodyFlyingFn(body: *PhysicsBody, flying: bool) void {
        const ob = getOdeBody(body);
        ob.is_flying = flying;
        if (ob.body_id != null) {
            c.dBodySetGravityMode(ob.body_id, if (flying) @as(c_int, 0) else @as(c_int, 1));
        }
    }

    fn isBodyOnGroundFn(body: *PhysicsBody) bool {
        return getOdeBody(body).on_ground;
    }

    fn raycastFn(_: *PhysicsEngine, _: Vec3, _: Vec3) interface.RaycastResult {
        return interface.RaycastResult.miss();
    }
};

fn nearCallback(data: ?*anyopaque, o1: c.dGeomID, o2: c.dGeomID) callconv(.c) void {
    const MAX_CONTACTS = 8;
    var contacts: [MAX_CONTACTS]c.dContactGeom = undefined;
    const numc = @as(usize, @intCast(c.dCollide(o1, o2, MAX_CONTACTS, &contacts, @sizeOf(c.dContactGeom))));
    if (numc == 0) return;
    const self: *OdeEngine = @ptrCast(@alignCast(data.?));
    for (contacts[0..numc]) |contact_geom| {
        var jc = c.dContact{
            .surface = .{
                .mode = c.dContactBounce | c.dContactSoftCFM,
                .mu = 0.5,
                .bounce = 0.1,
                .bounce_vel = 0.1,
                .soft_cfm = 0.01,
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
