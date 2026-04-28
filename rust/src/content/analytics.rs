//! Content Analytics for OpenSim Next
//!
//! Provides content usage analytics, performance tracking, and insights.

use super::{ContentAnalytics, ContentPerformanceMetrics, ContentResult, ContentRevenueData};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_id: Uuid,
    pub content_id: Uuid,
    pub user_id: Option<Uuid>,
    pub region_id: Option<Uuid>,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    Download,
    View,
    Use,
    Share,
    Rate,
    Purchase,
    LoadStart,
    LoadComplete,
    RenderStart,
    RenderComplete,
    CacheHit,
    CacheMiss,
    Error,
}

#[derive(Debug, Clone)]
struct ContentStats {
    downloads: u64,
    views: u64,
    uses: u64,
    shares: u64,
    purchases: u64,
    unique_users: std::collections::HashSet<Uuid>,
    ratings: Vec<f32>,
    usage_by_region: HashMap<Uuid, u64>,
    load_times: Vec<f32>,
    render_times: Vec<f32>,
    cache_hits: u64,
    cache_misses: u64,
    errors: u64,
    revenue: f64,
    first_event: Option<DateTime<Utc>>,
    last_event: Option<DateTime<Utc>>,
}

impl Default for ContentStats {
    fn default() -> Self {
        Self {
            downloads: 0,
            views: 0,
            uses: 0,
            shares: 0,
            purchases: 0,
            unique_users: std::collections::HashSet::new(),
            ratings: Vec::new(),
            usage_by_region: HashMap::new(),
            load_times: Vec::new(),
            render_times: Vec::new(),
            cache_hits: 0,
            cache_misses: 0,
            errors: 0,
            revenue: 0.0,
            first_event: None,
            last_event: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub content_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub region_id: Option<Uuid>,
    pub event_types: Option<Vec<EventType>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_events: u64,
    pub unique_contents: u32,
    pub unique_users: u32,
    pub events_by_type: HashMap<String, u64>,
    pub top_contents: Vec<(Uuid, u64)>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

pub struct ContentAnalyticsManager {
    events: Arc<RwLock<Vec<AnalyticsEvent>>>,
    content_stats: Arc<RwLock<HashMap<Uuid, ContentStats>>>,
    max_events: usize,
    load_start_times: Arc<RwLock<HashMap<(Uuid, Option<Uuid>), DateTime<Utc>>>>,
    render_start_times: Arc<RwLock<HashMap<(Uuid, Option<Uuid>), DateTime<Utc>>>>,
}

impl ContentAnalyticsManager {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            content_stats: Arc::new(RwLock::new(HashMap::new())),
            max_events: 100000,
            load_start_times: Arc::new(RwLock::new(HashMap::new())),
            render_start_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_max_events(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            content_stats: Arc::new(RwLock::new(HashMap::new())),
            max_events,
            load_start_times: Arc::new(RwLock::new(HashMap::new())),
            render_start_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_event(&self, event: AnalyticsEvent) -> ContentResult<()> {
        let content_id = event.content_id;
        let event_type = event.event_type.clone();
        let user_id = event.user_id;
        let region_id = event.region_id;
        let timestamp = event.timestamp;

        {
            let mut events = self.events.write().await;
            events.push(event.clone());

            if events.len() > self.max_events {
                let drain_count = events.len() - self.max_events;
                events.drain(0..drain_count);
            }
        }

        {
            let mut stats = self.content_stats.write().await;
            let content_stat = stats
                .entry(content_id)
                .or_insert_with(ContentStats::default);

            if content_stat.first_event.is_none() {
                content_stat.first_event = Some(timestamp);
            }
            content_stat.last_event = Some(timestamp);

            if let Some(uid) = user_id {
                content_stat.unique_users.insert(uid);
            }

            if let Some(rid) = region_id {
                *content_stat.usage_by_region.entry(rid).or_insert(0) += 1;
            }

            match event_type {
                EventType::Download => content_stat.downloads += 1,
                EventType::View => content_stat.views += 1,
                EventType::Use => content_stat.uses += 1,
                EventType::Share => content_stat.shares += 1,
                EventType::Purchase => {
                    content_stat.purchases += 1;
                    if let Some(amount) = event.metadata.get("amount") {
                        if let Some(val) = amount.as_f64() {
                            content_stat.revenue += val;
                        }
                    }
                }
                EventType::Rate => {
                    if let Some(rating) = event.metadata.get("rating") {
                        if let Some(val) = rating.as_f64() {
                            content_stat.ratings.push(val as f32);
                        }
                    }
                }
                EventType::LoadStart => {
                    let mut load_starts = self.load_start_times.write().await;
                    load_starts.insert((content_id, user_id), timestamp);
                }
                EventType::LoadComplete => {
                    let mut load_starts = self.load_start_times.write().await;
                    if let Some(start_time) = load_starts.remove(&(content_id, user_id)) {
                        let duration = (timestamp - start_time).num_milliseconds() as f32 / 1000.0;
                        content_stat.load_times.push(duration);
                    }
                }
                EventType::RenderStart => {
                    let mut render_starts = self.render_start_times.write().await;
                    render_starts.insert((content_id, user_id), timestamp);
                }
                EventType::RenderComplete => {
                    let mut render_starts = self.render_start_times.write().await;
                    if let Some(start_time) = render_starts.remove(&(content_id, user_id)) {
                        let duration = (timestamp - start_time).num_milliseconds() as f32 / 1000.0;
                        content_stat.render_times.push(duration);
                    }
                }
                EventType::CacheHit => content_stat.cache_hits += 1,
                EventType::CacheMiss => content_stat.cache_misses += 1,
                EventType::Error => content_stat.errors += 1,
            }
        }

        debug!(
            "Recorded analytics event for content {}: {:?}",
            content_id, event.event_type
        );
        Ok(())
    }

    pub async fn get_content_analytics(&self, content_id: Uuid) -> ContentResult<ContentAnalytics> {
        let stats = self.content_stats.read().await;

        let content_stat = stats.get(&content_id);

        let (
            total_downloads,
            unique_users,
            average_rating,
            usage_by_region,
            performance_metrics,
            revenue_data,
        ) = if let Some(stat) = content_stat {
            let avg_rating = if stat.ratings.is_empty() {
                0.0
            } else {
                stat.ratings.iter().sum::<f32>() / stat.ratings.len() as f32
            };

            let avg_load_time = if stat.load_times.is_empty() {
                0.0
            } else {
                stat.load_times.iter().sum::<f32>() / stat.load_times.len() as f32
            };

            let avg_render_time = if stat.render_times.is_empty() {
                0.0
            } else {
                stat.render_times.iter().sum::<f32>() / stat.render_times.len() as f32
            };

            let render_performance = if avg_render_time > 0.0 {
                (1.0 / avg_render_time * 60.0).min(100.0)
            } else {
                100.0
            };

            let total_cache_ops = stat.cache_hits + stat.cache_misses;
            let cache_hit_ratio = if total_cache_ops > 0 {
                stat.cache_hits as f32 / total_cache_ops as f32
            } else {
                1.0
            };

            let total_ops = stat.downloads + stat.views + stat.uses;
            let error_rate = if total_ops > 0 {
                stat.errors as f32 / total_ops as f32
            } else {
                0.0
            };

            let perf_metrics = ContentPerformanceMetrics {
                load_time_avg: avg_load_time,
                render_performance,
                memory_usage: (stat.downloads * 1024) as u64,
                cache_hit_ratio,
                error_rate,
            };

            let rev_data = if stat.revenue > 0.0 {
                Some(ContentRevenueData {
                    total_revenue: stat.revenue,
                    currency: "L$".to_string(),
                    sales_count: stat.purchases as u32,
                    refund_count: 0,
                    revenue_by_period: HashMap::new(),
                })
            } else {
                None
            };

            (
                stat.downloads,
                stat.unique_users.len() as u32,
                avg_rating,
                stat.usage_by_region.clone(),
                perf_metrics,
                rev_data,
            )
        } else {
            (
                0,
                0,
                0.0,
                HashMap::new(),
                ContentPerformanceMetrics {
                    load_time_avg: 0.0,
                    render_performance: 100.0,
                    memory_usage: 0,
                    cache_hit_ratio: 1.0,
                    error_rate: 0.0,
                },
                None,
            )
        };

        Ok(ContentAnalytics {
            content_id,
            total_downloads,
            unique_users,
            average_rating,
            usage_by_region,
            performance_metrics,
            revenue_data,
        })
    }

    pub async fn query_events(&self, query: AnalyticsQuery) -> ContentResult<Vec<AnalyticsEvent>> {
        let events = self.events.read().await;

        let filtered: Vec<AnalyticsEvent> = events
            .iter()
            .filter(|e| {
                if let Some(cid) = query.content_id {
                    if e.content_id != cid {
                        return false;
                    }
                }
                if let Some(uid) = query.user_id {
                    if e.user_id != Some(uid) {
                        return false;
                    }
                }
                if let Some(rid) = query.region_id {
                    if e.region_id != Some(rid) {
                        return false;
                    }
                }
                if let Some(ref types) = query.event_types {
                    if !types.contains(&e.event_type) {
                        return false;
                    }
                }
                if let Some(start) = query.start_time {
                    if e.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = query.end_time {
                    if e.timestamp > end {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let result = if let Some(limit) = query.limit {
            filtered.into_iter().rev().take(limit).collect()
        } else {
            filtered
        };

        debug!("Query returned {} events", result.len());
        Ok(result)
    }

    pub async fn get_summary(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> ContentResult<AnalyticsSummary> {
        let events = self.events.read().await;

        let filtered: Vec<&AnalyticsEvent> = events
            .iter()
            .filter(|e| e.timestamp >= start_time && e.timestamp <= end_time)
            .collect();

        let total_events = filtered.len() as u64;

        let mut unique_contents: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        let mut unique_users: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        let mut events_by_type: HashMap<String, u64> = HashMap::new();
        let mut content_counts: HashMap<Uuid, u64> = HashMap::new();

        for event in &filtered {
            unique_contents.insert(event.content_id);
            if let Some(uid) = event.user_id {
                unique_users.insert(uid);
            }

            let type_name = format!("{:?}", event.event_type);
            *events_by_type.entry(type_name).or_insert(0) += 1;
            *content_counts.entry(event.content_id).or_insert(0) += 1;
        }

        let mut top_contents: Vec<(Uuid, u64)> = content_counts.into_iter().collect();
        top_contents.sort_by(|a, b| b.1.cmp(&a.1));
        top_contents.truncate(10);

        Ok(AnalyticsSummary {
            total_events,
            unique_contents: unique_contents.len() as u32,
            unique_users: unique_users.len() as u32,
            events_by_type,
            top_contents,
            period_start: start_time,
            period_end: end_time,
        })
    }

    pub async fn get_trending_content(
        &self,
        duration: Duration,
        limit: usize,
    ) -> ContentResult<Vec<(Uuid, u64)>> {
        let end_time = Utc::now();
        let start_time = end_time - duration;

        let events = self.events.read().await;

        let mut content_activity: HashMap<Uuid, u64> = HashMap::new();

        for event in events.iter() {
            if event.timestamp >= start_time && event.timestamp <= end_time {
                let weight = match event.event_type {
                    EventType::Purchase => 10,
                    EventType::Download => 5,
                    EventType::Share => 4,
                    EventType::Rate => 3,
                    EventType::Use => 2,
                    EventType::View => 1,
                    _ => 0,
                };
                *content_activity.entry(event.content_id).or_insert(0) += weight;
            }
        }

        let mut trending: Vec<(Uuid, u64)> = content_activity.into_iter().collect();
        trending.sort_by(|a, b| b.1.cmp(&a.1));
        trending.truncate(limit);

        info!("Found {} trending content items", trending.len());
        Ok(trending)
    }

    pub async fn get_user_activity(&self, user_id: Uuid) -> ContentResult<HashMap<Uuid, u64>> {
        let events = self.events.read().await;

        let mut activity: HashMap<Uuid, u64> = HashMap::new();

        for event in events.iter() {
            if event.user_id == Some(user_id) {
                *activity.entry(event.content_id).or_insert(0) += 1;
            }
        }

        debug!(
            "User {} has activity on {} content items",
            user_id,
            activity.len()
        );
        Ok(activity)
    }

    pub async fn get_region_analytics(&self, region_id: Uuid) -> ContentResult<HashMap<Uuid, u64>> {
        let events = self.events.read().await;

        let mut region_content: HashMap<Uuid, u64> = HashMap::new();

        for event in events.iter() {
            if event.region_id == Some(region_id) {
                *region_content.entry(event.content_id).or_insert(0) += 1;
            }
        }

        debug!(
            "Region {} has {} content items tracked",
            region_id,
            region_content.len()
        );
        Ok(region_content)
    }

    pub async fn clear_old_events(&self, older_than: DateTime<Utc>) -> ContentResult<u32> {
        let mut events = self.events.write().await;
        let initial_count = events.len();

        events.retain(|e| e.timestamp >= older_than);

        let removed = (initial_count - events.len()) as u32;
        info!("Cleared {} old analytics events", removed);
        Ok(removed)
    }

    pub async fn export_events(
        &self,
        content_id: Option<Uuid>,
    ) -> ContentResult<Vec<AnalyticsEvent>> {
        let events = self.events.read().await;

        let result: Vec<AnalyticsEvent> = if let Some(cid) = content_id {
            events
                .iter()
                .filter(|e| e.content_id == cid)
                .cloned()
                .collect()
        } else {
            events.clone()
        };

        debug!("Exported {} analytics events", result.len());
        Ok(result)
    }
}

impl Default for ContentAnalyticsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalyticsEvent {
    pub fn new(content_id: Uuid, event_type: EventType) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            content_id,
            user_id: None,
            region_id: None,
            event_type,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_region(mut self, region_id: Uuid) -> Self {
        self.region_id = Some(region_id);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}
