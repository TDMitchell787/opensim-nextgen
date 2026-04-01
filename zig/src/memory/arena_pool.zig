const std = @import("std");

/// Arena pool allocator for performance-critical operations
/// Provides fast allocation/deallocation with minimal fragmentation
pub const ArenaPool = struct {
    const Pool = struct {
        data: []u8,
        used: usize,
        next: ?*Pool,
    };

    allocator: std.mem.Allocator,
    current_pool: ?*Pool,
    pool_size: usize,
    alignment: u29,

    pub fn init(allocator: std.mem.Allocator, pool_size: usize, alignment: u29) ArenaPool {
        return ArenaPool{
            .allocator = allocator,
            .current_pool = null,
            .pool_size = pool_size,
            .alignment = alignment,
        };
    }

    pub fn deinit(self: *ArenaPool) void {
        var pool = self.current_pool;
        while (pool) |current| {
            const next = current.next;
            self.allocator.free(current.data);
            self.allocator.destroy(current);
            pool = next;
        }
        self.current_pool = null;
    }

    pub fn allocator(self: *ArenaPool) std.mem.Allocator {
        return .{
            .ptr = self,
            .vtable = &.{
                .alloc = alloc,
                .resize = resize,
                .free = free,
            },
        };
    }

    fn alloc(ctx: *anyopaque, len: usize, alignment: u29, ret_addr: usize) ?[*]u8 {
        const self = @ptrCast(*ArenaPool, @alignCast(@alignOf(ArenaPool), ctx));
        
        // Ensure alignment is satisfied
        const aligned_len = std.mem.alignForward(len, alignment);
        
        // Try to allocate from current pool
        if (self.current_pool) |pool| {
            const aligned_offset = std.mem.alignForward(pool.used, alignment);
            if (aligned_offset + aligned_len <= pool.data.len) {
                const result = pool.data.ptr + aligned_offset;
                pool.used = aligned_offset + aligned_len;
                return result;
            }
        }

        // Need to create a new pool
        const new_pool_size = @max(self.pool_size, aligned_len);
        const new_pool = self.allocator.create(Pool) catch return null;
        new_pool.data = self.allocator.alloc(u8, new_pool_size) catch {
            self.allocator.destroy(new_pool);
            return null;
        };
        new_pool.used = aligned_len;
        new_pool.next = self.current_pool;
        self.current_pool = new_pool;

        return new_pool.data.ptr;
    }

    fn resize(ctx: *anyopaque, buf: []u8, buf_align: u29, new_len: usize, ret_addr: usize) bool {
        const self = @ptrCast(*ArenaPool, @alignCast(@alignOf(ArenaPool), ctx));
        
        // Can only resize if it's the last allocation in the current pool
        if (self.current_pool) |pool| {
            const aligned_offset = std.mem.alignForward(pool.used - buf.len, buf_align);
            const buf_start = pool.data.ptr + aligned_offset;
            
            if (buf.ptr == buf_start and aligned_offset + new_len <= pool.data.len) {
                pool.used = aligned_offset + new_len;
                return true;
            }
        }
        
        return false;
    }

    fn free(ctx: *anyopaque, buf: []u8, buf_align: u29, ret_addr: usize) void {
        // Arena allocators don't free individual allocations
        // Memory is freed when the entire arena is deinitialized
        _ = ctx;
        _ = buf;
        _ = buf_align;
        _ = ret_addr;
    }

    /// Reset the arena, making all memory available for new allocations
    pub fn reset(self: *ArenaPool) void {
        var pool = self.current_pool;
        while (pool) |current| {
            current.used = 0;
            pool = current.next;
        }
    }

    /// Get memory usage statistics
    pub fn getStats(self: *ArenaPool) struct {
        total_allocated: usize,
        total_used: usize,
        pool_count: usize,
    } {
        var total_allocated: usize = 0;
        var total_used: usize = 0;
        var pool_count: usize = 0;
        
        var pool = self.current_pool;
        while (pool) |current| {
            total_allocated += current.data.len;
            total_used += current.used;
            pool_count += 1;
            pool = current.next;
        }
        
        return .{
            .total_allocated = total_allocated,
            .total_used = total_used,
            .pool_count = pool_count,
        };
    }
};

/// Object pool for frequently allocated/deallocated objects
pub fn ObjectPool(comptime T: type) type {
    return struct {
        const Self = @This();
        
        allocator: std.mem.Allocator,
        free_list: ?*Node,
        pool_size: usize,
        
        const Node = struct {
            data: T,
            next: ?*Node,
        };
        
        pub fn init(allocator: std.mem.Allocator, pool_size: usize) !Self {
            return Self{
                .allocator = allocator,
                .free_list = null,
                .pool_size = pool_size,
            };
        }
        
        pub fn deinit(self: *Self) void {
            // Free all nodes in the free list
            var node = self.free_list;
            while (node) |current| {
                const next = current.next;
                self.allocator.destroy(current);
                node = next;
            }
            self.free_list = null;
        }
        
        pub fn acquire(self: *Self) !*T {
            if (self.free_list) |node| {
                self.free_list = node.next;
                return &node.data;
            }
            
            // Create new node
            const new_node = try self.allocator.create(Node);
            new_node.next = null;
            return &new_node.data;
        }
        
        pub fn release(self: *Self, obj: *T) void {
            const node = @fieldParentPtr(Node, "data", obj);
            node.next = self.free_list;
            self.free_list = node;
        }
    };
}

test "ArenaPool basic functionality" {
    const testing = std.testing;
    
    var arena = ArenaPool.init(testing.allocator, 1024, 8);
    defer arena.deinit();
    
    const allocator = arena.allocator();
    
    // Allocate some memory
    const buf1 = try allocator.alloc(u8, 100);
    const buf2 = try allocator.alloc(u8, 200);
    
    try testing.expect(buf1.len == 100);
    try testing.expect(buf2.len == 200);
    
    // Test resize
    const resized = allocator.resize(buf1, 8, 150);
    try testing.expect(resized);
    
    const stats = arena.getStats();
    try testing.expect(stats.pool_count > 0);
    try testing.expect(stats.total_used > 0);
}

test "ObjectPool basic functionality" {
    const testing = std.testing;
    
    var pool = try ObjectPool(u32).init(testing.allocator, 10);
    defer pool.deinit();
    
    // Acquire objects
    const obj1 = try pool.acquire();
    const obj2 = try pool.acquire();
    
    obj1.* = 42;
    obj2.* = 84;
    
    try testing.expect(obj1.* == 42);
    try testing.expect(obj2.* == 84);
    
    // Release and reacquire
    pool.release(obj1);
    const obj3 = try pool.acquire();
    
    // Should reuse the released object
    obj3.* = 123;
    try testing.expect(obj3.* == 123);
} 