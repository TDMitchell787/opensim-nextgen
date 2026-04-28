use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::ai::ml_integration::{
    ActivityType, AnomalyAlert, AssetEmbeddingRequest, CFContentRecommendation, CFUserProfile,
    CollaborativeRecommender, ContentItemType, ContentType, EmbeddingModel, EmbeddingPipeline,
    EmbeddingService, EngagementMetrics, LLMConfig, MLIntegrationConfig, MLIntegrationManager,
    ONNXPredictor, PredictorConfig, QualityAssessment, QualityService, QualityServiceConfig,
    RecommendationReason, RecommenderConfig, RecommenderStats, RegionEmbeddingRequest,
    RegionQualityReport, RegionQualityRequest, SocialRecommendation,
    UploadQualityRequest as QualityUploadRequest, UserActivity, VectorStore, VectorStoreConfig,
};
use crate::ai::npc_behavior::NPCProfile;
use crate::ai::npc_dialogue::{
    AssetDescriptionGenerator, AssetDescriptionRequest, AssetDescriptionResponse, DialogueConfig,
    DialogueContext, DialogueResponse, NPCDialogueEngine, TimeOfDay,
};
use crate::database::DatabaseManager;
use chrono::Utc;

#[derive(Clone)]
pub struct AiApiState {
    pub dialogue_engine: Arc<NPCDialogueEngine>,
    pub asset_generator: Arc<AssetDescriptionGenerator>,
    pub ml_manager: Option<Arc<MLIntegrationManager>>,
    pub vector_store: Option<Arc<VectorStore>>,
    pub embedding_pipeline: Option<Arc<EmbeddingPipeline>>,
    pub quality_service: Option<Arc<QualityService>>,
    pub recommender: Arc<CollaborativeRecommender>,
    pub db: Arc<DatabaseManager>,
}

#[derive(Debug, Serialize)]
pub struct AiApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub llm_available: bool,
}

