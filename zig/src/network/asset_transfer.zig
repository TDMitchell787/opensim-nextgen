const std = @import("std");
const NetworkConfig = @import("mod.zig").NetworkConfig;

pub const AssetTransferHandler = struct {
    allocator: std.mem.Allocator,
    config: NetworkConfig,
    active_transfers: std.AutoHashMap(u64, *AssetTransfer),
    transfer_counter: u64,

    pub fn init(allocator: std.mem.Allocator, config: NetworkConfig) !AssetTransferHandler {
        return AssetTransferHandler{
            .allocator = allocator,
            .config = config,
            .active_transfers = std.AutoHashMap(u64, *AssetTransfer).init(allocator),
            .transfer_counter = 0,
        };
    }

    pub fn deinit(self: *AssetTransferHandler) void {
        var it = self.active_transfers.iterator();
        while (it.next()) |entry| {
            entry.value_ptr.*.deinit();
            self.allocator.destroy(entry.value_ptr.*);
        }
        self.active_transfers.deinit();
    }

    pub fn canHandle(_: *AssetTransferHandler, data: []const u8) bool {
        if (data.len < @sizeOf(TransferHeader)) return false;
        const header = std.mem.bytesAsValue(TransferHeader, data[0..@sizeOf(TransferHeader)]).*;
        return header.magic == TRANSFER_MAGIC;
    }

    pub fn processChunk(self: *AssetTransferHandler, data: []const u8) !ChunkResult {
        if (!self.canHandle(data)) {
            return AssetTransferError.InvalidProtocol;
        }
        const header = std.mem.bytesAsValue(TransferHeader, data[0..@sizeOf(TransferHeader)]).*;
        const chunk_data = data[@sizeOf(TransferHeader)..];
        var transfer = self.active_transfers.get(header.transfer_id) orelse blk: {
            const new_transfer = try self.allocator.create(AssetTransfer);
            new_transfer.* = try AssetTransfer.init(self.allocator, header);
            try self.active_transfers.put(header.transfer_id, new_transfer);
            break :blk new_transfer;
        };
        return try transfer.processChunk(chunk_data);
    }

    pub fn createTransfer(self: *AssetTransferHandler, asset_data: []const u8, asset_type: u8) !TransferInfo {
        self.transfer_counter += 1;
        const transfer_id = self.transfer_counter;
        const transfer = try self.allocator.create(AssetTransfer);
        transfer.* = try AssetTransfer.initWithData(self.allocator, transfer_id, asset_data, asset_type);
        try self.active_transfers.put(transfer_id, transfer);
        return TransferInfo{
            .transfer_id = transfer_id,
            .total_size = asset_data.len,
            .chunk_size = self.config.buffer_size,
            .asset_type = asset_type,
        };
    }

    pub fn createResponse(self: *AssetTransferHandler) ![]u8 {
        // Create a simple success response
        var response = TransferResponse{
            .status = .Success,
            .message = "Transfer completed",
        };
        const response_size = @sizeOf(TransferResponse);
        const buffer = try self.allocator.alloc(u8, response_size);
        std.mem.copy(u8, buffer, std.mem.asBytes(&response));
        return buffer;
    }

    pub fn cleanupCompletedTransfers(self: *AssetTransferHandler) void {
        var it = self.active_transfers.iterator();
        while (it.next()) |entry| {
            if (entry.value_ptr.*.isComplete()) {
                entry.value_ptr.*.deinit();
                self.allocator.destroy(entry.value_ptr.*);
                _ = self.active_transfers.remove(entry.key);
            }
        }
    }
};

pub const AssetTransfer = struct {
    allocator: std.mem.Allocator,
    transfer_id: u64,
    asset_type: u8,
    total_size: usize,
    received_size: usize,
    chunks: std.ArrayList(Chunk),
    is_complete: bool,
    crc: u32,

    pub fn init(allocator: std.mem.Allocator, header: TransferHeader) !AssetTransfer {
        return AssetTransfer{
            .allocator = allocator,
            .transfer_id = header.transfer_id,
            .asset_type = header.asset_type,
            .total_size = header.size,
            .received_size = 0,
            .chunks = std.ArrayList(Chunk).init(allocator),
            .is_complete = false,
            .crc = header.crc,
        };
    }

    pub fn initWithData(allocator: std.mem.Allocator, transfer_id: u64, asset_data: []const u8, asset_type: u8) !AssetTransfer {
        const crc = calculateCRC32(asset_data);
        return AssetTransfer{
            .allocator = allocator,
            .transfer_id = transfer_id,
            .asset_type = asset_type,
            .total_size = asset_data.len,
            .received_size = 0,
            .chunks = std.ArrayList(Chunk).init(allocator),
            .is_complete = false,
            .crc = crc,
        };
    }

    pub fn deinit(self: *AssetTransfer) void {
        for (self.chunks.items) |*chunk| {
            self.allocator.free(chunk.data);
        }
        self.chunks.deinit();
    }

    pub fn processChunk(self: *AssetTransfer, chunk_data: []const u8) !ChunkResult {
        if (self.is_complete) {
            return ChunkResult{ .Complete = {} };
        }
        const chunk = Chunk{
            .data = try self.allocator.dupe(u8, chunk_data),
            .size = chunk_data.len,
        };
        try self.chunks.append(chunk);
        self.received_size += chunk_data.len;
        if (self.received_size >= self.total_size) {
            self.is_complete = true;
            const assembled_data = try self.assembleData();
            defer self.allocator.free(assembled_data);
            const calculated_crc = calculateCRC32(assembled_data);
            if (calculated_crc != self.crc) {
                return AssetTransferError.CRCMismatch;
            }
            return ChunkResult{ .Complete = {} };
        }
        return ChunkResult{ .Partial = self.received_size };
    }

    pub fn isComplete(self: *AssetTransfer) bool {
        return self.is_complete;
    }

    pub fn getProgress(self: *AssetTransfer) f32 {
        if (self.total_size == 0) return 0.0;
        return @as(f32, self.received_size) / @as(f32, self.total_size);
    }

    fn assembleData(self: *AssetTransfer) ![]u8 {
        const total_size = self.received_size;
        const assembled = try self.allocator.alloc(u8, total_size);
        var offset: usize = 0;
        for (self.chunks.items) |chunk| {
            std.mem.copy(u8, assembled[offset..][0..chunk.size], chunk.data);
            offset += chunk.size;
        }
        return assembled;
    }
};

