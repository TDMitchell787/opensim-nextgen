pub const interface = @import("interface.zig");
pub const registry = @import("registry.zig");
pub const collision = @import("collision/mod.zig");
pub const engines = struct {
    pub const basic = @import("engines/basic.zig");
    pub const pos = @import("engines/pos.zig");
    pub const ode = @import("engines/ode.zig");
    pub const ubode = @import("engines/ubode.zig");
    pub const bullet = @import("engines/bullet.zig");
};

const main = @import("engine/main.zig");

pub const physics_create_engine = main.physics_create_engine;
pub const physics_destroy_engine = main.physics_destroy_engine;
pub const physics_engine_name = main.physics_engine_name;
pub const physics_engine_type = main.physics_engine_type;
pub const physics_memory_init = main.physics_memory_init;
pub const physics_memory_deinit = main.physics_memory_deinit;
pub const physics_body_create = main.physics_body_create;
pub const physics_body_destroy = main.physics_body_destroy;
pub const physics_body_get_position = main.physics_body_get_position;
pub const physics_body_set_position = main.physics_body_set_position;
pub const physics_body_set_velocity = main.physics_body_set_velocity;
pub const physics_body_get_velocity = main.physics_body_get_velocity;
pub const physics_body_set_rotation = main.physics_body_set_rotation;
pub const physics_body_get_rotation = main.physics_body_get_rotation;
pub const physics_step = main.physics_step;
pub const physics_set_gravity = main.physics_set_gravity;
pub const physics_create_terrain = main.physics_create_terrain;
pub const physics_get_terrain_height = main.physics_get_terrain_height;
pub const physics_body_set_flying = main.physics_body_set_flying;
pub const physics_body_is_on_ground = main.physics_body_is_on_ground;
pub const physics_create_hull_shape = main.physics_create_hull_shape;
pub const physics_create_mesh_shape = main.physics_create_mesh_shape;
pub const physics_create_body_from_shape = main.physics_create_body_from_shape;
pub const physics_delete_collision_shape = main.physics_delete_collision_shape;
pub const physics_raycast = main.physics_raycast;

comptime {
    _ = main;
    _ = collision;
    _ = engines.basic;
    _ = engines.pos;
    _ = engines.ode;
    _ = engines.ubode;
    _ = engines.bullet;
}

test {
    _ = interface;
    _ = registry;
    _ = collision;
    _ = engines.basic;
    _ = engines.pos;
    _ = engines.ode;
    _ = engines.ubode;
    _ = engines.bullet;
    _ = main;
}
