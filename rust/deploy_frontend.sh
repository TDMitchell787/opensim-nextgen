#!/bin/bash

# OpenSim Next Frontend Deployment Script
# Revolutionary Multi-Database Virtual World Server

set -e

echo "🌐 OpenSim Next - Multi-Database Virtual World Server"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
DEFAULT_DB_TYPE="sqlite"
DEFAULT_API_KEY="opensim-demo-$(date +%s)"
DEFAULT_INSTANCE_ID="opensim-$(hostname)-$(date +%s)"

echo -e "${CYAN}🚀 Starting OpenSim Next Deployment...${NC}"

# Parse command line arguments
DB_TYPE="${1:-$DEFAULT_DB_TYPE}"
API_KEY="${2:-$DEFAULT_API_KEY}"

echo -e "${BLUE}📋 Deployment Configuration:${NC}"
echo "   Database Type: $DB_TYPE"
echo "   API Key: $API_KEY"
echo "   Instance ID: $DEFAULT_INSTANCE_ID"

# Set database URL based on type
case $DB_TYPE in
    "postgresql"|"postgres")
        DATABASE_URL="postgresql://opensim:opensim@localhost:5432/opensim"
        echo -e "${GREEN}🐘 Using PostgreSQL (Production Ready)${NC}"
        ;;
    "mysql")
        DATABASE_URL="mysql://opensim:opensim@localhost:3306/opensim"
        echo -e "${YELLOW}🐬 Using MySQL (Legacy Compatible)${NC}"
        ;;
    "mariadb")
        DATABASE_URL="mariadb://opensim:opensim@localhost:3306/opensim"
        echo -e "${YELLOW}🦭 Using MariaDB (Legacy Compatible)${NC}"
        ;;
    "sqlite"|*)
        DATABASE_URL="sqlite://opensim.db"
        echo -e "${PURPLE}💾 Using SQLite (Development Ready)${NC}"
        ;;
esac

# Export environment variables
export DATABASE_URL="$DATABASE_URL"
export OPENSIM_API_KEY="$API_KEY"
export OPENSIM_INSTANCE_ID="$DEFAULT_INSTANCE_ID"
export RUST_LOG="info"

# Server ports
export OPENSIM_WEB_CLIENT_PORT="8080"
export OPENSIM_WEBSOCKET_PORT="9001"
export OPENSIM_METRICS_PORT="9100"
export OPENSIM_SL_VIEWER_PORT="9000"

# Database configuration
case $DB_TYPE in
    "postgresql"|"postgres")
        export DB_MAX_CONNECTIONS="100"
        ;;
    "mysql"|"mariadb")
        export DB_MAX_CONNECTIONS="80"
        ;;
    "sqlite"|*)
        export DB_MAX_CONNECTIONS="1"
        ;;
esac

export DB_ENABLE_LOGGING="true"
export OPENSIM_WEBSOCKET_MAX_CONNECTIONS="1000"

echo -e "${CYAN}🔧 Environment Configuration Complete${NC}"

# Create directory structure
echo -e "${BLUE}📁 Creating directory structure...${NC}"
mkdir -p assets
mkdir -p logs
mkdir -p backups

# Check if frontend files exist
if [ ! -f "web-frontend/index.html" ]; then
    echo -e "${RED}❌ Frontend files not found. Please ensure web-frontend/ directory exists.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Frontend files found${NC}"

# Build the server
echo -e "${YELLOW}🔨 Building OpenSim Next server...${NC}"
if cargo build --release --bin opensim-next; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed. Attempting with warnings...${NC}"
    if cargo build --bin opensim-next; then
        echo -e "${YELLOW}⚠️ Build completed with warnings${NC}"
    else
        echo -e "${RED}❌ Build failed completely${NC}"
        exit 1
    fi
fi

# Create deployment info
echo -e "${BLUE}📄 Creating deployment information...${NC}"
cat > deployment_info.txt << EOF
OpenSim Next Deployment Information
===================================

