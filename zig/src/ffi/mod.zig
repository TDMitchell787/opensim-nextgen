const std = @import("std");
const empty: [1]u8 = [_]u8{0};

pub const FFIError = error{
    InvalidHandle,
    AllocationFailed,
    InvalidParameter,
    BufferTooSmall,
    NotInitialized,
    NetworkError,
    PhysicsError,
    ProtocolError,
    TimeoutError,
    InternalError,
};

// Standardized error codes for FFI
pub const FFI_ERROR_SUCCESS: c_int = 0;
pub const FFI_ERROR_INVALID_HANDLE: c_int = 1;
pub const FFI_ERROR_ALLOCATION_FAILED: c_int = 2;
pub const FFI_ERROR_INVALID_PARAMETER: c_int = 3;
pub const FFI_ERROR_BUFFER_TOO_SMALL: c_int = 4;
pub const FFI_ERROR_NOT_INITIALIZED: c_int = 5;
pub const FFI_ERROR_NETWORK_ERROR: c_int = 6;
pub const FFI_ERROR_PHYSICS_ERROR: c_int = 7;
pub const FFI_ERROR_PROTOCOL_ERROR: c_int = 8;
pub const FFI_ERROR_TIMEOUT_ERROR: c_int = 9;
pub const FFI_ERROR_INTERNAL_ERROR: c_int = 10;

// Make FFIResult extern compatible for FFI functions
pub const FFIResult = extern union {
    Success: void,
    ErrorCode: c_int,
    Data: [*]const u8,
    DataLen: usize,
    Handle: u64,
};

// FFI result wrapper for better error handling
pub const FFIResultWrapper = extern struct {
    error_code: c_int,
    data: [*]const u8,
    data_len: usize,
    handle: u64,
};

// Global logging state
var log_buffer: std.ArrayList(u8) = undefined;
var log_initialized: bool = false;

// Initialize logging
fn initLogging() void {
    if (!log_initialized) {
        log_buffer = .{};
        log_initialized = true;
    }
}

// Log error with context
fn logError(comptime fmt: []const u8, args: anytype) void {
    initLogging();
    const timestamp = std.time.timestamp();
    const log_entry = std.fmt.allocPrint(
        std.heap.page_allocator,
        "[{d}] ERROR: " ++ fmt ++ "\n",
        .{timestamp} ++ args,
    ) catch return;
    defer std.heap.page_allocator.free(log_entry);
    
    log_buffer.appendSlice(std.heap.page_allocator, log_entry) catch {};
    // Also print to stderr for immediate visibility
    std.debug.print("{s}", .{log_entry});
}

// Get error message for error code
fn getErrorMessage(error_code: c_int) []const u8 {
    return switch (error_code) {
        FFI_ERROR_SUCCESS => "Success",
        FFI_ERROR_INVALID_HANDLE => "Invalid handle",
        FFI_ERROR_ALLOCATION_FAILED => "Memory allocation failed",
        FFI_ERROR_INVALID_PARAMETER => "Invalid parameter",
        FFI_ERROR_BUFFER_TOO_SMALL => "Buffer too small",
        FFI_ERROR_NOT_INITIALIZED => "Not initialized",
        FFI_ERROR_NETWORK_ERROR => "Network error",
        FFI_ERROR_PHYSICS_ERROR => "Physics error",
        FFI_ERROR_PROTOCOL_ERROR => "Protocol error",
        FFI_ERROR_TIMEOUT_ERROR => "Timeout error",
        FFI_ERROR_INTERNAL_ERROR => "Internal error",
        else => "Unknown error",
    };
}

pub const FFIContext = struct {
    allocator: std.mem.Allocator,
    initialized: bool,
    error_count: u32,

    pub fn init(allocator: std.mem.Allocator) FFIContext {
        return FFIContext{
            .allocator = allocator,
            .initialized = true,
            .error_count = 0,
        };
    }

    pub fn deinit(self: *FFIContext) void {
        self.initialized = false;
    }

    pub fn isValid(self: *const FFIContext) bool {
        return self.initialized;
    }

    pub fn recordError(self: *FFIContext, error_code: c_int, context: []const u8) void {
        self.error_count += 1;
        logError("FFI Error {d}: {s} - {s}", .{ error_code, getErrorMessage(error_code), context });
    }
};

