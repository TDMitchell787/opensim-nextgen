use super::super::AIError;
use super::onnx_predictor::{ONNXPredictor, ContentCategory, AnomalyResult};
use crate::ai::content_validator::{
    ContentValidator, ValidationConfig, ValidationResult, ValidatableContent,
    ValidatableObject, ValidatableTexture, ValidatableScript, ValidatableMesh,
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, warn, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityServiceConfig {
    pub quality_threshold_low: f32,
    pub quality_threshold_high: f32,
    pub anomaly_threshold: f32,
    pub enable_auto_suggestions: bool,
    pub max_upload_history: usize,
    pub pattern_window_minutes: u64,
    pub max_uploads_per_window: usize,
}

impl Default for QualityServiceConfig {
    fn default() -> Self {
        Self {
            quality_threshold_low: 0.3,
            quality_threshold_high: 0.7,
            anomaly_threshold: 0.7,
            enable_auto_suggestions: true,
            max_upload_history: 10000,
            pattern_window_minutes: 60,
            max_uploads_per_window: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub asset_id: Uuid,
    pub quality_score: f32,
    pub quality_grade: QualityGrade,
    pub category: String,
    pub category_confidence: f32,
    pub anomaly_status: AnomalyStatus,
    pub validation_result: Option<ValidationSummary>,
    pub suggestions: Vec<QualitySuggestion>,
    pub processing_time_ms: u64,
    pub assessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityGrade {
    Excellent,
    Good,
    Acceptable,
    NeedsImprovement,
    Poor,
}

impl QualityGrade {
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.9 => QualityGrade::Excellent,
            s if s >= 0.7 => QualityGrade::Good,
            s if s >= 0.5 => QualityGrade::Acceptable,
            s if s >= 0.3 => QualityGrade::NeedsImprovement,
            _ => QualityGrade::Poor,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            QualityGrade::Excellent => "Excellent",
            QualityGrade::Good => "Good",
            QualityGrade::Acceptable => "Acceptable",
            QualityGrade::NeedsImprovement => "Needs Improvement",
            QualityGrade::Poor => "Poor",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyStatus {
    pub is_anomaly: bool,
    pub anomaly_score: f32,
    pub risk_level: RiskLevel,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.9 => RiskLevel::Critical,
            s if s >= 0.7 => RiskLevel::High,
            s if s >= 0.5 => RiskLevel::Medium,
            s if s >= 0.3 => RiskLevel::Low,
            _ => RiskLevel::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub is_valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub suggestion_count: usize,
    pub estimated_land_impact: u32,
    pub top_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySuggestion {
    pub category: SuggestionCategory,
    pub message: String,
    pub priority: SuggestionPriority,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Performance,
    Quality,
    Optimization,
    Security,
    BestPractice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionQualityReport {
    pub region_id: Uuid,
    pub region_name: String,
    pub overall_score: f32,
    pub overall_grade: QualityGrade,
    pub prim_efficiency: f32,
    pub script_load: f32,
    pub texture_efficiency: f32,
    pub physics_complexity: f32,
    pub performance_prediction: PerformancePrediction,
    pub recommendations: Vec<QualitySuggestion>,
    pub assessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    pub estimated_fps_impact: f32,
    pub estimated_memory_mb: f32,
    pub estimated_bandwidth_kbps: f32,
    pub bottleneck: String,
    pub scalability_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadPattern {
    pub user_id: Uuid,
    pub upload_count: usize,
    pub window_start: DateTime<Utc>,
    pub asset_types: Vec<String>,
    pub total_size_bytes: u64,
    pub anomaly_flags: Vec<String>,
}

pub struct QualityService {
    config: QualityServiceConfig,
    predictor: Arc<ONNXPredictor>,
    validator: ContentValidator,
    upload_history: Arc<RwLock<HashMap<Uuid, Vec<UploadRecord>>>>,
    anomaly_alerts: Arc<RwLock<Vec<AnomalyAlert>>>,
}

#[derive(Debug, Clone)]
struct UploadRecord {
    asset_id: Uuid,
    uploaded_at: DateTime<Utc>,
    asset_type: String,
    size_bytes: u64,
    quality_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub alert_id: Uuid,
    pub user_id: Uuid,
    pub alert_type: AlertType,
    pub message: String,
    pub severity: RiskLevel,
    pub created_at: DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    RapidUploads,
    SuspiciousContent,
    QualityAnomaly,
    PatternViolation,
    SizeAnomaly,
}

impl QualityService {
    pub fn new(
        predictor: Arc<ONNXPredictor>,
        config: QualityServiceConfig,
    ) -> Self {
        Self {
            config,
            predictor,
            validator: ContentValidator::with_defaults(),
            upload_history: Arc::new(RwLock::new(HashMap::new())),
            anomaly_alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn assess_upload_quality(
        &self,
        request: &UploadQualityRequest,
    ) -> Result<QualityAssessment, AIError> {
        let start_time = std::time::Instant::now();

        let features = self.extract_features(request);
        let quality_score = self.predictor.predict_quality(&features).await?;
        let quality_grade = QualityGrade::from_score(quality_score);

        let classification = self.predictor.classify_content(&features).await?;
        let category = format!("{:?}", classification.category);
        let category_confidence = classification.confidence;

        let anomaly_result = self.predictor.detect_anomaly(&features).await?;
        let anomaly_status = AnomalyStatus {
            is_anomaly: anomaly_result.is_anomaly,
            anomaly_score: anomaly_result.anomaly_score,
            risk_level: RiskLevel::from_score(anomaly_result.anomaly_score),
            flags: anomaly_result.contributing_features
                .iter()
                .map(|(name, score)| format!("{}: {:.2}", name, score))
                .collect(),
        };

        let validation_result = if let Some(content) = &request.validatable_content {
            let result = self.validator.validate_content(content);
            Some(ValidationSummary {
                is_valid: result.is_valid,
                error_count: result.errors.len(),
                warning_count: result.warnings.len(),
                suggestion_count: result.suggestions.len(),
                estimated_land_impact: result.metrics.estimated_land_impact,
                top_issues: result.errors.iter()
                    .take(3)
                    .map(|e| e.message.clone())
                    .collect(),
            })
        } else {
            None
        };

        let suggestions = if self.config.enable_auto_suggestions {
            self.generate_suggestions(quality_score, &anomaly_status, &validation_result, request)
        } else {
            Vec::new()
        };

        if let Some(user_id) = request.uploader_id {
            self.record_upload(user_id, request, quality_score).await;
            self.check_upload_patterns(user_id).await;
        }

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(QualityAssessment {
            asset_id: request.asset_id,
            quality_score,
            quality_grade,
            category,
            category_confidence,
            anomaly_status,
            validation_result,
            suggestions,
            processing_time_ms,
            assessed_at: Utc::now(),
        })
    }

    pub async fn assess_region_quality(
        &self,
        request: &RegionQualityRequest,
    ) -> Result<RegionQualityReport, AIError> {
        let prim_efficiency = 1.0 - (request.total_prims as f32 / request.max_prims as f32).min(1.0);
        let script_load = (request.active_scripts as f32 / 1000.0).min(1.0);
        let texture_efficiency = if request.unique_textures > 0 {
            1.0 - (request.texture_memory_mb / 512.0).min(1.0)
        } else {
            1.0
        };
        let physics_complexity = (request.physics_objects as f32 / request.total_prims.max(1) as f32).min(1.0);

        let features = vec![
            prim_efficiency,
            1.0 - script_load,
            texture_efficiency,
            1.0 - physics_complexity,
            request.avg_fps / 60.0,
        ];

        let overall_score = self.predictor.predict_quality(&features).await?;
        let overall_grade = QualityGrade::from_score(overall_score);

        let fps_impact = script_load * 15.0 + physics_complexity * 10.0 + (1.0 - prim_efficiency) * 5.0;
        let memory_mb = request.texture_memory_mb + (request.total_prims as f32 * 0.001);
        let bandwidth = request.active_scripts as f32 * 0.5 + request.total_prims as f32 * 0.01;

        let bottleneck = if script_load > physics_complexity && script_load > (1.0 - prim_efficiency) {
            "Script Load"
        } else if physics_complexity > (1.0 - prim_efficiency) {
            "Physics Complexity"
        } else {
            "Prim Count"
        };

        let scalability_score = (prim_efficiency * 0.3 + (1.0 - script_load) * 0.4 + texture_efficiency * 0.3)
            .max(0.0).min(1.0);

        let performance_prediction = PerformancePrediction {
            estimated_fps_impact: fps_impact,
            estimated_memory_mb: memory_mb,
            estimated_bandwidth_kbps: bandwidth,
            bottleneck: bottleneck.to_string(),
            scalability_score,
        };

        let mut recommendations = Vec::new();

        if script_load > 0.5 {
            recommendations.push(QualitySuggestion {
                category: SuggestionCategory::Performance,
                message: format!(
                    "High script load ({} scripts). Consider consolidating or removing idle scripts",
                    request.active_scripts
                ),
                priority: SuggestionPriority::High,
                auto_fixable: false,
            });
        }

        if prim_efficiency < 0.3 {
            recommendations.push(QualitySuggestion {
                category: SuggestionCategory::Optimization,
                message: format!(
                    "Region is {:.0}% full ({}/{}). Consider removing unused objects",
                    (1.0 - prim_efficiency) * 100.0, request.total_prims, request.max_prims
                ),
                priority: SuggestionPriority::Medium,
                auto_fixable: false,
            });
        }

        if request.texture_memory_mb > 256.0 {
            recommendations.push(QualitySuggestion {
                category: SuggestionCategory::Optimization,
                message: format!(
                    "Texture memory usage is {:.0}MB. Consider reducing texture sizes",
                    request.texture_memory_mb
                ),
                priority: SuggestionPriority::Medium,
                auto_fixable: false,
            });
        }

        if physics_complexity > 0.5 {
            recommendations.push(QualitySuggestion {
                category: SuggestionCategory::Performance,
                message: "High ratio of physics objects. Consider using simpler collision shapes".to_string(),
                priority: SuggestionPriority::Medium,
                auto_fixable: false,
            });
        }

        Ok(RegionQualityReport {
            region_id: request.region_id,
            region_name: request.region_name.clone(),
            overall_score,
            overall_grade,
            prim_efficiency,
            script_load,
            texture_efficiency,
            physics_complexity,
            performance_prediction,
            recommendations,
            assessed_at: Utc::now(),
        })
    }

    pub async fn get_anomaly_alerts(&self, limit: usize) -> Vec<AnomalyAlert> {
        let alerts = self.anomaly_alerts.read().await;
        alerts.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> bool {
        let mut alerts = self.anomaly_alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.alert_id == alert_id) {
            alert.acknowledged = true;
            true
        } else {
            false
        }
    }

    pub async fn get_user_upload_pattern(&self, user_id: Uuid) -> Option<UploadPattern> {
        let history = self.upload_history.read().await;
        if let Some(records) = history.get(&user_id) {
            let window_start = Utc::now() - chrono::Duration::minutes(self.config.pattern_window_minutes as i64);
            let recent: Vec<&UploadRecord> = records.iter()
                .filter(|r| r.uploaded_at >= window_start)
                .collect();

            if recent.is_empty() {
                return None;
            }

            let asset_types: Vec<String> = recent.iter()
                .map(|r| r.asset_type.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let total_size: u64 = recent.iter().map(|r| r.size_bytes).sum();

            let mut flags = Vec::new();
            if recent.len() > self.config.max_uploads_per_window {
                flags.push("rapid_uploads".to_string());
            }
            if total_size > 100_000_000 {
                flags.push("large_total_size".to_string());
            }

            Some(UploadPattern {
                user_id,
                upload_count: recent.len(),
                window_start,
                asset_types,
                total_size_bytes: total_size,
                anomaly_flags: flags,
            })
        } else {
            None
        }
    }

    fn extract_features(&self, request: &UploadQualityRequest) -> Vec<f32> {
        let mut features = Vec::with_capacity(16);

        features.push(request.prim_count.unwrap_or(1) as f32 / 256.0);
        features.push(request.script_count.unwrap_or(0) as f32 / 50.0);
        features.push(request.texture_count.unwrap_or(0) as f32 / 20.0);
        features.push(request.mesh_count.unwrap_or(0) as f32 / 10.0);
        features.push(request.size_bytes as f32 / 1_000_000.0);

        let name_len = request.name.len() as f32 / 50.0;
        features.push(name_len.min(1.0));

        let desc_len = request.description.as_ref().map_or(0.0, |d| d.len() as f32 / 500.0);
        features.push(desc_len.min(1.0));

        let has_mesh = if request.mesh_count.unwrap_or(0) > 0 { 1.0 } else { 0.0 };
        features.push(has_mesh);

        let has_scripts = if request.script_count.unwrap_or(0) > 0 { 1.0 } else { 0.0 };
        features.push(has_scripts);

        let type_factor = match request.asset_type.to_lowercase().as_str() {
            "mesh" => 0.8,
            "object" => 0.6,
            "texture" => 0.4,
            "script" => 0.5,
            "animation" => 0.7,
            "sound" => 0.3,
            _ => 0.5,
        };
        features.push(type_factor);

        while features.len() < 16 {
            features.push(0.0);
        }

        features
    }

    fn generate_suggestions(
        &self,
        quality_score: f32,
        anomaly: &AnomalyStatus,
        validation: &Option<ValidationSummary>,
        request: &UploadQualityRequest,
    ) -> Vec<QualitySuggestion> {
        let mut suggestions = Vec::new();

        if quality_score < self.config.quality_threshold_low {
            suggestions.push(QualitySuggestion {
                category: SuggestionCategory::Quality,
                message: "Content quality is below recommended threshold. Consider improving before publishing.".to_string(),
                priority: SuggestionPriority::High,
                auto_fixable: false,
            });
        }

        if request.description.is_none() || request.description.as_ref().map_or(true, |d| d.len() < 10) {
            suggestions.push(QualitySuggestion {
                category: SuggestionCategory::BestPractice,
                message: "Add a descriptive description to improve discoverability.".to_string(),
                priority: SuggestionPriority::Low,
                auto_fixable: false,
            });
        }

        if let Some(prim_count) = request.prim_count {
            if prim_count > 200 {
                suggestions.push(QualitySuggestion {
                    category: SuggestionCategory::Performance,
                    message: format!(
                        "High prim count ({}). Consider using mesh to reduce land impact.",
                        prim_count
                    ),
                    priority: SuggestionPriority::Medium,
                    auto_fixable: false,
                });
            }
        }

        if let Some(script_count) = request.script_count {
            if script_count > 20 {
                suggestions.push(QualitySuggestion {
                    category: SuggestionCategory::Performance,
                    message: format!(
                        "{} scripts detected. Consider consolidating for better sim performance.",
                        script_count
                    ),
                    priority: SuggestionPriority::Medium,
                    auto_fixable: false,
                });
            }
        }

        if anomaly.is_anomaly {
            suggestions.push(QualitySuggestion {
                category: SuggestionCategory::Security,
                message: "Content flagged as potentially anomalous. Manual review recommended.".to_string(),
                priority: SuggestionPriority::High,
                auto_fixable: false,
            });
        }

        if let Some(val) = validation {
            if !val.is_valid {
                suggestions.push(QualitySuggestion {
                    category: SuggestionCategory::Quality,
                    message: format!(
                        "{} validation errors found. Fix errors before upload.",
                        val.error_count
                    ),
                    priority: SuggestionPriority::Critical,
                    auto_fixable: false,
                });
            }
        }

        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
        suggestions
    }

    async fn record_upload(&self, user_id: Uuid, request: &UploadQualityRequest, quality_score: f32) {
        let record = UploadRecord {
            asset_id: request.asset_id,
            uploaded_at: Utc::now(),
            asset_type: request.asset_type.clone(),
            size_bytes: request.size_bytes,
            quality_score,
        };

        let mut history = self.upload_history.write().await;
        let records = history.entry(user_id).or_insert_with(Vec::new);
        records.push(record);

        if records.len() > self.config.max_upload_history {
            let drain_count = records.len() - self.config.max_upload_history;
            records.drain(..drain_count);
        }
    }

    async fn check_upload_patterns(&self, user_id: Uuid) {
        let history = self.upload_history.read().await;
        if let Some(records) = history.get(&user_id) {
            let window_start = Utc::now() - chrono::Duration::minutes(self.config.pattern_window_minutes as i64);
            let recent_count = records.iter()
                .filter(|r| r.uploaded_at >= window_start)
                .count();

            if recent_count > self.config.max_uploads_per_window {
                drop(history);
                let alert = AnomalyAlert {
                    alert_id: Uuid::new_v4(),
                    user_id,
                    alert_type: AlertType::RapidUploads,
                    message: format!(
                        "User uploaded {} items in {} minutes (threshold: {})",
                        recent_count, self.config.pattern_window_minutes,
                        self.config.max_uploads_per_window
                    ),
                    severity: RiskLevel::Medium,
                    created_at: Utc::now(),
                    acknowledged: false,
                };

                let mut alerts = self.anomaly_alerts.write().await;
                alerts.push(alert);

                if alerts.len() > 1000 {
                    alerts.drain(..100);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadQualityRequest {
    pub asset_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub asset_type: String,
    pub size_bytes: u64,
    pub uploader_id: Option<Uuid>,
    pub prim_count: Option<u32>,
    pub script_count: Option<u32>,
    pub texture_count: Option<u32>,
    pub mesh_count: Option<u32>,
    #[serde(skip)]
    pub validatable_content: Option<ValidatableContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionQualityRequest {
    pub region_id: Uuid,
    pub region_name: String,
    pub total_prims: u32,
    pub max_prims: u32,
    pub active_scripts: u32,
    pub physics_objects: u32,
    pub unique_textures: u32,
    pub texture_memory_mb: f32,
    pub avg_fps: f32,
}
