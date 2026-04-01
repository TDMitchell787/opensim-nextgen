# OpenSim Next — Instance Management

Each subdirectory is a self-contained server instance with its own configuration,
regions, and logs. Instances are isolated from each other by port ranges and
separate databases.

## Directory Structure

```
Instances/
├── template/          # Blank template — copy to create a new instance
├── Gaiagrid/          # 16-region grid with Hypergrid + Robust
└── standalone_dev/    # (create as needed)
```

## Environment Variables

Set `OPENSIM_INSTANCE_DIR` to the instance directory path to load its `.env` file
automatically. All other env vars are read from the instance `.env`.

## Quick Start

```bash
# Start a specific instance
OPENSIM_INSTANCE_DIR=./Instances/Gaiagrid OPENSIM_SERVICE_MODE=robust ./target/release/opensim-next &
sleep 3
OPENSIM_INSTANCE_DIR=./Instances/Gaiagrid OPENSIM_SERVICE_MODE=grid ./target/release/opensim-next &
```

## Preflight Check

```bash
./target/release/opensim-next preflight --instance Gaiagrid
```
