use super::super::AIError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingModel {
    AllMiniLML6V2,
    AllMpnetBaseV2,
    ParaphraseMultilingualMiniLML12V2,
    Custom(String),
}

impl EmbeddingModel {
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "all-minilm-l6-v2" => EmbeddingModel::AllMiniLML6V2,
            "all-mpnet-base-v2" => EmbeddingModel::AllMpnetBaseV2,
            "paraphrase-multilingual-minilm-l12-v2" => {
                EmbeddingModel::ParaphraseMultilingualMiniLML12V2
            }
            other => EmbeddingModel::Custom(other.to_string()),
        }
    }

    pub fn dimension(&self) -> usize {
        match self {
            EmbeddingModel::AllMiniLML6V2 => 384,
            EmbeddingModel::AllMpnetBaseV2 => 768,
            EmbeddingModel::ParaphraseMultilingualMiniLML12V2 => 384,
            EmbeddingModel::Custom(_) => 384,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            EmbeddingModel::AllMiniLML6V2 => "all-MiniLM-L6-v2",
            EmbeddingModel::AllMpnetBaseV2 => "all-mpnet-base-v2",
            EmbeddingModel::ParaphraseMultilingualMiniLML12V2 => {
                "paraphrase-multilingual-MiniLM-L12-v2"
            }
            EmbeddingModel::Custom(name) => name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEmbedding {
    pub content_id: Uuid,
    pub embedding: Vec<f32>,
    pub model: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub content_hash: String,
}

#[derive(Debug)]
pub struct EmbeddingService {
    model: EmbeddingModel,
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    cache_enabled: bool,
    max_cache_size: usize,
}

impl EmbeddingService {
    pub async fn new(
        model: EmbeddingModel,
        cache_enabled: bool,
        max_cache_size: usize,
    ) -> Result<Arc<Self>, AIError> {
        let service = Self {
            model,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_enabled,
            max_cache_size,
        };

        Ok(Arc::new(service))
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, AIError> {
        let cache_key = self.compute_cache_key(text);

        if self.cache_enabled {
            let cache = self.cache.read().await;
            if let Some(embedding) = cache.get(&cache_key) {
                return Ok(embedding.clone());
            }
        }

        let embedding = self.compute_embedding(text).await?;

        if self.cache_enabled {
            let mut cache = self.cache.write().await;

            if cache.len() >= self.max_cache_size {
                let keys_to_remove: Vec<String> =
                    cache.keys().take(cache.len() / 10).cloned().collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }

            cache.insert(cache_key, embedding.clone());
        }

        Ok(embedding)
    }

    async fn compute_embedding(&self, text: &str) -> Result<Vec<f32>, AIError> {
        let dimension = self.model.dimension();
        let mut embedding = vec![0.0f32; dimension];

        let chars: Vec<char> = text.chars().collect();
        let text_len = chars.len().max(1) as f32;

        for (i, ch) in chars.iter().enumerate() {
            let char_val = (*ch as u32) as f32 / 65536.0;
            let pos = i % dimension;
            embedding[pos] += char_val / text_len;
        }

        let word_count = text.split_whitespace().count() as f32;
        if dimension > 0 {
            embedding[0] = (word_count / 100.0).min(1.0);
        }

        let has_punctuation = text.contains('.') || text.contains('!') || text.contains('?');
        if dimension > 1 {
            embedding[1] = if has_punctuation { 0.8 } else { 0.2 };
        }

        let avg_word_len = if word_count > 0.0 {
            text.len() as f32 / word_count
        } else {
            0.0
        };
        if dimension > 2 {
            embedding[2] = (avg_word_len / 15.0).min(1.0);
        }

        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in embedding.iter_mut() {
                *val /= magnitude;
            }
        }

        Ok(embedding)
    }

    fn compute_cache_key(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        self.model.name().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, AIError> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            embeddings.push(self.embed_text(text).await?);
        }
        Ok(embeddings)
    }

    pub fn similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }

    pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::MAX;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    pub async fn find_most_similar(
        &self,
        query_embedding: &[f32],
        candidate_embeddings: &[(Uuid, Vec<f32>)],
        top_k: usize,
    ) -> Vec<(Uuid, f32)> {
        let mut similarities: Vec<(Uuid, f32)> = candidate_embeddings
            .iter()
            .map(|(id, emb)| (*id, self.similarity(query_embedding, emb)))
            .collect();

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(top_k);

        similarities
    }

    pub fn get_model(&self) -> &EmbeddingModel {
        &self.model
    }

    pub async fn get_cache_size(&self) -> usize {
        self.cache.read().await.len()
    }

    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    pub async fn create_content_embedding(
        &self,
        content_id: Uuid,
        text: &str,
    ) -> Result<ContentEmbedding, AIError> {
        let embedding = self.embed_text(text).await?;

        Ok(ContentEmbedding {
            content_id,
            embedding,
            model: self.model.name().to_string(),
            created_at: chrono::Utc::now(),
            content_hash: self.compute_cache_key(text),
        })
    }
}
