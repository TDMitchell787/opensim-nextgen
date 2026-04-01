//! Main simulation engine for OpenSim
//! 
//! This module orchestrates the entire simulation loop, including region updates,
//! physics steps, and network synchronization.

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{self, Duration, Instant};
use crate::region::{RegionManager, RegionError};
use crate::state::StateManager;

/// Simulation configuration parameters
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Simulation tick rate (Hz)
    pub tick_rate: u32,
    /// Maximum steps per tick
    pub max_steps: u32,
    /// Enable profiling
    pub profiling: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            tick_rate: 60,
            max_steps: 1,
            profiling: false,
        }
    }
}

/// Simulation engine for a region
pub struct SimulationEngine {
    /// Region manager
    region_manager: Arc<RegionManager>,
    /// State manager
    state_manager: Arc<StateManager>,
    /// Simulation configuration
    config: SimulationConfig,
    /// Running flag
    running: Arc<Mutex<bool>>,
}

impl SimulationEngine {
    /// Create a new simulation engine
    pub fn new(
        region_manager: Arc<RegionManager>,
        state_manager: Arc<StateManager>,
    ) -> Self {
        Self {
            region_manager,
            state_manager,
            config: SimulationConfig::default(),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the simulation loop (spawns a background task)
    pub async fn start(self: &Arc<Self>) -> Result<(), RegionError> {
        let this = Arc::clone(self);
        let tick_duration = Duration::from_secs_f64(1.0 / this.config.tick_rate as f64);
        tokio::spawn(async move {
            let running = this.running.clone();
            let region_manager = this.region_manager.clone();
            let config = this.config.clone();
            let mut interval = time::interval(tick_duration);
            let mut last_tick = Instant::now();
            *running.lock().await = true;

            while *running.lock().await {
                interval.tick().await;
                let now = Instant::now();
                let delta = now.duration_since(last_tick);
                last_tick = now;
                let delta_time = delta.as_secs_f32();

                // Profiling start
                let tick_start = Instant::now();

                // Update all regions
                if let Err(e) = region_manager.update_all(delta_time).await {
                    eprintln!("[Simulation] Error updating regions: {e}");
                }

                // Profiling end
                if config.profiling {
                    let elapsed = tick_start.elapsed();
                    println!("[Simulation] Tick took {:.3} ms", elapsed.as_secs_f64() * 1000.0);
                }
            }
        });
        Ok(())
    }

    /// Stop the simulation loop
    pub async fn stop(&self) {
        *self.running.lock().await = false;
    }

    /// Shutdown the simulation engine
    pub async fn shutdown(&self) -> Result<(), RegionError> {
        self.stop().await;
        Ok(())
    }

    /// Check if the simulation is running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::{RegionManager, RegionConfig, terrain, simulation};
    use crate::ffi::physics::PhysicsBridge;
    use crate::state::StateManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_simulation_engine_start_stop() {
        let physics_bridge = Arc::new(PhysicsBridge::new().unwrap());
        let state_manager = Arc::new(StateManager::new().unwrap());
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        let _config = SimulationConfig::default();
        let engine = Arc::new(SimulationEngine::new(region_manager, state_manager));

        engine.start().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(engine.is_running().await);
        engine.stop().await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!engine.is_running().await);
    }
} 