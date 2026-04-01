//! Content Management System for OpenSim Next
//!
//! Provides advanced content creation, distribution, validation, and security
//! for virtual world assets including 3D models, textures, scripts, and multimedia.
//!
//! Features:
//! - Advanced content creation tools with 3D model import and validation
//! - Content distribution with versioning and cross-region synchronization
//! - Content security with DRM, validation, and anti-piracy measures
//! - Content marketplace integration with ownership verification
//! - Real-time content analytics and performance tracking

pub mod creation;
pub mod distribution;
pub mod security;
pub mod marketplace;
pub mod validation;
pub mod versioning;
pub mod analytics;
pub mod import;
pub mod manager;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
/// EADS fix: Custom serde serialization for semver::Version
mod version_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use semver::Version;

    pub fn serialize<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        version.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Content types supported by the content management system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// 3D models (OBJ, FBX, GLTF, COLLADA)
    Model3D,
    /// Texture images (PNG, JPEG, TGA, DDS)
    Texture,
    /// Audio files (WAV, MP3, OGG)
    Audio,
    /// Video files (MP4, AVI, MOV)
    Video,
    /// LSL scripts and bytecode
    Script,
    /// Animation data (BVH, FBX animations)
    Animation,
    /// Particle system definitions
    ParticleSystem,
    /// Wearable items (clothing, attachments)
    Wearable,
    /// Gesture definitions
    Gesture,
    /// Landmark data
    Landmark,
    /// Notecard text content
    Notecard,
    /// Custom content types
    Custom(String),
}

/// Content quality levels for optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentQuality {
    /// Ultra-high quality (original)
    Ultra,
    /// High quality (minimal compression)
    High,
    /// Medium quality (balanced)
    Medium,
    /// Low quality (maximum compression)
    Low,
    /// Custom quality settings
    Custom {
        compression: f32,
        resolution_scale: f32,
        polygon_reduction: f32,
    },
}

/// Content distribution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionStrategy {
    /// Immediate distribution to all regions
    Immediate,
    /// Lazy loading on demand
    OnDemand,
    /// Scheduled distribution
    Scheduled {
        start_time: DateTime<Utc>,
        regions: Vec<Uuid>,
    },
    /// Progressive distribution based on usage
    Progressive {
        priority_regions: Vec<Uuid>,
        rollout_percentage: f32,
    },
}

/// Content metadata and information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub content_id: Uuid,
    pub name: String,
    pub description: String,
    pub content_type: ContentType,
    pub creator_id: Uuid,
    pub creation_date: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    #[serde(with = "version_serde")]
    pub version: semver::Version,
    pub file_size: u64,
    pub checksum: String,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub quality_levels: Vec<ContentQuality>,
    pub distribution_strategy: DistributionStrategy,
    pub usage_permissions: ContentPermissions,
    pub drm_protection: bool,
    pub marketplace_listed: bool,
    pub price: Option<ContentPrice>,
    pub download_count: u64,
    pub rating: ContentRating,
}

/// Content permissions and usage rights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPermissions {
    pub can_copy: bool,
    pub can_modify: bool,
    pub can_transfer: bool,
    pub can_resell: bool,
    pub restricted_regions: Vec<Uuid>,
    pub allowed_users: Option<Vec<Uuid>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub usage_limit: Option<u32>,
}

/// Content pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPrice {
    pub currency: String,
    pub amount: f64,
    pub discount_percentage: Option<f32>,
    pub bundle_pricing: Option<HashMap<u32, f64>>, // quantity -> price
}

/// Content rating and review system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRating {
    pub average_rating: f32,
    pub total_ratings: u32,
    pub rating_breakdown: HashMap<u8, u32>, // rating (1-5) -> count
    pub featured: bool,
    pub content_warnings: Vec<String>,
}

/// Content validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub performance_score: f32,
    pub security_score: f32,
    pub quality_score: f32,
    pub recommendations: Vec<String>,
}

/// Content distribution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDistributionStatus {
    pub content_id: Uuid,
    pub total_regions: u32,
    pub distributed_regions: u32,
    pub failed_regions: Vec<Uuid>,
    pub distribution_progress: f32,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub bandwidth_usage: u64,
}

/// Content analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalytics {
    pub content_id: Uuid,
    pub total_downloads: u64,
    pub unique_users: u32,
    pub average_rating: f32,
    pub usage_by_region: HashMap<Uuid, u64>,
    pub performance_metrics: ContentPerformanceMetrics,
    pub revenue_data: Option<ContentRevenueData>,
}

/// Content performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPerformanceMetrics {
    pub load_time_avg: f32,
    pub render_performance: f32,
    pub memory_usage: u64,
    pub cache_hit_ratio: f32,
    pub error_rate: f32,
}

/// Content revenue tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRevenueData {
    pub total_revenue: f64,
    pub currency: String,
    pub sales_count: u32,
    pub refund_count: u32,
    pub revenue_by_period: HashMap<String, f64>, // period -> revenue
}

/// Content import configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentImportConfig {
    pub auto_optimize: bool,
    pub target_quality: ContentQuality,
    pub generate_lods: bool,
    pub validate_content: bool,
    pub apply_drm: bool,
    pub auto_distribute: bool,
    pub notification_settings: ImportNotificationSettings,
}

/// Import notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportNotificationSettings {
    pub notify_on_completion: bool,
    pub notify_on_errors: bool,
    pub notification_channels: Vec<NotificationChannel>,
}

/// Notification channels for content events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email(String),
    Webhook(String),
    InWorld(Uuid), // User ID
    Dashboard,
}

