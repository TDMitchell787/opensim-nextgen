//! OAR (OpenSim Archive) writer
//!
//! Saves region data to an OAR file from the database.
//! Object serialization matches C# SceneObjectSerializer.ToOriginalXmlFormat().

use anyhow::{anyhow, Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::xml_schemas::{create_oar_archive_xml, create_region_settings_xml, OarRegionSettings, OarRegionGeneral, OarGroundTextures, OarTerrainSettings};
use crate::archives::common::{paths, AssetType, SaveStatistics};
use crate::archives::tar_handler::TarGzWriter;

#[derive(Debug, Clone)]
pub struct OarSaveResult {
    pub success: bool,
    pub stats: SaveStatistics,
    pub output_path: std::path::PathBuf,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OarSaveOptions {
    pub region_id: Uuid,
    pub include_assets: bool,
    pub include_terrain: bool,
    pub include_objects: bool,
    pub include_parcels: bool,
    pub object_uuids: Option<Vec<Uuid>>,
}

impl Default for OarSaveOptions {
    fn default() -> Self {
        Self {
            region_id: Uuid::nil(),
            include_assets: true,
            include_terrain: true,
            include_objects: true,
            include_parcels: true,
            object_uuids: None,
        }
    }
}

pub struct OarWriter {
    db_pool: PgPool,
}

impl OarWriter {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn save<P: AsRef<Path>>(
        &self,
        path: P,
        options: OarSaveOptions,
    ) -> Result<OarSaveResult> {
        let start = std::time::Instant::now();
        let mut stats = SaveStatistics::default();
        let mut warnings = Vec::new();
        let mut collected_assets: HashSet<Uuid> = HashSet::new();

        info!("Saving OAR to {:?} for region {}", path.as_ref(), options.region_id);

        let mut archive = TarGzWriter::create(path.as_ref())
            .with_context(|| "Failed to create OAR archive")?;

        let archive_xml = create_oar_archive_xml(1, 0);
        archive.add_file(paths::ARCHIVE_XML, archive_xml.as_bytes())?;

        match self.get_region_settings(&options.region_id).await {
            Ok(settings) => {
                let settings_xml = create_region_settings_xml(&settings);
                archive.add_file(&format!("{}region.xml", paths::SETTINGS_PATH), settings_xml.as_bytes())?;
                info!("Saved region settings");

                for tex in [&settings.ground_textures.texture1, &settings.ground_textures.texture2,
                            &settings.ground_textures.texture3, &settings.ground_textures.texture4] {
                    if !tex.is_empty() {
                        if let Ok(uuid) = Uuid::parse_str(tex) {
                            collected_assets.insert(uuid);
                        }
                    }
                }
            }
            Err(e) => {
                warnings.push(format!("Could not get region settings: {}", e));
            }
        }

        if options.include_terrain {
            match self.get_terrain_data(&options.region_id).await {
                Ok(Some(terrain_data)) => {
                    let terrain_path = format!("{}{}.r32", paths::TERRAINS_PATH, options.region_id);
                    archive.add_file(&terrain_path, &terrain_data)?;
                    stats.terrain_saved = true;
                    info!("Saved terrain ({} bytes)", terrain_data.len());
                }
                Ok(None) => {
                    warnings.push("No terrain data found".into());
                }
                Err(e) => {
                    warnings.push(format!("Failed to get terrain: {}", e));
                }
            }
        }

        if options.include_parcels {
            match self.get_parcels(&options.region_id).await {
                Ok(parcels) => {
                    for parcel in parcels {
                        let parcel_path = format!("{}{}.xml", paths::LANDDATA_PATH, parcel.uuid);
                        let parcel_xml = self.serialize_parcel(&parcel);
                        archive.add_file(&parcel_path, parcel_xml.as_bytes())?;
                        stats.parcels_saved += 1;

                        if let Some(ref snapshot) = parcel.snapshot_uuid {
                            collected_assets.insert(*snapshot);
                        }
                    }
                    info!("Saved {} parcels", stats.parcels_saved);
                }
                Err(e) => {
                    warnings.push(format!("Failed to get parcels: {}", e));
                }
            }
        }

        if options.include_objects {
            match self.get_objects(&options.region_id, &options.object_uuids).await {
                Ok(objects) => {
                    for obj in &objects {
                        let obj_path = format!(
                            "{}{}_{}__{}.xml",
                            paths::OBJECTS_PATH,
                            obj.local_id,
                            sanitize_filename(&obj.name),
                            obj.uuid
                        );
                        let obj_xml = serialize_object(obj);
                        archive.add_file(&obj_path, obj_xml.as_bytes())?;
                        stats.objects_saved += 1;

                        collected_assets.extend(obj.asset_uuids.iter());
                    }
                    info!("Saved {} objects", stats.objects_saved);
                }
                Err(e) => {
                    warn!("Failed to get objects for region {}: {}", options.region_id, e);
                    warnings.push(format!("Failed to get objects: {}", e));
                }
            }
        }

        if options.include_assets {
            info!("Collecting {} assets...", collected_assets.len());
            for asset_id in &collected_assets {
                match self.get_asset_data(asset_id).await {
                    Ok(Some((asset_type, data))) => {
                        let extension = AssetType::from_i32(asset_type).extension();
                        let asset_path = format!(
                            "{}{}{}",
                            paths::ASSETS_PATH,
                            asset_id.to_string().to_lowercase(),
                            extension
                        );
                        archive.add_file(&asset_path, &data)?;
                        stats.assets_saved += 1;
                    }
                    Ok(None) => {
                        debug!("Asset not found: {}", asset_id);
                    }
                    Err(e) => {
                        debug!("Failed to get asset {}: {}", asset_id, e);
                    }
                }
            }
            info!("Saved {} assets", stats.assets_saved);
        }

        archive.finish()?;

        stats.archive_size_bytes = std::fs::metadata(path.as_ref())
            .map(|m| m.len())
            .unwrap_or(0);

        stats.elapsed_ms = start.elapsed().as_millis() as u64;

        Ok(OarSaveResult {
            success: true,
            stats,
            output_path: path.as_ref().to_path_buf(),
            warnings,
        })
    }

    async fn get_region_settings(&self, region_id: &Uuid) -> Result<OarRegionSettings> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"SELECT
                block_terraform, block_fly, allow_damage, restrict_pushing,
                allow_land_resell, allow_land_join_divide, disable_scripts,
                disable_collisions, disable_physics, maturity, block_show_in_search,
                agent_limit, object_bonus,
                terrain_texture_1::text as terrain_texture_1, terrain_texture_2::text as terrain_texture_2,
                terrain_texture_3::text as terrain_texture_3, terrain_texture_4::text as terrain_texture_4,
                elevation_1_nw, elevation_2_ne, elevation_1_se, elevation_2_sw,
                water_height, terrain_raise_limit, terrain_lower_limit,
                use_estate_sun, fixed_sun, sun_position
            FROM regionsettings WHERE regionuuid = $1"#
        )
        .bind(region_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| anyhow!("Region settings not found"))?;

        let block_terraform: i32 = row.get("block_terraform");
        let block_fly: i32 = row.get("block_fly");
        let allow_damage: i32 = row.get("allow_damage");
        let restrict_pushing: i32 = row.get("restrict_pushing");
        let allow_land_resell: i32 = row.get("allow_land_resell");
        let allow_land_join_divide: i32 = row.get("allow_land_join_divide");
        let disable_scripts: i32 = row.get("disable_scripts");
        let disable_collisions: i32 = row.get("disable_collisions");
        let disable_physics: i32 = row.get("disable_physics");
        let block_show_in_search: i32 = row.get("block_show_in_search");
        let use_estate_sun: i32 = row.get("use_estate_sun");
        let fixed_sun: i32 = row.get("fixed_sun");

        Ok(OarRegionSettings {
            general: OarRegionGeneral {
                block_terraform: block_terraform != 0,
                block_fly: block_fly != 0,
                allow_damage: allow_damage != 0,
                restrict_pushing: restrict_pushing != 0,
                allow_land_resell: allow_land_resell != 0,
                allow_land_join_divide: allow_land_join_divide != 0,
                disable_scripts: disable_scripts != 0,
                disable_collisions: disable_collisions != 0,
                disable_physics: disable_physics != 0,
                maturity_rating: row.get("maturity"),
                block_land_show_in_search: block_show_in_search != 0,
                agent_limit: row.get("agent_limit"),
                object_bonus: row.get::<f32, _>("object_bonus") as f64,
            },
            ground_textures: OarGroundTextures {
                texture1: row.try_get("terrain_texture_1").unwrap_or_default(),
                texture2: row.try_get("terrain_texture_2").unwrap_or_default(),
                texture3: row.try_get("terrain_texture_3").unwrap_or_default(),
                texture4: row.try_get("terrain_texture_4").unwrap_or_default(),
                elevation_low_sw: row.try_get::<f32, _>("elevation_2_sw").unwrap_or(10.0) as f64,
                elevation_low_nw: row.try_get::<f32, _>("elevation_1_nw").unwrap_or(10.0) as f64,
                elevation_low_se: row.try_get::<f32, _>("elevation_1_se").unwrap_or(10.0) as f64,
                elevation_low_ne: row.try_get::<f32, _>("elevation_2_ne").unwrap_or(10.0) as f64,
                elevation_high_sw: 0.0,
                elevation_high_nw: 0.0,
                elevation_high_se: 0.0,
                elevation_high_ne: 0.0,
            },
            terrain: OarTerrainSettings {
                water_height: row.get::<f32, _>("water_height") as f64,
                terrain_raise_limit: row.get::<f32, _>("terrain_raise_limit") as f64,
                terrain_lower_limit: row.get::<f32, _>("terrain_lower_limit") as f64,
                use_estate_sun: use_estate_sun != 0,
                fixed_sun: fixed_sun != 0,
                sun_position: row.get::<f32, _>("sun_position") as f64,
            },
        })
    }

    async fn get_terrain_data(&self, region_id: &Uuid) -> Result<Option<Vec<u8>>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT heightfield FROM bakedterrain WHERE regionuuid = $1 ORDER BY revision DESC LIMIT 1"
        )
        .bind(region_id.to_string())
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.map(|(data,)| data))
    }

    async fn get_parcels(&self, region_id: &Uuid) -> Result<Vec<ParcelData>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"SELECT
                uuid, locallandid, name, description, owneruuid, groupuuid,
                isgroupowned, area, auctionid, category, claimdate, claimprice,
                landflags, landingtype, snapshotuuid
            FROM land WHERE regionuuid = $1"#
        )
        .bind(region_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            let is_group_owned_int: i32 = row.try_get("isgroupowned").unwrap_or(0);
            ParcelData {
                uuid: row.get("uuid"),
                local_id: row.get("locallandid"),
                name: row.get("name"),
                description: row.try_get("description").ok(),
                owner_uuid: row.get("owneruuid"),
                group_uuid: row.try_get("groupuuid").unwrap_or_default(),
                is_group_owned: is_group_owned_int != 0,
                area: row.get("area"),
                auction_id: row.try_get("auctionid").unwrap_or(0),
                category: row.try_get("category").unwrap_or(0),
                claim_date: row.try_get::<i32, _>("claimdate").unwrap_or(0) as i64,
                claim_price: row.try_get("claimprice").unwrap_or(0),
                flags: row.try_get::<i32, _>("landflags").unwrap_or(0) as u32,
                landing_type: row.try_get("landingtype").unwrap_or(0),
                snapshot_uuid: row.try_get("snapshotuuid").ok(),
            }
        }).collect())
    }

    fn serialize_parcel(&self, parcel: &ParcelData) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<LandData>
    <GlobalID>{}</GlobalID>
    <LocalID>{}</LocalID>
    <Name>{}</Name>
    <Description>{}</Description>
    <OwnerID>{}</OwnerID>
    <GroupID>{}</GroupID>
    <IsGroupOwned>{}</IsGroupOwned>
    <Area>{}</Area>
    <AuctionID>{}</AuctionID>
    <Category>{}</Category>
    <ClaimDate>{}</ClaimDate>
    <ClaimPrice>{}</ClaimPrice>
    <Flags>{}</Flags>
    <LandingType>{}</LandingType>
