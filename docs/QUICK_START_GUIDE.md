# OpenSim Next Quick Start Guide

**🚀 Get your first virtual world running in 15 minutes!**

This guide will help you set up a single-region development environment for OpenSim Next on your local machine. Perfect for learning, testing, or developing virtual world content.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Initial Setup](#initial-setup)
4. [Configuration](#configuration)
5. [Starting Your World](#starting-your-world)
6. [Connecting with Viewers](#connecting-with-viewers)
7. [Web Client Access](#web-client-access)
8. [Creating Your First User](#creating-your-first-user)
9. [Basic Administration](#basic-administration)
10. [Next Steps](#next-steps)

## Prerequisites

### System Requirements

**Minimum for Development:**
- **OS**: Ubuntu 20.04+, macOS 11+, or Windows 10+
- **CPU**: 4 cores (Intel i5 or AMD equivalent)
- **RAM**: 8GB (16GB recommended)
- **Storage**: 10GB free space (SSD recommended)
- **Network**: Broadband internet connection

### Required Software

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y git curl build-essential pkg-config

# macOS (with Homebrew)
brew install git curl

# Windows (with Chocolatey)
choco install git curl
```

### Install Rust and Zig

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Zig
curl -L https://ziglang.org/download/0.14.0/zig-linux-x86_64-0.14.0.tar.xz | tar -xJ
sudo mv zig-linux-x86_64-0.14.0/zig /usr/local/bin/
```

## Installation

### Clone OpenSim Next

```bash
# Clone the repository
git clone https://github.com/opensim-next/opensim-next.git
cd opensim-next

# Verify installation
./scripts/verify-installation.sh
```

### Quick Install Script

```bash
# Run the automated quick setup script
./scripts/quick-setup.sh --mode=development

# This will:
# ✅ Install dependencies
# ✅ Build Rust and Zig components
# ✅ Create development configuration
# ✅ Set up SQLite database
# ✅ Create default region
# ✅ Configure web client
```

## Initial Setup

### Development Configuration

The quick setup script creates a development-ready configuration:

**Default Settings:**
- **Database**: SQLite (no setup required)
- **Physics**: ODE engine (stable and fast)
- **Region**: "Welcome Island" at coordinates 1000,1000
- **Network**: Local access only (127.0.0.1)
- **Ports**: 9000 (viewers), 9001 (WebSocket), 8080 (web client)
- **Users**: Local account creation enabled

### Environment Variables

```bash
# Add to your ~/.bashrc or ~/.zshrc
export OPENSIM_HOME="$HOME/opensim-next"
export OPENSIM_MODE="development"
export PATH="$OPENSIM_HOME/bin:$PATH"

# Reload shell configuration
source ~/.bashrc
```

## Configuration

### Auto-Configuration Wizard

Launch the Flutter-based auto-configurator for easy setup:

```bash
# Option 1: Web-based configurator
cd opensim-next
./start-configurator.sh

# Then open: http://localhost:3000
```

```bash
# Option 2: Command-line configurator
./configure-opensim.sh --interactive

# Follow the guided setup:
# 1. Choose deployment type: Development
# 2. Select region name: MyFirstRegion
# 3. Set admin account details
# 4. Confirm settings
```

### Manual Configuration (Optional)

If you prefer manual configuration:

```bash
# Edit main configuration
nano config-include/OpenSim.ini

# Key settings for development:
[Network]
http_listener_port = 9000
hostname = "127.0.0.1"

[Database]
Include-Storage = "config-include/storage/SQLiteStandalone.ini"

[Physics]
DefaultPhysicsEngine = "ODE"

[WebSocket]
Enabled = true
Port = 9001

[WebClient]
Enabled = true
Port = 8080
```

## Starting Your World

### Start OpenSim Next

```bash
# Start the server
cd opensim-next
./start-opensim.sh

# You should see:
# ✅ Database initialized
# ✅ Region "Welcome Island" loaded
# ✅ HTTP listener started on port 9000
# ✅ WebSocket server started on port 9001
# ✅ Web client available on port 8080
# ✅ Ready for connections!
```

### Verify Server Status

```bash
# Check server health
curl http://localhost:9100/health

# Expected response:
{
  "status": "healthy",
  "uptime": "00:01:23",
  "regions": 1,
  "users_online": 0,
  "physics_engine": "ODE",
  "websocket_enabled": true
}
```

### Server Logs

```bash
# View real-time logs
tail -f logs/opensim.log

# Or use the web interface
open http://localhost:8090/logs
```

## Connecting with Viewers

### Firestorm Viewer Setup

1. **Download Firestorm**: https://www.firestormviewer.org/downloads/
2. **Create Grid Entry**:
   - Click "Preferences" → "Opensim"
   - Add new grid:
     - **Grid Name**: "My OpenSim Next"
     - **Login URI**: `http://127.0.0.1:9000/`
     - **Grid ID**: `opensim-next-dev`

3. **Connect**:
   - Select your new grid from the dropdown
   - Use credentials created during setup
   - Click "Log In"

### Singularity Viewer Setup

1. **Download Singularity**: http://www.singularityviewer.org/
2. **Grid Manager**:
   - Open "Grid Manager"
   - Add grid:
     - **Grid Name**: "OpenSim Next Dev"
     - **Login URL**: `http://127.0.0.1:9000/`
     - **Helper URL**: `http://127.0.0.1:9000/`

### Hippo Viewer Setup

1. **Download Hippo**: https://forge.opensimulator.org/gf/project/phpopensim/
2. **Grid Selection**:
   - Add custom grid
   - **Login URL**: `http://127.0.0.1:9000/login/`

## Web Client Access

### Browser-Based Access

OpenSim Next includes a revolutionary web client that works in any modern browser:

```bash
# Access the web client
open http://localhost:8080

# Or direct WebSocket connection
open http://localhost:8080/client.html
```

### Web Client Features

**✅ Full Virtual World Access:**
- 3D environment rendering
- Avatar movement and interaction
- Chat and communication
- Inventory management
- Object manipulation

**✅ Cross-Platform Support:**
- **Desktop**: Chrome, Firefox, Safari, Edge
- **Mobile**: iOS Safari, Android Chrome
- **Tablets**: iPad, Android tablets

### Web Client Login

1. **Open**: http://localhost:8080
2. **Select Login Method**:
   - Use same credentials as viewer login
   - Or create account directly in web interface
3. **Choose Avatar**: Select from default avatars
4. **Enter World**: Click "Enter World"

## Creating Your First User

### Using the Web Interface

```bash
# Open admin panel
open http://localhost:8090

# Or use command line
./create-user.sh \
  --first-name="John" \
  --last-name="Doe" \
  --email="john@example.com" \
  --password="secure123" \
  --god-mode=true
```

### Using Console Commands

```bash
# Connect to running OpenSim console
./connect-console.sh

# Create user account
create user John Doe john@example.com secure123

# Make user an admin
set user level John Doe 200

# Exit console
quit
```

### Using the Auto-Configurator

The Flutter-based configurator includes user management:

```bash
# Launch configurator
./start-configurator.sh

# Navigate to: User Management → Create User
# Fill in user details and permissions
```

## Basic Administration

### Server Management

```bash
# Stop server gracefully
./stop-opensim.sh

# Restart server
./restart-opensim.sh

# Check server status
./status-opensim.sh

# View performance metrics
./show-metrics.sh
```

### Region Management

```bash
# List regions
./list-regions.sh

# Create new region
./create-region.sh \
  --name="Sandbox" \
  --location="1001,1000" \
  --size="256x256" \
  --physics="ODE"

# Load OAR (region backup)
./load-oar.sh --file="regions/welcome-island.oar" --region="Welcome Island"

# Save OAR
./save-oar.sh --region="Welcome Island" --file="backup/welcome-$(date +%Y%m%d).oar"
```

### User Management

```bash
# List users
./list-users.sh

# Reset password
./reset-password.sh --user="John Doe" --password="newpassword123"

# Grant admin privileges
./grant-admin.sh --user="John Doe"

# Ban user (if needed)
./ban-user.sh --user="BadUser Griefer" --reason="Terms violation"
```

### Database Management

```bash
# Backup database
./backup-database.sh --output="backup/db-$(date +%Y%m%d).sqlite"

# View database statistics
./db-stats.sh

# Clean old logs
./cleanup-logs.sh --days=7
```

## Next Steps

### Essential First Tasks

**1. Explore Your World**
- Walk around Welcome Island
- Test avatar movement and camera controls
- Try both viewer and web client
- Experiment with chat and gestures

**2. Basic Building**
- Create primitive objects (cubes, spheres, cylinders)
- Learn object editing tools
- Practice positioning and scaling
- Save your creations to inventory

**3. Import Content**
- Download free OAR regions from OpenSim Community
- Load sample content: `./load-oar.sh --file=samples/sandbox.oar`
- Import mesh objects and textures
- Set up a building sandbox area

**4. Configure Physics**
- Test different physics engines: `./switch-physics.sh --engine=Bullet`
- Compare performance: ODE vs UBODE vs Bullet
- Try particle effects with POS engine
- Test vehicle physics (if available)

### Advanced Configuration

**Enable Production Features:**
```bash
# Enable monitoring
./enable-monitoring.sh --prometheus --grafana

# Set up asset CDN
./setup-cdn.sh --provider=cloudflare

# Configure backup automation
./setup-backups.sh --schedule="0 2 * * *" --retention=30

# Enable zero trust networking
./enable-ziti.sh --controller=your-controller.com
```

**Multi-Region Setup:**
```bash
# Add second region
./create-region.sh --name="Mainland" --location="1002,1000"

# Enable region crossing
./enable-region-crossing.sh

# Set up grid topology
./configure-grid.sh --topology=hub-and-spoke
```

### Learning Resources

**📚 Documentation:**
- **Complete Manual**: [USER_MANUAL.md](USER_MANUAL.md)
- **Configuration Guide**: [Configuration Chapter](USER_MANUAL.md#chapter-3-configuration)
- **Security Guide**: [SECURITY_HARDENING_GUIDE.md](SECURITY_HARDENING_GUIDE.md)
- **Migration Guide**: [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md)

**🎓 Tutorials:**
- Building Your First Virtual World
- Understanding Physics Engines
- Web Client Development
- Zero Trust Network Setup

**🛠️ Developer Resources:**
- **API Documentation**: http://localhost:8090/api/docs
- **SDK Downloads**: Multiple language support
- **GitHub Repository**: https://github.com/opensim-next
- **Community Forum**: https://forum.opensim-next.org

### Community and Support

**💬 Get Help:**
- **Discord**: Join our developer community
- **Forum**: Ask questions and share experiences
- **Documentation**: Comprehensive guides and references
- **GitHub Issues**: Report bugs and request features

**🤝 Contribute:**
- Test new features and report feedback
- Contribute translations for additional languages
- Create and share region content (OAR files)
- Help with documentation improvements

### Performance Optimization

**🚀 Speed Up Your World:**
```bash
# Optimize database
./optimize-database.sh

# Enable caching
./enable-caching.sh --size=1GB

# Tune physics performance
./tune-physics.sh --engine=ODE --max-prims=5000

# Enable compression
./enable-compression.sh --assets --textures
```

**📊 Monitor Performance:**
```bash
# Real-time monitoring
open http://localhost:8090/dashboard

# Performance reports
./generate-performance-report.sh --days=7

# Resource usage
./show-resource-usage.sh --detailed
```

### Production Deployment

When you're ready to move beyond development:

**🌐 Internet Access:**
- Configure firewall and port forwarding
- Set up dynamic DNS or static IP
- Enable SSL/TLS certificates
- Configure production database (PostgreSQL)

**🔒 Security Hardening:**
- Follow the [Security Hardening Guide](SECURITY_HARDENING_GUIDE.md)
- Enable zero trust networking
- Set up monitoring and alerting
- Configure automated backups

**📈 Scaling:**
- Add more regions
- Implement load balancing
- Enable auto-scaling
- Set up multiple physics engines

## Troubleshooting

### Common Issues

**❌ Server Won't Start**
```bash
# Check ports are available
netstat -tulpn | grep :9000
netstat -tulpn | grep :9001

# Check logs for errors
tail -f logs/opensim.log

# Verify configuration
./validate-config.sh
```

**❌ Can't Connect with Viewer**
```bash
# Verify server is running
curl http://localhost:9000/

# Check firewall settings
sudo ufw status

# Test login service
curl -X POST http://localhost:9000/login/
```

**❌ Web Client Not Loading**
```bash
# Check WebSocket service
curl -v http://localhost:9001/

# Verify web files
ls -la web/client/

# Check browser console for errors
# Open browser developer tools (F12)
```

**❌ Physics Not Working**
```bash
# Verify physics engine
./check-physics.sh --engine=ODE

# Rebuild physics libraries
./rebuild-physics.sh

# Switch to basic physics
./switch-physics.sh --engine=Basic
```

### Getting Help

**🆘 Emergency Issues:**
```bash
# Emergency reset
./emergency-reset.sh --preserve-data

# Safe mode startup
./start-opensim.sh --safe-mode

# Factory reset (⚠️ destroys all data)
./factory-reset.sh --confirm
```

**📞 Support Channels:**
- **Immediate Help**: Check logs in `logs/opensim.log`
- **Community Support**: Discord and Forums
- **Bug Reports**: GitHub Issues
- **Documentation**: Built-in help system

---

## Success! 🎉

You now have a fully functional OpenSim Next virtual world running on your machine!

**What You've Accomplished:**
- ✅ Installed OpenSim Next development environment
- ✅ Created your first virtual region
- ✅ Connected with traditional viewers
- ✅ Accessed through web browser
- ✅ Created user accounts
- ✅ Learned basic administration

**Your virtual world is accessible at:**
- **Viewers**: `http://127.0.0.1:9000/`
- **Web Client**: `http://localhost:8080`
- **Admin Panel**: `http://localhost:8090`

**Next recommended steps:**
1. Explore the [User Manual](USER_MANUAL.md) for advanced features
2. Join the community for support and sharing
3. Start building your virtual world content
4. Consider upgrading to production deployment

Welcome to the future of virtual worlds with OpenSim Next! 🌟

---

*Quick Start Guide - Last updated: December 2024 - v1.0.0*