#!/bin/bash

# Stable Server Startup with Enhanced Monitoring
# Phase 1: Server Stability Focus

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_DIR="logs/current"
SERVER_LOG="$LOG_DIR/stable_server_$TIMESTAMP.log"

# Create log directory
mkdir -p "$LOG_DIR"

echo "🚀 PHASE 1: STABLE SERVER STARTUP"
echo "=================================="
echo "📝 Server Log: $SERVER_LOG"
echo "🔍 Enhanced monitoring enabled"
echo ""

# Clean any existing processes
echo "🧹 Cleaning existing processes..."
pkill -f opensim-next 2>/dev/null || true
pkill -f monitor_server 2>/dev/null || true
sleep 2

# Start enhanced monitoring in background
echo "🔍 Starting enhanced monitoring..."
./enhanced_server_monitor.sh &
MONITOR_PID=$!

# Wait a moment for monitor to initialize
sleep 2

echo "⏳ Starting server with enhanced crash detection..."
echo "🎯 Goal: Achieve >5 minutes stable operation"
echo ""

# Start server with enhanced logging
RUST_LOG=info,opensim_next=debug cargo run 2>&1 | tee "$SERVER_LOG" &
SERVER_PID=$!

echo "✅ Startup complete!"
echo "📊 Server PID: $SERVER_PID"
echo "🔍 Monitor PID: $MONITOR_PID"
echo ""
echo "💡 Commands:"
echo "   Stop all: kill $SERVER_PID $MONITOR_PID"
echo "   Check status: ./check_server.sh"
echo "   View logs: tail -f $SERVER_LOG"
echo ""
echo "🎯 PHASE 1 OBJECTIVE: Server must run stably for >5 minutes"
echo "⚠️  DO NOT test login until server proven stable!"
echo ""

# Keep script running and monitoring
echo "📊 Monitoring server stability..."
wait $SERVER_PID