</LandData>"#,
            parcel.uuid,
            parcel.local_id,
            xml_escape(&parcel.name),
            xml_escape(parcel.description.as_deref().unwrap_or("")),
            parcel.owner_uuid,
            parcel.group_uuid,
            parcel.is_group_owned,
            parcel.area,
            parcel.auction_id,
            parcel.category,
            parcel.claim_date,
            parcel.claim_price,
            parcel.flags,
            parcel.landing_type,
        )
    }

    async fn get_objects(&self, region_id: &Uuid, filter_uuids: &Option<Vec<Uuid>>) -> Result<Vec<ObjectData>> {
        use sqlx::Row;

        let base_query = r#"SELECT p.uuid, p.creatorid, p.ownerid, p.groupid, p.lastownerid, p.scenegroupid,
                p.name, p.description, p.text, p.sitname, p.touchname,
                p.objectflags, p.ownermask, p.nextownermask, p.groupmask, p.everyonemask, p.basemask,
                p.positionx, p.positiony, p.positionz,
                p.grouppositionx, p.grouppositiony, p.grouppositionz,
                p.velocityx, p.velocityy, p.velocityz,
                p.angularvelocityx, p.angularvelocityy, p.angularvelocityz,
                p.accelerationx, p.accelerationy, p.accelerationz,
                p.rotationx, p.rotationy, p.rotationz, p.rotationw,
                p.sittargetoffsetx, p.sittargetoffsety, p.sittargetoffsetz,
                p.sittargetorientx, p.sittargetorienty, p.sittargetorientz, p.sittargetorientw,
                p.creationdate, p.material, p.linknumber, p.passcollisions,
                COALESCE(p.saleprice, 0) as saleprice, COALESCE(p.saletype, 0) as saletype,
                COALESCE(p.clickaction, 0) as clickaction,
                ps.shape as ps_shape, ps.scalex, ps.scaley, ps.scalez, ps.pcode,
                ps.pathbegin, ps.pathend, ps.pathscalex, ps.pathscaley,
                ps.pathshearx, ps.pathsheary, ps.pathskew, ps.pathcurve,
                ps.pathradiusoffset, ps.pathrevolutions, ps.pathtaperx, ps.pathtapery,
                ps.pathtwist, ps.pathtwistbegin,
                ps.profilebegin, ps.profileend, ps.profilecurve, ps.profilehollow,
                ps.texture, ps.extraparams, ps.state as ps_state, ps.media
            FROM prims p
            LEFT JOIN primshapes ps ON p.uuid = ps.uuid"#;

        let rows = if let Some(uuids) = filter_uuids {
            if uuids.is_empty() {
                info!("No object UUIDs to export");
                return Ok(Vec::new());
            }
            let placeholders: Vec<String> = uuids.iter().enumerate()
                .map(|(i, _)| format!("${}", i + 2))
                .collect();
            let query = format!(
                "{} WHERE p.regionuuid = $1 AND p.scenegroupid IN ({}) ORDER BY p.scenegroupid, p.linknumber",
                base_query,
                placeholders.join(", ")
            );
            info!("Querying {} specific objects for region {}", uuids.len(), region_id);
            let mut q = sqlx::query(&query).bind(region_id);
            for uuid in uuids {
                q = q.bind(uuid);
            }
            q.fetch_all(&self.db_pool).await?
        } else {
            let query = format!(
                "{} WHERE p.regionuuid = $1 ORDER BY p.scenegroupid, p.linknumber",
                base_query
            );
            info!("Querying all objects for region {}", region_id);
            sqlx::query(&query)
                .bind(region_id)
                .fetch_all(&self.db_pool)
                .await?
        };

        let mut groups: HashMap<Uuid, Vec<PrimData>> = HashMap::new();

        for row in &rows {
            let prim_uuid: Uuid = row.get("uuid");
            let scene_group_id: Uuid = row.get("scenegroupid");

            let texture_bytes: Vec<u8> = row.try_get("texture").unwrap_or_default();
            let extra_params_bytes: Vec<u8> = row.try_get("extraparams").unwrap_or_default();

            let shape = ShapeData {
                profile_curve: row.try_get("profilecurve").unwrap_or(1),
                path_curve: row.try_get("pathcurve").unwrap_or(16),
                profile_begin: row.try_get("profilebegin").unwrap_or(0),
                profile_end: row.try_get("profileend").unwrap_or(0),
                profile_hollow: row.try_get("profilehollow").unwrap_or(0),
                path_begin: row.try_get("pathbegin").unwrap_or(0),
                path_end: row.try_get("pathend").unwrap_or(0),
                path_scale_x: row.try_get("pathscalex").unwrap_or(100),
                path_scale_y: row.try_get("pathscaley").unwrap_or(100),
                path_shear_x: row.try_get("pathshearx").unwrap_or(0),
                path_shear_y: row.try_get("pathsheary").unwrap_or(0),
                path_twist: row.try_get("pathtwist").unwrap_or(0),
                path_twist_begin: row.try_get("pathtwistbegin").unwrap_or(0),
                path_radius_offset: row.try_get("pathradiusoffset").unwrap_or(0),
                path_taper_x: row.try_get("pathtaperx").unwrap_or(0),
                path_taper_y: row.try_get("pathtapery").unwrap_or(0),
                path_revolutions: row.try_get("pathrevolutions").unwrap_or(0),
                path_skew: row.try_get("pathskew").unwrap_or(0),
                pcode: row.try_get("pcode").unwrap_or(9),
                state: row.try_get("ps_state").unwrap_or(0),
                scale_x: row.try_get("scalex").unwrap_or(1.0),
                scale_y: row.try_get("scaley").unwrap_or(1.0),
                scale_z: row.try_get("scalez").unwrap_or(1.0),
                texture: texture_bytes,
                extra_params: extra_params_bytes,
                media: row.try_get("media").ok(),
            };

            let prim = PrimData {
                uuid: prim_uuid,
                creator_id: row.try_get::<String, _>("creatorid").unwrap_or_default(),
                owner_id: row.get("ownerid"),
                group_id: row.try_get("groupid").unwrap_or(Uuid::nil()),
                last_owner_id: row.try_get("lastownerid").unwrap_or(Uuid::nil()),
                name: row.try_get("name").unwrap_or_default(),
                description: row.try_get("description").unwrap_or_default(),
                text: row.try_get("text").unwrap_or_default(),
                sit_name: row.try_get("sitname").unwrap_or_default(),
                touch_name: row.try_get("touchname").unwrap_or_default(),
                object_flags: row.try_get("objectflags").unwrap_or(0),
                owner_mask: row.try_get("ownermask").unwrap_or(0x7FFFFFFF),
                next_owner_mask: row.try_get("nextownermask").unwrap_or(0x82000),
                group_mask: row.try_get("groupmask").unwrap_or(0),
                everyone_mask: row.try_get("everyonemask").unwrap_or(0),
                base_mask: row.try_get("basemask").unwrap_or(0x7FFFFFFF),
                position_x: row.try_get("positionx").unwrap_or(128.0),
                position_y: row.try_get("positiony").unwrap_or(128.0),
                position_z: row.try_get("positionz").unwrap_or(25.0),
                group_position_x: row.try_get("grouppositionx").unwrap_or(128.0),
                group_position_y: row.try_get("grouppositiony").unwrap_or(128.0),
                group_position_z: row.try_get("grouppositionz").unwrap_or(25.0),
                velocity_x: row.try_get("velocityx").unwrap_or(0.0),
                velocity_y: row.try_get("velocityy").unwrap_or(0.0),
                velocity_z: row.try_get("velocityz").unwrap_or(0.0),
                angular_velocity_x: row.try_get("angularvelocityx").unwrap_or(0.0),
                angular_velocity_y: row.try_get("angularvelocityy").unwrap_or(0.0),
                angular_velocity_z: row.try_get("angularvelocityz").unwrap_or(0.0),
                acceleration_x: row.try_get("accelerationx").unwrap_or(0.0),
                acceleration_y: row.try_get("accelerationy").unwrap_or(0.0),
                acceleration_z: row.try_get("accelerationz").unwrap_or(0.0),
                rotation_x: row.try_get("rotationx").unwrap_or(0.0),
                rotation_y: row.try_get("rotationy").unwrap_or(0.0),
                rotation_z: row.try_get("rotationz").unwrap_or(0.0),
                rotation_w: row.try_get("rotationw").unwrap_or(1.0),
                sit_target_offset_x: row.try_get("sittargetoffsetx").unwrap_or(0.0),
                sit_target_offset_y: row.try_get("sittargetoffsety").unwrap_or(0.0),
                sit_target_offset_z: row.try_get("sittargetoffsetz").unwrap_or(0.0),
                sit_target_orient_x: row.try_get("sittargetorientx").unwrap_or(0.0),
                sit_target_orient_y: row.try_get("sittargetorienty").unwrap_or(0.0),
                sit_target_orient_z: row.try_get("sittargetorientz").unwrap_or(0.0),
                sit_target_orient_w: row.try_get("sittargetorientw").unwrap_or(1.0),
                creation_date: row.try_get("creationdate").unwrap_or(0),
                material: row.try_get("material").unwrap_or(3),
                link_number: row.try_get("linknumber").unwrap_or(0),
                pass_collisions: row.try_get::<i32, _>("passcollisions").unwrap_or(0) != 0,
                sale_price: row.try_get("saleprice").unwrap_or(0),
                sale_type: row.try_get("saletype").unwrap_or(0),
                click_action: row.try_get("clickaction").unwrap_or(0),
                shape,
                items: Vec::new(),
            };

            groups.entry(scene_group_id).or_default().push(prim);
        }

        let all_prim_uuids: Vec<Uuid> = groups.values().flat_map(|v| v.iter().map(|p| p.uuid)).collect();
        let task_items = self.get_task_items(&all_prim_uuids).await?;

        let mut objects = Vec::new();
        for (scene_group_id, mut prims) in groups {
            prims.sort_by_key(|p| p.link_number);

            for prim in &mut prims {
                if let Some(items) = task_items.get(&prim.uuid) {
                    prim.items = items.clone();
                }
            }

            let root = match prims.iter().position(|p| p.link_number <= 1) {
                Some(idx) => prims.remove(idx),
                None => {
                    if prims.is_empty() { continue; }
                    prims.remove(0)
                }
            };

            let mut asset_uuids = Vec::new();
            let all_prims_iter = std::iter::once(&root).chain(prims.iter());
            for p in all_prims_iter {
                asset_uuids.extend(extract_texture_uuids(&p.shape.texture));
                asset_uuids.extend(extract_sculpt_uuid(&p.shape.extra_params));
                for item in &p.items {
                    if !item.asset_id.is_nil() {
                        asset_uuids.push(item.asset_id);
                    }
                }
            }
            asset_uuids.sort();
            asset_uuids.dedup();

            objects.push(ObjectData {
                uuid: scene_group_id,
                local_id: root.link_number as u32,
                name: root.name.clone(),
                root,
                children: prims,
                asset_uuids,
            });
        }

        info!("Loaded {} scene groups from DB", objects.len());
        Ok(objects)
    }

    async fn get_task_items(&self, prim_uuids: &[Uuid]) -> Result<HashMap<Uuid, Vec<TaskItemData>>> {
        use sqlx::Row;

        if prim_uuids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query(
            r#"SELECT itemid, primid, assetid, invtype, assettype, name, description,
                creationdate, creatorid, ownerid, lastownerid, groupid,
                nextpermissions, currentpermissions, basepermissions,
                everyonepermissions, grouppermissions, flags
            FROM primitems WHERE primid = ANY($1)"#
        )
        .bind(prim_uuids)
        .fetch_all(&self.db_pool)
        .await?;

        let mut result: HashMap<Uuid, Vec<TaskItemData>> = HashMap::new();
        for row in &rows {
            let prim_id: Uuid = row.get("primid");
            let item = TaskItemData {
                item_id: row.get("itemid"),
                asset_id: row.get("assetid"),
                name: row.try_get("name").unwrap_or_default(),
                description: row.try_get("description").unwrap_or_default(),
                inv_type: row.try_get("invtype").unwrap_or(0),
                asset_type: row.try_get("assettype").unwrap_or(0),
                creator_id: row.try_get::<String, _>("creatorid").unwrap_or_default(),
                owner_id: row.try_get::<String, _>("ownerid").unwrap_or_default(),
                last_owner_id: row.try_get::<String, _>("lastownerid").unwrap_or_default(),
                group_id: row.try_get::<String, _>("groupid").unwrap_or_default(),
                base_permissions: row.try_get("basepermissions").unwrap_or(0x7FFFFFFF),
                current_permissions: row.try_get("currentpermissions").unwrap_or(0x7FFFFFFF),
                next_permissions: row.try_get("nextpermissions").unwrap_or(0x82000),
                everyone_permissions: row.try_get("everyonepermissions").unwrap_or(0),
                group_permissions: row.try_get("grouppermissions").unwrap_or(0),
                flags: row.try_get("flags").unwrap_or(0),
                creation_date: row.try_get("creationdate").unwrap_or(0),
            };
            result.entry(prim_id).or_default().push(item);
        }
        Ok(result)
    }

    async fn get_asset_data(&self, asset_id: &Uuid) -> Result<Option<(i32, Vec<u8>)>> {
        let row: Option<(i64, Vec<u8>)> = sqlx::query_as(
            "SELECT assettype, data FROM assets WHERE id = $1"
        )
        .bind(asset_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.map(|(t, d)| (t as i32, d)))
    }
}

