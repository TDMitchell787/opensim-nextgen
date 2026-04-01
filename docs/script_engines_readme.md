# OpenSim Next Script Engines

This directory contains script engine implementations and configurations for OpenSim Next.

## Overview

OpenSim Next supports multiple script engines for different performance and compatibility requirements:

### Native Engine (Recommended)
- High-performance Rust/Zig implementation
- Advanced LSL features and optimization
- Memory safety and zero-cost abstractions
- SIMD optimization for math operations

### YEngine (Legacy Compatibility)
- Compatibility layer for existing YEngine scripts
- State migration from legacy installations
- Gradual transition path to Native engine

### XEngine (Legacy Compatibility)  
- Basic compatibility for legacy XEngine scripts
- Minimal feature set for simple scripts
- Migration assistance to modern engines

## Directory Structure

```
ScriptEngines/
├── README.md                 # This file
├── Native/                   # Native Rust/Zig engine
│   ├── README.md
│   └── template.ini
├── YEngine/                  # YEngine compatibility
│   ├── README.md
│   └── template.ini
├── XEngine/                  # XEngine compatibility
│   ├── README.md
│   └── template.ini
├── Common/                   # Shared components
└── Tests/                    # Test scripts and data
```

## Configuration

Script engines are configured in `config-include/ScriptEngines.ini`:

```ini
[ScriptEngines]
DefaultEngine = "Native"
EnableScriptDebugging = true
MaxScriptMemory = 65536
ScriptTimeout = 30

[Native]
Enabled = true
Class = "OpenSim.Region.ScriptEngine.Native.NativeScriptEngine"
Assembly = "OpenSim.Region.ScriptEngine.Native.dll"

[YEngine]
Enabled = false
Class = "OpenSim.Region.ScriptEngine.YEngine.YEngine"
Assembly = "OpenSim.Region.ScriptEngine.YEngine.dll"

[XEngine]
Enabled = false
Class = "OpenSim.Region.ScriptEngine.XEngine.XEngine"
Assembly = "OpenSim.Region.ScriptEngine.XEngine.dll"
```

## Management

Use the script engine manager tool:

```bash
# Initialize directory structure
cargo run --bin script_engine_manager init

# List available engines
cargo run --bin script_engine_manager list

# Enable/disable engines
cargo run --bin script_engine_manager enable Native
cargo run --bin script_engine_manager disable XEngine

# Show status
cargo run --bin script_engine_manager status

# Validate setup
cargo run --bin script_engine_manager validate
```

## Testing

Run the comprehensive test suite:

```bash
# Basic tests
cargo run --bin test_script_engines

# With performance benchmarks
cargo run --bin test_script_engines --performance

# Verbose output
cargo run --bin test_script_engines --verbose
```

## Performance Comparison

| Engine | Performance | Memory Safety | LSL Support | Legacy Compat |
|--------|-------------|---------------|-------------|---------------|
| Native | ⭐⭐⭐⭐⭐ | ✅ Guaranteed | Full | Partial |
| YEngine | ⭐⭐⭐ | ⚠️ Runtime | Good | ✅ Full |
| XEngine | ⭐⭐ | ⚠️ Runtime | Basic | ✅ Full |

## Migration Guide

### From XEngine to Native
1. Enable Native engine alongside XEngine
2. Test critical scripts with Native engine
3. Gradually migrate scripts by region
4. Disable XEngine when migration complete

### From YEngine to Native
1. Export script states from YEngine
2. Enable Native engine
3. Import states and verify functionality
4. Update scripts to use Native-specific features

## Troubleshooting

### Common Issues

**Script compilation errors:**
- Check LSL syntax compatibility
- Verify engine-specific features
- Review script memory usage

**Performance issues:**
- Enable Native engine for better performance
- Check script timeout settings
- Monitor memory usage per script

**Migration problems:**
- Backup script states before migration
- Test in development environment first
- Check compatibility matrix

### Debug Mode

Enable debug logging in `ScriptEngines.ini`:

```ini
[ScriptEngines]
LogLevel = "Debug"
EnableScriptDebugging = true
```

### Support

- Documentation: See USER_MANUAL.md Chapter 5
- Issues: https://github.com/opensim/opensim-next/issues
- Forums: OpenSim Next community forums