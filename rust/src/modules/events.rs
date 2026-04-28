use std::collections::HashMap;
use std::mem::discriminant;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ChatType {
    Whisper,
    Say,
    Shout,
    StartTyping,
    StopTyping,
    Region,
    Owner,
}

impl ChatType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => ChatType::Whisper,
            1 => ChatType::Say,
            2 => ChatType::Shout,
            4 => ChatType::StartTyping,
            5 => ChatType::StopTyping,
            6 => ChatType::Region,
            8 => ChatType::Owner,
            _ => ChatType::Say,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            ChatType::Whisper => 0,
            ChatType::Say => 1,
            ChatType::Shout => 2,
            ChatType::StartTyping => 4,
            ChatType::StopTyping => 5,
            ChatType::Region => 6,
            ChatType::Owner => 8,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PermissionAction {
    RezObject,
    EditObject,
    DeleteObject,
    MoveObject,
    CopyObject,
    TakeObject,
    ModifyTerrain,
    RunScript,
    Fly,
    CreateLandmark,
    SetHome,
}

#[derive(Debug, Clone)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SceneEvent {
    OnNewClient {
        agent_id: Uuid,
        session_id: Uuid,
        addr: SocketAddr,
    },
    OnClientClosed {
        agent_id: Uuid,
    },
    OnChatFromViewer {
        agent_id: Uuid,
        message: String,
        chat_type: ChatType,
        channel: i32,
        position: [f32; 3],
        sender_name: String,
    },
    OnInstantMessage {
        from_agent: Uuid,
        to_agent: Uuid,
        message: String,
        dialog_type: u8,
    },
    OnObjectGrab {
        agent_id: Uuid,
        object_local_id: u32,
        grab_offset: [f32; 3],
    },
    OnFrame {
        tick: u64,
    },
    OnPermissionCheck {
        agent_id: Uuid,
        target_id: Uuid,
        action: PermissionAction,
        result: Arc<RwLock<PermissionCheckResult>>,
    },
    OnParcelPropertiesRequest {
        agent_id: Uuid,
        sequence_id: i32,
        west: f32,
        south: f32,
        east: f32,
        north: f32,
        snap_selection: bool,
        dest: SocketAddr,
    },
    OnParcelPropertiesUpdate {
        agent_id: Uuid,
        local_id: i32,
        flags: u32,
        parcel_flags: u32,
        sale_price: i32,
        name: String,
        description: String,
    },
    OnModifyLand {
        agent_id: Uuid,
        action: u8,
        brush_size: u8,
        seconds: f32,
        height: f32,
        position: [f32; 3],
    },
    OnScriptDialog {
        avatar_id: Uuid,
        object_id: Uuid,
        object_name: String,
        owner_id: Uuid,
        message: String,
        buttons: Vec<String>,
        channel: i32,
        dest: SocketAddr,
    },
    OnEstateOwnerMessage {
        agent_id: Uuid,
        method: String,
        params: Vec<String>,
        dest: SocketAddr,
    },
    OnSoundTrigger {
        sound_id: Uuid,
        owner_id: Uuid,
        object_id: Uuid,
        parent_id: Uuid,
        gain: f32,
        position: [f32; 3],
        handle: u64,
    },
}

type Discriminant = std::mem::Discriminant<SceneEvent>;

fn event_discriminant(event: &SceneEvent) -> Discriminant {
    discriminant(event)
}

#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    async fn handle_event(
        &self,
        event: &SceneEvent,
        scene: &super::traits::SceneContext,
    ) -> Result<()>;
}

struct Subscription {
    handler: Arc<dyn EventHandler>,
    priority: i32,
}

pub struct EventBus {
    subscribers: RwLock<HashMap<Discriminant, Vec<Subscription>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(HashMap::new()),
        }
    }

    pub fn subscribe(&self, event_type: SceneEvent, handler: Arc<dyn EventHandler>, priority: i32) {
        let disc = event_discriminant(&event_type);
        let mut subs = self.subscribers.write();
        let list = subs.entry(disc).or_insert_with(Vec::new);
        list.push(Subscription { handler, priority });
        list.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub async fn publish(
        &self,
        event: &SceneEvent,
        scene: &super::traits::SceneContext,
    ) -> Result<()> {
        let disc = event_discriminant(event);
        let handlers: Vec<Arc<dyn EventHandler>> = {
            let subs = self.subscribers.read();
            match subs.get(&disc) {
                Some(list) => list.iter().map(|s| s.handler.clone()).collect(),
                None => return Ok(()),
            }
        };

        for handler in handlers {
            if let Err(e) = handler.handle_event(event, scene).await {
                warn!("Event handler error for {:?}: {}", disc, e);
            }
        }
        Ok(())
    }

    pub fn subscriber_count(&self, event_type: &SceneEvent) -> usize {
        let disc = event_discriminant(event_type);
        let subs = self.subscribers.read();
        subs.get(&disc).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