#[derive(Debug)]
struct ParcelData {
    uuid: Uuid,
    local_id: i32,
    name: String,
    description: Option<String>,
    owner_uuid: Uuid,
    group_uuid: Uuid,
    is_group_owned: bool,
    area: i32,
    auction_id: i32,
    category: i32,
    claim_date: i64,
    claim_price: i32,
    flags: u32,
    landing_type: i32,
    snapshot_uuid: Option<Uuid>,
}

#[derive(Debug, Clone)]
struct PrimData {
    uuid: Uuid,
    creator_id: String,
    owner_id: Uuid,
    group_id: Uuid,
    last_owner_id: Uuid,
    name: String,
    description: String,
    text: String,
    sit_name: String,
    touch_name: String,
    object_flags: i32,
    owner_mask: i32,
    next_owner_mask: i32,
    group_mask: i32,
    everyone_mask: i32,
    base_mask: i32,
    position_x: f32,
    position_y: f32,
    position_z: f32,
    group_position_x: f32,
    group_position_y: f32,
    group_position_z: f32,
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
    angular_velocity_x: f32,
    angular_velocity_y: f32,
    angular_velocity_z: f32,
    acceleration_x: f32,
    acceleration_y: f32,
    acceleration_z: f32,
    rotation_x: f32,
    rotation_y: f32,
    rotation_z: f32,
    rotation_w: f32,
    sit_target_offset_x: f32,
    sit_target_offset_y: f32,
    sit_target_offset_z: f32,
    sit_target_orient_x: f32,
    sit_target_orient_y: f32,
    sit_target_orient_z: f32,
    sit_target_orient_w: f32,
    creation_date: i32,
    material: i32,
    link_number: i32,
    pass_collisions: bool,
    sale_price: i32,
    sale_type: i32,
    click_action: i32,
    shape: ShapeData,
    items: Vec<TaskItemData>,
}

