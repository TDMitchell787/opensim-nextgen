const std = @import("std");
const root = @import("../root.zig");
const interface = root.interface;
const registry = root.registry;

pub const Vec3 = interface.Vec3;
pub const Quat = interface.Quat;
pub const PhysicsBody = interface.PhysicsBody;
pub const PhysicsEngine = interface.PhysicsEngine;
pub const EngineType = interface.EngineType;

const alloc = std.heap.page_allocator;

var global_engine: ?*PhysicsEngine = null;
var memory_initialized: bool = false;

pub export fn physics_create_engine(engine_type: u8) bool {
    if (global_engine != null) {
        physics_destroy_engine();
    }
    const et: EngineType = std.meta.intToEnum(EngineType, engine_type) catch return false;
    global_engine = registry.createEngine(et, alloc);
    if (global_engine) |eng| {
        memory_initialized = true;
        _ = eng.init();
        return true;
    }
    return false;
}

pub export fn physics_destroy_engine() void {
    if (global_engine) |eng| {
        registry.destroyEngine(eng, alloc);
        global_engine = null;
        memory_initialized = false;
    }
}

pub export fn physics_engine_name() [*:0]const u8 {
    if (global_engine) |eng| return eng.name;
    return "None";
}

pub export fn physics_engine_type() u8 {
    if (global_engine) |eng| return @intFromEnum(eng.engine_type);
    return 255;
}

pub export fn physics_memory_init(_: usize) void {
    if (memory_initialized) return;
    _ = physics_create_engine(@intFromEnum(EngineType.ode));
}

pub export fn physics_memory_deinit() void {
    physics_destroy_engine();
}

pub export fn physics_body_create() ?*PhysicsBody {
    if (!memory_initialized) physics_memory_init(0);
    const eng = global_engine orelse return null;
    return eng.createBody();
}

pub export fn physics_body_destroy(body: ?*PhysicsBody) void {
    if (body == null) return;
    const eng = global_engine orelse return;
    eng.destroyBody(body.?);
}

pub export fn physics_body_get_position(body: ?*PhysicsBody) Vec3 {
    if (body == null) return Vec3{ .x = 0, .y = 0, .z = 0 };
    const eng = body.?.engine orelse return Vec3{ .x = 0, .y = 0, .z = 0 };
    return eng.getPositionFn(body.?);
}

pub export fn physics_body_set_position(body: ?*PhysicsBody, pos: Vec3) void {
    if (body == null) return;
    const eng = body.?.engine orelse return;
    eng.setPositionFn(body.?, pos);
}

pub export fn physics_body_set_velocity(body: ?*PhysicsBody, vel: Vec3) void {
    if (body == null) return;
    const eng = body.?.engine orelse return;
    eng.setVelocityFn(body.?, vel);
}

pub export fn physics_body_get_velocity(body: ?*PhysicsBody) Vec3 {
    if (body == null) return Vec3{ .x = 0, .y = 0, .z = 0 };
    const eng = body.?.engine orelse return Vec3{ .x = 0, .y = 0, .z = 0 };
    return eng.getVelocityFn(body.?);
}

pub export fn physics_body_set_rotation(body: ?*PhysicsBody, rot: Quat) void {
    if (body == null) return;
    const eng = body.?.engine orelse return;
    eng.setRotationFn(body.?, rot);
}

pub export fn physics_body_get_rotation(body: ?*PhysicsBody) Quat {
    if (body == null) return Quat{ .x = 0, .y = 0, .z = 0, .w = 1 };
    const eng = body.?.engine orelse return Quat{ .x = 0, .y = 0, .z = 0, .w = 1 };
    return eng.getRotationFn(body.?);
}

pub export fn physics_step(timestep: f32) void {
    if (global_engine) |eng| eng.step(timestep);
}

pub export fn physics_set_gravity(gravity: Vec3) void {
    if (global_engine) |eng| eng.setGravity(gravity);
}

pub export fn physics_create_terrain(heights: [*]const f32, w: u32, h: u32) bool {
    const eng = global_engine orelse return false;
    return eng.createTerrain(heights, w, h);
}

