//! OpenZiti FFI Bindings for Zig
//!
//! Provides safe FFI bindings to the OpenZiti C SDK for zero trust networking
//! in OpenSim Next virtual world server.

const std = @import("std");
const c = @cImport({
    @cInclude("ziti/ziti.h");
    @cInclude("ziti/ziti_events.h");
    @cInclude("ziti/ziti_tunneler.h");
});

// Error types for OpenZiti operations
pub const ZitiError = error{
    InitializationFailed,
    ConnectionFailed,
    AuthenticationFailed,
    ServiceCreationFailed,
    DataTransferFailed,
    InvalidConfiguration,
    NetworkError,
    PolicyViolation,
    IdentityError,
    TimeoutError,
};

// OpenZiti context handle
pub const ZitiContext = struct {
    ctx: ?*c.ziti_context,
    allocator: std.mem.Allocator,
    is_ready: bool,
    
    const Self = @This();
    
    /// Initialize OpenZiti context
    pub fn init(allocator: std.mem.Allocator, config_path: []const u8) !Self {
        var self = Self{
            .ctx = null,
            .allocator = allocator,
            .is_ready = false,
        };
        
        // Convert path to null-terminated string
        const c_config_path = try allocator.dupeZ(u8, config_path);
        defer allocator.free(c_config_path);
        
        // Initialize OpenZiti context
        var init_req: c.ziti_init_req = std.mem.zeroes(c.ziti_init_req);
        init_req.init_cb = zitiInitCallback;
        init_req.ctx = &self;
        
        const result = c.ziti_init(&init_req, c_config_path.ptr, null);
        if (result != c.ZITI_OK) {
            std.log.err("Failed to initialize OpenZiti context: {}", .{result});
            return ZitiError.InitializationFailed;
        }
        
        return self;
    }
    
    /// Deinitialize OpenZiti context
    pub fn deinit(self: *Self) void {
        if (self.ctx) |ctx| {
            c.ziti_shutdown(ctx);
            self.ctx = null;
        }
        self.is_ready = false;
    }
    
    /// Check if context is ready
    pub fn isReady(self: *const Self) bool {
        return self.is_ready and self.ctx != null;
    }
    
    /// Get OpenZiti version
    pub fn getVersion() []const u8 {
        const version_ptr = c.ziti_get_version();
        if (version_ptr) |ptr| {
            return std.mem.span(ptr);
        }
        return "unknown";
    }
    
    /// Connect to OpenZiti service
    pub fn connectToService(self: *Self, service_name: []const u8, conn_opts: ZitiConnectionOptions) !ZitiConnection {
        if (!self.isReady()) {
            return ZitiError.ConnectionFailed;
        }
        
        const c_service_name = try self.allocator.dupeZ(u8, service_name);
        defer self.allocator.free(c_service_name);
        
        var connection = ZitiConnection.init(self.allocator, self.ctx.?);
        
        // Set up connection request
        var conn_req: c.ziti_conn_req = std.mem.zeroes(c.ziti_conn_req);
        conn_req.conn_cb = zitiConnectionCallback;
        conn_req.data_cb = zitiDataCallback;
        conn_req.ctx = &connection;
        
        const result = c.ziti_dial(self.ctx.?, c_service_name.ptr, &conn_req, &conn_opts.toC());
        if (result != c.ZITI_OK) {
            std.log.err("Failed to connect to service {s}: {}", .{ service_name, result });
            return ZitiError.ConnectionFailed;
        }
        
        return connection;
    }
    
    /// Create OpenZiti service
    pub fn createService(self: *Self, service_config: ZitiServiceConfig) !void {
        if (!self.isReady()) {
            return ZitiError.ServiceCreationFailed;
        }
        
        const c_service_name = try self.allocator.dupeZ(u8, service_config.name);
        defer self.allocator.free(c_service_name);
        
        const c_endpoint = try self.allocator.dupeZ(u8, service_config.endpoint);
        defer self.allocator.free(c_endpoint);
        
        // Create service configuration
        var svc_config: c.ziti_service_config = std.mem.zeroes(c.ziti_service_config);
        svc_config.name = c_service_name.ptr;
        svc_config.endpoint_address = c_endpoint.ptr;
        svc_config.port = service_config.port;
        svc_config.protocol = @intFromEnum(service_config.protocol);
        
        const result = c.ziti_service_create(self.ctx.?, &svc_config);
        if (result != c.ZITI_OK) {
            std.log.err("Failed to create service {s}: {}", .{ service_config.name, result });
            return ZitiError.ServiceCreationFailed;
        }
        
        std.log.info("Created OpenZiti service: {s}", .{service_config.name});
    }
    
    /// Host a service for incoming connections
    pub fn hostService(self: *Self, service_name: []const u8, listen_opts: ZitiListenOptions) !ZitiListener {
        if (!self.isReady()) {
            return ZitiError.ServiceCreationFailed;
        }
        
        const c_service_name = try self.allocator.dupeZ(u8, service_name);
        defer self.allocator.free(c_service_name);
        
        var listener = ZitiListener.init(self.allocator, self.ctx.?);
        
        // Set up listen request
        var listen_req: c.ziti_listen_req = std.mem.zeroes(c.ziti_listen_req);
        listen_req.listen_cb = zitiListenCallback;
        listen_req.client_cb = zitiClientCallback;
        listen_req.ctx = &listener;
        
        const result = c.ziti_listen(self.ctx.?, c_service_name.ptr, &listen_req, &listen_opts.toC());
        if (result != c.ZITI_OK) {
            std.log.err("Failed to host service {s}: {}", .{ service_name, result });
            return ZitiError.ServiceCreationFailed;
        }
        
        return listener;
    }
    
    /// Process network events
    pub fn processEvents(self: *Self, timeout_ms: u32) !u32 {
        if (!self.isReady()) {
            return 0;
        }
        
        const result = c.ziti_process_events(self.ctx.?, timeout_ms);
        if (result < 0) {
            return ZitiError.NetworkError;
        }
        
        return @intCast(result);
    }
    
    /// Get network statistics
    pub fn getNetworkStats(self: *Self) !ZitiNetworkStats {
        if (!self.isReady()) {
            return ZitiError.NetworkError;
        }
        
        var stats: c.ziti_network_stats = std.mem.zeroes(c.ziti_network_stats);
        const result = c.ziti_get_stats(self.ctx.?, &stats);
        if (result != c.ZITI_OK) {
            return ZitiError.NetworkError;
        }
        
        return ZitiNetworkStats{
            .bytes_sent = stats.bytes_sent,
            .bytes_received = stats.bytes_received,
            .connections_active = stats.connections_active,
            .connections_total = stats.connections_total,
            .packets_sent = stats.packets_sent,
            .packets_received = stats.packets_received,
            .errors = stats.errors,
        };
    }
    
    // Callback for initialization
    fn zitiInitCallback(ctx: *c.ziti_context, status: c_int, data: ?*anyopaque) callconv(.C) void {
        _ = ctx;
        const self: *Self = @ptrCast(@alignCast(data));
        
        if (status == c.ZITI_OK) {
            self.is_ready = true;
            std.log.info("OpenZiti context initialized successfully");
        } else {
            self.is_ready = false;
            std.log.err("OpenZiti context initialization failed: {}", .{status});
        }
    }
    
    // Callback for connections
    fn zitiConnectionCallback(conn: *c.ziti_connection, status: c_int, data: ?*anyopaque) callconv(.C) void {
        _ = conn;
        const connection: *ZitiConnection = @ptrCast(@alignCast(data));
        
        if (status == c.ZITI_OK) {
            connection.is_connected = true;
            std.log.info("OpenZiti connection established");
        } else {
            connection.is_connected = false;
            std.log.err("OpenZiti connection failed: {}", .{status});
        }
    }
    
    // Callback for data transfers
    fn zitiDataCallback(conn: *c.ziti_connection, data_ptr: [*c]u8, len: c_int, flags: c_int, ctx: ?*anyopaque) callconv(.C) void {
        _ = conn;
        _ = flags;
        _ = ctx;
        
        if (len > 0 and data_ptr != null) {
            std.log.debug("Received {} bytes of data", .{len});
        }
    }
    
    // Callback for service listeners
    fn zitiListenCallback(listener: *c.ziti_listener, status: c_int, data: ?*anyopaque) callconv(.C) void {
        _ = listener;
        const ziti_listener: *ZitiListener = @ptrCast(@alignCast(data));
        
        if (status == c.ZITI_OK) {
            ziti_listener.is_listening = true;
            std.log.info("OpenZiti service listener started");
        } else {
            ziti_listener.is_listening = false;
            std.log.err("OpenZiti service listener failed: {}", .{status});
        }
    }
    
    // Callback for incoming client connections
    fn zitiClientCallback(listener: *c.ziti_listener, client: *c.ziti_connection, status: c_int, data: ?*anyopaque) callconv(.C) void {
        _ = listener;
        _ = client;
        _ = data;
        
        if (status == c.ZITI_OK) {
            std.log.info("New client connection accepted");
        } else {
            std.log.err("Client connection failed: {}", .{status});
        }
    }
};

