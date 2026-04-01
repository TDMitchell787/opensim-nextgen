# Building OpenSim NextGen

## System Requirements

### Required
- **Rust** 1.70+ — [rustup.rs](https://rustup.rs)
- **Zig** 0.14+ — [ziglang.org/download](https://ziglang.org/download/)
- **PostgreSQL** 14+ or **SQLite** 3.35+ — Database backend
- **pkg-config** — Build dependency resolution

### Optional
- **Flutter** 3.x — For building the desktop client applications
- **Redis** — For distributed caching in multi-instance deployments
- **Docker** — For containerized deployment

### Platform Support
- **macOS 14+** (Sonoma) — Primary development platform
- **Linux** (Ubuntu 22.04+, Debian 12+) — Production deployment
- **Windows** — Experimental

## Build Steps

### 1. Build the Zig Physics Engine

The Zig physics engine must be built first, as the Rust server links to it via FFI.

```bash
cd zig
zig build
cd ..
```

This produces shared libraries in `zig/zig-out/lib/`:
- `libopensim_physics.dylib` (or `.so` on Linux)
- `libopensim_ffi.dylib`
- `libopensim_memory.dylib`
- `libopensim_network.dylib`

### 2. Build the Rust Server

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, optimized runtime)
cargo build --release
```

The server binary is produced at:
- Debug: `target/debug/opensim-next`
- Release: `target/release/opensim-next`

### 3. Build Flutter Applications (Optional)

#### OpenSim Configurator
```bash
cd flutter-client/opensim_configurator
flutter pub get
flutter build macos
```

#### User Manual Viewer
```bash
cd flutter-client/user_manual_viewer
flutter pub get
flutter build macos

# Package as DMG
codesign --deep --force --sign - build/macos/Build/Products/Release/user_manual_viewer.app
hdiutil create -volname "OpenSim User Manual" \
  -srcfolder build/macos/Build/Products/Release/user_manual_viewer.app \
  -ov -format UDZO OpenSimManual.dmg
```

## Running

### Set Library Path

The Rust server needs to find the Zig shared libraries at runtime:

```bash
# macOS
export DYLD_LIBRARY_PATH=./zig/zig-out/lib:$DYLD_LIBRARY_PATH

# Linux
export LD_LIBRARY_PATH=./zig/zig-out/lib:$LD_LIBRARY_PATH
```

### Start the Server

```bash
# Copy and edit environment config
cp .env.example .env
# Edit .env with your database URL, ports, and API key

# Run
RUST_LOG=info ./target/release/opensim-next
```

### Verify

- Open `http://localhost:9200` for the admin dashboard
- Open `http://localhost:8080` for the web client
- Connect a Second Life viewer to `localhost:9000`

## Running Tests

```bash
# All tests
cargo test

# Specific test module
cargo test --package opensim-next -- test_name

# With output
cargo test -- --nocapture
```

## Troubleshooting

### "dyld: missing symbol" on macOS
Ensure Zig libraries are built and `DYLD_LIBRARY_PATH` is set correctly.

### Database connection errors
Verify `DATABASE_URL` in your `.env` file. For PostgreSQL, ensure the database exists:
```bash
createdb opensim_db
```

### Flutter build fails
Ensure Flutter is on the stable channel:
```bash
flutter channel stable
flutter upgrade
```
