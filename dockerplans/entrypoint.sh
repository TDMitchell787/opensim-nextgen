#!/bin/bash
# OpenSim NextGen — Docker Container Entrypoint
# Handles initialization, database wait, config setup, and server startup

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log()     { echo -e "${BLUE}[$(date +'%H:%M:%S')] INFO:${NC} $1"; }
warn()    { echo -e "${YELLOW}[$(date +'%H:%M:%S')] WARN:${NC} $1"; }
error()   { echo -e "${RED}[$(date +'%H:%M:%S')] ERROR:${NC} $1"; }
success() { echo -e "${GREEN}[$(date +'%H:%M:%S')] OK:${NC} $1"; }

# Graceful shutdown
shutdown() {
    log "Shutting down OpenSim NextGen..."
    if [ -n "${OPENSIM_PID:-}" ]; then
        kill -TERM "$OPENSIM_PID" 2>/dev/null || true
        wait "$OPENSIM_PID" 2>/dev/null || true
    fi
    exit 0
}
trap shutdown SIGTERM SIGINT

check_environment() {
    if [ -z "${DATABASE_URL:-}" ]; then
        error "DATABASE_URL is required"
        exit 1
    fi

    export OPENSIM_API_KEY="${OPENSIM_API_KEY:-opensim-docker-key}"
    export OPENSIM_INSTANCE_ID="${OPENSIM_INSTANCE_ID:-opensim-docker-$(hostname)}"
    export RUST_LOG="${RUST_LOG:-info}"
    export OPENSIM_BIND_ADDRESS="${OPENSIM_BIND_ADDRESS:-0.0.0.0}"
    export OPENSIM_VIEWER_PORT="${OPENSIM_VIEWER_PORT:-9000}"
    export OPENSIM_WEB_PORT="${OPENSIM_WEB_PORT:-8080}"
    export OPENSIM_METRICS_PORT="${OPENSIM_METRICS_PORT:-9100}"
    export OPENSIM_ADMIN_PORT="${OPENSIM_ADMIN_PORT:-9200}"

    success "Environment validated"
}

init_directories() {
    mkdir -p "${OPENSIM_DATA_PATH}"/{regions,inventory,assets,cache,backups}
    mkdir -p "${OPENSIM_LOG_PATH}"/{server,database,network}
    success "Directories initialized"
}

wait_for_database() {
    log "Waiting for PostgreSQL..."

    # Extract host and port from DATABASE_URL
    local db_host db_port
    db_host=$(echo "$DATABASE_URL" | sed -n 's|.*@\([^:]*\):.*|\1|p')
    db_port=$(echo "$DATABASE_URL" | sed -n 's|.*:\([0-9]*\)/.*|\1|p')
    db_host="${db_host:-postgres}"
    db_port="${db_port:-5432}"

    local attempt=1
    local max_attempts=30

    while [ $attempt -le $max_attempts ]; do
        if pg_isready -h "$db_host" -p "$db_port" -q 2>/dev/null; then
            success "PostgreSQL is ready"
            return 0
        fi
        log "  Attempt $attempt/$max_attempts — waiting 2s..."
        sleep 2
        ((attempt++))
    done

    error "PostgreSQL not available after $max_attempts attempts"
    exit 1
}

apply_hostname() {
    local external_host="${OPENSIM_EXTERNAL_HOSTNAME:-SYSTEMIP}"
    log "Applying external hostname: $external_host"

    # Replace SYSTEMIP in config files with the actual hostname
    if [ "$external_host" != "SYSTEMIP" ]; then
        find "${OPENSIM_CONFIG_PATH}" -name "*.ini" -exec \
            sed -i "s/SYSTEMIP/$external_host/g" {} +
    fi
}

