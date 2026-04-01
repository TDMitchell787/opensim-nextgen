//! Avatar Factory Module for OpenSim Next
//!
//! Provides baked texture cache validation and management.
//! This is critical for the viewer to transition from "Connecting" to "In World" state.
//!
//! Based on OpenSim's AvatarFactoryModule.cs

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use tracing::{debug, info, warn};
use std::time::{Duration, Instant};
use crate::database::multi_backend::DatabaseConnection;

/// Baked texture face indices as defined in OpenSim/LibreMetaverse
/// These are the texture faces that contain baked avatar textures
pub const BAKE_INDICES: [usize; 11] = [
    8,   // TEX_HEAD_BAKED
    9,   // TEX_UPPER_BAKED
    10,  // TEX_LOWER_BAKED
    11,  // TEX_EYES_BAKED
    19,  // TEX_SKIRT_BAKED
    20,  // TEX_HAIR_BAKED
    40,  // TEX_LEFT_ARM_BAKED
    41,  // TEX_LEFT_LEG_BAKED
    42,  // TEX_AUX1_BAKED
    43,  // TEX_AUX2_BAKED
    44,  // TEX_AUX3_BAKED
];

/// Number of bake indices for protocol version 7+
pub const BAKES_COUNT_PV7: usize = 11;

/// Total number of texture faces in the avatar texture entry
pub const TEXTURE_COUNT: usize = 45;

/// Default avatar texture UUID - signals "needs rebake"
/// This is the standard sentinel value used by OpenSim/SL
pub const DEFAULT_AVATAR_TEXTURE: &str = "c228d1cf-4b5d-4ba8-84f4-899a0796aa97";

/// Minimum valid baked texture size in bytes.
/// Viewer sends ~800-byte placeholder bakes before wearable assets are loaded.
/// Real baked textures are 10-100+ KB. Reject anything under 2KB as garbage.
pub const MIN_BAKE_TEXTURE_SIZE: usize = 2048;

pub fn parse_texture_entry_uuids(raw: &[u8]) -> Vec<(usize, Uuid)> {
    let mut result = Vec::new();
    if raw.len() < 16 {
        return result;
    }

    let mut pos = 16;

    while pos < raw.len() {
        if raw[pos] == 0 {
            break;
        }

        let mut face_bits: u64 = 0;
        loop {
            if pos >= raw.len() { return result; }
            let b = raw[pos];
            face_bits = (face_bits << 7) | (b as u64 & 0x7F);
            pos += 1;
            if b & 0x80 == 0 { break; }
        }

        if pos + 16 > raw.len() { return result; }
        if let Ok(uuid) = Uuid::from_slice(&raw[pos..pos + 16]) {
            for bit in 0..45u32 {
                if face_bits & (1u64 << bit) != 0 {
                    result.push((bit as usize, uuid));
                }
            }
        }
        pos += 16;
    }

    result
}

/// Wearable cache item for tracking baked textures
/// cache_id = viewer's CacheId (hash for cache validation)
/// texture_id = actual baked texture UUID (from UploadBakedTexture CAPS)
#[derive(Debug, Clone)]
pub struct WearableCacheItem {
    pub cache_id: Uuid,
    pub texture_id: Uuid,
}

impl WearableCacheItem {
    pub fn new() -> Self {
        Self {
            cache_id: Uuid::nil(),
            texture_id: Uuid::parse_str(DEFAULT_AVATAR_TEXTURE).unwrap_or(Uuid::nil()),
        }
    }

    pub fn with_cache_and_texture(cache_id: Uuid, texture_id: Uuid) -> Self {
        Self { cache_id, texture_id }
    }

    pub fn with_texture(texture_id: Uuid) -> Self {
        Self {
            cache_id: Uuid::nil(),
            texture_id,
        }
    }
}

impl Default for WearableCacheItem {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent appearance data with baked texture cache
#[derive(Debug, Clone)]
pub struct AgentAppearanceData {
    pub agent_id: Uuid,
    pub wearable_cache: Vec<WearableCacheItem>,
    pub texture_ids: Vec<Uuid>,
    pub visual_params: Vec<u8>,
    pub is_validated: bool,
    pub needs_rebake: bool,
    pub raw_texture_entry: Option<Vec<u8>>,
    pub serial_num: u32,
}

impl AgentAppearanceData {
    pub fn new(agent_id: Uuid) -> Self {
        let default_texture = Uuid::parse_str(DEFAULT_AVATAR_TEXTURE).unwrap_or(Uuid::nil());

        let mut texture_ids = vec![default_texture; TEXTURE_COUNT];
        let mut wearable_cache = Vec::with_capacity(TEXTURE_COUNT);

        for _ in 0..TEXTURE_COUNT {
            wearable_cache.push(WearableCacheItem::new());
        }

        Self {
            agent_id,
            wearable_cache,
            texture_ids,
            visual_params: vec![128u8; 218],
            is_validated: false,
            needs_rebake: true,
            raw_texture_entry: None,
            serial_num: 0,
        }
    }

