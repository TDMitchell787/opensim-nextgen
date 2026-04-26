# OpenSim NextGen — Docker Deployment (Ubuntu 24.04)

4-region grid with PostgreSQL on Ubuntu 24.04.

## Quick Start

```bash
cd dockerplans/

# Review and customize settings
cp .env.example .env   # customize passwords, hostname, etc.

# Build and start
docker compose up -d --build

# Watch startup logs
docker logs -f opensim-nextgen
```

The server will:
1. Wait for PostgreSQL to be ready
2. Run database migrations automatically
3. Launch the setup wizard on first run (prompts for admin user)
4. Start all 4 regions
5. Seed the test user account

## Connection Details

| Service | URL |
|---------|-----|
| SL Viewer Login | `http://localhost:9000` |
| Web Client | `http://localhost:8080` |
| Admin API | `http://localhost:9200` |
| Health Check | `http://localhost:9100/health` |

### Viewer Setup (Firestorm)

1. Open Firestorm > Preferences > OpenSim
2. Add grid: `http://localhost:9000`
3. Login with your admin account or the test user

### Test User

| Field | Value |
|-------|-------|
| First Name | `Test` |
| Last Name | `User` |
| Password | `test_password` |

## Region Layout (2x2 Grid)

```
+-------------------+-------------------+
| Nature Reserve    | Sandbox Zone      |
| (1000,1001)       | (1001,1001)       |
| Port 9002         | Port 9003         |
+-------------------+-------------------+
| Welcome Plaza     | Builders Workshop |
| (1000,1000)       | (1001,1000)       |
| Port 9000 (home)  | Port 9001         |
+-------------------+-------------------+
```

Welcome Plaza is the default landing region.

## Port Reference

| Port | Protocol | Purpose |
|------|----------|---------|
| 9000 | TCP+UDP | Primary region (Welcome Plaza) + HTTP services |
| 9001 | UDP | Builders Workshop region |
| 9002 | UDP | Nature Reserve region |
| 9003 | UDP | Sandbox Zone region |
| 8080 | TCP | Web browser client |
| 9200 | TCP | Admin REST API |
| 9100 | TCP | Prometheus metrics + health endpoint |

## Configuration Files

```
dockerplans/
  .env                          # Environment variables
  docker-compose.yml            # Service orchestration
  Dockerfile.ubuntu24           # Multi-stage build (Rust+Zig → Ubuntu 24.04)
  entrypoint.sh                 # Container startup script
  postgres-init.sql             # Database initialization (monitoring, audit)
  seed-test-user.sql            # Test user account creation
  config/
    OpenSim.ini                 # Main server configuration
    llm.ini                     # AI NPC settings (disabled by default)
    Regions/
      Regions.ini               # 4-region grid definition
    config-include/
      Standalone.ini            # Standalone service connectors
      GridCommon.ini            # Grid mode connectors (for future use)
      osslDefaultEnable.ini     # OSSL function defaults
      osslEnable.ini            # OSSL local overrides
```

## Customization

### Change the external hostname

For LAN or public access, edit `.env`:

```
OPENSIM_EXTERNAL_HOSTNAME=192.168.1.100
```

### Enable AI NPCs

1. Install Ollama on the Docker host
2. Pull a model: `ollama pull llama4:scout`
3. Edit `config/llm.ini`: set `enabled = true` and `llm_enabled = true`
4. Restart: `docker compose restart opensim-next`

### Add more regions

Edit `config/Regions/Regions.ini` and add new sections. Each region needs:
- Unique UUID
- Unique grid location
- Unique InternalPort (9004, 9005, etc.)
- Corresponding port mapping in `docker-compose.yml`

### Switch to grid mode

For separate Robust server architecture, change in `.env`:
```
OPENSIM_SERVICE_MODE=grid
```
And update `OpenSim.ini` to use `Include-Architecture = "config-include/GridCommon.ini"`.

## Flutter Desktop Tools (Build Locally)

The server runs headless inside Docker. Two companion desktop apps are included
in the repo source at `flutter-client/` — these run on your Ubuntu desktop, not
inside the container.

### Prerequisites

```bash
# Install Flutter SDK
sudo snap install flutter --classic

# Install Linux desktop build dependencies
sudo apt-get update && sudo apt-get install -y \
    clang cmake ninja-build pkg-config \
    libgtk-3-dev liblzma-dev libstdc++-12-dev
    
# Enable Linux desktop target
flutter config --enable-linux-desktop
flutter doctor   # verify no issues
```

### Build the Grid Configurator

Desktop GUI for managing grid settings, regions, users, and server administration.

```bash
cd flutter-client/opensim_configurator
flutter pub get
flutter build linux
```

The built app is at:
`build/linux/x64/release/bundle/opensim_configurator`

### Build the User Manual Viewer

Documentation browser for the OpenSim NextGen user manual.

```bash
cd flutter-client/user_manual_viewer
flutter pub get
flutter build linux
```

The built app is at:
`build/linux/x64/release/bundle/user_manual_viewer`

### Connecting to the Server

Both apps connect to the Docker server endpoints:
- Admin API: `http://localhost:9200`
- Grid services: `http://localhost:9000`

If running on a different machine, replace `localhost` with the server's IP
(must match `OPENSIM_EXTERNAL_HOSTNAME` in `.env`).

## Management

```bash
# Stop
docker compose down

# Stop and remove data (fresh start)
docker compose down -v

# View logs
docker logs opensim-nextgen
docker logs opensim-postgres

# Shell into container
docker exec -it opensim-nextgen bash

# Database shell
docker exec -it opensim-postgres psql -U opensim -d opensim
```

## Architecture

- **Ubuntu 24.04** base image with Rust + Zig compiled from source
- **PostgreSQL 16** on internal Docker network (not exposed to host)
- **Standalone mode** — all grid services run in one process
- Migrations run automatically on startup via the Rust binary
- Test user seeded after migrations complete
- Setup wizard runs interactively on first boot if no config exists
