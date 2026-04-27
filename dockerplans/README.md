# OpenSim NextGen — Docker Deployment (Ubuntu 26.04)

A 4-region standalone grid with PostgreSQL, packaged as two containers
(`opensim-nextgen` and `opensim-postgres`) and built from a multi-stage
Ubuntu 26.04 image that compiles the Rust server, the Zig physics engine,
and the embedded sqlx schema in one shot.

## Quick Start

```bash
cd dockerplans/
./deploy.sh
```

That's it. `deploy.sh` runs preflight checks, copies `.env.example` to `.env`
on first run, builds the images, starts the stack, and verifies the server
is up. With no arguments it runs `check → build → up`.

### What the script checks before building

| Check | Why it matters |
|-------|----------------|
| Docker daemon reachable | uses `sudo` automatically if your user isn't in the `docker` group |
| `docker compose` plugin | apt-installs `docker-compose-v2` if missing |
| Nested overlayfs | the kernel refuses overlay-on-overlay; live-USB hosts need a separate disk for Docker's data-root |
| Disk space (≥20 GB free) | the build cache and image layers need room |
| Ports 9000-9003, 8080, 9100, 9200 free | `compose up` fails otherwise |
| `.env` present | created from `.env.example` if absent |

### Other commands

```bash
./deploy.sh check    # preflight only, don't build
./deploy.sh smoke    # fast cargo check in a throwaway container (~2 min)
./deploy.sh build    # just build images
./deploy.sh up       # start (skips build); runs postflight verification
./deploy.sh logs     # tail opensim-next container logs
./deploy.sh status   # compose ps
./deploy.sh down     # stop, keep data
./deploy.sh reset    # stop AND delete volumes (destroys all data)
```

`smoke` is the cheap version of the full build: it spins up an ephemeral
`rust:1.90` container, boots Postgres, applies the migrations, and runs
`cargo check --release`. That's enough to catch the entire class of errors
in [Issue #1](https://github.com/TDMitchell787/opensim-nextgen/issues/1)
(`error[E0282]: type annotations needed` on `sqlx::query!`) plus toolchain
version mismatches, in ~2 minutes instead of the ~10 minutes a full build
takes. The same checks run in CI on every push (`.github/workflows/build.yml`).

### When Docker's storage filesystem is unsuitable

On a live-USB Ubuntu image (or any environment where `/` is overlayfs),
Docker's default `overlay2` driver can't build images because the kernel
won't stack overlay on overlay. The script detects this and warns you.

If you have a spare disk available, point Docker at it:

```bash
# DESTRUCTIVE — wipes the named device, formats ext4, mounts at /mnt/docker-data,
# updates /etc/docker/daemon.json, restarts dockerd. Asks for confirmation.
sudo ./deploy.sh --format-device /dev/sdb
./deploy.sh                 # then build + up normally
```

The script refuses anything that isn't an `sd*`, `nvme*`, or `vd*` block device,
and asks you to retype the device path before wiping.

## What the build does

1. **Stage 1 (zig-builder)** — Ubuntu 26.04 + Zig 0.15.2; compiles
   `libopensim_physics.so` and `libopensim_ffi.so` against `libode-dev`
   and the vendored `bin/lib64/libBulletSim.so`.
2. **Stage 2 (rust-builder)** — Ubuntu 26.04 + Rust 1.90; pulls the Zig
   outputs, starts an in-build PostgreSQL, applies all migrations from
   `rust/migrations/postgres/`, then `cargo build --release` with
   `DATABASE_URL` set so sqlx compile-time query macros validate against
   the migrated schema. `RUSTFLAGS=-L /build/zig/zig-out/lib` lets the
   linker resolve `-lopensim_physics` / `-lopensim_ffi`.
3. **Stage 3 (production)** — minimal Ubuntu 26.04 + libssl3, libpq5,
   libsqlite3-0, libode8t64, postgresql-client. Ships the stripped binary,
   the Zig `.so` libs, and `bin/lib64/libBulletSim.so`.

Expect ~5-10 minutes the first time on a 16-core host with 47 GB RAM and
a fast SSD; ~2 minutes on subsequent builds with cache hits.

## Startup sequence

1. `opensim-postgres` starts and runs the postgres healthcheck.
2. `opensim-nextgen` waits for postgres healthy, then `entrypoint.sh`
   runs `opensim-next start --mode standalone`.
3. The Rust server runs its own SQLx migrations on first connect.
4. The seed-test-user SQL is applied if the tables exist.

## Connection Details

| Service | URL | Auth |
|---------|-----|------|
| SL Viewer Login | `http://localhost:9000` | none (XMLRPC POST) |
| Web Client | `http://localhost:8080` | none |
| Admin API | `http://localhost:9200` | `X-API-Key: <OPENSIM_API_KEY>` |
| Metrics / Health | `http://localhost:9100/health` | `X-API-Key: <OPENSIM_API_KEY>` |

The default API key is `changeme-opensim-nextgen-docker` — change it in `.env`.

> **Note on the Docker healthcheck.** The Dockerfile's `HEALTHCHECK` calls
> `curl -f http://localhost:9100/health` without the API-key header, so
> `docker ps` may show `health: unhealthy` even when the server is fine.
> The `deploy.sh up` postflight uses a log-based check instead, which is
> the authoritative signal.

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
  deploy.sh                     # Helper script: preflight, build, up, postflight
  .env                          # Environment variables (created from .env.example)
  docker-compose.yml            # Service orchestration
  Dockerfile.ubuntu26           # Multi-stage build (Zig + Rust + Postgres → Ubuntu 26.04)
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

Most operations are wrapped by `deploy.sh`. The raw `docker compose` calls
underneath:

```bash
# Stop                                        ./deploy.sh down
docker compose down

# Stop and remove data (fresh start)          ./deploy.sh reset
docker compose down -v

# Tail server logs                            ./deploy.sh logs
docker logs -f opensim-nextgen

# Shell into the container
docker exec -it opensim-nextgen bash

# Database shell
docker exec -it opensim-postgres psql -U opensim -d opensim
```

## Architecture

- **Ubuntu 26.04** base image; Rust 1.90 + Zig 0.15.2 compiled from source
- **PostgreSQL 16** on the internal Docker network (not exposed to host)
- **Standalone mode** — all grid services run in one process
- Schema migrations run twice: once in the build stage so sqlx compile-time
  query macros validate, and again at runtime on the live database
- Test user seeded after runtime migrations complete

## Troubleshooting

- **`overlay-on-overlay: invalid argument` during build** — your host's `/`
  is overlayfs (e.g. live-USB). Run `./deploy.sh --format-device /dev/sdX`
  to point Docker at a real disk.
- **`no space left on device`** — by default Docker uses `/var/lib/docker`
  on the root filesystem. Either free space, or use `--format-device` to
  move the data-root to a bigger disk.
- **`docker compose: command not found`** — `deploy.sh check` will install
  the `docker-compose-v2` apt package.
- **`permission denied while trying to connect to the Docker API`** —
  add yourself to the docker group (`sudo usermod -aG docker $USER` then
  log out / back in), or just let `deploy.sh` use `sudo` automatically.
- **`health: unhealthy` in `docker ps`** — known cosmetic issue; the
  Dockerfile's HEALTHCHECK doesn't pass the API key. Use `./deploy.sh logs`
  or hit `curl -H "X-API-Key: ..." http://localhost:9100/health` to verify.
