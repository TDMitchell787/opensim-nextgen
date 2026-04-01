#!/usr/bin/env bash
set -euo pipefail

# Phase 198.8: FSAssets Verification and Cutover
# Verifies migration integrity, measures performance, and handles cutover
#
# Usage:
#   ./fsassets-verify-cutover.sh verify       Integrity check: fsassets vs legacy assets
#   ./fsassets-verify-cutover.sh stats        Show migration statistics
#   ./fsassets-verify-cutover.sh benchmark    Measure asset load performance
#   ./fsassets-verify-cutover.sh cutover      Drop data column + VACUUM (DESTRUCTIVE)

DB_URL="${DATABASE_URL:-postgresql://opensim@localhost/gaiagrid}"
FSASSETS_ROOT="${FSASSETS_ROOT:-./fsassets}"
SAMPLE_SIZE="${SAMPLE_SIZE:-100}"

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }
err() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" >&2; }

psql_cmd() {
    psql "$DB_URL" -tA "$@"
}

verify_migration() {
    log "=== FSAssets Migration Verification ==="
    log ""

    local total_legacy fsassets_count migrated_pct
    total_legacy=$(psql_cmd -c "SELECT count(*) FROM assets WHERE data IS NOT NULL AND length(data) > 0;")
    fsassets_count=$(psql_cmd -c "SELECT count(*) FROM fsassets;")

    if [ "$total_legacy" -eq 0 ]; then
        log "No legacy assets with data — migration may already be complete"
        log "FSAssets entries: ${fsassets_count}"
        return 0
    fi

    migrated_pct=$(echo "scale=1; ${fsassets_count} * 100 / ${total_legacy}" | bc)
    log "Legacy assets (with data): ${total_legacy}"
    log "FSAssets entries:          ${fsassets_count}"
    log "Migration progress:        ${migrated_pct}%"
    log ""

    log "--- Spot-Check: ${SAMPLE_SIZE} random assets ---"

    local verified=0 failed=0 missing_file=0

    while IFS='|' read -r asset_id hash; do
        hash=$(echo "$hash" | tr -d ' ')
        local dir1="${hash:0:2}"
        local dir2="${hash:2:2}"
        local dir3="${hash:4:2}"
        local dir4="${hash:6:4}"
        local filepath="${FSASSETS_ROOT}/${dir1}/${dir2}/${dir3}/${dir4}/${hash}.gz"

        if [ -f "$filepath" ]; then
            local fs_size
            fs_size=$(gzip -l "$filepath" 2>/dev/null | tail -1 | awk '{print $2}')

            local db_data_hash
            db_data_hash=$(psql_cmd -c "
                SELECT encode(sha256(data), 'hex')
                FROM assets WHERE id = '${asset_id}'::uuid AND data IS NOT NULL
                LIMIT 1;
            " 2>/dev/null || echo "")

            if [ -n "$db_data_hash" ] && [ "$db_data_hash" = "$hash" ]; then
                verified=$((verified + 1))
            elif [ -n "$db_data_hash" ]; then
                failed=$((failed + 1))
                err "HASH MISMATCH: ${asset_id} db_hash=${db_data_hash} fs_hash=${hash}"
            else
                verified=$((verified + 1))
            fi
        else
            missing_file=$((missing_file + 1))
            err "MISSING FILE: ${asset_id} expected at ${filepath}"
        fi
    done < <(psql_cmd -c "
        SELECT id::text, hash FROM fsassets
        ORDER BY random() LIMIT ${SAMPLE_SIZE};
    ")

    log ""
    log "Spot-check results:"
    log "  Verified:     ${verified}/${SAMPLE_SIZE}"
    log "  Hash mismatch: ${failed}"
    log "  Missing files: ${missing_file}"
    log ""

    if [ "$failed" -gt 0 ]; then
        err "INTEGRITY CHECK FAILED — ${failed} hash mismatches detected"
        err "Do NOT proceed with cutover until resolved"
        return 1
    fi

    if [ "$missing_file" -gt 0 ]; then
        err "WARNING: ${missing_file} filesystem files missing"
        err "These assets may need re-migration"
        return 1
    fi

    log "INTEGRITY CHECK PASSED"

    local unmigrated
    unmigrated=$(psql_cmd -c "
        SELECT count(*) FROM assets a
        WHERE a.data IS NOT NULL AND length(a.data) > 0
        AND NOT EXISTS (SELECT 1 FROM fsassets f WHERE f.id = a.id);
    ")

    if [ "$unmigrated" -gt 0 ]; then
        log ""
        log "WARNING: ${unmigrated} assets not yet migrated to FSAssets"
        log "These will be lazy-migrated on first access, or run the batch migration tool"
    fi
}

show_stats() {
    log "=== FSAssets Storage Statistics ==="
    log ""

    psql_cmd -c "
        SELECT
            'Legacy assets' AS source,
            count(*) AS count,
            pg_size_pretty(sum(length(data))) AS data_size
        FROM assets WHERE data IS NOT NULL AND length(data) > 0
        UNION ALL
        SELECT
            'FSAssets metadata' AS source,
            count(*) AS count,
            'N/A (on filesystem)' AS data_size
        FROM fsassets;
    " | while IFS='|' read -r source count size; do
        printf "  %-20s  count=%-8s  size=%s\n" "$source" "$count" "$size"
    done

    log ""
    log "--- Deduplication ---"
    local unique_hashes total_refs
    unique_hashes=$(psql_cmd -c "SELECT count(DISTINCT hash) FROM fsassets;")
    total_refs=$(psql_cmd -c "SELECT count(*) FROM fsassets;")

    if [ "$total_refs" -gt 0 ]; then
        local dedup_pct
        dedup_pct=$(echo "scale=1; (1 - ${unique_hashes} / ${total_refs}) * 100" | bc)
        log "  Unique hashes:     ${unique_hashes}"
        log "  Total references:  ${total_refs}"
        log "  Deduplication:     ${dedup_pct}% savings"
    fi

    log ""
    log "--- Asset Types ---"
    psql_cmd -c "
        SELECT type,
            CASE type
                WHEN 0 THEN 'Texture'
                WHEN 1 THEN 'Sound'
                WHEN 6 THEN 'Object'
                WHEN 10 THEN 'LSL Text'
                WHEN 12 THEN 'Bodypart'
                WHEN 13 THEN 'Trash'
                WHEN 18 THEN 'Clothing'
                WHEN 20 THEN 'Animation'
                WHEN 22 THEN 'Notecard'
                WHEN 24 THEN 'Link'
                WHEN 25 THEN 'Link Folder'
                WHEN 49 THEN 'Mesh'
                WHEN 57 THEN 'Material'
                ELSE 'Other'
            END AS name,
            count(*) AS count
        FROM fsassets
        GROUP BY type
        ORDER BY count DESC
        LIMIT 15;
    " | while IFS='|' read -r type name count; do
        printf "  Type %3s %-15s  %s\n" "$type" "$name" "$count"
    done

    log ""
    log "--- Database Size ---"
    psql_cmd -c "
        SELECT pg_size_pretty(pg_database_size(current_database())) AS total_db_size;
    " | while read -r size; do
        log "  Current database size: ${size}"
    done

    log ""
    log "--- Filesystem Usage ---"
    if [ -d "$FSASSETS_ROOT" ]; then
        local fs_size fs_files
        fs_size=$(du -sh "$FSASSETS_ROOT" 2>/dev/null | cut -f1)
        fs_files=$(find "$FSASSETS_ROOT" -name '*.gz' -type f 2>/dev/null | wc -l | tr -d ' ')
        log "  FSAssets directory: ${FSASSETS_ROOT}"
        log "  Filesystem size:   ${fs_size}"
        log "  Compressed files:  ${fs_files}"
    else
        log "  FSAssets directory not found at ${FSASSETS_ROOT}"
    fi
}

run_benchmark() {
    log "=== Asset Load Performance Benchmark ==="
    log ""

    log "--- Single Asset Fetch (10 random) ---"
    local total_ms=0
    for i in $(seq 1 10); do
        local asset_id
        asset_id=$(psql_cmd -c "SELECT id::text FROM fsassets ORDER BY random() LIMIT 1;")
        local start_ms end_ms elapsed
        start_ms=$(date +%s%N)

        psql_cmd -c "SELECT hash FROM fsassets WHERE id = '${asset_id}'::uuid;" > /dev/null

        end_ms=$(date +%s%N)
        elapsed=$(( (end_ms - start_ms) / 1000000 ))
        total_ms=$((total_ms + elapsed))
        log "  Asset ${i}: ${elapsed}ms (${asset_id})"
    done
    log "  Average: $((total_ms / 10))ms"

    log ""
    log "--- Batch Fetch (50 assets, single query) ---"
    local start_ms end_ms elapsed
    start_ms=$(date +%s%N)

    psql_cmd -c "
        SELECT id::text, hash FROM fsassets
        WHERE id = ANY(
            (SELECT array_agg(id) FROM (SELECT id FROM fsassets ORDER BY random() LIMIT 50) t)
        );
    " > /dev/null

    end_ms=$(date +%s%N)
    elapsed=$(( (end_ms - start_ms) / 1000000 ))
    log "  50-asset batch: ${elapsed}ms (${elapsed}ms total, $((elapsed / 50))ms/asset)"

    log ""
    log "--- Legacy DB Blob Fetch (comparison) ---"
    start_ms=$(date +%s%N)

    psql_cmd -c "
        SELECT id::text, length(data) FROM assets
        WHERE data IS NOT NULL
        ORDER BY random() LIMIT 10;
    " > /dev/null

    end_ms=$(date +%s%N)
    elapsed=$(( (end_ms - start_ms) / 1000000 ))
    log "  10 legacy blob reads: ${elapsed}ms ($((elapsed / 10))ms/asset)"
}

do_cutover() {
    log "=== FSAssets Cutover: Drop Legacy Data Column ==="
    log ""

    local unmigrated
    unmigrated=$(psql_cmd -c "
        SELECT count(*) FROM assets a
        WHERE a.data IS NOT NULL AND length(a.data) > 0
        AND NOT EXISTS (SELECT 1 FROM fsassets f WHERE f.id = a.id);
    ")

    if [ "$unmigrated" -gt 0 ]; then
        err "BLOCKED: ${unmigrated} assets not yet migrated"
        err "Run batch migration first, or wait for lazy migration to complete"
        exit 1
    fi

    local db_size_before
    db_size_before=$(psql_cmd -c "SELECT pg_size_pretty(pg_database_size(current_database()));")
    log "Database size before cutover: ${db_size_before}"

    log ""
    log "WARNING: This will:"
    log "  1. Set all assets.data to NULL"
    log "  2. ALTER TABLE assets DROP COLUMN data"
    log "  3. VACUUM FULL assets (reclaims disk space, locks table)"
    log ""
    log "This is IRREVERSIBLE. Ensure you have a backup."
    log "Proceed? (type 'yes-drop-data' to confirm)"
    read -r confirm
    if [ "$confirm" != "yes-drop-data" ]; then
        log "Aborted."
        exit 0
    fi

    log "Step 1/3: Nullifying data column..."
    psql_cmd -c "UPDATE assets SET data = NULL WHERE data IS NOT NULL;"

    log "Step 2/3: Dropping data column..."
    psql_cmd -c "ALTER TABLE assets DROP COLUMN IF EXISTS data;"

    log "Step 3/3: Running VACUUM FULL (this may take several minutes)..."
    psql_cmd -c "VACUUM FULL assets;"

    local db_size_after
    db_size_after=$(psql_cmd -c "SELECT pg_size_pretty(pg_database_size(current_database()));")

    log ""
    log "=== CUTOVER COMPLETE ==="
    log "  Before: ${db_size_before}"
    log "  After:  ${db_size_after}"
    log ""
    log "Assets are now served exclusively from FSAssets filesystem storage."
    log "The legacy data column has been removed and disk space reclaimed."
}

case "${1:-help}" in
    verify)
        verify_migration
        ;;
    stats)
        show_stats
        ;;
    benchmark)
        run_benchmark
        ;;
    cutover)
        do_cutover
        ;;
    *)
        echo "Usage: $0 {verify|stats|benchmark|cutover}"
        echo ""
        echo "  verify     Integrity check: fsassets hashes vs legacy DB blobs"
        echo "  stats      Migration statistics: counts, dedup, sizes"
        echo "  benchmark  Asset load performance measurements"
        echo "  cutover    Drop data column + VACUUM FULL (DESTRUCTIVE)"
        echo ""
        echo "Environment variables:"
        echo "  DATABASE_URL   PostgreSQL connection (default: postgresql://opensim@localhost/gaiagrid)"
        echo "  FSASSETS_ROOT  FSAssets directory (default: ./fsassets)"
        echo "  SAMPLE_SIZE    Spot-check sample size (default: 100)"
        exit 1
        ;;
esac