#[derive(Debug, Clone)]
struct ShapeData {
    profile_curve: i32,
    path_curve: i32,
    profile_begin: i32,
    profile_end: i32,
    profile_hollow: i32,
    path_begin: i32,
    path_end: i32,
    path_scale_x: i32,
    path_scale_y: i32,
    path_shear_x: i32,
    path_shear_y: i32,
    path_twist: i32,
    path_twist_begin: i32,
    path_radius_offset: i32,
    path_taper_x: i32,
    path_taper_y: i32,
    path_revolutions: i32,
    path_skew: i32,
    pcode: i32,
    state: i32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    texture: Vec<u8>,
    extra_params: Vec<u8>,
    media: Option<String>,
}

#[derive(Debug, Clone)]
struct TaskItemData {
    item_id: Uuid,
    asset_id: Uuid,
    name: String,
    description: String,
    inv_type: i32,
    asset_type: i32,
    creator_id: String,
    owner_id: String,
    last_owner_id: String,
    group_id: String,
    base_permissions: i32,
    current_permissions: i32,
    next_permissions: i32,
    everyone_permissions: i32,
    group_permissions: i32,
    flags: i32,
    creation_date: i32,
}

#[derive(Debug)]
struct ObjectData {
    uuid: Uuid,
    local_id: u32,
    name: String,
    root: PrimData,
    children: Vec<PrimData>,
    asset_uuids: Vec<Uuid>,
}