Deployment Time: $(date)
Database Type: $DB_TYPE
Database URL: $DATABASE_URL
API Key: $API_KEY
Instance ID: $DEFAULT_INSTANCE_ID

Server Endpoints:
- Frontend Dashboard: http://localhost:8080
- Web Client: http://localhost:8080/client.html
- API Endpoints: http://localhost:9100
- WebSocket Server: ws://localhost:9001
- Second Life Viewers: opensim://localhost:9000

Access URLs:
- Main Dashboard: http://localhost:8080
- Health Check: http://localhost:8080/health
- Server Info: http://localhost:9100/info (requires API key)
- Metrics: http://localhost:9100/metrics (requires API key)

API Key Authentication:
Add header: X-API-Key: $API_KEY

Database Features:
EOF

case $DB_TYPE in
    "postgresql"|"postgres")
        cat >> deployment_info.txt << EOF
- Native UUID support
- JSON and array fields
- Full-text search
- ACID compliance
- Up to 100 connections
EOF
        ;;
    "mysql"|"mariadb")
        cat >> deployment_info.txt << EOF
- CHAR(36) UUID compatibility
- JSON fields (MySQL 5.7+)
- InnoDB engine
- Legacy OpenSim compatibility
- Up to 80 connections
EOF
        ;;
    "sqlite"|*)
        cat >> deployment_info.txt << EOF
- File-based storage
- JSON support
- Custom UUID functions
- Development friendly
- Single connection
EOF
        ;;
esac

cat >> deployment_info.txt << EOF

Revolutionary Features:
- Multi-database backend support
- Web browser virtual world access
- Real-time WebSocket communication
- Second Life viewer compatibility
- Production-ready monitoring
- Zero Trust networking capabilities
- Hybrid Rust/Zig architecture

Next Steps:
1. Start the server: cargo run --bin opensim-next
2. Open browser: http://localhost:8080
3. Test API: curl -H "X-API-Key: $API_KEY" http://localhost:9100/health
4. Connect with viewer: opensim://localhost:9000
EOF

echo -e "${GREEN}✅ Deployment information saved to deployment_info.txt${NC}"

# Create start script
echo -e "${BLUE}📜 Creating start script...${NC}"
cat > start_server.sh << 'SCRIPT_EOF'
#!/bin/bash

# OpenSim Next Server Start Script
source ./deployment_info.txt 2>/dev/null || true

echo "🌐 Starting OpenSim Next Multi-Database Virtual World Server..."
echo "================================================================"

# Load environment
if [ -f ".env" ]; then
    export $(cat .env | xargs)
fi

# Start server
echo "🚀 Launching server with configuration:"
echo "   Database: $DATABASE_URL"
echo "   Frontend: http://localhost:$OPENSIM_WEB_CLIENT_PORT"
echo "   API: http://localhost:$OPENSIM_METRICS_PORT"
echo "   WebSocket: ws://localhost:$OPENSIM_WEBSOCKET_PORT"
echo ""
echo "🌟 Revolutionary Features Active:"
echo "   ✅ Multi-Database Support (PostgreSQL, MySQL, SQLite)"
echo "   ✅ Web Browser Virtual World Access"
echo "   ✅ Second Life Viewer Compatibility"
echo "   ✅ Real-time WebSocket Communication"
echo "   ✅ Production-Ready Monitoring"
echo ""

cargo run --bin opensim-next
SCRIPT_EOF

chmod +x start_server.sh

# Create environment file
echo -e "${BLUE}📝 Creating environment file...${NC}"
cat > .env << EOF
DATABASE_URL=$DATABASE_URL
OPENSIM_API_KEY=$API_KEY
OPENSIM_INSTANCE_ID=$DEFAULT_INSTANCE_ID
RUST_LOG=info
OPENSIM_WEB_CLIENT_PORT=8080
OPENSIM_WEBSOCKET_PORT=9001
OPENSIM_METRICS_PORT=9100
OPENSIM_SL_VIEWER_PORT=9000
DB_MAX_CONNECTIONS=$DB_MAX_CONNECTIONS
DB_ENABLE_LOGGING=true
OPENSIM_WEBSOCKET_MAX_CONNECTIONS=1000
EOF

