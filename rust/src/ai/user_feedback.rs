use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::database::{DatabaseManager, DatabasePoolRef};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub id: Option<i64>,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub feedback_type: FeedbackType,
    pub feedback_value: Option<f64>,
    pub feedback_text: Option<String>,
    pub context_data: Option<FeedbackContext>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackType {
    Rating,
    Like,
    Dislike,
    Report,
    Skip,
    Accept,
    Reject,
    Custom(String),
}

impl FeedbackType {
    pub fn as_str(&self) -> &str {
        match self {
            FeedbackType::Rating => "rating",
            FeedbackType::Like => "like",
            FeedbackType::Dislike => "dislike",
            FeedbackType::Report => "report",
            FeedbackType::Skip => "skip",
            FeedbackType::Accept => "accept",
            FeedbackType::Reject => "reject",
            FeedbackType::Custom(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "rating" => FeedbackType::Rating,
            "like" => FeedbackType::Like,
            "dislike" => FeedbackType::Dislike,
            "report" => FeedbackType::Report,
            "skip" => FeedbackType::Skip,
            "accept" => FeedbackType::Accept,
            "reject" => FeedbackType::Reject,
            other => FeedbackType::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackContext {
    pub search_query: Option<String>,
    pub session_id: Option<String>,
    pub recommendation_source: Option<String>,
    pub interaction_duration_ms: Option<u64>,
    pub additional_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationOutcome {
    pub id: Option<i64>,
    pub recommendation_id: Uuid,
    pub user_id: Uuid,
    pub recommendation_type: RecommendationType,
    pub was_accepted: bool,
    pub time_to_decision_ms: Option<i64>,
    pub subsequent_satisfaction: Option<f64>,
    pub context_data: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    Content,
    Pattern,
    Search,
    Performance,
    Style,
    Custom(String),
}

impl RecommendationType {
    pub fn as_str(&self) -> &str {
        match self {
            RecommendationType::Content => "content",
            RecommendationType::Pattern => "pattern",
            RecommendationType::Search => "search",
            RecommendationType::Performance => "performance",
            RecommendationType::Style => "style",
            RecommendationType::Custom(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "content" => RecommendationType::Content,
            "pattern" => RecommendationType::Pattern,
            "search" => RecommendationType::Search,
            "performance" => RecommendationType::Performance,
            "style" => RecommendationType::Style,
            other => RecommendationType::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub user_id: Uuid,
    pub preferred_categories: HashMap<String, f64>,
    pub preferred_styles: HashMap<String, f64>,
    pub interaction_history_summary: InteractionSummary,
    pub preference_score: f64,
    pub total_interactions: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionSummary {
    pub likes: u64,
    pub dislikes: u64,
    pub accepts: u64,
    pub rejects: u64,
    pub ratings_sum: f64,
    pub ratings_count: u64,
}

impl InteractionSummary {
    pub fn average_rating(&self) -> Option<f64> {
        if self.ratings_count > 0 {
            Some(self.ratings_sum / self.ratings_count as f64)
        } else {
            None
        }
    }

    pub fn acceptance_rate(&self) -> f64 {
        let total = self.accepts + self.rejects;
        if total > 0 {
            self.accepts as f64 / total as f64
        } else {
            0.5
        }
    }

    pub fn satisfaction_score(&self) -> f64 {
        let like_ratio = if self.likes + self.dislikes > 0 {
            self.likes as f64 / (self.likes + self.dislikes) as f64
        } else {
            0.5
        };

        let acceptance = self.acceptance_rate();
        let avg_rating = self.average_rating().unwrap_or(0.5);

        (like_ratio * 0.3 + acceptance * 0.3 + avg_rating * 0.4).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAffinity {
    pub user_id: Uuid,
    pub pattern_id: Uuid,
    pub affinity_score: f64,
    pub interaction_count: u64,
    pub last_interaction_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct FeedbackRepository {
    db: Arc<DatabaseManager>,
    feedback_cache: Arc<RwLock<HashMap<Uuid, Vec<UserFeedback>>>>,
    preferences_cache: Arc<RwLock<HashMap<Uuid, UserPreferences>>>,
    affinity_cache: Arc<RwLock<HashMap<(Uuid, Uuid), PatternAffinity>>>,
}

impl FeedbackRepository {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            feedback_cache: Arc::new(RwLock::new(HashMap::new())),
            preferences_cache: Arc::new(RwLock::new(HashMap::new())),
            affinity_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn save_feedback(&self, feedback: &UserFeedback) -> Result<i64> {
        let context_json = feedback.context_data.as_ref()
            .map(|c| serde_json::to_string(c).unwrap_or_default());

        let pool = self.db.get_pool()?;
        let now = feedback.created_at;

        let last_id = match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let row: (i64,) = sqlx::query_as(
                    r#"
                    INSERT INTO ai_user_feedback
                    (user_id, content_id, feedback_type, feedback_value, feedback_text, context_data, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    RETURNING id
                    "#
                )
                .bind(feedback.user_id.to_string())
                .bind(feedback.content_id.to_string())
                .bind(feedback.feedback_type.as_str())
                .bind(feedback.feedback_value)
                .bind(&feedback.feedback_text)
                .bind(&context_json)
                .bind(now)
                .fetch_one(pg_pool)
                .await
                .context("Failed to save feedback to PostgreSQL")?;
                row.0
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO ai_user_feedback
                    (user_id, content_id, feedback_type, feedback_value, feedback_text, context_data, created_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(feedback.user_id.to_string())
                .bind(feedback.content_id.to_string())
                .bind(feedback.feedback_type.as_str())
                .bind(feedback.feedback_value)
                .bind(&feedback.feedback_text)
                .bind(&context_json)
                .bind(now)
                .execute(mysql_pool)
                .await
                .context("Failed to save feedback to MySQL")?;
                result.last_insert_id() as i64
            }
        };

        let mut cache = self.feedback_cache.write().await;
        cache
            .entry(feedback.user_id)
            .or_default()
            .push(feedback.clone());

        tracing::debug!("Saved feedback {} for user {}", last_id, feedback.user_id);
        Ok(last_id)
    }

    pub async fn save_outcome(&self, outcome: &RecommendationOutcome) -> Result<i64> {
        let pool = self.db.get_pool()?;
        let now = outcome.created_at;

        let last_id = match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let row: (i64,) = sqlx::query_as(
                    r#"
                    INSERT INTO ai_recommendation_outcomes
                    (recommendation_id, user_id, recommendation_type, was_accepted,
                     time_to_decision_ms, subsequent_satisfaction, context_data, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    RETURNING id
                    "#
                )
                .bind(outcome.recommendation_id.to_string())
                .bind(outcome.user_id.to_string())
                .bind(outcome.recommendation_type.as_str())
                .bind(outcome.was_accepted)
                .bind(outcome.time_to_decision_ms)
                .bind(outcome.subsequent_satisfaction)
                .bind(&outcome.context_data)
                .bind(now)
                .fetch_one(pg_pool)
                .await
                .context("Failed to save outcome to PostgreSQL")?;
                row.0
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO ai_recommendation_outcomes
                    (recommendation_id, user_id, recommendation_type, was_accepted,
                     time_to_decision_ms, subsequent_satisfaction, context_data, created_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(outcome.recommendation_id.to_string())
                .bind(outcome.user_id.to_string())
                .bind(outcome.recommendation_type.as_str())
                .bind(outcome.was_accepted)
                .bind(outcome.time_to_decision_ms)
                .bind(outcome.subsequent_satisfaction)
                .bind(&outcome.context_data)
                .bind(now)
                .execute(mysql_pool)
                .await
                .context("Failed to save outcome to MySQL")?;
                result.last_insert_id() as i64
            }
        };

        tracing::debug!("Saved outcome for recommendation {}", outcome.recommendation_id);
        Ok(last_id)
    }

    pub async fn get_user_feedback(&self, user_id: Uuid, limit: u32) -> Result<Vec<UserFeedback>> {
        let pool = self.db.get_pool()?;

        let feedback = match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let rows: Vec<(i64, String, String, String, Option<f64>, Option<String>, Option<String>, DateTime<Utc>)> = sqlx::query_as(
                    r#"
                    SELECT id, user_id, content_id, feedback_type, feedback_value,
                           feedback_text, context_data, created_at
                    FROM ai_user_feedback
                    WHERE user_id = $1
                    ORDER BY created_at DESC
                    LIMIT $2
                    "#
                )
                .bind(user_id.to_string())
                .bind(limit as i64)
                .fetch_all(pg_pool)
                .await
                .context("Failed to get feedback from PostgreSQL")?;

                self.rows_to_feedback(rows)
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let rows: Vec<(i64, String, String, String, Option<f64>, Option<String>, Option<String>, DateTime<Utc>)> = sqlx::query_as(
                    r#"
                    SELECT id, user_id, content_id, feedback_type, feedback_value,
                           feedback_text, context_data, created_at
                    FROM ai_user_feedback
                    WHERE user_id = ?
                    ORDER BY created_at DESC
                    LIMIT ?
                    "#
                )
                .bind(user_id.to_string())
                .bind(limit)
                .fetch_all(mysql_pool)
                .await
                .context("Failed to get feedback from MySQL")?;

                self.rows_to_feedback(rows)
            }
        };

        Ok(feedback)
    }

    fn rows_to_feedback(&self, rows: Vec<(i64, String, String, String, Option<f64>, Option<String>, Option<String>, DateTime<Utc>)>) -> Vec<UserFeedback> {
        rows.into_iter().map(|row| {
            let context_data = row.6.as_ref().and_then(|s| serde_json::from_str(s).ok());
            UserFeedback {
                id: Some(row.0),
                user_id: Uuid::parse_str(&row.1).unwrap_or_default(),
                content_id: Uuid::parse_str(&row.2).unwrap_or_default(),
                feedback_type: FeedbackType::from_str(&row.3),
                feedback_value: row.4,
                feedback_text: row.5,
                context_data,
                created_at: row.7,
            }
        }).collect()
    }

    pub async fn get_user_preferences(&self, user_id: Uuid) -> Result<Option<UserPreferences>> {
        {
            let cache = self.preferences_cache.read().await;
            if let Some(prefs) = cache.get(&user_id) {
                return Ok(Some(prefs.clone()));
            }
        }

        let pool = self.db.get_pool()?;

        let row = match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query_as::<_, (String, String, i64, Option<DateTime<Utc>>, f64, DateTime<Utc>)>(
                    r#"
                    SELECT user_id, preference_data, total_interactions,
                           last_interaction_at, preference_score, updated_at
                    FROM ai_user_preferences
                    WHERE user_id = $1
                    "#
                )
                .bind(user_id.to_string())
                .fetch_optional(pg_pool)
                .await
                .context("Failed to get preferences from PostgreSQL")?
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query_as::<_, (String, String, i64, Option<DateTime<Utc>>, f64, DateTime<Utc>)>(
                    r#"
                    SELECT user_id, preference_data, total_interactions,
                           last_interaction_at, preference_score, updated_at
                    FROM ai_user_preferences
                    WHERE user_id = ?
                    "#
                )
                .bind(user_id.to_string())
                .fetch_optional(mysql_pool)
                .await
                .context("Failed to get preferences from MySQL")?
            }
        };

        if let Some(row) = row {
            let pref_data: serde_json::Value = serde_json::from_str(&row.1).unwrap_or_default();
            let prefs = UserPreferences {
                user_id,
                preferred_categories: pref_data.get("categories")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default(),
                preferred_styles: pref_data.get("styles")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default(),
                interaction_history_summary: pref_data.get("summary")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default(),
                preference_score: row.4,
                total_interactions: row.2 as u64,
                last_updated: row.5,
            };

            let mut cache = self.preferences_cache.write().await;
            cache.insert(user_id, prefs.clone());

            Ok(Some(prefs))
        } else {
            Ok(None)
        }
    }

    pub async fn save_user_preferences(&self, prefs: &UserPreferences) -> Result<()> {
        let pref_data = serde_json::json!({
            "categories": prefs.preferred_categories,
            "styles": prefs.preferred_styles,
            "summary": prefs.interaction_history_summary,
        });

        let pool = self.db.get_pool()?;
        let now = Utc::now();

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO ai_user_preferences
                    (user_id, preference_data, total_interactions, last_interaction_at, preference_score, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    ON CONFLICT (user_id) DO UPDATE SET
                        preference_data = EXCLUDED.preference_data,
                        total_interactions = EXCLUDED.total_interactions,
                        last_interaction_at = EXCLUDED.last_interaction_at,
                        preference_score = EXCLUDED.preference_score,
                        updated_at = EXCLUDED.updated_at
                    "#
                )
                .bind(prefs.user_id.to_string())
                .bind(pref_data.to_string())
                .bind(prefs.total_interactions as i64)
                .bind(prefs.last_updated)
                .bind(prefs.preference_score)
                .bind(now)
                .execute(pg_pool)
                .await
                .context("Failed to save preferences to PostgreSQL")?;
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO ai_user_preferences
                    (user_id, preference_data, total_interactions, last_interaction_at, preference_score, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?)
                    ON DUPLICATE KEY UPDATE
                        preference_data = VALUES(preference_data),
                        total_interactions = VALUES(total_interactions),
                        last_interaction_at = VALUES(last_interaction_at),
                        preference_score = VALUES(preference_score),
                        updated_at = VALUES(updated_at)
                    "#
                )
                .bind(prefs.user_id.to_string())
                .bind(pref_data.to_string())
                .bind(prefs.total_interactions as i64)
                .bind(prefs.last_updated)
                .bind(prefs.preference_score)
                .bind(now)
                .execute(mysql_pool)
                .await
                .context("Failed to save preferences to MySQL")?;
            }
        };

        let mut cache = self.preferences_cache.write().await;
        cache.insert(prefs.user_id, prefs.clone());

        Ok(())
    }

    pub async fn update_pattern_affinity(
        &self,
        user_id: Uuid,
        pattern_id: Uuid,
        score_delta: f64,
    ) -> Result<PatternAffinity> {
        let cache_key = (user_id, pattern_id);
        let now = Utc::now();

        let pool = self.db.get_pool()?;

        let existing = match &pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query_as::<_, (f64, i64)>(
                    r#"
                    SELECT affinity_score, interaction_count
                    FROM ai_pattern_affinity
                    WHERE user_id = $1 AND pattern_id = $2
                    "#
                )
                .bind(user_id.to_string())
                .bind(pattern_id.to_string())
                .fetch_optional(*pg_pool)
                .await
                .context("Failed to get affinity from PostgreSQL")?
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query_as::<_, (f64, i64)>(
                    r#"
                    SELECT affinity_score, interaction_count
                    FROM ai_pattern_affinity
                    WHERE user_id = ? AND pattern_id = ?
                    "#
                )
                .bind(user_id.to_string())
                .bind(pattern_id.to_string())
                .fetch_optional(*mysql_pool)
                .await
                .context("Failed to get affinity from MySQL")?
            }
        };

        let (new_score, new_count) = if let Some((old_score, old_count)) = existing {
            let new_score = (old_score + score_delta).clamp(0.0, 1.0);
            (new_score, old_count + 1)
        } else {
            ((0.5 + score_delta).clamp(0.0, 1.0), 1)
        };

        match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO ai_pattern_affinity
                    (user_id, pattern_id, affinity_score, interaction_count, last_interaction_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $5)
                    ON CONFLICT (user_id, pattern_id) DO UPDATE SET
                        affinity_score = EXCLUDED.affinity_score,
                        interaction_count = EXCLUDED.interaction_count,
                        last_interaction_at = EXCLUDED.last_interaction_at,
                        updated_at = EXCLUDED.updated_at
                    "#
                )
                .bind(user_id.to_string())
                .bind(pattern_id.to_string())
                .bind(new_score)
                .bind(new_count)
                .bind(now)
                .execute(pg_pool)
                .await
                .context("Failed to save affinity to PostgreSQL")?;
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO ai_pattern_affinity
                    (user_id, pattern_id, affinity_score, interaction_count, last_interaction_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?)
                    ON DUPLICATE KEY UPDATE
                        affinity_score = VALUES(affinity_score),
                        interaction_count = VALUES(interaction_count),
                        last_interaction_at = VALUES(last_interaction_at),
                        updated_at = VALUES(updated_at)
                    "#
                )
                .bind(user_id.to_string())
                .bind(pattern_id.to_string())
                .bind(new_score)
                .bind(new_count)
                .bind(now)
                .bind(now)
                .execute(mysql_pool)
                .await
                .context("Failed to save affinity to MySQL")?;
            }
        };