    /// Check if a specific baked texture is valid (not default)
    pub fn is_baked_texture_valid(&self, face_index: usize) -> bool {
        if face_index >= self.texture_ids.len() {
            return false;
        }

        let default_uuid = Uuid::parse_str(DEFAULT_AVATAR_TEXTURE).unwrap_or(Uuid::nil());
        let texture_id = self.texture_ids[face_index];

        !texture_id.is_nil() && texture_id != default_uuid
    }

    /// Get baked texture for a face
    pub fn get_baked_texture(&self, face_index: usize) -> Option<Uuid> {
        self.texture_ids.get(face_index).copied()
    }

    /// Set baked texture for a face
    pub fn set_baked_texture(&mut self, face_index: usize, texture_id: Uuid) {
        if face_index < self.texture_ids.len() {
            self.texture_ids[face_index] = texture_id;

            if face_index < self.wearable_cache.len() {
                let existing_cache_id = self.wearable_cache[face_index].cache_id;
                if !existing_cache_id.is_nil() {
                    self.wearable_cache[face_index].texture_id = texture_id;
                } else {
                    self.wearable_cache[face_index] = WearableCacheItem::with_texture(texture_id);
                }
            }

            self.is_validated = false;
        }
    }
}

/// Appearance save request for the queue
#[derive(Debug, Clone)]
pub struct AppearanceSaveRequest {
    pub agent_id: Uuid,
    pub priority: SavePriority,
}

/// Priority levels for appearance saves
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavePriority {
    Immediate,  // Save right away (logout, crash recovery)
    Normal,     // Normal save (periodic, appearance change)
    Deferred,   // Can be delayed (minor changes)
}

/// Avatar Factory - manages baked texture cache and validation
#[derive(Debug)]
pub struct AvatarFactory {
    appearance_cache: Arc<RwLock<HashMap<Uuid, AgentAppearanceData>>>,
    asset_cache: Arc<RwLock<HashMap<Uuid, bool>>>,
    save_queue_tx: Option<mpsc::Sender<AppearanceSaveRequest>>,
    pending_saves: Arc<RwLock<HashMap<Uuid, Instant>>>,
    db_connection: Option<Arc<DatabaseConnection>>,
    dirty_save: Arc<parking_lot::RwLock<HashMap<Uuid, Instant>>>,
    dirty_send: Arc<parking_lot::RwLock<HashMap<Uuid, Instant>>>,
    last_broadcast: Arc<parking_lot::RwLock<HashMap<Uuid, Instant>>>,
    first_bake_complete: Arc<parking_lot::RwLock<HashSet<Uuid>>>,
}

impl AvatarFactory {
    pub fn new() -> Self {
        Self {
            appearance_cache: Arc::new(RwLock::new(HashMap::new())),
            asset_cache: Arc::new(RwLock::new(HashMap::new())),
            save_queue_tx: None,
            pending_saves: Arc::new(RwLock::new(HashMap::new())),
            db_connection: None,
            dirty_save: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            dirty_send: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            last_broadcast: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            first_bake_complete: Arc::new(parking_lot::RwLock::new(HashSet::new())),
        }
    }

    pub fn with_database(mut self, db: Arc<DatabaseConnection>) -> Self {
        self.db_connection = Some(db);
        self
    }

    pub fn with_queue() -> (Self, mpsc::Receiver<AppearanceSaveRequest>) {
        let (tx, rx) = mpsc::channel(100);

        let factory = Self {
            appearance_cache: Arc::new(RwLock::new(HashMap::new())),
            asset_cache: Arc::new(RwLock::new(HashMap::new())),
            save_queue_tx: Some(tx),
            pending_saves: Arc::new(RwLock::new(HashMap::new())),
            db_connection: None,
            dirty_save: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            dirty_send: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            last_broadcast: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            first_bake_complete: Arc::new(parking_lot::RwLock::new(HashSet::new())),
        };

        (factory, rx)
    }

    pub fn mark_appearance_dirty(&self, agent_id: Uuid) {
        let now = Instant::now();
        self.dirty_save.write().insert(agent_id, now);
        self.dirty_send.write().insert(agent_id, now);
    }

    pub fn should_broadcast(&self, agent_id: Uuid, min_interval: Duration) -> bool {
        let now = Instant::now();
        let mut map = self.last_broadcast.write();
        if let Some(last) = map.get(&agent_id) {
            if now.duration_since(*last) < min_interval {
                return false;
            }
        }
        map.insert(agent_id, now);
        true
    }

    pub fn is_first_bake_complete(&self, agent_id: Uuid) -> bool {
        self.first_bake_complete.read().contains(&agent_id)
    }

    pub fn mark_first_bake_complete(&self, agent_id: Uuid) {
        self.first_bake_complete.write().insert(agent_id);
    }

    pub fn clear_first_bake(&self, agent_id: Uuid) {
        self.first_bake_complete.write().remove(&agent_id);
    }

