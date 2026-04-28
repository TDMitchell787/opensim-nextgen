//! Real-Time Analytics Data Collection System
//!
//! Provides comprehensive data collection capabilities for virtual world analytics
//! with multi-category support, real-time processing, and efficient storage.

use super::{
    AnalyticsCategory, AnalyticsDataPoint, AnalyticsValue, ReportingError, ReportingResult,
    TimeWindow,
};
use crate::database::DatabaseManager;
use crate::monitoring::MetricsCollector;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row; // EADS fix for PgRow.get() method
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Real-time analytics data collector
pub struct DataCollector {
    database: Arc<DatabaseManager>,
    metrics_collector: Arc<MetricsCollector>,
    data_buffer: Arc<RwLock<Vec<AnalyticsDataPoint>>>,
    aggregation_cache: Arc<RwLock<HashMap<String, AggregatedMetric>>>,
    collection_config: DataCollectionConfig,
    event_sender: mpsc::UnboundedSender<CollectionEvent>,
    _event_receiver: Arc<RwLock<mpsc::UnboundedReceiver<CollectionEvent>>>,
}

/// Data collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionConfig {
    pub buffer_size: usize,
    pub flush_interval_seconds: u64,
    pub aggregation_window_seconds: u64,
    pub retention_policy: RetentionPolicy,
    pub collection_filters: Vec<CollectionFilter>,
    pub real_time_processing: bool,
    pub batch_processing: bool,
    pub compression_enabled: bool,
    pub quality_validation: bool,
}

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub raw_data_days: u32,
    pub hourly_aggregates_days: u32,
    pub daily_aggregates_days: u32,
    pub monthly_aggregates_days: u32,
    pub archive_to_cold_storage: bool,
    pub cold_storage_path: Option<String>,
}

/// Collection filter for data processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionFilter {
    pub filter_id: Uuid,
    pub name: String,
    pub category: Option<AnalyticsCategory>,
    pub metric_pattern: Option<String>,
    pub value_filter: Option<ValueFilter>,
    pub dimension_filters: HashMap<String, String>,
    pub sampling_rate: f32, // 0.0 to 1.0
    pub is_active: bool,
}

/// Value filter for numeric filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueFilter {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub exclude_outliers: bool,
    pub outlier_threshold: f64, // Standard deviations
}

/// Aggregated metric for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub metric_key: String,
    pub category: AnalyticsCategory,
    pub time_window: TimeWindow,
    pub count: u64,
    pub sum: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub variance: f64,
    pub percentiles: HashMap<u8, f64>, // 50th, 90th, 95th, 99th
    pub last_updated: DateTime<Utc>,
}

/// Collection events for real-time processing
#[derive(Debug, Clone)]
pub enum CollectionEvent {
    DataPointReceived(AnalyticsDataPoint),
    AggregationComplete(String),
    FlushRequired,
    QualityIssueDetected(QualityIssue),
    RetentionCleanupComplete,
}

/// Data quality issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub issue_id: Uuid,
    pub issue_type: QualityIssueType,
    pub data_point_id: Option<Uuid>,
    pub metric_name: String,
    pub description: String,
    pub severity: QualitySeverity,
    pub detected_at: DateTime<Utc>,
    pub suggested_action: String,
}

/// Types of data quality issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityIssueType {
    OutOfRange,
    MissingDimensions,
    DuplicateData,
    InvalidFormat,
    TemporalAnomaly,
    VolumeAnomaly,
    ConsistencyViolation,
}

/// Quality issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStatistics {
    pub total_data_points: u64,
    pub data_points_per_category: HashMap<AnalyticsCategory, u64>,
    pub data_points_per_hour: u64,
    pub buffer_utilization: f32,
    pub aggregation_cache_size: usize,
    pub quality_issues_count: u64,
    pub last_flush_time: DateTime<Utc>,
    pub collection_rate: f64, // points per second
    pub processing_latency_ms: f64,
    pub storage_size_mb: f64,
}

/// Real-time event for streaming analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: RealTimeEventType,
    pub category: AnalyticsCategory,
    pub data: HashMap<String, AnalyticsValue>,
    pub context: EventContext,
}

