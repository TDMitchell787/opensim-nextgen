use super::{AIError, PerformanceRecommendation};
use crate::database::DatabaseManager;
use crate::monitoring::metrics::MetricsCollector;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub snapshot_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub network_connections: usize,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
    pub physics_fps: f64,
    pub active_regions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedRecommendation {
    pub recommendation_id: Uuid,
    pub recommendation: PerformanceRecommendation,
    pub issued_at: DateTime<Utc>,
    pub before_snapshot: MetricsSnapshot,
    pub after_snapshot: Option<MetricsSnapshot>,
    pub was_applied: bool,
    pub applied_at: Option<DateTime<Utc>>,
    pub measured_at: Option<DateTime<Utc>>,
    pub effectiveness: Option<RecommendationEffectiveness>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationEffectiveness {
    pub cpu_change_percent: f64,
    pub memory_change_percent: f64,
    pub response_time_change_percent: f64,
    pub error_rate_change: f64,
    pub cache_hit_change: f64,
    pub overall_effectiveness_score: f64,
    pub was_effective: bool,
    pub analysis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationImpact {
    pub recommendation_id: Uuid,
    pub category: String,
    pub cpu_change: f64,
    pub memory_change: i64,
    pub response_time_change: f64,
    pub was_effective: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessReport {
    pub total_recommendations: usize,
    pub applied_recommendations: usize,
    pub effective_recommendations: usize,
    pub effectiveness_rate: f64,
    pub category_breakdown: HashMap<String, CategoryEffectiveness>,
    pub top_effective_recommendations: Vec<TrackedRecommendation>,
    pub ineffective_recommendations: Vec<TrackedRecommendation>,
    pub report_generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryEffectiveness {
    pub category: String,
    pub total_issued: usize,
    pub total_applied: usize,
    pub total_effective: usize,
    pub avg_effectiveness_score: f64,
    pub avg_cpu_improvement: f64,
    pub avg_response_time_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABExperiment {
    pub experiment_id: Uuid,
    pub name: String,
    pub description: String,
    pub recommendation_category: String,
    pub control_group: Vec<Uuid>,
    pub treatment_group: Vec<Uuid>,
    pub metric_to_track: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: ExperimentStatus,
    pub results: Option<ABExperimentResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABExperimentResults {
    pub control_avg_metric: f64,
    pub treatment_avg_metric: f64,
    pub improvement_percent: f64,
    pub statistical_significance: f64,
    pub sample_size_control: usize,
    pub sample_size_treatment: usize,
    pub conclusion: String,
}

#[derive(Debug)]
pub struct PerformanceRecommendationTracker {
    metrics: Arc<MetricsCollector>,
    db: Arc<DatabaseManager>,
    tracked_recommendations: Arc<RwLock<HashMap<Uuid, TrackedRecommendation>>>,
    experiments: Arc<RwLock<HashMap<Uuid, ABExperiment>>>,
    config: TrackerConfig,
}

#[derive(Debug, Clone)]
pub struct TrackerConfig {
    pub measurement_delay_seconds: u64,
    pub min_effectiveness_threshold: f64,
    pub snapshot_retention_hours: u32,
    pub enable_ab_testing: bool,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            measurement_delay_seconds: 300,
            min_effectiveness_threshold: 0.05,
            snapshot_retention_hours: 72,
            enable_ab_testing: true,
        }
    }
}

impl PerformanceRecommendationTracker {
    pub async fn new(
        metrics: Arc<MetricsCollector>,
        db: Arc<DatabaseManager>,
    ) -> Result<Arc<Self>, AIError> {
        let tracker = Self {
            metrics,
            db,
            tracked_recommendations: Arc::new(RwLock::new(HashMap::new())),
            experiments: Arc::new(RwLock::new(HashMap::new())),
            config: TrackerConfig::default(),
        };

        Ok(Arc::new(tracker))
    }

    pub async fn capture_metrics_snapshot(&self) -> Result<MetricsSnapshot, AIError> {
        let current = self
            .metrics
            .get_current_metrics()
            .await
            .map_err(|e| AIError::InferenceFailed(format!("Failed to get metrics: {}", e)))?;

        Ok(MetricsSnapshot {
            snapshot_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            cpu_usage: current.cpu_usage,
            memory_usage: current.memory_usage,
            network_connections: current.network_connections,
            avg_response_time_ms: current.avg_response_time.as_millis() as f64,
            error_rate: current.error_rate,
            cache_hit_rate: current.asset_cache_hit_rate,
            physics_fps: current.physics_fps,
            active_regions: current.active_regions,
        })
    }

    pub async fn track_recommendation(
        &self,
        recommendation: &PerformanceRecommendation,
    ) -> Result<Uuid, AIError> {
        let recommendation_id = Uuid::new_v4();
        let before_snapshot = self.capture_metrics_snapshot().await?;

        let tracked = TrackedRecommendation {
            recommendation_id,
            recommendation: recommendation.clone(),
            issued_at: Utc::now(),
            before_snapshot,
            after_snapshot: None,
            was_applied: false,
            applied_at: None,
            measured_at: None,
            effectiveness: None,
        };

        self.tracked_recommendations
            .write()
            .await
            .insert(recommendation_id, tracked);

        self.persist_recommendation(recommendation_id).await?;

        Ok(recommendation_id)
    }

    pub async fn mark_recommendation_applied(
        &self,
        recommendation_id: Uuid,
    ) -> Result<(), AIError> {
        let mut recommendations = self.tracked_recommendations.write().await;

        if let Some(tracked) = recommendations.get_mut(&recommendation_id) {
            tracked.was_applied = true;
            tracked.applied_at = Some(Utc::now());
            Ok(())
        } else {
            Err(AIError::ConfigurationError(format!(
                "Recommendation {} not found",
                recommendation_id
            )))
        }
    }

    pub async fn measure_recommendation_impact(
        &self,
        recommendation_id: Uuid,
    ) -> Result<RecommendationImpact, AIError> {
        let after_snapshot = self.capture_metrics_snapshot().await?;

        let mut recommendations = self.tracked_recommendations.write().await;

        let tracked = recommendations.get_mut(&recommendation_id).ok_or_else(|| {
            AIError::ConfigurationError(format!("Recommendation {} not found", recommendation_id))
        })?;

        tracked.after_snapshot = Some(after_snapshot.clone());
        tracked.measured_at = Some(Utc::now());

        let before = &tracked.before_snapshot;

        let cpu_change = after_snapshot.cpu_usage - before.cpu_usage;
        let memory_change = after_snapshot.memory_usage as i64 - before.memory_usage as i64;
        let response_time_change =
            after_snapshot.avg_response_time_ms - before.avg_response_time_ms;
        let error_rate_change = after_snapshot.error_rate - before.error_rate;
        let cache_hit_change = after_snapshot.cache_hit_rate - before.cache_hit_rate;

        let cpu_change_percent = if before.cpu_usage > 0.0 {
            (cpu_change / before.cpu_usage) * 100.0
        } else {
            0.0
        };

        let memory_change_percent = if before.memory_usage > 0 {
            (memory_change as f64 / before.memory_usage as f64) * 100.0
        } else {
            0.0
        };

        let response_time_change_percent = if before.avg_response_time_ms > 0.0 {
            (response_time_change / before.avg_response_time_ms) * 100.0
        } else {
            0.0
        };

        let overall_score = self.calculate_effectiveness_score(
            cpu_change_percent,
            memory_change_percent,
            response_time_change_percent,
            error_rate_change,
            cache_hit_change,
            &tracked.recommendation.category,
        );

        let was_effective = overall_score > self.config.min_effectiveness_threshold;

        let analysis = self.generate_effectiveness_analysis(
            cpu_change_percent,
            memory_change_percent,
            response_time_change_percent,
            error_rate_change,
            cache_hit_change,
            was_effective,
        );

        tracked.effectiveness = Some(RecommendationEffectiveness {
            cpu_change_percent,
            memory_change_percent,
            response_time_change_percent,
            error_rate_change,
            cache_hit_change,
            overall_effectiveness_score: overall_score,
            was_effective,
            analysis,
        });

        let confidence = self.calculate_confidence(before, &after_snapshot);

        Ok(RecommendationImpact {
            recommendation_id,
            category: tracked.recommendation.category.clone(),
            cpu_change,
            memory_change,
            response_time_change,
            was_effective,
            confidence,
        })
    }

    fn calculate_effectiveness_score(
        &self,
        cpu_change_percent: f64,
        memory_change_percent: f64,
        response_time_change_percent: f64,
        error_rate_change: f64,
        cache_hit_change: f64,
        category: &str,
    ) -> f64 {
        let weights = match category {
            "Load Balancing" => (0.4, 0.2, 0.2, 0.1, 0.1),
            "Caching" => (0.1, 0.1, 0.2, 0.1, 0.5),
            "Memory Optimization" => (0.1, 0.5, 0.2, 0.1, 0.1),
            "Response Time" => (0.2, 0.1, 0.5, 0.1, 0.1),
            "Error Reduction" => (0.1, 0.1, 0.1, 0.6, 0.1),
            _ => (0.25, 0.25, 0.25, 0.15, 0.1),
        };

        let cpu_score = (-cpu_change_percent / 10.0).max(-1.0).min(1.0);
        let memory_score = (-memory_change_percent / 10.0).max(-1.0).min(1.0);
        let response_score = (-response_time_change_percent / 10.0).max(-1.0).min(1.0);
        let error_score = (-error_rate_change * 100.0).max(-1.0).min(1.0);
        let cache_score = (cache_hit_change * 10.0).max(-1.0).min(1.0);

        (cpu_score * weights.0)
            + (memory_score * weights.1)
            + (response_score * weights.2)
            + (error_score * weights.3)
            + (cache_score * weights.4)
    }

    fn generate_effectiveness_analysis(
        &self,
        cpu_change: f64,
        memory_change: f64,
        response_time_change: f64,
        error_rate_change: f64,
        cache_hit_change: f64,
        was_effective: bool,
    ) -> String {
        let mut analysis = Vec::new();

        if cpu_change < -5.0 {
            analysis.push(format!("CPU usage improved by {:.1}%", -cpu_change));
        } else if cpu_change > 5.0 {
            analysis.push(format!("CPU usage increased by {:.1}%", cpu_change));
        }

        if memory_change < -5.0 {
            analysis.push(format!("Memory usage improved by {:.1}%", -memory_change));
        } else if memory_change > 5.0 {
            analysis.push(format!("Memory usage increased by {:.1}%", memory_change));
        }

        if response_time_change < -10.0 {
            analysis.push(format!(
                "Response time improved by {:.1}%",
                -response_time_change
            ));
        } else if response_time_change > 10.0 {
            analysis.push(format!(
                "Response time degraded by {:.1}%",
                response_time_change
            ));
        }

        if error_rate_change < -0.01 {
            analysis.push(format!(
                "Error rate reduced by {:.2}%",
                -error_rate_change * 100.0
            ));
        } else if error_rate_change > 0.01 {
            analysis.push(format!(
                "Error rate increased by {:.2}%",
                error_rate_change * 100.0
            ));
        }

        if cache_hit_change > 0.05 {
            analysis.push(format!(
                "Cache hit rate improved by {:.1}%",
                cache_hit_change * 100.0
            ));
        } else if cache_hit_change < -0.05 {
            analysis.push(format!(
                "Cache hit rate degraded by {:.1}%",
                -cache_hit_change * 100.0
            ));
        }

        if analysis.is_empty() {
            analysis.push("No significant metric changes observed".to_string());
        }

        let verdict = if was_effective {
            "Recommendation was EFFECTIVE"
        } else {
            "Recommendation showed LIMITED or NO improvement"
        };

        format!("{}. {}", verdict, analysis.join(". "))
    }

    fn calculate_confidence(&self, before: &MetricsSnapshot, after: &MetricsSnapshot) -> f64 {
        let time_diff = (after.timestamp - before.timestamp).num_seconds() as f64;

        let time_factor = if time_diff >= 60.0 && time_diff <= 600.0 {
            1.0
        } else if time_diff < 60.0 {
            time_diff / 60.0
        } else {
            600.0 / time_diff
        };

        let stability_factor = 0.8;

        (time_factor * 0.5 + stability_factor * 0.5)
            .min(1.0)
            .max(0.0)
    }

    pub async fn get_effective_recommendations(&self) -> Vec<TrackedRecommendation> {
        let recommendations = self.tracked_recommendations.read().await;

        recommendations
            .values()
            .filter(|r| {
                r.effectiveness
                    .as_ref()
                    .map(|e| e.was_effective)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub async fn get_ineffective_recommendations(&self) -> Vec<TrackedRecommendation> {
        let recommendations = self.tracked_recommendations.read().await;

        recommendations
            .values()
            .filter(|r| {
                r.effectiveness
                    .as_ref()
                    .map(|e| !e.was_effective)
                    .unwrap_or(false)
                    && r.was_applied
            })
            .cloned()
            .collect()
    }

    pub async fn generate_effectiveness_report(&self) -> EffectivenessReport {
        let recommendations = self.tracked_recommendations.read().await;

        let total = recommendations.len();
        let applied: Vec<_> = recommendations.values().filter(|r| r.was_applied).collect();
        let effective: Vec<_> = applied
            .iter()
            .filter(|r| {
                r.effectiveness
                    .as_ref()
                    .map(|e| e.was_effective)
                    .unwrap_or(false)
            })
            .collect();

        let effectiveness_rate = if !applied.is_empty() {
            effective.len() as f64 / applied.len() as f64
        } else {
            0.0
        };

        let mut category_breakdown: HashMap<String, CategoryEffectiveness> = HashMap::new();

        for rec in recommendations.values() {
            let category = &rec.recommendation.category;
            let entry =
                category_breakdown
                    .entry(category.clone())
                    .or_insert(CategoryEffectiveness {
                        category: category.clone(),
                        total_issued: 0,
                        total_applied: 0,
                        total_effective: 0,
                        avg_effectiveness_score: 0.0,
                        avg_cpu_improvement: 0.0,
                        avg_response_time_improvement: 0.0,
                    });

            entry.total_issued += 1;
            if rec.was_applied {
                entry.total_applied += 1;
                if let Some(eff) = &rec.effectiveness {
                    if eff.was_effective {
                        entry.total_effective += 1;
                    }
                    entry.avg_effectiveness_score += eff.overall_effectiveness_score;
                    entry.avg_cpu_improvement += -eff.cpu_change_percent;
                    entry.avg_response_time_improvement += -eff.response_time_change_percent;
                }
            }
        }

        for entry in category_breakdown.values_mut() {
            if entry.total_applied > 0 {
                entry.avg_effectiveness_score /= entry.total_applied as f64;
                entry.avg_cpu_improvement /= entry.total_applied as f64;
                entry.avg_response_time_improvement /= entry.total_applied as f64;
            }
        }

        let mut top_effective: Vec<_> = recommendations
            .values()
            .filter(|r| {
                r.effectiveness
                    .as_ref()
                    .map(|e| e.was_effective)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        top_effective.sort_by(|a, b| {
            let score_a = a
                .effectiveness
                .as_ref()
                .map(|e| e.overall_effectiveness_score)
                .unwrap_or(0.0);
            let score_b = b
                .effectiveness
                .as_ref()
                .map(|e| e.overall_effectiveness_score)
                .unwrap_or(0.0);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        top_effective.truncate(10);

        let ineffective: Vec<_> = recommendations
            .values()
            .filter(|r| {
                r.was_applied
                    && r.effectiveness
                        .as_ref()
                        .map(|e| !e.was_effective)
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        EffectivenessReport {
            total_recommendations: total,
            applied_recommendations: applied.len(),
            effective_recommendations: effective.len(),
            effectiveness_rate,
            category_breakdown,
            top_effective_recommendations: top_effective,
            ineffective_recommendations: ineffective,
            report_generated_at: Utc::now(),
        }
    }

    pub async fn create_ab_experiment(
        &self,
        name: String,
        description: String,
        recommendation_category: String,
        metric_to_track: String,
    ) -> Result<Uuid, AIError> {
        if !self.config.enable_ab_testing {
            return Err(AIError::ConfigurationError(
                "A/B testing is disabled".to_string(),
            ));
        }

        let experiment_id = Uuid::new_v4();

        let experiment = ABExperiment {
            experiment_id,
            name,
            description,
            recommendation_category,
            control_group: Vec::new(),
            treatment_group: Vec::new(),
            metric_to_track,
            start_time: Utc::now(),
            end_time: None,
            status: ExperimentStatus::Pending,
            results: None,
        };

        self.experiments
            .write()
            .await
            .insert(experiment_id, experiment);

        Ok(experiment_id)
    }

    pub async fn assign_to_experiment(
        &self,
        experiment_id: Uuid,
        user_id: Uuid,
        is_treatment: bool,
    ) -> Result<(), AIError> {
        let mut experiments = self.experiments.write().await;

        let experiment = experiments.get_mut(&experiment_id).ok_or_else(|| {
            AIError::ConfigurationError(format!("Experiment {} not found", experiment_id))
        })?;

        if experiment.status != ExperimentStatus::Pending
            && experiment.status != ExperimentStatus::Running
        {
            return Err(AIError::ConfigurationError(
                "Experiment is not active".to_string(),
            ));
        }

        if is_treatment {
            if !experiment.treatment_group.contains(&user_id) {
                experiment.treatment_group.push(user_id);
            }
        } else {
            if !experiment.control_group.contains(&user_id) {
                experiment.control_group.push(user_id);
            }
        }

        if experiment.status == ExperimentStatus::Pending {
            experiment.status = ExperimentStatus::Running;
        }

        Ok(())
    }

    pub async fn complete_experiment(
        &self,
        experiment_id: Uuid,
        control_metrics: Vec<f64>,
        treatment_metrics: Vec<f64>,
    ) -> Result<ABExperimentResults, AIError> {
        let mut experiments = self.experiments.write().await;

        let experiment = experiments.get_mut(&experiment_id).ok_or_else(|| {
            AIError::ConfigurationError(format!("Experiment {} not found", experiment_id))
        })?;

        let control_avg = if control_metrics.is_empty() {
            0.0
        } else {
            control_metrics.iter().sum::<f64>() / control_metrics.len() as f64
        };

        let treatment_avg = if treatment_metrics.is_empty() {
            0.0
        } else {
            treatment_metrics.iter().sum::<f64>() / treatment_metrics.len() as f64
        };

        let improvement = if control_avg > 0.0 {
            ((treatment_avg - control_avg) / control_avg) * 100.0
        } else {
            0.0
        };

        let statistical_significance =
            self.calculate_statistical_significance(&control_metrics, &treatment_metrics);

        let conclusion = if statistical_significance > 0.95 {
            if improvement > 0.0 {
                format!(
                    "Treatment shows {:.1}% improvement with high confidence",
                    improvement
                )
            } else {
                format!(
                    "Treatment shows {:.1}% degradation with high confidence",
                    -improvement
                )
            }
        } else if statistical_significance > 0.80 {
            format!(
                "Results suggest {:.1}% change but need more data",
                improvement
            )
        } else {
            "Insufficient data for conclusive results".to_string()
        };

        let results = ABExperimentResults {
            control_avg_metric: control_avg,
            treatment_avg_metric: treatment_avg,
            improvement_percent: improvement,
            statistical_significance,
            sample_size_control: control_metrics.len(),
            sample_size_treatment: treatment_metrics.len(),
            conclusion,
        };

        experiment.status = ExperimentStatus::Completed;
        experiment.end_time = Some(Utc::now());
        experiment.results = Some(results.clone());

        Ok(results)
    }

    fn calculate_statistical_significance(&self, control: &[f64], treatment: &[f64]) -> f64 {
        if control.len() < 5 || treatment.len() < 5 {
            return 0.0;
        }

        let control_mean: f64 = control.iter().sum::<f64>() / control.len() as f64;
        let treatment_mean: f64 = treatment.iter().sum::<f64>() / treatment.len() as f64;

        let control_variance: f64 = control
            .iter()
            .map(|x| (x - control_mean).powi(2))
            .sum::<f64>()
            / (control.len() - 1) as f64;

        let treatment_variance: f64 = treatment
            .iter()
            .map(|x| (x - treatment_mean).powi(2))
            .sum::<f64>()
            / (treatment.len() - 1) as f64;

        let pooled_se = ((control_variance / control.len() as f64)
            + (treatment_variance / treatment.len() as f64))
            .sqrt();

        if pooled_se == 0.0 {
            return 0.0;
        }

        let t_stat = (treatment_mean - control_mean).abs() / pooled_se;

        let confidence = 1.0 - (-0.5 * t_stat).exp();
        confidence.min(0.999)
    }

    async fn persist_recommendation(&self, _recommendation_id: Uuid) -> Result<(), AIError> {
        Ok(())
    }

    pub async fn start_background_tasks(self: Arc<Self>) {
        tokio::spawn({
            let tracker = self.clone();
            async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                    tracker.config.measurement_delay_seconds,
                ));
                loop {
                    interval.tick().await;
                    if let Err(e) = tracker.measure_pending_recommendations().await {
                        eprintln!("Error measuring recommendations: {}", e);
                    }
                }
            }
        });
    }

    async fn measure_pending_recommendations(&self) -> Result<(), AIError> {
        let recommendation_ids: Vec<Uuid> = {
            let recommendations = self.tracked_recommendations.read().await;
            recommendations
                .iter()
                .filter(|(_, r)| {
                    r.was_applied
                        && r.after_snapshot.is_none()
                        && r.applied_at
                            .map(|t| {
                                (Utc::now() - t).num_seconds()
                                    >= self.config.measurement_delay_seconds as i64
                            })
                            .unwrap_or(false)
                })
                .map(|(id, _)| *id)
                .collect()
        };

        for id in recommendation_ids {
            let _ = self.measure_recommendation_impact(id).await;
        }

        Ok(())
    }
}
