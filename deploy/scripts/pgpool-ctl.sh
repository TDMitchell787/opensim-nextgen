#!/bin/bash
# pgpool-ctl.sh — pgpool-II management script
# Phase 199.5 — Gaia Grid
#
# Usage: ./pgpool-ctl.sh {start|stop|restart|status|reload|attach|detach|monitor}

DEPLOY_DIR="$(cd "$(dirname "$0")/../pgpool" && pwd)"
PGPOOL_CONF="${DEPLOY_DIR}/pgpool.conf"
PCP_CONF="${DEPLOY_DIR}/pcp.conf"
HBA_CONF="${DEPLOY_DIR}/pool_hba.conf"
PID_FILE="/usr/local/var/pgpool-ii/pgpool.pid"
LOG_DIR="/usr/local/var/pgpool_logs"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

PCP_HOST="localhost"
PCP_PORT="9898"
PCP_USER="opensim"

check_shmmax() {
    local shmmax=$(sysctl -n kern.sysv.shmmax 2>/dev/null)
    if [ "$shmmax" -lt 167772160 ] 2>/dev/null; then
        echo "WARNING: kern.sysv.shmmax is ${shmmax} (need 167772160)"
        echo "Run: sudo sysctl -w kern.sysv.shmmax=167772160 kern.sysv.shmall=40960"
        return 1
    fi
    return 0
}

is_running() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE" 2>/dev/null)
        if kill -0 "$pid" 2>/dev/null; then
            return 0
        fi
    fi
    return 1
}

do_start() {
    if is_running; then
        echo "pgpool-II is already running (PID: $(cat $PID_FILE))"
        return 0
    fi

    check_shmmax || return 1

    mkdir -p "$LOG_DIR" /usr/local/var/pgpool-ii

    echo "Starting pgpool-II..."
    pgpool -n \
        -f "$PGPOOL_CONF" \
        -F "$PCP_CONF" \
        -a "$HBA_CONF" \
        2>>"${LOG_DIR}/pgpool-stderr.log" &

    sleep 3

    if is_running; then
        echo "pgpool-II started (PID: $(cat $PID_FILE))"
        echo "  pgpool port: 9999"
        echo "  PCP port: 9898"
    else
        echo "FAILED to start pgpool-II. Check ${LOG_DIR}/pgpool-$(date '+%Y-%m-%d').log"
        return 1
    fi
}

do_stop() {
    if ! is_running; then
        echo "pgpool-II is not running"
        return 0
    fi

    echo "Stopping pgpool-II..."
    pgpool -f "$PGPOOL_CONF" -F "$PCP_CONF" -a "$HBA_CONF" -m fast stop 2>&1
    echo "pgpool-II stopped"
}

do_restart() {
    do_stop
    sleep 2
    do_start
}

do_reload() {
    if ! is_running; then
        echo "pgpool-II is not running"
        return 1
    fi

    echo "Reloading pgpool-II configuration..."
    pgpool -f "$PGPOOL_CONF" -F "$PCP_CONF" -a "$HBA_CONF" reload 2>&1
    echo "Configuration reloaded"
}

do_status() {
    echo "=== pgpool-II Status ==="
    if is_running; then
        echo "  Status: RUNNING (PID: $(cat $PID_FILE))"
    else
        echo "  Status: STOPPED"
        return 1
    fi

    echo "  Config: ${PGPOOL_CONF}"
    echo ""

    psql -h localhost -p 9999 -U opensim -d gaiagrid -c "SHOW pool_nodes;" 2>/dev/null
    if [ $? -ne 0 ]; then
        echo "  Cannot connect to pgpool on port 9999"
    fi
}

do_attach() {
    local node_id="${1:-1}"
    echo "Attaching node ${node_id}..."
    pcp_attach_node -h "$PCP_HOST" -p "$PCP_PORT" -U "$PCP_USER" -w "$node_id" 2>&1
}

do_detach() {
    local node_id="${1:-1}"
    echo "Detaching node ${node_id}..."
    pcp_detach_node -h "$PCP_HOST" -p "$PCP_PORT" -U "$PCP_USER" -w "$node_id" 2>&1
}

do_monitor() {
    bash "${SCRIPT_DIR}/pgpool-monitor.sh" "$@"
}

case "$1" in
    start)    do_start ;;
    stop)     do_stop ;;
    restart)  do_restart ;;
    reload)   do_reload ;;
    status)   do_status ;;
    attach)   do_attach "$2" ;;
    detach)   do_detach "$2" ;;
    monitor)  shift; do_monitor "$@" ;;
    *)
        echo "Usage: $0 {start|stop|restart|reload|status|attach [node]|detach [node]|monitor [--watch|--json|--alert]}"
        exit 1
        ;;
esac
