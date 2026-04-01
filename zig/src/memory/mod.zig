const std = @import("std");

pub const MemoryPool = struct {
    allocator: std.mem.Allocator,
    pool_size: usize,
    used: usize,
    buffer: []u8,

    pub fn init(allocator: std.mem.Allocator, pool_size: usize) !MemoryPool {
        const buffer = try allocator.alloc(u8, pool_size);
        return MemoryPool{
            .allocator = allocator,
            .pool_size = pool_size,
            .used = 0,
            .buffer = buffer,
        };
    }

    pub fn deinit(self: *MemoryPool) void {
        self.allocator.free(self.buffer);
    }

    pub fn allocate(self: *MemoryPool, size: usize) ?[]u8 {
        if (self.used + size > self.pool_size) return null;
        const slice = self.buffer[self.used..self.used + size];
        self.used += size;
        return slice;
    }

    pub fn reset(self: *MemoryPool) void {
        self.used = 0;
    }
};

pub const ArenaAllocator = struct {
    arena: std.heap.ArenaAllocator,

    pub fn init(backing_allocator: std.mem.Allocator) ArenaAllocator {
        return ArenaAllocator{
            .arena = std.heap.ArenaAllocator.init(backing_allocator),
        };
    }

    pub fn deinit(self: *ArenaAllocator) void {
        self.arena.deinit();
    }

    pub fn allocator(self: *ArenaAllocator) std.mem.Allocator {
        return self.arena.allocator();
    }

    pub fn reset(self: *ArenaAllocator) void {
        self.arena.deinit();
        self.arena = std.heap.ArenaAllocator.init(self.arena.child_allocator);
    }
};

test "memory pool allocation" {
    const allocator = std.testing.allocator;
    var pool = try MemoryPool.init(allocator, 1024);
    defer pool.deinit();

    const slice1 = pool.allocate(100) orelse return error.AllocationFailed;
    const slice2 = pool.allocate(200) orelse return error.AllocationFailed;

    try std.testing.expect(slice1.len == 100);
    try std.testing.expect(slice2.len == 200);
    try std.testing.expect(pool.used == 300);
}

test "arena allocator" {
    const allocator = std.testing.allocator;
    var arena = ArenaAllocator.init(allocator);
    defer arena.deinit();

    const arena_allocator = arena.allocator();
    const slice = try arena_allocator.alloc(u8, 100);
    try std.testing.expect(slice.len == 100);
} 