fn extract_texture_uuids(te_bytes: &[u8]) -> Vec<Uuid> {
    let mut uuids = Vec::new();
    if te_bytes.len() < 16 { return uuids; }

    if let Ok(default_uuid) = Uuid::from_slice(&te_bytes[0..16]) {
        if !default_uuid.is_nil() {
            uuids.push(default_uuid);
        }
    }

    let mut offset = 16;
    while offset + 17 <= te_bytes.len() {
        let face_bits = te_bytes[offset];
        offset += 1;
        if face_bits == 0 { break; }
        if offset + 16 > te_bytes.len() { break; }
        if let Ok(face_uuid) = Uuid::from_slice(&te_bytes[offset..offset + 16]) {
            if !face_uuid.is_nil() && !uuids.contains(&face_uuid) {
                uuids.push(face_uuid);
            }
        }
        offset += 16;
    }

    uuids
}

fn extract_sculpt_uuid(extra_params: &[u8]) -> Vec<Uuid> {
    let mut uuids = Vec::new();
    if extra_params.len() < 2 { return uuids; }

    let mut offset = 0;
    while offset < extra_params.len() {
        if offset + 6 > extra_params.len() { break; }
        let param_type = u16::from_le_bytes([extra_params[offset], extra_params[offset + 1]]);
        let param_len = u32::from_le_bytes([
            extra_params[offset + 2], extra_params[offset + 3],
            extra_params[offset + 4], extra_params[offset + 5],
        ]) as usize;
        offset += 6;
        if param_type == 0x0030 && param_len >= 17 && offset + 17 <= extra_params.len() {
            if let Ok(sculpt_uuid) = Uuid::from_slice(&extra_params[offset..offset + 16]) {
                if !sculpt_uuid.is_nil() {
                    uuids.push(sculpt_uuid);
                }
            }
        }
        offset += param_len;
    }

    uuids
}

