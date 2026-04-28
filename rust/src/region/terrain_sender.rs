use crate::database::multi_backend::DatabaseConnection;
use crate::protocol::terrain::LayerType;
use crate::region::terrain_storage::{TerrainRevision, TerrainStorage};
use crate::udp::messages::LayerDataMessage;
use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::sleep;
use tracing::{info, warn};
use uuid::Uuid;

pub struct TerrainSender {
    terrain_storage: TerrainStorage,
    udp_socket: Arc<UdpSocket>,
    region_size_x: u32,
    region_size_y: u32,
    cached_packets: RwLock<HashMap<Uuid, Vec<LayerDataMessage>>>,
    cached_heightmaps: RwLock<HashMap<Uuid, Vec<f32>>>,
}

impl TerrainSender {
    pub fn new(db_connection: Arc<DatabaseConnection>, udp_socket: Arc<UdpSocket>) -> Self {
        Self {
            terrain_storage: TerrainStorage::new(db_connection),
            udp_socket,
            region_size_x: 256,
            region_size_y: 256,
            cached_packets: RwLock::new(HashMap::new()),
            cached_heightmaps: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_region_size(mut self, size_x: u32, size_y: u32) -> Self {
        self.region_size_x = size_x;
        self.region_size_y = size_y;
        self
    }

    pub async fn get_terrain_packets(&self, region_uuid: Uuid) -> Result<Vec<LayerDataMessage>> {
        {
            let cache = self.cached_packets.read();
            if let Some(packets) = cache.get(&region_uuid) {
                info!(
                    "[TERRAIN] Serving {} cached LayerData packets for region {}",
                    packets.len(),
                    region_uuid
                );
                return Ok(packets.clone());
            }
        }

        info!(
            "[TERRAIN] Loading terrain from DB for region {} (first time)",
            region_uuid
        );

        let heightmap = match self.terrain_storage.load_terrain(region_uuid).await? {
            Some(terrain) => {
                info!("[TERRAIN] Loaded terrain from database");
                terrain
            }
            None => {
                info!("[TERRAIN] No terrain found, using default flat terrain at 21m");
                let default_heightmap = TerrainStorage::create_default_heightmap_sized(
                    self.region_size_x,
                    self.region_size_y,
                );

                if let Err(e) = self
                    .terrain_storage
                    .store_terrain(region_uuid, &default_heightmap, TerrainRevision::Variable2D)
                    .await
                {
                    warn!("[TERRAIN] Failed to store default terrain: {}", e);
                }

                default_heightmap
            }
        };

        self.cached_heightmaps
            .write()
            .insert(region_uuid, heightmap.clone());

        let packets = LayerDataMessage::split_into_packets(&heightmap, LayerType::Land)?;
        info!(
            "[TERRAIN] Created {} LayerData packets (now cached)",
            packets.len()
        );

        self.cached_packets
            .write()
            .insert(region_uuid, packets.clone());

        Ok(packets)
    }

    pub async fn send_terrain_to_client(
        &self,
        region_uuid: Uuid,
        client_addr: SocketAddr,
        mut sequence: u32,
    ) -> Result<()> {
        info!(
            "[TERRAIN] Sending terrain for region {} to {}",
            region_uuid, client_addr
        );

        let packets = self.get_terrain_packets(region_uuid).await?;

        info!("[TERRAIN] Sending {} LayerData packets", packets.len());

        for (i, message) in packets.iter().enumerate() {
            let udp_packet = message.to_udp_packet(sequence, true);

            // Phase 70.10: Debug first packet bytes for terrain analysis
            if i == 0 {
                tracing::warn!(
                    "[TERRAIN] First LayerData packet (seq={}) total {} bytes:",
                    sequence,
                    udp_packet.len()
                );
                tracing::warn!(
                    "[TERRAIN] Header (first 10 bytes): {:02x?}",
                    &udp_packet[..udp_packet.len().min(10)]
                );
                tracing::warn!(
                    "[TERRAIN] Body (next 20 bytes): {:02x?}",
                    &udp_packet[10..udp_packet.len().min(30)]
                );
                tracing::warn!(
                    "[TERRAIN] Data bytes (10-40): {:02x?}",
                    &udp_packet[10..udp_packet.len().min(40)]
                );
            }

            sequence += 1;

            self.udp_socket.send_to(&udp_packet, client_addr).await?;

            // Phase 70.16: Add throttling between terrain packets to prevent UDP buffer overflow
            // 64 packets were being sent in ~350 microseconds, potentially overwhelming UDP buffers
            // Adding 2ms delay between packets gives viewer time to process each one
            sleep(Duration::from_millis(2)).await;

            if (i + 1) % 16 == 0 {
                info!("[TERRAIN] Sent {}/{} terrain packets", i + 1, packets.len());
            }
        }

        info!("[TERRAIN] ✅ All terrain packets sent to {}", client_addr);

        Ok(())
    }

    pub async fn initialize_region_terrain(&self, region_uuid: Uuid) -> Result<()> {
        let heightmap = match self.terrain_storage.load_terrain(region_uuid).await? {
            Some(terrain) => {
                info!(
                    "[TERRAIN] Loaded existing terrain for region {} into cache",
                    region_uuid
                );
                terrain
            }
            None => {
                info!(
                    "[TERRAIN] Initializing default terrain for region {}",
                    region_uuid
                );
                let default_heightmap = TerrainStorage::create_default_heightmap_sized(
                    self.region_size_x,
                    self.region_size_y,
                );
                self.terrain_storage
                    .store_terrain(region_uuid, &default_heightmap, TerrainRevision::Variable2D)
                    .await?;
                default_heightmap
            }
        };

        self.cached_heightmaps
            .write()
            .insert(region_uuid, heightmap.clone());

        let packets = LayerDataMessage::split_into_packets(&heightmap, LayerType::Land)?;
        info!(
            "[TERRAIN] Pre-cached {} terrain packets for region {}",
            packets.len(),
            region_uuid
        );
        self.cached_packets.write().insert(region_uuid, packets);

        Ok(())
    }

    pub fn invalidate_cache(&self, region_uuid: Uuid) {
        self.cached_packets.write().remove(&region_uuid);
        self.cached_heightmaps.write().remove(&region_uuid);
        info!("[TERRAIN] Cache invalidated for region {}", region_uuid);
    }

    pub async fn store_and_cache_heightmap(
        &self,
        region_uuid: Uuid,
        heightmap: Vec<f32>,
    ) -> Result<()> {
        self.invalidate_cache(region_uuid);

        self.terrain_storage
            .store_terrain(region_uuid, &heightmap, TerrainRevision::Variable2D)
            .await?;

        let packets = LayerDataMessage::split_into_packets(&heightmap, LayerType::Land)?;
        info!(
            "[TERRAIN] Stored new terrain and cached {} packets for region {}",
            packets.len(),
            region_uuid
        );
        self.cached_heightmaps
            .write()
            .insert(region_uuid, heightmap);
        self.cached_packets.write().insert(region_uuid, packets);

        Ok(())
    }

    pub async fn broadcast_full_terrain(
        &self,
        region_uuid: Uuid,
        client_addrs: &[SocketAddr],
    ) -> Result<()> {
        if client_addrs.is_empty() {
            info!("[TERRAIN] No clients to broadcast terrain to");
            return Ok(());
        }

        let packets = self.get_terrain_packets(region_uuid).await?;
        info!(
            "[TERRAIN] Broadcasting {} terrain packets to {} clients",
            packets.len(),
            client_addrs.len()
        );

        for addr in client_addrs {
            let mut sequence = 0u32;
            for (i, message) in packets.iter().enumerate() {
                let udp_packet = message.to_udp_packet(sequence, true);
                sequence += 1;
                self.udp_socket.send_to(&udp_packet, addr).await?;
                sleep(Duration::from_millis(2)).await;
            }
            info!("[TERRAIN] Sent all terrain packets to {}", addr);
        }

        Ok(())
    }

    pub async fn get_heightmap(&self, region_uuid: Uuid) -> Result<Vec<f32>> {
        {
            let cache = self.cached_heightmaps.read();
            if let Some(hm) = cache.get(&region_uuid) {
                return Ok(hm.clone());
            }
        }

        let hm = match self.terrain_storage.load_terrain(region_uuid).await? {
            Some(terrain) => terrain,
            None => TerrainStorage::create_default_heightmap(),
        };
        self.cached_heightmaps
            .write()
            .insert(region_uuid, hm.clone());
        Ok(hm)
    }
}