impl<T: Serialize> AiApiResponse<T> {
    pub fn success(data: T, llm_available: bool) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            llm_available,
        }
    }

    pub fn error(message: String, llm_available: bool) -> AiApiResponse<()> {
        AiApiResponse {
            success: false,
            data: None,
            error: Some(message),
            llm_available,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DialogueRequest {
    pub npc_id: Uuid,
    pub speaker_id: Uuid,
    pub speaker_name: String,
    pub message: String,
    pub location: Option<String>,
    pub time_hour: Option<u32>,
    pub weather: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssetDescRequest {
    pub asset_id: Uuid,
    pub asset_name: String,
    pub asset_type: String,
    pub creator_name: Option<String>,
    pub existing_description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub prim_count: Option<u32>,
    pub script_count: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub ai_system: String,
    pub llm_available: bool,
    pub embedding_available: bool,
    pub vector_store_available: bool,
    pub vector_store_entries: usize,
    pub dialogue_engine_ready: bool,
    pub asset_generator_ready: bool,
    pub recommender_ready: bool,
    pub recommender_stats: Option<RecommenderStats>,
    pub uptime_seconds: u64,
}

pub fn create_ai_api_router(state: AiApiState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/dialogue", post(generate_dialogue))
        .route(
            "/dialogue/clear/:npc_id/:speaker_id",
            post(clear_conversation),
        )
        .route("/asset/describe", post(generate_asset_description))
        .route("/asset/describe/:asset_id", get(get_cached_description))
        .route("/search", post(semantic_search))
        .route("/search/similar/:id", get(find_similar))
        .route("/embed/assets", post(embed_assets_batch))
        .route("/embed/regions", post(embed_regions_batch))
        .route("/embed/stats", get(get_embedding_stats))
        .route("/quality/assess", post(assess_upload_quality))
        .route("/quality/region", post(assess_region_quality))
        .route("/quality/alerts", get(get_quality_alerts))
        .route(
            "/quality/alerts/:alert_id/acknowledge",
            post(acknowledge_quality_alert),
        )
        .route("/recommend/trending", get(get_trending))
        .route("/recommend/stats", get(get_recommender_stats))
        .route("/recommend/activity", post(record_activity))
        .route("/recommend/profile", post(update_user_profile))
        .route("/recommend/:user_id", get(get_content_recommendations))
        .route(
            "/recommend/:user_id/social",
            get(get_social_recommendations),
        )
        .route(
            "/recommend/:user_id/creators",
            get(get_creator_recommendations),
        )
        .route(
            "/recommend/:user_id/engagement",
            get(get_engagement_metrics),
        )
        .with_state(state)
}

async fn health_check(State(state): State<AiApiState>) -> Json<AiApiResponse<HealthStatus>> {
    let llm_available = state.dialogue_engine.is_llm_available();
    let dialogue_ready = state.dialogue_engine.health_check().await;

    let embedding_available = if let Some(ml) = &state.ml_manager {
        ml.health_check().await
    } else {
        false
    };

    let (vector_store_available, vector_store_entries) = if let Some(vs) = &state.vector_store {
        let stats = vs.get_stats().await;
        (true, stats.total_entries)
    } else {
        (false, 0)
    };

    let recommender_stats = state.recommender.get_stats().await;

    let status = HealthStatus {
        ai_system: "OpenSim Next AI API v2.1".to_string(),
        llm_available,
        embedding_available,
        vector_store_available,
        vector_store_entries,
        dialogue_engine_ready: dialogue_ready,
        asset_generator_ready: state.asset_generator.is_llm_available(),
        recommender_ready: true,
        recommender_stats: Some(recommender_stats),
        uptime_seconds: 0,
    };

    Json(AiApiResponse::success(status, llm_available))
}

async fn generate_dialogue(
    State(state): State<AiApiState>,
    Json(request): Json<DialogueRequest>,
) -> Result<Json<AiApiResponse<DialogueResponse>>, (StatusCode, Json<Value>)> {
    info!(
        "Generating dialogue for NPC {} from {}",
        request.npc_id, request.speaker_name
    );

    let npc_profile = get_or_create_npc_profile(&state.db, request.npc_id)
        .await
        .map_err(|e| {
            error!("Failed to get NPC profile: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to get NPC profile: {}", e) })),
            )
        })?;

    let context = DialogueContext {
        npc_id: request.npc_id,
        speaker_id: request.speaker_id,
        speaker_name: request.speaker_name,
        location: request.location.unwrap_or_else(|| "Unknown".to_string()),
        time_of_day: TimeOfDay::from_hour(request.time_hour.unwrap_or(12)),
        weather: request.weather,
        previous_interactions: Vec::new(),
        nearby_objects: Vec::new(),
        current_quest: None,
    };

    match state
        .dialogue_engine
        .generate_dialogue(&npc_profile, &context, &request.message)
        .await
    {
        Ok(response) => {
            info!("Generated dialogue in {}ms", response.processing_time_ms);
            Ok(Json(AiApiResponse::success(
                response,
                state.dialogue_engine.is_llm_available(),
            )))
        }
        Err(e) => {
            error!("Dialogue generation failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Dialogue generation failed: {}", e) })),
            ))
        }
    }
}

async fn clear_conversation(
    State(state): State<AiApiState>,
    Path((npc_id, speaker_id)): Path<(Uuid, Uuid)>,
) -> Json<AiApiResponse<String>> {
    state
        .dialogue_engine
        .clear_conversation(npc_id, speaker_id)
        .await;
    Json(AiApiResponse::success(
        "Conversation cleared".to_string(),
        state.dialogue_engine.is_llm_available(),
    ))
}

