//! Business Intelligence and Reporting System for OpenSim Next
//!
//! Provides comprehensive analytics, business intelligence, predictive analytics,
//! and automated reporting capabilities for virtual world operations.
//!
//! Features:
//! - Real-time analytics data collection and processing
//! - Business intelligence dashboards with KPI management
//! - Predictive analytics with machine learning forecasting
//! - Automated report generation and distribution
//! - Multi-format export capabilities (PDF, Excel, PowerBI, Tableau)
//! - Advanced data visualization and insights

pub mod business_intelligence;
pub mod data_collection;
pub mod predictive_analytics;
pub mod report_generation;
// pub mod dashboard;
// pub mod export;
// pub mod insights;
// pub mod kpi_manager;
pub mod manager;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
/// Analytics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDataPoint {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub category: AnalyticsCategory,
    pub metric_name: String,
    pub value: AnalyticsValue,
    pub dimensions: HashMap<String, String>,
    pub region_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
}

/// Analytics categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AnalyticsCategory {
    UserEngagement,
    SystemPerformance,
    BusinessMetrics,
    SecurityEvents,
    ContentUsage,
    EconomicActivity,
    SocialInteractions,
    VirtualWorldActivity,
    TechnicalMetrics,
    Custom(String),
}

/// Analytics value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Counter(u64),
    Histogram(Vec<f64>),
    Percentage(f32),
    Duration(Duration),
    Array(Vec<AnalyticsValue>),
    Object(HashMap<String, AnalyticsValue>),
}

/// Business KPI definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessKPI {
    pub kpi_id: Uuid,
    pub name: String,
    pub description: String,
    pub category: KPICategory,
    pub metric_source: String,
    pub calculation_method: CalculationMethod,
    pub target_value: Option<f64>,
    pub threshold_warning: Option<f64>,
    pub threshold_critical: Option<f64>,
    pub unit: String,
    pub frequency: ReportingFrequency,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// KPI categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KPICategory {
    Revenue,
    UserGrowth,
    Engagement,
    Performance,
    Security,
    Content,
    Technical,
    Business,
    Operations,
}

/// KPI calculation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalculationMethod {
    Sum,
    Average,
    Count,
    Percentage,
    Ratio,
    GrowthRate,
    Custom(String),
}

/// Reporting frequency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportingFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom(Duration),
}

/// Business dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDashboard {
    pub dashboard_id: Uuid,
    pub name: String,
    pub description: String,
    pub dashboard_type: DashboardType,
    pub widgets: Vec<DashboardWidget>,
    pub filters: Vec<DashboardFilter>,
    pub refresh_interval: Duration,
    pub is_public: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Dashboard types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DashboardType {
    Executive,
    Operational,
    Technical,
    Financial,
    UserAnalytics,
    ContentAnalytics,
    SecurityMonitoring,
    Custom,
}

/// Dashboard widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub widget_id: Uuid,
    pub widget_type: WidgetType,
    pub title: String,
    pub data_source: String,
    pub configuration: HashMap<String, serde_json::Value>,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub refresh_interval: Option<Duration>,
}

/// Widget types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    Gauge,
    Table,
    Heatmap,
    Map,
    KPICard,
    TrendIndicator,
    Histogram,
    ScatterPlot,
    Custom(String),
}

/// Widget position on dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub z_index: u32,
}

/// Widget size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
    pub responsive: bool,
}

/// Dashboard filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardFilter {
    pub filter_id: Uuid,
    pub name: String,
    pub filter_type: FilterType,
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
    pub is_active: bool,
}

/// Filter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterType {
    TimeRange,
    Category,
    Region,
    User,
    Value,
    Custom,
}

/// Filter operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    StartsWith,
    EndsWith,
    Between,
    In,
    NotIn,
}

/// Report definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDefinition {
    pub report_id: Uuid,
    pub name: String,
    pub description: String,
    pub report_type: ReportType,
    pub template_id: Option<Uuid>,
    pub data_sources: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub schedule: Option<ReportSchedule>,
    pub output_formats: Vec<OutputFormat>,
    pub distribution_list: Vec<ReportDistribution>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Report types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    ExecutiveSummary,
    Financial,
    UserAnalytics,
    SystemPerformance,
    SecurityCompliance,
    ContentAnalytics,
    BusinessIntelligence,
    Custom,
}

/// Report scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSchedule {
    pub schedule_id: Uuid,
    pub frequency: ReportingFrequency,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub timezone: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub is_active: bool,
}

