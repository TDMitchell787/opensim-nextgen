#!/bin/bash
# pgpool-monitor.sh — pgpool-II monitoring script
# Phase 199.5 — Gaia Grid
#
# Usage: ./pgpool-monitor.sh [--json] [--watch] [--alert]

PCP_HOST="localhost"
PCP_PORT="9898"
PCP_USER="opensim"
PGPOOL_PORT="9999"

JSON_MODE=false
WATCH_MODE=false
ALERT_MODE=false

for arg in "$@"; do
    case "$arg" in
        --json) JSON_MODE=true ;;
        --watch) WATCH_MODE=true ;;
        --alert) ALERT_MODE=true ;;
    esac
done

ALERT_COUNT=0

alert() {
    local level="$1"
    local msg="$2"
    ALERT_COUNT=$((ALERT_COUNT + 1))
    if $ALERT_MODE; then
        logger -t pgpool-monitor "[${level}] ${msg}"
    fi
    echo "  *** ${level}: ${msg}"
}

check_backends() {
    echo "=== Backend Node Status ==="
    for i in 0 1; do
        result=$(pcp_node_info -h "$PCP_HOST" -p "$PCP_PORT" -U "$PCP_USER" -w "$i" 2>/dev/null)
        if [ $? -eq 0 ] && [ -n "$result" ]; then
            host=$(echo "$result" | awk '{print $1}')
            port=$(echo "$result" | awk '{print $2}')
            status_num=$(echo "$result" | awk '{print $3}')
            weight=$(echo "$result" | awk '{print $4}')
            status_name=$(echo "$result" | awk '{print $5}')
            role_name=$(echo "$result" | awk '{print $7}')

            case "$status_num" in
                1) status_text="INITIALIZING" ;;
                2) status_text="UP" ;;
                3) status_text="DOWN"; alert "CRITICAL" "Node $i (${host}:${port}) is DOWN" ;;
                *) status_text="${status_name}(${status_num})" ;;
            esac

            echo "  Node $i: ${host}:${port} | Status: ${status_text} | Role: ${role_name} | Weight: ${weight}"
        else
            echo "  Node $i: not configured"
        fi
    done
    echo ""
}

check_pool_status() {
    echo "=== Connection Pool Configuration ==="
    result=$(psql -h localhost -p "$PGPOOL_PORT" -U opensim -d gaiagrid -Atc "PGPOOL SHOW num_init_children;" 2>/dev/null)
    max_pool=$(psql -h localhost -p "$PGPOOL_PORT" -U opensim -d gaiagrid -Atc "PGPOOL SHOW max_pool;" 2>/dev/null)
    child_life=$(psql -h localhost -p "$PGPOOL_PORT" -U opensim -d gaiagrid -Atc "PGPOOL SHOW child_life_time;" 2>/dev/null)
    conn_life=$(psql -h localhost -p "$PGPOOL_PORT" -U opensim -d gaiagrid -Atc "PGPOOL SHOW connection_life_time;" 2>/dev/null)

    if [ -n "$result" ]; then
        echo "  num_init_children: ${result}"
        echo "  max_pool: ${max_pool}"
        echo "  child_life_time: ${child_life}"
        echo "  connection_life_time: ${conn_life}"
    else
        echo "  Unable to retrieve pool config (pgpool may not be running)"
    fi
    echo ""
}

check_process_info() {
    echo "=== Active Process Info ==="
    pids=$(pcp_proc_count -h "$PCP_HOST" -p "$PCP_PORT" -U "$PCP_USER" -w 2>/dev/null)
    if [ $? -eq 0 ] && [ -n "$pids" ]; then
        count=$(echo "$pids" | wc -w | tr -d ' ')
        echo "  Child processes: ${count}"

        active=0
        idle=0
        for pid in $pids; do
            info=$(pcp_proc_info -h "$PCP_HOST" -p "$PCP_PORT" -U "$PCP_USER" -w "$pid" 2>/dev/null)
            if [ -n "$info" ] && echo "$info" | grep -q "gaiagrid"; then
                active=$((active + 1))
            else
                idle=$((idle + 1))
            fi
        done
        echo "  Active (with DB connection): ${active}"
        echo "  Idle: ${idle}"

        total=$(psql -h localhost -p "$PGPOOL_PORT" -U opensim -d gaiagrid -Atc "PGPOOL SHOW num_init_children;" 2>/dev/null)
        if [ -n "$total" ] && [ "$total" -gt 0 ]; then
            utilization=$((active * 100 / total))
            echo "  Pool utilization: ${utilization}%"
            if [ "$utilization" -gt 80 ]; then
                alert "WARNING" "Pool utilization at ${utilization}% — consider increasing num_init_children"
            fi
        fi
    else
        echo "  Unable to retrieve process info"
    fi
    echo ""
}

