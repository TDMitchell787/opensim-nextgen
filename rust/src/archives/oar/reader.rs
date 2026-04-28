//! OAR (OpenSim Archive) reader
//!
//! Loads region data from an OAR file into the database.

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sqlx::PgPool;
use std::path::Path;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::xml_schemas::{
    OarArchiveXml, OarLandData, OarRegionSettings, OarSceneObjectGroup, OarSceneObjectPart,
};
use crate::archives::common::{extract_asset_uuid_from_path, paths, LoadStatistics};
use crate::archives::tar_handler::TarGzReader;

/// Result of loading an OAR
#[derive(Debug, Clone)]
pub struct OarLoadResult {
    pub success: bool,
    pub stats: LoadStatistics,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// OAR load options
#[derive(Debug, Clone)]
pub struct OarLoadOptions {
    pub region_id: Uuid,
    pub merge: bool,
    pub load_terrain: bool,
    pub load_objects: bool,
    pub load_parcels: bool,
    pub displacement: (f32, f32, f32),
    pub rotation_degrees: f32,
    pub force_terrain: bool,
    pub force_parcels: bool,
    pub default_user_id: Option<Uuid>,
}

impl Default for OarLoadOptions {
    fn default() -> Self {
        Self {
            region_id: Uuid::nil(),
            merge: false,
            load_terrain: true,
            load_objects: true,
            load_parcels: true,
            displacement: (0.0, 0.0, 0.0),
            rotation_degrees: 0.0,
            force_terrain: false,
            force_parcels: false,
            default_user_id: None,
        }
    }
}

/// OAR reader implementation
pub struct OarReader {
    db_pool: PgPool,
}

impl OarReader {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Load an OAR file into the database
    pub async fn load<P: AsRef<Path>>(
        &self,
        path: P,
        options: OarLoadOptions,
    ) -> Result<OarLoadResult> {
        let start = std::time::Instant::now();
        let mut stats = LoadStatistics::default();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        info!(
            "Loading OAR from {:?} for region {}",
            path.as_ref(),
            options.region_id
        );

        // Open the archive
        let archive =
            TarGzReader::open(path.as_ref()).with_context(|| "Failed to open OAR archive")?;

        // Parse archive.xml to verify format
        if let Some(archive_xml_data) = archive.get_archive_xml() {
            match self.parse_archive_xml(archive_xml_data) {
                Ok(meta) => {
                    info!("OAR version: {}.{}", meta.major_version, meta.minor_version);
                }
                Err(e) => {
                    warnings.push(format!("Could not parse archive.xml: {}", e));
                }
            }
        }

        // Clear existing data if not merging
        if !options.merge {
            info!("Clearing existing region data...");
            self.clear_region_data(&options.region_id).await?;
        }

        // Process assets
        info!("Processing assets...");
        for (path, data) in archive.get_asset_entries() {
            match self.import_asset(path, data).await {
                Ok(imported) => {
                    if imported {
                        stats.assets_loaded += 1;
                    } else {
                        stats.assets_skipped += 1;
                    }
                }
                Err(e) => {
                    stats.assets_failed += 1;
                    debug!("Failed to import asset {}: {}", path, e);
                }
            }
        }
        info!(
            "Assets: {} loaded, {} skipped, {} failed",
            stats.assets_loaded, stats.assets_skipped, stats.assets_failed
        );

        // Process terrain
        if options.load_terrain {
            info!("Processing terrain...");
            for (path, data) in archive.get_entries_with_prefix(paths::TERRAINS_PATH) {
                if path.ends_with(".r32") {
                    match self.import_terrain(&options.region_id, data).await {
                        Ok(_) => {
                            stats.terrain_loaded = true;
                            info!("Loaded terrain ({} bytes)", data.len());
                        }
                        Err(e) => {
                            warnings.push(format!("Failed to load terrain: {}", e));
                        }
                    }
                    break; // Only one terrain file
                }
            }
        }

        // Process region settings (find any .xml in settings/ - name varies per OAR)
        let mut settings_loaded = false;
        for (path, data) in archive.get_entries_with_prefix(paths::SETTINGS_PATH) {
            if path.ends_with(".xml") {
                match self.import_region_settings(&options.region_id, data).await {
                    Ok(_) => {
                        info!("Loaded region settings from {}", path);
                        settings_loaded = true;
                    }
                    Err(e) => warnings.push(format!(
                        "Failed to load region settings from {}: {}",
                        path, e
                    )),
                }
                break;
            }
        }
        if !settings_loaded {
            warnings.push("No region settings found in archive".into());
        }

        // Process parcels
        if options.load_parcels {
            info!("Processing parcels...");
            for (path, data) in archive.get_entries_with_prefix(paths::LANDDATA_PATH) {
                if path.ends_with(".xml") {
                    match self
                        .import_parcel(&options.region_id, data, options.default_user_id.as_ref())
                        .await
                    {
                        Ok(_) => stats.parcels_loaded += 1,
                        Err(e) => warnings.push(format!("Failed to load parcel {}: {}", path, e)),
                    }
                }
            }
            info!("Loaded {} parcels", stats.parcels_loaded);
        }

        // Process objects
        if options.load_objects {
            info!("Processing objects...");
            for (path, data) in archive.get_object_entries() {
                if path.ends_with(".xml") {
                    match self.import_object(&options.region_id, data, &options).await {
                        Ok(_) => stats.objects_created += 1,
                        Err(e) => {
                            warn!("Failed to load object {}: {}", path, e);
                        }
                    }
                }
            }
            info!("Loaded {} objects", stats.objects_created);
        }

        stats.elapsed_ms = start.elapsed().as_millis() as u64;

        Ok(OarLoadResult {
            success: errors.is_empty(),
            stats,
            warnings,
            errors,
        })
    }