/// Output formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    PDF,
    Excel,
    CSV,
    JSON,
    HTML,
    PowerBI,
    Tableau,
    PNG,
    SVG,
    Custom(String),
}

/// Report distribution methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportDistribution {
    Email {
        recipients: Vec<String>,
        subject_template: String,
        body_template: Option<String>,
    },
    Dashboard {
        dashboard_id: Uuid,
        widget_id: Option<Uuid>,
    },
    API {
        endpoint_url: String,
        method: String,
        headers: HashMap<String, String>,
    },
    FileSystem {
        path: String,
        filename_template: String,
    },
    CloudStorage {
        provider: String,
        bucket: String,
        path: String,
    },
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
    SFTP {
        host: String,
        username: String,
        path: String,
    },
}

/// Predictive model definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveModel {
    pub model_id: Uuid,
    pub name: String,
    pub description: String,
    pub model_type: ModelType,
    pub algorithm: ModelAlgorithm,
    pub input_features: Vec<String>,
    pub target_variable: String,
    pub training_data_source: String,
    pub model_parameters: HashMap<String, serde_json::Value>,
    pub performance_metrics: ModelPerformanceMetrics,
    pub last_trained: DateTime<Utc>,
    pub is_active: bool,
    pub created_by: Uuid,
}

/// Model types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Regression,
    Classification,
    TimeSeries,
    Clustering,
    AnomalyDetection,
    Recommendation,
    Forecasting,
}

/// ML algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelAlgorithm {
    LinearRegression,
    RandomForest,
    XGBoost,
    ARIMA,
    SARIMA,
    Prophet,
    NeuralNetwork,
    SVM,
    KMeans,
    DBSCAN,
    Custom(String),
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub accuracy: Option<f64>,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1_score: Option<f64>,
    pub rmse: Option<f64>,
    pub mae: Option<f64>,
    pub r_squared: Option<f64>,
    pub cross_validation_score: Option<f64>,
    pub training_time_seconds: f64,
    pub validation_date: DateTime<Utc>,
}

/// Forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    pub forecast_id: Uuid,
    pub model_id: Uuid,
    pub metric_name: String,
    pub forecast_period: Duration,
    pub confidence_level: f32,
    pub predicted_values: Vec<ForecastPoint>,
    pub confidence_intervals: Vec<ConfidenceInterval>,
    pub scenario_analysis: Option<ScenarioAnalysis>,
    pub generated_at: DateTime<Utc>,
}

/// Individual forecast point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    pub timestamp: DateTime<Utc>,
    pub predicted_value: f64,
    pub confidence_score: f32,
}

/// Confidence interval for predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub timestamp: DateTime<Utc>,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence_level: f32,
}

/// Scenario analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioAnalysis {
    pub best_case: ScenarioResult,
    pub worst_case: ScenarioResult,
    pub most_likely: ScenarioResult,
    pub custom_scenarios: HashMap<String, ScenarioResult>,
}

/// Scenario result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub probability: f32,
    pub predicted_outcome: f64,
    pub confidence_score: f32,
    pub assumptions: Vec<String>,
    pub risk_factors: Vec<RiskFactor>,
}

/// Risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_name: String,
    pub impact_level: RiskLevel,
    pub probability: f32,
    pub mitigation_strategies: Vec<String>,
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Analytics and reporting errors
#[derive(Debug, thiserror::Error)]
pub enum ReportingError {
    #[error("Data source not found: {0}")]
    DataSourceNotFound(String),

    #[error("Invalid report configuration: {reason}")]
    InvalidConfiguration { reason: String },

    #[error("Report generation failed: {reason}")]
    GenerationFailed { reason: String },

    #[error("Model training failed: {reason}")]
    ModelTrainingFailed { reason: String },

    #[error("Prediction failed: {reason}")]
    PredictionFailed { reason: String },

    #[error("Export failed: {format:?} - {reason}")]
    ExportFailed {
        format: OutputFormat,
        reason: String,
    },