pub export fn physics_get_terrain_height(x: f32, y: f32) f32 {
    const eng = global_engine orelse return 0.0;
    return eng.getTerrainHeight(x, y);
}

pub export fn physics_body_set_flying(body: ?*PhysicsBody, flying: bool) void {
    if (body == null) return;
    const eng = body.?.engine orelse return;
    eng.setBodyFlyingFn(body.?, flying);
}

pub export fn physics_body_is_on_ground(body: ?*PhysicsBody) bool {
    if (body == null) return true;
    const eng = body.?.engine orelse return true;
    return eng.isBodyOnGroundFn(body.?);
}

const collision_world = @import("../collision/world.zig");
const collision_shapes = @import("../collision/shapes.zig");
const collision_quat = @import("../collision/quat.zig");

pub export fn physics_collision_init() bool {
    return collision_world.initGlobalWorldManager(alloc);
}

pub export fn physics_collision_deinit() void {
    collision_world.deinitGlobalWorldManager(alloc);
}

pub export fn physics_create_collision_world(region_id_hi: u64, region_id_lo: u64) bool {
    const wm = collision_world.getGlobalWorldManager() orelse {
        if (!collision_world.initGlobalWorldManager(alloc)) return false;
        return physics_create_collision_world(region_id_hi, region_id_lo);
    };
    return wm.createWorld(region_id_hi, region_id_lo) != null;
}

pub export fn physics_destroy_collision_world(region_id_hi: u64, region_id_lo: u64) void {
    const wm = collision_world.getGlobalWorldManager() orelse return;
    wm.destroyWorld(region_id_hi, region_id_lo);
}

pub export fn physics_world_add_box(
    region_hi: u64,
    region_lo: u64,
    local_id: u32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    rot_x: f32,
    rot_y: f32,
    rot_z: f32,
    rot_w: f32,
    half_x: f32,
    half_y: f32,
    half_z: f32,
    flags: u32,
) bool {
    const wm = collision_world.getGlobalWorldManager() orelse return false;
    const w = wm.getWorld(region_hi, region_lo) orelse return false;
    w.addBody(.{
        .local_id = local_id,
        .position = .{ .x = pos_x, .y = pos_y, .z = pos_z },
        .rotation = .{ .x = rot_x, .y = rot_y, .z = rot_z, .w = rot_w },
        .shape = .{ .box = .{ .half_extents = .{ .x = half_x, .y = half_y, .z = half_z } } },
        .flags = flags,
        .is_avatar = false,
    });
    return true;
}

pub export fn physics_world_add_sphere(
    region_hi: u64,
    region_lo: u64,
    local_id: u32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    radius: f32,
    flags: u32,
) bool {
    const wm = collision_world.getGlobalWorldManager() orelse return false;
    const w = wm.getWorld(region_hi, region_lo) orelse return false;
    w.addBody(.{
        .local_id = local_id,
        .position = .{ .x = pos_x, .y = pos_y, .z = pos_z },
        .rotation = collision_quat.identity,
        .shape = .{ .sphere = .{ .radius = radius } },
        .flags = flags,
        .is_avatar = false,
    });
    return true;
}

pub export fn physics_world_add_avatar(
    region_hi: u64,
    region_lo: u64,
    local_id: u32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    radius: f32,
    height: f32,
) bool {
    const wm = collision_world.getGlobalWorldManager() orelse return false;
    const w = wm.getWorld(region_hi, region_lo) orelse return false;
    const half_height = (height * 0.5) - radius;
    w.addBody(.{
        .local_id = local_id,
        .position = .{ .x = pos_x, .y = pos_y, .z = pos_z },
        .rotation = collision_quat.identity,
        .shape = .{ .capsule = .{ .radius = radius, .half_height = if (half_height > 0) half_height else 0.1 } },
        .flags = collision_shapes.CollisionBody.FLAG_AVATAR,
        .is_avatar = true,
    });
    return true;
}