    pub fn take_dirty_saves(&self, max_age: Duration) -> Vec<Uuid> {
        let now = Instant::now();
        let mut dirty = self.dirty_save.write();
        let ready: Vec<Uuid> = dirty.iter()
            .filter(|(_, ts)| now.duration_since(**ts) >= max_age)
            .map(|(id, _)| *id)
            .collect();
        for id in &ready {
            dirty.remove(id);
        }
        ready
    }

    pub fn take_dirty_sends(&self, max_age: Duration) -> Vec<Uuid> {
        let now = Instant::now();
        let mut dirty = self.dirty_send.write();
        let ready: Vec<Uuid> = dirty.iter()
            .filter(|(_, ts)| now.duration_since(**ts) >= max_age)
            .map(|(id, _)| *id)
            .collect();
        for id in &ready {
            dirty.remove(id);
        }
        ready
    }

    pub fn flush_dirty_for_agent(&self, agent_id: Uuid) {
        self.dirty_save.write().remove(&agent_id);
        self.dirty_send.write().remove(&agent_id);
    }

    pub fn remove_appearance(&self, agent_id: &Uuid) {
        if let Ok(mut cache) = self.appearance_cache.try_write() {
            cache.remove(agent_id);
        }
        self.dirty_save.write().remove(agent_id);
        self.dirty_send.write().remove(agent_id);
        self.last_broadcast.write().remove(agent_id);
        self.first_bake_complete.write().remove(agent_id);
    }

    pub async fn process_save_queue(
        mut rx: mpsc::Receiver<AppearanceSaveRequest>,
        appearance_cache: Arc<RwLock<HashMap<Uuid, AgentAppearanceData>>>,
        db_connection: Option<Arc<DatabaseConnection>>,
    ) {
        info!("[AVATAR_FACTORY] Starting appearance save queue processor");

        while let Some(request) = rx.recv().await {
            let agent_id = request.agent_id;

            match request.priority {
                SavePriority::Immediate => {
                    info!("[AVATAR_FACTORY] Processing immediate save for {}", agent_id);
                    Self::save_appearance_to_db(&appearance_cache, agent_id, db_connection.as_deref()).await;
                }
                SavePriority::Normal => {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    debug!("[AVATAR_FACTORY] Processing normal save for {}", agent_id);
                    Self::save_appearance_to_db(&appearance_cache, agent_id, db_connection.as_deref()).await;
                }
                SavePriority::Deferred => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    debug!("[AVATAR_FACTORY] Processing deferred save for {}", agent_id);
                    Self::save_appearance_to_db(&appearance_cache, agent_id, db_connection.as_deref()).await;
                }
            }
        }

        info!("[AVATAR_FACTORY] Appearance save queue processor stopped");
    }

