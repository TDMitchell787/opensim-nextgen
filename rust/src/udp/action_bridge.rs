use anyhow::{anyhow, bail, Result};
use bytes::BufMut;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{info, warn};
use uuid::Uuid;

use super::reliability::ReliabilityManager;
use super::server::{AvatarMovementState, SceneObject};
use crate::caps::CapsManager;
use crate::database::multi_backend::DatabaseConnection;
use crate::session::SessionManager;

const OBJECT_UPDATE_ID: u32 = 0x0C;
const KILL_OBJECT_ID: u32 = 0x00000010;
const CHAT_FROM_SIMULATOR_ID: u32 = 0xFFFF008B;
const FLAG_RELIABLE: u8 = 0x40;
const FLAG_ZEROCODED: u8 = 0x80;

#[derive(Clone)]
pub struct PendingTerrain {
    pub heightmap: Vec<f32>,
    pub region_uuid: Uuid,
    pub preset: String,
    pub seed: u32,
    pub r32_path: String,
    pub preview_local_id: u32,
    pub grid_size: Option<u32>,
    pub grid_x: Option<u32>,
    pub grid_y: Option<u32>,
}

#[derive(Clone)]
pub struct ActionBridge {
    socket: Arc<UdpSocket>,
    scene_objects: Arc<parking_lot::RwLock<HashMap<u32, SceneObject>>>,
    avatar_states: Arc<parking_lot::RwLock<HashMap<Uuid, AvatarMovementState>>>,
    next_prim_local_id: Arc<AtomicU32>,
    session_manager: Arc<SessionManager>,
    reliability_manager: Arc<ReliabilityManager>,
    db_connection: Option<Arc<DatabaseConnection>>,
    yengine: Option<Arc<parking_lot::RwLock<crate::scripting::yengine_module::YEngineModule>>>,
    caps_manager: Option<Arc<CapsManager>>,
    region_name: String,
    region_handle: u64,
    pub default_region_uuid: Uuid,
    pending_terrains: Arc<parking_lot::RwLock<HashMap<String, PendingTerrain>>>,
}

