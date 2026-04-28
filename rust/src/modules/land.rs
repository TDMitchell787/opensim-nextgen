use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::database::multi_backend::DatabaseConnection;

use super::events::{EventHandler, SceneEvent};
use super::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};

const REGION_SIZE: u32 = 256;
const BITMAP_SIDE: u32 = 64;
const BITMAP_BYTES: usize = 512;

#[derive(Debug, Clone)]
pub struct Parcel {
    pub local_id: i32,
    pub uuid: Uuid,
    pub region_uuid: Uuid,
    pub owner_id: Uuid,
    pub group_id: Uuid,
    pub is_group_owned: bool,
    pub name: String,
    pub description: String,
    pub flags: u32,
    pub landing_type: u8,
    pub landing_point: [f32; 3],
    pub landing_look_at: [f32; 3],
    pub area: i32,
    pub category: u8,
    pub claim_date: i32,
    pub sale_price: i32,
    pub auction_id: i32,
    pub status: u8,
    pub music_url: String,
    pub media_url: String,
    pub media_auto_scale: bool,
    pub snapshot_id: Uuid,
    pub dwell: f32,
    pub pass_hours: f32,
    pub pass_price: i32,
    pub other_clean_time: i32,
    pub auth_buyer_id: Uuid,
    pub max_prims: i32,
    pub sim_wide_max_prims: i32,
    pub bitmap: Vec<u8>,
    pub access_list: Vec<ParcelAccessEntry>,
    pub ban_list: Vec<ParcelAccessEntry>,
}

#[derive(Debug, Clone)]
pub struct ParcelAccessEntry {
    pub agent_id: Uuid,
    pub flags: i32,
    pub expires: i64,
}

impl Parcel {
    pub fn new_default(region_uuid: Uuid, owner_id: Uuid) -> Self {
        let mut bitmap = vec![0xFFu8; BITMAP_BYTES];
        Self {
            local_id: 1,
            uuid: Uuid::new_v4(),
            region_uuid,
            owner_id,
            group_id: Uuid::nil(),
            is_group_owned: false,
            name: "Your Parcel".to_string(),
            description: String::new(),
            flags: 0x03FFFFFF,
            landing_type: 0,
            landing_point: [128.0, 128.0, 25.0],
            landing_look_at: [1.0, 0.0, 0.0],
            area: (REGION_SIZE * REGION_SIZE) as i32,
            category: 0,
            claim_date: 0,
            sale_price: -1,
            auction_id: 0,
            status: 0,
            music_url: String::new(),
            media_url: String::new(),
            media_auto_scale: false,
            snapshot_id: Uuid::nil(),
            dwell: 0.0,
            pass_hours: 0.0,
            pass_price: 0,
            other_clean_time: 0,
            auth_buyer_id: Uuid::nil(),
            max_prims: 20000,
            sim_wide_max_prims: 20000,
            bitmap,
            access_list: Vec::new(),
            ban_list: Vec::new(),
        }
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        let bx = (x / 4.0) as usize;
        let by = (y / 4.0) as usize;
        if bx >= BITMAP_SIDE as usize || by >= BITMAP_SIDE as usize {
            return false;
        }
        let byte_idx = by * (BITMAP_SIDE as usize / 8) + bx / 8;
        let bit_idx = bx % 8;
        if byte_idx >= self.bitmap.len() {
            return false;
        }
        (self.bitmap[byte_idx] & (1 << bit_idx)) != 0
    }
}

pub trait ILandModule: Send + Sync + 'static {
    fn get_parcel_at(&self, x: f32, y: f32) -> Option<Parcel>;
    fn get_parcel_by_id(&self, local_id: i32) -> Option<Parcel>;
    fn get_all_parcels(&self) -> Vec<Parcel>;
    fn can_build_at(&self, agent_id: &Uuid, x: f32, y: f32) -> bool;
}

pub struct LandManagementModule {
    parcels: Arc<RwLock<Vec<Parcel>>>,
    region_uuid: Option<Uuid>,
    socket: Option<Arc<tokio::net::UdpSocket>>,
    session_manager: Option<Arc<crate::session::SessionManager>>,
    avatar_states: Option<Arc<RwLock<HashMap<Uuid, crate::udp::server::AvatarMovementState>>>>,
    db: Option<Arc<DatabaseConnection>>,
}