    async fn save_appearance_to_db(
        appearance_cache: &Arc<RwLock<HashMap<Uuid, AgentAppearanceData>>>,
        agent_id: Uuid,
        db: Option<&DatabaseConnection>,
    ) {
        let cache = appearance_cache.read().await;

        let appearance = match cache.get(&agent_id) {
            Some(a) => a,
            None => {
                warn!("[AVATAR_FACTORY] No appearance data to save for {}", agent_id);
                return;
            }
        };

        let db = match db {
            Some(d) => d,
            None => {
                info!("[AVATAR_FACTORY] No DB connection, skipping save for {}", agent_id);
                return;
            }
        };

        let agent_id_str = agent_id.to_string();

        let mut entries: Vec<(String, String)> = Vec::new();
        let default_uuid = Uuid::parse_str(DEFAULT_AVATAR_TEXTURE).unwrap_or(Uuid::nil());

        if let Some(ref raw_te) = appearance.raw_texture_entry {
            let parsed_uuids = parse_texture_entry_uuids(raw_te);
            for (idx, tex_id) in &parsed_uuids {
                if !tex_id.is_nil() && *tex_id != default_uuid {
                    entries.push((format!("Texture {}", *idx), tex_id.to_string()));
                }
            }
            let hex: String = raw_te.iter().map(|b| format!("{:02x}", b)).collect();
            entries.push(("RawTextureEntry".to_string(), hex));
            info!("[AVATAR_FACTORY] Saving with parsed raw TE ({} face overrides, {} raw bytes)", parsed_uuids.len(), raw_te.len());
        } else {
            for (idx, tex_id) in appearance.texture_ids.iter().enumerate() {
                if !tex_id.is_nil() && *tex_id != default_uuid {
                    entries.push((format!("Texture {}", idx), tex_id.to_string()));
                }
            }
        }

        if !appearance.visual_params.is_empty() {
            let hex: String = appearance.visual_params.iter().map(|b| format!("{:02x}", b)).collect();
            entries.push(("VisualParams".to_string(), hex));
        }

        for &bake_idx in &BAKE_INDICES {
            if bake_idx < appearance.wearable_cache.len() {
                let item = &appearance.wearable_cache[bake_idx];
                if !item.cache_id.is_nil() && !item.texture_id.is_nil() && item.texture_id != default_uuid {
                    entries.push((
                        format!("WearableCache {}", bake_idx),
                        format!("{}|{}", item.cache_id, item.texture_id),
                    ));
                }
            }
        }

        if entries.is_empty() {
            debug!("[AVATAR_FACTORY] No meaningful appearance data to save for {}", agent_id);
            return;
        }

        match db {
            DatabaseConnection::PostgreSQL(pool) => {
                if let Err(e) = sqlx::query("DELETE FROM avatars WHERE principalid = $1::uuid AND (name LIKE 'Texture %' OR name = 'RawTextureEntry' OR name LIKE 'WearableCache %')")
                    .bind(&agent_id_str)
                    .execute(pool)
                    .await
                {
                    warn!("[AVATAR_FACTORY] Failed to clear old textures for {}: {}", agent_id, e);
                }
                if let Err(e) = sqlx::query("DELETE FROM avatars WHERE principalid = $1::uuid AND name = 'VisualParams'")
                    .bind(&agent_id_str)
                    .execute(pool)
                    .await
                {
                    warn!("[AVATAR_FACTORY] Failed to clear old visual params for {}: {}", agent_id, e);
                }

                let mut saved = 0;
                for (name, value) in &entries {
                    if let Err(e) = sqlx::query(
                        "INSERT INTO avatars (principalid, name, value) VALUES ($1::uuid, $2, $3)"
                    )
                    .bind(&agent_id_str)
                    .bind(name)
                    .bind(value)
                    .execute(pool)
                    .await
                    {
                        warn!("[AVATAR_FACTORY] Failed to save {} for {}: {}", name, agent_id, e);
                    } else {
                        saved += 1;
                    }
                }
                info!("[AVATAR_FACTORY] Saved {}/{} appearance entries for {}", saved, entries.len(), agent_id);
            }
            DatabaseConnection::MySQL(pool) => {
                if let Err(e) = sqlx::query("DELETE FROM Avatars WHERE PrincipalID = ? AND (Name LIKE 'Texture %' OR Name = 'RawTextureEntry' OR Name LIKE 'WearableCache %')")
                    .bind(&agent_id_str)
                    .execute(pool)
                    .await
                {
                    warn!("[AVATAR_FACTORY] Failed to clear old textures for {}: {}", agent_id, e);
                }
                if let Err(e) = sqlx::query("DELETE FROM Avatars WHERE PrincipalID = ? AND Name = 'VisualParams'")
                    .bind(&agent_id_str)
                    .execute(pool)
                    .await
                {
                    warn!("[AVATAR_FACTORY] Failed to clear old visual params for {}: {}", agent_id, e);
                }

                let mut saved = 0;
                for (name, value) in &entries {
                    if let Err(e) = sqlx::query(
                        "INSERT INTO Avatars (PrincipalID, Name, Value) VALUES (?, ?, ?)"
                    )
                    .bind(&agent_id_str)
                    .bind(name)
                    .bind(value)
                    .execute(pool)
                    .await
                    {
                        warn!("[AVATAR_FACTORY] Failed to save {} for {}: {}", name, agent_id, e);
                    } else {
                        saved += 1;
                    }
                }
                info!("[AVATAR_FACTORY] Saved {}/{} appearance entries for {}", saved, entries.len(), agent_id);
            }
        }
    }

    pub async fn create_appearance(&self, agent_id: Uuid) -> AgentAppearanceData {
        let mut cache = self.appearance_cache.write().await;
        if let Some(existing) = cache.get(&agent_id) {
            info!("[AVATAR_FACTORY] Appearance already exists for agent {} — preserving (not overwriting)", agent_id);
            return existing.clone();
        }
        let appearance = AgentAppearanceData::new(agent_id);
        cache.insert(agent_id, appearance.clone());
        info!("[AVATAR_FACTORY] Created NEW appearance data for agent {}", agent_id);
        appearance
    }

    pub async fn create_appearance_with_ruth_defaults(&self, agent_id: Uuid) -> AgentAppearanceData {
        let mut appearance = AgentAppearanceData::new(agent_id);

        let ruth_bakes: [(usize, &str); 5] = [
            (8,  "5a9f4a74-30f2-821c-b88d-70499d3e7183"), // Head
            (9,  "ae2de45c-d252-50b8-5c6e-19f39ce79317"), // Upper
            (10, "24daea5f-0539-cfcf-047f-fbc40b2786ba"), // Lower
            (11, "52cc6bb6-2ee5-e632-d3ad-50197b1dcb8a"), // Eyes
            (20, "09aac1fb-6bce-0bee-7d44-caac6dbb6c63"), // Hair
        ];

        for (idx, uuid_str) in &ruth_bakes {
            if let Ok(tex_id) = Uuid::parse_str(uuid_str) {
                appearance.set_baked_texture(*idx, tex_id);
            }
        }

        appearance.visual_params = get_ruth_visual_params().to_vec();
        appearance.needs_rebake = false;
        appearance.is_validated = true;

        let mut cache = self.appearance_cache.write().await;
        cache.insert(agent_id, appearance.clone());

        {
            let mut asset_cache = self.asset_cache.write().await;
            for (_, uuid_str) in &ruth_bakes {
                if let Ok(tex_id) = Uuid::parse_str(uuid_str) {
                    asset_cache.insert(tex_id, true);
                }
            }
        }

        info!("[AVATAR_FACTORY] Created appearance with Ruth baked defaults for agent {}", agent_id);
        appearance
    }