pub const FFIBuffer = struct {
    data: []u8,
    size: usize,
    capacity: usize,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, capacity: usize) !FFIBuffer {
        const data = try allocator.alloc(u8, capacity);
        return FFIBuffer{
            .data = data,
            .size = 0,
            .capacity = capacity,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *FFIBuffer) void {
        self.allocator.free(self.data);
    }

    pub fn write(self: *FFIBuffer, input: []const u8) !usize {
        const write_size = @min(input.len, self.capacity - self.size);
        if (write_size == 0) return FFIError.BufferTooSmall;

        @memcpy(self.data[self.size..self.size + write_size], input[0..write_size]);
        self.size += write_size;
        return write_size;
    }

    pub fn read(self: *FFIBuffer, output: []u8) !usize {
        const read_size = @min(output.len, self.size);
        if (read_size == 0) return 0;

        @memcpy(output[0..read_size], self.data[0..read_size]);
        return read_size;
    }

    pub fn reset(self: *FFIBuffer) void {
        self.size = 0;
    }

    pub fn getData(self: *const FFIBuffer) []const u8 {
        return self.data[0..self.size];
    }
};

pub const FFIHandle = struct {
    id: u64,
    context: *FFIContext,
    data: ?*anyopaque,

    pub fn init(id: u64, context: *FFIContext, data: ?*anyopaque) FFIHandle {
        return FFIHandle{
            .id = id,
            .context = context,
            .data = data,
        };
    }

    pub fn isValid(self: *const FFIHandle) bool {
        return self.context.isValid();
    }
};

pub const FFIManager = struct {
    allocator: std.mem.Allocator,
    handles: std.AutoHashMap(u64, FFIHandle),
    next_id: u64,
    context: *FFIContext,

    pub fn init(allocator: std.mem.Allocator, context: *FFIContext) FFIManager {
        return FFIManager{
            .allocator = allocator,
            .handles = std.AutoHashMap(u64, FFIHandle).init(allocator),
            .next_id = 1,
            .context = context,
        };
    }

    pub fn deinit(self: *FFIManager) void {
        self.handles.deinit();
    }

    pub fn createHandle(self: *FFIManager, context: *FFIContext, data: ?*anyopaque) !u64 {
        const id = self.next_id;
        self.next_id += 1;

        const handle = FFIHandle.init(id, context, data);
        try self.handles.put(id, handle);
        return id;
    }

    pub fn getHandle(self: *const FFIManager, id: u64) ?FFIHandle {
        return self.handles.get(id);
    }

    pub fn removeHandle(self: *FFIManager, id: u64) bool {
        return self.handles.remove(id);
    }

    pub fn cleanup(self: *FFIManager) void {
        var it = self.handles.iterator();
        while (it.next()) |entry| {
            if (!entry.value.isValid()) {
                _ = self.handles.remove(entry.key);
            }
        }
    }
};

// Global FFI manager instance
var global_manager: ?FFIManager = null;
var global_context: ?FFIContext = null;

// Export functions for Rust FFI with robust error handling
export fn ffi_init() FFIResultWrapper {
    if (global_context != null) {
        return FFIResultWrapper{
            .error_code = FFI_ERROR_SUCCESS,
            .data = &empty,
            .data_len = 0,
            .handle = 0,
        };
    }

    const allocator = std.heap.page_allocator;
    global_context = FFIContext.init(allocator);
    global_manager = FFIManager.init(allocator, &global_context.?);

    logError("FFI initialized successfully", .{});
    
    return FFIResultWrapper{
        .error_code = FFI_ERROR_SUCCESS,
        .data = &empty,
        .data_len = 0,
        .handle = 0,
    };
}

export fn ffi_cleanup() FFIResultWrapper {
    if (global_manager) |*manager| {
        manager.deinit();
        global_manager = null;
    }
    if (global_context) |*context| {
        context.deinit();
        global_context = null;
    }

    logError("FFI cleaned up successfully", .{});
    
    return FFIResultWrapper{
        .error_code = FFI_ERROR_SUCCESS,
        .data = &empty,
        .data_len = 0,
        .handle = 0,
    };
}