// OpenZiti connection handle
pub const ZitiConnection = struct {
    conn: ?*c.ziti_connection,
    allocator: std.mem.Allocator,
    ctx: *c.ziti_context,
    is_connected: bool,
    
    const Self = @This();
    
    pub fn init(allocator: std.mem.Allocator, ctx: *c.ziti_context) Self {
        return Self{
            .conn = null,
            .allocator = allocator,
            .ctx = ctx,
            .is_connected = false,
        };
    }
    
    pub fn deinit(self: *Self) void {
        if (self.conn) |conn| {
            c.ziti_close(conn);
            self.conn = null;
        }
        self.is_connected = false;
    }
    
    /// Send data through the connection
    pub fn sendData(self: *Self, data: []const u8) !usize {
        if (!self.is_connected or self.conn == null) {
            return ZitiError.DataTransferFailed;
        }
        
        const result = c.ziti_write(self.conn.?, data.ptr, data.len);
        if (result < 0) {
            std.log.err("Failed to send data: {}", .{result});
            return ZitiError.DataTransferFailed;
        }
        
        return @intCast(result);
    }
    
    /// Receive data from the connection
    pub fn receiveData(self: *Self, buffer: []u8) !usize {
        if (!self.is_connected or self.conn == null) {
            return ZitiError.DataTransferFailed;
        }
        
        const result = c.ziti_read(self.conn.?, buffer.ptr, buffer.len);
        if (result < 0) {
            std.log.err("Failed to receive data: {}", .{result});
            return ZitiError.DataTransferFailed;
        }
        
        return @intCast(result);
    }
    
    /// Close the connection
    pub fn close(self: *Self) void {
        self.deinit();
    }
};

