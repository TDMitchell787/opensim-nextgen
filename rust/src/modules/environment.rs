use std::any::Any;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::session::SessionManager;

use super::services::ServiceRegistry;
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};
use crate::udp::server::AvatarMovementState;

pub trait IEnvironmentModule: Send + Sync + 'static {
    fn get_sun_direction(&self) -> [f32; 3];
    fn get_sun_phase(&self) -> f32;
    fn set_sun_fixed(&self, fixed: bool, sun_hour: f32);
    fn get_region_environment(&self) -> Option<Vec<u8>>;
}

const DEFAULT_DAY_CYCLE: f64 = 14400.0; // 4 hours in seconds (SL default)

struct EnvironmentConfig {
    use_fixed_sun: bool,
    fixed_sun_hour: f32,
    day_cycle_length: f64,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            use_fixed_sun: false,
            fixed_sun_hour: 6.0,
            day_cycle_length: DEFAULT_DAY_CYCLE,
        }
    }
}

pub struct EnvironmentModule {
    config: Arc<RwLock<EnvironmentConfig>>,
    region_uuid: Option<Uuid>,
    socket: Option<Arc<tokio::net::UdpSocket>>,
    session_manager: Option<Arc<SessionManager>>,
    avatar_states: Option<Arc<RwLock<std::collections::HashMap<Uuid, AvatarMovementState>>>>,
    service_registry: Option<Arc<RwLock<ServiceRegistry>>>,
}

impl EnvironmentModule {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(EnvironmentConfig::default())),
            region_uuid: None,
            socket: None,
            session_manager: None,
            avatar_states: None,
            service_registry: None,
        }
    }

    fn compute_sun_direction(config: &EnvironmentConfig) -> ([f32; 3], f32) {
        let phase = if config.use_fixed_sun {
            config.fixed_sun_hour / 24.0
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();
            ((now % config.day_cycle_length) / config.day_cycle_length) as f32
        };

        let angle = phase * 2.0 * std::f32::consts::PI;

        let sun_x = angle.cos();
        let sun_y = 0.0f32;
        let sun_z = angle.sin();

        ([sun_x, sun_y, sun_z], phase)
    }
}

impl IEnvironmentModule for EnvironmentModule {
    fn get_sun_direction(&self) -> [f32; 3] {
        let config = self.config.read();
        let (dir, _) = Self::compute_sun_direction(&config);
        dir
    }

    fn get_sun_phase(&self) -> f32 {
        let config = self.config.read();
        let (_, phase) = Self::compute_sun_direction(&config);
        phase
    }

    fn set_sun_fixed(&self, fixed: bool, sun_hour: f32) {
        let mut config = self.config.write();
        config.use_fixed_sun = fixed;
        config.fixed_sun_hour = sun_hour;
    }

    fn get_region_environment(&self) -> Option<Vec<u8>> {
        None // Default Windlight
    }
}

const SIMULATOR_VIEWER_TIME_ID: u32 = 0xFFFF0096; // Low 150

fn build_viewer_time_message(
    sun_direction: &[f32; 3],
    sun_phase: f32,
    sun_angular_velocity: f32,
) -> Vec<u8> {
    let mut packet = Vec::with_capacity(40);
    packet.push(0x40);
    packet.extend_from_slice(&0u32.to_be_bytes());
    packet.push(0);
    packet.extend_from_slice(&[0xFF, 0xFF]);
    packet.extend_from_slice(&0x0096u16.to_be_bytes());

    // TimeInfo block
    let usec_since_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64;
    packet.extend_from_slice(&usec_since_start.to_le_bytes());

    let seconds_per_day = 14400u32;
    packet.extend_from_slice(&seconds_per_day.to_le_bytes());

    let seconds_per_year = seconds_per_day * 365;
    packet.extend_from_slice(&seconds_per_year.to_le_bytes());

    for &v in sun_direction {
        packet.extend_from_slice(&v.to_le_bytes());
    }

    packet.extend_from_slice(&sun_phase.to_le_bytes());

    let sun_ang_vel = [0.0f32, 0.0, sun_angular_velocity];
    for &v in &sun_ang_vel {
        packet.extend_from_slice(&v.to_le_bytes());
    }

    packet
}

#[async_trait]
impl RegionModule for EnvironmentModule {
    fn name(&self) -> &'static str {
        "EnvironmentModule"
    }
    fn replaceable_interface(&self) -> Option<&'static str> {
        Some("IEnvironmentModule")
    }

    async fn initialize(&mut self, config: &ModuleConfig) -> Result<()> {
        let mut env_config = self.config.write();
        env_config.use_fixed_sun = config.get_bool("use_fixed_sun", false);
        env_config.fixed_sun_hour = config.get_f32("fixed_sun_hour", 6.0);
        env_config.day_cycle_length =
            config.get_f32("day_cycle_length", DEFAULT_DAY_CYCLE as f32) as f64;
        info!(
            "[ENVIRONMENT MODULE] Initialized (fixed_sun={}, cycle={:.0}s)",
            env_config.use_fixed_sun, env_config.day_cycle_length
        );
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = Some(scene.region_uuid);
        self.socket = Some(scene.socket.clone());
        self.session_manager = Some(scene.session_manager.clone());
        self.avatar_states = Some(scene.avatar_states.clone());
        self.service_registry = Some(scene.service_registry.clone());

        scene
            .service_registry
            .write()
            .register::<EnvironmentModule>(Arc::new(EnvironmentModule {
                config: self.config.clone(),
                region_uuid: self.region_uuid,
                socket: self.socket.clone(),
                session_manager: self.session_manager.clone(),
                avatar_states: self.avatar_states.clone(),
                service_registry: self.service_registry.clone(),
            }));

        info!(
            "[ENVIRONMENT MODULE] Added to region {:?}",
            scene.region_name
        );
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[async_trait]
impl SharedRegionModule for EnvironmentModule {}
