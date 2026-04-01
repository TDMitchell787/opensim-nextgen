use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, debug};

use super::super::AIError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommenderConfig {
    pub max_recommendations: usize,
    pub min_similarity_threshold: f32,
    pub activity_decay_days: u64,
    pub min_interactions_for_recommendation: usize,
    pub enable_content_recommendations: bool,
    pub enable_social_recommendations: bool,
    pub enable_creator_recommendations: bool,
}

impl Default for RecommenderConfig {
    fn default() -> Self {
        Self {
            max_recommendations: 20,
            min_similarity_threshold: 0.1,
            activity_decay_days: 90,
            min_interactions_for_recommendation: 3,
            enable_content_recommendations: true,
            enable_social_recommendations: true,
            enable_creator_recommendations: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRecommendation {
    pub content_id: Uuid,
    pub content_type: ContentItemType,
    pub name: String,
    pub score: f32,
    pub reason: RecommendationReason,
    pub source_items: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentItemType {
    Region,
    Asset,
    Event,
    Group,
    Creator,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationReason {
    SimilarVisitors,
    SimilarInterests,
    PopularNearby,
    FriendsVisited,
    TrendingContent,
    CreatorFollowed,
    ActivityPattern,
}

impl RecommendationReason {
    pub fn description(&self) -> &'static str {
        match self {
            RecommendationReason::SimilarVisitors => "Users who visited similar places also visited this",
            RecommendationReason::SimilarInterests => "Based on your interests",
            RecommendationReason::PopularNearby => "Popular nearby",
            RecommendationReason::FriendsVisited => "Your friends visited this",
            RecommendationReason::TrendingContent => "Trending now",
            RecommendationReason::CreatorFollowed => "From creators you follow",
            RecommendationReason::ActivityPattern => "Based on your activity patterns",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialRecommendation {
    pub user_id: Uuid,
    pub display_name: String,
    pub score: f32,
    pub common_interests: Vec<String>,
    pub common_friends: usize,
    pub mutual_regions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: Uuid,
    pub activity_type: ActivityType,
    pub target_id: Uuid,
    pub target_name: String,
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: Option<u64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActivityType {
    RegionVisit,
    AssetPurchase,
    AssetView,
    GroupJoin,
    EventAttend,
    FriendAdd,
    CreatorFollow,
    ContentCreate,
    ChatInteraction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub interests: Vec<String>,
    pub visited_regions: Vec<Uuid>,
    pub owned_assets: Vec<Uuid>,
    pub groups: Vec<Uuid>,
    pub friends: Vec<Uuid>,
    pub followed_creators: Vec<Uuid>,
    pub activity_score: f32,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementMetrics {
    pub user_id: Uuid,
    pub sessions_last_30_days: u32,
    pub avg_session_duration_minutes: f32,
    pub regions_visited_30_days: u32,
    pub social_interactions_30_days: u32,
    pub content_created_30_days: u32,
    pub churn_risk_score: f32,
    pub engagement_trend: EngagementTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EngagementTrend {
    Increasing,
    Stable,
    Declining,
    Critical,
}

pub struct CollaborativeRecommender {
    config: RecommenderConfig,
    user_activities: Arc<RwLock<HashMap<Uuid, Vec<UserActivity>>>>,
    user_profiles: Arc<RwLock<HashMap<Uuid, UserProfile>>>,
    item_popularity: Arc<RwLock<HashMap<Uuid, u32>>>,
    similarity_cache: Arc<RwLock<HashMap<(Uuid, Uuid), f32>>>,
}

impl CollaborativeRecommender {
    pub fn new(config: RecommenderConfig) -> Self {
        Self {
            config,
            user_activities: Arc::new(RwLock::new(HashMap::new())),
            user_profiles: Arc::new(RwLock::new(HashMap::new())),
            item_popularity: Arc::new(RwLock::new(HashMap::new())),
            similarity_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_activity(&self, activity: UserActivity) {
        let user_id = activity.user_id;
        let target_id = activity.target_id;

        let mut activities = self.user_activities.write().await;
        activities.entry(user_id).or_default().push(activity);

        let mut popularity = self.item_popularity.write().await;
        *popularity.entry(target_id).or_insert(0) += 1;

        drop(activities);
        drop(popularity);

        self.invalidate_cache(user_id).await;
    }

    pub async fn update_user_profile(&self, profile: UserProfile) {
        let user_id = profile.user_id;
        let mut profiles = self.user_profiles.write().await;
        profiles.insert(user_id, profile);
        drop(profiles);
        self.invalidate_cache(user_id).await;
    }

    pub async fn get_content_recommendations(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<ContentRecommendation>, AIError> {
        if !self.config.enable_content_recommendations {
            return Ok(Vec::new());
        }

        let activities = self.user_activities.read().await;
        let user_activities = activities.get(&user_id);

        if user_activities.is_none() || user_activities.unwrap().len() < self.config.min_interactions_for_recommendation {
            return Ok(self.get_popular_items(limit).await);
        }

        let user_acts = user_activities.unwrap();
        let user_items: Vec<Uuid> = user_acts.iter()
            .map(|a| a.target_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let mut candidates: HashMap<Uuid, (f32, Vec<Uuid>)> = HashMap::new();

        for (other_id, other_acts) in activities.iter() {
            if *other_id == user_id {
                continue;
            }

            let similarity = self.compute_user_similarity(user_acts, other_acts);
            if similarity < self.config.min_similarity_threshold {
                continue;
            }

            for act in other_acts {
                if !user_items.contains(&act.target_id) {
                    let entry = candidates.entry(act.target_id).or_insert((0.0, Vec::new()));
                    entry.0 += similarity;
                    if !entry.1.contains(&act.target_id) {
                        entry.1.push(act.target_id);
                    }
                }
            }
        }

        let mut recommendations: Vec<ContentRecommendation> = candidates.into_iter()
            .map(|(content_id, (score, sources))| {
                ContentRecommendation {
                    content_id,
                    content_type: ContentItemType::Region,
                    name: format!("Item_{}", &content_id.to_string()[..8]),
                    score,
                    reason: RecommendationReason::SimilarVisitors,
                    source_items: sources,
                }
            })
            .collect();

        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        recommendations.truncate(limit.min(self.config.max_recommendations));

        debug!("Generated {} content recommendations for user {}", recommendations.len(), user_id);
        Ok(recommendations)
    }

    pub async fn get_social_recommendations(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<SocialRecommendation>, AIError> {
        if !self.config.enable_social_recommendations {
            return Ok(Vec::new());
        }

        let profiles = self.user_profiles.read().await;
        let user_profile = match profiles.get(&user_id) {
            Some(p) => p.clone(),
            None => return Ok(Vec::new()),
        };

        let mut candidates: Vec<SocialRecommendation> = Vec::new();

        for (other_id, other_profile) in profiles.iter() {
            if *other_id == user_id || user_profile.friends.contains(other_id) {
                continue;
            }

            let common_interests: Vec<String> = user_profile.interests.iter()
                .filter(|i| other_profile.interests.contains(i))
                .cloned()
                .collect();

            let common_friends = user_profile.friends.iter()
                .filter(|f| other_profile.friends.contains(f))
                .count();

            let mutual_regions: Vec<String> = user_profile.visited_regions.iter()
                .filter(|r| other_profile.visited_regions.contains(r))
                .map(|r| format!("Region_{}", &r.to_string()[..8]))
                .collect();

            let interest_score = common_interests.len() as f32 * 0.4;
            let friend_score = common_friends as f32 * 0.3;
            let region_score = mutual_regions.len() as f32 * 0.3;
            let total_score = (interest_score + friend_score + region_score).min(1.0);

            if total_score >= self.config.min_similarity_threshold {
                candidates.push(SocialRecommendation {
                    user_id: *other_id,
                    display_name: format!("User_{}", &other_id.to_string()[..8]),
                    score: total_score,
                    common_interests,
                    common_friends,
                    mutual_regions,
                });
            }
        }

        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(limit.min(self.config.max_recommendations));

        debug!("Generated {} social recommendations for user {}", candidates.len(), user_id);
        Ok(candidates)
    }

    pub async fn get_creator_recommendations(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<ContentRecommendation>, AIError> {
        if !self.config.enable_creator_recommendations {
            return Ok(Vec::new());
        }

        let profiles = self.user_profiles.read().await;
        let user_profile = match profiles.get(&user_id) {
            Some(p) => p.clone(),
            None => return Ok(Vec::new()),
        };

        let activities = self.user_activities.read().await;
        let mut creator_scores: HashMap<Uuid, f32> = HashMap::new();

        for friend_id in &user_profile.friends {
            if let Some(friend_profile) = profiles.get(friend_id) {
                for creator_id in &friend_profile.followed_creators {
                    if !user_profile.followed_creators.contains(creator_id) {
                        *creator_scores.entry(*creator_id).or_insert(0.0) += 0.3;
                    }
                }
            }
        }

        for (other_id, other_acts) in activities.iter() {
            if *other_id == user_id {
                continue;
            }
            if let Some(user_acts) = activities.get(&user_id) {
                let sim = self.compute_user_similarity(user_acts, other_acts);
                if sim > self.config.min_similarity_threshold {
                    for act in other_acts.iter().filter(|a| a.activity_type == ActivityType::CreatorFollow) {
                        if !user_profile.followed_creators.contains(&act.target_id) {
                            *creator_scores.entry(act.target_id).or_insert(0.0) += sim;
                        }
                    }
                }
            }
        }

        let mut recommendations: Vec<ContentRecommendation> = creator_scores.into_iter()
            .map(|(creator_id, score)| {
                ContentRecommendation {
                    content_id: creator_id,
                    content_type: ContentItemType::Creator,
                    name: format!("Creator_{}", &creator_id.to_string()[..8]),
                    score,
                    reason: RecommendationReason::CreatorFollowed,
                    source_items: Vec::new(),
                }
            })
            .collect();

        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        recommendations.truncate(limit.min(self.config.max_recommendations));

        Ok(recommendations)
    }

    pub async fn compute_engagement_metrics(&self, user_id: Uuid) -> EngagementMetrics {
        let activities = self.user_activities.read().await;
        let user_activities = activities.get(&user_id);

        let now = Utc::now();
        let thirty_days_ago = now - chrono::Duration::days(30);

        if let Some(acts) = user_activities {
            let recent: Vec<&UserActivity> = acts.iter()
                .filter(|a| a.timestamp >= thirty_days_ago)
                .collect();

            let sessions = recent.iter()
                .filter(|a| a.activity_type == ActivityType::RegionVisit)
                .count() as u32;

            let avg_duration = recent.iter()
                .filter_map(|a| a.duration_seconds)
                .map(|d| d as f32 / 60.0)
                .sum::<f32>() / sessions.max(1) as f32;

            let regions_visited: std::collections::HashSet<Uuid> = recent.iter()
                .filter(|a| a.activity_type == ActivityType::RegionVisit)
                .map(|a| a.target_id)
                .collect();

            let social_interactions = recent.iter()
                .filter(|a| matches!(a.activity_type,
                    ActivityType::ChatInteraction | ActivityType::FriendAdd | ActivityType::GroupJoin
                ))
                .count() as u32;

            let content_created = recent.iter()
                .filter(|a| a.activity_type == ActivityType::ContentCreate)
                .count() as u32;

            let sixty_days_ago = now - chrono::Duration::days(60);
            let previous_period: Vec<&UserActivity> = acts.iter()
                .filter(|a| a.timestamp >= sixty_days_ago && a.timestamp < thirty_days_ago)
                .collect();

            let previous_sessions = previous_period.iter()
                .filter(|a| a.activity_type == ActivityType::RegionVisit)
                .count() as f32;

            let current_sessions = sessions as f32;
            let trend_ratio = if previous_sessions > 0.0 {
                current_sessions / previous_sessions
            } else if current_sessions > 0.0 {
                2.0
            } else {
                0.0
            };

            let engagement_trend = match trend_ratio {
                r if r >= 1.2 => EngagementTrend::Increasing,
                r if r >= 0.8 => EngagementTrend::Stable,
                r if r >= 0.4 => EngagementTrend::Declining,
                _ => EngagementTrend::Critical,
            };

            let churn_risk = match engagement_trend {
                EngagementTrend::Increasing => 0.1,
                EngagementTrend::Stable => 0.3,
                EngagementTrend::Declining => 0.6,
                EngagementTrend::Critical => 0.9,
            };

            EngagementMetrics {
                user_id,
                sessions_last_30_days: sessions,
                avg_session_duration_minutes: avg_duration,
                regions_visited_30_days: regions_visited.len() as u32,
                social_interactions_30_days: social_interactions,
                content_created_30_days: content_created,
                churn_risk_score: churn_risk,
                engagement_trend,
            }
        } else {
            EngagementMetrics {
                user_id,
                sessions_last_30_days: 0,
                avg_session_duration_minutes: 0.0,
                regions_visited_30_days: 0,
                social_interactions_30_days: 0,
                content_created_30_days: 0,
                churn_risk_score: 1.0,
                engagement_trend: EngagementTrend::Critical,
            }
        }
    }

    pub async fn get_trending_content(&self, limit: usize) -> Vec<ContentRecommendation> {
        let popularity = self.item_popularity.read().await;
        let mut items: Vec<(Uuid, u32)> = popularity.iter()
            .map(|(id, count)| (*id, *count))
            .collect();

        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(limit);

        items.into_iter()
            .map(|(id, count)| {
                let max_count = popularity.values().max().copied().unwrap_or(1) as f32;
                ContentRecommendation {
                    content_id: id,
                    content_type: ContentItemType::Region,
                    name: format!("Trending_{}", &id.to_string()[..8]),
                    score: count as f32 / max_count,
                    reason: RecommendationReason::TrendingContent,
                    source_items: Vec::new(),
                }
            })
            .collect()
    }

    pub async fn get_stats(&self) -> RecommenderStats {
        let activities = self.user_activities.read().await;
        let profiles = self.user_profiles.read().await;
        let popularity = self.item_popularity.read().await;

        RecommenderStats {
            total_users: activities.len(),
            total_activities: activities.values().map(|v| v.len()).sum(),
            total_profiles: profiles.len(),
            tracked_items: popularity.len(),
            cache_size: self.similarity_cache.read().await.len(),
        }
    }

    fn compute_user_similarity(&self, user_a: &[UserActivity], user_b: &[UserActivity]) -> f32 {
        let items_a: std::collections::HashSet<Uuid> = user_a.iter().map(|a| a.target_id).collect();
        let items_b: std::collections::HashSet<Uuid> = user_b.iter().map(|a| a.target_id).collect();

        let intersection = items_a.intersection(&items_b).count() as f32;
        let union = items_a.union(&items_b).count() as f32;

        if union == 0.0 {
            0.0
        } else {
            intersection / union
        }
    }

    async fn get_popular_items(&self, limit: usize) -> Vec<ContentRecommendation> {
        self.get_trending_content(limit).await
    }

    async fn invalidate_cache(&self, user_id: Uuid) {
        let mut cache = self.similarity_cache.write().await;
        cache.retain(|(a, b), _| *a != user_id && *b != user_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommenderStats {
    pub total_users: usize,
    pub total_activities: usize,
    pub total_profiles: usize,
    pub tracked_items: usize,
    pub cache_size: usize,
}