check_query_routing() {
    echo "=== Query Routing Statistics ==="
    nodes=$(psql -h localhost -p "$PGPOOL_PORT" -U opensim -d gaiagrid -Atc "SHOW pool_nodes;" 2>/dev/null)
    if [ -n "$nodes" ]; then
        echo "$nodes" | while IFS='|' read -r nid host port status pg_status lb_weight role pg_role select_cnt lb_node repl_delay rest; do
            nid=$(echo "$nid" | tr -d ' ')
            select_cnt=$(echo "$select_cnt" | tr -d ' ')
            role=$(echo "$role" | tr -d ' ')
            status=$(echo "$status" | tr -d ' ')
            echo "  Node ${nid} (${role}): ${select_cnt} SELECTs | Status: ${status}"
        done
    else
        echo "  Unable to retrieve routing stats"
    fi
    echo ""
}

check_replication_lag() {
    echo "=== Replication Status ==="
    lag=$(psql -h localhost -p "$PGPOOL_PORT" -U opensim -d gaiagrid -Atc \
        "SELECT CASE WHEN pg_is_in_recovery() THEN
            extract(epoch from now() - pg_last_xact_replay_timestamp())::int
         ELSE 0 END;" 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "  Replication lag: ${lag:-0}s"
        if [ "${lag:-0}" -gt 30 ]; then
            alert "WARNING" "Replication lag is ${lag}s (threshold: 30s)"
        fi
    else
        echo "  Unable to check replication (pgpool may not be running)"
    fi

    standby_lag=$(psql -h localhost -p 5433 -U opensim -d gaiagrid -Atc \
        "SELECT CASE WHEN pg_is_in_recovery() THEN
            extract(epoch from now() - pg_last_xact_replay_timestamp())::int
         ELSE -1 END;" 2>/dev/null)
    if [ $? -eq 0 ] && [ "${standby_lag}" != "-1" ]; then
        echo "  Standby (direct): ${standby_lag:-N/A}s behind primary"
    else
        echo "  Standby (port 5433): not available"
    fi
    echo ""
}

check_connection_test() {
    echo "=== Connection Test ==="
    result=$(psql -h localhost -p "$PGPOOL_PORT" -U opensim -d gaiagrid -Atc "SELECT 'pgpool_ok';" 2>/dev/null)
    if [ "$result" = "pgpool_ok" ]; then
        echo "  pgpool connection: OK"
    else
        echo "  pgpool connection: FAILED"
        alert "CRITICAL" "Cannot connect to pgpool on port ${PGPOOL_PORT}"
    fi

    direct=$(psql -h localhost -p 5432 -U opensim -d gaiagrid -Atc "SELECT 'direct_ok';" 2>/dev/null)
    if [ "$direct" = "direct_ok" ]; then
        echo "  Direct PG (5432): OK"
    else
        echo "  Direct PG (5432): FAILED"
        alert "WARNING" "Direct PostgreSQL connection failed on port 5432"
    fi

    standby=$(psql -h localhost -p 5433 -U opensim -d gaiagrid -Atc "SELECT 'standby_ok';" 2>/dev/null)
    if [ "$standby" = "standby_ok" ]; then
        echo "  Standby PG (5433): OK"
    else
        echo "  Standby PG (5433): not available"
    fi
    echo ""
}

check_failover_log() {
    echo "=== Recent Failover Events ==="
    if [ -f /usr/local/var/pgpool_logs/failover.log ]; then
        tail -10 /usr/local/var/pgpool_logs/failover.log
    else
        echo "  No failover events recorded"
    fi
    echo ""
}

run_checks() {
    ALERT_COUNT=0
    echo "=========================================="
    echo "  pgpool-II Monitor — $(date '+%Y-%m-%d %H:%M:%S')"
    echo "=========================================="
    echo ""
    check_connection_test
    check_backends
    check_query_routing
    check_pool_status
    check_process_info
    check_replication_lag
    check_failover_log

    if [ "$ALERT_COUNT" -gt 0 ]; then
        echo "=========================================="
        echo "  ALERTS: ${ALERT_COUNT} issue(s) detected"
        echo "=========================================="
    else
        echo "  All systems healthy."
    fi
}

if $JSON_MODE; then
    pgpool_ok=$(psql -h localhost -p "$PGPOOL_PORT" -U opensim -d gaiagrid -Atc "SELECT 'ok';" 2>/dev/null)
    node0=$(pcp_node_info -h "$PCP_HOST" -p "$PCP_PORT" -U "$PCP_USER" -w 0 2>/dev/null | awk '{printf "{\"host\":\"%s\",\"port\":%s,\"status\":\"%s\",\"role\":\"%s\"}", $1, $2, $5, $7}')
    node1=$(pcp_node_info -h "$PCP_HOST" -p "$PCP_PORT" -U "$PCP_USER" -w 1 2>/dev/null | awk '{printf "{\"host\":\"%s\",\"port\":%s,\"status\":\"%s\",\"role\":\"%s\"}", $1, $2, $5, $7}')
    echo "{\"pgpool\":\"${pgpool_ok:-down}\",\"node0\":${node0:-null},\"node1\":${node1:-null},\"timestamp\":\"$(date -u '+%Y-%m-%dT%H:%M:%SZ')\"}"
elif $WATCH_MODE; then
    while true; do
        clear
        run_checks
        sleep 10
    done
else
    run_checks
fi
