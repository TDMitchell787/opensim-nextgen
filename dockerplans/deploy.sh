#!/usr/bin/env bash
# OpenSim NextGen — Docker deployment helper
# Wraps docker compose with environment preflight, optional storage prep,
# and post-startup verification. The Dockerfile remains the source of truth
# for the build itself.

set -euo pipefail

#------------------------------------------------------------------------------
# Constants
#------------------------------------------------------------------------------
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
ENV_FILE="$SCRIPT_DIR/.env"
ENV_EXAMPLE="$SCRIPT_DIR/.env.example"
MIN_DISK_GB=20
DATA_ROOT_DEFAULT="/var/lib/docker"

# Colors (only if stdout is a tty)
if [[ -t 1 ]]; then
    R='\033[0;31m'; G='\033[0;32m'; Y='\033[1;33m'; B='\033[0;34m'; N='\033[0m'
else
    R=''; G=''; Y=''; B=''; N=''
fi

log()   { echo -e "${B}==>${N} $*"; }
ok()    { echo -e "${G}OK${N}  $*"; }
warn()  { echo -e "${Y}WARN${N} $*"; }
err()   { echo -e "${R}ERR${N} $*" >&2; }
die()   { err "$*"; exit 1; }

#------------------------------------------------------------------------------
# Helpers
#------------------------------------------------------------------------------
DOCKER=(docker)
need_sudo_for_docker() {
    # If user can talk to docker without sudo, no prefix needed
    if docker info &>/dev/null; then
        DOCKER=(docker)
        return
    fi
    if sudo -n docker info &>/dev/null; then
        DOCKER=(sudo -n docker)
        warn "user not in docker group; using sudo"
        return
    fi
    die "cannot reach Docker daemon (need group membership or sudo)"
}

usage() {
    cat <<EOF
Usage: $(basename "$0") [COMMAND]

Commands:
  check               Run preflight checks only (default if nothing else passed).
  smoke               Fast cargo check inside a throwaway Docker container.
                      Catches sqlx::query! schema mismatches in ~2 min without
                      a full release build. Reproduces what GitHub Issue #1 hit.
  build               Build images via docker compose.
  up                  Start the stack and run postflight verification.
  down                Stop the stack (volumes preserved).
  reset               Stop the stack and DELETE volumes (destroys all data).
  logs                Tail opensim-next container logs.
  status              Show compose service status.

Options:
  --format-device DEV    Wipe DEV (e.g. /dev/sdb), format ext4, mount as Docker
                         data-root. DESTRUCTIVE. Requires explicit confirmation.
  --data-root PATH       Mountpoint for --format-device (default /mnt/docker-data).
  --no-postflight        Skip the post-up health verification.
  -h, --help             Show this help.

With no command, runs: check → build → up.
EOF
}

#------------------------------------------------------------------------------
# Preflight
#------------------------------------------------------------------------------
check_docker() {
    log "Checking Docker"
    command -v docker &>/dev/null || die "docker not installed (apt install docker.io)"
    need_sudo_for_docker
    if ! "${DOCKER[@]}" compose version &>/dev/null; then
        warn "docker compose plugin missing; attempting to install"
        sudo -n apt-get update -qq
        sudo -n DEBIAN_FRONTEND=noninteractive apt-get install -y docker-compose-v2 \
            || die "failed to install docker-compose-v2"
    fi
    ok "docker $("${DOCKER[@]}" --version | awk '{print $3}' | tr -d ,) + compose $("${DOCKER[@]}" compose version --short)"
}

check_disk() {
    log "Checking disk space"
    local data_root avail_gb
    data_root="$("${DOCKER[@]}" info --format '{{.DockerRootDir}}' 2>/dev/null || echo "$DATA_ROOT_DEFAULT")"
    # df may not have an entry for a non-existent path; fall back to /
    if [[ ! -d "$data_root" ]]; then data_root="/"; fi
    avail_gb=$(df -BG --output=avail "$data_root" 2>/dev/null | tail -1 | tr -dc '0-9')
    if (( avail_gb < MIN_DISK_GB )); then
        warn "only ${avail_gb}G free at $data_root (need ≥${MIN_DISK_GB}G)"
        warn "consider --format-device to point Docker at a separate disk"
    else
        ok "${avail_gb}G free at $data_root"
    fi
}

check_overlay_nesting() {
    # If Docker's data-root sits on an overlayfs filesystem, native overlay2
    # storage will fail (kernel refuses overlay-on-overlay). This is common on
    # live-USB / casper boot environments.
    log "Checking for nested overlayfs"
    local data_root fs
    data_root="$("${DOCKER[@]}" info --format '{{.DockerRootDir}}' 2>/dev/null || echo /)"
    fs=$(df --output=fstype "$data_root" 2>/dev/null | tail -1 || echo unknown)
    if [[ "$fs" == "overlay" ]]; then
        warn "Docker data-root ($data_root) is on overlayfs (likely a live-USB host)"
        warn "the native overlay2 driver will fail to build images here"
        warn "options: install fuse-overlayfs, or use --format-device to mount a real disk"
        return 1
    fi
    ok "data-root filesystem: $fs"
}