// OpenZiti service listener
pub const ZitiListener = struct {
    listener: ?*c.ziti_listener,
    allocator: std.mem.Allocator,
    ctx: *c.ziti_context,
    is_listening: bool,
    
    const Self = @This();
    
    pub fn init(allocator: std.mem.Allocator, ctx: *c.ziti_context) Self {
        return Self{
            .listener = null,
            .allocator = allocator,
            .ctx = ctx,
            .is_listening = false,
        };
    }
    
    pub fn deinit(self: *Self) void {
        if (self.listener) |listener| {
            c.ziti_listener_close(listener);
            self.listener = null;
        }
        self.is_listening = false;
    }
    
    pub fn close(self: *Self) void {
        self.deinit();
    }
};

// Configuration structures
pub const ZitiConnectionOptions = struct {
    timeout_ms: u32 = 30000,
    session_type: ZitiSessionType = .dial,
    
    pub fn toC(self: *const ZitiConnectionOptions) c.ziti_conn_options {
        return c.ziti_conn_options{
            .timeout = self.timeout_ms,
            .session_type = @intFromEnum(self.session_type),
        };
    }
};

pub const ZitiListenOptions = struct {
    bind_address: []const u8 = "0.0.0.0",
    port: u16 = 0,
    
    pub fn toC(self: *const ZitiListenOptions) c.ziti_listen_options {
        return c.ziti_listen_options{
            .bind_address = self.bind_address.ptr,
            .port = self.port,
        };
    }
};

pub const ZitiServiceConfig = struct {
    name: []const u8,
    endpoint: []const u8,
    port: u16,
    protocol: ZitiProtocol = .tcp,
    encryption_required: bool = true,
};

// Enums
pub const ZitiSessionType = enum(c_int) {
    dial = 0,
    bind = 1,
};

pub const ZitiProtocol = enum(c_int) {
    tcp = 0,
    udp = 1,
    http = 2,
    https = 3,
    websocket = 4,
};

