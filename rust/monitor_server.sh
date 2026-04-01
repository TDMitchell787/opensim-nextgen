#!/bin/bash

# OpenSim Next Server Monitor
# Checks if server is running and alerts if it drops

SERVER_NAME="opensim-next"
LOG_DIR="logs/current"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
MONITOR_LOG="$LOG_DIR/monitor_$TIMESTAMP.log"

# Create log directory if it doesn't exist
mkdir -p "$LOG_DIR"

echo "🔍 OpenSim Next Server Monitor Started - $(date)" | tee -a "$MONITOR_LOG"
echo "📊 Monitor Log: $MONITOR_LOG" | tee -a "$MONITOR_LOG"
echo "" | tee -a "$MONITOR_LOG"

while true; do
    # Check if server process is running
    if pgrep -f "$SERVER_NAME" > /dev/null; then
        # Server is running - check if ports are accessible
        HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:9000/ 2>/dev/null || echo "000")
        UDP_STATUS=$(netstat -an | grep ":9001 " | grep LISTEN > /dev/null && echo "LISTENING" || echo "NOT_LISTENING")
        
        if [ "$HTTP_STATUS" = "200" ] || [ "$HTTP_STATUS" = "404" ]; then
            echo "✅ $(date '+%H:%M:%S') - Server RUNNING - HTTP:$HTTP_STATUS UDP:$UDP_STATUS" | tee -a "$MONITOR_LOG"
        else
            echo "⚠️  $(date '+%H:%M:%S') - Server process running but HTTP not responding ($HTTP_STATUS)" | tee -a "$MONITOR_LOG"
        fi
    else
        echo "🚨 $(date '+%H:%M:%S') - SERVER DOWN! Process not found!" | tee -a "$MONITOR_LOG"
        echo "🚨 SERVER CRASHED OR STOPPED - TESTING INVALID!" 
        echo "🔄 Use 'cargo run' to restart the server"
        echo ""
    fi
    
    sleep 5
done