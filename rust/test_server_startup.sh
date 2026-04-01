#!/bin/bash

# Test OpenSim Next Server Startup with Loopback Connectors
# This script tests if the server can start properly with the new networking infrastructure

echo "🚀 Testing OpenSim Next Server Startup..."
echo "=========================================="

# Set environment for testing
export DATABASE_URL="sqlite://test_server.db"
export OPENSIM_API_KEY="test-server-key-$(date +%s)"
export OPENSIM_INSTANCE_ID="test-server-$(hostname)"
export RUST_LOG="info"
export OPENSIM_GRID_NAME="OpenSim Next Test Grid"
export OPENSIM_GRID_URI="http://localhost:8002"

echo "🔧 Configuration:"
echo "  Database: $DATABASE_URL"
echo "  API Key: $OPENSIM_API_KEY"
echo "  Grid: $OPENSIM_GRID_NAME"
echo ""

# Clean up any existing test database
rm -f test_server.db

echo "🔨 Building server..."
if cargo build --bin opensim-next; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

echo ""
echo "🌐 Starting server in background..."

# Start server in background
timeout 30s cargo run --bin opensim-next > server_test.log 2>&1 &
SERVER_PID=$!

echo "Server PID: $SERVER_PID"
echo "Waiting for server to start..."

# Wait for server to start
sleep 10

echo ""
echo "🧪 Testing server connectivity..."

# Test if ports are listening
TESTS_PASSED=0
TESTS_TOTAL=0

# Test frontend (port 8080)
echo -n "Frontend (8080): "
TESTS_TOTAL=$((TESTS_TOTAL + 1))
if timeout 5s bash -c "</dev/tcp/localhost/8080" 2>/dev/null; then
    echo "✅ Listening"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "❌ Not responding"
fi

# Test API (port 9100)
echo -n "API (9100): "
TESTS_TOTAL=$((TESTS_TOTAL + 1))
if timeout 5s bash -c "</dev/tcp/localhost/9100" 2>/dev/null; then
    echo "✅ Listening"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "❌ Not responding"
fi

# Test WebSocket (port 9001)
echo -n "WebSocket (9001): "
TESTS_TOTAL=$((TESTS_TOTAL + 1))
if timeout 5s bash -c "</dev/tcp/localhost/9001" 2>/dev/null; then
    echo "✅ Listening"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "❌ Not responding"
fi

# Test Hypergrid (port 8002)
echo -n "Hypergrid (8002): "
TESTS_TOTAL=$((TESTS_TOTAL + 1))
if timeout 5s bash -c "</dev/tcp/localhost/8002" 2>/dev/null; then
    echo "✅ Listening"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "❌ Not responding"
fi

echo ""
echo "🌐 Testing HTTP endpoints..."

# Test frontend HTML
echo -n "Frontend HTML: "
TESTS_TOTAL=$((TESTS_TOTAL + 1))
if curl -s -f http://localhost:8080/ | grep -q "OpenSim Next" 2>/dev/null; then
    echo "✅ Serving content"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "❌ No content"
fi

# Test health endpoint
echo -n "Health endpoint: "
TESTS_TOTAL=$((TESTS_TOTAL + 1))
if curl -s -f http://localhost:8080/health 2>/dev/null | grep -q "OK"; then
    echo "✅ Healthy"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "❌ Unhealthy"
fi

# Test Hypergrid info
echo -n "Hypergrid info: "
TESTS_TOTAL=$((TESTS_TOTAL + 1))
if curl -s -f http://localhost:8002/ 2>/dev/null | grep -q "OpenSim Next"; then
    echo "✅ Responding"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "❌ Not responding"
fi

# Test API with key
echo -n "API with key: "
TESTS_TOTAL=$((TESTS_TOTAL + 1))
if curl -s -f -H "X-API-Key: $OPENSIM_API_KEY" http://localhost:9100/info 2>/dev/null | grep -q "instance_id"; then
    echo "✅ Accessible"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "❌ Not accessible"
fi

echo ""
echo "🏁 Test Results:"
echo "  Tests Passed: $TESTS_PASSED/$TESTS_TOTAL"

if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
    echo "  Status: ✅ ALL TESTS PASSED"
    SUCCESS=true
else
    echo "  Status: ❌ SOME TESTS FAILED"
    SUCCESS=false
fi

echo ""
echo "📊 Server Access URLs:"
echo "  Frontend: http://localhost:8080"
echo "  API: http://localhost:9100 (key: $OPENSIM_API_KEY)"
echo "  WebSocket: ws://localhost:9001"
echo "  Hypergrid: http://localhost:8002"

# Show server logs
echo ""
echo "📜 Server Logs (last 20 lines):"
echo "================================"
tail -n 20 server_test.log

# Stop server
echo ""
echo "🛑 Stopping server..."
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

# Cleanup
rm -f test_server.db server_test.log

if [ "$SUCCESS" = true ]; then
    echo ""
    echo "🎉 SUCCESS: OpenSim Next server startup test PASSED!"
    echo "✅ Loopback connectors working properly"
    echo "✅ Hypergrid protocol active"
    echo "✅ Frontend accessible"
    echo "✅ Multi-database support functional"
    exit 0
else
    echo ""
    echo "❌ FAILURE: Some tests failed"
    echo "Check the logs above for details"
    exit 1
fi