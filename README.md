# OpenSim NextGen

A high-performance virtual world server built with Rust and Zig, featuring a Flutter desktop client. OpenSim NextGen is a modern reimplementation of the OpenSimulator platform found on Opensimular.org, designed for performance, scalability, and production readiness. While this can do single simulators it is a powerful grid system as well. This port used Claude Code Opus 4.6 and compiles easily.  The AI-NPC can be renamed in llm.ini and is a new and powerful tool that I will continue to develop. While the AI-NPC can operate with an paid account it also supports user supplied Ollama for your own llm.

## Features

- **Hybrid Rust/Zig Architecture** — Rust for networking, services, and simulation logic; Zig for physics, collision, and SIMD math
- **Multi-Protocol Support** — Compatible with Second Life viewers (Firestorm, etc.) and web browsers simultaneously
- **Quad-Database Support** — PostgreSQL, MySQL, MariaDB, and SQLite backends
- **Instance Management** — Run multiple independent grid instances from a single installation
- **AI Integration** — Optional LLM-powered NPC director system in 14 areas with 140 skills
- **Flutter Desktop Client** — Native macOS configuration tool and user manual viewer
- **Docker & Kubernetes** — Production deployment templates included

## Quick Start

### Prerequisites

- **Rust** 1.70+ with Cargo
- **Zig** 0.14+
- **PostgreSQL** or **SQLite** for data persistence
- **Flutter** 3.x (for desktop client builds)
- macOS 14+, Linux, or Windows (macOS primary development platform)

### Build

```bash
# 1. Build the Zig physics engine
cd zig
zig build
cd ..

# 2. Build the Rust server
cargo build --release

# 3. (Optional) Build the Flutter configurator
cd flutter-client/opensim_configurator
flutter build macos
cd ../..
```

### Configure

```bash
# Copy the example environment file
cp .env.example .env

# Edit with your database and network settings
# At minimum, set DATABASE_URL and OPENSIM_API_KEY
```

### Run

```bash
# Start the server
RUST_LOG=info DYLD_LIBRARY_PATH=./zig/zig-out/lib ./target/release/opensim-next
```

The server starts on:
- **Port 9000** — Second Life viewer connections (LLUDP) or as set in Opensim.ini and Regions.ini
- **Port 8080** — Web client interface
- **Port 9200** — Admin dashboard
- **Port 9100** — Prometheus metrics

### Create an Instance

```bash
# Copy the instance template
cp -r Instances/template Instances/my-grid

# Configure the instance
cp Instances/template/.env.template Instances/my-grid/.env
# Edit Instances/my-grid/.env with your settings
# Edit Instances/my-grid/Regions/Regions.ini with your region definitions
```

See `BUILDING.md` for detailed build instructions and `USER_MANUAL.md` for the full deployment guide.

## Project Structure

```
opensim-nextgen/
  rust/               # Rust server source (networking, services, game logic)
  zig/                # Zig physics engine (collision, raycasting, SIMD math)
  flutter-client/     # Flutter desktop applications
    opensim_configurator/   # Grid configuration tool
    user_manual_viewer/     # Documentation viewer
  bin/                # Instance creation base files
    assets/           # Default content library (animations, textures, sounds)
    config-include/   # Configuration templates
    openmetaverse_data/  # Avatar definitions and skeleton data
    ScriptEngines/    # LSL script engine configs
  config/             # Grid configuration
  deploy/             # Docker & Kubernetes deployment
  docs/               # Documentation
  Instances/          # Server instance directories
    template/         # Template for new instances
  Terrain/            # Terrain heightmap files
  tests/              # Integration and unit tests
```

## Documentation

- `USER_MANUAL.md` — Comprehensive deployment and operations guide
- `ARCHITECTURE.md` — System architecture overview
- `API_ENDPOINTS.md` — REST API reference
- `DATABASE_ARCHITECTURE.md` — Database schema documentation
- `BUILDING.md` — Detailed build instructions
- `docs/` — Additional guides (security hardening, monitoring, WebSocket client)

## External Content Resources

Avatar mesh models and LSL script libraries are maintained in separate repositories:

- **Ruth2/Roth2 Avatar Mesh** — [github.com/RuthAndRoth](https://github.com/RuthAndRoth)
- **LSL Script Library** — https://github.com/Outworldz 

## Database Support

| Backend | Use Case | Config |
|---------|----------|--------|
| PostgreSQL | Production (recommended) | `postgresql://user:pass@host/db` |
| MySQL | Enterprise alternative | `mysql://user:pass@host/db` |
| MariaDB | Open-source MySQL | `mariadb://user:pass@host/db` |
| SQLite | Development/testing | `sqlite:///path/to/file.db` |

All 38 migrations auto-initialize on first startup.

## License

BSD 3-Clause License. See `LICENSE` for details.