pub export fn physics_world_set_avatar_position(
    region_hi: u64,
    region_lo: u64,
    local_id: u32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
) void {
    const wm = collision_world.getGlobalWorldManager() orelse return;
    const w = wm.getWorld(region_hi, region_lo) orelse return;
    w.updateTransform(local_id, .{ .x = pos_x, .y = pos_y, .z = pos_z }, collision_quat.identity);
}

pub export fn physics_world_remove_body(
    region_hi: u64,
    region_lo: u64,
    local_id: u32,
) void {
    const wm = collision_world.getGlobalWorldManager() orelse return;
    const w = wm.getWorld(region_hi, region_lo) orelse return;
    w.removeBody(local_id);
}

pub export fn physics_world_update_body_transform(
    region_hi: u64,
    region_lo: u64,
    local_id: u32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    rot_x: f32,
    rot_y: f32,
    rot_z: f32,
    rot_w: f32,
) void {
    const wm = collision_world.getGlobalWorldManager() orelse return;
    const w = wm.getWorld(region_hi, region_lo) orelse return;
    w.updateTransform(local_id, .{ .x = pos_x, .y = pos_y, .z = pos_z }, .{ .x = rot_x, .y = rot_y, .z = rot_z, .w = rot_w });
}

pub export fn physics_world_set_terrain(
    region_hi: u64,
    region_lo: u64,
    heights: [*]const f32,
    width: u32,
    depth: u32,
) bool {
    const wm = collision_world.getGlobalWorldManager() orelse return false;
    const w = wm.getWorld(region_hi, region_lo) orelse return false;
    return w.setTerrain(heights, width, depth);
}

pub export fn physics_world_collision_step(
    region_hi: u64,
    region_lo: u64,
) u32 {
    const wm = collision_world.getGlobalWorldManager() orelse return 0;
    const w = wm.getWorld(region_hi, region_lo) orelse return 0;
    return w.step();
}

pub export fn physics_world_get_contact(
    region_hi: u64,
    region_lo: u64,
    index: u32,
    out_id_a: *u32,
    out_id_b: *u32,
    out_nx: *f32,
    out_ny: *f32,
    out_nz: *f32,
    out_depth: *f32,
    out_px: *f32,
    out_py: *f32,
    out_pz: *f32,
) bool {
    const wm = collision_world.getGlobalWorldManager() orelse return false;
    const w = wm.getWorld(region_hi, region_lo) orelse return false;
    const contact = w.getContact(index) orelse return false;
    out_id_a.* = contact.body_id_a;
    out_id_b.* = contact.body_id_b;
    out_nx.* = contact.normal.x;
    out_ny.* = contact.normal.y;
    out_nz.* = contact.normal.z;
    out_depth.* = contact.depth;
    out_px.* = contact.point.x;
    out_py.* = contact.point.y;
    out_pz.* = contact.point.z;
    return true;
}

pub export fn physics_world_raycast_down(
    region_hi: u64,
    region_lo: u64,
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    max_dist: f32,
    out_hit: *bool,
    out_body_id: *u32,
    out_px: *f32,
    out_py: *f32,
    out_pz: *f32,
    out_nx: *f32,
    out_ny: *f32,
    out_nz: *f32,
) void {
    out_hit.* = false;
    const wm = collision_world.getGlobalWorldManager() orelse return;
    const w = wm.getWorld(region_hi, region_lo) orelse return;
    const origin = Vec3{ .x = origin_x, .y = origin_y, .z = origin_z };
    if (w.castRayDown(origin, max_dist)) |hit| {
        out_hit.* = true;
        out_body_id.* = hit.body_id;
        out_px.* = hit.point.x;
        out_py.* = hit.point.y;
        out_pz.* = hit.point.z;
        out_nx.* = hit.normal.x;
        out_ny.* = hit.normal.y;
        out_nz.* = hit.normal.z;
    }
}

const BulletEngine = root.engines.bullet.BulletEngine;

const BSVector3 = extern struct { x: f32, y: f32, z: f32 };
const BSQuaternion = extern struct { x: f32, y: f32, z: f32, w: f32 };