    pub async fn set_appearance_from_db(
        &self,
        agent_id: Uuid,
        texture_entries: &[(usize, Uuid)],
        visual_params: Vec<u8>,
    ) {
        let mut cache = self.appearance_cache.write().await;
        let appearance = cache.entry(agent_id).or_insert_with(|| AgentAppearanceData::new(agent_id));

        for &(idx, tex_id) in texture_entries {
            appearance.set_baked_texture(idx, tex_id);
        }
        if !visual_params.is_empty() {
            appearance.visual_params = visual_params;
        }
        appearance.is_validated = true;
        appearance.needs_rebake = false;

        info!("[AVATAR_FACTORY] Loaded DB appearance for agent {} ({} textures)", agent_id, texture_entries.len());
    }

    pub async fn set_visual_params(&self, agent_id: Uuid, visual_params: Vec<u8>) {
        let mut cache = self.appearance_cache.write().await;
        if let Some(appearance) = cache.get_mut(&agent_id) {
            appearance.visual_params = visual_params;
        }
    }

    pub async fn set_raw_texture_entry(&self, agent_id: Uuid, te: Vec<u8>) -> bool {
        let mut cache = self.appearance_cache.write().await;
        let appearance = match cache.get_mut(&agent_id) {
            Some(a) => a,
            None => {
                let a = AgentAppearanceData::new(agent_id);
                cache.insert(agent_id, a);
                cache.get_mut(&agent_id).unwrap()
            }
        };
        let changed = match &appearance.raw_texture_entry {
            Some(old_te) => old_te != &te,
            None => true,
        };
        appearance.raw_texture_entry = Some(te);
        appearance.needs_rebake = false;
        appearance.is_validated = true;
        changed
    }

    pub async fn set_serial_num(&self, agent_id: Uuid, serial: u32) {
        let mut cache = self.appearance_cache.write().await;
        if let Some(appearance) = cache.get_mut(&agent_id) {
            appearance.serial_num = serial;
        }
    }

    pub async fn next_serial_num(&self, agent_id: Uuid) -> u32 {
        let mut cache = self.appearance_cache.write().await;
        if let Some(appearance) = cache.get_mut(&agent_id) {
            appearance.serial_num += 1;
            appearance.serial_num
        } else {
            1
        }
    }

    pub async fn clear_raw_texture_entry(&self, agent_id: Uuid) {
        let mut cache = self.appearance_cache.write().await;
        if let Some(appearance) = cache.get_mut(&agent_id) {
            appearance.raw_texture_entry = None;
        }
    }

    /// Get appearance data for an agent
    pub async fn get_appearance(&self, agent_id: Uuid) -> Option<AgentAppearanceData> {
        let cache = self.appearance_cache.read().await;
        cache.get(&agent_id).cloned()
    }

    /// Validate baked texture cache for an agent
    /// Returns true if all baked textures are valid (cached)
    /// This is the critical function that OpenSim calls during CompleteMovement
    pub async fn validate_baked_texture_cache(&self, agent_id: Uuid) -> bool {
        let mut cache = self.appearance_cache.write().await;

        let appearance = match cache.get_mut(&agent_id) {
            Some(a) => a,
            None => {
                warn!("[AVATAR_FACTORY] No appearance data for agent {}", agent_id);
                return false;
            }
        };

        let mut hits = 0;
        let mut all_valid = true;
        let default_uuid = Uuid::parse_str(DEFAULT_AVATAR_TEXTURE).unwrap_or(Uuid::nil());

        for &face_index in &BAKE_INDICES {
            let texture_id = appearance.texture_ids.get(face_index).copied().unwrap_or(Uuid::nil());
            let cache_item = appearance.wearable_cache.get_mut(face_index);

            // Check if texture is default or nil (needs rebake)
            if texture_id.is_nil() || texture_id == default_uuid {
                debug!(
                    "[AVATAR_FACTORY] Bake face {} needs rebake (texture: {})",
                    face_index, texture_id
                );

                if let Some(item) = cache_item {
                    item.cache_id = Uuid::nil();
                    item.texture_id = default_uuid;
                }

                // Even with default textures, count as a hit
                // The viewer will handle rebaking
                hits += 1;
                continue;
            }

            // Check if texture is in our asset cache
            let asset_cached = {
                let asset_cache = self.asset_cache.read().await;
                asset_cache.get(&texture_id).copied().unwrap_or(false)
            };

            if asset_cached {
                debug!(
                    "[AVATAR_FACTORY] Bake face {} is cached (texture: {})",
                    face_index, texture_id
                );
                hits += 1;
            } else {
                debug!(
                    "[AVATAR_FACTORY] Bake face {} NOT cached (texture: {})",
                    face_index, texture_id
                );

                if let Some(item) = cache_item {
                    item.cache_id = Uuid::nil();
                    item.texture_id = default_uuid;
                }

                all_valid = false;
            }
        }

        appearance.is_validated = true;
        appearance.needs_rebake = !all_valid;

        info!(
            "[AVATAR_FACTORY] Validated baked textures for {}: {}/{} cached, needs_rebake={}",
            agent_id, hits, BAKE_INDICES.len(), appearance.needs_rebake
        );

        all_valid
    }

