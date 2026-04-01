//! Advanced Analytics & Business Intelligence Platform
//! 
//! Comprehensive analytics, reporting, and business intelligence system
//! for enterprise virtual world operations and management.

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::database::DatabaseManager;
use crate::monitoring::MetricsCollector;
use std::sync::Arc;
use anyhow::Result;

pub mod data_collection;
pub mod business_intelligence;
pub mod predictive_analytics;
pub mod reporting;
pub mod dashboard;
pub mod export;

pub use data_collection::*;
pub use business_intelligence::*;
pub use predictive_analytics::*;
pub use reporting::*;
pub use dashboard::*;
pub use export::*;

/// Analytics system errors
#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Data collection failed: {reason}")]
    DataCollectionFailed { reason: String },
    
    #[error("Analytics processing failed: {reason}")]
    ProcessingFailed { reason: String },
    
    #[error("Report generation failed: {reason}")]
    ReportGenerationFailed { reason: String },
    
    #[error("Export failed: {reason}")]
    ExportFailed { reason: String },
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Configuration error: {reason}")]
    ConfigurationError { reason: String },
    
    #[error("Prediction failed: {reason}")]
    PredictionFailed { reason: String },
}

/// Analytics system result type
pub type AnalyticsResult<T> = Result<T, AnalyticsError>;

/// Main analytics manager
pub struct AnalyticsManager {
    database: Arc<DatabaseManager>,
    metrics_collector: Arc<MetricsCollector>,
    data_collector: DataCollector,
    bi_engine: BusinessIntelligenceEngine,
    predictive_engine: PredictiveAnalyticsEngine,
    reporting_engine: ReportingEngine,
    dashboard_manager: DashboardManager,
    export_manager: ExportManager,
    config: AnalyticsConfig,
}

/// Analytics system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub data_collection_interval_seconds: u32,
    pub real_time_analytics_enabled: bool,
    pub predictive_analytics_enabled: bool,
    pub data_retention_days: u32,
    pub max_concurrent_reports: u32,
    pub export_formats: Vec<ExportFormat>,
    pub dashboard_refresh_interval_seconds: u32,
    pub ai_insights_enabled: bool,
    pub cost_analytics_enabled: bool,
    pub security_analytics_enabled: bool,
    pub compliance_reporting_enabled: bool,
}

/// Supported export formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    JSON,
    CSV,
    Excel,
    PDF,
    PowerBI,
    Tableau,
    Grafana,
    Custom(String),
}

/// Analytics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDataPoint {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub category: DataCategory,
    pub metric_name: String,
    pub metric_value: MetricValue,
    pub dimensions: HashMap<String, String>,
    pub tags: Vec<String>,
    pub source: DataSource,
    pub confidence_score: Option<f32>,
}

/// Data categories for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataCategory {
    /// User engagement and behavior
    UserEngagement,
    /// System performance metrics
    SystemPerformance,
    /// Business metrics (revenue, costs)
    BusinessMetrics,
    /// Security and compliance
    SecurityCompliance,
    /// Grid federation metrics
    GridFederation,
    /// Virtual reality usage
    VRUsage,
    /// Mobile platform metrics
    MobileMetrics,
    /// Social interaction metrics
    SocialMetrics,
    /// Economic activity
    EconomicActivity,
    /// Content creation and consumption
    ContentMetrics,
    /// Custom business-specific metrics
    Custom(String),
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<String>),
    Object(HashMap<String, String>),
}

/// Data sources for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    /// Server system metrics
    SystemMetrics,
    /// User interaction tracking
    UserTracking,
    /// Database queries
    DatabaseQueries,
    /// External API calls
    ExternalAPIs,
    /// Mobile applications
    MobileApps,
    /// VR/XR systems
    VRSystems,
    /// Grid federation
    GridFederation,
    /// Economic transactions
    EconomicSystem,
    /// Social platforms
    SocialSystems,
    /// AI/ML systems
    AISystems,
    /// Third-party integrations
    ThirdParty(String),
}

