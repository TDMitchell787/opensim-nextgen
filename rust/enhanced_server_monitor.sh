#!/bin/bash

# Enhanced OpenSim Next Server Monitor with Crash Detection
# Provides detailed crash analysis and stability monitoring

SERVER_NAME="opensim-next"
LOG_DIR="logs/current"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
MONITOR_LOG="$LOG_DIR/enhanced_monitor_$TIMESTAMP.log"
CRASH_LOG="$LOG_DIR/crash_analysis_$TIMESTAMP.log"

# Create log directory if it doesn't exist
mkdir -p "$LOG_DIR"

echo "🔍 Enhanced OpenSim Next Server Monitor Started - $(date)" | tee -a "$MONITOR_LOG"
echo "📊 Monitor Log: $MONITOR_LOG" | tee -a "$MONITOR_LOG"
echo "💥 Crash Log: $CRASH_LOG" | tee -a "$MONITOR_LOG"
echo "" | tee -a "$MONITOR_LOG"

# Track server state
SERVER_WAS_RUNNING=false
STARTUP_TIME=""
LAST_SEEN_RUNNING=""
CRASH_COUNT=0

while true; do
    CURRENT_TIME=$(date '+%H:%M:%S')
    
    # Check if server process is running
    if pgrep -f "$SERVER_NAME" > /dev/null; then
        # Server is running
        if [ "$SERVER_WAS_RUNNING" = false ]; then
            # Server just started
            STARTUP_TIME="$CURRENT_TIME"
            echo "🚀 $CURRENT_TIME - SERVER STARTED!" | tee -a "$MONITOR_LOG"
            SERVER_WAS_RUNNING=true
        fi
        
        LAST_SEEN_RUNNING="$CURRENT_TIME"
        
        # Check if ports are accessible
        HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:9000/ 2>/dev/null || echo "000")
        UDP_STATUS=$(netstat -an 2>/dev/null | grep ":9001 " | grep LISTEN > /dev/null && echo "LISTENING" || echo "NOT_LISTENING")
        
        if [ "$HTTP_STATUS" = "200" ] || [ "$HTTP_STATUS" = "404" ] || [ "$HTTP_STATUS" = "405" ]; then
            UPTIME_SECONDS=$(( $(date +%s) - $(date -j -f "%H:%M:%S" "$STARTUP_TIME" +%s 2>/dev/null || echo 0) ))
            echo "✅ $CURRENT_TIME - Server STABLE - Uptime: ${UPTIME_SECONDS}s HTTP:$HTTP_STATUS UDP:$UDP_STATUS" | tee -a "$MONITOR_LOG"
        else
            echo "⚠️  $CURRENT_TIME - Server process running but HTTP not responding ($HTTP_STATUS)" | tee -a "$MONITOR_LOG"
        fi
    else
        # Server is not running
        if [ "$SERVER_WAS_RUNNING" = true ]; then
            # Server just crashed!
            CRASH_COUNT=$((CRASH_COUNT + 1))
            CRASH_TIME="$CURRENT_TIME"
            UPTIME_SECONDS=$(( $(date +%s) - $(date -j -f "%H:%M:%S" "$STARTUP_TIME" +%s 2>/dev/null || echo 0) ))
            
            echo "🚨 $CRASH_TIME - SERVER CRASHED! (Crash #$CRASH_COUNT)" | tee -a "$MONITOR_LOG" "$CRASH_LOG"
            echo "💔 Started: $STARTUP_TIME, Last seen: $LAST_SEEN_RUNNING, Uptime: ${UPTIME_SECONDS}s" | tee -a "$MONITOR_LOG" "$CRASH_LOG"
            
            # Analyze crash
            echo "🔍 CRASH ANALYSIS #$CRASH_COUNT - $CRASH_TIME" >> "$CRASH_LOG"
            echo "================================================" >> "$CRASH_LOG"
            echo "Startup Time: $STARTUP_TIME" >> "$CRASH_LOG"
            echo "Last Seen Running: $LAST_SEEN_RUNNING" >> "$CRASH_LOG"
            echo "Uptime: ${UPTIME_SECONDS} seconds" >> "$CRASH_LOG"
            
            # Check for recent log entries
            LATEST_LOG=$(ls -t logs/current/server_*.log 2>/dev/null | head -1)
            if [ -n "$LATEST_LOG" ]; then
                echo "Last 10 log entries:" >> "$CRASH_LOG"
                tail -10 "$LATEST_LOG" >> "$CRASH_LOG"
            fi
            
            # Check system resources
            echo "System Resources at crash:" >> "$CRASH_LOG"
            echo "Memory: $(free -h 2>/dev/null || vm_stat)" >> "$CRASH_LOG" 2>/dev/null
            echo "Disk: $(df -h . 2>/dev/null)" >> "$CRASH_LOG" 2>/dev/null
            echo "" >> "$CRASH_LOG"
            
            SERVER_WAS_RUNNING=false
            
            echo "🚨 CRASH DETECTED - TESTING INVALID!" 
            echo "📋 Crash analysis saved to: $CRASH_LOG"
            echo ""
        else
            echo "💤 $CURRENT_TIME - Server not running (Crashes: $CRASH_COUNT)" | tee -a "$MONITOR_LOG"
        fi
    fi
    
    sleep 3
done