    /// Update baked texture from viewer's AgentSetAppearance
    pub async fn set_baked_textures(
        &self,
        agent_id: Uuid,
        texture_entries: &[(usize, Uuid)],
    ) {
        let mut cache = self.appearance_cache.write().await;

        let appearance = match cache.get_mut(&agent_id) {
            Some(a) => a,
            None => {
                // Create new appearance data if not exists
                let appearance = AgentAppearanceData::new(agent_id);
                cache.insert(agent_id, appearance);
                cache.get_mut(&agent_id).unwrap()
            }
        };

        for &(face_index, texture_id) in texture_entries {
            appearance.set_baked_texture(face_index, texture_id);

            let mut asset_cache = self.asset_cache.write().await;
            asset_cache.insert(texture_id, true);
        }

        appearance.needs_rebake = false;
        appearance.is_validated = true;

        info!(
            "[AVATAR_FACTORY] Updated {} baked textures for agent {}, needs_rebake=false",
            texture_entries.len(), agent_id
        );
    }

    pub async fn store_wearable_cache_from_viewer(
        &self,
        agent_id: Uuid,
        viewer_cache_items: &[(usize, Uuid)],
        baked_texture_ids: &[(usize, Uuid)],
    ) {
        let mut cache = self.appearance_cache.write().await;
        let appearance = match cache.get_mut(&agent_id) {
            Some(a) => a,
            None => {
                let a = AgentAppearanceData::new(agent_id);
                cache.insert(agent_id, a);
                cache.get_mut(&agent_id).unwrap()
            }
        };

        let default_uuid = Uuid::parse_str(DEFAULT_AVATAR_TEXTURE).unwrap_or(Uuid::nil());

        for &(tex_idx, cache_id) in viewer_cache_items {
            if tex_idx >= appearance.wearable_cache.len() { continue; }
            let tex_id = baked_texture_ids.iter()
                .find(|(idx, _)| *idx == tex_idx)
                .map(|(_, id)| *id)
                .unwrap_or(appearance.texture_ids.get(tex_idx).copied().unwrap_or(default_uuid));

            appearance.wearable_cache[tex_idx] = WearableCacheItem::with_cache_and_texture(cache_id, tex_id);

            if !tex_id.is_nil() && tex_id != default_uuid {
                appearance.texture_ids[tex_idx] = tex_id;
            }
        }

        info!("[AVATAR_FACTORY] Stored {} wearable cache items for agent {} (linked to {} baked textures)",
              viewer_cache_items.len(), agent_id, baked_texture_ids.len());
    }

    // Source: OpenSim C# LLClientView.cs:12309 HandleAgentTextureCached — checks cache_id match
    // Bug 15 fix: Added fallback — if cache_id doesn't match but we have a valid baked texture,
    // return it anyway and store viewer's cache_id for future exact matches.
    // This prevents the infinite rebake loop caused by all-miss AgentCachedTextureResponse.
    pub async fn get_cached_texture_for_index(&self, agent_id: Uuid, texture_index: usize, viewer_cache_id: Uuid) -> Uuid {
        let mut cache = self.appearance_cache.write().await;
        if let Some(appearance) = cache.get_mut(&agent_id) {
            let default_uuid = Uuid::parse_str(DEFAULT_AVATAR_TEXTURE).unwrap_or(Uuid::nil());

            if texture_index < appearance.wearable_cache.len() {
                let item = &appearance.wearable_cache[texture_index];
                debug!("[CACHE] face[{}]: stored_cache_id={}, viewer_cache_id={}, texture_id={}",
                      texture_index, item.cache_id, viewer_cache_id, item.texture_id);
                if !item.cache_id.is_nil() && item.cache_id == viewer_cache_id {
                    if !item.texture_id.is_nil() && item.texture_id != default_uuid {
                        return item.texture_id;
                    }
                }
            }

            info!("[CACHE] MISS face[{}]: CacheId mismatch (viewer={}, stored={})",
                  texture_index, viewer_cache_id,
                  if texture_index < appearance.wearable_cache.len() {
                      appearance.wearable_cache[texture_index].cache_id.to_string()
                  } else { "none".to_string() });
        }
        Uuid::nil()
    }

    /// Queue appearance save with normal priority
    pub async fn queue_appearance_save(&self, agent_id: Uuid) {
        self.queue_appearance_save_with_priority(agent_id, SavePriority::Normal).await;
    }

