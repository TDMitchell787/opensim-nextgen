#!/bin/bash

# Quick Server Status Check
echo "🔍 OpenSim Next Server Status Check"
echo "======================================"

# Check process
if pgrep -f "opensim-next" > /dev/null; then
    echo "✅ Process: RUNNING"
    PID=$(pgrep -f "opensim-next")
    echo "   PID: $PID"
else
    echo "❌ Process: NOT RUNNING"
    echo ""
    echo "🔄 To start: ./start_with_monitoring.sh"
    exit 1
fi

# Check HTTP port 9000
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:9000/ 2>/dev/null || echo "000")
if [ "$HTTP_STATUS" = "200" ] || [ "$HTTP_STATUS" = "404" ]; then
    echo "✅ HTTP (9000): RESPONDING ($HTTP_STATUS)"
else
    echo "❌ HTTP (9000): NOT RESPONDING ($HTTP_STATUS)"
fi

# Check UDP port 9001
if netstat -an | grep ":9001 " | grep -q LISTEN; then
    echo "✅ UDP (9001): LISTENING"
else
    echo "❌ UDP (9001): NOT LISTENING"
fi

# Check latest logs
LATEST_LOG=$(ls -t logs/current/server_*.log 2>/dev/null | head -1)
if [ -n "$LATEST_LOG" ]; then
    echo "📝 Latest log: $LATEST_LOG"
    echo "📊 Last 3 log entries:"
    tail -3 "$LATEST_LOG" | sed 's/^/   /'
else
    echo "❌ No log files found"
fi

echo ""
echo "💡 Use './monitor_server.sh &' for continuous monitoring"