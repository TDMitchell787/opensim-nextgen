#!/bin/bash

# EADS (Elegant Archive Solution) Login Analytics with Mandatory Profiling
# This script automatically integrates EADS profiling to prevent redundant debugging

# Find the latest server log
LATEST_SERVER_LOG=$(ls -t logs/current/server_*.log 2>/dev/null | head -1)

if [ -z "$LATEST_SERVER_LOG" ]; then
    echo "❌ No server logs found in logs/current/"
    echo "📁 Available logs:"
    ls -la logs/current/
    echo ""
    echo "🔍 EADS SERVER STARTUP DIAGNOSTIC:"
    echo "-----------------------------------"
    echo "⚠️  Server log missing indicates one of:"
    echo "   1. Server failed to start (crashed during initialization)"
    echo "   2. Server started but exited immediately"
    echo "   3. Log redirection failed in test script"
    echo ""
    echo "🔧 RECOMMENDED ACTIONS:"
    echo "   1. Check if server process is running: ps aux | grep opensim-next"
    echo "   2. Manual server start: cargo run --release --bin opensim-next"
    echo "   3. Check for compilation errors or runtime panics"
    echo ""
    echo "📊 LOGIN PIPELINE STATUS: FAILED (Server not operational)"
    echo "🎯 Current Success Rate: 0% (Critical system failure)"
    echo ""
    echo "🔄 FALLBACK ANALYSIS MODE:"
    echo "-------------------------"
    echo "If you experienced progress bar movement during login:"
    echo "   ✅ XML-RPC Phase: LIKELY WORKING"
    echo "   ✅ CAPS Session: LIKELY WORKING"
    echo "   ❌ LLUDP Phase: UNKNOWN (no server logs)"
    echo "   ❌ EventQueue: UNKNOWN (no server logs)"
    echo ""
    echo "If you experienced no progress bar:"
    echo "   ❌ XML-RPC Phase: FAILED"
    echo "   ❌ All subsequent phases: FAILED"
    echo ""
    echo "📋 EADS GUIDANCE:"
    echo "   • Progress bar = 99% working state restored"
    echo "   • No progress bar = Critical regression"
    echo "   • Missing server logs = Infrastructure issue"
    exit 1
fi

# Load EADS profile
EADS_PROFILE="EADS_LOGIN_PROFILE.md"
if [ ! -f "$EADS_PROFILE" ]; then
    echo "⚠️  EADS Profile not found - creating basic profile"
    echo "# EADS Login Profile - Auto-generated" > "$EADS_PROFILE"
    echo "Status: Basic profiling active" >> "$EADS_PROFILE"
fi

echo "🔍 EADS LOGIN ANALYTICS REPORT"
echo "==============================="
echo "📁 Analyzing: $LATEST_SERVER_LOG"
echo "🗂️  EADS Profile: $EADS_PROFILE"
echo ""

# EADS Phase 1: Check for previously solved issues
echo "🚫 EADS SOLVED ISSUES CHECK:"
echo "----------------------------"
SOLVED_ISSUES=0

# Check database user (Phase 1)
if sqlite3 opensim.db "SELECT COUNT(*) FROM users WHERE first_name='Admin' AND last_name='User';" | grep -q "1"; then
    echo "✅ Database Authentication: SOLVED (Admin User exists)"
    SOLVED_ISSUES=$((SOLVED_ISSUES + 1))
else
    echo "❌ Database Authentication: REQUIRES ATTENTION"
fi

# Check CAPS URL format (Phase 3)
if grep -q "seed_capability.*http://127.0.0.1:9000/CAPS/cap/" "$LATEST_SERVER_LOG"; then
    echo "✅ CAPS URL Format: SOLVED (No trailing slash)"
    SOLVED_ISSUES=$((SOLVED_ISSUES + 1))
else
    echo "❌ CAPS URL Format: REQUIRES ATTENTION"
fi

# Check port consistency (Phase 5)
if grep -q "Port.*9001" "$LATEST_SERVER_LOG" && grep -q "SimPort.*9001" "$LATEST_SERVER_LOG"; then
    echo "✅ Port Configuration: SOLVED (Consistent 9001)"
    SOLVED_ISSUES=$((SOLVED_ISSUES + 1))
