use std::any::Any;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::session::SessionManager;

use super::events::{ChatType, EventHandler, SceneEvent};
use super::services::ServiceRegistry;
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};

pub trait IChatModule: Send + Sync + 'static {
    fn whisper_distance(&self) -> f32;
    fn say_distance(&self) -> f32;
    fn shout_distance(&self) -> f32;
}

struct ChatConfig {
    whisper_distance: f32,
    say_distance: f32,
    shout_distance: f32,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            whisper_distance: 10.0,
            say_distance: 20.0,
            shout_distance: 100.0,
        }
    }
}

pub struct ChatModule {
    config: ChatConfig,
    region_uuid: Option<Uuid>,
    socket: Option<Arc<tokio::net::UdpSocket>>,
    session_manager: Option<Arc<SessionManager>>,
    avatar_states: Option<Arc<RwLock<std::collections::HashMap<Uuid, crate::udp::server::AvatarMovementState>>>>,
    service_registry: Option<Arc<RwLock<ServiceRegistry>>>,
}

impl ChatModule {
    pub fn new() -> Self {
        Self {
            config: ChatConfig::default(),
            region_uuid: None,
            socket: None,
            session_manager: None,
            avatar_states: None,
            service_registry: None,
        }
    }
}

impl IChatModule for ChatModule {
    fn whisper_distance(&self) -> f32 {
        self.config.whisper_distance
    }

    fn say_distance(&self) -> f32 {
        self.config.say_distance
    }

    fn shout_distance(&self) -> f32 {
        self.config.shout_distance
    }
}

#[async_trait]
impl RegionModule for ChatModule {
    fn name(&self) -> &'static str {
        "ChatModule"
    }

    fn replaceable_interface(&self) -> Option<&'static str> {
        Some("IChatModule")
    }

    async fn initialize(&mut self, config: &ModuleConfig) -> Result<()> {
        self.config.whisper_distance = config.get_f32("whisper_distance", 10.0);
        self.config.say_distance = config.get_f32("say_distance", 20.0);
        self.config.shout_distance = config.get_f32("shout_distance", 100.0);
        info!(
            "[CHAT MODULE] Initialized: whisper={}m, say={}m, shout={}m",
            self.config.whisper_distance,
            self.config.say_distance,
            self.config.shout_distance,
        );
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = Some(scene.region_uuid);
        self.socket = Some(scene.socket.clone());
        self.session_manager = Some(scene.session_manager.clone());
        self.avatar_states = Some(scene.avatar_states.clone());
        self.service_registry = Some(scene.service_registry.clone());

        let handler = Arc::new(ChatEventHandler {
            whisper_distance: self.config.whisper_distance,
            say_distance: self.config.say_distance,
            shout_distance: self.config.shout_distance,
        });
        scene.event_bus.subscribe(
            SceneEvent::OnChatFromViewer {
                agent_id: Uuid::nil(),
                message: String::new(),
                chat_type: ChatType::Say,
                channel: 0,
                position: [0.0; 3],
                sender_name: String::new(),
            },
            handler,
            100,
        );

        scene.service_registry.write().register::<ChatModule>(
            Arc::new(ChatModule {
                config: ChatConfig {
                    whisper_distance: self.config.whisper_distance,
                    say_distance: self.config.say_distance,
                    shout_distance: self.config.shout_distance,
                },
                region_uuid: self.region_uuid,
                socket: self.socket.clone(),
                session_manager: self.session_manager.clone(),
                avatar_states: self.avatar_states.clone(),
                service_registry: self.service_registry.clone(),
            }),
        );

        info!("[CHAT MODULE] Added to region {:?}", scene.region_name);
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
impl SharedRegionModule for ChatModule {}

struct ChatEventHandler {
    whisper_distance: f32,
    say_distance: f32,
    shout_distance: f32,
}

impl ChatEventHandler {
    fn max_distance(&self, chat_type: &ChatType) -> Option<f32> {
        match chat_type {
            ChatType::Whisper => Some(self.whisper_distance),
            ChatType::Say => Some(self.say_distance),
            ChatType::Shout => Some(self.shout_distance),
            ChatType::StartTyping | ChatType::StopTyping => Some(30.0),
            ChatType::Region | ChatType::Owner => None,
        }
    }
}

fn distance_3d(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

const CHAT_FROM_SIMULATOR_ID: u32 = 0xFFFF008B;

fn build_chat_from_simulator(
    sender_name: &str,
    sender_id: &Uuid,
    owner_id: &Uuid,
    source_type: u8,
    chat_type: u8,
    position: &[f32; 3],
    message: &str,
) -> Vec<u8> {
    let name_bytes = sender_name.as_bytes();
    let msg_bytes = message.as_bytes();

    let mut packet = Vec::with_capacity(128 + name_bytes.len() + msg_bytes.len());

    // Header: flags=0x40 (reliable), sequence placeholder, extra=0, id
    packet.push(0x40);
    packet.extend_from_slice(&0u32.to_be_bytes());
    packet.push(0);
    packet.extend_from_slice(&[0xFF, 0xFF]);
    packet.extend_from_slice(&0x008Bu16.to_be_bytes());

    // FromName (variable-1)
    packet.push(name_bytes.len() as u8 + 1);
    packet.extend_from_slice(name_bytes);
    packet.push(0);

    // SourceID
    packet.extend_from_slice(sender_id.as_bytes());
    // OwnerID
    packet.extend_from_slice(owner_id.as_bytes());
    // SourceType
    packet.push(source_type);
    // ChatType
    packet.push(chat_type);
    // Audible
    packet.push(1);
    // Position
    for &v in position {
        packet.extend_from_slice(&v.to_le_bytes());
    }
    // Message (variable-2)
    let msg_len = (msg_bytes.len() as u16).to_le_bytes();
    packet.extend_from_slice(&msg_len);
    packet.extend_from_slice(msg_bytes);

    packet
}

#[async_trait]
impl EventHandler for ChatEventHandler {
    async fn handle_event(
        &self,
        event: &SceneEvent,
        scene: &SceneContext,
    ) -> Result<()> {
        if let SceneEvent::OnChatFromViewer {
            agent_id,
            message,
            chat_type,
            channel,
            position,
            sender_name,
        } = event
        {
            if *channel != 0 {
                return Ok(());
            }

            let max_dist = self.max_distance(chat_type);

            let recipients: Vec<(Uuid, SocketAddr)> = {
                let states = scene.avatar_states.read();
                states
                    .iter()
                    .filter(|(id, state)| {
                        if let Some(d) = max_dist {
                            distance_3d(position, &state.position) <= d
                        } else {
                            true
                        }
                    })
                    .map(|(id, state)| (*id, state.client_addr))
                    .collect()
            };

            let packet = build_chat_from_simulator(
                sender_name,
                agent_id,
                agent_id,
                1, // AGENT
                chat_type.to_u8(),
                position,
                message,
            );

            for (_, addr) in &recipients {
                let _ = scene.socket.send_to(&packet, addr).await;
            }

            info!(
                "[CHAT MODULE] {} ({}): '{}' → {} recipients",
                sender_name,
                chat_type.to_u8(),
                if message.len() > 60 {
                    &message[..60]
                } else {
                    message
                },
                recipients.len(),
            );
        }
        Ok(())
    }
}
