//! Report Generation Engine for Automated Business Reporting
//!
//! Provides comprehensive report generation capabilities with multiple formats,
//! automated scheduling, template management, and multi-channel distribution.

use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::Row;  // EADS fix for PgRow.get() method
use crate::database::DatabaseManager;
use crate::monitoring::MetricsCollector;
use super::{
    ReportDefinition, ReportType, ReportSchedule, ReportingFrequency, OutputFormat,
    ReportDistribution, AnalyticsDataPoint, BusinessKPI, BusinessDashboard,
    TimeWindow, ReportingError, ReportingResult
};

/// Report generation engine
pub struct ReportGenerationEngine {
    database: Arc<DatabaseManager>,
    metrics_collector: Arc<MetricsCollector>,
    report_templates: Arc<RwLock<HashMap<Uuid, ReportTemplate>>>,
    generated_reports: Arc<RwLock<HashMap<Uuid, Report>>>,
    scheduled_reports: Arc<RwLock<HashMap<Uuid, ScheduledReport>>>,
    distribution_manager: ReportDistributionManager,
    generation_queue: Arc<RwLock<std::collections::VecDeque<ReportGenerationTask>>>,
    config: ReportGenerationConfig,
}

/// Report generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGenerationConfig {
    pub max_concurrent_generations: u32,
    pub default_timeout_minutes: u32,
    pub template_cache_size: usize,
    pub output_directory: String,
    pub temp_directory: String,
    pub max_report_size_mb: u64,
    pub retention_policy: ReportRetentionPolicy,
    pub compression_enabled: bool,
    pub watermark_enabled: bool,
    pub security_enabled: bool,
}

/// Report retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRetentionPolicy {
    pub daily_reports_days: u32,
    pub weekly_reports_weeks: u32,
    pub monthly_reports_months: u32,
    pub annual_reports_years: u32,
    pub archive_to_cold_storage: bool,
    pub cold_storage_path: Option<String>,
}

/// Report template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub template_id: Uuid,
    pub template_name: String,
    pub description: String,
    pub report_type: ReportType,
    pub template_format: TemplateFormat,
    pub template_content: String,
    pub variables: Vec<TemplateVariable>,
    pub sections: Vec<ReportSection>,
    pub styling: ReportStyling,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub version: String,
    pub is_active: bool,
}

/// Template format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateFormat {
    HTML,
    Markdown,
    LaTeX,
    JSON,
    XML,
    Custom(String),
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub variable_name: String,
    pub variable_type: VariableType,
    pub default_value: Option<serde_json::Value>,
    pub description: String,
    pub is_required: bool,
    pub validation_rules: Vec<ValidationRule>,
}

/// Variable types for templates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableType {
    String,
    Number,
    Boolean,
    Date,
    Array,
    Object,
    KPI,
    Chart,
    Table,
    Image,
}

/// Validation rules for variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationRuleType,
    pub rule_value: serde_json::Value,
    pub error_message: String,
}

/// Types of validation rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationRuleType {
    Required,
    MinLength,
    MaxLength,
    Pattern,
    Range,
    Enum,
    Custom,
}

/// Report section definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub section_id: Uuid,
    pub section_name: String,
    pub section_type: SectionType,
    pub content_template: String,
    pub data_sources: Vec<DataSource>,
    pub visualizations: Vec<Visualization>,
    pub order_index: u32,
    pub is_optional: bool,
    pub page_break_before: bool,
    pub page_break_after: bool,
}

/// Report section types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SectionType {
    ExecutiveSummary,
    Introduction,
    DataAnalysis,
    Charts,
    Tables,
    KPIDashboard,
    Recommendations,
    Appendix,
    Conclusion,
    Custom(String),
}

/// Data source for report sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub source_id: String,
    pub source_type: DataSourceType,
    pub query: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub cache_duration: Option<Duration>,
}

/// Data source types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataSourceType {
    Database,
    API,
    File,
    Cache,
    Calculation,
    External,
}

/// Visualization definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visualization {
    pub visualization_id: Uuid,
    pub visualization_type: VisualizationType,
    pub title: String,
    pub data_source: String,
    pub configuration: VisualizationConfig,
    pub styling: VisualizationStyling,
}

/// Visualization types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VisualizationType {
    LineChart,
    BarChart,
    PieChart,
    ScatterPlot,
    Heatmap,
    Table,
    KPICard,
    Gauge,
    Map,
    TreeMap,
    Histogram,
    BoxPlot,
    Custom(String),
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub width: u32,
    pub height: u32,
    pub responsive: bool,
    pub interactive: bool,
    pub data_labels: bool,
    pub legend_position: LegendPosition,
    pub color_scheme: ColorScheme,
    pub axes_configuration: Option<AxesConfiguration>,
    pub filters: Vec<VisualizationFilter>,
}

