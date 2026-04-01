#!/bin/bash
# pgpool-benchmark.sh — Phase 199.2 Connection Pooling Benchmark
# Compares direct PostgreSQL (port 5432) vs pgpool-II (port 9999)

DIRECT_PORT=5432
PGPOOL_PORT=9999
DB_USER="opensim"
DB_NAME="gaiagrid"
ITERATIONS=100

echo "=============================================="
echo "  pgpool-II Connection Pooling Benchmark"
echo "  Phase 199.2 — $(date '+%Y-%m-%d %H:%M:%S')"
echo "=============================================="
echo ""
echo "Config: ${ITERATIONS} iterations per test"
echo "Direct: localhost:${DIRECT_PORT}"
echo "pgpool: localhost:${PGPOOL_PORT}"
echo ""

benchmark_query() {
    local label="$1"
    local port="$2"
    local query="$3"
    local iters="$4"

    local start=$(python3 -c "import time; print(time.time())")
    for i in $(seq 1 $iters); do
        psql -h localhost -p "$port" -U "$DB_USER" -d "$DB_NAME" -Atc "$query" > /dev/null 2>&1
    done
    local end=$(python3 -c "import time; print(time.time())")
    local elapsed=$(python3 -c "print(f'{($end - $start):.3f}')")
    local per_query=$(python3 -c "print(f'{($end - $start) / $iters * 1000:.2f}')")
    echo "  ${label}: ${elapsed}s total, ${per_query}ms/query"
}

benchmark_connect() {
    local label="$1"
    local port="$2"
    local iters="$3"

    local start=$(python3 -c "import time; print(time.time())")
    for i in $(seq 1 $iters); do
        psql -h localhost -p "$port" -U "$DB_USER" -d "$DB_NAME" -Atc "SELECT 1;" > /dev/null 2>&1
    done
    local end=$(python3 -c "import time; print(time.time())")
    local elapsed=$(python3 -c "print(f'{($end - $start):.3f}')")
    local per_conn=$(python3 -c "print(f'{($end - $start) / $iters * 1000:.2f}')")
    echo "  ${label}: ${elapsed}s total, ${per_conn}ms/connection"
}

echo "--- Test 1: Connection Overhead (SELECT 1) ---"
benchmark_connect "Direct PG " $DIRECT_PORT $ITERATIONS
benchmark_connect "pgpool-II " $PGPOOL_PORT $ITERATIONS
echo ""

echo "--- Test 2: Simple Read (COUNT fsassets) ---"
benchmark_query "Direct PG " $DIRECT_PORT "SELECT COUNT(*) FROM fsassets;" $ITERATIONS
benchmark_query "pgpool-II " $PGPOOL_PORT "SELECT COUNT(*) FROM fsassets;" $ITERATIONS
echo ""

echo "--- Test 3: Indexed Lookup (single asset by hash) ---"
HASH=$(psql -h localhost -p $DIRECT_PORT -U $DB_USER -d $DB_NAME -Atc "SELECT hash FROM fsassets LIMIT 1;" 2>/dev/null)
benchmark_query "Direct PG " $DIRECT_PORT "SELECT id, type, hash FROM fsassets WHERE hash = '${HASH}' LIMIT 1;" $ITERATIONS
benchmark_query "pgpool-II " $PGPOOL_PORT "SELECT id, type, hash FROM fsassets WHERE hash = '${HASH}' LIMIT 1;" $ITERATIONS
echo ""

echo "--- Test 4: Region Query (typical OpenSim workload) ---"
benchmark_query "Direct PG " $DIRECT_PORT "SELECT \"regionName\", \"locX\", \"locY\" FROM regions LIMIT 10;" $ITERATIONS
benchmark_query "pgpool-II " $PGPOOL_PORT "SELECT \"regionName\", \"locX\", \"locY\" FROM regions LIMIT 10;" $ITERATIONS
echo ""

echo "--- Test 5: Write Operation (INSERT + DELETE) ---"
WRITE_ITERS=50
benchmark_write() {
    local label="$1"
    local port="$2"
    local iters="$3"

    local start=$(python3 -c "import time; print(time.time())")
    for i in $(seq 1 $iters); do
        local uuid=$(python3 -c "import uuid; print(uuid.uuid4())")
        psql -h localhost -p "$port" -U "$DB_USER" -d "$DB_NAME" -Atc \
            "INSERT INTO griduser (\"UserID\", \"HomeRegionID\", \"HomePosition\", \"HomeLookAt\", \"LastRegionID\", \"LastPosition\", \"LastLookAt\", \"Online\", \"Login\", \"Logout\") VALUES ('${uuid}', '00000000-0000-0000-0000-000000000000', '<0,0,0>', '<0,0,0>', '00000000-0000-0000-0000-000000000000', '<0,0,0>', '<0,0,0>', 'false', '0', '0'); DELETE FROM griduser WHERE \"UserID\" = '${uuid}';" > /dev/null 2>&1
    done
    local end=$(python3 -c "import time; print(time.time())")
    local elapsed=$(python3 -c "print(f'{($end - $start):.3f}')")
    local per_write=$(python3 -c "print(f'{($end - $start) / $iters * 1000:.2f}')")
    echo "  ${label}: ${elapsed}s total, ${per_write}ms/write-cycle"
}

echo "  (${WRITE_ITERS} iterations — INSERT + DELETE per cycle)"
benchmark_write "Direct PG " $DIRECT_PORT $WRITE_ITERS
benchmark_write "pgpool-II " $PGPOOL_PORT $WRITE_ITERS
echo ""

echo "--- Test 6: Concurrent Connections (10 parallel queries) ---"
benchmark_parallel() {
    local label="$1"
    local port="$2"
    local parallel=10
    local per_worker=10

    local start=$(python3 -c "import time; print(time.time())")
    for w in $(seq 1 $parallel); do
        (for i in $(seq 1 $per_worker); do
            psql -h localhost -p "$port" -U "$DB_USER" -d "$DB_NAME" -Atc "SELECT COUNT(*) FROM regions;" > /dev/null 2>&1
        done) &
    done
    wait
    local end=$(python3 -c "import time; print(time.time())")
    local elapsed=$(python3 -c "print(f'{($end - $start):.3f}')")
    local total=$((parallel * per_worker))
    local per_query=$(python3 -c "print(f'{($end - $start) / $total * 1000:.2f}')")
    echo "  ${label}: ${elapsed}s total (${total} queries), ${per_query}ms/query effective"
}

benchmark_parallel "Direct PG " $DIRECT_PORT
benchmark_parallel "pgpool-II " $PGPOOL_PORT
echo ""

echo "=============================================="
echo "  Benchmark Complete"
echo "=============================================="