seed_test_user() {
    if [ "${SEED_TEST_USER:-true}" = "true" ] && [ -f /opt/opensim-next/seed-test-user.sql ]; then
        local db_host db_port db_name db_user
        db_host=$(echo "$DATABASE_URL" | sed -n 's|.*@\([^:]*\):.*|\1|p')
        db_port=$(echo "$DATABASE_URL" | sed -n 's|.*:\([0-9]*\)/.*|\1|p')
        db_name=$(echo "$DATABASE_URL" | sed -n 's|.*/\([^?]*\).*|\1|p')
        db_user=$(echo "$DATABASE_URL" | sed -n 's|.*://\([^:]*\):.*|\1|p')
        db_host="${db_host:-postgres}"
        db_port="${db_port:-5432}"
        db_name="${db_name:-opensim}"
        db_user="${db_user:-opensim}"

        log "Seeding test user (Test User / test_password)..."
        PGPASSWORD=$(echo "$DATABASE_URL" | sed -n 's|.*://[^:]*:\([^@]*\)@.*|\1|p') \
            psql -h "$db_host" -p "$db_port" -U "$db_user" -d "$db_name" \
            -f /opt/opensim-next/seed-test-user.sql 2>/dev/null || \
            warn "Test user seed skipped (tables may not exist yet — user will be created on first login via setup wizard)"
    fi
}

health_check() {
    curl -f -s "http://localhost:${OPENSIM_METRICS_PORT}/health" >/dev/null 2>&1
}

start_server() {
    log "Starting OpenSim NextGen server..."
    log "  Config:  ${OPENSIM_CONFIG_PATH}"
    log "  Data:    ${OPENSIM_DATA_PATH}"
    log "  Regions: 4 (Welcome Plaza, Builders Workshop, Nature Reserve, Sandbox Zone)"
    log "  Ports:   9000-9003 (LLUDP), 8080 (web), 9200 (admin), 9100 (metrics)"

    cd /opt/opensim-next
    ./bin/opensim-next \
        --config "${OPENSIM_CONFIG_PATH}" \
        --data-path "${OPENSIM_DATA_PATH}" \
        --log-path "${OPENSIM_LOG_PATH}" \
        > "${OPENSIM_LOG_PATH}/server.log" 2>&1 &

    OPENSIM_PID=$!
    log "Server started (PID: $OPENSIM_PID)"

    # Wait for startup then seed test user
    local wait=0
    while [ $wait -lt 60 ]; do
        if health_check; then
            success "Server is ready!"
            seed_test_user
            echo ""
            success "============================================"
            success " OpenSim NextGen Grid is running!"
            success "============================================"
            log "  Login URI:  http://${OPENSIM_EXTERNAL_HOSTNAME:-localhost}:9000"
            log "  Web client: http://${OPENSIM_EXTERNAL_HOSTNAME:-localhost}:8080"
            log "  Admin API:  http://${OPENSIM_EXTERNAL_HOSTNAME:-localhost}:9200"
            log ""
            log "  On first run, the setup wizard will prompt"
            log "  you to create your first user account."
            log ""
            log "  Test user:  Test User / test_password"
            log "  (if seeding succeeded)"
            echo ""
            return 0
        fi
        sleep 2
        ((wait += 2))
    done

    warn "Server may still be starting — check logs at ${OPENSIM_LOG_PATH}/server.log"
}

main() {
    echo ""
    log "============================================"
    log " OpenSim NextGen — Docker Container"
    log " Ubuntu 24.04 | 4 Regions | PostgreSQL"
    log "============================================"
    echo ""

    check_environment
    init_directories
    wait_for_database
    apply_hostname

    case "${1:-server}" in
        server)
            start_server
            wait "$OPENSIM_PID"
            ;;
        config)
            log "Showing configuration:"
            cat "${OPENSIM_CONFIG_PATH}/OpenSim.ini"
            ;;
        health)
            if health_check; then
                success "Healthy"
            else
                error "Not healthy"
                exit 1
            fi
            ;;
        shell)
            exec /bin/bash
            ;;
        *)
            exec "$@"
            ;;
    esac
}

main "$@"
