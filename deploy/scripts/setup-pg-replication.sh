#!/usr/bin/env bash
set -euo pipefail

# Phase 198.7: PostgreSQL Streaming Replication Setup
# Configures primary → standby streaming replication for OpenSim Next
#
# Usage:
#   PRIMARY:  ./setup-pg-replication.sh primary
#   STANDBY:  ./setup-pg-replication.sh standby <primary_host>
#   VERIFY:   ./setup-pg-replication.sh verify
#   FAILOVER: ./setup-pg-replication.sh failover

PG_VERSION="${PG_VERSION:-15}"
PG_DATA="${PGDATA:-/var/lib/postgresql/data}"
PG_CONF="${PG_DATA}/postgresql.conf"
PG_HBA="${PG_DATA}/pg_hba.conf"
REPL_USER="${REPL_USER:-replicator}"
REPL_PASSWORD="${REPL_PASSWORD:-repl_secure_$(openssl rand -hex 12)}"
REPL_SLOT="${REPL_SLOT:-opensim_standby}"
WAL_ARCHIVE="${WAL_ARCHIVE:-/var/lib/postgresql/wal_archive}"
STANDBY_SUBNET="${STANDBY_SUBNET:-10.0.0.0/8}"

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }
err() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" >&2; }

setup_primary() {
    log "=== Configuring PostgreSQL PRIMARY for streaming replication ==="

    if [ ! -f "$PG_CONF" ]; then
        err "PostgreSQL config not found at $PG_CONF"
        err "Set PGDATA to your PostgreSQL data directory"
        exit 1
    fi

    log "Creating replication user..."
    sudo -u postgres psql -c "
        DO \$\$
        BEGIN
            IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = '${REPL_USER}') THEN
                CREATE ROLE ${REPL_USER} WITH REPLICATION LOGIN PASSWORD '${REPL_PASSWORD}';
            ELSE
                ALTER ROLE ${REPL_USER} WITH PASSWORD '${REPL_PASSWORD}';
            END IF;
        END
        \$\$;
    "

    log "Creating replication slot..."
    sudo -u postgres psql -c "
        SELECT pg_create_physical_replication_slot('${REPL_SLOT}')
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_replication_slots WHERE slot_name = '${REPL_SLOT}'
        );
    "

    log "Creating WAL archive directory..."
    mkdir -p "$WAL_ARCHIVE"
    chown postgres:postgres "$WAL_ARCHIVE"
    chmod 700 "$WAL_ARCHIVE"

    log "Configuring postgresql.conf for replication..."

    cat >> "$PG_CONF" << PGCONF

# ============================================================
# Phase 198.7: Streaming Replication (Primary)
# Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')
# ============================================================

# WAL Settings
wal_level = replica
max_wal_senders = 5
max_replication_slots = 5
wal_keep_size = '1GB'

# WAL Archiving
archive_mode = on
archive_command = 'cp %p ${WAL_ARCHIVE}/%f'

# Synchronous Replication (optional — uncomment for zero data loss)
# synchronous_standby_names = 'opensim_standby'
# synchronous_commit = on

# Connection Settings
max_connections = 200
listen_addresses = '*'

# Performance Tuning for Replication
checkpoint_timeout = '15min'
checkpoint_completion_target = 0.9
wal_buffers = '64MB'
PGCONF

    log "Configuring pg_hba.conf for replication access..."

    if ! grep -q "replication.*${REPL_USER}" "$PG_HBA" 2>/dev/null; then
        cat >> "$PG_HBA" << PGHBA

