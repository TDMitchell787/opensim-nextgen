//! Content Management System Orchestrator for OpenSim Next
//!
//! Provides the main interface for all content management operations,
//! coordinating creation, distribution, security, and analytics.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::{
    ContentType, ContentQuality, ContentMetadata, ContentPermissions,
    ContentValidationResult, ContentResult, ContentError, DistributionStrategy,
    ContentAnalytics, ContentSearchFilter, ContentSearchResult,
};
use super::creation::{ContentCreationManager, ContentCreationConfig, ContentCreationOptions};
use super::distribution::{ContentDistributionManager, DistributionConfig, DistributionNode};

/// Content management system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentManagerConfig {
    /// Content creation configuration
    pub creation_config: ContentCreationConfig,
    /// Content distribution configuration
    pub distribution_config: DistributionConfig,
    /// Content storage configuration
    pub storage_config: ContentStorageConfig,
    /// Security configuration
    pub security_config: ContentSecurityConfig,
    /// Analytics configuration
    pub analytics_config: ContentAnalyticsConfig,
}

/// Content storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStorageConfig {
    /// Primary storage directory
    pub storage_directory: PathBuf,
    /// Backup storage directory
    pub backup_directory: Option<PathBuf>,
    /// Maximum storage size (bytes)
    pub max_storage_size: u64,
    /// Enable automatic cleanup
    pub auto_cleanup: bool,
    /// Storage cleanup threshold (percentage)
    pub cleanup_threshold: f32,
    /// Enable storage encryption
    pub enable_encryption: bool,
    /// Storage replication factor
    pub replication_factor: u8,
}

/// Content security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSecurityConfig {
    /// Enable DRM protection
    pub enable_drm: bool,
    /// Enable content scanning
    pub enable_scanning: bool,
    /// Enable access logging
    pub enable_access_logging: bool,
    /// Maximum failed access attempts
    pub max_failed_attempts: u32,
    /// Content encryption key rotation interval (hours)
    pub key_rotation_interval: u32,
    /// Enable content watermarking
    pub enable_watermarking: bool,
}

/// Content analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalyticsConfig {
    /// Enable usage tracking
    pub enable_usage_tracking: bool,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Analytics data retention period (days)
    pub retention_period: u32,
    /// Enable real-time analytics
    pub enable_realtime_analytics: bool,
    /// Analytics sampling rate (0.0 to 1.0)
    pub sampling_rate: f32,
}

/// Content operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentOperationRequest {
    pub operation_type: ContentOperationType,
    pub content_id: Option<Uuid>,
    pub user_id: Uuid,
    pub parameters: HashMap<String, serde_json::Value>,
    pub priority: OperationPriority,
}

/// Content operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentOperationType {
    Create,
    Update,
    Delete,
    Download,
    Distribute,
    Validate,
    Search,
    Analyze,
}

/// Operation priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Content operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentOperationResult {
    pub operation_id: Uuid,
    pub operation_type: ContentOperationType,
    pub status: OperationStatus,
    pub result_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub processing_time_ms: Option<u32>,
}

/// Operation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Content usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentUsageStats {
    pub content_id: Uuid,
    pub total_downloads: u64,
    pub unique_users: u32,
    pub last_accessed: DateTime<Utc>,
    pub average_rating: f32,
    pub usage_by_region: HashMap<Uuid, u64>,
    pub usage_by_day: HashMap<String, u64>,
    pub bandwidth_usage: u64,
}

/// Content health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHealthStatus {
    pub content_id: Uuid,
    pub health_score: f32,
    pub availability_percentage: f32,
    pub sync_status: HashMap<Uuid, String>, // region_id -> status
    pub last_health_check: DateTime<Utc>,
    pub issues: Vec<ContentHealthIssue>,
}

/// Content health issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHealthIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub affected_regions: Vec<Uuid>,
    pub first_detected: DateTime<Utc>,
    pub suggested_action: String,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Main content management system