extern "C" fn CreateHullShape2(world: *anyopaque, hullCount: c_int, hulls: [*]const f32) ?*anyopaque;
extern "C" fn CreateMeshShape2(world: *anyopaque, indicesCount: c_int, indices: [*]const c_int, verticesCount: c_int, vertices: [*]const f32) ?*anyopaque;
extern "C" fn CreateBodyFromShape2(world: *anyopaque, shape: *anyopaque, id: u32, pos: BSVector3, rot: BSQuaternion) ?*anyopaque;
extern "C" fn DeleteCollisionShape2(world: *anyopaque, shape: *anyopaque) bool;
extern "C" fn AddObjectToWorld2(world: *anyopaque, body: *anyopaque) bool;
extern "C" fn UpdateSingleAabb2(world: *anyopaque, body: *anyopaque) void;
extern "C" fn ForceActivationState2(body: *anyopaque, state: c_int) void;
extern "C" fn SetMassProps2(body: *anyopaque, mass: f32, inertia: BSVector3) void;
extern "C" fn SetCollisionGroupMask2(body: *anyopaque, filter: u32, mask: u32) bool;

fn getBulletEngine() ?*BulletEngine {
    const eng = global_engine orelse return null;
    if (eng.engine_type != .bullet) return null;
    return @ptrCast(@alignCast(eng.impl_ptr));
}

pub export fn physics_create_hull_shape(hull_count: c_int, hulls: [*]const f32) ?*anyopaque {
    const be = getBulletEngine() orelse return null;
    const world = be.world orelse return null;
    return CreateHullShape2(world, hull_count, hulls);
}

pub export fn physics_create_mesh_shape(indices_count: c_int, indices: [*]const c_int, vertices_count: c_int, vertices: [*]const f32) ?*anyopaque {
    const be = getBulletEngine() orelse return null;
    const world = be.world orelse return null;
    return CreateMeshShape2(world, indices_count, indices, vertices_count, vertices);
}

pub export fn physics_create_body_from_shape(shape: *anyopaque, id: u32, pos_x: f32, pos_y: f32, pos_z: f32, rot_x: f32, rot_y: f32, rot_z: f32, rot_w: f32) ?*anyopaque {
    const be = getBulletEngine() orelse return null;
    const world = be.world orelse return null;
    const body = CreateBodyFromShape2(world, shape, id, .{ .x = pos_x, .y = pos_y, .z = pos_z }, .{ .x = rot_x, .y = rot_y, .z = rot_z, .w = rot_w }) orelse return null;
    if (!AddObjectToWorld2(world, body)) return null;
    SetMassProps2(body, 0.0, .{ .x = 0, .y = 0, .z = 0 });
    ForceActivationState2(body, 4);
    _ = SetCollisionGroupMask2(body, 0x2000, 0x7FFF);
    UpdateSingleAabb2(world, body);
    return body;
}

pub export fn physics_delete_collision_shape(shape: *anyopaque) bool {
    const be = getBulletEngine() orelse return false;
    const world = be.world orelse return false;
    return DeleteCollisionShape2(world, shape);
}

pub export fn physics_raycast(
    from_x: f32, from_y: f32, from_z: f32,
    to_x: f32, to_y: f32, to_z: f32,
    out_hit: *bool, out_id: *u32, out_fraction: *f32,
    out_px: *f32, out_py: *f32, out_pz: *f32,
    out_nx: *f32, out_ny: *f32, out_nz: *f32,
) void {
    const eng = global_engine orelse {
        out_hit.* = false;
        return;
    };
    const result = eng.raycast(
        Vec3{ .x = from_x, .y = from_y, .z = from_z },
        Vec3{ .x = to_x, .y = to_y, .z = to_z },
    );
    out_hit.* = result.hit;
    out_id.* = result.body_id;
    out_fraction.* = result.fraction;
    out_px.* = result.point.x;
    out_py.* = result.point.y;
    out_pz.* = result.point.z;
    out_nx.* = result.normal.x;
    out_ny.* = result.normal.y;
    out_nz.* = result.normal.z;
}
