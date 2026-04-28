use std::any::Any;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::session::SessionManager;

use super::events::{EventHandler, SceneEvent};
use super::services::ServiceRegistry;
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};
use crate::udp::server::AvatarMovementState;

pub trait ITerrainModule: Send + Sync + 'static {
    fn modify_terrain(
        &self,
        action: u8,
        brush_size: u8,
        position: [f32; 3],
        agent_id: Uuid,
        seconds: f32,
    );
    fn get_height(&self, x: f32, y: f32) -> f32;
    fn set_height(&self, x: f32, y: f32, height: f32);
    fn save_terrain(&self);
    fn load_terrain(&self, filename: &str);
}

struct TerrainConfig {
    default_height: f32,
    raise_strength: f32,
    lower_strength: f32,
    smooth_strength: f32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            default_height: 21.0,
            raise_strength: 0.25,
            lower_strength: 0.25,
            smooth_strength: 0.5,
        }
    }
}

pub struct TerrainModuleImpl {
    config: TerrainConfig,
    heightmap: Arc<RwLock<Vec<f32>>>,
    dirty_patches: Arc<RwLock<HashSet<(u16, u16)>>>,
    region_uuid: Option<Uuid>,
    socket: Option<Arc<tokio::net::UdpSocket>>,
    session_manager: Option<Arc<SessionManager>>,
    db: Option<Arc<DatabaseConnection>>,
    avatar_states: Option<Arc<RwLock<std::collections::HashMap<Uuid, AvatarMovementState>>>>,
    service_registry: Option<Arc<RwLock<ServiceRegistry>>>,
}

impl TerrainModuleImpl {
    pub fn new() -> Self {
        Self {
            config: TerrainConfig::default(),
            heightmap: Arc::new(RwLock::new(vec![21.0f32; 256 * 256])),
            dirty_patches: Arc::new(RwLock::new(HashSet::new())),
            region_uuid: None,
            socket: None,
            session_manager: None,
            db: None,
            avatar_states: None,
            service_registry: None,
        }
    }

    fn apply_brush(&self, action: u8, brush_size: u8, center_x: f32, center_y: f32, seconds: f32) {
        let radius = match brush_size {
            1 => 2.0f32,
            2 => 4.0,
            3 => 8.0,
            _ => 4.0,
        };

        let strength = match action {
            0 => 0.0, // Flatten - handled differently
            1 => self.config.raise_strength * seconds,
            2 => -(self.config.lower_strength * seconds),
            3 => self.config.smooth_strength * seconds,
            4 => 0.1 * seconds, // Noise
            5 => 0.0,           // Revert
            _ => return,
        };

        let mut heightmap = self.heightmap.write();
        let mut dirty = self.dirty_patches.write();

        let min_x = ((center_x - radius).max(0.0) as usize).min(255);
        let max_x = ((center_x + radius).min(255.0) as usize).min(255);
        let min_y = ((center_y - radius).max(0.0) as usize).min(255);
        let max_y = ((center_y + radius).min(255.0) as usize).min(255);

        let center_height = if action == 0 {
            heightmap[(center_y as usize * 256 + center_x as usize).min(256 * 256 - 1)]
        } else {
            0.0
        };

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > radius {
                    continue;
                }

                let falloff = 1.0 - (dist / radius);
                let idx = y * 256 + x;

                match action {
                    0 => {
                        // Flatten
                        let current = heightmap[idx];
                        heightmap[idx] = current + (center_height - current) * falloff * 0.5;
                    }
                    1 | 2 => {
                        // Raise / Lower
                        heightmap[idx] += strength * falloff;
                        heightmap[idx] = heightmap[idx].max(0.0).min(500.0);
                    }
                    3 => {
                        // Smooth
                        let mut sum = 0.0f32;
                        let mut count = 0;
                        for sy in (y.saturating_sub(1))..=(y + 1).min(255) {
                            for sx in (x.saturating_sub(1))..=(x + 1).min(255) {
                                sum += heightmap[sy * 256 + sx];
                                count += 1;
                            }
                        }
                        let avg = sum / count as f32;
                        heightmap[idx] += (avg - heightmap[idx]) * strength * falloff;
                    }
                    4 => {
                        // Noise
                        let noise =
                            ((x as f32 * 17.3 + y as f32 * 31.7).sin() * 43758.5453).fract() - 0.5;
                        heightmap[idx] += noise * strength * falloff;
                    }
                    5 => {
                        // Revert
                        heightmap[idx] = self.config.default_height;
                    }
                    _ => {}
                }

                dirty.insert(((x / 16) as u16, (y / 16) as u16));
            }
        }
    }
}