async fn generate_asset_description(
    State(state): State<AiApiState>,
    Json(request): Json<AssetDescRequest>,
) -> Result<Json<AiApiResponse<AssetDescriptionResponse>>, (StatusCode, Json<Value>)> {
    info!("Generating description for asset {}", request.asset_id);

    let desc_request = AssetDescriptionRequest {
        asset_id: request.asset_id,
        asset_name: request.asset_name,
        asset_type: request.asset_type,
        creator_name: request.creator_name,
        existing_description: request.existing_description,
        tags: request.tags.unwrap_or_default(),
        prim_count: request.prim_count,
        script_count: request.script_count,
    };

    match state
        .asset_generator
        .generate_description(&desc_request)
        .await
    {
        Ok(response) => {
            info!("Generated description in {}ms", response.processing_time_ms);
            Ok(Json(AiApiResponse::success(
                response,
                state.asset_generator.is_llm_available(),
            )))
        }
        Err(e) => {
            error!("Description generation failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Description generation failed: {}", e) })),
            ))
        }
    }
}

async fn get_cached_description(
    State(_state): State<AiApiState>,
    Path(asset_id): Path<Uuid>,
) -> Json<AiApiResponse<Option<AssetDescriptionResponse>>> {
    Json(AiApiResponse::success(None, false))
}

#[derive(Debug, Deserialize)]
pub struct SemanticSearchRequest {
    pub query: String,
    pub content_type: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub similarity: f32,
    pub snippet: String,
}

async fn semantic_search(
    State(state): State<AiApiState>,
    Json(request): Json<SemanticSearchRequest>,
) -> Result<Json<AiApiResponse<Vec<SearchResult>>>, (StatusCode, Json<Value>)> {
    info!("Semantic search: '{}'", request.query);

    if let Some(vector_store) = &state.vector_store {
        let content_type = request
            .content_type
            .as_ref()
            .and_then(|ct| ContentType::from_str(ct));

        let limit = request.limit.unwrap_or(20);

        match vector_store
            .search(&request.query, content_type, limit)
            .await
        {
            Ok(vs_results) => {
                let results: Vec<SearchResult> = vs_results
                    .into_iter()
                    .map(|r| SearchResult {
                        id: r.id,
                        name: r.name,
                        content_type: r.content_type.as_str().to_string(),
                        similarity: r.similarity,
                        snippet: truncate_description(&r.description, 200),
                    })
                    .collect();

                info!(
                    "Found {} results for query '{}'",
                    results.len(),
                    request.query
                );
                Ok(Json(AiApiResponse::success(results, true)))
            }
            Err(e) => {
                error!("Semantic search failed: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Search failed: {}", e) })),
                ))
            }
        }
    } else if let Some(ml) = &state.ml_manager {
        let results: Vec<SearchResult> = Vec::new();
        Ok(Json(AiApiResponse::success(results, ml.is_enabled())))
    } else {
        Ok(Json(AiApiResponse::success(Vec::new(), false)))
    }
}

fn truncate_description(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}

async fn find_similar(
    State(state): State<AiApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AiApiResponse<Vec<SearchResult>>>, (StatusCode, Json<Value>)> {
    info!("Finding similar to: {}", id);

    if let Some(vector_store) = &state.vector_store {
        match vector_store.find_similar(id, 10).await {
            Ok(vs_results) => {
                let results: Vec<SearchResult> = vs_results
                    .into_iter()
                    .map(|r| SearchResult {
                        id: r.id,
                        name: r.name,
                        content_type: r.content_type.as_str().to_string(),
                        similarity: r.similarity,
                        snippet: truncate_description(&r.description, 200),
                    })
                    .collect();

                Ok(Json(AiApiResponse::success(results, true)))
            }
            Err(e) => {
                error!("Find similar failed: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Find similar failed: {}", e) })),
                ))
            }
        }
    } else {
        Ok(Json(AiApiResponse::success(Vec::new(), false)))
    }
}

#[derive(Debug, Deserialize)]
pub struct AssetBatchRequest {
    pub assets: Vec<AssetEmbedRequest>,
}

#[derive(Debug, Deserialize)]
pub struct AssetEmbedRequest {
    pub asset_id: Uuid,
    pub name: String,
    pub description: String,
    pub asset_type: String,
    pub creator_id: Option<Uuid>,
    #[serde(default)]
    pub tags: Vec<String>,
}

