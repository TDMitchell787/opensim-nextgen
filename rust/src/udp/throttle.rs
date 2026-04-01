use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, Duration};
use dashmap::DashMap;
use tracing::{warn, debug};

#[derive(Debug, Clone, Copy)]
pub enum ThrottleCategory {
    Resend = 0,
    Land = 1,
    Wind = 2,
    Cloud = 3,
    Task = 4,
    Texture = 5,
    Asset = 6,
}

impl ThrottleCategory {
    pub const COUNT: usize = 7;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThrottleDecision {
    Allow,
    Throttle,
    Drop,
    Disconnect,
}

pub struct AgentTrafficStats {
    pub declared_throttles: [f32; ThrottleCategory::COUNT],
    pub bytes_sent: [AtomicU64; ThrottleCategory::COUNT],
    pub bytes_received: AtomicU64,
    pub packets_received: AtomicU64,
    pub window_start: AtomicU64,
    pub total_declared_bps: f32,
}

impl AgentTrafficStats {
    fn new() -> Self {
        const DEFAULT_THROTTLE: f32 = 50000.0;
        Self {
            declared_throttles: [DEFAULT_THROTTLE; ThrottleCategory::COUNT],
            bytes_sent: std::array::from_fn(|_| AtomicU64::new(0)),
            bytes_received: AtomicU64::new(0),
            packets_received: AtomicU64::new(0),
            window_start: AtomicU64::new(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            ),
            total_declared_bps: DEFAULT_THROTTLE * ThrottleCategory::COUNT as f32,
        }
    }

    fn reset_window(&self, now_secs: u64) {
        for b in &self.bytes_sent {
            b.store(0, Ordering::Relaxed);
        }
        self.bytes_received.store(0, Ordering::Relaxed);
        self.packets_received.store(0, Ordering::Relaxed);
        self.window_start.store(now_secs, Ordering::Relaxed);
    }
}

pub struct ThrottleManager {
    agents: DashMap<u32, AgentTrafficStats>,
    overdraft_multiplier: f32,
}

impl ThrottleManager {
    pub fn new() -> Self {
        Self {
            agents: DashMap::new(),
            overdraft_multiplier: 5.0,
        }
    }

    pub fn register_agent(&self, circuit_code: u32) {
        self.agents.entry(circuit_code).or_insert_with(AgentTrafficStats::new);
    }

    pub fn remove_agent(&self, circuit_code: u32) {
        self.agents.remove(&circuit_code);
    }

    pub fn set_throttles(&self, circuit_code: u32, throttle_data: &[u8]) {
        if throttle_data.len() < 28 {
            debug!("[THROTTLE] Throttle data too short: {} bytes (need 28)", throttle_data.len());
            return;
        }

        let mut values = [0f32; ThrottleCategory::COUNT];
        for i in 0..ThrottleCategory::COUNT {
            let offset = i * 4;
            if offset + 4 <= throttle_data.len() {
                values[i] = f32::from_le_bytes([
                    throttle_data[offset],
                    throttle_data[offset + 1],
                    throttle_data[offset + 2],
                    throttle_data[offset + 3],
                ]);
            }
        }

        let total: f32 = values.iter().sum();
        debug!("[THROTTLE] Circuit {} declared throttles: resend={:.0} land={:.0} wind={:.0} cloud={:.0} task={:.0} texture={:.0} asset={:.0} total={:.0} bps",
               circuit_code, values[0], values[1], values[2], values[3], values[4], values[5], values[6], total);

        if let Some(mut stats) = self.agents.get_mut(&circuit_code) {
            stats.declared_throttles = values;
            stats.total_declared_bps = total;
        } else {
            let mut stats = AgentTrafficStats::new();
            stats.declared_throttles = values;
            stats.total_declared_bps = total;
            self.agents.insert(circuit_code, stats);
        }
    }

    pub fn record_packet(&self, circuit_code: u32, packet_size: usize) -> ThrottleDecision {
        let Some(stats) = self.agents.get(&circuit_code) else {
            return ThrottleDecision::Allow;
        };

        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let window_start = stats.window_start.load(Ordering::Relaxed);
        if now_secs - window_start >= 10 {
            stats.reset_window(now_secs);
        }

        let received = stats.bytes_received.fetch_add(packet_size as u64, Ordering::Relaxed) + packet_size as u64;
        stats.packets_received.fetch_add(1, Ordering::Relaxed);

        let elapsed = (now_secs - window_start).max(1) as f32;
        let current_bps = received as f32 / elapsed;
        let limit_bps = stats.total_declared_bps * self.overdraft_multiplier;

        if current_bps > limit_bps * 2.0 {
            warn!("[THROTTLE] Circuit {} EXTREME overdraft: {:.0} bps vs {:.0} limit — DISCONNECT",
                  circuit_code, current_bps, limit_bps);
            return ThrottleDecision::Disconnect;
        }

        if current_bps > limit_bps {
            debug!("[THROTTLE] Circuit {} overdraft: {:.0} bps vs {:.0} limit — DROP",
                   circuit_code, current_bps, limit_bps);
            return ThrottleDecision::Drop;
        }

        if current_bps > stats.total_declared_bps * 2.0 {
            return ThrottleDecision::Throttle;
        }

        ThrottleDecision::Allow
    }

    pub fn get_agent_stats(&self, circuit_code: u32) -> Option<AgentTrafficSummary> {
        let stats = self.agents.get(&circuit_code)?;
        Some(AgentTrafficSummary {
            circuit_code,
            declared_bps: stats.total_declared_bps,
            bytes_received: stats.bytes_received.load(Ordering::Relaxed),
            packets_received: stats.packets_received.load(Ordering::Relaxed),
        })
    }

    pub fn get_all_stats(&self) -> Vec<AgentTrafficSummary> {
        self.agents.iter().map(|entry| {
            AgentTrafficSummary {
                circuit_code: *entry.key(),
                declared_bps: entry.value().total_declared_bps,
                bytes_received: entry.value().bytes_received.load(Ordering::Relaxed),
                packets_received: entry.value().packets_received.load(Ordering::Relaxed),
            }
        }).collect()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentTrafficSummary {
    pub circuit_code: u32,
    pub declared_bps: f32,
    pub bytes_received: u64,
    pub packets_received: u64,
}