/// Real-time analytics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub region_id: Option<Uuid>,
    pub grid_id: Option<Uuid>,
    pub event_data: HashMap<String, MetricValue>,
    pub severity: EventSeverity,
    pub requires_action: bool,
}

/// Event types for real-time analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    UserLogin,
    UserLogout,
    RegionEntry,
    RegionExit,
    EconomicTransaction,
    SocialInteraction,
    SystemAlert,
    PerformanceIssue,
    SecurityIncident,
    GridFederationEvent,
    VRSessionStart,
    VRSessionEnd,
    ContentCreation,
    ContentConsumption,
    Custom(String),
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Analytics insight generated by AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsInsight {
    pub insight_id: Uuid,
    pub title: String,
    pub description: String,
    pub insight_type: InsightType,
    pub confidence_score: f32,
    pub impact_score: f32,
    pub recommended_actions: Vec<String>,
    pub supporting_data: Vec<AnalyticsDataPoint>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

/// Types of AI-generated insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    /// User behavior patterns
    UserBehaviorPattern,
    /// Performance optimization opportunity
    PerformanceOptimization,
    /// Cost optimization opportunity
    CostOptimization,
    /// Security risk detection
    SecurityRisk,
    /// Business opportunity
    BusinessOpportunity,
    /// System anomaly detection
    SystemAnomaly,
    /// Predictive maintenance
    PredictiveMaintenance,
    /// Capacity planning
    CapacityPlanning,
    /// Revenue optimization
    RevenueOptimization,
    /// User retention risk
    UserRetentionRisk,
}

/// Business intelligence KPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessKPI {
    pub kpi_id: Uuid,
    pub name: String,
    pub description: String,
    pub category: KPICategory,
    pub current_value: MetricValue,
    pub target_value: Option<MetricValue>,
    pub trend: TrendDirection,
    pub time_period: TimePeriod,
    pub calculation_method: String,
    pub last_updated: DateTime<Utc>,
    pub historical_values: Vec<HistoricalKPIValue>,
}

/// KPI categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KPICategory {
    /// User engagement metrics
    UserEngagement,
    /// Financial performance
    Financial,
    /// System performance
    SystemPerformance,
    /// Security metrics
    Security,
    /// Growth metrics
    Growth,
    /// Operational efficiency
    OperationalEfficiency,
    /// Customer satisfaction
    CustomerSatisfaction,
    /// Innovation metrics
    Innovation,
}

/// Trend direction for KPIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
    Unknown,
}

/// Time periods for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePeriod {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom { start: DateTime<Utc>, end: DateTime<Utc> },
}

/// Historical KPI value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalKPIValue {
    pub timestamp: DateTime<Utc>,
    pub value: MetricValue,
    pub context: Option<HashMap<String, String>>,
}

impl AnalyticsManager {
    /// Create new analytics manager
    pub async fn new(
        database: Arc<DatabaseManager>,
        metrics_collector: Arc<MetricsCollector>,
        config: AnalyticsConfig,
    ) -> AnalyticsResult<Self> {
        let data_collector = DataCollector::new(
            database.clone(),
            metrics_collector.clone(),
            config.clone(),
        )?;
        
        let bi_engine = BusinessIntelligenceEngine::new(
            database.clone(),
            config.clone(),
        )?;
        
        let predictive_engine = PredictiveAnalyticsEngine::new(
            database.clone(),
            config.clone(),
        )?;
        
        let reporting_engine = ReportingEngine::new(
            database.clone(),
            config.clone(),
        )?;
        
        let dashboard_manager = DashboardManager::new(
            database.clone(),
            config.clone(),
        )?;
        
        let export_manager = ExportManager::new(
            database.clone(),
            config.clone(),
        )?;
        
        Ok(Self {
            database,
            metrics_collector,
            data_collector,
            bi_engine,
            predictive_engine,
            reporting_engine,
            dashboard_manager,
            export_manager,
            config,
        })
    }
    
