use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

use super::decoder::{decode_lod_geometry, LodLevel};
use super::encoder::MeshGeometry;
use super::parser::parse_mesh_header;
use super::texture_resolver::TextureResolver;
use super::types::MeshAssetHeader;
use crate::region::avatar::appearance::{Appearance, Attachment};

pub const BAKE_HEAD: u32 = 8;
pub const BAKE_UPPER: u32 = 9;
pub const BAKE_LOWER: u32 = 10;
pub const BAKE_EYES: u32 = 11;
pub const BAKE_SKIRT: u32 = 12;
pub const BAKE_HAIR: u32 = 13;

pub struct SnapshotPiece {
    pub name: String,
    pub geometry: MeshGeometry,
    pub texture_paths: Vec<Option<PathBuf>>,
    pub texture_uuids: Vec<Option<Uuid>>,
    pub attachment_point: Option<u32>,
    pub position_offset: [f32; 3],
    pub rotation: [f32; 4],
}

pub struct AvatarSnapshot {
    pub avatar_id: Uuid,
    pub avatar_name: String,
    pub pieces: Vec<SnapshotPiece>,
    pub baked_texture_paths: BakedTexturePaths,
    pub visual_params: Vec<u8>,
}

pub struct BakedTexturePaths {
    pub head: Option<PathBuf>,
    pub upper: Option<PathBuf>,
    pub lower: Option<PathBuf>,
    pub eyes: Option<PathBuf>,
    pub skirt: Option<PathBuf>,
    pub hair: Option<PathBuf>,
    pub head_uuid: Option<Uuid>,
    pub upper_uuid: Option<Uuid>,
    pub lower_uuid: Option<Uuid>,
    pub eyes_uuid: Option<Uuid>,
}

impl Default for BakedTexturePaths {
    fn default() -> Self {
        Self {
            head: None,
            upper: None,
            lower: None,
            eyes: None,
            skirt: None,
            hair: None,
            head_uuid: None,
            upper_uuid: None,
            lower_uuid: None,
            eyes_uuid: None,
        }
    }
}

pub struct SnapshotCollector {
    pub texture_resolver: TextureResolver,
    output_dir: PathBuf,
}

impl SnapshotCollector {
    pub fn new(output_dir: &Path) -> Result<Self> {
        let tex_dir = output_dir.join("textures");
        std::fs::create_dir_all(&tex_dir)?;

        Ok(Self {
            texture_resolver: TextureResolver::new(&tex_dir),
            output_dir: output_dir.to_path_buf(),
        })
    }

    pub async fn collect_snapshot(
        &mut self,
        avatar_id: &Uuid,
        avatar_name: &str,
        appearance: &Appearance,
        db_pool: &sqlx::PgPool,
        asset_fetcher: Option<&crate::asset::AssetFetcher>,
    ) -> Result<AvatarSnapshot> {
        self.collect_snapshot_with_live_te(
            avatar_id,
            avatar_name,
            appearance,
            db_pool,
            asset_fetcher,
            &std::collections::HashMap::new(),
        )
        .await
    }

