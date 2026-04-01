use crate::database::DatabaseManager;
use super::AIError;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInteraction {
    pub user_id: Uuid,
    pub item_id: Uuid,
    pub interaction_type: InteractionType,
    pub score: f32,
    pub timestamp: DateTime<Utc>,
    pub context: Option<InteractionContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    View,
    Like,
    Dislike,
    Purchase,
    Use,
    Create,
    Share,
    Rate(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionContext {
    pub session_id: Option<Uuid>,
    pub location: Option<String>,
    pub duration_seconds: Option<u64>,
    pub referrer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRecommendation {
    pub item_id: Uuid,
    pub predicted_score: f32,
    pub confidence: f32,
    pub recommendation_type: RecommendationType,
    pub explanation: String,
    pub similar_items: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    UserBased,
    ItemBased,
    Hybrid,
    Popular,
    ContentBased,
    ColdStart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityScore {
    pub entity_a: Uuid,
    pub entity_b: Uuid,
    pub similarity: f32,
    pub common_interactions: usize,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct InteractionMatrix {
    user_item_ratings: HashMap<(Uuid, Uuid), f32>,
    item_similarity_cache: HashMap<(Uuid, Uuid), f32>,
    user_similarity_cache: HashMap<(Uuid, Uuid), f32>,
    user_interactions: HashMap<Uuid, Vec<Uuid>>,
    item_interactions: HashMap<Uuid, Vec<Uuid>>,
    item_popularity: HashMap<Uuid, usize>,
    last_similarity_update: DateTime<Utc>,
}

impl InteractionMatrix {
    fn new() -> Self {
        Self {
            user_item_ratings: HashMap::new(),
            item_similarity_cache: HashMap::new(),
            user_similarity_cache: HashMap::new(),
            user_interactions: HashMap::new(),
            item_interactions: HashMap::new(),
            item_popularity: HashMap::new(),
            last_similarity_update: Utc::now(),
        }
    }

    fn record_interaction(&mut self, user_id: Uuid, item_id: Uuid, score: f32) {
        self.user_item_ratings.insert((user_id, item_id), score);

        self.user_interactions.entry(user_id)
            .or_insert_with(Vec::new)
            .push(item_id);

        self.item_interactions.entry(item_id)
            .or_insert_with(Vec::new)
            .push(user_id);

        *self.item_popularity.entry(item_id).or_insert(0) += 1;
    }

    fn get_user_items(&self, user_id: &Uuid) -> Vec<Uuid> {
        self.user_interactions.get(user_id).cloned().unwrap_or_default()
    }

    fn get_item_users(&self, item_id: &Uuid) -> Vec<Uuid> {
        self.item_interactions.get(item_id).cloned().unwrap_or_default()
    }

    fn get_rating(&self, user_id: &Uuid, item_id: &Uuid) -> Option<f32> {
        self.user_item_ratings.get(&(*user_id, *item_id)).copied()
    }

    fn get_user_ratings(&self, user_id: &Uuid) -> HashMap<Uuid, f32> {
        self.user_item_ratings.iter()
            .filter(|((uid, _), _)| uid == user_id)
            .map(|((_, iid), score)| (*iid, *score))
            .collect()
    }

    fn get_item_ratings(&self, item_id: &Uuid) -> HashMap<Uuid, f32> {
        self.user_item_ratings.iter()
            .filter(|((_, iid), _)| iid == item_id)
            .map(|((uid, _), score)| (*uid, *score))
            .collect()
    }

    fn compute_item_similarity(&self, item_a: &Uuid, item_b: &Uuid) -> f32 {
        let users_a = self.get_item_users(item_a);
        let users_b = self.get_item_users(item_b);

        if users_a.is_empty() || users_b.is_empty() {
            return 0.0;
        }

        let common_users: Vec<Uuid> = users_a.iter()
            .filter(|u| users_b.contains(u))
            .cloned()
            .collect();

        if common_users.is_empty() {
            return 0.0;
        }

        let ratings_a: Vec<f32> = common_users.iter()
            .filter_map(|u| self.get_rating(u, item_a))
            .collect();
        let ratings_b: Vec<f32> = common_users.iter()
            .filter_map(|u| self.get_rating(u, item_b))
            .collect();

        self.cosine_similarity(&ratings_a, &ratings_b)
    }

    fn compute_user_similarity(&self, user_a: &Uuid, user_b: &Uuid) -> f32 {
        let items_a = self.get_user_items(user_a);
        let items_b = self.get_user_items(user_b);

        if items_a.is_empty() || items_b.is_empty() {
            return 0.0;
        }

        let common_items: Vec<Uuid> = items_a.iter()
            .filter(|i| items_b.contains(i))
            .cloned()
            .collect();

        if common_items.is_empty() {
            return 0.0;
        }

        let ratings_a: Vec<f32> = common_items.iter()
            .filter_map(|i| self.get_rating(user_a, i))
            .collect();
        let ratings_b: Vec<f32> = common_items.iter()
            .filter_map(|i| self.get_rating(user_b, i))
            .collect();

        self.cosine_similarity(&ratings_a, &ratings_b)
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

    fn get_popular_items(&self, n: usize) -> Vec<Uuid> {
        let mut items: Vec<_> = self.item_popularity.iter().collect();
        items.sort_by(|a, b| b.1.cmp(a.1));
        items.into_iter().take(n).map(|(id, _)| *id).collect()
    }

    fn get_all_items(&self) -> Vec<Uuid> {
        self.item_interactions.keys().cloned().collect()
    }

    fn get_all_users(&self) -> Vec<Uuid> {
        self.user_interactions.keys().cloned().collect()
    }

    fn cache_item_similarity(&mut self, item_a: Uuid, item_b: Uuid, similarity: f32) {
        let key = if item_a < item_b { (item_a, item_b) } else { (item_b, item_a) };
        self.item_similarity_cache.insert(key, similarity);
    }

    fn cache_user_similarity(&mut self, user_a: Uuid, user_b: Uuid, similarity: f32) {
        let key = if user_a < user_b { (user_a, user_b) } else { (user_b, user_a) };
        self.user_similarity_cache.insert(key, similarity);
    }

    fn get_cached_item_similarity(&self, item_a: &Uuid, item_b: &Uuid) -> Option<f32> {
        let key = if item_a < item_b { (*item_a, *item_b) } else { (*item_b, *item_a) };
        self.item_similarity_cache.get(&key).copied()
    }

    fn get_cached_user_similarity(&self, user_a: &Uuid, user_b: &Uuid) -> Option<f32> {
        let key = if user_a < user_b { (*user_a, *user_b) } else { (*user_b, *user_a) };
        self.user_similarity_cache.get(&key).copied()
    }
}

#[derive(Debug, Clone)]
pub struct CollaborativeFilteringConfig {
    pub min_interactions_for_recommendation: usize,
    pub min_common_items_for_user_similarity: usize,
    pub min_common_users_for_item_similarity: usize,
    pub similarity_cache_ttl_hours: u32,
    pub max_similar_items_to_consider: usize,
    pub max_similar_users_to_consider: usize,
    pub cold_start_popular_items_count: usize,
    pub hybrid_user_weight: f32,
    pub hybrid_item_weight: f32,
}

impl Default for CollaborativeFilteringConfig {
    fn default() -> Self {
        Self {
            min_interactions_for_recommendation: 5,
            min_common_items_for_user_similarity: 3,
            min_common_users_for_item_similarity: 3,
            similarity_cache_ttl_hours: 24,
            max_similar_items_to_consider: 50,
            max_similar_users_to_consider: 50,
            cold_start_popular_items_count: 10,
            hybrid_user_weight: 0.5,
            hybrid_item_weight: 0.5,
        }
    }
}

#[derive(Debug)]
pub struct CollaborativeRecommender {
    matrix: Arc<RwLock<InteractionMatrix>>,
    config: CollaborativeFilteringConfig,
    db: Arc<DatabaseManager>,
}

impl CollaborativeRecommender {
    pub async fn new(db: Arc<DatabaseManager>) -> Result<Arc<Self>, AIError> {
        let recommender = Self {
            matrix: Arc::new(RwLock::new(InteractionMatrix::new())),
            config: CollaborativeFilteringConfig::default(),
            db,
        };

        Ok(Arc::new(recommender))
    }

    pub async fn record_interaction(&self, interaction: UserInteraction) -> Result<(), AIError> {
        let score = match interaction.interaction_type {
            InteractionType::View => 0.3,
            InteractionType::Like => 0.8,
            InteractionType::Dislike => 0.1,
            InteractionType::Purchase => 1.0,
            InteractionType::Use => 0.7,
            InteractionType::Create => 0.9,
            InteractionType::Share => 0.85,
            InteractionType::Rate(r) => r,
        };

        let combined_score = (score + interaction.score) / 2.0;

        self.matrix.write().await.record_interaction(
            interaction.user_id,
            interaction.item_id,
            combined_score,
        );

        Ok(())
    }

    pub async fn recommend_for_user(
        &self,
        user_id: Uuid,
        n: usize,
    ) -> Result<Vec<ContentRecommendation>, AIError> {
        let matrix = self.matrix.read().await;

        let user_items = matrix.get_user_items(&user_id);

        if user_items.len() < self.config.min_interactions_for_recommendation {
            return self.cold_start_recommendations(n).await;
        }

        let user_based = self.user_based_recommendations(&matrix, &user_id, &user_items, n).await?;
        let item_based = self.item_based_recommendations(&matrix, &user_id, &user_items, n).await?;

        let recommendations = self.merge_recommendations(
            user_based,
            item_based,
            self.config.hybrid_user_weight,
            self.config.hybrid_item_weight,
            n,
        );

        Ok(recommendations)
    }

    async fn user_based_recommendations(
        &self,
        matrix: &InteractionMatrix,
        user_id: &Uuid,
        user_items: &[Uuid],
        n: usize,
    ) -> Result<Vec<ContentRecommendation>, AIError> {
        let mut similar_users: Vec<(Uuid, f32)> = Vec::new();

        for other_user in matrix.get_all_users() {
            if &other_user == user_id {
                continue;
            }

            let similarity = matrix.get_cached_user_similarity(user_id, &other_user)
                .unwrap_or_else(|| matrix.compute_user_similarity(user_id, &other_user));

            if similarity > 0.1 {
                similar_users.push((other_user, similarity));
            }
        }

        similar_users.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similar_users.truncate(self.config.max_similar_users_to_consider);

        let mut item_scores: HashMap<Uuid, (f32, f32)> = HashMap::new();

        for (similar_user, similarity) in &similar_users {
            let their_items = matrix.get_user_items(similar_user);

            for item in their_items {
                if user_items.contains(&item) {
                    continue;
                }

                if let Some(rating) = matrix.get_rating(similar_user, &item) {
                    let entry = item_scores.entry(item).or_insert((0.0, 0.0));
                    entry.0 += similarity * rating;
                    entry.1 += similarity.abs();
                }
            }
        }

        let mut recommendations: Vec<ContentRecommendation> = item_scores.into_iter()
            .filter(|(_, (_, weight_sum))| *weight_sum > 0.0)
            .map(|(item_id, (weighted_sum, weight_sum))| {
                let predicted_score = weighted_sum / weight_sum;
                let confidence = (weight_sum / self.config.max_similar_users_to_consider as f32).min(1.0);

                ContentRecommendation {
                    item_id,
                    predicted_score,
                    confidence,
                    recommendation_type: RecommendationType::UserBased,
                    explanation: "Users with similar preferences liked this".to_string(),
                    similar_items: Vec::new(),
                }
            })
            .collect();

        recommendations.sort_by(|a, b| b.predicted_score.partial_cmp(&a.predicted_score).unwrap_or(std::cmp::Ordering::Equal));
        recommendations.truncate(n);

        Ok(recommendations)
    }

    async fn item_based_recommendations(
        &self,
        matrix: &InteractionMatrix,
        user_id: &Uuid,
        user_items: &[Uuid],
        n: usize,
    ) -> Result<Vec<ContentRecommendation>, AIError> {
        let mut candidate_scores: HashMap<Uuid, (f32, f32, Vec<Uuid>)> = HashMap::new();

        for user_item in user_items {
            let user_rating = matrix.get_rating(user_id, user_item).unwrap_or(0.5);

            for candidate_item in matrix.get_all_items() {
                if user_items.contains(&candidate_item) {
                    continue;
                }

                let similarity = matrix.get_cached_item_similarity(user_item, &candidate_item)
                    .unwrap_or_else(|| matrix.compute_item_similarity(user_item, &candidate_item));

                if similarity > 0.1 {
                    let entry = candidate_scores.entry(candidate_item)
                        .or_insert((0.0, 0.0, Vec::new()));
                    entry.0 += similarity * user_rating;
                    entry.1 += similarity.abs();
                    entry.2.push(*user_item);
                }
            }
        }

        let mut recommendations: Vec<ContentRecommendation> = candidate_scores.into_iter()
            .filter(|(_, (_, weight_sum, _))| *weight_sum > 0.0)
            .map(|(item_id, (weighted_sum, weight_sum, similar))| {
                let predicted_score = weighted_sum / weight_sum;
                let confidence = (weight_sum / self.config.max_similar_items_to_consider as f32).min(1.0);

                ContentRecommendation {
                    item_id,
                    predicted_score,
                    confidence,
                    recommendation_type: RecommendationType::ItemBased,
                    explanation: format!("Similar to items you've interacted with"),
                    similar_items: similar.into_iter().take(3).collect(),
                }
            })
            .collect();

        recommendations.sort_by(|a, b| b.predicted_score.partial_cmp(&a.predicted_score).unwrap_or(std::cmp::Ordering::Equal));
        recommendations.truncate(n);

        Ok(recommendations)
    }

    fn merge_recommendations(
        &self,
        user_based: Vec<ContentRecommendation>,
        item_based: Vec<ContentRecommendation>,
        user_weight: f32,
        item_weight: f32,
        n: usize,
    ) -> Vec<ContentRecommendation> {
        let mut merged: HashMap<Uuid, ContentRecommendation> = HashMap::new();

        for rec in user_based {
            let combined_score = rec.predicted_score * user_weight;
            merged.insert(rec.item_id, ContentRecommendation {
                predicted_score: combined_score,
                recommendation_type: RecommendationType::Hybrid,
                explanation: "Recommended based on similar users".to_string(),
                ..rec
            });
        }

        for rec in item_based {
            let combined_score = rec.predicted_score * item_weight;

            merged.entry(rec.item_id)
                .and_modify(|existing| {
                    existing.predicted_score += combined_score;
                    existing.confidence = (existing.confidence + rec.confidence) / 2.0;
                    existing.explanation = "Recommended based on similar users and items".to_string();
                    existing.similar_items.extend(rec.similar_items.clone());
                })
                .or_insert(ContentRecommendation {
                    predicted_score: combined_score,
                    recommendation_type: RecommendationType::Hybrid,
                    explanation: "Recommended based on similar items".to_string(),
                    ..rec
                });
        }

        let mut recommendations: Vec<ContentRecommendation> = merged.into_values().collect();
        recommendations.sort_by(|a, b| b.predicted_score.partial_cmp(&a.predicted_score).unwrap_or(std::cmp::Ordering::Equal));
        recommendations.truncate(n);

        recommendations
    }

    async fn cold_start_recommendations(&self, n: usize) -> Result<Vec<ContentRecommendation>, AIError> {
        let matrix = self.matrix.read().await;
        let popular_items = matrix.get_popular_items(self.config.cold_start_popular_items_count.max(n));

        let recommendations = popular_items.into_iter()
            .take(n)
            .enumerate()
            .map(|(i, item_id)| ContentRecommendation {
                item_id,
                predicted_score: 1.0 - (i as f32 * 0.05),
                confidence: 0.5,
                recommendation_type: RecommendationType::ColdStart,
                explanation: "Popular item recommended for new users".to_string(),
                similar_items: Vec::new(),
            })
            .collect();

        Ok(recommendations)
    }

    pub async fn find_similar_items(
        &self,
        item_id: Uuid,
        n: usize,
    ) -> Result<Vec<SimilarityScore>, AIError> {
        let matrix = self.matrix.read().await;

        let mut similarities: Vec<SimilarityScore> = Vec::new();

        for other_item in matrix.get_all_items() {
            if other_item == item_id {
                continue;
            }

            let similarity = matrix.compute_item_similarity(&item_id, &other_item);

            if similarity > 0.0 {
                let item_users = matrix.get_item_users(&item_id);
                let other_users = matrix.get_item_users(&other_item);
                let common = item_users.iter().filter(|u| other_users.contains(u)).count();

                similarities.push(SimilarityScore {
                    entity_a: item_id,
                    entity_b: other_item,
                    similarity,
                    common_interactions: common,
                    computed_at: Utc::now(),
                });
            }
        }

        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(n);

        Ok(similarities)
    }

    pub async fn find_similar_users(
        &self,
        user_id: Uuid,
        n: usize,
    ) -> Result<Vec<SimilarityScore>, AIError> {
        let matrix = self.matrix.read().await;

        let mut similarities: Vec<SimilarityScore> = Vec::new();

        for other_user in matrix.get_all_users() {
            if other_user == user_id {
                continue;
            }

            let similarity = matrix.compute_user_similarity(&user_id, &other_user);

            if similarity > 0.0 {
                let user_items = matrix.get_user_items(&user_id);
                let other_items = matrix.get_user_items(&other_user);
                let common = user_items.iter().filter(|i| other_items.contains(i)).count();

                similarities.push(SimilarityScore {
                    entity_a: user_id,
                    entity_b: other_user,
                    similarity,
                    common_interactions: common,
                    computed_at: Utc::now(),
                });
            }
        }

        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(n);

        Ok(similarities)
    }

    pub async fn update_similarity_cache(&self) -> Result<(), AIError> {
        let mut matrix = self.matrix.write().await;

        let items: Vec<Uuid> = matrix.get_all_items();
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                let similarity = matrix.compute_item_similarity(&items[i], &items[j]);
                if similarity > 0.1 {
                    matrix.cache_item_similarity(items[i], items[j], similarity);
                }
            }
        }

        let users: Vec<Uuid> = matrix.get_all_users();
        for i in 0..users.len() {
            for j in (i + 1)..users.len() {
                let similarity = matrix.compute_user_similarity(&users[i], &users[j]);
                if similarity > 0.1 {
                    matrix.cache_user_similarity(users[i], users[j], similarity);
                }
            }
        }

        matrix.last_similarity_update = Utc::now();

        Ok(())
    }

    pub async fn get_user_stats(&self, user_id: Uuid) -> UserStats {
        let matrix = self.matrix.read().await;
        let items = matrix.get_user_items(&user_id);
        let ratings = matrix.get_user_ratings(&user_id);

        let avg_rating = if ratings.is_empty() {
            0.0
        } else {
            ratings.values().sum::<f32>() / ratings.len() as f32
        };

        UserStats {
            user_id,
            total_interactions: items.len(),
            unique_items: ratings.len(),
            average_rating: avg_rating,
            can_receive_personalized: items.len() >= self.config.min_interactions_for_recommendation,
        }
    }

    pub async fn get_item_stats(&self, item_id: Uuid) -> ItemStats {
        let matrix = self.matrix.read().await;
        let users = matrix.get_item_users(&item_id);
        let ratings = matrix.get_item_ratings(&item_id);

        let avg_rating = if ratings.is_empty() {
            0.0
        } else {
            ratings.values().sum::<f32>() / ratings.len() as f32
        };

        let popularity_rank = {
            let mut items: Vec<_> = matrix.item_popularity.iter().collect();
            items.sort_by(|a, b| b.1.cmp(a.1));
            items.iter().position(|(id, _)| *id == &item_id).unwrap_or(items.len()) + 1
        };

        ItemStats {
            item_id,
            total_interactions: users.len(),
            unique_users: ratings.len(),
            average_rating: avg_rating,
            popularity_rank,
        }
    }

    pub async fn start_background_tasks(self: Arc<Self>) {
        tokio::spawn({
            let recommender = self.clone();
            async move {
                let mut interval = tokio::time::interval(
                    tokio::time::Duration::from_secs(3600)
                );
                loop {
                    interval.tick().await;
                    if let Err(e) = recommender.update_similarity_cache().await {
                        eprintln!("Error updating similarity cache: {}", e);
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: Uuid,
    pub total_interactions: usize,
    pub unique_items: usize,
    pub average_rating: f32,
    pub can_receive_personalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStats {
    pub item_id: Uuid,
    pub total_interactions: usize,
    pub unique_users: usize,
    pub average_rating: f32,
    pub popularity_rank: usize,
}