/// Legend position for charts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LegendPosition {
    Top,
    Bottom,
    Left,
    Right,
    Hidden,
}

/// Color scheme for visualizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub scheme_name: String,
    pub primary_color: String,
    pub secondary_colors: Vec<String>,
    pub background_color: String,
    pub text_color: String,
}

/// Axes configuration for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxesConfiguration {
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub secondary_y_axis: Option<AxisConfig>,
}

/// Individual axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    pub title: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub tick_interval: Option<f64>,
    pub format: String,
}

/// Visualization filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationFilter {
    pub field_name: String,
    pub filter_type: FilterType,
    pub filter_value: serde_json::Value,
}

/// Filter types for visualizations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterType {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    Range,
    In,
    NotIn,
}

/// Visualization styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationStyling {
    pub font_family: String,
    pub font_size: u32,
    pub border_width: u32,
    pub border_color: String,
    pub padding: u32,
    pub margin: u32,
    pub custom_css: Option<String>,
}

/// Report styling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStyling {
    pub theme: ReportTheme,
    pub font_family: String,
    pub font_size: u32,
    pub header_styling: HeaderStyling,
    pub footer_styling: FooterStyling,
    pub color_palette: ColorPalette,
    pub page_layout: PageLayout,
    pub custom_css: Option<String>,
}

/// Report theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportTheme {
    Corporate,
    Modern,
    Minimal,
    Classic,
    Executive,
    Technical,
    Custom(String),
}

/// Header styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderStyling {
    pub logo_url: Option<String>,
    pub title_font_size: u32,
    pub subtitle_font_size: u32,
    pub background_color: String,
    pub text_color: String,
    pub height: u32,
}

/// Footer styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FooterStyling {
    pub include_page_numbers: bool,
    pub include_date: bool,
    pub include_company_info: bool,
    pub background_color: String,
    pub text_color: String,
    pub height: u32,
    pub custom_text: Option<String>,
}

/// Color palette for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
    pub success_color: String,
    pub warning_color: String,
    pub error_color: String,
}

/// Page layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLayout {
    pub page_size: PageSize,
    pub orientation: PageOrientation,
    pub margins: PageMargins,
    pub header_height: u32,
    pub footer_height: u32,
}

/// Page size options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
    A3,
    Tabloid,
    Custom { width: u32, height: u32 },
}

/// Page orientation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

/// Page margins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMargins {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

/// Generated report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub report_id: Uuid,
    pub report_name: String,
    pub report_type: ReportType,
    pub template_id: Option<Uuid>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub output_format: OutputFormat,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub generation_duration_ms: u64,
    pub status: ReportStatus,
    pub error_message: Option<String>,
    pub generated_by: Uuid,
    pub generated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub download_count: u32,
    pub last_accessed: Option<DateTime<Utc>>,
}

/// Report generation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportStatus {
    Pending,
    Generating,
    Completed,
    Failed,
    Expired,
    Archived,
}

/// Scheduled report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledReport {
    pub schedule_id: Uuid,
    pub report_definition: ReportDefinition,
    pub next_execution: DateTime<Utc>,
    pub last_execution: Option<DateTime<Utc>>,
    pub execution_count: u32,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Report generation task
#[derive(Debug, Clone)]
pub struct ReportGenerationTask {
    pub task_id: Uuid,
    pub report_definition: ReportDefinition,
    pub priority: TaskPriority,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
    pub timeout: Duration,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Report distribution manager
pub struct ReportDistributionManager {
    distribution_channels: HashMap<String, Box<dyn DistributionChannel + Send + Sync>>,
    distribution_queue: Arc<RwLock<std::collections::VecDeque<DistributionTask>>>,
}

/// Distribution task
#[derive(Debug, Clone)]
pub struct DistributionTask {
    pub task_id: Uuid,
    pub report_id: Uuid,
    pub distribution_method: ReportDistribution,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: DateTime<Utc>,
}

/// Distribution channel trait
pub trait DistributionChannel: Send + Sync {
    fn distribute_report(&self, report: &Report, distribution: &ReportDistribution) -> ReportingResult<()>;
    fn channel_name(&self) -> &str;
    fn is_available(&self) -> bool;
}

/// Email distribution channel
pub struct EmailDistributionChannel {
    smtp_config: SmtpConfig,
}

/// SMTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub tls_enabled: bool,
    pub from_address: String,
    pub from_name: String,
}