async fn embed_assets_batch(
    State(state): State<AiApiState>,
    Json(request): Json<AssetBatchRequest>,
) -> Result<Json<AiApiResponse<EmbedBatchResponse>>, (StatusCode, Json<Value>)> {
    info!("Embedding {} assets", request.assets.len());

    if let Some(pipeline) = &state.embedding_pipeline {
        let assets: Vec<AssetEmbeddingRequest> = request
            .assets
            .into_iter()
            .map(|a| {
                let mut metadata = HashMap::new();
                metadata.insert("asset_type".to_string(), a.asset_type.clone());
                if let Some(creator) = a.creator_id {
                    metadata.insert("creator_id".to_string(), creator.to_string());
                }
                for (i, tag) in a.tags.iter().enumerate() {
                    metadata.insert(format!("tag_{}", i), tag.clone());
                }

                AssetEmbeddingRequest {
                    asset_id: a.asset_id,
                    name: a.name,
                    description: a.description,
                    asset_type: a.asset_type,
                    creator_id: a.creator_id,
                    metadata,
                }
            })
            .collect();

        match pipeline.process_assets(assets).await {
            Ok(result) => {
                info!("Embedded {} assets successfully", result.success_count);
                Ok(Json(AiApiResponse::success(
                    EmbedBatchResponse {
                        success_count: result.success_count,
                        error_count: result.error_count,
                        errors: result.errors,
                    },
                    true,
                )))
            }
            Err(e) => {
                error!("Asset embedding failed: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Embedding failed: {}", e) })),
                ))
            }
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "Embedding pipeline not available" })),
        ))
    }
}

#[derive(Debug, Deserialize)]
pub struct RegionBatchRequest {
    pub regions: Vec<RegionEmbedRequest>,
}

#[derive(Debug, Deserialize)]
pub struct RegionEmbedRequest {
    pub region_id: Uuid,
    pub name: String,
    pub description: String,
    pub x: i32,
    pub y: i32,
    pub maturity_rating: String,
    #[serde(default)]
    pub features: Vec<String>,
}

async fn embed_regions_batch(
    State(state): State<AiApiState>,
    Json(request): Json<RegionBatchRequest>,
) -> Result<Json<AiApiResponse<EmbedBatchResponse>>, (StatusCode, Json<Value>)> {
    info!("Embedding {} regions", request.regions.len());

    if let Some(pipeline) = &state.embedding_pipeline {
        let regions: Vec<RegionEmbeddingRequest> = request
            .regions
            .into_iter()
            .map(|r| {
                let mut metadata = HashMap::new();
                metadata.insert("maturity_rating".to_string(), r.maturity_rating.clone());
                metadata.insert("x".to_string(), r.x.to_string());
                metadata.insert("y".to_string(), r.y.to_string());
                for (i, feature) in r.features.iter().enumerate() {
                    metadata.insert(format!("feature_{}", i), feature.clone());
                }

                RegionEmbeddingRequest {
                    region_id: r.region_id,
                    name: r.name,
                    description: r.description,
                    x: r.x,
                    y: r.y,
                    maturity_rating: r.maturity_rating,
                    metadata,
                }
            })
            .collect();

        match pipeline.process_regions(regions).await {
            Ok(result) => {
                info!("Embedded {} regions successfully", result.success_count);
                Ok(Json(AiApiResponse::success(
                    EmbedBatchResponse {
                        success_count: result.success_count,
                        error_count: result.error_count,
                        errors: result.errors,
                    },
                    true,
                )))
            }
            Err(e) => {
                error!("Region embedding failed: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Embedding failed: {}", e) })),
                ))
            }
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "Embedding pipeline not available" })),
        ))
    }
}

