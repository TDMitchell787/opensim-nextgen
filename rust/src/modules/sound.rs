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

pub trait ISoundModule: Send + Sync + 'static {
    fn play_sound(&self, object_id: Uuid, sound_id: Uuid, gain: f32, position: [f32; 3], radius: f32);
    fn loop_sound(&self, object_id: Uuid, sound_id: Uuid, gain: f32);
    fn stop_sound(&self, object_id: Uuid);
    fn trigger_sound(&self, sound_id: Uuid, owner_id: Uuid, object_id: Uuid,
                     parent_id: Uuid, gain: f32, position: [f32; 3], handle: u64);
    fn preload_sound(&self, object_id: Uuid, sound_id: Uuid);
}

pub struct SoundModule {
    region_uuid: Option<Uuid>,
    region_handle: u64,
    socket: Option<Arc<tokio::net::UdpSocket>>,
    avatar_states: Option<Arc<RwLock<std::collections::HashMap<Uuid, AvatarMovementState>>>>,
    service_registry: Option<Arc<RwLock<ServiceRegistry>>>,
}

impl SoundModule {
    pub fn new() -> Self {
        Self {
            region_uuid: None,
            region_handle: 0,
            socket: None,
            avatar_states: None,
            service_registry: None,
        }
    }
}

const SOUND_TRIGGER_ID: u32 = 0x001D; // Medium 29
const ATTACHED_SOUND_ID: u32 = 0x000D; // Medium 13
const ATTACHED_SOUND_GAIN_CHANGE_ID: u32 = 0x000E; // Medium 14
const PRELOAD_SOUND_ID: u32 = 0x000F; // Medium 15

fn build_sound_trigger(
    sound_id: &Uuid, owner_id: &Uuid, object_id: &Uuid,
    parent_id: &Uuid, handle: u64, position: &[f32; 3], gain: f32,
) -> Vec<u8> {
    let mut packet = Vec::with_capacity(100);
    packet.push(0x40); // reliable
    packet.extend_from_slice(&0u32.to_be_bytes());
    packet.push(0);
    // Medium frequency: single 0xFF byte + 1 byte ID
    packet.push(0xFF);
    packet.push(0x1D); // Medium 29

    packet.extend_from_slice(sound_id.as_bytes());
    packet.extend_from_slice(owner_id.as_bytes());
    packet.extend_from_slice(object_id.as_bytes());
    packet.extend_from_slice(parent_id.as_bytes());
    packet.extend_from_slice(&handle.to_le_bytes());
    for &v in position {
        packet.extend_from_slice(&v.to_le_bytes());
    }
    packet.extend_from_slice(&gain.to_le_bytes());

    packet
}

fn build_attached_sound(
    sound_id: &Uuid, object_id: &Uuid, owner_id: &Uuid, gain: f32, flags: u8,
) -> Vec<u8> {
    let mut packet = Vec::with_capacity(80);
    packet.push(0x40);
    packet.extend_from_slice(&0u32.to_be_bytes());
    packet.push(0);
    packet.push(0xFF);
    packet.push(0x0D); // Medium 13

    packet.extend_from_slice(sound_id.as_bytes());
    packet.extend_from_slice(object_id.as_bytes());
    packet.extend_from_slice(owner_id.as_bytes());
    packet.extend_from_slice(&gain.to_le_bytes());
    packet.push(flags);

    packet
}

fn build_preload_sound(sound_id: &Uuid, object_id: &Uuid, owner_id: &Uuid) -> Vec<u8> {
    let mut packet = Vec::with_capacity(60);
    packet.push(0x40);
    packet.extend_from_slice(&0u32.to_be_bytes());
    packet.push(0);
    packet.push(0xFF);
    packet.push(0x0F); // Medium 15

    packet.extend_from_slice(object_id.as_bytes());
    packet.extend_from_slice(owner_id.as_bytes());
    packet.extend_from_slice(sound_id.as_bytes());

    packet
}

impl ISoundModule for SoundModule {
    fn play_sound(&self, _object_id: Uuid, _sound_id: Uuid, _gain: f32, _position: [f32; 3], _radius: f32) {}
    fn loop_sound(&self, _object_id: Uuid, _sound_id: Uuid, _gain: f32) {}
    fn stop_sound(&self, _object_id: Uuid) {}
    fn trigger_sound(&self, _sound_id: Uuid, _owner_id: Uuid, _object_id: Uuid,
                     _parent_id: Uuid, _gain: f32, _position: [f32; 3], _handle: u64) {}
    fn preload_sound(&self, _object_id: Uuid, _sound_id: Uuid) {}
}

#[async_trait]
impl RegionModule for SoundModule {
    fn name(&self) -> &'static str { "SoundModule" }
    fn replaceable_interface(&self) -> Option<&'static str> { Some("ISoundModule") }

    async fn initialize(&mut self, _config: &ModuleConfig) -> Result<()> {
        info!("[SOUND MODULE] Initialized");
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = Some(scene.region_uuid);
        self.region_handle = scene.region_handle;
        self.socket = Some(scene.socket.clone());
        self.avatar_states = Some(scene.avatar_states.clone());
        self.service_registry = Some(scene.service_registry.clone());

        let handler = Arc::new(SoundEventHandler {
            socket: scene.socket.clone(),
            avatar_states: scene.avatar_states.clone(),
            region_handle: scene.region_handle,
        });
        scene.event_bus.subscribe(
            SceneEvent::OnSoundTrigger {
                sound_id: Uuid::nil(), owner_id: Uuid::nil(),
                object_id: Uuid::nil(), parent_id: Uuid::nil(),
                gain: 0.0, position: [0.0; 3], handle: 0,
            },
            handler,
            100,
        );

        scene.service_registry.write().register::<SoundModule>(
            Arc::new(SoundModule {
                region_uuid: self.region_uuid,
                region_handle: self.region_handle,
                socket: self.socket.clone(),
                avatar_states: self.avatar_states.clone(),
                service_registry: self.service_registry.clone(),
            }),
        );

        info!("[SOUND MODULE] Added to region {:?}", scene.region_name);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[async_trait]
impl SharedRegionModule for SoundModule {}

struct SoundEventHandler {
    socket: Arc<tokio::net::UdpSocket>,
    avatar_states: Arc<RwLock<std::collections::HashMap<Uuid, AvatarMovementState>>>,
    region_handle: u64,
}

#[async_trait]
impl EventHandler for SoundEventHandler {
    async fn handle_event(&self, event: &SceneEvent, _scene: &SceneContext) -> Result<()> {
        if let SceneEvent::OnSoundTrigger {
            sound_id, owner_id, object_id, parent_id, gain, position, handle,
        } = event {
            let packet = build_sound_trigger(
                sound_id, owner_id, object_id, parent_id,
                *handle, position, *gain,
            );

            let clients: Vec<SocketAddr> = {
                let states = self.avatar_states.read();
                states.values().map(|s| s.client_addr).collect()
            };

            for addr in &clients {
                let _ = self.socket.send_to(&packet, addr).await;
            }

            info!("[SOUND MODULE] Broadcast SoundTrigger {} to {} clients", sound_id, clients.len());
        }
        Ok(())
    }
}