# Phase 198.7: Replication access
host    replication     ${REPL_USER}    ${STANDBY_SUBNET}    scram-sha-256
host    replication     ${REPL_USER}    127.0.0.1/32         scram-sha-256
PGHBA
    fi

    log "Reloading PostgreSQL configuration..."
    sudo -u postgres pg_ctl reload -D "$PG_DATA"

    log ""
    log "=== PRIMARY CONFIGURATION COMPLETE ==="
    log ""
    log "Replication credentials (SAVE THESE):"
    log "  User:     ${REPL_USER}"
    log "  Password: ${REPL_PASSWORD}"
    log "  Slot:     ${REPL_SLOT}"
    log ""
    log "Next steps:"
    log "  1. Restart PostgreSQL: systemctl restart postgresql"
    log "  2. On standby server, run:"
    log "     REPL_PASSWORD='${REPL_PASSWORD}' ./setup-pg-replication.sh standby $(hostname -f)"
    log ""
}

setup_standby() {
    local primary_host="${1:-}"
    if [ -z "$primary_host" ]; then
        err "Usage: $0 standby <primary_host>"
        exit 1
    fi

    log "=== Configuring PostgreSQL STANDBY (primary: ${primary_host}) ==="

    if [ -d "$PG_DATA" ] && [ "$(ls -A $PG_DATA 2>/dev/null)" ]; then
        log "WARNING: Data directory ${PG_DATA} is not empty."
        log "This will DESTROY existing data. Are you sure? (yes/no)"
        read -r confirm
        if [ "$confirm" != "yes" ]; then
            log "Aborted."
            exit 0
        fi
        log "Stopping PostgreSQL..."
        sudo systemctl stop postgresql || true
        rm -rf "${PG_DATA:?}"/*
    fi

    log "Running pg_basebackup from ${primary_host}..."

    PGPASSWORD="${REPL_PASSWORD}" pg_basebackup \
        -h "$primary_host" \
        -U "$REPL_USER" \
        -D "$PG_DATA" \
        -Fp -Xs -P -R \
        -S "$REPL_SLOT" \
        --checkpoint=fast

    log "Configuring standby signal..."

    cat >> "${PG_DATA}/postgresql.conf" << PGCONF

# ============================================================
# Phase 198.7: Streaming Replication (Standby)
# Primary: ${primary_host}
# Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')
# ============================================================

primary_conninfo = 'host=${primary_host} port=5432 user=${REPL_USER} password=${REPL_PASSWORD} application_name=${REPL_SLOT}'
primary_slot_name = '${REPL_SLOT}'
hot_standby = on
hot_standby_feedback = on

recovery_target_timeline = 'latest'
restore_command = 'cp ${WAL_ARCHIVE}/%f %p 2>/dev/null || true'
PGCONF

    chown -R postgres:postgres "$PG_DATA"
    chmod 700 "$PG_DATA"

    log "Starting standby PostgreSQL..."
    sudo systemctl start postgresql

    log ""
    log "=== STANDBY CONFIGURATION COMPLETE ==="
    log ""
    log "Verify replication with:"
    log "  ./setup-pg-replication.sh verify"
    log ""
}

verify_replication() {
    log "=== Verifying PostgreSQL Streaming Replication ==="
    log ""

    log "--- Primary Status ---"
    sudo -u postgres psql -x -c "
        SELECT pid, usename, application_name, client_addr,
               state, sync_state,
               pg_wal_lsn_diff(sent_lsn, replay_lsn) AS replay_lag_bytes,
               write_lag, flush_lag, replay_lag
        FROM pg_stat_replication;
    " 2>/dev/null || log "(Not running as primary, or no standby connected)"

    log ""
    log "--- Replication Slots ---"
    sudo -u postgres psql -x -c "
        SELECT slot_name, active, restart_lsn,
               pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) AS slot_lag_bytes
        FROM pg_replication_slots;
    " 2>/dev/null || log "(No replication slots)"

    log ""
    log "--- Standby Status ---"
    sudo -u postgres psql -x -c "
        SELECT pg_is_in_recovery() AS is_standby,
               pg_last_wal_receive_lsn() AS last_received,
               pg_last_wal_replay_lsn() AS last_replayed,
               pg_last_xact_replay_timestamp() AS last_replay_time,
               NOW() - pg_last_xact_replay_timestamp() AS replay_age;
    " 2>/dev/null || log "(Not running as standby)"

    log ""
    log "--- WAL Archive ---"
    if [ -d "$WAL_ARCHIVE" ]; then
        local count
        count=$(find "$WAL_ARCHIVE" -name '0*' -type f 2>/dev/null | wc -l)
        log "WAL archive files: $count"
        log "Archive location: $WAL_ARCHIVE"
    else
        log "WAL archive directory not found at $WAL_ARCHIVE"
    fi

    log ""
    log "--- Database Size (metadata only after FSAssets migration) ---"
    sudo -u postgres psql -c "
        SELECT pg_size_pretty(pg_database_size('opensim')) AS db_size,
               (SELECT count(*) FROM fsassets) AS fsassets_count,
               (SELECT count(*) FROM assets) AS legacy_assets_count;
    " 2>/dev/null || sudo -u postgres psql -c "
        SELECT pg_size_pretty(pg_database_size('gaiagrid')) AS db_size;
    " 2>/dev/null || log "(Cannot query database size)"
}

do_failover() {
    log "=== Manual Failover: Promoting Standby to Primary ==="

    local is_standby
    is_standby=$(sudo -u postgres psql -tAc "SELECT pg_is_in_recovery();" 2>/dev/null || echo "error")

    if [ "$is_standby" != "t" ]; then
        err "This server is NOT a standby. Failover only works on standby servers."
        exit 1
    fi

    log "Current WAL position: $(sudo -u postgres psql -tAc "SELECT pg_last_wal_replay_lsn();")"
    log ""
    log "WARNING: This will promote this standby to PRIMARY."
    log "The old primary should NOT be restarted without reconfiguring."
    log "Proceed? (yes/no)"
    read -r confirm
    if [ "$confirm" != "yes" ]; then
        log "Aborted."
        exit 0
    fi

    log "Promoting standby to primary..."
    sudo -u postgres pg_ctl promote -D "$PG_DATA"

    sleep 2

    local check
    check=$(sudo -u postgres psql -tAc "SELECT pg_is_in_recovery();" 2>/dev/null)
    if [ "$check" = "f" ]; then
        log "SUCCESS: Server promoted to PRIMARY"
        log "New WAL position: $(sudo -u postgres psql -tAc "SELECT pg_current_wal_lsn();")"
        log ""
        log "Next steps:"
        log "  1. Update application DATABASE_URL to point to this server"
        log "  2. Reconfigure old primary as new standby (or replace)"
        log "  3. Re-establish replication slot on this new primary"
    else
        err "Promotion may have failed. Check PostgreSQL logs."
        exit 1
    fi
}

case "${1:-help}" in
    primary)
        setup_primary
        ;;
    standby)
        setup_standby "${2:-}"
        ;;
    verify)
        verify_replication
        ;;
    failover)
        do_failover
        ;;
    *)
        echo "Usage: $0 {primary|standby <host>|verify|failover}"
        echo ""
        echo "  primary          Configure this server as replication primary"
        echo "  standby <host>   Configure as standby replicating from <host>"
        echo "  verify           Check replication status on either primary or standby"
        echo "  failover         Promote this standby to primary (manual failover)"
        echo ""
        echo "Environment variables:"
        echo "  PGDATA           PostgreSQL data directory (default: /var/lib/postgresql/data)"
        echo "  REPL_USER        Replication username (default: replicator)"
        echo "  REPL_PASSWORD    Replication password (auto-generated if not set)"
        echo "  REPL_SLOT        Replication slot name (default: opensim_standby)"
        echo "  WAL_ARCHIVE      WAL archive directory (default: /var/lib/postgresql/wal_archive)"
        echo "  STANDBY_SUBNET   Allowed standby subnet (default: 10.0.0.0/8)"
        exit 1
        ;;
esac
