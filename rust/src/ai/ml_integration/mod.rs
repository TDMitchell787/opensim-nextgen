pub mod embeddings;
pub mod llm_client;
pub mod onnx_predictor;
pub mod vector_store;
pub mod quality_service;
pub mod collaborative_filtering;

use crate::database::DatabaseManager;
use super::AIError;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

pub use embeddings::{EmbeddingService, ContentEmbedding, EmbeddingModel};
pub use llm_client::{LocalLLMClient, LLMConfig, LLMResponse};
pub use onnx_predictor::{ONNXPredictor, PredictorConfig, PredictionResult};
pub use vector_store::{
    VectorStore, VectorStoreConfig, VectorEntry, SearchResult as VectorSearchResult,
    ContentType, EmbeddingPipeline, AssetEmbeddingRequest, RegionEmbeddingRequest,
    LandmarkEmbeddingRequest, UserProfileEmbeddingRequest, EmbeddingBatchResult,
    VectorStoreStats,
};
pub use quality_service::{
    QualityService, QualityServiceConfig, QualityAssessment, QualityGrade,
    UploadQualityRequest, RegionQualityRequest, RegionQualityReport,
    AnomalyAlert, AnomalyStatus, RiskLevel, QualitySuggestion,
};
pub use collaborative_filtering::{
    CollaborativeRecommender, RecommenderConfig,
    ContentRecommendation as CFContentRecommendation,
    SocialRecommendation, UserActivity, ActivityType, UserProfile as CFUserProfile,
    EngagementMetrics, ContentItemType, RecommendationReason, RecommenderStats,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLIntegrationConfig {
    pub enabled: bool,
    pub embedding_model: String,
    pub llm_endpoint: String,
    pub llm_model: String,
    pub onnx_models_path: String,
    pub cache_embeddings: bool,
    pub max_embedding_cache_size: usize,
    pub llm_timeout_seconds: u64,
    pub llm_max_tokens: usize,
    pub llm_temperature: f32,
}

impl Default for MLIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            embedding_model: "all-MiniLM-L6-v2".to_string(),
            llm_endpoint: "http://localhost:11434".to_string(),
            llm_model: "mistral".to_string(),
            onnx_models_path: "./models/".to_string(),
            cache_embeddings: true,
            max_embedding_cache_size: 10000,
            llm_timeout_seconds: 30,
            llm_max_tokens: 2048,
            llm_temperature: 0.7,
        }
    }
}

#[derive(Debug)]
pub struct MLIntegrationManager {
    config: MLIntegrationConfig,
    embedding_service: Option<Arc<EmbeddingService>>,
    llm_client: Option<Arc<LocalLLMClient>>,
    onnx_predictor: Option<Arc<ONNXPredictor>>,
    db: Arc<DatabaseManager>,
    status: Arc<RwLock<MLServiceStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLServiceStatus {
    pub embedding_service_available: bool,
    pub llm_service_available: bool,
    pub onnx_service_available: bool,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub error_count: usize,
    pub successful_requests: usize,
}

impl Default for MLServiceStatus {
    fn default() -> Self {
        Self {
            embedding_service_available: false,
            llm_service_available: false,
            onnx_service_available: false,
            last_health_check: chrono::Utc::now(),
            error_count: 0,
            successful_requests: 0,
        }
    }
}

impl MLIntegrationManager {
    pub async fn new(
        config: MLIntegrationConfig,
        db: Arc<DatabaseManager>,
    ) -> Result<Arc<Self>, AIError> {
        let mut manager = Self {
            config: config.clone(),
            embedding_service: None,
            llm_client: None,
            onnx_predictor: None,
            db,
            status: Arc::new(RwLock::new(MLServiceStatus::default())),
        };

        if config.enabled {
            manager.initialize_services().await?;
        }

        Ok(Arc::new(manager))
    }

    async fn initialize_services(&mut self) -> Result<(), AIError> {
        if let Ok(embedding) = EmbeddingService::new(
            EmbeddingModel::from_name(&self.config.embedding_model),
            self.config.cache_embeddings,
            self.config.max_embedding_cache_size,
        ).await {
            self.embedding_service = Some(embedding);
            self.status.write().await.embedding_service_available = true;
        }

        let llm_config = LLMConfig {
            endpoint: self.config.llm_endpoint.clone(),
            model: self.config.llm_model.clone(),
            timeout_seconds: self.config.llm_timeout_seconds,
            max_tokens: self.config.llm_max_tokens,
            temperature: self.config.llm_temperature,
            provider: "ollama".to_string(),
            api_key: String::new(),
            context_window: 32768,
        };

        if let Ok(llm) = LocalLLMClient::new(llm_config).await {
            if llm.health_check().await {
                self.llm_client = Some(llm);
                self.status.write().await.llm_service_available = true;
            }
        }

        let predictor_config = PredictorConfig {
            models_path: self.config.onnx_models_path.clone(),
            ..Default::default()
        };

        if let Ok(predictor) = ONNXPredictor::new(predictor_config).await {
            self.onnx_predictor = Some(predictor);
            self.status.write().await.onnx_service_available = true;
        }

        self.status.write().await.last_health_check = chrono::Utc::now();

        Ok(())
    }

