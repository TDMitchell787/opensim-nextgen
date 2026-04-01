use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, warn};

use crate::services::traits::MapImageServiceTrait;

pub struct LocalMapImageService {
    tiles_dir: String,
}

impl LocalMapImageService {
    pub fn new(tiles_dir: String) -> Self {
        Self { tiles_dir }
    }

    fn tile_path(&self, scope_id: &str, filename: &str) -> std::path::PathBuf {
        let scope = if scope_id.is_empty() || scope_id == "00000000-0000-0000-0000-000000000000" {
            "00000000-0000-0000-0000-000000000000"
        } else {
            scope_id
        };
        std::path::PathBuf::from(&self.tiles_dir).join(scope).join(filename)
    }

    fn tile_filename(zoom: i32, x: i32, y: i32) -> String {
        format!("map-{}-{}-{}-objects.jpg", zoom, x, y)
    }
}

#[async_trait]
impl MapImageServiceTrait for LocalMapImageService {
    async fn add_map_tile(&self, x: i32, y: i32, data: &[u8], scope_id: &str) -> Result<bool> {
        let filename = Self::tile_filename(1, x, y);
        let path = self.tile_path(scope_id, &filename);

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&path, data).await?;
        debug!("[MAP] Stored tile at {:?} ({} bytes)", path, data.len());
        Ok(true)
    }

    async fn remove_map_tile(&self, x: i32, y: i32, scope_id: &str) -> Result<bool> {
        let filename = Self::tile_filename(1, x, y);
        let path = self.tile_path(scope_id, &filename);

        if path.exists() {
            tokio::fs::remove_file(&path).await?;
            debug!("[MAP] Removed tile at {:?}", path);
            Ok(true)
        } else {
            warn!("[MAP] Tile not found at {:?}", path);
            Ok(false)
        }
    }

    async fn get_map_tile(&self, filename: &str, scope_id: &str) -> Result<Option<Vec<u8>>> {
        let path = self.tile_path(scope_id, filename);

        if path.exists() {
            let data = tokio::fs::read(&path).await?;
            debug!("[MAP] Read tile {:?} ({} bytes)", path, data.len());
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}
