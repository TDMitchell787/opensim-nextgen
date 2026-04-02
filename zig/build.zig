const std = @import("std");

fn addOdePaths(mod: *std.Build.Module, b: *std.Build) void {
    // Check ODE_PREFIX env var first, then common locations
    const ode_prefix: ?[]const u8 = std.process.getEnvVarOwned(b.allocator, "ODE_PREFIX") catch null;

    if (ode_prefix) |prefix| {
        const include = std.fmt.allocPrint(b.allocator, "{s}/include", .{prefix}) catch @panic("OOM");
        const lib = std.fmt.allocPrint(b.allocator, "{s}/lib", .{prefix}) catch @panic("OOM");
        mod.addIncludePath(.{ .cwd_relative = include });
        mod.addLibraryPath(.{ .cwd_relative = lib });
    } else {
        // Try common install locations
        const prefixes = [_][]const u8{
            "/usr/local",
            "/opt/homebrew",
            "/usr",
            "/usr/local/Cellar/ode/0.16.6",
        };
        for (prefixes) |prefix| {
            const include = std.fmt.allocPrint(b.allocator, "{s}/include/ode", .{prefix}) catch @panic("OOM");
            if (std.fs.cwd().access(include, .{})) |_| {
                const inc = std.fmt.allocPrint(b.allocator, "{s}/include", .{prefix}) catch @panic("OOM");
                const lib = std.fmt.allocPrint(b.allocator, "{s}/lib", .{prefix}) catch @panic("OOM");
                mod.addIncludePath(.{ .cwd_relative = inc });
                mod.addLibraryPath(.{ .cwd_relative = lib });
                break;
            } else |_| {}
        }
    }
}

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const physics_mod = b.createModule(.{
        .root_source_file = b.path("src/physics/root.zig"),
        .target = target,
        .optimize = optimize,
        .link_libc = true,
    });
    physics_mod.addIncludePath(b.path("../shared/ffi"));
    addOdePaths(physics_mod, b);
    physics_mod.addLibraryPath(b.path("../bin/lib64"));
    physics_mod.linkSystemLibrary("ode", .{});
    physics_mod.linkSystemLibrary("BulletSim", .{});
    physics_mod.addCSourceFile(.{ .file = b.path("src/physics/raycast_wrapper.c") });

    const physics_lib = b.addLibrary(.{
        .name = "opensim_physics",
        .linkage = .dynamic,
        .root_module = physics_mod,
    });

    const network_lib = b.addLibrary(.{
        .name = "opensim_network",
        .linkage = .dynamic,
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/network/mod.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    const memory_lib = b.addLibrary(.{
        .name = "opensim_memory",
        .linkage = .dynamic,
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/memory/mod.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    const ffi_lib = b.addLibrary(.{
        .name = "opensim_ffi",
        .linkage = .dynamic,
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/ffi/mod.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    b.installArtifact(physics_lib);
    b.installArtifact(network_lib);
    b.installArtifact(memory_lib);
    b.installArtifact(ffi_lib);

    const physics_test_mod = b.createModule(.{
        .root_source_file = b.path("src/physics/root.zig"),
        .target = target,
        .optimize = optimize,
        .link_libc = true,
    });
    addOdePaths(physics_test_mod, b);
    physics_test_mod.addLibraryPath(b.path("../bin/lib64"));
    physics_test_mod.linkSystemLibrary("ode", .{});
    physics_test_mod.linkSystemLibrary("BulletSim", .{});
    physics_test_mod.addCSourceFile(.{ .file = b.path("src/physics/raycast_wrapper.c") });

    const physics_tests = b.addTest(.{ .root_module = physics_test_mod });

    const network_tests = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/network/mod.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    const memory_tests = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/memory/mod.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    const ffi_tests = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/ffi/mod.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    const run_physics_tests = b.addRunArtifact(physics_tests);
    const run_network_tests = b.addRunArtifact(network_tests);
    const run_memory_tests = b.addRunArtifact(memory_tests);
    const run_ffi_tests = b.addRunArtifact(ffi_tests);

    const test_step = b.step("test", "Run all library tests");
    test_step.dependOn(&run_physics_tests.step);
    test_step.dependOn(&run_network_tests.step);
    test_step.dependOn(&run_memory_tests.step);
    test_step.dependOn(&run_ffi_tests.step);

    const physics_test_step = b.step("test:physics", "Run physics tests");
    physics_test_step.dependOn(&run_physics_tests.step);

    const network_test_step = b.step("test:network", "Run network tests");
    network_test_step.dependOn(&run_network_tests.step);

    const memory_test_step = b.step("test:memory", "Run memory tests");
    memory_test_step.dependOn(&run_memory_tests.step);

    const ffi_test_step = b.step("test:ffi", "Run FFI tests");
    ffi_test_step.dependOn(&run_ffi_tests.step);
}
