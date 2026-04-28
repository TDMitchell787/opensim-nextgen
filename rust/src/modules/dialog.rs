use std::any::Any;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bytes::BufMut;
use parking_lot::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::session::SessionManager;

use super::events::{EventHandler, SceneEvent};
use super::services::ServiceRegistry;
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};

pub trait IDialogModule: Send + Sync + 'static {
    fn send_dialog(
        &self,
        avatar_id: Uuid,
        object_name: &str,
        object_id: Uuid,
        owner_id: Uuid,
        message: &str,
        buttons: &[String],
        channel: i32,
        dest: SocketAddr,
    );
    fn send_text_box(
        &self,
        avatar_id: Uuid,
        object_name: &str,
        object_id: Uuid,
        owner_id: Uuid,
        message: &str,
        channel: i32,
        dest: SocketAddr,
    );
    fn send_url_dialog(
        &self,
        avatar_id: Uuid,
        object_name: &str,
        object_id: Uuid,
        owner_id: Uuid,
        message: &str,
        url: &str,
        dest: SocketAddr,
    );
}

pub struct DialogModule {
    region_uuid: Option<Uuid>,
    socket: Option<Arc<tokio::net::UdpSocket>>,
    session_manager: Option<Arc<SessionManager>>,
    service_registry: Option<Arc<RwLock<ServiceRegistry>>>,
}

impl DialogModule {
    pub fn new() -> Self {
        Self {
            region_uuid: None,
            socket: None,
            session_manager: None,
            service_registry: None,
        }
    }
}

const SCRIPT_DIALOG_ID: u32 = 0xFFFF00B7; // Low 183
const LOAD_URL_ID: u32 = 0xFFFF00C2; // Low 194
const SCRIPT_DIALOG_REPLY_ID: u32 = 0xFFFF00B8; // Low 184

fn build_script_dialog(
    object_id: &Uuid,
    first_name: &str,
    last_name: &str,
    object_name: &str,
    message: &str,
    channel: i32,
    buttons: &[String],
    owner_id: &Uuid,
) -> Vec<u8> {
    let mut packet = Vec::with_capacity(256);
    packet.push(0x40); // reliable
    packet.extend_from_slice(&0u32.to_be_bytes());
    packet.push(0);
    packet.extend_from_slice(&[0xFF, 0xFF]);
    packet.extend_from_slice(&0x00B7u16.to_be_bytes());

    // ObjectData block
    packet.extend_from_slice(object_id.as_bytes());

    let first_bytes = first_name.as_bytes();
    packet.push(first_bytes.len() as u8 + 1);
    packet.extend_from_slice(first_bytes);
    packet.push(0);

    let last_bytes = last_name.as_bytes();
    packet.push(last_bytes.len() as u8 + 1);
    packet.extend_from_slice(last_bytes);
    packet.push(0);

    let obj_name_bytes = object_name.as_bytes();
    packet.push(obj_name_bytes.len() as u8 + 1);
    packet.extend_from_slice(obj_name_bytes);
    packet.push(0);

    let msg_bytes = message.as_bytes();
    let msg_len = (msg_bytes.len() as u16).to_le_bytes();
    packet.extend_from_slice(&msg_len);
    packet.extend_from_slice(msg_bytes);

    packet.extend_from_slice(&channel.to_le_bytes());

    packet.extend_from_slice(Uuid::nil().as_bytes()); // ImageID

    // Buttons[] block
    let btn_count = buttons.len().min(12) as u8;
    packet.push(btn_count);
    for btn in buttons.iter().take(12) {
        let btn_bytes = btn.as_bytes();
        let truncated = &btn_bytes[..btn_bytes.len().min(24)];
        packet.push(truncated.len() as u8 + 1);
        packet.extend_from_slice(truncated);
        packet.push(0);
    }

    // OwnerData block
    packet.extend_from_slice(owner_id.as_bytes());

    packet
}