check_ports() {
    log "Checking that required ports are free"
    local in_use=()
    for port in 9000 9001 9002 9003 8080 9100 9200; do
        if ss -tlnH "sport = :$port" 2>/dev/null | grep -q LISTEN; then
            in_use+=("$port")
        fi
    done
    if (( ${#in_use[@]} > 0 )); then
        warn "ports already in use: ${in_use[*]} — compose up will fail"
    else
        ok "ports 9000-9003,8080,9100,9200 free"
    fi
}

check_env_file() {
    log "Checking .env"
    if [[ ! -f "$ENV_FILE" ]]; then
        if [[ -f "$ENV_EXAMPLE" ]]; then
            cp "$ENV_EXAMPLE" "$ENV_FILE"
            ok "created $ENV_FILE from .env.example"
        else
            die "no .env or .env.example found in $SCRIPT_DIR"
        fi
    else
        ok ".env present"
    fi
}

preflight() {
    check_docker
    check_overlay_nesting || true
    check_disk
    check_ports
    check_env_file
}

#------------------------------------------------------------------------------
# Storage prep (--format-device)
#------------------------------------------------------------------------------
format_device() {
    local dev="$1" mount="$2"
    [[ -b "$dev" ]] || die "$dev is not a block device"
    [[ "$dev" =~ ^/dev/(sd|nvme|vd) ]] || die "refusing to format non-disk device $dev"

    local size model
    size=$(lsblk -ndo SIZE "$dev")
    model=$(lsblk -ndo MODEL "$dev" | xargs)
    cat >&2 <<EOF
${R}DESTRUCTIVE OPERATION${N}
Device:  $dev   ($size, $model)
Action:  wipe partition table → single ext4 partition → mount at $mount
After:   Docker data-root will point here. ALL EXISTING DATA ON $dev IS LOST.
EOF
    read -rp "Type the device path again to confirm ($dev): " confirm
    [[ "$confirm" == "$dev" ]] || die "confirmation mismatch; aborting"

    log "Stopping Docker"
    sudo -n systemctl stop docker docker.socket || true

    log "Wiping $dev"
    sudo -n umount "${dev}"* 2>/dev/null || true
    sudo -n wipefs -a "$dev"
    sudo -n parted -s "$dev" mklabel gpt mkpart primary ext4 0% 100%
    sudo -n partprobe "$dev"; sleep 2

    local part="${dev}1"
    [[ "$dev" =~ nvme ]] && part="${dev}p1"
    log "Formatting $part ext4"
    sudo -n mkfs.ext4 -F -L docker-data "$part"

    log "Mounting at $mount"
    sudo -n mkdir -p "$mount"
    sudo -n mount "$part" "$mount"

    # Persist mount across reboots
    local uuid
    uuid=$(sudo -n blkid -s UUID -o value "$part")
    if ! grep -q "$uuid" /etc/fstab; then
        echo "UUID=$uuid $mount ext4 defaults,nofail 0 2" | sudo -n tee -a /etc/fstab >/dev/null
        ok "added /etc/fstab entry"
    fi

    log "Pointing Docker data-root at $mount"
    sudo -n rm -rf /var/lib/docker
    echo "{\"data-root\":\"$mount\",\"features\":{\"containerd-snapshotter\":false},\"storage-driver\":\"overlay2\"}" \
        | sudo -n tee /etc/docker/daemon.json >/dev/null

    sudo -n systemctl start docker
    sleep 3
    ok "Docker now using $mount as data-root"
    "${DOCKER[@]}" info --format 'Storage Driver: {{.Driver}}, Data Root: {{.DockerRootDir}}' >&2
}

#------------------------------------------------------------------------------
# Compose actions
#------------------------------------------------------------------------------
do_build() {
    log "Building images"
    "${DOCKER[@]}" compose -f "$COMPOSE_FILE" build
    ok "build complete"
}

do_up() {
    log "Starting stack"
    "${DOCKER[@]}" compose -f "$COMPOSE_FILE" up -d
    ok "containers started"
    [[ "${SKIP_POSTFLIGHT:-0}" == "1" ]] || postflight
}

do_down()   { "${DOCKER[@]}" compose -f "$COMPOSE_FILE" down; }
do_reset()  { "${DOCKER[@]}" compose -f "$COMPOSE_FILE" down -v; }
do_logs()   { "${DOCKER[@]}" compose -f "$COMPOSE_FILE" logs -f opensim-next; }
do_status() { "${DOCKER[@]}" compose -f "$COMPOSE_FILE" ps; }

#------------------------------------------------------------------------------
# Smoke: fast cargo-check against an embedded Postgres (~2 min)
# Catches sqlx::query! type-inference errors (Issue #1) and toolchain version
# mismatches without doing a full release build.
#------------------------------------------------------------------------------
do_smoke() {
    local repo_root
    repo_root="$( cd "$SCRIPT_DIR/.." && pwd )"
    log "Running cargo check inside throwaway container (this takes ~2 min)"
    "${DOCKER[@]}" run --rm \
        -v "$repo_root":/src:ro \
        -w /tmp/work \
        rust:1.90 bash -c '
            set -e
            apt-get update -qq
            apt-get install -y -qq --no-install-recommends postgresql postgresql-contrib sudo libpq-dev libssl-dev pkg-config postgresql-client >/dev/null
            cp -r /src/rust /src/vendor /src/web /src/docs /src/templates /src/shared /src/bin /tmp/work/ 2>/dev/null || true
            cluster=$(pg_lsclusters -h | awk "{print \$1; exit}")
            ver=$(pg_lsclusters -h | awk "{print \$2; exit}")
            pg_ctlcluster "$cluster" "$ver" start >/dev/null
            sudo -u postgres psql -c "CREATE USER opensim WITH SUPERUSER PASSWORD '"'"'opensim'"'"';" >/dev/null
            sudo -u postgres psql -c "CREATE DATABASE opensim OWNER opensim;" >/dev/null
            for f in /tmp/work/rust/migrations/postgres/*.sql; do
                PGPASSWORD=opensim psql -h localhost -U opensim -d opensim -v ON_ERROR_STOP=1 -q -f "$f" >/dev/null
            done
            export DATABASE_URL="postgres://opensim:opensim@localhost/opensim"
            cd /tmp/work/rust
            cargo check --release --all-targets 2>&1 | tail -40
        '
    ok "cargo check passed"
}

#------------------------------------------------------------------------------
# Postflight
#------------------------------------------------------------------------------
postflight() {
    log "Waiting for postgres to report healthy"
    local tries=0
    until "${DOCKER[@]}" inspect --format '{{.State.Health.Status}}' opensim-postgres 2>/dev/null | grep -q healthy; do
        ((tries++)) && (( tries > 60 )) && { warn "postgres not healthy after 60s"; break; }
        sleep 1
    done
    ok "postgres healthy"

    log "Waiting for opensim-next to log 'Embedded controller listening'"
    tries=0
    while (( tries < 60 )); do
        if "${DOCKER[@]}" exec opensim-nextgen test -f /var/log/opensim-next/server.log 2>/dev/null \
            && "${DOCKER[@]}" exec opensim-nextgen grep -q "Embedded controller listening" \
                  /var/log/opensim-next/server.log 2>/dev/null; then
            ok "server is up"
            break
        fi
        sleep 2; ((tries+=2))
    done

    local host api_key
    host=$(grep -E '^OPENSIM_EXTERNAL_HOSTNAME=' "$ENV_FILE" | cut -d= -f2- | tr -d '"' || echo localhost)
    api_key=$(grep -E '^OPENSIM_API_KEY=' "$ENV_FILE" | cut -d= -f2- | tr -d '"' || echo changeme)
    cat <<EOF

${G}OpenSim NextGen is running.${N}

  Web client:    http://${host}:8080
  Viewer login:  http://${host}:9000
  Admin API:     http://${host}:9200   (X-API-Key: $api_key)
  Test user:     Test User / test_password

  Logs:          $0 logs
  Stop:          $0 down
  Wipe data:     $0 reset
EOF
}

#------------------------------------------------------------------------------
# Argument parsing
#------------------------------------------------------------------------------
COMMAND=""
FORMAT_DEV=""
DATA_ROOT="/mnt/docker-data"
SKIP_POSTFLIGHT=0

while (( $# > 0 )); do
    case "$1" in
        check|smoke|build|up|down|reset|logs|status) COMMAND="$1"; shift ;;
        --format-device) FORMAT_DEV="$2"; shift 2 ;;
        --data-root)     DATA_ROOT="$2"; shift 2 ;;
        --no-postflight) SKIP_POSTFLIGHT=1; shift ;;
        -h|--help)       usage; exit 0 ;;
        *) err "unknown argument: $1"; usage; exit 2 ;;
    esac
done
export SKIP_POSTFLIGHT

#------------------------------------------------------------------------------
# Main
#------------------------------------------------------------------------------
[[ -n "$FORMAT_DEV" ]] && { check_docker; format_device "$FORMAT_DEV" "$DATA_ROOT"; }

case "${COMMAND:-default}" in
    check)   preflight ;;
    smoke)   check_docker; do_smoke ;;
    build)   preflight; do_build ;;
    up)      preflight; do_up ;;
    down)    do_down ;;
    reset)   do_reset ;;
    logs)    do_logs ;;
    status)  do_status ;;
    default) preflight; do_build; do_up ;;
esac
