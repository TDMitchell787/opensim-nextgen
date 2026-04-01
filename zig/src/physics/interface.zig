const std = @import("std");

pub const EngineType = enum(u8) {
    basic = 0,
    pos = 1,
    ode = 2,
    ubode = 3,
    bullet = 4,
};

pub const Vec3 = extern struct {
    x: f32,
    y: f32,
    z: f32,
};

pub const Quat = extern struct {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
};

pub const PhysicsBody = extern struct {
    engine: ?*PhysicsEngine,
    impl_data: ?*anyopaque,
};

pub const RaycastResult = extern struct {
    hit: bool,
    point: Vec3,
    normal: Vec3,
    fraction: f32,
    body_id: u32,

    pub fn miss() RaycastResult {
        return .{ .hit = false, .point = .{ .x = 0, .y = 0, .z = 0 }, .normal = .{ .x = 0, .y = 0, .z = 0 }, .fraction = 0, .body_id = 0 };
    }
};

pub const PhysicsEngine = struct {
    initFn: *const fn (*PhysicsEngine) bool,
    deinitFn: *const fn (*PhysicsEngine) void,
    stepFn: *const fn (*PhysicsEngine, f32) void,
    createBodyFn: *const fn (*PhysicsEngine) ?*PhysicsBody,
    destroyBodyFn: *const fn (*PhysicsEngine, *PhysicsBody) void,
    getPositionFn: *const fn (*PhysicsBody) Vec3,
    setPositionFn: *const fn (*PhysicsBody, Vec3) void,
    getVelocityFn: *const fn (*PhysicsBody) Vec3,
    setVelocityFn: *const fn (*PhysicsBody, Vec3) void,
    getRotationFn: *const fn (*PhysicsBody) Quat,
    setRotationFn: *const fn (*PhysicsBody, Quat) void,
    setGravityFn: *const fn (*PhysicsEngine, Vec3) void,
    createTerrainFn: *const fn (*PhysicsEngine, [*]const f32, u32, u32) bool,
    getTerrainHeightFn: *const fn (*PhysicsEngine, f32, f32) f32,
    setBodyFlyingFn: *const fn (*PhysicsBody, bool) void,
    isBodyOnGroundFn: *const fn (*PhysicsBody) bool,
    raycastFn: *const fn (*PhysicsEngine, Vec3, Vec3) RaycastResult,

    name: [*:0]const u8,
    engine_type: EngineType,
    impl_ptr: *anyopaque,

    pub fn init(self: *PhysicsEngine) bool {
        return self.initFn(self);
    }

    pub fn deinit(self: *PhysicsEngine) void {
        self.deinitFn(self);
    }

    pub fn step(self: *PhysicsEngine, dt: f32) void {
        self.stepFn(self, dt);
    }

    pub fn createBody(self: *PhysicsEngine) ?*PhysicsBody {
        return self.createBodyFn(self);
    }

    pub fn destroyBody(self: *PhysicsEngine, body: *PhysicsBody) void {
        self.destroyBodyFn(self, body);
    }

    pub fn getPosition(self: *PhysicsEngine, body: *PhysicsBody) Vec3 {
        _ = self;
        return body.engine.?.getPositionFn(body);
    }

    pub fn setPosition(self: *PhysicsEngine, body: *PhysicsBody, pos: Vec3) void {
        _ = self;
        body.engine.?.setPositionFn(body, pos);
    }

    pub fn getVelocity(self: *PhysicsEngine, body: *PhysicsBody) Vec3 {
        _ = self;
        return body.engine.?.getVelocityFn(body);
    }

    pub fn setVelocity(self: *PhysicsEngine, body: *PhysicsBody, vel: Vec3) void {
        _ = self;
        body.engine.?.setVelocityFn(body, vel);
    }

    pub fn getRotation(self: *PhysicsEngine, body: *PhysicsBody) Quat {
        _ = self;
        return body.engine.?.getRotationFn(body);
    }

    pub fn setRotation(self: *PhysicsEngine, body: *PhysicsBody, rot: Quat) void {
        _ = self;
        body.engine.?.setRotationFn(body, rot);
    }

    pub fn setGravity(self: *PhysicsEngine, gravity: Vec3) void {
        self.setGravityFn(self, gravity);
    }

    pub fn createTerrain(self: *PhysicsEngine, heights: [*]const f32, w: u32, h: u32) bool {
        return self.createTerrainFn(self, heights, w, h);
    }

    pub fn getTerrainHeight(self: *PhysicsEngine, x: f32, y: f32) f32 {
        return self.getTerrainHeightFn(self, x, y);
    }

    pub fn setBodyFlying(self: *PhysicsEngine, body: *PhysicsBody, flying: bool) void {
        _ = self;
        body.engine.?.setBodyFlyingFn(body, flying);
    }

    pub fn isBodyOnGround(self: *PhysicsEngine, body: *PhysicsBody) bool {
        _ = self;
        return body.engine.?.isBodyOnGroundFn(body);
    }

    pub fn raycast(self: *PhysicsEngine, from: Vec3, to: Vec3) RaycastResult {
        return self.raycastFn(self, from, to);
    }
};

pub fn engineTypeName(et: EngineType) [*:0]const u8 {
    return switch (et) {
        .basic => "BasicPhysics",
        .pos => "POS",
        .ode => "ODE",
        .ubode => "ubODE",
        .bullet => "BulletSim",
    };
}