    /// Queue appearance save with specified priority
    pub async fn queue_appearance_save_with_priority(&self, agent_id: Uuid, priority: SavePriority) {
        // Check if we have a queue
        if let Some(tx) = &self.save_queue_tx {
            // Check for duplicate saves (debounce)
            let should_queue = {
                let mut pending = self.pending_saves.write().await;
                let now = std::time::Instant::now();

                if let Some(last_queued) = pending.get(&agent_id) {
                    // Don't queue if we queued recently (unless immediate)
                    if priority != SavePriority::Immediate && now.duration_since(*last_queued) < Duration::from_secs(1) {
                        debug!("[AVATAR_FACTORY] Skipping duplicate save for {} (debounced)", agent_id);
                        false
                    } else {
                        pending.insert(agent_id, now);
                        true
                    }
                } else {
                    pending.insert(agent_id, now);
                    true
                }
            };

            if should_queue {
                let request = AppearanceSaveRequest { agent_id, priority };
                if let Err(e) = tx.send(request).await {
                    warn!("[AVATAR_FACTORY] Failed to queue appearance save for {}: {}", agent_id, e);
                } else {
                    info!("[AVATAR_FACTORY] Queued {:?} appearance save for {}", priority, agent_id);
                }
            }
        } else {
            info!("[AVATAR_FACTORY] Saving appearance for agent {} (sync mode)", agent_id);
            Self::save_appearance_to_db(&self.appearance_cache, agent_id, self.db_connection.as_deref()).await;
        }
    }

    pub async fn force_save(&self, agent_id: Uuid) {
        self.flush_dirty_for_agent(agent_id);
        self.queue_appearance_save_with_priority(agent_id, SavePriority::Immediate).await;
    }

    /// Cache an asset texture
    pub async fn cache_texture(&self, texture_id: Uuid) {
        let mut asset_cache = self.asset_cache.write().await;
        asset_cache.insert(texture_id, true);
        debug!("[AVATAR_FACTORY] Cached texture {}", texture_id);
    }

    /// Check if a texture is cached
    pub async fn is_texture_cached(&self, texture_id: Uuid) -> bool {
        let asset_cache = self.asset_cache.read().await;
        asset_cache.get(&texture_id).copied().unwrap_or(false)
    }

    /// Clear appearance data for an agent (on logout)
    pub async fn clear_appearance(&self, agent_id: Uuid) {
        let mut cache = self.appearance_cache.write().await;
        cache.remove(&agent_id);
        self.clear_first_bake(agent_id);
        info!("[AVATAR_FACTORY] Cleared appearance data for agent {}", agent_id);
    }

    /// Get the texture IDs for AvatarAppearance message
    pub async fn get_appearance_textures(&self, agent_id: Uuid) -> Vec<Uuid> {
        let cache = self.appearance_cache.read().await;

        match cache.get(&agent_id) {
            Some(appearance) => appearance.texture_ids.clone(),
            None => {
                let default_uuid = Uuid::parse_str(DEFAULT_AVATAR_TEXTURE).unwrap_or(Uuid::nil());
                vec![default_uuid; TEXTURE_COUNT]
            }
        }
    }

    /// Get the wearable cache items for AgentCachedTextureResponse
    pub async fn get_wearable_cache(&self, agent_id: Uuid) -> Vec<WearableCacheItem> {
        let cache = self.appearance_cache.read().await;

        match cache.get(&agent_id) {
            Some(appearance) => appearance.wearable_cache.clone(),
            None => {
                let mut items = Vec::with_capacity(TEXTURE_COUNT);
                for _ in 0..TEXTURE_COUNT {
                    items.push(WearableCacheItem::new());
                }
                items
            }
        }
    }
}

impl Default for AvatarFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AvatarFactory {
    fn clone(&self) -> Self {
        Self {
            appearance_cache: self.appearance_cache.clone(),
            asset_cache: self.asset_cache.clone(),
            save_queue_tx: self.save_queue_tx.clone(),
            pending_saves: self.pending_saves.clone(),
            db_connection: self.db_connection.clone(),
            dirty_save: self.dirty_save.clone(),
            dirty_send: self.dirty_send.clone(),
            last_broadcast: self.last_broadcast.clone(),
            first_bake_complete: self.first_bake_complete.clone(),
        }
    }
}