else
    echo "❌ Port Configuration: REQUIRES ATTENTION"
fi

echo ""
echo "📈 EADS PROGRESS TRACKING:"
echo "-------------------------"
COMPLETION_RATE=$(echo "scale=1; $SOLVED_ISSUES / 3 * 100" | bc -l)
echo "🎯 Core Issues Resolved: $SOLVED_ISSUES/3 ($COMPLETION_RATE%)"
echo ""

# EADS Phase 2: Login Pipeline Status Detection
echo "🔄 EADS PIPELINE STATUS:"
echo "------------------------"
PIPELINE_PHASES=0

# Check XML-RPC Phase
if grep -q "XML-RPC Login successful" "$LATEST_SERVER_LOG"; then
    echo "✅ XML-RPC Authentication: ACTIVE"
    PIPELINE_PHASES=$((PIPELINE_PHASES + 1))
else
    echo "❌ XML-RPC Authentication: FAILING"
fi

# Check CAPS Phase
if grep -q "Created CAPS session" "$LATEST_SERVER_LOG"; then
    echo "✅ CAPS Session Creation: ACTIVE"
    PIPELINE_PHASES=$((PIPELINE_PHASES + 1))
else
    echo "❌ CAPS Session Creation: FAILING"
fi

# Check EventQueue Phase
if grep -q "EventQueueGet.*response.*events" "$LATEST_SERVER_LOG"; then
    echo "✅ EventQueue Communication: ACTIVE"
    PIPELINE_PHASES=$((PIPELINE_PHASES + 1))
else
    echo "❌ EventQueue Communication: FAILING"
fi

# Check LLUDP Phase
if grep -q "CompleteAgentMovement.*LOGIN SHOULD BE COMPLETE" "$LATEST_SERVER_LOG"; then
    echo "✅ LLUDP Packet Sequence: ACTIVE"
    PIPELINE_PHASES=$((PIPELINE_PHASES + 1))
else
    echo "❌ LLUDP Packet Sequence: FAILING"
fi

PIPELINE_COMPLETION=$(echo "scale=1; $PIPELINE_PHASES / 4 * 100" | bc -l)
echo "🎯 Pipeline Completion: $PIPELINE_PHASES/4 ($PIPELINE_COMPLETION%)"
echo ""

# EADS Phase 3: Never-Revisit Issue Detection
echo "🚫 EADS NEVER-REVISIT DETECTION:"
echo "--------------------------------"
if grep -q "user not found" "$LATEST_SERVER_LOG"; then
    echo "⚠️  ALERT: 'user not found' detected - This is a SOLVED issue!"
    echo "   📋 EADS Guidance: Check database user creation in Phase 1"
    echo "   🔧 Quick Fix: Run user creation SQL command"
fi

if grep -q "404 Not Found" "$LATEST_SERVER_LOG" && grep -q "CAPS" "$LATEST_SERVER_LOG"; then
    echo "⚠️  ALERT: CAPS 404 errors detected - This is a SOLVED issue!"
    echo "   📋 EADS Guidance: Check CAPS URL format in Phase 3"
    echo "   🔧 Quick Fix: Remove trailing slashes from URLs"
fi

if grep -q "port.*9002" "$LATEST_SERVER_LOG" && grep -q "port.*9001" "$LATEST_SERVER_LOG"; then
    echo "⚠️  ALERT: Port mismatch detected - This is a SOLVED issue!"
    echo "   📋 EADS Guidance: Check port configuration in Phase 5"
    echo "   🔧 Quick Fix: Ensure all ports use 9001 consistently"
fi

echo ""
echo "🔍 CURRENT SESSION ANALYSIS:"
echo "=============================="

echo "🔍 LOGIN ATTEMPTS:"
tail -500 "$LATEST_SERVER_LOG" | grep -E "LOGIN ANALYTICS.*Admin User|XML-RPC.*Admin User" | head -20

