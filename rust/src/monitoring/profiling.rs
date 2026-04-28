//! Performance profiling for OpenSim
//!
//! Tracks execution times, memory usage, and performance bottlenecks.

use anyhow::Result;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::debug;

/// Performance profiler for tracking execution times and bottlenecks
pub struct PerformanceProfiler {
    /// Performance events
    events: Arc<RwLock<Vec<PerformanceEvent>>>,
    /// Performance profiles
    profiles: Arc<RwLock<HashMap<String, PerformanceProfile>>>,
    /// Sampling rate (0.0 to 1.0)
    sample_rate: f64,
    /// Whether profiling is active
    active: Arc<RwLock<bool>>,
    /// Maximum number of events to retain
    max_events: usize,
}

/// Performance event
#[derive(Debug, Clone)]
pub struct PerformanceEvent {
    /// Event name
    pub name: String,
    /// Event type
    pub event_type: EventType,
    /// Start time
    pub start_time: Instant,
    /// Duration
    pub duration: Duration,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl serde::Serialize for PerformanceEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PerformanceEvent", 5)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("event_type", &self.event_type)?;
        state.serialize_field("start_time_nanos", &self.start_time.elapsed().as_nanos())?;
        state.serialize_field("duration_nanos", &self.duration.as_nanos())?;
        state.serialize_field("metadata", &self.metadata)?;
        state.end()
    }
}

/// Type of performance event
#[derive(Debug, Clone, serde::Serialize)]
pub enum EventType {
    /// Function call
    FunctionCall,
    /// Database query
    DatabaseQuery,
    /// Network request
    NetworkRequest,
    /// Physics simulation
    PhysicsSimulation,
    /// Asset loading
    AssetLoading,
    /// Custom event
    Custom,
}

/// Performance profile for a specific operation
#[derive(Debug, Clone)]
pub struct PerformanceProfile {
    /// Operation name
    pub name: String,
    /// Total execution count
    pub total_count: u64,
    /// Total execution time
    pub total_time: Duration,
    /// Average execution time
    pub avg_time: Duration,
    /// Minimum execution time
    pub min_time: Duration,
    /// Maximum execution time
    pub max_time: Duration,
    /// Standard deviation
    pub std_dev: f64,
    /// Recent events (last 100)
    pub recent_events: Vec<PerformanceEvent>,
}

