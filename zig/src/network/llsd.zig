const std = @import("std");
const standard_alphabet = @import("std").base64.standard_alphabet;
const base64_alphabet: [64]u8 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".*;

pub const LLSDParser = struct {
    allocator: std.mem.Allocator,
    buffer: []u8,
    buffer_size: usize,

    pub fn init(allocator: std.mem.Allocator) !LLSDParser {
        const buffer_size = 8192;
        const buffer = try allocator.alloc(u8, buffer_size);
        
        return LLSDParser{
            .allocator = allocator,
            .buffer = buffer,
            .buffer_size = buffer_size,
        };
    }

    pub fn deinit(self: *LLSDParser) void {
        self.allocator.free(self.buffer);
    }

    pub fn canParse(_: *LLSDParser, data: []const u8) bool {
        // Check if data starts with LLSD XML tag
        if (data.len < 5) return false;
        return std.mem.startsWith(u8, data, "<llsd");
    }

    pub fn parse(self: *LLSDParser, data: []const u8) !LLSDValue {
        if (!self.canParse(data)) {
            return LLSDParseError.InvalidFormat;
        }

        var parser = XMLParser.init(self.allocator);
        defer parser.deinit();

        return try parser.parse(data);
    }

    pub fn serialize(self: *LLSDParser, value: LLSDValue) ![]u8 {
        var serializer = XMLSerializer.init(self.allocator);
        defer serializer.deinit();
        return try serializer.serialize(value);
    }
};

pub const LLSDParseError = error{
    InvalidFormat,
    MalformedXML,
    UnsupportedType,
    MemoryAllocationFailed,
    UnexpectedEnd,
};

pub const LLSDValue = union(enum) {
    Undefined: void,
    Boolean: bool,
    Integer: i32,
    Real: f64,
    String: []const u8,
    UUID: [16]u8,
    Binary: []const u8,
    Date: i64, // Unix timestamp
    URI: []const u8,
    Array: std.ArrayList(LLSDValue),
    Map: std.StringHashMap(LLSDValue),

    pub fn deinit(self: *LLSDValue) void {
        switch (self.*) {
            .String, .Binary, .URI => {},
            .Array => self.Array.deinit(),
            .Map => self.Map.deinit(),
            else => {},
        }
    }

    pub fn clone(self: LLSDValue, allocator: std.mem.Allocator) !LLSDValue {
        return switch (self) {
            .Undefined => .Undefined,
            .Boolean => |b| .{ .Boolean = b },
            .Integer => |i| .{ .Integer = i },
            .Real => |r| .{ .Real = r },
            .String => |s| .{ .String = try allocator.dupe(u8, s) },
            .UUID => |u| .{ .UUID = u },
            .Binary => |b| .{ .Binary = try allocator.dupe(u8, b) },
            .Date => |d| .{ .Date = d },
            .URI => |u| .{ .URI = try allocator.dupe(u8, u) },
            .Array => |arr| {
                var new_arr = std.ArrayList(LLSDValue).init(allocator);
                for (arr.items) |item| {
                    try new_arr.append(try item.clone(allocator));
                }
                return .{ .Array = new_arr };
            },
            .Map => |map| {
                var new_map = std.StringHashMap(LLSDValue).init(allocator);
                var it = map.iterator();
                while (it.next()) |entry| {
                    try new_map.put(try allocator.dupe(u8, entry.key), try entry.value.clone(allocator));
                }
                return .{ .Map = new_map };
            },
        };
    }
};

