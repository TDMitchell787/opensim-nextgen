#!/bin/bash
# failover.sh — pgpool-II automatic failover script
# Phase 199.4 — Gaia Grid
#
# Called by pgpool when a backend node fails.
# Arguments (from pgpool failover_command format string):
#   $1  = failed_node_id        (%d)
#   $2  = failed_host            (%h)
#   $3  = failed_port            (%p)
#   $4  = failed_data_dir        (%D)
#   $5  = new_main_node_id       (%m)
#   $6  = new_main_host          (%H)
#   $7  = new_main_port          (%M)
#   $8  = old_primary_node_id    (%P)
#   $9  = new_main_data_dir      (%r)
#   $10 = new_main_data_dir2     (%R)
#   $11 = old_primary_hostname   (%N)
#   $12 = old_primary_port       (%S)

FAILED_NODE_ID="$1"
FAILED_HOST="$2"
FAILED_PORT="$3"
FAILED_DATA_DIR="$4"
NEW_MAIN_ID="$5"
NEW_MAIN_HOST="$6"
NEW_MAIN_PORT="$7"
OLD_PRIMARY_ID="$8"
NEW_MAIN_DATA_DIR="$9"
NEW_MAIN_DATA_DIR2="${10}"
OLD_PRIMARY_HOST="${11}"
OLD_PRIMARY_PORT="${12}"

TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
LOG_DIR="/usr/local/var/pgpool_logs"
LOG_FILE="${LOG_DIR}/failover.log"

mkdir -p "$LOG_DIR"

log() {
    echo "${TIMESTAMP} [FAILOVER] $1" >> "$LOG_FILE"
    logger -t pgpool-failover "$1"
}

log "=== FAILOVER EVENT ==="
log "Failed node: id=${FAILED_NODE_ID} host=${FAILED_HOST}:${FAILED_PORT}"
log "Old primary: id=${OLD_PRIMARY_ID} host=${OLD_PRIMARY_HOST}:${OLD_PRIMARY_PORT}"
log "New main: id=${NEW_MAIN_ID} host=${NEW_MAIN_HOST}:${NEW_MAIN_PORT}"

if [ "${FAILED_NODE_ID}" = "${OLD_PRIMARY_ID}" ]; then
    log "PRIMARY FAILED — promoting standby node ${NEW_MAIN_ID} (${NEW_MAIN_HOST}:${NEW_MAIN_PORT})"

    if [ "${NEW_MAIN_HOST}" = "localhost" ] || [ "${NEW_MAIN_HOST}" = "127.0.0.1" ]; then
        # Local standby — use pg_ctl directly
        PG_CTL=$(which pg_ctl 2>/dev/null || echo "/usr/local/bin/pg_ctl")
        PROMOTE_CMD="${PG_CTL} promote -D ${NEW_MAIN_DATA_DIR2}"
        log "Executing: ${PROMOTE_CMD}"

        if ${PROMOTE_CMD} 2>>"${LOG_FILE}"; then
            log "SUCCESS: Standby promoted to primary"
        else
            log "ERROR: pg_ctl promote failed (exit $?)"
            exit 1
        fi
    else
        # Remote standby — use ssh
        PROMOTE_CMD="pg_ctl promote -D ${NEW_MAIN_DATA_DIR2}"
        log "Executing via SSH: ssh ${NEW_MAIN_HOST} '${PROMOTE_CMD}'"

        if ssh -T "${NEW_MAIN_HOST}" "${PROMOTE_CMD}" 2>>"${LOG_FILE}"; then
            log "SUCCESS: Remote standby promoted to primary"
        else
            log "ERROR: Remote pg_ctl promote failed (exit $?)"
            exit 1
        fi
    fi
else
    log "STANDBY FAILED — no promotion needed. Read queries will fall back to primary only."
fi

log "=== FAILOVER COMPLETE ==="
exit 0
