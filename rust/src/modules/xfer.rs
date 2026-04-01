use std::any::Any;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::services::ServiceRegistry;
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};

pub trait IXferModule: Send + Sync + 'static {
    fn request_xfer(&self, file_name: &str, data: Vec<u8>, dest: SocketAddr) -> u64;
}

const XFER_CHUNK_SIZE: usize = 1000;
const SEND_XFER_PACKET_ID: u32 = 0xFFFF0014; // Low 20
const CONFIRM_XFER_PACKET_ID: u32 = 0xFFFF0013; // Low 19

struct XferState {
    xfer_id: u64,
    data: Vec<u8>,
    packet_index: u32,
    dest: SocketAddr,
    complete: bool,
}

pub struct XferModule {
    transfers: Arc<RwLock<HashMap<u64, XferState>>>,
    next_xfer_id: Arc<std::sync::atomic::AtomicU64>,
    socket: Option<Arc<tokio::net::UdpSocket>>,
    service_registry: Option<Arc<RwLock<ServiceRegistry>>>,
}

impl XferModule {
    pub fn new() -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            next_xfer_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            socket: None,
            service_registry: None,
        }
    }

    pub fn start_xfer(&self, data: Vec<u8>, dest: SocketAddr) -> u64 {
        let xfer_id = self.next_xfer_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let state = XferState {
            xfer_id,
            data,
            packet_index: 0,
            dest,
            complete: false,
        };
        self.transfers.write().insert(xfer_id, state);
        xfer_id
    }

    pub fn build_xfer_packet(xfer_id: u64, packet_num: u32, data: &[u8], is_last: bool) -> Vec<u8> {
        let mut packet = Vec::with_capacity(data.len() + 30);
        packet.push(0x40); // reliable
        packet.extend_from_slice(&0u32.to_be_bytes());
        packet.push(0);
        packet.extend_from_slice(&[0xFF, 0xFF]);
        packet.extend_from_slice(&0x0014u16.to_be_bytes());

        packet.extend_from_slice(&xfer_id.to_le_bytes());

        let mut pkt_num = packet_num;
        if is_last {
            pkt_num |= 0x80000000;
        }
        packet.extend_from_slice(&pkt_num.to_le_bytes());

        let data_len = (data.len() as u16).to_le_bytes();
        packet.extend_from_slice(&data_len);
        packet.extend_from_slice(data);

        packet
    }

    pub fn handle_confirm(&self, xfer_id: u64, packet_num: u32) -> Option<(Vec<u8>, SocketAddr)> {
        let mut transfers = self.transfers.write();
        if let Some(state) = transfers.get_mut(&xfer_id) {
            let next_index = packet_num + 1;
            state.packet_index = next_index;

            let offset = next_index as usize * XFER_CHUNK_SIZE;
            if offset >= state.data.len() {
                state.complete = true;
                let dest = state.dest;
                transfers.remove(&xfer_id);
                debug!("[XFER] Transfer {} complete", xfer_id);
                return None;
            }

            let end = (offset + XFER_CHUNK_SIZE).min(state.data.len());
            let chunk = state.data[offset..end].to_vec();
            let is_last = end >= state.data.len();
            let dest = state.dest;

            let packet = Self::build_xfer_packet(xfer_id, next_index, &chunk, is_last);
            Some((packet, dest))
        } else {
            None
        }
    }

    pub fn handle_abort(&self, xfer_id: u64) {
        self.transfers.write().remove(&xfer_id);
        info!("[XFER] Transfer {} aborted", xfer_id);
    }

    pub fn send_first_packet(&self, xfer_id: u64) -> Option<(Vec<u8>, SocketAddr)> {
        let transfers = self.transfers.read();
        if let Some(state) = transfers.get(&xfer_id) {
            let end = XFER_CHUNK_SIZE.min(state.data.len());
            let is_last = end >= state.data.len();

            let mut first_data = Vec::with_capacity(4 + end);
            first_data.extend_from_slice(&(state.data.len() as u32).to_le_bytes());
            first_data.extend_from_slice(&state.data[..end]);

            let packet = Self::build_xfer_packet(xfer_id, 0, &first_data, is_last);
            Some((packet, state.dest))
        } else {
            None
        }
    }
}

impl IXferModule for XferModule {
    fn request_xfer(&self, _file_name: &str, data: Vec<u8>, dest: SocketAddr) -> u64 {
        self.start_xfer(data, dest)
    }
}

#[async_trait]
impl RegionModule for XferModule {
    fn name(&self) -> &'static str { "XferModule" }
    fn replaceable_interface(&self) -> Option<&'static str> { Some("IXferModule") }

    async fn initialize(&mut self, _config: &ModuleConfig) -> Result<()> {
        info!("[XFER MODULE] Initialized");
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.socket = Some(scene.socket.clone());
        self.service_registry = Some(scene.service_registry.clone());

        scene.service_registry.write().register::<XferModule>(
            Arc::new(XferModule {
                transfers: self.transfers.clone(),
                next_xfer_id: self.next_xfer_id.clone(),
                socket: self.socket.clone(),
                service_registry: self.service_registry.clone(),
            }),
        );

        info!("[XFER MODULE] Added to region {:?}", scene.region_name);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[async_trait]
impl SharedRegionModule for XferModule {}