/// Real-time event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealTimeEventType {
    UserLogin,
    UserLogout,
    AvatarCreated,
    AvatarUpdated,
    TransactionCompleted,
    RegionEntered,
    RegionExited,
    ObjectCreated,
    ObjectDeleted,
    ChatMessage,
    FriendRequest,
    GroupJoined,
    PerformanceAlert,
    SecurityEvent,
    SystemError,
    CustomEvent(String),
}

/// Event context for additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub region_id: Option<Uuid>,
    pub client_type: Option<String>,
    pub client_version: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub geographic_info: Option<GeographicInfo>,
}

/// Geographic information for location-based analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicInfo {
    pub country: String,
    pub region: Option<String>,
    pub city: Option<String>,
    pub timezone: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl DataCollector {
    /// Create new data collector
    pub async fn new(
        database: Arc<DatabaseManager>,
        metrics_collector: Arc<MetricsCollector>,
        config: DataCollectionConfig,
    ) -> ReportingResult<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let collector = Self {
            database,
            metrics_collector,
            data_buffer: Arc::new(RwLock::new(Vec::with_capacity(config.buffer_size))),
            aggregation_cache: Arc::new(RwLock::new(HashMap::new())),
            collection_config: config,
            event_sender,
            _event_receiver: Arc::new(RwLock::new(event_receiver)),
        };

        // Initialize database tables
        collector.initialize_tables().await?;

        // Start background tasks
        collector.start_background_tasks().await?;

        Ok(collector)
    }

    /// Initialize database tables for analytics data
    async fn initialize_tables(&self) -> ReportingResult<()> {
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;

        // Analytics data points table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analytics_data_points (
                id UUID PRIMARY KEY,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                category TEXT NOT NULL,
                metric_name TEXT NOT NULL,
                value_type TEXT NOT NULL,
                value_data JSONB NOT NULL,
                dimensions JSONB,
                region_id UUID,
                user_id UUID,
                session_id UUID,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
        "#,
        )
        .execute(pool)
        .await?;

        // Create indexes for efficient querying
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_analytics_timestamp ON analytics_data_points(timestamp);
            CREATE INDEX IF NOT EXISTS idx_analytics_category ON analytics_data_points(category);
            CREATE INDEX IF NOT EXISTS idx_analytics_metric ON analytics_data_points(metric_name);
            CREATE INDEX IF NOT EXISTS idx_analytics_user ON analytics_data_points(user_id);
            CREATE INDEX IF NOT EXISTS idx_analytics_region ON analytics_data_points(region_id);
            CREATE INDEX IF NOT EXISTS idx_analytics_composite ON analytics_data_points(category, metric_name, timestamp);
        "#).execute(pool).await?;

        // Aggregated metrics table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analytics_aggregated_metrics (
                metric_key TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                time_window TEXT NOT NULL,
                count BIGINT NOT NULL,
                sum_value DOUBLE PRECISION NOT NULL,
                avg_value DOUBLE PRECISION NOT NULL,
                min_value DOUBLE PRECISION NOT NULL,
                max_value DOUBLE PRECISION NOT NULL,
                variance DOUBLE PRECISION NOT NULL,
                percentiles JSONB NOT NULL,
                last_updated TIMESTAMP WITH TIME ZONE NOT NULL
            )
        "#,
        )
        .execute(pool)
        .await?;

        // Real-time events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analytics_realtime_events (
                event_id UUID PRIMARY KEY,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                event_type TEXT NOT NULL,
                category TEXT NOT NULL,
                data JSONB NOT NULL,
                context JSONB NOT NULL,
                processed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
        "#,
        )
        .execute(pool)
        .await?;

        // Quality issues table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analytics_quality_issues (
                issue_id UUID PRIMARY KEY,
                issue_type TEXT NOT NULL,
                data_point_id UUID,
                metric_name TEXT NOT NULL,
                description TEXT NOT NULL,
                severity TEXT NOT NULL,
                detected_at TIMESTAMP WITH TIME ZONE NOT NULL,
                suggested_action TEXT NOT NULL,
                resolved BOOLEAN DEFAULT FALSE,
                resolved_at TIMESTAMP WITH TIME ZONE
            )
        "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Start background processing tasks
    async fn start_background_tasks(&self) -> ReportingResult<()> {
        // Start data flushing task
        self.start_flush_task().await;

        // Start aggregation task
        self.start_aggregation_task().await;

        // Start retention cleanup task
        self.start_retention_cleanup_task().await;

        Ok(())
    }

    /// Collect analytics data point
    pub async fn collect_data_point(&self, data_point: AnalyticsDataPoint) -> ReportingResult<()> {
        // Validate data quality
        if self.collection_config.quality_validation {
            if let Some(issue) = self.validate_data_quality(&data_point).await? {
                self.handle_quality_issue(issue).await?;
            }
        }

        // Apply collection filters
        if !self.should_collect_data_point(&data_point).await? {
            return Ok(());
        }

        // Add to buffer
        {
            let mut buffer = self.data_buffer.write().await;
            buffer.push(data_point.clone());

            // Check if buffer is full
            if buffer.len() >= self.collection_config.buffer_size {
                self.event_sender
                    .send(CollectionEvent::FlushRequired)
                    .map_err(|e| ReportingError::GenerationFailed {
                        reason: format!("Failed to send flush event: {}", e),
                    })?;
            }
        }

        // Process for real-time analytics
        if self.collection_config.real_time_processing {
            self.process_real_time_data_point(&data_point).await?;
        }

        // Update metrics before sending event
        let mut tags = HashMap::new();
        tags.insert("category".to_string(), data_point.category.to_string());
        self.metrics_collector
            .increment_counter("analytics_data_points_collected", tags)
            .await?;

        // Send collection event
        self.event_sender
            .send(CollectionEvent::DataPointReceived(data_point))
            .map_err(|e| ReportingError::GenerationFailed {
                reason: format!("Failed to send data point event: {}", e),
            })?;

        Ok(())
    }

    /// Process real-time analytics event
    pub async fn process_real_time_event(&self, event: RealTimeEvent) -> ReportingResult<()> {
        // Store event in database
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;

        sqlx::query(
            r#"
            INSERT INTO analytics_realtime_events 
            (event_id, timestamp, event_type, category, data, context)
            VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        )
        .bind(&event.event_id)
        .bind(&event.timestamp)
        .bind(&format!("{:?}", event.event_type))
        .bind(&format!("{:?}", event.category))
        .bind(serde_json::to_value(&event.data)?)
        .bind(serde_json::to_value(&event.context)?)
        .execute(pool)
        .await?;

        // Update real-time metrics
        self.update_real_time_metrics(&event).await?;

        Ok(())
    }

    /// Get analytics data for time period
    pub async fn get_analytics_data(
        &self,
        category: Option<AnalyticsCategory>,
        metric_name: Option<String>,
        time_window: TimeWindow,
        limit: Option<u32>,
    ) -> ReportingResult<Vec<AnalyticsDataPoint>> {
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;
        let start_time = time_window.start_time();
        let end_time = time_window.end_time();

        let mut query = String::from(
            r#"
            SELECT id, timestamp, category, metric_name, value_type, value_data, 
                   dimensions, region_id, user_id, session_id
            FROM analytics_data_points 
            WHERE timestamp >= $1 AND timestamp <= $2
        "#,
        );

        let mut bind_count = 2;
        if category.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND category = ${}", bind_count));
        }

        if metric_name.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND metric_name = ${}", bind_count));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = limit {
            bind_count += 1;
            query.push_str(&format!(" LIMIT ${}", bind_count));
        }

        let mut query_builder = sqlx::query(&query).bind(&start_time).bind(&end_time);

        if let Some(cat) = category {
            let cat_str = format!("{:?}", cat);
            query_builder = query_builder.bind(cat_str);
        }

        if let Some(metric) = &metric_name {
            query_builder = query_builder.bind(metric);
        }

        if let Some(limit) = limit {
            query_builder = query_builder.bind(limit as i64);
        }

        let rows = query_builder.fetch_all(pool).await?;

        let mut data_points = Vec::new();
        for row in rows {
            let value_data: serde_json::Value = row.get("value_data");
            let value: AnalyticsValue = serde_json::from_value(value_data)?;

            let dimensions: Option<serde_json::Value> = row.get("dimensions");
            let dimensions = if let Some(d) = dimensions {
                serde_json::from_value(d)?
            } else {
                HashMap::new()
            };

            data_points.push(AnalyticsDataPoint {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                category: serde_json::from_str(&row.get::<String, _>("category"))?,
                metric_name: row.get("metric_name"),
                value,
                dimensions,
                region_id: row.get("region_id"),
                user_id: row.get("user_id"),
                session_id: row.get("session_id"),
            });
        }

        Ok(data_points)
    }

    /// Get aggregated metrics
    pub async fn get_aggregated_metrics(
        &self,
        category: Option<AnalyticsCategory>,
        time_window: TimeWindow,
    ) -> ReportingResult<Vec<AggregatedMetric>> {
        let cache = self.aggregation_cache.read().await;
        let time_window_str = format!("{:?}", time_window);

        let metrics: Vec<AggregatedMetric> = cache
            .values()
            .filter(|metric| {
                let category_match = category
                    .as_ref()
                    .map(|cat| metric.category == *cat)
                    .unwrap_or(true);

                let time_match = format!("{:?}", metric.time_window) == time_window_str;

                category_match && time_match
            })
            .cloned()
            .collect();

        Ok(metrics)
    }

    /// Get collection statistics
    pub async fn get_collection_statistics(&self) -> ReportingResult<CollectionStatistics> {
        let buffer = self.data_buffer.read().await;
        let cache = self.aggregation_cache.read().await;

        // Calculate statistics from buffer and cache
        let total_data_points = buffer.len() as u64;
        let buffer_utilization = buffer.len() as f32 / self.collection_config.buffer_size as f32;

        // Get data points per category from buffer
        let mut data_points_per_category = HashMap::new();
        for point in buffer.iter() {
            *data_points_per_category
                .entry(point.category.clone())
                .or_insert(0) += 1;
        }

        // Calculate collection rate (simplified - would need more sophisticated tracking)
        let collection_rate = total_data_points as f64 / 3600.0; // rough approximation

        Ok(CollectionStatistics {
            total_data_points,
            data_points_per_category,
            data_points_per_hour: total_data_points, // simplified
            buffer_utilization,
            aggregation_cache_size: cache.len(),
            quality_issues_count: 0,     // would query database
            last_flush_time: Utc::now(), // would track actual flush time
            collection_rate,
            processing_latency_ms: 5.0, // would measure actual latency
            storage_size_mb: 0.0,       // would calculate actual storage
        })
    }

    /// Validate data quality
    async fn validate_data_quality(
        &self,
        data_point: &AnalyticsDataPoint,
    ) -> ReportingResult<Option<QualityIssue>> {
        // Check for out-of-range values
        if let Some(numeric_value) = data_point.value.as_f64() {
            if numeric_value.is_infinite() || numeric_value.is_nan() {
                return Ok(Some(QualityIssue {
                    issue_id: Uuid::new_v4(),
                    issue_type: QualityIssueType::OutOfRange,
                    data_point_id: Some(data_point.id),
                    metric_name: data_point.metric_name.clone(),
                    description: "Numeric value is infinite or NaN".to_string(),
                    severity: QualitySeverity::High,
                    detected_at: Utc::now(),
                    suggested_action: "Investigate data source and fix calculation".to_string(),
                }));
            }
        }

        // Check for missing required dimensions
        let required_dimensions = vec!["source", "version"]; // example
        for required_dim in required_dimensions {
            if !data_point.dimensions.contains_key(required_dim) {
                return Ok(Some(QualityIssue {
                    issue_id: Uuid::new_v4(),
                    issue_type: QualityIssueType::MissingDimensions,
                    data_point_id: Some(data_point.id),
                    metric_name: data_point.metric_name.clone(),
                    description: format!("Missing required dimension: {}", required_dim),
                    severity: QualitySeverity::Medium,
                    detected_at: Utc::now(),
                    suggested_action: "Add missing dimension to data collection".to_string(),
                }));
            }
        }

        Ok(None)
    }

    /// Check if data point should be collected based on filters
    async fn should_collect_data_point(
        &self,
        data_point: &AnalyticsDataPoint,
    ) -> ReportingResult<bool> {
        for filter in &self.collection_config.collection_filters {
            if !filter.is_active {
                continue;
            }

            // Check category filter
            if let Some(filter_category) = &filter.category {
                if data_point.category != *filter_category {
                    continue;
                }
            }

            // Check metric pattern
            if let Some(pattern) = &filter.metric_pattern {
                if !data_point.metric_name.contains(pattern) {
                    continue;
                }
            }

            // Apply sampling rate
            if filter.sampling_rate < 1.0 {
                let random_value: f32 = rand::random();
                if random_value > filter.sampling_rate {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Process data point for real-time analytics
    async fn process_real_time_data_point(
        &self,
        data_point: &AnalyticsDataPoint,
    ) -> ReportingResult<()> {
        // Update real-time aggregations
        let metric_key = format!(
            "{}_{}_{:?}",
            data_point.category.to_string(),
            data_point.metric_name,
            TimeWindow::LastHour
        );

        let mut cache = self.aggregation_cache.write().await;
        let entry = cache
            .entry(metric_key.clone())
            .or_insert_with(|| AggregatedMetric {
                metric_key: metric_key.clone(),
                category: data_point.category.clone(),
                time_window: TimeWindow::LastHour,
                count: 0,
                sum: 0.0,
                average: 0.0,
                min: f64::MAX,
                max: f64::MIN,
                variance: 0.0,
                percentiles: HashMap::new(),
                last_updated: Utc::now(),
            });

        // Update aggregation with new data point
        if let Some(value) = data_point.value.as_f64() {
            entry.count += 1;
            entry.sum += value;
            entry.average = entry.sum / entry.count as f64;
            entry.min = entry.min.min(value);
            entry.max = entry.max.max(value);
            entry.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Update real-time metrics based on event
    async fn update_real_time_metrics(&self, event: &RealTimeEvent) -> ReportingResult<()> {
        // Update metrics collector with real-time event data
        let mut tags = HashMap::new();
        tags.insert("event_type".to_string(), format!("{:?}", event.event_type));
        tags.insert("category".to_string(), format!("{:?}", event.category));
        self.metrics_collector
            .increment_counter("analytics_realtime_events", tags)
            .await?;

        // Update specific metrics based on event type
        match event.event_type {
            RealTimeEventType::UserLogin => {
                self.metrics_collector
                    .increment_counter("user_logins_total", HashMap::new())
                    .await?;
            }
            RealTimeEventType::TransactionCompleted => {
                if let Some(AnalyticsValue::Float(amount)) = event.data.get("amount") {
                    self.metrics_collector
                        .record_histogram("transaction_amounts", *amount, HashMap::new())
                        .await?;
                }
            }
            RealTimeEventType::PerformanceAlert => {
                self.metrics_collector
                    .increment_counter("performance_alerts_total", HashMap::new())
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle quality issue
    async fn handle_quality_issue(&self, issue: QualityIssue) -> ReportingResult<()> {
        // Store quality issue in database
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;

        sqlx::query(
            r#"
            INSERT INTO analytics_quality_issues 
            (issue_id, issue_type, data_point_id, metric_name, description, 
             severity, detected_at, suggested_action)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        )
        .bind(&issue.issue_id)
        .bind(&format!("{:?}", issue.issue_type))
        .bind(&issue.data_point_id)
        .bind(&issue.metric_name)
        .bind(&issue.description)
        .bind(&format!("{:?}", issue.severity))
        .bind(&issue.detected_at)
        .bind(&issue.suggested_action)
        .execute(pool)
        .await?;

        // Send quality issue event
        self.event_sender
            .send(CollectionEvent::QualityIssueDetected(issue))
            .map_err(|e| ReportingError::GenerationFailed {
                reason: format!("Failed to send quality issue event: {}", e),
            })?;

        Ok(())
    }

    /// Start data flushing background task
    async fn start_flush_task(&self) {
        let buffer = self.data_buffer.clone();
        let database = self.database.clone();
        let flush_interval =
            Duration::seconds(self.collection_config.flush_interval_seconds as i64);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                flush_interval.num_seconds() as u64,
            ));

            loop {
                interval.tick().await;

                // Flush buffer to database
                let mut buffer_guard = buffer.write().await;
                if !buffer_guard.is_empty() {
                    let data_points = buffer_guard.drain(..).collect::<Vec<_>>();
                    drop(buffer_guard);

                    // Batch insert data points
                    if let Err(e) =
                        Self::flush_data_points_to_database(&database, data_points).await
                    {
                        tracing::error!("Failed to flush data points: {}", e);
                    }
                }
            }
        });
    }

    /// Start aggregation background task
    async fn start_aggregation_task(&self) {
        let cache = self.aggregation_cache.clone();
        let database = self.database.clone();
        let aggregation_interval =
            Duration::seconds(self.collection_config.aggregation_window_seconds as i64);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                aggregation_interval.num_seconds() as u64,
            ));

            loop {
                interval.tick().await;

                // Update aggregations
                if let Err(e) = Self::update_aggregations(&database, &cache).await {
                    tracing::error!("Failed to update aggregations: {}", e);
                }
            }
        });
    }

    /// Start retention cleanup background task
    async fn start_retention_cleanup_task(&self) {
        let database = self.database.clone();
        let retention_policy = self.collection_config.retention_policy.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400)); // Daily

            loop {
                interval.tick().await;

                // Clean up old data based on retention policy
                if let Err(e) = Self::cleanup_old_data(&database, &retention_policy).await {
                    tracing::error!("Failed to cleanup old data: {}", e);
                }
            }
        });
    }

    /// Flush data points to database
    async fn flush_data_points_to_database(
        database: &DatabaseManager,
        data_points: Vec<AnalyticsDataPoint>,
    ) -> ReportingResult<()> {
        let pool_ref = database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;

        for data_point in data_points {
            sqlx::query(
                r#"
                INSERT INTO analytics_data_points 
                (id, timestamp, category, metric_name, value_type, value_data, 
                 dimensions, region_id, user_id, session_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            )
            .bind(&data_point.id)
            .bind(&data_point.timestamp)
            .bind(&format!("{:?}", data_point.category))
            .bind(&data_point.metric_name)
            .bind(&format!("{:?}", std::mem::discriminant(&data_point.value)))
            .bind(serde_json::to_value(&data_point.value)?)
            .bind(serde_json::to_value(&data_point.dimensions)?)
            .bind(&data_point.region_id)
            .bind(&data_point.user_id)
            .bind(&data_point.session_id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Update aggregations in cache and database
    async fn update_aggregations(
        database: &DatabaseManager,
        cache: &Arc<RwLock<HashMap<String, AggregatedMetric>>>,
    ) -> ReportingResult<()> {
        let cache_guard = cache.read().await;
        let pool_ref = database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;

        for (_, metric) in cache_guard.iter() {
            sqlx::query(
                r#"
                INSERT INTO analytics_aggregated_metrics 
                (metric_key, category, time_window, count, sum_value, avg_value, 
                 min_value, max_value, variance, percentiles, last_updated)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (metric_key) DO UPDATE SET
                count = EXCLUDED.count,
                sum_value = EXCLUDED.sum_value,
                avg_value = EXCLUDED.avg_value,
                min_value = EXCLUDED.min_value,
                max_value = EXCLUDED.max_value,
                variance = EXCLUDED.variance,
                percentiles = EXCLUDED.percentiles,
                last_updated = EXCLUDED.last_updated
            "#,
            )
            .bind(&metric.metric_key)
            .bind(&format!("{:?}", metric.category))
            .bind(&format!("{:?}", metric.time_window))
            .bind(metric.count as i64)
            .bind(metric.sum)
            .bind(metric.average)
            .bind(metric.min)
            .bind(metric.max)
            .bind(metric.variance)
            .bind(serde_json::to_value(&metric.percentiles)?)
            .bind(&metric.last_updated)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Clean up old data based on retention policy
    async fn cleanup_old_data(
        database: &DatabaseManager,
        retention_policy: &RetentionPolicy,
    ) -> ReportingResult<()> {
        let pool_ref = database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;
        let cutoff_date = Utc::now() - Duration::days(retention_policy.raw_data_days as i64);

        // Delete old raw data points
        sqlx::query(
            r#"
            DELETE FROM analytics_data_points 
            WHERE timestamp < $1
        "#,
        )
        .bind(&cutoff_date)
        .execute(pool)
        .await?;

        // Delete old quality issues
        let quality_cutoff = Utc::now() - Duration::days(30); // Keep quality issues for 30 days
        sqlx::query(
            r#"
            DELETE FROM analytics_quality_issues 
            WHERE detected_at < $1 AND resolved = true
        "#,
        )
        .bind(&quality_cutoff)
        .execute(pool)
        .await?;

        Ok(())
    }
}

impl Default for DataCollectionConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            flush_interval_seconds: 60,
            aggregation_window_seconds: 300,
            retention_policy: RetentionPolicy {
                raw_data_days: 90,
                hourly_aggregates_days: 365,
                daily_aggregates_days: 730,
                monthly_aggregates_days: 1095,
                archive_to_cold_storage: false,
                cold_storage_path: None,
            },
            collection_filters: vec![],
            real_time_processing: true,
            batch_processing: true,
            compression_enabled: true,
            quality_validation: true,
        }
    }
}

impl AnalyticsCategory {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl std::fmt::Display for QualityIssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for QualitySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
