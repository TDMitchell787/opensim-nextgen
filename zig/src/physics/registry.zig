const std = @import("std");
const interface = @import("interface.zig");
const EngineType = interface.EngineType;
const PhysicsEngine = interface.PhysicsEngine;
const basic = @import("engines/basic.zig");
const pos = @import("engines/pos.zig");
const ode = @import("engines/ode.zig");
const ubode = @import("engines/ubode.zig");
const bullet = @import("engines/bullet.zig");

pub fn createEngine(engine_type: EngineType, alloc: std.mem.Allocator) ?*PhysicsEngine {
    return switch (engine_type) {
        .basic => basic.BasicEngine.create(alloc),
        .pos => pos.POSEngine.create(alloc),
        .ode => ode.OdeEngine.create(alloc),
        .ubode => ubode.UbOdeEngine.create(alloc),
        .bullet => bullet.BulletEngine.create(alloc),
    };
}

pub fn destroyEngine(engine: *PhysicsEngine, alloc: std.mem.Allocator) void {
    engine.deinit();
    alloc.destroy(engine);
}