fn build_load_url(
    object_name: &str,
    object_id: &Uuid,
    owner_id: &Uuid,
    owner_is_group: bool,
    message: &str,
    url: &str,
) -> Vec<u8> {
    let mut packet = Vec::with_capacity(256);
    packet.push(0x40);
    packet.extend_from_slice(&0u32.to_be_bytes());
    packet.push(0);
    packet.extend_from_slice(&[0xFF, 0xFF]);
    packet.extend_from_slice(&0x00C2u16.to_be_bytes());

    let name_bytes = object_name.as_bytes();
    packet.push(name_bytes.len() as u8 + 1);
    packet.extend_from_slice(name_bytes);
    packet.push(0);

    packet.extend_from_slice(object_id.as_bytes());
    packet.extend_from_slice(owner_id.as_bytes());
    packet.push(if owner_is_group { 1 } else { 0 });

    let msg_bytes = message.as_bytes();
    let msg_len = (msg_bytes.len() as u16).to_le_bytes();
    packet.extend_from_slice(&msg_len);
    packet.extend_from_slice(msg_bytes);

    let url_bytes = url.as_bytes();
    let url_len = (url_bytes.len() as u16).to_le_bytes();
    packet.extend_from_slice(&url_len);
    packet.extend_from_slice(url_bytes);

    packet
}

impl IDialogModule for DialogModule {
    fn send_dialog(
        &self,
        _avatar_id: Uuid,
        _object_name: &str,
        _object_id: Uuid,
        _owner_id: Uuid,
        _message: &str,
        _buttons: &[String],
        _channel: i32,
        _dest: SocketAddr,
    ) {
    }

    fn send_text_box(
        &self,
        _avatar_id: Uuid,
        _object_name: &str,
        _object_id: Uuid,
        _owner_id: Uuid,
        _message: &str,
        _channel: i32,
        _dest: SocketAddr,
    ) {
    }

    fn send_url_dialog(
        &self,
        _avatar_id: Uuid,
        _object_name: &str,
        _object_id: Uuid,
        _owner_id: Uuid,
        _message: &str,
        _url: &str,
        _dest: SocketAddr,
    ) {
    }
}

#[async_trait]
impl RegionModule for DialogModule {
    fn name(&self) -> &'static str {
        "DialogModule"
    }
    fn replaceable_interface(&self) -> Option<&'static str> {
        Some("IDialogModule")
    }

    async fn initialize(&mut self, _config: &ModuleConfig) -> Result<()> {
        info!("[DIALOG MODULE] Initialized");
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = Some(scene.region_uuid);
        self.socket = Some(scene.socket.clone());
        self.session_manager = Some(scene.session_manager.clone());
        self.service_registry = Some(scene.service_registry.clone());

        let handler = Arc::new(DialogEventHandler {
            socket: scene.socket.clone(),
            session_manager: scene.session_manager.clone(),
        });
        scene.event_bus.subscribe(
            SceneEvent::OnScriptDialog {
                avatar_id: Uuid::nil(),
                object_id: Uuid::nil(),
                object_name: String::new(),
                owner_id: Uuid::nil(),
                message: String::new(),
                buttons: Vec::new(),
                channel: 0,
                dest: "0.0.0.0:0".parse().unwrap(),
            },
            handler,
            100,
        );

        scene
            .service_registry
            .write()
            .register::<DialogModule>(Arc::new(DialogModule {
                region_uuid: self.region_uuid,
                socket: self.socket.clone(),
                session_manager: self.session_manager.clone(),
                service_registry: self.service_registry.clone(),
            }));

        info!("[DIALOG MODULE] Added to region {:?}", scene.region_name);
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
impl SharedRegionModule for DialogModule {}

struct DialogEventHandler {
    socket: Arc<tokio::net::UdpSocket>,
    session_manager: Arc<SessionManager>,
}

#[async_trait]
impl EventHandler for DialogEventHandler {
    async fn handle_event(&self, event: &SceneEvent, _scene: &SceneContext) -> Result<()> {
        if let SceneEvent::OnScriptDialog {
            avatar_id,
            object_id,
            object_name,
            owner_id,
            message,
            buttons,
            channel,
            dest,
        } = event
        {
            let (first, last) =
                if let Some(session) = self.session_manager.get_session_by_agent_id(*avatar_id) {
                    (session.first_name.clone(), session.last_name.clone())
                } else {
                    ("Object".to_string(), "Owner".to_string())
                };

            let packet = build_script_dialog(
                object_id,
                &first,
                &last,
                object_name,
                message,
                *channel,
                buttons,
                owner_id,
            );
            let _ = self.socket.send_to(&packet, dest).await;

            info!(
                "[DIALOG MODULE] Sent ScriptDialog to {} ({} buttons)",
                dest,
                buttons.len()
            );
        }
        Ok(())
    }
}
