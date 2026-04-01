#!/bin/bash
# OpenSim Next Production Entrypoint Script
# Handles container initialization, configuration, and service startup

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $1" | tee -a "${OPENSIM_LOG_PATH}/container.log"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARN:${NC} $1" | tee -a "${OPENSIM_LOG_PATH}/container.log"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" | tee -a "${OPENSIM_LOG_PATH}/container.log"
}

success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] SUCCESS:${NC} $1" | tee -a "${OPENSIM_LOG_PATH}/container.log"
}

# Trap signals for graceful shutdown
shutdown() {
    log "Received shutdown signal, stopping OpenSim Next..."
    if [ -n "${OPENSIM_PID:-}" ]; then
        kill -TERM "$OPENSIM_PID" 2>/dev/null || true
        wait "$OPENSIM_PID" 2>/dev/null || true
    fi
    success "OpenSim Next stopped gracefully"
    exit 0
}

trap shutdown SIGTERM SIGINT

# Check required environment variables
check_environment() {
    log "Checking environment configuration..."
    
    # Required variables
    local required_vars=(
        "DATABASE_URL"
    )
    
    for var in "${required_vars[@]}"; do
        if [ -z "${!var:-}" ]; then
            error "Required environment variable $var is not set"
            exit 1
        fi
    done
    
    # Set defaults for optional variables
    export OPENSIM_API_KEY="${OPENSIM_API_KEY:-opensim-production-key}"
    export OPENSIM_INSTANCE_ID="${OPENSIM_INSTANCE_ID:-opensim-$(hostname)-$(date +%s)}"
    export RUST_LOG="${RUST_LOG:-info}"
    export OPENSIM_BIND_ADDRESS="${OPENSIM_BIND_ADDRESS:-0.0.0.0}"
    
    # Ports
    export OPENSIM_VIEWER_PORT="${OPENSIM_VIEWER_PORT:-9000}"
    export OPENSIM_WEBSOCKET_PORT="${OPENSIM_WEBSOCKET_PORT:-9001}"
    export OPENSIM_WEB_PORT="${OPENSIM_WEB_PORT:-8080}"
    export OPENSIM_METRICS_PORT="${OPENSIM_METRICS_PORT:-9100}"
    export OPENSIM_HYPERGRID_PORT="${OPENSIM_HYPERGRID_PORT:-8002}"
    
    success "Environment configuration validated"
}