#[derive(Debug, Serialize)]
pub struct EmbedBatchResponse {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EmbeddingStatsResponse {
    pub total_entries: usize,
    pub entries_by_type: HashMap<String, usize>,
    pub average_embedding_dimension: usize,
    pub memory_usage_mb: f64,
}

async fn get_embedding_stats(
    State(state): State<AiApiState>,
) -> Json<AiApiResponse<EmbeddingStatsResponse>> {
    if let Some(vector_store) = &state.vector_store {
        let stats = vector_store.get_stats().await;
        let response = EmbeddingStatsResponse {
            total_entries: stats.total_entries,
            entries_by_type: stats.entries_by_type,
            average_embedding_dimension: stats.average_embedding_dimension,
            memory_usage_mb: stats.memory_usage_bytes as f64 / (1024.0 * 1024.0),
        };
        Json(AiApiResponse::success(response, true))
    } else {
        Json(AiApiResponse::success(
            EmbeddingStatsResponse {
                total_entries: 0,
                entries_by_type: HashMap::new(),
                average_embedding_dimension: 0,
                memory_usage_mb: 0.0,
            },
            false,
        ))
    }
}

async fn assess_upload_quality(
    State(state): State<AiApiState>,
    Json(request): Json<QualityUploadRequest>,
) -> Result<Json<AiApiResponse<QualityAssessment>>, (StatusCode, Json<Value>)> {
    info!("Assessing upload quality for asset {}", request.asset_id);

    if let Some(quality_service) = &state.quality_service {
        match quality_service.assess_upload_quality(&request).await {
            Ok(assessment) => {
                info!(
                    "Quality assessment: {} ({:.2}) for asset {}",
                    assessment.quality_grade.as_str(),
                    assessment.quality_score,
                    assessment.asset_id
                );
                Ok(Json(AiApiResponse::success(assessment, true)))
            }
            Err(e) => {
                error!("Quality assessment failed: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Quality assessment failed: {}", e) })),
                ))
            }
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "Quality service not available" })),
        ))
    }
}

async fn assess_region_quality(
    State(state): State<AiApiState>,
    Json(request): Json<RegionQualityRequest>,
) -> Result<Json<AiApiResponse<RegionQualityReport>>, (StatusCode, Json<Value>)> {
    info!("Assessing region quality for {}", request.region_name);

    if let Some(quality_service) = &state.quality_service {
        match quality_service.assess_region_quality(&request).await {
            Ok(report) => {
                info!(
                    "Region quality: {} ({:.2}) for {}",
                    report.overall_grade.as_str(),
                    report.overall_score,
                    report.region_name
                );
                Ok(Json(AiApiResponse::success(report, true)))
            }
            Err(e) => {
                error!("Region quality assessment failed: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Region assessment failed: {}", e) })),
                ))
            }
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "Quality service not available" })),
        ))
    }
}

async fn get_quality_alerts(
    State(state): State<AiApiState>,
) -> Json<AiApiResponse<Vec<AnomalyAlert>>> {
    if let Some(quality_service) = &state.quality_service {
        let alerts = quality_service.get_anomaly_alerts(50).await;
        Json(AiApiResponse::success(alerts, true))
    } else {
        Json(AiApiResponse::success(Vec::new(), false))
    }
}

async fn acknowledge_quality_alert(
    State(state): State<AiApiState>,
    Path(alert_id): Path<Uuid>,
) -> Result<Json<AiApiResponse<bool>>, (StatusCode, Json<Value>)> {
    info!("Acknowledging quality alert {}", alert_id);

    if let Some(quality_service) = &state.quality_service {
        let acknowledged = quality_service.acknowledge_alert(alert_id).await;
        if acknowledged {
            Ok(Json(AiApiResponse::success(true, true)))
        } else {
            Err((
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Alert {} not found", alert_id) })),
            ))
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "Quality service not available" })),
        ))
    }
}

#[derive(Debug, Serialize)]
pub struct RecommendationResponse {
    pub content_id: Uuid,
    pub content_type: String,
    pub name: String,
    pub reason: String,
    pub score: f32,
    pub source_items: Vec<Uuid>,
}

