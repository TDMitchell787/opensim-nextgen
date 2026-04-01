use std::any::Any;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::info;
use uuid::Uuid;

use super::events::{EventHandler, SceneEvent};
use super::services::ServiceRegistry;
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};
use crate::udp::server::AvatarMovementState;

pub trait IWindModule: Send + Sync + 'static {
    fn get_wind(&self, x: f32, y: f32) -> (f32, f32);
    fn get_average_wind(&self) -> (f32, f32);
}

const WIND_GRID_SIZE: usize = 16;
const WIND_UPDATE_INTERVAL: u64 = 110; // ~10 seconds at 11Hz heartbeat

pub struct WindModule {
    wind_grid: Arc<RwLock<Vec<(f32, f32)>>>,
    tick_counter: Arc<std::sync::atomic::AtomicU64>,
    socket: Option<Arc<tokio::net::UdpSocket>>,
    avatar_states: Option<Arc<RwLock<std::collections::HashMap<Uuid, AvatarMovementState>>>>,
    service_registry: Option<Arc<RwLock<ServiceRegistry>>>,
}

impl WindModule {
    pub fn new() -> Self {
        let mut grid = vec![(0.0f32, 0.0f32); WIND_GRID_SIZE * WIND_GRID_SIZE];
        for i in 0..grid.len() {
            let x = (i % WIND_GRID_SIZE) as f32;
            let y = (i / WIND_GRID_SIZE) as f32;
            grid[i] = (
                ((x * 0.7 + y * 1.3).sin() * 2.0),
                ((x * 1.1 + y * 0.5).cos() * 2.0),
            );
        }
        Self {
            wind_grid: Arc::new(RwLock::new(grid)),
            tick_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            socket: None,
            avatar_states: None,
            service_registry: None,
        }
    }

    fn update_wind(grid: &mut Vec<(f32, f32)>, seed: f64) {
        for i in 0..grid.len() {
            let x = (i % WIND_GRID_SIZE) as f64;
            let y = (i / WIND_GRID_SIZE) as f64;

            let u = ((x * 0.7 + y * 1.3 + seed * 0.01).sin() * 3.0) as f32;
            let v = ((x * 1.1 + y * 0.5 + seed * 0.013).cos() * 3.0) as f32;

            // Smooth transition
            grid[i].0 = grid[i].0 * 0.8 + u * 0.2;
            grid[i].1 = grid[i].1 * 0.8 + v * 0.2;

            // Clamp to reasonable range
            grid[i].0 = grid[i].0.max(-10.0).min(10.0);
            grid[i].1 = grid[i].1.max(-10.0).min(10.0);
        }
    }
}

impl IWindModule for WindModule {
    fn get_wind(&self, x: f32, y: f32) -> (f32, f32) {
        let grid = self.wind_grid.read();
        let gx = ((x / 16.0) as usize).min(WIND_GRID_SIZE - 1);
        let gy = ((y / 16.0) as usize).min(WIND_GRID_SIZE - 1);
        grid[gy * WIND_GRID_SIZE + gx]
    }

    fn get_average_wind(&self) -> (f32, f32) {
        let grid = self.wind_grid.read();
        let count = grid.len() as f32;
        let (sum_u, sum_v) = grid.iter().fold((0.0f32, 0.0f32), |acc, &(u, v)| {
            (acc.0 + u, acc.1 + v)
        });
        (sum_u / count, sum_v / count)
    }
}

#[async_trait]
impl RegionModule for WindModule {
    fn name(&self) -> &'static str { "WindModule" }
    fn replaceable_interface(&self) -> Option<&'static str> { Some("IWindModule") }

    async fn initialize(&mut self, _config: &ModuleConfig) -> Result<()> {
        info!("[WIND MODULE] Initialized (grid={}x{}, update interval={})",
              WIND_GRID_SIZE, WIND_GRID_SIZE, WIND_UPDATE_INTERVAL);
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.socket = Some(scene.socket.clone());
        self.avatar_states = Some(scene.avatar_states.clone());
        self.service_registry = Some(scene.service_registry.clone());

        let handler = Arc::new(WindEventHandler {
            wind_grid: self.wind_grid.clone(),
            tick_counter: self.tick_counter.clone(),
            socket: scene.socket.clone(),
            avatar_states: scene.avatar_states.clone(),
        });
        scene.event_bus.subscribe(
            SceneEvent::OnFrame { tick: 0 },
            handler,
            10, // low priority - wind is background
        );

        scene.service_registry.write().register::<WindModule>(
            Arc::new(WindModule {
                wind_grid: self.wind_grid.clone(),
                tick_counter: self.tick_counter.clone(),
                socket: self.socket.clone(),
                avatar_states: self.avatar_states.clone(),
                service_registry: self.service_registry.clone(),
            }),
        );

        info!("[WIND MODULE] Added to region {:?}", scene.region_name);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[async_trait]
impl SharedRegionModule for WindModule {}

struct WindEventHandler {
    wind_grid: Arc<RwLock<Vec<(f32, f32)>>>,
    tick_counter: Arc<std::sync::atomic::AtomicU64>,
    socket: Arc<tokio::net::UdpSocket>,
    avatar_states: Arc<RwLock<std::collections::HashMap<Uuid, AvatarMovementState>>>,
}

#[async_trait]
impl EventHandler for WindEventHandler {
    async fn handle_event(&self, event: &SceneEvent, _scene: &SceneContext) -> Result<()> {
        if let SceneEvent::OnFrame { tick } = event {
            let count = self.tick_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if count % WIND_UPDATE_INTERVAL != 0 { return Ok(()); }

            // Update wind grid
            {
                let mut grid = self.wind_grid.write();
                WindModule::update_wind(&mut grid, *tick as f64);
            }

            let (wind_data, clients) = {
                let grid = self.wind_grid.read();
                let mut data = Vec::with_capacity(WIND_GRID_SIZE * WIND_GRID_SIZE * 8 + 64);

                let method = b"Wind";
                data.push(0x40);
                data.extend_from_slice(&0u32.to_be_bytes());
                data.push(0);
                data.extend_from_slice(&[0xFF, 0xFF]);
                data.extend_from_slice(&0x0101u16.to_be_bytes()); // Low 257: GenericMessage

                let method_len = method.len() as u8 + 1;
                data.push(method_len);
                data.extend_from_slice(method);
                data.push(0);
                data.extend_from_slice(Uuid::nil().as_bytes());

                data.push(1u8);
                let mut param_data = Vec::with_capacity(WIND_GRID_SIZE * WIND_GRID_SIZE * 8);
                for &(u, v) in grid.iter() {
                    param_data.extend_from_slice(&u.to_le_bytes());
                    param_data.extend_from_slice(&v.to_le_bytes());
                }
                let param_len = (param_data.len() as u16).to_le_bytes();
                data.extend_from_slice(&param_len);
                data.extend_from_slice(&param_data);

                let c: Vec<SocketAddr> = {
                    let states = self.avatar_states.read();
                    states.values().map(|s| s.client_addr).collect()
                };
                (data, c)
            };

            for addr in &clients {
                let _ = self.socket.send_to(&wind_data, addr).await;
            }
        }
        Ok(())
    }
}