const XMLParser = struct {
    allocator: std.mem.Allocator,
    pos: usize,
    data: []const u8,

    pub fn init(allocator: std.mem.Allocator) XMLParser {
        return XMLParser{
            .allocator = allocator,
            .pos = 0,
            .data = undefined,
        };
    }

    pub fn deinit(self: *XMLParser) void {
        _ = self; // Use parameter to avoid warning
        // Cleanup if needed
    }

    pub fn parse(self: *XMLParser, data: []const u8) !LLSDValue {
        self.data = data;
        self.pos = 0;

        // Skip to first tag
        try self.skipWhitespace();
        if (!std.mem.startsWith(u8, self.data[self.pos..], "<llsd")) {
            return LLSDParseError.InvalidFormat;
        }

        // Skip opening llsd tag
        self.pos = std.mem.indexOf(u8, self.data[self.pos..], ">") orelse return LLSDParseError.MalformedXML;
        self.pos += 1;

        const value = try self.parseValue();
        
        // Skip closing llsd tag
        try self.skipWhitespace();
        if (!std.mem.startsWith(u8, self.data[self.pos..], "</llsd>")) {
            return LLSDParseError.MalformedXML;
        }

        return value;
    }

    fn parseValue(self: *XMLParser) !LLSDValue {
        try self.skipWhitespace();
        if (self.pos >= self.data.len) {
            return LLSDParseError.UnexpectedEnd;
        }
        const tag_start = self.pos;
        const tag_end = std.mem.indexOf(u8, self.data[self.pos..], ">") orelse return LLSDParseError.MalformedXML;
        const tag = self.data[tag_start..tag_start + tag_end];
        self.pos = tag_start + tag_end + 1;
        if (std.mem.eql(u8, tag[1..], "undef")) return .Undefined;
        if (std.mem.eql(u8, tag[1..], "boolean")) return try self.parseBoolean();
        if (std.mem.eql(u8, tag[1..], "integer")) return try self.parseInteger();
        if (std.mem.eql(u8, tag[1..], "real")) return try self.parseReal();
        if (std.mem.eql(u8, tag[1..], "string")) return try self.parseString();
        if (std.mem.eql(u8, tag[1..], "uuid")) return try self.parseUUID();
        if (std.mem.eql(u8, tag[1..], "binary")) return try self.parseBinary();
        if (std.mem.eql(u8, tag[1..], "date")) return try self.parseDate();
        if (std.mem.eql(u8, tag[1..], "uri")) return try self.parseURI();
        if (std.mem.eql(u8, tag[1..], "array")) return try self.parseArray();
        if (std.mem.eql(u8, tag[1..], "map")) return try self.parseMap();
        return LLSDParseError.UnsupportedType;
    }

    fn parseBoolean(self: *XMLParser) !LLSDValue {
        const content = try self.parseTagContent("boolean");
        return .{ .Boolean = std.mem.eql(u8, content, "1") or std.mem.eql(u8, content, "true") };
    }

    fn parseInteger(self: *XMLParser) !LLSDValue {
        const content = try self.parseTagContent("integer");
        const value = std.fmt.parseInt(i32, content, 10) catch return LLSDParseError.MalformedXML;
        return .{ .Integer = value };
    }

    fn parseReal(self: *XMLParser) !LLSDValue {
        const content = try self.parseTagContent("real");
        const value = std.fmt.parseFloat(f64, content) catch return LLSDParseError.MalformedXML;
        return .{ .Real = value };
    }

    fn parseString(self: *XMLParser) !LLSDValue {
        const content = try self.parseTagContent("string");
        return .{ .String = try self.allocator.dupe(u8, content) };
    }

    fn parseUUID(self: *XMLParser) !LLSDValue {
        const content = try self.parseTagContent("uuid");
        if (content.len != 36) return LLSDParseError.MalformedXML;
        // Placeholder: just fill with zeros (should parse UUID properly)
        const uuid: [16]u8 = [_]u8{0} ** 16;
        // TODO: Implement real UUID parsing from string
        return .{ .UUID = uuid };
    }

    fn parseBinary(self: *XMLParser) !LLSDValue {
        const content = try self.parseTagContent("binary");
        var decoder = std.base64.Base64Decoder.init(base64_alphabet, '=');
        const decoded = try decoder.decodeAlloc(self.allocator, content);
        return .{ .Binary = decoded };
    }

    fn parseDate(self: *XMLParser) !LLSDValue {
        const content = try self.parseTagContent("date");
        const timestamp = std.fmt.parseInt(i64, content, 10) catch return LLSDParseError.MalformedXML;
        return .{ .Date = timestamp };
    }

    fn parseURI(self: *XMLParser) !LLSDValue {
        const content = try self.parseTagContent("uri");
        return .{ .URI = try self.allocator.dupe(u8, content) };
    }

    fn parseArray(self: *XMLParser) !LLSDValue {
        var array = std.ArrayList(LLSDValue).init(self.allocator);
        
        try self.skipWhitespace();
        while (self.pos < self.data.len and !std.mem.startsWith(u8, self.data[self.pos..], "</array>")) {
            const value = try self.parseValue();
            try array.append(value);
            try self.skipWhitespace();
        }
        
        // Skip closing array tag
        if (!std.mem.startsWith(u8, self.data[self.pos..], "</array>")) {
            return LLSDParseError.MalformedXML;
        }
        self.pos += 8; // "</array>"
        
        return .{ .Array = array };
    }

    fn parseMap(self: *XMLParser) !LLSDValue {
        var map = std.StringHashMap(LLSDValue).init(self.allocator);
        
        try self.skipWhitespace();
        while (self.pos < self.data.len and !std.mem.startsWith(u8, self.data[self.pos..], "</map>")) {
            // Parse key
            if (!std.mem.startsWith(u8, self.data[self.pos..], "<key>")) {
                return LLSDParseError.MalformedXML;
            }
            const key = try self.parseTagContent("key");
            
            // Parse value
            const value = try self.parseValue();
            
            try map.put(try self.allocator.dupe(u8, key), value);
            try self.skipWhitespace();
        }
        
        // Skip closing map tag
        if (!std.mem.startsWith(u8, self.data[self.pos..], "</map>")) {
            return LLSDParseError.MalformedXML;
        }
        self.pos += 6; // "</map>"
        
        return .{ .Map = map };
    }

    fn parseTagContent(self: *XMLParser, tag_name: []const u8) ![]const u8 {
        // Remove unused start_pos and end_tag
        const end_tag = try std.fmt.allocPrint(self.allocator, "</{s}>", .{tag_name});
        defer self.allocator.free(end_tag);
        const end_pos = std.mem.indexOf(u8, self.data[self.pos..], end_tag) orelse return LLSDParseError.MalformedXML;
        const content = self.data[self.pos..self.pos + end_pos];
        self.pos += end_pos + end_tag.len;
        return content;
    }

    fn skipWhitespace(self: *XMLParser) !void {
        while (self.pos < self.data.len and std.ascii.isWhitespace(self.data[self.pos])) {
            self.pos += 1;
        }
    }
};