    pub async fn collect_snapshot_with_live_te(
        &mut self,
        avatar_id: &Uuid,
        avatar_name: &str,
        appearance: &Appearance,
        db_pool: &sqlx::PgPool,
        asset_fetcher: Option<&crate::asset::AssetFetcher>,
        live_texture_entries: &std::collections::HashMap<Uuid, Vec<u8>>,
    ) -> Result<AvatarSnapshot> {
        info!(
            "[SNAPSHOT] Collecting snapshot for {} ({})",
            avatar_name, avatar_id
        );

        let baked = self
            .resolve_baked_textures(appearance, db_pool, asset_fetcher)
            .await;

        let mut pieces = Vec::new();

        for attachment in &appearance.attachments {
            if (30..=37).contains(&attachment.point) {
                info!(
                    "[SNAPSHOT] Skipping HUD attachment point {} ({})",
                    attachment.point, attachment.asset_id
                );
                continue;
            }
            let live_te = live_texture_entries.get(&attachment.item_id);
            match self
                .collect_attachment_piece_with_te(attachment, db_pool, asset_fetcher, live_te)
                .await
            {
                Ok(piece) => {
                    info!(
                        "[SNAPSHOT] Collected attachment: {} (point {}, {} faces)",
                        piece.name,
                        attachment.point,
                        piece.geometry.faces.len()
                    );
                    pieces.push(piece);
                }
                Err(e) => {
                    warn!(
                        "[SNAPSHOT] Failed to collect attachment {} at point {}: {}",
                        attachment.asset_id, attachment.point, e
                    );
                }
            }
        }

        let mut visual_params = vec![0u8; 256];
        for (&id, &val) in &appearance.visual_params.params {
            if (id as usize) < visual_params.len() {
                visual_params[id as usize] = (val.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }

        info!(
            "[SNAPSHOT] Complete: {} pieces, baked textures: head={} upper={} lower={} eyes={}",
            pieces.len(),
            baked.head.is_some(),
            baked.upper.is_some(),
            baked.lower.is_some(),
            baked.eyes.is_some(),
        );

        Ok(AvatarSnapshot {
            avatar_id: *avatar_id,
            avatar_name: avatar_name.to_string(),
            pieces,
            baked_texture_paths: baked,
            visual_params,
        })
    }

    async fn resolve_baked_textures(
        &mut self,
        appearance: &Appearance,
        db_pool: &sqlx::PgPool,
        asset_fetcher: Option<&crate::asset::AssetFetcher>,
    ) -> BakedTexturePaths {
        let mut baked = BakedTexturePaths::default();

        for te in &appearance.textures {
            let path = self
                .texture_resolver
                .resolve_texture(&te.texture_id, db_pool, asset_fetcher)
                .await
                .ok();

            match te.face {
                BAKE_HEAD => {
                    baked.head = path;
                    baked.head_uuid = Some(te.texture_id);
                }
                BAKE_UPPER => {
                    baked.upper = path;
                    baked.upper_uuid = Some(te.texture_id);
                }
                BAKE_LOWER => {
                    baked.lower = path;
                    baked.lower_uuid = Some(te.texture_id);
                }
                BAKE_EYES => {
                    baked.eyes = path;
                    baked.eyes_uuid = Some(te.texture_id);
                }
                BAKE_SKIRT => baked.skirt = path,
                BAKE_HAIR => baked.hair = path,
                _ => {}
            }
        }

        baked
    }

    async fn collect_attachment_piece(
        &mut self,
        attachment: &Attachment,
        db_pool: &sqlx::PgPool,
        asset_fetcher: Option<&crate::asset::AssetFetcher>,
    ) -> Result<SnapshotPiece> {
        self.collect_attachment_piece_with_te(attachment, db_pool, asset_fetcher, None)
            .await
    }

    async fn collect_attachment_piece_with_te(
        &mut self,
        attachment: &Attachment,
        db_pool: &sqlx::PgPool,
        asset_fetcher: Option<&crate::asset::AssetFetcher>,
        live_te: Option<&Vec<u8>>,
    ) -> Result<SnapshotPiece> {
        let mesh_data = fetch_mesh_asset(&attachment.asset_id, db_pool, asset_fetcher).await?;
        let header = parse_mesh_header(&mesh_data)?;
        let geometry = decode_lod_geometry(&mesh_data, &header)?;

        let face_texture_ids = if let Some(te_bytes) = live_te {
            if !te_bytes.is_empty() {
                info!(
                    "[SNAPSHOT] Using live texture_entry ({} bytes) for attachment point {}",
                    te_bytes.len(),
                    attachment.point
                );
                super::texture_resolver::parse_texture_entry_face_textures(te_bytes)
            } else {
                self.extract_face_texture_ids_from_object(&attachment.item_id, db_pool)
                    .await
                    .unwrap_or_default()
            }
        } else {
            self.extract_face_texture_ids_from_object(&attachment.item_id, db_pool)
                .await
                .unwrap_or_default()
        };

        let mut texture_paths = Vec::new();
        let mut texture_uuids = Vec::new();
        for (i, _face) in geometry.faces.iter().enumerate() {
            if let Some(tex_id) = face_texture_ids.get(i) {
                if *tex_id != Uuid::nil() {
                    texture_uuids.push(Some(*tex_id));
                    match self
                        .texture_resolver
                        .resolve_texture(tex_id, db_pool, asset_fetcher)
                        .await
                    {
                        Ok(path) => texture_paths.push(Some(path)),
                        Err(_) => texture_paths.push(None),
                    }
                } else {
                    texture_uuids.push(None);
                    texture_paths.push(None);
                }
            } else {
                texture_uuids.push(None);
                texture_paths.push(None);
            }
        }

        let name = format!(
            "attach_{}_{}",
            attachment.point,
            attachment
                .asset_id
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );

        Ok(SnapshotPiece {
            name,
            geometry,
            texture_paths,
            texture_uuids,
            attachment_point: Some(attachment.point),
            position_offset: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
        })
    }

    async fn extract_face_texture_ids_from_object(
        &self,
        item_id: &Uuid,
        db_pool: &sqlx::PgPool,
    ) -> Result<Vec<Uuid>> {
        let row: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT texture FROM prims WHERE inventoryitemid = $1::uuid LIMIT 1")
                .bind(item_id)
                .fetch_optional(db_pool)
                .await?;

        if let Some((te_bytes,)) = row {
            Ok(super::texture_resolver::parse_texture_entry_face_textures(
                &te_bytes,
            ))
        } else {
            Ok(vec![])
        }
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

async fn fetch_mesh_asset(
    asset_id: &Uuid,
    db_pool: &sqlx::PgPool,
    asset_fetcher: Option<&crate::asset::AssetFetcher>,
) -> Result<Vec<u8>> {
    if let Some(fetcher) = asset_fetcher {
        if let Ok(Some(data)) = fetcher
            .fetch_asset_data_typed_pg(&asset_id.to_string(), Some(49), db_pool)
            .await
        {
            return Ok(data);
        }
    }

    let row: Option<(Vec<u8>,)> =
        sqlx::query_as("SELECT data FROM assets WHERE id = $1::uuid AND assettype = 49")
            .bind(asset_id)
            .fetch_optional(db_pool)
            .await?;

    row.map(|r| r.0)
        .ok_or_else(|| anyhow!("Mesh asset {} not found", asset_id))
}

pub fn attachment_point_name(point: u32) -> &'static str {
    match point {
        0 => "chest",
        1 => "skull",
        2 => "left_shoulder",
        3 => "right_shoulder",
        4 => "left_hand",
        5 => "right_hand",
        6 => "left_foot",
        7 => "right_foot",
        8 => "spine",
        9 => "pelvis",
        10 => "mouth",
        11 => "chin",
        12 => "left_ear",
        13 => "right_ear",
        14 => "left_eye",
        15 => "right_eye",
        16 => "nose",
        17 => "r_upper_arm",
        18 => "r_forearm",
        19 => "l_upper_arm",
        20 => "l_forearm",
        21 => "right_hip",
        22 => "r_upper_leg",
        23 => "r_lower_leg",
        24 => "left_hip",
        25 => "l_upper_leg",
        26 => "l_lower_leg",
        27 => "stomach",
        28 => "left_pec",
        29 => "right_pec",
        30 => "center2",
        31 => "top_right",
        32 => "top",
        33 => "top_left",
        34 => "center",
        35 => "bottom_left",
        36 => "bottom",
        37 => "bottom_right",
        38 => "neck",
        39 => "avatar_center",
        40 => "left_ring_finger",
        41 => "right_ring_finger",
        42 => "tail_base",
        43 => "tail_tip",
        44 => "left_wing",
        45 => "right_wing",
        46 => "jaw",
        47 => "alt_left_ear",
        48 => "alt_right_ear",
        49 => "alt_left_eye",
        50 => "alt_right_eye",
        51 => "tongue",
        52 => "groin",
        53 => "left_hind_foot",
        54 => "right_hind_foot",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_point_names() {
        assert_eq!(attachment_point_name(0), "chest");
        assert_eq!(attachment_point_name(1), "skull");
        assert_eq!(attachment_point_name(9), "pelvis");
        assert_eq!(attachment_point_name(31), "top_right");
        assert_eq!(attachment_point_name(39), "avatar_center");
        assert_eq!(attachment_point_name(255), "unknown");
    }

    #[test]
    fn test_baked_texture_defaults() {
        let baked = BakedTexturePaths::default();
        assert!(baked.head.is_none());
        assert!(baked.upper.is_none());
        assert!(baked.lower.is_none());
        assert!(baked.eyes.is_none());
    }

    #[test]
    fn test_bake_constants() {
        assert_eq!(BAKE_HEAD, 8);
        assert_eq!(BAKE_UPPER, 9);
        assert_eq!(BAKE_LOWER, 10);
        assert_eq!(BAKE_EYES, 11);
    }
}