    fn parse_archive_xml(&self, data: &[u8]) -> Result<OarArchiveXml> {
        let xml_str = std::str::from_utf8(data)?;
        quick_xml::de::from_str(xml_str).map_err(|e| anyhow!("XML parse error: {}", e))
    }

    async fn clear_region_data(&self, region_id: &Uuid) -> Result<()> {
        // Clear prims/objects
        sqlx::query("DELETE FROM prims WHERE regionuuid = $1")
            .bind(region_id)
            .execute(&self.db_pool)
            .await?;

        // Clear terrain (if exists)
        sqlx::query("DELETE FROM bakedterrain WHERE regionuuid = $1")
            .bind(region_id.to_string())
            .execute(&self.db_pool)
            .await?;

        // Clear parcels
        sqlx::query("DELETE FROM land WHERE regionuuid = $1")
            .bind(region_id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    async fn import_asset(&self, path: &str, data: &[u8]) -> Result<bool> {
        let uuid = extract_asset_uuid_from_path(path)
            .ok_or_else(|| anyhow!("Could not extract UUID from path: {}", path))?;

        let asset_type = self.determine_asset_type_from_path(path);

        let exists: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM assets WHERE id = $1")
            .bind(uuid)
            .fetch_optional(&self.db_pool)
            .await?;

        if exists.is_some() {
            return Ok(false);
        }

        sqlx::query(
            r#"INSERT INTO assets (id, assettype, name, description, data, create_time, local, temporary)
               VALUES ($1, $2, $3, '', $4, EXTRACT(EPOCH FROM NOW())::bigint, 0, 0)"#
        )
        .bind(uuid)
        .bind(asset_type as i32)
        .bind(format!("OAR Import {}", uuid))
        .bind(data)
        .execute(&self.db_pool)
        .await?;

        Ok(true)
    }

    fn determine_asset_type_from_path(&self, path: &str) -> i32 {
        use crate::archives::common::AssetType;
        if path.contains("_texture") || path.ends_with(".jp2") {
            return AssetType::Texture as i32;
        }
        if path.contains("_mesh") || path.ends_with(".llmesh") {
            return AssetType::Mesh as i32;
        }
        if path.contains("_sound") {
            return AssetType::Sound as i32;
        }
        if path.contains("_object") {
            return AssetType::Object as i32;
        }
        if path.contains("_notecard") {
            return AssetType::Notecard as i32;
        }
        if path.contains("_script") {
            return AssetType::LSLText as i32;
        }
        if path.contains("_animation") {
            return AssetType::Animation as i32;
        }
        AssetType::Unknown as i32
    }

    async fn import_terrain(&self, region_id: &Uuid, data: &[u8]) -> Result<()> {
        let expected_r32 = 256 * 256 * 4; // 262,144 bytes for raw .r32
        let terrain_data: Vec<u8>;
        let revision: i32;

        if data.len() == expected_r32 {
            // Raw .r32 format: 256x256 little-endian f32 heights
            // Convert to Variable2D format: sizeX(i32_le) + sizeY(i32_le) + heights(f32_le)
            let mut v2d = Vec::with_capacity(8 + data.len());
            v2d.extend_from_slice(&256_i32.to_le_bytes()); // sizeX
            v2d.extend_from_slice(&256_i32.to_le_bytes()); // sizeY
            v2d.extend_from_slice(data); // raw f32 heights
            terrain_data = v2d;
            revision = 22; // Variable2D
            info!(
                "[OAR] Converted .r32 terrain ({} bytes) to Variable2D ({} bytes)",
                data.len(),
                terrain_data.len()
            );
        } else {
            // Already in some other format, store as-is with Legacy256
            terrain_data = data.to_vec();
            revision = 11; // Legacy256
            info!("[OAR] Storing terrain as Legacy256 ({} bytes)", data.len());
        }

        // Delete existing terrain first (opensim-master pattern), then insert
        sqlx::query("DELETE FROM bakedterrain WHERE regionuuid = $1")
            .bind(region_id.to_string())
            .execute(&self.db_pool)
            .await?;

        sqlx::query(
            r#"INSERT INTO bakedterrain (regionuuid, heightfield, revision)
               VALUES ($1, $2, $3)"#,
        )
        .bind(region_id.to_string())
        .bind(&terrain_data)
        .bind(revision)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn import_region_settings(&self, region_id: &Uuid, data: &[u8]) -> Result<()> {
        let xml_str = std::str::from_utf8(data)?;
        let settings: OarRegionSettings = quick_xml::de::from_str(xml_str)
            .map_err(|e| anyhow!("Failed to parse region settings: {}", e))?;

        let tex1 = Uuid::parse_str(&settings.ground_textures.texture1).unwrap_or_default();
        let tex2 = Uuid::parse_str(&settings.ground_textures.texture2).unwrap_or_default();
        let tex3 = Uuid::parse_str(&settings.ground_textures.texture3).unwrap_or_default();
        let tex4 = Uuid::parse_str(&settings.ground_textures.texture4).unwrap_or_default();

        sqlx::query(
            r#"UPDATE regionsettings SET
                block_terraform = $2,
                block_fly = $3,
                allow_damage = $4,
                restrict_pushing = $5,
                allow_land_resell = $6,
                allow_land_join_divide = $7,
                agent_limit = $8,
                object_bonus = $9,
                disable_scripts = $10,
                disable_collisions = $11,
                disable_physics = $12,
                terrain_texture_1 = $13,
                terrain_texture_2 = $14,
                terrain_texture_3 = $15,
                terrain_texture_4 = $16,
                water_height = $17,
                terrain_raise_limit = $18,
                terrain_lower_limit = $19,
                use_estate_sun = $20,
                fixed_sun = $21,
                sun_position = $22
            WHERE regionuuid = $1"#,
        )
        .bind(region_id)
        .bind(if settings.general.block_terraform {
            1i32
        } else {
            0
        })
        .bind(if settings.general.block_fly { 1i32 } else { 0 })
        .bind(if settings.general.allow_damage {
            1i32
        } else {
            0
        })
        .bind(if settings.general.restrict_pushing {
            1i32
        } else {
            0
        })
        .bind(if settings.general.allow_land_resell {
            1i32
        } else {
            0
        })
        .bind(if settings.general.allow_land_join_divide {
            1i32
        } else {
            0
        })
        .bind(settings.general.agent_limit)
        .bind(settings.general.object_bonus as f32)
        .bind(if settings.general.disable_scripts {
            1i32
        } else {
            0
        })
        .bind(if settings.general.disable_collisions {
            1i32
        } else {
            0
        })
        .bind(if settings.general.disable_physics {
            1i32
        } else {
            0
        })
        .bind(tex1)
        .bind(tex2)
        .bind(tex3)
        .bind(tex4)
        .bind(settings.terrain.water_height as f32)
        .bind(settings.terrain.terrain_raise_limit as f32)
        .bind(settings.terrain.terrain_lower_limit as f32)
        .bind(if settings.terrain.use_estate_sun {
            1i32
        } else {
            0
        })
        .bind(if settings.terrain.fixed_sun { 1i32 } else { 0 })
        .bind(settings.terrain.sun_position as f32)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn import_parcel(
        &self,
        region_id: &Uuid,
        data: &[u8],
        default_user_id: Option<&Uuid>,
    ) -> Result<()> {
        let xml_str = std::str::from_utf8(data)?;
        let parcel: OarLandData = quick_xml::de::from_str(xml_str)
            .map_err(|e| anyhow!("Failed to parse parcel data: {}", e))?;

        let parcel_uuid = Uuid::parse_str(&parcel.global_id)?;
        let original_owner = Uuid::parse_str(&parcel.owner_id)?;
        let owner_uuid = default_user_id.copied().unwrap_or(original_owner);
        let group_uuid = parcel
            .group_id
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or(Uuid::nil());

        let (loc_x, loc_y, loc_z) =
            parse_vector3(parcel.user_location.as_deref().unwrap_or("<0,0,0>"));
        let (look_x, look_y, look_z) =
            parse_vector3(parcel.user_look_at.as_deref().unwrap_or("<0,0,0>"));

        sqlx::query(
            r#"INSERT INTO land (
                uuid, regionuuid, locallandid, bitmap, name, description,
                owneruuid, groupuuid, isgroupowned, area, auctionid, category,
                claimdate, claimprice, landflags, landingtype, passhours, passprice,
                saleprice, snapshotuuid, userlocationx, userlocationy, userlocationz,
                userlookatx, userlookaty, userlookatz,
                musicurl, mediaurl, mediaautoscale
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29
            ) ON CONFLICT (uuid) DO UPDATE SET name = $5"#,
        )
        .bind(parcel_uuid)
        .bind(region_id)
        .bind(parcel.local_id)
        .bind(parcel.bitmap.as_deref().and_then(|s| BASE64.decode(s).ok()))
        .bind(&parcel.name)
        .bind(parcel.description.as_deref().unwrap_or(""))
        .bind(owner_uuid)
        .bind(group_uuid)
        .bind(if parcel.is_group_owned { 1i32 } else { 0i32 })
        .bind(parcel.area)
        .bind(parcel.auction_id)
        .bind(parcel.category)
        .bind(parcel.claim_date as i32)
        .bind(parcel.claim_price)
        .bind(parcel.flags as i32)
        .bind(parcel.landing_type)
        .bind(parcel.pass_hours)
        .bind(parcel.pass_price)
        .bind(parcel.sale_price)
        .bind(
            parcel
                .snapshot_id
                .as_deref()
                .and_then(|s| Uuid::parse_str(s).ok()),
        )
        .bind(loc_x)
        .bind(loc_y)
        .bind(loc_z)
        .bind(look_x)
        .bind(look_y)
        .bind(look_z)
        .bind(parcel.music_url.as_deref().unwrap_or(""))
        .bind(parcel.media_url.as_deref().unwrap_or(""))
        .bind(if parcel.media_auto_scale { 1i32 } else { 0i32 })
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn import_object(
        &self,
        region_id: &Uuid,
        data: &[u8],
        options: &OarLoadOptions,
    ) -> Result<()> {
        let xml_str = std::str::from_utf8(data)?;
        let object: OarSceneObjectGroup = quick_xml::de::from_str(xml_str)
            .map_err(|e| anyhow!("Failed to parse object: {}", e))?;

        let scene_group_id = object.root_uuid();

        self.insert_prim(
            region_id,
            &scene_group_id,
            object.root_part(),
            1,
            options.default_user_id.as_ref(),
        )
        .await?;
        self.insert_primshape(object.root_part()).await?;

        if let Some(ref other_parts) = object.other_parts {
            for (i, wrapper) in other_parts.parts.iter().enumerate() {
                let link_num = (i + 2) as i32;
                self.insert_prim(
                    region_id,
                    &scene_group_id,
                    &wrapper.part,
                    link_num,
                    options.default_user_id.as_ref(),
                )
                .await?;
                self.insert_primshape(&wrapper.part).await?;
            }
        }

        Ok(())
    }

    async fn insert_prim(
        &self,
        region_id: &Uuid,
        scene_group_id: &Uuid,
        part: &OarSceneObjectPart,
        link_num: i32,
        default_user_id: Option<&Uuid>,
    ) -> Result<()> {
        let prim_uuid = part.uuid.parse_or_nil();
        let creator_id = part.creator_id.uuid.clone();
        let original_owner = part.owner_id.parse_or_nil();
        let owner_id = default_user_id.copied().unwrap_or(original_owner);
        let group_id = part.group_id.parse_or_nil();
        let original_last_owner = part.last_owner_id.parse_or_nil();
        let last_owner_id = default_user_id.copied().unwrap_or(original_last_owner);

        sqlx::query(
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
                creationdate, material, linknumber, passcollisions
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18,
                $19, $20, $21, $22, $23, $24, $25, $26, $27,
                $28, $29, $30, $31, $32, $33,
                $34, $35, $36, $37,
                $38, $39, $40, $41, $42, $43, $44,
                $45, $46, $47, $48
            ) ON CONFLICT (uuid) DO UPDATE SET
                name = $8, objectflags = $13, positionx = $19, positiony = $20, positionz = $21"#,
        )
        .bind(prim_uuid)
        .bind(region_id)
        .bind(&creator_id)
        .bind(owner_id)
        .bind(group_id)
        .bind(last_owner_id)
        .bind(scene_group_id)
        .bind(&part.name)
        .bind(&part.text)
        .bind(&part.description)
        .bind(&part.sit_name)
        .bind(&part.touch_name)
        .bind(part.flags_as_u32() as i32)
        .bind(part.owner_mask as i32)
        .bind(part.next_owner_mask as i32)
        .bind(part.group_mask as i32)
        .bind(part.everyone_mask as i32)
        .bind(part.base_mask as i32)
        .bind(if link_num == 1 {
            part.group_position.x
        } else {
            part.offset_position.x
        })
        .bind(if link_num == 1 {
            part.group_position.y
        } else {
            part.offset_position.y
        })
        .bind(if link_num == 1 {
            part.group_position.z
        } else {
            part.offset_position.z
        })
        .bind(part.group_position.x)
        .bind(part.group_position.y)
        .bind(part.group_position.z)
        .bind(part.velocity.x)
        .bind(part.velocity.y)
        .bind(part.velocity.z)
        .bind(part.angular_velocity.x)
        .bind(part.angular_velocity.y)
        .bind(part.angular_velocity.z)
        .bind(part.acceleration.x)
        .bind(part.acceleration.y)
        .bind(part.acceleration.z)
        .bind(part.rotation_offset.x)
        .bind(part.rotation_offset.y)
        .bind(part.rotation_offset.z)
        .bind(part.rotation_offset.w)
        .bind(part.sit_target_position.x)
        .bind(part.sit_target_position.y)
        .bind(part.sit_target_position.z)
        .bind(part.sit_target_orientation.w)
        .bind(part.sit_target_orientation.x)
        .bind(part.sit_target_orientation.y)
        .bind(part.sit_target_orientation.z)
        .bind(part.creation_date as i32)
        .bind(part.material)
        .bind(link_num)
        .bind(if part.pass_collisions { 1i32 } else { 0i32 })
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn insert_primshape(&self, part: &OarSceneObjectPart) -> Result<()> {
        let prim_uuid = part.uuid.parse_or_nil();
        let shape = &part.shape;

        let texture_bytes = if !shape.texture_entry.is_empty() {
            BASE64.decode(&shape.texture_entry).unwrap_or_default()
        } else {
            Vec::new()
        };

        let extra_params_bytes = if !shape.extra_params.is_empty() {
            BASE64.decode(&shape.extra_params).unwrap_or_default()
        } else {
            vec![0u8]
        };

        sqlx::query(
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
        .bind(prim_uuid)
        .bind(shape.profile_curve)
        .bind(part.scale.x)
        .bind(part.scale.y)
        .bind(part.scale.z)
        .bind(shape.pcode)
        .bind(shape.path_begin as i32)
        .bind(shape.path_end as i32)
        .bind(shape.path_scale_x)
        .bind(shape.path_scale_y)
        .bind(shape.path_shear_x)
        .bind(shape.path_shear_y)
        .bind(shape.path_skew)
        .bind(shape.path_curve)
        .bind(shape.path_radius_offset)
        .bind(shape.path_revolutions)
        .bind(shape.path_taper_x)
        .bind(shape.path_taper_y)
        .bind(shape.path_twist)
        .bind(shape.path_twist_begin)
        .bind(shape.profile_begin as i32)
        .bind(shape.profile_end as i32)
        .bind(shape.profile_curve)
        .bind(shape.profile_hollow as i32)
        .bind(&texture_bytes)
        .bind(&extra_params_bytes)
        .bind(shape.state)
        .bind(shape.media.as_deref())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

fn parse_vector3(s: &str) -> (f32, f32, f32) {
    let trimmed = s.trim_start_matches('<').trim_end_matches('>');
    let parts: Vec<f32> = trimmed
        .split(',')
        .map(|p| p.trim().parse::<f32>().unwrap_or(0.0))
        .collect();
    (
        parts.first().copied().unwrap_or(0.0),
        parts.get(1).copied().unwrap_or(0.0),
        parts.get(2).copied().unwrap_or(0.0),
    )
}
