pub mod blender_worker;
pub mod collada_geometry;
pub mod collada_skin;
pub mod dae_writer;
pub mod decoder;
pub mod encoder;
pub mod glc_bridge;
pub mod parser;
pub mod snapshot_collector;
pub mod texture_compositor;
pub mod texture_resolver;
pub mod types;

pub use blender_worker::{ruth2_base_dir, ruth2_dae_path, ruth2_texture_path, ruth2_uv_path};
pub use glc_bridge::{import_ruth2_body_set, import_ruth2_part};
pub use texture_compositor::{
    compose_garment_texture_with_uv, load_ruth2_uv_map, load_sl_avatar_texture,
};

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use types::ConvexPhysicsData;

pub struct MeshPhysicsCache {
    cache: RwLock<HashMap<Uuid, Arc<ConvexPhysicsData>>>,
}

impl MeshPhysicsCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, asset_id: &Uuid) -> Option<Arc<ConvexPhysicsData>> {
        self.cache.read().await.get(asset_id).cloned()
    }

    pub async fn insert(&self, asset_id: Uuid, data: ConvexPhysicsData) -> Arc<ConvexPhysicsData> {
        let arc = Arc::new(data);
        self.cache.write().await.insert(asset_id, arc.clone());
        arc
    }

    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }
}

pub fn extract_mesh_asset_uuid(extra_params: &[u8]) -> Option<Uuid> {
    if extra_params.len() < 2 {
        return None;
    }
    let count = extra_params[0] as usize;
    let mut i = 1usize;
    for _ in 0..count {
        if i + 6 > extra_params.len() {
            break;
        }
        let ep_type = extra_params[i];
        let data_len = u32::from_le_bytes([
            extra_params[i + 2],
            extra_params[i + 3],
            extra_params[i + 4],
            extra_params[i + 5],
        ]) as usize;
        let payload_start = i + 6;
        let payload_end = payload_start + data_len;
        if payload_end > extra_params.len() {
            break;
        }
        if (ep_type == 0x30 || ep_type == 0x60) && data_len >= 17 {
            let sculpt_type = extra_params[payload_start + 16];
            if sculpt_type & 0x07 == 5 {
                let uuid_bytes: [u8; 16] = extra_params[payload_start..payload_start + 16]
                    .try_into()
                    .ok()?;
                return Some(Uuid::from_bytes(uuid_bytes));
            }
        }
        i = payload_end;
    }
    None
}

pub async fn create_mesh_physics_shape(
    asset_uuid: &Uuid,
    db_pool: &sqlx::PgPool,
    scale: [f32; 3],
    cache: &MeshPhysicsCache,
    asset_fetcher: Option<&crate::asset::AssetFetcher>,
) -> Result<(Arc<ConvexPhysicsData>, Vec<f32>)> {
    if let Some(cached) = cache.get(asset_uuid).await {
        let hull_array = parser::to_bullet_hull_array(&cached, scale);
        return Ok((cached, hull_array));
    }

    let data = if let Some(fetcher) = asset_fetcher {
        fetcher
            .fetch_asset_data_typed_pg(&asset_uuid.to_string(), Some(49), db_pool)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Mesh asset {} not found", asset_uuid))?
    } else {
        let row: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT data FROM assets WHERE id = $1::uuid AND assettype = 49")
                .bind(asset_uuid)
                .fetch_optional(db_pool)
                .await?;
        row.ok_or_else(|| anyhow::anyhow!("Mesh asset {} not found in DB", asset_uuid))?
            .0
    };

    let header = parser::parse_mesh_header(&data)?;
    let convex = parser::extract_physics_convex(&data, &header)?;

    info!(
        "[MESH-PHYSICS] Asset {} parsed: {} hulls, {} total vertices",
        asset_uuid,
        convex.hull_count(),
        convex.total_vertices()
    );

    let hull_array = parser::to_bullet_hull_array(&convex, scale);
    let cached = cache.insert(*asset_uuid, convex).await;

    Ok((cached, hull_array))
}
