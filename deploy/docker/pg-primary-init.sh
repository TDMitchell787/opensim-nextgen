#!/usr/bin/env bash
set -e

echo "=== Phase 198.7: Configuring replication user and slot ==="

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    DO \$\$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'replicator') THEN
            CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD '${REPL_PASSWORD:-CHANGEME}';
        END IF;
    END
    \$\$;
EOSQL

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    SELECT pg_create_physical_replication_slot('opensim_standby')
    WHERE NOT EXISTS (
        SELECT 1 FROM pg_replication_slots WHERE slot_name = 'opensim_standby'
    );
EOSQL

echo "host replication replicator all scram-sha-256" >> "$PGDATA/pg_hba.conf"

echo "=== Replication user and slot created ==="
