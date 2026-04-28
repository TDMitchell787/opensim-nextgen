use super::super::AIError;
use super::embeddings::{ContentEmbedding, EmbeddingService};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    Asset,
    Region,
    UserProfile,
    NPC,
    Script,
    Object,
    Landmark,
    Event,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Asset => "asset",
            ContentType::Region => "region",
            ContentType::UserProfile => "user_profile",
            ContentType::NPC => "npc",
            ContentType::Script => "script",
            ContentType::Object => "object",
            ContentType::Landmark => "landmark",
            ContentType::Event => "event",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "asset" => Some(ContentType::Asset),
            "region" => Some(ContentType::Region),
            "user_profile" | "userprofile" | "user" => Some(ContentType::UserProfile),
            "npc" => Some(ContentType::NPC),
            "script" => Some(ContentType::Script),
            "object" => Some(ContentType::Object),
            "landmark" => Some(ContentType::Landmark),
            "event" => Some(ContentType::Event),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: Uuid,
    pub content_type: ContentType,
    pub name: String,
    pub description: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub content_type: ContentType,
    pub name: String,
    pub description: String,
    pub similarity: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub max_entries: usize,
    pub similarity_threshold: f32,
    pub enable_persistence: bool,
    pub persistence_path: Option<String>,
    pub auto_cleanup_days: u32,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            max_entries: 100_000,
            similarity_threshold: 0.5,
            enable_persistence: false,
            persistence_path: None,
            auto_cleanup_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreStats {
    pub total_entries: usize,
    pub entries_by_type: HashMap<String, usize>,
    pub average_embedding_dimension: usize,
    pub memory_usage_bytes: usize,
    pub last_updated: DateTime<Utc>,
}

pub struct VectorStore {
    entries: Arc<RwLock<HashMap<Uuid, VectorEntry>>>,
    type_index: Arc<RwLock<HashMap<ContentType, Vec<Uuid>>>>,
    embedding_service: Arc<EmbeddingService>,
    config: VectorStoreConfig,
}

impl VectorStore {
    pub async fn new(
        embedding_service: Arc<EmbeddingService>,
        config: VectorStoreConfig,
    ) -> Result<Arc<Self>, AIError> {
        let store = Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            embedding_service,
            config,
        };

        if store.config.enable_persistence {
            if let Some(path) = &store.config.persistence_path {
                if let Err(e) = store.load_from_disk(path).await {
                    warn!("Failed to load vector store from disk: {}", e);
                }
            }
        }

