use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::SystemTime;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimDenialReason {
    AgentPerSecond,
    AgentPerMinute,
    AgentTotal,
    RegionTotal,
    PhysicalLimit,
}

impl std::fmt::Display for PrimDenialReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgentPerSecond => write!(f, "Too many prims created per second"),
            Self::AgentPerMinute => write!(f, "Too many prims created per minute"),
            Self::AgentTotal => write!(f, "Agent prim limit reached"),
            Self::RegionTotal => write!(f, "Region prim limit reached"),
            Self::PhysicalLimit => write!(f, "Physical prim limit reached"),
        }
    }
}

struct AgentPrimStats {
    prims_this_second: AtomicU32,
    prims_this_minute: AtomicU32,
    total_prims: AtomicU32,
    second_epoch: AtomicU32,
    minute_epoch: AtomicU32,
}

impl AgentPrimStats {
    fn new(now: u32) -> Self {
        Self {
            prims_this_second: AtomicU32::new(0),
            prims_this_minute: AtomicU32::new(0),
            total_prims: AtomicU32::new(0),
            second_epoch: AtomicU32::new(now),
            minute_epoch: AtomicU32::new(now),
        }
    }
}

pub struct PrimCreationLimiter {
    agent_stats: DashMap<Uuid, AgentPrimStats>,
    region_prim_count: AtomicU32,
    physical_prim_count: AtomicU32,
    max_prims_per_second: u32,
    max_prims_per_minute: u32,
    max_prims_per_agent: u32,
    max_prims_per_region: u32,
    max_physical_prims: u32,
}

impl PrimCreationLimiter {
    pub fn new() -> Self {
        Self {
            agent_stats: DashMap::new(),
            region_prim_count: AtomicU32::new(0),
            physical_prim_count: AtomicU32::new(0),
            max_prims_per_second: 10,
            max_prims_per_minute: 100,
            max_prims_per_agent: std::env::var("OPENSIM_MAX_AGENT_PRIMS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),
            max_prims_per_region: std::env::var("OPENSIM_MAX_PRIMS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(15000),
            max_physical_prims: std::env::var("OPENSIM_MAX_PHYSICAL_PRIMS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        }
    }

    pub fn check_prim_creation(
        &self,
        agent_id: Uuid,
        is_physical: bool,
    ) -> Result<(), PrimDenialReason> {
        let region_count = self.region_prim_count.load(Ordering::Relaxed);
        if region_count >= self.max_prims_per_region {
            return Err(PrimDenialReason::RegionTotal);
        }

        if is_physical {
            let phys_count = self.physical_prim_count.load(Ordering::Relaxed);
            if phys_count >= self.max_physical_prims {
                return Err(PrimDenialReason::PhysicalLimit);
            }
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        let stats = self
            .agent_stats
            .entry(agent_id)
            .or_insert_with(|| AgentPrimStats::new(now));

        let stored_sec = stats.second_epoch.load(Ordering::Relaxed);
        if now != stored_sec {
            stats.prims_this_second.store(0, Ordering::Relaxed);
            stats.second_epoch.store(now, Ordering::Relaxed);
        }

        let stored_min = stats.minute_epoch.load(Ordering::Relaxed);
        if now.wrapping_sub(stored_min) >= 60 {
            stats.prims_this_minute.store(0, Ordering::Relaxed);
            stats.minute_epoch.store(now, Ordering::Relaxed);
        }

        let pps = stats.prims_this_second.load(Ordering::Relaxed);
        if pps >= self.max_prims_per_second {
            return Err(PrimDenialReason::AgentPerSecond);
        }

        let ppm = stats.prims_this_minute.load(Ordering::Relaxed);
        if ppm >= self.max_prims_per_minute {
            return Err(PrimDenialReason::AgentPerMinute);
        }

        let total = stats.total_prims.load(Ordering::Relaxed);
        if total >= self.max_prims_per_agent {
            return Err(PrimDenialReason::AgentTotal);
        }

        stats.prims_this_second.fetch_add(1, Ordering::Relaxed);
        stats.prims_this_minute.fetch_add(1, Ordering::Relaxed);
        stats.total_prims.fetch_add(1, Ordering::Relaxed);
        self.region_prim_count.fetch_add(1, Ordering::Relaxed);
        if is_physical {
            self.physical_prim_count.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }

    pub fn record_prim_deleted(&self, agent_id: Uuid, is_physical: bool) {
        self.region_prim_count.fetch_sub(1, Ordering::Relaxed);
        if is_physical {
            self.physical_prim_count.fetch_sub(1, Ordering::Relaxed);
        }
        if let Some(stats) = self.agent_stats.get(&agent_id) {
            let prev = stats.total_prims.load(Ordering::Relaxed);
            if prev > 0 {
                stats.total_prims.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    pub fn set_region_count(&self, count: u32) {
        self.region_prim_count.store(count, Ordering::Relaxed);
    }

    pub fn get_region_count(&self) -> u32 {
        self.region_prim_count.load(Ordering::Relaxed)
    }

    pub fn get_physical_count(&self) -> u32 {
        self.physical_prim_count.load(Ordering::Relaxed)
    }

    pub fn get_limits(&self) -> PrimLimitsSummary {
        PrimLimitsSummary {
            region_count: self.region_prim_count.load(Ordering::Relaxed),
            region_max: self.max_prims_per_region,
            physical_count: self.physical_prim_count.load(Ordering::Relaxed),
            physical_max: self.max_physical_prims,
            agent_max: self.max_prims_per_agent,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PrimLimitsSummary {
    pub region_count: u32,
    pub region_max: u32,
    pub physical_count: u32,
    pub physical_max: u32,
    pub agent_max: u32,
}