impl ITerrainModule for TerrainModuleImpl {
    fn modify_terrain(
        &self,
        action: u8,
        brush_size: u8,
        position: [f32; 3],
        _agent_id: Uuid,
        seconds: f32,
    ) {
        self.apply_brush(action, brush_size, position[0], position[1], seconds);
    }

    fn get_height(&self, x: f32, y: f32) -> f32 {
        let ix = (x as usize).min(255);
        let iy = (y as usize).min(255);
        self.heightmap.read()[iy * 256 + ix]
    }

    fn set_height(&self, x: f32, y: f32, height: f32) {
        let ix = (x as usize).min(255);
        let iy = (y as usize).min(255);
        self.heightmap.write()[iy * 256 + ix] = height;
        self.dirty_patches
            .write()
            .insert(((ix / 16) as u16, (iy / 16) as u16));
    }

    fn save_terrain(&self) {
        info!("[TERRAIN MODULE] Save terrain requested");
    }

    fn load_terrain(&self, filename: &str) {
        info!(
            "[TERRAIN MODULE] Load terrain from '{}' requested",
            filename
        );
    }
}

#[async_trait]
impl RegionModule for TerrainModuleImpl {
    fn name(&self) -> &'static str {
        "TerrainModule"
    }
    fn replaceable_interface(&self) -> Option<&'static str> {
        Some("ITerrainModule")
    }

    async fn initialize(&mut self, config: &ModuleConfig) -> Result<()> {
        self.config.default_height = config.get_f32("default_height", 21.0);
        self.config.raise_strength = config.get_f32("raise_strength", 0.25);
        self.config.lower_strength = config.get_f32("lower_strength", 0.25);
        self.config.smooth_strength = config.get_f32("smooth_strength", 0.5);
        info!(
            "[TERRAIN MODULE] Initialized (default_height={})",
            self.config.default_height
        );
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = Some(scene.region_uuid);
        self.socket = Some(scene.socket.clone());
        self.session_manager = Some(scene.session_manager.clone());
        self.db = scene.db.clone();
        self.avatar_states = Some(scene.avatar_states.clone());
        self.service_registry = Some(scene.service_registry.clone());

        let handler = Arc::new(TerrainEventHandler {
            heightmap: self.heightmap.clone(),
            dirty_patches: self.dirty_patches.clone(),
            config: TerrainConfig {
                default_height: self.config.default_height,
                raise_strength: self.config.raise_strength,
                lower_strength: self.config.lower_strength,
                smooth_strength: self.config.smooth_strength,
            },
        });
        scene.event_bus.subscribe(
            SceneEvent::OnModifyLand {
                agent_id: Uuid::nil(),
                action: 0,
                brush_size: 0,
                seconds: 0.0,
                height: 0.0,
                position: [0.0; 3],
            },
            handler,
            100,
        );

        scene
            .service_registry
            .write()
            .register::<TerrainModuleImpl>(Arc::new(TerrainModuleImpl {
                config: TerrainConfig {
                    default_height: self.config.default_height,
                    raise_strength: self.config.raise_strength,
                    lower_strength: self.config.lower_strength,
                    smooth_strength: self.config.smooth_strength,
                },
                heightmap: self.heightmap.clone(),
                dirty_patches: self.dirty_patches.clone(),
                region_uuid: self.region_uuid,
                socket: self.socket.clone(),
                session_manager: self.session_manager.clone(),
                db: self.db.clone(),
                avatar_states: self.avatar_states.clone(),
                service_registry: self.service_registry.clone(),
            }));

        info!("[TERRAIN MODULE] Added to region {:?}", scene.region_name);
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
impl SharedRegionModule for TerrainModuleImpl {}

struct TerrainEventHandler {
    heightmap: Arc<RwLock<Vec<f32>>>,
    dirty_patches: Arc<RwLock<HashSet<(u16, u16)>>>,
    config: TerrainConfig,
}

impl TerrainEventHandler {
    fn apply_brush_internal(
        &self,
        action: u8,
        brush_size: u8,
        center_x: f32,
        center_y: f32,
        seconds: f32,
    ) {
        let radius = match brush_size {
            1 => 2.0f32,
            2 => 4.0,
            3 => 8.0,
            _ => 4.0,
        };

        let strength = match action {
            0 => 0.0,
            1 => self.config.raise_strength * seconds,
            2 => -(self.config.lower_strength * seconds),
            3 => self.config.smooth_strength * seconds,
            4 => 0.1 * seconds,
            5 => 0.0,
            _ => return,
        };

        let mut heightmap = self.heightmap.write();
        let mut dirty = self.dirty_patches.write();

        let min_x = ((center_x - radius).max(0.0) as usize).min(255);
        let max_x = ((center_x + radius).min(255.0) as usize).min(255);
        let min_y = ((center_y - radius).max(0.0) as usize).min(255);
        let max_y = ((center_y + radius).min(255.0) as usize).min(255);

        let center_height = if action == 0 {
            heightmap[(center_y as usize * 256 + center_x as usize).min(256 * 256 - 1)]
        } else {
            0.0
        };

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > radius {
                    continue;
                }

                let falloff = 1.0 - (dist / radius);
                let idx = y * 256 + x;

                match action {
                    0 => {
                        let current = heightmap[idx];
                        heightmap[idx] = current + (center_height - current) * falloff * 0.5;
                    }
                    1 | 2 => {
                        heightmap[idx] += strength * falloff;
                        heightmap[idx] = heightmap[idx].max(0.0).min(500.0);
                    }
                    3 => {
                        let mut sum = 0.0f32;
                        let mut count = 0;
                        for sy in (y.saturating_sub(1))..=(y + 1).min(255) {
                            for sx in (x.saturating_sub(1))..=(x + 1).min(255) {
                                sum += heightmap[sy * 256 + sx];
                                count += 1;
                            }
                        }
                        let avg = sum / count as f32;
                        heightmap[idx] += (avg - heightmap[idx]) * strength * falloff;
                    }
                    4 => {
                        let noise =
                            ((x as f32 * 17.3 + y as f32 * 31.7).sin() * 43758.5453).fract() - 0.5;
                        heightmap[idx] += noise * strength * falloff;
                    }
                    5 => {
                        heightmap[idx] = self.config.default_height;
                    }
                    _ => {}
                }

                dirty.insert(((x / 16) as u16, (y / 16) as u16));
            }
        }
    }
}

#[async_trait]
impl EventHandler for TerrainEventHandler {
    async fn handle_event(&self, event: &SceneEvent, _scene: &SceneContext) -> Result<()> {
        if let SceneEvent::OnModifyLand {
            agent_id,
            action,
            brush_size,
            seconds,
            position,
            ..
        } = event
        {
            self.apply_brush_internal(*action, *brush_size, position[0], position[1], *seconds);
            info!(
                "[TERRAIN MODULE] Agent {} modified terrain (action={}, brush={}) at ({:.0},{:.0})",
                agent_id, action, brush_size, position[0], position[1]
            );
        }
        Ok(())
    }
}