impl LandManagementModule {
    pub fn new() -> Self {
        Self {
            parcels: Arc::new(RwLock::new(Vec::new())),
            region_uuid: None,
            socket: None,
            session_manager: None,
            avatar_states: None,
            db: None,
        }
    }

    async fn load_parcels_from_db(
        db: &DatabaseConnection,
        region_uuid: &Uuid,
    ) -> Result<Vec<Parcel>> {
        let mut parcels = Vec::new();

        match db {
            DatabaseConnection::PostgreSQL(pool) => {
                let rows = sqlx::query(
                    "SELECT uuid, regionuuid, locallandid, name, description, \
                     owneruuid, groupuuid, isgroupowned, area, auctionid, \
                     category, claimdate, claimprice, landstatus, landflags, \
                     landingtype, mediaautoscale, mediatextureuuid, mediaurl, \
                     musicurl, passhours, passprice, snapshotuuid, \
                     userlocationx, userlocationy, userlocationz, \
                     userlookatx, userlookaty, userlookatz, \
                     authbuyerid, othercleantime, dwell, saleprice, bitmap \
                     FROM land WHERE regionuuid = $1::uuid",
                )
                .bind(region_uuid.to_string())
                .fetch_all(pool)
                .await;

                match rows {
                    Ok(rows) => {
                        use sqlx::Row;
                        for row in rows {
                            let parcel_uuid: Uuid =
                                row.try_get("uuid").unwrap_or_else(|_| Uuid::new_v4());
                            let owner_uuid: Uuid =
                                row.try_get("owneruuid").unwrap_or_else(|_| Uuid::nil());
                            let group_uuid: Uuid =
                                row.try_get("groupuuid").unwrap_or_else(|_| Uuid::nil());
                            let snapshot_uuid: Uuid =
                                row.try_get("snapshotuuid").unwrap_or_else(|_| Uuid::nil());
                            let auth_buyer_uuid: Uuid =
                                row.try_get("authbuyerid").unwrap_or_else(|_| Uuid::nil());

                            let bitmap_data: Option<Vec<u8>> = row.try_get("bitmap").ok();
                            let bitmap = bitmap_data.unwrap_or_else(|| vec![0xFF; BITMAP_BYTES]);

                            let p = Parcel {
                                local_id: row.get::<i32, _>("locallandid"),
                                uuid: parcel_uuid,
                                region_uuid: *region_uuid,
                                owner_id: owner_uuid,
                                group_id: group_uuid,
                                is_group_owned: row
                                    .try_get::<i32, _>("isgroupowned")
                                    .map(|v| v != 0)
                                    .unwrap_or(false),
                                name: row.get("name"),
                                description: row.try_get("description").unwrap_or_default(),
                                flags: row.try_get::<i32, _>("landflags").unwrap_or(0x03FFFFFF)
                                    as u32,
                                landing_type: row.try_get::<i32, _>("landingtype").unwrap_or(0)
                                    as u8,
                                landing_point: [
                                    row.try_get::<f32, _>("userlocationx").unwrap_or(128.0),
                                    row.try_get::<f32, _>("userlocationy").unwrap_or(128.0),
                                    row.try_get::<f32, _>("userlocationz").unwrap_or(25.0),
                                ],
                                landing_look_at: [
                                    row.try_get::<f32, _>("userlookatx").unwrap_or(1.0),
                                    row.try_get::<f32, _>("userlookaty").unwrap_or(0.0),
                                    row.try_get::<f32, _>("userlookatz").unwrap_or(0.0),
                                ],
                                area: row.try_get::<i32, _>("area").unwrap_or(65536),
                                category: row.try_get::<i32, _>("category").unwrap_or(0) as u8,
                                claim_date: row.try_get::<i32, _>("claimdate").unwrap_or(0),
                                sale_price: row.try_get::<i32, _>("saleprice").unwrap_or(-1),
                                auction_id: row.try_get::<i32, _>("auctionid").unwrap_or(0),
                                status: row.try_get::<i32, _>("landstatus").unwrap_or(0) as u8,
                                music_url: row.try_get("musicurl").unwrap_or_default(),
                                media_url: row.try_get("mediaurl").unwrap_or_default(),
                                media_auto_scale: row
                                    .try_get::<i32, _>("mediaautoscale")
                                    .unwrap_or(0)
                                    != 0,
                                snapshot_id: snapshot_uuid,
                                dwell: row.try_get::<f32, _>("dwell").unwrap_or(0.0),
                                pass_hours: row.try_get::<f32, _>("passhours").unwrap_or(0.0),
                                pass_price: row.try_get::<i32, _>("passprice").unwrap_or(0),
                                other_clean_time: row
                                    .try_get::<i32, _>("othercleantime")
                                    .unwrap_or(0),
                                auth_buyer_id: auth_buyer_uuid,
                                max_prims: 20000,
                                sim_wide_max_prims: 20000,
                                bitmap,
                                access_list: Vec::new(),
                                ban_list: Vec::new(),
                            };
                            parcels.push(p);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "[LAND MODULE] No land table or error loading parcels: {}",
                            e
                        );
                    }
                }
            }
            DatabaseConnection::MySQL(_pool) => {
                warn!("[LAND MODULE] MySQL parcel loading not implemented yet");
            }
        }

        Ok(parcels)
    }

