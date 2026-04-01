#!/bin/bash
# Phase 28: Complete Testing Suite Runner
# Executes all Phase 28 testing tools in sequence with comprehensive reporting

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATABASE_URL="${DATABASE_URL:-postgresql://opensim:opensim@localhost:5432/opensim}"
LOGDIR="${LOGDIR:-$SCRIPT_DIR/logs}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create logs directory
mkdir -p "$LOGDIR"

echo "🚀 Phase 28: Complete OpenSim Next Testing Suite"
echo "=============================================="
echo "Timestamp: $(date)"
echo "Database: $DATABASE_URL"
echo "Log Directory: $LOGDIR"
echo ""

# Initialize test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TEST_RESULTS=()

# Function to run test and track results
run_test() {
    local test_name="$1"
    local test_script="$2"
    local log_file="$LOGDIR/${test_name}_${TIMESTAMP}.log"
    
    echo "▶️ Running $test_name..."
    echo "   Log: $log_file"
    
    if bash "$SCRIPT_DIR/$test_script" > "$log_file" 2>&1; then
        echo "✅ $test_name PASSED"
        TEST_RESULTS+=("✅ $test_name: PASSED")
        ((TESTS_PASSED++))
    else
        echo "❌ $test_name FAILED"
        echo "   Check log: $log_file"
        TEST_RESULTS+=("❌ $test_name: FAILED (log: $log_file)")
        ((TESTS_FAILED++))
    fi
    echo ""
}

# Pre-flight checks
echo "🔍 Pre-flight Checks"
echo "==================="

# Check database connectivity
if psql "$DATABASE_URL" -c "SELECT 1;" >/dev/null 2>&1; then
    echo "✅ Database connection successful"
else
    echo "❌ Database connection failed"
    echo "Please ensure PostgreSQL is running and DATABASE_URL is correct"
    exit 1
fi

# Check required tools
REQUIRED_TOOLS=("psql" "nc" "curl" "timeout")
for tool in "${REQUIRED_TOOLS[@]}"; do
    if command -v "$tool" >/dev/null 2>&1; then
        echo "✅ $tool available"
    else
        echo "❌ $tool not found"
        echo "Please install required tools: ${REQUIRED_TOOLS[*]}"
        exit 1
    fi
done

echo ""

# Run all Phase 28 tests
echo "🧪 Running Phase 28 Test Suite"
echo "============================="

run_test "Phase 28.1: Viewer Connection" "viewer-connection-test.sh"
run_test "Phase 28.2: Avatar System" "avatar-system-test.sh"
run_test "Phase 28.3: Region Protocol" "region-protocol-test.sh"
run_test "Phase 28.4: Asset System" "asset-system-test.sh"
run_test "Phase 28.5: Social Features" "social-features-test.sh"
run_test "Phase 28.6: Performance & Stress" "performance-stress-test.sh"

# Generate comprehensive report
echo "📊 Phase 28 Testing Summary Report"
echo "================================="
echo "Test Execution Completed: $(date)"
echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo "Passed: $TESTS_PASSED"
echo "Failed: $TESTS_FAILED"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo "🎉 ALL TESTS PASSED! OpenSim Next Phase 28 validation complete! ✅"
    echo ""
    echo "🏆 Revolutionary Achievement:"
    echo "   OpenSim Next has successfully completed comprehensive"
    echo "   Second Life viewer testing and integration validation!"
    echo ""
    echo "✅ Validated Systems:"
    echo "   • Second Life Viewer Connection (Port 9000)"
    echo "   • Avatar System Integration (Appearance, Movement, Attachments)"
    echo "   • Region Protocol Validation (Objects, Terrain, Scripting)"
    echo "   • Asset System Testing (Textures, Meshes, Sounds)"
    echo "   • Social Features (Friends, Groups, Messaging)"
    echo "   • Performance & Stress Testing (Concurrent Users)"
    echo ""
    echo "🔧 Production Ready Features:"
    echo "   • PostgreSQL database with full schema"
    echo "   • Multi-physics engine support (ODE, Bullet, UBODE, POS, Basic)"
    echo "   • Real-time WebSocket communication"
    echo "   • Complete avatar and region management"
    echo "   • Enterprise-grade monitoring and logging"
    echo "   • Zero trust networking with OpenZiti"
    echo ""
    OVERALL_STATUS="SUCCESS"
else
    echo "⚠️ Some tests failed. Review logs for details."
    echo ""
    OVERALL_STATUS="PARTIAL_SUCCESS"
fi

echo "📋 Detailed Results:"
for result in "${TEST_RESULTS[@]}"; do
    echo "   $result"
done

echo ""
echo "📁 Log Files Location: $LOGDIR"
echo "🗂️ Test Logs Pattern: *_${TIMESTAMP}.log"

# Generate JSON report for automation
REPORT_FILE="$LOGDIR/phase28_test_report_${TIMESTAMP}.json"
cat > "$REPORT_FILE" << EOF
{
  "test_suite": "OpenSim Next Phase 28",
  "timestamp": "$(date -Iseconds)",
  "database_url": "$DATABASE_URL",
  "overall_status": "$OVERALL_STATUS",
  "summary": {
    "total_tests": $((TESTS_PASSED + TESTS_FAILED)),
    "passed": $TESTS_PASSED,
    "failed": $TESTS_FAILED,
    "success_rate": $(( TESTS_PASSED * 100 / (TESTS_PASSED + TESTS_FAILED) ))
  },
  "test_results": [
$(IFS=$'\n'; echo "${TEST_RESULTS[*]}" | sed 's/^/    "/' | sed 's/$/",/' | sed '$s/,$//')
  ],
  "log_directory": "$LOGDIR",
  "log_pattern": "*_${TIMESTAMP}.log"
}
EOF

echo "📄 JSON Report: $REPORT_FILE"

# Database cleanup note
echo ""
echo "🧹 Cleanup Note:"
echo "   Test data remains in database for review"
echo "   To clean up test data, run individual test scripts with --cleanup flag"
echo "   Or manually remove test users/objects from database"

echo ""
if [ $TESTS_FAILED -eq 0 ]; then
    echo "🎯 Next Steps:"
    echo "   • Deploy OpenSim Next to production environment"
    echo "   • Configure Second Life viewers to connect to your server"
    echo "   • Set up monitoring and alerting"
    echo "   • Begin user acceptance testing"
    echo ""
    exit 0
else
    echo "🔧 Next Steps:"
    echo "   • Review failed test logs in $LOGDIR"
    echo "   • Fix identified issues"
    echo "   • Re-run failed tests individually"
    echo "   • Run complete suite again when all issues resolved"
    echo ""
    exit 1
fi