        let affinity = PatternAffinity {
            user_id,
            pattern_id,
            affinity_score: new_score,
            interaction_count: new_count as u64,
            last_interaction_at: now,
        };

        let mut cache = self.affinity_cache.write().await;
        cache.insert(cache_key, affinity.clone());

        Ok(affinity)
    }

    pub async fn get_user_affinities(&self, user_id: Uuid) -> Result<Vec<PatternAffinity>> {
        let pool = self.db.get_pool()?;

        let affinities = match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                let rows: Vec<(String, String, f64, i64, DateTime<Utc>)> = sqlx::query_as(
                    r#"
                    SELECT user_id, pattern_id, affinity_score, interaction_count, last_interaction_at
                    FROM ai_pattern_affinity
                    WHERE user_id = $1
                    ORDER BY affinity_score DESC
                    "#
                )
                .bind(user_id.to_string())
                .fetch_all(pg_pool)
                .await
                .context("Failed to get affinities from PostgreSQL")?;

                self.rows_to_affinities(rows)
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                let rows: Vec<(String, String, f64, i64, DateTime<Utc>)> = sqlx::query_as(
                    r#"
                    SELECT user_id, pattern_id, affinity_score, interaction_count, last_interaction_at
                    FROM ai_pattern_affinity
                    WHERE user_id = ?
                    ORDER BY affinity_score DESC
                    "#
                )
                .bind(user_id.to_string())
                .fetch_all(mysql_pool)
                .await
                .context("Failed to get affinities from MySQL")?;

                self.rows_to_affinities(rows)
            }
        };

        Ok(affinities)
    }

    fn rows_to_affinities(&self, rows: Vec<(String, String, f64, i64, DateTime<Utc>)>) -> Vec<PatternAffinity> {
        rows.into_iter().map(|row| {
            PatternAffinity {
                user_id: Uuid::parse_str(&row.0).unwrap_or_default(),
                pattern_id: Uuid::parse_str(&row.1).unwrap_or_default(),
                affinity_score: row.2,
                interaction_count: row.3 as u64,
                last_interaction_at: row.4,
            }
        }).collect()
    }

    pub async fn get_acceptance_rate(&self, user_id: Uuid, rec_type: Option<&RecommendationType>) -> Result<f64> {
        let pool = self.db.get_pool()?;

        let (accepted, total): (i64, i64) = match pool {
            DatabasePoolRef::PostgreSQL(pg_pool) => {
                if let Some(rt) = rec_type {
                    sqlx::query_as(
                        r#"
                        SELECT
                            COALESCE(SUM(CASE WHEN was_accepted THEN 1 ELSE 0 END), 0),
                            COUNT(*)
                        FROM ai_recommendation_outcomes
                        WHERE user_id = $1 AND recommendation_type = $2
                        "#
                    )
                    .bind(user_id.to_string())
                    .bind(rt.as_str())
                    .fetch_one(pg_pool)
                    .await
                    .context("Failed to get acceptance rate from PostgreSQL")?
                } else {
                    sqlx::query_as(
                        r#"
                        SELECT
                            COALESCE(SUM(CASE WHEN was_accepted THEN 1 ELSE 0 END), 0),
                            COUNT(*)
                        FROM ai_recommendation_outcomes
                        WHERE user_id = $1
                        "#
                    )
                    .bind(user_id.to_string())
                    .fetch_one(pg_pool)
                    .await
                    .context("Failed to get acceptance rate from PostgreSQL")?
                }
            }
            DatabasePoolRef::MySQL(mysql_pool) => {
                if let Some(rt) = rec_type {
                    sqlx::query_as(
                        r#"
                        SELECT
                            COALESCE(SUM(CASE WHEN was_accepted = 1 THEN 1 ELSE 0 END), 0),
                            COUNT(*)
                        FROM ai_recommendation_outcomes
                        WHERE user_id = ? AND recommendation_type = ?
                        "#
                    )
                    .bind(user_id.to_string())
                    .bind(rt.as_str())
                    .fetch_one(mysql_pool)
                    .await
                    .context("Failed to get acceptance rate from MySQL")?
                } else {
                    sqlx::query_as(
                        r#"
                        SELECT
                            COALESCE(SUM(CASE WHEN was_accepted = 1 THEN 1 ELSE 0 END), 0),
                            COUNT(*)
                        FROM ai_recommendation_outcomes
                        WHERE user_id = ?
                        "#
                    )
                    .bind(user_id.to_string())
                    .fetch_one(mysql_pool)
                    .await
                    .context("Failed to get acceptance rate from MySQL")?
                }
            }
        };

        if total > 0 {
            Ok(accepted as f64 / total as f64)
        } else {
            Ok(0.5)
        }
    }

    pub async fn clear_cache(&self) {
        self.feedback_cache.write().await.clear();
        self.preferences_cache.write().await.clear();
        self.affinity_cache.write().await.clear();
    }
}

