# OpenSim Next Monitoring and Administration Setup Guide

## Table of Contents

1. [Overview](#overview)
2. [Monitoring Architecture](#monitoring-architecture)
3. [Prometheus Metrics Setup](#prometheus-metrics-setup)
4. [Grafana Dashboard Configuration](#grafana-dashboard-configuration)
5. [Log Management and Analysis](#log-management-and-analysis)
6. [Health Checks and Alerting](#health-checks-and-alerting)
7. [Performance Monitoring](#performance-monitoring)
8. [Administration Dashboard](#administration-dashboard)
9. [Real-Time Statistics](#real-time-statistics)
10. [Database Monitoring](#database-monitoring)
11. [Network and Security Monitoring](#network-and-security-monitoring)
12. [Automated Alert Management](#automated-alert-management)
13. [Capacity Planning and Scaling](#capacity-planning-and-scaling)
14. [Troubleshooting and Maintenance](#troubleshooting-and-maintenance)

## Overview

OpenSim Next features a comprehensive monitoring and administration system designed for enterprise-grade virtual world deployments. This guide covers the complete setup and configuration of production-ready monitoring, alerting, and administration capabilities.

### Key Monitoring Features

🔍 **Comprehensive Metrics**: Prometheus-compatible metrics for all system components  
📊 **Real-Time Dashboards**: Grafana integration with custom OpenSim Next dashboards  
🚨 **Intelligent Alerting**: Multi-channel alerting with escalation policies  
📈 **Performance Analytics**: Deep insights into virtual world performance  
🌐 **Multi-Protocol Monitoring**: Traditional viewers, WebSocket, and web clients  
🔒 **Security Monitoring**: Zero trust network and authentication tracking  
⚡ **Auto-Scaling Integration**: Capacity-based scaling decisions  
📱 **Mobile Administration**: Responsive admin interface for on-the-go management  

## Monitoring Architecture

### Production Monitoring Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                    External Monitoring                         │
│  Uptime Monitors │ External APIs │ CDN Monitoring │ DNS        │
├─────────────────────────────────────────────────────────────────┤
│                     Alert Management                           │
│ PagerDuty │ Slack │ Discord │ Email │ SMS │ Webhooks           │
├─────────────────────────────────────────────────────────────────┤
│                   Visualization Layer                          │
│       Grafana Dashboards │ Admin Console │ Mobile App          │
├─────────────────────────────────────────────────────────────────┤
│                   Analytics & Intelligence                     │
│ Prometheus │ InfluxDB │ ElasticSearch │ Machine Learning      │
├─────────────────────────────────────────────────────────────────┤
│                   Log Aggregation Layer                        │
│  Structured Logs │ Audit Trails │ Security Events │ Traces   │
├─────────────────────────────────────────────────────────────────┤
│                  Application Metrics Layer                     │
│ Business Metrics │ User Analytics │ Performance │ Errors      │
├─────────────────────────────────────────────────────────────────┤
│                    OpenSim Next Core                           │
│ Main Server │ WebSocket │ Physics │ Database │ Zero Trust     │
└─────────────────────────────────────────────────────────────────┘
```

### Monitoring Components

**Core Monitoring:**
- **Prometheus Server**: Central metrics collection and storage
- **Grafana**: Visualization and dashboard platform
- **AlertManager**: Alert routing and management
- **Node Exporter**: System-level metrics collection
- **Custom Exporters**: OpenSim Next specific metrics

**Advanced Monitoring:**
- **Jaeger/Zipkin**: Distributed tracing for performance analysis
- **ElasticSearch + Kibana**: Log aggregation and analysis
- **InfluxDB**: Time-series data for high-frequency metrics
- **Machine Learning**: Anomaly detection and predictive analytics

## Prometheus Metrics Setup

### Basic Prometheus Configuration

Create prometheus configuration file:

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    monitor: 'opensim-next-monitor'
    environment: 'production'

rule_files:
  - "opensim_rules.yml"
  - "alert_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  # OpenSim Next main server
  - job_name: 'opensim-next'
    static_configs:
      - targets: ['localhost:9100']
    scrape_interval: 5s
    metrics_path: '/metrics'
    basic_auth:
      username: 'prometheus'
      password: 'your-api-key'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        replacement: 'opensim-main'

  # WebSocket server metrics
  - job_name: 'opensim-websocket'
    static_configs:
      - targets: ['localhost:9101']
    scrape_interval: 5s
    metrics_path: '/metrics'
    basic_auth:
      username: 'prometheus'
      password: 'your-api-key'

  # Physics engines
  - job_name: 'opensim-physics'
    static_configs:
      - targets: ['localhost:9102']
    scrape_interval: 10s
    metrics_path: '/physics/metrics'

  # Database metrics
  - job_name: 'postgresql'
    static_configs:
      - targets: ['localhost:9187']
    scrape_interval: 15s

  # System metrics
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']
    scrape_interval: 15s

  # Zero Trust network
  - job_name: 'opensim-ziti'
    static_configs:
      - targets: ['localhost:9103']
    scrape_interval: 30s
    metrics_path: '/ziti/metrics'

  # Load balancer metrics
  - job_name: 'opensim-loadbalancer'
    static_configs:
      - targets: ['localhost:9104']
    scrape_interval: 10s

  # Asset CDN metrics
  - job_name: 'opensim-cdn'
    static_configs:
      - targets: ['localhost:9105']
    scrape_interval: 30s
```

### OpenSim Next Metrics Configuration

Configure metrics in OpenSim Next:

```ini
[Monitoring]
; Enable Prometheus metrics endpoint
EnableMetrics = true
MetricsPort = 9100
MetricsPath = "/metrics"

; Metrics authentication
RequireAuthentication = true
MetricsUsername = "prometheus"
MetricsPassword = "your-secure-api-key"

; Metrics collection intervals
BasicMetricsInterval = 5      ; seconds
PerformanceMetricsInterval = 1 ; seconds
DatabaseMetricsInterval = 15  ; seconds
PhysicsMetricsInterval = 10   ; seconds

; Enable detailed metrics
EnableDetailedMetrics = true
EnableUserMetrics = true
EnableRegionMetrics = true
EnablePhysicsMetrics = true
EnableNetworkMetrics = true
EnableWebSocketMetrics = true
EnableZeroTrustMetrics = true

; Metrics retention
MetricsRetentionDays = 30
MetricsCleanupInterval = 3600 ; seconds

; Custom metrics labels
Environment = "production"
Datacenter = "us-east-1"
Cluster = "opensim-cluster-1"
```

### Custom Metrics Implementation

Example of custom metrics in Rust:

```rust
// metrics/mod.rs
use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge, Registry, 
    register_counter_with_registry, register_gauge_with_registry,
    register_histogram_with_registry, register_int_counter_with_registry,
    register_int_gauge_with_registry
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct OpenSimMetrics {
    registry: Registry,
    
    // Connection metrics
    pub active_connections: IntGauge,
    pub total_connections: IntCounter,
    pub failed_connections: IntCounter,
    pub websocket_connections: IntGauge,
    
    // User metrics
    pub online_users: IntGauge,
    pub total_users: IntGauge,
    pub user_logins: IntCounter,
    pub user_logouts: IntCounter,
    pub failed_logins: IntCounter,
    
    // Region metrics
    pub active_regions: IntGauge,
    pub region_fps: Gauge,
    pub region_memory_usage: Gauge,
    pub region_cpu_usage: Gauge,
    pub physics_simulations_per_second: Gauge,
    
    // Performance metrics
    pub request_duration: Histogram,
    pub response_time: Histogram,
    pub message_processing_time: Histogram,
    pub database_query_time: Histogram,
    
    // Error metrics
    pub total_errors: IntCounter,
    pub critical_errors: IntCounter,
    pub warning_count: IntCounter,
    
    // Business metrics
    pub chat_messages_sent: IntCounter,
    pub objects_created: IntCounter,
    pub assets_uploaded: IntCounter,
    pub teleports_completed: IntCounter,
    
    // Zero Trust metrics
    pub ziti_connections: IntGauge,
    pub encrypted_messages: IntCounter,
    pub auth_failures: IntCounter,
    pub policy_violations: IntCounter,
}

impl OpenSimMetrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        
        let metrics = Self {
            // Register all metrics with the registry
            active_connections: register_int_gauge_with_registry!(
                "opensim_active_connections_total",
                "Number of active connections",
                registry
            )?,
            
            total_connections: register_int_counter_with_registry!(
                "opensim_connections_total",
                "Total number of connections",
                registry
            )?,
            
            failed_connections: register_int_counter_with_registry!(
                "opensim_failed_connections_total",
                "Number of failed connections",
                registry
            )?,
            
            websocket_connections: register_int_gauge_with_registry!(
                "opensim_websocket_connections",
                "Number of active WebSocket connections",
                registry
            )?,
            
            online_users: register_int_gauge_with_registry!(
                "opensim_online_users",
                "Number of users currently online",
                registry
            )?,
            
            total_users: register_int_gauge_with_registry!(
                "opensim_total_users",
                "Total number of registered users",
                registry
            )?,
            
            user_logins: register_int_counter_with_registry!(
                "opensim_user_logins_total",
                "Total number of user logins",
                registry
            )?,
            
            user_logouts: register_int_counter_with_registry!(
                "opensim_user_logouts_total",
                "Total number of user logouts",
                registry
            )?,
            
            failed_logins: register_int_counter_with_registry!(
                "opensim_failed_logins_total",
                "Number of failed login attempts",
                registry
            )?,
            
            active_regions: register_int_gauge_with_registry!(
                "opensim_active_regions",
                "Number of active regions",
                registry
            )?,
            
            region_fps: register_gauge_with_registry!(
                "opensim_region_fps",
                "Region simulation frames per second",
                registry
            )?,
            
            region_memory_usage: register_gauge_with_registry!(
                "opensim_region_memory_bytes",
                "Memory usage per region in bytes",
                registry
            )?,
            
            region_cpu_usage: register_gauge_with_registry!(
                "opensim_region_cpu_percent",
                "CPU usage per region as percentage",
                registry
            )?,
            
            physics_simulations_per_second: register_gauge_with_registry!(
                "opensim_physics_simulations_per_second",
                "Physics simulations per second",
                registry
            )?,
            
            request_duration: register_histogram_with_registry!(
                "opensim_request_duration_seconds",
                "Request duration in seconds",
                vec![0.001, 0.01, 0.1, 0.5, 1.0, 2.5, 5.0, 10.0],
                registry
            )?,
            
            response_time: register_histogram_with_registry!(
                "opensim_response_time_seconds",
                "Response time in seconds",
                vec![0.001, 0.01, 0.1, 0.5, 1.0, 2.5, 5.0, 10.0],
                registry
            )?,
            
            message_processing_time: register_histogram_with_registry!(
                "opensim_message_processing_seconds",
                "Message processing time in seconds",
                vec![0.0001, 0.001, 0.01, 0.1, 0.5, 1.0],
                registry
            )?,
            
            database_query_time: register_histogram_with_registry!(
                "opensim_database_query_seconds",
                "Database query time in seconds",
                vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0],
                registry
            )?,
            
            total_errors: register_int_counter_with_registry!(
                "opensim_errors_total",
                "Total number of errors",
                registry
            )?,
            
            critical_errors: register_int_counter_with_registry!(
                "opensim_critical_errors_total",
                "Number of critical errors",
                registry
            )?,
            
            warning_count: register_int_counter_with_registry!(
                "opensim_warnings_total",
                "Number of warnings",
                registry
            )?,
            
            chat_messages_sent: register_int_counter_with_registry!(
                "opensim_chat_messages_total",
                "Total chat messages sent",
                registry
            )?,
            
            objects_created: register_int_counter_with_registry!(
                "opensim_objects_created_total",
                "Total objects created",
                registry
            )?,
            
            assets_uploaded: register_int_counter_with_registry!(
                "opensim_assets_uploaded_total",
                "Total assets uploaded",
                registry
            )?,
            
            teleports_completed: register_int_counter_with_registry!(
                "opensim_teleports_completed_total",
                "Total teleports completed",
                registry
            )?,
            
            ziti_connections: register_int_gauge_with_registry!(
                "opensim_ziti_connections",
                "Number of OpenZiti connections",
                registry
            )?,
            
            encrypted_messages: register_int_counter_with_registry!(
                "opensim_encrypted_messages_total",
                "Total encrypted messages",
                registry
            )?,
            
            auth_failures: register_int_counter_with_registry!(
                "opensim_auth_failures_total",
                "Authentication failures",
                registry
            )?,
            
            policy_violations: register_int_counter_with_registry!(
                "opensim_policy_violations_total",
                "Zero trust policy violations",
                registry
            )?,
            
            registry,
        };
        
        Ok(metrics)
    }
    
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    
    // Convenience methods for updating metrics
    pub fn increment_user_login(&self) {
        self.user_logins.inc();
        self.online_users.inc();
    }
    
    pub fn increment_user_logout(&self) {
        self.user_logouts.inc();
        self.online_users.dec();
    }
    
    pub fn record_request_duration(&self, duration: f64) {
        self.request_duration.observe(duration);
    }
    
    pub fn record_database_query(&self, duration: f64) {
        self.database_query_time.observe(duration);
    }
    
    pub fn update_region_performance(&self, region_id: &str, fps: f64, memory_mb: f64, cpu_percent: f64) {
        // Update with labels for specific regions
        self.region_fps.set(fps);
        self.region_memory_usage.set(memory_mb * 1024.0 * 1024.0); // Convert to bytes
        self.region_cpu_usage.set(cpu_percent);
    }
}

// Global metrics instance
lazy_static::lazy_static! {
    pub static ref METRICS: Arc<RwLock<Option<OpenSimMetrics>>> = Arc::new(RwLock::new(None));
}

pub async fn initialize_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let metrics = OpenSimMetrics::new()?;
    *METRICS.write().await = Some(metrics);
    Ok(())
}

pub async fn get_metrics() -> Option<Arc<OpenSimMetrics>> {
    METRICS.read().await.as_ref().map(|m| Arc::new(m.clone()))
}
```

### Alert Rules Configuration

Create Prometheus alert rules:

```yaml
# opensim_rules.yml
groups:
- name: opensim.rules
  rules:
  # High-level health indicators
  - record: opensim:health_score
    expr: |
      (
        (opensim_active_connections > 0) * 0.2 +
        (opensim_region_fps > 55) * 0.3 +
        (opensim_response_time_seconds:p95 < 0.1) * 0.2 +
        (opensim_database_query_seconds:p95 < 0.05) * 0.2 +
        (opensim_critical_errors_total:rate5m == 0) * 0.1
      ) * 100

  # Performance indicators
  - record: opensim:response_time_seconds:p95
    expr: histogram_quantile(0.95, rate(opensim_response_time_seconds_bucket[5m]))

  - record: opensim:response_time_seconds:p99
    expr: histogram_quantile(0.99, rate(opensim_response_time_seconds_bucket[5m]))

  - record: opensim:request_rate
    expr: rate(opensim_request_duration_seconds_count[5m])

  - record: opensim:error_rate
    expr: rate(opensim_errors_total[5m])

  - record: opensim:critical_error_rate
    expr: rate(opensim_critical_errors_total[5m])

  # User activity
  - record: opensim:user_login_rate
    expr: rate(opensim_user_logins_total[1h])

  - record: opensim:chat_message_rate
    expr: rate(opensim_chat_messages_total[5m])

  # Resource utilization
  - record: opensim:memory_utilization_percent
    expr: (opensim_region_memory_bytes / (1024 * 1024 * 1024)) * 100

  - record: opensim:connection_utilization_percent
    expr: (opensim_active_connections / 1000) * 100

- name: opensim.alerts
  rules:
  # Critical alerts
  - alert: OpenSimDown
    expr: up{job="opensim-next"} == 0
    for: 1m
    labels:
      severity: critical
      service: opensim-next
    annotations:
      summary: "OpenSim Next server is down"
      description: "OpenSim Next server has been down for more than 1 minute"

  - alert: HighErrorRate
    expr: opensim:error_rate > 10
    for: 5m
    labels:
      severity: critical
      service: opensim-next
    annotations:
      summary: "High error rate detected"
      description: "Error rate is {{ $value }} errors/second over the last 5 minutes"

  - alert: CriticalError
    expr: opensim:critical_error_rate > 0
    for: 0m
    labels:
      severity: critical
      service: opensim-next
    annotations:
      summary: "Critical error detected"
      description: "Critical error rate: {{ $value }} errors/second"

  # Performance alerts
  - alert: HighResponseTime
    expr: opensim:response_time_seconds:p95 > 1.0
    for: 10m
    labels:
      severity: warning
      service: opensim-next
    annotations:
      summary: "High response time"
      description: "95th percentile response time is {{ $value }}s"

  - alert: LowRegionFPS
    expr: opensim_region_fps < 30
    for: 5m
    labels:
      severity: warning
      service: opensim-next
    annotations:
      summary: "Low region FPS"
      description: "Region FPS is {{ $value }}, below 30 FPS threshold"

  # Capacity alerts
  - alert: HighConnectionUsage
    expr: opensim:connection_utilization_percent > 80
    for: 5m
    labels:
      severity: warning
      service: opensim-next
    annotations:
      summary: "High connection usage"
      description: "Connection utilization is {{ $value }}%"

  - alert: HighMemoryUsage
    expr: opensim:memory_utilization_percent > 85
    for: 10m
    labels:
      severity: warning
      service: opensim-next
    annotations:
      summary: "High memory usage"
      description: "Memory utilization is {{ $value }}%"

  # Security alerts
  - alert: HighAuthFailures
    expr: rate(opensim_auth_failures_total[5m]) > 5
    for: 2m
    labels:
      severity: warning
      service: opensim-next
      category: security
    annotations:
      summary: "High authentication failure rate"
      description: "Authentication failure rate: {{ $value }} failures/second"

  - alert: ZeroTrustPolicyViolation
    expr: rate(opensim_policy_violations_total[1m]) > 0
    for: 0m
    labels:
      severity: critical
      service: opensim-next
      category: security
    annotations:
      summary: "Zero trust policy violation"
      description: "Policy violation rate: {{ $value }} violations/second"

  # WebSocket specific alerts
  - alert: WebSocketConnectionDrop
    expr: delta(opensim_websocket_connections[5m]) < -10
    for: 1m
    labels:
      severity: warning
      service: websocket
    annotations:
      summary: "Significant WebSocket connection drop"
      description: "WebSocket connections dropped by {{ $value }} in 5 minutes"

  # Database alerts
  - alert: SlowDatabaseQueries
    expr: histogram_quantile(0.95, rate(opensim_database_query_seconds_bucket[5m])) > 0.5
    for: 10m
    labels:
      severity: warning
      service: database
    annotations:
      summary: "Slow database queries"
      description: "95th percentile database query time: {{ $value }}s"
```

## Grafana Dashboard Configuration

### Installation and Setup

Install Grafana and configure data sources:

```bash
# Install Grafana
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
sudo apt-get update
sudo apt-get install grafana

# Start Grafana
sudo systemctl start grafana-server
sudo systemctl enable grafana-server

# Access Grafana at http://localhost:3000
# Default login: admin/admin
```

### Data Source Configuration

Configure Prometheus as a data source:

```json
{
  "name": "OpenSim-Prometheus",
  "type": "prometheus",
  "url": "http://localhost:9090",
  "access": "proxy",
  "basicAuth": false,
  "jsonData": {
    "httpMethod": "POST",
    "queryTimeout": "60s",
    "timeInterval": "5s"
  }
}
```

### OpenSim Next Dashboard JSON

Complete Grafana dashboard configuration:

```json
{
  "dashboard": {
    "id": null,
    "title": "OpenSim Next - Production Overview",
    "tags": ["opensim", "virtual-world", "production"],
    "timezone": "browser",
    "refresh": "30s",
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "panels": [
      {
        "id": 1,
        "title": "System Health Overview",
        "type": "stat",
        "targets": [
          {
            "expr": "opensim:health_score",
            "legendFormat": "Health Score"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "color": {
              "mode": "thresholds"
            },
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 70},
                {"color": "green", "value": 90}
              ]
            },
            "unit": "percent",
            "min": 0,
            "max": 100
          }
        },
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Active Users & Connections",
        "type": "timeseries",
        "targets": [
          {
            "expr": "opensim_online_users",
            "legendFormat": "Online Users"
          },
          {
            "expr": "opensim_active_connections",
            "legendFormat": "Total Connections"
          },
          {
            "expr": "opensim_websocket_connections",
            "legendFormat": "WebSocket Connections"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}
      },
      {
        "id": 3,
        "title": "Region Performance",
        "type": "timeseries",
        "targets": [
          {
            "expr": "opensim_region_fps",
            "legendFormat": "Region FPS"
          },
          {
            "expr": "opensim_physics_simulations_per_second",
            "legendFormat": "Physics Simulations/sec"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "custom": {
              "drawStyle": "line",
              "lineInterpolation": "smooth",
              "fillOpacity": 10
            }
          }
        },
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 8}
      },
      {
        "id": 4,
        "title": "Response Time Distribution",
        "type": "heatmap",
        "targets": [
          {
            "expr": "rate(opensim_response_time_seconds_bucket[5m])",
            "legendFormat": "{{le}}"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16}
      },
      {
        "id": 5,
        "title": "Error Rates",
        "type": "timeseries",
        "targets": [
          {
            "expr": "opensim:error_rate",
            "legendFormat": "Total Errors/sec"
          },
          {
            "expr": "opensim:critical_error_rate",
            "legendFormat": "Critical Errors/sec"
          },
          {
            "expr": "rate(opensim_warnings_total[5m])",
            "legendFormat": "Warnings/sec"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "color": {
              "mode": "palette-classic"
            },
            "custom": {
              "axisPlacement": "auto",
              "drawStyle": "line",
              "fillOpacity": 20
            }
          }
        },
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 16}
      },
      {
        "id": 6,
        "title": "Virtual World Activity",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(opensim_chat_messages_total[1h])",
            "legendFormat": "Chat Messages/hour"
          },
          {
            "expr": "rate(opensim_objects_created_total[1h])",
            "legendFormat": "Objects Created/hour"
          },
          {
            "expr": "rate(opensim_teleports_completed_total[1h])",
            "legendFormat": "Teleports/hour"
          },
          {
            "expr": "rate(opensim_assets_uploaded_total[1h])",
            "legendFormat": "Assets Uploaded/hour"
          }
        ],
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 24}
      },
      {
        "id": 7,
        "title": "Database Performance",
        "type": "timeseries",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(opensim_database_query_seconds_bucket[5m]))",
            "legendFormat": "50th percentile"
          },
          {
            "expr": "histogram_quantile(0.95, rate(opensim_database_query_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.99, rate(opensim_database_query_seconds_bucket[5m]))",
            "legendFormat": "99th percentile"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "s",
            "custom": {
              "drawStyle": "line",
              "fillOpacity": 10
            }
          }
        },
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 32}
      },
      {
        "id": 8,
        "title": "Zero Trust Network Security",
        "type": "timeseries",
        "targets": [
          {
            "expr": "opensim_ziti_connections",
            "legendFormat": "Active Zero Trust Connections"
          },
          {
            "expr": "rate(opensim_encrypted_messages_total[5m])",
            "legendFormat": "Encrypted Messages/sec"
          },
          {
            "expr": "rate(opensim_auth_failures_total[5m])",
            "legendFormat": "Auth Failures/sec"
          },
          {
            "expr": "rate(opensim_policy_violations_total[5m])",
            "legendFormat": "Policy Violations/sec"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 32}
      }
    ]
  }
}
```

### Multi-Region Dashboard

Dashboard for monitoring multiple regions:

```json
{
  "dashboard": {
    "title": "OpenSim Next - Multi-Region Overview",
    "panels": [
      {
        "id": 1,
        "title": "Region Status Map",
        "type": "geomap",
        "targets": [
          {
            "expr": "opensim_active_regions",
            "legendFormat": "{{region_name}}"
          }
        ],
        "gridPos": {"h": 12, "w": 24, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Physics Engines by Region",
        "type": "piechart",
        "targets": [
          {
            "expr": "count by (physics_engine) (opensim_region_fps)",
            "legendFormat": "{{physics_engine}}"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 12}
      },
      {
        "id": 3,
        "title": "Top Active Regions",
        "type": "table",
        "targets": [
          {
            "expr": "topk(10, opensim_online_users by (region_name))",
            "legendFormat": "{{region_name}}"
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 12}
      }
    ]
  }
}
```

## Log Management and Analysis

### Structured Logging Configuration

Configure comprehensive logging in OpenSim Next:

```ini
[Logging]
; Enable structured logging
EnableStructuredLogging = true
LogFormat = "json"  ; json, text, console
LogLevel = "info"   ; trace, debug, info, warn, error, critical

; Log outputs
EnableConsoleLogging = true
EnableFileLogging = true
EnableSyslogLogging = true
EnableElasticsearchLogging = true

; File logging
LogDirectory = "/var/log/opensim-next"
LogFileName = "opensim-{date}.log"
MaxLogFileSize = "100MB"
MaxLogFiles = 30
CompressOldLogs = true

; Elasticsearch integration
ElasticsearchUrl = "http://localhost:9200"
ElasticsearchIndex = "opensim-logs-{date}"
ElasticsearchBulkSize = 100
ElasticsearchFlushInterval = 5  ; seconds

; Log filtering
LogUserActions = true
LogSecurityEvents = true
LogPerformanceEvents = true
LogErrorDetails = true
LogDebugInfo = false  ; Set to true for debugging

; Privacy and compliance
ObfuscateUserData = true
LogRetentionDays = 90
EnableAuditTrail = true
```

### Elasticsearch and Kibana Setup

Set up log analysis infrastructure:

```bash
# Install Elasticsearch
wget -qO - https://artifacts.elastic.co/GPG-KEY-elasticsearch | sudo apt-key add -
sudo apt-get install apt-transport-https
echo "deb https://artifacts.elastic.co/packages/7.x/apt stable main" | sudo tee /etc/apt/sources.list.d/elastic-7.x.list
sudo apt-get update && sudo apt-get install elasticsearch

# Configure Elasticsearch
sudo tee /etc/elasticsearch/elasticsearch.yml << 'EOF'
cluster.name: opensim-logs
node.name: opensim-log-node-1
path.data: /var/lib/elasticsearch
path.logs: /var/log/elasticsearch
network.host: localhost
http.port: 9200
discovery.type: single-node

# Index settings for OpenSim logs
index.number_of_shards: 1
index.number_of_replicas: 0
EOF

# Start Elasticsearch
sudo systemctl start elasticsearch
sudo systemctl enable elasticsearch

# Install Kibana
sudo apt-get install kibana

# Configure Kibana
sudo tee /etc/kibana/kibana.yml << 'EOF'
server.port: 5601
server.host: "localhost"
elasticsearch.hosts: ["http://localhost:9200"]
kibana.index: ".kibana"
EOF

# Start Kibana
sudo systemctl start kibana
sudo systemctl enable kibana
```

### Log Analysis Queries

Common Elasticsearch queries for OpenSim Next logs:

```json
// User activity analysis
{
  "query": {
    "bool": {
      "must": [
        {"range": {"timestamp": {"gte": "now-1h"}}},
        {"term": {"event_type": "user_action"}}
      ]
    }
  },
  "aggs": {
    "user_actions": {
      "terms": {
        "field": "action_type.keyword",
        "size": 10
      }
    },
    "users_by_region": {
      "terms": {
        "field": "region_name.keyword",
        "size": 10
      }
    }
  }
}

// Security event analysis
{
  "query": {
    "bool": {
      "must": [
        {"range": {"timestamp": {"gte": "now-24h"}}},
        {"term": {"category": "security"}}
      ]
    }
  },
  "aggs": {
    "security_events": {
      "terms": {
        "field": "event_type.keyword"
      }
    },
    "failed_logins_by_ip": {
      "filter": {"term": {"event_type": "login_failed"}},
      "aggs": {
        "top_ips": {
          "terms": {
            "field": "source_ip.keyword",
            "size": 20
          }
        }
      }
    }
  }
}

// Performance issue detection
{
  "query": {
    "bool": {
      "must": [
        {"range": {"timestamp": {"gte": "now-1h"}}},
        {"range": {"response_time_ms": {"gte": 1000}}}
      ]
    }
  },
  "sort": [{"response_time_ms": {"order": "desc"}}],
  "size": 100
}

// Error analysis
{
  "query": {
    "bool": {
      "must": [
        {"range": {"timestamp": {"gte": "now-24h"}}},
        {"terms": {"level": ["error", "critical"]}}
      ]
    }
  },
  "aggs": {
    "errors_by_component": {
      "terms": {
        "field": "component.keyword"
      }
    },
    "error_trends": {
      "date_histogram": {
        "field": "timestamp",
        "calendar_interval": "1h"
      }
    }
  }
}
```

### Kibana Dashboard Configuration

Create comprehensive Kibana dashboards:

```json
{
  "version": "7.15.0",
  "objects": [
    {
      "id": "opensim-overview",
      "type": "dashboard",
      "attributes": {
        "title": "OpenSim Next - Log Overview",
        "panelsJSON": "[{\"version\":\"7.15.0\",\"panelIndex\":\"1\",\"gridData\":{\"x\":0,\"y\":0,\"w\":24,\"h\":15},\"panelConfig\":{\"id\":\"log-levels-pie\",\"type\":\"visualization\",\"title\":\"Log Levels Distribution\"}}]"
      }
    },
    {
      "id": "security-dashboard",
      "type": "dashboard",
      "attributes": {
        "title": "OpenSim Next - Security Events",
        "panelsJSON": "[{\"version\":\"7.15.0\",\"panelIndex\":\"1\",\"gridData\":{\"x\":0,\"y\":0,\"w\":24,\"h\":15},\"panelConfig\":{\"id\":\"security-events-timeline\",\"type\":\"visualization\",\"title\":\"Security Events Timeline\"}}]"
      }
    }
  ]
}
```

## Health Checks and Alerting

### Health Check Implementation

Comprehensive health check system:

```rust
// health/mod.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub last_check: Instant,
    pub duration: Duration,
    pub message: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallHealth {
    pub status: HealthStatus,
    pub score: f64,  // 0.0 to 100.0
    pub checks: Vec<HealthCheck>,
    pub uptime: Duration,
    pub last_updated: Instant,
}

pub struct HealthChecker {
    checks: HashMap<String, Box<dyn HealthCheckTrait + Send + Sync>>,
    interval: Duration,
    timeout: Duration,
}

#[async_trait::async_trait]
pub trait HealthCheckTrait {
    async fn check(&self) -> HealthCheck;
    fn name(&self) -> &str;
    fn timeout(&self) -> Duration;
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
        }
    }
    
    pub fn add_check(&mut self, check: Box<dyn HealthCheckTrait + Send + Sync>) {
        self.checks.insert(check.name().to_string(), check);
    }
    
    pub async fn run_all_checks(&self) -> OverallHealth {
        let start_time = Instant::now();
        let mut results = Vec::new();
        
        for (name, check) in &self.checks {
            let check_start = Instant::now();
            
            // Run check with timeout
            let result = tokio::time::timeout(check.timeout(), check.check()).await;
            
            let health_check = match result {
                Ok(check_result) => check_result,
                Err(_) => HealthCheck {
                    name: name.clone(),
                    status: HealthStatus::Critical,
                    last_check: check_start,
                    duration: check.timeout(),
                    message: Some("Health check timed out".to_string()),
                    details: HashMap::new(),
                },
            };
            
            results.push(health_check);
        }
        
        let overall_status = self.calculate_overall_status(&results);
        let score = self.calculate_health_score(&results);
        
        OverallHealth {
            status: overall_status,
            score,
            checks: results,
            uptime: start_time.elapsed(),
            last_updated: Instant::now(),
        }
    }
    
    fn calculate_overall_status(&self, checks: &[HealthCheck]) -> HealthStatus {
        if checks.iter().any(|c| matches!(c.status, HealthStatus::Critical)) {
            HealthStatus::Critical
        } else if checks.iter().any(|c| matches!(c.status, HealthStatus::Warning)) {
            HealthStatus::Warning
        } else if checks.iter().all(|c| matches!(c.status, HealthStatus::Healthy)) {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }
    
    fn calculate_health_score(&self, checks: &[HealthCheck]) -> f64 {
        if checks.is_empty() {
            return 0.0;
        }
        
        let total_score: f64 = checks.iter().map(|check| {
            match check.status {
                HealthStatus::Healthy => 100.0,
                HealthStatus::Warning => 60.0,
                HealthStatus::Critical => 0.0,
                HealthStatus::Unknown => 30.0,
            }
        }).sum();
        
        total_score / checks.len() as f64
    }
}

// Individual health checks
pub struct DatabaseHealthCheck {
    pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl HealthCheckTrait for DatabaseHealthCheck {
    async fn check(&self) -> HealthCheck {
        let start = Instant::now();
        let mut details = HashMap::new();
        
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => {
                // Get connection pool stats
                details.insert("pool_size".to_string(), 
                             serde_json::Value::Number(self.pool.size().into()));
                details.insert("idle_connections".to_string(), 
                             serde_json::Value::Number(self.pool.num_idle().into()));
                
                HealthCheck {
                    name: "database".to_string(),
                    status: HealthStatus::Healthy,
                    last_check: start,
                    duration: start.elapsed(),
                    message: Some("Database connection successful".to_string()),
                    details,
                }
            }
            Err(e) => HealthCheck {
                name: "database".to_string(),
                status: HealthStatus::Critical,
                last_check: start,
                duration: start.elapsed(),
                message: Some(format!("Database connection failed: {}", e)),
                details,
            }
        }
    }
    
    fn name(&self) -> &str {
        "database"
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

pub struct WebSocketHealthCheck {
    active_connections: Arc<RwLock<u32>>,
}

#[async_trait::async_trait]
impl HealthCheckTrait for WebSocketHealthCheck {
    async fn check(&self) -> HealthCheck {
        let start = Instant::now();
        let connections = *self.active_connections.read().await;
        let mut details = HashMap::new();
        
        details.insert("active_connections".to_string(), 
                     serde_json::Value::Number(connections.into()));
        
        let status = if connections > 1000 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
        
        HealthCheck {
            name: "websocket".to_string(),
            status,
            last_check: start,
            duration: start.elapsed(),
            message: Some(format!("{} active WebSocket connections", connections)),
            details,
        }
    }
    
    fn name(&self) -> &str {
        "websocket"
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(2)
    }
}

pub struct PhysicsHealthCheck {
    physics_manager: Arc<PhysicsManager>,
}

#[async_trait::async_trait]
impl HealthCheckTrait for PhysicsHealthCheck {
    async fn check(&self) -> HealthCheck {
        let start = Instant::now();
        let mut details = HashMap::new();
        
        let active_engines = self.physics_manager.get_active_engines().await;
        let total_bodies = self.physics_manager.get_total_body_count().await;
        let average_fps = self.physics_manager.get_average_fps().await;
        
        details.insert("active_engines".to_string(), 
                     serde_json::Value::Number(active_engines.into()));
        details.insert("total_bodies".to_string(), 
                     serde_json::Value::Number(total_bodies.into()));
        details.insert("average_fps".to_string(), 
                     serde_json::Value::Number(serde_json::Number::from_f64(average_fps).unwrap()));
        
        let status = if average_fps < 30.0 {
            HealthStatus::Warning
        } else if average_fps < 15.0 {
            HealthStatus::Critical
        } else {
            HealthStatus::Healthy
        };
        
        HealthCheck {
            name: "physics".to_string(),
            status,
            last_check: start,
            duration: start.elapsed(),
            message: Some(format!("Physics simulation at {:.1} FPS", average_fps)),
            details,
        }
    }
    
    fn name(&self) -> &str {
        "physics"
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(3)
    }
}
```

### AlertManager Configuration

Configure AlertManager for routing alerts:

```yaml
# alertmanager.yml
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alerts@opensim-next.org'
  slack_api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'

route:
  group_by: ['alertname', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 12h
  receiver: 'web.hook'
  routes:
  - match:
      severity: critical
    receiver: 'critical-alerts'
    group_wait: 10s
    repeat_interval: 1m
  - match:
      service: opensim-next
    receiver: 'opensim-alerts'
  - match:
      category: security
    receiver: 'security-alerts'
    group_wait: 0s

receivers:
- name: 'web.hook'
  webhook_configs:
  - url: 'http://localhost:8080/webhook'
    send_resolved: true

- name: 'critical-alerts'
  email_configs:
  - to: 'oncall@opensim-next.org'
    subject: 'CRITICAL: OpenSim Next Alert'
    body: |
      {{ range .Alerts }}
      Alert: {{ .Annotations.summary }}
      Description: {{ .Annotations.description }}
      Labels: {{ range .Labels.SortedPairs }}{{ .Name }}={{ .Value }} {{ end }}
      {{ end }}
  slack_configs:
  - channel: '#critical-alerts'
    title: 'CRITICAL: OpenSim Next Alert'
    text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
    color: 'danger'
  pagerduty_configs:
  - service_key: 'YOUR_PAGERDUTY_SERVICE_KEY'
    description: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'

- name: 'opensim-alerts'
  slack_configs:
  - channel: '#opensim-monitoring'
    title: 'OpenSim Next Alert'
    text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
    color: 'warning'

- name: 'security-alerts'
  email_configs:
  - to: 'security@opensim-next.org'
    subject: 'SECURITY: OpenSim Next Alert'
    body: |
      SECURITY ALERT
      {{ range .Alerts }}
      Alert: {{ .Annotations.summary }}
      Description: {{ .Annotations.description }}
      Time: {{ .StartsAt }}
      {{ end }}
  slack_configs:
  - channel: '#security-alerts'
    title: 'SECURITY: OpenSim Next Alert'
    text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
    color: 'danger'

inhibit_rules:
- source_match:
    severity: 'critical'
  target_match:
    severity: 'warning'
  equal: ['alertname', 'instance']
```

## Performance Monitoring

### Advanced Performance Metrics

Implement detailed performance tracking:

```rust
// performance/monitor.rs
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub network_io: NetworkIO,
    pub disk_io: DiskIO,
    pub application_metrics: ApplicationMetrics,
    pub region_metrics: HashMap<String, RegionMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub connections_active: u32,
    pub connections_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIO {
    pub reads_completed: u64,
    pub writes_completed: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub io_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub request_rate: f64,
    pub response_time_p50: Duration,
    pub response_time_p95: Duration,
    pub response_time_p99: Duration,
    pub error_rate: f64,
    pub active_users: u32,
    pub database_connections: u32,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionMetrics {
    pub region_id: String,
    pub region_name: String,
    pub fps: f64,
    pub physics_engine: String,
    pub active_objects: u32,
    pub active_avatars: u32,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub network_throughput: f64,
    pub script_events_per_second: f64,
}

pub struct PerformanceMonitor {
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    collection_interval: Duration,
    retention_period: Duration,
    thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub cpu_warning: f64,
    pub cpu_critical: f64,
    pub memory_warning: f64,
    pub memory_critical: f64,
    pub response_time_warning: Duration,
    pub response_time_critical: Duration,
    pub error_rate_warning: f64,
    pub error_rate_critical: f64,
    pub fps_warning: f64,
    pub fps_critical: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            cpu_warning: 70.0,
            cpu_critical: 90.0,
            memory_warning: 80.0,
            memory_critical: 95.0,
            response_time_warning: Duration::from_millis(500),
            response_time_critical: Duration::from_millis(2000),
            error_rate_warning: 1.0,
            error_rate_critical: 5.0,
            fps_warning: 45.0,
            fps_critical: 30.0,
        }
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            collection_interval: Duration::from_secs(5),
            retention_period: Duration::from_secs(24 * 60 * 60), // 24 hours
            thresholds: PerformanceThresholds::default(),
        }
    }
    
    pub async fn start_monitoring(&self) {
        let metrics_history = Arc::clone(&self.metrics_history);
        let collection_interval = self.collection_interval;
        let retention_period = self.retention_period;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(collection_interval);
            
            loop {
                interval.tick().await;
                
                let metrics = Self::collect_metrics().await;
                
                {
                    let mut history = metrics_history.write().await;
                    history.push(metrics);
                    
                    // Clean up old metrics
                    let cutoff_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() - retention_period.as_secs();
                    
                    history.retain(|m| m.timestamp > cutoff_time);
                }
            }
        });
    }
    
    async fn collect_metrics() -> PerformanceMetrics {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Collect system metrics
        let cpu_usage = Self::get_cpu_usage().await;
        let memory_usage = Self::get_memory_usage().await;
        let network_io = Self::get_network_io().await;
        let disk_io = Self::get_disk_io().await;
        
        // Collect application metrics
        let application_metrics = Self::get_application_metrics().await;
        
        // Collect region-specific metrics
        let region_metrics = Self::get_region_metrics().await;
        
        PerformanceMetrics {
            timestamp,
            cpu_usage,
            memory_usage,
            network_io,
            disk_io,
            application_metrics,
            region_metrics,
        }
    }
    
    async fn get_cpu_usage() -> f64 {
        // Implementation would use system APIs or /proc/stat on Linux
        // This is a simplified example
        use sysinfo::{System, SystemExt};
        let mut system = System::new_all();
        system.refresh_cpu();
        
        // Wait a bit to get accurate CPU usage
        tokio::time::sleep(Duration::from_millis(200)).await;
        system.refresh_cpu();
        
        system.global_cpu_info().cpu_usage() as f64
    }
    
    async fn get_memory_usage() -> u64 {
        use sysinfo::{System, SystemExt};
        let mut system = System::new_all();
        system.refresh_memory();
        
        system.used_memory()
    }
    
    async fn get_network_io() -> NetworkIO {
        // Implementation would collect network statistics
        // This is a placeholder
        NetworkIO {
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            connections_active: 0,
            connections_total: 0,
        }
    }
    
    async fn get_disk_io() -> DiskIO {
        // Implementation would collect disk I/O statistics
        DiskIO {
            reads_completed: 0,
            writes_completed: 0,
            bytes_read: 0,
            bytes_written: 0,
            io_time_ms: 0,
        }
    }
    
    async fn get_application_metrics() -> ApplicationMetrics {
        // Collect metrics from application
        ApplicationMetrics {
            request_rate: 0.0,
            response_time_p50: Duration::from_millis(50),
            response_time_p95: Duration::from_millis(200),
            response_time_p99: Duration::from_millis(500),
            error_rate: 0.1,
            active_users: 0,
            database_connections: 0,
            cache_hit_rate: 0.95,
        }
    }
    
    async fn get_region_metrics() -> HashMap<String, RegionMetrics> {
        // Collect per-region metrics
        HashMap::new()
    }
    
    pub async fn get_latest_metrics(&self) -> Option<PerformanceMetrics> {
        let history = self.metrics_history.read().await;
        history.last().cloned()
    }
    
    pub async fn get_metrics_history(&self, duration: Duration) -> Vec<PerformanceMetrics> {
        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() - duration.as_secs();
        
        let history = self.metrics_history.read().await;
        history.iter()
            .filter(|m| m.timestamp > cutoff_time)
            .cloned()
            .collect()
    }
    
    pub async fn analyze_performance_trends(&self) -> PerformanceAnalysis {
        let history = self.get_metrics_history(Duration::from_secs(3600)).await; // Last hour
        
        if history.is_empty() {
            return PerformanceAnalysis::default();
        }
        
        let cpu_trend = self.calculate_trend(history.iter().map(|m| m.cpu_usage).collect());
        let memory_trend = self.calculate_trend(history.iter().map(|m| m.memory_usage as f64).collect());
        let response_time_trend = self.calculate_trend(
            history.iter().map(|m| m.application_metrics.response_time_p95.as_millis() as f64).collect()
        );
        
        PerformanceAnalysis {
            cpu_trend,
            memory_trend,
            response_time_trend,
            recommendations: self.generate_recommendations(&history).await,
        }
    }
    
    fn calculate_trend(&self, values: Vec<f64>) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }
        
        let first_half: f64 = values.iter().take(values.len() / 2).sum::<f64>() / (values.len() / 2) as f64;
        let second_half: f64 = values.iter().skip(values.len() / 2).sum::<f64>() / (values.len() / 2) as f64;
        
        let change_percent = ((second_half - first_half) / first_half) * 100.0;
        
        if change_percent > 10.0 {
            TrendDirection::Increasing
        } else if change_percent < -10.0 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }
    
    async fn generate_recommendations(&self, history: &[PerformanceMetrics]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if let Some(latest) = history.last() {
            if latest.cpu_usage > self.thresholds.cpu_warning {
                recommendations.push("Consider scaling horizontally or optimizing CPU-intensive processes".to_string());
            }
            
            if latest.memory_usage as f64 > self.thresholds.memory_warning {
                recommendations.push("Memory usage is high. Consider increasing memory or optimizing memory usage".to_string());
            }
            
            if latest.application_metrics.response_time_p95 > self.thresholds.response_time_warning {
                recommendations.push("Response times are elevated. Check database performance and optimize queries".to_string());
            }
            
            if latest.application_metrics.error_rate > self.thresholds.error_rate_warning {
                recommendations.push("Error rate is above normal. Investigate recent deployments and logs".to_string());
            }
            
            // Region-specific recommendations
            for (region_id, region_metrics) in &latest.region_metrics {
                if region_metrics.fps < self.thresholds.fps_warning {
                    recommendations.push(format!("Region {} has low FPS ({}). Consider switching physics engine or reducing object count", region_id, region_metrics.fps));
                }
            }
        }
        
        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub cpu_trend: TrendDirection,
    pub memory_trend: TrendDirection,
    pub response_time_trend: TrendDirection,
    pub recommendations: Vec<String>,
}

impl Default for PerformanceAnalysis {
    fn default() -> Self {
        Self {
            cpu_trend: TrendDirection::Stable,
            memory_trend: TrendDirection::Stable,
            response_time_trend: TrendDirection::Stable,
            recommendations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}
```

## Administration Dashboard

### Web-Based Administration Interface

Create a comprehensive admin dashboard:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSim Next - Administration Dashboard</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/3.9.1/chart.min.css">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f8fafc;
            color: #1e293b;
        }
        
        .admin-header {
            background: #1e40af;
            color: white;
            padding: 1rem 2rem;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        
        .admin-nav {
            display: flex;
            align-items: center;
            justify-content: space-between;
        }
        
        .admin-title {
            font-size: 1.5rem;
            font-weight: 600;
        }
        
        .admin-user {
            display: flex;
            align-items: center;
            gap: 1rem;
        }
        
        .admin-main {
            display: grid;
            grid-template-columns: 250px 1fr;
            min-height: calc(100vh - 70px);
        }
        
        .admin-sidebar {
            background: white;
            border-right: 1px solid #e2e8f0;
            padding: 2rem 0;
        }
        
        .sidebar-nav {
            list-style: none;
        }
        
        .sidebar-nav li {
            margin: 0.25rem 0;
        }
        
        .sidebar-nav a {
            display: block;
            padding: 0.75rem 2rem;
            color: #64748b;
            text-decoration: none;
            transition: all 0.2s;
        }
        
        .sidebar-nav a:hover,
        .sidebar-nav a.active {
            background: #eff6ff;
            color: #1e40af;
            border-right: 3px solid #1e40af;
        }
        
        .admin-content {
            padding: 2rem;
            overflow-y: auto;
        }
        
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 1.5rem;
            margin-bottom: 2rem;
        }
        
        .stat-card {
            background: white;
            border-radius: 8px;
            padding: 1.5rem;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
            border: 1px solid #e2e8f0;
        }
        
        .stat-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 1rem;
        }
        
        .stat-title {
            font-size: 0.875rem;
            font-weight: 500;
            color: #64748b;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }
        
        .stat-value {
            font-size: 2rem;
            font-weight: 700;
            color: #1e293b;
        }
        
        .stat-change {
            font-size: 0.875rem;
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
        }
        
        .stat-change.positive {
            background: #dcfce7;
            color: #166534;
        }
        
        .stat-change.negative {
            background: #fef2f2;
            color: #dc2626;
        }
        
        .chart-container {
            background: white;
            border-radius: 8px;
            padding: 1.5rem;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
            border: 1px solid #e2e8f0;
            margin-bottom: 2rem;
        }
        
        .chart-title {
            font-size: 1.125rem;
            font-weight: 600;
            margin-bottom: 1rem;
            color: #1e293b;
        }
        
        .alerts-section {
            background: white;
            border-radius: 8px;
            padding: 1.5rem;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
            border: 1px solid #e2e8f0;
        }
        
        .alert-item {
            display: flex;
            align-items: center;
            padding: 1rem;
            border-left: 4px solid #ef4444;
            background: #fef2f2;
            margin-bottom: 1rem;
            border-radius: 0 4px 4px 0;
        }
        
        .alert-item.warning {
            border-left-color: #f59e0b;
            background: #fffbeb;
        }
        
        .alert-item.info {
            border-left-color: #3b82f6;
            background: #eff6ff;
        }
        
        .alert-content {
            flex: 1;
        }
        
        .alert-title {
            font-weight: 600;
            margin-bottom: 0.25rem;
        }
        
        .alert-time {
            font-size: 0.875rem;
            color: #64748b;
        }
        
        .status-indicator {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            display: inline-block;
            margin-right: 0.5rem;
        }
        
        .status-healthy {
            background: #10b981;
        }
        
        .status-warning {
            background: #f59e0b;
        }
        
        .status-critical {
            background: #ef4444;
        }
        
        .btn {
            display: inline-flex;
            align-items: center;
            padding: 0.5rem 1rem;
            border-radius: 6px;
            font-weight: 500;
            text-decoration: none;
            border: none;
            cursor: pointer;
            transition: all 0.2s;
        }
        
        .btn-primary {
            background: #1e40af;
            color: white;
        }
        
        .btn-primary:hover {
            background: #1d4ed8;
        }
        
        .btn-secondary {
            background: #e2e8f0;
            color: #64748b;
        }
        
        .btn-secondary:hover {
            background: #cbd5e1;
        }
        
        @media (max-width: 768px) {
            .admin-main {
                grid-template-columns: 1fr;
            }
            
            .admin-sidebar {
                display: none;
            }
            
            .stats-grid {
                grid-template-columns: 1fr;
            }
        }
    </style>
</head>
<body>
    <header class="admin-header">
        <nav class="admin-nav">
            <h1 class="admin-title">OpenSim Next Administration</h1>
            <div class="admin-user">
                <span id="server-status">
                    <span class="status-indicator status-healthy"></span>
                    Server Healthy
                </span>
                <span>Admin User</span>
                <button class="btn btn-secondary" onclick="logout()">Logout</button>
            </div>
        </nav>
    </header>
    
    <main class="admin-main">
        <aside class="admin-sidebar">
            <nav>
                <ul class="sidebar-nav">
                    <li><a href="#dashboard" class="active">Dashboard</a></li>
                    <li><a href="#regions">Regions</a></li>
                    <li><a href="#users">Users</a></li>
                    <li><a href="#monitoring">Monitoring</a></li>
                    <li><a href="#performance">Performance</a></li>
                    <li><a href="#security">Security</a></li>
                    <li><a href="#assets">Assets</a></li>
                    <li><a href="#configuration">Configuration</a></li>
                    <li><a href="#logs">Logs</a></li>
                    <li><a href="#alerts">Alerts</a></li>
                </ul>
            </nav>
        </aside>
        
        <section class="admin-content">
            <!-- Dashboard Overview -->
            <div id="dashboard-section">
                <h2>System Overview</h2>
                
                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-header">
                            <span class="stat-title">Online Users</span>
                            <span class="stat-change positive">+12%</span>
                        </div>
                        <div class="stat-value" id="online-users">156</div>
                    </div>
                    
                    <div class="stat-card">
                        <div class="stat-header">
                            <span class="stat-title">Active Regions</span>
                            <span class="stat-change positive">+2</span>
                        </div>
                        <div class="stat-value" id="active-regions">8</div>
                    </div>
                    
                    <div class="stat-card">
                        <div class="stat-header">
                            <span class="stat-title">Server Uptime</span>
                            <span class="stat-change positive">99.9%</span>
                        </div>
                        <div class="stat-value" id="server-uptime">15d 6h</div>
                    </div>
                    
                    <div class="stat-card">
                        <div class="stat-header">
                            <span class="stat-title">Response Time</span>
                            <span class="stat-change negative">+5ms</span>
                        </div>
                        <div class="stat-value" id="response-time">42ms</div>
                    </div>
                    
                    <div class="stat-card">
                        <div class="stat-header">
                            <span class="stat-title">Memory Usage</span>
                            <span class="stat-change positive">-3%</span>
                        </div>
                        <div class="stat-value" id="memory-usage">68%</div>
                    </div>
                    
                    <div class="stat-card">
                        <div class="stat-header">
                            <span class="stat-title">CPU Usage</span>
                            <span class="stat-change positive">-8%</span>
                        </div>
                        <div class="stat-value" id="cpu-usage">45%</div>
                    </div>
                </div>
                
                <!-- Performance Charts -->
                <div class="chart-container">
                    <h3 class="chart-title">System Performance (Last 24 Hours)</h3>
                    <canvas id="performance-chart" width="400" height="200"></canvas>
                </div>
                
                <div class="chart-container">
                    <h3 class="chart-title">User Activity</h3>
                    <canvas id="user-activity-chart" width="400" height="200"></canvas>
                </div>
                
                <!-- Active Alerts -->
                <div class="alerts-section">
                    <h3>Active Alerts</h3>
                    <div id="alerts-container">
                        <div class="alert-item warning">
                            <div class="alert-content">
                                <div class="alert-title">High Memory Usage on Region 'Sandbox'</div>
                                <div class="alert-time">2 minutes ago</div>
                            </div>
                            <button class="btn btn-secondary">Acknowledge</button>
                        </div>
                        
                        <div class="alert-item info">
                            <div class="alert-content">
                                <div class="alert-title">New User Registration: JohnDoe</div>
                                <div class="alert-time">5 minutes ago</div>
                            </div>
                            <button class="btn btn-secondary">View</button>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    </main>
    
    <script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/3.9.1/chart.min.js"></script>
    <script>
        // Real-time dashboard functionality
        class AdminDashboard {
            constructor() {
                this.wsConnection = null;
                this.charts = {};
                this.updateInterval = 5000; // 5 seconds
                
                this.init();
            }
            
            async init() {
                await this.connectWebSocket();
                this.initializeCharts();
                this.startRealTimeUpdates();
                this.bindEvents();
            }
            
            async connectWebSocket() {
                try {
                    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                    const wsUrl = `${protocol}//${window.location.host}/admin/ws`;
                    
                    this.wsConnection = new WebSocket(wsUrl);
                    
                    this.wsConnection.onopen = () => {
                        console.log('Admin WebSocket connected');
                        this.updateServerStatus('healthy');
                    };
                    
                    this.wsConnection.onmessage = (event) => {
                        const data = JSON.parse(event.data);
                        this.handleRealtimeUpdate(data);
                    };
                    
                    this.wsConnection.onclose = () => {
                        console.log('Admin WebSocket disconnected');
                        this.updateServerStatus('disconnected');
                        // Attempt to reconnect
                        setTimeout(() => this.connectWebSocket(), 5000);
                    };
                    
                    this.wsConnection.onerror = (error) => {
                        console.error('WebSocket error:', error);
                        this.updateServerStatus('error');
                    };
                } catch (error) {
                    console.error('Failed to connect WebSocket:', error);
                }
            }
            
            initializeCharts() {
                // Performance Chart
                const perfCtx = document.getElementById('performance-chart').getContext('2d');
                this.charts.performance = new Chart(perfCtx, {
                    type: 'line',
                    data: {
                        labels: [],
                        datasets: [
                            {
                                label: 'CPU Usage (%)',
                                data: [],
                                borderColor: '#3b82f6',
                                backgroundColor: '#3b82f6',
                                tension: 0.4,
                                fill: false
                            },
                            {
                                label: 'Memory Usage (%)',
                                data: [],
                                borderColor: '#10b981',
                                backgroundColor: '#10b981',
                                tension: 0.4,
                                fill: false
                            },
                            {
                                label: 'Response Time (ms)',
                                data: [],
                                borderColor: '#f59e0b',
                                backgroundColor: '#f59e0b',
                                tension: 0.4,
                                fill: false,
                                yAxisID: 'y1'
                            }
                        ]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        scales: {
                            y: {
                                type: 'linear',
                                display: true,
                                position: 'left',
                                max: 100
                            },
                            y1: {
                                type: 'linear',
                                display: true,
                                position: 'right',
                                grid: {
                                    drawOnChartArea: false,
                                }
                            }
                        },
                        plugins: {
                            legend: {
                                display: true
                            }
                        }
                    }
                });
                
                // User Activity Chart
                const userCtx = document.getElementById('user-activity-chart').getContext('2d');
                this.charts.userActivity = new Chart(userCtx, {
                    type: 'bar',
                    data: {
                        labels: ['00:00', '04:00', '08:00', '12:00', '16:00', '20:00'],
                        datasets: [
                            {
                                label: 'Active Users',
                                data: [45, 32, 78, 123, 156, 134],
                                backgroundColor: '#1e40af',
                            },
                            {
                                label: 'WebSocket Connections',
                                data: [12, 8, 23, 45, 67, 52],
                                backgroundColor: '#7c3aed',
                            }
                        ]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        scales: {
                            y: {
                                beginAtZero: true
                            }
                        }
                    }
                });
            }
            
            startRealTimeUpdates() {
                setInterval(async () => {
                    await this.updateDashboardStats();
                }, this.updateInterval);
            }
            
            async updateDashboardStats() {
                try {
                    const response = await fetch('/admin/api/stats', {
                        headers: {
                            'Authorization': `Bearer ${localStorage.getItem('admin_token')}`
                        }
                    });
                    
                    if (response.ok) {
                        const stats = await response.json();
                        this.updateStatsCards(stats);
                        this.updateCharts(stats);
                    }
                } catch (error) {
                    console.error('Failed to update dashboard stats:', error);
                }
            }
            
            updateStatsCards(stats) {
                document.getElementById('online-users').textContent = stats.online_users;
                document.getElementById('active-regions').textContent = stats.active_regions;
                document.getElementById('server-uptime').textContent = this.formatUptime(stats.uptime_seconds);
                document.getElementById('response-time').textContent = `${stats.avg_response_time}ms`;
                document.getElementById('memory-usage').textContent = `${Math.round(stats.memory_usage_percent)}%`;
                document.getElementById('cpu-usage').textContent = `${Math.round(stats.cpu_usage_percent)}%`;
            }
            
            updateCharts(stats) {
                // Update performance chart
                const perfChart = this.charts.performance;
                const now = new Date().toLocaleTimeString();
                
                perfChart.data.labels.push(now);
                perfChart.data.datasets[0].data.push(stats.cpu_usage_percent);
                perfChart.data.datasets[1].data.push(stats.memory_usage_percent);
                perfChart.data.datasets[2].data.push(stats.avg_response_time);
                
                // Keep only last 20 data points
                if (perfChart.data.labels.length > 20) {
                    perfChart.data.labels.shift();
                    perfChart.data.datasets.forEach(dataset => dataset.data.shift());
                }
                
                perfChart.update('none');
            }
            
            handleRealtimeUpdate(data) {
                switch (data.type) {
                    case 'alert':
                        this.addAlert(data.alert);
                        break;
                    case 'user_login':
                        this.updateUserActivity(data);
                        break;
                    case 'performance_update':
                        this.updatePerformanceMetrics(data.metrics);
                        break;
                    case 'region_status':
                        this.updateRegionStatus(data.region);
                        break;
                    default:
                        console.log('Unknown realtime update:', data);
                }
            }
            
            addAlert(alert) {
                const alertsContainer = document.getElementById('alerts-container');
                const alertElement = document.createElement('div');
                alertElement.className = `alert-item ${alert.severity}`;
                alertElement.innerHTML = `
                    <div class="alert-content">
                        <div class="alert-title">${alert.title}</div>
                        <div class="alert-time">Just now</div>
                    </div>
                    <button class="btn btn-secondary" onclick="acknowledgeAlert('${alert.id}')">Acknowledge</button>
                `;
                
                alertsContainer.insertBefore(alertElement, alertsContainer.firstChild);
            }
            
            updateServerStatus(status) {
                const statusElement = document.getElementById('server-status');
                const indicator = statusElement.querySelector('.status-indicator');
                
                indicator.className = 'status-indicator';
                
                switch (status) {
                    case 'healthy':
                        indicator.classList.add('status-healthy');
                        statusElement.innerHTML = `<span class="status-indicator status-healthy"></span> Server Healthy`;
                        break;
                    case 'warning':
                        indicator.classList.add('status-warning');
                        statusElement.innerHTML = `<span class="status-indicator status-warning"></span> Server Warning`;
                        break;
                    case 'critical':
                        indicator.classList.add('status-critical');
                        statusElement.innerHTML = `<span class="status-indicator status-critical"></span> Server Critical`;
                        break;
                    case 'disconnected':
                        indicator.classList.add('status-critical');
                        statusElement.innerHTML = `<span class="status-indicator status-critical"></span> Disconnected`;
                        break;
                }
            }
            
            formatUptime(seconds) {
                const days = Math.floor(seconds / (24 * 3600));
                const hours = Math.floor((seconds % (24 * 3600)) / 3600);
                
                if (days > 0) {
                    return `${days}d ${hours}h`;
                } else {
                    return `${hours}h ${Math.floor((seconds % 3600) / 60)}m`;
                }
            }
            
            bindEvents() {
                // Navigation
                document.querySelectorAll('.sidebar-nav a').forEach(link => {
                    link.addEventListener('click', (e) => {
                        e.preventDefault();
                        this.navigateTo(link.getAttribute('href').substring(1));
                    });
                });
            }
            
            navigateTo(section) {
                // Update active navigation
                document.querySelectorAll('.sidebar-nav a').forEach(link => {
                    link.classList.remove('active');
                });
                document.querySelector(`[href="#${section}"]`).classList.add('active');
                
                // Load section content
                this.loadSectionContent(section);
            }
            
            async loadSectionContent(section) {
                // This would load different admin sections
                console.log(`Loading section: ${section}`);
                
                // Example: Load different content based on section
                switch (section) {
                    case 'regions':
                        await this.loadRegionsSection();
                        break;
                    case 'users':
                        await this.loadUsersSection();
                        break;
                    case 'monitoring':
                        await this.loadMonitoringSection();
                        break;
                    // Add more sections as needed
                }
            }
            
            async loadRegionsSection() {
                // Implementation for regions management
                console.log('Loading regions section...');
            }
            
            async loadUsersSection() {
                // Implementation for user management
                console.log('Loading users section...');
            }
            
            async loadMonitoringSection() {
                // Implementation for monitoring section
                console.log('Loading monitoring section...');
            }
        }
        
        // Global functions
        function acknowledgeAlert(alertId) {
            fetch(`/admin/api/alerts/${alertId}/acknowledge`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('admin_token')}`
                }
            }).then(() => {
                // Remove alert from UI
                event.target.parentElement.remove();
            });
        }
        
        function logout() {
            localStorage.removeItem('admin_token');
            window.location.href = '/admin/login';
        }
        
        // Initialize dashboard when page loads
        document.addEventListener('DOMContentLoaded', () => {
            new AdminDashboard();
        });
    </script>
</body>
</html>
```

## Real-Time Statistics

### WebSocket-Based Real-Time Monitoring

Implement real-time statistics broadcasting system:

```rust
// realtime/stats_broadcaster.rs
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeStats {
    pub timestamp: u64,
    pub server_stats: ServerStats,
    pub region_stats: HashMap<String, RegionStats>,
    pub user_stats: UserStats,
    pub performance_stats: PerformanceStats,
    pub security_stats: SecurityStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub uptime_seconds: u64,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub active_connections: u32,
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub errors_per_minute: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionStats {
    pub region_id: String,
    pub region_name: String,
    pub online_users: u32,
    pub fps: f64,
    pub physics_engine: String,
    pub active_objects: u32,
    pub memory_usage_mb: f64,
    pub scripts_running: u32,
    pub events_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub total_users: u32,
    pub online_users: u32,
    pub new_users_today: u32,
    pub active_sessions: u32,
    pub average_session_duration: f64,
    pub concurrent_logins_per_hour: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub database_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub throughput_requests_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub failed_login_attempts: u32,
    pub blocked_ips: u32,
    pub security_events_per_hour: u32,
    pub zero_trust_violations: u32,
    pub encrypted_connections: u32,
}

pub struct StatsBroadcaster {
    clients: Arc<RwLock<HashMap<String, WebSocketSender>>>,
    stats_sender: broadcast::Sender<RealTimeStats>,
    collection_interval: Duration,
}

type WebSocketSender = tokio::sync::mpsc::UnboundedSender<Message>;

impl StatsBroadcaster {
    pub fn new() -> Self {
        let (stats_sender, _) = broadcast::channel(1000);
        
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            stats_sender,
            collection_interval: Duration::from_secs(1),
        }
    }
    
    pub async fn start(&self) {
        let clients = Arc::clone(&self.clients);
        let stats_sender = self.stats_sender.clone();
        let collection_interval = self.collection_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(collection_interval);
            
            loop {
                interval.tick().await;
                
                let stats = Self::collect_realtime_stats().await;
                
                // Broadcast to all connected clients
                let client_list: Vec<_> = {
                    clients.read().await.values().cloned().collect()
                };
                
                let stats_json = serde_json::to_string(&stats)
                    .unwrap_or_else(|_| "{}".to_string());
                let message = Message::Text(stats_json);
                
                for client in client_list {
                    let _ = client.send(message.clone());
                }
                
                // Also send to broadcast channel for other subscribers
                let _ = stats_sender.send(stats);
            }
        });
    }
    
    pub async fn add_client(&self, client_id: String, sender: WebSocketSender) {
        let mut clients = self.clients.write().await;
        clients.insert(client_id, sender);
    }
    
    pub async fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<RealTimeStats> {
        self.stats_sender.subscribe()
    }
    
    async fn collect_realtime_stats() -> RealTimeStats {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        RealTimeStats {
            timestamp,
            server_stats: Self::collect_server_stats().await,
            region_stats: Self::collect_region_stats().await,
            user_stats: Self::collect_user_stats().await,
            performance_stats: Self::collect_performance_stats().await,
            security_stats: Self::collect_security_stats().await,
        }
    }
    
    async fn collect_server_stats() -> ServerStats {
        // Implementation would collect actual server statistics
        ServerStats {
            uptime_seconds: 0,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            active_connections: 0,
            total_requests: 0,
            requests_per_second: 0.0,
            errors_per_minute: 0.0,
        }
    }
    
    async fn collect_region_stats() -> HashMap<String, RegionStats> {
        // Implementation would collect per-region statistics
        HashMap::new()
    }
    
    async fn collect_user_stats() -> UserStats {
        // Implementation would collect user statistics
        UserStats {
            total_users: 0,
            online_users: 0,
            new_users_today: 0,
            active_sessions: 0,
            average_session_duration: 0.0,
            concurrent_logins_per_hour: vec![0; 24],
        }
    }
    
    async fn collect_performance_stats() -> PerformanceStats {
        // Implementation would collect performance metrics
        PerformanceStats {
            avg_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            database_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            throughput_requests_per_second: 0.0,
        }
    }
    
    async fn collect_security_stats() -> SecurityStats {
        // Implementation would collect security metrics
        SecurityStats {
            failed_login_attempts: 0,
            blocked_ips: 0,
            security_events_per_hour: 0,
            zero_trust_violations: 0,
            encrypted_connections: 0,
        }
    }
}
```

### Real-Time Dashboard Integration

JavaScript client for real-time statistics:

```javascript
// admin/js/realtime-stats.js
class RealTimeStatsClient {
    constructor(dashboardContainer) {
        this.container = dashboardContainer;
        this.wsConnection = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectInterval = 5000;
        this.statsHistory = [];
        this.maxHistorySize = 300; // 5 minutes at 1-second intervals
        
        this.init();
    }
    
    async init() {
        await this.connectWebSocket();
        this.setupUI();
    }
    
    async connectWebSocket() {
        try {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${window.location.host}/admin/realtime`;
            
            this.wsConnection = new WebSocket(wsUrl);
            
            this.wsConnection.onopen = () => {
                console.log('Real-time stats WebSocket connected');
                this.reconnectAttempts = 0;
                this.updateConnectionStatus(true);
            };
            
            this.wsConnection.onmessage = (event) => {
                try {
                    const stats = JSON.parse(event.data);
                    this.handleStatsUpdate(stats);
                } catch (error) {
                    console.error('Failed to parse stats data:', error);
                }
            };
            
            this.wsConnection.onclose = () => {
                console.log('Real-time stats WebSocket disconnected');
                this.updateConnectionStatus(false);
                this.attemptReconnect();
            };
            
            this.wsConnection.onerror = (error) => {
                console.error('WebSocket error:', error);
                this.updateConnectionStatus(false);
            };
        } catch (error) {
            console.error('Failed to connect to real-time stats:', error);
            this.attemptReconnect();
        }
    }
    
    attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
            
            setTimeout(() => {
                this.connectWebSocket();
            }, this.reconnectInterval * this.reconnectAttempts);
        } else {
            console.error('Max reconnection attempts reached');
            this.showReconnectionError();
        }
    }
    
    handleStatsUpdate(stats) {
        // Add to history
        this.statsHistory.push(stats);
        if (this.statsHistory.length > this.maxHistorySize) {
            this.statsHistory.shift();
        }
        
        // Update UI components
        this.updateServerStats(stats.server_stats);
        this.updateRegionStats(stats.region_stats);
        this.updateUserStats(stats.user_stats);
        this.updatePerformanceStats(stats.performance_stats);
        this.updateSecurityStats(stats.security_stats);
        this.updateCharts();
    }
    
    setupUI() {
        this.container.innerHTML = `
            <div class="realtime-dashboard">
                <div class="connection-status" id="connection-status">
                    <span class="status-indicator"></span>
                    <span class="status-text">Connecting...</span>
                </div>
                
                <div class="stats-grid">
                    <div class="stat-card server-stats">
                        <h3>Server Performance</h3>
                        <div class="stat-item">
                            <span class="label">CPU Usage:</span>
                            <span class="value" id="cpu-usage">--</span>
                        </div>
                        <div class="stat-item">
                            <span class="label">Memory:</span>
                            <span class="value" id="memory-usage">--</span>
                        </div>
                        <div class="stat-item">
                            <span class="label">Connections:</span>
                            <span class="value" id="active-connections">--</span>
                        </div>
                        <div class="stat-item">
                            <span class="label">Requests/sec:</span>
                            <span class="value" id="requests-per-sec">--</span>
                        </div>
                    </div>
                    
                    <div class="stat-card user-stats">
                        <h3>User Activity</h3>
                        <div class="stat-item">
                            <span class="label">Online Users:</span>
                            <span class="value" id="online-users">--</span>
                        </div>
                        <div class="stat-item">
                            <span class="label">Active Sessions:</span>
                            <span class="value" id="active-sessions">--</span>
                        </div>
                        <div class="stat-item">
                            <span class="label">New Today:</span>
                            <span class="value" id="new-users-today">--</span>
                        </div>
                    </div>
                    
                    <div class="stat-card performance-stats">
                        <h3>Performance Metrics</h3>
                        <div class="stat-item">
                            <span class="label">Avg Response:</span>
                            <span class="value" id="avg-response-time">--</span>
                        </div>
                        <div class="stat-item">
                            <span class="label">P95 Response:</span>
                            <span class="value" id="p95-response-time">--</span>
                        </div>
                        <div class="stat-item">
                            <span class="label">Cache Hit Rate:</span>
                            <span class="value" id="cache-hit-rate">--</span>
                        </div>
                    </div>
                    
                    <div class="stat-card security-stats">
                        <h3>Security Status</h3>
                        <div class="stat-item">
                            <span class="label">Failed Logins:</span>
                            <span class="value" id="failed-logins">--</span>
                        </div>
                        <div class="stat-item">
                            <span class="label">Blocked IPs:</span>
                            <span class="value" id="blocked-ips">--</span>
                        </div>
                        <div class="stat-item">
                            <span class="label">ZT Violations:</span>
                            <span class="value" id="zt-violations">--</span>
                        </div>
                    </div>
                </div>
                
                <div class="realtime-charts">
                    <div class="chart-container">
                        <h3>Real-Time Performance</h3>
                        <canvas id="realtime-performance-chart"></canvas>
                    </div>
                    
                    <div class="chart-container">
                        <h3>User Activity</h3>
                        <canvas id="realtime-user-chart"></canvas>
                    </div>
                </div>
                
                <div class="region-stats" id="region-stats">
                    <h3>Region Status</h3>
                    <div class="region-grid" id="region-grid">
                        <!-- Dynamic region cards will be added here -->
                    </div>
                </div>
            </div>
        `;
        
        this.initializeCharts();
    }
    
    initializeCharts() {
        // Real-time performance chart
        const perfCtx = document.getElementById('realtime-performance-chart').getContext('2d');
        this.performanceChart = new Chart(perfCtx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'CPU %',
                        data: [],
                        borderColor: '#ef4444',
                        backgroundColor: 'rgba(239, 68, 68, 0.1)',
                        tension: 0.4,
                        fill: true
                    },
                    {
                        label: 'Memory %',
                        data: [],
                        borderColor: '#3b82f6',
                        backgroundColor: 'rgba(59, 130, 246, 0.1)',
                        tension: 0.4,
                        fill: true
                    },
                    {
                        label: 'Response Time (ms)',
                        data: [],
                        borderColor: '#10b981',
                        backgroundColor: 'rgba(16, 185, 129, 0.1)',
                        tension: 0.4,
                        fill: false,
                        yAxisID: 'y1'
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                scales: {
                    x: {
                        type: 'linear',
                        position: 'bottom',
                        max: 60,
                        min: 0
                    },
                    y: {
                        type: 'linear',
                        display: true,
                        position: 'left',
                        max: 100,
                        min: 0
                    },
                    y1: {
                        type: 'linear',
                        display: true,
                        position: 'right',
                        grid: {
                            drawOnChartArea: false,
                        }
                    }
                },
                plugins: {
                    legend: {
                        display: true,
                        position: 'top'
                    }
                }
            }
        });
        
        // User activity chart
        const userCtx = document.getElementById('realtime-user-chart').getContext('2d');
        this.userChart = new Chart(userCtx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'Online Users',
                        data: [],
                        borderColor: '#8b5cf6',
                        backgroundColor: 'rgba(139, 92, 246, 0.1)',
                        tension: 0.4,
                        fill: true
                    },
                    {
                        label: 'Active Sessions',
                        data: [],
                        borderColor: '#f59e0b',
                        backgroundColor: 'rgba(245, 158, 11, 0.1)',
                        tension: 0.4,
                        fill: true
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                scales: {
                    x: {
                        type: 'linear',
                        position: 'bottom',
                        max: 60,
                        min: 0
                    },
                    y: {
                        beginAtZero: true
                    }
                }
            }
        });
    }
    
    updateServerStats(serverStats) {
        document.getElementById('cpu-usage').textContent = `${serverStats.cpu_usage_percent.toFixed(1)}%`;
        document.getElementById('memory-usage').textContent = this.formatBytes(serverStats.memory_usage_bytes);
        document.getElementById('active-connections').textContent = serverStats.active_connections.toLocaleString();
        document.getElementById('requests-per-sec').textContent = serverStats.requests_per_second.toFixed(1);
    }
    
    updateUserStats(userStats) {
        document.getElementById('online-users').textContent = userStats.online_users.toLocaleString();
        document.getElementById('active-sessions').textContent = userStats.active_sessions.toLocaleString();
        document.getElementById('new-users-today').textContent = userStats.new_users_today.toLocaleString();
    }
    
    updatePerformanceStats(performanceStats) {
        document.getElementById('avg-response-time').textContent = `${performanceStats.avg_response_time_ms.toFixed(1)}ms`;
        document.getElementById('p95-response-time').textContent = `${performanceStats.p95_response_time_ms.toFixed(1)}ms`;
        document.getElementById('cache-hit-rate').textContent = `${(performanceStats.cache_hit_rate * 100).toFixed(1)}%`;
    }
    
    updateSecurityStats(securityStats) {
        document.getElementById('failed-logins').textContent = securityStats.failed_login_attempts.toLocaleString();
        document.getElementById('blocked-ips').textContent = securityStats.blocked_ips.toLocaleString();
        document.getElementById('zt-violations').textContent = securityStats.zero_trust_violations.toLocaleString();
    }
    
    updateRegionStats(regionStats) {
        const regionGrid = document.getElementById('region-grid');
        regionGrid.innerHTML = '';
        
        Object.values(regionStats).forEach(region => {
            const regionCard = document.createElement('div');
            regionCard.className = 'region-card';
            regionCard.innerHTML = `
                <div class="region-header">
                    <h4>${region.region_name}</h4>
                    <span class="region-fps ${region.fps < 30 ? 'warning' : 'healthy'}">${region.fps.toFixed(1)} FPS</span>
                </div>
                <div class="region-details">
                    <div class="region-stat">
                        <span class="label">Users:</span>
                        <span class="value">${region.online_users}</span>
                    </div>
                    <div class="region-stat">
                        <span class="label">Objects:</span>
                        <span class="value">${region.active_objects}</span>
                    </div>
                    <div class="region-stat">
                        <span class="label">Physics:</span>
                        <span class="value">${region.physics_engine}</span>
                    </div>
                    <div class="region-stat">
                        <span class="label">Memory:</span>
                        <span class="value">${region.memory_usage_mb.toFixed(1)}MB</span>
                    </div>
                </div>
            `;
            regionGrid.appendChild(regionCard);
        });
    }
    
    updateCharts() {
        if (this.statsHistory.length === 0) return;
        
        const recentStats = this.statsHistory.slice(-60); // Last 60 seconds
        const labels = recentStats.map((_, index) => index - recentStats.length + 1);
        
        // Update performance chart
        this.performanceChart.data.labels = labels;
        this.performanceChart.data.datasets[0].data = recentStats.map(s => s.server_stats.cpu_usage_percent);
        this.performanceChart.data.datasets[1].data = recentStats.map(s => (s.server_stats.memory_usage_bytes / (1024 * 1024 * 1024)) * 100); // GB to %
        this.performanceChart.data.datasets[2].data = recentStats.map(s => s.performance_stats.avg_response_time_ms);
        this.performanceChart.update('none');
        
        // Update user chart
        this.userChart.data.labels = labels;
        this.userChart.data.datasets[0].data = recentStats.map(s => s.user_stats.online_users);
        this.userChart.data.datasets[1].data = recentStats.map(s => s.user_stats.active_sessions);
        this.userChart.update('none');
    }
    
    updateConnectionStatus(connected) {
        const statusElement = document.getElementById('connection-status');
        const indicator = statusElement.querySelector('.status-indicator');
        const text = statusElement.querySelector('.status-text');
        
        if (connected) {
            indicator.className = 'status-indicator connected';
            text.textContent = 'Real-time Connected';
        } else {
            indicator.className = 'status-indicator disconnected';
            text.textContent = 'Disconnected';
        }
    }
    
    showReconnectionError() {
        const statusElement = document.getElementById('connection-status');
        const text = statusElement.querySelector('.status-text');
        text.textContent = 'Connection Failed - Refresh Page';
    }
    
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }
}
```

## Database Monitoring

### PostgreSQL Monitoring Setup

Configure comprehensive database monitoring:

```yaml
# postgres_exporter configuration
# postgres_exporter.yml
web:
  listen-address: ":9187"
  telemetry-path: "/metrics"

log:
  level: info
  format: logfmt

data_source_name: "postgresql://postgres_exporter:password@localhost:5432/opensim?sslmode=disable"

queries:
  # OpenSim-specific queries
  opensim_user_count:
    query: "SELECT COUNT(*) as total_users FROM UserAccounts"
    metrics:
      - total_users:
          usage: "GAUGE"
          description: "Total number of registered users"

  opensim_online_users:
    query: "SELECT COUNT(DISTINCT UserID) as online_users FROM Presence WHERE RegionID != '00000000-0000-0000-0000-000000000000'"
    metrics:
      - online_users:
          usage: "GAUGE"
          description: "Number of users currently online"

  opensim_active_regions:
    query: "SELECT COUNT(*) as active_regions FROM regions WHERE active = true"
    metrics:
      - active_regions:
          usage: "GAUGE"
          description: "Number of active regions"

  opensim_asset_count:
    query: "SELECT COUNT(*) as total_assets FROM assets"
    metrics:
      - total_assets:
          usage: "GAUGE"
          description: "Total number of assets in database"

  opensim_inventory_items:
    query: "SELECT COUNT(*) as inventory_items FROM inventoryitems"
    metrics:
      - inventory_items:
          usage: "GAUGE"
          description: "Total number of inventory items"

  opensim_chat_messages_24h:
    query: "SELECT COUNT(*) as chat_messages FROM chat_logs WHERE timestamp > NOW() - INTERVAL '24 hours'"
    metrics:
      - chat_messages_24h:
          usage: "GAUGE"
          description: "Chat messages in last 24 hours"

  opensim_teleports_24h:
    query: "SELECT COUNT(*) as teleports FROM teleport_logs WHERE timestamp > NOW() - INTERVAL '24 hours'"
    metrics:
      - teleports_24h:
          usage: "GAUGE"
          description: "Teleports in last 24 hours"

  opensim_database_size:
    query: "SELECT pg_database_size('opensim') as database_size_bytes"
    metrics:
      - database_size_bytes:
          usage: "GAUGE"
          description: "OpenSim database size in bytes"

  opensim_connection_count:
    query: "SELECT count(*) as connections FROM pg_stat_activity WHERE datname = 'opensim'"
    metrics:
      - connections:
          usage: "GAUGE"
          description: "Number of active database connections"

  opensim_slow_queries:
    query: "SELECT COUNT(*) as slow_queries FROM pg_stat_statements WHERE mean_time > 1000"
    metrics:
      - slow_queries:
          usage: "GAUGE"
          description: "Number of queries taking more than 1 second"
```

### Database Performance Monitoring

SQL queries for database health monitoring:

```sql
-- Database health check queries
-- Save as: monitoring/sql/health_checks.sql

-- 1. Connection monitoring
SELECT 
    datname,
    numbackends as active_connections,
    xact_commit,
    xact_rollback,
    blks_read,
    blks_hit,
    tup_returned,
    tup_fetched,
    tup_inserted,
    tup_updated,
    tup_deleted
FROM pg_stat_database 
WHERE datname = 'opensim';

-- 2. Table statistics
SELECT 
    schemaname,
    tablename,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes,
    n_live_tup as live_tuples,
    n_dead_tup as dead_tuples,
    last_vacuum,
    last_autovacuum,
    last_analyze,
    last_autoanalyze
FROM pg_stat_user_tables
ORDER BY n_live_tup DESC
LIMIT 20;

-- 3. Index usage
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_tup_read,
    idx_tup_fetch,
    idx_blks_read,
    idx_blks_hit
FROM pg_stat_user_indexes
ORDER BY idx_tup_read DESC
LIMIT 20;

-- 4. Slow queries (requires pg_stat_statements)
SELECT 
    query,
    calls,
    total_time,
    mean_time,
    rows,
    100.0 * shared_blks_hit / nullif(shared_blks_hit + shared_blks_read, 0) AS hit_percent
FROM pg_stat_statements 
WHERE query NOT LIKE '%pg_stat_statements%'
ORDER BY mean_time DESC 
LIMIT 20;

-- 5. Lock monitoring
SELECT 
    pid,
    mode,
    locktype,
    database,
    relation::regclass,
    page,
    tuple,
    classid,
    granted
FROM pg_locks 
WHERE database = (SELECT oid FROM pg_database WHERE datname = 'opensim')
ORDER BY granted, mode;

-- 6. Blocking queries
SELECT 
    blocked_locks.pid AS blocked_pid,
    blocked_activity.usename AS blocked_user,
    blocking_locks.pid AS blocking_pid,
    blocking_activity.usename AS blocking_user,
    blocked_activity.query AS blocked_statement,
    blocking_activity.query AS current_statement_in_blocking_process
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
    AND blocking_locks.database IS NOT DISTINCT FROM blocked_locks.database
    AND blocking_locks.relation IS NOT DISTINCT FROM blocked_locks.relation
    AND blocking_locks.page IS NOT DISTINCT FROM blocked_locks.page
    AND blocking_locks.tuple IS NOT DISTINCT FROM blocked_locks.tuple
    AND blocking_locks.virtualxid IS NOT DISTINCT FROM blocked_locks.virtualxid
    AND blocking_locks.transactionid IS NOT DISTINCT FROM blocked_locks.transactionid
    AND blocking_locks.classid IS NOT DISTINCT FROM blocked_locks.classid
    AND blocking_locks.objid IS NOT DISTINCT FROM blocked_locks.objid
    AND blocking_locks.objsubid IS NOT DISTINCT FROM blocked_locks.objsubid
    AND blocking_locks.pid != blocked_locks.pid
JOIN pg_catalog.pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.granted;

-- 7. Database size monitoring
SELECT 
    pg_size_pretty(pg_database_size('opensim')) as database_size,
    pg_size_pretty(pg_total_relation_size('UserAccounts')) as user_accounts_size,
    pg_size_pretty(pg_total_relation_size('assets')) as assets_size,
    pg_size_pretty(pg_total_relation_size('inventoryitems')) as inventory_size,
    pg_size_pretty(pg_total_relation_size('regions')) as regions_size;

-- 8. Cache hit ratio
SELECT 
    sum(heap_blks_read) as heap_read,
    sum(heap_blks_hit) as heap_hit,
    round(sum(heap_blks_hit) / (sum(heap_blks_hit) + sum(heap_blks_read)) * 100, 2) as hit_ratio
FROM pg_statio_user_tables;
```

### Database Alert Rules

Create specific database alert rules:

```yaml
# database_alerts.yml
groups:
- name: opensim.database
  rules:
  - alert: DatabaseDown
    expr: pg_up == 0
    for: 1m
    labels:
      severity: critical
      service: database
    annotations:
      summary: "PostgreSQL database is down"
      description: "PostgreSQL database has been down for more than 1 minute"

  - alert: DatabaseHighConnections
    expr: pg_stat_database_numbackends > 80
    for: 5m
    labels:
      severity: warning
      service: database
    annotations:
      summary: "High number of database connections"
      description: "Database has {{ $value }} active connections"

  - alert: DatabaseSlowQueries
    expr: rate(pg_stat_statements_mean_time[5m]) > 1000
    for: 10m
    labels:
      severity: warning
      service: database
    annotations:
      summary: "Database slow queries detected"
      description: "Average query time is {{ $value }}ms"

  - alert: DatabaseLowCacheHitRatio
    expr: pg_stat_database_blks_hit / (pg_stat_database_blks_hit + pg_stat_database_blks_read) < 0.95
    for: 15m
    labels:
      severity: warning
      service: database
    annotations:
      summary: "Low database cache hit ratio"
      description: "Cache hit ratio is {{ $value | humanizePercentage }}"

  - alert: DatabaseHighDiskUsage
    expr: pg_database_size_bytes > 50 * 1024 * 1024 * 1024  # 50GB
    for: 30m
    labels:
      severity: warning
      service: database
    annotations:
      summary: "Database size is large"
      description: "Database size is {{ $value | humanizeBytes }}"

  - alert: DatabaseDeadlocks
    expr: rate(pg_stat_database_deadlocks[5m]) > 0
    for: 2m
    labels:
      severity: warning
      service: database
    annotations:
      summary: "Database deadlocks detected"
      description: "Deadlock rate: {{ $value }} deadlocks/second"
```

## Network and Security Monitoring

### Zero Trust Network Monitoring

Monitor OpenZiti zero trust network:

```rust
// monitoring/ziti_monitor.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitiNetworkMetrics {
    pub controller_status: ControllerStatus,
    pub edge_routers: Vec<EdgeRouterStatus>,
    pub identities: Vec<IdentityStatus>,
    pub services: Vec<ServiceStatus>,
    pub policies: Vec<PolicyStatus>,
    pub tunnels: Vec<TunnelStatus>,
    pub security_events: Vec<SecurityEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerStatus {
    pub id: String,
    pub version: String,
    pub uptime: Duration,
    pub connected_edge_routers: u32,
    pub active_identities: u32,
    pub active_services: u32,
    pub policy_count: u32,
    pub api_response_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRouterStatus {
    pub id: String,
    pub name: String,
    pub version: String,
    pub online: bool,
    pub connected_identities: u32,
    pub tunneled_connections: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityStatus {
    pub id: String,
    pub name: String,
    pub identity_type: String,
    pub online: bool,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub active_connections: u32,
    pub authenticated: bool,
    pub mfa_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub permissions: Vec<String>,
    pub active_sessions: u32,
    pub terminator_count: u32,
    pub health_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStatus {
    pub id: String,
    pub name: String,
    pub policy_type: String,
    pub enabled: bool,
    pub identity_count: u32,
    pub service_count: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub id: String,
    pub identity_id: String,
    pub service_id: String,
    pub edge_router_id: String,
    pub established: chrono::DateTime<chrono::Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub encrypted: bool,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: SecurityEventType,
    pub identity_id: Option<String>,
    pub service_id: Option<String>,
    pub description: String,
    pub severity: SecuritySeverity,
    pub source_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationFailure,
    PolicyViolation,
    UnauthorizedAccess,
    SuspiciousActivity,
    MfaFailure,
    IdentityCompromise,
    ServiceUnavailable,
    TunnelEstablishmentFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct ZitiMonitor {
    controller_url: String,
    api_token: String,
    metrics_history: Vec<ZitiNetworkMetrics>,
    collection_interval: Duration,
}

impl ZitiMonitor {
    pub fn new(controller_url: String, api_token: String) -> Self {
        Self {
            controller_url,
            api_token,
            metrics_history: Vec::new(),
            collection_interval: Duration::from_secs(30),
        }
    }
    
    pub async fn start_monitoring(&mut self) {
        let mut interval = interval(self.collection_interval);
        
        loop {
            interval.tick().await;
            
            match self.collect_metrics().await {
                Ok(metrics) => {
                    self.metrics_history.push(metrics);
                    
                    // Keep only last hour of data
                    if self.metrics_history.len() > 120 {
                        self.metrics_history.remove(0);
                    }
                }
                Err(e) => {
                    log::error!("Failed to collect Ziti metrics: {}", e);
                }
            }
        }
    }
    
    async fn collect_metrics(&self) -> Result<ZitiNetworkMetrics, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        
        // Collect controller status
        let controller_status = self.get_controller_status(&client).await?;
        
        // Collect edge router status
        let edge_routers = self.get_edge_routers(&client).await?;
        
        // Collect identity status
        let identities = self.get_identities(&client).await?;
        
        // Collect service status
        let services = self.get_services(&client).await?;
        
        // Collect policy status
        let policies = self.get_policies(&client).await?;
        
        // Collect active tunnels
        let tunnels = self.get_tunnels(&client).await?;
        
        // Collect security events
        let security_events = self.get_security_events(&client).await?;
        
        Ok(ZitiNetworkMetrics {
            controller_status,
            edge_routers,
            identities,
            services,
            policies,
            tunnels,
            security_events,
        })
    }
    
    async fn get_controller_status(&self, client: &reqwest::Client) -> Result<ControllerStatus, Box<dyn std::error::Error>> {
        let response = client
            .get(&format!("{}/version", self.controller_url))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;
        
        // Parse response and create ControllerStatus
        // This is a simplified implementation
        Ok(ControllerStatus {
            id: "controller-1".to_string(),
            version: "0.27.0".to_string(),
            uptime: Duration::from_secs(86400),
            connected_edge_routers: 0,
            active_identities: 0,
            active_services: 0,
            policy_count: 0,
            api_response_time_ms: 50.0,
        })
    }
    
    async fn get_edge_routers(&self, client: &reqwest::Client) -> Result<Vec<EdgeRouterStatus>, Box<dyn std::error::Error>> {
        let response = client
            .get(&format!("{}/edge-routers", self.controller_url))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;
        
        // Parse and return edge router status
        Ok(Vec::new())
    }
    
    async fn get_identities(&self, client: &reqwest::Client) -> Result<Vec<IdentityStatus>, Box<dyn std::error::Error>> {
        let response = client
            .get(&format!("{}/identities", self.controller_url))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;
        
        // Parse and return identity status
        Ok(Vec::new())
    }
    
    async fn get_services(&self, client: &reqwest::Client) -> Result<Vec<ServiceStatus>, Box<dyn std::error::Error>> {
        // Implementation for service status collection
        Ok(Vec::new())
    }
    
    async fn get_policies(&self, client: &reqwest::Client) -> Result<Vec<PolicyStatus>, Box<dyn std::error::Error>> {
        // Implementation for policy status collection
        Ok(Vec::new())
    }
    
    async fn get_tunnels(&self, client: &reqwest::Client) -> Result<Vec<TunnelStatus>, Box<dyn std::error::Error>> {
        // Implementation for tunnel status collection
        Ok(Vec::new())
    }
    
    async fn get_security_events(&self, client: &reqwest::Client) -> Result<Vec<SecurityEvent>, Box<dyn std::error::Error>> {
        // Implementation for security event collection
        Ok(Vec::new())
    }
    
    pub fn analyze_security_trends(&self) -> SecurityAnalysis {
        if self.metrics_history.is_empty() {
            return SecurityAnalysis::default();
        }
        
        let recent_events: Vec<_> = self.metrics_history
            .iter()
            .flat_map(|m| &m.security_events)
            .collect();
        
        let auth_failures = recent_events
            .iter()
            .filter(|e| matches!(e.event_type, SecurityEventType::AuthenticationFailure))
            .count();
        
        let policy_violations = recent_events
            .iter()
            .filter(|e| matches!(e.event_type, SecurityEventType::PolicyViolation))
            .count();
        
        let critical_events = recent_events
            .iter()
            .filter(|e| matches!(e.severity, SecuritySeverity::Critical))
            .count();
        
        SecurityAnalysis {
            total_events: recent_events.len(),
            auth_failures,
            policy_violations,
            critical_events,
            trend: self.calculate_security_trend(),
            recommendations: self.generate_security_recommendations(),
        }
    }
    
    fn calculate_security_trend(&self) -> SecurityTrend {
        // Analyze trends in security events
        SecurityTrend::Stable
    }
    
    fn generate_security_recommendations(&self) -> Vec<String> {
        // Generate security recommendations based on analysis
        Vec::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    pub total_events: usize,
    pub auth_failures: usize,
    pub policy_violations: usize,
    pub critical_events: usize,
    pub trend: SecurityTrend,
    pub recommendations: Vec<String>,
}

impl Default for SecurityAnalysis {
    fn default() -> Self {
        Self {
            total_events: 0,
            auth_failures: 0,
            policy_violations: 0,
            critical_events: 0,
            trend: SecurityTrend::Stable,
            recommendations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityTrend {
    Improving,
    Stable,
    Concerning,
    Critical,
}
```

## Automated Alert Management

### Multi-Channel Alert Delivery

Configure comprehensive alert delivery system:

```yaml
# alertmanager_advanced.yml
global:
  smtp_smarthost: 'smtp.example.com:587'
  smtp_from: 'alerts@opensim-next.org'
  smtp_auth_username: 'alerts@opensim-next.org'
  smtp_auth_password: 'smtp_password'
  slack_api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
  pagerduty_url: 'https://events.pagerduty.com/v2/enqueue'

templates:
  - '/etc/alertmanager/templates/*.tmpl'

route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 12h
  receiver: 'default'
  routes:
  # Critical alerts - immediate escalation
  - match:
      severity: critical
    receiver: 'critical-escalation'
    group_wait: 0s
    repeat_interval: 1m
    routes:
    - match:
        service: opensim-next
      receiver: 'opensim-critical'
    - match:
        service: database
      receiver: 'database-critical'
    - match:
        category: security
      receiver: 'security-critical'

  # OpenSim specific alerts
  - match:
      service: opensim-next
    receiver: 'opensim-team'
    routes:
    - match:
        alertname: 'HighResponseTime'
      receiver: 'performance-team'
    - match:
        alertname: 'LowRegionFPS'
      receiver: 'physics-team'

  # Security alerts
  - match:
      category: security
    receiver: 'security-team'
    group_wait: 0s

  # Database alerts
  - match:
      service: database
    receiver: 'database-team'

  # WebSocket alerts
  - match:
      service: websocket
    receiver: 'websocket-team'

receivers:
- name: 'default'
  slack_configs:
  - channel: '#general-alerts'
    title: 'OpenSim Next Alert'
    text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'

- name: 'critical-escalation'
  email_configs:
  - to: 'oncall@opensim-next.org'
    subject: 'CRITICAL: OpenSim Next {{ .GroupLabels.service }}'
    html: |
      <h2 style="color: #d32f2f;">CRITICAL ALERT</h2>
      {{ range .Alerts }}
      <div style="margin: 10px 0; padding: 10px; border-left: 4px solid #d32f2f;">
        <h3>{{ .Annotations.summary }}</h3>
        <p><strong>Description:</strong> {{ .Annotations.description }}</p>
        <p><strong>Service:</strong> {{ .Labels.service }}</p>
        <p><strong>Instance:</strong> {{ .Labels.instance }}</p>
        <p><strong>Started:</strong> {{ .StartsAt }}</p>
        <p><strong>Labels:</strong></p>
        <ul>
        {{ range .Labels.SortedPairs }}
          <li>{{ .Name }}: {{ .Value }}</li>
        {{ end }}
        </ul>
      </div>
      {{ end }}
  
  slack_configs:
  - channel: '#critical-alerts'
    title: 'CRITICAL: OpenSim Next Alert'
    text: |
      🚨 *CRITICAL ALERT* 🚨
      {{ range .Alerts }}
      *{{ .Annotations.summary }}*
      {{ .Annotations.description }}
      Service: {{ .Labels.service }}
      Instance: {{ .Labels.instance }}
      {{ end }}
    color: 'danger'
    send_resolved: true

  pagerduty_configs:
  - service_key: 'PAGERDUTY_SERVICE_KEY'
    description: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
    details:
      service: '{{ .GroupLabels.service }}'
      severity: '{{ .GroupLabels.severity }}'
      instance: '{{ .GroupLabels.instance }}'

- name: 'opensim-critical'
  email_configs:
  - to: 'opensim-team@opensim-next.org'
    subject: 'CRITICAL: OpenSim Next Server Issue'
    html: |
      {{ template "email.html" . }}
  
  slack_configs:
  - channel: '#opensim-critical'
    title: 'CRITICAL: OpenSim Next Server'
    text: |
      🚨 *CRITICAL SERVER ISSUE* 🚨
      {{ range .Alerts }}
      *{{ .Annotations.summary }}*
      {{ .Annotations.description }}
      
      *Troubleshooting Steps:*
      1. Check server logs: `tail -f /var/log/opensim-next/opensim.log`
      2. Verify database connectivity: `psql -h localhost -U opensim opensim`
      3. Check system resources: `htop`
      4. Review recent deployments
      {{ end }}
    color: 'danger'

- name: 'security-critical'
  email_configs:
  - to: 'security@opensim-next.org,oncall@opensim-next.org'
    subject: 'SECURITY ALERT: OpenSim Next'
    html: |
      <h2 style="color: #d32f2f;">SECURITY ALERT</h2>
      {{ range .Alerts }}
      <div style="margin: 10px 0; padding: 10px; border-left: 4px solid #ff5722;">
        <h3>🔒 {{ .Annotations.summary }}</h3>
        <p><strong>Description:</strong> {{ .Annotations.description }}</p>
        <p><strong>Severity:</strong> {{ .Labels.severity }}</p>
        <p><strong>Source:</strong> {{ .Labels.instance }}</p>
        <p><strong>Detection Time:</strong> {{ .StartsAt }}</p>
      </div>
      {{ end }}
  
  slack_configs:
  - channel: '#security-alerts'
    title: '🔒 SECURITY ALERT'
    text: |
      🔒 *SECURITY INCIDENT DETECTED* 🔒
      {{ range .Alerts }}
      *{{ .Annotations.summary }}*
      {{ .Annotations.description }}
      
      *Immediate Actions Required:*
      1. Review security logs immediately
      2. Check for unauthorized access attempts
      3. Verify zero trust policies
      4. Monitor for lateral movement
      {{ end }}
    color: 'danger'

- name: 'opensim-team'
  slack_configs:
  - channel: '#opensim-monitoring'
    title: 'OpenSim Next Alert'
    text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
    color: 'warning'

- name: 'performance-team'
  slack_configs:
  - channel: '#performance-alerts'
    title: 'Performance Alert'
    text: |
      ⚡ *Performance Issue Detected*
      {{ range .Alerts }}
      {{ .Annotations.summary }}
      {{ .Annotations.description }}
      {{ end }}
    color: 'warning'

- name: 'security-team'
  slack_configs:
  - channel: '#security-monitoring'
    title: 'Security Alert'
    text: |
      🔐 *Security Event*
      {{ range .Alerts }}
      {{ .Annotations.summary }}
      {{ .Annotations.description }}
      {{ end }}
    color: 'warning'

- name: 'database-team'
  slack_configs:
  - channel: '#database-alerts'
    title: 'Database Alert'
    text: |
      🗄️ *Database Issue*
      {{ range .Alerts }}
      {{ .Annotations.summary }}
      {{ .Annotations.description }}
      {{ end }}
    color: 'warning'

- name: 'websocket-team'
  slack_configs:
  - channel: '#websocket-alerts'
    title: 'WebSocket Alert'
    text: |
      🌐 *WebSocket Issue*
      {{ range .Alerts }}
      {{ .Annotations.summary }}
      {{ .Annotations.description }}
      {{ end }}
    color: 'warning'

inhibit_rules:
- source_match:
    severity: 'critical'
  target_match:
    severity: 'warning'
  equal: ['alertname', 'instance']

- source_match:
    alertname: 'OpenSimDown'
  target_match_re:
    alertname: '(HighResponseTime|DatabaseSlowQueries|WebSocketConnectionDrop)'
  equal: ['instance']
```

### Alert Templates

Create custom alert templates:

```html
<!-- /etc/alertmanager/templates/email.html -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>OpenSim Next Alert</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        
        .alert-container {
            background: white;
            border-radius: 8px;
            padding: 30px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        
        .alert-header {
            border-bottom: 3px solid #e74c3c;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }
        
        .alert-header.warning {
            border-bottom-color: #f39c12;
        }
        
        .alert-header.critical {
            border-bottom-color: #e74c3c;
        }
        
        .alert-title {
            margin: 0;
            color: #2c3e50;
            font-size: 28px;
            font-weight: 300;
        }
        
        .alert-item {
            margin: 20px 0;
            padding: 20px;
            border-left: 4px solid #3498db;
            background: #f8f9fa;
            border-radius: 0 4px 4px 0;
        }
        
        .alert-item.critical {
            border-left-color: #e74c3c;
            background: #fdf2f2;
        }
        
        .alert-item.warning {
            border-left-color: #f39c12;
            background: #fffbf0;
        }
        
        .alert-summary {
            font-size: 18px;
            font-weight: 600;
            color: #2c3e50;
            margin-bottom: 10px;
        }
        
        .alert-description {
            font-size: 14px;
            color: #5d6d7e;
            margin-bottom: 15px;
        }
        
        .alert-details {
            background: white;
            padding: 15px;
            border-radius: 4px;
            border: 1px solid #dee2e6;
        }
        
        .detail-row {
            display: flex;
            justify-content: space-between;
            padding: 5px 0;
            border-bottom: 1px solid #f1f2f6;
        }
        
        .detail-row:last-child {
            border-bottom: none;
        }
        
        .detail-label {
            font-weight: 600;
            color: #34495e;
        }
        
        .detail-value {
            color: #5d6d7e;
            font-family: 'Courier New', monospace;
        }
        
        .alert-footer {
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #dee2e6;
            text-align: center;
            color: #7f8c8d;
            font-size: 12px;
        }
        
        .btn {
            display: inline-block;
            padding: 10px 20px;
            background: #3498db;
            color: white;
            text-decoration: none;
            border-radius: 4px;
            margin: 10px 5px;
        }
        
        .btn.critical {
            background: #e74c3c;
        }
        
        .troubleshooting {
            background: #e8f4fd;
            border: 1px solid #3498db;
            border-radius: 4px;
            padding: 15px;
            margin: 20px 0;
        }
        
        .troubleshooting h4 {
            margin: 0 0 10px 0;
            color: #2980b9;
        }
        
        .troubleshooting ol {
            margin: 0;
            padding-left: 20px;
        }
    </style>
</head>
<body>
    <div class="alert-container">
        <div class="alert-header {{ .Status }}">
            <h1 class="alert-title">
                {{ if eq .Status "firing" }}🚨{{ else }}✅{{ end }}
                OpenSim Next Alert - {{ .Status | title }}
            </h1>
        </div>
        
        {{ range .Alerts }}
        <div class="alert-item {{ .Labels.severity }}">
            <div class="alert-summary">{{ .Annotations.summary }}</div>
            <div class="alert-description">{{ .Annotations.description }}</div>
            
            <div class="alert-details">
                <div class="detail-row">
                    <span class="detail-label">Severity:</span>
                    <span class="detail-value">{{ .Labels.severity | upper }}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">Service:</span>
                    <span class="detail-value">{{ .Labels.service }}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">Instance:</span>
                    <span class="detail-value">{{ .Labels.instance }}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">Started:</span>
                    <span class="detail-value">{{ .StartsAt.Format "2006-01-02 15:04:05" }}</span>
                </div>
                {{ if .EndsAt }}
                <div class="detail-row">
                    <span class="detail-label">Ended:</span>
                    <span class="detail-value">{{ .EndsAt.Format "2006-01-02 15:04:05" }}</span>
                </div>
                {{ end }}
            </div>
            
            {{ if .Annotations.runbook_url }}
            <div style="margin-top: 15px;">
                <a href="{{ .Annotations.runbook_url }}" class="btn">📖 Runbook</a>
            </div>
            {{ end }}
            
            {{ if eq .Labels.severity "critical" }}
            <div class="troubleshooting">
                <h4>🔧 Immediate Actions Required:</h4>
                <ol>
                    <li>Check server status and logs</li>
                    <li>Verify database connectivity</li>
                    <li>Review system resources (CPU, memory, disk)</li>
                    <li>Check recent deployments or changes</li>
                    <li>Escalate to on-call engineer if needed</li>
                </ol>
            </div>
            {{ end }}
        </div>
        {{ end }}
        
        <div class="alert-footer">
            <p>This alert was generated by OpenSim Next Monitoring System</p>
            <p>
                <a href="http://monitoring.opensim-next.org" class="btn">🔍 View Dashboard</a>
                <a href="http://logs.opensim-next.org" class="btn">📋 View Logs</a>
            </p>
        </div>
    </div>
</body>
</html>
```

## Capacity Planning and Scaling

### Capacity Analysis System

Implement intelligent capacity planning:

```rust
// capacity/planner.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlan {
    pub timestamp: DateTime<Utc>,
    pub current_capacity: ResourceCapacity,
    pub predicted_demand: ResourceDemand,
    pub scaling_recommendations: Vec<ScalingRecommendation>,
    pub cost_analysis: CostAnalysis,
    pub timeline: CapacityTimeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapacity {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub storage_gb: u32,
    pub network_bandwidth_mbps: u32,
    pub max_concurrent_users: u32,
    pub max_regions: u32,
    pub database_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDemand {
    pub cpu_utilization_percent: f64,
    pub memory_utilization_percent: f64,
    pub storage_utilization_percent: f64,
    pub network_utilization_percent: f64,
    pub concurrent_users: u32,
    pub active_regions: u32,
    pub database_connections_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRecommendation {
    pub resource_type: ResourceType,
    pub action: ScalingAction,
    pub current_value: f64,
    pub recommended_value: f64,
    pub urgency: Urgency,
    pub reasoning: String,
    pub estimated_cost_change: f64,
    pub implementation_timeline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    CPU,
    Memory,
    Storage,
    Network,
    DatabaseConnections,
    ServerInstances,
    LoadBalancers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAction {
    ScaleUp,
    ScaleDown,
    Optimize,
    AddInstances,
    RemoveInstances,
    Maintain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Urgency {
    Immediate,    // < 24 hours
    Soon,        // 1-7 days
    Planned,     // 1-4 weeks
    Future,      // > 1 month
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    pub current_monthly_cost: f64,
    pub projected_monthly_cost: f64,
    pub cost_per_user: f64,
    pub cost_optimization_opportunities: Vec<CostOptimization>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    pub description: String,
    pub potential_savings: f64,
    pub implementation_effort: String,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityTimeline {
    pub next_7_days: HashMap<String, ResourceDemand>,
    pub next_30_days: HashMap<String, ResourceDemand>,
    pub next_90_days: HashMap<String, ResourceDemand>,
    pub seasonal_patterns: Vec<SeasonalPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    pub name: String,
    pub period: String,
    pub demand_multiplier: f64,
    pub description: String,
}

pub struct CapacityPlanner {
    historical_data: Vec<ResourceDemand>,
    current_capacity: ResourceCapacity,
    growth_models: HashMap<String, GrowthModel>,
    cost_models: HashMap<String, CostModel>,
}

#[derive(Debug, Clone)]
struct GrowthModel {
    linear_rate: f64,
    seasonal_factor: f64,
    trend_factor: f64,
}

#[derive(Debug, Clone)]
struct CostModel {
    base_cost: f64,
    cost_per_unit: f64,
    volume_discounts: Vec<(u32, f64)>,
}

impl CapacityPlanner {
    pub fn new(current_capacity: ResourceCapacity) -> Self {
        Self {
            historical_data: Vec::new(),
            current_capacity,
            growth_models: HashMap::new(),
            cost_models: HashMap::new(),
        }
    }
    
    pub fn add_historical_data(&mut self, demand: ResourceDemand) {
        self.historical_data.push(demand);
        
        // Keep only last 90 days of data
        if self.historical_data.len() > 90 * 24 {
            self.historical_data.remove(0);
        }
    }
    
    pub async fn generate_capacity_plan(&self) -> CapacityPlan {
        let predicted_demand = self.predict_future_demand().await;
        let scaling_recommendations = self.generate_scaling_recommendations(&predicted_demand).await;
        let cost_analysis = self.analyze_costs(&scaling_recommendations).await;
        let timeline = self.generate_timeline().await;
        
        CapacityPlan {
            timestamp: Utc::now(),
            current_capacity: self.current_capacity.clone(),
            predicted_demand,
            scaling_recommendations,
            cost_analysis,
            timeline,
        }
    }
    
    async fn predict_future_demand(&self) -> ResourceDemand {
        if self.historical_data.is_empty() {
            return ResourceDemand {
                cpu_utilization_percent: 50.0,
                memory_utilization_percent: 60.0,
                storage_utilization_percent: 40.0,
                network_utilization_percent: 30.0,
                concurrent_users: 100,
                active_regions: 5,
                database_connections_used: 50,
            };
        }
        
        // Use time series analysis to predict future demand
        let recent_data = &self.historical_data[self.historical_data.len().saturating_sub(168)..]; // Last week
        
        let avg_cpu = recent_data.iter().map(|d| d.cpu_utilization_percent).sum::<f64>() / recent_data.len() as f64;
        let avg_memory = recent_data.iter().map(|d| d.memory_utilization_percent).sum::<f64>() / recent_data.len() as f64;
        let avg_storage = recent_data.iter().map(|d| d.storage_utilization_percent).sum::<f64>() / recent_data.len() as f64;
        let avg_network = recent_data.iter().map(|d| d.network_utilization_percent).sum::<f64>() / recent_data.len() as f64;
        let avg_users = recent_data.iter().map(|d| d.concurrent_users).sum::<u32>() / recent_data.len() as u32;
        let avg_regions = recent_data.iter().map(|d| d.active_regions).sum::<u32>() / recent_data.len() as u32;
        let avg_db_conn = recent_data.iter().map(|d| d.database_connections_used).sum::<u32>() / recent_data.len() as u32;
        
        // Apply growth factors
        let growth_factor = self.calculate_growth_factor();
        
        ResourceDemand {
            cpu_utilization_percent: avg_cpu * growth_factor,
            memory_utilization_percent: avg_memory * growth_factor,
            storage_utilization_percent: avg_storage * growth_factor,
            network_utilization_percent: avg_network * growth_factor,
            concurrent_users: (avg_users as f64 * growth_factor) as u32,
            active_regions: (avg_regions as f64 * growth_factor) as u32,
            database_connections_used: (avg_db_conn as f64 * growth_factor) as u32,
        }
    }
    
    fn calculate_growth_factor(&self) -> f64 {
        // Analyze historical growth trends
        if self.historical_data.len() < 14 {
            return 1.1; // Default 10% growth assumption
        }
        
        let recent_week = &self.historical_data[self.historical_data.len() - 7..];
        let previous_week = &self.historical_data[self.historical_data.len() - 14..self.historical_data.len() - 7];
        
        let recent_avg_users = recent_week.iter().map(|d| d.concurrent_users).sum::<u32>() / recent_week.len() as u32;
        let previous_avg_users = previous_week.iter().map(|d| d.concurrent_users).sum::<u32>() / previous_week.len() as u32;
        
        if previous_avg_users > 0 {
            let week_over_week_growth = recent_avg_users as f64 / previous_avg_users as f64;
            // Project 4 weeks forward and cap at reasonable limits
            (week_over_week_growth.powf(4.0)).min(2.0).max(0.5)
        } else {
            1.1
        }
    }
    
    async fn generate_scaling_recommendations(&self, predicted_demand: &ResourceDemand) -> Vec<ScalingRecommendation> {
        let mut recommendations = Vec::new();
        
        // CPU scaling
        if predicted_demand.cpu_utilization_percent > 80.0 {
            recommendations.push(ScalingRecommendation {
                resource_type: ResourceType::CPU,
                action: ScalingAction::ScaleUp,
                current_value: self.current_capacity.cpu_cores as f64,
                recommended_value: (self.current_capacity.cpu_cores as f64 * 1.5).ceil(),
                urgency: if predicted_demand.cpu_utilization_percent > 90.0 { Urgency::Immediate } else { Urgency::Soon },
                reasoning: format!("CPU utilization predicted to reach {:.1}%", predicted_demand.cpu_utilization_percent),
                estimated_cost_change: 250.0, // Example cost per additional core
                implementation_timeline: "2-4 hours for vertical scaling".to_string(),
            });
        }
        
        // Memory scaling
        if predicted_demand.memory_utilization_percent > 85.0 {
            recommendations.push(ScalingRecommendation {
                resource_type: ResourceType::Memory,
                action: ScalingAction::ScaleUp,
                current_value: self.current_capacity.memory_gb as f64,
                recommended_value: (self.current_capacity.memory_gb as f64 * 1.4).ceil(),
                urgency: if predicted_demand.memory_utilization_percent > 95.0 { Urgency::Immediate } else { Urgency::Soon },
                reasoning: format!("Memory utilization predicted to reach {:.1}%", predicted_demand.memory_utilization_percent),
                estimated_cost_change: 100.0, // Example cost per GB
                implementation_timeline: "2-4 hours for vertical scaling".to_string(),
            });
        }
        
        // User capacity
        let user_capacity_ratio = predicted_demand.concurrent_users as f64 / self.current_capacity.max_concurrent_users as f64;
        if user_capacity_ratio > 0.8 {
            recommendations.push(ScalingRecommendation {
                resource_type: ResourceType::ServerInstances,
                action: ScalingAction::AddInstances,
                current_value: 1.0, // Assuming single instance
                recommended_value: (user_capacity_ratio.ceil()).max(2.0),
                urgency: if user_capacity_ratio > 0.9 { Urgency::Soon } else { Urgency::Planned },
                reasoning: format!("User capacity at {:.1}% with {} concurrent users", user_capacity_ratio * 100.0, predicted_demand.concurrent_users),
                estimated_cost_change: 500.0, // Example cost per additional instance
                implementation_timeline: "1-2 days for horizontal scaling".to_string(),
            });
        }
        
        // Database connections
        let db_capacity_ratio = predicted_demand.database_connections_used as f64 / self.current_capacity.database_connections as f64;
        if db_capacity_ratio > 0.8 {
            recommendations.push(ScalingRecommendation {
                resource_type: ResourceType::DatabaseConnections,
                action: ScalingAction::ScaleUp,
                current_value: self.current_capacity.database_connections as f64,
                recommended_value: (self.current_capacity.database_connections as f64 * 1.5).ceil(),
                urgency: if db_capacity_ratio > 0.9 { Urgency::Soon } else { Urgency::Planned },
                reasoning: format!("Database connection pool at {:.1}% capacity", db_capacity_ratio * 100.0),
                estimated_cost_change: 150.0,
                implementation_timeline: "30 minutes for configuration change".to_string(),
            });
        }
        
        recommendations
    }
    
    async fn analyze_costs(&self, recommendations: &[ScalingRecommendation]) -> CostAnalysis {
        let current_monthly_cost = 1000.0; // Example base cost
        let additional_cost: f64 = recommendations.iter().map(|r| r.estimated_cost_change).sum();
        
        let cost_optimizations = vec![
            CostOptimization {
                description: "Implement auto-scaling to reduce idle capacity".to_string(),
                potential_savings: 200.0,
                implementation_effort: "Medium".to_string(),
                risk_level: "Low".to_string(),
            },
            CostOptimization {
                description: "Use spot instances for non-critical workloads".to_string(),
                potential_savings: 300.0,
                implementation_effort: "High".to_string(),
                risk_level: "Medium".to_string(),
            },
        ];
        
        CostAnalysis {
            current_monthly_cost,
            projected_monthly_cost: current_monthly_cost + additional_cost,
            cost_per_user: (current_monthly_cost + additional_cost) / 1000.0, // Assuming 1000 users
            cost_optimization_opportunities: cost_optimizations,
        }
    }
    
    async fn generate_timeline(&self) -> CapacityTimeline {
        let mut next_7_days = HashMap::new();
        let mut next_30_days = HashMap::new();
        let mut next_90_days = HashMap::new();
        
        // Generate predictions for different time horizons
        for day in 1..=7 {
            let date = (Utc::now() + Duration::days(day)).format("%Y-%m-%d").to_string();
            next_7_days.insert(date, self.predict_demand_for_day(day).await);
        }
        
        for day in (8..=30).step_by(7) {
            let date = (Utc::now() + Duration::days(day)).format("%Y-%m-%d").to_string();
            next_30_days.insert(date, self.predict_demand_for_day(day).await);
        }
        
        for day in (31..=90).step_by(15) {
            let date = (Utc::now() + Duration::days(day)).format("%Y-%m-%d").to_string();
            next_90_days.insert(date, self.predict_demand_for_day(day).await);
        }
        
        let seasonal_patterns = vec![
            SeasonalPattern {
                name: "Weekend Peak".to_string(),
                period: "Weekly".to_string(),
                demand_multiplier: 1.3,
                description: "Higher user activity on weekends".to_string(),
            },
            SeasonalPattern {
                name: "Holiday Season".to_string(),
                period: "Annual".to_string(),
                demand_multiplier: 1.8,
                description: "Increased activity during holiday periods".to_string(),
            },
        ];
        
        CapacityTimeline {
            next_7_days,
            next_30_days,
            next_90_days,
            seasonal_patterns,
        }
    }
    
    async fn predict_demand_for_day(&self, days_ahead: i64) -> ResourceDemand {
        // Simple prediction model - in reality, this would use more sophisticated algorithms
        let base_demand = if let Some(latest) = self.historical_data.last() {
            latest.clone()
        } else {
            ResourceDemand {
                cpu_utilization_percent: 50.0,
                memory_utilization_percent: 60.0,
                storage_utilization_percent: 40.0,
                network_utilization_percent: 30.0,
                concurrent_users: 100,
                active_regions: 5,
                database_connections_used: 50,
            }
        };
        
        // Apply growth and seasonal factors
        let growth_factor = 1.0 + (days_ahead as f64 * 0.01); // 1% growth per day
        let seasonal_factor = if (Utc::now() + Duration::days(days_ahead)).weekday().number_from_monday() >= 6 {
            1.3 // Weekend multiplier
        } else {
            1.0
        };
        
        ResourceDemand {
            cpu_utilization_percent: base_demand.cpu_utilization_percent * growth_factor * seasonal_factor,
            memory_utilization_percent: base_demand.memory_utilization_percent * growth_factor * seasonal_factor,
            storage_utilization_percent: base_demand.storage_utilization_percent * growth_factor,
            network_utilization_percent: base_demand.network_utilization_percent * growth_factor * seasonal_factor,
            concurrent_users: (base_demand.concurrent_users as f64 * growth_factor * seasonal_factor) as u32,
            active_regions: (base_demand.active_regions as f64 * growth_factor) as u32,
            database_connections_used: (base_demand.database_connections_used as f64 * growth_factor * seasonal_factor) as u32,
        }
    }
}
```

## Troubleshooting and Maintenance

### Automated Troubleshooting System

Implement intelligent troubleshooting capabilities:

```rust
// troubleshooting/diagnostics.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub system_health: SystemHealth,
    pub issues_detected: Vec<DetectedIssue>,
    pub recommended_actions: Vec<RecommendedAction>,
    pub performance_impact: PerformanceImpact,
    pub resolution_status: ResolutionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_score: f64, // 0-100
    pub component_scores: HashMap<String, f64>,
    pub critical_issues: u32,
    pub warnings: u32,
    pub healthy_components: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedIssue {
    pub id: String,
    pub category: IssueCategory,
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub affected_components: Vec<String>,
    pub symptoms: Vec<String>,
    pub potential_causes: Vec<String>,
    pub detection_time: chrono::DateTime<chrono::Utc>,
    pub confidence: f64, // 0-1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Performance,
    Connectivity,
    Configuration,
    Resource,
    Security,
    Database,
    Physics,
    WebSocket,
    ZeroTrust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub id: String,
    pub title: String,
    pub description: String,
    pub commands: Vec<String>,
    pub expected_outcome: String,
    pub risk_level: RiskLevel,
    pub estimated_time: String,
    pub prerequisites: Vec<String>,
    pub rollback_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    pub response_time_degradation: f64,
    pub throughput_reduction: f64,
    pub user_experience_impact: String,
    pub affected_regions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStatus {
    Investigating,
    ActionRequired,
    InProgress,
    Resolved,
    Monitoring,
}

pub struct DiagnosticEngine {
    diagnostic_rules: Vec<DiagnosticRule>,
    system_state: SystemState,
}

#[derive(Debug, Clone)]
pub struct DiagnosticRule {
    pub id: String,
    pub name: String,
    pub category: IssueCategory,
    pub condition: Box<dyn Fn(&SystemState) -> bool + Send + Sync>,
    pub issue_template: DetectedIssue,
    pub actions: Vec<RecommendedAction>,
}

#[derive(Debug, Clone, Default)]
pub struct SystemState {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_latency: f64,
    pub active_connections: u32,
    pub database_connections: u32,
    pub response_time_p95: f64,
    pub error_rate: f64,
    pub region_fps: HashMap<String, f64>,
    pub websocket_connections: u32,
    pub ziti_connections: u32,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            diagnostic_rules: Vec::new(),
            system_state: SystemState::default(),
        };
        
        engine.register_default_rules();
        engine
    }
    
    fn register_default_rules(&mut self) {
        // High CPU usage rule
        self.add_rule(DiagnosticRule {
            id: "high_cpu_usage".to_string(),
            name: "High CPU Usage".to_string(),
            category: IssueCategory::Performance,
            condition: Box::new(|state| state.cpu_usage > 85.0),
            issue_template: DetectedIssue {
                id: "high_cpu_usage".to_string(),
                category: IssueCategory::Performance,
                severity: IssueSeverity::High,
                title: "High CPU Usage Detected".to_string(),
                description: "CPU usage is above 85%, which may impact server performance".to_string(),
                affected_components: vec!["server".to_string()],
                symptoms: vec![
                    "Slow response times".to_string(),
                    "High server load".to_string(),
                    "Potential request timeouts".to_string(),
                ],
                potential_causes: vec![
                    "High user activity".to_string(),
                    "Inefficient physics calculations".to_string(),
                    "Memory pressure causing CPU overhead".to_string(),
                    "Background processes consuming resources".to_string(),
                ],
                detection_time: chrono::Utc::now(),
                confidence: 0.9,
            },
            actions: vec![
                RecommendedAction {
                    id: "investigate_cpu_usage".to_string(),
                    title: "Investigate CPU Usage".to_string(),
                    description: "Analyze which processes are consuming CPU".to_string(),
                    commands: vec![
                        "top -n 1 -b".to_string(),
                        "ps aux --sort=-%cpu | head -20".to_string(),
                        "iotop -n 1".to_string(),
                    ],
                    expected_outcome: "Identify processes causing high CPU usage".to_string(),
                    risk_level: RiskLevel::Safe,
                    estimated_time: "2-5 minutes".to_string(),
                    prerequisites: vec![],
                    rollback_commands: vec![],
                },
                RecommendedAction {
                    id: "optimize_physics_engines".to_string(),
                    title: "Optimize Physics Engines".to_string(),
                    description: "Check and optimize physics engine settings".to_string(),
                    commands: vec![
                        "curl -H 'X-API-Key: $API_KEY' http://localhost:8090/api/physics/status".to_string(),
                        "curl -H 'X-API-Key: $API_KEY' -X POST http://localhost:8090/api/physics/optimize".to_string(),
                    ],
                    expected_outcome: "Reduced physics calculation overhead".to_string(),
                    risk_level: RiskLevel::Low,
                    estimated_time: "5-10 minutes".to_string(),
                    prerequisites: vec!["API access".to_string()],
                    rollback_commands: vec![
                        "curl -H 'X-API-Key: $API_KEY' -X POST http://localhost:8090/api/physics/reset".to_string(),
                    ],
                },
            ],
        });
        
        // High memory usage rule
        self.add_rule(DiagnosticRule {
            id: "high_memory_usage".to_string(),
            name: "High Memory Usage".to_string(),
            category: IssueCategory::Resource,
            condition: Box::new(|state| state.memory_usage > 90.0),
            issue_template: DetectedIssue {
                id: "high_memory_usage".to_string(),
                category: IssueCategory::Resource,
                severity: IssueSeverity::Critical,
                title: "Critical Memory Usage".to_string(),
                description: "Memory usage is above 90%, system may become unstable".to_string(),
                affected_components: vec!["server".to_string(), "database".to_string()],
                symptoms: vec![
                    "Slow performance".to_string(),
                    "Potential out-of-memory errors".to_string(),
                    "System instability".to_string(),
                ],
                potential_causes: vec![
                    "Memory leaks in application".to_string(),
                    "Large asset cache".to_string(),
                    "Database connection pool issues".to_string(),
                    "Excessive number of active objects".to_string(),
                ],
                detection_time: chrono::Utc::now(),
                confidence: 0.95,
            },
            actions: vec![
                RecommendedAction {
                    id: "analyze_memory_usage".to_string(),
                    title: "Analyze Memory Usage".to_string(),
                    description: "Investigate memory consumption patterns".to_string(),
                    commands: vec![
                        "free -h".to_string(),
                        "ps aux --sort=-%mem | head -20".to_string(),
                        "cat /proc/meminfo".to_string(),
                    ],
                    expected_outcome: "Identify memory-consuming processes".to_string(),
                    risk_level: RiskLevel::Safe,
                    estimated_time: "2-5 minutes".to_string(),
                    prerequisites: vec![],
                    rollback_commands: vec![],
                },
                RecommendedAction {
                    id: "clear_asset_cache".to_string(),
                    title: "Clear Asset Cache".to_string(),
                    description: "Clear asset cache to free memory".to_string(),
                    commands: vec![
                        "curl -H 'X-API-Key: $API_KEY' -X DELETE http://localhost:8090/api/cache/assets".to_string(),
                    ],
                    expected_outcome: "Freed memory from asset cache".to_string(),
                    risk_level: RiskLevel::Low,
                    estimated_time: "1-2 minutes".to_string(),
                    prerequisites: vec!["API access".to_string()],
                    rollback_commands: vec![],
                },
            ],
        });
        
        // Database connection issues
        self.add_rule(DiagnosticRule {
            id: "database_connection_exhaustion".to_string(),
            name: "Database Connection Exhaustion".to_string(),
            category: IssueCategory::Database,
            condition: Box::new(|state| state.database_connections > 90),
            issue_template: DetectedIssue {
                id: "database_connection_exhaustion".to_string(),
                category: IssueCategory::Database,
                severity: IssueSeverity::High,
                title: "Database Connection Pool Nearly Exhausted".to_string(),
                description: "Database connection pool is at capacity, new connections may fail".to_string(),
                affected_components: vec!["database".to_string(), "application".to_string()],
                symptoms: vec![
                    "Database connection timeouts".to_string(),
                    "Failed database operations".to_string(),
                    "Slow query performance".to_string(),
                ],
                potential_causes: vec![
                    "Connection leaks".to_string(),
                    "Long-running transactions".to_string(),
                    "Insufficient connection pool size".to_string(),
                    "Database performance issues".to_string(),
                ],
                detection_time: chrono::Utc::now(),
                confidence: 0.9,
            },
            actions: vec![
                RecommendedAction {
                    id: "analyze_database_connections".to_string(),
                    title: "Analyze Database Connections".to_string(),
                    description: "Check active database connections and queries".to_string(),
                    commands: vec![
                        "psql -h localhost -U opensim opensim -c \"SELECT count(*) FROM pg_stat_activity;\"".to_string(),
                        "psql -h localhost -U opensim opensim -c \"SELECT pid, usename, application_name, state, query_start, query FROM pg_stat_activity WHERE state != 'idle';\"".to_string(),
                    ],
                    expected_outcome: "Identify connection usage patterns".to_string(),
                    risk_level: RiskLevel::Safe,
                    estimated_time: "2-5 minutes".to_string(),
                    prerequisites: vec!["Database access".to_string()],
                    rollback_commands: vec![],
                },
                RecommendedAction {
                    id: "increase_connection_pool".to_string(),
                    title: "Increase Connection Pool Size".to_string(),
                    description: "Temporarily increase database connection pool size".to_string(),
                    commands: vec![
                        "curl -H 'X-API-Key: $API_KEY' -X POST http://localhost:8090/api/database/pool/resize -d '{\"size\": 150}'".to_string(),
                    ],
                    expected_outcome: "More database connections available".to_string(),
                    risk_level: RiskLevel::Medium,
                    estimated_time: "1-2 minutes".to_string(),
                    prerequisites: vec!["API access".to_string()],
                    rollback_commands: vec![
                        "curl -H 'X-API-Key: $API_KEY' -X POST http://localhost:8090/api/database/pool/resize -d '{\"size\": 100}'".to_string(),
                    ],
                },
            ],
        });
        
        // Low region FPS
        self.add_rule(DiagnosticRule {
            id: "low_region_fps".to_string(),
            name: "Low Region FPS".to_string(),
            category: IssueCategory::Physics,
            condition: Box::new(|state| {
                state.region_fps.values().any(|&fps| fps < 30.0)
            }),
            issue_template: DetectedIssue {
                id: "low_region_fps".to_string(),
                category: IssueCategory::Physics,
                severity: IssueSeverity::Medium,
                title: "Low Region FPS Detected".to_string(),
                description: "One or more regions are running below 30 FPS".to_string(),
                affected_components: vec!["physics".to_string(), "regions".to_string()],
                symptoms: vec![
                    "Laggy avatar movement".to_string(),
                    "Slow object physics".to_string(),
                    "Poor user experience".to_string(),
                ],
                potential_causes: vec![
                    "Too many physics objects".to_string(),
                    "Complex physics calculations".to_string(),
                    "Inappropriate physics engine for content".to_string(),
                    "CPU resource constraints".to_string(),
                ],
                detection_time: chrono::Utc::now(),
                confidence: 0.85,
            },
            actions: vec![
                RecommendedAction {
                    id: "analyze_region_performance".to_string(),
                    title: "Analyze Region Performance".to_string(),
                    description: "Check region physics performance and object counts".to_string(),
                    commands: vec![
                        "curl -H 'X-API-Key: $API_KEY' http://localhost:8090/api/regions/performance".to_string(),
                        "curl -H 'X-API-Key: $API_KEY' http://localhost:8090/api/physics/engines/status".to_string(),
                    ],
                    expected_outcome: "Identify performance bottlenecks".to_string(),
                    risk_level: RiskLevel::Safe,
                    estimated_time: "2-5 minutes".to_string(),
                    prerequisites: vec!["API access".to_string()],
                    rollback_commands: vec![],
                },
                RecommendedAction {
                    id: "optimize_physics_engine".to_string(),
                    title: "Switch Physics Engine".to_string(),
                    description: "Switch to a more appropriate physics engine for the region".to_string(),
                    commands: vec![
                        "curl -H 'X-API-Key: $API_KEY' -X POST http://localhost:8090/api/regions/{region_id}/physics -d '{\"engine\": \"ODE\"}'".to_string(),
                    ],
                    expected_outcome: "Improved region FPS".to_string(),
                    risk_level: RiskLevel::Medium,
                    estimated_time: "1-2 minutes".to_string(),
                    prerequisites: vec!["API access".to_string(), "Region identification".to_string()],
                    rollback_commands: vec![
                        "curl -H 'X-API-Key: $API_KEY' -X POST http://localhost:8090/api/regions/{region_id}/physics -d '{\"engine\": \"Bullet\"}'".to_string(),
                    ],
                },
            ],
        });
    }
    
    pub fn add_rule(&mut self, rule: DiagnosticRule) {
        self.diagnostic_rules.push(rule);
    }
    
    pub async fn run_diagnostics(&mut self) -> DiagnosticReport {
        // Update system state
        self.update_system_state().await;
        
        // Run all diagnostic rules
        let mut detected_issues = Vec::new();
        let mut recommended_actions = Vec::new();
        
        for rule in &self.diagnostic_rules {
            if (rule.condition)(&self.system_state) {
                let mut issue = rule.issue_template.clone();
                issue.detection_time = chrono::Utc::now();
                
                // Customize issue based on current state
                self.customize_issue(&mut issue).await;
                
                detected_issues.push(issue);
                recommended_actions.extend(rule.actions.clone());
            }
        }
        
        // Calculate system health
        let system_health = self.calculate_system_health(&detected_issues);
        
        // Estimate performance impact
        let performance_impact = self.estimate_performance_impact(&detected_issues);
        
        DiagnosticReport {
            timestamp: chrono::Utc::now(),
            system_health,
            issues_detected: detected_issues,
            recommended_actions,
            performance_impact,
            resolution_status: if detected_issues.is_empty() {
                ResolutionStatus::Resolved
            } else {
                ResolutionStatus::ActionRequired
            },
        }
    }
    
    async fn update_system_state(&mut self) {
        // Collect current system metrics
        // This would integrate with the monitoring system
        
        // Example: Get CPU usage
        if let Ok(output) = Command::new("cat")
            .arg("/proc/loadavg")
            .output()
            .await
        {
            if let Ok(content) = String::from_utf8(output.stdout) {
                if let Some(load_avg) = content.split_whitespace().next() {
                    if let Ok(load) = load_avg.parse::<f64>() {
                        self.system_state.cpu_usage = (load * 100.0).min(100.0);
                    }
                }
            }
        }
        
        // Example: Get memory usage
        if let Ok(output) = Command::new("free")
            .arg("-m")
            .output()
            .await
        {
            if let Ok(content) = String::from_utf8(output.stdout) {
                // Parse memory information
                // This is simplified - real implementation would be more robust
                for line in content.lines() {
                    if line.starts_with("Mem:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            if let (Ok(total), Ok(used)) = (parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                                self.system_state.memory_usage = (used / total) * 100.0;
                            }
                        }
                        break;
                    }
                }
            }
        }
        
        // Get application-specific metrics via API
        // This would make HTTP requests to the monitoring endpoints
        // For brevity, using example values
        self.system_state.active_connections = 150;
        self.system_state.database_connections = 85;
        self.system_state.response_time_p95 = 250.0;
        self.system_state.error_rate = 0.5;
        self.system_state.websocket_connections = 45;
        self.system_state.ziti_connections = 12;
        
        // Example region FPS data
        self.system_state.region_fps.insert("region-1".to_string(), 25.0);
        self.system_state.region_fps.insert("region-2".to_string(), 58.0);
    }
    
    async fn customize_issue(&self, issue: &mut DetectedIssue) {
        match issue.category {
            IssueCategory::Performance => {
                if self.system_state.cpu_usage > 95.0 {
                    issue.severity = IssueSeverity::Critical;
                    issue.description = format!(
                        "CPU usage is critically high at {:.1}%, immediate action required",
                        self.system_state.cpu_usage
                    );
                }
            }
            IssueCategory::Resource => {
                if self.system_state.memory_usage > 95.0 {
                    issue.severity = IssueSeverity::Critical;
                    issue.description = format!(
                        "Memory usage is critically high at {:.1}%, system may crash",
                        self.system_state.memory_usage
                    );
                }
            }
            IssueCategory::Physics => {
                let low_fps_regions: Vec<_> = self.system_state.region_fps
                    .iter()
                    .filter(|(_, &fps)| fps < 30.0)
                    .map(|(name, fps)| format!("{}: {:.1} FPS", name, fps))
                    .collect();
                
                issue.description = format!(
                    "Regions with low FPS: {}",
                    low_fps_regions.join(", ")
                );
                issue.affected_components = self.system_state.region_fps
                    .iter()
                    .filter(|(_, &fps)| fps < 30.0)
                    .map(|(name, _)| name.clone())
                    .collect();
            }
            _ => {}
        }
    }
    
    fn calculate_system_health(&self, issues: &[DetectedIssue]) -> SystemHealth {
        let critical_issues = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count() as u32;
        let high_issues = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::High)).count() as u32;
        let medium_issues = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Medium)).count() as u32;
        let warnings = high_issues + medium_issues;
        
        // Calculate overall health score
        let health_penalty = (critical_issues * 30) + (high_issues * 15) + (medium_issues * 5);
        let overall_score = (100.0 - health_penalty as f64).max(0.0);
        
        // Calculate component scores
        let mut component_scores = HashMap::new();
        component_scores.insert("server".to_string(), if self.system_state.cpu_usage < 80.0 && self.system_state.memory_usage < 85.0 { 95.0 } else { 60.0 });
        component_scores.insert("database".to_string(), if self.system_state.database_connections < 80 { 90.0 } else { 70.0 });
        component_scores.insert("physics".to_string(), {
            let avg_fps = self.system_state.region_fps.values().sum::<f64>() / self.system_state.region_fps.len() as f64;
            if avg_fps > 50.0 { 95.0 } else if avg_fps > 30.0 { 75.0 } else { 50.0 }
        });
        component_scores.insert("websocket".to_string(), if self.system_state.websocket_connections < 900 { 90.0 } else { 70.0 });
        
        let healthy_components = component_scores.values().filter(|&&score| score > 80.0).count() as u32;
        
        SystemHealth {
            overall_score,
            component_scores,
            critical_issues,
            warnings,
            healthy_components,
        }
    }
    
    fn estimate_performance_impact(&self, issues: &[DetectedIssue]) -> PerformanceImpact {
        let mut response_time_degradation = 0.0;
        let mut throughput_reduction = 0.0;
        let mut affected_regions = Vec::new();
        
        for issue in issues {
            match issue.category {
                IssueCategory::Performance => {
                    response_time_degradation += match issue.severity {
                        IssueSeverity::Critical => 50.0,
                        IssueSeverity::High => 30.0,
                        IssueSeverity::Medium => 15.0,
                        _ => 5.0,
                    };
                }
                IssueCategory::Resource => {
                    throughput_reduction += match issue.severity {
                        IssueSeverity::Critical => 40.0,
                        IssueSeverity::High => 25.0,
                        IssueSeverity::Medium => 10.0,
                        _ => 3.0,
                    };
                }
                IssueCategory::Physics => {
                    affected_regions.extend(issue.affected_components.clone());
                }
                _ => {}
            }
        }
        
        let user_experience_impact = if response_time_degradation > 40.0 || throughput_reduction > 30.0 {
            "Severe - Users will experience significant lag and poor responsiveness".to_string()
        } else if response_time_degradation > 20.0 || throughput_reduction > 15.0 {
            "Moderate - Users may notice slower performance".to_string()
        } else if response_time_degradation > 10.0 || throughput_reduction > 5.0 {
            "Minor - Slight performance degradation may be noticeable".to_string()
        } else {
            "Minimal - Little to no impact on user experience".to_string()
        };
        
        PerformanceImpact {
            response_time_degradation,
            throughput_reduction,
            user_experience_impact,
            affected_regions,
        }
    }
}
```

---

## Conclusion

This comprehensive **Monitoring and Administration Setup Guide** provides enterprise-grade monitoring capabilities for OpenSim Next production deployments. The guide covers all essential aspects of monitoring, from basic metrics collection to advanced capacity planning and automated troubleshooting.

### Key Features Documented

🔍 **Complete Monitoring Stack**: Prometheus, Grafana, AlertManager, and custom metrics  
📊 **Real-Time Statistics**: WebSocket-based live monitoring with interactive dashboards  
🗄️ **Database Monitoring**: PostgreSQL performance tracking with custom queries  
🔒 **Security Monitoring**: Zero trust network monitoring and security event tracking  
🚨 **Intelligent Alerting**: Multi-channel alert delivery with escalation policies  
📈 **Capacity Planning**: Predictive scaling recommendations and cost analysis  
🔧 **Automated Troubleshooting**: Self-diagnosing system with recommended actions  
📱 **Mobile Administration**: Responsive admin interface for on-the-go management  

### Production-Ready Features

- **Enterprise Scalability**: Supports monitoring of large multi-region deployments
- **High Availability**: Redundant monitoring with failover capabilities  
- **Cost Optimization**: Intelligent capacity planning with cost-benefit analysis
- **Security Integration**: Complete zero trust network monitoring
- **Performance Intelligence**: Machine learning-enhanced anomaly detection
- **Automated Response**: Self-healing capabilities with automated remediation

This monitoring system ensures OpenSim Next deployments maintain optimal performance, security, and reliability in production environments, making it suitable for enterprise virtual world hosting at any scale.

*Last updated: December 2024 - OpenSim Next v1.0.0*