impl serde::Serialize for PerformanceProfile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PerformanceProfile", 8)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("total_count", &self.total_count)?;
        state.serialize_field("total_time_nanos", &self.total_time.as_nanos())?;
        state.serialize_field("avg_time_nanos", &self.avg_time.as_nanos())?;
        state.serialize_field("min_time_nanos", &self.min_time.as_nanos())?;
        state.serialize_field("max_time_nanos", &self.max_time.as_nanos())?;
        state.serialize_field("std_dev", &self.std_dev)?;
        state.serialize_field("recent_events", &self.recent_events)?;
        state.end()
    }
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new(sample_rate: f64) -> Result<Self> {
        Ok(Self {
            events: Arc::new(RwLock::new(Vec::new())),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            sample_rate,
            active: Arc::new(RwLock::new(false)),
            max_events: 10000,
        })
    }

    /// Start profiling
    pub async fn start_profiling(&self) -> Result<()> {
        *self.active.write().await = true;
        debug!("Performance profiling started");
        Ok(())
    }

    /// Stop profiling
    pub async fn stop_profiling(&self) -> Result<()> {
        *self.active.write().await = false;
        debug!("Performance profiling stopped");
        Ok(())
    }

    /// Record a performance event
    pub async fn record_event(&self, event: PerformanceEvent) -> Result<()> {
        // Apply sampling if profiling is active
        if !*self.active.read().await {
            return Ok(());
        }

        // Random sampling based on sample rate
        if rand::random::<f64>() > self.sample_rate {
            return Ok(());
        }

        let mut events = self.events.write().await;
        events.push(event.clone());

        // Maintain event limit
        while events.len() > self.max_events {
            events.remove(0);
        }

        // Update performance profile
        self.update_profile(event.clone()).await?;

        debug!(
            "Recorded performance event: {} ({:?})",
            event.name, event.duration
        );
        Ok(())
    }

    /// Update performance profile for an event
    async fn update_profile(&self, event: PerformanceEvent) -> Result<()> {
        let mut profiles = self.profiles.write().await;

        let profile = profiles
            .entry(event.name.clone())
            .or_insert_with(|| PerformanceProfile {
                name: event.name.clone(),
                total_count: 0,
                total_time: Duration::ZERO,
                avg_time: Duration::ZERO,
                min_time: Duration::MAX,
                max_time: Duration::ZERO,
                std_dev: 0.0,
                recent_events: Vec::new(),
            });

        // Update statistics
        profile.total_count += 1;
        profile.total_time += event.duration;
        profile.avg_time =
            Duration::from_nanos(profile.total_time.as_nanos() as u64 / profile.total_count);

        if event.duration < profile.min_time {
            profile.min_time = event.duration;
        }
        if event.duration > profile.max_time {
            profile.max_time = event.duration;
        }

        // Update recent events
        profile.recent_events.push(event.clone());
        if profile.recent_events.len() > 100 {
            profile.recent_events.remove(0);
        }

        // Calculate standard deviation
        self.calculate_std_dev(profile).await;

        Ok(())
    }

    /// Calculate standard deviation for a profile
    async fn calculate_std_dev(&self, profile: &mut PerformanceProfile) {
        if profile.recent_events.len() < 2 {
            profile.std_dev = 0.0;
            return;
        }

        let avg_nanos = profile.avg_time.as_nanos() as f64;
        let variance: f64 = profile
            .recent_events
            .iter()
            .map(|event| {
                let diff = event.duration.as_nanos() as f64 - avg_nanos;
                diff * diff
            })
            .sum::<f64>()
            / profile.recent_events.len() as f64;

        profile.std_dev = variance.sqrt();
    }

    /// Get performance profile for a specific operation
    pub async fn get_profile(&self, operation_name: &str) -> Result<Option<PerformanceProfile>> {
        let profiles = self.profiles.read().await;
        Ok(profiles.get(operation_name).cloned())
    }

    /// Get all performance profiles
    pub async fn get_all_profiles(&self) -> Result<Vec<PerformanceProfile>> {
        let profiles = self.profiles.read().await;
        Ok(profiles.values().cloned().collect())
    }

    /// Get performance events
    pub async fn get_events(&self, limit: Option<usize>) -> Result<Vec<PerformanceEvent>> {
        let events = self.events.read().await;
        let limit = limit.unwrap_or(events.len());
        Ok(events.iter().rev().take(limit).cloned().collect())
    }

    /// Get events for a specific operation
    pub async fn get_events_for_operation(
        &self,
        operation_name: &str,
    ) -> Result<Vec<PerformanceEvent>> {
        let events = self.events.read().await;
        Ok(events
            .iter()
            .filter(|event| event.name == operation_name)
            .cloned()
            .collect())
    }

    /// Clear all profiling data
    pub async fn clear_data(&self) -> Result<()> {
        self.events.write().await.clear();
        self.profiles.write().await.clear();
        debug!("Performance profiling data cleared");
        Ok(())
    }

    /// Get profiling statistics
    pub async fn get_stats(&self) -> Result<ProfilingStats> {
        let events = self.events.read().await;
        let profiles = self.profiles.read().await;
        let active = *self.active.read().await;

        Ok(ProfilingStats {
            total_events: events.len(),
            total_profiles: profiles.len(),
            active,
            sample_rate: self.sample_rate,
        })
    }

    /// Export profiling data in JSON format
    pub async fn export_json(&self) -> Result<String> {
        let profiles = self.get_all_profiles().await?;
        let events = self.get_events(Some(1000)).await?; // Last 1000 events

        let export_data = ProfilingExport {
            profiles,
            recent_events: events,
            export_time: chrono::Utc::now(),
        };

        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    /// Find performance bottlenecks
    pub async fn find_bottlenecks(&self) -> Result<Vec<PerformanceBottleneck>> {
        let profiles = self.get_all_profiles().await?;
        let mut bottlenecks = Vec::new();

        for profile in profiles {
            // Check for slow operations (avg time > 100ms)
            if profile.avg_time > Duration::from_millis(100) {
                bottlenecks.push(PerformanceBottleneck {
                    operation: profile.name.clone(),
                    issue: "Slow average execution time".to_string(),
                    severity: BottleneckSeverity::High,
                    avg_time: profile.avg_time,
                    suggestion: "Consider optimization or caching".to_string(),
                });
            }

            // Check for high variance (std dev > 50% of avg)
            if profile.std_dev > profile.avg_time.as_nanos() as f64 * 0.5 {
                bottlenecks.push(PerformanceBottleneck {
                    operation: profile.name.clone(),
                    issue: "High execution time variance".to_string(),
                    severity: BottleneckSeverity::Medium,
                    avg_time: profile.avg_time,
                    suggestion: "Investigate inconsistent performance".to_string(),
                });
            }

            // Check for frequent operations (high count with significant time)
            if profile.total_count > 1000 && profile.total_time > Duration::from_secs(10) {
                bottlenecks.push(PerformanceBottleneck {
                    operation: profile.name.clone(),
                    issue: "Frequent operation with high total time".to_string(),
                    severity: BottleneckSeverity::Medium,
                    avg_time: profile.avg_time,
                    suggestion: "Consider batching or optimization".to_string(),
                });
            }
        }

        Ok(bottlenecks)
    }
}

/// Performance profiling statistics
#[derive(Debug, Clone)]
pub struct ProfilingStats {
    /// Total number of events recorded
    pub total_events: usize,
    /// Total number of profiles
    pub total_profiles: usize,
    /// Whether profiling is active
    pub active: bool,
    /// Current sampling rate
    pub sample_rate: f64,
}

/// Performance bottleneck information
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    /// Operation name
    pub operation: String,
    /// Issue description
    pub issue: String,
    /// Severity level
    pub severity: BottleneckSeverity,
    /// Average execution time
    pub avg_time: Duration,
    /// Suggested improvement
    pub suggestion: String,
}

/// Bottleneck severity level
#[derive(Debug, Clone)]
pub enum BottleneckSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Profiling data export structure
#[derive(Debug, serde::Serialize)]
struct ProfilingExport {
    profiles: Vec<PerformanceProfile>,
    recent_events: Vec<PerformanceEvent>,
    export_time: chrono::DateTime<chrono::Utc>,
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new(0.1).expect("Failed to create PerformanceProfiler")
    }
}
