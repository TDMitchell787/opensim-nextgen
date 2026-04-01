const std = @import("std");

// Network module for OpenSim Next
pub const llsd = @import("llsd.zig");
pub const asset_transfer = @import("asset_transfer.zig");

pub const NetworkError = error{
    InvalidPacket,
    ConnectionClosed,
    BufferTooSmall,
    InvalidProtocol,
    TimeoutError,
};

pub const NetworkConfig = struct {
    max_connections: u32 = 1000,
    buffer_size: usize = 8192,
    timeout_ms: u32 = 5000,
};

pub const NetworkManager = struct {
    config: NetworkConfig,
    allocator: std.mem.Allocator,
    
    pub fn init(allocator: std.mem.Allocator, config: NetworkConfig) NetworkManager {
        return NetworkManager{
            .config = config,
            .allocator = allocator,
        };
    }
    
    pub fn deinit(self: *NetworkManager) void {
        _ = self;
        // Cleanup resources
    }
    
    pub fn processPacket(self: *NetworkManager, data: []const u8) NetworkError!void {
        _ = self;
        _ = data;
        // Process network packet
    }
};

// Export for FFI
export fn network_init(max_connections: u32, buffer_size: usize, timeout_ms: u32) ?*NetworkManager {
    const allocator = std.heap.page_allocator;
    const config = NetworkConfig{
        .max_connections = max_connections,
        .buffer_size = buffer_size,
        .timeout_ms = timeout_ms,
    };
    
    const manager = allocator.create(NetworkManager) catch return null;
    manager.* = NetworkManager.init(allocator, config);
    return manager;
}

export fn network_deinit(manager: ?*NetworkManager) void {
    if (manager) |m| {
        m.deinit();
        const allocator = std.heap.page_allocator;
        allocator.destroy(m);
    }
}

export fn network_process_packet(manager: ?*NetworkManager, data: [*]const u8, len: usize) c_int {
    if (manager) |m| {
        const packet_data = data[0..len];
        m.processPacket(packet_data) catch return -1;
        return 0;
    }
    return -1;
}

test "network manager initialization" {
    const allocator = std.testing.allocator;
    const config = NetworkConfig{};
    var manager = NetworkManager.init(allocator, config);
    defer manager.deinit();
    
    try std.testing.expect(manager.config.max_connections == 1000);
}