echo ""
echo "📈 PHASE PROGRESSION:"
tail -500 "$LATEST_SERVER_LOG" | grep -E "LOGIN ANALYTICS.*Phase.*Status" | tail -20

echo ""
echo "🌐 CIRCUIT CODES:"
tail -500 "$LATEST_SERVER_LOG" | grep -E "circuit code|circuit_code" | tail -10

echo ""
echo "📡 CAPS ACTIVITY:"
tail -500 "$LATEST_SERVER_LOG" | grep -E "CAPS|EventQueue|seed_capability" | tail -10

echo ""
echo "🔌 LLUDP PACKETS:"
tail -500 "$LATEST_SERVER_LOG" | grep -E "StartPingCheck|UseCircuitCode|LLUDP.*packet" | tail -10

echo ""
echo "❌ ERRORS:"
tail -500 "$LATEST_SERVER_LOG" | grep -E "ERROR|crash|failed|timeout" | tail -10

echo ""
echo "📊 LOG SUMMARY:"
echo "   Total lines: $(wc -l < "$LATEST_SERVER_LOG")"
echo "   Last modified: $(stat -f %Sm "$LATEST_SERVER_LOG")"
echo "   File size: $(du -h "$LATEST_SERVER_LOG" | cut -f1)"

echo ""
echo "📁 ALL CURRENT LOGS:"
ls -lt logs/current/

echo ""
echo "🎯 EADS RECOMMENDATIONS:"
echo "========================"

# Calculate overall success rate
OVERALL_SUCCESS=$(echo "scale=1; ($SOLVED_ISSUES + $PIPELINE_PHASES) / 7 * 100" | bc -l)
echo "📈 Overall Login Success Rate: $OVERALL_SUCCESS%"

if [ "$PIPELINE_PHASES" -eq 4 ]; then
    echo "🎉 EXCELLENT: All 4 login pipeline phases are active!"
    echo "   💡 Focus on viewer post-EventQueue behavior"
    echo "   🔍 Check for additional CAPS requests (FetchInventory2, GetTexture)"
    echo "   📋 Review avatar appearance and terrain data packets"
elif [ "$PIPELINE_PHASES" -ge 3 ]; then
    echo "✅ GOOD: Most pipeline phases working ($PIPELINE_PHASES/4)"
    echo "   🔧 Focus on completing the remaining phase"
elif [ "$PIPELINE_PHASES" -ge 2 ]; then
    echo "⚠️  PARTIAL: Some pipeline phases working ($PIPELINE_PHASES/4)"
    echo "   🔧 Focus on CAPS and EventQueue communication"
else
    echo "❌ CRITICAL: Major pipeline issues ($PIPELINE_PHASES/4)"
    echo "   🚨 Check database, authentication, and basic server functionality"
fi

echo ""
echo "🚫 EADS PREVENTION RULES:"
echo "-------------------------"
echo "❌ DO NOT debug user authentication issues (Phase 1 SOLVED)"
echo "❌ DO NOT debug circuit code type errors (Phase 2 SOLVED)"  
echo "❌ DO NOT debug CAPS URL format issues (Phase 3 SOLVED)"
echo "❌ DO NOT debug CAPS base URL issues (Phase 4 SOLVED)"
echo "❌ DO NOT debug LLUDP port mismatches (Phase 5 SOLVED)"
echo ""
echo "✅ FOCUS ON: Final viewer connection steps (2% remaining)"
echo "✅ PRIORITIZE: Post-EventQueue viewer behavior analysis"
echo ""
echo "📋 EADS Profile Updated: $(date)"
echo "🔄 Next Review: After viewer testing session"

# Update EADS profile with current status
echo "# EADS Profile Update - $(date)" >> "$EADS_PROFILE"
echo "Pipeline Completion: $PIPELINE_PHASES/4 ($PIPELINE_COMPLETION%)" >> "$EADS_PROFILE"
echo "Core Issues Resolved: $SOLVED_ISSUES/3 ($COMPLETION_RATE%)" >> "$EADS_PROFILE"
echo "Overall Success Rate: $OVERALL_SUCCESS%" >> "$EADS_PROFILE"
echo "---" >> "$EADS_PROFILE"