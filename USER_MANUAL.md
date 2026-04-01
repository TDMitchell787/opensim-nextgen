# OpenSim Next - Complete User Manual & Operations Guide

**Version:** 3.0.0
**Last Updated:** March 2026
**Target Audience:** System Administrators, Virtual World Operators, Developers, Content Creators

---

## Table of Contents

- [Chapter 1: Installation Guide](#chapter-1-installation-guide)
- [Chapter 2: Configuration Guide](#chapter-2-configuration-guide)
- [Chapter 3: Database Setup](#chapter-3-database-setup)
- [Chapter 4: OpenZiti Zero Trust Configuration](#chapter-4-openziti-zero-trust-configuration)
- [Chapter 5: Encrypted Overlay Network Setup](#chapter-5-encrypted-overlay-network-setup)
- [Chapter 6: Multi-Physics Engine Configuration](#chapter-6-multi-physics-engine-configuration)
- [Chapter 7: Client Configuration Guide](#chapter-7-client-configuration-guide)
- [Chapter 8: Region Management and Scaling](#chapter-8-region-management-and-scaling)
- [Chapter 9: WebSocket and Web Client Setup](#chapter-9-websocket-and-web-client-setup)
- [Chapter 10: Monitoring and Administration Setup](#chapter-10-monitoring-and-administration-setup)
- [Chapter 11: Backup and Disaster Recovery](#chapter-11-backup-and-disaster-recovery)
- [Chapter 12: Asset Management and Content Creation Workflows](#chapter-12-asset-management-and-content-creation-workflows)
- [Chapter 13: Security Hardening and Production Deployment Guide](#chapter-13-security-hardening-and-production-deployment-guide)
- [Chapter 14: Troubleshooting Guide](#chapter-14-troubleshooting-guide)
- [Chapter 15: API Reference for Developers and Integrators](#chapter-15-api-reference-for-developers-and-integrators)
- [Chapter 16: Performance Tuning and Optimization Guide](#chapter-16-performance-tuning-and-optimization-guide)
- [Chapter 17: Quick Start Guide](#chapter-17-quick-start-guide)
- [Chapter 22: AI/ML Features and Galadriel AI System](#chapter-22-aiml-features-and-galadriel-ai-system)
- [Chapter 23: Administrator's Addendum — Instance Admin Controller](#chapter-23-administrators-addendum--instance-admin-controller)

---

## Introduction

Welcome to OpenSim Next, the world's most advanced virtual world server platform. This comprehensive manual will guide you through every aspect of installing, configuring, and operating OpenSim Next from simple single-region setups to enterprise-scale multi-region virtual world grids.

### What Makes OpenSim Next Revolutionary

OpenSim Next represents a complete modernization of virtual world server technology with these groundbreaking features:

- **🤖 Galadriel AI Director**: In-world AI NPC that builds structures, generates terrain, creates vehicles, designs clothing, and sets up cinematic camera shots from natural language
- **🔒 Zero Trust Security**: OpenZiti integration with encrypted overlay networks
- **🌐 Web-First Architecture**: Native browser support alongside traditional viewers
- **⚡ Multi-Physics Engines**: 5 physics engines with per-region selection
- **🏗️ Enterprise Scalability**: Production-ready monitoring and auto-scaling
- **🔧 OpenSim Compatibility**: 100% backward compatibility with existing content
- **🚀 Modern Technology Stack**: Rust/Zig hybrid for maximum performance
- **🏔️ Procedural Terrain**: 8 terrain presets with preview-then-approve workflow, multi-region grids
- **🎬 Drone Cinematography**: 8 professional shot types with automated lighting rigs

### Documentation Philosophy

This manual follows a progression from simple to complex:
1. **Get Running Quickly**: Quick start for immediate results
2. **Understand the Basics**: Core concepts and simple configurations
3. **Scale Up**: Multi-region and enterprise deployments
4. **Optimize**: Performance tuning and advanced features
5. **Secure**: Production hardening and zero trust networking

# Chapter 16: Performance Tuning and Optimization Guide

This chapter provides comprehensive performance tuning and optimization strategies for OpenSim Next deployments, covering everything from single-region optimization to enterprise-scale grid performance management.

## 16.1 Performance Monitoring and Baseline Establishment

### 16.1.1 Performance Metrics Collection

OpenSim Next provides comprehensive performance monitoring through multiple channels:

```bash
# Enable detailed performance monitoring
export OPENSIM_PERFORMANCE_MONITORING=true
export OPENSIM_METRICS_COLLECTION_INTERVAL=5
export OPENSIM_PROFILING_ENABLED=true

# Start server with performance monitoring
cargo run --release

# Access performance metrics
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  http://localhost:9100/metrics | grep opensim_performance
```

**Key Performance Indicators (KPIs):**

| Metric Category | Key Metrics | Target Values |
|----------------|-------------|---------------|
| **CPU Performance** | CPU utilization, thread efficiency | < 70% average |
| **Memory Management** | Heap usage, garbage collection | < 80% heap, < 100ms GC |
| **Network Performance** | Latency, throughput, packet loss | < 50ms latency, > 100Mbps |
| **Physics Performance** | Physics FPS, object count | 60+ FPS, 10K+ objects |
| **Database Performance** | Query time, connection pool | < 10ms queries, 95% pool |
| **Avatar Performance** | Avatar count, movement latency | 100+ avatars, < 20ms |

### 16.1.2 Establishing Performance Baselines

```bash
# Run comprehensive performance baseline test
cd rust/tools/performance-testing
./establish-baseline.sh

# Generate baseline report
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  http://localhost:8090/api/performance/baseline-report > baseline-$(date +%Y%m%d).json

# Compare current performance to baseline
./compare-performance.sh baseline-20250701.json
```

**Baseline Configuration Template:**
```toml
# opensim-next/config/performance-baseline.toml
[baseline]
measurement_duration = "10min"
avatar_count = 10
region_count = 4
physics_engine = "ODE"
database_type = "PostgreSQL"

[targets]
cpu_utilization_max = 70.0
memory_utilization_max = 80.0
network_latency_max = 50.0
physics_fps_min = 60.0
database_query_time_max = 10.0
```

## 16.2 System-Level Optimization

### 16.2.1 Operating System Tuning

**Linux Kernel Optimization:**
```bash
# Network performance tuning
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_rmem = 4096 16384 134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_wmem = 4096 65536 134217728' >> /etc/sysctl.conf
echo 'net.core.netdev_max_backlog = 5000' >> /etc/sysctl.conf

# CPU performance tuning
echo 'kernel.sched_migration_cost_ns = 5000000' >> /etc/sysctl.conf
echo 'kernel.sched_autogroup_enabled = 0' >> /etc/sysctl.conf

# Memory management
echo 'vm.swappiness = 1' >> /etc/sysctl.conf
echo 'vm.dirty_ratio = 15' >> /etc/sysctl.conf
echo 'vm.dirty_background_ratio = 5' >> /etc/sysctl.conf

# Apply settings
sysctl -p
```

**CPU Affinity and NUMA Optimization:**
```bash
# Check NUMA topology
numactl --hardware

# Run OpenSim Next with CPU affinity
numactl --cpunodebind=0 --membind=0 cargo run --release

# Advanced CPU pinning for multi-region
taskset -c 0-7 cargo run --bin region1 &
taskset -c 8-15 cargo run --bin region2 &
taskset -c 16-23 cargo run --bin region3 &
```

### 16.2.2 Storage Performance Optimization

**NVMe SSD Configuration:**
```bash
# Optimize I/O scheduler for NVMe
echo noop > /sys/block/nvme0n1/queue/scheduler

# Tune filesystem for performance
mount -o noatime,nodiratime,nobarrier /dev/nvme0n1p1 /opt/opensim-data

# Database storage optimization
# PostgreSQL configuration for NVMe
echo "random_page_cost = 1.1" >> /etc/postgresql/15/main/postgresql.conf
echo "effective_io_concurrency = 200" >> /etc/postgresql/15/main/postgresql.conf
```

**RAID Configuration for High Performance:**
```bash
# RAID 0 for maximum performance (development/testing)
mdadm --create /dev/md0 --level=0 --raid-devices=2 /dev/nvme0n1 /dev/nvme1n1

# RAID 10 for performance + redundancy (production)
mdadm --create /dev/md0 --level=10 --raid-devices=4 \
  /dev/nvme0n1 /dev/nvme1n1 /dev/nvme2n1 /dev/nvme3n1
```

## 16.3 OpenSim Next Application Optimization

### 16.3.1 Rust Runtime Optimization

**Cargo Build Optimization:**
```toml
# Cargo.toml - Production optimization profile
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.release-with-debug]
inherits = "release"
debug = true
strip = false
```

**Runtime Configuration:**
```bash
# Rust runtime tuning
export RUST_MIN_STACK=8388608  # 8MB stack size
export RUST_BACKTRACE=0        # Disable backtraces in production
export RUSTFLAGS="-C target-cpu=native -C target-feature=+crt-static"

# Memory allocator optimization
export MALLOC_CONF="background_thread:true,metadata_thp:auto,dirty_decay_ms:30000"

# Tokio runtime tuning
export TOKIO_WORKER_THREADS=16
export TOKIO_BLOCKING_THREADS=32
```

### 16.3.2 Physics Engine Performance Tuning

**Per-Engine Optimization Settings:**

```rust
// ODE Engine - Optimized for avatar movement
let ode_config = PhysicsEngineConfig {
    timestep: 1.0/90.0,  // 90 FPS for responsive avatars
    max_substeps: 4,
    solver_iterations: 10,
    contact_surface_layer: 0.001,
    contact_max_correcting_vel: 10.0,
    damping: PhysicsDamping {
        linear: 0.01,
        angular: 0.01,
    },
    gravity: Vector3::new(0.0, 0.0, -9.81),
    collision_detection: CollisionDetection::Discrete,
};

// Bullet Engine - Optimized for vehicles and complex dynamics
let bullet_config = PhysicsEngineConfig {
    timestep: 1.0/120.0,  // 120 FPS for smooth vehicles
    max_substeps: 10,
    solver_iterations: 20,
    constraint_solver_type: ConstraintSolverType::Sequential,
    collision_margin: 0.04,
    continuous_collision_detection: true,
    gpu_acceleration: true,
};

// UBODE Engine - Optimized for large worlds
let ubode_config = PhysicsEngineConfig {
    timestep: 1.0/60.0,   // 60 FPS balanced performance
    max_substeps: 6,
    spatial_partitioning: SpatialPartitioning::Octree,
    body_sleeping: true,
    sleep_threshold: 0.5,
    large_world_optimization: true,
    level_of_detail: true,
};
```

**Physics Performance Monitoring:**
```bash
# Monitor physics performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  http://localhost:8090/api/physics/performance

# Per-region physics stats
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  http://localhost:8090/api/regions/stats | jq '.physics_performance'
```

### 16.3.3 Database Performance Optimization

**PostgreSQL Configuration for OpenSim Next:**
```sql
-- postgresql.conf optimization
shared_buffers = '4GB'                    -- 25% of total RAM
effective_cache_size = '12GB'             -- 75% of total RAM
work_mem = '256MB'                        -- Per-operation memory
maintenance_work_mem = '1GB'              -- Maintenance operations
max_connections = 200                     -- Connection limit
checkpoint_segments = 32                  -- WAL checkpointing
checkpoint_completion_target = 0.9        -- Checkpoint spreading
wal_buffers = '16MB'                     -- WAL buffer size
random_page_cost = 1.1                   -- SSD optimization
effective_io_concurrency = 200           -- Concurrent I/O
```

**Database Index Optimization:**
```sql
-- Critical indexes for OpenSim Next performance
CREATE INDEX CONCURRENTLY idx_prims_regionuuid ON prims(RegionUUID);
CREATE INDEX CONCURRENTLY idx_prims_location ON prims(RegionUUID, PositionX, PositionY);
CREATE INDEX CONCURRENTLY idx_assets_id ON assets(id);
CREATE INDEX CONCURRENTLY idx_assets_type ON assets(assetType);
CREATE INDEX CONCURRENTLY idx_inventory_owner ON inventoryitems(avatarID);
CREATE INDEX CONCURRENTLY idx_regions_location ON regions(locX, locY);

-- Spatial indexes for location-based queries
CREATE INDEX CONCURRENTLY idx_prims_spatial 
  ON prims USING gist(box(point(PositionX, PositionY), point(PositionX, PositionY)));
```

**Connection Pool Optimization:**
```rust
// Database connection pool configuration
let pool_config = sqlx::postgres::PgPoolOptions::new()
    .max_connections(100)
    .min_connections(10)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .test_before_acquire(true)
    .connect(&database_url).await?;
```

## 16.4 Network Performance Optimization

### 16.4.1 WebSocket Performance Tuning

**WebSocket Server Configuration:**
```rust
// High-performance WebSocket configuration
let websocket_config = WebSocketConfig {
    max_connections: 5000,
    max_message_size: 1024 * 1024,  // 1MB
    max_frame_size: 256 * 1024,     // 256KB
    heartbeat_interval: Duration::from_secs(30),
    client_timeout: Duration::from_secs(60),
    compression: CompressionConfig {
        enabled: true,
        level: 6,
        window_bits: 15,
    },
    buffer_size: 64 * 1024,         // 64KB buffers
    tcp_nodelay: true,
    tcp_keepalive: Some(Duration::from_secs(30)),
};
```

**Load Balancer Configuration for WebSocket:**
```nginx
# nginx.conf - WebSocket load balancing
upstream opensim_websocket {
    least_conn;
    server 127.0.0.1:9001 weight=3 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:9002 weight=3 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:9003 weight=2 max_fails=3 fail_timeout=30s;
    keepalive 32;
}

server {
    listen 80;
    server_name websocket.opensim.example.com;
    
    location /ws {
        proxy_pass http://opensim_websocket;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket-specific timeouts
        proxy_connect_timeout 4s;
        proxy_send_timeout 12s;
        proxy_read_timeout 12s;
        
        # Buffer configuration
        proxy_buffering off;
        proxy_buffer_size 4k;
    }
}
```

### 16.4.2 Second Life Viewer Protocol Optimization

**LLSD Protocol Performance:**
```rust
// LLSD serialization optimization
let llsd_config = LLSDConfig {
    compression: true,
    binary_format: true,
    max_depth: 32,
    max_array_size: 10000,
    string_interning: true,
    buffer_pool_size: 1000,
};

// Capability URL optimization
let capability_config = CapabilityConfig {
    cache_size: 10000,
    cache_ttl: Duration::from_secs(300),
    compression: true,
    batch_requests: true,
    max_batch_size: 100,
};
```

## 16.5 Multi-Region Performance Scaling

### 16.5.1 Region Distribution Strategies

**Geographic Distribution:**
```rust
// Region placement optimization
let region_placement = RegionPlacementStrategy {
    strategy: PlacementStrategy::Geographic,
    load_balancing: LoadBalancingStrategy::LeastConnections,
    affinity_rules: vec![
        AffinityRule::SameDatacenter,
        AffinityRule::NetworkProximity,
    ],
    anti_affinity_rules: vec![
        AntiAffinityRule::DifferentPhysicalHosts,
    ],
};
```

**Load-Based Region Migration:**
```bash
# Monitor region performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  http://localhost:8090/api/regions/performance-stats

# Trigger region migration based on load
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/regions/migrate \
  -d '{
    "region_id": "region-uuid",
    "target_server": "server-2",
    "migration_strategy": "live_migration"
  }'
```

### 16.5.2 Cross-Region Communication Optimization

**Encrypted Overlay Network Tuning:**
```rust
// Zero trust network optimization
let overlay_config = OverlayNetworkConfig {
    topology: NetworkTopology::Mesh,
    encryption: EncryptionConfig {
        algorithm: EncryptionAlgorithm::Aes256Gcm,
        key_rotation_interval: Duration::from_secs(3600),
        forward_secrecy: true,
    },
    compression: CompressionConfig {
        enabled: true,
        algorithm: CompressionAlgorithm::Lz4,
        level: 3,
    },
    routing: RoutingConfig {
        algorithm: RoutingAlgorithm::ShortestPath,
        load_aware: true,
        failover_timeout: Duration::from_secs(5),
    },
    buffer_sizes: BufferConfig {
        send_buffer: 256 * 1024,
        receive_buffer: 256 * 1024,
        message_queue: 10000,
    },
};
```

## 16.6 Caching and Asset Optimization

### 16.6.1 Multi-Tier Caching Strategy

**Cache Configuration:**
```rust
// Multi-level cache hierarchy
let cache_config = CacheConfig {
    l1_cache: L1CacheConfig {
        type_: CacheType::Memory,
        size: CacheSize::Megabytes(512),
        eviction: EvictionPolicy::Lru,
        ttl: Duration::from_secs(300),
    },
    l2_cache: L2CacheConfig {
        type_: CacheType::Redis,
        size: CacheSize::Gigabytes(4),
        eviction: EvictionPolicy::Lfu,
        ttl: Duration::from_secs(3600),
        cluster_mode: true,
    },
    l3_cache: L3CacheConfig {
        type_: CacheType::Filesystem,
        size: CacheSize::Gigabytes(100),
        eviction: EvictionPolicy::Ttl,
        ttl: Duration::from_secs(86400),
        compression: true,
    },
};
```

**Asset Delivery Optimization:**
```bash
# Configure CDN for asset delivery
export OPENSIM_CDN_ENABLED=true
export OPENSIM_CDN_PROVIDER="cloudflare"
export OPENSIM_CDN_ENDPOINT="https://assets.opensim.example.com"

# Asset compression settings
export OPENSIM_ASSET_COMPRESSION=true
export OPENSIM_ASSET_COMPRESSION_LEVEL=6
export OPENSIM_ASSET_CACHE_SIZE=10GB
```

### 16.6.2 Texture and Mesh Optimization

**Texture Optimization Pipeline:**
```rust
// Automatic texture optimization
let texture_optimizer = TextureOptimizer {
    quality_levels: vec![
        QualityLevel::Ultra,    // Original quality
        QualityLevel::High,     // 80% quality
        QualityLevel::Medium,   // 60% quality
        QualityLevel::Low,      // 40% quality
    ],
    formats: vec![
        TextureFormat::Jpeg2000,
        TextureFormat::WebP,
        TextureFormat::Jpeg,
    ],
    automatic_scaling: true,
    progressive_download: true,
    level_of_detail: true,
};
```

## 16.7 Monitoring and Alerting

### 16.7.1 Performance Alert Configuration

**Prometheus Alert Rules:**
```yaml
# prometheus-alerts.yml
groups:
  - name: opensim_performance
    rules:
      - alert: HighCPUUsage
        expr: opensim_cpu_usage_percent > 80
        for: 2m
        annotations:
          summary: "OpenSim Next high CPU usage"
          description: "CPU usage is above 80% for more than 2 minutes"
      
      - alert: HighMemoryUsage
        expr: opensim_memory_usage_percent > 85
        for: 1m
        annotations:
          summary: "OpenSim Next high memory usage"
          description: "Memory usage is above 85%"
      
      - alert: PhysicsPerformanceDegradation
        expr: opensim_physics_fps < 50
        for: 30s
        annotations:
          summary: "Physics performance degradation"
          description: "Physics FPS dropped below 50"
      
      - alert: DatabaseSlowQueries
        expr: opensim_database_query_duration_seconds > 0.1
        for: 1m
        annotations:
          summary: "Database slow queries detected"
          description: "Database queries taking longer than 100ms"
```

### 16.7.2 Performance Dashboard

**Grafana Dashboard Configuration:**
```json
{
  "dashboard": {
    "title": "OpenSim Next Performance Dashboard",
    "panels": [
      {
        "title": "CPU and Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "opensim_cpu_usage_percent",
            "legendFormat": "CPU Usage %"
          },
          {
            "expr": "opensim_memory_usage_percent",
            "legendFormat": "Memory Usage %"
          }
        ]
      },
      {
        "title": "Physics Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "opensim_physics_fps",
            "legendFormat": "Physics FPS"
          },
          {
            "expr": "opensim_physics_objects_count",
            "legendFormat": "Physics Objects"
          }
        ]
      },
      {
        "title": "Network Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "opensim_network_latency_seconds",
            "legendFormat": "Network Latency"
          },
          {
            "expr": "opensim_websocket_connections",
            "legendFormat": "WebSocket Connections"
          }
        ]
      }
    ]
  }
}
```

## 16.8 Advanced Performance Optimization

### 16.8.1 GPU Acceleration

**CUDA Integration for Physics:**
```rust
// GPU-accelerated physics configuration
let gpu_physics_config = GpuPhysicsConfig {
    enabled: true,
    device_id: 0,
    memory_allocation: GpuMemoryAllocation::Gigabytes(2),
    compute_capability: ComputeCapability::Sm75,
    max_threads_per_block: 1024,
    max_blocks_per_grid: 65535,
    optimization_level: OptimizationLevel::Aggressive,
};

// Enable GPU acceleration for specific physics engines
let bullet_gpu_config = BulletGpuConfig {
    gpu_broadphase: true,
    gpu_solver: true,
    gpu_collision_detection: true,
    memory_pool_size: GpuMemorySize::Gigabytes(1),
};
```

### 16.8.2 SIMD Optimization

**Vectorized Operations:**
```rust
// SIMD-optimized vector operations
use std::simd::f32x4;

fn simd_vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    let chunks_a = a.chunks_exact(4);
    let chunks_b = b.chunks_exact(4);
    let chunks_result = result.chunks_exact_mut(4);
    
    for ((chunk_a, chunk_b), chunk_result) in 
        chunks_a.zip(chunks_b).zip(chunks_result) {
        let va = f32x4::from_slice(chunk_a);
        let vb = f32x4::from_slice(chunk_b);
        let vr = va + vb;
        chunk_result.copy_from_slice(vr.as_array());
    }
}
```

### 16.8.3 Lock-Free Data Structures

**High-Performance Concurrent Collections:**
```rust
use crossbeam::queue::SegQueue;
use dashmap::DashMap;

// Lock-free message queue for inter-region communication
let message_queue: Arc<SegQueue<RegionMessage>> = Arc::new(SegQueue::new());

// Lock-free hash map for avatar tracking
let avatar_registry: Arc<DashMap<Uuid, AvatarState>> = Arc::new(DashMap::new());

// Atomic counters for performance metrics
use std::sync::atomic::{AtomicU64, Ordering};
let connection_counter = Arc::new(AtomicU64::new(0));
let message_counter = Arc::new(AtomicU64::new(0));
```

## 16.9 Performance Testing and Benchmarking

### 16.9.1 Automated Performance Testing

**Performance Test Suite:**
```bash
#!/bin/bash
# performance-test-suite.sh

echo "🚀 Starting OpenSim Next Performance Test Suite"

# Load test configuration
source ./config/performance-test-config.sh

# 1. CPU and Memory Stress Test
echo "📊 Running CPU and Memory stress test..."
./tests/cpu-memory-stress-test.sh

# 2. Physics Engine Performance Test
echo "⚡ Testing physics engine performance..."
./tests/physics-performance-test.sh

# 3. Network Performance Test
echo "🌐 Testing network performance..."
./tests/network-performance-test.sh

# 4. Database Performance Test
echo "🗄️ Testing database performance..."
./tests/database-performance-test.sh

# 5. Multi-Region Scaling Test
echo "🏗️ Testing multi-region scaling..."
./tests/multi-region-scaling-test.sh

# 6. WebSocket Load Test
echo "📡 Testing WebSocket load handling..."
./tests/websocket-load-test.sh

# Generate performance report
echo "📋 Generating performance report..."
./tools/generate-performance-report.sh

echo "✅ Performance test suite completed"
```

### 16.9.2 Continuous Performance Monitoring

**CI/CD Performance Integration:**
```yaml
# .github/workflows/performance-ci.yml
name: Performance CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  performance-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Performance Testing Environment
        run: |
          ./scripts/setup-performance-env.sh
          
      - name: Run Performance Benchmarks
        run: |
          cargo bench --bench performance_suite
          
      - name: Performance Regression Check
        run: |
          ./tools/performance-regression-check.sh
          
      - name: Upload Performance Results
        uses: actions/upload-artifact@v3
        with:
          name: performance-results
          path: target/criterion/
```

## 16.10 Production Performance Best Practices

### 16.10.1 Deployment Optimization Checklist

**Pre-Deployment Performance Checklist:**
- [ ] Operating system tuned for network and CPU performance
- [ ] Database indexes optimized for OpenSim Next query patterns
- [ ] Caching strategy implemented and configured
- [ ] Physics engines selected and tuned for region content
- [ ] Monitoring and alerting configured
- [ ] Load balancing configured for expected traffic
- [ ] Backup strategy optimized for minimal performance impact
- [ ] Security measures implemented without performance degradation

### 16.10.2 Capacity Planning

**Resource Estimation Guidelines:**

| Users per Region | CPU Cores | RAM (GB) | Network (Mbps) | Storage (GB/day) |
|-----------------|-----------|----------|----------------|------------------|
| 1-10 | 2-4 | 4-8 | 10-50 | 1-5 |
| 11-25 | 4-8 | 8-16 | 50-100 | 5-15 |
| 26-50 | 8-16 | 16-32 | 100-250 | 15-30 |
| 51-100 | 16-32 | 32-64 | 250-500 | 30-60 |
| 100+ | 32+ | 64+ | 500+ | 60+ |

### 16.10.3 Performance Troubleshooting Workflow

**Systematic Performance Issue Resolution:**

1. **Identify Performance Issue**
   ```bash
   # Check overall system health
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     http://localhost:8090/api/health
   
   # Review performance metrics
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     http://localhost:9100/metrics | grep -E "(cpu|memory|latency)"
   ```

2. **Isolate Problem Component**
   ```bash
   # Check database performance
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     http://localhost:8090/api/database/performance
   
   # Check physics performance per region
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     http://localhost:8090/api/regions/physics/performance
   
   # Check network performance
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     http://localhost:8090/api/network/performance
   ```

3. **Apply Targeted Optimization**
   ```bash
   # Adjust physics engine settings
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     -X POST http://localhost:8090/api/regions/region-id/physics/config \
     -d '{"timestep": 0.0167, "solver_iterations": 10}'
   
   # Adjust cache settings
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     -X POST http://localhost:8090/api/cache/config \
     -d '{"size_mb": 1024, "eviction_policy": "lru"}'
   ```

4. **Verify Performance Improvement**
   ```bash
   # Run performance comparison
   ./tools/performance-before-after-comparison.sh
   
   # Generate improvement report
   ./tools/generate-optimization-report.sh
   ```

This comprehensive performance tuning guide provides the foundation for optimizing OpenSim Next deployments from single-region installations to enterprise-scale grids, ensuring maximum performance, scalability, and user experience.

---

# Chapter 1: Installation Guide

This chapter provides comprehensive installation instructions for OpenSim Next across all supported platforms, with detailed system requirements and dependency management.

## 1.1 System Requirements

### Minimum Requirements (Single Region)

| Component | Specification |
|-----------|---------------|
| **CPU** | 4 cores, 2.4 GHz (x86_64 or ARM64) |
| **RAM** | 8 GB |
| **Storage** | 50 GB SSD |
| **Network** | 100 Mbps connection |
| **OS** | Linux, macOS, Windows 10+ |

### Recommended Requirements (Multi-Region Production)

| Component | Specification |
|-----------|---------------|
| **CPU** | 16+ cores, 3.0+ GHz (x86_64) |
| **RAM** | 32+ GB |
| **Storage** | 500+ GB NVMe SSD |
| **Network** | 1+ Gbps dedicated connection |
| **OS** | Ubuntu 22.04 LTS or CentOS Stream 9 |

### Enterprise Requirements (Large Grid)

| Component | Specification |
|-----------|---------------|
| **CPU** | 32+ cores, 3.5+ GHz |
| **RAM** | 128+ GB |
| **Storage** | 2+ TB NVMe SSD RAID |
| **Network** | 10+ Gbps with redundancy |
| **Database** | Dedicated PostgreSQL cluster |
| **Load Balancer** | Hardware or cloud load balancer |

## 1.2 Supported Operating Systems

### Linux Distributions (Recommended)

- **Ubuntu 22.04 LTS** ✅ Primary target
- **Ubuntu 20.04 LTS** ✅ Supported
- **CentOS Stream 9** ✅ Enterprise choice
- **Red Hat Enterprise Linux 9** ✅ Enterprise
- **Debian 12** ✅ Stable
- **Fedora 38+** ✅ Latest features
- **Arch Linux** ⚠️ Advanced users only

### macOS Support

- **macOS 12 Monterey** ✅ Intel and Apple Silicon
- **macOS 13 Ventura** ✅ Intel and Apple Silicon
- **macOS 14 Sonoma** ✅ Intel and Apple Silicon

### Windows Support

- **Windows 11** ✅ Professional or Enterprise
- **Windows 10** ✅ Version 1909 or later
- **Windows Server 2022** ✅ Enterprise deployments
- **Windows Server 2019** ✅ Legacy enterprise

## 1.3 Prerequisites and Dependencies

### Core Development Tools

#### Linux (Ubuntu/Debian)
```bash
# Update package manager
sudo apt update && sudo apt upgrade -y

# Install essential build tools
sudo apt install -y build-essential curl git cmake pkg-config \
  libssl-dev libpq-dev sqlite3 libsqlite3-dev redis-server

# Install Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Zig (0.15.x required)
wget https://ziglang.org/download/0.15.2/zig-linux-x86_64-0.15.2.tar.xz
tar -xf zig-linux-x86_64-0.15.2.tar.xz
sudo mv zig-linux-x86_64-0.15.2 /opt/zig
echo 'export PATH="/opt/zig:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### Linux (CentOS/RHEL)
```bash
# Enable EPEL repository
sudo dnf install -y epel-release

# Install development tools
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y curl git cmake pkg-config openssl-devel \
  postgresql-devel sqlite-devel redis

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Zig (0.15.x required)
wget https://ziglang.org/download/0.15.2/zig-linux-x86_64-0.15.2.tar.xz
tar -xf zig-linux-x86_64-0.15.2.tar.xz
sudo mv zig-linux-x86_64-0.15.2 /opt/zig
echo 'export PATH="/opt/zig:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install curl git cmake pkg-config openssl postgresql sqlite redis zig

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Windows
```powershell
# Install Chocolatey package manager
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Install dependencies
choco install -y git cmake pkgconfiglite openssl postgresql sqlite redis zig

# Install Rust
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
.\rustup-init.exe -y
$env:PATH += ";$env:USERPROFILE\.cargo\bin"
```

### Optional Dependencies

#### Graphics and Media Libraries (For Advanced Features)
```bash
# Linux
sudo apt install -y libgl1-mesa-dev libglu1-mesa-dev \
  libopenal-dev libvorbis-dev libfreetype6-dev

# macOS
brew install freetype libvorbis openal-soft

# Windows
choco install -y openal freetype
```

#### OpenZiti SDK (For Zero Trust Networking)
```bash
# Linux/macOS
curl -sSfL https://get.openziti.io/install.sh | sudo bash

# Windows (PowerShell as Administrator)
iwr https://get.openziti.io/install.ps1 | iex
```

## 1.4 Source Code Installation

### Method 1: Git Clone (Recommended for Development)

```bash
# Clone the repository
git clone https://github.com/opensim/opensim-next.git
cd opensim-next

# Verify submodules are initialized
git submodule update --init --recursive

# Check out the latest stable release
git checkout main
```

### Method 2: Release Package Download

```bash
# Download latest stable release
curl -L https://github.com/opensim/opensim-next/releases/latest/download/opensim-next-source.tar.gz -o opensim-next.tar.gz

# Extract the archive
tar -xzf opensim-next.tar.gz
cd opensim-next
```

## 1.5 Compilation Process

### Tested Versions

| Component | Version |
|-----------|---------|
| **Rust** | 1.93.1 or later |
| **Zig** | 0.15.2 |
| **PostgreSQL** | 14+ (production) |
| **SQLite** | 3.x (development) |

### Compile Zig Components (Physics Engine)

```bash
# From the opensim-next project root
cd zig
zig build

# Verify Zig build — should produce libopensim_physics.dylib (macOS) or .so (Linux)
ls -la zig-out/lib/
cd ..
```

### Compile Rust Components

```bash
# From the opensim-next project root (NOT the rust/ subdirectory)
cd opensim-next

# Release build (recommended)
cargo build --release --bin opensim-next

# The binary is created at the WORKSPACE root:
ls -la target/release/opensim-next
```

**Important:** The build command must be run from the `opensim-next/` workspace root directory, not from `opensim-next/rust/`. The `--bin opensim-next` flag is required.

### Verify Installation

```bash
# Verify binary exists and is fresh
ls -la target/release/opensim-next

# Or use the verification script
./verify_build.sh
```

### Runtime Library Path (macOS)

On macOS, the Zig physics library and BulletSim must be on the dynamic library path:

```bash
export DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64
```

On Linux, use `LD_LIBRARY_PATH` instead:

```bash
export LD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64
```

Without this, the server will fail to load the physics engine at startup.

## 1.6 Platform-Specific Installation Notes

### Ubuntu 22.04 LTS (Recommended Linux Distribution)

```bash
# Complete installation script for Ubuntu 22.04
#!/bin/bash
set -e

echo "Installing OpenSim Next on Ubuntu 22.04..."

# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y build-essential curl git cmake pkg-config \
  libssl-dev libpq-dev sqlite3 libsqlite3-dev redis-server \
  nginx certbot python3-certbot-nginx

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install Zig
ZIG_VERSION="0.15.2"
wget "https://ziglang.org/download/${ZIG_VERSION}/zig-linux-x86_64-${ZIG_VERSION}.tar.xz"
tar -xf "zig-linux-x86_64-${ZIG_VERSION}.tar.xz"
sudo mv "zig-linux-x86_64-${ZIG_VERSION}" /opt/zig
echo 'export PATH="/opt/zig:$PATH"' >> ~/.bashrc

# Clone and build OpenSim Next
git clone https://github.com/opensim/opensim-next.git
cd opensim-next

# Build Zig physics components
cd zig
/opt/zig/zig build
cd ..

# Build Rust server (from workspace root)
cargo build --release --bin opensim-next

echo "OpenSim Next installation completed successfully!"
echo "Binary location: $(pwd)/target/release/opensim-next"
```

### macOS (Universal Binary)

```bash
# macOS-specific build script
#!/bin/bash
set -e

echo "Installing OpenSim Next on macOS..."

# Install Xcode Command Line Tools if not present
if ! command -v git &> /dev/null; then
    xcode-select --install
    echo "Please install Xcode Command Line Tools and re-run this script"
    exit 1
fi

# Install Homebrew if not present
if ! command -v brew &> /dev/null; then
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Install dependencies
brew install curl git cmake pkg-config openssl postgresql sqlite redis zig

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Add ARM64 target for Apple Silicon Macs
if [[ $(uname -m) == "arm64" ]]; then
    rustup target add aarch64-apple-darwin
    rustup target add x86_64-apple-darwin
fi

# Clone and build
git clone https://github.com/opensim/opensim-next.git
cd opensim-next

# Build Zig physics components
cd zig && zig build
cd ..

# Build Rust server (from workspace root)
cargo build --release --bin opensim-next

echo "OpenSim Next installation completed for macOS!"
echo "Binary location: $(pwd)/target/release/opensim-next"
echo ""
echo "Run with:"
echo "  RUST_LOG=info DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 ./target/release/opensim-next"
```

### Windows 11 (PowerShell)

```powershell
# Windows installation script
# Run as Administrator in PowerShell

Write-Host "Installing OpenSim Next on Windows..." -ForegroundColor Green

# Install Chocolatey
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Refresh environment
refreshenv

# Install dependencies
choco install -y git cmake pkgconfiglite openssl postgresql sqlite redis zig visualstudio2022buildtools

# Install Rust
$env:RUSTUP_INIT_SKIP_PATH_CHECK = "yes"
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
.\rustup-init.exe -y --default-toolchain stable
Remove-Item rustup-init.exe

# Add Rust to PATH for current session
$env:PATH += ";$env:USERPROFILE\.cargo\bin"

# Clone repository
git clone https://github.com/opensim/opensim-next.git
Set-Location opensim-next

# Build Zig physics components
Set-Location zig
zig build
Set-Location ..

# Build Rust server (from workspace root)
cargo build --release --bin opensim-next

Write-Host "OpenSim Next installation completed for Windows!" -ForegroundColor Green
Write-Host "Binary location: $(Get-Location)\target\release\opensim-next.exe" -ForegroundColor Yellow
```

## 1.7 Docker Installation (Alternative Method)

### Pre-built Docker Image

```bash
# Pull the official OpenSim Next Docker image
docker pull opensim/opensim-next:latest

# Run a single-region development instance
docker run -d \
  --name opensim-next-dev \
  -p 9000:9000 \
  -p 9001:9001 \
  -p 8080:8080 \
  -p 9100:9100 \
  -e OPENSIM_MODE="development" \
  -e OPENSIM_REGION_NAME="Development Region" \
  -v opensim-data:/data \
  opensim/opensim-next:latest

# Check logs
docker logs -f opensim-next-dev
```

### Build Custom Docker Image

```dockerfile
# Dockerfile for custom OpenSim Next build
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential curl git cmake pkg-config \
    libssl-dev libpq-dev sqlite3 libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Zig
RUN wget https://ziglang.org/download/0.15.2/zig-linux-x86_64-0.15.2.tar.xz \
    && tar -xf zig-linux-x86_64-0.15.2.tar.xz \
    && mv zig-linux-x86_64-0.15.2 /opt/zig \
    && rm zig-linux-x86_64-0.15.2.tar.xz
ENV PATH="/opt/zig:${PATH}"

# Copy source code
COPY . /opensim-next
WORKDIR /opensim-next

# Build application
RUN cd zig && zig build
RUN cargo build --release --bin opensim-next

# Create runtime user
RUN useradd -m -s /bin/bash opensim

# Copy binary and set permissions
RUN cp target/release/opensim-next /usr/local/bin/ \
    && chown opensim:opensim /usr/local/bin/opensim-next \
    && chmod +x /usr/local/bin/opensim-next

# Switch to runtime user
USER opensim
WORKDIR /home/opensim

# Expose ports
EXPOSE 9000 9001 8080 9100

# Start OpenSim Next
CMD ["opensim-next", "--config", "/data/config/opensim.ini"]
```

## 1.8 Post-Installation Verification

### System Health Check

```bash
# Run comprehensive system check
./target/release/opensim-next --health-check

# Expected output:
# ✅ Rust runtime: OK
# ✅ Zig FFI: OK  
# ✅ Database connectivity: OK
# ✅ Network interfaces: OK
# ✅ OpenZiti SDK: OK
# ✅ Physics engines: 5 available
# ✅ Memory: 8.2 GB available
# ✅ Storage: 45.3 GB available
```

### Component Verification

```bash
# Verify the binary runs (should print version/help)
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next --help

# Verify Zig physics library is loadable
ls -la zig/zig-out/lib/libopensim_physics.*

# Verify BulletSim library exists
ls -la bin/lib64/libBulletSim*

# Run Rust unit tests
cargo test --release --bin opensim-next
```

### Performance Baseline

```bash
# Run unit tests including terrain generator benchmarks
cargo test --release -- terrain

# Example output:
# Physics Engine Performance:
# - ODE: 10,000 bodies @ 60 FPS
# - Bullet: 50,000 bodies @ 60 FPS
# - POS: 100,000 particles @ 60 FPS
# 
# Network Performance:
# - TCP throughput: 980 Mbps
# - WebSocket connections: 1,000 concurrent
# - Region communication: <5ms latency
```

## 1.9 Common Installation Issues

### Issue: Rust Compilation Fails

**Symptoms:**
```
error: linking with `cc` failed: exit status: 1
```

**Solution:**
```bash
# Ensure all build tools are installed
sudo apt install build-essential gcc g++ libc6-dev

# Update Rust to latest stable
rustup update stable

# Clean and rebuild
cargo clean
cargo build --release
```

### Issue: Zig Build Fails

**Symptoms:**
```
error: FileNotFound
```

**Solution:**
```bash
# Verify Zig installation
zig version

# Ensure Zig is in PATH
export PATH="/opt/zig:$PATH"

# Clean and rebuild Zig components
cd zig
rm -rf .zig-cache zig-out
zig build
```

### Issue: Missing Dependencies

**Symptoms:**
```
error: failed to run custom build command for `openssl-sys`
```

**Solution:**
```bash
# Install OpenSSL development headers
# Ubuntu/Debian:
sudo apt install libssl-dev pkg-config

# CentOS/RHEL:
sudo dnf install openssl-devel pkgconf

# macOS:
brew install openssl pkg-config
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"
```

### Issue: Permission Denied

**Symptoms:**
```
Permission denied (os error 13)
```

**Solution:**
```bash
# Fix file permissions
chmod +x target/release/opensim-next

# For system-wide installation
sudo cp target/release/opensim-next /usr/local/bin/
sudo chmod +x /usr/local/bin/opensim-next
```

## 1.10 Next Steps

After successful installation, proceed to:

1. **Chapter 2: Configuration Guide** - Set up your first region
2. **Chapter 3: Database Setup** - Configure data persistence  
3. **Chapter 17: Quick Start Guide** - Get running in 15 minutes

---

**Installation Complete!** 🎉

You now have OpenSim Next successfully installed on your system. The next chapter will guide you through configuring your first virtual world region.

---

# Chapter 2: Configuration Guide

This chapter provides comprehensive configuration instructions for OpenSim Next, covering everything from basic single-region setups to complex multi-region enterprise deployments.

## 2.1 Configuration File Structure

OpenSim Next uses a hierarchical configuration system that's fully compatible with legacy OpenSimulator installations while adding powerful new features.

### Configuration File Hierarchy

```
opensim-next/
├── config/
│   ├── opensim.ini                    # Main configuration
│   ├── config-include/
│   │   ├── GridCommon.ini             # Grid-wide settings
│   │   ├── StandaloneCommon.ini       # Standalone mode settings
│   │   ├── FlotsamCache.ini           # Asset caching configuration
│   │   ├── OpenSimDefaults.ini        # System defaults (do not modify)
│   │   └── opensim-next/
│   │       ├── Physics.ini            # Multi-physics engine settings
│   │       ├── ZeroTrust.ini          # OpenZiti configuration
│   │       ├── WebSockets.ini         # Web client settings
│   │       └── Monitoring.ini         # Observability settings
│   ├── Regions/
│   │   ├── RegionConfig.ini           # Individual region definitions
│   │   └── Estates/
│   │       └── DefaultEstate.ini      # Estate settings
│   └── scripts/
│       ├── startup.sh                 # Startup scripts
│       └── backup.sh                  # Backup scripts
```

### Configuration Loading Order

1. **OpenSimDefaults.ini** - System defaults (never modify)
2. **opensim.ini** - Main configuration file
3. **config-include/*.ini** - Feature-specific configurations
4. **Regions/*.ini** - Region-specific overrides
5. **Environment variables** - Runtime overrides

## 2.2 Basic Configuration (opensim.ini)

### Minimal Configuration for Development

Create `config/opensim.ini`:

```ini
[Startup]
; OpenSim Next - Basic Development Configuration
; Compatible with OpenSimulator 0.9.3+ configurations

; === Core Settings ===
physics = ODE
meshing = ubODEMeshmerizer
permissionmodules = DefaultPermissionsModule

; === Network Configuration ===
http_listener_port = 9000
region_console_port = 0

; === Database Configuration ===
; SQLite for development (change to PostgreSQL for production)
ConnectionString = "Data Source=opensim.db;Version=3;New=True;"
StorageProvider = OpenSim.Data.SQLite.dll

; === Asset Storage ===
AssetConnectionString = ${ConnectionString}
AssetStorageProvider = ${StorageProvider}

; === Inventory Storage ===
InventoryConnectionString = ${ConnectionString}
InventoryStorageProvider = ${StorageProvider}

; === User Account Storage ===
UserAccountConnectionString = ${ConnectionString}
UserAccountStorageProvider = ${StorageProvider}

; === Authentication ===
AuthenticationConnectionString = ${ConnectionString}
AuthenticationStorageProvider = ${StorageProvider}

; === Grid Services ===
GridInfoConnectionString = ${ConnectionString}
GridInfoStorageProvider = ${StorageProvider}

; === Presence Service ===
PresenceConnectionString = ${ConnectionString}
PresenceStorageProvider = ${StorageProvider}

; === Friends Service ===
FriendsConnectionString = ${ConnectionString}
FriendsStorageProvider = ${StorageProvider}

; === Avatar Service ===
AvatarConnectionString = ${ConnectionString}
AvatarStorageProvider = ${StorageProvider}

[Network]
; === OpenSim Next Network Features ===
; Enable WebSocket support for browser clients
enable_websockets = true
websocket_port = 9001

; Enable web client interface
enable_web_client = true
web_client_port = 8080

; Enable monitoring endpoints
enable_monitoring = true
monitoring_port = 9100

; Enable admin dashboard
enable_admin_dashboard = true
admin_dashboard_port = 8090

[Architecture]
; === OpenSim Next Advanced Features ===
; Multi-physics engine support
physics_engine_selection = per_region
default_physics_engine = ODE

; Zero trust networking
enable_zero_trust = false
zero_trust_config = config-include/opensim-next/ZeroTrust.ini

; Encrypted overlay network
enable_overlay_network = false
overlay_network_config = config-include/opensim-next/OverlayNetwork.ini

[Hypergrid]
; === Hypergrid Configuration ===
; Disable for initial setup, enable for inter-grid communication
Enabled = false

[Modules]
; === Module Configuration ===
; Include OpenSim Next enhanced modules
Include-Architecture = "config-include/opensim-next/Physics.ini"
Include-WebSocket = "config-include/opensim-next/WebSockets.ini"
Include-Monitoring = "config-include/opensim-next/Monitoring.ini"

[Economy]
; === Economy Settings ===
; Basic economy module (extend for custom currencies)
EconomyModule = BareBonesEconomy
PriceEnergyUnit = 100
PriceObjectClaim = 10
PricePublicObjectDecay = 4
PricePublicObjectDelete = 4
PriceParcelClaim = 1
PriceParcelClaimFactor = 1
PriceUpload = 0
PriceRentLight = 5
TeleportMinPrice = 2
TeleportPriceExponent = 2
EnergyEfficiency = 1
PriceGroupCreate = -1
```

### Production Configuration

For production deployments, create `config/opensim-production.ini`:

```ini
[Startup]
; OpenSim Next - Production Configuration
; High-performance, secure, scalable deployment

; === Core Settings ===
physics = UBODE
meshing = ubODEMeshmerizer
permissionmodules = DefaultPermissionsModule

; === Network Configuration ===
http_listener_port = 9000
region_console_port = 0

; Enable all OpenSim Next features
enable_all_features = true

; === Database Configuration (PostgreSQL) ===
ConnectionString = "Host=localhost;Database=opensim;Username=opensim;Password=${OPENSIM_DB_PASSWORD}"
StorageProvider = OpenSim.Data.PGSQL.dll

; === Asset Storage (Production) ===
AssetConnectionString = ${ConnectionString}
AssetStorageProvider = ${StorageProvider}

; === All Services Use PostgreSQL ===
InventoryConnectionString = ${ConnectionString}
InventoryStorageProvider = ${StorageProvider}
UserAccountConnectionString = ${ConnectionString}
UserAccountStorageProvider = ${StorageProvider}
AuthenticationConnectionString = ${ConnectionString}
AuthenticationStorageProvider = ${StorageProvider}
GridInfoConnectionString = ${ConnectionString}
GridInfoStorageProvider = ${StorageProvider}
PresenceConnectionString = ${ConnectionString}
PresenceStorageProvider = ${StorageProvider}
FriendsConnectionString = ${ConnectionString}
FriendsStorageProvider = ${StorageProvider}
AvatarConnectionString = ${ConnectionString}
AvatarStorageProvider = ${StorageProvider}

[Network]
; === Production Network Configuration ===
; WebSocket support
enable_websockets = true
websocket_port = 9001
websocket_max_connections = 5000
websocket_rate_limit = 100

; Web client
enable_web_client = true
web_client_port = 8080

; Monitoring (Prometheus compatible)
enable_monitoring = true
monitoring_port = 9100

; Admin dashboard with authentication
enable_admin_dashboard = true
admin_dashboard_port = 8090
admin_api_key = ${OPENSIM_API_KEY}

; SSL/TLS Configuration
enable_ssl = true
ssl_cert_path = /etc/ssl/certs/opensim.crt
ssl_key_path = /etc/ssl/private/opensim.key

[Architecture]
; === Production Architecture ===
; Multi-physics engine support
physics_engine_selection = per_region
default_physics_engine = UBODE

; Zero trust networking enabled
enable_zero_trust = true
zero_trust_config = config-include/opensim-next/ZeroTrust.ini

; Encrypted overlay network enabled
enable_overlay_network = true
overlay_network_config = config-include/opensim-next/OverlayNetwork.ini

; Load balancing
enable_load_balancing = true
load_balancer_strategy = least_connections

; Auto-scaling
enable_auto_scaling = true
auto_scaling_config = config-include/opensim-next/AutoScaling.ini

[Performance]
; === Production Performance Settings ===
; Thread pool configuration
min_threads = 50
max_threads = 500

; Physics timestep optimization
physics_timestep = 0.0167  ; 60 FPS

; Garbage collection tuning
gc_server_mode = true
gc_concurrent = true

; Asset caching (production settings)
asset_cache_size = 2048  ; MB
asset_cache_timeout = 3600  ; seconds

[Security]
; === Production Security ===
; Rate limiting
enable_rate_limiting = true
max_requests_per_second = 100
max_connections_per_ip = 10

; DDoS protection
enable_ddos_protection = true
ddos_threshold = 1000

; Authentication hardening
password_minimum_length = 12
require_strong_passwords = true
enable_two_factor_auth = true

[Logging]
; === Production Logging ===
LogLevel = INFO
FileAppender = true
LogFile = logs/opensim.log
MaxFileSize = 100MB
MaxFiles = 10

; Structured logging (JSON format)
StructuredLogging = true
LogFormat = JSON

; Remote logging (optional)
RemoteLogging = false
LogEndpoint = https://logs.example.com/opensim

[Monitoring]
; === Production Monitoring ===
; Prometheus metrics
PrometheusEnabled = true
PrometheusPort = 9100

; Health checks
HealthCheckEnabled = true
HealthCheckInterval = 30

; Performance metrics
CollectPerformanceMetrics = true
MetricsRetention = 7d

; Alerting
AlertingEnabled = true
AlertWebhookURL = https://alerts.example.com/webhook
```

## 2.3 Region Configuration

### Basic Region Setup

Create `config/Regions/RegionConfig.ini`:

```ini
[Region-Development]
; === Basic Region Configuration ===
RegionUUID = 11111111-1111-1111-1111-111111111111
Location = 1000,1000
InternalAddress = 0.0.0.0
InternalPort = 9000
AllowAlternatePorts = false
ExternalHostName = localhost

; === Region Details ===
RegionName = Development Region
RegionType = Mainland
MasterAvatarFirstName = Administrator
MasterAvatarLastName = User
MasterAvatarSandboxPassword = password

; === Physics Configuration ===
; Choose physics engine for this specific region
PhysicsEngine = ODE
MaxPhysicalPrims = 10000
MaxNonPhysicalPrims = 50000

; === Terrain Configuration ===
TerrainFile = terrain/default.r32
TerrainGuess = 0
;TerrainImageID = 00000000-0000-0000-0000-000000000000

; === Estate Configuration ===
EstateOwnerFirstName = Estate
EstateOwnerLastName = Owner
EstateOwnerUUID = 22222222-2222-2222-2222-222222222222
EstateOwnerPassword = estate_password

[Region-Production-Main]
; === Production Region Example ===
RegionUUID = 33333333-3333-3333-3333-333333333333
Location = 2000,2000
InternalAddress = 0.0.0.0
InternalPort = 9000
ExternalHostName = grid.example.com

; === Region Details ===
RegionName = Main Island
RegionType = Mainland
MasterAvatarFirstName = Grid
MasterAvatarLastName = Administrator

; === Production Physics ===
PhysicsEngine = UBODE
MaxPhysicalPrims = 50000
MaxNonPhysicalPrims = 200000

; === High-Performance Settings ===
NonPhysicalPrimMax = 1024
PhysicalPrimMax = 512
ClampPrimSize = false
MaximumPrimScale = 1000
MaximumLinkCount = 256

[Region-Avatar-Hub]
; === Avatar-Focused Region ===
RegionUUID = 44444444-4444-4444-4444-444444444444
Location = 2001,2000
InternalAddress = 0.0.0.0
InternalPort = 9000
ExternalHostName = grid.example.com

RegionName = Avatar Hub
RegionType = Mainland

; === Avatar-Optimized Physics ===
PhysicsEngine = ODE
MaxAvatars = 100
AvatarPhysicsOptimized = true

[Region-Vehicle-Showcase]
; === Vehicle Physics Region ===
RegionUUID = 55555555-5555-5555-5555-555555555555
Location = 2002,2000
InternalAddress = 0.0.0.0
InternalPort = 9000
ExternalHostName = grid.example.com

RegionName = Vehicle Showcase
RegionType = Mainland

; === Vehicle-Optimized Physics ===
PhysicsEngine = Bullet
VehiclePhysicsEnabled = true
AdvancedVehiclePhysics = true

[Region-Particle-Lab]
; === Particle Physics Region ===
RegionUUID = 66666666-6666-6666-6666-666666666666
Location = 2003,2000
InternalAddress = 0.0.0.0
InternalPort = 9000
ExternalHostName = grid.example.com

RegionName = Particle Laboratory
RegionType = Mainland

; === Particle-Optimized Physics ===
PhysicsEngine = POS
ParticleSystemsEnabled = true
FluidDynamicsEnabled = true
GPUAccelerationEnabled = true
MaxParticles = 100000
```

## 2.4 Advanced Configuration Files

### Physics Engine Configuration

Create `config/config-include/opensim-next/Physics.ini`:

```ini
[Physics]
; === Multi-Physics Engine Configuration ===
; OpenSim Next supports 5 physics engines with per-region selection

; === ODE Physics Engine ===
[ODEPhysicsSettings]
; Traditional OpenSimulator physics engine
Enabled = true
MaxBodyCount = 10000
DefaultFriction = 0.6
DefaultRestitution = 0.1
GravityX = 0
GravityY = -9.8
GravityZ = 0
ContactMaxCorrectingVel = 100
ContactSurfaceLayer = 0.001
WorldStepSize = 0.02
WorldHashSpace = 256

; === UBODE Physics Engine ===
[UBODEPhysicsSettings]
; Enhanced ODE with better performance
Enabled = true
MaxBodyCount = 20000
avPIDD = 2200
avPIDP = 900
avStandupTau = 2000000
avDensity = 3.5
avMovementDivisorWalk = 1.3
avMovementDivisorRun = 0.8
avCapRadius = 0.37
avCapsuleStandup = 0.5

; === Bullet Physics Engine ===
[BulletPhysicsSettings]
; Modern physics with advanced features
Enabled = true
MaxBodyCount = 50000
SolverIterations = 10
SplitImpulseEnabled = true
ContactBreakingThreshold = 0.02
LinearDamping = 0.0
AngularDamping = 0.0
DeactivationTime = 0.2
LinearSleepingThreshold = 0.8
AngularSleepingThreshold = 1.0
MaxProxies = 32766
CollisionMargin = 0.04

; === POS (Position-Based Dynamics) Engine ===
[POSPhysicsSettings]
; Advanced particle and fluid physics
Enabled = true
MaxBodyCount = 100000
MaxParticles = 1000000
EnableParticlePhysics = true
EnableFluidDynamics = true
EnableGPUAcceleration = true
ParticleRadius = 0.1
FluidDensity = 1000
FluidViscosity = 0.001
SPHKernelRadius = 0.2
ConstraintIterations = 5

; === Basic Physics Engine ===
[BasicPhysicsSettings]
; Lightweight physics for testing
Enabled = true
MaxBodyCount = 1000
SimpleGravity = true
BasicCollision = true
```

### WebSocket Configuration

Create `config/config-include/opensim-next/WebSockets.ini`:

```ini
[WebSockets]
; === WebSocket Server Configuration ===
; Enables revolutionary web browser access to virtual worlds

; === Server Settings ===
Enabled = true
Port = 9001
MaxConnections = 5000
ConnectionTimeout = 300
HeartbeatInterval = 30

; === Protocol Settings ===
ProtocolVersion = 13
EnableCompression = true
CompressionLevel = 6
MaxMessageSize = 1048576  ; 1MB
EnableBinaryMessages = true

; === Authentication ===
RequireAuthentication = true
AuthenticationTimeout = 60
EnableTokenAuth = true
TokenExpiration = 3600

; === Rate Limiting ===
EnableRateLimiting = true
MaxMessagesPerSecond = 100
MaxBytesPerSecond = 1048576  ; 1MB/s
BurstLimit = 200

; === Cross-Origin Support ===
EnableCORS = true
AllowedOrigins = *
AllowedMethods = GET,POST,PUT,DELETE,OPTIONS
AllowedHeaders = Content-Type,Authorization,X-Requested-With

; === SSL/TLS Support ===
EnableSSL = false
SSLCertificatePath = /etc/ssl/certs/opensim.crt
SSLPrivateKeyPath = /etc/ssl/private/opensim.key

[WebClient]
; === Web Client Interface ===
Enabled = true
Port = 8080
StaticContentPath = www/
EnableDevelopmentMode = false

; === Client Features ===
EnableVoiceChat = false
EnableVideoChat = false
EnableFileUpload = true
MaxUploadSize = 10485760  ; 10MB

; === 3D Rendering ===
Enable3DRenderer = true
RendererType = WebGL
EnableVR = false
EnableAR = false
```

### Monitoring Configuration

Create `config/config-include/opensim-next/Monitoring.ini`:

```ini
[Monitoring]
; === OpenSim Next Monitoring & Observability ===

; === Prometheus Metrics ===
[Prometheus]
Enabled = true
Port = 9100
MetricsPath = /metrics
EnableAuthentication = true
ApiKey = ${OPENSIM_API_KEY}

; === Metrics Collection ===
CollectSystemMetrics = true
CollectPhysicsMetrics = true
CollectNetworkMetrics = true
CollectDatabaseMetrics = true
CollectWebSocketMetrics = true

; === Retention ===
MetricsRetention = 7d
DetailedMetricsRetention = 1d

; === Health Checks ===
[HealthChecks]
Enabled = true
Interval = 30
Timeout = 10
Endpoint = /health

; === Health Check Types ===
CheckDatabase = true
CheckPhysics = true
CheckNetwork = true
CheckMemory = true
CheckDisk = true
CheckZeroTrust = true

; === Admin Dashboard ===
[AdminDashboard]
Enabled = true
Port = 8090
AuthenticationRequired = true
ApiKey = ${OPENSIM_API_KEY}

; === Dashboard Features ===
EnableRealTimeStats = true
EnableRegionManagement = true
EnableUserManagement = true
EnableAssetManagement = true
EnableLogViewer = true

; === Alerting ===
[Alerting]
Enabled = false
WebhookURL = https://alerts.example.com/webhook
AlertThresholds = config/alert-thresholds.json

; === Performance Profiling ===
[Profiling]
Enabled = false
SamplingInterval = 100
ProfilingPort = 9200
EnableMemoryProfiling = true
EnableCPUProfiling = true
```

## 2.5 Environment Variables

OpenSim Next uses environment variables for runtime configuration. These are the primary variables that control server behavior.

### Instance and Service Mode (Critical)

OpenSim Next supports multiple service modes via environment variables. This is the primary way to configure how the server runs.

```bash
# Instance directory — points to the folder containing all config for this instance
# Contains: OpenSim.ini, Regions/*.ini, database settings, estate config
export OPENSIM_INSTANCE_DIR="./Instances/Gaiagrid"

# Service mode — determines what services this process runs
#   standalone  — All services in one process (development, single-user)
#   grid        — Region simulator only, connects to external Robust services
#   robust      — Central services only (user accounts, inventory, assets, grid)
#   controller  — Admin controller for multi-instance management
export OPENSIM_SERVICE_MODE="standalone"

# Robust URL — required when OPENSIM_SERVICE_MODE=grid
# Points to the Robust services process
export OPENSIM_ROBUST_URL="http://localhost:8503"
```

### Gaia Grid (2-Process Mode) Example

The recommended production setup runs two processes: one for Robust services and one for the region simulator.

```bash
# Terminal 1: Start Robust services
OPENSIM_INSTANCE_DIR=./Instances/Gaiagrid \
OPENSIM_SERVICE_MODE=robust \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next

# Terminal 2: Start region simulator
OPENSIM_INSTANCE_DIR=./Instances/Gaiagrid \
OPENSIM_SERVICE_MODE=grid \
OPENSIM_ROBUST_URL=http://localhost:8503 \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next
```

### Library Path (Required)

The Zig physics library and BulletSim must be on the library path:

```bash
# macOS
export DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64

# Linux
export LD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64
```

### Logging

```bash
# Tracing log level (uses the `tracing` crate, NOT `log`)
# Values: error, warn, info, debug, trace
export RUST_LOG="info"

# For verbose protocol debugging:
export RUST_LOG="opensim_next=debug"
```

### Database Configuration

Database connection is configured in the instance's OpenSim.ini, but can be overridden:

```bash
# PostgreSQL connection (used by Gaia grid)
# Format: postgresql://user@host/database
export DATABASE_URL="postgresql://opensim@localhost/gaiagrid"
```

### AI Configuration (Galadriel)

```bash
# Ollama endpoint for Galadriel AI
export OLLAMA_URL="http://localhost:11434"

# Default model for Galadriel (if not set in llm.ini)
export OLLAMA_MODEL="llama3.1:8b"
```

## 2.6 Configuration Validation

OpenSim Next validates configuration on startup. If critical settings are missing, the server logs errors and exits cleanly.

### Startup Validation

When the server starts, it automatically checks:

1. Instance directory exists and contains `OpenSim.ini`
2. Region configuration files are valid
3. Database connection is reachable
4. Zig physics library is loadable (via DYLD_LIBRARY_PATH/LD_LIBRARY_PATH)
5. Required ports are available

### Manual Validation

```bash
# Check that the binary runs and can find its libraries
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next --help

# Verify Zig library is built
ls -la zig/zig-out/lib/libopensim_physics.dylib   # macOS
ls -la zig/zig-out/lib/libopensim_physics.so       # Linux

# Verify instance directory structure
ls Instances/Gaiagrid/OpenSim.ini
ls Instances/Gaiagrid/Regions/
```

## 2.7 Configuration Migration

### From OpenSimulator 0.9.x

```bash
# Automatic migration utility
opensim-next --migrate-config --source=/path/to/opensim.ini

# Manual migration assistance
opensim-next --analyze-config --source=/path/to/opensim.ini
```

### Configuration Backup and Restore

```bash
# Backup current configuration
opensim-next --backup-config --output=config-backup-$(date +%Y%m%d).tar.gz

# Restore configuration
opensim-next --restore-config --input=config-backup-20250623.tar.gz
```

## 2.8 Multi-Region Configuration

### Grid Configuration Example

For a multi-region grid, create `config/config-include/GridCommon.ini`:

```ini
[DatabaseService]
; === Grid-Wide Database Configuration ===
ConnectionString = "Host=db.grid.com;Database=opensim_grid;Username=opensim;Password=${OPENSIM_DB_PASSWORD}"
StorageProvider = OpenSim.Data.PGSQL.dll

[GridInfoService]
; === Grid Information ===
GridName = "OpenSim Next Grid"
GridNick = "opensim-next"
GridOwner = "Grid Administrator"
GridURL = "https://grid.example.com"
WelcomeMessage = "Welcome to OpenSim Next Grid!"

[GridService]
; === Grid Service Configuration ===
LocalServiceModule = OpenSim.Services.GridService.dll:GridService
StorageProvider = ${DatabaseService|StorageProvider}
ConnectionString = ${DatabaseService|ConnectionString}

; === Grid Features ===
AllowDuplicateNames = false
AllowHypergridMapSearch = true
ExportSupported = true
Region_Welcome = "Welcome Region"

[UserAccountService]
; === User Account Service ===
LocalServiceModule = OpenSim.Services.UserAccountService.dll:UserAccountService
StorageProvider = ${DatabaseService|StorageProvider}
ConnectionString = ${DatabaseService|ConnectionString}

[PresenceService]
; === Presence Service ===
LocalServiceModule = OpenSim.Services.PresenceService.dll:PresenceService
StorageProvider = ${DatabaseService|StorageProvider}
ConnectionString = ${DatabaseService|ConnectionString}

[AvatarService]
; === Avatar Service ===
LocalServiceModule = OpenSim.Services.AvatarService.dll:AvatarService
StorageProvider = ${DatabaseService|StorageProvider}
ConnectionString = ${DatabaseService|ConnectionString}

[FriendsService]
; === Friends Service ===
LocalServiceModule = OpenSim.Services.FriendsService.dll:FriendsService
StorageProvider = ${DatabaseService|StorageProvider}
ConnectionString = ${DatabaseService|ConnectionString}

[InventoryService]
; === Inventory Service ===
LocalServiceModule = OpenSim.Services.InventoryService.dll:XInventoryService
StorageProvider = ${DatabaseService|StorageProvider}
ConnectionString = ${DatabaseService|ConnectionString}

[AssetService]
; === Asset Service ===
LocalServiceModule = OpenSim.Services.AssetService.dll:AssetService
StorageProvider = ${DatabaseService|StorageProvider}
ConnectionString = ${DatabaseService|ConnectionString}

; === Asset Caching ===
DefaultAssetLoader = OpenSim.Framework.AssetLoader.Filesystem.dll
AssetLoaderArgs = assets/AssetSets.xml

[LibraryService]
; === Library Service ===
LocalServiceModule = OpenSim.Services.InventoryService.dll:LibraryService
LibraryName = "OpenSim Library"
DefaultLibrary = ./inventory/Libraries.xml
```

## 2.9 Configuration Best Practices

### Development Environment
- Use SQLite for database (faster setup)
- Enable detailed logging
- Use single physics engine (ODE)
- Disable zero trust networking initially
- Enable all monitoring features

### Staging Environment  
- Use PostgreSQL database
- Enable moderate logging
- Test multi-physics engines
- Enable zero trust networking
- Enable monitoring and alerting

### Production Environment
- Use PostgreSQL with clustering
- Use structured JSON logging
- Optimize physics engine per region
- Enable all security features
- Enable comprehensive monitoring
- Use environment variables for secrets
- Enable SSL/TLS everywhere
- Configure backup automation

### Configuration Security
- Never commit passwords to version control
- Use environment variables for secrets
- Rotate API keys regularly
- Enable strong authentication
- Use SSL/TLS certificates
- Configure firewalls appropriately
- Enable audit logging

---

**Configuration Complete!** 🎉

Your OpenSim Next server is now properly configured. The next chapter will guide you through database setup and migration procedures.

---

# Chapter 3: Database Setup

This chapter provides comprehensive database setup and migration procedures for OpenSim Next, covering both SQLite for development and PostgreSQL for production deployments.

## 3.1 Database Overview

OpenSim Next supports multiple database backends with full compatibility for existing OpenSimulator schemas. The system provides automatic migration capabilities and supports clustering for high-availability deployments.

### Supported Database Systems

| Database | Use Case | Performance | Scalability | High Availability |
|----------|----------|-------------|-------------|-------------------|
| **SQLite** | Development, Testing | Good | Limited | ❌ |
| **PostgreSQL** | Production, Enterprise | Excellent | High | ✅ |
| **MySQL/MariaDB** | Legacy compatibility | Good | Medium | ✅ |

### Database Schema Compatibility

OpenSim Next maintains 100% backward compatibility with existing OpenSimulator database schemas:

- **OpenSim Standalone** - Direct migration supported
- **OpenSim Grid Mode** - Full grid service migration
- **ROBUST Services** - Complete service database migration
- **Third-party modifications** - Custom schema preservation

## 3.2 SQLite Setup (Development)

SQLite is ideal for development and testing environments due to its simplicity and zero-configuration setup.

### Automatic SQLite Configuration

OpenSim Next automatically creates and configures SQLite databases on first startup:

```bash
# Start server with SQLite (default for development)
cd opensim-next/rust
cargo run

# Database files are created automatically in:
# - opensim.db (main database)
# - logs/ (log files)
# - cache/ (asset cache)
```

### Manual SQLite Configuration

For custom SQLite setup, modify `config/opensim.ini`:

```ini
[Startup]
; SQLite Configuration for Development
ConnectionString = "Data Source=data/opensim-dev.db;Version=3;New=True;Journal Mode=WAL;"
StorageProvider = OpenSim.Data.SQLite.dll

; Asset Storage
AssetConnectionString = "Data Source=data/assets.db;Version=3;New=True;Journal Mode=WAL;"
AssetStorageProvider = OpenSim.Data.SQLite.dll

; Inventory Storage  
InventoryConnectionString = ${ConnectionString}
InventoryStorageProvider = ${StorageProvider}

; User Accounts
UserAccountConnectionString = ${ConnectionString}
UserAccountStorageProvider = ${StorageProvider}

; Authentication
AuthenticationConnectionString = ${ConnectionString}
AuthenticationStorageProvider = ${StorageProvider}

; Grid Information
GridInfoConnectionString = ${ConnectionString}  
GridInfoStorageProvider = ${StorageProvider}

; Presence Service
PresenceConnectionString = ${ConnectionString}
PresenceStorageProvider = ${StorageProvider}

; Friends Service
FriendsConnectionString = ${ConnectionString}
FriendsStorageProvider = ${StorageProvider}

; Avatar Service
AvatarConnectionString = ${ConnectionString}
AvatarStorageProvider = ${StorageProvider}

[SQLite]
; SQLite-specific optimizations
WAL_Mode = true
Synchronous = NORMAL
Cache_Size = 10000
Temp_Store = MEMORY
```

### SQLite Performance Optimization

```bash
# Enable Write-Ahead Logging (WAL) mode for better performance
sqlite3 data/opensim.db "PRAGMA journal_mode=WAL;"

# Optimize cache size (adjust based on available memory)
sqlite3 data/opensim.db "PRAGMA cache_size=10000;"

# Enable memory temp storage
sqlite3 data/opensim.db "PRAGMA temp_store=MEMORY;"

# Check database integrity
sqlite3 data/opensim.db "PRAGMA integrity_check;"
```

### SQLite Maintenance

```bash
# Vacuum database to reclaim space
sqlite3 data/opensim.db "VACUUM;"

# Analyze for query optimization  
sqlite3 data/opensim.db "ANALYZE;"

# Backup SQLite database
cp data/opensim.db backup/opensim-$(date +%Y%m%d-%H%M%S).db
```

## 3.3 PostgreSQL Setup (Production)

PostgreSQL is the recommended database for production deployments, offering superior performance, scalability, and high availability features.

### PostgreSQL Installation

#### Ubuntu/Debian
```bash
# Install PostgreSQL server
sudo apt update
sudo apt install -y postgresql postgresql-contrib postgresql-client

# Start and enable PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Verify installation
sudo systemctl status postgresql
```

#### CentOS/RHEL
```bash
# Install PostgreSQL
sudo dnf install -y postgresql postgresql-server postgresql-contrib

# Initialize database
sudo postgresql-setup --initdb

# Start and enable service
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### macOS
```bash
# Install via Homebrew
brew install postgresql

# Start PostgreSQL service
brew services start postgresql

# Create default database
createdb opensim
```

### PostgreSQL Database Creation

```bash
# Switch to postgres user
sudo -u postgres psql

-- Create OpenSim database and user
CREATE DATABASE opensim_next;
CREATE USER opensim_user WITH ENCRYPTED PASSWORD 'secure_password_here';

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE opensim_next TO opensim_user;
GRANT USAGE, CREATE ON SCHEMA public TO opensim_user;

-- Create additional databases for services (optional)
CREATE DATABASE opensim_assets;
CREATE DATABASE opensim_inventory; 
CREATE DATABASE opensim_users;

-- Grant access to service databases
GRANT ALL PRIVILEGES ON DATABASE opensim_assets TO opensim_user;
GRANT ALL PRIVILEGES ON DATABASE opensim_inventory TO opensim_user;
GRANT ALL PRIVILEGES ON DATABASE opensim_users TO opensim_user;

-- Exit PostgreSQL console
\q
```

### PostgreSQL Configuration

Update `config/opensim.ini` for PostgreSQL:

```ini
[Startup]
; PostgreSQL Configuration for Production
ConnectionString = "Server=localhost;Database=opensim_next;User Id=opensim_user;Password=secure_password_here;SSL Mode=Require;"
StorageProvider = OpenSim.Data.PGSQL.dll

; Asset Storage (can use separate database)
AssetConnectionString = "Server=localhost;Database=opensim_assets;User Id=opensim_user;Password=secure_password_here;SSL Mode=Require;"
AssetStorageProvider = OpenSim.Data.PGSQL.dll

; Inventory Storage
InventoryConnectionString = "Server=localhost;Database=opensim_inventory;User Id=opensim_user;Password=secure_password_here;SSL Mode=Require;"
InventoryStorageProvider = OpenSim.Data.PGSQL.dll

; User Accounts
UserAccountConnectionString = "Server=localhost;Database=opensim_users;User Id=opensim_user;Password=secure_password_here;SSL Mode=Require;"
UserAccountStorageProvider = OpenSim.Data.PGSQL.dll

; All other services can use main database
AuthenticationConnectionString = ${ConnectionString}
AuthenticationStorageProvider = ${StorageProvider}

GridInfoConnectionString = ${ConnectionString}
GridInfoStorageProvider = ${StorageProvider}

PresenceConnectionString = ${ConnectionString}
PresenceStorageProvider = ${StorageProvider}

FriendsConnectionString = ${ConnectionString}
FriendsStorageProvider = ${StorageProvider}

AvatarConnectionString = ${ConnectionString}
AvatarStorageProvider = ${StorageProvider}

[PostgreSQL]
; PostgreSQL-specific settings
ConnectionPool_MinSize = 5
ConnectionPool_MaxSize = 50
ConnectionTimeout = 30
CommandTimeout = 300
SSL_Mode = Require
```

### PostgreSQL Performance Tuning

Edit `/etc/postgresql/15/main/postgresql.conf`:

```ini
# Memory settings (adjust based on available RAM)
shared_buffers = 256MB                    # 25% of total RAM
effective_cache_size = 1GB                # 75% of total RAM
work_mem = 4MB                           # Per-connection memory
maintenance_work_mem = 64MB              # Maintenance operations

# Connection settings
max_connections = 200                     # Adjust based on load
connection_limit = 100                    # Per-database limit

# Write-ahead logging
wal_buffers = 16MB
checkpoint_completion_target = 0.9
wal_level = replica

# Query optimization
random_page_cost = 1.1                   # For SSD storage
effective_io_concurrency = 200           # For SSD storage

# Logging (for production debugging)
log_statement = 'mod'                    # Log modifications
log_duration = on
log_min_duration_statement = 1000        # Log slow queries

# Autovacuum tuning
autovacuum = on
autovacuum_vacuum_scale_factor = 0.1
autovacuum_analyze_scale_factor = 0.05
```

Restart PostgreSQL to apply changes:
```bash
sudo systemctl restart postgresql
```

## 3.4 Automatic Database Initialization

OpenSim Next automatically creates all database tables on first startup via `DatabaseInitializer::initialize()`. No manual migration is required.

### Auto-Initialization Process

When the server starts, it:
1. Detects the database backend (SQLite, PostgreSQL, MySQL) from the connection string
2. Runs all 38 migrations (001-038) in order
3. Skips tables that already exist (safe for repeated startups)
4. Creates all indexes and constraints

### Migration Files

All 38 migrations live in `rust/src/database/migrations/` and cover:

| Migration Range | Store Types |
|----------------|-------------|
| 001-005 | UserAccount, AssetStore, AuthStore |
| 006-010 | InventoryStore, RegionStore (v67) |
| 011-015 | Avatar (v3), Presence (v4), FriendsStore (v4) |
| 016-020 | GridUserStore (v2), EstateStore (v36) |
| 021-025 | UserProfiles (v5), XAssetStore (v2), GridStore (v10) |
| 026-030 | AgentPrefs, HGTravelStore, IM_Store |
| 031-035 | MuteListStore, os_groups_Store, LogStore |
| 036-038 | Land, PrimShapes, terrain |

### Pre-Migration Backup

Always backup your existing database before the first startup:

```bash
# SQLite backup
cp existing-opensim.db opensim-backup-$(date +%Y%m%d-%H%M%S).db

# PostgreSQL backup
pg_dump -h localhost -U opensim gaiagrid > gaiagrid-backup-$(date +%Y%m%d-%H%M%S).sql

# MySQL backup
mysqldump -u opensim -p opensim_db > opensim-backup-$(date +%Y%m%d-%H%M%S).sql
```

### Migrating from C# OpenSim to OpenSim Next

OpenSim Next uses the same database schema as C# OpenSim 0.9.x. You can point OpenSim Next at an existing database:

1. **Backup your existing database** (see above)
2. **Set the connection string** in your instance's `OpenSim.ini` to point to the existing database
3. **Start OpenSim Next** — it will add any missing tables without modifying existing ones
4. **Verify** by logging in with an existing account

For PostgreSQL (recommended for production):
```bash
# Create a new database from existing OpenSim data
pg_dump -h localhost -U opensim old_opensim_db | psql -U opensim gaiagrid

# Point OpenSim Next at it
# In Instances/Gaiagrid/OpenSim.ini:
# DatabaseConnectionString = "postgresql://opensim@localhost/gaiagrid"
```

### Schema Migration Details

OpenSim Next migrations handle:

#### User Accounts Table
```sql
-- Legacy OpenSim schema
CREATE TABLE UserAccounts (
    PrincipalID UUID PRIMARY KEY,
    ScopeID UUID NOT NULL,
    FirstName VARCHAR(64) NOT NULL,
    LastName VARCHAR(64) NOT NULL,
    Email VARCHAR(64),
    ServiceURLs TEXT,
    Created INTEGER NOT NULL
);

-- OpenSim Next enhanced schema (backward compatible)
CREATE TABLE UserAccounts (
    PrincipalID UUID PRIMARY KEY,
    ScopeID UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    FirstName VARCHAR(64) NOT NULL,
    LastName VARCHAR(64) NOT NULL,
    Email VARCHAR(256),  -- Increased email length
    ServiceURLs TEXT,
    Created INTEGER NOT NULL,
    -- OpenSim Next enhancements
    LastLogin TIMESTAMP DEFAULT NULL,
    LoginCount INTEGER DEFAULT 0,
    UserFlags INTEGER DEFAULT 0,
    UserTitle VARCHAR(256) DEFAULT NULL,
    ProfileImage UUID DEFAULT NULL,
    ProfileFirstImage UUID DEFAULT NULL,
    ProfileAboutText TEXT DEFAULT NULL,
    ProfileFirstText TEXT DEFAULT NULL,
    ProfileURL VARCHAR(256) DEFAULT NULL,
    ProfileWantToMask INTEGER DEFAULT 0,
    ProfileWantToText TEXT DEFAULT NULL,
    ProfileSkillsMask INTEGER DEFAULT 0,
    ProfileSkillsText TEXT DEFAULT NULL,
    ProfileLanguages TEXT DEFAULT NULL,
    -- Indexes for performance
    INDEX idx_useraccounts_name (FirstName, LastName),
    INDEX idx_useraccounts_email (Email),
    INDEX idx_useraccounts_lastlogin (LastLogin)
);
```

#### Assets Table
```sql
-- Enhanced assets table with OpenSim Next features
CREATE TABLE assets (
    name VARCHAR(64) PRIMARY KEY,
    description VARCHAR(64),
    assetType TINYINT NOT NULL,
    local BOOLEAN NOT NULL,
    temporary BOOLEAN NOT NULL,
    data LONGBLOB,
    -- OpenSim Next enhancements
    creation_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    creator_id UUID DEFAULT NULL,
    access_time TIMESTAMP DEFAULT NULL,
    hash_sha256 VARCHAR(64) DEFAULT NULL,
    compressed BOOLEAN DEFAULT FALSE,
    compression_type VARCHAR(16) DEFAULT NULL,
    metadata JSON DEFAULT NULL,
    -- Performance indexes
    INDEX idx_assets_type (assetType),
    INDEX idx_assets_creator (creator_id),
    INDEX idx_assets_hash (hash_sha256),
    INDEX idx_assets_access_time (access_time)
);
```

### Post-Migration Validation

After starting with a migrated database, verify data integrity:

```bash
# Check that tables exist (PostgreSQL)
psql -U opensim gaiagrid -c "\dt"

# Verify user accounts loaded
psql -U opensim gaiagrid -c "SELECT COUNT(*) FROM useraccount;"

# Verify assets
psql -U opensim gaiagrid -c "SELECT COUNT(*) FROM assets;"

# Verify regions registered
psql -U opensim gaiagrid -c "SELECT regionname, locx, locy, sizex, sizey FROM regions;"

# Check server logs for any migration errors
grep -i "migration\|error\|failed" /tmp/robust.log
```

## 3.5 High Availability Database Setup

For enterprise deployments, OpenSim Next supports PostgreSQL clustering and high availability configurations.

### PostgreSQL Primary-Replica Setup

#### Primary Server Configuration

Edit `/etc/postgresql/15/main/postgresql.conf`:
```ini
# Replication settings
wal_level = replica
max_wal_senders = 10
max_replication_slots = 10
synchronous_commit = on
synchronous_standby_names = 'standby1,standby2'

# Archive settings for point-in-time recovery
archive_mode = on
archive_command = 'rsync -a %p backup-server:/backup/wal/%f'
```

Edit `/etc/postgresql/15/main/pg_hba.conf`:
```
# Replication connections
host replication replication 192.168.1.0/24 md5
```

#### Replica Server Setup

```bash
# Stop PostgreSQL on replica
sudo systemctl stop postgresql

# Remove existing data directory
sudo rm -rf /var/lib/postgresql/15/main

# Create base backup from primary
sudo -u postgres pg_basebackup \
  -h primary-server-ip \
  -D /var/lib/postgresql/15/main \
  -U replication \
  -v -P -W

# Create recovery configuration
sudo -u postgres cat > /var/lib/postgresql/15/main/standby.signal << EOF
standby_mode = 'on'
primary_conninfo = 'host=primary-server-ip port=5432 user=replication'
trigger_file = '/tmp/postgresql.trigger'
EOF

# Start replica
sudo systemctl start postgresql
```

### Connection Pooling with PgBouncer

Install and configure PgBouncer for connection pooling:

```bash
# Install PgBouncer
sudo apt install pgbouncer

# Configure PgBouncer
sudo cat > /etc/pgbouncer/pgbouncer.ini << EOF
[databases]
opensim_next = host=localhost port=5432 dbname=opensim_next
opensim_assets = host=localhost port=5432 dbname=opensim_assets

[pgbouncer]
pool_mode = transaction
listen_port = 6432
listen_addr = 127.0.0.1
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt
logfile = /var/log/postgresql/pgbouncer.log
pidfile = /var/run/postgresql/pgbouncer.pid
max_client_conn = 1000
default_pool_size = 50
max_db_connections = 100
EOF

# Create user authentication file
sudo cat > /etc/pgbouncer/userlist.txt << EOF
"opensim_user" "md5hash_of_password"
EOF

# Start PgBouncer
sudo systemctl start pgbouncer
sudo systemctl enable pgbouncer
```

Update OpenSim Next configuration to use PgBouncer:
```ini
[Startup]
ConnectionString = "Server=localhost;Port=6432;Database=opensim_next;User Id=opensim_user;Password=secure_password_here;SSL Mode=Disable;"
```

## 3.6 Database Monitoring and Maintenance

### Performance Monitoring

```bash
# Install pg_stat_statements extension
sudo -u postgres psql opensim_next -c "CREATE EXTENSION IF NOT EXISTS pg_stat_statements;"

# Monitor slow queries
sudo -u postgres psql opensim_next -c "
SELECT query, calls, total_time, mean_time, rows 
FROM pg_stat_statements 
WHERE mean_time > 1000 
ORDER BY mean_time DESC 
LIMIT 10;"

# Monitor database size
sudo -u postgres psql opensim_next -c "
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public' 
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;"
```

### Automated Backup

Create automated backup script:

```bash
#!/bin/bash
# /usr/local/bin/opensim-backup.sh

BACKUP_DIR="/backup/opensim"
RETENTION_DAYS=30
DATE=$(date +%Y%m%d-%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup main database
pg_dump -h localhost -U opensim_user opensim_next | gzip > $BACKUP_DIR/opensim_next_$DATE.sql.gz

# Backup assets database  
pg_dump -h localhost -U opensim_user opensim_assets | gzip > $BACKUP_DIR/opensim_assets_$DATE.sql.gz

# Remove old backups
find $BACKUP_DIR -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete

# Log backup completion
echo "$(date): Backup completed successfully" >> /var/log/opensim-backup.log
```

Schedule via cron:
```bash
# Add to crontab (daily backup at 2 AM)
0 2 * * * /usr/local/bin/opensim-backup.sh
```

### Database Maintenance

```bash
# Regular maintenance script
#!/bin/bash
# /usr/local/bin/opensim-maintenance.sh

# Update table statistics
sudo -u postgres psql opensim_next -c "ANALYZE;"

# Vacuum dead tuples
sudo -u postgres psql opensim_next -c "VACUUM;"

# Reindex tables (monthly)
if [ $(date +%d) -eq "01" ]; then
    sudo -u postgres psql opensim_next -c "REINDEX DATABASE opensim_next;"
fi

# Log maintenance completion
echo "$(date): Database maintenance completed" >> /var/log/opensim-maintenance.log
```

## 3.7 Troubleshooting Database Issues

### Common Issues and Solutions

#### Connection Issues

**Problem**: Cannot connect to database
```
ERROR: connection to server at "localhost" (127.0.0.1), port 5432 failed
```

**Solution**:
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Check PostgreSQL configuration
sudo -u postgres psql -c "SELECT version();"

# Verify connection string
psql "Server=localhost;Database=opensim_next;User Id=opensim_user;Password=secure_password_here;"

# Check firewall rules
sudo ufw status
sudo iptables -L | grep 5432
```

#### Performance Issues

**Problem**: Slow database queries

**Solution**:
```bash
# Enable query logging
sudo sed -i "s/#log_statement = 'none'/log_statement = 'all'/" /etc/postgresql/15/main/postgresql.conf
sudo sed -i "s/#log_min_duration_statement = -1/log_min_duration_statement = 1000/" /etc/postgresql/15/main/postgresql.conf

# Restart PostgreSQL
sudo systemctl restart postgresql

# Analyze slow queries
sudo tail -f /var/log/postgresql/postgresql-15-main.log | grep "duration:"

# Update table statistics
sudo -u postgres psql opensim_next -c "ANALYZE;"
```

#### Migration Issues

**Problem**: Migration fails with data integrity errors

**Solution**:
```bash
# Check database constraints
sudo -u postgres psql opensim_next -c "
SELECT conname, contype, conkey, confkey 
FROM pg_constraint 
WHERE contype IN ('f', 'p', 'u');"

# Rebuild indexes
sudo -u postgres psql opensim_next -c "REINDEX DATABASE opensim_next;"
```

## 3.8 Database Security

### Access Control

```bash
# Create read-only user for monitoring
sudo -u postgres psql opensim_next -c "
CREATE USER opensim_monitor WITH PASSWORD 'monitor_password';
GRANT CONNECT ON DATABASE opensim_next TO opensim_monitor;
GRANT USAGE ON SCHEMA public TO opensim_monitor;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO opensim_monitor;"

# Create backup user
sudo -u postgres psql opensim_next -c "
CREATE USER opensim_backup WITH PASSWORD 'backup_password';
GRANT CONNECT ON DATABASE opensim_next TO opensim_backup;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO opensim_backup;"
```

### SSL/TLS Configuration

Configure PostgreSQL for SSL:

```bash
# Generate SSL certificates
sudo openssl req -new -x509 -days 365 -nodes -text \
  -out /etc/ssl/certs/server.crt \
  -keyout /etc/ssl/private/server.key \
  -subj "/CN=opensim-db-server"

# Set permissions
sudo chmod 600 /etc/ssl/private/server.key
sudo chown postgres:postgres /etc/ssl/private/server.key /etc/ssl/certs/server.crt
```

Update `/etc/postgresql/15/main/postgresql.conf`:
```ini
ssl = on
ssl_cert_file = '/etc/ssl/certs/server.crt'
ssl_key_file = '/etc/ssl/private/server.key'
ssl_ciphers = 'HIGH:MEDIUM:+3DES:!aNULL'
ssl_prefer_server_ciphers = on
```

Update connection strings to require SSL:
```ini
ConnectionString = "Server=localhost;Database=opensim_next;User Id=opensim_user;Password=secure_password_here;SSL Mode=Require;Trust Server Certificate=true;"
```

---

**Database Setup Complete!** 🎉

Your OpenSim Next database is now properly configured with high availability, monitoring, and security features. The next chapter will guide you through OpenZiti zero trust network configuration.

---

# Chapter 4: OpenZiti Zero Trust Configuration

This chapter provides comprehensive configuration instructions for OpenSim Next's revolutionary zero trust networking capabilities using OpenZiti. This enterprise-grade security framework enables secure, encrypted communication between all virtual world components.

## 4.1 Zero Trust Architecture Overview

OpenSim Next implements the world's first zero trust virtual world architecture, providing unprecedented security for virtual world deployments.

### What is Zero Trust?

Zero Trust is a security model that operates on the principle of "never trust, always verify." Every user, device, and service must be authenticated and authorized before accessing any resource, regardless of their location or network.

### OpenSim Next Zero Trust Benefits

- **🔒 End-to-End Encryption**: All communication is encrypted with AES-256-GCM
- **🎯 Identity-Based Access**: Services authenticate based on cryptographic identity
- **🚫 Network Agnostic**: Works across any network infrastructure
- **📊 Real-Time Monitoring**: Complete visibility into all network traffic
- **⚡ High Performance**: Optimized for real-time virtual world communication

### Architecture Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    OpenZiti Zero Trust Network                 │
├─────────────────────────────────────────────────────────────────┤
│                        Control Plane                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ OpenZiti       │  │ Policy Engine   │  │ Identity        │ │
│  │ Controller     │  │                 │  │ Authority       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                          Data Plane                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ OpenSim        │  │ Edge Router     │  │ OpenSim         │ │
│  │ Region A       │◄─┤ (Encrypted      ├─►│ Region B        │ │
│  │ (ODE Physics)  │  │  Tunnel)        │  │ (Bullet Physics)│ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Asset Service  │  │ Edge Router     │  │ User Service    │ │
│  │                │◄─┤ (Encrypted      ├─►│                 │ │
│  │                │  │  Tunnel)        │  │                 │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Web Clients    │  │ Edge Router     │  │ Database        │ │
│  │ (Browsers)     │◄─┤ (Encrypted      ├─►│ Cluster         │ │
│  │                │  │  Tunnel)        │  │                 │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 4.2 OpenZiti Installation and Setup

### Prerequisites

Before configuring OpenZiti, ensure you have:

- OpenSim Next installed and configured (Chapters 1-3)
- Administrative access to all servers
- Network connectivity between all components
- Valid SSL certificates (recommended)

### OpenZiti Controller Installation

#### Linux (Ubuntu/Debian)
```bash
# Download OpenZiti
curl -sL https://get.openziti.io/install.sh | sudo bash

# Verify installation
ziti --version

# Create OpenZiti user
sudo useradd -r -s /bin/false ziti
sudo mkdir -p /opt/ziti
sudo chown ziti:ziti /opt/ziti
```

#### Docker Installation (Recommended for Development)
```bash
# Create OpenZiti network
docker network create ziti

# Run OpenZiti Controller
docker run --name ziti-controller \
  --network ziti \
  -p 8441:8441 \
  -p 8442:8442 \
  -v ziti-controller-data:/persistent \
  openziti/ziti-controller:latest
```

### Initial Controller Configuration

1. **Initialize the Controller**:
```bash
# Set environment variables
export ZITI_HOME=/opt/ziti
export ZITI_NETWORK_NAME="opensim-next"
export ZITI_CONTROLLER_HOSTNAME="controller.opensim.local"
export ZITI_EDGE_ROUTER_HOSTNAME="router.opensim.local"

# Initialize PKI infrastructure
ziti pki create ca --pki-root "$ZITI_HOME/pki" --ca-file ca

# Create server certificates
ziti pki create server --pki-root "$ZITI_HOME/pki" \
  --ca-name ca \
  --server-file server \
  --dns "$ZITI_CONTROLLER_HOSTNAME" \
  --ip 127.0.0.1

# Initialize controller database
ziti controller create config \
  --pki-root "$ZITI_HOME/pki" \
  --output "$ZITI_HOME/controller.yaml"
```

2. **Start the Controller**:
```bash
# Start controller service
sudo systemd-run --unit=ziti-controller \
  --working-directory="$ZITI_HOME" \
  /usr/local/bin/ziti controller run "$ZITI_HOME/controller.yaml"

# Enable auto-start
sudo systemctl enable ziti-controller
```

3. **Verify Controller**:
```bash
# Check controller status
curl -k https://localhost:8441/health-checks

# Login to controller
ziti edge login https://localhost:8441 \
  -u admin \
  -p $(cat "$ZITI_HOME/credentials/admin.password")
```

## 4.3 Edge Router Configuration

Edge Routers provide secure tunnels for OpenSim Next services to communicate.

### Edge Router Installation

```bash
# Create edge router certificates
ziti pki create server --pki-root "$ZITI_HOME/pki" \
  --ca-name ca \
  --server-file edge-router \
  --dns "$ZITI_EDGE_ROUTER_HOSTNAME" \
  --ip 127.0.0.1

# Create edge router configuration
ziti create config router edge \
  --pki-root "$ZITI_HOME/pki" \
  --output "$ZITI_HOME/edge-router.yaml"

# Register edge router with controller
ziti edge create edge-router opensim-edge-router \
  -o "$ZITI_HOME/opensim-edge-router.jwt"

# Enroll edge router
ziti router enroll "$ZITI_HOME/edge-router.yaml" \
  --jwt "$ZITI_HOME/opensim-edge-router.jwt"
```

### Edge Router Service Configuration

Create systemd service:
```bash
sudo cat > /etc/systemd/system/ziti-edge-router.service << EOF
[Unit]
Description=OpenZiti Edge Router
After=network.target
Wants=network.target

[Service]
Type=simple
User=ziti
Group=ziti
ExecStart=/usr/local/bin/ziti router run /opt/ziti/edge-router.yaml
Restart=always
RestartSec=2
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl enable ziti-edge-router
sudo systemctl start ziti-edge-router
sudo systemctl status ziti-edge-router
```

## 4.4 OpenSim Next Integration

### Service Identity Creation

Create identities for OpenSim Next services:

```bash
# Create identities for OpenSim services
ziti edge create identity device opensim-region-a \
  -o "$ZITI_HOME/opensim-region-a.jwt" \
  -a region,physics-ode

ziti edge create identity device opensim-region-b \
  -o "$ZITI_HOME/opensim-region-b.jwt" \
  -a region,physics-bullet

ziti edge create identity device opensim-asset-service \
  -o "$ZITI_HOME/opensim-asset-service.jwt" \
  -a asset-service

ziti edge create identity device opensim-user-service \
  -o "$ZITI_HOME/opensim-user-service.jwt" \
  -a user-service

ziti edge create identity device opensim-database \
  -o "$ZITI_HOME/opensim-database.jwt" \
  -a database

ziti edge create identity device opensim-web-client \
  -o "$ZITI_HOME/opensim-web-client.jwt" \
  -a web-client
```

### Service Configuration

Define OpenZiti services for OpenSim Next:

```bash
# Region-to-region communication service
ziti edge create service opensim-region-comm \
  --configs intercept.v1,host.v1

# Asset service
ziti edge create service opensim-assets \
  --configs intercept.v1,host.v1

# Database service
ziti edge create service opensim-database \
  --configs intercept.v1,host.v1

# WebSocket service for web clients
ziti edge create service opensim-websocket \
  --configs intercept.v1,host.v1

# Monitoring service
ziti edge create service opensim-monitoring \
  --configs intercept.v1,host.v1
```

### Service Policies

Create policies to control access:

```bash
# Region communication policy
ziti edge create service-policy region-comm-dial Dial \
  --identity-roles '@region' \
  --service-roles '@region-comm'

ziti edge create service-policy region-comm-bind Bind \
  --identity-roles '@region' \
  --service-roles '@region-comm'

# Asset service policy
ziti edge create service-policy asset-service-dial Dial \
  --identity-roles '@region,@web-client' \
  --service-roles '@asset-service'

ziti edge create service-policy asset-service-bind Bind \
  --identity-roles '@asset-service' \
  --service-roles '@asset-service'

# Database access policy
ziti edge create service-policy database-dial Dial \
  --identity-roles '@region,@asset-service,@user-service' \
  --service-roles '@database'

ziti edge create service-policy database-bind Bind \
  --identity-roles '@database' \
  --service-roles '@database'

# Web client policy
ziti edge create service-policy web-client-dial Dial \
  --identity-roles '@web-client' \
  --service-roles '@websocket'

ziti edge create service-policy web-client-bind Bind \
  --identity-roles '@region' \
  --service-roles '@websocket'
```

## 4.5 OpenSim Next Configuration

Update OpenSim Next configuration to use OpenZiti.

### Update opensim.ini

```ini
[Architecture]
; Enable zero trust networking
enable_zero_trust = true
zero_trust_config = config-include/opensim-next/ZeroTrust.ini

; Encrypted overlay network
enable_overlay_network = true
overlay_network_topology = full_mesh
overlay_network_encryption = AES-256-GCM

[ZeroTrust]
; OpenZiti controller settings
controller_url = https://controller.opensim.local:8441
api_session_token_file = /opt/ziti/tokens/opensim-session.token

; Service identity
identity_file = /opt/ziti/opensim-region-a.json
identity_password_file = /opt/ziti/opensim-region-a.password

; Edge router settings
edge_router_url = tls:router.opensim.local:8442

; Connection settings
connection_timeout_seconds = 30
keepalive_interval_seconds = 60
max_reconnect_attempts = 10

; Security settings
require_encryption = true
verify_certificates = true
```

### Create ZeroTrust.ini

Create detailed OpenZiti configuration:

```ini
; config-include/opensim-next/ZeroTrust.ini
; OpenZiti Zero Trust Network Configuration

[OpenZiti]
; Controller configuration
ControllerUrl = https://controller.opensim.local:8441
ControllerCert = /opt/ziti/pki/ca/certs/ca.cert

; Identity configuration
IdentityFile = /opt/ziti/opensim-region-a.json
IdentityKeyFile = /opt/ziti/opensim-region-a.key

; Edge router configuration
EdgeRouterUrl = tls:router.opensim.local:8442
EdgeRouterCert = /opt/ziti/pki/edge-router/certs/edge-router.cert

[Services]
; Define OpenZiti services for OpenSim components
RegionCommunication = opensim-region-comm
AssetService = opensim-assets
DatabaseService = opensim-database
WebSocketService = opensim-websocket
MonitoringService = opensim-monitoring

[Security]
; Security settings
RequireEncryption = true
EncryptionAlgorithm = AES-256-GCM
KeyRotationInterval = 24h
SessionTimeout = 8h

; Certificate validation
VerifyCertificates = true
RequireClientCertificates = true
MinTLSVersion = 1.3

[Networking]
; Network topology
Topology = full_mesh
AutoDiscovery = true
LoadBalancing = true

; Connection settings
ConnectTimeout = 30s
KeepAliveInterval = 60s
MaxReconnectAttempts = 10
BackoffMultiplier = 2.0

; Buffer sizes
SendBufferSize = 64KB
ReceiveBufferSize = 64KB
MaxMessageSize = 1MB

[Monitoring]
; Monitoring and logging
EnableMetrics = true
MetricsInterval = 30s
LogLevel = INFO
LogFormat = JSON

; Health checks
HealthCheckInterval = 10s
HealthCheckTimeout = 5s
MaxFailedHealthChecks = 3

[Policies]
; Access control policies
DefaultDenyAll = true
RequireIdentityValidation = true
MaxConnectionsPerIdentity = 100

; Service-specific policies
RegionToRegion = allow
RegionToAsset = allow
RegionToDatabase = allow
WebClientToRegion = allow
WebClientToAsset = allow
```

## 4.6 Service Enrollment

Enroll each OpenSim Next service with OpenZiti.

### Region Service Enrollment

```bash
# Copy JWT to region server
scp /opt/ziti/opensim-region-a.jwt region-a:/opt/opensim/

# On region server, enroll identity
cd /opt/opensim
ziti edge enroll opensim-region-a.jwt

# Verify enrollment
ziti edge list identities
```

### Asset Service Enrollment

```bash
# Copy JWT to asset server
scp /opt/ziti/opensim-asset-service.jwt asset-server:/opt/opensim/

# On asset server, enroll identity
cd /opt/opensim
ziti edge enroll opensim-asset-service.jwt

# Test connection
ziti edge verify --identity opensim-asset-service.json
```

### Database Service Enrollment

```bash
# Copy JWT to database server
scp /opt/ziti/opensim-database.jwt db-server:/opt/opensim/

# On database server, enroll identity
cd /opt/opensim
ziti edge enroll opensim-database.jwt

# Configure database connection through OpenZiti
export OPENSIM_DB_CONNECTION="ziti:opensim-database:5432"
```

## 4.7 Network Topology Configuration

Configure different network topologies based on your deployment needs.

### Full Mesh Topology

Every service connects directly to every other service:

```bash
# Create full mesh configuration
cat > /opt/ziti/topology-full-mesh.yaml << EOF
topology: full_mesh
encryption: AES-256-GCM
services:
  - name: opensim-region-a
    identity: opensim-region-a
    endpoints:
      - protocol: tcp
        port: 9000
        address: "0.0.0.0"
  - name: opensim-region-b  
    identity: opensim-region-b
    endpoints:
      - protocol: tcp
        port: 9001
        address: "0.0.0.0"
  - name: opensim-assets
    identity: opensim-asset-service
    endpoints:
      - protocol: tcp
        port: 8080
        address: "0.0.0.0"
connections:
  - from: opensim-region-a
    to: [opensim-region-b, opensim-assets]
  - from: opensim-region-b
    to: [opensim-region-a, opensim-assets]
  - from: opensim-assets
    to: [opensim-region-a, opensim-region-b]
EOF

# Apply topology
ziti edge create config opensim-full-mesh intercept.v1 \
  --data @/opt/ziti/topology-full-mesh.yaml
```

### Hub-and-Spoke Topology

Central hub with efficient spoke connections:

```bash
# Create hub-and-spoke configuration
cat > /opt/ziti/topology-hub-spoke.yaml << EOF
topology: hub_spoke
hub: opensim-central-hub
encryption: AES-256-GCM
services:
  hub:
    name: opensim-central-hub
    identity: opensim-hub
    capabilities: [routing, load_balancing, monitoring]
  spokes:
    - name: opensim-region-a
      identity: opensim-region-a
      hub_connection: required
    - name: opensim-region-b
      identity: opensim-region-b
      hub_connection: required
    - name: opensim-assets
      identity: opensim-asset-service
      hub_connection: required
routing:
  strategy: least_latency
  health_check_interval: 10s
EOF

# Apply topology
ziti edge create config opensim-hub-spoke intercept.v1 \
  --data @/opt/ziti/topology-hub-spoke.yaml
```

### Hierarchical Topology

Multi-level organization for large grids:

```bash
# Create hierarchical configuration
cat > /opt/ziti/topology-hierarchical.yaml << EOF
topology: hierarchical
encryption: AES-256-GCM
levels:
  - level: 0
    name: global_hub
    services:
      - opensim-global-controller
      - opensim-global-assets
  - level: 1
    name: regional_hubs
    services:
      - opensim-north-america-hub
      - opensim-europe-hub
      - opensim-asia-hub
  - level: 2
    name: regional_services
    north_america:
      - opensim-region-us-east
      - opensim-region-us-west
    europe:
      - opensim-region-eu-central
      - opensim-region-eu-west
    asia:
      - opensim-region-asia-pacific
routing:
  inter_level: allow
  intra_level: allow
  cross_region: policy_based
EOF

# Apply topology
ziti edge create config opensim-hierarchical intercept.v1 \
  --data @/opt/ziti/topology-hierarchical.yaml
```

## 4.8 Monitoring and Analytics

Configure comprehensive monitoring for the zero trust network.

### Enable Network Analytics

```bash
# Create analytics configuration
cat > /opt/ziti/analytics.yaml << EOF
analytics:
  enabled: true
  collectors:
    - prometheus
    - elasticsearch
    - custom
  metrics:
    - connection_count
    - throughput
    - latency
    - error_rate
    - identity_usage
  retention: 30d
  sampling_rate: 100%
prometheus:
  endpoint: http://prometheus.opensim.local:9090
  interval: 30s
  labels:
    environment: production
    service: opensim-next
elasticsearch:
  endpoint: https://elasticsearch.opensim.local:9200
  index_pattern: ziti-analytics-%{+YYYY.MM.dd}
  template: ziti-analytics
custom:
  webhook_url: https://monitoring.opensim.local/webhook
  format: json
  batch_size: 100
  flush_interval: 60s
EOF

# Apply analytics configuration
ziti edge create config opensim-analytics host.v1 \
  --data @/opt/ziti/analytics.yaml
```

### Real-Time Dashboard

```bash
# Install Grafana for visualization
docker run -d \
  --name=grafana \
  -p 3000:3000 \
  -v grafana-storage:/var/lib/grafana \
  grafana/grafana

# Configure OpenZiti dashboard
curl -X POST http://admin:admin@localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @/opt/ziti/grafana-dashboard.json
```

## 4.9 Security Policies

Configure advanced security policies for production deployment.

### Identity-Based Access Control

```bash
# Create role-based access policies
ziti edge create identity device opensim-admin \
  -o /opt/ziti/opensim-admin.jwt \
  -a admin,management

ziti edge create identity device opensim-developer \
  -o /opt/ziti/opensim-developer.jwt \
  -a developer,read-only

ziti edge create identity device opensim-monitor \
  -o /opt/ziti/opensim-monitor.jwt \
  -a monitoring,metrics

# Admin access policy
ziti edge create service-policy admin-access Dial \
  --identity-roles '@admin' \
  --service-roles '@management,@monitoring,@region-comm,@asset-service,@database'

# Developer access policy (read-only)
ziti edge create service-policy developer-access Dial \
  --identity-roles '@developer' \
  --service-roles '@monitoring'

# Monitoring access policy
ziti edge create service-policy monitor-access Dial \
  --identity-roles '@monitoring' \
  --service-roles '@monitoring,@metrics'
```

### Time-Based Access Control

```bash
# Create time-based policies
cat > /opt/ziti/time-policies.yaml << EOF
policies:
  - name: business_hours_only
    type: posture_check
    conditions:
      time_range:
        start: "08:00"
        end: "18:00"
        timezone: "UTC"
        days: ["monday", "tuesday", "wednesday", "thursday", "friday"]
  - name: maintenance_window
    type: posture_check
    conditions:
      time_range:
        start: "02:00"
        end: "04:00"
        timezone: "UTC"
        days: ["sunday"]
      actions: ["deny_all"]
EOF

# Apply time-based policies
ziti edge create posture-check opensim-business-hours \
  --type TIME_RANGE \
  --data @/opt/ziti/time-policies.yaml
```

### Geolocation Policies

```bash
# Create geolocation-based access control
cat > /opt/ziti/geo-policies.yaml << EOF
policies:
  - name: allowed_countries
    type: posture_check
    conditions:
      geolocation:
        allowed_countries: ["US", "CA", "GB", "DE", "FR", "JP", "AU"]
        denied_countries: []
        accuracy_threshold: 1000  # meters
  - name: restricted_regions
    type: posture_check
    conditions:
      geolocation:
        denied_regions:
          - country: "US"
            states: []  # Allow all US states
          - country: "CN"
            action: "deny"
EOF

# Apply geolocation policies
ziti edge create posture-check opensim-geolocation \
  --type GEO_LOCATION \
  --data @/opt/ziti/geo-policies.yaml
```

## 4.10 Troubleshooting Zero Trust Network

### Common Issues and Solutions

#### Connection Issues

**Problem**: Cannot establish OpenZiti connection
```
ERROR: Failed to connect to controller
```

**Solution**:
```bash
# Check controller status
curl -k https://controller.opensim.local:8441/health-checks

# Verify certificates
openssl x509 -in /opt/ziti/pki/ca/certs/ca.cert -text -noout

# Check edge router connectivity
ziti edge list edge-routers

# Test identity enrollment
ziti edge verify --identity /opt/opensim/opensim-region-a.json
```

#### Policy Issues

**Problem**: Service access denied
```
ERROR: Access denied to service 'opensim-assets'
```

**Solution**:
```bash
# Check service policies
ziti edge list service-policies

# Verify identity roles
ziti edge list identities -j | jq '.[] | {name: .name, roleAttributes: .roleAttributes}'

# Check service roles
ziti edge list services -j | jq '.[] | {name: .name, roleAttributes: .roleAttributes}'

# Update policies if needed
ziti edge update service-policy asset-service-dial \
  --identity-roles '@region,@web-client,@admin'
```

#### Performance Issues

**Problem**: High latency or packet loss

**Solution**:
```bash
# Check edge router status
ziti edge list edge-routers --format json | \
  jq '.[] | {name: .name, online: .isOnline, version: .versionInfo}'

# Monitor connection metrics
ziti edge list sessions --format json | \
  jq '.[] | {id: .id, service: .service.name, created: .createdAt}'

# Optimize buffer sizes
ziti edge update config opensim-perf host.v1 \
  --data '{"protocol":"tcp","hostname":"localhost","port":9000,"bufferSize":"64KB"}'
```

#### Certificate Issues

**Problem**: Certificate validation failures

**Solution**:
```bash
# Regenerate certificates
cd /opt/ziti
ziti pki create server --pki-root pki \
  --ca-name ca \
  --server-file new-server \
  --dns controller.opensim.local \
  --ip 127.0.0.1

# Update controller configuration
ziti controller update config controller.yaml \
  --server-cert pki/new-server/certs/new-server.cert \
  --server-key pki/new-server/keys/new-server.key

# Restart controller
sudo systemctl restart ziti-controller
```

## 4.11 Performance Optimization

### Network Optimization

```bash
# Optimize network settings for OpenZiti
cat > /opt/ziti/network-optimization.yaml << EOF
network:
  tcp_optimization:
    no_delay: true
    keep_alive: true
    keep_alive_time: 60
    keep_alive_probes: 3
    keep_alive_interval: 10
  buffer_optimization:
    send_buffer: 262144    # 256KB
    receive_buffer: 262144 # 256KB
    max_message_size: 1048576  # 1MB
  connection_pooling:
    max_connections: 1000
    idle_timeout: 300
    max_idle_connections: 100
  compression:
    enabled: true
    algorithm: "lz4"
    level: 1
EOF

# Apply optimizations
ziti edge create config opensim-network-opt host.v1 \
  --data @/opt/ziti/network-optimization.yaml
```

### Security Optimization

```bash
# Optimize security settings
cat > /opt/ziti/security-optimization.yaml << EOF
security:
  encryption:
    algorithm: "AES-256-GCM"
    key_rotation_interval: "24h"
    perfect_forward_secrecy: true
  authentication:
    certificate_renewal_threshold: "72h"
    session_timeout: "8h"
    max_failed_attempts: 3
    lockout_duration: "15m"
  integrity:
    message_authentication: true
    replay_protection: true
    sequence_number_validation: true
EOF

# Apply security optimizations
ziti edge create config opensim-security-opt intercept.v1 \
  --data @/opt/ziti/security-optimization.yaml
```

---

**OpenZiti Zero Trust Configuration Complete!** 🎉

Your OpenSim Next deployment now features enterprise-grade zero trust networking with OpenZiti. All communication between virtual world components is encrypted and authenticated. The next chapter will guide you through encrypted overlay network setup and topology management.

---

# Chapter 5: Encrypted Overlay Network Setup

This chapter provides comprehensive setup and management instructions for OpenSim Next's revolutionary encrypted overlay network. Building on the OpenZiti foundation from Chapter 4, this system enables secure, real-time communication between virtual world regions with enterprise-grade encryption and performance.

## 5.1 Encrypted Overlay Network Overview

OpenSim Next features the world's first **encrypted overlay network specifically designed for virtual world region communication**. This breakthrough technology provides secure, scalable multi-region communication with real-time performance.

### Revolutionary Network Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│           OpenSim Next Encrypted Overlay Network               │
├─────────────────────────────────────────────────────────────────┤
│                    Control & Management Layer                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Network         │  │ Topology        │  │ Security        │ │
│  │ Controller      │  │ Manager         │  │ Policy Engine   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                 Encrypted Communication Layer                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ AES-256-GCM     │  │ Perfect Forward │  │ Real-Time       │ │
│  │ Encryption      │  │ Secrecy         │  │ Key Exchange    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Virtual World Services                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Region A        │◄─┤ Encrypted       ├─►│ Region B        │ │
│  │ Avatar Crossing │  │ Avatar Transfer │  │ Asset Sharing   │ │
│  │ ODE Physics     │  │ Protocol        │  │ Bullet Physics  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Grid-Wide       │◄─┤ Secure Message  ├─►│ Asset           │ │
│  │ Chat System     │  │ Broadcasting    │  │ Distribution    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Web Clients     │◄─┤ Real-Time       ├─►│ Traditional     │ │
│  │ (Browsers)      │  │ Synchronization │  │ SL Viewers      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Key Features

- **🔒 AES-256-GCM Encryption**: Military-grade encryption for all inter-region communication
- **🔑 Perfect Forward Secrecy**: Unique session keys that cannot be compromised retroactively
- **⚡ Real-Time Performance**: Optimized for virtual world latency requirements (<50ms)
- **📊 Dynamic Load Balancing**: Intelligent routing based on region load and network conditions
- **🌐 Global Scale**: Support for thousands of regions across multiple continents
- **🔄 Self-Healing**: Automatic recovery from network failures and partitions

## 5.2 Network Topology Configuration

OpenSim Next supports multiple network topologies optimized for different deployment scenarios.

### Topology Types

| Topology | Use Case | Max Regions | Latency | Fault Tolerance | Cost |
|----------|----------|-------------|---------|-----------------|------|
| **Full Mesh** | Small grids (< 50 regions) | 50 | Lowest | Highest | High |
| **Hub-and-Spoke** | Medium grids (50-500 regions) | 500 | Medium | Medium | Medium |
| **Hierarchical** | Large grids (500+ regions) | 10,000+ | Variable | High | Low |
| **Hybrid** | Enterprise deployments | Unlimited | Optimized | Highest | Variable |

### Full Mesh Topology Setup

Optimal for small to medium deployments requiring lowest latency.

```bash
# Create full mesh topology configuration
cat > /opt/opensim/topology-full-mesh.yaml << EOF
network:
  topology: full_mesh
  encryption:
    algorithm: AES-256-GCM
    key_rotation_interval: 24h
    perfect_forward_secrecy: true
  
regions:
  - name: "Welcome Region"
    uuid: "00000000-0000-0000-0000-000000000001"
    location: [1000, 1000]
    physics_engine: ODE
    max_avatars: 100
    encryption_key: auto
    
  - name: "Sandbox Region"
    uuid: "00000000-0000-0000-0000-000000000002"
    location: [1001, 1000]
    physics_engine: Bullet
    max_avatars: 50
    encryption_key: auto
    
  - name: "Social Hub"
    uuid: "00000000-0000-0000-0000-000000000003"
    location: [1000, 1001]
    physics_engine: UBODE
    max_avatars: 200
    encryption_key: auto

connections:
  matrix: full_mesh
  protocols:
    - avatar_crossing
    - object_transfer
    - chat_relay
    - asset_sync
  
performance:
  max_latency_ms: 50
  bandwidth_per_connection: "10Mbps"
  compression: true
  batching: true

monitoring:
  enable_metrics: true
  health_check_interval: 10s
  performance_tracking: true
  security_audit: true
EOF

# Apply full mesh configuration
cargo run --bin network-manager -- apply-topology \
  --config /opt/opensim/topology-full-mesh.yaml \
  --validate
```

### Hub-and-Spoke Topology Setup

Efficient for larger deployments with centralized management.

```bash
# Create hub-and-spoke topology
cat > /opt/opensim/topology-hub-spoke.yaml << EOF
network:
  topology: hub_spoke
  encryption:
    algorithm: AES-256-GCM
    key_rotation_interval: 12h
    perfect_forward_secrecy: true

hub:
  name: "Central Hub"
  location: "us-east-1"
  specifications:
    cpu_cores: 32
    memory_gb: 128
    network_bandwidth: "10Gbps"
  services:
    - routing
    - load_balancing
    - security_enforcement
    - global_chat
    - asset_distribution
  redundancy:
    backup_hubs: 2
    failover_time: "5s"

spokes:
  - region_group: "north_america"
    hub_connection: primary
    regions:
      - name: "NA Welcome"
        location: [2000, 2000]
        physics_engine: ODE
        max_avatars: 150
      - name: "NA Sandbox"
        location: [2001, 2000]
        physics_engine: Bullet
        max_avatars: 100
  
  - region_group: "europe"
    hub_connection: primary
    regions:
      - name: "EU Welcome"
        location: [3000, 3000]
        physics_engine: UBODE
        max_avatars: 150
      - name: "EU Commercial"
        location: [3001, 3000]
        physics_engine: POS
        max_avatars: 200

routing:
  strategy: least_latency
  load_balancing: true
  geo_optimization: true
  failover_detection: 5s

security:
  inter_spoke_communication: hub_only
  direct_spoke_connections: false
  security_scanning: true
  anomaly_detection: true
EOF

# Apply hub-and-spoke configuration
cargo run --bin network-manager -- apply-topology \
  --config /opt/opensim/topology-hub-spoke.yaml \
  --enable-monitoring
```

### Hierarchical Topology Setup

Scalable solution for enterprise and massive virtual world deployments.

```bash
# Create hierarchical topology
cat > /opt/opensim/topology-hierarchical.yaml << EOF
network:
  topology: hierarchical
  encryption:
    algorithm: AES-256-GCM
    key_rotation_interval: 6h
    perfect_forward_secrecy: true

hierarchy:
  levels: 3
  
  level_0:  # Global level
    name: "Global Controller"
    location: "global"
    services:
      - global_routing
      - cross_continental_communication
      - global_asset_distribution
      - worldwide_events
    nodes:
      - name: "Global Primary"
        location: "us-central"
        backup_locations: ["eu-central", "asia-pacific"]
  
  level_1:  # Continental level
    nodes:
      - name: "North America Hub"
        location: "us-central"
        coverage: ["US", "CA", "MX"]
        capacity: 2000
        
      - name: "Europe Hub"
        location: "eu-central"
        coverage: ["GB", "DE", "FR", "IT", "ES"]
        capacity: 1500
        
      - name: "Asia Pacific Hub"
        location: "ap-southeast"
        coverage: ["JP", "AU", "SG", "KR"]
        capacity: 1000
  
  level_2:  # Regional level
    north_america:
      - name: "US East Cluster"
        regions: 500
        specialization: ["commercial", "educational"]
        
      - name: "US West Cluster"
        regions: 300
        specialization: ["entertainment", "gaming"]
        
      - name: "Canada Cluster"
        regions: 200
        specialization: ["government", "healthcare"]
    
    europe:
      - name: "Western Europe Cluster"
        regions: 400
        specialization: ["arts", "culture"]
        
      - name: "Central Europe Cluster"
        regions: 300
        specialization: ["business", "finance"]
    
    asia_pacific:
      - name: "Japan Cluster"
        regions: 250
        specialization: ["technology", "research"]
        
      - name: "Australia Cluster"
        regions: 150
        specialization: ["mining", "agriculture"]

communication:
  inter_level: encrypted_tunnels
  intra_level: mesh_network
  cross_continental: compressed_streams
  
performance:
  max_global_latency: 200ms
  max_regional_latency: 50ms
  bandwidth_allocation: adaptive
  
failover:
  global_redundancy: 3
  regional_redundancy: 2
  automatic_rerouting: true
  data_replication: synchronous
EOF

# Apply hierarchical configuration
cargo run --bin network-manager -- apply-topology \
  --config /opt/opensim/topology-hierarchical.yaml \
  --enable-global-monitoring \
  --setup-redundancy
```

## 5.3 Encryption and Security Configuration

### Advanced Encryption Settings

Configure military-grade encryption for maximum security:

```bash
# Create advanced encryption configuration
cat > /opt/opensim/encryption-config.yaml << EOF
encryption:
  primary_algorithm: AES-256-GCM
  key_derivation: PBKDF2-SHA256
  iterations: 100000
  salt_length: 32
  
key_management:
  rotation_interval: 6h
  overlap_period: 30m
  perfect_forward_secrecy: true
  quantum_resistance: true
  
algorithms:
  symmetric:
    - AES-256-GCM  # Primary
    - ChaCha20-Poly1305  # Fallback
  
  key_exchange:
    - X25519  # Primary
    - ECDH-P256  # Fallback
  
  hashing:
    - SHA-256  # Primary
    - Blake2b  # Fallback
  
  signatures:
    - Ed25519  # Primary
    - ECDSA-P256  # Fallback

tunnels:
  protocol: TLS-1.3
  cipher_suites:
    - TLS_AES_256_GCM_SHA384
    - TLS_CHACHA20_POLY1305_SHA256
  
  verification:
    certificate_pinning: true
    ocsp_stapling: true
    transparency_logs: true

performance:
  hardware_acceleration: true
  batch_operations: true
  zero_copy: true
  memory_protection: true
EOF

# Apply encryption configuration
cargo run --bin security-manager -- apply-encryption \
  --config /opt/opensim/encryption-config.yaml \
  --enable-hardware-accel
```

### Security Policies

```bash
# Create comprehensive security policies
cat > /opt/opensim/security-policies.yaml << EOF
access_control:
  default_policy: deny_all
  authentication_required: true
  authorization_layers: 3
  
identity_verification:
  certificate_validation: strict
  revocation_checking: online
  identity_binding: cryptographic
  session_management: secure

network_security:
  ingress_filtering: strict
  egress_monitoring: enabled
  anomaly_detection: ml_enhanced
  intrusion_prevention: active
  
data_protection:
  encryption_at_rest: true
  encryption_in_transit: true
  key_escrow: false
  data_classification: automatic
  
compliance:
  frameworks:
    - SOC2_TYPE2
    - ISO27001
    - GDPR
    - HIPAA_optional
  
  auditing:
    all_access: logged
    retention_period: 7_years
    integrity_protection: cryptographic
    
monitoring:
  security_events: real_time
  performance_impact: minimized
  alert_thresholds: adaptive
  incident_response: automated
EOF

# Apply security policies
cargo run --bin security-manager -- apply-policies \
  --config /opt/opensim/security-policies.yaml \
  --enable-compliance-mode
```

## 5.4 Real-Time Communication Protocols

### Avatar Crossing Protocol

Configure secure avatar movement between regions:

```bash
# Create avatar crossing configuration
cat > /opt/opensim/avatar-crossing.yaml << EOF
avatar_crossing:
  protocol_version: "2.0"
  encryption: mandatory
  authentication: mutual_tls
  
security:
  avatar_verification: cryptographic
  state_validation: comprehensive
  asset_verification: hash_based
  permissions_check: strict
  
performance:
  timeout_seconds: 30
  retry_attempts: 3
  backoff_strategy: exponential
  compression: true
  
state_transfer:
  components:
    - avatar_appearance
    - inventory_cache
    - animation_state
    - physics_properties
    - script_state
    - attachment_data
  
  validation:
    checksum_verification: true
    schema_validation: true
    size_limits: enforced
    content_filtering: enabled

regions:
  source_region:
    preparation_phase: 5s
    cleanup_timeout: 60s
    rollback_capability: true
    
  target_region:
    acceptance_criteria: strict
    resource_reservation: true
    conflict_resolution: prioritized
    
monitoring:
  crossing_analytics: enabled
  performance_metrics: detailed
  failure_tracking: comprehensive
  user_experience_monitoring: true
EOF

# Apply avatar crossing configuration
cargo run --bin protocol-manager -- configure-avatar-crossing \
  --config /opt/opensim/avatar-crossing.yaml \
  --enable-analytics
```

### Object Transfer System

Secure object sharing across regions:

```bash
# Create object transfer configuration
cat > /opt/opensim/object-transfer.yaml << EOF
object_transfer:
  protocol: secure_stream
  encryption: AES-256-GCM
  integrity: SHA-256
  
transfer_types:
  - temporary_objects
  - permanent_objects
  - scripted_objects
  - physical_objects
  - linked_objects
  
security:
  ownership_verification: required
  permissions_inheritance: strict
  content_scanning: enabled
  size_limits:
    max_object_size: "100MB"
    max_script_memory: "1MB"
    max_textures_per_object: 50
  
streaming:
  chunk_size: "1MB"
  parallel_streams: 4
  compression: true
  resume_capability: true
  
validation:
  asset_integrity: cryptographic
  script_validation: sandboxed
  physics_validation: enabled
  content_policy: enforced
  
performance:
  bandwidth_limit: "50Mbps"
  priority_queuing: true
  cache_optimization: true
  deduplication: true

monitoring:
  transfer_analytics: enabled
  bandwidth_monitoring: real_time
  error_tracking: detailed
  user_satisfaction: tracked
EOF

# Apply object transfer configuration
cargo run --bin protocol-manager -- configure-object-transfer \
  --config /opt/opensim/object-transfer.yaml \
  --enable-streaming
```

### Grid-Wide Event Broadcasting

Real-time event distribution across the virtual world:

```bash
# Create event broadcasting configuration
cat > /opt/opensim/event-broadcasting.yaml << EOF
event_broadcasting:
  architecture: publish_subscribe
  encryption: end_to_end
  delivery: guaranteed
  
event_types:
  - chat_messages
  - system_announcements
  - region_events
  - user_presence
  - grid_status
  - emergency_alerts
  
channels:
  global_chat:
    encryption: true
    moderation: automatic
    rate_limiting: adaptive
    history_retention: 30d
    
  region_events:
    scope: local_and_neighbors
    priority: high
    batching: true
    compression: true
    
  system_notifications:
    scope: grid_wide
    priority: critical
    bypass_filters: true
    persistent: true

delivery:
  strategies:
    - real_time
    - store_and_forward
    - priority_based
    - geographic_routing
  
  guarantees:
    delivery_confirmation: required
    ordering: causal
    deduplication: automatic
    offline_storage: 7d

performance:
  max_latency: 100ms
  throughput: "10000 events/second"
  fan_out: unlimited
  load_balancing: automatic
  
monitoring:
  event_analytics: comprehensive
  delivery_tracking: detailed
  performance_monitoring: real_time
  capacity_planning: predictive
EOF

# Apply event broadcasting configuration
cargo run --bin protocol-manager -- configure-event-broadcasting \
  --config /opt/opensim/event-broadcasting.yaml \
  --enable-global-scope
```

## 5.5 Performance Optimization

### Network Performance Tuning

```bash
# Create network performance configuration
cat > /opt/opensim/network-performance.yaml << EOF
performance:
  optimization_targets:
    - latency_minimization
    - throughput_maximization
    - connection_efficiency
    - resource_utilization
  
tcp_optimization:
  congestion_control: bbr
  window_scaling: true
  selective_ack: true
  fast_retransmit: true
  nagle_disabled: true
  
buffer_management:
  send_buffer: "2MB"
  receive_buffer: "2MB"
  socket_buffer: "4MB"
  application_buffer: "8MB"
  zero_copy: true
  
connection_pooling:
  max_connections_per_region: 100
  idle_timeout: 300s
  keep_alive: true
  connection_reuse: aggressive
  
compression:
  algorithm: lz4
  level: 1  # Fast compression
  threshold: 1024  # Compress messages > 1KB
  streaming: true
  
batching:
  enabled: true
  max_batch_size: 50
  max_delay: 5ms
  adaptive_sizing: true
  
load_balancing:
  algorithm: least_latency
  health_checking: continuous
  failover_time: 1s
  geographic_preference: true

monitoring:
  performance_metrics:
    - latency_percentiles
    - throughput_rates
    - connection_counts
    - error_rates
    - resource_utilization
  
  alerting:
    latency_threshold: 100ms
    throughput_threshold: "1Gbps"
    error_rate_threshold: 0.1%
    resource_threshold: 80%
EOF

# Apply performance configuration
cargo run --bin performance-manager -- optimize-network \
  --config /opt/opensim/network-performance.yaml \
  --enable-monitoring
```

### Quality of Service (QoS)

```bash
# Create QoS configuration
cat > /opt/opensim/qos-config.yaml << EOF
qos:
  traffic_classes:
    critical:
      priority: 1
      bandwidth_guarantee: "100Mbps"
      max_latency: 10ms
      traffic_types:
        - avatar_movement
        - emergency_alerts
        - system_control
    
    interactive:
      priority: 2
      bandwidth_guarantee: "500Mbps"
      max_latency: 50ms
      traffic_types:
        - chat_messages
        - object_interaction
        - user_interface
    
    streaming:
      priority: 3
      bandwidth_guarantee: "1Gbps"
      max_latency: 100ms
      traffic_types:
        - asset_transfer
        - voice_data
        - video_streams
    
    background:
      priority: 4
      bandwidth_guarantee: "100Mbps"
      max_latency: 1000ms
      traffic_types:
        - bulk_transfers
        - backups
        - analytics
  
shaping:
  rate_limiting: per_user_and_global
  burst_allowance: "10MB"
  fair_queuing: true
  priority_scheduling: strict
  
admission_control:
  connection_limits: enforced
  bandwidth_allocation: dynamic
  resource_reservation: true
  congestion_control: active
  
monitoring:
  qos_metrics: real_time
  sla_tracking: enabled
  performance_reports: automated
  capacity_planning: predictive
EOF

# Apply QoS configuration
cargo run --bin qos-manager -- configure \
  --config /opt/opensim/qos-config.yaml \
  --enable-traffic-shaping
```

## 5.6 Monitoring and Analytics

### Network Analytics Dashboard

```bash
# Create monitoring configuration
cat > /opt/opensim/monitoring-config.yaml << EOF
monitoring:
  collection_interval: 10s
  retention_period: 90d
  aggregation_levels:
    - real_time
    - hourly
    - daily
    - weekly
  
metrics:
    network:
      - connection_count
      - bandwidth_utilization
      - latency_distribution
      - packet_loss_rate
      - jitter_measurements
      
    security:
      - encryption_status
      - authentication_failures
      - policy_violations
      - anomaly_scores
      - threat_detections
      
    performance:
      - throughput_rates
      - response_times
      - resource_utilization
      - error_rates
      - availability_metrics
      
    application:
      - avatar_crossings
      - object_transfers
      - chat_volume
      - region_load
      - user_experience

visualization:
  dashboards:
    - network_overview
    - security_status
    - performance_analytics
    - capacity_planning
    - incident_response
  
  alerts:
    delivery_methods:
      - email
      - slack
      - webhook
      - sms
    
    severity_levels:
      - critical
      - warning
      - info
    
    escalation_policies:
      - immediate
      - business_hours
      - follow_sun

reporting:
  scheduled_reports:
    - daily_summary
    - weekly_performance
    - monthly_capacity
    - quarterly_security
  
  ad_hoc_queries: enabled
  data_export: multiple_formats
  compliance_reports: automated
EOF

# Apply monitoring configuration
cargo run --bin monitoring-manager -- setup \
  --config /opt/opensim/monitoring-config.yaml \
  --enable-dashboards
```

### Real-Time Analytics

```bash
# Install Grafana and Prometheus for monitoring
docker-compose up -d <<EOF
version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    container_name: opensim-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=90d'
      - '--web.enable-lifecycle'
  
  grafana:
    image: grafana/grafana:latest
    container_name: opensim-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
  
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.0
    container_name: opensim-elasticsearch
    ports:
      - "9200:9200"
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
    volumes:
      - elasticsearch-data:/usr/share/elasticsearch/data
  
  kibana:
    image: docker.elastic.co/kibana/kibana:8.11.0
    container_name: opensim-kibana
    ports:
      - "5601:5601"
    environment:
      - ELASTICSEARCH_HOSTS=http://elasticsearch:9200

volumes:
  prometheus-data:
  grafana-data:
  elasticsearch-data:
EOF

# Configure Grafana dashboards
curl -X POST http://admin:admin@localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @/opt/opensim/grafana-opensim-dashboard.json
```

## 5.7 Troubleshooting Network Issues

### Common Network Problems

#### High Latency Issues

**Problem**: Network latency exceeding acceptable thresholds

**Symptoms**:
```
Avatar crossing delays > 5 seconds
Chat messages delayed > 1 second
Object rez time > 10 seconds
```

**Diagnosis**:
```bash
# Check network performance metrics
cargo run --bin network-diagnostics -- latency-analysis \
  --regions all \
  --duration 1h \
  --output detailed

# Trace network paths
cargo run --bin network-diagnostics -- trace-routes \
  --source region-a \
  --targets all-regions \
  --protocol encrypted-overlay

# Analyze encryption overhead
cargo run --bin performance-analyzer -- encryption-impact \
  --baseline unencrypted \
  --current encrypted \
  --duration 30m
```

**Solutions**:
```bash
# Optimize network topology
cargo run --bin topology-optimizer -- recommend \
  --current-config /opt/opensim/current-topology.yaml \
  --performance-target low-latency

# Enable compression for large transfers
cargo run --bin network-manager -- enable-compression \
  --algorithms lz4,zstd \
  --threshold 1024

# Implement edge caching
cargo run --bin cache-manager -- deploy-edge-cache \
  --locations geographic-distribution \
  --size 10GB
```

#### Connection Failures

**Problem**: Intermittent connection failures between regions

**Diagnosis**:
```bash
# Monitor connection health
cargo run --bin health-checker -- monitor \
  --connections inter-region \
  --interval 5s \
  --alerts enabled

# Check certificate validity
cargo run --bin security-diagnostics -- verify-certificates \
  --all-services \
  --check-expiration \
  --validate-chains

# Analyze network partitions
cargo run --bin network-diagnostics -- partition-analysis \
  --topology current \
  --simulate-failures
```

**Solutions**:
```bash
# Implement connection redundancy
cargo run --bin network-manager -- add-redundancy \
  --type multi-path \
  --backup-routes 2 \
  --failover-time 5s

# Configure automatic reconnection
cargo run --bin connection-manager -- configure-reconnect \
  --strategy exponential-backoff \
  --max-attempts 10 \
  --base-delay 1s
```

#### Performance Degradation

**Problem**: Gradual performance degradation over time

**Diagnosis**:
```bash
# Performance trend analysis
cargo run --bin analytics-engine -- trend-analysis \
  --metrics throughput,latency,error_rate \
  --timeframe 30d \
  --detect-anomalies

# Resource utilization tracking
cargo run --bin resource-monitor -- utilization-report \
  --components network,cpu,memory \
  --period 24h \
  --identify-bottlenecks

# Connection pool analysis
cargo run --bin connection-analyzer -- pool-health \
  --check-leaks \
  --validate-cleanup \
  --optimize-sizing
```

**Solutions**:
```bash
# Optimize resource allocation
cargo run --bin resource-optimizer -- rebalance \
  --target-utilization 70% \
  --consider-growth-trends \
  --schedule-maintenance

# Update network configurations
cargo run --bin config-manager -- optimize \
  --based-on-metrics \
  --apply-automatically \
  --backup-current
```

## 5.8 Security Auditing and Compliance

### Security Audit Framework

```bash
# Create security audit configuration
cat > /opt/opensim/security-audit.yaml << EOF
security_audit:
  scope: comprehensive
  frequency: daily
  compliance_frameworks:
    - SOC2_TYPE2
    - ISO27001
    - NIST_CSF
    - GDPR
  
audit_components:
  encryption:
    - algorithm_strength
    - key_rotation
    - perfect_forward_secrecy
    - quantum_readiness
  
  access_control:
    - authentication_mechanisms
    - authorization_policies
    - session_management
    - privilege_escalation
  
  network_security:
    - traffic_encryption
    - intrusion_detection
    - anomaly_monitoring
    - vulnerability_scanning
  
  data_protection:
    - data_classification
    - encryption_at_rest
    - backup_security
    - retention_policies

reporting:
  formats:
    - executive_summary
    - technical_details
    - compliance_report
    - remediation_plan
  
  distribution:
    - security_team
    - compliance_officer
    - executive_leadership
    - external_auditors

remediation:
  priority_levels:
    - critical
    - high
    - medium
    - low
  
  response_times:
    critical: 4h
    high: 24h
    medium: 72h
    low: 30d
  
  tracking:
    - issue_identification
    - remediation_progress
    - verification_testing
    - documentation_updates
EOF

# Run comprehensive security audit
cargo run --bin security-auditor -- comprehensive-audit \
  --config /opt/opensim/security-audit.yaml \
  --generate-report \
  --schedule-remediation
```

### Compliance Monitoring

```bash
# Set up continuous compliance monitoring
cargo run --bin compliance-monitor -- setup \
  --frameworks SOC2,ISO27001,GDPR \
  --monitoring-interval 1h \
  --alert-on-violations \
  --auto-remediation partial

# Generate compliance reports
cargo run --bin compliance-reporter -- generate \
  --framework SOC2_TYPE2 \
  --period quarterly \
  --include-evidence \
  --external-auditor-ready
```

---

**Encrypted Overlay Network Setup Complete!** 🎉

Your OpenSim Next deployment now features the world's most advanced encrypted overlay network for virtual world communication. The system provides enterprise-grade security, real-time performance, and global scalability for secure multi-region virtual world deployments. The next chapter will guide you through multi-physics engine selection and configuration.

---

# Chapter 6: Multi-Physics Engine Configuration

This chapter provides comprehensive configuration instructions for OpenSim Next's revolutionary **multi-physics engine system**. OpenSim Next is the first virtual world platform to support **5 different physics engines** with **per-region selection**, enabling optimal performance and features based on specific use cases.

## 6.1 Multi-Physics Engine Overview

OpenSim Next's groundbreaking multi-physics architecture allows each region to independently choose the most suitable physics engine, enabling unprecedented flexibility and optimization for different virtual world scenarios.

### Physics Engine Comparison

| Engine | Max Bodies | Soft Bodies | Fluids | Particles | GPU Accel | Best For |
|--------|------------|-------------|--------|-----------|-----------|----------|
| **ODE** | 10,000 | ❌ | ❌ | ❌ | ❌ | Stable avatars, traditional content |
| **UBODE** | 20,000 | ❌ | ❌ | ❌ | ❌ | Large worlds, many objects |
| **Bullet** | 50,000 | ✅ | ❌ | Limited | ❌ | Vehicles, advanced physics |
| **POS** | 100,000 | ✅ | ✅ | ✅ | ✅ | Particles, fluids, cloth |
| **Basic** | 1,000 | ❌ | ❌ | ❌ | ❌ | Testing, lightweight scenarios |

### Revolutionary Features

- **🔄 Runtime Engine Switching**: Change physics engines without server restart
- **🎯 Per-Region Optimization**: Each region uses the optimal engine for its content
- **📊 Automatic Recommendations**: AI-powered engine selection based on content analysis
- **⚡ GPU Acceleration**: Advanced engines support GPU computing for massive simulations
- **🔗 Unified API**: Consistent interface across all physics engines
- **📈 Performance Monitoring**: Real-time metrics and optimization recommendations

### Multi-Physics Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                OpenSim Next Multi-Physics System               │
├─────────────────────────────────────────────────────────────────┤
│                    Physics Engine Manager                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Engine Selector │  │ Performance     │  │ Configuration   │ │
│  │ & Optimizer     │  │ Monitor         │  │ Manager         │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Unified Physics API                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Common          │  │ Engine          │  │ Performance     │ │
│  │ Interface       │  │ Abstraction     │  │ Profiler        │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                     Physics Engines                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Region A        │  │ Region B        │  │ Region C        │ │
│  │ ODE Engine      │  │ Bullet Engine   │  │ POS Engine      │ │
│  │ (Avatars)       │  │ (Vehicles)      │  │ (Particles)     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Region D        │  │ Region E        │  │ GPU Compute     │ │
│  │ UBODE Engine    │  │ Basic Engine    │  │ Acceleration    │ │
│  │ (Large Worlds)  │  │ (Testing)       │  │ (POS Engine)    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Physics Library Requirements

The physics engine is implemented in Zig and compiled to a shared library. This library must be on the dynamic library path at runtime:

```bash
# macOS — both Zig physics and BulletSim libraries needed
export DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64

# Linux
export LD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64
```

If the library is missing, the server will crash on startup with a `dyld: missing symbol` error. Always rebuild the Zig library after source changes:

```bash
cd zig && zig build && cd ..
```

The physics engine is a global singleton accessed via vtable dispatch. The `Physics::with_engine()` function selects the active engine. Only one physics instance should exist per process.

## 6.2 Physics Engine Selection Guide

### Use Case Matrix

| Scenario | Recommended Engine | Reason | Configuration |
|----------|-------------------|--------|---------------|
| **Avatar Regions** | ODE | Stable, responsive movement | High timestep, low damping |
| **Vehicle Sandbox** | Bullet | Advanced rigid body dynamics | Continuous collision detection |
| **Large Builds** | UBODE | Optimized for many objects | Spatial partitioning |
| **Particle Effects** | POS | GPU-accelerated particles | High particle count |
| **Water/Fluids** | POS | SPH fluid dynamics | Real-time fluid simulation |
| **Social Spaces** | ODE | Stable avatar interactions | Balanced performance |
| **Educational** | Bullet | Realistic physics behavior | Accurate simulation |
| **Gaming** | Bullet | Advanced collision detection | High-performance settings |
| **Art Installations** | POS | Creative particle systems | GPU acceleration |
| **Testing/Development** | Basic | Minimal overhead | Fast iteration |

### Performance Characteristics

#### ODE Engine
```
Strengths:
- Rock-solid stability for avatars
- Excellent network synchronization
- Low CPU overhead
- Proven reliability (15+ years)

Optimal For:
- Social regions with many avatars
- Traditional SL-style content
- Stable platform experiences
- Regions with frequent teleports

Configuration Focus:
- Avatar responsiveness
- Network sync accuracy
- Collision stability
```

#### UBODE Engine
```
Strengths:
- Enhanced ODE with optimizations
- Better performance with many objects
- Improved spatial partitioning
- Maintains ODE compatibility

Optimal For:
- Large architectural builds
- Regions with 10,000+ objects
- Shopping malls and exhibitions
- Complex static environments

Configuration Focus:
- Object count optimization
- Memory efficiency
- Spatial indexing
```

#### Bullet Physics Engine
```
Strengths:
- Advanced rigid body dynamics
- Soft body simulation
- Continuous collision detection
- Constraint solving

Optimal For:
- Vehicle simulations
- Realistic physics demonstrations
- Interactive mechanical systems
- Advanced collision scenarios

Configuration Focus:
- Solver precision
- Collision accuracy
- Dynamic constraints
```

#### POS (Position-Based Dynamics) Engine
```
Strengths:
- GPU acceleration support
- Real-time fluid simulation
- Massive particle systems
- Soft body deformation

Optimal For:
- Particle effects and art
- Fluid dynamics simulations
- Cloth and soft materials
- GPU-accelerated scenarios

Configuration Focus:
- GPU utilization
- Particle density
- Fluid parameters
```

#### Basic Engine
```
Strengths:
- Minimal resource usage
- Fast startup times
- Simple collision detection
- No external dependencies

Optimal For:
- Development testing
- Lightweight demonstrations
- Resource-constrained environments
- Simple collision scenarios

Configuration Focus:
- Resource conservation
- Fast iteration
- Basic collision only
```

## 6.3 Global Physics Configuration

### Main Configuration File

Update `config/opensim.ini` for multi-physics support:

```ini
[Architecture]
; Enable multi-physics engine support
physics_engine_selection = per_region
default_physics_engine = ODE

; Physics engine configuration
physics_config = config-include/opensim-next/Physics.ini

; Enable GPU acceleration (for POS engine)
enable_gpu_physics = true
gpu_device_preference = discrete

; Performance monitoring
physics_monitoring = true
physics_metrics_interval = 30s

[MultiPhysics]
; Engine availability
available_engines = ODE,UBODE,Bullet,POS,Basic
engine_switching = runtime_allowed
engine_validation = strict

; Global physics settings
timestep = 0.0167  ; 60 FPS
max_substeps = 5
physics_frame_rate = 60
adaptive_timestep = true

; Memory management
physics_memory_limit = 4GB
object_cleanup_interval = 300s
garbage_collection = aggressive

; Performance limits
max_objects_per_region = 50000
max_active_bodies = 10000
physics_cpu_limit = 80%

[PhysicsEngines]
; ODE Engine settings
ODE_enabled = true
ODE_solver = quickstep
ODE_iterations = 10
ODE_damping = 0.02

; UBODE Engine settings
UBODE_enabled = true
UBODE_spatial_optimization = true
UBODE_object_limit = 20000

; Bullet Engine settings
Bullet_enabled = true
Bullet_solver_type = sequential_impulse
Bullet_collision_margin = 0.01

; POS Engine settings
POS_enabled = true
POS_gpu_acceleration = true
POS_particle_limit = 100000

; Basic Engine settings
Basic_enabled = true
Basic_collision_only = true
```

### Advanced Physics Configuration

Create `config-include/opensim-next/Physics.ini`:

```ini
; Multi-Physics Engine Configuration
; This file configures all available physics engines

[Global]
; Global physics settings that apply to all engines
world_gravity = -9.8
physics_quality = high
debug_rendering = false
performance_profiling = true

; Memory management
memory_pool_size = 2GB
object_recycling = true
garbage_collection_frequency = 300s

; Threading
physics_thread_count = auto  ; Detects CPU cores
thread_affinity = performance_cores
parallel_processing = true

[ODE]
; Open Dynamics Engine configuration
enabled = true
world_cfm = 0.0001  ; Constraint force mixing
world_erp = 0.2     ; Error reduction parameter

; Solver settings
solver_type = quickstep
solver_iterations = 10
contact_max_correcting_vel = 100.0
contact_surface_layer = 0.001

; Collision detection
collision_contacts_per_geom = 3
collision_geometry_margin = 0.01
body_auto_disable = true
body_linear_threshold = 0.01
body_angular_threshold = 0.01

; Performance tuning
max_bodies = 10000
max_joints = 5000
max_contacts = 10000
space_type = hash_space
space_hash_levels = 2

; Avatar-specific settings
avatar_density = 3.5
avatar_damping = 0.02
avatar_capsule_radius = 0.37
avatar_capsule_height = 1.75

[UBODE]
; Enhanced ODE with optimizations
enabled = true
base_engine = ODE
optimization_level = high

; Spatial optimization
spatial_partitioning = true
partition_type = octree
partition_depth = 6
dynamic_partitioning = true

; Object management
static_object_optimization = true
sleeping_object_detection = true
level_of_detail = true
distance_culling = true

; Memory optimization
object_pooling = true
collision_cache = true
memory_compaction = true

; Performance scaling
auto_quality_adjustment = true
frame_rate_target = 60
cpu_usage_target = 70%

[Bullet]
; Bullet Physics Engine configuration
enabled = true
solver_type = sequential_impulse
solver_iterations = 10

; Collision detection
collision_algorithm = convex_cast
continuous_collision_detection = true
collision_margin = 0.01
collision_filter = optimized

; Rigid body dynamics
rigid_body_linear_damping = 0.04
rigid_body_angular_damping = 0.1
rigid_body_deactivation = true
deactivation_time = 2.0

; Soft body physics (if enabled)
soft_body_enabled = true
soft_body_position_iterations = 4
soft_body_drift_compensation = 0.005
soft_body_cluster_iterations = 8

; Vehicle simulation
vehicle_constraint_enabled = true
vehicle_suspension_stiffness = 20.0
vehicle_suspension_damping = 2.3
vehicle_friction_slip = 10.5

; Advanced features
constraint_solver_type = projected_gauss_seidel
split_impulse = true
warm_starting = true

[POS]
; Position-Based Dynamics Engine
enabled = true
gpu_acceleration = true
gpu_device = auto  ; auto, discrete, integrated

; Particle systems
max_particles = 100000
particle_radius = 0.05
particle_mass = 1.0
particle_damping = 0.99

; Fluid dynamics (SPH)
fluid_simulation = true
fluid_density = 1000.0
fluid_viscosity = 0.01
fluid_surface_tension = 0.07
kernel_radius = 0.2
smoothing_length = 0.1

; Soft body dynamics
soft_body_enabled = true
bending_stiffness = 0.1
stretching_stiffness = 1.0
damping_coefficient = 0.01

; GPU compute settings
gpu_work_group_size = 256
gpu_memory_allocation = 1GB
gpu_computation_precision = float32

; Constraints
constraint_iterations = 4
constraint_relaxation = 1.0
collision_response = 0.8

; Integration
integration_method = verlet
substeps = 4
adaptive_substeps = true
time_step = 0.016667  ; 1/60

[Basic]
; Basic Physics Engine (minimal)
enabled = true
collision_only = true
simple_collision_detection = true

; Minimal settings
max_objects = 1000
collision_precision = low
no_dynamics = true
static_world = true

; Performance
minimal_cpu_usage = true
no_threading = true
simple_shapes_only = true

[Debugging]
; Debug and profiling settings
debug_mode = false
visual_debugging = false
performance_counters = true
memory_tracking = true

; Logging
physics_log_level = INFO
log_file = logs/physics.log
log_rotation = daily
log_max_size = 100MB

; Profiling
enable_profiler = true
profiler_output = logs/physics_profile.json
profile_frequency = 60s

; Validation
physics_validation = true
nan_detection = true
infinite_detection = true
constraint_validation = true
```

## 6.4 Per-Region Physics Configuration

### Region-Specific Engine Selection

Configure physics engines per region in `Regions/RegionConfig.ini`:

```ini
[Region: Welcome Area]
RegionUUID = 00000000-0000-0000-0000-000000000001
Location = 1000,1000
SizeX = 256
SizeY = 256

; Physics engine selection
PhysicsEngine = ODE
PhysicsConfig = avatars_optimized

; Performance settings
MaxPhysicalPrims = 5000
MaxAvatars = 100
PhysicsQuality = high
PhysicsTimestep = 0.0167

[Region: Vehicle Sandbox]
RegionUUID = 00000000-0000-0000-0000-000000000002
Location = 1001,1000
SizeX = 512
SizeY = 512

; Physics engine selection
PhysicsEngine = Bullet
PhysicsConfig = vehicles_advanced

; Performance settings
MaxPhysicalPrims = 10000
MaxAvatars = 50
PhysicsQuality = ultra
PhysicsTimestep = 0.0111  ; 90 FPS for smooth vehicles

[Region: Particle Gallery]
RegionUUID = 00000000-0000-0000-0000-000000000003
Location = 1000,1001
SizeX = 256
SizeY = 256

; Physics engine selection
PhysicsEngine = POS
PhysicsConfig = particles_gpu

; Performance settings
MaxPhysicalPrims = 2000
MaxParticles = 50000
MaxAvatars = 30
PhysicsQuality = ultra
GPUAcceleration = true

[Region: Mega Build]
RegionUUID = 00000000-0000-0000-0000-000000000004
Location = 1002,1000
SizeX = 1024
SizeY = 1024

; Physics engine selection
PhysicsEngine = UBODE
PhysicsConfig = large_world

; Performance settings
MaxPhysicalPrims = 25000
MaxAvatars = 200
PhysicsQuality = balanced
SpatialOptimization = true

[Region: Test Lab]
RegionUUID = 00000000-0000-0000-0000-000000000005
Location = 1003,1000
SizeX = 256
SizeY = 256

; Physics engine selection
PhysicsEngine = Basic
PhysicsConfig = testing

; Performance settings
MaxPhysicalPrims = 500
MaxAvatars = 10
PhysicsQuality = minimal
FastStartup = true
```

### Physics Engine Profiles

Create optimized configurations for common scenarios:

```bash
# Create physics profiles directory
mkdir -p /opt/opensim/physics-profiles

# Avatar-optimized profile (ODE)
cat > /opt/opensim/physics-profiles/avatars_optimized.ini << EOF
[ODE]
; Optimized for avatar movement and social interaction
solver_iterations = 12
contact_max_correcting_vel = 50.0
body_linear_threshold = 0.005
body_angular_threshold = 0.005

; Avatar tuning
avatar_density = 4.0
avatar_damping = 0.015
avatar_capsule_radius = 0.35
avatar_capsule_height = 1.70

; Social space optimization
max_avatars_physics = 150
avatar_collision_priority = high
network_sync_frequency = 30Hz
prediction_enabled = true
EOF

# Vehicle-optimized profile (Bullet)
cat > /opt/opensim/physics-profiles/vehicles_advanced.ini << EOF
[Bullet]
; Optimized for realistic vehicle simulation
solver_iterations = 15
continuous_collision_detection = true
collision_margin = 0.005

; Vehicle dynamics
vehicle_constraint_enabled = true
vehicle_suspension_stiffness = 25.0
vehicle_suspension_damping = 3.0
vehicle_friction_slip = 15.0
vehicle_max_suspension_travel = 0.3

; Wheel physics
wheel_friction_coefficient = 1.2
wheel_rolling_resistance = 0.02
wheel_slip_threshold = 0.8

; Stability
constraint_breaking_threshold = 100.0
motor_impulse_limit = 1000.0
steering_response = 0.3
EOF

# GPU particle profile (POS)
cat > /opt/opensim/physics-profiles/particles_gpu.ini << EOF
[POS]
; GPU-accelerated particle systems
gpu_acceleration = true
gpu_device = discrete
gpu_memory_allocation = 2GB

; Particle configuration
max_particles = 75000
particle_radius = 0.04
particle_mass = 0.8
particle_lifetime = 30.0

; Fluid dynamics
fluid_simulation = true
fluid_density = 1200.0
fluid_viscosity = 0.008
sph_kernel_radius = 0.15
sph_pressure_stiffness = 3.0

; GPU optimization
work_group_size = 512
memory_coalescing = true
compute_shader_version = 5.0
parallel_reduction = true
EOF

# Large world profile (UBODE)
cat > /opt/opensim/physics-profiles/large_world.ini << EOF
[UBODE]
; Optimized for large builds with many objects
spatial_partitioning = true
partition_type = adaptive_octree
partition_depth = 8
dynamic_rebalancing = true

; Object management
static_object_optimization = true
sleeping_threshold = 2.0
wakeup_threshold = 0.1
object_pooling = aggressive

; Memory optimization
memory_compaction_frequency = 60s
garbage_collection = generational
object_cache_size = 50MB

; Performance scaling
auto_lod = true
distance_culling_range = 512.0
frustum_culling = true
occlusion_culling = true
EOF

# Testing profile (Basic)
cat > /opt/opensim/physics-profiles/testing.ini << EOF
[Basic]
; Minimal physics for rapid testing
collision_only = true
no_dynamics = true
simple_shapes_only = true

; Fast iteration
startup_time_optimization = true
minimal_validation = true
no_threading = true
cache_disabled = true

; Debug features
debug_collisions = true
collision_wireframe = true
timing_output = true
memory_reporting = true
EOF
```

## 6.5 Runtime Engine Management

### Engine Switching Commands

OpenSim Next supports runtime physics engine switching without server restart:

```bash
# List available physics engines
cargo run --bin physics-manager -- list-engines

# Check current engine for a region
cargo run --bin physics-manager -- get-engine \
  --region "Welcome Area"

# Switch physics engine for a region
cargo run --bin physics-manager -- switch-engine \
  --region "Welcome Area" \
  --engine Bullet \
  --profile vehicles_advanced \
  --validate-before-switch

# Get engine performance statistics
cargo run --bin physics-manager -- get-stats \
  --region "Welcome Area" \
  --duration 5m \
  --format detailed

# Optimize engine configuration automatically
cargo run --bin physics-manager -- auto-optimize \
  --region "Welcome Area" \
  --target balanced \
  --analyze-content

# Bulk engine management
cargo run --bin physics-manager -- batch-operation \
  --config bulk-engine-config.yaml \
  --validate-all \
  --rollback-on-failure
```

### Engine Switching Configuration

```bash
# Create engine switching configuration
cat > /opt/opensim/bulk-engine-config.yaml << EOF
engine_operations:
  - regions: ["Welcome Area", "Social Hub"]
    action: switch
    target_engine: ODE
    profile: avatars_optimized
    reason: "Social spaces need stable avatar physics"
    
  - regions: ["Vehicle Sandbox", "Racing Track"]
    action: switch
    target_engine: Bullet
    profile: vehicles_advanced
    reason: "Vehicle regions need advanced rigid body dynamics"
    
  - regions: ["Art Gallery", "Particle Demo"]
    action: switch
    target_engine: POS
    profile: particles_gpu
    reason: "Art installations benefit from GPU particles"
    
  - regions: ["Mega Mall", "Convention Center"]
    action: switch
    target_engine: UBODE
    profile: large_world
    reason: "Large builds need optimized object handling"

validation:
  check_prerequisites: true
  validate_content_compatibility: true
  performance_baseline: required
  rollback_on_failure: true
  
scheduling:
  execution_time: maintenance_window
  stagger_delay: 30s
  monitor_after_switch: 5m
  
notifications:
  notify_users: true
  notification_lead_time: 10m
  status_updates: true
EOF

# Execute bulk engine switching
cargo run --bin physics-manager -- batch-operation \
  --config /opt/opensim/bulk-engine-config.yaml \
  --dry-run  # Test first, then remove --dry-run
```

### Automatic Engine Recommendation

```bash
# Analyze region content and recommend optimal engines
cargo run --bin physics-analyzer -- analyze-regions \
  --all-regions \
  --content-analysis deep \
  --performance-history 30d \
  --generate-recommendations

# Auto-configure based on content analysis
cargo run --bin physics-manager -- auto-configure \
  --region "New Region" \
  --analyze-content \
  --consider-neighbors \
  --apply-best-practices

# Performance-based engine selection
cargo run --bin physics-optimizer -- recommend-engine \
  --region "Performance Test" \
  --target-framerate 60 \
  --max-cpu-usage 70% \
  --consider-gpu-availability
```

## 6.6 Performance Monitoring and Optimization

### Real-Time Physics Monitoring

```bash
# Create physics monitoring configuration
cat > /opt/opensim/physics-monitoring.yaml << EOF
monitoring:
  collection_interval: 1s
  retention_period: 30d
  real_time_dashboard: true
  
metrics:
  performance:
    - physics_fps
    - frame_time_ms
    - cpu_usage_percent
    - memory_usage_mb
    - gpu_usage_percent  # POS engine
    
  simulation:
    - active_bodies
    - collision_pairs
    - constraint_count
    - particle_count  # POS engine
    - fluid_elements  # POS engine
    
  quality:
    - solver_iterations
    - constraint_errors
    - penetration_depth
    - energy_conservation
    
  network:
    - physics_sync_rate
    - prediction_accuracy
    - rollback_frequency
    - bandwidth_usage

alerts:
  physics_fps_below: 30
  cpu_usage_above: 85%
  memory_usage_above: 2GB
  constraint_errors_above: 10
  
visualization:
  grafana_dashboard: true
  prometheus_metrics: true
  custom_charts: enabled
  historical_trends: 90d
EOF

# Start physics monitoring
cargo run --bin physics-monitor -- start \
  --config /opt/opensim/physics-monitoring.yaml \
  --enable-alerts \
  --web-dashboard

# Generate performance report
cargo run --bin physics-analyzer -- generate-report \
  --timeframe 24h \
  --regions all \
  --include-recommendations \
  --format pdf
```

### Performance Optimization

```bash
# Automatic performance optimization
cargo run --bin physics-optimizer -- optimize \
  --region "Performance Critical" \
  --target-fps 60 \
  --max-cpu 70% \
  --quality balanced \
  --auto-apply

# Engine-specific optimization
cargo run --bin physics-optimizer -- tune-engine \
  --engine Bullet \
  --region "Vehicle Sandbox" \
  --optimize-for vehicles \
  --measure-baseline 5m

# GPU optimization (POS engine)
cargo run --bin physics-optimizer -- optimize-gpu \
  --region "Particle Demo" \
  --gpu-memory-limit 1GB \
  --compute-optimization true \
  --profile-gpu-usage

# Memory optimization
cargo run --bin physics-optimizer -- optimize-memory \
  --all-engines \
  --target-usage 1.5GB \
  --enable-compression \
  --garbage-collection tuned
```

## 6.7 GPU Acceleration Configuration

### POS Engine GPU Setup

The POS (Position-Based Dynamics) engine supports GPU acceleration for massive particle and fluid simulations:

```bash
# Configure GPU acceleration
cat > /opt/opensim/gpu-physics.yaml << EOF
gpu_acceleration:
  enabled: true
  device_selection: auto  # auto, discrete, integrated, cpu_fallback
  
compute_configuration:
  api: vulkan  # vulkan, opencl, cuda
  compute_shader_version: "4.6"
  work_group_size: 256
  max_work_groups: 65535
  
memory_management:
  gpu_memory_allocation: 2GB
  cpu_gpu_transfer_optimization: true
  memory_pooling: true
  automatic_cleanup: true
  
particle_systems:
  max_particles_gpu: 100000
  particle_buffer_size: 256MB
  position_buffer_format: float32x3
  velocity_buffer_format: float32x3
  
fluid_simulation:
  sph_gpu_acceleration: true
  neighbor_search_gpu: true
  density_calculation_gpu: true
  force_calculation_gpu: true
  integration_gpu: true
  
performance:
  gpu_cpu_synchronization: minimal
  async_compute: true
  compute_queue_priority: high
  gpu_profiling: true
  
fallback:
  cpu_fallback_threshold: 10%  # GPU usage below 10%
  automatic_fallback: true
  fallback_notification: true
EOF

# Initialize GPU physics
cargo run --bin gpu-physics -- initialize \
  --config /opt/opensim/gpu-physics.yaml \
  --test-compute-capability \
  --benchmark-performance

# Test GPU acceleration
cargo run --bin gpu-physics -- test \
  --particle-count 50000 \
  --fluid-simulation \
  --measure-performance \
  --compare-cpu
```

### GPU Performance Monitoring

```bash
# Monitor GPU physics performance
cargo run --bin gpu-monitor -- start \
  --real-time-dashboard \
  --metrics gpu_usage,memory_usage,compute_throughput \
  --alerts-enabled

# GPU profiling for optimization
cargo run --bin gpu-profiler -- profile \
  --region "Particle Demo" \
  --duration 5m \
  --detailed-analysis \
  --optimization-suggestions
```

## 6.8 Physics Engine Integration Testing

### Comprehensive Testing Framework

```bash
# Create physics testing configuration
cat > /opt/opensim/physics-testing.yaml << EOF
testing:
  comprehensive_suite: true
  engines_to_test: [ODE, UBODE, Bullet, POS, Basic]
  
test_scenarios:
  avatar_physics:
    - walking_stability
    - jumping_dynamics
    - collision_response
    - network_synchronization
    
  object_physics:
    - rigid_body_dynamics
    - collision_detection
    - constraint_solving
    - mass_properties
    
  vehicle_physics:  # Bullet only
    - suspension_dynamics
    - wheel_friction
    - steering_response
    - stability_control
    
  particle_systems:  # POS only
    - particle_emission
    - collision_response
    - fluid_dynamics
    - gpu_acceleration
    
  performance_tests:
    - stress_testing
    - memory_usage
    - cpu_utilization
    - frame_rate_stability
    
validation:
  accuracy_thresholds:
    position_error: 0.01
    velocity_error: 0.1
    energy_conservation: 0.05
    
  performance_requirements:
    min_fps: 30
    max_cpu_usage: 80%
    max_memory_usage: 2GB
    
  compatibility_checks:
    cross_engine_migration: true
    save_state_integrity: true
    network_synchronization: true
EOF

# Run comprehensive physics tests
cargo run --bin physics-tester -- run-suite \
  --config /opt/opensim/physics-testing.yaml \
  --output-report \
  --generate-benchmarks

# Test engine switching
cargo run --bin physics-tester -- test-switching \
  --from ODE \
  --to Bullet \
  --test-state-preservation \
  --measure-downtime

# Performance benchmarking
cargo run --bin physics-benchmarker -- benchmark \
  --all-engines \
  --standard-scenarios \
  --generate-comparison
```

### Testing Results Analysis

```bash
# Analyze test results
cargo run --bin test-analyzer -- analyze \
  --test-results latest \
  --generate-recommendations \
  --identify-regressions

# Performance comparison
cargo run --bin performance-comparator -- compare \
  --engines ODE,Bullet,POS \
  --scenarios vehicles,particles,avatars \
  --generate-charts
```

## 6.9 Troubleshooting Physics Issues

### Common Physics Problems

#### Avatar Movement Issues

**Problem**: Avatars moving erratically or getting stuck

**Diagnosis**:
```bash
# Check avatar physics configuration
cargo run --bin physics-diagnostics -- check-avatars \
  --region "Problem Region" \
  --detailed-analysis

# Monitor avatar physics in real-time
cargo run --bin avatar-physics-monitor -- monitor \
  --avatar "Problem User" \
  --duration 5m \
  --detailed-logging
```

**Solutions**:
```bash
# Reset avatar physics
cargo run --bin physics-manager -- reset-avatar \
  --avatar "Problem User" \
  --preserve-position

# Adjust avatar physics parameters
cargo run --bin physics-tuner -- tune-avatars \
  --region "Problem Region" \
  --increase-damping \
  --reduce-bounce
```

#### Vehicle Physics Problems

**Problem**: Vehicles behaving unrealistically

**Diagnosis**:
```bash
# Check vehicle constraints
cargo run --bin physics-diagnostics -- check-vehicles \
  --region "Vehicle Region" \
  --analyze-constraints

# Monitor vehicle dynamics
cargo run --bin vehicle-monitor -- monitor \
  --vehicle "Problem Vehicle" \
  --track-forces \
  --analyze-stability
```

**Solutions**:
```bash
# Recalibrate vehicle physics
cargo run --bin vehicle-tuner -- calibrate \
  --vehicle "Problem Vehicle" \
  --realistic-parameters

# Switch to Bullet engine if using ODE
cargo run --bin physics-manager -- switch-engine \
  --region "Vehicle Region" \
  --engine Bullet \
  --profile vehicles_advanced
```

#### Particle System Issues

**Problem**: Poor particle performance or visual artifacts

**Diagnosis**:
```bash
# Check GPU acceleration status
cargo run --bin gpu-diagnostics -- check-status \
  --region "Particle Region"

# Analyze particle system performance
cargo run --bin particle-analyzer -- analyze \
  --particle-system "Problem System" \
  --check-gpu-utilization
```

**Solutions**:
```bash
# Optimize particle parameters
cargo run --bin particle-optimizer -- optimize \
  --particle-system "Problem System" \
  --reduce-count-if-needed \
  --improve-gpu-usage

# Enable GPU acceleration if not active
cargo run --bin gpu-physics -- enable \
  --region "Particle Region" \
  --test-compatibility
```

### Physics Performance Debugging

```bash
# Enable physics debugging
cargo run --bin physics-debugger -- enable \
  --visual-debugging \
  --performance-counters \
  --memory-tracking

# Generate physics diagnostic report
cargo run --bin physics-diagnostics -- generate-report \
  --region "Problem Region" \
  --include-suggestions \
  --detailed-analysis
```

---

**Multi-Physics Engine Configuration Complete!** 🎉

Your OpenSim Next deployment now features the world's most advanced multi-physics engine system. You can optimize each region with the perfect physics engine for its content, from stable avatar interactions to GPU-accelerated particle systems. The next chapter will guide you through viewer client configuration for Second Life, Firestorm, and web browsers.

---

# Chapter 7: Client Configuration Guide

This chapter provides comprehensive configuration instructions for connecting to OpenSim Next using **traditional Second Life viewers**, **modern third-party viewers**, and the **revolutionary web browser interface**. OpenSim Next is the world's first virtual world platform to support both traditional viewers and web browsers simultaneously.

## 7.1 Multi-Client Architecture Overview

OpenSim Next's groundbreaking **multi-protocol client support** enables users to access virtual worlds through their preferred method, from traditional viewers to modern web browsers.

### Supported Client Types

| Client Type | Protocol | Platform | Features | Best For |
|-------------|----------|----------|----------|----------|
| **Second Life Viewer** | LLUDP | Windows, macOS, Linux | Full compatibility | Traditional users |
| **Firestorm** | LLUDP | Windows, macOS, Linux | Advanced features | Power users |
| **Singularity** | LLUDP | Windows, macOS, Linux | Lightweight | Older hardware |
| **Web Browser** | WebSocket | Universal | Revolutionary access | Modern users |
| **Mobile Web** | WebSocket | iOS, Android | Touch interface | Mobile users |
| **Custom Clients** | Both | Any | SDK integration | Developers |

### Revolutionary Multi-Protocol Support

```
┌─────────────────────────────────────────────────────────────────┐
│                OpenSim Next Client Architecture                │
├─────────────────────────────────────────────────────────────────┤
│                    Client Connection Layer                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Traditional     │  │ Web Browser     │  │ Mobile          │ │
│  │ Viewers         │  │ Interface       │  │ Clients         │ │
│  │ (LLUDP)         │  │ (WebSocket)     │  │ (WebSocket)     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Protocol Translation                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ LLUDP Handler   │  │ WebSocket       │  │ Protocol        │ │
│  │ & Message       │  │ Message         │  │ Bridge          │ │
│  │ Processing      │  │ Gateway         │  │ & Sync          │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                     Core Services                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Authentication  │  │ Avatar          │  │ Region          │ │
│  │ & Sessions      │  │ Management      │  │ Services        │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Asset           │  │ Chat & IM       │  │ Physics         │ │
│  │ Distribution    │  │ System          │  │ Simulation      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Cross-Platform Compatibility

- **🖥️ Desktop**: Windows, macOS, Linux support for all viewers
- **🌐 Web Universal**: Any modern browser (Chrome, Firefox, Safari, Edge)
- **📱 Mobile**: iOS Safari, Android Chrome, mobile browsers
- **🔄 Real-Time Sync**: Live synchronization between all client types
- **🎨 Unified Experience**: Consistent features across all platforms

## 7.2 Traditional Viewer Configuration

### Second Life Viewer Setup

The official Second Life Viewer provides the most compatible experience with OpenSim Next.

#### Installation and Initial Setup

```bash
# Download Second Life Viewer
# Windows: https://secondlife.com/support/downloads/
# macOS: Available from Mac App Store or website
# Linux: Available as .deb package or AppImage

# For Linux Ubuntu/Debian:
wget https://download.cloud.secondlife.com/Viewer-7/Second_Life_7_1_9_12345_x86_64.tar.xz
tar -xf Second_Life_7_1_9_12345_x86_64.tar.xz
sudo mv SecondLife-x86_64-7.1.9.12345 /opt/secondlife
sudo ln -s /opt/secondlife/secondlife /usr/local/bin/secondlife
```

#### OpenSim Next Connection Configuration

1. **Create Grid Configuration**:
   - Open Second Life Viewer
   - Click "Grid Manager" (or press Ctrl+Shift+G)
   - Click "Add New Grid"

2. **Configure OpenSim Next Grid**:
```
Grid Name: OpenSim Next
Login URI: http://your-server.com:9000/
Helper URI: http://your-server.com:9000/

Advanced Settings:
- Grid Info URI: http://your-server.com:9000/get_grid_info
- Login Page URI: http://your-server.com:9000/
- Search URI: http://your-server.com:9000/search
- Web Profile URL: http://your-server.com:8080/profiles/
```

3. **User Account Setup**:
```
Username: your-username
Password: your-password
Grid: OpenSim Next (from dropdown)
Location: Home or Last Location
```

#### Advanced Viewer Configuration

Create optimized settings for OpenSim Next:

```bash
# Create viewer configuration file
cat > ~/.secondlife/grids.xml << EOF
<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<llsd>
  <map>
    <key>OpenSim Next</key>
    <map>
      <key>gridname</key>
      <string>OpenSim Next</string>
      <key>gridnick</key>
      <string>opensim-next</string>
      <key>loginuri</key>
      <array>
        <string>http://your-server.com:9000/</string>
      </array>
      <key>helperuri</key>
      <string>http://your-server.com:9000/</string>
      <key>website</key>
      <string>http://your-server.com:8080/</string>
      <key>support</key>
      <string>http://your-server.com:8080/support</string>
      <key>account</key>
      <string>http://your-server.com:8080/register</string>
      <key>password</key>
      <string>http://your-server.com:8080/password</string>
      <key>search</key>
      <string>http://your-server.com:9000/search</string>
      <key>render_compat</key>
      <boolean>true</boolean>
      <key>platform</key>
      <string>OpenSim</string>
      <key>version</key>
      <string>Next</string>
    </map>
  </map>
</llsd>
EOF
```

### Firestorm Viewer Setup

Firestorm is the most popular third-party viewer with advanced features perfect for OpenSim Next.

#### Installation

```bash
# Download Firestorm
# Website: https://www.firestormviewer.org/

# Linux Installation
wget https://downloads.firestormviewer.org/linux/Phoenix_FireStorm-Releasex64_7_1_9_70838_x86_64.tar.xz
tar -xf Phoenix_FireStorm-Releasex64_7_1_9_70838_x86_64.tar.xz
sudo mv Phoenix_FireStorm-Releasex64_7_1_9_70838 /opt/firestorm
sudo ln -s /opt/firestorm/firestorm /usr/local/bin/firestorm
```

#### OpenSim Next Grid Configuration

1. **Grid Manager Setup**:
   - Launch Firestorm
   - Open Preferences → Firestorm → General
   - Enable "Use Grid Manager on login screen"

2. **Add OpenSim Next Grid**:
```
Grid Name: OpenSim Next
Grid URI: http://your-server.com:9000/
```

3. **Advanced Firestorm Settings**:
```bash
# Create Firestorm-specific configuration
cat > ~/.firestorm_x64/grids.xml << EOF
<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<llsd>
  <map>
    <key>opensim-next.your-domain.com</key>
    <map>
      <key>favorite</key>
      <integer>1</integer>
      <key>grid_login_id</key>
      <string>opensim-next</string>
      <key>grid_login_page</key>
      <string>http://your-server.com:9000/</string>
      <key>helper_uri</key>
      <string>http://your-server.com:9000/</string>
      <key>label</key>
      <string>OpenSim Next</string>
      <key>login_uri</key>
      <array>
        <string>http://your-server.com:9000/</string>
      </array>
      <key>platform</key>
      <string>OpenSim</string>
      <key>search</key>
      <string>http://your-server.com:9000/search</string>
      <key>website</key>
      <string>http://your-server.com:8080</string>
      <key>register</key>
      <string>http://your-server.com:8080/register</string>
      <key>password</key>
      <string>http://your-server.com:8080/password</string>
      <key>help</key>
      <string>http://your-server.com:8080/help</string>
    </map>
  </map>
</llsd>
EOF
```

#### Firestorm Optimization for OpenSim Next

```bash
# Create optimized Firestorm settings
cat > ~/.firestorm_x64/user_settings/opensim_next_settings.xml << EOF
<llsd>
  <map>
    <!-- Performance optimizations -->
    <key>RenderVolumeLODFactor</key>
    <real>2.0</real>
    <key>RenderAvatarLODFactor</key>
    <real>1.0</real>
    <key>RenderTreeLODFactor</key>
    <real>1.0</real>
    <key>RenderTerrainLODFactor</key>
    <real>2.0</real>
    
    <!-- OpenSim-specific settings -->
    <key>AllowLargeSounds</key>
    <boolean>true</boolean>
    <key>AllowStreamingMusic</key>
    <boolean>true</boolean>
    <key>EnableVoiceChat</key>
    <boolean>true</boolean>
    
    <!-- Multi-physics engine support -->
    <key>ShowPhysicsEngineInfo</key>
    <boolean>true</boolean>
    <key>PhysicsEngineAwareness</key>
    <boolean>true</boolean>
    
    <!-- WebSocket awareness -->
    <key>EnableWebSocketBridge</key>
    <boolean>true</boolean>
    <key>WebSocketEndpoint</key>
    <string>ws://your-server.com:9001/bridge</string>
  </map>
</llsd>
EOF
```

### Other Compatible Viewers

#### Singularity Viewer
```bash
# Lightweight viewer for older hardware
wget https://github.com/SingularityViewer/SingularityViewer/releases/download/1.8.7/Singularity-1.8.7-Linux-x86_64.tar.xz
tar -xf Singularity-1.8.7-Linux-x86_64.tar.xz
./Singularity

# Grid configuration similar to other viewers
# Login URI: http://your-server.com:9000/
```

#### Alchemy Viewer
```bash
# Advanced viewer with modern features
# Download from: https://alchemyviewer.org/
# Configuration similar to Firestorm
```

### Source-Built Viewers (macOS)

OpenSim Next development includes three working viewer builds, two of which are source-built with a critical macOS TLS alignment crash fix.

#### Available Viewer Builds

| Viewer | Location | Type | Notes |
|--------|----------|------|-------|
| **FirestormOSNext** | `/Applications/FirestormOSNext.app` | Source-built Firestorm | General purpose |
| **ApertureViewer** | `/Applications/ApertureViewer.app` | Source-built Aperture | Cinematography focused |
| **Stock Firestorm** | Downloaded from website | Pre-built binary | Fallback |

#### macOS TLS Alignment Crash Fix

Source-built viewers on macOS require a critical fix for a thread-local storage (TLS) alignment crash. The macOS TLS allocator does not honor `alignas(16)` for `thread_local` objects. At `-O3`, the compiler generates `movaps` (16-byte aligned SSE moves) for struct initialization, causing `SIGBUS` crashes.

**Fix applied to both FirestormOSNext and ApertureViewer:**
- `llrender.cpp`: Replace `thread_local LLRender gGL;` with heap-allocated 64-byte-aligned per-thread pointer using `posix_memalign`
- `llrender.h`: Add `#define gGL (gGL_ref())` on `__APPLE__` to indirect through the heap pointer

#### Building Firestorm from Source (macOS)

```bash
# Source location
cd opensim-master/Firestorm/phoenix-firestorm-master/

# Build
xcodebuild -project build-darwin-x86_64/Firestorm.xcodeproj \
  -target firestorm-bin -configuration Release

# Deploy
cp build-darwin-x86_64/Release/firestorm-bin \
  /Applications/FirestormOSNext.app/Contents/MacOS/Firestorm
```

#### Building Aperture Viewer from Source (macOS)

```bash
# Source location
cd opensim-master/Aperture-Viewer/Aperture-Viewer-dev/

# Build
xcodebuild -project build-darwin-x86_64/Aperture.xcodeproj \
  -target aperture -configuration Release

# Deploy to /Applications/ApertureViewer.app
```

## 7.3 Revolutionary Web Browser Interface

OpenSim Next features the world's first **production-ready web browser interface** for virtual worlds, enabling access through any modern browser without downloads or installations.

### Web Client Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                  Web Browser Virtual World                     │
├─────────────────────────────────────────────────────────────────┤
│                    Browser Interface                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ 3D World        │  │ User Interface  │  │ Chat & Social   │ │
│  │ Renderer        │  │ Controls        │  │ Features        │ │
│  │ (WebGL/GPU)     │  │ (HTML5/CSS3)    │  │ (WebSocket)     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                  Real-Time Communication                       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ WebSocket       │  │ Asset Loading   │  │ Physics Sync    │ │
│  │ Messaging       │  │ & Caching       │  │ & Updates       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                     Core Technologies                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ WebGL 2.0       │  │ WebAssembly     │  │ Service Workers │ │
│  │ GPU Rendering   │  │ Performance     │  │ Offline Support │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Web Client Access

#### Direct Browser Access

1. **Open Web Interface**:
   - Navigate to: `http://your-server.com:8080`
   - No downloads or installations required
   - Works on any modern browser

2. **Login Process**:
```html
<!-- Automatic login form -->
<form id="web-login">
    <input type="text" name="username" placeholder="Username" required>
    <input type="password" name="password" placeholder="Password" required>
    <select name="startLocation">
        <option value="home">Home</option>
        <option value="last">Last Location</option>
        <option value="region">Specific Region</option>
    </select>
    <button type="submit">Enter Virtual World</button>
</form>
```

3. **World Interface**:
   - **3D World View**: Full WebGL rendering
   - **Avatar Controls**: WASD movement, mouse look
   - **Chat Interface**: Real-time messaging
   - **Inventory**: Asset management
   - **Friends List**: Social features

#### Mobile Web Access

OpenSim Next provides optimized mobile web interfaces:

```html
<!-- Mobile-optimized interface -->
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<link rel="manifest" href="/manifest.json">

<!-- Touch controls for mobile -->
<div class="mobile-controls">
    <div class="movement-joystick"></div>
    <div class="camera-controls"></div>
    <div class="action-buttons">
        <button class="chat-button">💬</button>
        <button class="inventory-button">🎒</button>
        <button class="menu-button">☰</button>
    </div>
</div>
```

### Web Client Configuration

#### Server-Side Configuration

Enable web client support in `config/opensim.ini`:

```ini
[WebInterface]
; Enable web browser interface
enable_web_client = true
web_client_port = 8080
web_client_ssl = false  ; Set to true for HTTPS

; WebSocket configuration
websocket_port = 9001
websocket_ssl = false
websocket_compression = true
websocket_max_connections = 1000

; Web client features
enable_webgl_rendering = true
enable_progressive_web_app = true
enable_offline_support = true
enable_mobile_interface = true

; Asset delivery optimization
web_asset_cdn = true
web_asset_compression = true
web_asset_caching = aggressive
web_texture_format = webp

; Performance settings
web_render_distance = 256
web_max_prims = 10000
web_avatar_complexity = medium
web_particle_count = 5000

[WebSecurity]
; Web security settings
cors_enabled = true
cors_origins = *
csp_enabled = true
secure_cookies = true
session_timeout = 8h

; WebGL security
webgl_context_lost_handling = true
memory_limit = 1GB
shader_validation = strict
```

#### Client-Side Optimization

```javascript
// Web client configuration
const webClientConfig = {
    server: {
        host: 'your-server.com',
        websocketPort: 9001,
        httpPort: 8080,
        ssl: false
    },
    
    rendering: {
        engine: 'webgl2',
        antialias: true,
        shadows: true,
        maxDrawDistance: 256,
        lodBias: 1.0,
        particleCount: 5000
    },
    
    performance: {
        targetFPS: 60,
        adaptiveQuality: true,
        memoryLimit: 1024, // MB
        textureMemoryLimit: 512, // MB
        geometryMemoryLimit: 256 // MB
    },
    
    features: {
        voiceChat: true,
        textToSpeech: true,
        spatialAudio: true,
        hapticFeedback: true, // Mobile
        backgroundSync: true
    },
    
    mobile: {
        touchControls: true,
        gyroscopeCamera: true,
        adaptiveUI: true,
        batteryOptimization: true
    }
};
```

#### Progressive Web App (PWA) Setup

```json
{
  "name": "OpenSim Next Virtual World",
  "short_name": "OpenSim Next",
  "description": "Revolutionary web-based virtual world platform",
  "start_url": "/",
  "display": "fullscreen",
  "background_color": "#000000",
  "theme_color": "#1a73e8",
  "orientation": "landscape-primary",
  "icons": [
    {
      "src": "/icons/icon-192.png",
      "sizes": "192x192",
      "type": "image/png"
    },
    {
      "src": "/icons/icon-512.png",
      "sizes": "512x512",
      "type": "image/png"
    }
  ],
  "categories": ["social", "games", "entertainment"],
  "shortcuts": [
    {
      "name": "Home Region",
      "url": "/region/home",
      "description": "Go to home region"
    },
    {
      "name": "Friends",
      "url": "/friends",
      "description": "View friends list"
    }
  ]
}
```

## 7.4 Cross-Platform Client Synchronization

OpenSim Next enables real-time synchronization between traditional viewers and web browsers.

### Real-Time Sync Features

- **👥 Cross-Platform Avatars**: See users from any client type
- **💬 Universal Chat**: Messages sync between all platforms
- **🎯 Shared Objects**: Interact with objects from any client
- **🌍 Region Sync**: Seamless experience across client types
- **📱 Mobile Continuity**: Switch between devices seamlessly

### Synchronization Configuration

```bash
# Configure cross-platform sync
cat > config-include/opensim-next/CrossPlatformSync.ini << EOF
[ClientSync]
; Enable cross-platform synchronization
enable_cross_platform_sync = true
sync_interval = 100ms
sync_tolerance = 50ms

; Avatar synchronization
avatar_position_sync = realtime
avatar_animation_sync = true
avatar_appearance_sync = true
avatar_attachment_sync = true

; Object synchronization
object_position_sync = true
object_physics_sync = true
object_script_sync = true
object_texture_sync = true

; Chat and communication
chat_cross_platform = true
im_cross_platform = true
voice_bridge_enabled = true
group_chat_sync = true

; Asset synchronization
asset_streaming_optimization = true
texture_format_negotiation = true
mesh_lod_adaptation = true
animation_compatibility = true

[WebBridge]
; Bridge between LLUDP and WebSocket
enable_protocol_bridge = true
bridge_port = 9002
bridge_buffer_size = 1MB
bridge_compression = true

; Protocol translation
lludp_to_websocket = true
websocket_to_lludp = true
message_queue_size = 1000
translation_cache = 100MB

; Performance optimization
bridge_threading = true
async_processing = true
batch_translation = true
EOF
```

### Cross-Platform Testing

```bash
# Test cross-platform functionality
cargo run --bin cross-platform-tester -- test-all \
  --traditional-viewer firestorm \
  --web-browser chrome \
  --mobile-browser safari \
  --test-scenarios comprehensive

# Monitor sync performance
cargo run --bin sync-monitor -- monitor \
  --clients all \
  --metrics latency,accuracy,bandwidth \
  --duration 10m \
  --generate-report
```

## 7.5 Client Performance Optimization

### Traditional Viewer Optimization

#### Firestorm Performance Settings

```xml
<!-- Optimal Firestorm settings for OpenSim Next -->
<llsd>
  <map>
    <!-- Rendering optimizations -->
    <key>RenderMaxPartCount</key>
    <integer>8192</integer>
    <key>RenderMaxNodeSize</key>
    <real>65536</real>
    <key>RenderVolumeLODFactor</key>
    <real>2.0</real>
    
    <!-- Memory management -->
    <key>TextureMemory</key>
    <integer>1024</integer>
    <key>MeshMaxConcurrentRequests</key>
    <integer>32</integer>
    <key>Mesh2MaxConcurrentRequests</key>
    <integer>16</integer>
    
    <!-- Network optimization -->
    <key>ThrottleBandwidthKBPS</key>
    <real>3000.0</real>
    <key>HTTPPipelining</key>
    <boolean>true</boolean>
    <key>Mesh2UseGetMesh1</key>
    <boolean>false</boolean>
    
    <!-- OpenSim-specific -->
    <key>OpenSimMode</key>
    <boolean>true</boolean>
    <key>AllowLargeSounds</key>
    <boolean>true</boolean>
    <key>UseServerTextureBaking</key>
    <boolean>true</boolean>
  </map>
</llsd>
```

#### Second Life Viewer Optimization

```xml
<!-- Optimal SL Viewer settings -->
<llsd>
  <map>
    <!-- Graphics settings -->
    <key>RenderQualityPerformance</key>
    <integer>3</integer>
    <key>WindLightUseAtmosShaders</key>
    <boolean>true</boolean>
    <key>VertexShaderEnable</key>
    <boolean>true</boolean>
    
    <!-- Bandwidth settings -->
    <key>CurrentBandwidth</key>
    <real>2000</real>
    <key>ObjectBandwidth</key>
    <real>1000</real>
    <key>TextureBandwidth</key>
    <real>1000</real>
    
    <!-- Cache settings -->
    <key>CacheSize</key>
    <integer>2048</integer>
    <key>CacheValidateCounter</key>
    <integer>0</integer>
  </map>
</llsd>
```

### Web Client Optimization

#### Browser-Specific Settings

```javascript
// Chrome optimization
if (navigator.userAgent.includes('Chrome')) {
    config.rendering.webglVersion = 2;
    config.performance.memoryLimit = 2048;
    config.features.webassembly = true;
}

// Firefox optimization
if (navigator.userAgent.includes('Firefox')) {
    config.rendering.webglVersion = 2;
    config.performance.conservativeMemory = true;
    config.features.offscreenCanvas = true;
}

// Safari optimization
if (navigator.userAgent.includes('Safari')) {
    config.rendering.webglVersion = 1; // More compatible
    config.performance.memoryLimit = 1024;
    config.features.webassembly = false; // Compatibility
}

// Mobile optimization
if (navigator.userAgent.includes('Mobile')) {
    config.rendering.maxDrawDistance = 128;
    config.performance.targetFPS = 30;
    config.features.batteryOptimization = true;
}
```

#### Asset Loading Optimization

```javascript
// Progressive asset loading
const assetLoader = {
    priorities: {
        avatars: 1,      // Highest priority
        terrain: 2,
        buildings: 3,
        decorations: 4,
        particles: 5     // Lowest priority
    },
    
    compression: {
        textures: 'webp',
        meshes: 'draco',
        animations: 'compact'
    },
    
    caching: {
        strategy: 'lru',
        maxSize: '500MB',
        persistence: 'indexeddb'
    },
    
    streaming: {
        enabled: true,
        chunkSize: '1MB',
        concurrent: 6
    }
};
```

## 7.6 Client Troubleshooting

### Traditional Viewer Issues

#### Connection Problems

**Problem**: Cannot connect to OpenSim Next grid

**Symptoms**:
```
Login failed - Unknown error
Cannot resolve grid URL
SSL certificate errors
```

**Solutions**:
```bash
# Verify server accessibility
curl -I http://your-server.com:9000/

# Check grid configuration
cat ~/.secondlife/grids.xml | grep -A 10 "OpenSim Next"

# Test login endpoint
curl -X POST http://your-server.com:9000/ \
  -H "Content-Type: application/xml" \
  -d '<?xml version="1.0"?><methodCall><methodName>login_to_simulator</methodName></methodCall>'

# Verify DNS resolution
nslookup your-server.com
```

#### Performance Issues

**Problem**: Poor performance in traditional viewers

**Diagnosis**:
```bash
# Check viewer logs
tail -f ~/.secondlife/logs/SecondLife.log

# Monitor network usage
iftop -i wlan0

# Check system resources
top -p $(pgrep SecondLife)
```

**Solutions**:
```bash
# Clear viewer cache
rm -rf ~/.secondlife/cache/*

# Update graphics drivers
sudo ubuntu-drivers autoinstall

# Optimize viewer settings
# Use the optimized XML configurations above
```

### Web Client Issues

#### Browser Compatibility

**Problem**: Web client not loading properly

**Diagnosis**:
```javascript
// Check browser capabilities
console.log('WebGL Support:', !!window.WebGLRenderingContext);
console.log('WebGL2 Support:', !!window.WebGL2RenderingContext);
console.log('WebSocket Support:', !!window.WebSocket);
console.log('WebAssembly Support:', !!window.WebAssembly);

// Check available memory
console.log('Memory Info:', navigator.deviceMemory);
console.log('Hardware Concurrency:', navigator.hardwareConcurrency);
```

**Solutions**:
```javascript
// Enable hardware acceleration in browser settings
// Chrome: chrome://settings/system
// Firefox: about:config -> layers.acceleration.force-enabled

// Clear browser cache and cookies
// Update browser to latest version
// Try different browser if issues persist
```

#### Performance Issues

**Problem**: Poor web client performance

**Solutions**:
```javascript
// Reduce rendering quality
config.rendering.quality = 'medium';
config.rendering.maxDrawDistance = 128;
config.performance.targetFPS = 30;

// Enable adaptive quality
config.performance.adaptiveQuality = true;
config.performance.autoDowngrade = true;

// Optimize for mobile
if (navigator.userAgent.includes('Mobile')) {
    config = mobileOptimizedConfig;
}
```

## 7.7 Client Feature Comparison

### Feature Matrix

| Feature | SL Viewer | Firestorm | Web Browser | Mobile Web |
|---------|-----------|-----------|-------------|------------|
| **Basic Movement** | ✅ | ✅ | ✅ | ✅ |
| **Avatar Customization** | ✅ | ✅ | ✅ | Limited |
| **Building Tools** | ✅ | ✅ | ✅ | Touch-optimized |
| **Scripting** | ✅ | ✅ | View-only | View-only |
| **Voice Chat** | ✅ | ✅ | ✅ | ✅ |
| **Media Streaming** | ✅ | ✅ | ✅ | ✅ |
| **Mesh Upload** | ✅ | ✅ | ✅ | ❌ |
| **Advanced Graphics** | ✅ | ✅ | WebGL | WebGL Mobile |
| **Physics Interaction** | Full | Full | Full | Touch |
| **Inventory Management** | ✅ | ✅ | ✅ | Simplified |

### Client Recommendations

#### For Different Use Cases

**Social Users**: 
- Primary: Web Browser
- Alternative: Second Life Viewer
- Mobile: Mobile Web Interface

**Content Creators**:
- Primary: Firestorm Viewer
- Alternative: Second Life Viewer
- Testing: Web Browser

**Developers**:
- Primary: Firestorm Viewer
- Testing: All clients
- Mobile Testing: Mobile Web

**Mobile Users**:
- Primary: Mobile Web Interface
- Desktop: Web Browser
- Full Features: Firestorm Viewer

**Enterprise Users**:
- Primary: Web Browser (no installation)
- Desktop: Second Life Viewer
- Management: Web Interface

## 7.8 Advanced Client Configuration

### Custom Viewer Integration

For organizations wanting to create custom viewers:

```rust
// SDK integration example
use opensim_client_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to OpenSim Next
    let client = OpenSimClient::new("ws://your-server.com:9001").await?;
    
    // Authenticate
    let session = client.login("username", "password").await?;
    
    // Join virtual world
    let avatar = session.spawn_avatar("Welcome Region").await?;
    
    // Handle real-time events
    client.on_chat_message(|message| {
        println!("Chat: {}", message.text);
    });
    
    client.on_avatar_move(|avatar_id, position| {
        println!("Avatar {} moved to {:?}", avatar_id, position);
    });
    
    // Keep connection alive
    client.run().await?;
    
    Ok(())
}
```

### WebSocket API Integration

```javascript
// Custom web client integration
class OpenSimWebClient {
    constructor(serverUrl) {
        this.ws = new WebSocket(`ws://${serverUrl}:9001/ws`);
        this.setupEventHandlers();
    }
    
    setupEventHandlers() {
        this.ws.onopen = () => {
            console.log('Connected to OpenSim Next');
            this.authenticate();
        };
        
        this.ws.onmessage = (event) => {
            const message = JSON.parse(event.data);
            this.handleMessage(message);
        };
    }
    
    authenticate() {
        this.send({
            type: 'Auth',
            username: 'user',
            password: 'pass'
        });
    }
    
    send(message) {
        this.ws.send(JSON.stringify(message));
    }
    
    handleMessage(message) {
        switch(message.type) {
            case 'ChatMessage':
                this.displayChat(message);
                break;
            case 'AvatarUpdate':
                this.updateAvatar(message);
                break;
            case 'RegionUpdate':
                this.updateRegion(message);
                break;
        }
    }
}
```

---

**Client Configuration Complete!** 🎉

Your users can now access OpenSim Next through their preferred client - from traditional Second Life viewers to revolutionary web browsers and mobile devices. The platform provides seamless cross-platform synchronization and optimal performance for every access method. The next chapter will guide you through comprehensive region management and scaling strategies.

---

# Chapter 8: Region Management and Scaling

This chapter provides comprehensive guidance for managing virtual world regions at scale, from single-region development environments to enterprise-grade multi-region grids with thousands of concurrent users.

## 8.1 Region Lifecycle Management

### 8.1.1 Region Creation and Initialization

**Creating New Regions**

OpenSim Next supports multiple methods for region creation:

1. **Configuration-Based Region Creation**
   ```ini
   ; Regions/MyRegion.ini
   [Region Settings]
   RegionName = "My New Region" 
   RegionUUID = 550e8400-e29b-41d4-a716-446655440000
   Location = 1000,1000
   SizeX = 256
   SizeY = 256
   SizeZ = 4096
   InternalAddress = 0.0.0.0
   InternalPort = 9000
   AllowAlternatePorts = false
   ExternalHostName = SYSTEMIP
   PhysicsEngine = ODE
   ```

   **Varregion (Large Region) Configuration**

   OpenSim Next supports varregions — regions larger than the standard 256x256. Supported sizes are 256, 512, 768, and 1024 meters per side. Varregions must be square and sized in multiples of 256.

   ```ini
   ; Regions/LargeRegion.ini — 1024x1024 varregion (16x the area of a standard region)
   [Region Settings]
   RegionName = "Varregion One"
   RegionUUID = 550e8400-e29b-41d4-a716-446655440010
   Location = 1000,1000
   SizeX = 1024
   SizeY = 1024
   SizeZ = 4096
   InternalAddress = 0.0.0.0
   InternalPort = 9000
   ExternalHostName = SYSTEMIP
   ```

   Varregions occupy multiple grid cells. A 1024x1024 region at location 1000,1000 occupies grid cells (1000,1000) through (1003,1003). Other regions cannot overlap these cells.

   **OAR Import/Export**

   Save and restore regions including terrain, objects, and scripts:

   ```bash
   # These are handled via the server console or admin API
   # OAR files preserve full region state including:
   # - Terrain heightmap
   # - All prims and their inventories
   # - Scripts (source code)
   # - Region settings and parcels
   ```

2. **Runtime Region Creation via API**
   ```bash
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     -X POST http://localhost:8090/api/regions/create \
     -H "Content-Type: application/json" \
     -d '{
       "name": "Dynamic Region",
       "uuid": "550e8400-e29b-41d4-a716-446655440001",
       "location": {"x": 1001, "y": 1000},
       "size": {"x": 256, "y": 256, "z": 4096},
       "physics_engine": "Bullet",
       "terrain_type": "flat"
     }'
   ```

3. **Template-Based Region Creation**
   ```bash
   # Create from existing OAR template
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     -X POST http://localhost:8090/api/regions/create-from-template \
     -d '{
       "template_oar": "/path/to/template.oar",
       "region_name": "Templated Region",
       "location": {"x": 1002, "y": 1000}
     }'
   ```

**Region Initialization Process**

1. **Terrain Generation**: Automatic heightmap creation or import
2. **Default Assets**: Population with system textures and objects  
3. **Physics Initialization**: Engine startup with configured parameters
4. **Network Binding**: Port assignment and service registration
5. **Database Integration**: Region data persistence setup
6. **Monitoring Registration**: Metrics collection enablement

### 8.1.2 Region Configuration Management

**Dynamic Configuration Updates**

OpenSim Next supports hot-reloading of region configurations:

```bash
# Update region physics engine without restart
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/regions/my-region/config \
  -d '{
    "physics_engine": "POS",
    "physics_config": {
      "timestep": 0.0083,
      "max_particles": 100000,
      "enable_gpu": true
    }
  }'

# Update region limits and permissions
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/regions/my-region/limits \
  -d '{
    "max_prims": 45000,
    "max_avatars": 100,
    "script_limits": {
      "max_scripts": 10000,
      "memory_limit_mb": 512
    },
    "physics_limits": {
      "max_physical_prims": 1000,
      "max_vehicles": 50
    }
  }'
```

**Configuration Validation and Rollback**

```bash
# Validate configuration before applying
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/regions/my-region/config/validate \
  -d '{"physics_engine": "InvalidEngine"}'

# Rollback to previous configuration
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/regions/my-region/config/rollback
```

### 8.1.3 Region Health Monitoring

**Comprehensive Region Metrics**

OpenSim Next provides detailed metrics for each region:

```bash
# Get region health overview
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  http://localhost:8090/api/regions/my-region/health

# Sample response:
{
  "region_name": "My Region",
  "status": "healthy",
  "uptime_seconds": 86400,
  "performance": {
    "fps": 45.2,
    "physics_fps": 89.7,
    "script_events_per_second": 1205,
    "network_packets_per_second": 8943
  },
  "resources": {
    "memory_usage_mb": 512,
    "cpu_usage_percent": 23.5,
    "active_scripts": 234,
    "total_prims": 12567,
    "physics_bodies": 1834
  },
  "connectivity": {
    "active_avatars": 12,
    "connection_count": 15,
    "bandwidth_mbps": 12.3
  }
}
```

**Health Thresholds and Alerting**

Configure automatic alerting based on region health:

```ini
; OpenSim.ini
[RegionMonitoring]
; FPS thresholds
MinFPS = 20
WarningFPS = 30

; Resource limits
MaxMemoryMB = 2048
MaxCPUPercent = 80

; Connection limits
MaxAvatars = 100
MaxConnections = 150

; Alert settings
AlertEmail = admin@yourgrid.com
AlertWebhook = https://alerts.yourgrid.com/webhook
```

## 8.2 Multi-Region Architecture

### 8.2.1 Grid Topology Design

**Grid Layout Strategies**

1. **Contiguous Grid Layout**
   ```
   ┌─────┬─────┬─────┬─────┐
   │1000,│1001,│1002,│1003,│
   │1000 │1000 │1000 │1000 │
   ├─────┼─────┼─────┼─────┤
   │1000,│1001,│1002,│1003,│
   │1001 │1001 │1001 │1001 │
   ├─────┼─────┼─────┼─────┤
   │1000,│1001,│1002,│1003,│
   │1002 │1002 │1002 │1002 │
   └─────┴─────┴─────┴─────┘
   ```

2. **Clustered Grid Layout**
   ```
   Residential Cluster    Commercial Cluster
   ┌─────┬─────┐         ┌─────┬─────┐
   │Home │Home │         │Shop │Shop │
   │ 1   │ 2   │         │ 1   │ 2   │
   ├─────┼─────┤         ├─────┼─────┤
   │Home │Home │         │Shop │Shop │
   │ 3   │ 4   │         │ 3   │ 4   │
   └─────┴─────┘         └─────┴─────┘
   ```

3. **Hub-and-Spoke Layout**
   ```
          ┌─────┐
          │Spoke│
          │  1  │
          └──┬──┘
             │
   ┌─────┐   │   ┌─────┐
   │Spoke├───┼───┤Spoke│
   │  2  │   │   │  3  │
   └─────┘   │   └─────┘
             │
          ┌──┴──┐
          │ Hub │
          │     │
          └──┬──┘
             │
          ┌──┴──┐
          │Spoke│
          │  4  │
          └─────┘
   ```

**Grid Coordinate Management**

```bash
# Reserve coordinate ranges for organized development
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/grid/coordinates/reserve \
  -d '{
    "range": {"x_start": 1000, "x_end": 1010, "y_start": 1000, "y_end": 1010},
    "purpose": "residential_area",
    "owner": "CommunityGroup1",
    "reserved_until": "2025-12-31T23:59:59Z"
  }'

# Check coordinate availability
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/grid/coordinates/check?x=1005&y=1005"
```

### 8.2.2 Inter-Region Communication

**Region Discovery and Registration**

```bash
# Register region with grid services
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/grid/regions/register \
  -d '{
    "region_uuid": "550e8400-e29b-41d4-a716-446655440000",
    "region_name": "My Region",
    "location": {"x": 1000, "y": 1000},
    "size": {"x": 256, "y": 256},
    "endpoints": {
      "public_ip": "192.168.1.100",
      "internal_ip": "10.0.0.100",
      "port": 9000
    },
    "capabilities": ["avatar_crossing", "object_transfer", "chat_relay"],
    "physics_engine": "ODE",
    "max_avatars": 100
  }'

# Discover neighboring regions
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/grid/regions/neighbors?x=1000&y=1000&radius=3"
```

**Avatar Crossing Protocol**

OpenSim Next implements seamless avatar movement between regions:

```bash
# Monitor avatar crossings
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/regions/my-region/crossings/status"

# Configure crossing parameters
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/regions/my-region/crossings/config \
  -d '{
    "crossing_timeout_ms": 30000,
    "max_concurrent_crossings": 10,
    "state_transfer_compression": true,
    "validate_destination": true
  }'
```

**Object Transfer and Replication**

```bash
# Enable object sharing between regions
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/regions/source-region/objects/transfer \
  -d '{
    "object_uuid": "123e4567-e89b-12d3-a456-426614174000",
    "destination_region": "target-region",
    "transfer_type": "copy",
    "preserve_ownership": true
  }'

# Configure automatic asset replication
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/grid/asset-replication/config \
  -d '{
    "replication_strategy": "distributed",
    "replication_factor": 3,
    "cache_regions": ["cache-region-1", "cache-region-2"],
    "auto_replicate_threshold_mb": 10
  }'
```

## 8.3 Load Balancing and Auto-Scaling

### 8.3.1 Region Load Balancing

**Load Balancing Strategies**

OpenSim Next supports multiple load balancing algorithms:

1. **Round Robin Load Balancing**
   ```bash
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     -X PUT http://localhost:8090/api/load-balancer/config \
     -d '{
       "strategy": "round_robin",
       "regions": ["region-1", "region-2", "region-3"],
       "health_check_interval_ms": 5000
     }'
   ```

2. **Least Connections Load Balancing**
   ```bash
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     -X PUT http://localhost:8090/api/load-balancer/config \
     -d '{
       "strategy": "least_connections",
       "weight_factors": {
         "avatar_count": 0.4,
         "cpu_usage": 0.3,
         "memory_usage": 0.2,
         "network_load": 0.1
       }
     }'
   ```

3. **Geographic Load Balancing**
   ```bash
   curl -H "X-API-Key: $OPENSIM_API_KEY" \
     -X PUT http://localhost:8090/api/load-balancer/config \
     -d '{
       "strategy": "geographic",
       "regions": [
         {"name": "us-east", "location": {"lat": 40.7, "lon": -74.0}},
         {"name": "us-west", "location": {"lat": 37.4, "lon": -122.1}},
         {"name": "europe", "location": {"lat": 51.5, "lon": -0.1}}
       ],
       "prefer_local_region": true,
       "max_latency_ms": 100
     }'
   ```

**Load Balancer Health Monitoring**

```bash
# Monitor load balancer status
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  http://localhost:8090/api/load-balancer/status

# Get detailed load metrics
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  http://localhost:8090/api/load-balancer/metrics

# Test load balancer failover
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/load-balancer/test-failover \
  -d '{"simulate_failure": "region-1"}'
```

### 8.3.2 Auto-Scaling Configuration

**Scaling Policies and Triggers**

Configure automatic scaling based on various metrics:

```bash
# Set up CPU-based auto-scaling
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/auto-scaling/policies \
  -d '{
    "policy_name": "cpu_scaling",
    "metric": "cpu_usage",
    "scale_up_threshold": 75,
    "scale_down_threshold": 25,
    "scale_up_instances": 2,
    "scale_down_instances": 1,
    "cooldown_minutes": 10,
    "min_instances": 1,
    "max_instances": 20
  }'

# Set up avatar-count-based scaling
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/auto-scaling/policies \
  -d '{
    "policy_name": "avatar_scaling",
    "metric": "avatar_count",
    "scale_up_threshold": 80,
    "scale_down_threshold": 30,
    "target_avatars_per_region": 50,
    "scale_up_regions": 1,
    "scale_down_regions": 1
  }'

# Set up memory-based scaling
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/auto-scaling/policies \
  -d '{
    "policy_name": "memory_scaling",
    "metric": "memory_usage",
    "scale_up_threshold": 85,
    "scale_down_threshold": 40,
    "memory_target_mb": 1536,
    "evaluation_period_minutes": 5
  }'
```

**Dynamic Server Provisioning**

For cloud deployments, OpenSim Next can automatically provision new servers:

```bash
# Configure cloud provider settings
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/auto-scaling/cloud-config \
  -d '{
    "provider": "aws",
    "instance_type": "c5.2xlarge",
    "ami_id": "ami-0abcdef1234567890",
    "key_pair": "opensim-keypair",
    "security_groups": ["opensim-sg"],
    "subnet_id": "subnet-12345678",
    "user_data_script": "/path/to/startup-script.sh"
  }'

# Manual scaling operations
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/auto-scaling/scale-up \
  -d '{"instances": 3, "region_template": "production-template"}'

curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/auto-scaling/scale-down \
  -d '{"instances": 1, "strategy": "least_utilized"}'
```

## 8.4 Performance Optimization

### 8.4.1 Region Performance Tuning

**Physics Engine Optimization**

Different physics engines excel in different scenarios:

```bash
# Optimize for avatar-heavy regions (social spaces)
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/regions/social-hub/physics \
  -d '{
    "engine": "ODE",
    "config": {
      "timestep": 0.0111,
      "iterations": 10,
      "contact_max_correcting_vel": 10.0,
      "contact_surface_layer": 0.001,
      "world_cfm": 1e-5,
      "world_erp": 0.2,
      "avatar_density": 3.5,
      "avatar_capsule_width": 0.6,
      "avatar_capsule_depth": 0.45,
      "avatar_capsule_height": 1.5
    }
  }'

# Optimize for vehicle regions (racing, flying)
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/regions/racing-track/physics \
  -d '{
    "engine": "Bullet",
    "config": {
      "timestep": 0.0083,
      "max_sub_steps": 10,
      "solver_iterations": 15,
      "split_impulse": true,
      "continuous_collision": true,
      "vehicle_linear_damping": 0.2,
      "vehicle_angular_damping": 0.8,
      "terrain_collision_margin": 0.04
    }
  }'

# Optimize for particle effects regions
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/regions/particle-demo/physics \
  -d '{
    "engine": "POS",
    "config": {
      "timestep": 0.0167,
      "max_particles": 100000,
      "enable_gpu": true,
      "particle_radius": 0.1,
      "fluid_density": 1000.0,
      "viscosity": 0.01,
      "surface_tension": 0.0728,
      "gravity": [0, -9.81, 0]
    }
  }'
```

**Script Performance Optimization**

```bash
# Configure script execution limits
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/regions/my-region/scripts/config \
  -d '{
    "max_script_time_ms": 50,
    "max_script_memory_mb": 16,
    "script_timeout_ms": 10000,
    "max_scripts_per_object": 100,
    "enable_script_debugging": false,
    "script_compilation_cache": true
  }'

# Monitor script performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/regions/my-region/scripts/stats"
```

**Memory Management Optimization**

```bash
# Configure memory pools and garbage collection
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/regions/my-region/memory/config \
  -d '{
    "object_pool_size": 10000,
    "texture_cache_mb": 512,
    "mesh_cache_mb": 256,
    "sound_cache_mb": 128,
    "gc_interval_ms": 30000,
    "enable_memory_profiling": false
  }'
```

### 8.4.2 Network Performance Optimization

**Connection Pool Management**

```bash
# Configure connection pooling
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/network/connection-pool/config \
  -d '{
    "max_connections_per_region": 1000,
    "connection_timeout_ms": 30000,
    "keepalive_interval_ms": 60000,
    "max_idle_connections": 100,
    "connection_pool_size": 50
  }'

# Monitor connection pool statistics
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/network/connection-pool/stats"
```

**Bandwidth Management**

```bash
# Configure bandwidth limits and Quality of Service
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/network/bandwidth/config \
  -d '{
    "max_bandwidth_mbps": 100,
    "avatar_bandwidth_kbps": 256,
    "object_bandwidth_kbps": 512,
    "texture_bandwidth_kbps": 1024,
    "terrain_bandwidth_kbps": 128,
    "qos_enabled": true,
    "priority_weights": {
      "avatar_updates": 0.4,
      "chat_messages": 0.3,
      "object_updates": 0.2,
      "texture_downloads": 0.1
    }
  }'
```

---

**Region Management and Scaling Complete!** ✅

This comprehensive chapter provides all the tools and knowledge needed to manage OpenSim Next regions at any scale. From single development regions to enterprise grids with thousands of users, you now have the foundation for optimal virtual world operation.

---

# Chapter 9: WebSocket and Web Client Setup

**Revolutionary Web-First Virtual World Access**

OpenSim Next is the world's first virtual world server to provide native browser support alongside traditional viewer clients. This groundbreaking WebSocket implementation enables users to access virtual worlds through any modern web browser without plugins or downloads.

## 9.1 WebSocket Architecture Overview

### 9.1.1 Multi-Protocol Design

OpenSim Next supports simultaneous connections from:
- **Traditional Second Life Viewers** (Firestorm, Singularity, etc.)
- **Modern Web Browsers** (Chrome, Firefox, Safari, Edge)
- **Mobile Browsers** (iOS Safari, Android Chrome)
- **Custom Applications** via WebSocket API

### 9.1.2 WebSocket Protocol Features

**Core Capabilities:**
- Real-time bidirectional communication
- JSON-based message protocol
- Authentication integration
- Live avatar updates and region events
- Cross-platform synchronization
- Rate limiting and security controls

**Protocol Endpoints:**
```
WebSocket Server: ws://your-server:9001/ws
Web Interface: http://your-server:8080
Client Interface: http://your-server:8080/client.html
API Documentation: http://your-server:8080/api-docs
```

## 9.2 WebSocket Server Configuration

### 9.2.1 Basic WebSocket Setup

**Environment Configuration:**
```bash
# WebSocket server settings
export OPENSIM_WEBSOCKET_PORT=9001
export OPENSIM_WEB_CLIENT_PORT=8080
export OPENSIM_WEBSOCKET_MAX_CONNECTIONS=1000
export OPENSIM_WEBSOCKET_RATE_LIMIT=100
export OPENSIM_WEBSOCKET_HEARTBEAT_MS=30000

# Security settings
export OPENSIM_WEBSOCKET_REQUIRE_AUTH=true
export OPENSIM_WEBSOCKET_CORS_ORIGINS="*"
export OPENSIM_WEBSOCKET_SSL_ENABLED=false
```

**OpenSim.ini Configuration:**
```ini
[WebSocket]
    ; Enable WebSocket server
    Enabled = true
    Port = 9001
    
    ; Connection limits
    MaxConnections = 1000
    MaxConnectionsPerIP = 10
    
    ; Rate limiting (messages per second)
    RateLimit = 100
    RateLimitBurst = 200
    
    ; Heartbeat settings
    HeartbeatInterval = 30000
    ConnectionTimeout = 120000
    
    ; Security
    RequireAuthentication = true
    AllowedOrigins = "*"
    
[WebClient]
    ; Web interface settings
    Enabled = true
    Port = 8080
    StaticFilesPath = "web/"
    
    ; API settings
    EnableAPI = true
    APIDocumentation = true
```

### 9.2.2 SSL/TLS Configuration

**Production SSL Setup:**
```ini
[WebSocket]
    SSLEnabled = true
    SSLCertificatePath = "/path/to/certificate.crt"
    SSLPrivateKeyPath = "/path/to/private.key"
    SSLPort = 9443
    
[WebClient]
    SSLEnabled = true
    SSLCertificatePath = "/path/to/certificate.crt"
    SSLPrivateKeyPath = "/path/to/private.key"
    SSLPort = 8443
```

**Let's Encrypt Integration:**
```bash
# Install certificates
sudo certbot certonly --webroot -w /var/www/html -d your-domain.com

# Configure automatic renewal
echo "0 12 * * * /usr/bin/certbot renew --quiet" | sudo crontab -
```

## 9.3 Web Client Interface Setup

### 9.3.1 Static Web Files

**Directory Structure:**
```
opensim-next/web/
├── index.html              # Landing page
├── client.html             # Web client interface
├── admin.html              # Administrative interface
├── css/
│   ├── client.css          # Client styling
│   └── admin.css           # Admin styling
├── js/
│   ├── websocket-client.js # WebSocket client library
│   ├── virtual-world.js    # Virtual world rendering
│   └── admin-panel.js      # Administrative controls
└── assets/
    ├── textures/           # Default textures
    ├── sounds/             # UI sounds
    └── models/             # 3D models
```

### 9.3.2 Web Client Features

**Browser Client Capabilities:**
- Real-time avatar representation
- Chat and messaging
- Object interaction
- Inventory management
- Friend and group management
- Region navigation
- Administrative controls

**Responsive Design:**
```css
/* Mobile-first responsive design */
@media (min-width: 768px) {
    .client-interface {
        grid-template-columns: 250px 1fr 300px;
    }
}

@media (max-width: 767px) {
    .client-interface {
        grid-template-columns: 1fr;
        grid-template-rows: auto 1fr auto;
    }
}
```

## 9.4 WebSocket API Reference

### 9.4.1 Message Protocol

**Message Structure:**
```json
{
    "id": "unique-message-id",
    "timestamp": 1234567890,
    "message": {
        "type": "MessageType",
        "data": {...}
    }
}
```

**Core Message Types:**

**Authentication:**
```json
{
    "type": "Auth",
    "data": {
        "token": "authentication-token",
        "session_id": "session-identifier"
    }
}
```

**Chat Messages:**
```json
{
    "type": "ChatMessage",
    "data": {
        "from": "username",
        "message": "Hello, world!",
        "channel": 0,
        "position": {"x": 128, "y": 128, "z": 25}
    }
}
```

**Avatar Updates:**
```json
{
    "type": "AvatarUpdate",
    "data": {
        "avatar_id": "user-uuid",
        "position": {"x": 128.5, "y": 127.8, "z": 25.0},
        "rotation": {"x": 0, "y": 0, "z": 0, "w": 1},
        "animation": "walking"
    }
}
```

### 9.4.2 Client JavaScript Library

**WebSocket Client Implementation:**
```javascript
class OpenSimWebClient {
    constructor(serverUrl, options = {}) {
        this.serverUrl = serverUrl;
        this.options = {
            autoReconnect: true,
            heartbeatInterval: 30000,
            ...options
        };
        this.socket = null;
        this.messageHandlers = new Map();
        this.isConnected = false;
    }
    
    async connect() {
        return new Promise((resolve, reject) => {
            this.socket = new WebSocket(this.serverUrl);
            
            this.socket.onopen = () => {
                this.isConnected = true;
                this.startHeartbeat();
                resolve();
            };
            
            this.socket.onmessage = (event) => {
                this.handleMessage(JSON.parse(event.data));
            };
            
            this.socket.onclose = () => {
                this.isConnected = false;
                if (this.options.autoReconnect) {
                    setTimeout(() => this.connect(), 5000);
                }
            };
            
            this.socket.onerror = reject;
        });
    }
    
    sendMessage(type, data) {
        if (!this.isConnected) return false;
        
        const message = {
            id: this.generateMessageId(),
            timestamp: Date.now(),
            message: { type, data }
        };
        
        this.socket.send(JSON.stringify(message));
        return true;
    }
    
    onMessage(type, handler) {
        this.messageHandlers.set(type, handler);
    }
    
    handleMessage(message) {
        const handler = this.messageHandlers.get(message.message.type);
        if (handler) {
            handler(message.message.data);
        }
    }
    
    generateMessageId() {
        return 'msg_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
    }
    
    startHeartbeat() {
        setInterval(() => {
            if (this.isConnected) {
                this.sendMessage('Heartbeat', {});
            }
        }, this.options.heartbeatInterval);
    }
}
```

## 9.5 Cross-Platform Integration

### 9.5.1 Unified User Experience

**Synchronized Features:**
- Chat messages appear in both viewers and browsers
- Avatar movements sync across all clients
- Object changes broadcast to all users
- Friend and group updates propagate everywhere

**Protocol Bridge:**
```javascript
// Example: Bridge between WebSocket and Second Life viewer protocols
class ProtocolBridge {
    constructor(websocketClient, slViewerClient) {
        this.ws = websocketClient;
        this.sl = slViewerClient;
        this.setupBridge();
    }
    
    setupBridge() {
        // Forward chat from WebSocket to SL protocol
        this.ws.onMessage('ChatMessage', (data) => {
            this.sl.sendChatMessage(data.message, data.channel);
        });
        
        // Forward avatar updates from SL to WebSocket
        this.sl.onAvatarUpdate((avatarData) => {
            this.ws.sendMessage('AvatarUpdate', avatarData);
        });
    }
}
```

### 9.5.2 Mobile Browser Support

**Mobile Optimizations:**
```css
/* Touch-friendly interface */
.mobile-controls {
    min-height: 44px;
    min-width: 44px;
    touch-action: manipulation;
}

/* Optimize for mobile bandwidth */
.mobile-client {
    --texture-quality: low;
    --update-frequency: 30fps;
    --max-draw-distance: 96m;
}
```

**Progressive Web App Features:**
```json
{
    "name": "OpenSim Next",
    "short_name": "OpenSim",
    "start_url": "/client.html",
    "display": "standalone",
    "background_color": "#000000",
    "theme_color": "#0066cc",
    "icons": [
        {
            "src": "/assets/icon-192.png",
            "sizes": "192x192",
            "type": "image/png"
        }
    ]
}
```

## 9.6 Performance Optimization

### 9.6.1 Connection Management

**Connection Pool Configuration:**
```bash
# Configure WebSocket connection limits
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/websocket/config \
  -d '{
    "max_connections": 1000,
    "max_connections_per_ip": 10,
    "connection_timeout_ms": 120000,
    "heartbeat_interval_ms": 30000,
    "rate_limit_per_second": 100,
    "rate_limit_burst": 200
  }'
```

**Performance Monitoring:**
```bash
# Monitor WebSocket performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/websocket/stats"

# Response includes:
# - Active connections
# - Messages per second
# - Bandwidth usage
# - Error rates
# - Latency statistics
```

### 9.6.2 Message Optimization

**Efficient Message Batching:**
```javascript
class MessageBatcher {
    constructor(client, batchSize = 10, flushInterval = 16) {
        this.client = client;
        this.batch = [];
        this.batchSize = batchSize;
        
        setInterval(() => this.flush(), flushInterval);
    }
    
    add(type, data) {
        this.batch.push({ type, data });
        
        if (this.batch.length >= this.batchSize) {
            this.flush();
        }
    }
    
    flush() {
        if (this.batch.length > 0) {
            this.client.sendMessage('BatchUpdate', {
                updates: this.batch
            });
            this.batch = [];
        }
    }
}
```

## 9.7 Security Configuration

### 9.7.1 Authentication Integration

**Token-Based Authentication:**
```javascript
// Authenticate with the server
async function authenticateWebClient(username, password) {
    const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password })
    });
    
    const authData = await response.json();
    
    if (authData.success) {
        // Store token and connect to WebSocket
        localStorage.setItem('authToken', authData.token);
        await connectWebSocket(authData.token);
    }
    
    return authData;
}
```

### 9.7.2 CORS and Security Headers

**Security Configuration:**
```bash
# Configure CORS settings
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/websocket/cors \
  -d '{
    "allowed_origins": ["https://yourdomain.com"],
    "allowed_methods": ["GET", "POST"],
    "allowed_headers": ["Authorization", "Content-Type"],
    "max_age_seconds": 86400
  }'
```

## 9.8 Troubleshooting WebSocket Issues

### 9.8.1 Common Connection Problems

**Connection Diagnostics:**
```bash
# Test WebSocket connectivity
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/websocket/health"

# Check WebSocket server status
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/websocket/diagnostics"
```

**Browser Console Debugging:**
```javascript
// Enable WebSocket debugging
localStorage.setItem('debug', 'opensim:websocket');

// Monitor connection health
client.socket.addEventListener('error', (error) => {
    console.error('WebSocket error:', error);
});

client.socket.addEventListener('close', (event) => {
    console.log('WebSocket closed:', event.code, event.reason);
});
```

### 9.8.2 Performance Issues

**Bandwidth Monitoring:**
```javascript
// Monitor message rates
class PerformanceMonitor {
    constructor() {
        this.messageCount = 0;
        this.bytesReceived = 0;
        this.startTime = Date.now();
    }
    
    recordMessage(message) {
        this.messageCount++;
        this.bytesReceived += JSON.stringify(message).length;
    }
    
    getStats() {
        const elapsed = (Date.now() - this.startTime) / 1000;
        return {
            messagesPerSecond: this.messageCount / elapsed,
            bytesPerSecond: this.bytesReceived / elapsed,
            totalMessages: this.messageCount,
            totalBytes: this.bytesReceived
        };
    }
}
```

---

**WebSocket and Web Client Setup Complete!** ✅

This revolutionary chapter establishes OpenSim Next as the world's first virtual world server with native browser support. Users can now access virtual worlds through any modern web browser while maintaining full compatibility with traditional Second Life viewers.

---

# Chapter 10: Monitoring and Administration Setup

**Enterprise-Grade Observability and Management**

OpenSim Next provides comprehensive monitoring and administration capabilities designed for enterprise virtual world deployments. This chapter covers the advanced observability platform with real-time analytics, distributed tracing, centralized logging, and comprehensive administrative interfaces.

## 10.1 Monitoring Architecture Overview

### 10.1.1 Multi-Layer Observability

OpenSim Next implements a comprehensive observability stack:

**Monitoring Layers:**
- **Infrastructure Monitoring**: System resources, network, storage
- **Application Monitoring**: Service health, performance metrics, errors
- **Business Monitoring**: User engagement, virtual economy, content usage
- **Security Monitoring**: Authentication, authorization, threat detection
- **Zero Trust Analytics**: Network security, encrypted overlay performance

**Monitoring Endpoints:**
```
Admin Dashboard: http://your-server:8090
Prometheus Metrics: http://your-server:9100/metrics
Health Checks: http://your-server:9100/health
Zero Trust Analytics: http://your-server:8090/ziti/analytics
Performance Profiler: http://your-server:8090/profiler
Real-Time Stats: ws://your-server:9001/admin
```

### 10.1.2 Observability Components

**Core Components:**
- **Prometheus Integration**: Metrics collection and storage
- **Grafana Dashboards**: Visualization and alerting
- **Distributed Tracing**: Request flow analysis
- **Centralized Logging**: Structured log aggregation
- **Real-Time Analytics**: Live performance monitoring
- **Administrative API**: Programmatic management interface

## 10.2 Admin Dashboard Configuration

### 10.2.1 Dashboard Setup

**Environment Configuration:**
```bash
# Admin dashboard settings
export OPENSIM_ADMIN_PORT=8090
export OPENSIM_ADMIN_API_KEY="your-secure-admin-key"
export OPENSIM_METRICS_PORT=9100
export OPENSIM_ENABLE_PROFILER=true
export OPENSIM_ADMIN_AUTH_REQUIRED=true

# Monitoring settings
export OPENSIM_METRICS_ENABLED=true
export OPENSIM_TRACING_ENABLED=true
export OPENSIM_LOGGING_LEVEL=INFO
export OPENSIM_HEALTH_CHECK_INTERVAL=30
```

**OpenSim.ini Configuration:**
```ini
[AdminDashboard]
    ; Enable admin dashboard
    Enabled = true
    Port = 8090
    
    ; Authentication
    RequireAuthentication = true
    APIKey = "your-secure-admin-key"
    SessionTimeout = 3600
    
    ; Dashboard features
    EnableRealTimeStats = true
    EnablePerformanceProfiler = true
    EnableSystemMonitoring = true
    EnableUserManagement = true
    
[Monitoring]
    ; Prometheus metrics
    EnableMetrics = true
    MetricsPort = 9100
    MetricsPath = "/metrics"
    
    ; Health checks
    EnableHealthChecks = true
    HealthCheckInterval = 30
    HealthCheckTimeout = 10
    
    ; Logging
    LogLevel = INFO
    StructuredLogging = true
    LogFormat = JSON
    
[Tracing]
    ; Distributed tracing
    Enabled = true
    SamplingRate = 0.1
    TraceStorage = "memory"
    MaxTraces = 10000
```

### 10.2.2 Authentication and Security

**API Key Management:**
```bash
# Generate secure API key
ADMIN_KEY=$(openssl rand -hex 32)
echo "OPENSIM_ADMIN_API_KEY=$ADMIN_KEY" >> .env

# Configure API key authentication
curl -H "X-API-Key: $ADMIN_KEY" \
  -X POST http://localhost:8090/api/auth/configure \
  -d '{
    "require_api_key": true,
    "session_timeout_seconds": 3600,
    "max_sessions_per_user": 5,
    "enable_audit_logging": true
  }'
```

**Role-Based Access Control:**
```bash
# Create admin user roles
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/users/roles \
  -d '{
    "role_name": "system_admin",
    "permissions": [
      "server_management",
      "user_management", 
      "region_management",
      "database_administration",
      "security_monitoring"
    ]
  }'
```

## 10.3 Real-Time Monitoring Setup

### 10.3.1 System Metrics Collection

**Core System Metrics:**
```bash
# Monitor system performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/system/metrics"

# Response includes:
# - CPU usage per core
# - Memory usage and allocation
# - Disk I/O and space
# - Network throughput
# - Process statistics
```

**Virtual World Metrics:**
```bash
# Monitor virtual world performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/world/metrics"

# Response includes:
# - Active users and regions
# - Avatar and object counts
# - Physics engine performance
# - Script execution statistics
# - Asset transfer rates
```

### 10.3.2 Real-Time Analytics Dashboard

**Dashboard Widgets:**
- **Live User Activity**: Real-time user connections and activity
- **Performance Gauges**: CPU, memory, network utilization
- **Region Statistics**: Region health and load distribution
- **Database Metrics**: Query performance and connection pools
- **Zero Trust Analytics**: Security events and network health
- **Error Tracking**: Real-time error rates and alerts

**Dashboard Configuration:**
```javascript
// Configure real-time dashboard
const dashboardConfig = {
    refreshInterval: 5000,
    widgets: [
        {
            type: "metric_gauge",
            title: "CPU Usage",
            metric: "system.cpu.usage_percent",
            thresholds: [70, 90]
        },
        {
            type: "time_series",
            title: "Active Users",
            metrics: ["world.users.active", "world.users.concurrent"],
            timeRange: "1h"
        },
        {
            type: "status_grid",
            title: "Region Health",
            endpoint: "/api/regions/health"
        }
    ]
};
```

## 10.4 Prometheus Integration

### 10.4.1 Metrics Configuration

**Prometheus Setup:**
```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "opensim_rules.yml"

scrape_configs:
  - job_name: 'opensim-next'
    static_configs:
      - targets: ['localhost:9100']
    scrape_interval: 10s
    metrics_path: /metrics
    
  - job_name: 'opensim-admin'
    static_configs:
      - targets: ['localhost:8090']
    scrape_interval: 30s
    metrics_path: /api/metrics/prometheus

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

**Custom Metrics Registration:**
```bash
# Register custom business metrics
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/metrics/register \
  -d '{
    "name": "opensim_user_engagement_score",
    "type": "gauge",
    "description": "User engagement score based on activity",
    "labels": ["region", "user_type"]
  }'

# Register performance metrics
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/metrics/register \
  -d '{
    "name": "opensim_physics_simulation_time",
    "type": "histogram",
    "description": "Physics simulation execution time",
    "labels": ["engine_type", "region_id"],
    "buckets": [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
  }'
```

### 10.4.2 Alert Rules

**OpenSim Alert Rules:**
```yaml
# opensim_rules.yml
groups:
  - name: opensim_alerts
    rules:
      - alert: HighCPUUsage
        expr: opensim_system_cpu_usage_percent > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage detected"
          description: "CPU usage is {{ $value }}% for more than 5 minutes"
          
      - alert: RegionDown
        expr: opensim_region_health_status == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Region is down"
          description: "Region {{ $labels.region_name }} is not responding"
          
      - alert: DatabaseConnectionIssues
        expr: opensim_database_connection_errors_total > 10
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Database connection issues"
          description: "{{ $value }} database connection errors in 2 minutes"
          
      - alert: ZeroTrustSecurityEvent
        expr: opensim_ziti_security_events_total > 5
        for: 1m
        labels:
          severity: high
        annotations:
          summary: "Zero trust security events detected"
          description: "{{ $value }} security events in the last minute"
```

## 10.5 Grafana Dashboard Setup

### 10.5.1 Dashboard Installation

**Grafana Configuration:**
```yaml
# grafana/provisioning/datasources/prometheus.yml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
    access: proxy
    isDefault: true
    
  - name: OpenSim Logs
    type: loki
    url: http://loki:3100
    access: proxy
```

**OpenSim Dashboard JSON:**
```json
{
  "dashboard": {
    "title": "OpenSim Next Monitoring",
    "panels": [
      {
        "title": "Virtual World Overview",
        "type": "stat",
        "targets": [
          {
            "expr": "opensim_users_active_total",
            "legendFormat": "Active Users"
          },
          {
            "expr": "opensim_regions_healthy_total",
            "legendFormat": "Healthy Regions"
          }
        ]
      },
      {
        "title": "System Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(opensim_system_cpu_usage_percent[5m])",
            "legendFormat": "CPU Usage %"
          },
          {
            "expr": "opensim_system_memory_usage_bytes / 1024 / 1024 / 1024",
            "legendFormat": "Memory Usage GB"
          }
        ]
      },
      {
        "title": "Zero Trust Security",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(opensim_ziti_connections_total[5m])",
            "legendFormat": "Connection Rate"
          },
          {
            "expr": "opensim_ziti_security_events_total",
            "legendFormat": "Security Events"
          }
        ]
      }
    ]
  }
}
```

### 10.5.2 Custom Dashboard Widgets

**Performance Heatmap:**
```json
{
  "title": "Region Performance Heatmap",
  "type": "heatmap",
  "targets": [
    {
      "expr": "opensim_region_response_time_seconds",
      "legendFormat": "{{ region_name }}"
    }
  ],
  "heatmap": {
    "xBucketSize": "1m",
    "yBucketSize": "auto"
  }
}
```

**User Activity Timeline:**
```json
{
  "title": "User Activity Timeline",
  "type": "graph",
  "targets": [
    {
      "expr": "opensim_user_logins_total",
      "legendFormat": "Logins"
    },
    {
      "expr": "opensim_user_sessions_active",
      "legendFormat": "Active Sessions"
    }
  ]
}
```

## 10.6 Centralized Logging

### 10.6.1 Log Configuration

**Structured Logging Setup:**
```ini
[Logging]
    ; Log levels: TRACE, DEBUG, INFO, WARN, ERROR, FATAL
    LogLevel = INFO
    
    ; Output formats: TEXT, JSON
    LogFormat = JSON
    
    ; Log destinations
    LogToConsole = true
    LogToFile = true
    LogToSyslog = false
    LogToElasticsearch = true
    
    ; File logging
    LogFilePath = "/var/log/opensim/opensim.log"
    LogFileMaxSize = 100MB
    LogFileMaxBackups = 10
    LogFileMaxAge = 30
    
    ; Elasticsearch logging
    ElasticsearchURL = "http://elasticsearch:9200"
    ElasticsearchIndex = "opensim-logs"
```

**Log Aggregation:**
```bash
# Configure Fluentd for log collection
# fluentd.conf
<source>
  @type tail
  path /var/log/opensim/opensim.log
  pos_file /var/log/fluentd/opensim.log.pos
  tag opensim.app
  format json
</source>

<match opensim.**>
  @type elasticsearch
  host elasticsearch
  port 9200
  index_name opensim-logs
  type_name _doc
</match>
```

### 10.6.2 Log Analysis and Searching

**Log Search Interface:**
```bash
# Search logs via API
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/logs/search" \
  -d '{
    "query": "ERROR",
    "time_range": "1h",
    "filters": {
      "component": "physics",
      "region": "Welcome Area"
    },
    "limit": 100
  }'
```

**Common Log Queries:**
```json
{
  "queries": [
    {
      "name": "Authentication Errors",
      "query": "level:ERROR AND component:auth"
    },
    {
      "name": "Database Issues", 
      "query": "level:ERROR AND (message:*database* OR message:*sql*)"
    },
    {
      "name": "Zero Trust Events",
      "query": "component:ziti AND level:WARN"
    },
    {
      "name": "High Latency Requests",
      "query": "response_time:>1000"
    }
  ]
}
```

## 10.7 Health Checks and Service Discovery

### 10.7.1 Health Check Configuration

**Comprehensive Health Checks:**
```bash
# System health check
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:9100/health"

# Detailed component health
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/health/detailed"

# Response format:
{
  "status": "healthy",
  "timestamp": "2025-06-30T12:00:00Z",
  "components": {
    "database": {
      "status": "healthy",
      "response_time_ms": 15,
      "connection_pool": "8/10 active"
    },
    "physics": {
      "status": "healthy", 
      "engines_active": 5,
      "simulation_fps": 60
    },
    "ziti": {
      "status": "healthy",
      "overlay_status": "connected",
      "security_score": 95
    }
  }
}
```

**Custom Health Checks:**
```bash
# Register custom health check
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/health/checks \
  -d '{
    "name": "custom_service_check",
    "endpoint": "http://custom-service:8080/health",
    "interval_seconds": 30,
    "timeout_seconds": 10,
    "critical_threshold": 3
  }'
```

### 10.7.2 Service Discovery

**Service Registration:**
```bash
# Register external service
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/services/register \
  -d '{
    "name": "asset_storage_service",
    "address": "assets.yourdomain.com:443",
    "tags": ["assets", "storage", "cdn"],
    "health_check": {
      "http": "https://assets.yourdomain.com/health",
      "interval": "30s"
    }
  }'
```

## 10.8 Performance Profiling

### 10.8.1 CPU and Memory Profiling

**Profiler Configuration:**
```bash
# Enable performance profiling
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/profiler/enable \
  -d '{
    "cpu_profiling": true,
    "memory_profiling": true,
    "duration_seconds": 300,
    "sampling_rate": 100
  }'

# Get profiling data
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/profiler/cpu" > cpu_profile.prof

curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/profiler/memory" > memory_profile.prof
```

**Flame Graph Generation:**
```bash
# Generate flame graphs
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/profiler/flamegraph?type=cpu" > cpu_flamegraph.svg

curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/profiler/flamegraph?type=memory" > memory_flamegraph.svg
```

### 10.8.2 Application Performance Monitoring

**Request Tracing:**
```bash
# Enable distributed tracing
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/tracing/configure \
  -d '{
    "enabled": true,
    "sampling_rate": 0.1,
    "max_trace_duration": "30s",
    "export_traces": true
  }'

# Query traces
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/tracing/search?operation=user_login&duration=>1s"
```

## 10.9 Administrative Operations

### 10.9.1 User Management

**User Administration:**
```bash
# List all users
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/admin/users"

# Create new user
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/admin/users \
  -d '{
    "username": "newuser",
    "email": "user@example.com",
    "password": "secure_password",
    "role": "user"
  }'

# Suspend user account
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/admin/users/username/suspend
```

### 10.9.2 Region Administration

**Region Management:**
```bash
# List all regions
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/admin/regions"

# Create new region
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/admin/regions \
  -d '{
    "name": "New Region",
    "location": {"x": 1000, "y": 1000},
    "size": {"x": 256, "y": 256},
    "physics_engine": "ODE"
  }'

# Restart region
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/admin/regions/region-id/restart
```

## 10.10 Troubleshooting and Diagnostics

### 10.10.1 Diagnostic Tools

**System Diagnostics:**
```bash
# Run comprehensive diagnostics
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/run \
  -d '{
    "tests": [
      "system_resources",
      "database_connectivity", 
      "network_performance",
      "physics_engines",
      "zero_trust_connectivity"
    ]
  }'

# Get diagnostic report
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/report/latest"
```

**Performance Analysis:**
```bash
# Analyze performance bottlenecks
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/performance/analyze?duration=1h"

# Generate performance recommendations
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/performance/recommendations"
```

### 10.10.2 Emergency Procedures

**Emergency Response:**
```bash
# Emergency shutdown
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/emergency/shutdown \
  -d '{"reason": "emergency_maintenance", "grace_period_seconds": 300}'

# Force garbage collection
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/system/gc/force

# Clear all caches
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X DELETE http://localhost:8090/api/cache/clear-all
```

---

**Monitoring and Administration Setup Complete!** ✅

This comprehensive chapter provides enterprise-grade monitoring and administration capabilities for OpenSim Next. With real-time analytics, distributed tracing, centralized logging, and comprehensive administrative interfaces, you have complete observability and control over your virtual world infrastructure.

---

# Chapter 11: Backup and Disaster Recovery

**Enterprise-Grade Data Protection and Business Continuity**

OpenSim Next provides comprehensive backup and disaster recovery capabilities designed for enterprise virtual world deployments. This chapter covers automated backup strategies, point-in-time recovery, cross-region replication, and business continuity planning to ensure your virtual world infrastructure remains resilient and recoverable.

## 11.1 Backup Architecture Overview

### 11.1.1 Comprehensive Backup Strategy

OpenSim Next implements a multi-layered backup approach:

**Backup Categories:**
- **Database Backups**: Full PostgreSQL/MySQL database dumps with point-in-time recovery
- **Asset Backups**: Virtual world assets, textures, meshes, and user content
- **Configuration Backups**: Server configurations, region settings, and security policies
- **User Data Backups**: Avatar data, inventory, and social connections
- **Zero Trust Backups**: OpenZiti configurations and security certificates

**Backup Types:**
- **Full Backups**: Complete system snapshots for baseline recovery
- **Incremental Backups**: Changed data only for efficient storage
- **Differential Backups**: All changes since last full backup
- **Continuous Backups**: Real-time replication for near-zero RPO

### 11.1.2 Backup Infrastructure

**Storage Options:**
- **Local Storage**: Fast recovery, limited to single site
- **Network Storage**: Shared backup infrastructure across regions
- **Cloud Storage**: Amazon S3, Google Cloud, Azure Blob Storage
- **Hybrid Storage**: Combination of local and cloud for optimal performance

**Backup Endpoints:**
```
Backup API: http://your-server:8090/api/backup
Backup Status: http://your-server:8090/api/backup/status
Recovery API: http://your-server:8090/api/recovery
Backup Dashboard: http://your-server:8090/backup
```

## 11.2 Automated Backup Configuration

### 11.2.1 Backup Scheduling

**Environment Configuration:**
```bash
# Backup settings
export OPENSIM_BACKUP_ENABLED=true
export OPENSIM_BACKUP_STORAGE_PATH="/backup/opensim"
export OPENSIM_BACKUP_RETENTION_DAYS=90
export OPENSIM_BACKUP_COMPRESSION=true
export OPENSIM_BACKUP_ENCRYPTION=true

# Backup schedule (cron format)
export OPENSIM_BACKUP_SCHEDULE_FULL="0 2 * * 0"      # Weekly full backup
export OPENSIM_BACKUP_SCHEDULE_INCREMENTAL="0 2 * * 1-6" # Daily incremental
export OPENSIM_BACKUP_SCHEDULE_CONTINUOUS=true        # Real-time replication

# Cloud backup settings
export OPENSIM_BACKUP_CLOUD_PROVIDER="s3"
export OPENSIM_BACKUP_S3_BUCKET="opensim-backups"
export OPENSIM_BACKUP_S3_REGION="us-west-2"
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
```

**OpenSim.ini Configuration:**
```ini
[Backup]
    ; Enable backup system
    Enabled = true
    StoragePath = "/backup/opensim"
    
    ; Backup scheduling
    FullBackupSchedule = "0 2 * * 0"      ; Sunday 2 AM
    IncrementalSchedule = "0 2 * * 1-6"   ; Monday-Saturday 2 AM
    EnableContinuousBackup = true
    
    ; Backup options
    CompressionEnabled = true
    EncryptionEnabled = true
    EncryptionKey = "your-encryption-key"
    RetentionDays = 90
    
    ; Backup validation
    VerifyBackups = true
    ValidationSchedule = "0 4 * * 0"      ; Sunday 4 AM
    
[CloudBackup]
    ; Cloud storage provider: s3, gcs, azure
    Provider = "s3"
    
    ; S3 configuration
    S3Bucket = "opensim-backups"
    S3Region = "us-west-2"
    S3StorageClass = "STANDARD_IA"
    
    ; Cross-region replication
    EnableReplication = true
    ReplicationRegions = ["us-east-1", "eu-west-1"]
    
[DisasterRecovery]
    ; RTO/RPO targets
    RecoveryTimeObjective = 4h
    RecoveryPointObjective = 15m
    
    ; Disaster recovery site
    EnableDRSite = true
    DRSiteURL = "https://dr.yourdomain.com"
    DRSyncInterval = 300
```

### 11.2.2 Database Backup Configuration

**PostgreSQL Backup Setup:**
```bash
# Configure PostgreSQL backup
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/database/configure \
  -d '{
    "database_type": "postgresql",
    "connection_string": "postgresql://opensim:password@localhost:5432/opensim",
    "backup_options": {
      "format": "custom",
      "compression": 9,
      "verbose": true,
      "parallel_jobs": 4
    },
    "point_in_time_recovery": {
      "enabled": true,
      "wal_archiving": true,
      "archive_command": "cp %p /backup/postgresql/wal/%f"
    }
  }'
```

**MySQL Backup Setup:**
```bash
# Configure MySQL backup
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/database/configure \
  -d '{
    "database_type": "mysql",
    "connection_string": "mysql://opensim:password@localhost:3306/opensim",
    "backup_options": {
      "single_transaction": true,
      "routines": true,
      "triggers": true,
      "events": true,
      "flush_logs": true
    },
    "binary_logging": {
      "enabled": true,
      "expire_logs_days": 7,
      "max_binlog_size": "1GB"
    }
  }'
```

## 11.3 Asset and Content Backup

### 11.3.1 Asset Backup Strategy

**Asset Backup Configuration:**
```bash
# Configure asset backup
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/assets/configure \
  -d '{
    "backup_strategy": "incremental",
    "storage_location": "/backup/assets",
    "cloud_sync": {
      "enabled": true,
      "provider": "s3",
      "bucket": "opensim-assets-backup",
      "sync_interval_hours": 6
    },
    "compression": {
      "enabled": true,
      "algorithm": "zstd",
      "level": 3
    },
    "deduplication": {
      "enabled": true,
      "hash_algorithm": "sha256"
    }
  }'
```

**Asset Validation and Integrity:**
```bash
# Verify asset integrity
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/assets/verify \
  -d '{
    "check_type": "full",
    "repair_corrupted": true,
    "generate_report": true
  }'

# Asset backup statistics
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/backup/assets/stats"
```

### 11.3.2 User Data Backup

**User Data Protection:**
```bash
# Configure user data backup
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/userdata/configure \
  -d '{
    "include_categories": [
      "avatars",
      "inventory", 
      "friends",
      "groups",
      "preferences",
      "economy_transactions"
    ],
    "backup_frequency": "daily",
    "encryption": {
      "enabled": true,
      "algorithm": "AES-256-GCM",
      "key_rotation_days": 90
    },
    "privacy_compliance": {
      "gdpr_compatible": true,
      "anonymize_deleted_users": true,
      "data_retention_days": 2555  // 7 years
    }
  }'
```

## 11.4 Point-in-Time Recovery

### 11.4.1 Recovery Point Management

**Recovery Point Creation:**
```bash
# Create manual recovery point
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/recovery/checkpoint \
  -d '{
    "name": "pre_maintenance_checkpoint",
    "description": "Before major system upgrade",
    "include_components": [
      "database",
      "assets",
      "configurations",
      "user_data"
    ],
    "retention_days": 180
  }'

# List available recovery points
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/recovery/points"
```

**Point-in-Time Recovery Options:**
```bash
# Database point-in-time recovery
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/recovery/database/pitr \
  -d '{
    "target_time": "2025-06-30T10:30:00Z",
    "recovery_mode": "timeline_recovery",
    "target_database": "opensim_recovery",
    "validate_before_restore": true
  }'
```

### 11.4.2 Granular Recovery

**Selective Data Recovery:**
```bash
# Recover specific user data
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/recovery/selective \
  -d '{
    "recovery_type": "user_data",
    "target_time": "2025-06-30T09:00:00Z",
    "filters": {
      "user_ids": ["user-uuid-1", "user-uuid-2"],
      "data_types": ["inventory", "avatar_appearance"]
    },
    "destination": "recovery_staging"
  }'

# Recover specific region
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/recovery/region \
  -d '{
    "region_id": "region-uuid",
    "recovery_point": "checkpoint-id",
    "include_components": [
      "terrain",
      "objects", 
      "scripts",
      "region_settings"
    ]
  }'
```

## 11.5 Cross-Region Replication

### 11.5.1 Geographic Replication

**Multi-Region Setup:**
```bash
# Configure cross-region replication
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/replication/configure \
  -d '{
    "replication_mode": "async",
    "target_regions": [
      {
        "name": "us-east",
        "endpoint": "https://us-east.yourdomain.com",
        "priority": 1,
        "lag_tolerance_seconds": 300
      },
      {
        "name": "eu-west", 
        "endpoint": "https://eu-west.yourdomain.com",
        "priority": 2,
        "lag_tolerance_seconds": 600
      }
    ],
    "replication_components": [
      "database",
      "assets",
      "configurations"
    ],
    "conflict_resolution": "source_wins"
  }'
```

**Replication Monitoring:**
```bash
# Monitor replication health
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/replication/status"

# Response includes:
{
  "replication_status": "healthy",
  "last_sync": "2025-06-30T12:30:00Z",
  "lag_seconds": 45,
  "replicas": [
    {
      "region": "us-east",
      "status": "synced",
      "lag_seconds": 30,
      "bytes_behind": 0
    },
    {
      "region": "eu-west",
      "status": "syncing", 
      "lag_seconds": 60,
      "bytes_behind": 1024000
    }
  ]
}
```

### 11.5.2 Failover Management

**Automatic Failover Configuration:**
```bash
# Configure automatic failover
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/failover/configure \
  -d '{
    "auto_failover": {
      "enabled": true,
      "health_check_interval": 30,
      "failure_threshold": 3,
      "failover_timeout": 300
    },
    "failover_policies": [
      {
        "condition": "primary_unreachable",
        "action": "promote_replica",
        "target": "us-east"
      },
      {
        "condition": "data_corruption_detected",
        "action": "emergency_restore",
        "source": "latest_verified_backup"
      }
    ]
  }'
```

## 11.6 Backup Security and Encryption

### 11.6.1 Encryption at Rest

**Backup Encryption Configuration:**
```bash
# Configure backup encryption
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/encryption/configure \
  -d '{
    "encryption_algorithm": "AES-256-GCM",
    "key_management": {
      "provider": "vault",
      "vault_url": "https://vault.yourdomain.com",
      "key_rotation_days": 90,
      "key_derivation": "PBKDF2"
    },
    "compression_before_encryption": true,
    "integrity_verification": {
      "enabled": true,
      "algorithm": "HMAC-SHA256"
    }
  }'
```

**Key Management:**
```bash
# Rotate encryption keys
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/encryption/rotate-keys

# Backup encryption status
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/backup/encryption/status"
```

### 11.6.2 Access Control and Auditing

**Backup Access Control:**
```bash
# Configure backup access policies
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/access/policies \
  -d '{
    "policies": [
      {
        "name": "backup_admin",
        "permissions": [
          "backup.create",
          "backup.restore", 
          "backup.delete",
          "backup.configure"
        ],
        "principals": ["admin", "backup-service"]
      },
      {
        "name": "read_only_backup",
        "permissions": [
          "backup.list",
          "backup.status"
        ],
        "principals": ["monitoring-service"]
      }
    ]
  }'
```

**Audit Logging:**
```bash
# Query backup audit logs
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/backup/audit/logs" \
  -d '{
    "time_range": "24h",
    "actions": ["backup.create", "backup.restore", "backup.delete"],
    "users": ["admin"]
  }'
```

## 11.7 Disaster Recovery Planning

### 11.7.1 Business Continuity Planning

**RTO/RPO Definitions:**
```bash
# Configure business continuity requirements
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/disaster-recovery/requirements \
  -d '{
    "service_tiers": [
      {
        "tier": "critical",
        "services": ["user_authentication", "avatar_services"],
        "rto_minutes": 15,
        "rpo_minutes": 5
      },
      {
        "tier": "important", 
        "services": ["region_services", "asset_delivery"],
        "rto_minutes": 60,
        "rpo_minutes": 30
      },
      {
        "tier": "standard",
        "services": ["social_features", "marketplace"],
        "rto_minutes": 240,
        "rpo_minutes": 60
      }
    ]
  }'
```

**Disaster Recovery Procedures:**
```bash
# Create disaster recovery runbook
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/disaster-recovery/runbook \
  -d '{
    "scenarios": [
      {
        "name": "primary_datacenter_failure",
        "description": "Complete loss of primary datacenter",
        "procedures": [
          {
            "step": 1,
            "action": "activate_dr_site",
            "automation": "api_call",
            "endpoint": "/api/failover/activate",
            "timeout": 300
          },
          {
            "step": 2,
            "action": "update_dns",
            "automation": "script",
            "script": "/scripts/update-dns-to-dr.sh"
          },
          {
            "step": 3,
            "action": "verify_services",
            "automation": "health_check",
            "endpoints": ["auth", "regions", "assets"]
          }
        ]
      }
    ]
  }'
```

### 11.7.2 Disaster Recovery Testing

**DR Testing Framework:**
```bash
# Schedule disaster recovery test
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/disaster-recovery/test/schedule \
  -d '{
    "test_type": "failover_simulation",
    "test_environment": "staging",
    "schedule": "0 2 1 * *",  // First day of each month
    "test_scenarios": [
      "database_failure",
      "network_partition",
      "datacenter_failure"
    ],
    "success_criteria": {
      "max_rto_minutes": 30,
      "max_rpo_minutes": 15,
      "data_integrity_check": true
    }
  }'

# Execute manual DR test
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/disaster-recovery/test/execute \
  -d '{
    "test_scenario": "database_failure",
    "test_duration_minutes": 60,
    "rollback_after_test": true
  }'
```

## 11.8 Backup Monitoring and Reporting

### 11.8.1 Backup Health Monitoring

**Backup Monitoring Dashboard:**
```bash
# Get comprehensive backup status
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/backup/dashboard"

# Response includes:
{
  "backup_health": "healthy",
  "last_full_backup": "2025-06-30T02:00:00Z",
  "last_incremental_backup": "2025-06-30T12:00:00Z",
  "backup_sizes": {
    "database": "2.5GB",
    "assets": "150GB", 
    "user_data": "5.2GB",
    "total": "157.7GB"
  },
  "success_rates": {
    "full_backup": 98.5,
    "incremental_backup": 99.8,
    "cloud_sync": 96.2
  },
  "upcoming_schedules": [
    {
      "type": "incremental",
      "scheduled_time": "2025-07-01T02:00:00Z"
    }
  ]
}
```

**Backup Alerting:**
```bash
# Configure backup alerts
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/alerts/configure \
  -d '{
    "alert_rules": [
      {
        "name": "backup_failure",
        "condition": "backup_success_rate < 95",
        "severity": "critical",
        "notification_channels": ["email", "slack"]
      },
      {
        "name": "backup_size_anomaly",
        "condition": "backup_size_change > 50%",
        "severity": "warning",
        "notification_channels": ["email"]
      },
      {
        "name": "recovery_test_failure", 
        "condition": "dr_test_success = false",
        "severity": "high",
        "notification_channels": ["email", "pagerduty"]
      }
    ]
  }'
```

### 11.8.2 Compliance Reporting

**Backup Compliance Reports:**
```bash
# Generate compliance report
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/backup/reports/compliance \
  -d '{
    "report_type": "monthly",
    "compliance_frameworks": ["SOC2", "ISO27001", "GDPR"],
    "time_period": {
      "start": "2025-06-01T00:00:00Z",
      "end": "2025-06-30T23:59:59Z"
    },
    "include_sections": [
      "backup_schedules",
      "recovery_testing",
      "data_retention",
      "encryption_status",
      "access_controls"
    ]
  }'

# Download backup report
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/backup/reports/download/report-id" \
  -o backup_compliance_report.pdf
```

## 11.9 Recovery Procedures

### 11.9.1 Emergency Recovery

**Emergency Recovery Playbook:**
```bash
# Emergency full system restore
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/recovery/emergency \
  -d '{
    "recovery_point": "latest_verified_backup",
    "recovery_mode": "full_system",
    "target_environment": "production",
    "pre_recovery_validation": true,
    "emergency_contact": "admin@yourdomain.com"
  }'

# Monitor recovery progress
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/recovery/status/recovery-job-id"
```

**Recovery Validation:**
```bash
# Validate restored system
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/recovery/validate \
  -d '{
    "validation_tests": [
      "database_integrity",
      "asset_accessibility",
      "user_authentication",
      "region_functionality",
      "physics_engines"
    ]
  }'
```

### 11.9.2 Partial Recovery

**Selective Component Recovery:**
```bash
# Recover only database
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/recovery/database \
  -d '{
    "backup_id": "backup-20250630-020000",
    "recovery_mode": "new_instance",
    "target_database": "opensim_recovered",
    "pre_recovery_checks": true
  }'

# Recover specific assets
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/recovery/assets \
  -d '{
    "asset_filter": {
      "region_id": "region-uuid",
      "asset_types": ["texture", "mesh"],
      "time_range": {
        "start": "2025-06-29T00:00:00Z",
        "end": "2025-06-30T00:00:00Z"
      }
    },
    "recovery_destination": "/recovery/assets/"
  }'
```

---

**Backup and Disaster Recovery Complete!** ✅

This comprehensive chapter provides enterprise-grade backup and disaster recovery capabilities for OpenSim Next. With automated backup strategies, point-in-time recovery, cross-region replication, and comprehensive business continuity planning, your virtual world infrastructure is protected against data loss and equipped for rapid recovery from any disaster scenario.

---

# Chapter 12: Asset Management and Content Creation Workflows

**Revolutionary Content Pipeline and Asset Management System**

OpenSim Next features the world's most advanced asset management and content creation pipeline designed for modern virtual world environments. This chapter covers comprehensive asset lifecycle management, content creation workflows, cross-platform asset distribution, and enterprise-grade content security for professional virtual world deployment.

## 12.1 Asset Management Architecture

### 12.1.1 Advanced Asset Pipeline Overview

OpenSim Next implements a revolutionary asset management system supporting multiple content types with intelligent processing and distribution:

**Supported Asset Types:**
- **3D Models**: Meshes, animations, rigged models with bone structures
- **Textures**: High-resolution images, normal maps, emission maps, PBR materials
- **Audio Assets**: Spatial audio, ambient sounds, voice recordings, music
- **Scripts**: LSL scripts, JavaScript behaviors, C# assemblies
- **Animations**: Avatar animations, object animations, particle systems
- **Materials**: Physically-based rendering (PBR) material definitions
- **Prefabs**: Complete object assemblies with scripts and materials
- **Terrain**: Heightmaps, texture layers, environmental data

**Asset Processing Pipeline:**
```
┌─────────────────────────────────────────────────────────────────┐
│                OpenSim Next Asset Pipeline                     │
├─────────────────────────────────────────────────────────────────┤
│  Content Creation  │  Asset Processing  │   Distribution        │
│                    │                    │                        │
│  ┌──────────────┐  │  ┌──────────────┐  │  ┌──────────────┐     │
│  │ Blender      │──┼─►│ Format       │──┼─►│ Content      │     │
│  │ 3DS Max      │  │  │ Conversion   │  │  │ Delivery     │     │
│  │ Maya         │  │  │              │  │  │ Network      │     │
│  └──────────────┘  │  └──────────────┘  │  └──────────────┘     │
│  ┌──────────────┐  │  ┌──────────────┐  │  ┌──────────────┐     │
│  │ Photoshop    │──┼─►│ Optimization │──┼─►│ Multi-Tier   │     │
│  │ GIMP         │  │  │ & Compression│  │  │ Caching      │     │
│  │ Substance    │  │  │              │  │  │              │     │
│  └──────────────┘  │  └──────────────┘  │  └──────────────┘     │
│  ┌──────────────┐  │  ┌──────────────┐  │  ┌──────────────┐     │
│  │ Audacity     │──┼─►│ Quality      │──┼─►│ Real-Time    │     │
│  │ Pro Tools    │  │  │ Validation   │  │  │ Streaming    │     │
│  │ Audio Tools  │  │  │              │  │  │              │     │
│  └──────────────┘  │  └──────────────┘  │  └──────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

### 12.1.2 Asset Storage and Versioning

**Asset Storage Architecture:**
```bash
# Asset storage configuration
export OPENSIM_ASSET_STORAGE_TYPE="hybrid"          # local, s3, hybrid
export OPENSIM_ASSET_LOCAL_PATH="/assets/opensim"   # Local storage path
export OPENSIM_ASSET_S3_BUCKET="opensim-assets"     # S3 bucket name
export OPENSIM_ASSET_CDN_ENDPOINT="https://cdn.example.com/assets"
export OPENSIM_ASSET_CACHE_SIZE_GB="50"             # Local cache size
export OPENSIM_ASSET_VERSIONING_ENABLED="true"      # Asset versioning
export OPENSIM_ASSET_COMPRESSION_LEVEL="6"          # Compression level (1-9)
```

**Asset API Endpoints:**
```
Asset Management: http://your-server:8090/api/assets
Asset Upload: http://your-server:8090/api/assets/upload
Asset Download: http://your-server:8090/api/assets/{asset_id}
Asset Metadata: http://your-server:8090/api/assets/{asset_id}/metadata
Asset Versions: http://your-server:8090/api/assets/{asset_id}/versions
Asset Dashboard: http://your-server:8090/assets
```

## 12.2 Content Creation Integration

### 12.2.1 3D Model Import and Processing

**Supported 3D Formats:**
- **Native**: DAE (Collada), FBX, OBJ, glTF 2.0
- **Advanced**: Blender (.blend), 3DS Max (.max), Maya (.ma/.mb)
- **Game Engines**: Unity Assets, Unreal Engine assets
- **CAD Formats**: STEP, IGES, STL for architectural content

**3D Model Import Workflow:**
```bash
# Upload 3D model with automatic processing
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -F "file=@model.fbx" \
  -F "metadata={\"name\":\"Building Model\",\"category\":\"architecture\",\"lod_levels\":3}" \
  http://localhost:8090/api/assets/upload/model

# Response includes asset UUID and processing status
{
  "asset_id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "processing",
  "estimated_completion": "2025-06-30T15:30:00Z",
  "processing_options": {
    "auto_lod": true,
    "collision_mesh": true,
    "texture_optimization": true,
    "animation_compression": true
  }
}
```

**Advanced 3D Processing Options:**
```bash
# Configure advanced 3D model processing
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/assets/processing/config \
  -d '{
    "lod_generation": {
      "enabled": true,
      "levels": [1.0, 0.5, 0.25, 0.1],
      "algorithm": "quadric_decimation"
    },
    "collision_mesh": {
      "enabled": true,
      "simplification": 0.1,
      "convex_hull": false
    },
    "texture_processing": {
      "max_resolution": 1024,
      "compression": "dxt5",
      "mipmap_generation": true
    },
    "animation_optimization": {
      "keyframe_reduction": true,
      "compression_ratio": 0.8
    }
  }'
```

### 12.2.2 Texture and Material Management

**PBR Material Support:**
OpenSim Next supports physically-based rendering (PBR) materials with industry-standard workflows:

```bash
# Upload PBR material set
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -F "albedo=@material_albedo.jpg" \
  -F "normal=@material_normal.jpg" \
  -F "metallic=@material_metallic.jpg" \
  -F "roughness=@material_roughness.jpg" \
  -F "emission=@material_emission.jpg" \
  -F "metadata={\"name\":\"Metal Surface\",\"material_type\":\"pbr\"}" \
  http://localhost:8090/api/assets/upload/material

# Create material definition
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/assets/materials \
  -d '{
    "name": "Advanced Metal Surface",
    "type": "pbr",
    "properties": {
      "albedo_texture": "albedo-asset-uuid",
      "normal_texture": "normal-asset-uuid",
      "metallic_texture": "metallic-asset-uuid",
      "roughness_texture": "roughness-asset-uuid",
      "emission_texture": "emission-asset-uuid",
      "albedo_color": [0.8, 0.8, 0.8, 1.0],
      "metallic_factor": 0.9,
      "roughness_factor": 0.2,
      "emission_strength": 0.0
    }
  }'
```

**Texture Optimization and Formats:**
```bash
# Configure texture processing pipeline
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/assets/textures/config \
  -d '{
    "format_preferences": ["dxt5", "etc2", "astc", "png"],
    "quality_levels": {
      "high": {"max_size": 2048, "compression": 0.9},
      "medium": {"max_size": 1024, "compression": 0.8},
      "low": {"max_size": 512, "compression": 0.6}
    },
    "auto_mipmap": true,
    "normal_map_compression": "bc5",
    "hdr_format": "rgba16f"
  }'
```

### 12.2.3 Audio Asset Management

**Spatial Audio Processing:**
```bash
# Upload spatial audio with 3D positioning
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -F "audio=@ambient_sound.wav" \
  -F "metadata={\"name\":\"Forest Ambience\",\"spatial_type\":\"environmental\"}" \
  http://localhost:8090/api/assets/upload/audio

# Configure spatial audio properties
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/assets/audio/spatial \
  -d '{
    "asset_id": "audio-asset-uuid",
    "spatial_properties": {
      "attenuation_curve": "logarithmic",
      "max_distance": 100.0,
      "rolloff_factor": 1.0,
      "doppler_factor": 1.0,
      "air_absorption": true,
      "occlusion_enabled": true
    }
  }'
```

**Audio Format Support:**
- **Uncompressed**: WAV, AIFF, FLAC
- **Compressed**: MP3, OGG Vorbis, AAC, Opus
- **Spatial**: Ambisonics, 5.1/7.1 surround, binaural
- **Streaming**: HLS, DASH for large audio files

## 12.3 Content Distribution and Delivery

### 12.3.1 Multi-Tier Asset Caching

OpenSim Next implements intelligent multi-tier caching for optimal content delivery performance:

**Caching Architecture:**
```
┌─────────────────────────────────────────────────────────────────┐
│                    Multi-Tier Asset Caching                    │
├─────────────────────────────────────────────────────────────────┤
│  Level 1: Memory Cache (Hot Assets)     │  Level 2: SSD Cache   │
│  ┌─────────────────────────────────────┐│  ┌─────────────────────┐│
│  │ • Recently accessed assets         ││  │ • Frequently used   ││
│  │ • 1-2GB RAM allocation             ││  │   assets            ││
│  │ • Sub-millisecond access           ││  │ • 10-50GB SSD       ││
│  │ • LRU eviction policy              ││  │ • <10ms access      ││
│  └─────────────────────────────────────┘│  └─────────────────────┘│
├─────────────────────────────────────────┼─────────────────────────┤
│  Level 3: Network Storage              │  Level 4: Cold Storage │
│  ┌─────────────────────────────────────┐│  ┌─────────────────────┐│
│  │ • Archive and backup assets        ││  │ • Long-term archive ││
│  │ • NAS/SAN storage                  ││  │ • Cloud storage     ││
│  │ • 100ms-1s access                  ││  │ • Minutes to hours  ││
│  │ • High capacity                    ││  │ • Unlimited scale   ││
│  └─────────────────────────────────────┘│  └─────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

**Cache Configuration:**
```bash
# Configure multi-tier caching
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/assets/cache/config \
  -d '{
    "memory_cache": {
      "enabled": true,
      "size_mb": 2048,
      "eviction_policy": "lru",
      "hot_asset_threshold": 10
    },
    "ssd_cache": {
      "enabled": true,
      "path": "/cache/assets",
      "size_gb": 50,
      "compression": true
    },
    "network_cache": {
      "enabled": true,
      "endpoints": ["cache-1.example.com", "cache-2.example.com"],
      "replication_factor": 2
    },
    "cold_storage": {
      "enabled": true,
      "provider": "s3",
      "bucket": "opensim-cold-storage",
      "archive_threshold_days": 30
    }
  }'
```

### 12.3.2 Content Delivery Network Integration

**CDN Configuration for Global Distribution:**
```bash
# Configure CDN integration
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/assets/cdn/config \
  -d '{
    "provider": "cloudflare",
    "endpoints": {
      "us_east": "https://us-east.cdn.example.com",
      "us_west": "https://us-west.cdn.example.com",
      "europe": "https://eu.cdn.example.com",
      "asia": "https://asia.cdn.example.com"
    },
    "caching_rules": {
      "textures": {"ttl": 86400, "edge_cache": true},
      "models": {"ttl": 604800, "edge_cache": true},
      "audio": {"ttl": 43200, "edge_cache": false},
      "scripts": {"ttl": 0, "edge_cache": false}
    },
    "geographic_routing": true,
    "bandwidth_optimization": true
  }'
```

### 12.3.3 Real-Time Asset Streaming

**Progressive Asset Loading:**
```bash
# Configure progressive loading for large assets
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/assets/streaming/config \
  -d '{
    "progressive_loading": {
      "enabled": true,
      "chunk_size_kb": 64,
      "max_concurrent_chunks": 8,
      "priority_algorithm": "distance_based"
    },
    "level_of_detail": {
      "enabled": true,
      "distance_thresholds": [10, 50, 200, 1000],
      "quality_levels": ["ultra", "high", "medium", "low"]
    },
    "prefetching": {
      "enabled": true,
      "radius_meters": 100,
      "max_prefetch_mb": 100
    }
  }'
```

## 12.4 Content Security and Rights Management

### 12.4.1 Digital Rights Management (DRM)

**Asset Protection and Licensing:**
```bash
# Apply DRM protection to assets
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/assets/drm/protect \
  -d '{
    "asset_id": "protected-asset-uuid",
    "protection_level": "commercial",
    "usage_rights": {
      "copy": false,
      "modify": false,
      "transfer": true,
      "resell": false
    },
    "license_terms": {
      "duration_days": 365,
      "max_uses": 1000,
      "geographic_restrictions": ["US", "EU"],
      "usage_tracking": true
    },
    "watermarking": {
      "enabled": true,
      "type": "invisible",
      "user_identification": true
    }
  }'
```

### 12.4.2 Content Validation and Safety

**Automated Content Scanning:**
```bash
# Configure content safety scanning
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/assets/safety/config \
  -d '{
    "image_analysis": {
      "enabled": true,
      "inappropriate_content": true,
      "copyright_detection": true,
      "face_detection": true
    },
    "model_analysis": {
      "enabled": true,
      "geometry_validation": true,
      "malicious_script_detection": true,
      "performance_impact_analysis": true
    },
    "audio_analysis": {
      "enabled": true,
      "content_classification": true,
      "copyright_detection": true,
      "volume_normalization": true
    },
    "quarantine_policy": {
      "suspicious_content": "quarantine",
      "notification_required": true,
      "manual_review": true
    }
  }'
```

## 12.5 Marketplace and Asset Distribution

### 12.5.1 Integrated Marketplace

**Asset Marketplace Configuration:**
```bash
# Configure integrated marketplace
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/marketplace/config \
  -d '{
    "marketplace_enabled": true,
    "commission_rate": 0.05,
    "supported_currencies": ["USD", "L$", "OPENSIM"],
    "payment_processing": {
      "provider": "stripe",
      "supported_methods": ["credit_card", "paypal", "crypto"],
      "escrow_enabled": true,
      "automatic_delivery": true
    },
    "seller_verification": {
      "required": true,
      "identity_verification": true,
      "tax_compliance": true
    },
    "content_categories": [
      "avatars", "clothing", "buildings", "vehicles", 
      "scripts", "animations", "textures", "sounds"
    ]
  }'
```

### 12.5.2 Asset Analytics and Performance

**Content Performance Monitoring:**
```bash
# Monitor asset usage and performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/assets/analytics" | jq '{
    asset_usage: .usage_statistics,
    performance_metrics: .performance_data,
    popular_content: .trending_assets,
    revenue_analytics: .marketplace_revenue
  }'

# Get detailed asset performance report
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/assets/123e4567-e89b-12d3-a456-426614174000/analytics" | jq '{
    download_count: .downloads,
    cache_hit_ratio: .cache_performance.hit_ratio,
    average_load_time: .performance.avg_load_time_ms,
    user_ratings: .ratings.average_score,
    revenue_generated: .marketplace.total_revenue
  }'
```

## 12.6 Content Creation Workflows

### 12.6.1 Automated Content Pipelines

**CI/CD for Virtual World Content:**
```yaml
# .opensim/content-pipeline.yaml
name: "Content Creation Pipeline"
version: "1.0"

stages:
  - name: "asset_validation"
    tasks:
      - validate_file_formats
      - check_file_sizes
      - scan_for_malicious_content
      - verify_metadata_completeness

  - name: "processing"
    tasks:
      - generate_lod_levels
      - optimize_textures
      - compress_audio
      - validate_scripts

  - name: "quality_assurance"
    tasks:
      - automated_testing
      - performance_benchmarking
      - compatibility_verification
      - user_acceptance_testing

  - name: "deployment"
    tasks:
      - upload_to_staging
      - cache_warming
      - cdn_distribution
      - production_deployment

triggers:
  - git_push: "content/"
  - file_upload: "*.fbx,*.dae,*.jpg,*.png,*.wav"
  - schedule: "0 2 * * *"  # Daily at 2 AM

notifications:
  success: ["email", "slack"]
  failure: ["email", "sms"]
```

### 12.6.2 Collaborative Content Creation

**Multi-User Content Creation Environment:**
```bash
# Setup collaborative content workspace
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/content/workspace \
  -d '{
    "name": "Building Project Alpha",
    "type": "collaborative",
    "participants": [
      {"user_id": "user1", "role": "lead_designer"},
      {"user_id": "user2", "role": "3d_artist"},
      {"user_id": "user3", "role": "texture_artist"},
      {"user_id": "user4", "role": "scripter"}
    ],
    "permissions": {
      "upload_assets": ["lead_designer", "3d_artist", "texture_artist"],
      "modify_scripts": ["lead_designer", "scripter"],
      "approve_changes": ["lead_designer"],
      "publish_content": ["lead_designer"]
    },
    "version_control": {
      "enabled": true,
      "branch_protection": true,
      "review_required": true
    }
  }'
```

## 12.7 Asset Import and Export Tools

### 12.7.1 Bulk Asset Operations

**Mass Asset Import:**
```bash
# Bulk import assets from directory
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/assets/import/bulk \
  -d '{
    "source_type": "directory",
    "source_path": "/content/import/project_alpha/",
    "processing_options": {
      "auto_categorize": true,
      "generate_previews": true,
      "optimize_for_streaming": true,
      "create_lod_variants": true
    },
    "metadata_source": "filename_pattern",
    "conflict_resolution": "skip_existing",
    "notification_webhook": "https://project.example.com/webhook/import"
  }'

# Monitor bulk import progress
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/assets/import/bulk/status/import-job-uuid"
```

### 12.7.2 Asset Export and Backup

**Complete Asset Archive Export:**
```bash
# Export asset collection for backup
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/assets/export \
  -d '{
    "export_type": "complete_archive",
    "asset_filter": {
      "categories": ["textures", "models", "audio"],
      "date_range": {
        "start": "2025-01-01T00:00:00Z",
        "end": "2025-06-30T23:59:59Z"
      },
      "size_limit_gb": 100
    },
    "export_format": "opensim_archive",
    "compression": "gzip",
    "include_metadata": true,
    "destination": "s3://backup-bucket/assets/",
    "encryption": "aes256"
  }'
```

---

**Asset Management and Content Creation Complete!** ✅

This comprehensive chapter provides revolutionary asset management and content creation capabilities for OpenSim Next. With advanced content pipelines, multi-tier caching, CDN integration, marketplace functionality, collaborative workflows, and enterprise-grade security, OpenSim Next offers the world's most sophisticated virtual world content management system for professional deployment scenarios.

---

# Chapter 13: Security Hardening and Production Deployment Guide

**Enterprise-Grade Security and Production-Ready Deployment**

OpenSim Next provides comprehensive security hardening capabilities designed for enterprise and production environments. This chapter covers advanced security configurations, threat protection, compliance frameworks, production deployment best practices, and enterprise-grade infrastructure management for professional virtual world operations.

## 13.1 Security Architecture Overview

### 13.1.1 Multi-Layer Security Framework

OpenSim Next implements a comprehensive defense-in-depth security architecture:

**Security Layers:**
```
┌─────────────────────────────────────────────────────────────────┐
│                OpenSim Next Security Architecture              │
├─────────────────────────────────────────────────────────────────┤
│  Network Security Layer        │  Application Security Layer    │
│  ┌─────────────────────────────┐│  ┌─────────────────────────────┐│
│  │ • Firewall Rules           ││  │ • Authentication &          ││
│  │ • DDoS Protection          ││  │   Authorization             ││
│  │ • Rate Limiting            ││  │ • Input Validation          ││
│  │ • IP Whitelisting          ││  │ • SQL Injection Protection  ││
│  │ • SSL/TLS Termination      ││  │ • XSS Prevention            ││
│  └─────────────────────────────┘│  └─────────────────────────────┘│
├─────────────────────────────────┼─────────────────────────────────┤
│  Data Security Layer           │  Infrastructure Security       │
│  ┌─────────────────────────────┐│  ┌─────────────────────────────┐│
│  │ • Database Encryption      ││  │ • OS Hardening              ││
│  │ • Asset Encryption         ││  │ • Container Security        ││
│  │ • Key Management           ││  │ • Secrets Management        ││
│  │ • Backup Encryption        ││  │ • Vulnerability Scanning    ││
│  │ • Zero Trust Networking    ││  │ • Compliance Monitoring     ││
│  └─────────────────────────────┘│  └─────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### 13.1.2 Security Configuration Framework

**Core Security Configuration:**
```bash
# Security environment variables
export OPENSIM_SECURITY_MODE="production"           # development, staging, production
export OPENSIM_ENABLE_SECURITY_HEADERS="true"      # Security headers in HTTP responses
export OPENSIM_FORCE_HTTPS="true"                  # Redirect HTTP to HTTPS
export OPENSIM_SESSION_SECURITY="strict"           # Session security mode
export OPENSIM_AUDIT_LOGGING="enabled"             # Comprehensive audit logging
export OPENSIM_RATE_LIMITING="aggressive"          # Rate limiting configuration
export OPENSIM_IP_FILTERING="enabled"              # IP-based filtering
export OPENSIM_CONTENT_SECURITY_POLICY="strict"    # CSP for web clients
```

**Security API Endpoints:**
```
Security Dashboard: http://your-server:8090/security
Security Config: http://your-server:8090/api/security/config
Threat Monitoring: http://your-server:8090/api/security/threats
Audit Logs: http://your-server:8090/api/security/audit
Compliance Report: http://your-server:8090/api/security/compliance
```

## 13.2 Authentication and Authorization Hardening

### 13.2.1 Multi-Factor Authentication (MFA)

**Enable Enterprise MFA:**
```bash
# Configure MFA for all administrative accounts
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/mfa/config \
  -d '{
    "mfa_required": true,
    "supported_methods": ["totp", "sms", "email", "hardware_key"],
    "backup_codes": true,
    "grace_period_hours": 24,
    "enforcement_level": "admin_required",
    "trusted_devices": {
      "enabled": true,
      "trust_duration_days": 30,
      "max_trusted_devices": 5
    }
  }'

# Enable TOTP (Time-based One-Time Password)
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/security/mfa/totp/setup \
  -d '{
    "user_id": "admin_user",
    "issuer": "OpenSim Next Production",
    "account_name": "admin@example.com"
  }'
```

### 13.2.2 Advanced Password Policies

**Enterprise Password Security:**
```bash
# Configure strict password policies
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/password/policy \
  -d '{
    "minimum_length": 12,
    "require_uppercase": true,
    "require_lowercase": true,
    "require_numbers": true,
    "require_special_chars": true,
    "prevent_common_passwords": true,
    "prevent_username_inclusion": true,
    "password_history": 12,
    "max_age_days": 90,
    "lockout_policy": {
      "max_attempts": 3,
      "lockout_duration_minutes": 30,
      "progressive_delay": true
    },
    "breach_detection": {
      "enabled": true,
      "database": "haveibeenpwned",
      "action": "warn"
    }
  }'
```

### 13.2.3 Role-Based Access Control (RBAC)

**Advanced Permission Management:**
```bash
# Create enterprise security roles
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/security/roles \
  -d '{
    "role_name": "security_admin",
    "description": "Security administration and monitoring",
    "permissions": [
      "security.audit.read",
      "security.config.modify",
      "security.threats.investigate",
      "security.users.manage",
      "security.logs.access"
    ],
    "restrictions": {
      "ip_whitelist": ["192.168.1.0/24", "10.0.0.0/8"],
      "time_restrictions": {
        "allowed_hours": "06:00-22:00",
        "timezone": "UTC"
      },
      "session_limits": {
        "max_concurrent": 2,
        "max_duration_hours": 8
      }
    }
  }'

# Apply principle of least privilege
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/rbac/config \
  -d '{
    "default_permissions": "deny_all",
    "explicit_grants_required": true,
    "permission_inheritance": false,
    "audit_permission_changes": true,
    "temporary_permissions": {
      "enabled": true,
      "max_duration_hours": 24,
      "approval_required": true
    }
  }'
```

## 13.3 Network Security Hardening

### 13.3.1 Advanced Firewall Configuration

**Production Firewall Rules:**
```bash
# Configure iptables for OpenSim Next
cat > /etc/opensim/firewall-rules.sh << 'EOF'
#!/bin/bash

# Clear existing rules
iptables -F
iptables -X

# Default policies
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# Allow loopback
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT

# Allow established connections
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# SSH access (restrict to management network)
iptables -A INPUT -p tcp --dport 22 -s 192.168.1.0/24 -j ACCEPT

# OpenSim Next services
iptables -A INPUT -p tcp --dport 8080 -j ACCEPT  # Web interface
iptables -A INPUT -p tcp --dport 8090 -j ACCEPT  # Admin API
iptables -A INPUT -p tcp --dport 9000 -j ACCEPT  # SL Viewer
iptables -A INPUT -p tcp --dport 9001 -j ACCEPT  # WebSocket
iptables -A INPUT -p tcp --dport 9100 -j ACCEPT  # Metrics

# Rate limiting for public services
iptables -A INPUT -p tcp --dport 9000 -m limit --limit 25/min --limit-burst 100 -j ACCEPT
iptables -A INPUT -p tcp --dport 9001 -m limit --limit 50/min --limit-burst 200 -j ACCEPT

# DDoS protection
iptables -A INPUT -p tcp --syn -m limit --limit 1/s --limit-burst 3 -j ACCEPT
iptables -A INPUT -p tcp --syn -j DROP

# Log dropped packets
iptables -A INPUT -j LOG --log-prefix "DROPPED: "
iptables -A INPUT -j DROP

EOF

chmod +x /etc/opensim/firewall-rules.sh
/etc/opensim/firewall-rules.sh
```

### 13.3.2 DDoS Protection and Rate Limiting

**Advanced Rate Limiting Configuration:**
```bash
# Configure comprehensive rate limiting
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/rate-limiting/config \
  -d '{
    "global_limits": {
      "requests_per_minute": 1000,
      "concurrent_connections": 500,
      "bandwidth_mbps": 100
    },
    "per_ip_limits": {
      "requests_per_minute": 60,
      "concurrent_connections": 10,
      "login_attempts_per_hour": 5
    },
    "service_specific_limits": {
      "viewer_login": {
        "requests_per_minute": 10,
        "burst_allowance": 5
      },
      "asset_downloads": {
        "requests_per_minute": 100,
        "bandwidth_mbps": 10
      },
      "websocket_connections": {
        "new_connections_per_minute": 20,
        "messages_per_minute": 1000
      }
    },
    "adaptive_limiting": {
      "enabled": true,
      "increase_threshold": 0.8,
      "decrease_threshold": 0.5,
      "adjustment_factor": 0.1
    },
    "geoblocking": {
      "enabled": true,
      "blocked_countries": ["CN", "RU", "KP"],
      "allowed_countries": ["US", "CA", "GB", "DE", "AU"]
    }
  }'
```

### 13.3.3 SSL/TLS Hardening

**Advanced TLS Configuration:**
```bash
# Generate strong SSL certificates
openssl req -x509 -nodes -days 365 -newkey rsa:4096 \
  -keyout /etc/ssl/private/opensim.key \
  -out /etc/ssl/certs/opensim.crt \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=opensim.example.com"

# Configure TLS settings
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/tls/config \
  -d '{
    "minimum_version": "TLSv1.3",
    "cipher_suites": [
      "TLS_AES_256_GCM_SHA384",
      "TLS_CHACHA20_POLY1305_SHA256",
      "TLS_AES_128_GCM_SHA256"
    ],
    "perfect_forward_secrecy": true,
    "hsts_enabled": true,
    "hsts_max_age": 31536000,
    "hsts_include_subdomains": true,
    "certificate_transparency": true,
    "ocsp_stapling": true
  }'
```

## 13.4 Data Protection and Encryption

### 13.4.1 Database Security Hardening

**PostgreSQL Security Configuration:**
```bash
# PostgreSQL hardening script
cat > /etc/opensim/postgresql-security.sql << 'EOF'
-- Remove default databases and users
DROP DATABASE IF EXISTS template0;
DROP DATABASE IF EXISTS template1;

-- Create dedicated OpenSim user with minimal privileges
CREATE USER opensim_app WITH PASSWORD 'strong_random_password_here';
GRANT CONNECT ON DATABASE opensim TO opensim_app;
GRANT USAGE ON SCHEMA public TO opensim_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO opensim_app;

-- Enable row-level security
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE inventory ENABLE ROW LEVEL SECURITY;
ALTER TABLE assets ENABLE ROW LEVEL SECURITY;

-- Configure connection limits
ALTER USER opensim_app CONNECTION LIMIT 20;

-- Enable audit logging
LOAD 'pgaudit';
SET pgaudit.log = 'all';
SET pgaudit.log_catalog = on;
SET pgaudit.log_parameter = on;

-- Configure SSL requirements
ALTER SYSTEM SET ssl = on;
ALTER SYSTEM SET ssl_cert_file = '/etc/ssl/certs/postgresql.crt';
ALTER SYSTEM SET ssl_key_file = '/etc/ssl/private/postgresql.key';
ALTER SYSTEM SET ssl_min_protocol_version = 'TLSv1.2';

SELECT pg_reload_conf();
EOF

# Apply PostgreSQL security configuration
sudo -u postgres psql -f /etc/opensim/postgresql-security.sql
```

### 13.4.2 Asset Encryption and Protection

**Advanced Asset Security:**
```bash
# Configure asset encryption
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/assets/encryption \
  -d '{
    "encryption_enabled": true,
    "encryption_algorithm": "AES-256-GCM",
    "key_rotation_days": 90,
    "encrypt_in_transit": true,
    "encrypt_at_rest": true,
    "compression_before_encryption": true,
    "metadata_encryption": true,
    "key_derivation": {
      "algorithm": "PBKDF2",
      "iterations": 100000,
      "salt_length": 32
    },
    "backup_encryption": {
      "enabled": true,
      "separate_keys": true,
      "escrow_backup": true
    }
  }'

# Configure digital watermarking
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/assets/watermarking \
  -d '{
    "watermarking_enabled": true,
    "watermark_types": ["invisible", "robust", "fragile"],
    "user_identification": true,
    "timestamp_embedding": true,
    "tamper_detection": true,
    "forensic_tracking": true
  }'
```

## 13.5 Compliance and Regulatory Framework

### 13.5.1 GDPR Compliance Configuration

**Data Protection Compliance:**
```bash
# Configure GDPR compliance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/compliance/gdpr \
  -d '{
    "gdpr_enabled": true,
    "data_retention_days": 2555,  # 7 years
    "consent_management": {
      "explicit_consent_required": true,
      "consent_withdrawal": true,
      "consent_audit_trail": true
    },
    "data_subject_rights": {
      "right_to_access": true,
      "right_to_rectification": true,
      "right_to_erasure": true,
      "right_to_portability": true,
      "right_to_restrict_processing": true
    },
    "privacy_by_design": {
      "data_minimization": true,
      "purpose_limitation": true,
      "storage_limitation": true,
      "pseudonymization": true
    },
    "breach_notification": {
      "enabled": true,
      "notification_window_hours": 72,
      "severity_thresholds": {
        "high": "immediate",
        "medium": "24_hours",
        "low": "72_hours"
      }
    }
  }'
```

### 13.5.2 Industry-Specific Compliance

**Healthcare (HIPAA) Compliance:**
```bash
# Configure HIPAA compliance for healthcare environments
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/compliance/hipaa \
  -d '{
    "hipaa_enabled": true,
    "phi_protection": {
      "encryption_required": true,
      "access_logging": true,
      "minimum_necessary_rule": true,
      "business_associate_agreements": true
    },
    "technical_safeguards": {
      "unique_user_identification": true,
      "automatic_logoff": true,
      "encryption_decryption": true,
      "audit_controls": true,
      "integrity": true,
      "transmission_security": true
    },
    "administrative_safeguards": {
      "security_officer": true,
      "workforce_training": true,
      "information_access_management": true,
      "security_awareness": true,
      "incident_procedures": true,
      "contingency_plan": true
    }
  }'
```

## 13.6 Production Deployment Architecture

### 13.6.1 High-Availability Deployment

**Production Infrastructure Setup:**
```yaml
# docker-compose-production.yml
version: '3.8'

services:
  opensim-primary:
    image: opensim-next:latest
    environment:
      - OPENSIM_DEPLOYMENT_MODE=production
      - OPENSIM_INSTANCE_ROLE=primary
      - OPENSIM_DATABASE_URL=postgresql://user:pass@db-cluster:5432/opensim
      - OPENSIM_REDIS_URL=redis://redis-cluster:6379
    volumes:
      - opensim_data:/data
      - opensim_assets:/assets
      - opensim_logs:/logs
    networks:
      - opensim_internal
      - opensim_external
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9100/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  opensim-secondary:
    image: opensim-next:latest
    environment:
      - OPENSIM_DEPLOYMENT_MODE=production
      - OPENSIM_INSTANCE_ROLE=secondary
      - OPENSIM_PRIMARY_ENDPOINT=http://opensim-primary:8090
    depends_on:
      - opensim-primary
    networks:
      - opensim_internal
    restart: unless-stopped

  load-balancer:
    image: haproxy:2.8
    volumes:
      - ./haproxy.cfg:/usr/local/etc/haproxy/haproxy.cfg:ro
    ports:
      - "80:80"
      - "443:443"
      - "9000:9000"
      - "9001:9001"
    depends_on:
      - opensim-primary
      - opensim-secondary
    networks:
      - opensim_external
      - opensim_internal
    restart: unless-stopped

  database-cluster:
    image: postgres:15
    environment:
      - POSTGRES_DB=opensim
      - POSTGRES_USER=opensim
      - POSTGRES_PASSWORD_FILE=/run/secrets/db_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./postgresql.conf:/etc/postgresql/postgresql.conf
    secrets:
      - db_password
    networks:
      - opensim_internal
    restart: unless-stopped

  redis-cluster:
    image: redis:7-alpine
    command: redis-server --requirepass /run/secrets/redis_password
    volumes:
      - redis_data:/data
    secrets:
      - redis_password
    networks:
      - opensim_internal
    restart: unless-stopped

volumes:
  opensim_data:
  opensim_assets:
  opensim_logs:
  postgres_data:
  redis_data:

networks:
  opensim_internal:
    driver: bridge
    internal: true
  opensim_external:
    driver: bridge

secrets:
  db_password:
    external: true
  redis_password:
    external: true
```

### 13.6.2 Load Balancer Configuration

**HAProxy Production Configuration:**
```bash
# Create HAProxy configuration
cat > /etc/haproxy/haproxy.cfg << 'EOF'
global
    daemon
    log stdout local0
    maxconn 4096
    ssl-default-bind-ciphers ECDHE+aRSA+AESGCM:ECDHE+aRSA+SHA384:ECDHE+aRSA+SHA256:!aNULL:!eNULL:!LOW:!3DES:!MD5:!EXP:!PSK:!SRP:!DSS
    ssl-default-bind-options ssl-min-ver TLSv1.2 no-tls-tickets

defaults
    mode http
    log global
    option httplog
    option dontlognull
    option log-health-checks
    timeout connect 5s
    timeout client 30s
    timeout server 30s
    timeout check 5s

# Frontend for HTTPS traffic
frontend opensim_https
    bind *:443 ssl crt /etc/ssl/certs/opensim.pem
    redirect scheme https if !{ ssl_fc }
    
    # Security headers
    http-response set-header Strict-Transport-Security "max-age=31536000; includeSubDomains"
    http-response set-header X-Frame-Options "DENY"
    http-response set-header X-Content-Type-Options "nosniff"
    http-response set-header X-XSS-Protection "1; mode=block"
    
    # Route based on path
    acl is_websocket hdr(Upgrade) -i websocket
    acl is_admin path_beg /admin
    acl is_api path_beg /api
    
    use_backend opensim_websocket if is_websocket
    use_backend opensim_admin if is_admin
    use_backend opensim_api if is_api
    default_backend opensim_web

# Backend for OpenSim web interface
backend opensim_web
    balance roundrobin
    option httpchk GET /health
    server opensim1 opensim-primary:8080 check
    server opensim2 opensim-secondary:8080 check backup

# Backend for OpenSim admin interface
backend opensim_admin
    balance roundrobin
    option httpchk GET /api/health
    server opensim1 opensim-primary:8090 check
    server opensim2 opensim-secondary:8090 check backup

# Backend for WebSocket connections
backend opensim_websocket
    balance source
    server opensim1 opensim-primary:9001 check
    server opensim2 opensim-secondary:9001 check backup

# Frontend for Second Life viewers
frontend opensim_viewers
    bind *:9000
    mode tcp
    default_backend opensim_viewer_backend

backend opensim_viewer_backend
    mode tcp
    balance roundrobin
    server opensim1 opensim-primary:9000 check
    server opensim2 opensim-secondary:9000 check backup

# Statistics interface
listen stats
    bind *:8404
    stats enable
    stats uri /stats
    stats refresh 30s
    stats admin if LOCALHOST
EOF
```

### 13.6.3 Monitoring and Alerting

**Production Monitoring Setup:**
```bash
# Configure comprehensive monitoring
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/monitoring/production \
  -d '{
    "alerting": {
      "enabled": true,
      "channels": ["email", "slack", "pagerduty"],
      "escalation_policy": {
        "level1_timeout_minutes": 5,
        "level2_timeout_minutes": 15,
        "level3_timeout_minutes": 30
      }
    },
    "health_checks": {
      "interval_seconds": 30,
      "timeout_seconds": 10,
      "failure_threshold": 3,
      "recovery_threshold": 2
    },
    "performance_monitoring": {
      "cpu_threshold": 80,
      "memory_threshold": 85,
      "disk_threshold": 90,
      "network_threshold": 100,
      "response_time_threshold_ms": 1000
    },
    "security_monitoring": {
      "failed_login_threshold": 5,
      "suspicious_activity_detection": true,
      "intrusion_detection": true,
      "vulnerability_scanning": true
    },
    "business_metrics": {
      "user_activity_tracking": true,
      "performance_analytics": true,
      "availability_reporting": true,
      "capacity_planning": true
    }
  }'
```

## 13.7 Incident Response and Security Operations

### 13.7.1 Security Incident Response Plan

**Automated Incident Response:**
```bash
# Configure incident response procedures
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/security/incident-response \
  -d '{
    "response_team": {
      "primary_contact": "security@example.com",
      "escalation_contacts": ["cto@example.com", "ceo@example.com"],
      "on_call_rotation": true
    },
    "automated_responses": {
      "ddos_attack": {
        "action": "rate_limit_aggressive",
        "duration_minutes": 30,
        "notification": true
      },
      "brute_force": {
        "action": "ip_block",
        "duration_hours": 24,
        "notification": true
      },
      "data_breach": {
        "action": "immediate_notification",
        "containment": "isolate_affected_systems",
        "preservation": "forensic_snapshots"
      }
    },
    "compliance_requirements": {
      "notification_timeline": {
        "internal": "immediate",
        "regulatory": "72_hours",
        "customer": "without_undue_delay"
      },
      "documentation_required": true,
      "forensic_preservation": true,
      "post_incident_review": true
    }
  }'
```

### 13.7.2 Backup and Disaster Recovery

**Enterprise Backup Strategy:**
```bash
# Configure production backup procedures
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/backup/production \
  -d '{
    "backup_strategy": {
      "schedule": {
        "full_backup": "0 2 * * 0",      # Weekly full backup
        "incremental": "0 2 * * 1-6",    # Daily incremental
        "continuous": true               # Real-time replication
      },
      "retention": {
        "daily": 30,
        "weekly": 12,
        "monthly": 12,
        "yearly": 7
      },
      "storage_locations": [
        "local_raid",
        "network_storage",
        "cloud_primary",
        "cloud_secondary"
      ]
    },
    "disaster_recovery": {
      "rpo_minutes": 15,              # Recovery Point Objective
      "rto_minutes": 60,              # Recovery Time Objective
      "failover_automation": true,
      "geographic_distribution": true,
      "regular_testing": {
        "frequency": "monthly",
        "automation": true,
        "validation": true
      }
    },
    "business_continuity": {
      "critical_functions": [
        "user_authentication",
        "avatar_services",
        "asset_delivery",
        "database_access"
      ],
      "degraded_mode": {
        "enabled": true,
        "functionality": ["read_only", "essential_services"],
        "automatic_failback": true
      }
    }
  }'
```

---

**Security Hardening and Production Deployment Complete!** ✅

This comprehensive chapter provides enterprise-grade security hardening and production deployment capabilities for OpenSim Next. With multi-layer security architecture, advanced authentication systems, comprehensive compliance frameworks, high-availability deployment strategies, and professional incident response procedures, OpenSim Next meets the highest standards for production virtual world server security and operational excellence.

---

# Chapter 14: Troubleshooting Guide

**Comprehensive Problem Resolution and System Diagnostics**

OpenSim Next provides extensive diagnostic capabilities and systematic troubleshooting procedures for rapid problem resolution. This chapter covers common issues, advanced debugging techniques, performance optimization, log analysis, and proactive system health monitoring to ensure smooth operation of your virtual world infrastructure.

## 14.1 Diagnostic Framework Overview

### 14.1.1 Built-in Diagnostic System

OpenSim Next includes a comprehensive diagnostic framework designed for systematic problem identification and resolution:

**Diagnostic Categories:**
```
┌─────────────────────────────────────────────────────────────────┐
│                OpenSim Next Diagnostic Framework               │
├─────────────────────────────────────────────────────────────────┤
│  System Health Diagnostics    │  Network Diagnostics           │
│  ┌─────────────────────────────┐│  ┌─────────────────────────────┐│
│  │ • CPU Usage Monitoring     ││  │ • Connection Status         ││
│  │ • Memory Leak Detection    ││  │ • Latency Analysis          ││
│  │ • Disk Space Analysis      ││  │ • Bandwidth Utilization     ││
│  │ • Process Health Checks    ││  │ • Protocol Validation       ││
│  │ • Database Connectivity    ││  │ • WebSocket Diagnostics     ││
│  └─────────────────────────────┘│  └─────────────────────────────┘│
├─────────────────────────────────┼─────────────────────────────────┤
│  Application Diagnostics       │  Security Diagnostics          │
│  ┌─────────────────────────────┐│  ┌─────────────────────────────┐│
│  │ • Service Status Checks    ││  │ • Authentication Issues     ││
│  │ • API Response Testing     ││  │ • SSL/TLS Validation        ││
│  │ • Asset Pipeline Status    ││  │ • Rate Limiting Analysis    ││
│  │ • Physics Engine Health    ││  │ • Intrusion Detection       ││
│  │ • WebSocket Connectivity   ││  │ • Compliance Monitoring     ││
│  └─────────────────────────────┘│  └─────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

**Diagnostic API Endpoints:**
```
Diagnostic Dashboard: http://your-server:8090/diagnostics
System Health: http://your-server:8090/api/diagnostics/health
Network Analysis: http://your-server:8090/api/diagnostics/network
Performance Stats: http://your-server:8090/api/diagnostics/performance
Error Analysis: http://your-server:8090/api/diagnostics/errors
```

### 14.1.2 Automated Health Monitoring

**Continuous Health Assessment:**
```bash
# Enable comprehensive health monitoring
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/diagnostics/health/config \
  -d '{
    "monitoring_enabled": true,
    "check_interval_seconds": 30,
    "health_thresholds": {
      "cpu_usage_percent": 80,
      "memory_usage_percent": 85,
      "disk_usage_percent": 90,
      "response_time_ms": 1000,
      "error_rate_percent": 5
    },
    "automated_alerts": {
      "email_notifications": true,
      "webhook_alerts": true,
      "escalation_policy": "immediate"
    },
    "health_score_calculation": {
      "cpu_weight": 0.25,
      "memory_weight": 0.25,
      "network_weight": 0.2,
      "application_weight": 0.2,
      "security_weight": 0.1
    }
  }'
```

## 14.2 Common Issues and Solutions

### 14.2.1 Server Startup Issues

**Issue: Server Fails to Start**

**Symptoms:**
- Process exits immediately after startup
- "Address already in use" errors
- Database connection failures
- Missing configuration files

**Diagnostic Commands:**
```bash
# Check server startup logs
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/logs/startup" | jq '.startup_errors'

# Verify port availability
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/network/ports" | jq '.port_status'

# Test database connectivity
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/database/connection"
```

**Common Solutions:**

1. **Port Conflicts:**
```bash
# Check for port conflicts
netstat -tulpn | grep -E ':(8080|8090|9000|9001|9100)'

# Change ports in configuration if needed
export OPENSIM_WEB_PORT=8081
export OPENSIM_ADMIN_PORT=8091
export OPENSIM_VIEWER_PORT=9002
```

2. **Database Connection Issues:**
```bash
# Test database connection manually
psql "$DATABASE_URL" -c "SELECT version();"

# Reset database connection pool
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/database/reset-pool
```

3. **Configuration Validation:**
```bash
# Validate configuration files
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/config/validate" | jq '.validation_errors'

# Generate default configuration
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/config/generate-defaults
```

### 14.2.2 Performance Issues

**Issue: High CPU Usage**

**Symptoms:**
- Server response times > 2 seconds
- CPU usage consistently > 80%
- Physics simulation lag
- WebSocket connection timeouts

**Diagnostic Analysis:**
```bash
# Analyze CPU usage patterns
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/performance/cpu" | jq '{
    current_usage: .cpu_percent,
    top_processes: .top_processes,
    physics_engine_load: .physics_load,
    recommendations: .optimization_suggestions
  }'

# Profile function-level performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/performance/profiler" | jq '.hot_spots'
```

**Performance Optimization:**

1. **Physics Engine Optimization:**
```bash
# Switch to more efficient physics engine for high-load regions
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/regions/high-load-region/physics \
  -d '{
    "engine_type": "UBODE",
    "optimization_mode": "performance",
    "max_bodies": 5000,
    "timestep": 0.02
  }'
```

2. **Asset Caching Optimization:**
```bash
# Increase cache sizes for better performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/assets/cache/config \
  -d '{
    "memory_cache_mb": 4096,
    "ssd_cache_gb": 100,
    "cache_preloading": true,
    "compression_enabled": true
  }'
```

### 14.2.3 Database Issues

**Issue: Database Connection Timeouts**

**Symptoms:**
- "Connection pool exhausted" errors
- Slow query performance
- Transaction deadlocks
- Database connection failures

**Database Diagnostics:**
```bash
# Analyze database performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/database/performance" | jq '{
    active_connections: .connection_count,
    slow_queries: .slow_query_log,
    index_usage: .index_efficiency,
    deadlocks: .deadlock_count
  }'

# Check database locks
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/database/locks"
```

**Database Optimization:**

1. **Connection Pool Tuning:**
```bash
# Optimize connection pool settings
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/database/pool/config \
  -d '{
    "max_connections": 50,
    "min_connections": 10,
    "connection_timeout_seconds": 30,
    "idle_timeout_seconds": 300,
    "max_lifetime_seconds": 1800
  }'
```

2. **Index Optimization:**
```bash
# Analyze and create missing indexes
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/database/analyze-indexes

# Apply recommended indexes
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/database/indexes/optimize
```

### 14.2.4 WebSocket Connection Issues

**Issue: WebSocket Disconnections**

**Symptoms:**
- Frequent client disconnections
- "WebSocket connection failed" errors
- Real-time updates not working
- Browser console WebSocket errors

**WebSocket Diagnostics:**
```bash
# Analyze WebSocket connection health
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/websocket/health" | jq '{
    active_connections: .connection_count,
    connection_errors: .error_statistics,
    message_throughput: .message_rate,
    latency_stats: .latency_distribution
  }'

# Test WebSocket connectivity
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/websocket/connectivity-test"
```

**WebSocket Fixes:**

1. **Connection Stability:**
```bash
# Configure WebSocket keepalive
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/websocket/config \
  -d '{
    "heartbeat_interval_ms": 30000,
    "connection_timeout_ms": 60000,
    "max_frame_size": 65536,
    "compression_enabled": true,
    "auto_reconnect": true
  }'
```

2. **Load Balancer Configuration:**
```bash
# Fix load balancer WebSocket handling
cat >> /etc/haproxy/haproxy.cfg << 'EOF'
# WebSocket specific configuration
backend websocket_backend
    balance source
    stick-table type ip size 200k expire 30m
    stick on src
    server ws1 opensim1:9001 check
    server ws2 opensim2:9001 check backup
EOF
```

## 14.3 Advanced Debugging Techniques

### 14.3.1 Log Analysis and Correlation

**Centralized Log Analysis:**
```bash
# Search logs with advanced filtering
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/logs/search" \
  -d '{
    "level": "ERROR",
    "time_range": {
      "start": "2025-06-30T00:00:00Z",
      "end": "2025-06-30T23:59:59Z"
    },
    "components": ["database", "physics", "websocket"],
    "pattern": "connection.*failed",
    "correlation_id": true
  }' | jq '.matching_entries'

# Generate log correlation analysis
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/logs/correlate" | jq '{
    error_patterns: .common_patterns,
    temporal_correlation: .time_based_correlation,
    component_interaction: .component_correlation
  }'
```

**Log Pattern Analysis:**
```bash
# Identify recurring issues
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/logs/patterns" | jq '{
    frequent_errors: .error_frequency,
    anomaly_detection: .anomalies,
    trend_analysis: .trends,
    recommendations: .suggested_actions
  }'
```

### 14.3.2 Network Debugging

**Network Performance Analysis:**
```bash
# Comprehensive network diagnostics
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/network/analysis" | jq '{
    latency_analysis: .latency_stats,
    bandwidth_utilization: .bandwidth_usage,
    packet_loss: .packet_loss_rate,
    connection_quality: .connection_metrics
  }'

# Test connectivity to external services
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/network/connectivity-test \
  -d '{
    "targets": [
      {"host": "cdn.example.com", "port": 443, "protocol": "https"},
      {"host": "database.example.com", "port": 5432, "protocol": "tcp"},
      {"host": "redis.example.com", "port": 6379, "protocol": "tcp"}
    ],
    "timeout_seconds": 10,
    "retry_count": 3
  }'
```

### 14.3.3 Memory Leak Detection

**Memory Analysis:**
```bash
# Monitor memory usage patterns
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/memory/analysis" | jq '{
    current_usage: .memory_stats,
    leak_detection: .potential_leaks,
    allocation_patterns: .allocation_analysis,
    gc_performance: .garbage_collection_stats
  }'

# Generate memory dump for analysis
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/memory/dump \
  -d '{
    "dump_type": "heap",
    "include_stack_traces": true,
    "compression": true
  }'
```

## 14.4 Error Code Reference

### 14.4.1 HTTP Status Codes

**OpenSim Next Specific Error Codes:**

| Code | Description | Common Causes | Solution |
|------|-------------|---------------|----------|
| **1001** | Database Connection Failed | DB server down, invalid credentials | Check database status, verify connection string |
| **1002** | Physics Engine Error | Engine crash, invalid configuration | Restart physics engine, check configuration |
| **1003** | Asset Pipeline Failure | Corrupt asset, processing timeout | Validate asset, increase timeout |
| **1004** | WebSocket Authentication Failed | Invalid token, expired session | Refresh authentication token |
| **1005** | Region Load Failure | Missing region data, database corruption | Restore region from backup |
| **1006** | Zero Trust Network Error | OpenZiti connection failed | Check OpenZiti configuration |

**Error Code Lookup:**
```bash
# Get detailed error information
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/errors/1001" | jq '{
    error_description: .description,
    common_causes: .causes,
    resolution_steps: .solutions,
    related_logs: .log_references
  }'
```

### 14.4.2 System Alert Categories

**Alert Severity Levels:**

1. **CRITICAL** - Service unavailable, data loss risk
2. **HIGH** - Performance degradation, security issues
3. **MEDIUM** - Resource utilization warnings
4. **LOW** - Information and maintenance notifications

**Alert Management:**
```bash
# View active alerts
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/alerts/active" | jq '{
    critical_alerts: [.alerts[] | select(.severity == "CRITICAL")],
    alert_summary: .summary,
    trending_issues: .trends
  }'

# Acknowledge alerts
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/alerts/acknowledge \
  -d '{
    "alert_ids": ["alert-123", "alert-456"],
    "acknowledged_by": "admin_user",
    "notes": "Investigating root cause"
  }'
```

## 14.5 Performance Optimization Troubleshooting

### 14.5.1 Bottleneck Identification

**Performance Bottleneck Analysis:**
```bash
# Comprehensive bottleneck analysis
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/performance/bottlenecks" | jq '{
    cpu_bottlenecks: .cpu_analysis,
    memory_bottlenecks: .memory_analysis,
    io_bottlenecks: .io_analysis,
    network_bottlenecks: .network_analysis,
    database_bottlenecks: .database_analysis,
    recommendations: .optimization_recommendations
  }'
```

**Resource Utilization Analysis:**
```bash
# Analyze resource usage patterns
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/performance/resources" | jq '{
    cpu_usage_breakdown: .cpu_breakdown,
    memory_allocation: .memory_breakdown,
    disk_io_patterns: .disk_io,
    network_utilization: .network_stats
  }'
```

### 14.5.2 Physics Engine Optimization

**Physics Performance Diagnostics:**
```bash
# Analyze physics engine performance
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/physics/performance" | jq '{
    engine_performance: .engine_stats,
    simulation_quality: .simulation_metrics,
    optimization_opportunities: .recommendations
  }'

# Switch physics engine for problematic regions
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/physics/auto-optimize \
  -d '{
    "region_id": "problematic-region-uuid",
    "optimization_goal": "performance",
    "allow_engine_switch": true
  }'
```

## 14.6 Security Issue Troubleshooting

### 14.6.1 Authentication Problems

**Authentication Diagnostics:**
```bash
# Analyze authentication failures
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/security/auth-failures" | jq '{
    failed_attempts: .failure_statistics,
    common_failure_reasons: .failure_analysis,
    suspicious_patterns: .security_concerns,
    blocked_ips: .blocked_addresses
  }'

# Test authentication flow
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/security/auth-test \
  -d '{
    "test_type": "complete_flow",
    "test_user": "test_account",
    "include_mfa": true
  }'
```

### 14.6.2 SSL/TLS Issues

**TLS Diagnostics:**
```bash
# Analyze SSL/TLS configuration
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/security/tls-analysis" | jq '{
    certificate_status: .cert_analysis,
    cipher_strength: .cipher_analysis,
    protocol_support: .protocol_analysis,
    security_score: .overall_score
  }'

# Test SSL/TLS connectivity
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/security/tls-test \
  -d '{
    "endpoints": [
      "https://localhost:8090",
      "wss://localhost:9001"
    ],
    "test_ciphers": true,
    "test_protocols": true
  }'
```

## 14.7 Proactive Monitoring and Maintenance

### 14.7.1 Predictive Issue Detection

**Anomaly Detection:**
```bash
# Enable predictive monitoring
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/diagnostics/predictive/config \
  -d '{
    "anomaly_detection": {
      "enabled": true,
      "sensitivity": "medium",
      "learning_period_days": 7,
      "alert_threshold": 0.8
    },
    "trend_analysis": {
      "enabled": true,
      "prediction_window_hours": 24,
      "confidence_threshold": 0.75
    },
    "capacity_planning": {
      "enabled": true,
      "growth_prediction": true,
      "resource_forecasting": true
    }
  }'

# Get predictive analysis
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  "http://localhost:8090/api/diagnostics/predictive/analysis" | jq '{
    predicted_issues: .predictions,
    capacity_warnings: .capacity_analysis,
    maintenance_recommendations: .maintenance_schedule
  }'
```

### 14.7.2 Automated Recovery Procedures

**Self-Healing Configuration:**
```bash
# Configure automated recovery
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X PUT http://localhost:8090/api/diagnostics/recovery/config \
  -d '{
    "auto_recovery_enabled": true,
    "recovery_procedures": {
      "database_connection_failure": {
        "action": "restart_connection_pool",
        "max_attempts": 3,
        "delay_seconds": 30
      },
      "high_memory_usage": {
        "action": "trigger_garbage_collection",
        "threshold": 90,
        "cooldown_minutes": 5
      },
      "physics_engine_crash": {
        "action": "restart_physics_engine",
        "max_attempts": 2,
        "delay_seconds": 60
      }
    },
    "escalation_policy": {
      "notification_delay_minutes": 5,
      "escalation_levels": ["email", "sms", "pager"]
    }
  }'
```

## 14.8 Emergency Procedures

### 14.8.1 System Recovery

**Emergency System Recovery:**
```bash
# Emergency server restart with diagnostics
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/emergency/restart \
  -d '{
    "restart_type": "graceful",
    "backup_before_restart": true,
    "diagnostic_collection": true,
    "notification_required": true
  }'

# Emergency rollback to previous version
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/emergency/rollback \
  -d '{
    "rollback_target": "previous_stable",
    "preserve_data": true,
    "validation_required": true
  }'
```

### 14.8.2 Data Recovery

**Emergency Data Recovery:**
```bash
# Emergency database recovery
curl -H "X-API-Key: $OPENSIM_API_KEY" \
  -X POST http://localhost:8090/api/diagnostics/emergency/data-recovery \
  -d '{
    "recovery_type": "point_in_time",
    "target_timestamp": "2025-06-30T12:00:00Z",
    "validation_required": true,
    "backup_current_state": true
  }'
```

---

**Troubleshooting Guide Complete!** ✅

This comprehensive troubleshooting guide provides systematic problem resolution capabilities for OpenSim Next. With advanced diagnostic frameworks, detailed error analysis, performance optimization techniques, security issue resolution, predictive monitoring, and emergency recovery procedures, administrators have complete tools for maintaining optimal virtual world server operation and quickly resolving any issues that arise.

---

# Chapter 15: API Reference for Developers and Integrators

**Comprehensive API Documentation and Integration Guide**

OpenSim Next provides extensive REST APIs, WebSocket protocols, and multi-language SDKs for seamless integration with external applications, custom clients, and enterprise systems. This chapter provides complete API documentation, authentication methods, integration examples, and best practices for professional development and enterprise deployment.

## 15.1 API Architecture Overview

### 15.1.1 Multi-Protocol API Framework

OpenSim Next implements a comprehensive API architecture supporting multiple protocols and integration patterns:

**API Protocol Stack:**
```
┌─────────────────────────────────────────────────────────────────┐
│                OpenSim Next API Architecture                   │
├─────────────────────────────────────────────────────────────────┤
│  Client Applications        │  Integration Protocols            │
│  ┌─────────────────────────┐│  ┌─────────────────────────────────┐│
│  │ Web Applications       ││  │ REST API (HTTP/HTTPS)          ││
│  │ Mobile Apps            ││  │ WebSocket (Real-time)          ││
│  │ Desktop Clients        ││  │ GraphQL (Advanced Queries)     ││
│  │ Enterprise Systems     ││  │ gRPC (High Performance)        ││
│  │ Third-party Services   ││  │ Webhook (Event Notifications)  ││
│  └─────────────────────────┘│  └─────────────────────────────────┘│
├─────────────────────────────┼─────────────────────────────────────┤
│  Authentication Layer       │  Data Format Support               │
│  ┌─────────────────────────┐│  ┌─────────────────────────────────┐│
│  │ API Keys               ││  │ JSON (Primary)                 ││
│  │ OAuth 2.0              ││  │ XML (Legacy Compatibility)     ││
│  │ JWT Tokens             ││  │ MessagePack (Compact)          ││
│  │ Session Authentication ││  │ Protocol Buffers (gRPC)        ││
│  │ Rate Limiting          ││  │ CBOR (IoT Integration)         ││
│  └─────────────────────────┘│  └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### 15.1.2 API Endpoints Overview

**Core API Categories:**

| Category | Base URL | Description | Authentication |
|----------|----------|-------------|----------------|
| **Admin API** | `/api/admin` | Server administration and management | API Key Required |
| **User API** | `/api/users` | User management and profiles | API Key + User Auth |
| **Region API** | `/api/regions` | Region management and control | API Key Required |
| **Asset API** | `/api/assets` | Asset upload, download, management | API Key + Permissions |
| **Economy API** | `/api/economy` | Virtual currency and transactions | API Key + User Auth |
| **Social API** | `/api/social` | Friends, groups, messaging | User Authentication |
| **Monitoring API** | `/api/monitoring` | System metrics and health | API Key Required |
| **WebSocket API** | `/ws` | Real-time communication | Token-based |

**API Base URLs:**
```
Production: https://your-domain.com:8090/api
Development: http://localhost:8090/api
WebSocket: wss://your-domain.com:9001/ws
```

## 15.2 Authentication and Authorization

### 15.2.1 API Key Authentication

**API Key Management:**
```bash
# Generate new API key
curl -X POST http://localhost:8090/api/auth/keys \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "name": "Production Integration",
    "permissions": ["read:users", "write:regions", "admin:monitoring"],
    "expires_at": "2026-12-31T23:59:59Z",
    "rate_limit": 1000,
    "ip_whitelist": ["192.168.1.0/24", "10.0.0.0/8"]
  }' | jq '.api_key'

# Use API key in requests
curl -H "X-API-Key: your-api-key-here" \
  http://localhost:8090/api/server/status
```

**API Key Permissions:**
```json
{
  "permissions": {
    "read": ["users", "regions", "assets", "monitoring"],
    "write": ["regions", "assets", "economy"],
    "admin": ["server", "users", "monitoring", "security"],
    "websocket": ["connect", "broadcast", "rooms"]
  },
  "rate_limits": {
    "requests_per_minute": 1000,
    "burst_capacity": 100,
    "concurrent_connections": 50
  }
}
```

### 15.2.2 OAuth 2.0 Integration

**OAuth 2.0 Flow Implementation:**
```bash
# Step 1: Authorization URL
https://your-domain.com:8090/oauth/authorize?
  client_id=your_client_id&
  response_type=code&
  scope=read:users,write:regions&
  redirect_uri=https://your-app.com/callback&
  state=random_state_string

# Step 2: Exchange code for token
curl -X POST http://localhost:8090/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code&
      code=authorization_code_here&
      client_id=your_client_id&
      client_secret=your_client_secret&
      redirect_uri=https://your-app.com/callback"

# Step 3: Use access token
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  http://localhost:8090/api/users/profile
```

### 15.2.3 JWT Token Authentication

**JWT Token Management:**
```bash
# Login and get JWT token
curl -X POST http://localhost:8090/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin_user",
    "password": "secure_password",
    "mfa_token": "123456"
  }' | jq '.jwt_token'

# Use JWT token for authenticated requests
curl -H "Authorization: Bearer $JWT_TOKEN" \
  http://localhost:8090/api/admin/dashboard

# Refresh JWT token
curl -X POST http://localhost:8090/api/auth/refresh \
  -H "Authorization: Bearer $JWT_TOKEN"
```

## 15.3 REST API Reference

### 15.3.1 Server Administration API

**Server Status and Health:**
```bash
# Get server status
GET /api/server/status
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/api/server/status

Response:
{
  "status": "running",
  "version": "1.0.0",
  "instance_id": "instance-1234567890",
  "uptime_seconds": 86400,
  "health_score": 95.5,
  "active_connections": 150,
  "memory_usage_mb": 2048,
  "cpu_usage_percent": 35.2
}

# Get detailed server information
GET /api/server/info
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/api/server/info

# Server configuration
GET /api/server/config
PUT /api/server/config
curl -H "X-API-Key: $API_KEY" \
  -X PUT http://localhost:8090/api/server/config \
  -d '{
    "max_connections": 1000,
    "log_level": "info",
    "physics_engine": "ODE",
    "websocket_enabled": true
  }'
```

**Region Management:**
```bash
# List all regions
GET /api/regions
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/api/regions

# Get specific region
GET /api/regions/{region_id}
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/api/regions/550e8400-e29b-41d4-a716-446655440000

# Create new region
POST /api/regions
curl -H "X-API-Key: $API_KEY" \
  -X POST http://localhost:8090/api/regions \
  -d '{
    "name": "New Region",
    "size": {"x": 256, "y": 256},
    "location": {"x": 1000, "y": 1000},
    "physics_engine": "ODE",
    "max_avatars": 100,
    "public": true
  }'

# Update region configuration
PUT /api/regions/{region_id}
curl -H "X-API-Key: $API_KEY" \
  -X PUT http://localhost:8090/api/regions/region-uuid \
  -d '{
    "physics_engine": "Bullet",
    "max_avatars": 200,
    "description": "Updated region description"
  }'

# Delete region
DELETE /api/regions/{region_id}
curl -H "X-API-Key: $API_KEY" \
  -X DELETE http://localhost:8090/api/regions/region-uuid
```

### 15.3.2 User Management API

**User Operations:**
```bash
# List users with pagination
GET /api/users?page=1&limit=50&sort=created_at
curl -H "X-API-Key: $API_KEY" \
  "http://localhost:8090/api/users?page=1&limit=50"

# Get user profile
GET /api/users/{user_id}
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/api/users/123e4567-e89b-12d3-a456-426614174000

Response:
{
  "user_id": "123e4567-e89b-12d3-a456-426614174000",
  "username": "john_doe",
  "email": "john@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "created_at": "2025-01-15T10:00:00Z",
  "last_login": "2025-06-30T15:30:00Z",
  "status": "active",
  "profile": {
    "avatar_appearance": {},
    "bio": "Virtual world enthusiast",
    "location": "Virtual City"
  }
}

# Create new user
POST /api/users
curl -H "X-API-Key: $API_KEY" \
  -X POST http://localhost:8090/api/users \
  -d '{
    "username": "new_user",
    "email": "newuser@example.com",
    "password": "secure_password",
    "first_name": "New",
    "last_name": "User"
  }'

# Update user profile
PUT /api/users/{user_id}
curl -H "X-API-Key: $API_KEY" \
  -X PUT http://localhost:8090/api/users/user-uuid \
  -d '{
    "bio": "Updated biography",
    "location": "New Virtual City"
  }'

# User authentication status
GET /api/users/{user_id}/auth-status
POST /api/users/{user_id}/reset-password
POST /api/users/{user_id}/disable
POST /api/users/{user_id}/enable
```

### 15.3.3 Asset Management API

**Asset Operations:**
```bash
# Upload new asset
POST /api/assets/upload
curl -H "X-API-Key: $API_KEY" \
  -F "file=@texture.jpg" \
  -F "metadata={\"name\":\"Building Texture\",\"category\":\"texture\"}" \
  http://localhost:8090/api/assets/upload

Response:
{
  "asset_id": "asset-uuid-here",
  "name": "Building Texture",
  "type": "texture",
  "size_bytes": 1048576,
  "upload_status": "processing",
  "download_url": "/api/assets/asset-uuid-here/download"
}

# Get asset metadata
GET /api/assets/{asset_id}
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/api/assets/asset-uuid-here

# Download asset
GET /api/assets/{asset_id}/download
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/api/assets/asset-uuid-here/download \
  -o downloaded_asset.jpg

# List assets with filtering
GET /api/assets?type=texture&creator=user-uuid&page=1&limit=20
curl -H "X-API-Key: $API_KEY" \
  "http://localhost:8090/api/assets?type=texture&page=1&limit=20"

# Update asset metadata
PUT /api/assets/{asset_id}/metadata
curl -H "X-API-Key: $API_KEY" \
  -X PUT http://localhost:8090/api/assets/asset-uuid/metadata \
  -d '{
    "name": "Updated Asset Name",
    "description": "Updated description",
    "tags": ["building", "texture", "stone"]
  }'

# Delete asset
DELETE /api/assets/{asset_id}
curl -H "X-API-Key: $API_KEY" \
  -X DELETE http://localhost:8090/api/assets/asset-uuid
```

### 15.3.4 Economy and Transactions API

**Virtual Economy Operations:**
```bash
# Get user balance
GET /api/economy/users/{user_id}/balance
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/api/economy/users/user-uuid/balance

Response:
{
  "user_id": "user-uuid",
  "balances": {
    "L$": 5000,
    "OPENSIM": 1500,
    "USD": 25.50
  },
  "total_usd_value": 75.50
}

# Transfer currency
POST /api/economy/transfer
curl -H "X-API-Key: $API_KEY" \
  -X POST http://localhost:8090/api/economy/transfer \
  -d '{
    "from_user": "sender-uuid",
    "to_user": "recipient-uuid",
    "amount": 100,
    "currency": "L$",
    "description": "Payment for virtual goods"
  }'

# Get transaction history
GET /api/economy/users/{user_id}/transactions?page=1&limit=50
curl -H "X-API-Key: $API_KEY" \
  "http://localhost:8090/api/economy/users/user-uuid/transactions"

# Create marketplace listing
POST /api/economy/marketplace/listings
curl -H "X-API-Key: $API_KEY" \
  -X POST http://localhost:8090/api/economy/marketplace/listings \
  -d '{
    "asset_id": "asset-uuid",
    "title": "Premium Building Texture",
    "description": "High-quality stone texture",
    "price": 50,
    "currency": "L$",
    "category": "textures"
  }'

# Get marketplace listings
GET /api/economy/marketplace/listings?category=textures&max_price=100
curl -H "X-API-Key: $API_KEY" \
  "http://localhost:8090/api/economy/marketplace/listings?category=textures"
```

## 15.4 WebSocket API Reference

### 15.4.1 WebSocket Connection and Authentication

**WebSocket Connection Setup:**
```javascript
// JavaScript WebSocket client example
const ws = new WebSocket('ws://localhost:9001/ws');

// Authentication message
const authMessage = {
  id: "auth-1",
  timestamp: Date.now(),
  message: {
    type: "Auth",
    token: "your-jwt-token-here",
    session_id: null
  }
};

ws.onopen = function() {
  ws.send(JSON.stringify(authMessage));
};

ws.onmessage = function(event) {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
};
```

**WebSocket Message Format:**
```json
{
  "id": "unique-message-id",
  "timestamp": 1719843600000,
  "message": {
    "type": "MessageType",
    "payload": {}
  }
}
```

### 15.4.2 Real-Time Communication Messages

**Chat and Messaging:**
```javascript
// Send chat message
const chatMessage = {
  id: "chat-" + Date.now(),
  timestamp: Date.now(),
  message: {
    type: "ChatMessage",
    from: "user-uuid",
    message: "Hello virtual world!",
    channel: 0,
    position: {"x": 128, "y": 128, "z": 25}
  }
};

// Avatar movement update
const avatarUpdate = {
  id: "avatar-" + Date.now(),
  timestamp: Date.now(),
  message: {
    type: "AvatarUpdate",
    user_id: "user-uuid",
    position: {"x": 130, "y": 125, "z": 25},
    rotation: {"x": 0, "y": 0, "z": 0, "w": 1},
    animation: "walking"
  }
};

// Object update
const objectUpdate = {
  id: "object-" + Date.now(),
  timestamp: Date.now(),
  message: {
    type: "ObjectUpdate",
    object_id: "object-uuid",
    position: {"x": 100, "y": 100, "z": 22},
    rotation: {"x": 0, "y": 0, "z": 0, "w": 1},
    scale: {"x": 1, "y": 1, "z": 1}
  }
};
```

**System Events:**
```javascript
// Heartbeat (sent every 30 seconds)
const heartbeat = {
  id: "heartbeat-" + Date.now(),
  timestamp: Date.now(),
  message: {
    type: "Heartbeat"
  }
};

// Region events
const regionEvent = {
  id: "region-" + Date.now(),
  timestamp: Date.now(),
  message: {
    type: "RegionEvent",
    event_type: "user_entered",
    region_id: "region-uuid",
    user_id: "user-uuid",
    data: {}
  }
};
```

## 15.5 SDK Integration Examples

### 15.5.1 JavaScript/TypeScript SDK

**Installation and Setup:**
```bash
npm install opensim-next-sdk
```

**Basic Usage:**
```typescript
import { OpenSimClient } from 'opensim-next-sdk';

// Initialize client
const client = new OpenSimClient({
  baseUrl: 'http://localhost:8090',
  apiKey: 'your-api-key',
  websocketUrl: 'ws://localhost:9001'
});

// Authenticate
await client.auth.login('username', 'password');

// Get server status
const status = await client.server.getStatus();
console.log('Server status:', status);

// List regions
const regions = await client.regions.list();
console.log('Available regions:', regions);

// Upload asset
const asset = await client.assets.upload(file, {
  name: 'My Texture',
  category: 'texture'
});

// Real-time WebSocket connection
client.websocket.connect();
client.websocket.on('ChatMessage', (message) => {
  console.log('Chat received:', message);
});

// Send chat message
client.websocket.sendChat('Hello from SDK!', 0);
```

### 15.5.2 Python SDK

**Installation and Setup:**
```bash
pip install opensim-next-python
```

**Basic Usage:**
```python
from opensim_next import OpenSimClient
import asyncio

async def main():
    # Initialize client
    client = OpenSimClient(
        base_url='http://localhost:8090',
        api_key='your-api-key',
        websocket_url='ws://localhost:9001'
    )
    
    # Authenticate
    await client.auth.login('username', 'password')
    
    # Get server status
    status = await client.server.get_status()
    print(f"Server status: {status}")
    
    # List users
    users = await client.users.list(page=1, limit=10)
    print(f"Users: {users}")
    
    # Upload asset
    with open('texture.jpg', 'rb') as f:
        asset = await client.assets.upload(
            f, 
            name='Python Texture',
            category='texture'
        )
    
    # WebSocket connection
    async def on_chat_message(message):
        print(f"Chat: {message['from']}: {message['message']}")
    
    await client.websocket.connect()
    client.websocket.on('ChatMessage', on_chat_message)
    
    # Send chat message
    await client.websocket.send_chat('Hello from Python!')

# Run the async function
asyncio.run(main())
```

### 15.5.3 Rust SDK

**Cargo.toml:**
```toml
[dependencies]
opensim-next-sdk = "1.0.0"
tokio = { version = "1.0", features = ["full"] }
```

**Basic Usage:**
```rust
use opensim_next_sdk::{OpenSimClient, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let config = ClientConfig {
        base_url: "http://localhost:8090".to_string(),
        api_key: Some("your-api-key".to_string()),
        websocket_url: Some("ws://localhost:9001".to_string()),
    };
    
    let client = OpenSimClient::new(config).await?;
    
    // Authenticate
    client.auth().login("username", "password").await?;
    
    // Get server status
    let status = client.server().get_status().await?;
    println!("Server status: {:?}", status);
    
    // List regions
    let regions = client.regions().list().await?;
    println!("Regions: {:?}", regions);
    
    // WebSocket connection
    let mut ws = client.websocket().connect().await?;
    
    // Handle messages
    while let Some(message) = ws.next().await {
        match message? {
            WebSocketMessage::ChatMessage(chat) => {
                println!("Chat: {}: {}", chat.from, chat.message);
            }
            WebSocketMessage::AvatarUpdate(update) => {
                println!("Avatar update: {:?}", update);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

## 15.6 Error Handling and Status Codes

### 15.6.1 HTTP Status Codes

**Standard HTTP Status Codes:**

| Code | Status | Description | Example |
|------|--------|-------------|---------|
| **200** | OK | Request successful | Successful API call |
| **201** | Created | Resource created successfully | New user created |
| **400** | Bad Request | Invalid request format | Missing required fields |
| **401** | Unauthorized | Authentication required | Invalid API key |
| **403** | Forbidden | Insufficient permissions | Access denied |
| **404** | Not Found | Resource not found | User does not exist |
| **409** | Conflict | Resource conflict | Username already exists |
| **422** | Unprocessable Entity | Validation errors | Invalid data format |
| **429** | Too Many Requests | Rate limit exceeded | API rate limit hit |
| **500** | Internal Server Error | Server error | Database connection failed |

**OpenSim-Specific Error Codes:**

| Code | Description | Resolution |
|------|-------------|------------|
| **1001** | Database Connection Failed | Check database status |
| **1002** | Physics Engine Error | Restart physics engine |
| **1003** | Asset Pipeline Failure | Retry asset upload |
| **1004** | WebSocket Authentication Failed | Refresh authentication token |
| **1005** | Region Load Failure | Check region configuration |
| **1006** | Zero Trust Network Error | Verify OpenZiti setup |

### 15.6.2 Error Response Format

**Standard Error Response:**
```json
{
  "error": {
    "code": 1001,
    "message": "Database connection failed",
    "details": "Unable to connect to PostgreSQL database",
    "timestamp": "2025-06-30T15:30:00Z",
    "request_id": "req-123e4567-e89b-12d3-a456-426614174000",
    "documentation_url": "https://docs.opensim-next.org/api/errors/1001"
  },
  "retry_after": 30,
  "support_contact": "support@opensim-next.org"
}
```

### 15.6.3 Rate Limiting

**Rate Limit Headers:**
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1719843660
X-RateLimit-Retry-After: 60
```

**Rate Limit Configuration:**
```bash
# Configure rate limits per API key
curl -H "X-API-Key: $API_KEY" \
  -X PUT http://localhost:8090/api/auth/keys/key-uuid/rate-limits \
  -d '{
    "requests_per_minute": 1000,
    "burst_capacity": 100,
    "websocket_connections": 10,
    "upload_bandwidth_mbps": 50
  }'
```

## 15.7 Webhook Integration

### 15.7.1 Webhook Configuration

**Setting Up Webhooks:**
```bash
# Register webhook endpoint
curl -H "X-API-Key: $API_KEY" \
  -X POST http://localhost:8090/api/webhooks \
  -d '{
    "url": "https://your-app.com/webhook",
    "events": ["user.created", "region.updated", "transaction.completed"],
    "secret": "webhook-secret-key",
    "active": true,
    "retry_policy": {
      "max_attempts": 3,
      "retry_delay_seconds": 60
    }
  }'

# List webhooks
GET /api/webhooks
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/api/webhooks

# Update webhook
PUT /api/webhooks/{webhook_id}
curl -H "X-API-Key: $API_KEY" \
  -X PUT http://localhost:8090/api/webhooks/webhook-uuid \
  -d '{
    "events": ["user.created", "user.updated", "user.deleted"],
    "active": true
  }'
```

### 15.7.2 Webhook Event Types

**Available Webhook Events:**

| Event | Description | Payload |
|-------|-------------|---------|
| `user.created` | New user registered | User object |
| `user.updated` | User profile updated | User object with changes |
| `user.deleted` | User account deleted | User ID and timestamp |
| `region.created` | New region created | Region object |
| `region.updated` | Region configuration changed | Region object with changes |
| `asset.uploaded` | New asset uploaded | Asset metadata |
| `transaction.completed` | Economy transaction finished | Transaction details |
| `chat.message` | Chat message sent | Message content and metadata |

**Webhook Payload Example:**
```json
{
  "event": "user.created",
  "timestamp": "2025-06-30T15:30:00Z",
  "webhook_id": "webhook-uuid",
  "data": {
    "user_id": "123e4567-e89b-12d3-a456-426614174000",
    "username": "new_user",
    "email": "newuser@example.com",
    "created_at": "2025-06-30T15:30:00Z"
  },
  "signature": "sha256=webhook-signature-here"
}
```

## 15.8 API Versioning and Migration

### 15.8.1 API Versioning Strategy

**Version Header:**
```bash
# Specify API version in header
curl -H "X-API-Key: $API_KEY" \
  -H "X-API-Version: v1" \
  http://localhost:8090/api/users

# URL-based versioning (alternative)
curl -H "X-API-Key: $API_KEY" \
  http://localhost:8090/v1/api/users
```

**Version Compatibility:**
```json
{
  "supported_versions": ["v1", "v2"],
  "current_version": "v2",
  "deprecated_versions": [],
  "sunset_dates": {
    "v1": "2026-12-31T23:59:59Z"
  }
}
```

### 15.8.2 Migration Guide

**API v1 to v2 Migration:**
```bash
# v1 User endpoint (deprecated)
GET /v1/api/user/{user_id}

# v2 User endpoint (current)
GET /v2/api/users/{user_id}

# Breaking changes in v2:
# - Endpoint path changed from /user to /users
# - Response format includes additional metadata
# - New required authentication headers
```

---

**API Reference Complete!** ✅

This comprehensive API reference provides complete documentation for integrating with OpenSim Next. With detailed REST API endpoints, WebSocket protocols, multi-language SDKs, authentication methods, error handling, webhook integration, and versioning strategies, developers have everything needed for professional integration with the OpenSim Next virtual world platform.

---

# Chapter 17: Quick Start Guide

This chapter gets you from zero to a running OpenSim Next grid with Galadriel AI in 15 minutes.

## 17.1 Prerequisites

You need:
- **macOS** (Intel or Apple Silicon), **Linux** (Ubuntu 22.04+), or **Windows 11**
- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Zig** (0.15.2): Download from https://ziglang.org/download/
- **PostgreSQL** (14+): `brew install postgresql` (macOS) or `apt install postgresql` (Ubuntu)
- **Ollama** (for Galadriel AI): https://ollama.ai — then `ollama pull llama3.1:8b`
- **Firestorm viewer**: https://www.firestormviewer.org/ (OpenSim edition)

## 17.2 Build OpenSim Next

```bash
# Clone and enter the project
git clone https://github.com/opensim/opensim-next.git
cd opensim-next

# Build Zig physics library
cd zig && zig build && cd ..

# Build the Rust server (from workspace root — NOT from rust/ subdirectory)
cargo build --release --bin opensim-next

# Verify the binary exists
ls -la target/release/opensim-next
```

## 17.3 Set Up the Database

```bash
# Create a PostgreSQL database
createdb gaiagrid

# Tables are created automatically on first startup — no manual SQL needed
```

## 17.4 Configure Your Instance

Your instance configuration lives in `Instances/Gaiagrid/`. The key files are:

```
Instances/Gaiagrid/
├── OpenSim.ini          # Main config (database URL, grid name, ports)
├── Regions/
│   └── Regions.ini      # Region definitions (name, location, size)
└── llm.ini              # Galadriel AI config (Ollama endpoint, model)
```

Ensure `OpenSim.ini` has your database connection string:
```ini
[DatabaseService]
ConnectionString = "postgresql://opensim@localhost/gaiagrid"
```

## 17.5 Start the Grid (2-Process Mode)

Open two terminal windows in the `opensim-next/` directory.

**Terminal 1 — Robust Services** (user accounts, inventory, assets, grid):
```bash
OPENSIM_INSTANCE_DIR=./Instances/Gaiagrid \
OPENSIM_SERVICE_MODE=robust \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next
```

Wait for "Robust services started on port 8503" in the log.

**Terminal 2 — Region Simulator**:
```bash
OPENSIM_INSTANCE_DIR=./Instances/Gaiagrid \
OPENSIM_SERVICE_MODE=grid \
OPENSIM_ROBUST_URL=http://localhost:8503 \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next
```

Wait for "Region 'Gaia One' ready" (or your region name).

## 17.6 Connect with Firestorm

1. Launch Firestorm (OpenSim edition)
2. Open **Grid Manager** (Preferences > Firestorm > General > "Use grid manager")
3. Add grid: `http://localhost:9000/`
4. Log in with your user account (first name, last name, password)
5. You should appear in your region

## 17.7 Talk to Galadriel

Galadriel AI is an in-world NPC. Once you're connected:

1. Open local chat (press Enter)
2. Say: **"Hey Galadriel, build me a small house"**
3. Galadriel will acknowledge and start building prims in front of you
4. Try more commands:
   - **"Create a volcanic island terrain"** — generates and previews terrain
   - **"Make a red sports car"** — builds a vehicle from mesh templates
   - **"Set up a sunset drone shot"** — creates a cinematic camera rig

Galadriel uses Ollama running locally. Make sure Ollama is running (`ollama serve`) and the model is downloaded (`ollama pull llama3.1:8b`).

## 17.8 Standalone Mode (Simpler, Single Process)

For quick testing without grid mode:

```bash
OPENSIM_INSTANCE_DIR=./Instances/Gaiagrid \
OPENSIM_SERVICE_MODE=standalone \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next
```

This runs all services in one process. Good for development, not recommended for production.

## 17.9 Next Steps

- **Chapter 2**: Deep configuration guide (all settings explained)
- **Chapter 3**: Database setup (PostgreSQL production tuning)
- **Chapter 8**: Region management (varregions, multi-region grids)
- **Chapter 22**: Full Galadriel AI reference (all build commands, terrain presets, cinematography)
- **Section 22.3**: AI Quick Start — get Galadriel running with Ollama in 5 minutes

---

# Chapter 22: AI/ML Features and Galadriel AI System

OpenSim Next includes a comprehensive AI platform with two major subsystems: (1) a background AI/ML analytics engine for content intelligence, and (2) **Galadriel**, an in-world AI director NPC who can build structures, generate terrain, create vehicles, design clothing, set up cinematic camera shots, write scripts, and converse naturally with users. All AI processing runs entirely on local hardware with no data sent to external services.

## 22.1 Feature Overview

### 22.1.1 AI/ML Analytics Engine

| Feature | Description | Requires LLM |
|---------|-------------|:---:|
| NPC Dialogue | Context-aware NPC conversations using local LLM | Yes |
| Asset Descriptions | Auto-generate descriptions for uploaded content | Yes |
| Semantic Search | Natural language search across assets and regions | No |
| Quality Prediction | Assess content quality on upload | No |
| Recommendations | Personalized content, social, and creator suggestions | No |
| Anomaly Detection | Flag unusual upload patterns and security risks | No |
| Engagement Metrics | User activity tracking with churn risk scoring | No |

### 22.1.2 Galadriel AI Director (In-World NPC)

| Capability | Description | Key Features |
|------------|-------------|-------------|
| **Building** | Create 3D structures from natural language | 7 prim types, color, texture, linking, furniture templates |
| **Mesh Creation** | Generate 3D meshes via Blender templates | Tables, chairs, arches, columns, stairs, stones, paths |
| **Terrain Generation** | Procedural terrain with 8 presets | Preview-then-approve, multi-region grids, .r32/.png/.jpg import |
| **Drone Cinematography** | Professional camera shots and lighting | 8 shot types, 7 lighting presets, automatic waypoints |
| **Clothing Design** | Bento mesh clothing rigged to standard bodies | Shirts, pants, jackets, dresses via Blender templates |
| **Vehicle Scripts** | Drivable cars, boats, aircraft | Physics-based vehicle controllers with HUD integration |
| **LSL Scripting** | In-world script insertion and templates | 14 script templates, custom script insertion |
| **Landscape Elements** | Trees, rocks, paths, decorative objects | Prim and mesh based natural scenery |
| **Guidance** | Help new users navigate the virtual world | Movement, building, inventory, appearance, teleportation |

## 22.2 System Requirements

**Lite Mode (No LLM):** +512 MB RAM. Provides recommendations, semantic search, quality gates, and anomaly detection. Galadriel uses canned/template responses.

**Full Mode (With LLM):** +6-10 GB RAM. Adds natural language NPC dialogue, AI building from descriptions, and asset description generation. Requires [Ollama](https://ollama.com) running locally.

GPU is optional but provides 3-10x faster LLM inference.

## 22.3 Quick Start

> For general server installation and setup, see [Chapter 17: Quick Start Guide](#chapter-17-quick-start-guide). This section covers AI-specific setup only.

```bash
# 1. Install Ollama (for full Galadriel AI features)
brew install ollama          # macOS
# curl -fsSL https://ollama.com/install.sh | sh   # Linux

# 2. Pull a model (mistral recommended for balanced speed/quality)
ollama pull mistral

# 3. Start Ollama
ollama serve

# 4. Enable AI in OpenSim Next
export OPENSIM_AI_ENABLED=true
export OPENSIM_LLM_ENDPOINT=http://localhost:11434
export OPENSIM_LLM_MODEL=mistral

# 5. Start the server
cargo run --release

# 6. Verify AI health
curl http://localhost:8080/api/ai/health
```

### 22.3.1 Talking to Galadriel In-World

Once logged in with a Second Life viewer (Firestorm recommended):

1. Open Local Chat (press Enter)
2. Type a message on channel -15400, or simply chat near Galadriel
3. Examples:
   - "build me a wooden table"
   - "create a volcanic island terrain"
   - "set up a cinematic orbit shot around the tower"
   - "make me a red sports car"

Galadriel responds in local chat and executes building actions automatically.

## 22.4 Configuration

### 22.4.1 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `OPENSIM_AI_ENABLED` | `false` | Master switch for all AI features |
| `OPENSIM_LLM_ENDPOINT` | `http://localhost:11434` | Ollama API endpoint |
| `OPENSIM_LLM_MODEL` | `mistral` | Model for text generation |
| `OPENSIM_EMBEDDING_MODEL` | `all-MiniLM-L6-v2` | Model for embeddings |
| `OPENSIM_EMBEDDING_CACHE_SIZE` | `10000` | Max cached embeddings |
| `OPENSIM_VECTOR_MAX_ENTRIES` | `100000` | Max vector store entries |

### 22.4.2 Galadriel Configuration (llm.ini)

Located in your instance directory (`OPENSIM_INSTANCE_DIR/llm.ini`):

```ini
[galadriel]
enabled=true
heartbeat_interval=120
heartbeat_greet=true
heartbeat_session_check=true
```

| Setting | Default | Description |
|---------|---------|-------------|
| `enabled` | `true` | Enable/disable Galadriel NPC |
| `heartbeat_interval` | `120` | Seconds between NPC heartbeat checks |
| `heartbeat_greet` | `true` | Greet new users when they arrive |
| `heartbeat_session_check` | `true` | Check active build sessions periodically |

### 22.4.3 Galadriel Identity

| Property | Value |
|----------|-------|
| UUID | `a01a0010-0010-0010-0010-000000000010` |
| Name | Galadriel |
| Chat Channel | -15400 |
| Role | AI Director (all domains) |

## 22.5 Galadriel AI: Building Domain

Galadriel can create 3D objects from natural language descriptions. She understands architectural concepts, furniture design, and decorative elements.

### 22.5.1 Primitive Types

| Command | Shape | Best For |
|---------|-------|----------|
| `rez_box` | Box/cube | Walls, floors, tables, shelves, doors |
| `rez_cylinder` | Cylinder | Columns, poles, pipes, tree trunks |
| `rez_sphere` | Sphere | Decorative balls, lights, domes |
| `rez_torus` | Torus (donut) | Arches, rings, picture frames |
| `rez_tube` | Hollow tube | Tunnels, pipes, hollow columns |
| `rez_ring` | Ring | Decorative rings, picture frames |
| `rez_prism` | Triangular prism | Roof peaks, ramps, wedges |

Each prim accepts: `position [x,y,z]`, `scale [x,y,z]`, and `name` (string).

### 22.5.2 Modification Actions

After creating objects, Galadriel can modify them:

| Action | Parameters | Example |
|--------|-----------|---------|
| `set_position` | local_id, pos [x,y,z] | Move an object |
| `set_rotation` | local_id, rot [x,y,z,w] | Rotate an object (quaternion) |
| `set_scale` | local_id, scale [x,y,z] | Resize an object |
| `set_color` | local_id, color [r,g,b,a] | Change object color (0.0-1.0) |
| `set_texture` | local_id, texture_uuid | Apply a texture |
| `set_name` | local_id, name | Rename an object |
| `link_objects` | root_id, child_ids[] | Link multiple prims into one object |
| `delete_object` | local_id | Remove an object |

### 22.5.3 Standard Colors

Galadriel understands these color names and their RGB values:

| Name | RGB | Use |
|------|-----|-----|
| Red | [0.8, 0.2, 0.1] | Accents, warnings |
| Green | [0.2, 0.6, 0.2] | Nature, foliage |
| Blue | [0.2, 0.3, 0.8] | Water, sky elements |
| White | [1.0, 1.0, 1.0] | Clean, modern |
| Black | [0.1, 0.1, 0.1] | Formal, contrast |
| Yellow | [0.9, 0.8, 0.2] | Highlights, warmth |
| Wood | [0.6, 0.4, 0.2] | Furniture, organic |
| Stone | [0.5, 0.5, 0.45] | Architecture, paths |

### 22.5.4 Example: Building a Table

Ask Galadriel: *"build me a wooden table"*

She creates a tabletop + 4 legs and links them into a single object:

```
1. Rez tabletop box at center, scale [1.5, 1.0, 0.08]
2. Rez 4 leg boxes, each scale [0.08, 0.08, 0.7]
3. Set all parts to wood color [0.6, 0.4, 0.2]
4. Link all 5 parts with tabletop as root
```

### 22.5.5 Object Delivery

Galadriel can give created objects to users:

| Action | Description |
|--------|-------------|
| `give_object` | Give object to a specific user by agent UUID |
| `give_to_requester` | Give object directly to the person who asked |
| `package_object_into_prim` | Place one object inside another as inventory |

### 22.5.6 Image-to-Build: Floor Plans & Elevations

Galadriel can analyze architectural images and build structures directly from floor plans, elevations, or blueprints. This feature requires a vision-capable LLM provider (Claude API or Ollama with LLaVA).

**Supported Image Types:**

| Action | Input | What It Builds |
|--------|-------|---------------|
| `import_floorplan` | Top-down floor plan image | Walls, doors, windows, floor slab |
| `import_elevation` | Front/side facade image | Windows per floor, roof geometry |
| `import_blueprint` | Floor plan + optional elevation | Complete structure with layout and facade |

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `image_path` | string | required | Path to PNG/JPG image |
| `pos` | [x,y,z] | [128,128,25] | Center position for the building |
| `wall_height` | float | 3.0 | Wall height in meters |
| `scale` | float | 1.0 | Scale multiplier for the entire structure |

**How It Works:**
1. You provide a floor plan image (hand-drawn sketch, CAD export, or photograph)
2. Galadriel's vision model analyzes the image and extracts walls, doors, windows, and rooms
3. The geometry generator creates prims for each structural element — walls split around openings, sill and transom segments for windows, door headers
4. All prims are linked into a single walkable building

**Example Prompts:**
- *"Build from this floor plan"* (with image in inventory or at a file path)
- *"Here's my house blueprint — build it at 128,128,25"*
- *"Use this elevation for the front of the building"*
- *"Make it bigger"* — use `scale: 1.5` to enlarge
- *"Make the walls 4 meters high"* — use `wall_height: 4.0`

**Tips:**
- Simple, clean floor plans with clear wall lines produce the best results
- Include a scale reference in the image if possible (e.g., "1cm = 1m")
- Standard room sizes are assumed if no scale is given (bedrooms ~3x4m, kitchens ~3x3m)
- After building, refine with natural language: *"make the kitchen bigger"*, *"add a window to the north wall"*

**Provider Requirements:**
- **Claude API** (`provider=anthropic` or `provider=auto`): Native vision support, best accuracy
- **Ollama with LLaVA** (`provider=ollama`, model must be `llava` or similar): Local vision, good for simple plans
- **Claude Code** (`provider=claude-code`): Uses Claude vision via auto-discovered credentials

### 22.5.7 LLM Provider Selection

Galadriel supports multiple LLM backends. Configure in `llm.ini` under the `[llm]` section:

| Provider | Setting | Cost | Vision | Context Window |
|----------|---------|------|--------|---------------|
| Ollama (local) | `provider=ollama` | Free | LLaVA model needed | 32K default |
| Anthropic API | `provider=anthropic` | Per-token | Native | 200K (Sonnet) / 1M (Opus) |
| Claude Code | `provider=claude-code` | Uses existing auth | Native | 200K+ |
| Auto-detect | `provider=auto` | Varies | If available | Best available |

**Auto-detection priority** (`provider=auto`):
1. `ANTHROPIC_API_KEY` environment variable → Anthropic API
2. `~/.claude/.credentials.json` → Claude Code credentials
3. Ollama on `localhost:11434` → Local Ollama

```ini
# Example: Use Claude automatically if Claude Code is running
[llm]
provider=auto
temperature=0.7
max_tokens=4096
```

```ini
# Example: Explicit Anthropic API
[llm]
provider=anthropic
model=claude-sonnet-4-20250514
api_key=sk-ant-your-key-here
max_tokens=4096
```

```ini
# Example: Local Ollama with vision
[llm]
provider=ollama
model=llava
context_window=32768
```

## 22.6 Galadriel AI: Mesh and Blender Templates

For complex shapes that go beyond basic primitives, Galadriel can generate mesh objects using Blender templates. These produce proper 3D mesh geometry with correct face counts and materials.

### 22.6.1 Available Mesh Templates

| Template | Parameters | Description |
|----------|-----------|-------------|
| `table` | WIDTH, HEIGHT, DEPTH | Solid table with legs |
| `chair` | SEAT_W, SEAT_H | Chair with back and legs |
| `shelf` | WIDTH, HEIGHT, DEPTH | Open shelf unit |
| `arch` | RADIUS, HEIGHT | Architectural arch |
| `staircase` | STEPS, WIDTH | Multi-step staircase |
| `stone` | SIZE, ROUGHNESS (0.1-0.3), SUBDIVISIONS (2-4) | Natural looking stone |
| `stone_ring` | RING_RADIUS, STONE_SIZE, STONE_COUNT, ROUGHNESS | Circle of stones (campfire ring) |
| `boulder` | SIZE, ROUGHNESS | Large decorative boulder |
| `column` | COL_RADIUS, COL_HEIGHT, FLUTING | Classical fluted column |
| `path` | PATH_LENGTH, PATH_WIDTH, PATH_DEPTH, PATH_CURVE, PATH_SEGMENTS, PATH_COBBLE | Walkway/garden path |

**Path Curve Types:** 0 = straight, 1 = S-curve, 2 = double-S
**Path Cobble:** 0 = smooth surface, 1 = cobblestone texture

### 22.6.2 Clothing Templates (Bento Mesh)

Galadriel can generate mesh clothing rigged to standard Bento skeleton bodies (compatible with Ruth2, Roth2, and Athena):

| Template | Parameters | Description |
|----------|-----------|-------------|
| `shirt` | SLEEVE_LENGTH, FIT, COLLAR | Upper body garment |
| `pants` | LEG_LENGTH, FIT, WAIST | Lower body garment |

**Shirt Parameters:**
- SLEEVE_LENGTH: 0 = tank top, 0.5 = short sleeve, 1.0 = long sleeve
- FIT: "tight", "normal", "loose"
- COLLAR: "crew", "v-neck"

**Pants Parameters:**
- LEG_LENGTH: 0 = shorts, 0.5 = capri, 1.0 = full length
- FIT: "tight", "normal", "loose"
- WAIST: "high", "mid", "low"

## 22.7 Galadriel AI: Terrain Generation

Galadriel can generate entire region terrain using noise-based procedural algorithms. This replaces the terrain for the whole region, so backups are automatically saved.

### 22.7.1 Terrain Presets

| Preset | Character | Algorithm |
|--------|-----------|-----------|
| `island` | Central landmass with beaches, underwater at borders | FBM + radial falloff mask |
| `mountains` | Jagged peaks and deep valleys, high elevation range | RidgedMulti + FBM blend |
| `rolling_hills` | Gentle undulating pastoral terrain | Low-octave FBM |
| `desert` | Sand dunes with flat basins | FBM + sinusoidal dune ridges |
| `tropical` | Elevated center with coastal lowlands | FBM + island mask + plateau |
| `canyon` | Deep carved channels through plateaus | FBM - abs(RidgedMulti) |
| `plateau` | Flat-topped mesas with steep edges | FBM with hard threshold clamp |
| `volcanic` | Central peak with crater and lava flow channels | Cone function + RidgedMulti + domain warping |

### 22.7.2 Terrain Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `preset` | string | required | One of the 8 presets above |
| `seed` | integer | random | Seed for reproducibility (same seed = same terrain) |
| `scale` | float | 1.0 | Height multiplier (0.1 = flat, 3.0 = extreme) |
| `roughness` | float | 0.5 | Detail/noise level (0.0 = smooth, 1.0 = very rough) |
| `water_level` | float | 20.0 | Sea level in meters (for island/coastal presets) |
| `region_id` | UUID string | current | Target a specific region by UUID |
| `grid_size` | integer | none | Multi-region grid dimension (e.g., 4 for a 4x4 grid) |
| `grid_x` | integer | none | This region's X position in grid (0 to grid_size-1) |
| `grid_y` | integer | none | This region's Y position in grid (0 to grid_size-1) |

### 22.7.3 Preview-Then-Approve Workflow (Recommended)

Terrain generation uses a **two-step approval process** by default:

**Step 1: Preview** - Ask Galadriel to create terrain. She generates a heightmap and places a 1/32 scale preview model (approximately 8m x 8m) near you. The region terrain is NOT changed yet.

Example: *"create a volcanic island terrain"*

Galadriel responds: *"PREVIEW: 'volcanic' terrain (seed=42, Central peak with crater and lava channels) - height range 5.2m to 87.3m. Preview model placed nearby. Say 'approve terrain' to apply it, or 'reject terrain' to discard. Preview ID: volcanic_42"*

**Step 2: Approve or Reject**

- Say *"approve terrain"* or *"looks good, apply it"* - Galadriel applies the terrain to the region, saves a .r32 backup, and removes the preview model.
- Say *"reject terrain"* or *"try again with more mountains"* - Galadriel discards the preview and offers to generate a new one with different parameters.

### 22.7.4 Direct Apply Mode

If you want to skip the preview step, tell Galadriel explicitly:
- *"just apply a mountain terrain directly"*
- *"generate desert terrain, skip the preview"*

This uses `terrain_generate` which applies immediately.

### 22.7.5 Loading Saved Terrain

**From .r32 file (raw 32-bit float heightmap):**
- *"load terrain from Terrains/island_42.r32"*

**From PNG/JPG image (grayscale = height, white = peaks, black = valleys):**
- *"import terrain from Terrains/my_terrain.png with height range 5 to 80 meters"*

All generated terrains are automatically backed up to `Terrains/` in .r32 format.

### 22.7.6 Multi-Region Grid Terrain

For large builds spanning multiple adjacent regions, Galadriel can generate seamless edge-matched terrain across a grid of regions. All regions must use the same seed and preset:

Example: Creating a 4x4 (16 region) mountain range:
1. *"create mountains terrain for a 4x4 grid, tile 0,0, seed 9999"*
2. *"create mountains terrain for 4x4 grid, tile 1,0, seed 9999"*
3. Repeat for all 16 tiles...

The generator samples from a single continuous noise field, offsetting coordinates per tile, so adjacent region edges match perfectly.

### 22.7.7 Terrain Height Range

All presets auto-scale heights to the OpenSim standard range (approximately 0-100 meters). The `scale` parameter multiplies the height range:
- `scale: 0.5` - half height (flatter, gentler terrain)
- `scale: 1.0` - standard height range
- `scale: 2.0` - double height (more dramatic elevation)

## 22.8 Galadriel AI: Drone Cinematography

Galadriel can set up complete cinematic camera rigs with professional-grade shot types, automated camera paths, and 3-point lighting systems. The system creates a physical drone camera object in the scene that users sit on to start filming.

### 22.8.1 Shot Types

| Shot Type | Use Case | Camera Behavior |
|-----------|----------|-----------------|
| `orbit` | Product showcase, architecture | 360-degree circle around subject (24 waypoints) |
| `dolly` | Establishing shots | Smooth straight track past subject |
| `crane` | Dramatic reveal | Vertical sweep from low to high |
| `flyby` | Action sequences | Fast diagonal pass |
| `reveal` | Building reveal | Starts close, pulls back to wide shot |
| `tracking` | Movement shots | Follows alongside subject path |
| `dutch` | Tension, unease | Tilted dramatic angle |
| `push_in` | Focus, intensity | Slow approach toward subject |

### 22.8.2 Lighting Presets

| Preset | Lights | Character |
|--------|--------|-----------|
| `rembrandt` | 3 | Classic portrait: strong key, soft fill, rim accent |
| `butterfly` | 2 | Beauty/portrait: overhead key, subtle fill |
| `split` | 2 | Half-lit dramatic: side key with edge light |
| `rim` | 2 | Silhouette edge: dual rim lights from behind |
| `studio` | 3 | Professional balanced: key + fill + rim |
| `golden_hour` | 3 | Warm sunset: sun key + sky fill + warm backlight |
| `noir` | 1 | Hard dramatic: single side light |

### 22.8.3 How to Use

1. Tell Galadriel what you want to film:
   - *"set up a cinematic orbit shot around the building at 128,128,30"*
   - *"film a dramatic crane reveal of the tower"*
   - *"create a noir-lit push-in shot focused on the statue"*

2. Galadriel creates:
   - A camera drone object with scripted flight path
   - Light objects positioned around the subject
   - Automatic waypoints based on the shot type

3. Sit on the drone camera to begin the cinematic sequence
4. Touch the drone to pause/resume

### 22.8.4 Camera Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `scene_name` | string | Descriptive name for the scene |
| `shot_type` | string | One of the 8 shot types above |
| `subject_position` | [x,y,z] | What the camera should focus on |
| `speed` | float | 0.5 = slow/cinematic, 1.0 = normal, 2.0 = fast |
| `lights` | string or array | Preset name ("rembrandt") or custom light array |
| `camera_waypoints` | array | Optional custom waypoints (auto-generated if omitted) |

### 22.8.5 Media Production Pipeline

Beyond in-world drone cameras, OpenSim Next includes a server-side media production pipeline that renders high-quality content to files. This pipeline uses industry-standard tools orchestrated by Galadriel:

**Production Workflow:**
```
Scene Data (database) → OBJ Export → Blender Headless Render → ffmpeg Encode → Post-Processing → Output
```

**Output Directories:**
| Directory | Content |
|-----------|---------|
| `Mediastorage/video/` | Rendered video files (MP4) |
| `Mediastorage/images/` | Still photographs and composites |
| `Mediastorage/audio/` | Synthesized ambient audio (WAV) |
| `Mediastorage/print/` | Print-ready compositions |
| `Mediastorage/projects/` | JSON recipes for re-rendering |

**Media Actions via Galadriel:**

| Action | Description | Example Prompt |
|--------|-------------|----------------|
| `compose_film` | Multi-shot video production | *"film a sunset flyover of the harbor"* |
| `compose_photo` | Professional scene photography | *"take a portrait photo of the fountain with golden hour lighting"* |
| `compose_ad` | Advertisement/billboard composition | *"create an ad for the new beach resort"* |
| `compose_music` | Ambient audio generation | *"generate ocean sounds for the beach"* |
| `drone_cinematography` | In-world drone camera setup | *"orbit the lighthouse at sunset"* |

**Photo Composition Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `scene_name` | string | Name for the output file |
| `description` | string | Scene description for AI framing |
| `camera_angle` | string | "eye_level", "bird_eye", "low_angle", "dutch" |
| `framing` | string | "rule_of_thirds", "centered", "golden_ratio" |
| `depth_of_field` | string | "shallow" (blurred bg), "deep" (all sharp), "medium" |
| `lighting_preset` | string | "golden_hour", "blue_hour", "studio", "dramatic" |
| `time_of_day` | string | "sunrise", "noon", "sunset", "night" |

**Post-Processing Filters:**

| Filter | Effect |
|--------|--------|
| `golden_hour` | Warm golden color grading |
| `noir` | Black-and-white with high contrast |
| `vignette` | Darkened edges drawing focus to center |
| `letterbox` | Cinematic black bars (2.39:1 ratio) |
| `cool_moonlight` | Blue-shifted nighttime mood |
| `film_grain` | Subtle analog film texture |

**Audio Presets:**

Five pure-Rust synthesized ambient soundscapes: `ocean` (waves and surf), `wind` (gusts and whistles), `rain` (rainfall patterns), `forest` (birds and rustling), `urban` (city ambience).

**Requirements:** Blender 4.x (headless rendering), ffmpeg 7+ (video encoding), GIMP 2.10+ (advanced filters). The server automatically detects available tools and degrades gracefully if any are missing.

### 22.8.6 Luxor Camera System

The Luxor Camera System is a pure Rust server-side raytracer that renders images and video directly from live scene state in memory. Unlike the Media Production Pipeline (which uses Blender), Luxor renders in seconds without any external tools, making it ideal for previews, real-time photography, and rapid iteration.

**Key Advantages:**
- Renders a 1080p still in under 3 seconds (Draft quality)
- No external dependencies (pure Rust + rayon parallelism)
- Reads live scene geometry directly from server memory
- Works with ANY viewer via LSL HUD on channel -15500
- Also controllable via Galadriel AI or direct API

#### Camera Presets

| Preset | Focal Length | f-Stop | Best For |
|--------|-------------|--------|----------|
| `wide` | 24mm | f/8 | Landscapes, architecture, interiors |
| `normal` | 50mm | f/5.6 | Natural perspective, everyday shots |
| `portrait` | 85mm | f/1.8 | Shallow depth-of-field, bokeh backgrounds |
| `telephoto` | 200mm | f/4 | Distant subjects, compression effect |
| `macro` | 100mm | f/2.8 | Close-up details, extreme depth-of-field |
| `cinematic` | 35mm | f/2 | Film-like perspective with shallow DoF |
| `drone` | 14mm | f/5.6 | Ultra-wide aerial photography |
| `security` | 28mm | f/11 | Deep focus surveillance-style |

#### Screen Sizes

| Size | Resolution | Aspect | Use |
|------|-----------|--------|-----|
| `SD` | 640x480 | 4:3 | Quick preview/thumbnail |
| `HD` | 1280x720 | 16:9 | Web video |
| `FullHD` | 1920x1080 | 16:9 | Standard video/stills |
| `QHD` | 2560x1440 | 16:9 | High quality |
| `UHD4K` | 3840x2160 | 16:9 | 4K video and stills |
| `Cinema` | 2560x1080 | 2.39:1 | Ultrawide cinematic |
| `Square` | 1080x1080 | 1:1 | Social media |
| `Portrait` | 1080x1920 | 9:16 | Vertical/mobile |
| `Poster` | 2480x3508 | A4 | Print poster |
| `Banner` | 3840x1080 | 32:9 | Banner/header image |

#### Lighting Presets

Luxor includes 10 professional studio lighting configurations:

| Preset | Description |
|--------|-------------|
| `studio_3point` | Classic key + fill + rim setup |
| `rembrandt` | Dramatic portrait with triangle shadow |
| `butterfly` | Beauty lighting from above |
| `noir` | Hard single-source film noir |
| `golden_hour` | Warm low-angle sunset simulation |
| `moonlight` | Cool blue nighttime ambience |
| `split` | Half-lit dramatic side lighting |
| `flat` | Even illumination for documentation |
| `backlit` | Silhouette/rim-light emphasis |
| `neon` | Colorful cyberpunk-style gels |

#### Quality Levels

| Quality | Samples/Pixel | 1080p Time | 4K Time | Use |
|---------|:---:|:---:|:---:|-----|
| `draft` | 1 | <1s | <3s | Quick preview |
| `standard` | 4 | <3s | <10s | General photography |
| `high` | 16 | <10s | <30s | Publication quality |
| `ultra` | 64 | <30s | <2min | Maximum quality |

#### Post-Processing Effects

13 pure Rust effects applied after rendering:

| Effect | Description |
|--------|-------------|
| `vignette` | Darkened edges with radial falloff |
| `bloom` | Bright areas glow softly |
| `letterbox` | Cinematic black bars for widescreen ratio |
| `film_grain` | Subtle random noise overlay |
| `color_grade_warm` | Shift toward golden/orange tones |
| `color_grade_cool` | Shift toward blue/teal tones |
| `color_grade_noir` | Desaturate with high contrast |
| `tone_map_aces` | ACES filmic tone mapping |
| `tone_map_reinhard` | Reinhard tone mapping |
| `sharpen` | Unsharp mask sharpening |
| `chromatic_aberration` | RGB channel offset at edges |
| `depth_fog` | Distance-based atmospheric fog |
| `tilt_shift` | Miniature/toy effect |

#### Using Luxor via LSL HUD

Attach the Luxor HUD (available as `Test_Scripts/luxor_hud.lsl`) to your avatar. The HUD communicates with the server on channel -15500 using JSON commands.

**HUD Modes (cycle by touching):**
- **SNAP**: Take snapshots with preset cycling (left/right to change preset)
- **CAM**: Adjust camera settings (focal length, f-stop, focus distance)
- **LIGHT**: Cycle through lighting presets
- **REC**: Video recording (start/stop, change path type)
- **FX**: Toggle post-processing effects

**JSON Commands (for advanced users or custom HUDs):**

```json
// Take a snapshot
{"cmd": "snapshot", "preset": "portrait", "size": "4K", "effects": ["vignette", "warm"]}

// Quick preview
{"cmd": "preview", "size": "SD"}

// Adjust camera
{"cmd": "set_camera", "focal": 85, "fstop": 1.8, "focus": 5.0}

// Position a light
{"cmd": "set_light", "slot": 0, "type": "key", "pos": [130,128,27], "color": [1,0.9,0.8], "intensity": 800}

// Apply lighting preset
{"cmd": "set_lighting", "preset": "golden_hour"}

// Enable effects
{"cmd": "set_effect", "effects": ["noir", "film_grain", "letterbox"]}

// Start video recording
{"cmd": "record_start", "fps": 30, "size": "FullHD", "path_type": "orbit", "duration": 10}

// Stop recording
{"cmd": "record_stop"}

// Add camera waypoint for video
{"cmd": "add_waypoint", "pos": [128,128,30], "lookat": [128,128,25], "focal": 50}

// Check status
{"cmd": "status"}

// Reset all settings
{"cmd": "reset"}
```

#### Using Luxor via Galadriel AI

Tell Galadriel what you want to photograph or film:

- *"take a portrait photo of the garden with golden hour lighting"*
- *"shoot a cinematic video orbiting the castle at sunset, 10 seconds"*
- *"snap a noir-style wide shot of the city skyline"*
- *"film a crane shot revealing the harbor, use neon lighting"*

Galadriel translates your request into Luxor actions and saves output to `Mediastorage/images/` or `Mediastorage/video/`.

**Luxor Snapshot Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `preset` | string | Camera preset name (see table above) |
| `size` | string | Screen size name (see table above) |
| `quality` | string | "draft", "standard", "high", "ultra" |
| `effects` | array | List of post-processing effects |
| `lighting` | string | Lighting preset name |
| `subject_position` | [x,y,z] | Point the camera focuses on |
| `name` | string | Output filename |

**Luxor Video Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `shot_type` | string | "orbit", "dolly", "crane", "flythrough", "static" |
| `duration` | float | Video length in seconds |
| `fps` | int | Frames per second (24, 30, 60) |
| `size` | string | Screen size name |
| `quality` | string | Quality level |
| `effects` | array | Post-processing effects per frame |
| `lighting` | string | Lighting preset |
| `subject_position` | [x,y,z] | Camera target point |
| `name` | string | Output filename |

#### Video Path Types

| Path Type | Description |
|-----------|-------------|
| `orbit` | 360-degree circle around subject |
| `dolly` | Smooth linear track |
| `crane` | Vertical sweep (low to high or high to low) |
| `flythrough` | Forward-moving path through scene |
| `static` | Fixed camera position (for time-lapse style) |

Camera motion uses Catmull-Rom spline interpolation for smooth, professional-quality movement between keyframes.

#### Luxor vs Media Production Pipeline

| Feature | Luxor | Media Pipeline |
|---------|-------|----------------|
| **Speed** | Seconds | Minutes |
| **Dependencies** | None (pure Rust) | Blender, ffmpeg, GIMP |
| **Rendering** | Server-side raytracer | Blender headless |
| **Scene Source** | Live memory | Database export |
| **Textures** | Color-only (geometry + light) | Full UV textures via Blender |
| **Best For** | Previews, rapid iteration, HUD control | Final production, textured renders |
| **Output** | PNG stills, MP4 video | PNG/JPG stills, MP4 video, WAV audio |

Use Luxor for fast, interactive photography. Use the Media Pipeline when you need full-texture, production-quality renders.

## 22.9 Galadriel AI: LSL Script Templates

Galadriel can insert ready-made LSL scripts into objects, or write custom scripts based on user descriptions.

### 22.9.1 Pre-Built Script Templates

| Template | Description | Key Parameters |
|----------|-------------|----------------|
| `rotating` | Object rotates continuously when touched | AXIS [x,y,z], SPEED (float) |
| `sliding_door` | Door slides open on touch, auto-closes | SLIDE_DISTANCE, AUTO_CLOSE (seconds) |
| `toggle_light` | Point light turns on/off | COLOR [r,g,b], INTENSITY, RADIUS |
| `floating_text` | Displays floating text above object | TEXT (string), COLOR [r,g,b] |
| `sit_target` | Configures avatar sit position | SIT_OFFSET [x,y,z] |
| `touch_say` | Speaks a message when touched | MESSAGE (string), CHANNEL (int) |
| `timer_color` | Changes to random color on a timer | INTERVAL (seconds) |
| `touch_hide` | Toggles object visibility on touch | (no parameters) |
| `vendor_give` | Gives inventory contents to buyer | EMPTY_MESSAGE (string) |

### 22.9.2 Vehicle Script Templates

| Template | Vehicle Type | Key Parameters |
|----------|-------------|----------------|
| `car_controller` | Drivable land vehicle | MAX_SPEED, FORWARD_POWER, REVERSE_POWER, BRAKE_POWER, TURN_RATE |
| `plane_controller` | Flyable aircraft | MAX_THRUST, STALL_SPEED, MAX_SPEED, ROLL_RATE, PITCH_RATE, LIFT_FACTOR |
| `vessel_controller` | Sailboat with wind physics | FORWARD_POWER, REVERSE_POWER, TURN_RATE, WIND_BASE_SPEED |

**Vehicle HUD Channels:**
- Land vehicles: -14710
- Aircraft: -14720
- Marine vessels: -14700

### 22.9.3 Cinematic Script Templates

| Template | Description | Key Parameters |
|----------|-------------|----------------|
| `drone_camera` | Autonomous cinema drone | CINEMA_CH (channel), SPEED |
| `cinema_light` | Controllable scene light | COLOR, INTENSITY, RADIUS, FALLOFF, LIGHT_NAME |
| `luxor_hud` | Luxor Camera System HUD | Channel -15500, 5 modes (SNAP/CAM/LIGHT/REC/FX) |

### 22.9.4 Custom Script Insertion

Galadriel can also insert custom LSL scripts written from scratch:

- *"add a script to this door that opens when I say 'open sesame' on channel 5"*
- *"put a rotation script on this sign that spins slowly"*

She uses `insert_script` with custom source code or `insert_template_script` with a template name and parameters.

## 22.10 Galadriel AI: Skill Domains

Galadriel's knowledge is organized into specialized skill domains, each based on the expertise of a specialist NPC personality:

| Domain | Specialist | Capabilities |
|--------|-----------|-------------|
| **Building** | Aria | Prim creation, modification, linking, mesh generation, Blender templates |
| **Clothing** | Zara | Wearable design, Bento mesh clothing, fashion advice, color theory |
| **Scripting** | Reed | LSL/OSSL scripting, event handling, script debugging, template insertion |
| **Landscaping** | Terra | Terrain generation, heightmaps, natural scenery, landscape elements |
| **Guidance** | Nova | Movement help, building basics, inventory management, appearance customization |
| **Media** | (Director) | Drone cinematography, Luxor camera system, film/photo/ad composition, ambient audio, server-side rendering |

When you ask Galadriel a question, she automatically activates the relevant skill domain based on your request.

## 22.11 Conversation and Memory

### 22.11.1 Talking to Galadriel

Galadriel listens on local chat near her position. She understands natural language and can:
- Build objects from descriptions ("make me a house with a red roof")
- Answer questions about the virtual world
- Provide building tutorials and guidance
- Remember context within a conversation session

### 22.11.2 Build Sessions

When Galadriel builds something for you, she tracks a "build session" that records:
- All objects created (with local IDs)
- Project name and description
- Modifications and deletions

This allows her to reference previously built objects in follow-up requests: *"make the table legs thicker"* or *"delete the second chair"*.

### 22.11.3 NPC Memory

Galadriel has a persistent memory system that stores facts about users and interactions:
- User preferences (favorite colors, building styles)
- Previous project context
- Category-tagged facts for contextual retrieval

Memory persists across conversations and server restarts.

### 22.11.4 Muting Galadriel

If you want Galadriel to be quiet:
- `/mode quiet` - Silences Galadriel (she stops responding to chat)
- `/mode listen` - Reactivates Galadriel

## 22.12 AI/ML Analytics API Endpoints

All analytics AI endpoints are prefixed with `/api/ai/`:

| Category | Endpoint | Method | Description |
|----------|----------|--------|-------------|
| Health | `/health` | GET | System status and capabilities |
| Dialogue | `/dialogue` | POST | NPC conversation API |
| Assets | `/asset/describe` | POST | Auto-generate asset description |
| Search | `/search` | POST | Semantic natural language search |
| Search | `/search/similar/:id` | GET | Find similar content |
| Embed | `/embed/assets` | POST | Batch embed assets into vector store |
| Embed | `/embed/regions` | POST | Batch embed region descriptions |
| Quality | `/quality/assess` | POST | Quality assessment for uploaded content |
| Quality | `/quality/alerts` | GET | Anomaly detection alerts |
| Recommend | `/recommend/:user_id` | GET | Personalized content recommendations |
| Recommend | `/recommend/:user_id/social` | GET | Friend suggestions |
| Recommend | `/recommend/:user_id/creators` | GET | Creator suggestions |
| Recommend | `/recommend/:user_id/engagement` | GET | Engagement metrics |
| Recommend | `/recommend/trending` | GET | Trending content across grid |
| Recommend | `/recommend/activity` | POST | Record user activity event |
| Recommend | `/recommend/profile` | POST | Update user recommendation profile |

## 22.13 Operational Modes

| Mode | LLM Required | RAM Overhead | Features |
|------|:---:|---:|------|
| **Full Mode** | Yes | +6-10 GB | All features: Galadriel AI, building, terrain, cinematography, Luxor camera, media production, analytics |
| **Lite Mode** | No | +512 MB | Recommendations, quality gates, search, anomaly detection, template responses |
| **Disabled** | No | 0 | `OPENSIM_AI_ENABLED=false`, zero overhead |

The server degrades gracefully: if Ollama is unavailable, Galadriel uses fallback template responses for common requests while all non-LLM features continue operating normally.

## 22.14 Privacy and Security

- All AI processing runs locally on your hardware
- No external API calls are made by any AI feature
- User activity data stays in your local database
- Conversation history is in-memory only (cleared on server restart)
- NPC memory facts are stored locally and never transmitted
- Embeddings cannot be reverse-engineered to recover original content
- No user data, chat logs, or content leaves your server

## 22.15 Skill Engine (Phase 209)

The Skill Engine is a unified framework that makes every AI Director capability discoverable, invocable, and trackable. It provides three access paths to 120+ skills across 14 domains.

### 22.15.1 Access Paths

| Path | Use Case | Example |
|------|----------|---------|
| **Chat** | Natural language via AI Director | "build me a table" |
| **REST API** | Flutter admin, external tools | `GET /skills/dashboard` |
| **LSL** | In-world scripts | `osInvokeSkill("navigation", "teleport_agent", params)` |

### 22.15.2 Skill Domains (14)

| Domain | Skills | Maturity | Key Capabilities |
|--------|:------:|:--------:|-----------------|
| Building | 26 | L7 | Rez prims, mesh, Blender, linksets, import/export |
| Scripting | 4 | L7 | Insert/update LSL, templates, give objects |
| Landscaping | 6 | L7 | Procedural terrain, heightmaps, preview/apply |
| Media | 14 | L7/L0 | Film, photo, drone cinematography, Luxor raytrace, tutorial video pipeline |
| Vehicles | 2 | L7/L5 | 7 vehicle recipes (car, bike, plane, vtol, vessel, starship, lani) |
| Clothing | 6 | L7/L1 | T-shirts (complete), pants/dress/jacket/skirt (defined) |
| Navigation | 7 | L3/L0 | Teleport, landmarks, waypoint tours, home location |
| Estate | 11 | L3 | Region restart, parcel access/media/music, ban/unban, sun, water, flags |
| Economy | 6 | L3 | Pay agent, check balance, vendors, tip jars, transactions |
| Social | 6 | L3 | Greet, announce, events, group invites, notices, greeter NPCs |
| Animation | 6 | L3/L6 | Play/stop animation, poses, statue baking, AO |
| Inventory | 6 | L3 | Give items/folders, starter kits, search, bulk distribute |
| NPC Management | 8 | L3 | Spawn/despawn, appearance, walk, say, animate, patrol, roles |
| Tutorial | 5 | L3 | Interactive tutorials, hints, info boards, welcome areas, progress tracking |

### 22.15.3 Maturity Model

Each skill follows a 7-level lifecycle:

| Level | Name | Meaning |
|-------|------|---------|
| L0 | Seed | Name and purpose defined |
| L1 | Defined | Parameters and examples documented |
| L2 | Stubbed | Code exists, returns "not yet implemented" |
| L3 | Functional | Core operation works end-to-end |
| L4 | Robust | Error handling, validation, permissions |
| L5 | Integrated | REST + LSL + Chat all work |
| L6 | Verified | In-world tested by human |
| L7 | Production | Battle-tested, documented, patterns extracted |

### 22.15.4 REST API Endpoints

All skill endpoints are on the admin API port (default 9200):

```
GET  /skills                        — List all skills grouped by domain
GET  /skills/dashboard              — Maturity dashboard with scores
GET  /skills/search?q=terrain       — Full-text search
GET  /skills/{domain}               — List skills in a domain
GET  /skills/{domain}/{skill_id}    — Skill detail with params and examples
```

Example:
```bash
curl http://localhost:9200/skills/dashboard
```

Returns JSON with per-domain maturity scores, skill counts, and level distribution.

### 22.15.5 LSL Integration

Any in-world script can invoke skills directly:

```lsl
// Teleport an agent
osInvokeSkill("navigation", "teleport_agent",
    "{\"agent_id\": \"uuid-here\", \"position\": [45, 128, 22]}");

// Check economy balance
osInvokeSkill("economy", "check_balance",
    "{\"agent_id\": \"uuid-here\"}");

// List agents in region
osInvokeSkill("estate", "list_agents", "{}");
```

### 22.15.6 Flutter Skills Dashboard

The OpenSim Configurator app (v2.4.2+) includes a Skills Dashboard accessible from the sidebar. Features:

- Domain cards with maturity scores and progress bars
- Drill-down to individual skills with parameter documentation
- Full-text search across all 120+ skills
- Color-coded maturity levels (L0 grey through L7 green)
- Skill detail view with params, tags, examples, and requirements

### 22.15.7 Configurable AI Director Name

Grid operators can customize the AI Director's display name:

```ini
# In llm.ini under [galadriel] section:
[galadriel]
enabled=true
name=Athena
```

The configured name appears in:
- In-world greetings and chat responses
- LLM system prompt (the AI knows its own name)
- Help text and fallback responses

Default name is "Galadriel" if not configured.

## 22.16 Detailed API Documentation

For comprehensive operational documentation including detailed API examples with curl commands, performance tuning parameters, troubleshooting guide, resource planning, and model selection guide, see: **[AI_OPERATIONS.md](AI_OPERATIONS.md)**

---

# Chapter 23: Administrator's Addendum — Instance Admin Controller

This chapter covers the Instance Admin Controller, the master management system for OpenSim Next deployments. The controller provides centralized oversight, process lifecycle management, real-time monitoring, and role-based access control for all virtual world instances under your administration.

## 23.1 Architecture Overview

The Instance Admin Controller is a **management plane** — a lightweight orchestration layer that sits above your virtual world instances and provides complete operational control. It does not serve regions, handle logins, or process UDP traffic. Its sole purpose is to manage the instances that do.

### 23.1.1 System Architecture

```
┌─────────────────────────────────────────────┐
│         Flutter Configurator (macOS)         │
│         Instance Admin Dashboard             │
│         Real-time status, console, control   │
└──────────────────┬──────────────────────────┘
                   │ WebSocket ws://localhost:9300/ws
                   ▼
┌─────────────────────────────────────────────┐
│           Instance Admin Controller           │
│           (Management Plane)                  │
│                                               │
│  ┌─────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ Process  │ │ Instance │ │   Access     │  │
│  │ Manager  │ │ Registry │ │   Control    │  │
│  └─────────┘ └──────────┘ └──────────────┘  │
│  ┌─────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ Health   │ │ Console  │ │  WebSocket   │  │
│  │ Monitor  │ │ Streamer │ │  Server      │  │
│  └─────────┘ └──────────┘ └──────────────┘  │
└──────┬──────────┬──────────┬────────────────┘
       │ spawn    │ spawn    │ spawn
       ▼          ▼          ▼
┌──────────┐ ┌──────────┐ ┌──────────┐
│ Gaiagrid │ │ TestGrid │ │ MyWorld  │
│ grid +   │ │ standalone│ │ standalone│
│ robust   │ │ port 9000│ │ port 9010│
│ port 9500│ │          │ │          │
└──────────┘ └──────────┘ └──────────┘
      ▲            ▲            ▲
      └────────────┴────────────┘
        Announce back on boot
        Periodic heartbeats
```

### 23.1.2 Instance Lifecycle

Every instance moves through a defined set of states:

| State | Description | Actions Available |
|-------|-------------|-------------------|
| **Discovered** | Directory found in `Instances/`, not yet started | Start |
| **Starting** | Process spawned, waiting for announcement | — |
| **Running** | Instance announced, serving clients | Stop, Restart |
| **Stopping** | Graceful shutdown in progress | — |
| **Stopped** | Process exited cleanly | Start |
| **Error** | Process exited with error or failed to start | Start, Restart |

The lifecycle flows: **Discovered → Starting → Running → Stopping → Stopped**

When the controller starts, it scans the `Instances/` directory and registers every valid instance directory as **Discovered**. The operator decides what to start and when.

### 23.1.3 The Sense Function

Instances are not passive children — they actively participate in the management system through the **sense function**:

1. **Announcement**: When an instance finishes booting, it sends a `POST /api/instance/announce` to the controller with its identity, service mode, ports, and capabilities
2. **Heartbeat**: Every 30 seconds, each running instance sends `POST /api/instance/heartbeat` with current metrics (CPU, memory, active users, regions)
3. **Graceful degradation**: If the controller is unreachable, instances continue operating normally — they simply queue their announcements

This design means the controller never needs to poll instances. The instances tell the controller what they're doing.

## 23.2 Deployment Modes

OpenSim Next supports three deployment modes for the controller, each suited to different scales and requirements.

### 23.2.1 Dedicated Controller Mode

**Best for**: Production grids, multi-server deployments, enterprise operations

The controller runs as its own process with zero region/login/UDP overhead. Maximum resources available for management operations.

```bash
# Terminal 1: Start the controller
OPENSIM_SERVICE_MODE=controller \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next

# Terminal 2: Start an instance (controller spawns it, or manually)
curl -X POST http://localhost:9300/api/instance/gaiagrid/start
```

**When to use**:
- Running 3+ instances
- Production environments where management overhead must be isolated
- When you need the controller on a different machine from the instances
- Enterprise deployments with strict resource separation

### 23.2.2 Embedded Controller Mode

**Best for**: Small grids, personal worlds, development, single-server deployments

The controller runs inside an existing instance as a background task on port 9300. One process does everything — serves regions AND manages other instances. No separate process needed.

```bash
# Single process: region server + embedded controller
OPENSIM_EMBEDDED_CONTROLLER=true \
OPENSIM_SERVICE_MODE=standalone \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next
```

The instance serves its own regions on its normal port AND provides the management API on port 9300. It automatically registers itself as **Running** in the controller registry.

**Optional**: Override the controller port:
```bash
OPENSIM_CONTROLLER_PORT=9300  # default
```

**When to use**:
- Running 1-2 instances
- Personal or hobbyist grids
- Development and testing
- Demos and evaluations
- Any deployment where simplicity matters more than resource isolation

### 23.2.3 Standalone Mode (No Controller)

**Best for**: Instances managed by external tools, legacy configurations

Instances run independently with zero management overhead. No controller, no announcements, no heartbeats.

```bash
# Just run the server — no controller involvement
OPENSIM_SERVICE_MODE=standalone \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next
```

**When to use**:
- Legacy deployments being migrated gradually
- Instances managed by systemd, Docker, or Kubernetes natively
- When you don't need centralized management

### 23.2.4 Choosing the Right Mode

| Factor | Dedicated | Embedded | Standalone |
|--------|-----------|----------|------------|
| Instances | 3+ | 1-2 | Any |
| Processes | N+1 | 1 | N |
| Management UI | Yes | Yes | No |
| Resource overhead | Separate | Shared | None |
| Production ready | Yes | Small scale | Yes (no mgmt) |
| Complexity | Medium | Low | Lowest |

## 23.3 Access Control Hierarchy

The controller enforces a six-tier role-based access control system. Every API endpoint requires a minimum access level. Higher levels inherit all permissions of lower levels.

### 23.3.1 Role Definitions

| Level | Role | Capabilities |
|-------|------|-------------|
| **500** | **CentralAdmin** | Full system control. Assigns access levels to all users. Manages controller configuration, all instances, all users. The only role that can promote or demote other administrators. |
| **400** | **Operator** | Start, stop, and restart instances. View all consoles. Manage instance configurations. Cannot change access levels or controller settings. |
| **300** | **GridAdmin** | Manage grid settings, user accounts, and regions within assigned instances. Cannot start or stop instances. |
| **200** | **RegionOwner** | Manage their own regions: backup, OAR load/save, restart individual regions. View their own region metrics. |
| **100** | **User** | View the instance status dashboard. See public metrics. No control actions. |
| **0** | **Newbie** | Read-only access. Can see that instances exist and their status. No management capabilities. |

### 23.3.2 Authentication Methods

The controller supports two authentication methods:

**API Key Authentication** (default for development):
```bash
# Set the master API key
export OPENSIM_API_KEY="your-secure-api-key"

# Use it in requests — API key holders get CentralAdmin (500)
curl -H "X-API-Key: your-secure-api-key" http://localhost:9300/api/instances
```

**JWT Token Authentication** (recommended for production):
```bash
# Set the JWT secret
export OPENSIM_JWT_SECRET="your-jwt-secret-key"

# Use Bearer token — access level comes from the token's user_level claim
curl -H "Authorization: Bearer eyJhbG..." http://localhost:9300/api/instances
```

JWT tokens carry the user's identity and access level in their claims:
```json
{
  "sub": "user-uuid-here",
  "username": "GridOperator",
  "user_level": 400,
  "exp": 1740000000
}
```

### 23.3.3 Endpoint Access Matrix

| Endpoint | Method | Minimum Level | Description |
|----------|--------|---------------|-------------|
| `/health` | GET | None | Controller health check |
| `/api/instances` | GET | User (100) | List all instances with status |
| `/api/instance-dirs` | GET | User (100) | Scan available instance directories |
| `/api/instance/:id/start` | POST | Operator (400) | Start a discovered instance |
| `/api/instance/:id/stop` | POST | Operator (400) | Gracefully stop an instance |
| `/api/instance/:id/restart` | POST | Operator (400) | Restart an instance |
| `/api/instance/:id/console` | GET | Operator (400) | View instance console output |
| `/api/instance/announce` | POST | None (internal) | Instance self-announcement |
| `/api/instance/heartbeat` | POST | None (internal) | Instance heartbeat |
| `/api/admin/set-level` | POST | CentralAdmin (500) | Set a user's access level |
| `/api/admin/users` | GET | Operator (400) | List users and their levels |
| `/ws` | GET | User (100) | WebSocket for real-time updates |

### 23.3.4 Promoting Users

Only CentralAdmin (level 500) can change access levels:

```bash
# Promote a user to Operator
curl -X POST http://localhost:9300/api/admin/set-level \
  -H "X-API-Key: your-admin-key" \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user-uuid-here", "level": 400}'

# Response:
# {"success": true, "message": "User level updated to 400 (Operator)",
#  "user_id": "user-uuid-here", "new_level": 400, "level_name": "Operator"}
```

## 23.4 Instance Directory Structure

The controller discovers instances by scanning subdirectories of the configured `instances_base_dir` (default: `./Instances`). Each subdirectory must contain a `.env` file to be recognized.

### 23.4.1 Required Structure

```
Instances/
├── Gaiagrid/
│   ├── .env                  # Required: instance configuration
│   ├── Regions/
│   │   └── Regions.ini       # Region definitions
│   ├── config/               # OpenSim configuration overrides
│   ├── start.sh              # Optional: custom start script
│   └── preflight.sh          # Optional: pre-start validation
├── TestWorld/
│   ├── .env
│   └── Regions/
│       └── Regions.ini
└── template/                  # Ignored (reserved name)
```

### 23.4.2 Instance .env File

The `.env` file defines the instance's identity and configuration:

```bash
# Instance identity
OPENSIM_SERVICE_MODE=grid          # standalone | grid | robust
OPENSIM_LOGIN_PORT=9500            # Primary listener port

# Database
DATABASE_URL=postgresql://opensim@localhost/gaiagrid

# Hypergrid (optional)
OPENSIM_HYPERGRID_ENABLED=true
OPENSIM_HOME_URI=http://192.168.0.6:9500

# Robust server (grid mode only)
OPENSIM_ROBUST_URL=http://localhost:8503
OPENSIM_ROBUST_PORT=8503
```

When the controller spawns an instance, it reads this `.env` file and passes all variables to the child process along with `OPENSIM_CONTROLLER_URL` pointing back to the controller.

## 23.5 Configuration Reference

### 23.5.1 instances.toml

The `instances.toml` file in the project root configures the controller and pre-defines known instances:

```toml
[controller]
discovery_mode = "config"          # config | multicast | consul | kubernetes
health_check_interval_ms = 5000
heartbeat_timeout_ms = 15000
reconnect_delay_ms = 3000
max_reconnect_attempts = 5
command_timeout_ms = 30000
controller_port = 9300             # Management API port
instances_base_dir = "./Instances" # Where to scan for instance directories
binary_path = ""                   # Empty = use current executable

[[instances]]
id = "local-dev"
name = "Local Development"
description = "Local development server for testing"
host = "localhost"
websocket_port = 9001
admin_port = 9200
metrics_port = 9100
http_port = 9000
udp_port = 9000
api_key = "dev-api-key-change-me"
environment = "development"
auto_connect = true
tags = ["development", "local", "testing"]
```

### 23.5.2 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `OPENSIM_SERVICE_MODE` | `standalone` | `standalone`, `grid`, `robust`, or `controller` |
| `OPENSIM_EMBEDDED_CONTROLLER` | — | Set to `true` to run controller inside the instance |
| `OPENSIM_CONTROLLER_URL` | — | URL of external controller (e.g., `http://localhost:9300`) |
| `OPENSIM_CONTROLLER_PORT` | `9300` | Port for embedded controller |
| `OPENSIM_API_KEY` | `dev-api-key-change-me` | API key for CentralAdmin access |
| `OPENSIM_JWT_SECRET` | — | Secret for JWT token validation |
| `OPENSIM_INSTANCE_ID` | auto-detected | Override instance identity |
| `OPENSIM_INSTANCE_DIR` | — | Path to this instance's directory |

### 23.5.3 Environment Variable Precedence

1. `OPENSIM_EMBEDDED_CONTROLLER=true` — starts embedded controller, ignores `OPENSIM_CONTROLLER_URL`
2. `OPENSIM_CONTROLLER_URL` set — announces to external controller
3. Neither set — standalone operation, no management overhead

## 23.6 Operational Procedures

### 23.6.1 Starting the System (Dedicated Controller)

```bash
# Step 1: Start the controller
OPENSIM_SERVICE_MODE=controller \
OPENSIM_API_KEY="your-secure-key" \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next &

# Step 2: Verify controller is healthy
curl http://localhost:9300/health
# {"status":"healthy","controller_port":9300,"instances":{"total":1,"running":0,"discovered":1}}

# Step 3: Check what's available
curl -H "X-API-Key: your-secure-key" http://localhost:9300/api/instances
# Shows all discovered instances with "Discovered" status

# Step 4: Start the instances you need
curl -X POST -H "X-API-Key: your-secure-key" \
  http://localhost:9300/api/instance/gaiagrid/start
# {"success":true,"message":"Instance gaiagrid started (PID 12345)","pid":12345}

# Step 5: Watch it come online
curl -H "X-API-Key: your-secure-key" http://localhost:9300/api/instances
# Gaiagrid now shows "Running" status
```

### 23.6.2 Starting the System (Embedded Controller)

```bash
# Single command — instance + controller in one process
OPENSIM_EMBEDDED_CONTROLLER=true \
OPENSIM_SERVICE_MODE=standalone \
OPENSIM_API_KEY="your-secure-key" \
RUST_LOG=info \
DYLD_LIBRARY_PATH=$(pwd)/zig/zig-out/lib:$(pwd)/bin/lib64 \
./target/release/opensim-next

# The instance serves regions AND the controller API
# Region clients connect to port 9000 (or configured port)
# Management API available on port 9300
# Flutter dashboard connects to ws://localhost:9300/ws
```

### 23.6.3 Monitoring Instance Health

```bash
# List all instances with their current status
curl -H "X-API-Key: $KEY" http://localhost:9300/api/instances

# View recent console output (last 200 lines)
curl -H "X-API-Key: $KEY" http://localhost:9300/api/instance/gaiagrid/console

# Check overall controller health
curl http://localhost:9300/health
```

### 23.6.4 Graceful Shutdown

```bash
# Stop a specific instance (SIGTERM, 10s grace period, then SIGKILL)
curl -X POST -H "X-API-Key: $KEY" \
  http://localhost:9300/api/instance/gaiagrid/stop

# Stop the controller itself: Ctrl+C
# Running child instances will receive SIGTERM automatically
```

### 23.6.5 Emergency Procedures

If an instance becomes unresponsive:

```bash
# Try graceful stop first
curl -X POST -H "X-API-Key: $KEY" \
  http://localhost:9300/api/instance/gaiagrid/stop

# If that fails, check the PID and kill manually
ps aux | grep opensim-next

# Restart the instance
curl -X POST -H "X-API-Key: $KEY" \
  http://localhost:9300/api/instance/gaiagrid/restart
```

## 23.7 Flutter Dashboard

The Flutter Configurator is a native macOS application that provides a graphical interface to the controller.

### 23.7.1 Connection

The dashboard connects to the controller via WebSocket at `ws://localhost:9300/ws`. On connection, it immediately receives the full instance list. All subsequent state changes are pushed in real-time.

### 23.7.2 Capabilities

- **Instance overview**: See all discovered and running instances at a glance
- **Start/Stop/Restart**: One-click instance lifecycle management
- **Console streaming**: Real-time stdout/stderr from all managed instances
- **Status indicators**: Color-coded health status (green=running, purple=discovered, red=error)
- **Metrics display**: CPU, memory, active users, uptime for each running instance

### 23.7.3 Building the Dashboard

The Flutter dashboard is built exclusively for macOS (no web builds):

```bash
cd flutter-client/opensim_configurator
flutter build macos
```

The resulting application connects directly to the controller's HTTP and WebSocket endpoints without any CORS restrictions, since native applications bypass browser security policies entirely.

## 23.8 API Reference

### 23.8.1 Health Check

```
GET /health
```

No authentication required. Returns controller status and instance counts.

**Response**:
```json
{
  "status": "healthy",
  "controller_port": 9300,
  "instances": {
    "total": 3,
    "running": 1,
    "discovered": 2
  }
}
```

### 23.8.2 List Instances

```
GET /api/instances
```

Returns all registered instances with their current status, metrics, and connection state.

### 23.8.3 Scan Instance Directories

```
GET /api/instance-dirs
```

Re-scans the `Instances/` directory and returns all discoverable instance directories with their configuration details (service mode, port, region count, database URL, hypergrid status).

### 23.8.4 Start Instance

```
POST /api/instance/:id/start
```

Spawns the instance as a child process. The controller reads the instance's `.env` file, sets up environment variables (including `OPENSIM_CONTROLLER_URL`), captures stdout/stderr, and monitors the process.

**Response**:
```json
{
  "success": true,
  "message": "Instance gaiagrid started (PID 12345)",
  "pid": 12345
}
```

### 23.8.5 Stop Instance

```
POST /api/instance/:id/stop
```

Sends SIGTERM to the instance process. Waits up to 10 seconds for graceful shutdown. If the process doesn't exit, sends SIGKILL.

### 23.8.6 Restart Instance

```
POST /api/instance/:id/restart
```

Stops the instance (graceful), waits 2 seconds, then starts it again.

### 23.8.7 View Console

```
GET /api/instance/:id/console
```

Returns the most recent 200 lines of console output (stdout + stderr) from the managed instance process.

**Response**:
```json
{
  "entries": [
    {"line": "Starting OpenSim Next Server...", "stream": "stdout", "timestamp": 1740000000},
    {"line": "Region Gaia One loaded", "stream": "stdout", "timestamp": 1740000005}
  ],
  "count": 200
}
```

### 23.8.8 Instance Announcement (Internal)

```
POST /api/instance/announce
```

Called by instances on boot. No authentication required (internal service communication).

**Request Body**:
```json
{
  "instance_id": "gaiagrid",
  "service_mode": "grid",
  "ports": {"login": 9500, "admin": 9200, "metrics": 9100},
  "region_count": 16,
  "capabilities": ["lludp", "http", "caps"],
  "version": "0.1.0",
  "host": "localhost"
}
```

### 23.8.9 Heartbeat (Internal)

```
POST /api/instance/heartbeat
```

Called every 30 seconds by running instances.

**Request Body**:
```json
{
  "instance_id": "gaiagrid",
  "status": "running",
  "active_users": 5,
  "active_regions": 16,
  "uptime_seconds": 3600,
  "cpu_usage": 12.5,
  "memory_usage_mb": 512
}
```

### 23.8.10 Set User Level

```
POST /api/admin/set-level
```

CentralAdmin only. Sets a user's access level.

**Request Body**:
```json
{
  "user_id": "uuid-of-user",
  "level": 400
}
```

### 23.8.11 WebSocket

```
GET /ws → Upgrade to WebSocket
```

Real-time bidirectional communication. On connect, receives `InstanceList` message. Subsequently receives:

| Message Type | Direction | Description |
|-------------|-----------|-------------|
| `InstanceList` | Server → Client | Full instance list (sent on connect) |
| `InstanceAnnounced` | Server → Client | New instance came online |
| `InstanceDeparted` | Server → Client | Instance went offline |
| `ProcessOutput` | Server → Client | Console line from managed instance |
| `Heartbeat` | Client → Server | Keep-alive ping |
| `Subscribe` | Client → Server | Subscribe to event channels |

## 23.9 Security Considerations

### 23.9.1 API Key Management

- **Never use the default API key** (`dev-api-key-change-me`) in production
- Set `OPENSIM_API_KEY` to a strong, random value (minimum 32 characters)
- Rotate API keys periodically
- API key holders receive CentralAdmin (500) privileges — treat them as root credentials

### 23.9.2 JWT Tokens

- Set `OPENSIM_JWT_SECRET` to a strong secret (minimum 256 bits)
- Issue tokens with appropriate `user_level` claims — do not over-provision
- Set reasonable expiration times on tokens
- The controller validates tokens on every request — no session state

### 23.9.3 Network Security

- The controller listens on `0.0.0.0` by default — restrict with firewall rules in production
- Instance announcements and heartbeats are unauthenticated (internal service calls) — ensure the controller port is not exposed to untrusted networks
- For multi-server deployments, use a private network or VPN between controller and instances

### 23.9.4 Principle of Least Privilege

Assign the minimum access level required:
- Grid operators managing day-to-day operations: **Operator (400)**
- Staff managing user accounts and regions: **GridAdmin (300)**
- Region owners managing their own content: **RegionOwner (200)**
- Dashboard viewers: **User (100)**
- Reserve **CentralAdmin (500)** for the system owner only

## 23.10 Deployment Topology Examples

### 23.10.1 Personal Grid (Simplest)

```
┌──────────────────────────────┐
│  Single Process               │
│  OPENSIM_EMBEDDED_CONTROLLER  │
│  ┌────────────────────────┐  │
│  │ Region Server (9000)   │  │
│  │ Embedded Controller    │  │
│  │ (9300)                 │  │
│  └────────────────────────┘  │
└──────────────────────────────┘
```

One process, one machine. Serves a personal virtual world and provides the management dashboard. Start it, connect Firestorm, connect Flutter dashboard — done.

### 23.10.2 Community Grid (Medium)

```
┌─────────────────┐   ┌─────────────────┐
│ Controller (9300)│   │ Region Server   │
│ SERVICE_MODE=    │──▶│ Gaiagrid        │
│ controller       │   │ 16 regions      │
│                  │   │ Ports 9500-9515 │
└─────────────────┘   └─────────────────┘
```

Dedicated controller spawns and manages the region server. The controller can be on the same machine (different process) or a different machine entirely.

### 23.10.3 Enterprise Grid (Large)

```
┌─────────────────┐
│ Controller (9300)│
│ CentralAdmin     │
│ manages all      │
└───┬───┬───┬─────┘
    │   │   │
    ▼   ▼   ▼
┌──────┐┌──────┐┌──────┐
│Robust││Region││Region│
│Server││Srv 1 ││Srv 2 │
│(8503)││(9500)││(9600)│
└──────┘└──────┘└──────┘
```

One controller managing a Robust services server plus multiple region servers. Each region server can host multiple regions. All announce back to the controller. The Flutter dashboard shows the complete picture.

## 23.11 Troubleshooting

### 23.11.1 Instance Won't Start

**Check the instance directory**:
```bash
# Verify .env exists
ls Instances/gaiagrid/.env

# Verify the .env has OPENSIM_SERVICE_MODE
grep OPENSIM_SERVICE_MODE Instances/gaiagrid/.env
```

**Check the console output**:
```bash
curl -H "X-API-Key: $KEY" http://localhost:9300/api/instance/gaiagrid/console
```

**Common causes**:
- Missing or invalid `.env` file
- Port already in use by another process
- Missing `DYLD_LIBRARY_PATH` / `LD_LIBRARY_PATH` (Zig libraries not found)
- Database connection failure

### 23.11.2 Instance Shows "Discovered" But Never "Running"

The instance process started but never announced back. Check:

1. Console output for startup errors
2. That the child process received `OPENSIM_CONTROLLER_URL` correctly
3. That port 9300 is accessible from the child process
4. Network/firewall rules

### 23.11.3 Flutter Dashboard Won't Connect

- Verify the controller is running: `curl http://localhost:9300/health`
- Verify WebSocket port is accessible: check firewall rules for port 9300
- Ensure Flutter app is built for macOS (not web)
- Check that the dashboard is configured to connect to the correct host and port

### 23.11.4 "Port Already in Use" on Controller Start

```bash
# Find what's using port 9300
lsof -i :9300 -P -n

# Kill the offending process or use a different port
OPENSIM_CONTROLLER_PORT=9301
```

### 23.11.5 Ghost Processes After Force-Kill

If the controller is killed without graceful shutdown, child processes may become orphaned:

```bash
# Find all opensim-next processes
ps aux | grep opensim-next

# Kill orphaned processes
pkill -f opensim-next

# On macOS, check for ghost UDP sockets
lsof -i UDP:9000-9600 -P -n
```

## 23.12 Future Roadmap

The Instance Admin Controller lays the foundation for:

- **Customer control panels**: Web-based self-service portals where customers can spin up their own virtual world instances on demand
- **Auto-scaling**: Automatic instance spawning based on load metrics
- **Multi-machine orchestration**: Controller managing instances across multiple physical servers
- **Kubernetes integration**: `discovery_mode = "kubernetes"` for cloud-native deployments
- **Consul integration**: `discovery_mode = "consul"` for service mesh environments
- **Billing integration**: Metered usage tracking per instance for commercial grids
- **Automated backups**: Scheduled OAR/IAR backups managed by the controller
