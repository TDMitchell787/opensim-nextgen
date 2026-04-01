#!/bin/bash

# Get timestamp for unique log files
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_DIR="logs/current"
SERVER_LOG="$LOG_DIR/server_$TIMESTAMP.log"
TEST_LOG="$LOG_DIR/test_$TIMESTAMP.log"

# Ensure log directory exists
mkdir -p "$LOG_DIR"

echo "🔄 MANDATORY CLEANUP - Killing all opensim-next processes..." | tee "$TEST_LOG"
pkill -9 -f "opensim-next" || true
sleep 2

echo "🧹 CLEANING OLD CURRENT LOGS..." | tee -a "$TEST_LOG"
rm -f "$LOG_DIR"/server_*.log "$LOG_DIR"/test_*.log "$LOG_DIR"/login_*.log "$LOG_DIR"/debug_*.log

echo "🔨 MANDATORY RECOMPILE - Building with latest code changes..." | tee -a "$TEST_LOG"
echo "   Please wait - this may take 1-2 minutes..." | tee -a "$TEST_LOG"
if ! cargo build --release --bin opensim-next 2>&1 | tee -a "$TEST_LOG"; then
    echo "❌ COMPILATION FAILED - Cannot proceed with testing" | tee -a "$TEST_LOG"
    echo "📋 Check compilation errors above" | tee -a "$TEST_LOG"
    exit 1
fi
echo "✅ COMPILATION SUCCESSFUL" | tee -a "$TEST_LOG"

echo "🚀 STARTING FRESH SERVER..." | tee -a "$TEST_LOG"
RUST_LOG=info cargo run --release --bin opensim-next > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID" | tee -a "$TEST_LOG"

echo "⏰ WAITING FOR SERVER STARTUP (15 seconds)..." | tee -a "$TEST_LOG"
echo "   Please wait - server is initializing..." | tee -a "$TEST_LOG"
sleep 15

echo "✅ SERVER READY - PID: $SERVER_PID" | tee -a "$TEST_LOG"
echo "" | tee -a "$TEST_LOG"

# EADS-Enhanced Login Prompt with Clear Notifications
echo "🚨 ====================================== 🚨" | tee -a "$TEST_LOG"
echo "🎯 READY FOR LOGIN TEST!" | tee -a "$TEST_LOG"
echo "🚨 ====================================== 🚨" | tee -a "$TEST_LOG"
echo "" | tee -a "$TEST_LOG"
echo "📢 NOTIFICATION: Please attempt Second Life viewer login NOW" | tee -a "$TEST_LOG"
echo "   >>> SECOND LIFE VIEWER LOGIN REQUIRED <<<" | tee -a "$TEST_LOG"
echo "" | tee -a "$TEST_LOG"
echo "📋 LOGIN TEST PROTOCOL:" | tee -a "$TEST_LOG"
echo "   1. 🔑 Open Second Life viewer" | tee -a "$TEST_LOG"
echo "   2. 🌐 Set login URI: http://127.0.0.1:9000/" | tee -a "$TEST_LOG"
echo "   3. 👤 Username: Admin" | tee -a "$TEST_LOG"
echo "   4. 👤 Surname: User" | tee -a "$TEST_LOG"
echo "   5. 🔐 Password: password123" | tee -a "$TEST_LOG"
echo "   6. 🎯 Click LOGIN and observe behavior" | tee -a "$TEST_LOG"
echo "" | tee -a "$TEST_LOG"
echo "📊 REPORT OPTIONS:" | tee -a "$TEST_LOG"
echo "   ✅ SUCCESS: Login completed successfully" | tee -a "$TEST_LOG"
echo "   ❌ CRASH: Viewer crashed during login" | tee -a "$TEST_LOG"
echo "   ⚠️  QUIET FAILURE: Progress bar then quiet failure" | tee -a "$TEST_LOG"
echo "   🔄 STUCK: Progress bar stuck at certain point" | tee -a "$TEST_LOG"
echo "" | tee -a "$TEST_LOG"
echo "📁 LOG FILES:" | tee -a "$TEST_LOG"
echo "   Server Log: $SERVER_LOG" | tee -a "$TEST_LOG"
echo "   Test Log: $TEST_LOG" | tee -a "$TEST_LOG"
echo "" | tee -a "$TEST_LOG"
echo "🔍 AFTER LOGIN ATTEMPT:" | tee -a "$TEST_LOG"
echo "   Run: ./check_logs_clean.sh (includes EADS analysis)" | tee -a "$TEST_LOG"
echo "" | tee -a "$TEST_LOG"
echo "🛑 TO STOP SERVER: pkill -f opensim-next" | tee -a "$TEST_LOG"
echo "📊 CURRENT LOGS: ls -lt logs/current/" | tee -a "$TEST_LOG"

# Add audio notification (if available)
if command -v say >/dev/null 2>&1; then
    echo "🔊 Playing audio notification..." | tee -a "$TEST_LOG"
    say "OpenSim server ready for Second Life viewer login test" &
fi

# Add visual notification with countdown
echo "" | tee -a "$TEST_LOG"
echo "⏰ WAITING FOR LOGIN ATTEMPT..." | tee -a "$TEST_LOG"
echo "   (Script will continue monitoring)" | tee -a "$TEST_LOG"
echo "   Press Ctrl+C to stop server manually" | tee -a "$TEST_LOG"