        info!(
            "VectorStore initialized with max {} entries",
            store.config.max_entries
        );
        Ok(Arc::new(store))
    }

    pub async fn add_entry(
        &self,
        id: Uuid,
        content_type: ContentType,
        name: String,
        description: String,
        metadata: HashMap<String, String>,
    ) -> Result<(), AIError> {
        let text = format!("{} {}", name, description);
        let embedding = self.embedding_service.embed_text(&text).await?;

        let entry = VectorEntry {
            id,
            content_type,
            name,
            description,
            embedding,
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut entries = self.entries.write().await;

        if entries.len() >= self.config.max_entries {
            self.evict_oldest_entries(&mut entries).await;
        }

        entries.insert(id, entry);

        let mut type_index = self.type_index.write().await;
        type_index
            .entry(content_type)
            .or_insert_with(Vec::new)
            .push(id);

        debug!(
            "Added entry {} of type {:?} to vector store",
            id, content_type
        );
        Ok(())
    }

    pub async fn add_entry_with_embedding(
        &self,
        id: Uuid,
        content_type: ContentType,
        name: String,
        description: String,
        embedding: Vec<f32>,
        metadata: HashMap<String, String>,
    ) -> Result<(), AIError> {
        let entry = VectorEntry {
            id,
            content_type,
            name,
            description,
            embedding,
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut entries = self.entries.write().await;

        if entries.len() >= self.config.max_entries {
            self.evict_oldest_entries(&mut entries).await;
        }

        entries.insert(id, entry);

        let mut type_index = self.type_index.write().await;
        type_index
            .entry(content_type)
            .or_insert_with(Vec::new)
            .push(id);

        Ok(())
    }

    pub async fn update_entry(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), AIError> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(&id) {
            if let Some(n) = name {
                entry.name = n;
            }
            if let Some(d) = description.clone() {
                entry.description = d;
            }
            if let Some(m) = metadata {
                entry.metadata = m;
            }

            if description.is_some() {
                let text = format!("{} {}", entry.name, entry.description);
                entry.embedding = self.embedding_service.embed_text(&text).await?;
            }

            entry.updated_at = Utc::now();
            Ok(())
        } else {
            Err(AIError::EngineNotAvailable(format!(
                "Entry {} not found",
                id
            )))
        }
    }

    pub async fn remove_entry(&self, id: Uuid) -> Result<Option<VectorEntry>, AIError> {
        let mut entries = self.entries.write().await;
        let removed = entries.remove(&id);

        if let Some(entry) = &removed {
            let mut type_index = self.type_index.write().await;
            if let Some(ids) = type_index.get_mut(&entry.content_type) {
                ids.retain(|eid| *eid != id);
            }
        }

        Ok(removed)
    }

    pub async fn get_entry(&self, id: Uuid) -> Option<VectorEntry> {
        let entries = self.entries.read().await;
        entries.get(&id).cloned()
    }

    pub async fn search(
        &self,
        query: &str,
        content_type: Option<ContentType>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, AIError> {
        let query_embedding = self.embedding_service.embed_text(query).await?;
        self.search_by_embedding(&query_embedding, content_type, limit)
            .await
    }

    pub async fn search_by_embedding(
        &self,
        query_embedding: &[f32],
        content_type: Option<ContentType>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, AIError> {
        let entries = self.entries.read().await;

        let candidates: Vec<&VectorEntry> = if let Some(ct) = content_type {
            let type_index = self.type_index.read().await;
            if let Some(ids) = type_index.get(&ct) {
                ids.iter().filter_map(|id| entries.get(id)).collect()
            } else {
                Vec::new()
            }
        } else {
            entries.values().collect()
        };

        let mut scored: Vec<SearchResult> = candidates
            .iter()
            .map(|entry| {
                let similarity = self
                    .embedding_service
                    .similarity(query_embedding, &entry.embedding);
                SearchResult {
                    id: entry.id,
                    content_type: entry.content_type,
                    name: entry.name.clone(),
                    description: entry.description.clone(),
                    similarity,
                    metadata: entry.metadata.clone(),
                }
            })
            .filter(|result| result.similarity >= self.config.similarity_threshold)
            .collect();

        scored.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(limit);

        Ok(scored)
    }

    pub async fn find_similar(&self, id: Uuid, limit: usize) -> Result<Vec<SearchResult>, AIError> {
        let entries = self.entries.read().await;

        let entry = entries
            .get(&id)
            .ok_or_else(|| AIError::EngineNotAvailable(format!("Entry {} not found", id)))?;

        let query_embedding = entry.embedding.clone();
        let content_type = entry.content_type;
        drop(entries);

        let mut results = self
            .search_by_embedding(&query_embedding, Some(content_type), limit + 1)
            .await?;
        results.retain(|r| r.id != id);
        results.truncate(limit);

        Ok(results)
    }

    pub async fn get_stats(&self) -> VectorStoreStats {
        let entries = self.entries.read().await;

        let mut entries_by_type: HashMap<String, usize> = HashMap::new();
        let mut total_dimension: usize = 0;

        for entry in entries.values() {
            *entries_by_type
                .entry(entry.content_type.as_str().to_string())
                .or_insert(0) += 1;
            total_dimension += entry.embedding.len();
        }

        let avg_dimension = if entries.is_empty() {
            0
        } else {
            total_dimension / entries.len()
        };

        let memory_bytes = entries
            .values()
            .map(|e| {
                std::mem::size_of::<VectorEntry>()
                    + e.embedding.len() * std::mem::size_of::<f32>()
                    + e.name.len()
                    + e.description.len()
            })
            .sum();

        VectorStoreStats {
            total_entries: entries.len(),
            entries_by_type,
            average_embedding_dimension: avg_dimension,
            memory_usage_bytes: memory_bytes,
            last_updated: Utc::now(),
        }
    }

    pub async fn get_entries_by_type(&self, content_type: ContentType) -> Vec<VectorEntry> {
        let entries = self.entries.read().await;
        let type_index = self.type_index.read().await;

        if let Some(ids) = type_index.get(&content_type) {
            ids.iter()
                .filter_map(|id| entries.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        let mut type_index = self.type_index.write().await;

        entries.clear();
        type_index.clear();

        info!("Vector store cleared");
    }

    pub async fn clear_type(&self, content_type: ContentType) {
        let mut entries = self.entries.write().await;
        let mut type_index = self.type_index.write().await;

        if let Some(ids) = type_index.remove(&content_type) {
            for id in ids {
                entries.remove(&id);
            }
        }

        info!("Cleared all entries of type {:?}", content_type);
    }

    async fn evict_oldest_entries(&self, entries: &mut HashMap<Uuid, VectorEntry>) {
        let evict_count = entries.len() / 10;

        let mut by_age: Vec<(Uuid, DateTime<Utc>)> = entries
            .iter()
            .map(|(id, entry)| (*id, entry.updated_at))
            .collect();

        by_age.sort_by(|a, b| a.1.cmp(&b.1));

        for (id, _) in by_age.into_iter().take(evict_count) {
            entries.remove(&id);
        }

        info!("Evicted {} oldest entries from vector store", evict_count);
    }

    async fn load_from_disk(&self, _path: &str) -> Result<(), AIError> {
        Ok(())
    }

    pub async fn save_to_disk(&self, path: &str) -> Result<(), AIError> {
        let entries = self.entries.read().await;
        let data = serde_json::to_string(&*entries)
            .map_err(|e| AIError::ConfigurationError(format!("Failed to serialize: {}", e)))?;

        tokio::fs::write(path, data)
            .await
            .map_err(|e| AIError::ConfigurationError(format!("Failed to write: {}", e)))?;

        info!("Saved {} entries to {}", entries.len(), path);
        Ok(())
    }
}

pub struct EmbeddingPipeline {
    vector_store: Arc<VectorStore>,
    embedding_service: Arc<EmbeddingService>,
    batch_size: usize,
}

impl EmbeddingPipeline {
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_service: Arc<EmbeddingService>,
        batch_size: usize,
    ) -> Self {
        Self {
            vector_store,
            embedding_service,
            batch_size,
        }
    }

    pub async fn process_assets(
        &self,
        assets: Vec<AssetEmbeddingRequest>,
    ) -> Result<EmbeddingBatchResult, AIError> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors: Vec<String> = Vec::new();

        for chunk in assets.chunks(self.batch_size) {
            for asset in chunk {
                match self
                    .vector_store
                    .add_entry(
                        asset.asset_id,
                        ContentType::Asset,
                        asset.name.clone(),
                        asset.description.clone(),
                        asset.metadata.clone(),
                    )
                    .await
                {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        error_count += 1;
                        errors.push(format!("Asset {}: {}", asset.asset_id, e));
                    }
                }
            }
        }

        info!(
            "Processed {} assets: {} success, {} errors",
            success_count + error_count,
            success_count,
            error_count
        );

        Ok(EmbeddingBatchResult {
            success_count,
            error_count,
            errors,
            processed_at: Utc::now(),
        })
    }

    pub async fn process_regions(
        &self,
        regions: Vec<RegionEmbeddingRequest>,
    ) -> Result<EmbeddingBatchResult, AIError> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors: Vec<String> = Vec::new();

        for chunk in regions.chunks(self.batch_size) {
            for region in chunk {
                let description = format!(
                    "{} Located at ({}, {}). {}",
                    region.name, region.x, region.y, region.description
                );

                match self
                    .vector_store
                    .add_entry(
                        region.region_id,
                        ContentType::Region,
                        region.name.clone(),
                        description,
                        region.metadata.clone(),
                    )
                    .await
                {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        error_count += 1;
                        errors.push(format!("Region {}: {}", region.region_id, e));
                    }
                }
            }
        }

        info!(
            "Processed {} regions: {} success, {} errors",
            success_count + error_count,
            success_count,
            error_count
        );

        Ok(EmbeddingBatchResult {
            success_count,
            error_count,
            errors,
            processed_at: Utc::now(),
        })
    }

    pub async fn process_landmarks(
        &self,
        landmarks: Vec<LandmarkEmbeddingRequest>,
    ) -> Result<EmbeddingBatchResult, AIError> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors: Vec<String> = Vec::new();

        for landmark in landmarks {
            let description = format!(
                "{} in region {}. {}",
                landmark.name, landmark.region_name, landmark.description
            );

            match self
                .vector_store
                .add_entry(
                    landmark.landmark_id,
                    ContentType::Landmark,
                    landmark.name.clone(),
                    description,
                    landmark.metadata.clone(),
                )
                .await
            {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    errors.push(format!("Landmark {}: {}", landmark.landmark_id, e));
                }
            }
        }

        Ok(EmbeddingBatchResult {
            success_count,
            error_count,
            errors,
            processed_at: Utc::now(),
        })
    }

    pub async fn process_user_profiles(
        &self,
        profiles: Vec<UserProfileEmbeddingRequest>,
    ) -> Result<EmbeddingBatchResult, AIError> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors: Vec<String> = Vec::new();

        for profile in profiles {
            let description = format!(
                "{} Interests: {}. About: {}",
                profile.display_name,
                profile.interests.join(", "),
                profile.about_text
            );

            match self
                .vector_store
                .add_entry(
                    profile.user_id,
                    ContentType::UserProfile,
                    profile.display_name.clone(),
                    description,
                    profile.metadata.clone(),
                )
                .await
            {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    errors.push(format!("Profile {}: {}", profile.user_id, e));
                }
            }
        }

        Ok(EmbeddingBatchResult {
            success_count,
            error_count,
            errors,
            processed_at: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEmbeddingRequest {
    pub asset_id: Uuid,
    pub name: String,
    pub description: String,
    pub asset_type: String,
    pub creator_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionEmbeddingRequest {
    pub region_id: Uuid,
    pub name: String,
    pub description: String,
    pub x: i32,
    pub y: i32,
    pub maturity_rating: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandmarkEmbeddingRequest {
    pub landmark_id: Uuid,
    pub name: String,
    pub description: String,
    pub region_name: String,
    pub position: (f32, f32, f32),
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileEmbeddingRequest {
    pub user_id: Uuid,
    pub display_name: String,
    pub about_text: String,
    pub interests: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingBatchResult {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
    pub processed_at: DateTime<Utc>,
}
