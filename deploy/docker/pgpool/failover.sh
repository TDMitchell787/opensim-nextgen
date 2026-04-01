#!/bin/bash
# failover.sh — pgpool-II Docker failover script
# Phase 199.6 — OpenSim Next
#
# Called by pgpool when a backend node fails.
# In Docker, promotion uses pg_ctl via docker exec or direct SSH.

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

log() {
    echo "${TIMESTAMP} [FAILOVER] $1" >&2
}

log "=== FAILOVER EVENT ==="
log "Failed node: id=${FAILED_NODE_ID} host=${FAILED_HOST}:${FAILED_PORT}"
log "Old primary: id=${OLD_PRIMARY_ID} host=${OLD_PRIMARY_HOST}:${OLD_PRIMARY_PORT}"
log "New main: id=${NEW_MAIN_ID} host=${NEW_MAIN_HOST}:${NEW_MAIN_PORT}"

if [ "${FAILED_NODE_ID}" = "${OLD_PRIMARY_ID}" ]; then
    log "PRIMARY FAILED — promoting standby node ${NEW_MAIN_ID} (${NEW_MAIN_HOST}:${NEW_MAIN_PORT})"

    PGPASSWORD="${REPL_PASSWORD:-repl_secure_password}" psql -h "${NEW_MAIN_HOST}" -p "${NEW_MAIN_PORT}" -U opensim -d opensim -c "SELECT pg_promote();" 2>&1
    RESULT=$?

    if [ $RESULT -eq 0 ]; then
        log "SUCCESS: Standby promoted to primary via pg_promote()"
    else
        log "WARNING: pg_promote() returned $RESULT — standby may already be promoted or unreachable"
    fi
else
    log "STANDBY FAILED — no promotion needed. Read queries will fall back to primary only."
fi

log "=== FAILOVER COMPLETE ==="
exit 0