    fn build_parcel_overlay(&self) -> Vec<u8> {
        let parcels = self.parcels.read();
        let mut overlay = vec![0u8; (REGION_SIZE * REGION_SIZE / 4) as usize];

        for y in 0..REGION_SIZE {
            for x in 0..REGION_SIZE {
                let idx = (y * REGION_SIZE + x) as usize;
                let byte_idx = idx / 4;
                let shift = (idx % 4) * 2;

                let mut owner_type = 0u8;
                for parcel in parcels.iter() {
                    if parcel.contains_point(x as f32, y as f32) {
                        if parcel.sale_price >= 0 {
                            owner_type = 3;
                        } else if parcel.is_group_owned {
                            owner_type = 2;
                        } else {
                            owner_type = 1;
                        }
                        break;
                    }
                }

                if byte_idx < overlay.len() {
                    overlay[byte_idx] |= owner_type << shift;
                }
            }
        }

        overlay
    }
}

impl ILandModule for LandManagementModule {
    fn get_parcel_at(&self, x: f32, y: f32) -> Option<Parcel> {
        let parcels = self.parcels.read();
        for p in parcels.iter() {
            if p.contains_point(x, y) {
                return Some(p.clone());
            }
        }
        None
    }

    fn get_parcel_by_id(&self, local_id: i32) -> Option<Parcel> {
        let parcels = self.parcels.read();
        parcels.iter().find(|p| p.local_id == local_id).cloned()
    }

    fn get_all_parcels(&self) -> Vec<Parcel> {
        self.parcels.read().clone()
    }

    fn can_build_at(&self, agent_id: &Uuid, x: f32, y: f32) -> bool {
        if let Some(parcel) = self.get_parcel_at(x, y) {
            parcel.owner_id == *agent_id || parcel.group_id != Uuid::nil()
        } else {
            true
        }
    }
}