pub const Chunk = struct {
    data: []u8,
    size: usize,
};

pub const ChunkResult = union(enum) {
    Complete: void,
    Partial: usize,
    Error: AssetTransferError,
};

pub const TransferInfo = struct {
    transfer_id: u64,
    total_size: usize,
    chunk_size: usize,
    asset_type: u8,
};

pub const TransferResponse = struct {
    status: TransferStatus,
    message: [64]u8,
};

pub const TransferStatus = enum(u8) {
    Success = 0,
    Error = 1,
    Partial = 2,
    Timeout = 3,
};

pub const AssetTransferError = error{
    InvalidProtocol,
    CRCMismatch,
    BufferOverflow,
    TransferTimeout,
    InvalidChunk,
    MemoryAllocationFailed,
};

pub const TransferHeader = struct {
    magic: u32,
    transfer_id: u64,
    asset_type: u8,
    size: u32,
    crc: u32,
    reserved0: u8,
    reserved1: u8,
    reserved2: u8,
};

const TRANSFER_MAGIC: u32 = 0x41535345; // "ASSE"

fn calculateCRC32(data: []const u8) u32 {
    var crc: u32 = 0xFFFFFFFF;
    for (data) |byte| {
        crc ^= @as(u32, byte);
        var i: u32 = 0;
        while (i < 8) : (i += 1) {
            if (crc & 1 != 0) {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    return crc ^ 0xFFFFFFFF;
}

pub const AssetTransferMetrics = struct {
    total_transfers: u64,
    completed_transfers: u64,
    failed_transfers: u64,
    total_bytes_transferred: u64,
    average_transfer_time_ms: u64,
    pub fn recordTransfer(self: *AssetTransferMetrics, success: bool, bytes: u64, time_ms: u64) void {
        self.total_transfers += 1;
        if (success) {
            self.completed_transfers += 1;
        } else {
            self.failed_transfers += 1;
        }
        self.total_bytes_transferred += bytes;
        const total_time = self.average_transfer_time_ms * (self.total_transfers - 1) + time_ms;
        self.average_transfer_time_ms = total_time / self.total_transfers;
    }
};

test "asset transfer handler initialization" {
    const allocator = std.testing.allocator;
    const config = NetworkConfig{};
    var handler = try AssetTransferHandler.init(allocator, config);
    defer handler.deinit();
    try std.testing.expect(handler.transfer_counter == 0);
    try std.testing.expect(handler.active_transfers.count() == 0);
}

test "asset transfer creation and processing" {
    const allocator = std.testing.allocator;
    const config = NetworkConfig{};
    var handler = try AssetTransferHandler.init(allocator, config);
    defer handler.deinit();
    const test_data = "Hello, OpenSim Asset Transfer!";
    const transfer_info = try handler.createTransfer(test_data, 1);
    try std.testing.expect(transfer_info.transfer_id > 0);
    try std.testing.expect(transfer_info.total_size == test_data.len);
    try std.testing.expect(transfer_info.asset_type == 1);
}

test "CRC32 calculation" {
    const test_data = "test";
    const crc = calculateCRC32(test_data);
    try std.testing.expect(crc == 0xD87F7E0C); // Known CRC32 value for "test"
}

test "transfer header packing" {
    const header = TransferHeader{
        .magic = TRANSFER_MAGIC,
        .transfer_id = 12345,
        .asset_type = 1,
        .size = 1024,
        .crc = 0x12345678,
        .reserved0 = 0,
        .reserved1 = 0,
        .reserved2 = 0,
    };
    try std.testing.expect(@sizeOf(TransferHeader) == 28);
    try std.testing.expect(header.magic == TRANSFER_MAGIC);
    try std.testing.expect(header.transfer_id == 12345);
} 