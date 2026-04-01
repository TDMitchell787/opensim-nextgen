#!/bin/bash

# OpenSim Next Startup with Monitoring
# Starts server with timestamped logs and monitoring

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_DIR="logs/current"
SERVER_LOG="$LOG_DIR/server_$TIMESTAMP.log"

# Create log directory
mkdir -p "$LOG_DIR"

echo "🚀 Starting OpenSim Next Server with Monitoring"
echo "📝 Server Log: $SERVER_LOG"
echo "🔍 Monitor will start in background"
echo ""

# Start server with timestamped log
echo "⏳ Starting server..."
RUST_LOG=info cargo run 2>&1 | tee "$SERVER_LOG" &
SERVER_PID=$!

# Wait a moment for server to start
sleep 3

# Start monitor in background
echo "🔍 Starting monitor..."
./monitor_server.sh &
MONITOR_PID=$!

echo ""
echo "✅ Setup complete!"
echo "📊 Server PID: $SERVER_PID"
echo "🔍 Monitor PID: $MONITOR_PID"
echo ""
echo "💡 To stop everything:"
echo "   kill $SERVER_PID $MONITOR_PID"
echo ""
echo "📝 Watch logs: tail -f $SERVER_LOG"
echo "🔍 Monitor status will appear above"

# Keep script running
wait $SERVER_PID