fn profile_shape_name(profile_curve: i32) -> &'static str {
    match profile_curve & 0x0F {
        0 => "Circle",
        1 => "Square",
        2 => "IsometricTriangle",
        3 => "EquilateralTriangle",
        4 => "RightTriangle",
        5 => "HalfCircle",
        _ => "Square",
    }
}

fn hollow_shape_name(profile_curve: i32) -> &'static str {
    match (profile_curve >> 4) & 0x0F {
        0 => "Same",
        16 | 1 => "Circle",
        32 | 2 => "Square",
        48 | 3 => "Triangle",
        _ => "Same",
    }
}

fn serialize_object(obj: &ObjectData) -> String {
    let mut xml = String::with_capacity(8192);
    xml.push_str("<SceneObjectGroup>\n");

    xml.push_str("  <RootPart>\n");
    serialize_prim(&mut xml, &obj.root, true);
    xml.push_str("  </RootPart>\n");

    xml.push_str("  <OtherParts>\n");
    for child in &obj.children {
        xml.push_str("    <Part>\n");
        serialize_prim(&mut xml, child, false);
        xml.push_str("    </Part>\n");
    }
    xml.push_str("  </OtherParts>\n");

    xml.push_str("</SceneObjectGroup>");
    xml
}

fn serialize_prim(xml: &mut String, prim: &PrimData, is_root: bool) {
    let indent = if is_root { "    " } else { "      " };
    let i2 = format!("{}  ", indent);

    xml.push_str(&format!("{}<SceneObjectPart xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\">\n", indent));

    write_elem(xml, &i2, "AllowedDrop", "false");
    write_uuid_field(xml, &i2, "CreatorID", &uuid_from_str(&prim.creator_id));
    write_uuid_field(xml, &i2, "FolderID", &Uuid::nil());
    write_elem(xml, &i2, "InventorySerial", &prim.items.len().to_string());

    if !prim.items.is_empty() {
        serialize_task_inventory(xml, &i2, &prim.items, &prim.uuid);
    }

    write_uuid_field(xml, &i2, "UUID", &prim.uuid);
    write_elem(xml, &i2, "LocalId", &(prim.link_number.max(1) as u32).to_string());
    write_elem(xml, &i2, "Name", &xml_escape(&prim.name));
    write_elem(xml, &i2, "Material", &prim.material.to_string());
    write_elem(xml, &i2, "PassTouches", "false");
    write_elem(xml, &i2, "PassCollisions", &prim.pass_collisions.to_string());
    write_elem(xml, &i2, "RegionHandle", "0");
    write_elem(xml, &i2, "ScriptAccessPin", "0");

    write_vector3(xml, &i2, "GroupPosition", prim.group_position_x, prim.group_position_y, prim.group_position_z);

    if is_root {
        write_vector3(xml, &i2, "OffsetPosition", 0.0, 0.0, 0.0);
    } else {
        write_vector3(xml, &i2, "OffsetPosition", prim.position_x, prim.position_y, prim.position_z);
    }

    write_quat(xml, &i2, "RotationOffset", prim.rotation_x, prim.rotation_y, prim.rotation_z, prim.rotation_w);
    write_vector3(xml, &i2, "Velocity", prim.velocity_x, prim.velocity_y, prim.velocity_z);
    write_vector3(xml, &i2, "AngularVelocity", prim.angular_velocity_x, prim.angular_velocity_y, prim.angular_velocity_z);
    write_vector3(xml, &i2, "Acceleration", prim.acceleration_x, prim.acceleration_y, prim.acceleration_z);
    write_elem(xml, &i2, "Description", &xml_escape(&prim.description));
    write_elem(xml, &i2, "Text", &xml_escape(&prim.text));
    write_elem(xml, &i2, "SitName", &xml_escape(&prim.sit_name));
    write_elem(xml, &i2, "TouchName", &xml_escape(&prim.touch_name));
    write_elem(xml, &i2, "LinkNum", &prim.link_number.to_string());
    write_elem(xml, &i2, "ClickAction", &prim.click_action.to_string());

    serialize_shape(xml, &i2, &prim.shape);

    write_vector3(xml, &i2, "Scale", prim.shape.scale_x, prim.shape.scale_y, prim.shape.scale_z);
    write_quat(xml, &i2, "SitTargetOrientation", prim.sit_target_orient_x, prim.sit_target_orient_y, prim.sit_target_orient_z, prim.sit_target_orient_w);
    write_vector3(xml, &i2, "SitTargetPosition", prim.sit_target_offset_x, prim.sit_target_offset_y, prim.sit_target_offset_z);
    write_vector3(xml, &i2, "SitTargetPositionLL", prim.sit_target_offset_x, prim.sit_target_offset_y, prim.sit_target_offset_z);
    write_quat(xml, &i2, "SitTargetOrientationLL", prim.sit_target_orient_x, prim.sit_target_orient_y, prim.sit_target_orient_z, prim.sit_target_orient_w);

    write_elem(xml, &i2, "ParentID", "0");
    write_elem(xml, &i2, "CreationDate", &prim.creation_date.to_string());
    write_elem(xml, &i2, "Category", "0");
    write_elem(xml, &i2, "SalePrice", &prim.sale_price.to_string());
    write_elem(xml, &i2, "ObjectSaleType", &prim.sale_type.to_string());
    write_elem(xml, &i2, "OwnershipCost", "0");

    write_uuid_field(xml, &i2, "GroupID", &prim.group_id);
    write_uuid_field(xml, &i2, "OwnerID", &prim.owner_id);
    write_uuid_field(xml, &i2, "LastOwnerID", &prim.last_owner_id);
    write_uuid_field(xml, &i2, "RezzerID", &Uuid::nil());

    write_elem(xml, &i2, "BaseMask", &(prim.base_mask as u32).to_string());
    write_elem(xml, &i2, "OwnerMask", &(prim.owner_mask as u32).to_string());
    write_elem(xml, &i2, "GroupMask", &(prim.group_mask as u32).to_string());
    write_elem(xml, &i2, "EveryoneMask", &(prim.everyone_mask as u32).to_string());
    write_elem(xml, &i2, "NextOwnerMask", &(prim.next_owner_mask as u32).to_string());

    let flags_str = (prim.object_flags as u32).to_string();
    write_elem(xml, &i2, "Flags", &flags_str);

    write_uuid_field(xml, &i2, "CollisionSound", &Uuid::nil());
    write_elem(xml, &i2, "CollisionSoundVolume", "0");

    xml.push_str(&format!("{}</SceneObjectPart>\n", indent));
}