#[derive(Debug)]
pub struct FeedbackProcessor {
    repository: Arc<FeedbackRepository>,
    learning_threshold: u32,
    feedback_count: Arc<RwLock<u32>>,
}

impl FeedbackProcessor {
    pub fn new(repository: Arc<FeedbackRepository>) -> Self {
        Self {
            repository,
            learning_threshold: 10,
            feedback_count: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn record_feedback(&self, feedback: UserFeedback) -> Result<()> {
        self.repository.save_feedback(&feedback).await?;

        self.update_pattern_affinity_from_feedback(&feedback).await?;

        let mut count = self.feedback_count.write().await;
        *count += 1;

        if *count >= self.learning_threshold {
            *count = 0;
            drop(count);
            self.trigger_pattern_relearning().await?;
        }

        Ok(())
    }

    pub async fn record_outcome(&self, outcome: RecommendationOutcome) -> Result<()> {
        self.repository.save_outcome(&outcome).await?;
        Ok(())
    }

    async fn update_pattern_affinity_from_feedback(&self, feedback: &UserFeedback) -> Result<()> {
        let score_delta = match &feedback.feedback_type {
            FeedbackType::Like => 0.1,
            FeedbackType::Dislike => -0.1,
            FeedbackType::Accept => 0.15,
            FeedbackType::Reject => -0.15,
            FeedbackType::Rating => {
                let rating = feedback.feedback_value.unwrap_or(0.5);
                (rating - 0.5) * 0.2
            }
            FeedbackType::Report => -0.3,
            FeedbackType::Skip => -0.05,
            FeedbackType::Custom(_) => 0.0,
        };

        if score_delta.abs() > 0.001 {
            self.repository
                .update_pattern_affinity(feedback.user_id, feedback.content_id, score_delta)
                .await?;
        }

        Ok(())
    }

    async fn trigger_pattern_relearning(&self) -> Result<()> {
        tracing::info!("Triggering pattern relearning based on accumulated feedback");
        Ok(())
    }

    pub async fn get_user_preferences(&self, user_id: Uuid) -> Result<UserPreferences> {
        if let Some(prefs) = self.repository.get_user_preferences(user_id).await? {
            return Ok(prefs);
        }

        let feedback = self.repository.get_user_feedback(user_id, 100).await?;
        let prefs = self.build_preferences_from_feedback(user_id, &feedback).await;

        self.repository.save_user_preferences(&prefs).await?;

        Ok(prefs)
    }

    async fn build_preferences_from_feedback(
        &self,
        user_id: Uuid,
        feedback: &[UserFeedback],
    ) -> UserPreferences {
        let mut summary = InteractionSummary::default();
        let mut categories: HashMap<String, f64> = HashMap::new();

        for f in feedback {
            match &f.feedback_type {
                FeedbackType::Like => summary.likes += 1,
                FeedbackType::Dislike => summary.dislikes += 1,
                FeedbackType::Accept => summary.accepts += 1,
                FeedbackType::Reject => summary.rejects += 1,
                FeedbackType::Rating => {
                    if let Some(val) = f.feedback_value {
                        summary.ratings_sum += val;
                        summary.ratings_count += 1;
                    }
                }
                _ => {}
            }

            if let Some(ctx) = &f.context_data {
                if let Some(source) = &ctx.recommendation_source {
                    *categories.entry(source.clone()).or_insert(0.0) += match &f.feedback_type {
                        FeedbackType::Like | FeedbackType::Accept => 1.0,
                        FeedbackType::Dislike | FeedbackType::Reject => -1.0,
                        _ => 0.0,
                    };
                }
            }
        }

        UserPreferences {
            user_id,
            preferred_categories: categories,
            preferred_styles: HashMap::new(),
            interaction_history_summary: summary.clone(),
            preference_score: summary.satisfaction_score(),
            total_interactions: feedback.len() as u64,
            last_updated: Utc::now(),
        }
    }

    pub async fn adjust_recommendations<T: Clone>(
        &self,
        base_recommendations: Vec<(T, f64)>,
        user_id: Uuid,
    ) -> Result<Vec<(T, f64)>> {
        let prefs = self.get_user_preferences(user_id).await?;
        let _affinities = self.repository.get_user_affinities(user_id).await?;

        let preference_boost = prefs.preference_score;

        let adjusted: Vec<(T, f64)> = base_recommendations
            .into_iter()
            .map(|(item, score)| {
                let adjusted_score = score * (0.8 + preference_boost * 0.4);
                (item, adjusted_score)
            })
            .collect();

        Ok(adjusted)
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            user_id: Uuid::nil(),
            preferred_categories: HashMap::new(),
            preferred_styles: HashMap::new(),
            interaction_history_summary: InteractionSummary::default(),
            preference_score: 0.5,
            total_interactions: 0,
            last_updated: Utc::now(),
        }
    }
}