/// File system distribution channel
pub struct FileSystemDistributionChannel {
    base_path: PathBuf,
}

/// API distribution channel
pub struct ApiDistributionChannel {
    http_client: reqwest::Client,
}

/// Cloud storage distribution channel
pub struct CloudStorageDistributionChannel {
    provider: CloudProvider,
    credentials: CloudCredentials,
}

/// Cloud provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CloudProvider {
    AWS,
    Azure,
    GoogleCloud,
    Custom(String),
}

/// Cloud storage credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCredentials {
    pub access_key: String,
    pub secret_key: String,
    pub region: Option<String>,
    pub bucket: String,
}

impl ReportGenerationEngine {
    /// Create new report generation engine
    pub async fn new(
        database: Arc<DatabaseManager>,
        metrics_collector: Arc<MetricsCollector>,
        config: ReportGenerationConfig,
    ) -> ReportingResult<Self> {
        let distribution_manager = ReportDistributionManager::new().await?;
        
        let engine = Self {
            database: database.clone(),
            metrics_collector,
            report_templates: Arc::new(RwLock::new(HashMap::new())),
            generated_reports: Arc::new(RwLock::new(HashMap::new())),
            scheduled_reports: Arc::new(RwLock::new(HashMap::new())),
            distribution_manager,
            generation_queue: Arc::new(RwLock::new(std::collections::VecDeque::new())),
            config,
        };
        
        // Initialize database tables
        engine.initialize_tables().await?;
        
        // Load existing templates and schedules
        engine.load_existing_data().await?;
        
        // Start background tasks
        engine.start_background_tasks().await?;
        
        Ok(engine)
    }
    
    /// Initialize database tables
    async fn initialize_tables(&self) -> ReportingResult<()> {
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;
        
        // Report templates table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS report_templates (
                template_id UUID PRIMARY KEY,
                template_name TEXT NOT NULL,
                description TEXT NOT NULL,
                report_type TEXT NOT NULL,
                template_format TEXT NOT NULL,
                template_content TEXT NOT NULL,
                variables JSONB NOT NULL,
                sections JSONB NOT NULL,
                styling JSONB NOT NULL,
                created_by UUID NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                version TEXT NOT NULL,
                is_active BOOLEAN DEFAULT true
            )
        "#).execute(pool).await?;
        