pub fn get_bakes_bytes(raw_texture_entry: &[u8]) -> Vec<u8> {
    if raw_texture_entry.len() < 16 {
        return raw_texture_entry.to_vec();
    }

    let default_uuid_bytes = raw_texture_entry[0..16].to_vec();
    let default_uuid = Uuid::from_slice(&default_uuid_bytes).unwrap_or(Uuid::nil());

    let overrides = parse_texture_entry_uuids(raw_texture_entry);

    let mut baked_faces: Vec<(usize, Uuid)> = Vec::new();
    for &(idx, tex_id) in &overrides {
        if BAKE_INDICES.contains(&idx) && !tex_id.is_nil() {
            baked_faces.push((idx, tex_id));
        }
    }

    let mut te = Vec::with_capacity(200);
    te.extend_from_slice(&default_uuid_bytes);

    for &(idx, tex_id) in &baked_faces {
        if tex_id != default_uuid {
            let mut face_bits: u64 = 1u64 << idx;
            loop {
                let mut byte = (face_bits & 0x7F) as u8;
                face_bits >>= 7;
                if face_bits > 0 { byte |= 0x80; }
                te.push(byte);
                if face_bits == 0 { break; }
            }
            te.extend_from_slice(tex_id.as_bytes());
        }
    }

    te.push(0);

    let remaining_start = find_color_section_start(raw_texture_entry);
    if remaining_start < raw_texture_entry.len() {
        te.extend_from_slice(&raw_texture_entry[remaining_start..]);
    } else {
        te.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // RGBA
        te.push(0);
        te.extend_from_slice(&1.0f32.to_le_bytes()); // RepeatU
        te.push(0);
        te.extend_from_slice(&1.0f32.to_le_bytes()); // RepeatV
        te.push(0);
        te.extend_from_slice(&0i16.to_le_bytes()); // OffsetU
        te.push(0);
        te.extend_from_slice(&0i16.to_le_bytes()); // OffsetV
        te.push(0);
        te.extend_from_slice(&0i16.to_le_bytes()); // Rotation
        te.push(0);
        te.push(0); te.push(0); // Material
        te.push(0); te.push(0); // Media
        te.push(0); te.push(0); // Glow
        te.extend_from_slice(&[0u8; 16]); // MaterialID
        te.push(0);
    }

    te
}

fn find_color_section_start(raw: &[u8]) -> usize {
    if raw.len() < 16 { return raw.len(); }
    let mut pos = 16;
    while pos < raw.len() {
        if raw[pos] == 0 {
            return pos + 1;
        }
        loop {
            if pos >= raw.len() { return raw.len(); }
            let b = raw[pos];
            pos += 1;
            if b & 0x80 == 0 { break; }
        }
        if pos + 16 > raw.len() { return raw.len(); }
        pos += 16;
    }
    raw.len()
}

pub fn get_ruth_visual_params() -> [u8; 218] {
    let mut params = [128u8; 218];
    params[1] = 61;
    params[2] = 127;
    params[4] = 84;
    params[5] = 150;
    params[6] = 187;
    params[7] = 97;
    params[8] = 74;
    params[10] = 33;
    params[11] = 61;
    params[12] = 150;
    params[13] = 140;
    params[14] = 165;
    params[15] = 102;
    params[16] = 97;
    params[17] = 142;
    params[18] = 155;
    params[19] = 135;
    params[20] = 140;
    params[21] = 28;
    params[22] = 130;
    params[23] = 89;
    params[24] = 100;
    params[25] = 155;
    params[26] = 74;
    params[27] = 145;
    params[28] = 155;
    params[33] = 33;
    params[34] = 40;
    params[80] = 85;
    params[105] = 132;
    params[155] = 90;
    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_avatar_factory_creation() {
        let factory = AvatarFactory::new();
        let agent_id = Uuid::new_v4();

        let appearance = factory.create_appearance(agent_id).await;
        assert_eq!(appearance.agent_id, agent_id);
        assert_eq!(appearance.texture_ids.len(), TEXTURE_COUNT);
        assert!(appearance.needs_rebake);
    }

    #[tokio::test]
    async fn test_validate_baked_textures() {
        let factory = AvatarFactory::new();
        let agent_id = Uuid::new_v4();

        factory.create_appearance(agent_id).await;

        // With default textures, validation should succeed (viewer handles rebake)
        let valid = factory.validate_baked_texture_cache(agent_id).await;
        assert!(valid);
    }

    #[tokio::test]
    async fn test_set_baked_textures() {
        let factory = AvatarFactory::new();
        let agent_id = Uuid::new_v4();

        factory.create_appearance(agent_id).await;

        let texture_id = Uuid::new_v4();
        factory.set_baked_textures(agent_id, &[(8, texture_id)]).await;

        let appearance = factory.get_appearance(agent_id).await.unwrap();
        assert_eq!(appearance.texture_ids[8], texture_id);
    }

    #[tokio::test]
    async fn test_bake_indices() {
        assert_eq!(BAKE_INDICES.len(), 11);
        assert_eq!(BAKE_INDICES[0], 8);  // TEX_HEAD_BAKED
        assert_eq!(BAKE_INDICES[1], 9);  // TEX_UPPER_BAKED
        assert_eq!(BAKE_INDICES[2], 10); // TEX_LOWER_BAKED
        assert_eq!(BAKE_INDICES[3], 11); // TEX_EYES_BAKED
        assert_eq!(BAKE_INDICES[4], 19); // TEX_SKIRT_BAKED
        assert_eq!(BAKE_INDICES[5], 20); // TEX_HAIR_BAKED
        assert_eq!(BAKE_INDICES[6], 40); // TEX_LEFT_ARM_BAKED
        assert_eq!(BAKE_INDICES[7], 41); // TEX_LEFT_LEG_BAKED
        assert_eq!(BAKE_INDICES[8], 42); // TEX_AUX1_BAKED
        assert_eq!(BAKE_INDICES[9], 43); // TEX_AUX2_BAKED
        assert_eq!(BAKE_INDICES[10], 44); // TEX_AUX3_BAKED
    }
}