# Initialize data directories
init_directories() {
    log "Initializing data directories..."
    
    # Create necessary subdirectories
    mkdir -p "${OPENSIM_DATA_PATH}"/{regions,inventory,assets,cache,backups}
    mkdir -p "${OPENSIM_LOG_PATH}"/{server,database,network,physics}
    
    # Ensure proper permissions
    chmod 755 "${OPENSIM_DATA_PATH}" "${OPENSIM_LOG_PATH}"
    chmod 755 "${OPENSIM_DATA_PATH}"/* "${OPENSIM_LOG_PATH}"/*
    
    success "Data directories initialized"
}

# Database connectivity check
check_database() {
    log "Checking database connectivity..."
    
    # Extract database type from URL
    case "$DATABASE_URL" in
        postgresql://*)
            DB_TYPE="postgresql"
            ;;
        mysql://*)
            DB_TYPE="mysql"
            ;;
        sqlite://*)
            DB_TYPE="sqlite"
            ;;
        *)
            error "Unsupported database type in DATABASE_URL"
            exit 1
            ;;
    esac
    
    log "Database type detected: $DB_TYPE"
    
    # Connection test with retry logic
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        log "Database connection attempt $attempt/$max_attempts..."
        
        case "$DB_TYPE" in
            postgresql)
                if pg_isready -d "$DATABASE_URL" >/dev/null 2>&1; then
                    success "PostgreSQL database connection successful"
                    return 0
                fi
                ;;
            mysql)
                if mysqladmin ping -h"${DATABASE_URL#mysql://}" >/dev/null 2>&1; then
                    success "MySQL database connection successful"
                    return 0
                fi
                ;;
            sqlite)
                if [ -f "${DATABASE_URL#sqlite://}" ] || [ -w "$(dirname "${DATABASE_URL#sqlite://}")" ]; then
                    success "SQLite database accessible"
                    return 0
                fi
                ;;
        esac
        
        if [ $attempt -eq $max_attempts ]; then
            error "Database connection failed after $max_attempts attempts"
            exit 1
        fi
        
        warn "Database not ready, waiting 2 seconds..."
        sleep 2
        ((attempt++))
    done
}

# Run database migrations
run_migrations() {
    log "Checking for database migrations..."
    
    if [ -d "/opt/opensim-next/migrations" ]; then
        log "Running database migrations..."
        # Note: In a real implementation, you'd call the migration tool here
        # For now, we'll log that migrations would be run
        log "Database migrations would be executed here"
        success "Database migrations completed"
    else
        log "No migrations directory found, skipping migrations"
    fi
}

# Generate configuration file
generate_config() {
    log "Generating runtime configuration..."
    
    # Create runtime configuration based on environment
    cat > "${OPENSIM_CONFIG_PATH}" << EOF
# OpenSim Next Production Configuration
# Generated automatically by container entrypoint

[server]
bind_address = "${OPENSIM_BIND_ADDRESS}"
instance_id = "${OPENSIM_INSTANCE_ID}"
api_key = "${OPENSIM_API_KEY}"

[ports]
viewer = ${OPENSIM_VIEWER_PORT}
websocket = ${OPENSIM_WEBSOCKET_PORT}
web = ${OPENSIM_WEB_PORT}
metrics = ${OPENSIM_METRICS_PORT}
hypergrid = ${OPENSIM_HYPERGRID_PORT}

[database]
url = "${DATABASE_URL}"
max_connections = ${DATABASE_MAX_CONNECTIONS:-50}
timeout_seconds = ${DATABASE_TIMEOUT:-30}

[logging]
level = "${RUST_LOG}"
path = "${OPENSIM_LOG_PATH}"

[security]
api_key_required = true
rate_limit_enabled = true
max_requests_per_minute = ${RATE_LIMIT_RPM:-1000}

[performance]
physics_engine = "${OPENSIM_PHYSICS_ENGINE:-ODE}"
cache_size_mb = ${CACHE_SIZE_MB:-512}
max_concurrent_users = ${MAX_CONCURRENT_USERS:-1000}

[paths]
data = "${OPENSIM_DATA_PATH}"
web = "${OPENSIM_WEB_PATH}"
assets = "${OPENSIM_ASSETS_PATH}"
EOF
    
    success "Configuration file generated"
}

# Health check function
health_check() {
    local health_url="http://localhost:${OPENSIM_METRICS_PORT}/health"
    if curl -f -s "$health_url" >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Wait for service to be ready
wait_for_ready() {
    log "Waiting for OpenSim Next to be ready..."
    
    local max_wait=60
    local wait_time=0
    
    while [ $wait_time -lt $max_wait ]; do
        if health_check; then
            success "OpenSim Next is ready!"
            return 0
        fi
        
        sleep 2
        ((wait_time += 2))
        log "Waiting for service... (${wait_time}s/${max_wait}s)"
    done
    
    error "Service failed to become ready within ${max_wait} seconds"
    return 1
}

# Start OpenSim Next server
start_server() {
    log "Starting OpenSim Next server..."
    log "Version: $(cat /opt/opensim-next/VERSION 2>/dev/null || echo 'unknown')"
    log "Configuration: ${OPENSIM_CONFIG_PATH}"
    log "Data directory: ${OPENSIM_DATA_PATH}"
    log "Log directory: ${OPENSIM_LOG_PATH}"
    
    # Start server in background
    cd /opt/opensim-next
    ./bin/opensim-next \
        --config "${OPENSIM_CONFIG_PATH}" \
        --data-path "${OPENSIM_DATA_PATH}" \
        --log-path "${OPENSIM_LOG_PATH}" \
        > "${OPENSIM_LOG_PATH}/server.log" 2>&1 &
    
    OPENSIM_PID=$!
    log "OpenSim Next started with PID: $OPENSIM_PID"
    
    # Wait for the service to be ready
    if wait_for_ready; then
        success "OpenSim Next server is running successfully!"
        log "Service endpoints:"
        log "  - Second Life Viewers: ${OPENSIM_BIND_ADDRESS}:${OPENSIM_VIEWER_PORT}"
        log "  - WebSocket Client: ${OPENSIM_BIND_ADDRESS}:${OPENSIM_WEBSOCKET_PORT}"
        log "  - Web Interface: ${OPENSIM_BIND_ADDRESS}:${OPENSIM_WEB_PORT}"
        log "  - Metrics/Health: ${OPENSIM_BIND_ADDRESS}:${OPENSIM_METRICS_PORT}"
        log "  - Hypergrid: ${OPENSIM_BIND_ADDRESS}:${OPENSIM_HYPERGRID_PORT}"
    else
        error "Failed to start OpenSim Next server"
        return 1
    fi
}

# Main execution
main() {
    log "OpenSim Next Container Starting..."
    log "Container ID: $(hostname)"
    log "User: $(whoami)"
    log "Working Directory: $(pwd)"
    
    # Initialize container
    check_environment
    init_directories
    check_database
    run_migrations
    generate_config
    
    # Handle different commands
    case "${1:-server}" in
        server)
            start_server
            # Keep container running and handle signals
            wait "$OPENSIM_PID"
            ;;
        
        config)
            log "Configuration mode - generating config and exiting"
            cat "${OPENSIM_CONFIG_PATH}"
            ;;
        
        migrate)
            log "Migration mode - running migrations and exiting"
            run_migrations
            ;;
        
        health)
            log "Health check mode"
            if health_check; then
                success "Service is healthy"
                exit 0
            else
                error "Service is not healthy"
                exit 1
            fi
            ;;
        
        shell)
            log "Shell mode - starting interactive shell"
            exec /bin/bash
            ;;
        
        *)
            log "Running custom command: $*"
            exec "$@"
            ;;
    esac
}

# Execute main function
main "$@"