export fn ffi_create_buffer(capacity: usize) FFIResultWrapper {
    if (global_context == null) {
        logError("FFI not initialized", .{});
        return FFIResultWrapper{
            .error_code = FFI_ERROR_NOT_INITIALIZED,
            .data = &empty,
            .data_len = 0,
            .handle = 0,
        };
    }

    const allocator = std.heap.page_allocator;
    const buffer = FFIBuffer.init(allocator, capacity) catch {
        const error_code = FFI_ERROR_INTERNAL_ERROR;
        global_context.?.recordError(error_code, "Failed to create buffer");
        return FFIResultWrapper{
            .error_code = error_code,
            .data = &empty,
            .data_len = 0,
            .handle = 0,
        };
    };

    const handle_id = global_manager.?.createHandle(&global_context.?, @constCast(&buffer)) catch {
        const error_code = FFI_ERROR_INTERNAL_ERROR;
        global_context.?.recordError(error_code, "Failed to create handle for buffer");
        return FFIResultWrapper{
            .error_code = error_code,
            .data = &empty,
            .data_len = 0,
            .handle = 0,
        };
    };

    logError("Buffer created with handle {d}", .{handle_id});
    
    return FFIResultWrapper{
        .error_code = FFI_ERROR_SUCCESS,
        .data = &empty,
        .data_len = 0,
        .handle = handle_id,
    };
}

export fn ffi_destroy_buffer(buffer_id: u64) FFIResultWrapper {
    if (global_context == null) {
        logError("FFI not initialized", .{});
        return FFIResultWrapper{
            .error_code = FFI_ERROR_NOT_INITIALIZED,
            .data = &empty,
            .data_len = 0,
            .handle = 0,
        };
    }

    const removed = global_manager.?.removeHandle(buffer_id);
    if (!removed) {
        global_context.?.recordError(FFI_ERROR_INVALID_HANDLE, "Failed to destroy buffer: invalid handle");
        return FFIResultWrapper{
            .error_code = FFI_ERROR_INVALID_HANDLE,
            .data = &empty,
            .data_len = 0,
            .handle = 0,
        };
    }

    logError("Buffer {d} destroyed successfully", .{buffer_id});
    
    return FFIResultWrapper{
        .error_code = FFI_ERROR_SUCCESS,
        .data = &empty,
        .data_len = 0,
        .handle = 0,
    };
}

export fn ffi_get_error_count() c_int {
    if (global_context) |context| {
        return @intCast(context.error_count);
    }
    return 0;
}

export fn ffi_get_last_error() [*]const u8 {
    if (log_buffer.items.len > 0) {
        return log_buffer.items.ptr;
    }
    return "No errors logged";
}

export fn ffi_clear_logs() void {
    if (log_initialized) {
        log_buffer.clearRetainingCapacity();
    }
}

test "FFI context initialization" {
    const allocator = std.testing.allocator;
    var context = FFIContext.init(allocator);
    defer context.deinit();

    try std.testing.expect(context.isValid());
}

test "FFI buffer operations" {
    const allocator = std.testing.allocator;
    var buffer = try FFIBuffer.init(allocator, 1024);
    defer buffer.deinit();

    const test_data = "Hello, FFI!";
    const written = try buffer.write(test_data);
    try std.testing.expect(written == test_data.len);

    var output: [20]u8 = undefined;
    const read = try buffer.read(&output);
    try std.testing.expect(read == test_data.len);
    try std.testing.expectEqualStrings(test_data, output[0..read]);
}

test "FFI manager handle operations" {
    const allocator = std.testing.allocator;
    var context = FFIContext.init(allocator);
    defer context.deinit();

    var manager = FFIManager.init(allocator, &context);
    defer manager.deinit();

    const handle_id = try manager.createHandle(&context, null);
    try std.testing.expect(handle_id > 0);

    const handle = manager.getHandle(handle_id);
    try std.testing.expect(handle != null);
    try std.testing.expect(handle.?.isValid());

    const removed = manager.removeHandle(handle_id);
    try std.testing.expect(removed == true);
}

test "FFI error handling" {
    const allocator = std.testing.allocator;
    var context = FFIContext.init(allocator);
    defer context.deinit();

    context.recordError(FFI_ERROR_INVALID_PARAMETER, "Test error");
    try std.testing.expect(context.error_count == 1);
} 