#[async_trait]
impl RegionModule for LandManagementModule {
    fn name(&self) -> &'static str {
        "LandManagementModule"
    }

    fn replaceable_interface(&self) -> Option<&'static str> {
        Some("ILandModule")
    }

    async fn initialize(&mut self, _config: &ModuleConfig) -> Result<()> {
        info!("[LAND MODULE] Initialized");
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = Some(scene.region_uuid);
        self.socket = Some(scene.socket.clone());
        self.session_manager = Some(scene.session_manager.clone());
        self.avatar_states = Some(scene.avatar_states.clone());
        self.db = scene.db.clone();

        if let Some(db) = &scene.db {
            match Self::load_parcels_from_db(db, &scene.region_uuid).await {
                Ok(loaded) if !loaded.is_empty() => {
                    info!(
                        "[LAND MODULE] Loaded {} parcels from database",
                        loaded.len()
                    );
                    *self.parcels.write() = loaded;
                }
                Ok(_) => {
                    info!("[LAND MODULE] No parcels in DB, creating default parcel");
                    let default = Parcel::new_default(scene.region_uuid, Uuid::nil());
                    *self.parcels.write() = vec![default];
                }
                Err(e) => {
                    warn!("[LAND MODULE] DB load error: {}, using default parcel", e);
                    let default = Parcel::new_default(scene.region_uuid, Uuid::nil());
                    *self.parcels.write() = vec![default];
                }
            }
        } else {
            let default = Parcel::new_default(scene.region_uuid, Uuid::nil());
            *self.parcels.write() = vec![default];
        }

        let handler = Arc::new(ParcelEventHandler {
            parcels: self.parcels.clone(),
        });

        scene.event_bus.subscribe(
            SceneEvent::OnParcelPropertiesRequest {
                agent_id: Uuid::nil(),
                sequence_id: 0,
                west: 0.0,
                south: 0.0,
                east: 0.0,
                north: 0.0,
                snap_selection: false,
                dest: "0.0.0.0:0".parse().unwrap(),
            },
            handler.clone(),
            100,
        );

        scene.event_bus.subscribe(
            SceneEvent::OnParcelPropertiesUpdate {
                agent_id: Uuid::nil(),
                local_id: 0,
                flags: 0,
                parcel_flags: 0,
                sale_price: 0,
                name: String::new(),
                description: String::new(),
            },
            handler,
            100,
        );

        scene
            .service_registry
            .write()
            .register::<LandManagementModule>(Arc::new(LandManagementModule {
                parcels: self.parcels.clone(),
                region_uuid: self.region_uuid,
                socket: self.socket.clone(),
                session_manager: self.session_manager.clone(),
                avatar_states: self.avatar_states.clone(),
                db: self.db.clone(),
            }));

        info!(
            "[LAND MODULE] Added to region '{}' with {} parcels",
            scene.region_name,
            self.parcels.read().len()
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
impl SharedRegionModule for LandManagementModule {}

struct ParcelEventHandler {
    parcels: Arc<RwLock<Vec<Parcel>>>,
}

const PARCEL_PROPERTIES_ID: u32 = 0xFFFF0017;

impl ParcelEventHandler {
    fn build_parcel_properties_packet(
        &self,
        parcel: &Parcel,
        sequence_id: i32,
        agent_id: &Uuid,
    ) -> Vec<u8> {
        let mut packet = Vec::with_capacity(1024);

        // Header: reliable
        packet.push(0x40);
        packet.extend_from_slice(&0u32.to_be_bytes());
        packet.push(0);
        packet.extend_from_slice(&[0xFF, 0xFF]);
        packet.extend_from_slice(&0x0017u16.to_be_bytes());

        // ParcelData block
        packet.extend_from_slice(&(sequence_id as i32).to_le_bytes());
        packet.extend_from_slice(&parcel.local_id.to_le_bytes());
        packet.extend_from_slice(parcel.owner_id.as_bytes());
        packet.push(if parcel.is_group_owned { 1 } else { 0 });
        packet.extend_from_slice(parcel.group_id.as_bytes());
        packet.extend_from_slice(&parcel.claim_date.to_le_bytes());
        packet.extend_from_slice(&0i32.to_le_bytes()); // ClaimPrice
        packet.extend_from_slice(&0i32.to_le_bytes()); // RentPrice

        let aabb_min = [0.0f32, 0.0, 0.0];
        let aabb_max = [256.0f32, 256.0, 0.0];
        for v in &aabb_min {
            packet.extend_from_slice(&v.to_le_bytes());
        }
        for v in &aabb_max {
            packet.extend_from_slice(&v.to_le_bytes());
        }

        // Bitmap (variable-2)
        let bitmap_len = parcel.bitmap.len() as u16;
        packet.extend_from_slice(&bitmap_len.to_le_bytes());
        packet.extend_from_slice(&parcel.bitmap);

        packet.extend_from_slice(&parcel.area.to_le_bytes());
        packet.push(parcel.status);
        packet.extend_from_slice(&parcel.sim_wide_max_prims.to_le_bytes());
        packet.extend_from_slice(&parcel.sim_wide_max_prims.to_le_bytes()); // SimWideTotalPrims
        packet.extend_from_slice(&parcel.max_prims.to_le_bytes());
        packet.extend_from_slice(&0i32.to_le_bytes()); // TotalPrims
        packet.extend_from_slice(&0i32.to_le_bytes()); // OwnerPrims
        packet.extend_from_slice(&0i32.to_le_bytes()); // GroupPrims
        packet.extend_from_slice(&0i32.to_le_bytes()); // OtherPrims
        packet.extend_from_slice(&0i32.to_le_bytes()); // SelectedPrims
        packet.extend_from_slice(&0.0f32.to_le_bytes()); // ParcelPrimBonus
        packet.extend_from_slice(&parcel.other_clean_time.to_le_bytes());
        packet.extend_from_slice(&parcel.flags.to_le_bytes());
        packet.extend_from_slice(&parcel.sale_price.to_le_bytes());

        // Name (variable-1)
        let name_bytes = parcel.name.as_bytes();
        packet.push(name_bytes.len() as u8 + 1);
        packet.extend_from_slice(name_bytes);
        packet.push(0);

        // Description (variable-1)
        let desc_bytes = parcel.description.as_bytes();
        packet.push(desc_bytes.len() as u8 + 1);
        packet.extend_from_slice(desc_bytes);
        packet.push(0);

        // MusicURL (variable-1)
        let music_bytes = parcel.music_url.as_bytes();
        packet.push(music_bytes.len() as u8 + 1);
        packet.extend_from_slice(music_bytes);
        packet.push(0);

        // MediaURL (variable-1)
        let media_bytes = parcel.media_url.as_bytes();
        packet.push(media_bytes.len() as u8 + 1);
        packet.extend_from_slice(media_bytes);
        packet.push(0);

        // MediaID
        packet.extend_from_slice(Uuid::nil().as_bytes());
        // MediaAutoScale
        packet.push(if parcel.media_auto_scale { 1 } else { 0 });

        // SnapshotID
        packet.extend_from_slice(parcel.snapshot_id.as_bytes());

        // UserLocation
        for v in &parcel.landing_point {
            packet.extend_from_slice(&v.to_le_bytes());
        }
        // UserLookAt
        for v in &parcel.landing_look_at {
            packet.extend_from_slice(&v.to_le_bytes());
        }

        // LandingType
        packet.push(parcel.landing_type);
        // RegionPushOverride
        packet.push(0);
        // RegionDenyAnonymous
        packet.push(0);
        // RegionDenyIdentified
        packet.push(0);
        // RegionDenyTransacted
        packet.push(0);

        // GroupPrims... (padding for missing fields viewer expects)
        packet.push(parcel.category);
        packet.extend_from_slice(parcel.auth_buyer_id.as_bytes());
        packet.extend_from_slice(&parcel.dwell.to_le_bytes());

        // AgeVerificationBlock: RegionDenyAgeUnverified
        packet.push(1); // block count
        packet.push(0); // false

        packet
    }
}

#[async_trait]
impl EventHandler for ParcelEventHandler {
    async fn handle_event(&self, event: &SceneEvent, scene: &SceneContext) -> Result<()> {
        match event {
            SceneEvent::OnParcelPropertiesRequest {
                agent_id,
                sequence_id: _,
                west,
                south,
                east,
                north,
                snap_selection: _,
                dest: _,
            } => {
                let cx = (west + east) / 2.0;
                let cy = (south + north) / 2.0;
                let parcels = self.parcels.read();
                let parcel = parcels
                    .iter()
                    .find(|p| p.contains_point(cx, cy))
                    .or_else(|| parcels.first());
                if let Some(p) = parcel {
                    info!(
                        "[LAND MODULE] Parcel '{}' (id={}) at ({:.0},{:.0}) for {}",
                        p.name, p.local_id, cx, cy, agent_id,
                    );
                }
            }
            SceneEvent::OnParcelPropertiesUpdate {
                agent_id,
                local_id,
                flags,
                parcel_flags,
                sale_price,
                name,
                description,
            } => {
                let mut parcels = self.parcels.write();
                if let Some(parcel) = parcels.iter_mut().find(|p| p.local_id == *local_id) {
                    if parcel.owner_id == *agent_id || parcel.group_id != Uuid::nil() {
                        if !name.is_empty() {
                            parcel.name = name.clone();
                        }
                        if !description.is_empty() {
                            parcel.description = description.clone();
                        }
                        parcel.flags = *parcel_flags;
                        parcel.sale_price = *sale_price;
                        info!("[LAND MODULE] Updated parcel {} by {}", local_id, agent_id);
                    } else {
                        warn!(
                            "[LAND MODULE] Agent {} denied update to parcel {}",
                            agent_id, local_id
                        );
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