    /// Initialize the analytics system
    pub async fn initialize(&self) -> AnalyticsResult<()> {
        // Initialize data collection
        self.data_collector.initialize().await?;
        
        // Initialize business intelligence engine
        self.bi_engine.initialize().await?;
        
        // Initialize predictive analytics
        if self.config.predictive_analytics_enabled {
            self.predictive_engine.initialize().await?;
        }
        
        // Initialize reporting engine
        self.reporting_engine.initialize().await?;
        
        // Initialize dashboard manager
        self.dashboard_manager.initialize().await?;
        
        // Initialize export manager
        self.export_manager.initialize().await?;
        
        Ok(())
    }
    
    /// Collect analytics data point
    pub async fn collect_data_point(&self, data_point: AnalyticsDataPoint) -> AnalyticsResult<()> {
        self.data_collector.collect_data_point(data_point).await
    }
    
    /// Process real-time event
    pub async fn process_real_time_event(&self, event: RealTimeEvent) -> AnalyticsResult<()> {
        self.data_collector.process_real_time_event(event).await
    }
    
    /// Generate business intelligence insights
    pub async fn generate_insights(&self, time_period: TimePeriod) -> AnalyticsResult<Vec<AnalyticsInsight>> {
        self.bi_engine.generate_insights(time_period).await
    }
    
    /// Get business KPIs
    pub async fn get_business_kpis(&self, category: Option<KPICategory>) -> AnalyticsResult<Vec<BusinessKPI>> {
        self.bi_engine.get_business_kpis(category).await
    }
    
    /// Generate predictive forecast
    pub async fn generate_forecast(
        &self,
        metric_name: String,
        forecast_period: TimePeriod,
    ) -> AnalyticsResult<PredictiveForecast> {
        self.predictive_engine.generate_forecast(metric_name, forecast_period).await
    }
    
    /// Generate analytics report
    pub async fn generate_report(&self, report_request: ReportRequest) -> AnalyticsResult<Report> {
        self.reporting_engine.generate_report(report_request).await
    }
    
    /// Get dashboard data
    pub async fn get_dashboard_data(&self, dashboard_id: Uuid) -> AnalyticsResult<DashboardData> {
        self.dashboard_manager.get_dashboard_data(dashboard_id).await
    }
    
    /// Export analytics data
    pub async fn export_data(&self, export_request: ExportRequest) -> AnalyticsResult<ExportResult> {
        self.export_manager.export_data(export_request).await
    }
    
    /// Get analytics system health
    pub async fn get_system_health(&self) -> AnalyticsSystemHealth {
        AnalyticsSystemHealth {
            status: SystemHealthStatus::Healthy,
            data_collection_rate: 1000.0,
            processing_latency_ms: 50.0,
            storage_usage_percent: 75.0,
            active_dashboards: 15,
            active_reports: 8,
            real_time_events_per_second: 250.0,
            ai_insights_generated_today: 42,
            last_health_check: Utc::now(),
        }
    }
}

/// Analytics system health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSystemHealth {
    pub status: SystemHealthStatus,
    pub data_collection_rate: f64,
    pub processing_latency_ms: f64,
    pub storage_usage_percent: f64,
    pub active_dashboards: u32,
    pub active_reports: u32,
    pub real_time_events_per_second: f64,
    pub ai_insights_generated_today: u32,
    pub last_health_check: DateTime<Utc>,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemHealthStatus {
    Healthy,
    Warning,
    Critical,
    Degraded,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            data_collection_interval_seconds: 60,
            real_time_analytics_enabled: true,
            predictive_analytics_enabled: true,
            data_retention_days: 365,
            max_concurrent_reports: 10,
            export_formats: vec![
                ExportFormat::JSON,
                ExportFormat::CSV,
                ExportFormat::Excel,
                ExportFormat::PDF,
            ],
            dashboard_refresh_interval_seconds: 30,
            ai_insights_enabled: true,
            cost_analytics_enabled: true,
            security_analytics_enabled: true,
            compliance_reporting_enabled: true,
        }
    }
}