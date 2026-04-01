use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(e) => {
            eprintln!("Warning: OUT_DIR not set ({}), using fallback", e);
            PathBuf::from("target/build")
        }
    };
    
    let lib_dir = PathBuf::from("../zig/zig-out/lib");

    // Copy the physics library to the output directory
    if let Err(e) = fs::copy(
        lib_dir.join("libopensim_physics.dylib"),
        out_dir.join("libopensim_physics.dylib"),
    ) {
        eprintln!("Warning: Failed to copy physics library: {}", e);
        eprintln!("Build will continue, but physics features may not be available");
    }

    // Copy the FFI library to the output directory
    if let Err(e) = fs::copy(
        lib_dir.join("libopensim_ffi.dylib"),
        out_dir.join("libopensim_ffi.dylib"),
    ) {
        eprintln!("Warning: Failed to copy FFI library: {}", e);
        eprintln!("Build will continue, but FFI features may not be available");
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=dylib=opensim_physics");
    println!("cargo:rustc-link-lib=dylib=opensim_ffi");
    println!("cargo:rustc-env=DYLD_LIBRARY_PATH={}", out_dir.display());
    println!("cargo:rerun-if-changed=../zig/src/physics/root.zig");
    println!("cargo:rerun-if-changed=../zig/src/physics/interface.zig");
    println!("cargo:rerun-if-changed=../zig/src/physics/registry.zig");
    println!("cargo:rerun-if-changed=../zig/src/physics/engine/main.zig");
    println!("cargo:rerun-if-changed=../zig/src/physics/engines/basic.zig");
    println!("cargo:rerun-if-changed=../zig/src/physics/engines/pos.zig");
    println!("cargo:rerun-if-changed=../zig/src/physics/engines/ode.zig");
    println!("cargo:rerun-if-changed=../zig/src/physics/engines/ubode.zig");
    println!("cargo:rerun-if-changed=../zig/src/physics/engines/bullet.zig");
    println!("cargo:rerun-if-changed=../shared/ffi/physics.h");
} 