use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::database::DatabaseManager;

fn string_to_uuid(s: &str) -> Uuid {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();
    let bytes: [u8; 16] = {
        let mut arr = [0u8; 16];
        arr[0..8].copy_from_slice(&hash.to_le_bytes());
        arr[8..16].copy_from_slice(&hash.to_be_bytes());
        arr
    };
    Uuid::from_bytes(bytes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentCategory {
    Primitives,
    Architecture,
    Landscape,
    Interactive,
    Environments,
    Vehicles,
    Wearables,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: String,
    pub name: String,
    pub category: ContentCategory,
    pub recognition_score: f64,
    pub usage_frequency: u32,
    pub success_rate: f64,
    pub elegance_score: f64,
    pub characteristics: Vec<String>,
    pub examples: Vec<String>,
    pub improvement_history: Vec<ImprovementRecord>,
    pub learned_from: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementRecord {
    pub timestamp: DateTime<Utc>,
    pub improvement_type: String,
    pub before_score: f64,
    pub after_score: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalPattern {
    pub style: String,
    pub period: Option<String>,
    pub characteristics: Vec<String>,
    pub proportions: ProportionRules,
    pub materials: Vec<String>,
    pub construction_methods: Vec<String>,
    pub quality_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProportionRules {
    pub height_to_width: f64,
    pub depth_to_width: f64,
    pub floor_height: f64,
    pub window_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialPattern {
    pub material_type: String,
    pub usage_contexts: Vec<String>,
    pub color_palettes: Vec<ColorPalette>,
    pub texture_properties: TextureProperties,
    pub quality_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorPalette {
    pub name: String,
    pub primary_color: (f32, f32, f32),
    pub secondary_colors: Vec<(f32, f32, f32)>,
    pub accent_color: Option<(f32, f32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextureProperties {
    pub scale: (f32, f32),
    pub rotation: f32,
    pub tiling: bool,
    pub glossiness: f32,
    pub roughness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialPattern {
    pub layout_type: String,
    pub density_rules: DensityRules,
    pub accessibility_patterns: Vec<String>,
    pub flow_patterns: Vec<String>,
    pub optimization_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DensityRules {
    pub min_spacing: f32,
    pub max_density: f32,
    pub clustering_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptingPattern {
    pub function_type: String,
    pub common_implementations: Vec<String>,
    pub optimization_techniques: Vec<String>,
    pub error_patterns: Vec<String>,
    pub best_practices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    pub name: String,
    pub description: String,
    pub why_bad: String,
    pub detection_rules: Vec<String>,
    pub correction_suggestions: Vec<String>,
    pub severity: AntiPatternSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AntiPatternSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPattern {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub pattern_data: Vec<u8>,
    pub quality_score: f64,
    pub usage_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetric {
    pub id: i64,
    pub pattern_id: Uuid,
    pub metric_type: String,
    pub metric_value: f64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Content,
    Architectural,
    Material,
    Spatial,
    Scripting,
    Anti,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternType::Content => write!(f, "content"),
            PatternType::Architectural => write!(f, "architectural"),
            PatternType::Material => write!(f, "material"),
            PatternType::Spatial => write!(f, "spatial"),
            PatternType::Scripting => write!(f, "scripting"),
            PatternType::Anti => write!(f, "anti"),
        }
    }
}

pub struct PatternRepository {
    db: Arc<DatabaseManager>,
    cache: Arc<RwLock<PatternCache>>,
    config: PatternRepositoryConfig,
}

#[derive(Debug, Clone)]
pub struct PatternRepositoryConfig {
    pub cache_size: usize,
    pub sync_interval_secs: u64,
    pub auto_save: bool,
}

impl Default for PatternRepositoryConfig {
    fn default() -> Self {
        Self {
            cache_size: 10000,
            sync_interval_secs: 300,
            auto_save: true,
        }
    }
}

#[derive(Debug, Default)]
struct PatternCache {
    content_patterns: HashMap<Uuid, LearnedPattern>,
    architectural_patterns: HashMap<String, ArchitecturalPattern>,
    material_patterns: HashMap<String, MaterialPattern>,
    spatial_patterns: HashMap<String, SpatialPattern>,
    scripting_patterns: HashMap<String, ScriptingPattern>,
    anti_patterns: HashMap<String, AntiPattern>,
    dirty: bool,
    last_sync: Option<DateTime<Utc>>,
}

impl PatternRepository {
    pub async fn new(db: Arc<DatabaseManager>, config: PatternRepositoryConfig) -> Result<Self> {
        let repository = Self {
            db,
            cache: Arc::new(RwLock::new(PatternCache::default())),
            config,
        };

        repository.load_all_patterns().await?;

        Ok(repository)
    }

    pub async fn save_learned_pattern(&self, pattern: &LearnedPattern) -> Result<()> {
        let id = Uuid::parse_str(&pattern.id).unwrap_or_else(|_| Uuid::new_v4());
        let pattern_data = serde_json::to_vec(pattern)
            .context("Failed to serialize pattern")?;

        let category = format!("{:?}", pattern.category);

        self.save_pattern_to_db(
            id,
            &pattern.name,
            &category,
            &pattern_data,
            pattern.elegance_score,
            pattern.usage_frequency,
        ).await?;

        let mut cache = self.cache.write().await;
        cache.content_patterns.insert(id, pattern.clone());
        cache.dirty = true;

        Ok(())
    }

    pub async fn save_architectural_pattern(&self, key: &str, pattern: &ArchitecturalPattern) -> Result<()> {
        let id = string_to_uuid(key);
        let pattern_data = serde_json::to_vec(pattern)
            .context("Failed to serialize architectural pattern")?;

        self.save_pattern_to_db(
            id,
            &pattern.style,
            "architectural",
            &pattern_data,
            0.0,
            0,
        ).await?;

        let mut cache = self.cache.write().await;
        cache.architectural_patterns.insert(key.to_string(), pattern.clone());
        cache.dirty = true;

        Ok(())
    }

    pub async fn save_material_pattern(&self, key: &str, pattern: &MaterialPattern) -> Result<()> {
        let id = string_to_uuid(key);
        let pattern_data = serde_json::to_vec(pattern)
            .context("Failed to serialize material pattern")?;

        self.save_pattern_to_db(
            id,
            &pattern.material_type,
            "material",
            &pattern_data,
            0.0,
            0,
        ).await?;

        let mut cache = self.cache.write().await;
        cache.material_patterns.insert(key.to_string(), pattern.clone());
        cache.dirty = true;

        Ok(())
    }

    pub async fn save_spatial_pattern(&self, key: &str, pattern: &SpatialPattern) -> Result<()> {
        let id = string_to_uuid(key);
        let pattern_data = serde_json::to_vec(pattern)
            .context("Failed to serialize spatial pattern")?;

        self.save_pattern_to_db(
            id,
            &pattern.layout_type,
            "spatial",
            &pattern_data,
            0.0,
            0,
        ).await?;

        let mut cache = self.cache.write().await;
        cache.spatial_patterns.insert(key.to_string(), pattern.clone());
        cache.dirty = true;

        Ok(())
    }

    pub async fn save_scripting_pattern(&self, key: &str, pattern: &ScriptingPattern) -> Result<()> {
        let id = string_to_uuid(key);
        let pattern_data = serde_json::to_vec(pattern)
            .context("Failed to serialize scripting pattern")?;

        self.save_pattern_to_db(
            id,
            &pattern.function_type,
            "scripting",
            &pattern_data,
            0.0,
            0,
        ).await?;

        let mut cache = self.cache.write().await;
        cache.scripting_patterns.insert(key.to_string(), pattern.clone());
        cache.dirty = true;

        Ok(())
    }

    pub async fn save_anti_pattern(&self, key: &str, pattern: &AntiPattern) -> Result<()> {
        let id = string_to_uuid(key);
        let pattern_data = serde_json::to_vec(pattern)
            .context("Failed to serialize anti pattern")?;

        self.save_pattern_to_db(
            id,
            &pattern.name,
            "anti",
            &pattern_data,
            0.0,
            0,
        ).await?;

        let mut cache = self.cache.write().await;
        cache.anti_patterns.insert(key.to_string(), pattern.clone());
        cache.dirty = true;

        Ok(())
    }

    async fn save_pattern_to_db(
        &self,
        id: Uuid,
        name: &str,
        category: &str,
        pattern_data: &[u8],
        quality_score: f64,
        usage_count: u32,
    ) -> Result<()> {
        let pool = self.db.get_pool()?;
        let now = Utc::now();

        match pool {
            crate::database::DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO ai_learned_patterns (id, name, category, pattern_data, quality_score, usage_count, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
                    ON CONFLICT (id) DO UPDATE SET
                        name = EXCLUDED.name,
                        category = EXCLUDED.category,
                        pattern_data = EXCLUDED.pattern_data,
                        quality_score = EXCLUDED.quality_score,
                        usage_count = EXCLUDED.usage_count,
                        updated_at = EXCLUDED.updated_at
                    "#
                )
                .bind(id.to_string())
                .bind(name)
                .bind(category)
                .bind(pattern_data)
                .bind(quality_score)
                .bind(usage_count as i32)
                .bind(now)
                .execute(pg_pool)
                .await
                .context("Failed to save pattern to PostgreSQL")?;
            }
            crate::database::DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO ai_learned_patterns (id, name, category, pattern_data, quality_score, usage_count, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    ON DUPLICATE KEY UPDATE
                        name = VALUES(name),
                        category = VALUES(category),
                        pattern_data = VALUES(pattern_data),
                        quality_score = VALUES(quality_score),
                        usage_count = VALUES(usage_count),
                        updated_at = VALUES(updated_at)
                    "#
                )
                .bind(id.to_string())
                .bind(name)
                .bind(category)
                .bind(pattern_data)
                .bind(quality_score)
                .bind(usage_count as i32)
                .bind(now)
                .bind(now)
                .execute(mysql_pool)
                .await
                .context("Failed to save pattern to MySQL")?;
            }
        }

        Ok(())
    }

    pub async fn load_learned_pattern(&self, id: Uuid) -> Result<Option<LearnedPattern>> {
        let cache = self.cache.read().await;
        if let Some(pattern) = cache.content_patterns.get(&id) {
            return Ok(Some(pattern.clone()));
        }
        drop(cache);

        let stored = self.load_pattern_from_db(id).await?;
        if let Some(stored) = stored {
            let pattern: LearnedPattern = serde_json::from_slice(&stored.pattern_data)
                .context("Failed to deserialize pattern")?;

            let mut cache = self.cache.write().await;
            cache.content_patterns.insert(id, pattern.clone());

            Ok(Some(pattern))
        } else {
            Ok(None)
        }
    }

    async fn load_pattern_from_db(&self, id: Uuid) -> Result<Option<StoredPattern>> {
        let pool = self.db.get_pool()?;

        match pool {
            crate::database::DatabasePoolRef::PostgreSQL(pg_pool) => {
                let row: Option<(String, String, String, Vec<u8>, f64, i32, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
                    "SELECT id, name, category, pattern_data, quality_score, usage_count, created_at, updated_at FROM ai_learned_patterns WHERE id = $1"
                )
                .bind(id.to_string())
                .fetch_optional(pg_pool)
                .await
                .context("Failed to load pattern from PostgreSQL")?;

                Ok(row.map(|(id_str, name, category, pattern_data, quality_score, usage_count, created_at, updated_at)| {
                    StoredPattern {
                        id: Uuid::parse_str(&id_str).unwrap_or(id),
                        name,
                        category,
                        pattern_data,
                        quality_score,
                        usage_count: usage_count as u32,
                        created_at,
                        updated_at,
                    }
                }))
            }
            crate::database::DatabasePoolRef::MySQL(mysql_pool) => {
                let row: Option<(String, String, String, Vec<u8>, f64, i32, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
                    "SELECT id, name, category, pattern_data, quality_score, usage_count, created_at, updated_at FROM ai_learned_patterns WHERE id = ?"
                )
                .bind(id.to_string())
                .fetch_optional(mysql_pool)
                .await
                .context("Failed to load pattern from MySQL")?;

                Ok(row.map(|(id_str, name, category, pattern_data, quality_score, usage_count, created_at, updated_at)| {
                    StoredPattern {
                        id: Uuid::parse_str(&id_str).unwrap_or(id),
                        name,
                        category,
                        pattern_data,
                        quality_score,
                        usage_count: usage_count as u32,
                        created_at,
                        updated_at,
                    }
                }))
            }
        }
    }

    pub async fn load_all_patterns(&self) -> Result<()> {
        tracing::info!("Loading all patterns from database");

        let pool = self.db.get_pool()?;
        let mut cache = self.cache.write().await;

        let rows: Vec<(String, String, String, Vec<u8>, f64, i32)> = match pool {
            crate::database::DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query_as(
                    "SELECT id, name, category, pattern_data, quality_score, usage_count FROM ai_learned_patterns"
                )
                .fetch_all(pg_pool)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to load patterns from PostgreSQL (table may not exist yet): {}", e);
                    Vec::new()
                })
            }
            crate::database::DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query_as(
                    "SELECT id, name, category, pattern_data, quality_score, usage_count FROM ai_learned_patterns"
                )
                .fetch_all(mysql_pool)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to load patterns from MySQL (table may not exist yet): {}", e);
                    Vec::new()
                })
            }
        };

        let mut content_count = 0;
        let mut architectural_count = 0;
        let mut material_count = 0;
        let mut spatial_count = 0;
        let mut scripting_count = 0;
        let mut anti_count = 0;

        for (id_str, name, category, pattern_data, _quality_score, _usage_count) in rows {
            let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());

            match category.as_str() {
                "content" | "Primitives" | "Architecture" | "Landscape" | "Interactive" | "Environments" | "Vehicles" | "Wearables" => {
                    if let Ok(pattern) = serde_json::from_slice::<LearnedPattern>(&pattern_data) {
                        cache.content_patterns.insert(id, pattern);
                        content_count += 1;
                    }
                }
                "architectural" => {
                    if let Ok(pattern) = serde_json::from_slice::<ArchitecturalPattern>(&pattern_data) {
                        cache.architectural_patterns.insert(name, pattern);
                        architectural_count += 1;
                    }
                }
                "material" => {
                    if let Ok(pattern) = serde_json::from_slice::<MaterialPattern>(&pattern_data) {
                        cache.material_patterns.insert(name, pattern);
                        material_count += 1;
                    }
                }
                "spatial" => {
                    if let Ok(pattern) = serde_json::from_slice::<SpatialPattern>(&pattern_data) {
                        cache.spatial_patterns.insert(name, pattern);
                        spatial_count += 1;
                    }
                }
                "scripting" => {
                    if let Ok(pattern) = serde_json::from_slice::<ScriptingPattern>(&pattern_data) {
                        cache.scripting_patterns.insert(name, pattern);
                        scripting_count += 1;
                    }
                }
                "anti" => {
                    if let Ok(pattern) = serde_json::from_slice::<AntiPattern>(&pattern_data) {
                        cache.anti_patterns.insert(name, pattern);
                        anti_count += 1;
                    }
                }
                _ => {
                    tracing::debug!("Unknown pattern category: {}", category);
                }
            }
        }

        cache.last_sync = Some(Utc::now());
        cache.dirty = false;

        tracing::info!(
            "Loaded patterns: content={}, architectural={}, material={}, spatial={}, scripting={}, anti={}",
            content_count, architectural_count, material_count, spatial_count, scripting_count, anti_count
        );

        Ok(())
    }

    pub async fn delete_pattern(&self, id: Uuid) -> Result<bool> {
        let pool = self.db.get_pool()?;

        let rows_affected = match pool {
            crate::database::DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query("DELETE FROM ai_learned_patterns WHERE id = $1")
                    .bind(id.to_string())
                    .execute(pg_pool)
                    .await
                    .context("Failed to delete pattern from PostgreSQL")?
                    .rows_affected()
            }
            crate::database::DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query("DELETE FROM ai_learned_patterns WHERE id = ?")
                    .bind(id.to_string())
                    .execute(mysql_pool)
                    .await
                    .context("Failed to delete pattern from MySQL")?
                    .rows_affected()
            }
        };

        let mut cache = self.cache.write().await;
        cache.content_patterns.remove(&id);
        cache.dirty = true;

        Ok(rows_affected > 0)
    }

    pub async fn sync_to_disk(&self) -> Result<()> {
        let cache = self.cache.read().await;
        if !cache.dirty {
            tracing::debug!("No changes to sync");
            return Ok(());
        }
        drop(cache);

        tracing::info!("Syncing patterns to database");

        let cache = self.cache.read().await;

        for (id, pattern) in &cache.content_patterns {
            let pattern_data = serde_json::to_vec(pattern)?;
            let category = format!("{:?}", pattern.category);
            self.save_pattern_to_db(
                *id,
                &pattern.name,
                &category,
                &pattern_data,
                pattern.elegance_score,
                pattern.usage_frequency,
            ).await?;
        }

        drop(cache);

        let mut cache = self.cache.write().await;
        cache.dirty = false;
        cache.last_sync = Some(Utc::now());

        tracing::info!("Pattern sync complete");
        Ok(())
    }

    pub async fn record_metric(&self, pattern_id: Uuid, metric_type: &str, metric_value: f64) -> Result<()> {
        let pool = self.db.get_pool()?;
        let now = Utc::now();

        match pool {
            crate::database::DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query(
                    "INSERT INTO ai_learning_metrics (pattern_id, metric_type, metric_value, recorded_at) VALUES ($1, $2, $3, $4)"
                )
                .bind(pattern_id.to_string())
                .bind(metric_type)
                .bind(metric_value)
                .bind(now)
                .execute(pg_pool)
                .await
                .context("Failed to record metric in PostgreSQL")?;
            }
            crate::database::DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query(
                    "INSERT INTO ai_learning_metrics (pattern_id, metric_type, metric_value, recorded_at) VALUES (?, ?, ?, ?)"
                )
                .bind(pattern_id.to_string())
                .bind(metric_type)
                .bind(metric_value)
                .bind(now)
                .execute(mysql_pool)
                .await
                .context("Failed to record metric in MySQL")?;
            }
        }

        Ok(())
    }

    pub async fn get_pattern_metrics(&self, pattern_id: Uuid) -> Result<Vec<LearningMetric>> {
        let pool = self.db.get_pool()?;

        let rows: Vec<(i64, String, String, f64, DateTime<Utc>)> = match pool {
            crate::database::DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query_as(
                    "SELECT id, pattern_id, metric_type, metric_value, recorded_at FROM ai_learning_metrics WHERE pattern_id = $1 ORDER BY recorded_at DESC"
                )
                .bind(pattern_id.to_string())
                .fetch_all(pg_pool)
                .await
                .context("Failed to get metrics from PostgreSQL")?
            }
            crate::database::DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query_as(
                    "SELECT id, pattern_id, metric_type, metric_value, recorded_at FROM ai_learning_metrics WHERE pattern_id = ? ORDER BY recorded_at DESC"
                )
                .bind(pattern_id.to_string())
                .fetch_all(mysql_pool)
                .await
                .context("Failed to get metrics from MySQL")?
            }
        };

        Ok(rows.into_iter().map(|(id, pattern_id_str, metric_type, metric_value, recorded_at)| {
            LearningMetric {
                id,
                pattern_id: Uuid::parse_str(&pattern_id_str).unwrap_or(pattern_id),
                metric_type,
                metric_value,
                recorded_at,
            }
        }).collect())
    }

    pub async fn get_content_patterns(&self) -> HashMap<Uuid, LearnedPattern> {
        let cache = self.cache.read().await;
        cache.content_patterns.clone()
    }

    pub async fn get_architectural_patterns(&self) -> HashMap<String, ArchitecturalPattern> {
        let cache = self.cache.read().await;
        cache.architectural_patterns.clone()
    }

    pub async fn get_material_patterns(&self) -> HashMap<String, MaterialPattern> {
        let cache = self.cache.read().await;
        cache.material_patterns.clone()
    }

    pub async fn get_spatial_patterns(&self) -> HashMap<String, SpatialPattern> {
        let cache = self.cache.read().await;
        cache.spatial_patterns.clone()
    }

    pub async fn get_scripting_patterns(&self) -> HashMap<String, ScriptingPattern> {
        let cache = self.cache.read().await;
        cache.scripting_patterns.clone()
    }

    pub async fn get_anti_patterns(&self) -> HashMap<String, AntiPattern> {
        let cache = self.cache.read().await;
        cache.anti_patterns.clone()
    }

    pub async fn get_stats(&self) -> PatternRepositoryStats {
        let cache = self.cache.read().await;
        PatternRepositoryStats {
            content_patterns: cache.content_patterns.len(),
            architectural_patterns: cache.architectural_patterns.len(),
            material_patterns: cache.material_patterns.len(),
            spatial_patterns: cache.spatial_patterns.len(),
            scripting_patterns: cache.scripting_patterns.len(),
            anti_patterns: cache.anti_patterns.len(),
            last_sync: cache.last_sync,
            has_pending_changes: cache.dirty,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRepositoryStats {
    pub content_patterns: usize,
    pub architectural_patterns: usize,
    pub material_patterns: usize,
    pub spatial_patterns: usize,
    pub scripting_patterns: usize,
    pub anti_patterns: usize,
    pub last_sync: Option<DateTime<Utc>>,
    pub has_pending_changes: bool,
}