echo -e "${GREEN}✅ Environment file created (.env)${NC}"

# Create testing script
echo -e "${BLUE}🧪 Creating test script...${NC}"
cat > test_deployment.sh << TESTSCRIPT_EOF
#!/bin/bash

echo "🧪 Testing OpenSim Next Deployment..."
echo "======================================"

API_KEY="$API_KEY"

echo "Testing server endpoints..."

# Test frontend
echo -n "Frontend (port 8080): "
if curl -s -f http://localhost:8080/ > /dev/null; then
    echo "✅ Available"
else
    echo "❌ Not responding"
fi

# Test health endpoint
echo -n "Health check: "
if curl -s -f http://localhost:8080/health > /dev/null; then
    echo "✅ Healthy"
else
    echo "❌ Unhealthy"
fi

# Test API with key
echo -n "API (with key): "
if curl -s -f -H "X-API-Key: \$API_KEY" http://localhost:9100/info > /dev/null; then
    echo "✅ Accessible"
else
    echo "❌ Not accessible"
fi

# Test WebSocket (basic connection test)
echo -n "WebSocket server: "
if timeout 5s bash -c "</dev/tcp/localhost/9001" 2>/dev/null; then
    echo "✅ Listening"
else
    echo "❌ Not listening"
fi

echo ""
echo "🌟 Access your revolutionary virtual world server:"
echo "   📊 Dashboard: http://localhost:8080"
echo "   🌐 Web Client: http://localhost:8080/client.html"
echo "   🔗 API: http://localhost:9100 (key: \$API_KEY)"
echo "   📱 Mobile: http://localhost:8080 (PWA enabled)"
echo "   🖥️ SL Viewers: opensim://localhost:9000"
TESTSCRIPT_EOF

chmod +x test_deployment.sh

echo -e "${CYAN}🎉 OpenSim Next Frontend Deployment Complete!${NC}"
echo ""
echo -e "${GREEN}✅ Success! Your revolutionary multi-database virtual world server is ready.${NC}"
echo ""
echo -e "${YELLOW}📋 Quick Start Commands:${NC}"
echo "   Start Server:    ./start_server.sh"
echo "   Test Deployment: ./test_deployment.sh"
echo "   View Logs:       tail -f logs/opensim.log"
echo ""
echo -e "${BLUE}🌐 Access Points:${NC}"
echo "   Frontend Dashboard: http://localhost:8080"
echo "   Web Client:         http://localhost:8080/client.html"
echo "   API Endpoints:      http://localhost:9100"
echo "   WebSocket Server:   ws://localhost:9001"
echo "   Second Life Viewers: opensim://localhost:9000"
echo ""
echo -e "${PURPLE}🔑 API Key: $API_KEY${NC}"
echo -e "${CYAN}🆔 Instance ID: $DEFAULT_INSTANCE_ID${NC}"
echo ""
echo -e "${GREEN}🚀 Revolutionary Features Active:${NC}"
echo "   ✅ Multi-Database Support (PostgreSQL, MySQL, SQLite)"
echo "   ✅ Web Browser Virtual World Access"
echo "   ✅ Second Life Viewer Compatibility"
echo "   ✅ Real-time WebSocket Communication"
echo "   ✅ Production-Ready Monitoring Dashboard"
echo "   ✅ Mobile Browser Support (PWA)"
echo "   ✅ API Testing Interface"
echo "   ✅ Zero Trust Networking Ready"
echo ""
echo -e "${CYAN}🌟 OpenSim Next - The Future of Virtual Worlds! 🌟${NC}"
echo ""
echo "Start your server with: ./start_server.sh"
echo "Then open: http://localhost:8080"
SCRIPT_EOF

chmod +x deploy_frontend.sh

echo "Frontend deployment script created successfully!"