const XMLSerializer = struct {
    allocator: std.mem.Allocator,
    buffer: std.ArrayList(u8),

    pub fn init(allocator: std.mem.Allocator) XMLSerializer {
        return XMLSerializer{
            .allocator = allocator,
            .buffer = std.ArrayList(u8).init(allocator),
        };
    }

    pub fn deinit(self: *XMLSerializer) void {
        self.buffer.deinit();
    }

    pub fn serialize(self: *XMLSerializer, value: LLSDValue) ![]u8 {
        try self.buffer.appendSlice("<llsd>");
        try self.serializeValue(value);
        try self.buffer.appendSlice("</llsd>");
        
        return self.buffer.toOwnedSlice();
    }

    fn serializeValue(self: *XMLSerializer, value: LLSDValue) !void {
        switch (value) {
            .Undefined => try self.buffer.appendSlice("<undef/>"),
            .Boolean => |b| {
                try self.buffer.appendSlice("<boolean>");
                if (b) try self.buffer.appendSlice("1") else try self.buffer.appendSlice("0");
                try self.buffer.appendSlice("</boolean>");
            },
            .Integer => |i| {
                try self.buffer.appendSlice("<integer>");
                try std.fmt.format(self.buffer.writer(), "{d}", .{i});
                try self.buffer.appendSlice("</integer>");
            },
            .Real => |r| {
                try self.buffer.appendSlice("<real>");
                try std.fmt.format(self.buffer.writer(), "{d}", .{r});
                try self.buffer.appendSlice("</real>");
            },
            .String => |s| {
                try self.buffer.appendSlice("<string>");
                try self.escapeXml(s);
                try self.buffer.appendSlice("</string>");
            },
            .UUID => |_| {
                try self.buffer.appendSlice("<uuid>");
                // Placeholder: just print zeros
                try self.buffer.appendSlice("00000000-0000-0000-0000-000000000000");
                try self.buffer.appendSlice("</uuid>");
            },
            .Binary => |b| {
                try self.buffer.appendSlice("<binary>");
                var encoder = std.base64.Base64Encoder.init(base64_alphabet, '=');
                try encoder.encode(self.buffer.writer(), b);
                try self.buffer.appendSlice("</binary>");
            },
            .Date => |d| {
                try self.buffer.appendSlice("<date>");
                try std.fmt.format(self.buffer.writer(), "{d}", .{d});
                try self.buffer.appendSlice("</date>");
            },
            .URI => |u| {
                try self.buffer.appendSlice("<uri>");
                try self.escapeXml(u);
                try self.buffer.appendSlice("</uri>");
            },
            .Array => |arr| {
                try self.buffer.appendSlice("<array>");
                for (arr.items) |item| {
                    try self.serializeValue(item);
                }
                try self.buffer.appendSlice("</array>");
            },
            .Map => |map| {
                try self.buffer.appendSlice("<map>");
                var it = map.iterator();
                while (it.next()) |entry| {
                    try self.buffer.appendSlice("<key>");
                    try self.escapeXml(entry.key);
                    try self.buffer.appendSlice("</key>");
                    try self.serializeValue(entry.value);
                }
                try self.buffer.appendSlice("</map>");
            },
        }
    }

    fn escapeXml(self: *XMLSerializer, text: []const u8) !void {
        for (text) |c| {
            switch (c) {
                '&' => try self.buffer.appendSlice("&amp;"),
                '<' => try self.buffer.appendSlice("&lt;"),
                '>' => try self.buffer.appendSlice("&gt;"),
                '"' => try self.buffer.appendSlice("&quot;"),
                '\'' => try self.buffer.appendSlice("&apos;"),
                else => try self.buffer.append(c),
            }
        }
    }
};

