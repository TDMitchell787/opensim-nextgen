use super::types::WearableCacheItem;
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

pub struct BakedTextureCache {
    cache: RwLock<LruCache<Uuid, Vec<WearableCacheItem>>>,
    texture_cache: RwLock<LruCache<Uuid, Vec<u8>>>,
}

impl BakedTextureCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap()),
            )),
            texture_cache: RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity * 10).unwrap_or(NonZeroUsize::new(1000).unwrap()),
            )),
        }
    }

    pub fn get(&self, agent_id: &Uuid) -> Option<Vec<WearableCacheItem>> {
        let mut cache = self.cache.write();
        cache.get(agent_id).cloned()
    }

    pub fn store(&self, agent_id: Uuid, items: Vec<WearableCacheItem>) {
        info!(
            "🎨 Storing {} baked textures for agent {}",
            items.len(),
            agent_id
        );
        let mut cache = self.cache.write();
        cache.put(agent_id, items);
    }

    pub fn get_texture(&self, texture_id: &Uuid) -> Option<Vec<u8>> {
        let mut cache = self.texture_cache.write();
        cache.get(texture_id).cloned()
    }

    pub fn store_texture(&self, texture_id: Uuid, data: Vec<u8>) {
        debug!(
            "🎨 Caching baked texture {} ({} bytes)",
            texture_id,
            data.len()
        );
        let mut cache = self.texture_cache.write();
        cache.put(texture_id, data);
    }

    pub fn remove(&self, agent_id: &Uuid) {
        let mut cache = self.cache.write();
        cache.pop(agent_id);
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        let mut texture_cache = self.texture_cache.write();
        texture_cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.read().len()
    }

    pub fn texture_cache_len(&self) -> usize {
        self.texture_cache.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.read().is_empty()
    }
}

impl Default for BakedTextureCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

pub type SharedBakedTextureCache = Arc<BakedTextureCache>;

pub fn create_shared_cache(capacity: usize) -> SharedBakedTextureCache {
    Arc::new(BakedTextureCache::new(capacity))
}