    #[error("Distribution failed: {reason}")]
    DistributionFailed { reason: String },

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Generic error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

/// Result type for reporting operations
pub type ReportingResult<T> = Result<T, ReportingError>;

impl AnalyticsValue {
    /// Extract numeric value if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            AnalyticsValue::Integer(i) => Some(*i as f64),
            AnalyticsValue::Float(f) => Some(*f),
            AnalyticsValue::Counter(c) => Some(*c as f64),
            AnalyticsValue::Percentage(p) => Some(*p as f64),
            _ => None,
        }
    }

    /// Check if value is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            AnalyticsValue::Integer(_)
                | AnalyticsValue::Float(_)
                | AnalyticsValue::Counter(_)
                | AnalyticsValue::Percentage(_)
        )
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            AnalyticsValue::Integer(i) => i.to_string(),
            AnalyticsValue::Float(f) => f.to_string(),
            AnalyticsValue::Boolean(b) => b.to_string(),
            AnalyticsValue::String(s) => s.clone(),
            AnalyticsValue::Counter(c) => c.to_string(),
            AnalyticsValue::Percentage(p) => format!("{:.2}%", p),
            AnalyticsValue::Duration(d) => format!("{}ms", d.num_milliseconds()),
            AnalyticsValue::Array(arr) => format!("[{}]", arr.len()),
            AnalyticsValue::Object(obj) => format!("{{{} keys}}", obj.len()),
            AnalyticsValue::Histogram(hist) => format!("histogram[{}]", hist.len()),
        }
    }
}

impl Default for WidgetPosition {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z_index: 0,
        }
    }
}

impl Default for WidgetSize {
    fn default() -> Self {
        Self {
            width: 300,
            height: 200,
            responsive: true,
        }
    }
}

impl Default for ModelPerformanceMetrics {
    fn default() -> Self {
        Self {
            accuracy: None,
            precision: None,
            recall: None,
            f1_score: None,
            rmse: None,
            mae: None,
            r_squared: None,
            cross_validation_score: None,
            training_time_seconds: 0.0,
            validation_date: Utc::now(),
        }
    }
}

impl BusinessKPI {
    /// Calculate current KPI value based on recent data
    pub fn calculate_current_value(&self, data_points: &[AnalyticsDataPoint]) -> Option<f64> {
        let relevant_points: Vec<&AnalyticsDataPoint> = data_points
            .iter()
            .filter(|dp| dp.metric_name == self.metric_source)
            .collect();

        if relevant_points.is_empty() {
            return None;
        }

        match self.calculation_method {
            CalculationMethod::Sum => Some(
                relevant_points
                    .iter()
                    .filter_map(|dp| dp.value.as_f64())
                    .sum(),
            ),
            CalculationMethod::Average => {
                let values: Vec<f64> = relevant_points
                    .iter()
                    .filter_map(|dp| dp.value.as_f64())
                    .collect();
                if values.is_empty() {
                    None
                } else {
                    Some(values.iter().sum::<f64>() / values.len() as f64)
                }
            }
            CalculationMethod::Count => Some(relevant_points.len() as f64),
            _ => None, // More complex calculations would be implemented
        }
    }

    /// Get KPI status based on current value
    pub fn get_status(&self, current_value: f64) -> KPIStatus {
        if let Some(critical) = self.threshold_critical {
            if current_value <= critical {
                return KPIStatus::Critical;
            }
        }

        if let Some(warning) = self.threshold_warning {
            if current_value <= warning {
                return KPIStatus::Warning;
            }
        }

        if let Some(target) = self.target_value {
            if current_value >= target {
                KPIStatus::Excellent
            } else {
                KPIStatus::Good
            }
        } else {
            KPIStatus::Unknown
        }
    }
}

/// KPI status levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KPIStatus {
    Excellent,
    Good,
    Warning,
    Critical,
    Unknown,
}

/// Data aggregation time windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeWindow {
    LastHour,
    Last24Hours,
    Last7Days,
    Last30Days,
    LastQuarter,
    LastYear,
    Custom {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

impl TimeWindow {
    /// Get the start time for this window
    pub fn start_time(&self) -> DateTime<Utc> {
        let now = Utc::now();
        match self {
            TimeWindow::LastHour => now - Duration::hours(1),
            TimeWindow::Last24Hours => now - Duration::days(1),
            TimeWindow::Last7Days => now - Duration::days(7),
            TimeWindow::Last30Days => now - Duration::days(30),
            TimeWindow::LastQuarter => now - Duration::days(90),
            TimeWindow::LastYear => now - Duration::days(365),
            TimeWindow::Custom { start, .. } => *start,
        }
    }

    /// Get the end time for this window
    pub fn end_time(&self) -> DateTime<Utc> {
        match self {
            TimeWindow::Custom { end, .. } => *end,
            _ => Utc::now(),
        }
    }
}