fn serialize_shape(xml: &mut String, indent: &str, shape: &ShapeData) {
    let i2 = format!("{}  ", indent);

    xml.push_str(&format!("{}<Shape>\n", indent));

    write_elem(xml, &i2, "ProfileCurve", &shape.profile_curve.to_string());

    let te_b64 = BASE64.encode(&shape.texture);
    write_elem(xml, &i2, "TextureEntry", &te_b64);

    let ep_b64 = BASE64.encode(&shape.extra_params);
    write_elem(xml, &i2, "ExtraParams", &ep_b64);

    write_elem(xml, &i2, "PathBegin", &shape.path_begin.to_string());
    write_elem(xml, &i2, "PathCurve", &shape.path_curve.to_string());
    write_elem(xml, &i2, "PathEnd", &shape.path_end.to_string());
    write_elem(xml, &i2, "PathRadiusOffset", &shape.path_radius_offset.to_string());
    write_elem(xml, &i2, "PathRevolutions", &shape.path_revolutions.to_string());
    write_elem(xml, &i2, "PathScaleX", &shape.path_scale_x.to_string());
    write_elem(xml, &i2, "PathScaleY", &shape.path_scale_y.to_string());
    write_elem(xml, &i2, "PathShearX", &shape.path_shear_x.to_string());
    write_elem(xml, &i2, "PathShearY", &shape.path_shear_y.to_string());
    write_elem(xml, &i2, "PathSkew", &shape.path_skew.to_string());
    write_elem(xml, &i2, "PathTaperX", &shape.path_taper_x.to_string());
    write_elem(xml, &i2, "PathTaperY", &shape.path_taper_y.to_string());
    write_elem(xml, &i2, "PathTwist", &shape.path_twist.to_string());
    write_elem(xml, &i2, "PathTwistBegin", &shape.path_twist_begin.to_string());
    write_elem(xml, &i2, "PCode", &shape.pcode.to_string());
    write_elem(xml, &i2, "ProfileBegin", &shape.profile_begin.to_string());
    write_elem(xml, &i2, "ProfileEnd", &shape.profile_end.to_string());
    write_elem(xml, &i2, "ProfileHollow", &shape.profile_hollow.to_string());

    write_vector3(xml, &i2, "Scale", shape.scale_x, shape.scale_y, shape.scale_z);

    write_elem(xml, &i2, "State", &shape.state.to_string());
    write_elem(xml, &i2, "LastAttachPoint", "0");

    write_elem(xml, &i2, "ProfileShape", profile_shape_name(shape.profile_curve));
    write_elem(xml, &i2, "HollowShape", hollow_shape_name(shape.profile_curve));

    write_elem(xml, &i2, "SculptType", "0");
    write_elem(xml, &i2, "FlexiEntry", "false");
    write_elem(xml, &i2, "LightEntry", "false");
    write_elem(xml, &i2, "SculptEntry", "false");

    if let Some(ref media) = shape.media {
        write_elem(xml, &i2, "Media", &xml_escape(media));
    }

    xml.push_str(&format!("{}</Shape>\n", indent));
}

