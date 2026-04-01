# Zig Physics Compilation Reference

## Directory Structure

```
zig/
├── build.zig                          # Build configuration
├── src/physics/
│   ├── root.zig                       # Module root (shared library entry point)
│   ├── interface.zig                  # Abstract PhysicsEngine vtable, Vec3, Quat, PhysicsBody
│   ├── registry.zig                   # Engine factory: createEngine(type, alloc)
│   ├── engine/
│   │   ├── main.zig                   # FFI export shell (dispatches through global_engine)
│   │   └── simd.zig                   # SIMD utilities
│   └── engines/
│       ├── basic.zig                  # BasicPhysics (pure Zig, no deps)
│       ├── pos.zig                    # POS (pure Zig, terrain heightmap)
│       ├── ode.zig                    # ODE (links libode)
│       ├── ubode.zig                  # ubODE (links libode, enhanced config)
│       └── bullet.zig                 # BulletSim (links libBulletSim.dylib)
```

## Build Commands

### Build all libraries
```bash
cd opensim-next/zig
zig build
```

Output: `zig-out/lib/libopensim_physics.dylib` (+ network, memory, ffi)

### Build physics only (faster)
```bash
cd opensim-next/zig
zig build -Dtarget=native
```

### Run physics tests
```bash
cd opensim-next/zig
zig build test:physics
```

### Run FFI tests
```bash
cd opensim-next/zig
zig build test:ffi
```

### Run all tests
```bash
cd opensim-next/zig
zig build test
```

## External Dependencies

### ODE (Open Dynamics Engine)
- **Required by**: ode.zig, ubode.zig
- **Source**: Homebrew
- **Include**: `/usr/local/Cellar/ode/0.16.6/include`
- **Library**: `/usr/local/Cellar/ode/0.16.6/lib`
- **Install**: `brew install ode`

### BulletSim
- **Required by**: bullet.zig
- **Source**: Bundled in `bin/lib64/libBulletSim.dylib` (v3.26, universal binary)
- **Library path**: `../bin/lib64/` (relative to zig/)
- **No headers needed** — extern declarations are manual in bullet.zig
- **Optional**: If missing, BulletSim engine creation returns null, fallback to ODE

## Module System Rules (Zig 0.14)

1. **root.zig is the shared library entry point** — set in build.zig as `root_source_file`
2. **No `../` imports from subdirectories** — Zig 0.14 prohibits imports outside module root
3. **engine/main.zig imports `../root.zig`** which is allowed (going up to module root)
4. **Exported FFI symbols must be re-exported in root.zig** — Zig lazy compilation skips unreferenced exports
5. **Use `comptime { _ = module; }` in root.zig** to force compilation of all engine modules

### How FFI exports work
```
root.zig
  → imports engine/main.zig
  → re-exports: pub const physics_create_engine = main.physics_create_engine;
  → comptime { _ = main; }  // forces linker to include all exports
```

### Adding a new engine
1. Create `zig/src/physics/engines/myengine.zig`
2. Implement the vtable (see basic.zig for simplest example)
3. Add to `EngineType` enum in `interface.zig`
4. Add case to `registry.zig` switch
5. Add import to `root.zig` engines struct
6. Add `comptime { _ = engines.myengine; }` to root.zig
7. If external lib needed, add `physics_lib.linkSystemLibrary("mylib")` to build.zig

## Rust Integration

### Build sequence
```bash
# 1. Build Zig first
cd opensim-next/zig && zig build

# 2. Build Rust (build.rs copies dylibs automatically)
cd opensim-next && cargo build --release --bin opensim-next

# 3. Run with DYLD_LIBRARY_PATH
RUST_LOG=info DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib ./target/release/opensim-next
```

### Rust build.rs behavior
- Copies `libopensim_physics.dylib` and `libopensim_ffi.dylib` from `zig/zig-out/lib/` to `OUT_DIR`
- Tells Cargo to link them as dylibs
- Triggers rebuild when any `.zig` source file changes

### Runtime library loading
- `DYLD_LIBRARY_PATH` must include the directory containing the .dylib files
- The binary loads `libopensim_physics.dylib` which transitively loads `libode` and `libBulletSim`
- If `libBulletSim.dylib` is missing at runtime, BulletSim engine creation fails gracefully (returns null)

## Verify Build

```bash
# Check symbols are exported
nm -gU zig/zig-out/lib/libopensim_physics.dylib | grep physics

# Expected: 20 symbols including physics_create_engine, physics_body_create, etc.
```

## Engine Type IDs

| Engine | ID | Dependencies | Use Case |
|--------|-----|-------------|----------|
| BasicPhysics | 0 | None | Testing, minimal physics |
| POS | 1 | None | Terrain-aware, lightweight |
| ODE | 2 | libode | Default, full collision |
| ubODE | 3 | libode | Enhanced ODE, avatar capsules |
| BulletSim | 4 | libBulletSim | High-fidelity physics |
