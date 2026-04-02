# OpenSim NextGen — Project Guide

## Overview

OpenSim NextGen is a modern reimplementation of the OpenSimulator virtual world server platform. It uses a hybrid Rust/Zig architecture with a Flutter desktop client. The server is compatible with Second Life viewers (Firestorm, etc.) and web browsers.

## Tech Stack

- **Rust** (edition 2021) — Networking, services, simulation logic, login, asset management, database
- **Zig** (0.14+) — Physics engine, collision detection, raycasting, SIMD math
- **Flutter** (3.x) — Desktop configurator and user manual viewer (macOS primary)
- **SQLx** — Async database layer supporting PostgreSQL, MySQL, MariaDB, SQLite
- **Tokio** — Async runtime
- **Axum** — HTTP/WebSocket server
- **wgpu** — GPU compute (Luxor raytracer)

## Project Structure

```
opensim-nextgen/
  rust/                  # Main Rust crate (binary: opensim-next)
    src/
      main.rs            # Entry point
      lib.rs             # Library root
      ai/                # AI/LLM NPC system
      auth/              # Authentication (JWT)
      database/          # SQLx database layer, migrations
      login_service.rs   # XMLRPC login for SL viewers
      network/           # LLUDP protocol, networking
      protocol/          # SL protocol implementation
      udp/               # UDP packet handling
      region/            # Region/sim management
      instance_manager/  # Multi-instance support
      scripting/         # LSL script engine
      ffi/               # Zig FFI bindings
      asset/             # Asset storage/retrieval
      inventory/         # Inventory system
      avatar/            # Avatar services, baking
      mesh/              # 3D mesh handling
      config.rs          # Configuration (figment/toml)
      services/          # Core grid services
      xmlrpc/            # XMLRPC protocol support
      ...
  zig/                   # Zig physics engine
    src/
      physics/           # Physics simulation
      memory/            # Memory management
      network/           # Network utilities
      ffi/               # C FFI exports for Rust
    build.zig            # Zig build configuration
  flutter-client/
    opensim_configurator/  # Grid configuration GUI
    user_manual_viewer/    # Documentation viewer
  config/                # Grid configuration (OpenSim.ini, Regions/)
  deploy/                # Docker, Kubernetes, Terraform, CI/CD
  tests/                 # Integration tests (rust/, zig/, integration/)
  bin/                   # Instance creation base files, default assets
  Instances/             # Server instance directories
  web/admin/             # Admin dashboard
  Terrain/               # Terrain heightmap files
```

## Build Commands

```bash
# Build Zig physics engine
cd zig && zig build && cd ..

# Build Rust server (debug)
cargo build

# Build Rust server (release)
cargo build --release

# Run server
RUST_LOG=info DYLD_LIBRARY_PATH=./zig/zig-out/lib ./target/release/opensim-next

# Run tests
cargo test

# Build Flutter configurator
cd flutter-client/opensim_configurator && flutter build macos
```

## Key Ports

- **9000** — Second Life viewer connections (LLUDP)
- **8080** — Web client interface
- **9200** — Admin dashboard
- **9100** — Prometheus metrics

## Cargo Features

- `physics-zig` (default) — Zig physics engine via FFI
- `redis-cache` (default) — Redis caching support
- `jit` — Cranelift JIT compilation for LSL scripts

## Architecture Notes

- The Rust server communicates with the Zig physics engine via C FFI (`rust/src/ffi/` <-> `zig/src/ffi/`)
- Configuration uses figment with TOML files and environment variables
- Database migrations (38 total) auto-run on first startup via SQLx
- The server supports running multiple independent grid instances from a single installation
- AI NPCs are configured via `llm.ini` and support both paid API and local Ollama
- External dependency on GLC Player crates (`glc-core`, `glc-io`) via local path — ensure `../../../../GLC Player Mac/glc-player/` exists

## Development Conventions

- Use `tracing` for logging (not `println!`)
- Error handling: `anyhow::Result` for application errors, `thiserror` for library errors
- Async code uses Tokio runtime
- Database queries use SQLx with compile-time or runtime query checking
- Configuration follows OpenSim INI conventions where applicable
- Tests live in `tests/` (integration) and inline `#[cfg(test)]` modules (unit)

## Documentation

- `USER_MANUAL.md` — Deployment and operations guide
- `ARCHITECTURE.md` — System architecture overview
- `API_ENDPOINTS.md` — REST API reference
- `DATABASE_ARCHITECTURE.md` — Database schema docs
- `BUILDING.md` — Detailed build instructions