/// Content search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSearchFilter {
    pub content_types: Option<Vec<ContentType>>,
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub creator_ids: Option<Vec<Uuid>>,
    pub min_rating: Option<f32>,
    pub max_price: Option<f64>,
    pub quality_levels: Option<Vec<ContentQuality>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub has_drm: Option<bool>,
    pub marketplace_only: Option<bool>,
}

/// Content search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSearchResult {
    pub results: Vec<ContentMetadata>,
    pub total_count: u32,
    pub page: u32,
    pub page_size: u32,
    pub search_time_ms: u32,
    pub facets: HashMap<String, HashMap<String, u32>>, // facet -> value -> count
}

/// Content operation result
pub type ContentResult<T> = Result<T, ContentError>;

/// Content management errors
#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("Content not found: {id}")]
    ContentNotFound { id: Uuid },

    #[error("Resource not found: {id}")]
    NotFound { id: Uuid },

    #[error("Invalid content format: {format}")]
    InvalidFormat { format: String },

    #[error("Content validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Validation error: {reason}")]
    ValidationError { reason: String },

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("Distribution failed: {reason}")]
    DistributionFailed { reason: String },

    #[error("Import failed: {reason}")]
    ImportFailed { reason: String },

    #[error("DRM protection error: {reason}")]
    DrmError { reason: String },

    #[error("Marketplace error: {reason}")]
    MarketplaceError { reason: String },

    #[error("Storage error: {reason}")]
    StorageError { reason: String },

    #[error("Network error: {reason}")]
    NetworkError { reason: String },

    #[error("Invalid operation: {reason}")]
    InvalidOperation { reason: String },

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

}

impl Default for ContentPermissions {
    fn default() -> Self {
        Self {
            can_copy: true,
            can_modify: false,
            can_transfer: true,
            can_resell: false,
            restricted_regions: Vec::new(),
            allowed_users: None,
            expiration_date: None,
            usage_limit: None,
        }
    }
}

impl Default for ContentRating {
    fn default() -> Self {
        Self {
            average_rating: 0.0,
            total_ratings: 0,
            rating_breakdown: HashMap::new(),
            featured: false,
            content_warnings: Vec::new(),
        }
    }
}

impl Default for ContentQuality {
    fn default() -> Self {
        Self::High
    }
}

impl Default for DistributionStrategy {
    fn default() -> Self {
        Self::OnDemand
    }
}

impl ContentType {
    /// Get the file extensions associated with this content type
    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            ContentType::Model3D => vec!["obj", "fbx", "gltf", "glb", "dae", "3ds", "blend"],
            ContentType::Texture => vec!["png", "jpg", "jpeg", "tga", "bmp", "dds", "exr"],
            ContentType::Audio => vec!["wav", "mp3", "ogg", "flac", "aac"],
            ContentType::Video => vec!["mp4", "avi", "mov", "mkv", "webm"],
            ContentType::Script => vec!["lsl", "cs", "js", "py"],
            ContentType::Animation => vec!["bvh", "fbx", "anim"],
            ContentType::ParticleSystem => vec!["json", "xml"],
            ContentType::Wearable => vec!["wearable", "clothing"],
            ContentType::Gesture => vec!["gesture"],
            ContentType::Landmark => vec!["landmark"],
            ContentType::Notecard => vec!["txt", "md", "notecard"],
            ContentType::Custom(_) => vec!["*"],
        }
    }
    
    /// Check if a file extension is supported by this content type
    pub fn supports_extension(&self, extension: &str) -> bool {
        let ext = extension.to_lowercase();
        self.file_extensions().iter().any(|&e| e == ext || e == "*")
    }
    
    /// Get the MIME type for this content type
    pub fn mime_type(&self) -> &'static str {
        match self {
            ContentType::Model3D => "model/obj",
            ContentType::Texture => "image/png",
            ContentType::Audio => "audio/wav",
            ContentType::Video => "video/mp4",
            ContentType::Script => "text/plain",
            ContentType::Animation => "application/octet-stream",
            ContentType::ParticleSystem => "application/json",
            ContentType::Wearable => "application/opensim-wearable",
            ContentType::Gesture => "application/opensim-gesture",
            ContentType::Landmark => "application/opensim-landmark",
            ContentType::Notecard => "text/plain",
            ContentType::Custom(_) => "application/octet-stream",
        }
    }
}

impl ContentQuality {
    /// Get the compression ratio for this quality level
    pub fn compression_ratio(&self) -> f32 {
        match self {
            ContentQuality::Ultra => 0.0,   // No compression
            ContentQuality::High => 0.1,    // 10% compression
            ContentQuality::Medium => 0.3,  // 30% compression
            ContentQuality::Low => 0.6,     // 60% compression
            ContentQuality::Custom { compression, .. } => *compression,
        }
    }
    
    /// Get the resolution scale for this quality level
    pub fn resolution_scale(&self) -> f32 {
        match self {
            ContentQuality::Ultra => 1.0,   // Full resolution
            ContentQuality::High => 1.0,    // Full resolution
            ContentQuality::Medium => 0.75, // 75% resolution
            ContentQuality::Low => 0.5,     // 50% resolution
            ContentQuality::Custom { resolution_scale, .. } => *resolution_scale,
        }
    }
    
    /// Get the polygon reduction ratio for 3D models
    pub fn polygon_reduction(&self) -> f32 {
        match self {
            ContentQuality::Ultra => 0.0,   // No reduction
            ContentQuality::High => 0.05,   // 5% reduction
            ContentQuality::Medium => 0.2,  // 20% reduction
            ContentQuality::Low => 0.5,     // 50% reduction
            ContentQuality::Custom { polygon_reduction, .. } => *polygon_reduction,
        }
    }
}