impl ActionBridge {
    pub fn new(
        socket: Arc<UdpSocket>,
        scene_objects: Arc<parking_lot::RwLock<HashMap<u32, SceneObject>>>,
        avatar_states: Arc<parking_lot::RwLock<HashMap<Uuid, AvatarMovementState>>>,
        next_prim_local_id: Arc<AtomicU32>,
        session_manager: Arc<SessionManager>,
        reliability_manager: Arc<ReliabilityManager>,
        db_connection: Option<Arc<DatabaseConnection>>,
        yengine: Option<Arc<parking_lot::RwLock<crate::scripting::yengine_module::YEngineModule>>>,
        caps_manager: Option<Arc<CapsManager>>,
        region_name: String,
        region_handle: u64,
        default_region_uuid: Uuid,
    ) -> Self {
        Self {
            socket,
            scene_objects,
            avatar_states,
            next_prim_local_id,
            session_manager,
            reliability_manager,
            db_connection,
            yengine,
            caps_manager,
            region_name,
            region_handle,
            default_region_uuid,
            pending_terrains: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    pub fn scene_objects_ref(&self) -> &Arc<parking_lot::RwLock<HashMap<u32, SceneObject>>> {
        &self.scene_objects
    }

    pub fn get_avatar_position(&self, agent_id: Uuid) -> Option<[f32; 3]> {
        let states = self.avatar_states.read();
        states.get(&agent_id).map(|s| s.position)
    }

    pub async fn get_terrain_heightmap(&self, region_uuid: Uuid) -> Option<Vec<f32>> {
        use crate::region::terrain_sender::TerrainSender;
        if let Some(db) = &self.db_connection {
            let ts = TerrainSender::new(db.clone(), self.socket.clone());
            match ts.get_heightmap(region_uuid).await {
                Ok(hm) if !hm.is_empty() => Some(hm),
                _ => None,
            }
        } else {
            None
        }
    }

    pub async fn say(
        &self,
        speaker_id: Uuid,
        speaker_name: &str,
        message: &str,
        position: [f32; 3],
    ) -> Result<()> {
        let recipients: Vec<SocketAddr> = {
            let states = self.avatar_states.read();
            states
                .values()
                .filter(|s| !s.is_npc)
                .map(|s| s.client_addr)
                .collect()
        };

        for addr in &recipients {
            self.send_chat_from_simulator(
                message,
                speaker_name,
                speaker_id,
                position,
                1,
                1,
                1,
                *addr,
            )
            .await?;
        }

        info!(
            "[ACTION_BRIDGE] {} says: '{}' to {} viewers",
            speaker_name,
            message,
            recipients.len()
        );
        Ok(())
    }

    pub async fn rez_prim(
        &self,
        shape: PrimShape,
        owner_id: Uuid,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        let local_id = self.next_prim_local_id.fetch_add(1, Ordering::SeqCst);

        let mut obj = match shape {
            PrimShape::Box => SceneObject::new_box(local_id, owner_id, position),
            PrimShape::Cylinder => SceneObject::new_cylinder(local_id, owner_id, position),
            PrimShape::Sphere => SceneObject::new_sphere(local_id, owner_id, position),
            PrimShape::Torus => SceneObject::new_torus(local_id, owner_id, position),
            PrimShape::Tube => SceneObject::new_tube(local_id, owner_id, position),
            PrimShape::Ring => SceneObject::new_ring(local_id, owner_id, position),
            PrimShape::Prism => SceneObject::new_prism(local_id, owner_id, position),
        };
        obj.scale = scale;
        obj.name = name.to_string();

        let obj_uuid = obj.uuid;
        let te_persist = obj.texture_entry.clone();
        {
            let mut objects = self.scene_objects.write();
            objects.insert(local_id, obj.clone());
        }

        self.broadcast_object_update(local_id).await?;

        if let Some(db) = &self.db_connection {
            let db = db.clone();
            let region_uuid = self.default_region_uuid;
            let creator_str = owner_id.to_string();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i32;
            let owner_mask = 0x7FFFFFFFi32;
            let pcode = obj.pcode;
            let material = obj.material;
            let pc = obj.path_curve;
            let prc = obj.profile_curve;
            let pb = obj.path_begin;
            let pe = obj.path_end;
            let psx = obj.path_scale_x;
            let psy = obj.path_scale_y;
            let phx = obj.path_shear_x;
            let phy = obj.path_shear_y;
            let path_twist = obj.path_twist;
            let path_twist_begin = obj.path_twist_begin;
            let path_radius_offset = obj.path_radius_offset;
            let path_taper_x = obj.path_taper_x;
            let path_taper_y = obj.path_taper_y;
            let path_revolutions = obj.path_revolutions;
            let path_skew = obj.path_skew;
            let profile_begin = obj.profile_begin;
            let profile_end = obj.profile_end;
            let profile_hollow = obj.profile_hollow;
            let rot = obj.rotation;
            let extra_params = obj.extra_params.clone();
            let obj_name = name.to_string();
            let scene_objects_for_persist = self.scene_objects.clone();
            tokio::spawn(async move {
                let Some(pool) = db.postgres_pool() else {
                    return;
                };
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let (persist_group_id, persist_link_number) = {
                    let objects = scene_objects_for_persist.read();
                    objects
                        .get(&local_id)
                        .map(|o| (o.scene_group_id, o.link_number))
                        .unwrap_or((obj_uuid, 1))
                };
                let _ = sqlx::query(
                    r#"INSERT INTO prims (
                        uuid, regionuuid, creatorid, ownerid, groupid, lastownerid, scenegroupid,
                        name, text, description, sitname, touchname,
                        objectflags, ownermask, nextownermask, groupmask, everyonemask, basemask,
                        positionx, positiony, positionz,
                        grouppositionx, grouppositiony, grouppositionz,
                        velocityx, velocityy, velocityz,
                        angularvelocityx, angularvelocityy, angularvelocityz,
                        accelerationx, accelerationy, accelerationz,
                        rotationx, rotationy, rotationz, rotationw,
                        sittargetoffsetx, sittargetoffsety, sittargetoffsetz,
                        sittargetorientw, sittargetorientx, sittargetorienty, sittargetorientz,
                        creationdate, material, linknumber, passcollisions, physicsshapetype
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                        $13, $14, $15, $16, $17, $18,
                        $19, $20, $21, $22, $23, $24, $25, $26, $27,
                        $28, $29, $30, $31, $32, $33,
                        $34, $35, $36, $37,
                        $38, $39, $40, $41, $42, $43, $44,
                        $45, $46, $47, $48, $49
                    ) ON CONFLICT (uuid) DO UPDATE SET
                        positionx = $19, positiony = $20, positionz = $21,
                        rotationx = $34, rotationy = $35, rotationz = $36, rotationw = $37,
                        scenegroupid = $7, linknumber = $47"#,
                )
                .bind(obj_uuid)
                .bind(region_uuid)
                .bind(&creator_str)
                .bind(owner_id)
                .bind(Uuid::nil())
                .bind(owner_id)
                .bind(persist_group_id)
                .bind(&obj_name)
                .bind("")
                .bind("")
                .bind("")
                .bind("")
                .bind(0i32)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(position[0])
                .bind(position[1])
                .bind(position[2])
                .bind(position[0])
                .bind(position[1])
                .bind(position[2])
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(rot[0])
                .bind(rot[1])
                .bind(rot[2])
                .bind(rot[3])
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(1.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(now)
                .bind(material as i32)
                .bind(persist_link_number)
                .bind(0i32)
                .bind(0i32)
                .execute(pool)
                .await;

                let _ = sqlx::query(
                    r#"INSERT INTO primshapes (
                        uuid, shape, scalex, scaley, scalez, pcode,
                        pathbegin, pathend, pathscalex, pathscaley,
                        pathshearx, pathsheary, pathskew, pathcurve,
                        pathradiusoffset, pathrevolutions, pathtaperx, pathtapery,
                        pathtwist, pathtwistbegin,
                        profilebegin, profileend, profilecurve, profilehollow,
                        texture, extraparams, state, media
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6,
                        $7, $8, $9, $10, $11, $12, $13, $14,
                        $15, $16, $17, $18, $19, $20,
                        $21, $22, $23, $24, $25, $26, $27, $28
                    ) ON CONFLICT (uuid) DO UPDATE SET
                        texture = $25, scalex = $3, scaley = $4, scalez = $5"#,
                )
                .bind(obj_uuid)
                .bind(prc as i32)
                .bind(scale[0])
                .bind(scale[1])
                .bind(scale[2])
                .bind(pcode as i32)
                .bind(pb as i32)
                .bind(pe as i32)
                .bind(psx as i32)
                .bind(psy as i32)
                .bind(phx as i32)
                .bind(phy as i32)
                .bind(path_skew as i32)
                .bind(pc as i32)
                .bind(path_radius_offset as i32)
                .bind(path_revolutions as i32)
                .bind(path_taper_x as i32)
                .bind(path_taper_y as i32)
                .bind(path_twist as i32)
                .bind(path_twist_begin as i32)
                .bind(profile_begin as i32)
                .bind(profile_end as i32)
                .bind(prc as i32)
                .bind(profile_hollow as i32)
                .bind(&te_persist as &[u8])
                .bind(&extra_params as &[u8])
                .bind(0i32)
                .bind("")
                .execute(pool)
                .await;

                info!(
                    "[ACTION_BRIDGE] Persisted prim {} '{}' to DB",
                    obj_uuid, obj_name
                );
            });
        }

        self.add_to_ai_inventory(obj_uuid, name, owner_id).await;

        info!(
            "[ACTION_BRIDGE] Rezzed {:?} '{}' local_id={} uuid={} at {:?}",
            shape, name, local_id, obj_uuid, position
        );
        Ok(local_id)
    }

    pub async fn rez_box(
        &self,
        owner_id: Uuid,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        self.rez_prim(PrimShape::Box, owner_id, position, scale, name)
            .await
    }

    pub async fn rez_cylinder(
        &self,
        owner_id: Uuid,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        self.rez_prim(PrimShape::Cylinder, owner_id, position, scale, name)
            .await
    }

    pub async fn rez_sphere(
        &self,
        owner_id: Uuid,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        self.rez_prim(PrimShape::Sphere, owner_id, position, scale, name)
            .await
    }

    pub async fn rez_torus(
        &self,
        owner_id: Uuid,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        self.rez_prim(PrimShape::Torus, owner_id, position, scale, name)
            .await
    }

    pub async fn rez_tube(
        &self,
        owner_id: Uuid,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        self.rez_prim(PrimShape::Tube, owner_id, position, scale, name)
            .await
    }

    pub async fn rez_ring(
        &self,
        owner_id: Uuid,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        self.rez_prim(PrimShape::Ring, owner_id, position, scale, name)
            .await
    }

    pub async fn rez_prism(
        &self,
        owner_id: Uuid,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        self.rez_prim(PrimShape::Prism, owner_id, position, scale, name)
            .await
    }

    pub async fn set_object_position(&self, local_id: u32, position: [f32; 3]) -> Result<()> {
        {
            let mut objects = self.scene_objects.write();
            let obj = objects
                .get_mut(&local_id)
                .ok_or_else(|| anyhow!("Object {} not found", local_id))?;
            obj.position = position;
        }
        self.broadcast_object_update(local_id).await
    }

    pub async fn set_object_rotation(&self, local_id: u32, rotation: [f32; 4]) -> Result<()> {
        {
            let mut objects = self.scene_objects.write();
            let obj = objects
                .get_mut(&local_id)
                .ok_or_else(|| anyhow!("Object {} not found", local_id))?;
            obj.rotation = rotation;
        }
        self.broadcast_object_update(local_id).await
    }

    pub async fn set_object_scale(&self, local_id: u32, scale: [f32; 3]) -> Result<()> {
        {
            let mut objects = self.scene_objects.write();
            let obj = objects
                .get_mut(&local_id)
                .ok_or_else(|| anyhow!("Object {} not found", local_id))?;
            obj.scale = scale;
        }
        self.broadcast_object_update(local_id).await
    }

    pub async fn set_object_name(&self, local_id: u32, name: &str) -> Result<()> {
        let mut objects = self.scene_objects.write();
        let obj = objects
            .get_mut(&local_id)
            .ok_or_else(|| anyhow!("Object {} not found", local_id))?;
        obj.name = name.to_string();
        Ok(())
    }

    pub fn find_objects_near(
        &self,
        center: [f32; 3],
        radius: f32,
        name_filter: Option<&str>,
    ) -> Vec<(u32, String, [f32; 3], bool)> {
        let objects = self.scene_objects.read();
        let r2 = radius * radius;
        let mut results: Vec<_> = objects
            .values()
            .filter(|o| {
                let dx = o.position[0] - center[0];
                let dy = o.position[1] - center[1];
                let dz = o.position[2] - center[2];
                let dist2 = dx * dx + dy * dy + dz * dz;
                if dist2 > r2 {
                    return false;
                }
                if o.parent_id != 0 {
                    return false;
                }
                if let Some(filter) = name_filter {
                    o.name.to_lowercase().contains(&filter.to_lowercase())
                } else {
                    true
                }
            })
            .map(|o| {
                let is_linkset = !o.scene_group_id.is_nil()
                    && objects
                        .values()
                        .any(|c| c.scene_group_id == o.scene_group_id && c.local_id != o.local_id);
                (o.local_id, o.name.clone(), o.position, is_linkset)
            })
            .collect();
        results.sort_by(|a, b| {
            let da = (a.2[0] - center[0]).powi(2)
                + (a.2[1] - center[1]).powi(2)
                + (a.2[2] - center[2]).powi(2);
            let db = (b.2[0] - center[0]).powi(2)
                + (b.2[1] - center[1]).powi(2)
                + (b.2[2] - center[2]).powi(2);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    pub fn scan_linkset(&self, root_local_id: u32) -> Result<Vec<(i32, u32, String, [f32; 3])>> {
        let objects = self.scene_objects.read();
        let root = objects
            .get(&root_local_id)
            .ok_or_else(|| anyhow!("Object {} not found", root_local_id))?;
        let group_id = root.scene_group_id;
        let mut result = vec![(
            root.link_number,
            root.local_id,
            root.name.clone(),
            root.scale,
        )];
        if !group_id.is_nil() {
            let mut children: Vec<_> = objects
                .values()
                .filter(|o| o.scene_group_id == group_id && o.local_id != root_local_id)
                .map(|o| (o.link_number, o.local_id, o.name.clone(), o.scale))
                .collect();
            children.sort_by_key(|(ln, _, _, _)| *ln);
            result.extend(children);
        }
        Ok(result)
    }

    pub async fn set_object_color(&self, local_id: u32, color: [f32; 4]) -> Result<()> {
        {
            let mut objects = self.scene_objects.write();
            let obj = objects
                .get_mut(&local_id)
                .ok_or_else(|| anyhow!("Object {} not found", local_id))?;
            obj.texture_entry =
                crate::udp::messages::object_update::build_colored_prim_texture_entry(color);
        }
        self.broadcast_object_update(local_id).await
    }

    pub async fn set_object_texture(&self, local_id: u32, texture_uuid: Uuid) -> Result<()> {
        {
            let mut objects = self.scene_objects.write();
            let obj = objects
                .get_mut(&local_id)
                .ok_or_else(|| anyhow!("Object {} not found", local_id))?;
            obj.texture_entry =
                crate::udp::messages::object_update::build_textured_prim_texture_entry(
                    texture_uuid,
                );
        }
        self.broadcast_object_update(local_id).await
    }

    pub async fn link_objects(&self, root_id: u32, child_ids: &[u32]) -> Result<()> {
        let (group_id, db_updates) = {
            let objects = self.scene_objects.read();
            let root = objects
                .get(&root_id)
                .ok_or_else(|| anyhow!("Root object {} not found", root_id))?;
            let gid = root.uuid;
            let mut updates: Vec<(Uuid, Uuid, i32)> = Vec::with_capacity(1 + child_ids.len());
            updates.push((root.uuid, gid, 1));
            for (i, &child_id) in child_ids.iter().enumerate() {
                if let Some(child) = objects.get(&child_id) {
                    updates.push((child.uuid, gid, (i as i32) + 2));
                }
            }
            (gid, updates)
        };

        {
            let mut objects = self.scene_objects.write();
            let root_pos = objects
                .get(&root_id)
                .map(|r| r.position)
                .unwrap_or([0.0; 3]);
            if let Some(root) = objects.get_mut(&root_id) {
                root.scene_group_id = group_id;
                root.link_number = 1;
                root.parent_id = 0;
            }
            for (i, &child_id) in child_ids.iter().enumerate() {
                if let Some(child) = objects.get_mut(&child_id) {
                    child.scene_group_id = group_id;
                    child.parent_id = root_id;
                    child.link_number = (i as i32) + 2;
                    child.position = [
                        child.position[0] - root_pos[0],
                        child.position[1] - root_pos[1],
                        child.position[2] - root_pos[2],
                    ];
                }
            }
        }

        if let Some(db) = &self.db_connection {
            let db = db.clone();
            let updates = db_updates;
            tokio::spawn(async move {
                if let DatabaseConnection::PostgreSQL(pool) = db.as_ref() {
                    for (prim_uuid, scene_group_uuid, link_num) in &updates {
                        let _ = sqlx::query(
                            "UPDATE prims SET scenegroupid = $1, linknumber = $2 WHERE uuid = $3",
                        )
                        .bind(scene_group_uuid)
                        .bind(*link_num)
                        .bind(prim_uuid)
                        .execute(pool)
                        .await;
                    }
                    info!(
                        "[ACTION_BRIDGE] Persisted linkset {} ({} prims) to DB",
                        updates[0].1,
                        updates.len()
                    );
                }
            });
        }

        self.broadcast_object_update(root_id).await?;
        for &child_id in child_ids {
            self.broadcast_object_update(child_id).await?;
        }

        info!(
            "[ACTION_BRIDGE] Linked {} children to root {}",
            child_ids.len(),
            root_id
        );
        Ok(())
    }

    pub async fn add_to_linkset(&self, root_id: u32, new_ids: &[u32]) -> Result<Vec<(u32, i32)>> {
        let (group_id, max_link, db_updates, assignments) = {
            let objects = self.scene_objects.read();
            let root = objects
                .get(&root_id)
                .ok_or_else(|| anyhow!("Root object {} not found for add_to_linkset", root_id))?;
            let gid = root.scene_group_id;
            if gid.is_nil() {
                return Err(anyhow!(
                    "Root object {} has no linkset (scene_group_id is nil)",
                    root_id
                ));
            }
            let max_ln = objects
                .values()
                .filter(|o| o.scene_group_id == gid)
                .map(|o| o.link_number)
                .max()
                .unwrap_or(1);

            let mut updates: Vec<(Uuid, Uuid, i32, u32)> = Vec::new();
            let mut assigns: Vec<(u32, i32)> = Vec::new();
            for (i, &new_id) in new_ids.iter().enumerate() {
                let link_num = max_ln + 1 + i as i32;
                if let Some(child) = objects.get(&new_id) {
                    updates.push((child.uuid, gid, link_num, root_id));
                    assigns.push((new_id, link_num));
                } else {
                    return Err(anyhow!("New prim {} not found for add_to_linkset", new_id));
                }
            }
            (gid, max_ln, updates, assigns)
        };

        {
            let mut objects = self.scene_objects.write();
            for &(new_id, link_num) in &assignments {
                if let Some(child) = objects.get_mut(&new_id) {
                    child.scene_group_id = group_id;
                    child.parent_id = root_id;
                    child.link_number = link_num;
                }
            }
        }

        if let Some(db) = &self.db_connection {
            let db = db.clone();
            let updates = db_updates;
            tokio::spawn(async move {
                if let DatabaseConnection::PostgreSQL(pool) = db.as_ref() {
                    for (prim_uuid, scene_group_uuid, link_num, _parent) in &updates {
                        let _ = sqlx::query(
                            "UPDATE prims SET scenegroupid = $1, linknumber = $2 WHERE uuid = $3",
                        )
                        .bind(scene_group_uuid)
                        .bind(*link_num)
                        .bind(prim_uuid)
                        .execute(pool)
                        .await;
                    }
                    info!(
                        "[ACTION_BRIDGE] add_to_linkset: persisted {} new prims to linkset {}",
                        updates.len(),
                        updates[0].1
                    );
                }
            });
        }

        for &(new_id, _) in &assignments {
            self.broadcast_object_update(new_id).await?;
        }

        info!(
            "[ACTION_BRIDGE] add_to_linkset: added {} prims to root {} (links {}..{})",
            assignments.len(),
            root_id,
            max_link + 1,
            max_link + assignments.len() as i32
        );

        Ok(assignments)
    }

    pub async fn delete_object(&self, local_id: u32) -> Result<()> {
        {
            let mut objects = self.scene_objects.write();
            objects.remove(&local_id);
        }

        let clients: Vec<SocketAddr> = {
            let states = self.avatar_states.read();
            states
                .values()
                .filter(|s| !s.is_npc)
                .map(|s| s.client_addr)
                .collect()
        };

        let mut data = Vec::with_capacity(5);
        data.push(1u8);
        data.extend_from_slice(&local_id.to_le_bytes());

        for addr in clients {
            let _ = self
                .send_message(KILL_OBJECT_ID, &data, addr, true, false)
                .await;
        }

        info!("[ACTION_BRIDGE] Deleted object local_id={}", local_id);
        Ok(())
    }

    pub async fn rez_mesh(
        &self,
        owner_id: Uuid,
        geometry: crate::mesh::encoder::MeshGeometry,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        self.rez_mesh_with_te(owner_id, geometry, position, scale, name, None)
            .await
    }

    pub async fn rez_mesh_with_te(
        &self,
        owner_id: Uuid,
        geometry: crate::mesh::encoder::MeshGeometry,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
        texture_entry: Option<Vec<u8>>,
    ) -> Result<u32> {
        let mesh_data = crate::mesh::encoder::encode_mesh_asset(&geometry)?;
        let asset_id = Uuid::new_v4();

        if let Some(db) = &self.db_connection {
            if let Some(pool) = db.postgres_pool() {
                let agent_str = owner_id.to_string();
                let _ = sqlx::query(
                    "INSERT INTO assets (id, name, description, assettype, local, temporary, asset_flags, creatorid, data, create_time, access_time) \
                     VALUES ($1::uuid, $2, '', 49, 0, 0, 0, $3, $4, EXTRACT(EPOCH FROM NOW())::integer, EXTRACT(EPOCH FROM NOW())::integer) \
                     ON CONFLICT (id) DO NOTHING"
                )
                .bind(asset_id).bind(name).bind(&agent_str).bind(&mesh_data)
                .execute(pool).await;
                info!(
                    "[ACTION_BRIDGE] Mesh asset persisted: {} ({} bytes)",
                    asset_id,
                    mesh_data.len()
                );
            }
        }

        let local_id = self.next_prim_local_id.fetch_add(1, Ordering::SeqCst);
        let mut obj = SceneObject::new_box(local_id, owner_id, position);
        obj.name = name.to_string();
        obj.scale = scale;
        obj.extra_params = crate::mesh::encoder::build_mesh_extra_params(&asset_id);
        obj.physics_shape_type = 2;

        let num_faces = geometry
            .faces
            .len()
            .min(crate::mesh::encoder::MAX_MESH_FACES);
        match num_faces {
            1 => {
                obj.profile_curve = 48;
                obj.path_curve = 32;
                obj.path_scale_y = 150;
            }
            2 => {
                obj.profile_curve = 48;
                obj.path_curve = 32;
                obj.profile_hollow = 27500;
                obj.path_scale_y = 150;
            }
            3 => {
                obj.profile_curve = 48;
                obj.path_curve = 16;
            }
            4 => {
                obj.profile_curve = 48;
                obj.path_curve = 16;
                obj.profile_hollow = 27500;
            }
            5 => {
                obj.profile_curve = 51;
                obj.path_curve = 16;
            }
            6 => {
                obj.profile_curve = 49;
                obj.path_curve = 16;
            }
            7 => {
                obj.profile_curve = 49;
                obj.path_curve = 16;
                obj.profile_hollow = 27500;
            }
            _ => {
                obj.profile_curve = 49;
                obj.path_curve = 16;
                obj.profile_begin = 9375;
            }
        }
        if let Some(ref te) = texture_entry {
            obj.texture_entry = te.clone();
        }
        info!(
            "[REZ_MESH] faces={}, profile_curve={}, path_curve={}, te_len={}, asset={}",
            num_faces,
            obj.profile_curve,
            obj.path_curve,
            obj.texture_entry.len(),
            asset_id
        );

        {
            let mut objects = self.scene_objects.write();
            objects.insert(local_id, obj.clone());
        }

        self.broadcast_object_update(local_id).await?;

        if let Some(db) = &self.db_connection {
            let db = db.clone();
            let region_uuid = self.default_region_uuid;
            let obj_uuid = obj.uuid;
            let creator_str = owner_id.to_string();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i32;
            let owner_mask = 0x7FFFFFFFi32;
            let pcode = obj.pcode;
            let material = obj.material;
            let pc = obj.path_curve;
            let prc = obj.profile_curve;
            let pb = obj.path_begin;
            let pe = obj.path_end;
            let psx = obj.path_scale_x;
            let psy = obj.path_scale_y;
            let phx = obj.path_shear_x;
            let phy = obj.path_shear_y;
            let prof_begin = obj.profile_begin;
            let prof_hollow = obj.profile_hollow;
            let rot = obj.rotation;
            let te_persist = obj.texture_entry.clone();
            let extra_params = obj.extra_params.clone();
            let obj_name = name.to_string();
            tokio::spawn(async move {
                let Some(pool) = db.postgres_pool() else {
                    return;
                };
                let _ = sqlx::query(
                    r#"INSERT INTO prims (
                        uuid, regionuuid, creatorid, ownerid, groupid, lastownerid, scenegroupid,
                        name, text, description, sitname, touchname,
                        objectflags, ownermask, nextownermask, groupmask, everyonemask, basemask,
                        positionx, positiony, positionz,
                        grouppositionx, grouppositiony, grouppositionz,
                        velocityx, velocityy, velocityz,
                        angularvelocityx, angularvelocityy, angularvelocityz,
                        accelerationx, accelerationy, accelerationz,
                        rotationx, rotationy, rotationz, rotationw,
                        sittargetoffsetx, sittargetoffsety, sittargetoffsetz,
                        sittargetorientw, sittargetorientx, sittargetorienty, sittargetorientz,
                        creationdate, material, linknumber, passcollisions, physicsshapetype
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                        $13, $14, $15, $16, $17, $18,
                        $19, $20, $21, $22, $23, $24, $25, $26, $27,
                        $28, $29, $30, $31, $32, $33,
                        $34, $35, $36, $37,
                        $38, $39, $40, $41, $42, $43, $44,
                        $45, $46, $47, $48, $49
                    ) ON CONFLICT (uuid) DO UPDATE SET
                        positionx = $19, positiony = $20, positionz = $21,
                        rotationx = $34, rotationy = $35, rotationz = $36, rotationw = $37"#,
                )
                .bind(obj_uuid)
                .bind(region_uuid)
                .bind(&creator_str)
                .bind(owner_id)
                .bind(Uuid::nil())
                .bind(owner_id)
                .bind(obj_uuid)
                .bind(&obj_name)
                .bind("")
                .bind("")
                .bind("")
                .bind("")
                .bind(0i32)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(position[0])
                .bind(position[1])
                .bind(position[2])
                .bind(position[0])
                .bind(position[1])
                .bind(position[2])
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(rot[0])
                .bind(rot[1])
                .bind(rot[2])
                .bind(rot[3])
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(1.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(now)
                .bind(material as i32)
                .bind(1i32)
                .bind(0i32)
                .bind(2i32)
                .execute(pool)
                .await;

                let _ = sqlx::query(
                    r#"INSERT INTO primshapes (
                        uuid, shape, scalex, scaley, scalez, pcode,
                        pathbegin, pathend, pathscalex, pathscaley,
                        pathshearx, pathsheary, pathskew, pathcurve,
                        pathradiusoffset, pathrevolutions, pathtaperx, pathtapery,
                        pathtwist, pathtwistbegin,
                        profilebegin, profileend, profilecurve, profilehollow,
                        texture, extraparams, state, media
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6,
                        $7, $8, $9, $10, $11, $12, $13, $14,
                        $15, $16, $17, $18, $19, $20,
                        $21, $22, $23, $24, $25, $26, $27, $28
                    ) ON CONFLICT (uuid) DO UPDATE SET
                        texture = $25, extraparams = $26, scalex = $3, scaley = $4, scalez = $5"#,
                )
                .bind(obj_uuid)
                .bind(prc as i32)
                .bind(scale[0])
                .bind(scale[1])
                .bind(scale[2])
                .bind(pcode as i32)
                .bind(pb as i32)
                .bind(pe as i32)
                .bind(psx as i32)
                .bind(psy as i32)
                .bind(phx as i32)
                .bind(phy as i32)
                .bind(0i32)
                .bind(pc as i32)
                .bind(0i32)
                .bind(0i32)
                .bind(0i32)
                .bind(0i32)
                .bind(0i32)
                .bind(0i32)
                .bind(prof_begin as i32)
                .bind(0i32)
                .bind(prc as i32)
                .bind(prof_hollow as i32)
                .bind(&te_persist as &[u8])
                .bind(&extra_params as &[u8])
                .bind(0i32)
                .bind("")
                .execute(pool)
                .await;

                info!(
                    "[ACTION_BRIDGE] Persisted mesh prim {} '{}' to DB (pc={} prc={} pb={} ph={})",
                    obj_uuid, obj_name, pc, prc, prof_begin, prof_hollow
                );
            });
        }

        self.add_to_ai_inventory(obj.uuid, name, owner_id).await;

        info!(
            "[ACTION_BRIDGE] Rezzed mesh '{}' local_id={} asset={} at {:?}",
            name, local_id, asset_id, position
        );
        Ok(local_id)
    }

    pub async fn import_mesh(
        &self,
        owner_id: Uuid,
        file_path: &str,
        name: &str,
        position: [f32; 3],
    ) -> Result<u32> {
        let geometry = crate::mesh::glc_bridge::import_file(file_path)?;
        self.rez_mesh(owner_id, geometry, position, [1.0, 1.0, 1.0], name)
            .await
    }

    pub async fn rez_mesh_multi_lod(
        &self,
        owner_id: Uuid,
        multi_lod: crate::mesh::encoder::MultiLodGeometry,
        position: [f32; 3],
        scale: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        let num_faces = multi_lod.high.faces.len();
        let mesh_data = crate::mesh::encoder::encode_mesh_asset_multi_lod(&multi_lod)?;
        let asset_id = Uuid::new_v4();

        if let Some(db) = &self.db_connection {
            if let Some(pool) = db.postgres_pool() {
                let agent_str = owner_id.to_string();
                let _ = sqlx::query(
                    "INSERT INTO assets (id, name, description, assettype, local, temporary, asset_flags, creatorid, data, create_time, access_time) \
                     VALUES ($1::uuid, $2, '', 49, 0, 0, 0, $3, $4, EXTRACT(EPOCH FROM NOW())::integer, EXTRACT(EPOCH FROM NOW())::integer) \
                     ON CONFLICT (id) DO NOTHING"
                )
                .bind(asset_id).bind(name).bind(&agent_str).bind(&mesh_data)
                .execute(pool).await;
                info!(
                    "[ACTION_BRIDGE] Multi-LOD mesh asset persisted: {} ({} bytes)",
                    asset_id,
                    mesh_data.len()
                );
            }
        }

        let local_id = self.next_prim_local_id.fetch_add(1, Ordering::SeqCst);
        let mut obj = SceneObject::new_box(local_id, owner_id, position);
        obj.name = name.to_string();
        obj.scale = scale;
        obj.extra_params = crate::mesh::encoder::build_mesh_extra_params(&asset_id);
        obj.physics_shape_type = 2;

        match num_faces {
            1 => {
                obj.profile_curve = 48;
                obj.path_curve = 32;
                obj.path_scale_y = 150;
            }
            2 => {
                obj.profile_curve = 48;
                obj.path_curve = 32;
                obj.profile_hollow = 27500;
                obj.path_scale_y = 150;
            }
            3 => {
                obj.profile_curve = 48;
                obj.path_curve = 16;
            }
            4 => {
                obj.profile_curve = 48;
                obj.path_curve = 16;
                obj.profile_hollow = 27500;
            }
            5 => {
                obj.profile_curve = 51;
                obj.path_curve = 16;
            }
            6 => {
                obj.profile_curve = 49;
                obj.path_curve = 16;
            }
            7 => {
                obj.profile_curve = 49;
                obj.path_curve = 16;
                obj.profile_hollow = 27500;
            }
            _ => {
                obj.profile_curve = 49;
                obj.path_curve = 16;
                obj.profile_begin = 9375;
            }
        }
        info!(
            "[REZ_MESH_MULTI_LOD] faces={}, profile_curve={}, path_curve={}, asset={}",
            num_faces, obj.profile_curve, obj.path_curve, asset_id
        );

        {
            let mut objects = self.scene_objects.write();
            objects.insert(local_id, obj.clone());
        }

        self.broadcast_object_update(local_id).await?;

        if let Some(db) = &self.db_connection {
            let db = db.clone();
            let region_uuid = self.default_region_uuid;
            let obj_uuid = obj.uuid;
            let creator_str = owner_id.to_string();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i32;
            let owner_mask = 0x7FFFFFFFi32;
            let pcode = obj.pcode;
            let material = obj.material;
            let pc = obj.path_curve;
            let prc = obj.profile_curve;
            let pb = obj.path_begin;
            let pe = obj.path_end;
            let psx = obj.path_scale_x;
            let psy = obj.path_scale_y;
            let phx = obj.path_shear_x;
            let phy = obj.path_shear_y;
            let prof_begin = obj.profile_begin;
            let prof_hollow = obj.profile_hollow;
            let rot = obj.rotation;
            let te_persist = obj.texture_entry.clone();
            let extra_params = obj.extra_params.clone();
            let obj_name = name.to_string();
            tokio::spawn(async move {
                let Some(pool) = db.postgres_pool() else {
                    return;
                };
                let _ = sqlx::query(
                    r#"INSERT INTO prims (
                        uuid, regionuuid, creatorid, ownerid, groupid, lastownerid, scenegroupid,
                        name, text, description, sitname, touchname,
                        objectflags, ownermask, nextownermask, groupmask, everyonemask, basemask,
                        positionx, positiony, positionz,
                        grouppositionx, grouppositiony, grouppositionz,
                        velocityx, velocityy, velocityz,
                        angularvelocityx, angularvelocityy, angularvelocityz,
                        accelerationx, accelerationy, accelerationz,
                        rotationx, rotationy, rotationz, rotationw,
                        sittargetoffsetx, sittargetoffsety, sittargetoffsetz,
                        sittargetorientw, sittargetorientx, sittargetorienty, sittargetorientz,
                        creationdate, material, linknumber, passcollisions, physicsshapetype
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                        $13, $14, $15, $16, $17, $18,
                        $19, $20, $21, $22, $23, $24, $25, $26, $27,
                        $28, $29, $30, $31, $32, $33,
                        $34, $35, $36, $37,
                        $38, $39, $40, $41, $42, $43, $44,
                        $45, $46, $47, $48, $49
                    ) ON CONFLICT (uuid) DO UPDATE SET
                        positionx = $19, positiony = $20, positionz = $21,
                        rotationx = $34, rotationy = $35, rotationz = $36, rotationw = $37"#,
                )
                .bind(obj_uuid)
                .bind(region_uuid)
                .bind(&creator_str)
                .bind(owner_id)
                .bind(Uuid::nil())
                .bind(owner_id)
                .bind(obj_uuid)
                .bind(&obj_name)
                .bind("")
                .bind("")
                .bind("")
                .bind("")
                .bind(0i32)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(owner_mask)
                .bind(position[0])
                .bind(position[1])
                .bind(position[2])
                .bind(position[0])
                .bind(position[1])
                .bind(position[2])
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(rot[0])
                .bind(rot[1])
                .bind(rot[2])
                .bind(rot[3])
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(1.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(0.0f32)
                .bind(now)
                .bind(material as i32)
                .bind(1i32)
                .bind(0i32)
                .bind(2i32)
                .execute(pool)
                .await;

                let _ = sqlx::query(
                    r#"INSERT INTO primshapes (
                        uuid, shape, scalex, scaley, scalez, pcode,
                        pathbegin, pathend, pathscalex, pathscaley,
                        pathshearx, pathsheary, pathskew, pathcurve,
                        pathradiusoffset, pathrevolutions, pathtaperx, pathtapery,
                        pathtwist, pathtwistbegin,
                        profilebegin, profileend, profilecurve, profilehollow,
                        texture, extraparams, state, media
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6,
                        $7, $8, $9, $10, $11, $12, $13, $14,
                        $15, $16, $17, $18, $19, $20,
                        $21, $22, $23, $24, $25, $26, $27, $28
                    ) ON CONFLICT (uuid) DO UPDATE SET
                        texture = $25, extraparams = $26, scalex = $3, scaley = $4, scalez = $5"#,
                )
                .bind(obj_uuid)
                .bind(prc as i32)
                .bind(scale[0])
                .bind(scale[1])
                .bind(scale[2])
                .bind(pcode as i32)
                .bind(pb as i32)
                .bind(pe as i32)
                .bind(psx as i32)
                .bind(psy as i32)
                .bind(phx as i32)
                .bind(phy as i32)
                .bind(0i32)
                .bind(pc as i32)
                .bind(0i32)
                .bind(0i32)
                .bind(0i32)
                .bind(0i32)
                .bind(0i32)
                .bind(0i32)
                .bind(prof_begin as i32)
                .bind(0i32)
                .bind(prc as i32)
                .bind(prof_hollow as i32)
                .bind(&te_persist as &[u8])
                .bind(&extra_params as &[u8])
                .bind(0i32)
                .bind("")
                .execute(pool)
                .await;

                info!("[ACTION_BRIDGE] Persisted multi-LOD mesh prim {} '{}' to DB (pc={} prc={} pb={} ph={})", obj_uuid, obj_name, pc, prc, prof_begin, prof_hollow);
            });
        }

        self.add_to_ai_inventory(obj.uuid, name, owner_id).await;

        info!(
            "[ACTION_BRIDGE] Rezzed multi-LOD mesh '{}' local_id={} asset={} at {:?}",
            name, local_id, asset_id, position
        );
        Ok(local_id)
    }

    pub async fn blender_generate(
        &self,
        owner_id: Uuid,
        template: &str,
        params: &std::collections::HashMap<String, String>,
        name: &str,
        position: [f32; 3],
    ) -> Result<u32> {
        use crate::mesh::blender_worker::{is_clothing_template, BlenderWorker, ExportFormat};

        let worker = BlenderWorker::new()?;
        let script = BlenderWorker::get_template(template, params)?;

        if is_clothing_template(template) {
            let dae_path = worker
                .generate_with_format(&script, ExportFormat::Dae)
                .await?;
            let geometry =
                crate::mesh::glc_bridge::import_dae_with_skin(&dae_path.to_string_lossy())?;
            info!(
                "[ACTION_BRIDGE] Clothing '{}': {} faces, skin={}",
                name,
                geometry.faces.len(),
                geometry.skin_info.is_some()
            );
            self.rez_mesh(owner_id, geometry, position, [1.0, 1.0, 1.0], name)
                .await
        } else {
            let obj_path = worker.generate(&script).await?;
            let _ = std::fs::copy(&obj_path, "/tmp/granite_working.obj");
            info!("[ACTION_BRIDGE] Saved granite OBJ to /tmp/granite_working.obj");
            self.import_mesh(owner_id, &obj_path.to_string_lossy(), name, position)
                .await
        }
    }

    pub async fn blender_generate_snapshot(
        &self,
        owner_id: Uuid,
        anim_name: &str,
        frame: i32,
        position: [f32; 3],
        name: &str,
    ) -> Result<u32> {
        use crate::mesh::blender_worker::{ruth2_base_dir, BlenderWorker, ExportFormat};
        use crate::mesh::dae_writer::{write_dae, DaeWriterOptions, UpAxis};
        use crate::mesh::snapshot_collector::SnapshotCollector;
        use crate::region::avatar::appearance::Appearance;

        let db = self
            .db_connection
            .as_ref()
            .ok_or_else(|| anyhow!("No database connection for snapshot statue"))?;
        let pool = db
            .postgres_pool()
            .ok_or_else(|| anyhow!("Snapshot statue requires PostgreSQL"))?;

        let appearance = self.load_appearance_for_snapshot(&owner_id, pool).await?;

        let mut live_texture_entries: std::collections::HashMap<Uuid, Vec<u8>> =
            std::collections::HashMap::new();
        {
            let objects = self.scene_objects.read();
            for obj in objects.values() {
                if obj.owner_id == owner_id
                    && obj.attachment_point > 0
                    && !obj.texture_entry.is_empty()
                {
                    live_texture_entries.insert(obj.uuid, obj.texture_entry.clone());
                }
            }
        }

        let tmp_dir = std::env::temp_dir().join(format!("snapshot_statue_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&tmp_dir)?;

        let mut collector = SnapshotCollector::new(&tmp_dir)?;
        let avatar_name = Self::get_avatar_name_pg(&owner_id, pool)
            .await
            .unwrap_or_else(|| "Avatar".to_string());

        let snapshot = collector
            .collect_snapshot_with_live_te(
                &owner_id,
                &avatar_name,
                &appearance,
                pool,
                None,
                &live_texture_entries,
            )
            .await?;

        let mut pieces_json = Vec::new();
        let mut piece_texture_map: std::collections::HashMap<String, Vec<Option<Uuid>>> =
            std::collections::HashMap::new();
        for piece in &snapshot.pieces {
            let dae_path = tmp_dir.join(format!("{}.dae", piece.name));
            let tex_files: Vec<Option<String>> = piece
                .texture_paths
                .iter()
                .map(|tp| tp.as_ref().map(|p| p.to_string_lossy().to_string()))
                .collect();
            let opts = DaeWriterOptions {
                mesh_name: piece.name.clone(),
                texture_files: tex_files.clone(),
                up_axis: UpAxis::ZUp,
            };
            let dae_xml = write_dae(&piece.geometry, &opts)?;
            std::fs::write(&dae_path, &dae_xml)?;
            info!(
                "[SNAPSHOT] Wrote piece DAE: {} ({} faces, {} tex_uuids)",
                piece.name,
                piece.geometry.faces.len(),
                piece.texture_uuids.len()
            );

            piece_texture_map.insert(piece.name.clone(), piece.texture_uuids.clone());

            let textures: Vec<serde_json::Value> = tex_files
                .iter()
                .map(|t| match t {
                    Some(p) => serde_json::Value::String(p.clone()),
                    None => serde_json::Value::Null,
                })
                .collect();

            pieces_json.push(serde_json::json!({
                "name": piece.name,
                "dae_path": dae_path.to_string_lossy(),
                "textures": textures,
            }));
        }

        let baked = &snapshot.baked_texture_paths;
        let baked_json = serde_json::json!({
            "head": baked.head.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            "upper": baked.upper.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            "lower": baked.lower.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            "eyes": baked.eyes.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
        });

        let baked_head_uuid = if baked.head.is_some() {
            baked.head_uuid
        } else {
            None
        };
        let baked_upper_uuid = if baked.upper.is_some() {
            baked.upper_uuid
        } else {
            None
        };
        let baked_lower_uuid = if baked.lower.is_some() {
            baked.lower_uuid
        } else {
            None
        };
        let baked_eyes_uuid = if baked.eyes.is_some() {
            baked.eyes_uuid
        } else {
            None
        };
        info!("[SNAPSHOT] Baked texture UUIDs (asset-verified): head={:?} upper={:?} lower={:?} eyes={:?}",
            baked_head_uuid, baked_upper_uuid, baked_lower_uuid, baked_eyes_uuid);

        let body_blend = ruth2_base_dir()
            .map(|d| format!("{}/Ruth2v4Dev_PartialLindenSkeleton.blend", d))
            .unwrap_or_default();

        let pose_json_path = Self::resolve_pose_json_pg(anim_name, &tmp_dir, pool)
            .await
            .unwrap_or_default();

        let manifest = serde_json::json!({
            "avatar_name": avatar_name,
            "pieces": pieces_json,
            "baked_textures": baked_json,
            "body_blend_path": body_blend,
            "pose_json_path": pose_json_path,
            "frame": frame,
        });

        let manifest_path = tmp_dir.join("manifest.json");
        std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
        info!(
            "[SNAPSHOT] Wrote manifest: {} ({} pieces)",
            manifest_path.display(),
            pieces_json.len()
        );

        let mut params = std::collections::HashMap::new();
        params.insert(
            "MANIFEST_PATH".to_string(),
            manifest_path.to_string_lossy().to_string(),
        );

        let worker = BlenderWorker::new()?;
        let script = BlenderWorker::get_template("snapshot_statue", &params)?;
        worker.run_script(&script).await?;

        let parts_manifest_path = tmp_dir.join("parts_manifest.json");
        if !parts_manifest_path.exists() {
            bail!("Blender did not produce parts_manifest.json");
        }
        let parts_data: Vec<serde_json::Value> =
            serde_json::from_str(&std::fs::read_to_string(&parts_manifest_path)?)?;
        info!("[SNAPSHOT] Blender produced {} parts", parts_data.len());

        let mut rezzed_ids = Vec::new();

        let body_tex_map: std::collections::HashMap<&str, Option<Uuid>> = [
            ("Ruth2v4Body", baked_upper_uuid),
            ("Ruth2v4Head", baked_head_uuid),
            ("Ruth2v4Hands", baked_upper_uuid),
            ("Ruth2v4FeetFlat", baked_lower_uuid),
            ("Ruth2v4EyeBall_L", baked_eyes_uuid),
            ("Ruth2v4Eyeball_R", baked_eyes_uuid),
            ("Ruth2v4Eyelashes", baked_head_uuid),
        ]
        .iter()
        .cloned()
        .collect();

        use crate::udp::messages::object_update::TextureEntryData;

        for (idx, part) in parts_data.iter().enumerate() {
            let part_path = part["path"].as_str().unwrap_or("");
            let part_name = part["name"].as_str().unwrap_or("part");
            if part_path.is_empty() || !std::path::Path::new(part_path).exists() {
                warn!("[SNAPSHOT] Part {} missing DAE: {}", idx, part_path);
                continue;
            }

            let offset = part["offset"]
                .as_array()
                .map(|a| {
                    [
                        a.get(0).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                        a.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                        a.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    ]
                })
                .unwrap_or([0.0; 3]);
            let part_pos = [
                position[0] + offset[0],
                position[1] + offset[1],
                position[2] + offset[2],
            ];
            let scale = [1.0f32, 1.0, 1.0];

            let mut ted = TextureEntryData::new();
            let mut has_texture = false;
            if let Some(bake_uuid) = body_tex_map.get(part_name).and_then(|u| *u) {
                ted.set_texture(-1, bake_uuid);
                has_texture = true;
                info!(
                    "[SNAPSHOT] Body part {} → baked texture {}",
                    part_name, bake_uuid
                );
            } else if let Some(face_uuids) = piece_texture_map.get(part_name) {
                let first_valid = face_uuids.iter().find_map(|u| *u);
                if let Some(default_uuid) = first_valid {
                    ted.default_face.texture_id = default_uuid;
                    has_texture = true;
                }
                for (fi, tex_uuid) in face_uuids.iter().enumerate() {
                    if let Some(uuid) = tex_uuid {
                        ted.set_texture(fi as i32, *uuid);
                        has_texture = true;
                    }
                }
                if has_texture {
                    info!(
                        "[SNAPSHOT] Attachment {} → {} face textures",
                        part_name,
                        face_uuids.iter().filter(|u| u.is_some()).count()
                    );
                }
            }
            let te_bytes = if has_texture {
                Some(ted.to_bytes())
            } else {
                None
            };

            match crate::mesh::glc_bridge::import_file(part_path) {
                Ok(mut geometry) => {
                    let orig_faces = geometry.faces.len();
                    geometry.faces.retain(|face| {
                        if face.positions.is_empty() { return false; }
                        let mut min = [f32::MAX; 3];
                        let mut max = [f32::MIN; 3];
                        for p in &face.positions {
                            for i in 0..3 {
                                if p[i] < min[i] { min[i] = p[i]; }
                                if p[i] > max[i] { max[i] = p[i]; }
                            }
                        }
                        let range = (max[0]-min[0]).max(max[1]-min[1]).max(max[2]-min[2]);
                        if range > 10.0 {
                            warn!("[SNAPSHOT] Filtering oversized face ({} verts, range={:.1}m) - likely physics hull", face.positions.len(), range);
                            false
                        } else {
                            true
                        }
                    });
                    if geometry.faces.is_empty() {
                        warn!("[SNAPSHOT] Part {} has no valid faces after filtering", idx);
                        continue;
                    }
                    if geometry.faces.len() < orig_faces {
                        info!(
                            "[SNAPSHOT] Filtered {}/{} faces (removed physics hulls)",
                            geometry.faces.len(),
                            orig_faces
                        );
                    }
                    let piece_name = if idx == 0 {
                        name.to_string()
                    } else {
                        format!("{}_{}", name, idx)
                    };
                    let verts: usize = geometry.faces.iter().map(|f| f.positions.len()).sum();
                    info!("[SNAPSHOT] Part {}: {} faces, {} verts ({}) pos=[{:.1},{:.1},{:.1}] offset=[{:.3},{:.3},{:.3}] te={}",
                        idx, geometry.faces.len(), verts, part_name,
                        part_pos[0], part_pos[1], part_pos[2],
                        offset[0], offset[1], offset[2],
                        te_bytes.as_ref().map(|t| t.len()).unwrap_or(0));
                    match self
                        .rez_mesh_with_te(
                            owner_id,
                            geometry,
                            part_pos,
                            scale,
                            &piece_name,
                            te_bytes.clone(),
                        )
                        .await
                    {
                        Ok(local_id) => {
                            rezzed_ids.push(local_id);
                        }
                        Err(e) => {
                            warn!("[SNAPSHOT] Failed to rez part {}: {}", idx, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("[SNAPSHOT] Failed to import part {} DAE: {}", idx, e);
                }
            }
        }

        info!(
            "[SNAPSHOT] Rezzed {}/{} parts",
            rezzed_ids.len(),
            parts_data.len()
        );

        if rezzed_ids.is_empty() {
            bail!("No parts were successfully rezzed");
        }

        if rezzed_ids.len() > 1 {
            let root_id = rezzed_ids[0];
            let child_ids = &rezzed_ids[1..];
            if let Err(e) = self.link_objects(root_id, child_ids).await {
                warn!("[SNAPSHOT] Failed to link parts to root {}: {}", root_id, e);
            } else {
                info!(
                    "[SNAPSHOT] Linked {} parts into linkset (root={})",
                    rezzed_ids.len(),
                    root_id
                );
            }
        }

        let local_id = rezzed_ids[0];

        info!(
            "[SNAPSHOT] Keeping temp dir for debugging: {}",
            tmp_dir.display()
        );
        // if let Err(e) = std::fs::remove_dir_all(&tmp_dir) {
        //     warn!("[SNAPSHOT] Failed to cleanup temp dir {}: {}", tmp_dir.display(), e);
        // }

        Ok(local_id)
    }

    async fn load_appearance_for_snapshot(
        &self,
        agent_id: &Uuid,
        pool: &sqlx::PgPool,
    ) -> Result<crate::region::avatar::appearance::Appearance> {
        use crate::region::avatar::appearance::{Appearance, Attachment, TextureEntry};

        let mut appearance = Appearance::default();

        let tex_rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT name, value FROM avatars WHERE principalid = $1::uuid AND name LIKE 'Texture %'"
        )
        .bind(agent_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (name, tex_str) in &tex_rows {
            let face_str = name.strip_prefix("Texture ").unwrap_or("0");
            let face: u32 = face_str.trim().parse().unwrap_or(0);
            if let Ok(tex_id) = tex_str.parse::<Uuid>() {
                appearance.textures.push(TextureEntry {
                    face,
                    texture_id: tex_id,
                });
            }
        }

        let ap_rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT name, value FROM avatars WHERE principalid = $1::uuid AND name LIKE '_ap_%'",
        )
        .bind(agent_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (name, value) in &ap_rows {
            let point_str = name.strip_prefix("_ap_").unwrap_or("0");
            let point: u32 = point_str.parse().unwrap_or(0);
            let parts: Vec<&str> = value.split(',').collect();
            if parts.len() >= 2 {
                if let (Ok(item_id), Ok(asset_id)) =
                    (parts[0].parse::<Uuid>(), parts[1].parse::<Uuid>())
                {
                    appearance.attachments.push(Attachment {
                        item_id,
                        asset_id,
                        point,
                    });
                }
            }
        }

        let db_attach_count = appearance.attachments.len();

        {
            let objects = self.scene_objects.read();
            for obj in objects.values() {
                if obj.owner_id == *agent_id && obj.attachment_point > 0 {
                    if let Some(mesh_uuid) = crate::mesh::extract_mesh_asset_uuid(&obj.extra_params)
                    {
                        let point = obj.attachment_point as u32;
                        let already_has = appearance
                            .attachments
                            .iter()
                            .any(|a| a.asset_id == mesh_uuid && a.point == point);
                        if !already_has {
                            info!(
                                "[SNAPSHOT] Found live mesh attachment: '{}' point={} mesh={}",
                                obj.name, point, mesh_uuid
                            );
                            appearance.attachments.push(Attachment {
                                item_id: obj.uuid,
                                asset_id: mesh_uuid,
                                point,
                            });
                        }
                    }
                }
            }
        }

        info!("[SNAPSHOT] Loaded appearance for {}: {} textures, {} attachments ({} from DB, {} from live scene)",
            agent_id, appearance.textures.len(), appearance.attachments.len(),
            db_attach_count, appearance.attachments.len() - db_attach_count);

        Ok(appearance)
    }

    async fn get_avatar_name_pg(agent_id: &Uuid, pool: &sqlx::PgPool) -> Option<String> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT firstname, lastname FROM useraccount WHERE principalid = $1::uuid",
        )
        .bind(agent_id)
        .fetch_optional(pool)
        .await
        .ok()?;

        row.map(|(first, last)| format!("{} {}", first, last))
    }

    async fn resolve_pose_json_pg(
        anim_name: &str,
        tmp_dir: &std::path::Path,
        pool: &sqlx::PgPool,
    ) -> Option<String> {
        let instance_dir =
            std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
        let candidates = vec![
            format!(
                "{}/../../assets/poses/{}.pose.json",
                instance_dir, anim_name
            ),
            format!(
                "{}/../../opensim-next/assets/poses/{}.pose.json",
                instance_dir, anim_name
            ),
            format!("opensim-next/assets/poses/{}.pose.json", anim_name),
            format!("assets/poses/{}.pose.json", anim_name),
        ];
        for c in &candidates {
            if std::path::Path::new(c).exists() {
                return Some(c.clone());
            }
        }

        let row: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT data FROM assets WHERE name = $1 AND assettype = 20")
                .bind(anim_name)
                .fetch_optional(pool)
                .await
                .ok()?;

        if let Some((bvh_data,)) = row {
            let bvh_path = tmp_dir.join(format!("{}.bvh", anim_name));
            std::fs::write(&bvh_path, &bvh_data).ok()?;
            return Some(bvh_path.to_string_lossy().to_string());
        }

        None
    }

    pub async fn create_tshirt(
        &self,
        owner_id: Uuid,
        target_agent_id: Uuid,
        logo_path: &str,
        shirt_color: [u8; 4],
        front_offset: f32,
        back_offset: Option<f32>,
        sleeve: f32,
        fit: &str,
        collar: &str,
        name: &str,
    ) -> Result<String> {
        use crate::asset::jpeg2000::J2KCodec;
        use crate::mesh::blender_worker::{BlenderWorker, ExportFormat};
        use crate::mesh::texture_compositor::{
            compose_tshirt_texture, BodyRegion, LogoPlacement, PlacementSide, TShirtTextureConfig,
        };

        let worker = BlenderWorker::new()?;
        let temp_dir = std::env::temp_dir().join(format!("opensim_tshirt_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        let config = TShirtTextureConfig {
            base_color: shirt_color,
            logo_path: logo_path.to_string(),
            front_placement: Some(LogoPlacement {
                side: PlacementSide::Front,
                offset_from_collar_inches: front_offset,
                centered: true,
                body_region: BodyRegion::Upper,
            }),
            back_placement: back_offset.map(|off| LogoPlacement {
                side: PlacementSide::Back,
                offset_from_collar_inches: off,
                centered: true,
                body_region: BodyRegion::Upper,
            }),
            texture_size: 1024,
        };
        let composed = compose_tshirt_texture(&config)?;

        let texture_png_path = temp_dir.join("shirt_texture.png");
        composed.save(&texture_png_path)?;
        info!(
            "[TSHIRT] Composed texture saved: {}",
            texture_png_path.display()
        );

        let codec = J2KCodec::new();
        let j2k_data = codec.encode_image_to_j2k(&composed)?;
        let texture_asset_id = Uuid::new_v4();

        let pool = self
            .db_connection
            .as_ref()
            .and_then(|db| db.postgres_pool())
            .ok_or_else(|| anyhow!("No PostgreSQL connection"))?;

        let now = chrono::Utc::now().timestamp() as i32;
        let agent_str = owner_id.to_string();
        sqlx::query(
            "INSERT INTO assets (id, name, description, assettype, local, temporary, asset_flags, creatorid, data, create_time, access_time) \
             VALUES ($1::uuid, $2, '', 0, 0, 0, 0, $3, $4, $5, $6) ON CONFLICT (id) DO NOTHING"
        )
        .bind(texture_asset_id)
        .bind(format!("{} texture", name))
        .bind(&agent_str)
        .bind(&j2k_data)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
        info!(
            "[TSHIRT] Texture asset {} uploaded ({} bytes J2K)",
            texture_asset_id,
            j2k_data.len()
        );

        let mut params = HashMap::new();
        params.insert("SLEEVE_LENGTH".to_string(), sleeve.to_string());
        params.insert("FIT".to_string(), fit.to_string());
        params.insert("COLLAR".to_string(), collar.to_string());
        params.insert(
            "TEXTURE_PATH".to_string(),
            texture_png_path.to_string_lossy().to_string(),
        );

        let script = BlenderWorker::get_template("shirt", &params)?;
        let dae_path = worker
            .generate_with_format(&script, ExportFormat::Dae)
            .await?;
        let geometry = crate::mesh::glc_bridge::import_dae_with_skin(&dae_path.to_string_lossy())?;
        info!(
            "[TSHIRT] Mesh generated: {} faces, skin={}",
            geometry.faces.len(),
            geometry.skin_info.is_some()
        );

        let npc_pos = self
            .session_manager
            .get_avatar_position(owner_id)
            .unwrap_or([128.0, 128.0, 25.0]);
        let rez_pos = [npc_pos[0] + 1.0, npc_pos[1], npc_pos[2]];

        let local_id = self
            .rez_mesh(owner_id, geometry, rez_pos, [1.0, 1.0, 1.0], name)
            .await?;

        {
            let mut objects = self.scene_objects.write();
            if let Some(obj) = objects.get_mut(&local_id) {
                let te = crate::udp::messages::object_update::build_textured_prim_texture_entry(
                    texture_asset_id,
                );
                obj.texture_entry = te;
            }
        }
        self.broadcast_object_update(local_id).await?;

        let _result = self.give_object(local_id, target_agent_id).await?;

        if let Err(e) = self.delete_object(local_id).await {
            info!("[TSHIRT] Failed to clean up in-world object: {}", e);
        }

        self.send_body_alpha_for_garment("shirt");

        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(format!("T-shirt '{}' created and delivered", name))
    }

    pub fn send_body_alpha_for_garment(&self, garment_type: &str) {
        const BODY_ALPHA_CHANNEL: i32 = -12340;
        let zones: &[&str] = match garment_type {
            "shirt" | "jacket" => &[
                "alpha|chest_upper_L|0",
                "alpha|chest_upper_R|0",
                "alpha|nipple_L|0",
                "alpha|nipple_R|0",
                "alpha|belly|0",
                "alpha|upper_back|0",
                "alpha|lower_back|0",
                "alpha|shoulder_L|0",
                "alpha|shoulder_R|0",
                "alpha|upper_arm_L|0",
                "alpha|upper_arm_R|0",
            ],
            "pants" => &[
                "alpha|crotch|0",
                "alpha|buttocks|0",
                "alpha|upper_leg_L|0",
                "alpha|upper_leg_R|0",
                "alpha|lower_leg_L|0",
                "alpha|lower_leg_R|0",
            ],
            "dress" => &[
                "alpha|chest_upper_L|0",
                "alpha|chest_upper_R|0",
                "alpha|nipple_L|0",
                "alpha|nipple_R|0",
                "alpha|belly|0",
                "alpha|upper_back|0",
                "alpha|lower_back|0",
                "alpha|shoulder_L|0",
                "alpha|shoulder_R|0",
                "alpha|crotch|0",
                "alpha|buttocks|0",
                "alpha|upper_leg_L|0",
                "alpha|upper_leg_R|0",
            ],
            "skirt" => &[
                "alpha|crotch|0",
                "alpha|buttocks|0",
                "alpha|upper_leg_L|0",
                "alpha|upper_leg_R|0",
            ],
            _ => &[],
        };

        if let Some(ref ye) = self.yengine {
            let engine = ye.read();
            for cmd in zones {
                engine.deliver_chat(BODY_ALPHA_CHANNEL, "GarmentAlpha", Uuid::nil(), cmd);
            }
            if !zones.is_empty() {
                info!(
                    "[ALPHA] Sent {} alpha hide commands for garment type '{}'",
                    zones.len(),
                    garment_type
                );
            }
        }
    }

    pub async fn export_oar(
        &self,
        region_id: Uuid,
        filename: &str,
        object_uuids: Option<Vec<Uuid>>,
    ) -> Result<String> {
        use crate::archives::oar::writer::{OarSaveOptions, OarWriter};

        let safe_filename = if filename.starts_with('/') {
            filename.to_string()
        } else {
            let instance_dir =
                std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
            let oar_dir = format!("{}/OAR", instance_dir);
            let _ = std::fs::create_dir_all(&oar_dir);
            format!("{}/{}", oar_dir, filename)
        };

        let pool = self
            .db_connection
            .as_ref()
            .and_then(|db| db.postgres_pool())
            .ok_or_else(|| anyhow!("No PostgreSQL connection for OAR export"))?;

        let writer = OarWriter::new(pool.clone());
        let has_filter = object_uuids.is_some();
        let options = OarSaveOptions {
            region_id,
            include_assets: true,
            include_terrain: !has_filter,
            include_objects: true,
            include_parcels: !has_filter,
            object_uuids,
        };
        let result = writer.save(&safe_filename, options).await?;
        if !result.warnings.is_empty() {
            for w in &result.warnings {
                warn!("[ACTION_BRIDGE] OAR export warning: {}", w);
            }
        }
        let msg = format!(
            "Saved {} objects, {} assets to {} ({} bytes)",
            result.stats.objects_saved,
            result.stats.assets_saved,
            safe_filename,
            result.stats.archive_size_bytes
        );
        info!("[ACTION_BRIDGE] OAR export: {}", msg);
        Ok(msg)
    }

    pub async fn insert_script(
        &self,
        owner_id: Uuid,
        local_id: u32,
        script_name: &str,
        script_source: &str,
    ) -> Result<()> {
        let object_uuid = {
            let objects = self.scene_objects.read();
            objects.get(&local_id).map(|o| o.uuid)
        };
        let object_id = object_uuid
            .ok_or_else(|| anyhow!("Object {} not found for script insertion", local_id))?;

        let asset_id = Uuid::new_v4();
        let new_item_id = Uuid::new_v4();

        if let Some(db) = &self.db_connection {
            if let Some(pool) = db.postgres_pool() {
                let script_bytes = script_source.as_bytes().to_vec();
                let agent_str = owner_id.to_string();
                let _ = sqlx::query(
                    "INSERT INTO assets (id, name, description, assettype, local, temporary, asset_flags, creatorid, data, create_time, access_time) \
                     VALUES ($1::uuid, $2, '', 10, 0, 0, 0, $3, $4, EXTRACT(EPOCH FROM NOW())::integer, EXTRACT(EPOCH FROM NOW())::integer) \
                     ON CONFLICT (id) DO NOTHING"
                )
                .bind(asset_id).bind(script_name).bind(&agent_str).bind(&script_bytes)
                .execute(pool).await;

                let _ = sqlx::query(
                    "INSERT INTO primitems (itemid, primid, assetid, parentfolderid, invtype, assettype, name, description, \
                     creationdate, creatorid, ownerid, lastownerid, groupid, \
                     basepermissions, currentpermissions, grouppermissions, everyonepermissions, nextpermissions, flags) \
                     VALUES ($1, $2, $3, $4, 10, 10, $5, '', \
                     EXTRACT(EPOCH FROM NOW())::integer, $6, $6, $6, '00000000-0000-0000-0000-000000000000', \
                     2147483647, 2147483647, 0, 0, 581632, 0)"
                )
                .bind(new_item_id).bind(object_id).bind(asset_id).bind(object_id)
                .bind(script_name).bind(&agent_str)
                .execute(pool).await;

                info!(
                    "[ACTION_BRIDGE] Script '{}' persisted: item={} asset={} in prim={}",
                    script_name, new_item_id, asset_id, object_id
                );
            }
        }

        let script_id = Uuid::new_v5(&object_id, new_item_id.as_bytes());
        {
            let mut objects = self.scene_objects.write();
            if let Some(obj) = objects.values_mut().find(|o| o.uuid == object_id) {
                if !obj.script_items.contains(&script_id) {
                    obj.script_items.push(script_id);
                }
            }
        }

        let task_inv = if let Some(db) = &self.db_connection {
            if let Some(pool) = db.postgres_pool() {
                let rows: Vec<(String, Uuid, i32, i32, i32)> = sqlx::query_as(
                    "SELECT name, assetid, invtype, assettype, currentpermissions FROM primitems WHERE primid = $1"
                )
                .bind(object_id)
                .fetch_all(pool)
                .await
                .unwrap_or_default();
                rows.into_iter()
                    .map(|(name, asset_id, inv_type, asset_type, perms)| {
                        crate::scripting::executor::TaskInventoryEntry {
                            name,
                            asset_id,
                            inv_type,
                            asset_type,
                            permissions: perms as u32,
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        if let Some(yengine) = &self.yengine {
            let engine = yengine.read();
            match engine.rez_script(script_id, script_source, 1) {
                Ok(()) => {
                    let obj_ctx = {
                        let objects = self.scene_objects.read();
                        objects.values().find(|o| o.uuid == object_id).map(|o| {
                            let group_id = o.scene_group_id;
                            let my_link_num = o.link_number;
                            let mut link_names = Vec::new();
                            let mut link_scales = Vec::new();
                            let mut link_count = 1i32;
                            if group_id != Uuid::nil() {
                                let siblings: Vec<_> = objects
                                    .values()
                                    .filter(|s| s.scene_group_id == group_id)
                                    .collect();
                                link_count = siblings.len() as i32;
                                for s in &siblings {
                                    link_names.push((s.link_number, s.name.clone()));
                                    link_scales.push((
                                        s.link_number,
                                        crate::scripting::LSLVector::new(
                                            s.scale[0], s.scale[1], s.scale[2],
                                        ),
                                    ));
                                }
                            }
                            crate::scripting::executor::ObjectContext {
                                object_id: o.uuid,
                                owner_id: o.owner_id,
                                object_name: o.name.clone(),
                                position: crate::scripting::LSLVector::new(
                                    o.position[0],
                                    o.position[1],
                                    o.position[2],
                                ),
                                rotation: crate::scripting::LSLRotation::new(
                                    o.rotation[0],
                                    o.rotation[1],
                                    o.rotation[2],
                                    o.rotation[3],
                                ),
                                scale: crate::scripting::LSLVector::new(
                                    o.scale[0], o.scale[1], o.scale[2],
                                ),
                                velocity: crate::scripting::LSLVector::zero(),
                                region_name: self.region_name.clone(),
                                detect_params: Vec::new(),
                                granted_perms: 0,
                                perm_granter: Uuid::nil(),
                                sitting_avatar_id: o.sitting_avatar.unwrap_or(Uuid::nil()),
                                link_num: my_link_num,
                                link_count,
                                link_names,
                                link_scales,
                                inventory: task_inv.clone(),
                                base_mask: o.base_mask,
                                owner_mask: o.owner_mask,
                                group_mask: o.group_mask,
                                everyone_mask: o.everyone_mask,
                                next_owner_mask: o.next_owner_mask,
                            }
                        })
                    };
                    let lc = obj_ctx.as_ref().map(|c| c.link_count).unwrap_or(1);
                    if let Some(ctx) = obj_ctx {
                        engine.set_script_context(script_id, ctx);
                    }
                    info!("[ACTION_BRIDGE] Script '{}' compiled and started with {} inventory items, {} links: {}", script_name, task_inv.len(), lc, script_id);
                }
                Err(e) => info!(
                    "[ACTION_BRIDGE] Script '{}' compile failed: {}",
                    script_name, e
                ),
            }
        }

        Ok(())
    }

    pub async fn update_script(
        &self,
        owner_id: Uuid,
        local_id: u32,
        script_name: &str,
        script_source: &str,
    ) -> Result<()> {
        let object_uuid = {
            let objects = self.scene_objects.read();
            objects.get(&local_id).map(|o| o.uuid)
        };
        let object_id = object_uuid
            .ok_or_else(|| anyhow!("Object {} not found for script update", local_id))?;

        if let Some(db) = &self.db_connection {
            if let Some(pool) = db.postgres_pool() {
                let existing: Option<(Uuid, Uuid)> = sqlx::query_as(
                    "SELECT itemid, assetid FROM primitems WHERE primid = $1 AND name = $2 LIMIT 1",
                )
                .bind(object_id)
                .bind(script_name)
                .fetch_optional(pool)
                .await?;

                if let Some((item_id, old_asset_id)) = existing {
                    let new_asset_id = Uuid::new_v4();
                    let script_bytes = script_source.as_bytes().to_vec();
                    let agent_str = owner_id.to_string();
                    let _ = sqlx::query(
                        "INSERT INTO assets (id, name, description, assettype, local, temporary, asset_flags, creatorid, data, create_time, access_time) \
                         VALUES ($1::uuid, $2, '', 10, 0, 0, 0, $3, $4, EXTRACT(EPOCH FROM NOW())::integer, EXTRACT(EPOCH FROM NOW())::integer) \
                         ON CONFLICT (id) DO NOTHING"
                    )
                    .bind(new_asset_id).bind(script_name).bind(&agent_str).bind(&script_bytes)
                    .execute(pool).await;

                    let _ = sqlx::query("UPDATE primitems SET assetid = $1 WHERE itemid = $2")
                        .bind(new_asset_id)
                        .bind(item_id)
                        .execute(pool)
                        .await;

                    let script_id = Uuid::new_v5(&object_id, item_id.as_bytes());
                    if let Some(yengine) = &self.yengine {
                        let engine = yengine.read();
                        let _ = engine.stop_script(script_id);
                        match engine.rez_script(script_id, script_source, 1) {
                            Ok(()) => {
                                let obj_ctx = {
                                    let objects = self.scene_objects.read();
                                    objects.values().find(|o| o.uuid == object_id).map(|o| {
                                        crate::scripting::executor::ObjectContext {
                                            object_id: o.uuid,
                                            owner_id: o.owner_id,
                                            object_name: o.name.clone(),
                                            position: crate::scripting::LSLVector::new(
                                                o.position[0],
                                                o.position[1],
                                                o.position[2],
                                            ),
                                            rotation: crate::scripting::LSLRotation::new(
                                                o.rotation[0],
                                                o.rotation[1],
                                                o.rotation[2],
                                                o.rotation[3],
                                            ),
                                            scale: crate::scripting::LSLVector::new(
                                                o.scale[0], o.scale[1], o.scale[2],
                                            ),
                                            velocity: crate::scripting::LSLVector::zero(),
                                            region_name: self.region_name.clone(),
                                            detect_params: Vec::new(),
                                            granted_perms: 0,
                                            perm_granter: Uuid::nil(),
                                            sitting_avatar_id: o
                                                .sitting_avatar
                                                .unwrap_or(Uuid::nil()),
                                            link_num: 0,
                                            link_count: 1,
                                            link_names: Vec::new(),
                                            link_scales: Vec::new(),
                                            inventory: Vec::new(),
                                            base_mask: o.base_mask,
                                            owner_mask: o.owner_mask,
                                            group_mask: o.group_mask,
                                            everyone_mask: o.everyone_mask,
                                            next_owner_mask: o.next_owner_mask,
                                        }
                                    })
                                };
                                if let Some(ctx) = obj_ctx {
                                    engine.set_script_context(script_id, ctx);
                                }
                                info!(
                                    "[ACTION_BRIDGE] Script '{}' updated and restarted: {}",
                                    script_name, script_id
                                );
                            }
                            Err(e) => info!(
                                "[ACTION_BRIDGE] Script '{}' recompile failed: {}",
                                script_name, e
                            ),
                        }
                    }
                    info!(
                        "[ACTION_BRIDGE] Script '{}' asset updated: {} -> {}",
                        script_name, old_asset_id, new_asset_id
                    );
                } else {
                    info!(
                        "[ACTION_BRIDGE] Script '{}' not found in prim {}, inserting new",
                        script_name, object_id
                    );
                    return self
                        .insert_script(owner_id, local_id, script_name, script_source)
                        .await;
                }
            }
        }
        Ok(())
    }

    pub async fn give_object(&self, local_id: u32, target_agent_id: Uuid) -> Result<String> {
        let (root_obj, children, obj_name, owner_id) = {
            let objects = self.scene_objects.read();
            let obj = objects
                .get(&local_id)
                .ok_or_else(|| anyhow!("Object {} not found for give_object", local_id))?;
            let root = obj.clone();
            let group_id = obj.scene_group_id;
            let kids: Vec<SceneObject> = if !group_id.is_nil() {
                objects
                    .values()
                    .filter(|o| o.scene_group_id == group_id && o.local_id != local_id)
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };
            (root, kids, obj.name.clone(), obj.owner_id)
        };

        if let Some(db) = &self.db_connection {
            if let Some(pool) = db.postgres_pool() {
                let mut task_inv_map = std::collections::HashMap::new();
                let root_items = query_task_inventory_for_prim(pool, root_obj.uuid).await;
                if !root_items.is_empty() {
                    task_inv_map.insert(
                        root_obj.uuid,
                        serialize_task_inv_xml_standalone(&root_items, root_obj.uuid),
                    );
                }
                for child in &children {
                    let child_items = query_task_inventory_for_prim(pool, child.uuid).await;
                    if !child_items.is_empty() {
                        task_inv_map.insert(
                            child.uuid,
                            serialize_task_inv_xml_standalone(&child_items, child.uuid),
                        );
                    }
                }
                let asset_id = Uuid::new_v4();
                info!(
                    "[GIVE_OBJECT] '{}' extra_params={} bytes{}, sculpt_type={}, children={}",
                    obj_name,
                    root_obj.extra_params.len(),
                    if root_obj.extra_params.len() >= 24 {
                        format!(
                            " (mesh_uuid={})",
                            Uuid::from_slice(&root_obj.extra_params[7..23]).unwrap_or(Uuid::nil())
                        )
                    } else {
                        String::new()
                    },
                    if root_obj.extra_params.len() >= 24 {
                        root_obj.extra_params[23]
                    } else {
                        0
                    },
                    children.len()
                );
                let xml = serialize_linkset_to_xml_standalone(
                    &root_obj,
                    &children,
                    &task_inv_map,
                    self.region_handle,
                );
                let xml_bytes = xml.as_bytes().to_vec();
                let agent_str = owner_id.to_string();

                let _ = sqlx::query(
                    "INSERT INTO assets (id, name, description, assettype, local, temporary, asset_flags, creatorid, data, create_time, access_time) \
                     VALUES ($1::uuid, $2, '', 6, 0, 0, 0, $3, $4, EXTRACT(EPOCH FROM NOW())::integer, EXTRACT(EPOCH FROM NOW())::integer) \
                     ON CONFLICT (id) DO NOTHING"
                )
                .bind(asset_id).bind(&obj_name).bind(&agent_str).bind(&xml_bytes)
                .execute(pool).await;

                let objects_folder: Option<(Uuid,)> = sqlx::query_as(
                    "SELECT folderid FROM inventoryfolders WHERE agentid = $1 AND type = 6 LIMIT 1",
                )
                .bind(target_agent_id)
                .fetch_optional(pool)
                .await?;

                let folder_id = if let Some((fid,)) = objects_folder {
                    fid
                } else {
                    Uuid::nil()
                };

                let item_id = Uuid::new_v4();
                let nil_uuid = Uuid::nil();
                let _ = sqlx::query(
                    "INSERT INTO inventoryitems (inventoryid, assetid, assettype, inventoryname, inventorydescription, \
                     invtype, creatorid, inventorycurrentpermissions, inventorybasepermissions, \
                     inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions, \
                     groupid, groupowned, saleprice, saletype, flags, \
                     creationdate, parentfolderid, avatarid) \
                     VALUES ($1, $2, 6, $3, '', 6, $4, 2147483647, 2147483647, 0, 0, 581632, \
                     $5, 0, 0, 0, 0, \
                     EXTRACT(EPOCH FROM NOW())::integer, $6, $7)"
                )
                .bind(item_id).bind(asset_id)
                .bind(&obj_name).bind(&agent_str)
                .bind(nil_uuid).bind(folder_id).bind(target_agent_id)
                .execute(pool).await;

                info!(
                    "[ACTION_BRIDGE] Object '{}' given to {}: item={} asset={}",
                    obj_name, target_agent_id, item_id, asset_id
                );

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i32;

                let viewer_addr = {
                    let states = self.avatar_states.read();
                    states
                        .get(&target_agent_id)
                        .filter(|s| !s.is_npc)
                        .map(|s| s.client_addr)
                };

                if let Some(dest) = viewer_addr {
                    let name_bytes = obj_name.as_bytes();
                    let desc_bytes = b"Given by Galadriel";
                    let mut body = Vec::with_capacity(256 + name_bytes.len());

                    body.extend_from_slice(target_agent_id.as_bytes());
                    body.extend_from_slice(Uuid::new_v4().as_bytes());

                    body.push(1u8);
                    body.extend_from_slice(Uuid::nil().as_bytes());
                    body.extend_from_slice(Uuid::nil().as_bytes());
                    body.push((-1i8) as u8);
                    body.push(1u8);
                    body.push(0u8);

                    body.push(1u8);
                    body.extend_from_slice(item_id.as_bytes());
                    body.extend_from_slice(&0u32.to_le_bytes());
                    body.extend_from_slice(folder_id.as_bytes());
                    body.extend_from_slice(owner_id.as_bytes());
                    body.extend_from_slice(target_agent_id.as_bytes());
                    body.extend_from_slice(Uuid::nil().as_bytes());
                    body.extend_from_slice(&2147483647u32.to_le_bytes());
                    body.extend_from_slice(&2147483647u32.to_le_bytes());
                    body.extend_from_slice(&0u32.to_le_bytes());
                    body.extend_from_slice(&0u32.to_le_bytes());
                    body.extend_from_slice(&581632u32.to_le_bytes());
                    body.push(0u8);
                    body.extend_from_slice(asset_id.as_bytes());
                    body.push(6u8);
                    body.push(6u8);
                    body.extend_from_slice(&0u32.to_le_bytes());
                    body.push(0u8);
                    body.extend_from_slice(&0i32.to_le_bytes());
                    body.push((name_bytes.len() + 1) as u8);
                    body.extend_from_slice(name_bytes);
                    body.push(0u8);
                    body.push((desc_bytes.len() + 1) as u8);
                    body.extend_from_slice(desc_bytes);
                    body.push(0u8);
                    body.extend_from_slice(&now.to_le_bytes());
                    body.extend_from_slice(&0u32.to_le_bytes());

                    const BULK_UPDATE_INVENTORY_ID: u32 = 0xFFFF0119;
                    let _ = self
                        .send_message(BULK_UPDATE_INVENTORY_ID, &body, dest, true, true)
                        .await;
                    info!("[ACTION_BRIDGE] Sent BulkUpdateInventory UDP for item {} to agent {} at {}", item_id, target_agent_id, dest);
                }

                let position = {
                    let states = self.avatar_states.read();
                    states
                        .get(&target_agent_id)
                        .map(|s| s.position)
                        .unwrap_or([128.0, 128.0, 25.0])
                };
                let _ = self
                    .say(
                        Uuid::nil(),
                        "Galadriel",
                        &format!("I've placed '{}' in your Objects folder.", obj_name),
                        position,
                    )
                    .await;

                return Ok(format!("Object '{}' added to inventory", obj_name));
            }
        }
        Err(anyhow!("No database connection for give_object"))
    }

    pub async fn package_object_into_prim(
        &self,
        source_local_id: u32,
        container_local_id: u32,
    ) -> Result<String> {
        let (source_root, children, source_name, owner_id) = {
            let objects = self.scene_objects.read();
            let src = objects
                .get(&source_local_id)
                .ok_or_else(|| anyhow!("Source object {} not found", source_local_id))?;
            let root = src.clone();
            let group_id = src.scene_group_id;
            let kids: Vec<SceneObject> = if !group_id.is_nil() {
                objects
                    .values()
                    .filter(|o| o.scene_group_id == group_id && o.local_id != source_local_id)
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };
            (root, kids, src.name.clone(), src.owner_id)
        };

        let container_uuid = {
            let objects = self.scene_objects.read();
            objects
                .get(&container_local_id)
                .ok_or_else(|| anyhow!("Container object {} not found", container_local_id))?
                .uuid
        };

        let db = self
            .db_connection
            .as_ref()
            .ok_or_else(|| anyhow!("No DB for package_object"))?;
        let pool = db
            .postgres_pool()
            .ok_or_else(|| anyhow!("No PG pool for package_object"))?;

        let mut task_inv_map = std::collections::HashMap::new();
        let root_items = query_task_inventory_for_prim(pool, source_root.uuid).await;
        if !root_items.is_empty() {
            task_inv_map.insert(
                source_root.uuid,
                serialize_task_inv_xml_standalone(&root_items, source_root.uuid),
            );
        }
        for child in &children {
            let child_items = query_task_inventory_for_prim(pool, child.uuid).await;
            if !child_items.is_empty() {
                task_inv_map.insert(
                    child.uuid,
                    serialize_task_inv_xml_standalone(&child_items, child.uuid),
                );
            }
        }

        let asset_id = Uuid::new_v4();
        let xml = serialize_linkset_to_xml_standalone(
            &source_root,
            &children,
            &task_inv_map,
            self.region_handle,
        );
        let xml_bytes = xml.as_bytes().to_vec();
        let owner_str = owner_id.to_string();

        let _ = sqlx::query(
            "INSERT INTO assets (id, name, description, assettype, local, temporary, asset_flags, creatorid, data, create_time, access_time) \
             VALUES ($1::uuid, $2, '', 6, 0, 0, 0, $3, $4, EXTRACT(EPOCH FROM NOW())::integer, EXTRACT(EPOCH FROM NOW())::integer) \
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(asset_id).bind(&source_name).bind(&owner_str).bind(&xml_bytes)
        .execute(pool).await;

        let item_id = Uuid::new_v4();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i32;
        let _ = sqlx::query(
            "INSERT INTO primitems (itemid, primid, assetid, parentfolderid, invtype, assettype, name, description, \
             creationdate, creatorid, ownerid, lastownerid, groupid, \
             nextpermissions, currentpermissions, basepermissions, everyonepermissions, grouppermissions, flags) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, $2::uuid, 6, 6, $4, '', \
             $5, $6, $6, $6, '00000000-0000-0000-0000-000000000000', \
             581632, 2147483647, 2147483647, 0, 0, 0)"
        )
        .bind(item_id).bind(container_uuid).bind(asset_id).bind(&source_name)
        .bind(now).bind(&owner_str)
        .execute(pool).await;

        {
            let mut objects = self.scene_objects.write();
            if let Some(container) = objects.get_mut(&container_local_id) {
                container.name = source_name.clone();
                container.description = "Delivery Box".to_string();
            }
        }
        let container_name_db = source_name.clone();
        let container_uuid_db = container_uuid;
        let db_name_update = db.clone();
        tokio::spawn(async move {
            if let Some(pool) = db_name_update.postgres_pool() {
                let _ = sqlx::query("UPDATE prims SET name = $1, description = 'Delivery Box' WHERE uuid = $2::uuid")
                    .bind(&container_name_db).bind(container_uuid_db)
                    .execute(pool).await;
            }
        });
        let _ = self.broadcast_object_update(container_local_id).await;

        let mut ids_to_remove = vec![source_local_id];
        for child in &children {
            ids_to_remove.push(child.local_id);
        }

        {
            let mut objects = self.scene_objects.write();
            for lid in &ids_to_remove {
                objects.remove(lid);
            }
        }

        let kill_count = ids_to_remove.len() as u8;
        let mut kill_data = Vec::with_capacity(1 + ids_to_remove.len() * 4);
        kill_data.push(kill_count);
        for lid in &ids_to_remove {
            kill_data.extend_from_slice(&lid.to_le_bytes());
        }
        let recipients: Vec<SocketAddr> = {
            let states = self.avatar_states.read();
            states
                .values()
                .filter(|s| !s.is_npc)
                .map(|s| s.client_addr)
                .collect()
        };
        for dest in &recipients {
            let _ = self
                .send_message(KILL_OBJECT_ID, &kill_data, *dest, true, false)
                .await;
        }

        let db_clone = db.clone();
        let source_uuid = source_root.uuid;
        let child_uuids: Vec<Uuid> = children.iter().map(|c| c.uuid).collect();
        tokio::spawn(async move {
            if let Some(pool) = db_clone.postgres_pool() {
                for uuid in std::iter::once(source_uuid).chain(child_uuids) {
                    let _ = sqlx::query("DELETE FROM primitems WHERE primid = $1::uuid")
                        .bind(uuid)
                        .execute(pool)
                        .await;
                    let _ = sqlx::query("DELETE FROM primshapes WHERE uuid = $1::uuid")
                        .bind(uuid)
                        .execute(pool)
                        .await;
                    let _ = sqlx::query("DELETE FROM prims WHERE uuid = $1::uuid")
                        .bind(uuid)
                        .execute(pool)
                        .await;
                }
            }
        });

        info!(
            "[ACTION_BRIDGE] Packaged '{}' (local_id={}) into container {} as task inv item {}",
            source_name, source_local_id, container_local_id, item_id
        );
        Ok(format!("Packaged '{}' into container", source_name))
    }

    async fn add_to_ai_inventory(&self, obj_uuid: Uuid, obj_name: &str, owner_id: Uuid) {
        let Some(db) = &self.db_connection else {
            return;
        };
        let Some(pool) = db.postgres_pool() else {
            return;
        };

        let obj_name = obj_name.to_string();
        let pool = pool.clone();
        let owner_str = owner_id.to_string();

        tokio::spawn(async move {
            let objects_folder: Option<(String,)> = sqlx::query_as(
                "SELECT folderid FROM inventoryfolders WHERE agentid = $1 AND type = 6 LIMIT 1",
            )
            .bind(&owner_str)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

            let parent_folder_id = match objects_folder {
                Some((fid,)) => Uuid::parse_str(&fid).unwrap_or(Uuid::nil()),
                None => {
                    return;
                }
            };

            let ai_folder: Option<(String,)> = sqlx::query_as(
                "SELECT folderid FROM inventoryfolders WHERE agentid = $1 AND parentfolderid = $2 AND foldername = 'AI Generated' LIMIT 1"
            )
            .bind(&owner_str).bind(parent_folder_id.to_string()).fetch_optional(&pool).await.unwrap_or(None);

            let ai_folder_id = if let Some((fid,)) = ai_folder {
                Uuid::parse_str(&fid).unwrap_or(Uuid::nil())
            } else {
                let new_folder_id = Uuid::new_v4();
                let _ = sqlx::query(
                    "INSERT INTO inventoryfolders (folderid, agentid, parentfolderid, foldername, type, version) \
                     VALUES ($1, $2, $3, 'AI Generated', -1, 1) ON CONFLICT (folderid) DO NOTHING"
                )
                .bind(new_folder_id.to_string()).bind(&owner_str).bind(parent_folder_id.to_string())
                .execute(&pool).await;
                info!(
                    "[ACTION_BRIDGE] Created 'AI Generated' folder {} for {}",
                    new_folder_id, owner_str
                );
                new_folder_id
            };

            let asset_id = Uuid::new_v4();
            let xml = format!(
                "<SceneObjectGroup><RootPart><SceneObjectPart \
                 xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" \
                 xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\">\
                 <UUID><UUID>{}</UUID></UUID>\
                 <Name>{}</Name>\
                 <CreatorID><UUID>{}</UUID></CreatorID>\
                 <OwnerID><UUID>{}</UUID></OwnerID>\
                 </SceneObjectPart></RootPart></SceneObjectGroup>",
                obj_uuid,
                xml_escape(&obj_name),
                owner_id,
                owner_id
            );
            let xml_bytes = xml.as_bytes().to_vec();

            let _ = sqlx::query(
                "INSERT INTO assets (id, name, description, assettype, local, temporary, asset_flags, creatorid, data, create_time, access_time) \
                 VALUES ($1::uuid, $2, '', 6, 0, 0, 0, $3, $4, EXTRACT(EPOCH FROM NOW())::integer, EXTRACT(EPOCH FROM NOW())::integer) \
                 ON CONFLICT (id) DO NOTHING"
            )
            .bind(asset_id).bind(&obj_name).bind(&owner_str).bind(&xml_bytes)
            .execute(&pool).await;

            let item_id = Uuid::new_v4();
            let _ = sqlx::query(
                "INSERT INTO inventoryitems (inventoryid, assetid, assettype, inventoryname, inventorydescription, \
                 invtype, creatorid, inventorycurrentpermissions, inventorybasepermissions, \
                 inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions, \
                 groupid, groupowned, saleprice, saletype, flags, \
                 creationdate, parentfolderid, avatarid) \
                 VALUES ($1, $2, 6, $3, 'Built by AI', 6, $4, 2147483647, 2147483647, 0, 0, 581632, \
                 '00000000-0000-0000-0000-000000000000', 0, 0, 0, 0, \
                 EXTRACT(EPOCH FROM NOW())::integer, $5, $6)"
            )
            .bind(item_id.to_string()).bind(asset_id.to_string())
            .bind(&obj_name).bind(&owner_str)
            .bind(ai_folder_id.to_string()).bind(&owner_str)
            .execute(&pool).await;

            info!(
                "[ACTION_BRIDGE] Added '{}' to AI Generated inventory: item={} asset={}",
                obj_name, item_id, asset_id
            );
        });
    }

    async fn broadcast_object_update(&self, local_id: u32) -> Result<()> {
        use super::zero_encoder::ZeroEncoder;
        use crate::udp::messages::object_update;

        let region_handle = self.region_handle;

        let (obj_clone, obj_pos, obj_scale, obj_pcode) = {
            let objects = self.scene_objects.read();
            let obj = objects
                .get(&local_id)
                .ok_or_else(|| anyhow!("Object {} not found for update", local_id))?;
            (obj.clone(), obj.position, obj.scale, obj.pcode)
        };

        let mut prim_data = object_update::AvatarObjectData::create_prim_from_scene_object(
            &obj_clone,
            region_handle,
        );
        prim_data.update_flags |= object_update::FLAGS_CREATE_SELECTED;

        let update = object_update::ObjectUpdateMessage {
            region_handle,
            time_dilation: 65535,
            objects: vec![prim_data],
        };
        let serialized_body = update.serialize();

        let clients: Vec<SocketAddr> = {
            let states = self.avatar_states.read();
            states
                .values()
                .filter(|s| !s.is_npc)
                .map(|s| s.client_addr)
                .collect()
        };

        info!("[ACTION_BRIDGE] Broadcasting ObjectUpdate local_id={} pcode={} pos={:?} scale={:?} region_handle=0x{:016x} to {} viewers ({} body bytes)",
              local_id, obj_pcode, obj_pos, obj_scale, region_handle, clients.len(), serialized_body.len());

        for addr in &clients {
            let sequence = self.reliability_manager.next_sequence();
            let mut buffer = bytes::BytesMut::with_capacity(12 + serialized_body.len());
            buffer.put_u8(FLAG_RELIABLE | FLAG_ZEROCODED);
            buffer.put_u32(sequence);
            buffer.put_u8(0x00);

            let mut encoder = ZeroEncoder::new(serialized_body.len() + 10);
            encoder.add_byte(OBJECT_UPDATE_ID as u8);
            encoder.add_bytes(&serialized_body);
            let encoded = encoder.finish();
            buffer.put_slice(&encoded);

            info!(
                "[ACTION_BRIDGE] ObjectUpdate packet: {} bytes, seq={}",
                buffer.len(),
                sequence
            );

            match self.socket.send_to(&buffer, *addr).await {
                Ok(n) => info!(
                    "[ACTION_BRIDGE] Sent ObjectUpdate local_id={} to {} ({} bytes, seq={})",
                    local_id, addr, n, sequence
                ),
                Err(e) => info!(
                    "[ACTION_BRIDGE] FAILED ObjectUpdate local_id={} to {}: {}",
                    local_id, addr, e
                ),
            }
        }

        Ok(())
    }

    async fn send_chat_from_simulator(
        &self,
        message: &str,
        from_name: &str,
        source_id: Uuid,
        from_pos: [f32; 3],
        chat_type: u8,
        source_type: u8,
        audible: u8,
        dest: SocketAddr,
    ) -> Result<()> {
        let mut data = Vec::with_capacity(80 + from_name.len() + message.len());

        let name_bytes = from_name.as_bytes();
        let name_len = name_bytes.len().min(255);
        data.push(name_len as u8);
        data.extend_from_slice(&name_bytes[..name_len]);

        data.extend_from_slice(source_id.as_bytes());
        data.extend_from_slice(source_id.as_bytes());

        data.push(source_type);
        data.push(chat_type);
        data.push(audible);

        data.extend_from_slice(&from_pos[0].to_le_bytes());
        data.extend_from_slice(&from_pos[1].to_le_bytes());
        data.extend_from_slice(&from_pos[2].to_le_bytes());

        let msg_bytes = message.as_bytes();
        let msg_len = msg_bytes.len().min(1024);
        data.extend_from_slice(&(msg_len as u16).to_le_bytes());
        data.extend_from_slice(&msg_bytes[..msg_len]);

        self.send_message(CHAT_FROM_SIMULATOR_ID, &data, dest, true, false)
            .await
    }

    async fn send_message(
        &self,
        message_id: u32,
        data: &[u8],
        dest: SocketAddr,
        reliable: bool,
        zero_encoded: bool,
    ) -> Result<()> {
        use super::zero_encoder::ZeroEncoder;

        let mut buffer = bytes::BytesMut::with_capacity(12 + data.len());

        let mut flags: u8 = 0;
        if reliable {
            flags |= 0x40;
        }
        if zero_encoded {
            flags |= 0x80;
        }

        let sequence = if reliable {
            self.reliability_manager.next_sequence()
        } else {
            0
        };

        buffer.put_u8(flags);
        buffer.put_u32(sequence);
        buffer.put_u8(0x00);

        if zero_encoded {
            let mut encoder = ZeroEncoder::new(data.len() + 10);

            if message_id <= 0xFF {
                encoder.add_byte(message_id as u8);
            } else if message_id <= 0xFFFF {
                encoder.add_byte(0xFF);
                encoder.add_byte((message_id & 0xFF) as u8);
            } else {
                encoder.add_byte(0xFF);
                encoder.add_byte(0xFF);
                let low_bytes = (message_id & 0xFFFF) as u16;
                encoder.add_byte((low_bytes >> 8) as u8);
                encoder.add_byte((low_bytes & 0xFF) as u8);
            }

            encoder.add_bytes(data);
            let encoded = encoder.finish();
            buffer.put_slice(&encoded);
        } else {
            if message_id <= 0xFF {
                buffer.put_u8(message_id as u8);
            } else if message_id <= 0xFFFF {
                buffer.put_u8(0xFF);
                buffer.put_u8((message_id & 0xFF) as u8);
            } else {
                buffer.put_u8(0xFF);
                buffer.put_u8(0xFF);
                let low_bytes = (message_id & 0xFFFF) as u16;
                buffer.put_u8((low_bytes >> 8) as u8);
                buffer.put_u8((low_bytes & 0xFF) as u8);
            }
            buffer.put_slice(data);
        }

        self.socket.send_to(&buffer, dest).await?;
        Ok(())
    }

    pub async fn setup_cinematography(
        &self,
        owner_id: Uuid,
        scene_name: &str,
        shot_type: &str,
        mut camera_waypoints: Vec<crate::ai::npc_avatar::CameraWaypoint>,
        mut lights: Vec<crate::ai::npc_avatar::CinemaLight>,
        lighting_preset: Option<String>,
        subject_position: [f32; 3],
        speed: f32,
    ) -> Result<Vec<(String, u32)>> {
        use crate::ai::cinematography;

        if camera_waypoints.is_empty() {
            camera_waypoints = cinematography::generate_shot_waypoints(shot_type, subject_position);
        }
        if lights.is_empty() {
            let preset = lighting_preset
                .as_deref()
                .unwrap_or_else(|| cinematography::default_lighting_for_shot(shot_type));
            lights = cinematography::generate_lighting_preset(preset, subject_position, 8.0);
        }

        let mut created_ids = Vec::new();
        info!(
            "[CINEMATOGRAPHY] Setting up scene '{}': {} lights, {} waypoints, shot='{}'",
            scene_name,
            lights.len(),
            camera_waypoints.len(),
            shot_type
        );

        for light in &lights {
            let light_params = {
                let mut p = std::collections::HashMap::new();
                p.insert(
                    "COLOR".to_string(),
                    format!("{},{},{}", light.color[0], light.color[1], light.color[2]),
                );
                p.insert("INTENSITY".to_string(), format!("{}", light.intensity));
                p.insert("RADIUS".to_string(), format!("{}", light.radius));
                p.insert("FALLOFF".to_string(), format!("{}", light.falloff));
                p.insert("LIGHT_NAME".to_string(), light.name.clone());
                p
            };
            match self
                .rez_sphere(owner_id, light.position, [0.3, 0.3, 0.3], &light.name)
                .await
            {
                Ok(local_id) => {
                    created_ids.push((light.name.clone(), local_id));
                    if let Err(e) = self
                        .set_object_color(
                            local_id,
                            [light.color[0], light.color[1], light.color[2], 0.1],
                        )
                        .await
                    {
                        info!("[CINEMATOGRAPHY] Failed to set light alpha: {}", e);
                    }
                    if let Some(source) =
                        crate::ai::script_templates::apply_template("cinema_light", &light_params)
                    {
                        if let Err(e) = self
                            .insert_script(owner_id, local_id, "cinema_light", &source)
                            .await
                        {
                            info!("[CINEMATOGRAPHY] Failed to insert light script: {}", e);
                        }
                    }
                    info!(
                        "[CINEMATOGRAPHY] Light '{}' placed at {:?} (intensity={}, radius={})",
                        light.name, light.position, light.intensity, light.radius
                    );
                }
                Err(e) => info!(
                    "[CINEMATOGRAPHY] Failed to rez light '{}': {}",
                    light.name, e
                ),
            }
        }

        let drone_pos = if !camera_waypoints.is_empty() {
            camera_waypoints[0].position
        } else {
            [
                subject_position[0] - 10.0,
                subject_position[1],
                subject_position[2] + 5.0,
            ]
        };

        let drone_name = format!("{} Camera", scene_name);
        match self
            .rez_sphere(owner_id, drone_pos, [0.5, 0.5, 0.5], &drone_name)
            .await
        {
            Ok(drone_id) => {
                created_ids.push((drone_name.clone(), drone_id));
                if let Err(e) = self.set_object_color(drone_id, [0.0, 0.0, 0.0, 0.0]).await {
                    info!("[CINEMATOGRAPHY] Failed to set drone alpha: {}", e);
                }

                let mut wp_data = String::new();
                for (i, wp) in camera_waypoints.iter().enumerate() {
                    if i > 0 {
                        wp_data.push('|');
                    }
                    wp_data.push_str(&format!(
                        "{}|{}|{}|{}|{}|{}|{}",
                        wp.position[0],
                        wp.position[1],
                        wp.position[2],
                        wp.focus[0],
                        wp.focus[1],
                        wp.focus[2],
                        wp.dwell
                    ));
                }

                let mut drone_params = std::collections::HashMap::new();
                drone_params.insert("SPEED".to_string(), format!("{}", speed));
                if let Some(source) =
                    crate::ai::script_templates::apply_template("drone_camera", &drone_params)
                {
                    if let Err(e) = self
                        .insert_script(owner_id, drone_id, "drone_camera", &source)
                        .await
                    {
                        info!("[CINEMATOGRAPHY] Failed to insert drone script: {}", e);
                    }
                }

                {
                    let objects = self.scene_objects.read();
                    if let Some(drone_obj) = objects.get(&drone_id) {
                        if let Some(yengine) = &self.yengine {
                            let engine = yengine.read();
                            for script_id in &drone_obj.script_items {
                                engine.post_event(
                                    *script_id,
                                    crate::scripting::state_machine::ScriptEvent {
                                        event_type: crate::scripting::state_machine::ScriptEventType::LinkMessage,
                                        args: vec![
                                            crate::scripting::LSLValue::Integer(0),
                                            crate::scripting::LSLValue::Integer(9000),
                                            crate::scripting::LSLValue::String(wp_data.clone()),
                                            crate::scripting::LSLValue::String(String::new()),
                                        ],
                                    },
                                );
                            }
                        }
                    }
                }

                info!(
                    "[CINEMATOGRAPHY] Drone camera placed at {:?} with {} waypoints",
                    drone_pos,
                    camera_waypoints.len()
                );
            }
            Err(e) => info!("[CINEMATOGRAPHY] Failed to rez drone camera: {}", e),
        }

        if created_ids.len() > 1 {
            let root_id = created_ids.last().unwrap().1;
            let child_ids: Vec<u32> = created_ids
                .iter()
                .take(created_ids.len() - 1)
                .map(|(_, id)| *id)
                .collect();
            if let Err(e) = self.link_objects(root_id, &child_ids).await {
                info!("[CINEMATOGRAPHY] Failed to link cinema rig: {}", e);
            }
        }

        let position = {
            let states = self.avatar_states.read();
            states
                .values()
                .find(|s| !s.is_npc)
                .map(|s| s.position)
                .unwrap_or([128.0, 128.0, 25.0])
        };
        let _ = self.say(
            owner_id,
            "Galadriel",
            &format!("Cinematic scene '{}' ready! Sit on the drone camera to start the {} shot. Touch to pause/resume.", scene_name, shot_type),
            position,
        ).await;

        info!(
            "[CINEMATOGRAPHY] Scene '{}' complete: {} objects created",
            scene_name,
            created_ids.len()
        );
        Ok(created_ids)
    }

    pub async fn generate_terrain(
        &self,
        npc_id: Uuid,
        preset: &str,
        seed: Option<u32>,
        scale: Option<f32>,
        roughness: Option<f32>,
        water_level: Option<f32>,
        region_id: Option<&str>,
        grid_size: Option<u32>,
        grid_x: Option<u32>,
        grid_y: Option<u32>,
    ) -> Result<String> {
        use crate::region::terrain_generator::{self, preset_description, TerrainParams};
        use crate::region::terrain_sender::TerrainSender;

        let db = self
            .db_connection
            .as_ref()
            .ok_or_else(|| anyhow!("No database connection for terrain generation"))?;

        let region_uuid = if let Some(rid) = region_id {
            Uuid::parse_str(rid).unwrap_or(self.default_region_uuid)
        } else {
            self.default_region_uuid
        };
        let actual_seed = seed.unwrap_or_else(|| rand::random::<u32>());

        let params = TerrainParams {
            preset: preset.to_string(),
            seed: actual_seed,
            scale: scale.unwrap_or(1.0),
            roughness: roughness.unwrap_or(0.5),
            water_level: water_level.unwrap_or(20.0),
            size: 256,
        };

        let heightmap = if let (Some(gs), Some(gx), Some(gy)) = (grid_size, grid_x, grid_y) {
            terrain_generator::generate_grid_tile(&params, gs, gx, gy)
        } else {
            terrain_generator::generate(&params)
        };

        let terrain_sender = TerrainSender::new(db.clone(), self.socket.clone());
        terrain_sender
            .store_and_cache_heightmap(region_uuid, heightmap.clone())
            .await?;

        let client_addrs: Vec<SocketAddr> = {
            let states = self.avatar_states.read();
            states
                .values()
                .filter(|s| !s.is_npc)
                .map(|s| s.client_addr)
                .collect()
        };
        terrain_sender
            .broadcast_full_terrain(region_uuid, &client_addrs)
            .await?;

        let instance_dir =
            std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
        let terrains_dir = format!("{}/Terrains", instance_dir);
        std::fs::create_dir_all(&terrains_dir).ok();
        let r32_path = if let (Some(gs), Some(gx), Some(gy)) = (grid_size, grid_x, grid_y) {
            format!(
                "{}/{}_{}_grid{}x{}_tile{}_{}.r32",
                terrains_dir, preset, actual_seed, gs, gs, gx, gy
            )
        } else {
            format!("{}/{}_{}.r32", terrains_dir, preset, actual_seed)
        };
        save_heightmap_r32(&heightmap, 256, &r32_path)?;
        info!("[TERRAIN] Backed up terrain to {}", r32_path);

        let position = {
            let states = self.avatar_states.read();
            states
                .values()
                .find(|s| !s.is_npc)
                .map(|s| s.position)
                .unwrap_or([128.0, 128.0, 25.0])
        };

        let preview_pos = [position[0] + 2.0, position[1], position[2] + 1.0];
        match self
            .rez_terrain_preview(npc_id, &heightmap, preview_pos, preset)
            .await
        {
            Ok(lid) => info!("[TERRAIN] Preview model rezzed: local_id={}", lid),
            Err(e) => warn!("[TERRAIN] Failed to rez preview model: {}", e),
        }

        let hmin = heightmap.iter().cloned().fold(f32::INFINITY, f32::min);
        let hmax = heightmap.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let desc = preset_description(preset);
        let msg = format!(
            "Generated '{}' terrain (seed={}, {}) — height range {:.1}m to {:.1}m. Preview model placed nearby. Saved to {}",
            preset, actual_seed, desc, hmin, hmax, r32_path
        );
        info!("[TERRAIN] {}", msg);

        let _ = self.say(npc_id, "Galadriel", &msg, position).await;

        Ok(msg)
    }

    pub async fn preview_terrain(
        &self,
        npc_id: Uuid,
        preset: &str,
        seed: Option<u32>,
        scale: Option<f32>,
        roughness: Option<f32>,
        water_level: Option<f32>,
        region_id: Option<&str>,
        grid_size: Option<u32>,
        grid_x: Option<u32>,
        grid_y: Option<u32>,
    ) -> Result<String> {
        use crate::region::terrain_generator::{self, preset_description, TerrainParams};

        let region_uuid = if let Some(rid) = region_id {
            Uuid::parse_str(rid).unwrap_or(self.default_region_uuid)
        } else {
            self.default_region_uuid
        };
        let actual_seed = seed.unwrap_or_else(|| rand::random::<u32>());

        let params = TerrainParams {
            preset: preset.to_string(),
            seed: actual_seed,
            scale: scale.unwrap_or(1.0),
            roughness: roughness.unwrap_or(0.5),
            water_level: water_level.unwrap_or(20.0),
            size: 256,
        };

        let heightmap = if let (Some(gs), Some(gx), Some(gy)) = (grid_size, grid_x, grid_y) {
            terrain_generator::generate_grid_tile(&params, gs, gx, gy)
        } else {
            terrain_generator::generate(&params)
        };

        let instance_dir =
            std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
        let terrains_dir = format!("{}/Terrains", instance_dir);
        std::fs::create_dir_all(&terrains_dir).ok();
        let r32_path = if let (Some(gs), Some(gx), Some(gy)) = (grid_size, grid_x, grid_y) {
            format!(
                "{}/{}_{}_grid{}x{}_tile{}_{}.r32",
                terrains_dir, preset, actual_seed, gs, gs, gx, gy
            )
        } else {
            format!("{}/{}_{}.r32", terrains_dir, preset, actual_seed)
        };

        let position = {
            let states = self.avatar_states.read();
            states
                .values()
                .find(|s| !s.is_npc)
                .map(|s| s.position)
                .unwrap_or([128.0, 128.0, 25.0])
        };

        let preview_pos = [position[0] + 2.0, position[1], position[2] + 1.0];
        let preview_local_id = match self
            .rez_terrain_preview(npc_id, &heightmap, preview_pos, preset)
            .await
        {
            Ok(lid) => {
                info!("[TERRAIN] Preview model rezzed: local_id={}", lid);
                lid
            }
            Err(e) => {
                warn!("[TERRAIN] Failed to rez preview model: {}", e);
                0
            }
        };

        let preview_id = format!("{}_{}", preset, actual_seed);

        {
            let mut pending = self.pending_terrains.write();
            pending.insert(
                preview_id.clone(),
                PendingTerrain {
                    heightmap,
                    region_uuid,
                    preset: preset.to_string(),
                    seed: actual_seed,
                    r32_path,
                    preview_local_id,
                    grid_size,
                    grid_x,
                    grid_y,
                },
            );
        }

        let hmin = {
            let pending = self.pending_terrains.read();
            let pt = pending.get(&preview_id).unwrap();
            pt.heightmap.iter().cloned().fold(f32::INFINITY, f32::min)
        };
        let hmax = {
            let pending = self.pending_terrains.read();
            let pt = pending.get(&preview_id).unwrap();
            pt.heightmap
                .iter()
                .cloned()
                .fold(f32::NEG_INFINITY, f32::max)
        };
        let desc = preset_description(preset);
        let msg = format!(
            "PREVIEW: '{}' terrain (seed={}, {}) — height range {:.1}m to {:.1}m. Preview model placed nearby. \
             Say 'approve terrain' to apply it, or 'reject terrain' to discard. Preview ID: {}",
            preset, actual_seed, desc, hmin, hmax, preview_id
        );
        info!("[TERRAIN] {}", msg);
        let _ = self.say(npc_id, "Galadriel", &msg, position).await;

        Ok(msg)
    }

    pub async fn apply_pending_terrain(&self, npc_id: Uuid, preview_id: &str) -> Result<String> {
        use crate::region::terrain_sender::TerrainSender;

        let db = self
            .db_connection
            .as_ref()
            .ok_or_else(|| anyhow!("No database connection for terrain apply"))?;

        let pending_terrain = {
            let mut pending = self.pending_terrains.write();
            pending.remove(preview_id)
                .ok_or_else(|| anyhow!("No pending terrain with preview ID '{}'. It may have expired or already been applied.", preview_id))?
        };

        let terrain_sender = TerrainSender::new(db.clone(), self.socket.clone());
        terrain_sender
            .store_and_cache_heightmap(
                pending_terrain.region_uuid,
                pending_terrain.heightmap.clone(),
            )
            .await?;

        let client_addrs: Vec<SocketAddr> = {
            let states = self.avatar_states.read();
            states
                .values()
                .filter(|s| !s.is_npc)
                .map(|s| s.client_addr)
                .collect()
        };
        terrain_sender
            .broadcast_full_terrain(pending_terrain.region_uuid, &client_addrs)
            .await?;

        save_heightmap_r32(&pending_terrain.heightmap, 256, &pending_terrain.r32_path)?;
        info!(
            "[TERRAIN] Backed up terrain to {}",
            pending_terrain.r32_path
        );

        if pending_terrain.preview_local_id > 0 {
            self.scene_objects
                .write()
                .remove(&pending_terrain.preview_local_id);
            let _ = self.delete_object(pending_terrain.preview_local_id).await;
        }

        let position = {
            let states = self.avatar_states.read();
            states
                .values()
                .find(|s| !s.is_npc)
                .map(|s| s.position)
                .unwrap_or([128.0, 128.0, 25.0])
        };

        let hmin = pending_terrain
            .heightmap
            .iter()
            .cloned()
            .fold(f32::INFINITY, f32::min);
        let hmax = pending_terrain
            .heightmap
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        let msg = format!(
            "Terrain APPLIED: '{}' (seed={}) — {:.1}m to {:.1}m. Saved to {}",
            pending_terrain.preset, pending_terrain.seed, hmin, hmax, pending_terrain.r32_path
        );
        info!("[TERRAIN] {}", msg);
        let _ = self.say(npc_id, "Galadriel", &msg, position).await;

        Ok(msg)
    }

    pub async fn reject_pending_terrain(&self, npc_id: Uuid, preview_id: &str) -> Result<String> {
        let pending_terrain = {
            let mut pending = self.pending_terrains.write();
            pending.remove(preview_id)
                .ok_or_else(|| anyhow!("No pending terrain with preview ID '{}'. It may have expired or already been applied.", preview_id))?
        };

        if pending_terrain.preview_local_id > 0 {
            self.scene_objects
                .write()
                .remove(&pending_terrain.preview_local_id);
            let _ = self.delete_object(pending_terrain.preview_local_id).await;
        }

        let position = {
            let states = self.avatar_states.read();
            states
                .values()
                .find(|s| !s.is_npc)
                .map(|s| s.position)
                .unwrap_or([128.0, 128.0, 25.0])
        };

        let msg = format!(
            "Terrain REJECTED: '{}' (seed={}) — preview removed. Try again with different parameters.",
            pending_terrain.preset, pending_terrain.seed
        );
        info!("[TERRAIN] {}", msg);
        let _ = self.say(npc_id, "Galadriel", &msg, position).await;

        Ok(msg)
    }

    pub async fn rez_terrain_preview(
        &self,
        owner_id: Uuid,
        heightmap: &[f32],
        position: [f32; 3],
        preset_name: &str,
    ) -> Result<u32> {
        let side = (heightmap.len() as f64).sqrt() as u32;
        let hmin = heightmap.iter().cloned().fold(f32::INFINITY, f32::min);
        let hmax = heightmap.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = (hmax - hmin).max(0.001);

        let sculpt_size = 64u32;
        let mut sculpt_img = image::RgbaImage::new(sculpt_size, sculpt_size);
        for sy in 0..sculpt_size {
            for sx in 0..sculpt_size {
                let hx = (sx as f32 / (sculpt_size - 1) as f32 * (side - 1) as f32) as u32;
                let hy = (sy as f32 / (sculpt_size - 1) as f32 * (side - 1) as f32) as u32;
                let hx = hx.min(side - 1);
                let hy = hy.min(side - 1);
                let h = heightmap[(hy * side + hx) as usize];
                let normalized_h = ((h - hmin) / range).clamp(0.0, 1.0);
                let r = (sx as f32 / (sculpt_size - 1) as f32 * 255.0) as u8;
                let g = (sy as f32 / (sculpt_size - 1) as f32 * 255.0) as u8;
                let b = (normalized_h * 255.0) as u8;
                sculpt_img.put_pixel(sx, sy, image::Rgba([r, g, b, 255]));
            }
        }

        let dynamic_img = image::DynamicImage::ImageRgba8(sculpt_img);
        let mut png_data = std::io::Cursor::new(Vec::new());
        dynamic_img
            .write_to(&mut png_data, image::ImageFormat::Png)
            .map_err(|e| anyhow!("Failed to encode terrain sculpt PNG: {}", e))?;
        let j2k_data = png_data.into_inner();

        let sculpt_asset_id = Uuid::new_v4();
        if let Some(db) = &self.db_connection {
            if let Some(pool) = db.postgres_pool() {
                let agent_str = owner_id.to_string();
                let _ = sqlx::query(
                    "INSERT INTO assets (id, name, description, assettype, local, temporary, asset_flags, creatorid, data, create_time, access_time) \
                     VALUES ($1::uuid, $2, 'Terrain sculpt map', 0, 0, 0, 0, $3, $4, EXTRACT(EPOCH FROM NOW())::integer, EXTRACT(EPOCH FROM NOW())::integer) \
                     ON CONFLICT (id) DO NOTHING"
                )
                .bind(sculpt_asset_id)
                .bind(format!("terrain_sculpt_{}", preset_name))
                .bind(&agent_str)
                .bind(&j2k_data)
                .execute(pool).await;
                info!(
                    "[TERRAIN_PREVIEW] Sculpt asset stored: {} ({} bytes)",
                    sculpt_asset_id,
                    j2k_data.len()
                );
            }
        }

        let preview_scale = (side as f32) / 128.0;
        let height_scale = range / 128.0;
        let local_id = self.next_prim_local_id.fetch_add(1, Ordering::SeqCst);
        let mut obj = SceneObject::new_box(local_id, owner_id, position);
        obj.name = format!("Terrain Preview: {}", preset_name);
        obj.scale = [preview_scale, preview_scale, height_scale.max(0.1)];

        obj.path_curve = 32;
        obj.profile_curve = 5;

        let mut extra = Vec::with_capacity(24);
        extra.push(1u8);
        let ep_type: u16 = 0x30;
        extra.extend_from_slice(&ep_type.to_le_bytes());
        let ep_len: u32 = 17;
        extra.extend_from_slice(&ep_len.to_le_bytes());
        extra.extend_from_slice(sculpt_asset_id.as_bytes());
        extra.push(3);
        obj.extra_params = extra;

        obj.description = format!("Heightmap: {:.1}m-{:.1}m | {}x{}", hmin, hmax, side, side);

        self.scene_objects.write().insert(local_id, obj);
        self.broadcast_object_update(local_id).await?;

        info!(
            "[TERRAIN_PREVIEW] Rezzed terrain preview model at {:?} ({:.0}m x {:.0}m)",
            position, preview_scale, preview_scale
        );
        Ok(local_id)
    }

    pub async fn load_terrain_image(
        &self,
        npc_id: Uuid,
        file_path: &str,
        height_min: Option<f32>,
        height_max: Option<f32>,
    ) -> Result<String> {
        use crate::region::terrain_sender::TerrainSender;

        let db = self
            .db_connection
            .as_ref()
            .ok_or_else(|| anyhow!("No database connection for terrain loading"))?;

        let heightmap = load_heightmap_from_image(file_path, height_min, height_max)?;
        let side = (heightmap.len() as f64).sqrt() as u32;
        let region_uuid = self.default_region_uuid;

        let terrain_sender = TerrainSender::new(db.clone(), self.socket.clone());
        terrain_sender
            .store_and_cache_heightmap(region_uuid, heightmap.clone())
            .await?;

        let client_addrs: Vec<SocketAddr> = {
            let states = self.avatar_states.read();
            states
                .values()
                .filter(|s| !s.is_npc)
                .map(|s| s.client_addr)
                .collect()
        };
        terrain_sender
            .broadcast_full_terrain(region_uuid, &client_addrs)
            .await?;

        let instance_dir =
            std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
        let terrains_dir = format!("{}/Terrains", instance_dir);
        std::fs::create_dir_all(&terrains_dir).ok();
        let basename = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");
        let r32_path = format!("{}/{}_imported.r32", terrains_dir, basename);
        save_heightmap_r32(&heightmap, side, &r32_path)?;

        let position = {
            let states = self.avatar_states.read();
            states
                .values()
                .find(|s| !s.is_npc)
                .map(|s| s.position)
                .unwrap_or([128.0, 128.0, 25.0])
        };

        let preview_pos = [position[0] + 2.0, position[1], position[2] + 1.0];
        let _ = self
            .rez_terrain_preview(npc_id, &heightmap, preview_pos, basename)
            .await;

        let hmin = heightmap.iter().cloned().fold(f32::INFINITY, f32::min);
        let hmax = heightmap.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let msg = format!(
            "Loaded terrain from image '{}' ({}x{}) — height range {:.1}m to {:.1}m. Preview placed nearby. Backed up to {}",
            file_path, side, side, hmin, hmax, r32_path
        );
        info!("[TERRAIN] {}", msg);
        let _ = self.say(npc_id, "Galadriel", &msg, position).await;

        Ok(msg)
    }

    pub async fn load_terrain_r32(&self, npc_id: Uuid, file_path: &str) -> Result<String> {
        use crate::region::terrain_sender::TerrainSender;

        let db = self
            .db_connection
            .as_ref()
            .ok_or_else(|| anyhow!("No database connection for terrain loading"))?;

        let heightmap = load_heightmap_r32(file_path)?;
        let side = (heightmap.len() as f64).sqrt() as u32;
        let region_uuid = self.default_region_uuid;

        let terrain_sender = TerrainSender::new(db.clone(), self.socket.clone());
        terrain_sender
            .store_and_cache_heightmap(region_uuid, heightmap.clone())
            .await?;

        let client_addrs: Vec<SocketAddr> = {
            let states = self.avatar_states.read();
            states
                .values()
                .filter(|s| !s.is_npc)
                .map(|s| s.client_addr)
                .collect()
        };
        terrain_sender
            .broadcast_full_terrain(region_uuid, &client_addrs)
            .await?;

        let hmin = heightmap.iter().cloned().fold(f32::INFINITY, f32::min);
        let hmax = heightmap.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let msg = format!(
            "Loaded terrain from '{}' ({}x{}) — height range {:.1}m to {:.1}m",
            file_path, side, side, hmin, hmax
        );
        info!("[TERRAIN] {}", msg);

        let position = {
            let states = self.avatar_states.read();
            states
                .values()
                .find(|s| !s.is_npc)
                .map(|s| s.position)
                .unwrap_or([128.0, 128.0, 25.0])
        };
        let _ = self.say(npc_id, "Galadriel", &msg, position).await;

        Ok(msg)
    }
}

fn heightmap_color(normalized: f32) -> (u8, u8, u8) {
    if normalized < 0.15 {
        (30, 60, 120)
    } else if normalized < 0.2 {
        let t = (normalized - 0.15) / 0.05;
        (
            (30.0 + t * 164.0) as u8,
            (60.0 + t * 150.0) as u8,
            (120.0 + t * 20.0) as u8,
        )
    } else if normalized < 0.3 {
        (194, 210, 140)
    } else if normalized < 0.6 {
        let t = (normalized - 0.3) / 0.3;
        (
            (90.0 - t * 30.0) as u8,
            (160.0 - t * 40.0) as u8,
            (50.0 + t * 10.0) as u8,
        )
    } else if normalized < 0.8 {
        let t = (normalized - 0.6) / 0.2;
        (
            (100.0 + t * 40.0) as u8,
            (100.0 + t * 30.0) as u8,
            (80.0 + t * 20.0) as u8,
        )
    } else {
        let t = (normalized - 0.8) / 0.2;
        (
            (200.0 + t * 55.0) as u8,
            (200.0 + t * 55.0) as u8,
            (200.0 + t * 55.0) as u8,
        )
    }
}

pub fn save_heightmap_r32(heightmap: &[f32], size: u32, path: &str) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;
    for &h in heightmap {
        file.write_all(&h.to_le_bytes())?;
    }
    info!("[TERRAIN] Saved {0}x{0} .r32 heightmap to {1}", size, path);
    Ok(())
}

pub fn load_heightmap_r32(path: &str) -> Result<Vec<f32>> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)
        .map_err(|e| anyhow!("Cannot open .r32 file '{}': {}", path, e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    if data.len() % 4 != 0 {
        return Err(anyhow!(
            "Invalid .r32 file: size {} not multiple of 4",
            data.len()
        ));
    }
    let count = data.len() / 4;
    let side = (count as f64).sqrt() as usize;
    if side * side != count {
        return Err(anyhow!(
            "Invalid .r32 file: {} values is not a square grid",
            count
        ));
    }
    let mut heightmap = Vec::with_capacity(count);
    for i in 0..count {
        let offset = i * 4;
        let h = f32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        heightmap.push(h);
    }
    info!(
        "[TERRAIN] Loaded {0}x{0} .r32 heightmap from {1}",
        side, path
    );
    Ok(heightmap)
}

pub fn load_heightmap_from_image(
    path: &str,
    height_min: Option<f32>,
    height_max: Option<f32>,
) -> Result<Vec<f32>> {
    let img = image::open(path).map_err(|e| anyhow!("Cannot open image '{}': {}", path, e))?;
    let gray = img.to_luma8();
    let (w, h) = gray.dimensions();

    let target_size = if w >= 512 && h >= 512 { 512u32 } else { 256u32 };

    let resized = if w != target_size || h != target_size {
        image::imageops::resize(
            &gray,
            target_size,
            target_size,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        gray
    };

    let lo = height_min.unwrap_or(0.0);
    let hi = height_max.unwrap_or(100.0);
    let range = (hi - lo).max(0.001);

    let total = (target_size * target_size) as usize;
    let mut heightmap = Vec::with_capacity(total);
    for y in 0..target_size {
        for x in 0..target_size {
            let pixel = resized.get_pixel(x, y).0[0];
            let normalized = pixel as f32 / 255.0;
            heightmap.push(lo + normalized * range);
        }
    }

    info!(
        "[TERRAIN] Loaded image '{}' ({}x{}) → {}x{} heightmap, range {:.1}..{:.1}m",
        path, w, h, target_size, target_size, lo, hi
    );
    Ok(heightmap)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn extract_sculpt_from_extra_params(extra: &[u8]) -> (Uuid, u8) {
    if extra.len() >= 24 && extra[0] >= 1 {
        let param_type = u16::from_le_bytes([extra[1], extra[2]]);
        if param_type == 0x0030 {
            let data_len = u32::from_le_bytes([extra[3], extra[4], extra[5], extra[6]]) as usize;
            if data_len == 17 && extra.len() >= 7 + data_len {
                let uuid = Uuid::from_slice(&extra[7..23]).unwrap_or(Uuid::nil());
                let sculpt_type = extra[23];
                return (uuid, sculpt_type);
            }
        }
    }
    (Uuid::nil(), 0)
}

fn serialize_part_xml_standalone(
    obj: &SceneObject,
    group_pos: [f32; 3],
    offset_pos: [f32; 3],
    parent_id: u32,
    task_inv_xml: &str,
    region_handle: u64,
) -> String {
    use base64::Engine;
    let tex_hex = base64::engine::general_purpose::STANDARD.encode(&obj.texture_entry);
    let extra_b64 = if obj.extra_params.is_empty() {
        "AA==".to_string()
    } else {
        base64::engine::general_purpose::STANDARD.encode(&obj.extra_params)
    };
    let (sculpt_texture, sculpt_type) = extract_sculpt_from_extra_params(&obj.extra_params);
    let creation_date = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!(
        r#"<SceneObjectPart xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema"><AllowedDrop>false</AllowedDrop><CreatorID><Guid>{owner}</Guid></CreatorID><FolderID><Guid>{uuid}</Guid></FolderID><InventorySerial>{inv_serial}</InventorySerial>{task_inv}<UUID><Guid>{uuid}</Guid></UUID><LocalId>{local_id}</LocalId><Name>{name}</Name><Material>{material}</Material><RegionHandle>{region_handle}</RegionHandle><ScriptAccessPin>0</ScriptAccessPin><GroupPosition><X>{gpx}</X><Y>{gpy}</Y><Z>{gpz}</Z></GroupPosition><OffsetPosition><X>{opx}</X><Y>{opy}</Y><Z>{opz}</Z></OffsetPosition><RotationOffset><X>{rx}</X><Y>{ry}</Y><Z>{rz}</Z><W>{rw}</W></RotationOffset><Velocity><X>0</X><Y>0</Y><Z>0</Z></Velocity><AngularVelocity><X>0</X><Y>0</Y><Z>0</Z></AngularVelocity><Acceleration><X>0</X><Y>0</Y><Z>0</Z></Acceleration><Description>{desc}</Description><Color /><Text /><SitName /><TouchName /><LinkNum>{link_num}</LinkNum><ClickAction>0</ClickAction><Shape><ProfileCurve>{profile_curve}</ProfileCurve><TextureEntry>{tex}</TextureEntry><ExtraParams>{extra}</ExtraParams><PathBegin>{path_begin}</PathBegin><PathCurve>{path_curve}</PathCurve><PathEnd>{path_end}</PathEnd><PathRadiusOffset>{path_radius_offset}</PathRadiusOffset><PathRevolutions>{path_revolutions}</PathRevolutions><PathScaleX>{path_scale_x}</PathScaleX><PathScaleY>{path_scale_y}</PathScaleY><PathShearX>{path_shear_x}</PathShearX><PathShearY>{path_shear_y}</PathShearY><PathSkew>{path_skew}</PathSkew><PathTaperX>{path_taper_x}</PathTaperX><PathTaperY>{path_taper_y}</PathTaperY><PathTwist>{path_twist}</PathTwist><PathTwistBegin>{path_twist_begin}</PathTwistBegin><PCode>{pcode}</PCode><ProfileBegin>{profile_begin}</ProfileBegin><ProfileEnd>{profile_end}</ProfileEnd><ProfileHollow>{profile_hollow}</ProfileHollow><State>0</State><ProfileShape>0</ProfileShape><HollowShape>0</HollowShape><SculptTexture><Guid>{sculpt_texture}</Guid></SculptTexture><SculptType>{sculpt_type}</SculptType><FlexiSoftness>0</FlexiSoftness><FlexiTension>0</FlexiTension><FlexiDrag>0</FlexiDrag><FlexiGravity>0</FlexiGravity><FlexiWind>0</FlexiWind><FlexiForceX>0</FlexiForceX><FlexiForceY>0</FlexiForceY><FlexiForceZ>0</FlexiForceZ><LightColorR>0</LightColorR><LightColorG>0</LightColorG><LightColorB>0</LightColorB><LightColorA>1</LightColorA><LightRadius>0</LightRadius><LightCutoff>0</LightCutoff><LightFalloff>0</LightFalloff><LightIntensity>0</LightIntensity></Shape><Scale><X>{sx}</X><Y>{sy}</Y><Z>{sz}</Z></Scale><SitTargetOrientation><X>0</X><Y>0</Y><Z>0</Z><W>1</W></SitTargetOrientation><SitTargetPosition><X>0</X><Y>0</Y><Z>0</Z></SitTargetPosition><SitTargetPositionLL><X>0</X><Y>0</Y><Z>0</Z></SitTargetPositionLL><SitTargetOrientationLL><X>0</X><Y>0</Y><Z>0</Z><W>1</W></SitTargetOrientationLL><ParentID>{parent_id}</ParentID><CreationDate>{creation_date}</CreationDate><Category>0</Category><SalePrice>0</SalePrice><ObjectSaleType>0</ObjectSaleType><OwnershipCost>0</OwnershipCost><GroupID><Guid>{group_id}</Guid></GroupID><OwnerID><Guid>{owner}</Guid></OwnerID><LastOwnerID><Guid>{owner}</Guid></LastOwnerID><BaseMask>{base_mask}</BaseMask><OwnerMask>{owner_mask}</OwnerMask><GroupMask>0</GroupMask><EveryoneMask>0</EveryoneMask><NextOwnerMask>{next_owner_mask}</NextOwnerMask><Flags>{flags}</Flags><CollisionSound><Guid>00000000-0000-0000-0000-000000000000</Guid></CollisionSound><CollisionSoundVolume>0</CollisionSoundVolume></SceneObjectPart>"#,
        uuid = obj.uuid,
        local_id = obj.local_id,
        name = xml_escape(&obj.name),
        desc = xml_escape(&obj.description),
        material = obj.material,
        region_handle = region_handle,
        gpx = group_pos[0],
        gpy = group_pos[1],
        gpz = group_pos[2],
        opx = offset_pos[0],
        opy = offset_pos[1],
        opz = offset_pos[2],
        rx = obj.rotation[0],
        ry = obj.rotation[1],
        rz = obj.rotation[2],
        rw = obj.rotation[3],
        sx = obj.scale[0],
        sy = obj.scale[1],
        sz = obj.scale[2],
        link_num = obj.link_number,
        parent_id = parent_id,
        pcode = obj.pcode,
        profile_curve = obj.profile_curve,
        path_curve = obj.path_curve,
        path_begin = obj.path_begin,
        path_end = obj.path_end,
        path_scale_x = obj.path_scale_x,
        path_scale_y = obj.path_scale_y,
        path_shear_x = obj.path_shear_x,
        path_shear_y = obj.path_shear_y,
        path_twist = obj.path_twist,
        path_twist_begin = obj.path_twist_begin,
        path_radius_offset = obj.path_radius_offset,
        path_taper_x = obj.path_taper_x,
        path_taper_y = obj.path_taper_y,
        path_revolutions = obj.path_revolutions,
        path_skew = obj.path_skew,
        profile_begin = obj.profile_begin,
        profile_end = obj.profile_end,
        profile_hollow = obj.profile_hollow,
        tex = tex_hex,
        extra = extra_b64,
        sculpt_texture = sculpt_texture,
        sculpt_type = sculpt_type,
        owner = obj.owner_id,
        group_id = obj.group_id,
        flags = obj.flags,
        base_mask = obj.owner_mask,
        owner_mask = obj.owner_mask,
        next_owner_mask = obj.owner_mask,
        creation_date = creation_date,
        inv_serial = if task_inv_xml.is_empty() { 0 } else { 1 },
        task_inv = task_inv_xml,
    )
}

fn serialize_task_inv_xml_standalone(
    items: &[(
        Uuid,
        Uuid,
        i32,
        i32,
        String,
        String,
        i32,
        String,
        String,
        String,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
    )],
    prim_id: Uuid,
) -> String {
    if items.is_empty() {
        return String::new();
    }
    let mut xml = String::from("<TaskInventory>");
    for (
        item_id,
        asset_id,
        asset_type,
        inv_type,
        name,
        desc,
        creation_date,
        creator_id,
        owner_id,
        group_id,
        base_perm,
        cur_perm,
        grp_perm,
        every_perm,
        next_perm,
        flags,
    ) in items
    {
        xml.push_str(&format!(
            "<TaskInventoryItem>\
             <AssetID><Guid>{asset_id}</Guid></AssetID>\
             <BasePermissions>{base_perm}</BasePermissions>\
             <CreationDate>{creation_date}</CreationDate>\
             <CreatorID><Guid>{creator_id}</Guid></CreatorID>\
             <Description>{desc}</Description>\
             <EveryonePermissions>{every_perm}</EveryonePermissions>\
             <Flags>{flags}</Flags>\
             <GroupID><Guid>{group_id}</Guid></GroupID>\
             <GroupPermissions>{grp_perm}</GroupPermissions>\
             <InvType>{inv_type}</InvType>\
             <ItemID><Guid>{item_id}</Guid></ItemID>\
             <OldItemID><Guid>00000000-0000-0000-0000-000000000000</Guid></OldItemID>\
             <LastOwnerID><Guid>{owner_id}</Guid></LastOwnerID>\
             <Name>{name}</Name>\
             <NextPermissions>{next_perm}</NextPermissions>\
             <OwnerID><Guid>{owner_id}</Guid></OwnerID>\
             <CurrentPermissions>{cur_perm}</CurrentPermissions>\
             <ParentID><Guid>{prim_id}</Guid></ParentID>\
             <ParentPartID><Guid>{prim_id}</Guid></ParentPartID>\
             <PermsGranter><Guid>00000000-0000-0000-0000-000000000000</Guid></PermsGranter>\
             <PermsMask>0</PermsMask>\
             <Type>{asset_type}</Type>\
             <OwnerChanged>false</OwnerChanged>\
             </TaskInventoryItem>",
            asset_id = asset_id,
            base_perm = base_perm,
            creation_date = creation_date,
            creator_id = creator_id,
            desc = xml_escape(desc),
            every_perm = every_perm,
            flags = flags,
            group_id = group_id,
            grp_perm = grp_perm,
            inv_type = inv_type,
            item_id = item_id,
            owner_id = owner_id,
            name = xml_escape(name),
            next_perm = next_perm,
            cur_perm = cur_perm,
            prim_id = prim_id,
            asset_type = asset_type,
        ));
    }
    xml.push_str("</TaskInventory>");
    xml
}

pub fn serialize_linkset_to_xml_standalone(
    root: &SceneObject,
    children: &[SceneObject],
    task_inv_map: &std::collections::HashMap<Uuid, String>,
    region_handle: u64,
) -> String {
    let root_pos = root.position;
    let empty = String::new();
    let mut xml = String::from("<SceneObjectGroup><RootPart>");
    let root_task_inv = task_inv_map.get(&root.uuid).unwrap_or(&empty);
    xml.push_str(&serialize_part_xml_standalone(
        root,
        root_pos,
        [0.0, 0.0, 0.0],
        0,
        root_task_inv,
        region_handle,
    ));
    xml.push_str("</RootPart>");
    if children.is_empty() {
        xml.push_str("<OtherParts />");
    } else {
        xml.push_str("<OtherParts>");
        for child in children {
            let child_task_inv = task_inv_map.get(&child.uuid).unwrap_or(&empty);
            xml.push_str("<Part>");
            xml.push_str(&serialize_part_xml_standalone(
                child,
                root_pos,
                child.position,
                root.local_id,
                child_task_inv,
                region_handle,
            ));
            xml.push_str("</Part>");
        }
        xml.push_str("</OtherParts>");
    }
    xml.push_str("</SceneObjectGroup>");
    xml
}

pub async fn query_task_inventory_for_prim(
    pool: &sqlx::PgPool,
    prim_uuid: Uuid,
) -> Vec<(
    Uuid,
    Uuid,
    i32,
    i32,
    String,
    String,
    i32,
    String,
    String,
    String,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
)> {
    let rows: Vec<(Uuid, Uuid, i32, i32, String, String, i32, String, String, String, i32, i32, i32, i32, i32, i32)> = sqlx::query_as(
        "SELECT itemid, assetid, assettype, invtype, name, description, \
         creationdate, creatorid, ownerid, groupid, \
         basepermissions, currentpermissions, grouppermissions, everyonepermissions, nextpermissions, flags \
         FROM primitems WHERE primid = $1::uuid"
    )
    .bind(prim_uuid)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    rows
}

#[derive(Debug, Clone, Copy)]
pub enum PrimShape {
    Box,
    Cylinder,
    Sphere,
    Torus,
    Tube,
    Ring,
    Prism,
}