test "LLSD parser basic types" {
    const allocator = std.testing.allocator;
    var parser = try LLSDParser.init(allocator);
    defer parser.deinit();

    // Test boolean
    const bool_data = "<llsd><boolean>1</boolean></llsd>";
    const bool_result = try parser.parse(bool_data);
    try std.testing.expect(bool_result == .Boolean);
    try std.testing.expect(bool_result.Boolean == true);

    // Test integer
    const int_data = "<llsd><integer>42</integer></llsd>";
    const int_result = try parser.parse(int_data);
    try std.testing.expect(int_result == .Integer);
    try std.testing.expect(int_result.Integer == 42);

    // Test string
    const string_data = "<llsd><string>Hello World</string></llsd>";
    const string_result = try parser.parse(string_data);
    try std.testing.expect(string_result == .String);
    try std.testing.expectEqualStrings("Hello World", string_result.String);
}

test "LLSD serializer" {
    const allocator = std.testing.allocator;
    var parser = try LLSDParser.init(allocator);
    defer parser.deinit();
    const test_value = LLSDValue{ .String = "test" };
    const serialized = try parser.serialize(test_value);
    defer allocator.free(serialized);
    const parsed = try parser.parse(serialized);
    try std.testing.expect(parsed == .String);
    try std.testing.expectEqualStrings("test", parsed.String);
} 