impl From<CFContentRecommendation> for RecommendationResponse {
    fn from(rec: CFContentRecommendation) -> Self {
        Self {
            content_id: rec.content_id,
            content_type: format!("{:?}", rec.content_type),
            name: rec.name,
            reason: rec.reason.description().to_string(),
            score: rec.score,
            source_items: rec.source_items,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RecommendationQuery {
    pub limit: Option<usize>,
}

async fn get_content_recommendations(
    State(state): State<AiApiState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<RecommendationQuery>,
) -> Result<Json<AiApiResponse<Vec<RecommendationResponse>>>, (StatusCode, Json<Value>)> {
    info!("Getting content recommendations for user {}", user_id);

    let limit = query.limit.unwrap_or(20);

    match state
        .recommender
        .get_content_recommendations(user_id, limit)
        .await
    {
        Ok(recs) => {
            let responses: Vec<RecommendationResponse> =
                recs.into_iter().map(RecommendationResponse::from).collect();
            info!(
                "Returned {} content recommendations for user {}",
                responses.len(),
                user_id
            );
            Ok(Json(AiApiResponse::success(responses, true)))
        }
        Err(e) => {
            error!("Content recommendation failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Recommendation failed: {}", e) })),
            ))
        }
    }
}

async fn get_social_recommendations(
    State(state): State<AiApiState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<RecommendationQuery>,
) -> Result<Json<AiApiResponse<Vec<SocialRecommendation>>>, (StatusCode, Json<Value>)> {
    info!("Getting social recommendations for user {}", user_id);

    let limit = query.limit.unwrap_or(10);

    match state
        .recommender
        .get_social_recommendations(user_id, limit)
        .await
    {
        Ok(recs) => {
            info!(
                "Returned {} social recommendations for user {}",
                recs.len(),
                user_id
            );
            Ok(Json(AiApiResponse::success(recs, true)))
        }
        Err(e) => {
            error!("Social recommendation failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Social recommendation failed: {}", e) })),
            ))
        }
    }
}

async fn get_creator_recommendations(
    State(state): State<AiApiState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<RecommendationQuery>,
) -> Result<Json<AiApiResponse<Vec<RecommendationResponse>>>, (StatusCode, Json<Value>)> {
    info!("Getting creator recommendations for user {}", user_id);

    let limit = query.limit.unwrap_or(10);

    match state
        .recommender
        .get_creator_recommendations(user_id, limit)
        .await
    {
        Ok(recs) => {
            let responses: Vec<RecommendationResponse> =
                recs.into_iter().map(RecommendationResponse::from).collect();
            info!(
                "Returned {} creator recommendations for user {}",
                responses.len(),
                user_id
            );
            Ok(Json(AiApiResponse::success(responses, true)))
        }
        Err(e) => {
            error!("Creator recommendation failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Creator recommendation failed: {}", e) })),
            ))
        }
    }
}

async fn get_engagement_metrics(
    State(state): State<AiApiState>,
    Path(user_id): Path<Uuid>,
) -> Json<AiApiResponse<EngagementMetrics>> {
    info!("Getting engagement metrics for user {}", user_id);

    let metrics = state.recommender.compute_engagement_metrics(user_id).await;
    Json(AiApiResponse::success(metrics, true))
}

async fn get_trending(
    State(state): State<AiApiState>,
    Query(query): Query<RecommendationQuery>,
) -> Json<AiApiResponse<Vec<RecommendationResponse>>> {
    let limit = query.limit.unwrap_or(20);
    let trending = state.recommender.get_trending_content(limit).await;
    let responses: Vec<RecommendationResponse> = trending
        .into_iter()
        .map(RecommendationResponse::from)
        .collect();
    Json(AiApiResponse::success(responses, true))
}

async fn get_recommender_stats(
    State(state): State<AiApiState>,
) -> Json<AiApiResponse<RecommenderStats>> {
    let stats = state.recommender.get_stats().await;
    Json(AiApiResponse::success(stats, true))
}

