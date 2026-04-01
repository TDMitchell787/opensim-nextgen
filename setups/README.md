# OpenSim Next Setup Archives

This directory contains organized setup configurations and templates for OpenSim Next, implementing Phase 44 - Enhanced Setup Wizard with Archive Management.

## Directory Structure

### `/templates/` - Pre-built Configuration Templates
- **`beginner/`** - Easy-to-use templates for new users
- **`intermediate/`** - Customizable templates for experienced users  
- **`advanced/`** - Enterprise and specialized configurations

### `/saved-configs/` - User's Archived Configurations
- **`by-name/`** - Organized by setup name (e.g., "gaia-grid-2024")
- **`by-category/`** - Cross-referenced by type (grids, standalone, development)

### `/active-instances/` - Currently Running Server Instances
- Each active instance gets its own directory for multi-instance management

## Template Categories

### 🟢 Beginner Templates
- **Quick Start Standalone** - Single region, auto-configured
- **Small Grid (2x2)** - 4-region grid for learning
- **Educational Basic** - School/classroom setup

### 🟡 Intermediate Templates  
- **Creative Sandbox** - Artist/builder focused with enhanced tools
- **Community Grid** - Social features and group management
- **Hypergrid Enabled** - Inter-grid connectivity setup

### 🔴 Advanced Templates
- **Production Enterprise** - Full enterprise deployment
- **Multi-Physics Demo** - Advanced physics engines showcase
- **Custom Economy** - Complete economy and marketplace

## Archive Package Structure

Each saved configuration contains:
```
setup-name/
├── configs/              # Generated OpenSim configuration files
│   ├── OpenSim.ini
│   ├── Regions/
│   └── config-include/
├── documentation/         # Auto-generated setup documentation
│   ├── README.md
│   ├── STARTUP_GUIDE.md
│   └── TROUBLESHOOTING.md
├── scripts/              # Automation and startup scripts
│   ├── start_server.sh
│   ├── setup_admin.sh
│   └── backup.sh
└── metadata.json         # Configuration metadata and settings
```

## Usage

1. **Via FWDFE Interface**: Use the Setup Wizard button dropdown
2. **Direct Access**: Browse templates and configurations manually
3. **Command Line**: Use `cargo run setup --template <name>` 
4. **API Integration**: RESTful API for programmatic access

## Features

- **Progressive Complexity**: Beginner → Intermediate → Advanced paths
- **Reusable Configurations**: Save and deploy any setup multiple times
- **Multi-Instance Support**: Run multiple OpenSim servers simultaneously
- **Auto-Documentation**: Every setup generates complete documentation
- **One-Click Deployment**: Deploy archived configurations instantly
- **Configuration Cloning**: Duplicate and modify existing setups

---

*This archive system eliminates OpenSim's steep learning curve while maintaining full power for advanced deployments.*