        // Generated reports table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS generated_reports (
                report_id UUID PRIMARY KEY,
                report_name TEXT NOT NULL,
                report_type TEXT NOT NULL,
                template_id UUID,
                parameters JSONB NOT NULL,
                output_format TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_size_bytes BIGINT NOT NULL,
                generation_duration_ms BIGINT NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                generated_by UUID NOT NULL,
                generated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                expires_at TIMESTAMP WITH TIME ZONE,
                download_count INTEGER DEFAULT 0,
                last_accessed TIMESTAMP WITH TIME ZONE
            )
        "#).execute(pool).await?;
        
        // Scheduled reports table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS scheduled_reports (
                schedule_id UUID PRIMARY KEY,
                report_definition JSONB NOT NULL,
                next_execution TIMESTAMP WITH TIME ZONE NOT NULL,
                last_execution TIMESTAMP WITH TIME ZONE,
                execution_count INTEGER DEFAULT 0,
                is_active BOOLEAN DEFAULT true,
                created_by UUID NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
        "#).execute(pool).await?;
        
        // Report distribution log table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS report_distribution_log (
                distribution_id UUID PRIMARY KEY,
                report_id UUID NOT NULL,
                distribution_method TEXT NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                attempted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                completed_at TIMESTAMP WITH TIME ZONE,
                retry_count INTEGER DEFAULT 0,
                FOREIGN KEY (report_id) REFERENCES generated_reports(report_id)
            )
        "#).execute(pool).await?;
        
        Ok(())
    }
    
    /// Load existing data from database
    async fn load_existing_data(&self) -> ReportingResult<()> {
        // Load templates
        self.load_report_templates().await?;
        
        // Load scheduled reports
        self.load_scheduled_reports().await?;
        
        Ok(())
    }
    
    /// Load report templates from database
    async fn load_report_templates(&self) -> ReportingResult<()> {
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;
        
        let rows = sqlx::query(r#"
            SELECT template_id, template_name, description, report_type, template_format,
                   template_content, variables, sections, styling, created_by,
                   created_at, last_updated, version, is_active
            FROM report_templates
            WHERE is_active = true
        "#).fetch_all(pool).await?;
        
        let mut templates = self.report_templates.write().await;
        
        for row in rows {
            let template = ReportTemplate {
                template_id: row.get("template_id"),
                template_name: row.get("template_name"),
                description: row.get("description"),
                report_type: serde_json::from_str(&row.get::<String, _>("report_type"))?,
                template_format: serde_json::from_str(&row.get::<String, _>("template_format"))?,
                template_content: row.get("template_content"),
                variables: serde_json::from_value(row.get("variables"))?,
                sections: serde_json::from_value(row.get("sections"))?,
                styling: serde_json::from_value(row.get("styling"))?,
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                last_updated: row.get("last_updated"),
                version: row.get("version"),
                is_active: row.get("is_active"),
            };
            
            templates.insert(template.template_id, template);
        }
        
        Ok(())
    }
    
    /// Load scheduled reports from database
    async fn load_scheduled_reports(&self) -> ReportingResult<()> {
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;
        
        let rows = sqlx::query(r#"
            SELECT schedule_id, report_definition, next_execution, last_execution,
                   execution_count, is_active, created_by, created_at, last_updated
            FROM scheduled_reports
            WHERE is_active = true
        "#).fetch_all(pool).await?;
        
        let mut scheduled = self.scheduled_reports.write().await;
        
        for row in rows {
            let scheduled_report = ScheduledReport {
                schedule_id: row.get("schedule_id"),
                report_definition: serde_json::from_value(row.get("report_definition"))?,
                next_execution: row.get("next_execution"),
                last_execution: row.get("last_execution"),
                execution_count: row.get::<i32, _>("execution_count") as u32,
                is_active: row.get("is_active"),
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                last_updated: row.get("last_updated"),
            };
            
            scheduled.insert(scheduled_report.schedule_id, scheduled_report);
        }
        
        Ok(())
    }
    
    /// Start background processing tasks
    async fn start_background_tasks(&self) -> ReportingResult<()> {
        // Start report generation worker
        self.start_generation_worker().await;
        
        // Start scheduled report processor
        self.start_scheduled_report_processor().await;
        
        // Start distribution worker
        self.start_distribution_worker().await;
        
        // Start cleanup task
        self.start_cleanup_task().await;
        
        Ok(())
    }
    
    /// Generate report
    pub async fn generate_report(&self, report_request: ReportRequest) -> ReportingResult<Uuid> {
        // Create report generation task
        let task = ReportGenerationTask {
            task_id: Uuid::new_v4(),
            report_definition: report_request.definition,
            priority: report_request.priority.unwrap_or(TaskPriority::Normal),
            requested_by: report_request.requested_by,
            requested_at: Utc::now(),
            timeout: Duration::minutes(self.config.default_timeout_minutes as i64),
        };
        
        // Add to generation queue
        let mut queue = self.generation_queue.write().await;
        queue.push_back(task.clone());
        
        // Sort queue by priority
        let mut temp_vec: Vec<_> = queue.drain(..).collect();
        temp_vec.sort_by(|a, b| b.priority.cmp(&a.priority));
        queue.extend(temp_vec);
        
        Ok(task.task_id)
    }
    
    /// Create report template
    pub async fn create_template(&self, template: ReportTemplate) -> ReportingResult<Uuid> {
        // Validate template
        self.validate_template(&template).await?;
        
        // Store in database
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;
        
        sqlx::query(r#"
            INSERT INTO report_templates 
            (template_id, template_name, description, report_type, template_format,
             template_content, variables, sections, styling, created_by, version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#)
        .bind(&template.template_id)
        .bind(&template.template_name)
        .bind(&template.description)
        .bind(&format!("{:?}", template.report_type))
        .bind(&format!("{:?}", template.template_format))
        .bind(&template.template_content)
        .bind(serde_json::to_value(&template.variables)?)
        .bind(serde_json::to_value(&template.sections)?)
        .bind(serde_json::to_value(&template.styling)?)
        .bind(&template.created_by)
        .bind(&template.version)
        .execute(pool).await?;
        
        // Add to cache
        let mut templates = self.report_templates.write().await;
        let template_id = template.template_id;
        templates.insert(template_id, template);
        
        Ok(template_id)
    }
    
    /// Schedule report
    pub async fn schedule_report(&self, schedule: ScheduledReport) -> ReportingResult<Uuid> {
        // Store in database
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;
        
        sqlx::query(r#"
            INSERT INTO scheduled_reports 
            (schedule_id, report_definition, next_execution, created_by)
            VALUES ($1, $2, $3, $4)
        "#)
        .bind(&schedule.schedule_id)
        .bind(serde_json::to_value(&schedule.report_definition)?)
        .bind(&schedule.next_execution)
        .bind(&schedule.created_by)
        .execute(pool).await?;
        
        // Add to memory
        let mut scheduled = self.scheduled_reports.write().await;
        let schedule_id = schedule.schedule_id;
        scheduled.insert(schedule_id, schedule);
        
        Ok(schedule_id)
    }
    
    /// Get report status
    pub async fn get_report_status(&self, report_id: Uuid) -> ReportingResult<ReportStatus> {
        let reports = self.generated_reports.read().await;
        
        if let Some(report) = reports.get(&report_id) {
            Ok(report.status.clone())
        } else {
            // Check if it's still in queue
            let queue = self.generation_queue.read().await;
            if queue.iter().any(|task| task.task_id == report_id) {
                Ok(ReportStatus::Pending)
            } else {
                Err(ReportingError::DataSourceNotFound(
                    format!("Report {}", report_id) 
                ))
            }
        }
    }
    
    /// Get generated report
    pub async fn get_report(&self, report_id: Uuid) -> ReportingResult<Report> {
        let reports = self.generated_reports.read().await;
        
        reports.get(&report_id)
            .cloned()
            .ok_or_else(|| ReportingError::DataSourceNotFound(
                format!("Report {}", report_id) 
            ))
    }
    
    /// Download report file
    pub async fn download_report(&self, report_id: Uuid) -> ReportingResult<Vec<u8>> {
        let report = self.get_report(report_id).await?;
        
        // Read file from disk
        let file_content = tokio::fs::read(&report.file_path).await
            .map_err(|e| ReportingError::IoError(e))?;
        
        // Update download count
        self.update_download_count(report_id).await?;
        
        Ok(file_content)
    }
    
    /// Generate report from task
    async fn generate_report_from_task(&self, task: ReportGenerationTask) -> ReportingResult<Report> {
        let start_time = Utc::now();
        
        // Get template if specified
        let template = if let Some(template_id) = task.report_definition.template_id {
            let templates = self.report_templates.read().await;
            templates.get(&template_id).cloned()
        } else {
            None
        };
        
        // Generate report content
        let content = self.generate_report_content(&task.report_definition, template.as_ref()).await?;
        
        // Convert to output format
        let output_data = self.convert_to_output_format(&content, &task.report_definition.output_formats[0]).await?;
        
        // Save to file
        let file_path = self.save_report_file(&task.task_id, &task.report_definition.output_formats[0], &output_data).await?;
        
        let generation_duration = Utc::now().signed_duration_since(start_time);
        
        let report = Report {
            report_id: task.task_id,
            report_name: task.report_definition.name,
            report_type: task.report_definition.report_type,
            template_id: task.report_definition.template_id,
            parameters: task.report_definition.parameters,
            output_format: task.report_definition.output_formats[0].clone(),
            file_path,
            file_size_bytes: output_data.len() as u64,
            generation_duration_ms: generation_duration.num_milliseconds() as u64,
            status: ReportStatus::Completed,
            error_message: None,
            generated_by: task.requested_by,
            generated_at: Utc::now(),
            expires_at: None, // Would calculate based on retention policy
            download_count: 0,
            last_accessed: None,
        };
        
        // Store report in database
        self.store_generated_report(&report).await?;
        
        // Add to memory cache
        let mut reports = self.generated_reports.write().await;
        reports.insert(report.report_id, report.clone());
        
        // Update metrics
        let mut tags = HashMap::new();
        tags.insert("type".to_string(), format!("{:?}", report.report_type));
        tags.insert("format".to_string(), format!("{:?}", report.output_format));
        self.metrics_collector.increment_counter("reports_generated", tags).await?;
        
        Ok(report)
    }
    
    /// Generate report content
    async fn generate_report_content(
        &self,
        definition: &ReportDefinition,
        template: Option<&ReportTemplate>,
    ) -> ReportingResult<ReportContent> {
        // Simplified implementation - would use actual template engine
        let mut sections = Vec::new();
        
        // Generate executive summary
        sections.push(ReportContentSection {
            section_name: "Executive Summary".to_string(),
            content: "This report provides an overview of virtual world analytics...".to_string(),
        });
        
        // Generate data analysis section
        sections.push(ReportContentSection {
            section_name: "Data Analysis".to_string(),
            content: "Key metrics show positive trends in user engagement...".to_string(),
        });
        
        Ok(ReportContent {
            title: definition.name.clone(),
            subtitle: definition.description.clone(),
            generated_at: Utc::now(),
            sections,
            metadata: definition.parameters.clone(),
        })
    }
    
    /// Convert content to output format
    async fn convert_to_output_format(
        &self,
        content: &ReportContent,
        format: &OutputFormat,
    ) -> ReportingResult<Vec<u8>> {
        match format {
            OutputFormat::PDF => self.generate_pdf(content).await,
            OutputFormat::HTML => self.generate_html(content).await,
            OutputFormat::Excel => self.generate_excel(content).await,
            OutputFormat::CSV => self.generate_csv(content).await,
            OutputFormat::JSON => self.generate_json(content).await,
            _ => Err(ReportingError::GenerationFailed {
                reason: format!("Unsupported output format: {:?}", format)
            })
        }
    }
    
    /// Generate PDF output
    async fn generate_pdf(&self, content: &ReportContent) -> ReportingResult<Vec<u8>> {
        // Simplified implementation - would use actual PDF generation library
        let pdf_content = format!(
            "PDF Report: {}\n\nGenerated: {}\n\n{}",
            content.title,
            content.generated_at.format("%Y-%m-%d %H:%M:%S"),
            content.sections.iter()
                .map(|s| format!("{}\n{}\n", s.section_name, s.content))
                .collect::<String>()
        );
        
        Ok(pdf_content.into_bytes())
    }
    
    /// Generate HTML output
    async fn generate_html(&self, content: &ReportContent) -> ReportingResult<Vec<u8>> {
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #333; }}
        h2 {{ color: #666; }}
        .generated-date {{ color: #999; font-style: italic; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p class="generated-date">Generated: {}</p>
    {}
</body>
</html>"#,
            content.title,
            content.title,
            content.generated_at.format("%Y-%m-%d %H:%M:%S"),
            content.sections.iter()
                .map(|s| format!("<h2>{}</h2><p>{}</p>", s.section_name, s.content))
                .collect::<String>()
        );
        
        Ok(html_content.into_bytes())
    }
    
    /// Generate Excel output
    async fn generate_excel(&self, content: &ReportContent) -> ReportingResult<Vec<u8>> {
        // Simplified implementation - would use actual Excel generation library
        let excel_content = format!(
            "Excel Report: {}\nGenerated: {}\n\n{}",
            content.title,
            content.generated_at.format("%Y-%m-%d %H:%M:%S"),
            content.sections.iter()
                .map(|s| format!("{}\t{}\n", s.section_name, s.content))
                .collect::<String>()
        );
        
        Ok(excel_content.into_bytes())
    }
    
    /// Generate CSV output
    async fn generate_csv(&self, content: &ReportContent) -> ReportingResult<Vec<u8>> {
        let csv_content = format!(
            "Section,Content\n{}",
            content.sections.iter()
                .map(|s| format!("\"{}\",\"{}\"", s.section_name, s.content))
                .collect::<Vec<_>>()
                .join("\n")
        );
        
        Ok(csv_content.into_bytes())
    }
    
    /// Generate JSON output
    async fn generate_json(&self, content: &ReportContent) -> ReportingResult<Vec<u8>> {
        let json_content = serde_json::to_string_pretty(content)?;
        Ok(json_content.into_bytes())
    }
    
    /// Save report file to disk
    async fn save_report_file(
        &self,
        report_id: &Uuid,
        format: &OutputFormat,
        data: &[u8],
    ) -> ReportingResult<String> {
        let file_extension = match format {
            OutputFormat::PDF => "pdf",
            OutputFormat::HTML => "html",
            OutputFormat::Excel => "xlsx",
            OutputFormat::CSV => "csv",
            OutputFormat::JSON => "json",
            _ => "bin",
        };
        
        let file_name = format!("report_{}_{}.{}", 
            report_id, 
            Utc::now().format("%Y%m%d_%H%M%S"),
            file_extension
        );
        
        let file_path = format!("{}/{}", self.config.output_directory, file_name);
        
        // Ensure output directory exists
        tokio::fs::create_dir_all(&self.config.output_directory).await
            .map_err(|e| ReportingError::IoError(e))?;
        
        // Write file
        tokio::fs::write(&file_path, data).await
            .map_err(|e| ReportingError::IoError(e))?;
        
        Ok(file_path)
    }
    
    /// Store generated report in database
    async fn store_generated_report(&self, report: &Report) -> ReportingResult<()> {
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;
        
        sqlx::query(r#"
            INSERT INTO generated_reports 
            (report_id, report_name, report_type, template_id, parameters, output_format,
             file_path, file_size_bytes, generation_duration_ms, status, generated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#)
        .bind(&report.report_id)
        .bind(&report.report_name)
        .bind(&format!("{:?}", report.report_type))
        .bind(&report.template_id)
        .bind(serde_json::to_value(&report.parameters)?)
        .bind(&format!("{:?}", report.output_format))
        .bind(&report.file_path)
        .bind(report.file_size_bytes as i64)
        .bind(report.generation_duration_ms as i64)
        .bind(&format!("{:?}", report.status))
        .bind(&report.generated_by)
        .execute(pool).await?;
        
        Ok(())
    }
    
    /// Update download count
    async fn update_download_count(&self, report_id: Uuid) -> ReportingResult<()> {
        let pool_ref = self.database.get_pool()?;
        let pool = pool_ref.as_postgres_pool()?;
        
        sqlx::query(r#"
            UPDATE generated_reports 
            SET download_count = download_count + 1, last_accessed = NOW()
            WHERE report_id = $1
        "#)
        .bind(&report_id)
        .execute(pool).await?;
        
        // Update in memory cache
        let mut reports = self.generated_reports.write().await;
        if let Some(report) = reports.get_mut(&report_id) {
            report.download_count += 1;
            report.last_accessed = Some(Utc::now());
        }
        
        Ok(())
    }
    
    /// Validate template
    async fn validate_template(&self, template: &ReportTemplate) -> ReportingResult<()> {
        // Validate template content
        if template.template_content.is_empty() {
            return Err(ReportingError::InvalidConfiguration {
                reason: "Template content cannot be empty".to_string()
            });
        }
        
        // Validate variables
        for variable in &template.variables {
            if variable.variable_name.is_empty() {
                return Err(ReportingError::InvalidConfiguration {
                    reason: "Variable name cannot be empty".to_string()
                });
            }
        }
        
        // Validate sections
        for section in &template.sections {
            if section.section_name.is_empty() {
                return Err(ReportingError::InvalidConfiguration {
                    reason: "Section name cannot be empty".to_string()
                });
            }
        }
        
        Ok(())
    }
    
    /// Start report generation worker
    async fn start_generation_worker(&self) {
        let queue = self.generation_queue.clone();
        let engine = self.clone_for_worker();
        
        tokio::spawn(async move {
            loop {
                // Check for tasks in queue
                let task = {
                    let mut queue_guard = queue.write().await;
                    queue_guard.pop_front()
                };
                
                if let Some(task) = task {
                    // Generate report
                    match engine.generate_report_from_task(task.clone()).await {
                        Ok(report) => {
                            tracing::info!("Report generated successfully: {}", report.report_id);
                        },
                        Err(e) => {
                            tracing::error!("Failed to generate report {}: {}", task.task_id, e);
                        }
                    }
                } else {
                    // No tasks, wait before checking again
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });
    }
    
    /// Start scheduled report processor
    async fn start_scheduled_report_processor(&self) {
        let scheduled_reports = self.scheduled_reports.clone();
        let generation_queue = self.generation_queue.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // Check every minute
            
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                let mut scheduled_guard = scheduled_reports.write().await;
                
                for (_, scheduled_report) in scheduled_guard.iter_mut() {
                    if scheduled_report.is_active && scheduled_report.next_execution <= now {
                        // Create generation task
                        let task = ReportGenerationTask {
                            task_id: Uuid::new_v4(),
                            report_definition: scheduled_report.report_definition.clone(),
                            priority: TaskPriority::Normal,
                            requested_by: scheduled_report.created_by,
                            requested_at: now,
                            timeout: Duration::minutes(30),
                        };
                        
                        // Add to queue
                        let mut queue_guard = generation_queue.write().await;
                        queue_guard.push_back(task);
                        
                        // Update next execution time
                        scheduled_report.last_execution = Some(now);
                        scheduled_report.execution_count += 1;
                        scheduled_report.next_execution = Self::calculate_next_execution(
                            &scheduled_report.report_definition.schedule.as_ref().unwrap().frequency,
                            now
                        );
                    }
                }
            }
        });
    }
    
    /// Start distribution worker
    async fn start_distribution_worker(&self) {
        tokio::spawn(async move {
            // Distribution worker implementation would go here
            tracing::info!("Distribution worker started");
        });
    }
    
    /// Start cleanup task
    async fn start_cleanup_task(&self) {
        let reports = self.generated_reports.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Hourly
            
            loop {
                interval.tick().await;
                
                // Clean up expired reports
                let now = Utc::now();
                let mut reports_guard = reports.write().await;
                
                reports_guard.retain(|_, report| {
                    if let Some(expires_at) = report.expires_at {
                        expires_at > now
                    } else {
                        true
                    }
                });
            }
        });
    }
    
    /// Calculate next execution time
    fn calculate_next_execution(frequency: &ReportingFrequency, from: DateTime<Utc>) -> DateTime<Utc> {
        match frequency {
            ReportingFrequency::Hourly => from + Duration::hours(1),
            ReportingFrequency::Daily => from + Duration::days(1),
            ReportingFrequency::Weekly => from + Duration::weeks(1),
            ReportingFrequency::Monthly => from + Duration::days(30), // Simplified
            ReportingFrequency::Quarterly => from + Duration::days(90),
            ReportingFrequency::Yearly => from + Duration::days(365),
            ReportingFrequency::Custom(duration) => from + *duration,
            _ => from + Duration::hours(1),
        }
    }
    
    /// Clone engine for worker tasks
    fn clone_for_worker(&self) -> ReportGenerationEngine {
        // This is a simplified clone - in practice would need proper Arc handling
        self.clone()
    }
}

/// Report request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    pub definition: ReportDefinition,
    pub priority: Option<TaskPriority>,
    pub requested_by: Uuid,
}

/// Report content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportContent {
    pub title: String,
    pub subtitle: String,
    pub generated_at: DateTime<Utc>,
    pub sections: Vec<ReportContentSection>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Report content section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportContentSection {
    pub section_name: String,
    pub content: String,
}

// Implementation stubs for distribution channels
impl ReportDistributionManager {
    async fn new() -> ReportingResult<Self> {
        let mut distribution_channels: HashMap<String, Box<dyn DistributionChannel + Send + Sync>> = HashMap::new();
        
        // Add default distribution channels
        distribution_channels.insert(
            "email".to_string(),
            Box::new(EmailDistributionChannel::new()?)
        );
        distribution_channels.insert(
            "filesystem".to_string(),
            Box::new(FileSystemDistributionChannel::new()?)
        );
        
        Ok(Self {
            distribution_channels,
            distribution_queue: Arc::new(RwLock::new(std::collections::VecDeque::new())),
        })
    }
}

impl EmailDistributionChannel {
    fn new() -> ReportingResult<Self> {
        Ok(Self {
            smtp_config: SmtpConfig {
                host: "localhost".to_string(),
                port: 587,
                username: "".to_string(),
                password: "".to_string(),
                tls_enabled: true,
                from_address: "reports@opensim.org".to_string(),
                from_name: "OpenSim Reports".to_string(),
            },
        })
    }
}

impl DistributionChannel for EmailDistributionChannel {
    fn distribute_report(&self, _report: &Report, _distribution: &ReportDistribution) -> ReportingResult<()> {
        // Would implement actual email sending
        Ok(())
    }
    
    fn channel_name(&self) -> &str {
        "email"
    }
    
    fn is_available(&self) -> bool {
        true
    }
}

impl FileSystemDistributionChannel {
    fn new() -> ReportingResult<Self> {
        Ok(Self {
            base_path: PathBuf::from("/var/reports"),
        })
    }
}

impl DistributionChannel for FileSystemDistributionChannel {
    fn distribute_report(&self, _report: &Report, _distribution: &ReportDistribution) -> ReportingResult<()> {
        // Would implement file system distribution
        Ok(())
    }
    
    fn channel_name(&self) -> &str {
        "filesystem"
    }
    
    fn is_available(&self) -> bool {
        true
    }
}

// Simplified Clone implementation for the engine
impl Clone for ReportGenerationEngine {
    fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            metrics_collector: self.metrics_collector.clone(),
            report_templates: self.report_templates.clone(),
            generated_reports: self.generated_reports.clone(),
            scheduled_reports: self.scheduled_reports.clone(),
            distribution_manager: ReportDistributionManager {
                distribution_channels: HashMap::new(),
                distribution_queue: Arc::new(RwLock::new(std::collections::VecDeque::new())),
            }, // Create new instance
            generation_queue: self.generation_queue.clone(),
            config: self.config.clone(),
        }
    }
}

impl Default for ReportGenerationConfig {
    fn default() -> Self {
        Self {
            max_concurrent_generations: 5,
            default_timeout_minutes: 30,
            template_cache_size: 100,
            output_directory: "/var/opensim/reports".to_string(),
            temp_directory: "/tmp/opensim_reports".to_string(),
            max_report_size_mb: 100,
            retention_policy: ReportRetentionPolicy {
                daily_reports_days: 30,
                weekly_reports_weeks: 12,
                monthly_reports_months: 24,
                annual_reports_years: 5,
                archive_to_cold_storage: false,
                cold_storage_path: None,
            },
            compression_enabled: true,
            watermark_enabled: true,
            security_enabled: true,
        }
    }
}