#[derive(Debug, Deserialize)]
pub struct RecordActivityRequest {
    pub user_id: Uuid,
    pub activity_type: String,
    pub target_id: Uuid,
    pub target_name: String,
    pub duration_seconds: Option<u64>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

async fn record_activity(
    State(state): State<AiApiState>,
    Json(request): Json<RecordActivityRequest>,
) -> Result<Json<AiApiResponse<bool>>, (StatusCode, Json<Value>)> {
    info!(
        "Recording activity for user {}: {} on {}",
        request.user_id, request.activity_type, request.target_id
    );

    let activity_type = match request.activity_type.as_str() {
        "region_visit" => ActivityType::RegionVisit,
        "asset_purchase" => ActivityType::AssetPurchase,
        "asset_view" => ActivityType::AssetView,
        "group_join" => ActivityType::GroupJoin,
        "event_attend" => ActivityType::EventAttend,
        "friend_add" => ActivityType::FriendAdd,
        "creator_follow" => ActivityType::CreatorFollow,
        "content_create" => ActivityType::ContentCreate,
        "chat_interaction" => ActivityType::ChatInteraction,
        unknown => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Unknown activity type: {}", unknown) })),
            ));
        }
    };

    let activity = UserActivity {
        user_id: request.user_id,
        activity_type,
        target_id: request.target_id,
        target_name: request.target_name,
        timestamp: Utc::now(),
        duration_seconds: request.duration_seconds,
        metadata: request.metadata,
    };

    state.recommender.record_activity(activity).await;

    Ok(Json(AiApiResponse::success(true, true)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub user_id: Uuid,
    #[serde(default)]
    pub interests: Vec<String>,
    #[serde(default)]
    pub visited_regions: Vec<Uuid>,
    #[serde(default)]
    pub owned_assets: Vec<Uuid>,
    #[serde(default)]
    pub groups: Vec<Uuid>,
    #[serde(default)]
    pub friends: Vec<Uuid>,
    #[serde(default)]
    pub followed_creators: Vec<Uuid>,
}

async fn update_user_profile(
    State(state): State<AiApiState>,
    Json(request): Json<UpdateProfileRequest>,
) -> Json<AiApiResponse<bool>> {
    info!("Updating profile for user {}", request.user_id);

    let profile = CFUserProfile {
        user_id: request.user_id,
        interests: request.interests,
        visited_regions: request.visited_regions,
        owned_assets: request.owned_assets,
        groups: request.groups,
        friends: request.friends,
        followed_creators: request.followed_creators,
        activity_score: 0.0,
        last_active: Utc::now(),
    };

    state.recommender.update_user_profile(profile).await;

    Json(AiApiResponse::success(true, true))
}

async fn get_or_create_npc_profile(
    _db: &Arc<DatabaseManager>,
    npc_id: Uuid,
) -> Result<NPCProfile, String> {
    use crate::ai::npc_behavior::{Activity, BehaviorState, Mood, NPCPersonality, NPCRole};
    use chrono::Utc;
    use std::collections::HashMap;

    Ok(NPCProfile {
        npc_id,
        name: format!("NPC_{}", &npc_id.to_string()[..8]),
        personality: NPCPersonality {
            friendliness: 0.7,
            curiosity: 0.6,
            helpfulness: 0.8,
            sociability: 0.6,
            assertiveness: 0.5,
            intelligence: 0.6,
            creativity: 0.5,
        },
        role: NPCRole::Citizen,
        behavior_state: BehaviorState {
            current_activity: Activity::Idle,
            mood: Mood {
                happiness: 0.6,
                stress: 0.3,
                boredom: 0.2,
                confidence: 0.7,
            },
            energy_level: 0.8,
            social_need: 0.5,
            goals: Vec::new(),
            last_interaction: None,
        },
        social_network: HashMap::new(),
        location_preferences: vec!["town_square".to_string()],
        activity_schedule: Vec::new(),
        memory: crate::ai::npc_behavior::NPCMemory {
            short_term: Vec::new(),
            long_term: Vec::new(),
            locations_visited: HashMap::new(),
            people_met: HashMap::new(),
        },
        created_at: Utc::now(),
        last_update: Utc::now(),
    })
}

pub async fn initialize_ai_api(
    db: Arc<DatabaseManager>,
    llm_config: Option<LLMConfig>,
    ml_config: Option<MLIntegrationConfig>,
) -> Result<AiApiState, String> {
    info!("Initializing AI API v2.0...");

    let dialogue_engine =
        NPCDialogueEngine::new(llm_config.clone(), db.clone(), DialogueConfig::default())
            .await
            .map_err(|e| format!("Failed to create dialogue engine: {}", e))?;

    let asset_generator = AssetDescriptionGenerator::new(llm_config)
        .await
        .map_err(|e| format!("Failed to create asset generator: {}", e))?;

    let ml_manager = if let Some(cfg) = ml_config {
        match MLIntegrationManager::new(cfg, db.clone()).await {
            Ok(manager) => {
                info!("ML Integration Manager initialized");
                Some(manager)
            }
            Err(e) => {
                warn!("ML Integration Manager failed to initialize: {}", e);
                None
            }
        }
    } else {
        None
    };

    let (vector_store, embedding_pipeline) = match initialize_vector_store().await {
        Ok((vs, ep)) => {
            info!("Vector store and embedding pipeline initialized");
            (Some(vs), Some(Arc::new(ep)))
        }
        Err(e) => {
            warn!("Vector store failed to initialize: {}", e);
            (None, None)
        }
    };

    let quality_service = match initialize_quality_service().await {
        Ok(qs) => {
            info!("Quality service initialized");
            Some(qs)
        }
        Err(e) => {
            warn!("Quality service failed to initialize: {}", e);
            None
        }
    };

    let llm_status = if dialogue_engine.is_llm_available() {
        "connected"
    } else {
        "fallback mode"
    };

    let vector_status = if vector_store.is_some() {
        "ready"
    } else {
        "unavailable"
    };

    let quality_status = if quality_service.is_some() {
        "ready"
    } else {
        "unavailable"
    };

    let recommender = Arc::new(CollaborativeRecommender::new(RecommenderConfig::default()));
    info!("Collaborative recommender initialized");

    info!(
        "AI API initialized - LLM: {}, VectorStore: {}, Quality: {}, Recommender: ready",
        llm_status, vector_status, quality_status
    );

    Ok(AiApiState {
        dialogue_engine,
        asset_generator,
        ml_manager,
        vector_store,
        embedding_pipeline,
        quality_service,
        recommender,
        db,
    })
}

async fn initialize_vector_store() -> Result<(Arc<VectorStore>, EmbeddingPipeline), String> {
    let embedding_service = EmbeddingService::new(EmbeddingModel::AllMiniLML6V2, true, 10000)
        .await
        .map_err(|e| format!("Failed to create embedding service: {}", e))?;

    let vector_config = VectorStoreConfig {
        max_entries: 100_000,
        similarity_threshold: 0.3,
        enable_persistence: false,
        persistence_path: None,
        auto_cleanup_days: 90,
    };

    let vector_store = VectorStore::new(embedding_service.clone(), vector_config)
        .await
        .map_err(|e| format!("Failed to create vector store: {}", e))?;

    let embedding_pipeline = EmbeddingPipeline::new(vector_store.clone(), embedding_service, 100);

    Ok((vector_store, embedding_pipeline))
}

async fn initialize_quality_service() -> Result<Arc<QualityService>, String> {
    let predictor_config = PredictorConfig {
        models_path: std::env::var("OPENSIM_ONNX_MODELS_PATH")
            .unwrap_or_else(|_| "./models/".to_string()),
        ..Default::default()
    };

    let predictor = ONNXPredictor::new(predictor_config)
        .await
        .map_err(|e| format!("Failed to create ONNX predictor: {}", e))?;

    let quality_config = QualityServiceConfig::default();
    let service = QualityService::new(predictor, quality_config);

    Ok(Arc::new(service))
}