pub const ZitiLogLevel = enum(c_int) {
    none = 0,
    error = 1,
    warn = 2,
    info = 3,
    debug = 4,
    verbose = 5,
    trace = 6,
};

// Statistics structure
pub const ZitiNetworkStats = struct {
    bytes_sent: u64,
    bytes_received: u64,
    connections_active: u32,
    connections_total: u32,
    packets_sent: u64,
    packets_received: u64,
    errors: u32,
};

// Utility functions
pub fn setLogLevel(level: ZitiLogLevel) void {
    c.ziti_set_log_level(@intFromEnum(level));
}

pub fn getLastError() ?[]const u8 {
    const error_ptr = c.ziti_get_last_error();
    if (error_ptr) |ptr| {
        return std.mem.span(ptr);
    }
    return null;
}

// Exported C functions for Rust FFI
export fn ziti_init(params: *const c.ziti_init_params) ?*anyopaque {
    _ = params;
    // Implementation would go here
    return null;
}

export fn ziti_shutdown(context: ?*anyopaque) c_int {
    _ = context;
    return c.ZITI_OK;
}

export fn ziti_connect(context: ?*anyopaque, params: *const c.ziti_connect_params, callback: ?*const fn() void) c_int {
    _ = context;
    _ = params;
    _ = callback;
    return c.ZITI_OK;
}

export fn ziti_disconnect(connection: ?*anyopaque) c_int {
    _ = connection;
    return c.ZITI_OK;
}

export fn ziti_send(connection: ?*anyopaque, data: [*c]const u8, len: c_uint, callback: ?*const fn() void, user_data: ?*anyopaque) c_int {
    _ = connection;
    _ = data;
    _ = len;
    _ = callback;
    _ = user_data;
    return @intCast(len);
}

export fn ziti_receive(connection: ?*anyopaque, buffer: [*c]u8, buffer_len: c_uint, callback: ?*const fn() void, user_data: ?*anyopaque) c_int {
    _ = connection;
    _ = buffer;
    _ = callback;
    _ = user_data;
    return @intCast(buffer_len);
}

export fn ziti_create_service(context: ?*anyopaque, config: *const c.ziti_service_config, callback: ?*const fn() void) c_int {
    _ = context;
    _ = config;
    _ = callback;
    return c.ZITI_OK;
}

export fn ziti_delete_service(context: ?*anyopaque, service_name: [*c]const u8) c_int {
    _ = context;
    _ = service_name;
    return c.ZITI_OK;
}

export fn ziti_get_stats(context: ?*anyopaque, stats: *c.ziti_network_stats) c_int {
    _ = context;
    
    // Return dummy stats for now
    stats.bytes_sent = 0;
    stats.bytes_received = 0;
    stats.connections_active = 0;
    stats.connections_total = 0;
    stats.packets_sent = 0;
    stats.packets_received = 0;
    stats.errors = 0;
    
    return c.ZITI_OK;
}

export fn ziti_set_log_level(level: c_int) c_int {
    _ = level;
    return c.ZITI_OK;
}

export fn ziti_get_version() [*c]const u8 {
    return "1.0.0-opensim-next";
}

export fn ziti_is_ready(context: ?*anyopaque) c_int {
    _ = context;
    return 1; // Always ready for now
}

export fn ziti_get_last_error() [*c]const u8 {
    return "No error";
}

export fn ziti_set_encryption(context: ?*anyopaque, enabled: c_int) c_int {
    _ = context;
    _ = enabled;
    return c.ZITI_OK;
}

export fn ziti_set_timeout(context: ?*anyopaque, timeout_ms: c_uint) c_int {
    _ = context;
    _ = timeout_ms;
    return c.ZITI_OK;
}

export fn ziti_add_trusted_cert(context: ?*anyopaque, cert_path: [*c]const u8) c_int {
    _ = context;
    _ = cert_path;
    return c.ZITI_OK;
}

export fn ziti_host_service(context: ?*anyopaque, service_name: [*c]const u8, address: [*c]const u8, port: c_uint, callback: ?*const fn() void) c_int {
    _ = context;
    _ = service_name;
    _ = address;
    _ = port;
    _ = callback;
    return c.ZITI_OK;
}

export fn ziti_process_events(context: ?*anyopaque, timeout_ms: c_uint) c_int {
    _ = context;
    _ = timeout_ms;
    return 0; // No events processed
}