pub struct ContentManager {
    config: ContentManagerConfig,
    creation_manager: Arc<RwLock<ContentCreationManager>>,
    distribution_manager: Arc<RwLock<ContentDistributionManager>>,
    content_metadata: Arc<RwLock<HashMap<Uuid, ContentMetadata>>>,
    operation_history: Arc<RwLock<HashMap<Uuid, ContentOperationResult>>>,
    usage_stats: Arc<RwLock<HashMap<Uuid, ContentUsageStats>>>,
    health_status: Arc<RwLock<HashMap<Uuid, ContentHealthStatus>>>,
}

impl ContentManager {
    /// Create a new content manager
    pub async fn new(config: ContentManagerConfig) -> ContentResult<Self> {
        // Initialize sub-managers
        let creation_manager = ContentCreationManager::new(config.creation_config.clone())?;
        let distribution_manager = ContentDistributionManager::new(config.distribution_config.clone());
        
        // Ensure storage directories exist
        std::fs::create_dir_all(&config.storage_config.storage_directory)?;
        if let Some(backup_dir) = &config.storage_config.backup_directory {
            std::fs::create_dir_all(backup_dir)?;
        }
        
        Ok(Self {
            config,
            creation_manager: Arc::new(RwLock::new(creation_manager)),
            distribution_manager: Arc::new(RwLock::new(distribution_manager)),
            content_metadata: Arc::new(RwLock::new(HashMap::new())),
            operation_history: Arc::new(RwLock::new(HashMap::new())),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Create new content
    pub async fn create_content(
        &mut self,
        creator_id: Uuid,
        content_name: String,
        content_type: ContentType,
        source_file: PathBuf,
        permissions: Option<ContentPermissions>,
        options: ContentCreationOptions,
    ) -> ContentResult<Uuid> {
        let operation_id = Uuid::new_v4();
        let start_time = Utc::now();
        
        // Start creation job
        let job_id = self.creation_manager.write().await.create_content(
            creator_id,
            content_name.clone(),
            content_type.clone(),
            source_file,
            permissions,
            options,
        ).await?;
        
        // Record operation
        let operation_result = ContentOperationResult {
            operation_id,
            operation_type: ContentOperationType::Create,
            status: OperationStatus::Processing,
            result_data: Some(serde_json::json!({ "job_id": job_id })),
            error_message: None,
            started_at: start_time,
            completed_at: None,
            processing_time_ms: None,
        };
        
        self.operation_history.write().await.insert(operation_id, operation_result);
        
        tracing::info!("Content creation initiated: {} (job: {})", content_name, job_id);
        Ok(job_id)
    }
    
    /// Distribute content to regions
    pub async fn distribute_content(
        &mut self,
        content_id: Uuid,
        target_regions: Option<Vec<Uuid>>,
        strategy: Option<DistributionStrategy>,
    ) -> ContentResult<Uuid> {
        // Get content metadata
        let metadata = self.content_metadata.read().await
            .get(&content_id)
            .cloned()
            .ok_or(ContentError::ContentNotFound { id: content_id })?;
        
        // Start distribution
        let job_id = self.distribution_manager.write().await.distribute_content(
            metadata,
            target_regions,
            strategy,
        ).await?;
        
        tracing::info!("Content distribution initiated: {} (job: {})", content_id, job_id);
        Ok(job_id)
    }
    
    /// Validate content
    pub async fn validate_content(
        &self,
        content_id: Uuid,
    ) -> ContentResult<ContentValidationResult> {
        let metadata = self.content_metadata.read().await
            .get(&content_id)
            .cloned()
            .ok_or(ContentError::ContentNotFound { id: content_id })?;
        
        // Get content file path
        let file_path = self.get_content_file_path(&content_id)?;
        
        // Perform validation
        let result = self.creation_manager.read().await.validate_content(
            &metadata.content_type,
            &file_path,
            Some(&metadata),
        ).await?;
        
        tracing::info!("Content validation completed: {} (valid: {})", content_id, result.is_valid);
        Ok(result)
    }
    
    /// Search content
    pub async fn search_content(
        &self,
        filter: ContentSearchFilter,
        page: u32,
        page_size: u32,
    ) -> ContentResult<ContentSearchResult> {
        let start_time = std::time::Instant::now();
        let all_content = self.content_metadata.read().await;
        let mut results = Vec::new();
        
        // Apply filters
        for metadata in all_content.values() {
            if self.matches_filter(metadata, &filter) {
                results.push(metadata.clone());
            }
        }
        
        // Sort results (by creation date, newest first)
        results.sort_by(|a, b| b.creation_date.cmp(&a.creation_date));
        
        // Paginate
        let total_count = results.len() as u32;
        let start_idx = (page * page_size) as usize;
        let end_idx = ((page + 1) * page_size) as usize;
        
        let paginated_results = if start_idx < results.len() {
            results[start_idx..end_idx.min(results.len())].to_vec()
        } else {
            Vec::new()
        };
        
        // Calculate facets (simplified)
        let mut facets = HashMap::new();
        let mut type_facets = HashMap::new();
        for metadata in &results {
            let type_name = format!("{:?}", metadata.content_type);
            *type_facets.entry(type_name).or_insert(0) += 1;
        }
        facets.insert("content_type".to_string(), type_facets);
        
        let search_time = start_time.elapsed().as_millis() as u32;
        
        let result = ContentSearchResult {
            results: paginated_results,
            total_count,
            page,
            page_size,
            search_time_ms: search_time,
            facets,
        };
        
        tracing::info!("Content search completed: {} results in {}ms", total_count, search_time);
        Ok(result)
    }
    
    /// Get content metadata
    pub async fn get_content_metadata(&self, content_id: Uuid) -> ContentResult<ContentMetadata> {
        self.content_metadata.read().await
            .get(&content_id)
            .cloned()
            .ok_or(ContentError::ContentNotFound { id: content_id })
    }
    
    /// Update content metadata
    pub async fn update_content_metadata(
        &mut self,
        content_id: Uuid,
        updates: HashMap<String, serde_json::Value>,
    ) -> ContentResult<()> {
        let mut metadata_map = self.content_metadata.write().await;
        if let Some(metadata) = metadata_map.get_mut(&content_id) {
            // Apply updates (simplified implementation)
            if let Some(name) = updates.get("name") {
                metadata.name = name.as_str().unwrap_or(&metadata.name).to_string();
            }
            if let Some(description) = updates.get("description") {
                metadata.description = description.as_str().unwrap_or(&metadata.description).to_string();
            }
            metadata.last_modified = Utc::now();
            
            tracing::info!("Content metadata updated: {}", content_id);
            Ok(())
        } else {
            Err(ContentError::ContentNotFound { id: content_id })
        }
    }
    
    /// Delete content
    pub async fn delete_content(&mut self, content_id: Uuid) -> ContentResult<()> {
        // Remove metadata
        if self.content_metadata.write().await.remove(&content_id).is_none() {
            return Err(ContentError::ContentNotFound { id: content_id });
        }
        
        // Remove usage stats and health status
        self.usage_stats.write().await.remove(&content_id);
        self.health_status.write().await.remove(&content_id);
        
        // Delete actual files (stub implementation)
        let file_path = self.get_content_file_path(&content_id)?;
        if file_path.exists() {
            tokio::fs::remove_file(&file_path).await?;
        }
        
        tracing::info!("Content deleted: {}", content_id);
        Ok(())
    }
    
    /// Get content usage statistics
    pub async fn get_usage_stats(&self, content_id: Uuid) -> ContentResult<ContentUsageStats> {
        self.usage_stats.read().await
            .get(&content_id)
            .cloned()
            .ok_or(ContentError::ContentNotFound { id: content_id })
    }
    
    /// Get content health status
    pub async fn get_health_status(&self, content_id: Uuid) -> ContentResult<ContentHealthStatus> {
        self.health_status.read().await
            .get(&content_id)
            .cloned()
            .ok_or(ContentError::ContentNotFound { id: content_id })
    }
    
    /// Register distribution node
    pub async fn register_distribution_node(&mut self, node: DistributionNode) -> ContentResult<()> {
        self.distribution_manager.write().await.register_node(node).await
    }
    
    /// Get system analytics
    pub async fn get_system_analytics(&self) -> ContentResult<ContentAnalytics> {
        let metadata_map = self.content_metadata.read().await;
        let usage_map = self.usage_stats.read().await;
        
        let total_content = metadata_map.len() as u64;
        let total_downloads = usage_map.values().map(|stats| stats.total_downloads).sum();
        let total_users = usage_map.values().map(|stats| stats.unique_users).sum::<u32>() as u64;
        
        // Calculate performance metrics (simplified)
        let performance_metrics = super::ContentPerformanceMetrics {
            load_time_avg: 2.5,
            render_performance: 85.0,
            memory_usage: 512 * 1024 * 1024, // 512 MB
            cache_hit_ratio: 0.85,
            error_rate: 0.02,
        };
        
        Ok(ContentAnalytics {
            content_id: Uuid::nil(), // System-wide stats
            total_downloads,
            unique_users: total_users as u32,
            average_rating: 4.2, // Would calculate from actual ratings
            usage_by_region: HashMap::new(), // Would aggregate from usage stats
            performance_metrics,
            revenue_data: None, // Would include if marketplace is enabled
        })
    }
    
    /// Perform system maintenance
    pub async fn perform_maintenance(&mut self) -> ContentResult<()> {
        tracing::info!("Starting content system maintenance");
        
        // Clean up old operations
        self.cleanup_old_operations().await?;
        
        // Update health status for all content
        self.update_all_health_status().await?;
        
        // Optimize storage
        if self.config.storage_config.auto_cleanup {
            self.optimize_storage().await?;
        }
        
        // Update distribution routes
        self.distribution_manager.write().await.optimize_distribution_routes().await?;
        
        tracing::info!("Content system maintenance completed");
        Ok(())
    }
    
    // Private helper methods
    
    fn matches_filter(&self, metadata: &ContentMetadata, filter: &ContentSearchFilter) -> bool {
        // Content type filter
        if let Some(types) = &filter.content_types {
            if !types.contains(&metadata.content_type) {
                return false;
            }
        }
        
        // Category filter
        if let Some(categories) = &filter.categories {
            if !categories.iter().any(|cat| metadata.categories.contains(cat)) {
                return false;
            }
        }
        
        // Tags filter
        if let Some(tags) = &filter.tags {
            if !tags.iter().any(|tag| metadata.tags.contains(tag)) {
                return false;
            }
        }
        
        // Creator filter
        if let Some(creator_ids) = &filter.creator_ids {
            if !creator_ids.contains(&metadata.creator_id) {
                return false;
            }
        }
        
        // Rating filter
        if let Some(min_rating) = filter.min_rating {
            if metadata.rating.average_rating < min_rating {
                return false;
            }
        }
        
        // Price filter
        if let Some(max_price) = filter.max_price {
            if let Some(price) = &metadata.price {
                if price.amount > max_price {
                    return false;
                }
            }
        }
        
        // Date filters
        if let Some(after) = filter.created_after {
            if metadata.creation_date < after {
                return false;
            }
        }
        
        if let Some(before) = filter.created_before {
            if metadata.creation_date > before {
                return false;
            }
        }
        
        // DRM filter
        if let Some(has_drm) = filter.has_drm {
            if metadata.drm_protection != has_drm {
                return false;
            }
        }
        
        // Marketplace filter
        if let Some(marketplace_only) = filter.marketplace_only {
            if metadata.marketplace_listed != marketplace_only {
                return false;
            }
        }
        
        true
    }
    
    fn get_content_file_path(&self, content_id: &Uuid) -> ContentResult<PathBuf> {
        Ok(self.config.storage_config.storage_directory.join(format!("{}.content", content_id)))
    }
    
    async fn cleanup_old_operations(&mut self) -> ContentResult<()> {
        let cutoff_date = Utc::now() - chrono::Duration::days(30);
        let mut operations = self.operation_history.write().await;
        
        operations.retain(|_, op| {
            op.started_at > cutoff_date || op.status == OperationStatus::Processing
        });
        
        Ok(())
    }
    
    async fn update_all_health_status(&mut self) -> ContentResult<()> {
        let content_ids: Vec<Uuid> = self.content_metadata.read().await.keys().cloned().collect();
        
        for content_id in content_ids {
            self.update_content_health_status(content_id).await?;
        }
        
        Ok(())
    }
    
    async fn update_content_health_status(&mut self, content_id: Uuid) -> ContentResult<()> {
        let mut health_status = ContentHealthStatus {
            content_id,
            health_score: 100.0,
            availability_percentage: 100.0,
            sync_status: HashMap::new(),
            last_health_check: Utc::now(),
            issues: Vec::new(),
        };

        let file_path = self.get_content_file_path(&content_id)?;
        if !file_path.exists() {
            health_status.health_score -= 50.0;
            health_status.issues.push(ContentHealthIssue {
                severity: IssueSeverity::Critical,
                description: "Content file not found".to_string(),
                affected_regions: Vec::new(),
                first_detected: Utc::now(),
                suggested_action: "Restore content file from backup".to_string(),
            });
        } else {
            if let Ok(metadata) = tokio::fs::metadata(&file_path).await {
                if metadata.len() == 0 {
                    health_status.health_score -= 25.0;
                    health_status.issues.push(ContentHealthIssue {
                        severity: IssueSeverity::Error,
                        description: "Content file is empty".to_string(),
                        affected_regions: Vec::new(),
                        first_detected: Utc::now(),
                        suggested_action: "Re-upload content or restore from backup".to_string(),
                    });
                }

                if let Ok(modified) = metadata.modified() {
                    let age = std::time::SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or_default();
                    if age > std::time::Duration::from_secs(365 * 24 * 60 * 60) {
                        health_status.issues.push(ContentHealthIssue {
                            severity: IssueSeverity::Info,
                            description: "Content has not been modified in over a year".to_string(),
                            affected_regions: Vec::new(),
                            first_detected: Utc::now(),
                            suggested_action: "Consider reviewing content for updates".to_string(),
                        });
                    }
                }
            }
        }

        let version_records = self.distribution_manager.read().await
            .check_content_versions(content_id).await
            .unwrap_or_default();

        let mut synchronized_count = 0u32;
        let mut failed_regions = Vec::new();
        let mut outdated_regions = Vec::new();

        for (region_id, record) in &version_records {
            let status_str = match record.sync_status {
                super::distribution::SyncStatus::Synchronized => {
                    synchronized_count += 1;
                    "synchronized"
                },
                super::distribution::SyncStatus::OutOfDate => {
                    outdated_regions.push(*region_id);
                    "outdated"
                },
                super::distribution::SyncStatus::Synchronizing => "synchronizing",
                super::distribution::SyncStatus::Failed => {
                    failed_regions.push(*region_id);
                    "failed"
                },
                super::distribution::SyncStatus::NotCached => "not_cached",
            };
            health_status.sync_status.insert(*region_id, status_str.to_string());
        }

        let total_regions = version_records.len() as f32;
        if total_regions > 0.0 {
            health_status.availability_percentage = (synchronized_count as f32 / total_regions) * 100.0;

            if !failed_regions.is_empty() {
                let penalty = (failed_regions.len() as f32 / total_regions) * 30.0;
                health_status.health_score -= penalty;
                health_status.issues.push(ContentHealthIssue {
                    severity: IssueSeverity::Error,
                    description: format!("Content sync failed for {} region(s)", failed_regions.len()),
                    affected_regions: failed_regions.clone(),
                    first_detected: Utc::now(),
                    suggested_action: "Retry synchronization or check region connectivity".to_string(),
                });
            }

            if !outdated_regions.is_empty() {
                let penalty = (outdated_regions.len() as f32 / total_regions) * 15.0;
                health_status.health_score -= penalty;
                health_status.issues.push(ContentHealthIssue {
                    severity: IssueSeverity::Warning,
                    description: format!("Content is outdated in {} region(s)", outdated_regions.len()),
                    affected_regions: outdated_regions,
                    first_detected: Utc::now(),
                    suggested_action: "Trigger content synchronization".to_string(),
                });
            }
        }

        if let Some(usage) = self.usage_stats.read().await.get(&content_id) {
            let now = Utc::now();
            let days_since_access = (now - usage.last_accessed).num_days();
            if days_since_access > 90 {
                health_status.issues.push(ContentHealthIssue {
                    severity: IssueSeverity::Info,
                    description: format!("Content has not been accessed in {} days", days_since_access),
                    affected_regions: Vec::new(),
                    first_detected: Utc::now(),
                    suggested_action: "Consider archiving if no longer needed".to_string(),
                });
            }
        }

        health_status.health_score = health_status.health_score.clamp(0.0, 100.0);

        self.health_status.write().await.insert(content_id, health_status);
        Ok(())
    }
    
    async fn optimize_storage(&mut self) -> ContentResult<()> {
        // Check storage usage
        let storage_dir = &self.config.storage_config.storage_directory;
        let usage = self.calculate_storage_usage(storage_dir).await?;
        let max_size = self.config.storage_config.max_storage_size;
        
        if usage as f32 / max_size as f32 > self.config.storage_config.cleanup_threshold {
            // Implement cleanup logic (remove old, unused content)
            tracing::info!("Storage usage {:.1}% - cleanup needed", 
                          (usage as f32 / max_size as f32) * 100.0);
        }
        
        Ok(())
    }
    
    async fn calculate_storage_usage(&self, directory: &Path) -> ContentResult<u64> {
        let mut total_size = 0u64;
        
        if directory.exists() {
            let mut entries = tokio::fs::read_dir(directory).await?;
            while let Some(entry) = entries.next_entry().await? {
                if let Ok(metadata) = entry.metadata().await {
                    total_size += metadata.len();
                }
            }
        }
        
        Ok(total_size)
    }
}

impl Default for ContentManagerConfig {
    fn default() -> Self {
        Self {
            creation_config: ContentCreationConfig::default(),
            distribution_config: DistributionConfig::default(),
            storage_config: ContentStorageConfig::default(),
            security_config: ContentSecurityConfig::default(),
            analytics_config: ContentAnalyticsConfig::default(),
        }
    }
}

impl Default for ContentStorageConfig {
    fn default() -> Self {
        Self {
            storage_directory: PathBuf::from("content/storage"),
            backup_directory: Some(PathBuf::from("content/backup")),
            max_storage_size: 100 * 1024 * 1024 * 1024, // 100 GB
            auto_cleanup: true,
            cleanup_threshold: 0.85, // 85%
            enable_encryption: false,
            replication_factor: 1,
        }
    }
}

impl Default for ContentSecurityConfig {
    fn default() -> Self {
        Self {
            enable_drm: false,
            enable_scanning: true,
            enable_access_logging: true,
            max_failed_attempts: 5,
            key_rotation_interval: 24, // 24 hours
            enable_watermarking: false,
        }
    }
}

impl Default for ContentAnalyticsConfig {
    fn default() -> Self {
        Self {
            enable_usage_tracking: true,
            enable_performance_monitoring: true,
            retention_period: 365, // 1 year
            enable_realtime_analytics: true,
            sampling_rate: 1.0, // 100%
        }
    }
}