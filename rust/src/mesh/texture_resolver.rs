use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

use crate::asset::jpeg2000::J2KCodec;

pub struct TextureResolver {
    j2k_codec: J2KCodec,
    cache: HashMap<Uuid, PathBuf>,
    output_dir: PathBuf,
}

impl TextureResolver {
    pub fn new(output_dir: &Path) -> Self {
        Self {
            j2k_codec: J2KCodec::new(),
            cache: HashMap::new(),
            output_dir: output_dir.to_path_buf(),
        }
    }

    pub async fn resolve_texture(
        &mut self,
        texture_id: &Uuid,
        db_pool: &sqlx::PgPool,
        asset_fetcher: Option<&crate::asset::AssetFetcher>,
    ) -> Result<PathBuf> {
        if *texture_id == Uuid::nil() {
            return Err(anyhow!("Nil texture UUID"));
        }

        if let Some(cached) = self.cache.get(texture_id) {
            return Ok(cached.clone());
        }

        let j2k_data = fetch_texture_asset(texture_id, db_pool, asset_fetcher).await?;
        let png_data = self.j2k_codec.decode_to_png(&j2k_data)?;

        let filename = format!("{}.png", texture_id);
        let path = self.output_dir.join(&filename);
        std::fs::write(&path, &png_data)?;

        info!("[TEXTURE-RESOLVER] Decoded {} → {} ({} bytes)", texture_id, path.display(), png_data.len());

        self.cache.insert(*texture_id, path.clone());
        Ok(path)
    }

    pub async fn resolve_textures_batch(
        &mut self,
        texture_ids: &[Uuid],
        db_pool: &sqlx::PgPool,
        asset_fetcher: Option<&crate::asset::AssetFetcher>,
    ) -> Vec<Option<PathBuf>> {
        let mut results = Vec::with_capacity(texture_ids.len());
        for tid in texture_ids {
            if *tid == Uuid::nil() {
                results.push(None);
                continue;
            }
            match self.resolve_texture(tid, db_pool, asset_fetcher).await {
                Ok(path) => results.push(Some(path)),
                Err(e) => {
                    warn!("[TEXTURE-RESOLVER] Failed to resolve {}: {}", tid, e);
                    results.push(None);
                }
            }
        }
        results
    }

    pub fn get_cached_path(&self, texture_id: &Uuid) -> Option<&PathBuf> {
        self.cache.get(texture_id)
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

pub async fn fetch_texture_asset(
    texture_id: &Uuid,
    db_pool: &sqlx::PgPool,
    asset_fetcher: Option<&crate::asset::AssetFetcher>,
) -> Result<Vec<u8>> {
    if let Some(fetcher) = asset_fetcher {
        if let Ok(Some(data)) = fetcher.fetch_asset_data_typed_pg(
            &texture_id.to_string(), Some(0), db_pool
        ).await {
            return Ok(data);
        }
    }

    let row: Option<(Vec<u8>,)> = sqlx::query_as(
        "SELECT data FROM assets WHERE id = $1::uuid AND assettype = 0"
    )
    .bind(texture_id)
    .fetch_optional(db_pool)
    .await?;

    row.map(|r| r.0)
        .ok_or_else(|| anyhow!("Texture asset {} not found", texture_id))
}

pub fn parse_texture_entry_face_textures(texture_entry: &[u8]) -> Vec<Uuid> {
    if texture_entry.len() < 16 {
        return vec![];
    }

    let default_texture = uuid_from_le_bytes(&texture_entry[0..16]);
    let mut face_textures: Vec<Uuid> = vec![default_texture; 32];

    let mut pos = 16;
    while pos + 17 <= texture_entry.len() {
        let face_bits = texture_entry[pos];
        pos += 1;
        if pos + 16 > texture_entry.len() {
            break;
        }
        let tex_id = uuid_from_le_bytes(&texture_entry[pos..pos + 16]);
        pos += 16;

        for face_idx in 0..8u8 {
            if face_bits & (1 << face_idx) != 0 {
                if (face_idx as usize) < face_textures.len() {
                    face_textures[face_idx as usize] = tex_id;
                }
            }
        }

        if pos >= texture_entry.len() || texture_entry[pos] == 0 {
            break;
        }
    }

    face_textures
}

fn uuid_from_le_bytes(bytes: &[u8]) -> Uuid {
    if bytes.len() < 16 {
        return Uuid::nil();
    }
    Uuid::from_bytes([
        bytes[3], bytes[2], bytes[1], bytes[0],
        bytes[5], bytes[4],
        bytes[7], bytes[6],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_texture_entry_default_only() {
        let mut te = vec![0u8; 16];
        let test_uuid = Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let bytes = test_uuid.as_bytes();
        te[0] = bytes[3]; te[1] = bytes[2]; te[2] = bytes[1]; te[3] = bytes[0];
        te[4] = bytes[5]; te[5] = bytes[4];
        te[6] = bytes[7]; te[7] = bytes[6];
        te[8..16].copy_from_slice(&bytes[8..16]);

        let faces = parse_texture_entry_face_textures(&te);
        assert_eq!(faces.len(), 32);
        assert_eq!(faces[0], test_uuid);
        assert_eq!(faces[1], test_uuid);
    }

    #[test]
    fn test_uuid_from_le_bytes_roundtrip() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let bytes = uuid.as_bytes();
        let le: Vec<u8> = vec![
            bytes[3], bytes[2], bytes[1], bytes[0],
            bytes[5], bytes[4],
            bytes[7], bytes[6],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ];
        let result = uuid_from_le_bytes(&le);
        assert_eq!(result, uuid);
    }

    #[test]
    fn test_texture_resolver_cache() {
        let dir = std::env::temp_dir().join("test_tex_resolver");
        let resolver = TextureResolver::new(&dir);
        assert!(resolver.get_cached_path(&Uuid::nil()).is_none());
    }
}