fn serialize_task_inventory(xml: &mut String, indent: &str, items: &[TaskItemData], parent_prim_id: &Uuid) {
    let i2 = format!("{}  ", indent);
    let i3 = format!("{}    ", indent);

    xml.push_str(&format!("{}<TaskInventory>\n", indent));
    for item in items {
        xml.push_str(&format!("{}<TaskInventoryItem>\n", i2));

        write_uuid_field(xml, &i3, "AssetID", &item.asset_id);
        write_elem(xml, &i3, "BasePermissions", &(item.base_permissions as u32).to_string());
        write_elem(xml, &i3, "CreationDate", &item.creation_date.to_string());
        write_uuid_field(xml, &i3, "CreatorID", &uuid_from_str(&item.creator_id));
        write_elem(xml, &i3, "Description", &xml_escape(&item.description));
        write_elem(xml, &i3, "EveryonePermissions", &(item.everyone_permissions as u32).to_string());
        write_elem(xml, &i3, "Flags", &(item.flags as u32).to_string());
        write_uuid_field(xml, &i3, "GroupID", &uuid_from_str(&item.group_id));
        write_elem(xml, &i3, "GroupPermissions", &(item.group_permissions as u32).to_string());
        write_elem(xml, &i3, "InvType", &item.inv_type.to_string());
        write_uuid_field(xml, &i3, "ItemID", &item.item_id);
        write_uuid_field(xml, &i3, "OldItemID", &Uuid::nil());
        write_uuid_field(xml, &i3, "LastOwnerID", &uuid_from_str(&item.last_owner_id));
        write_elem(xml, &i3, "Name", &xml_escape(&item.name));
        write_elem(xml, &i3, "NextPermissions", &(item.next_permissions as u32).to_string());
        write_uuid_field(xml, &i3, "OwnerID", &uuid_from_str(&item.owner_id));
        write_elem(xml, &i3, "CurrentPermissions", &(item.current_permissions as u32).to_string());
        write_uuid_field(xml, &i3, "ParentID", &Uuid::nil());
        write_uuid_field(xml, &i3, "ParentPartID", parent_prim_id);
        write_uuid_field(xml, &i3, "PermsGranter", &Uuid::nil());
        write_elem(xml, &i3, "PermsMask", "0");
        write_elem(xml, &i3, "Type", &item.asset_type.to_string());
        write_elem(xml, &i3, "OwnerChanged", "false");

        xml.push_str(&format!("{}</TaskInventoryItem>\n", i2));
    }
    xml.push_str(&format!("{}</TaskInventory>\n", indent));
}

fn write_elem(xml: &mut String, indent: &str, name: &str, value: &str) {
    xml.push_str(indent);
    xml.push('<');
    xml.push_str(name);
    xml.push('>');
    xml.push_str(value);
    xml.push_str("</");
    xml.push_str(name);
    xml.push_str(">\n");
}

fn write_uuid_field(xml: &mut String, indent: &str, name: &str, uuid: &Uuid) {
    let i2 = format!("{}  ", indent);
    xml.push_str(indent);
    xml.push('<');
    xml.push_str(name);
    xml.push_str(">\n");
    write_elem(xml, &i2, "UUID", &uuid.to_string());
    xml.push_str(indent);
    xml.push_str("</");
    xml.push_str(name);
    xml.push_str(">\n");
}

fn write_vector3(xml: &mut String, indent: &str, name: &str, x: f32, y: f32, z: f32) {
    let i2 = format!("{}  ", indent);
    xml.push_str(indent);
    xml.push('<');
    xml.push_str(name);
    xml.push_str(">\n");
    write_elem(xml, &i2, "X", &x.to_string());
    write_elem(xml, &i2, "Y", &y.to_string());
    write_elem(xml, &i2, "Z", &z.to_string());
    xml.push_str(indent);
    xml.push_str("</");
    xml.push_str(name);
    xml.push_str(">\n");
}

fn write_quat(xml: &mut String, indent: &str, name: &str, x: f32, y: f32, z: f32, w: f32) {
    let i2 = format!("{}  ", indent);
    xml.push_str(indent);
    xml.push('<');
    xml.push_str(name);
    xml.push_str(">\n");
    write_elem(xml, &i2, "X", &x.to_string());
    write_elem(xml, &i2, "Y", &y.to_string());
    write_elem(xml, &i2, "Z", &z.to_string());
    write_elem(xml, &i2, "W", &w.to_string());
    xml.push_str(indent);
    xml.push_str("</");
    xml.push_str(name);
    xml.push_str(">\n");
}

fn uuid_from_str(s: &str) -> Uuid {
    Uuid::parse_str(s).unwrap_or(Uuid::nil())
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | ' ' => '_',
            _ => c,
        })
        .take(50)
        .collect()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