    pub async fn get_embedding(&self, text: &str) -> Result<Vec<f32>, AIError> {
        if let Some(service) = &self.embedding_service {
            let embedding = service.embed_text(text).await?;
            self.record_success().await;
            Ok(embedding)
        } else {
            self.record_error().await;
            Err(AIError::EngineNotAvailable("Embedding service not available".to_string()))
        }
    }

    pub async fn generate_text(&self, prompt: &str) -> Result<String, AIError> {
        if let Some(client) = &self.llm_client {
            let response = client.generate(prompt).await?;
            self.record_success().await;
            Ok(response.text)
        } else {
            self.record_error().await;
            Err(AIError::EngineNotAvailable("LLM service not available".to_string()))
        }
    }

    pub async fn predict_quality(&self, features: &[f32]) -> Result<f32, AIError> {
        if let Some(predictor) = &self.onnx_predictor {
            let result = predictor.predict_quality(features).await?;
            self.record_success().await;
            Ok(result)
        } else {
            self.record_error().await;
            Err(AIError::EngineNotAvailable("ONNX predictor not available".to_string()))
        }
    }

    pub async fn analyze_content(&self, content: &str) -> Result<ContentAnalysis, AIError> {
        let embedding = if self.embedding_service.is_some() {
            Some(self.get_embedding(content).await?)
        } else {
            None
        };

        let llm_analysis = if self.llm_client.is_some() {
            let prompt = format!(
                "Analyze the following content and provide a brief summary of its quality, style, and key characteristics:\n\n{}",
                content
            );
            Some(self.generate_text(&prompt).await?)
        } else {
            None
        };

        let quality_score = if let Some(emb) = &embedding {
            if self.onnx_predictor.is_some() {
                Some(self.predict_quality(emb).await?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(ContentAnalysis {
            embedding,
            llm_analysis,
            quality_score,
            analyzed_at: chrono::Utc::now(),
        })
    }

    pub async fn suggest_improvements(&self, content: &str) -> Result<Vec<String>, AIError> {
        if let Some(client) = &self.llm_client {
            let prompt = format!(
                "Analyze the following content and suggest specific improvements. \
                Return each suggestion on a new line:\n\n{}",
                content
            );

            let response = client.generate(&prompt).await?;
            let suggestions: Vec<String> = response.text
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect();

            self.record_success().await;
            Ok(suggestions)
        } else {
            self.record_error().await;
            Err(AIError::EngineNotAvailable("LLM service not available".to_string()))
        }
    }

    pub async fn semantic_search(
        &self,
        query: &str,
        documents: &[(&str, uuid::Uuid)],
        top_k: usize,
    ) -> Result<Vec<(uuid::Uuid, f32)>, AIError> {
        if let Some(service) = &self.embedding_service {
            let query_embedding = service.embed_text(query).await?;

            let mut scored: Vec<(uuid::Uuid, f32)> = Vec::new();

            for (doc_text, doc_id) in documents {
                let doc_embedding = service.embed_text(doc_text).await?;
                let similarity = self.cosine_similarity(&query_embedding, &doc_embedding);
                scored.push((*doc_id, similarity));
            }

            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            scored.truncate(top_k);

            self.record_success().await;
            Ok(scored)
        } else {
            self.record_error().await;
            Err(AIError::EngineNotAvailable("Embedding service not available".to_string()))
        }
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
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

    pub async fn get_status(&self) -> MLServiceStatus {
        self.status.read().await.clone()
    }

    pub async fn health_check(&self) -> bool {
        let mut status = self.status.write().await;

        if let Some(client) = &self.llm_client {
            status.llm_service_available = client.health_check().await;
        }

        status.embedding_service_available = self.embedding_service.is_some();
        status.onnx_service_available = self.onnx_predictor.is_some();
        status.last_health_check = chrono::Utc::now();

        status.embedding_service_available || status.llm_service_available || status.onnx_service_available
    }

    async fn record_success(&self) {
        let mut status = self.status.write().await;
        status.successful_requests += 1;
    }

    async fn record_error(&self) {
        let mut status = self.status.write().await;
        status.error_count += 1;
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    pub embedding: Option<Vec<f32>>,
    pub llm_analysis: Option<String>,
    pub quality_score: Option<f32>,
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}
