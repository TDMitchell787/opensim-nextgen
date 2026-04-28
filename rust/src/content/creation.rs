//! Content Creation Tools for OpenSim Next
//!
//! Provides advanced content creation capabilities including 3D model import,
//! texture optimization, validation, and automatic quality level generation.

use super::{
    ContentError, ContentMetadata, ContentPermissions, ContentQuality, ContentRating,
    ContentResult, ContentType, ContentValidationResult, DistributionStrategy,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Content creation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCreationConfig {
    /// Maximum file size allowed for uploads (bytes)
    pub max_file_size: u64,
    /// Supported content types
    pub supported_types: Vec<ContentType>,
    /// Auto-optimization settings
    pub auto_optimization: AutoOptimizationConfig,
    /// Validation rules
    pub validation_rules: ValidationRules,
    /// Default permissions for new content
    pub default_permissions: ContentPermissions,
    /// Upload directory path
    pub upload_directory: PathBuf,
    /// Temporary processing directory
    pub temp_directory: PathBuf,
}

/// Auto-optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoOptimizationConfig {
    /// Generate multiple quality levels automatically
    pub generate_quality_levels: bool,
    /// Quality levels to generate
    pub target_qualities: Vec<ContentQuality>,
    /// Optimize textures automatically
    pub optimize_textures: bool,
    /// Generate LOD (Level of Detail) models
    pub generate_lods: bool,
    /// Compress audio files
    pub compress_audio: bool,
    /// Optimize for web delivery
    pub web_optimization: bool,
}

/// Content validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Maximum polygon count for 3D models
    pub max_polygons: u32,
    /// Maximum texture resolution
    pub max_texture_resolution: (u32, u32),
    /// Maximum audio duration (seconds)
    pub max_audio_duration: u32,
    /// Allowed file formats per content type
    pub allowed_formats: HashMap<ContentType, Vec<String>>,
    /// Security scanning enabled
    pub security_scanning: bool,
    /// Content rating requirements
    pub content_rating_required: bool,
}

/// Content creation job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCreationJob {
    pub job_id: Uuid,
    pub creator_id: Uuid,
    pub content_name: String,
    pub content_type: ContentType,
    pub source_file_path: PathBuf,
    pub target_quality: ContentQuality,
    pub permissions: ContentPermissions,
    pub distribution_strategy: DistributionStrategy,
    pub auto_optimize: bool,
    pub status: CreationJobStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: f32,
    pub error_message: Option<String>,
    pub result_content_id: Option<Uuid>,
}

/// Content creation job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CreationJobStatus {
    Queued,
    Processing,
    Validating,
    Optimizing,
    Uploading,
    Completed,
    Failed,
    Cancelled,
}

/// 3D model processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model3DProcessingResult {
    pub original_polygons: u32,
    pub optimized_polygons: u32,
    pub polygon_reduction: f32,
    pub generated_lods: Vec<LevelOfDetail>,
    pub material_count: u32,
    pub texture_count: u32,
    pub animation_count: u32,
    pub bounding_box: BoundingBox3D,
    pub processing_time_ms: u32,
}

/// Level of Detail information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelOfDetail {
    pub lod_level: u8,
    pub polygon_count: u32,
    pub distance_threshold: f32,
    pub file_size: u64,
    pub file_path: PathBuf,
}

/// 3D bounding box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox3D {
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
}

/// Texture processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureProcessingResult {
    pub original_resolution: (u32, u32),
    pub optimized_resolution: (u32, u32),
    pub original_format: String,
    pub optimized_format: String,
    pub compression_ratio: f32,
    pub quality_score: f32,
    pub mipmap_levels: u8,
    pub processing_time_ms: u32,
}

/// Audio processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioProcessingResult {
    pub duration_seconds: f32,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub channels: u8,
    pub original_format: String,
    pub optimized_format: String,
    pub compression_ratio: f32,
    pub quality_score: f32,
    pub processing_time_ms: u32,
}

/// Content creation manager
pub struct ContentCreationManager {
    config: ContentCreationConfig,
    creation_jobs: Arc<RwLock<HashMap<Uuid, ContentCreationJob>>>,
    processing_queue: Arc<RwLock<Vec<Uuid>>>,
    workers_active: Arc<RwLock<u32>>,
    max_workers: u32,
}

impl ContentCreationManager {
    /// Create a new content creation manager
    pub fn new(config: ContentCreationConfig) -> ContentResult<Self> {
        // Ensure directories exist
        std::fs::create_dir_all(&config.upload_directory)?;
        std::fs::create_dir_all(&config.temp_directory)?;

        Ok(Self {
            config,
            creation_jobs: Arc::new(RwLock::new(HashMap::new())),
            processing_queue: Arc::new(RwLock::new(Vec::new())),
            workers_active: Arc::new(RwLock::new(0)),
            max_workers: 4, // Default to 4 workers
        })
    }

    /// Start content creation job
    pub async fn create_content(
        &mut self,
        creator_id: Uuid,
        content_name: String,
        content_type: ContentType,
        source_file: PathBuf,
        permissions: Option<ContentPermissions>,
        options: ContentCreationOptions,
    ) -> ContentResult<Uuid> {
        // Validate input file
        self.validate_input_file(&source_file, &content_type)
            .await?;

        let job_id = Uuid::new_v4();
        let job = ContentCreationJob {
            job_id,
            creator_id,
            content_name,
            content_type,
            source_file_path: source_file,
            target_quality: options.target_quality,
            permissions: permissions.unwrap_or_else(|| self.config.default_permissions.clone()),
            distribution_strategy: options.distribution_strategy,
            auto_optimize: options.auto_optimize,
            status: CreationJobStatus::Queued,
            created_at: Utc::now(),
            completed_at: None,
            progress: 0.0,
            error_message: None,
            result_content_id: None,
        };

        // Store job
        self.creation_jobs.write().await.insert(job_id, job);

        // Add to processing queue
        self.processing_queue.write().await.push(job_id);

        // Start processing if workers available
        self.try_start_worker().await?;

        tracing::info!("Content creation job started: {}", job_id);
        Ok(job_id)
    }

    /// Get content creation job status
    pub async fn get_job_status(&self, job_id: Uuid) -> ContentResult<ContentCreationJob> {
        self.creation_jobs
            .read()
            .await
            .get(&job_id)
            .cloned()
            .ok_or(ContentError::ContentNotFound { id: job_id })
    }

    /// Cancel content creation job
    pub async fn cancel_job(&mut self, job_id: Uuid) -> ContentResult<()> {
        let mut jobs = self.creation_jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            if job.status == CreationJobStatus::Queued
                || job.status == CreationJobStatus::Processing
            {
                job.status = CreationJobStatus::Cancelled;
                tracing::info!("Content creation job cancelled: {}", job_id);
            }
        }
        Ok(())
    }

    /// Process 3D model content
    pub async fn process_3d_model(
        &self,
        file_path: &Path,
        target_quality: &ContentQuality,
        generate_lods: bool,
    ) -> ContentResult<Model3DProcessingResult> {
        let start_time = std::time::Instant::now();

        // Load 3D model (stub implementation)
        let model_data = self.load_3d_model(file_path).await?;

        // Count original polygons
        let original_polygons = model_data.polygon_count;

        // Optimize based on quality settings
        let optimized_polygons =
            self.optimize_polygon_count(original_polygons, target_quality.polygon_reduction());

        // Generate LODs if requested
        let mut generated_lods = Vec::new();
        if generate_lods {
            generated_lods = self.generate_lods(&model_data, file_path).await?;
        }

        // Calculate bounding box
        let bounding_box = self.calculate_bounding_box(&model_data);

        let processing_time = start_time.elapsed().as_millis() as u32;

        let result = Model3DProcessingResult {
            original_polygons,
            optimized_polygons,
            polygon_reduction: 1.0 - (optimized_polygons as f32 / original_polygons as f32),
            generated_lods,
            material_count: model_data.material_count,
            texture_count: model_data.texture_count,
            animation_count: model_data.animation_count,
            bounding_box,
            processing_time_ms: processing_time,
        };

        tracing::info!(
            "3D model processed: {} polygons -> {} polygons ({:.1}% reduction)",
            original_polygons,
            optimized_polygons,
            result.polygon_reduction * 100.0
        );

        Ok(result)
    }

    /// Process texture content
    pub async fn process_texture(
        &self,
        file_path: &Path,
        target_quality: &ContentQuality,
        generate_mipmaps: bool,
    ) -> ContentResult<TextureProcessingResult> {
        let start_time = std::time::Instant::now();

        // Load texture data (stub implementation)
        let texture_data = self.load_texture(file_path).await?;

        // Calculate optimized resolution
        let scale = target_quality.resolution_scale();
        let optimized_resolution = (
            (texture_data.width as f32 * scale) as u32,
            (texture_data.height as f32 * scale) as u32,
        );

        // Optimize texture
        let compression_ratio = target_quality.compression_ratio();
        let quality_score = self.calculate_texture_quality(&texture_data, compression_ratio);

        // Generate mipmaps if requested
        let mipmap_levels = if generate_mipmaps {
            self.calculate_mipmap_levels(optimized_resolution.0, optimized_resolution.1)
        } else {
            1
        };

        let processing_time = start_time.elapsed().as_millis() as u32;

        let result = TextureProcessingResult {
            original_resolution: (texture_data.width, texture_data.height),
            optimized_resolution,
            original_format: texture_data.format.clone(),
            optimized_format: self.select_optimal_format(&texture_data.format, target_quality),
            compression_ratio,
            quality_score,
            mipmap_levels,
            processing_time_ms: processing_time,
        };

        tracing::info!(
            "Texture processed: {}x{} -> {}x{} ({:.1}% compression)",
            result.original_resolution.0,
            result.original_resolution.1,
            result.optimized_resolution.0,
            result.optimized_resolution.1,
            compression_ratio * 100.0
        );

        Ok(result)
    }

    /// Process audio content
    pub async fn process_audio(
        &self,
        file_path: &Path,
        target_quality: &ContentQuality,
    ) -> ContentResult<AudioProcessingResult> {
        let start_time = std::time::Instant::now();

        // Load audio data (stub implementation)
        let audio_data = self.load_audio(file_path).await?;

        // Calculate compression settings
        let compression_ratio = target_quality.compression_ratio();
        let quality_score = self.calculate_audio_quality(&audio_data, compression_ratio);

        let processing_time = start_time.elapsed().as_millis() as u32;

        let result = AudioProcessingResult {
            duration_seconds: audio_data.duration_seconds,
            sample_rate: audio_data.sample_rate,
            bit_depth: audio_data.bit_depth,
            channels: audio_data.channels,
            original_format: audio_data.format.clone(),
            optimized_format: self.select_optimal_audio_format(&audio_data.format, target_quality),
            compression_ratio,
            quality_score,
            processing_time_ms: processing_time,
        };

        tracing::info!(
            "Audio processed: {:.1}s duration, {:.1}% compression",
            result.duration_seconds,
            compression_ratio * 100.0
        );

        Ok(result)
    }

    /// Validate content against rules
    pub async fn validate_content(
        &self,
        content_type: &ContentType,
        file_path: &Path,
        metadata: Option<&ContentMetadata>,
    ) -> ContentResult<ContentValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // File size validation
        let file_size = tokio::fs::metadata(file_path).await?.len();
        if file_size > self.config.max_file_size {
            errors.push(format!(
                "File size {} exceeds maximum allowed size {}",
                file_size, self.config.max_file_size
            ));
        }

        // Content-specific validation
        match content_type {
            ContentType::Model3D => {
                self.validate_3d_model(file_path, &mut errors, &mut warnings, &mut recommendations)
                    .await?;
            }
            ContentType::Texture => {
                self.validate_texture(file_path, &mut errors, &mut warnings, &mut recommendations)
                    .await?;
            }
            ContentType::Audio => {
                self.validate_audio(file_path, &mut errors, &mut warnings, &mut recommendations)
                    .await?;
            }
            ContentType::Script => {
                self.validate_script(file_path, &mut errors, &mut warnings, &mut recommendations)
                    .await?;
            }
            _ => {
                // Basic validation for other types
                if !file_path.exists() {
                    errors.push("File does not exist".to_string());
                }
            }
        }

        // Security validation
        if self.config.validation_rules.security_scanning {
            self.security_scan(file_path, &mut errors, &mut warnings)
                .await?;
        }

        // Calculate scores
        let performance_score = self
            .calculate_performance_score(content_type, file_path)
            .await?;
        let security_score = if errors.iter().any(|e| e.contains("security")) {
            0.0
        } else {
            100.0
        };
        let quality_score = self
            .calculate_quality_score(content_type, file_path)
            .await?;

        let result = ContentValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            performance_score,
            security_score,
            quality_score,
            recommendations,
        };

        tracing::info!("Content validation completed: valid={}, performance={:.1}, security={:.1}, quality={:.1}",
                      result.is_valid, result.performance_score, result.security_score, result.quality_score);

        Ok(result)
    }

    // Private helper methods (stub implementations)

    async fn validate_input_file(
        &self,
        file_path: &Path,
        content_type: &ContentType,
    ) -> ContentResult<()> {
        if !file_path.exists() {
            return Err(ContentError::ImportFailed {
                reason: "File does not exist".to_string(),
            });
        }

        // Check file extension
        if let Some(extension) = file_path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if !content_type.supports_extension(&ext) {
                return Err(ContentError::InvalidFormat { format: ext });
            }
        }

        Ok(())
    }

    async fn try_start_worker(&self) -> ContentResult<()> {
        let active_workers = *self.workers_active.read().await;
        if active_workers < self.max_workers {
            let queue_len = self.processing_queue.read().await.len();
            if queue_len > 0 {
                // Start worker (stub implementation)
                *self.workers_active.write().await += 1;
                tracing::info!(
                    "Started content processing worker ({}/{})",
                    active_workers + 1,
                    self.max_workers
                );
            }
        }
        Ok(())
    }

    async fn load_3d_model(&self, file_path: &Path) -> ContentResult<Model3DData> {
        let data = tokio::fs::read(file_path).await?;
        let extension = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        match extension.as_str() {
            "obj" => self.parse_obj_model(&data),
            "stl" => self.parse_stl_model(&data),
            "gltf" | "glb" => self.parse_gltf_model(&data, &extension),
            "fbx" => self.parse_fbx_model(&data),
            _ => Err(ContentError::InvalidFormat { format: extension }),
        }
    }

    fn parse_obj_model(&self, data: &[u8]) -> ContentResult<Model3DData> {
        let content = String::from_utf8_lossy(data);
        let mut vertex_count = 0u32;
        let mut face_count = 0u32;
        let mut material_count = 0u32;
        let mut texture_count = 0u32;
        let mut current_material = String::new();
        let mut materials_seen = std::collections::HashSet::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("v ") {
                vertex_count += 1;
            } else if line.starts_with("f ") {
                let parts: Vec<&str> = line[2..].split_whitespace().collect();
                if parts.len() >= 3 {
                    face_count += (parts.len() - 2) as u32;
                }
            } else if line.starts_with("usemtl ") {
                current_material = line[7..].trim().to_string();
                materials_seen.insert(current_material.clone());
            } else if line.starts_with("mtllib ") {
                material_count += 1;
            } else if line.starts_with("map_Kd ")
                || line.starts_with("map_Ka ")
                || line.starts_with("map_Ks ")
            {
                texture_count += 1;
            }
        }

        material_count = material_count.max(materials_seen.len() as u32);
        let polygon_count = face_count;

        Ok(Model3DData {
            polygon_count,
            vertex_count,
            material_count,
            texture_count,
            animation_count: 0,
            vertices: Vec::new(),
        })
    }

    fn parse_stl_model(&self, data: &[u8]) -> ContentResult<Model3DData> {
        let is_ascii = data.starts_with(b"solid") && !data[..80.min(data.len())].contains(&0);

        let polygon_count = if is_ascii {
            let content = String::from_utf8_lossy(data);
            content.matches("facet normal").count() as u32
        } else if data.len() > 84 {
            u32::from_le_bytes([data[80], data[81], data[82], data[83]])
        } else {
            0
        };

        let vertex_count = polygon_count * 3;

        Ok(Model3DData {
            polygon_count,
            vertex_count,
            material_count: 1,
            texture_count: 0,
            animation_count: 0,
            vertices: Vec::new(),
        })
    }

    fn parse_gltf_model(&self, data: &[u8], extension: &str) -> ContentResult<Model3DData> {
        if extension == "glb" {
            if data.len() < 12 || &data[0..4] != b"glTF" {
                return Err(ContentError::ValidationError {
                    reason: "Invalid GLB header".to_string(),
                });
            }
            let json_length = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
            if data.len() < 20 + json_length {
                return Err(ContentError::ValidationError {
                    reason: "Truncated GLB file".to_string(),
                });
            }
            let json_data = &data[20..20 + json_length];
            self.parse_gltf_json(json_data)
        } else {
            self.parse_gltf_json(data)
        }
    }

    fn parse_gltf_json(&self, data: &[u8]) -> ContentResult<Model3DData> {
        let content = String::from_utf8_lossy(data);

        let mesh_count = content.matches("\"primitives\"").count() as u32;
        let material_count = content.matches("\"pbrMetallicRoughness\"").count() as u32;
        let texture_count = content
            .matches("\"textures\"")
            .count()
            .max(content.matches("\"images\"").count()) as u32;
        let animation_count = content.matches("\"animations\"").count() as u32;

        let polygon_estimate = mesh_count * 1000;

        Ok(Model3DData {
            polygon_count: polygon_estimate,
            vertex_count: polygon_estimate * 3,
            material_count: material_count.max(1),
            texture_count,
            animation_count,
            vertices: Vec::new(),
        })
    }

    fn parse_fbx_model(&self, data: &[u8]) -> ContentResult<Model3DData> {
        let is_binary = data.len() >= 20 && &data[0..20] == b"Kaydara FBX Binary  ";

        if is_binary {
            let polygon_estimate = (data.len() / 100) as u32;
            Ok(Model3DData {
                polygon_count: polygon_estimate,
                vertex_count: polygon_estimate * 3,
                material_count: 1,
                texture_count: 0,
                animation_count: 0,
                vertices: Vec::new(),
            })
        } else {
            let content = String::from_utf8_lossy(data);
            let vertex_count = content.matches("Vertices:").count() as u32 * 100;
            let polygon_count = content.matches("PolygonVertexIndex:").count() as u32 * 100;
            let material_count = content.matches("Material:").count() as u32;
            let texture_count = content.matches("Texture:").count() as u32;

            Ok(Model3DData {
                polygon_count: polygon_count.max(vertex_count / 3),
                vertex_count,
                material_count: material_count.max(1),
                texture_count,
                animation_count: 0,
                vertices: Vec::new(),
            })
        }
    }

    async fn load_texture(&self, file_path: &Path) -> ContentResult<TextureData> {
        let data = tokio::fs::read(file_path).await?;

        if data.len() < 8 {
            return Err(ContentError::ValidationError {
                reason: "File too small to be a valid image".to_string(),
            });
        }

        if data.len() >= 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
            return self.parse_png_header(&data);
        }

        if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
            return self.parse_jpeg_header(&data);
        }

        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return self.parse_webp_header(&data);
        }

        if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
            let width = u16::from_le_bytes([data[6], data[7]]) as u32;
            let height = u16::from_le_bytes([data[8], data[9]]) as u32;
            return Ok(TextureData {
                width,
                height,
                format: "GIF".to_string(),
                channels: 4,
                bit_depth: 8,
                has_alpha: true,
            });
        }

        if data.len() >= 4
            && (data[0] == 0x00 && data[1] == 0x00 && data[2] == 0x00 && data[3] == 0x0C)
        {
            return self.parse_jp2_header(&data);
        }

        if data.len() >= 18 {
            let width = u16::from_le_bytes([data[12], data[13]]) as u32;
            let height = u16::from_le_bytes([data[14], data[15]]) as u32;
            let bit_depth = data[16];
            let image_type = data[2];
            if width > 0 && width <= 16384 && height > 0 && height <= 16384 && bit_depth <= 32 {
                let has_alpha = image_type == 2 || image_type == 10;
                return Ok(TextureData {
                    width,
                    height,
                    format: "TGA".to_string(),
                    channels: if has_alpha { 4 } else { 3 },
                    bit_depth,
                    has_alpha,
                });
            }
        }

        Err(ContentError::InvalidFormat {
            format: "unknown".to_string(),
        })
    }

    fn parse_png_header(&self, data: &[u8]) -> ContentResult<TextureData> {
        if data.len() < 24 {
            return Err(ContentError::ValidationError {
                reason: "Truncated PNG header".to_string(),
            });
        }

        let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        let bit_depth = data[24];
        let color_type = data[25];

        let (channels, has_alpha) = match color_type {
            0 => (1, false),
            2 => (3, false),
            3 => (1, false),
            4 => (2, true),
            6 => (4, true),
            _ => (4, true),
        };

        Ok(TextureData {
            width,
            height,
            format: "PNG".to_string(),
            channels,
            bit_depth,
            has_alpha,
        })
    }

    fn parse_jpeg_header(&self, data: &[u8]) -> ContentResult<TextureData> {
        let mut i = 2;
        while i < data.len() - 9 {
            if data[i] != 0xFF {
                i += 1;
                continue;
            }

            let marker = data[i + 1];

            if (marker >= 0xC0 && marker <= 0xCF)
                && marker != 0xC4
                && marker != 0xC8
                && marker != 0xCC
            {
                let bit_depth = data[i + 4];
                let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                let channels = data[i + 9];

                return Ok(TextureData {
                    width,
                    height,
                    format: "JPEG".to_string(),
                    channels,
                    bit_depth,
                    has_alpha: false,
                });
            }

            if marker == 0xD8 || marker == 0xD9 || (marker >= 0xD0 && marker <= 0xD7) {
                i += 2;
            } else {
                let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                i += 2 + length;
            }
        }

        Ok(TextureData {
            width: 0,
            height: 0,
            format: "JPEG".to_string(),
            channels: 3,
            bit_depth: 8,
            has_alpha: false,
        })
    }

    fn parse_webp_header(&self, data: &[u8]) -> ContentResult<TextureData> {
        if data.len() < 30 {
            return Err(ContentError::ValidationError {
                reason: "Truncated WebP header".to_string(),
            });
        }

        let chunk_type = &data[12..16];

        if chunk_type == b"VP8 " && data.len() >= 30 {
            let width = (u16::from_le_bytes([data[26], data[27]]) & 0x3FFF) as u32;
            let height = (u16::from_le_bytes([data[28], data[29]]) & 0x3FFF) as u32;
            return Ok(TextureData {
                width,
                height,
                format: "WEBP".to_string(),
                channels: 3,
                bit_depth: 8,
                has_alpha: false,
            });
        }

        if chunk_type == b"VP8L" && data.len() >= 25 {
            let b0 = data[21] as u32;
            let b1 = data[22] as u32;
            let b2 = data[23] as u32;
            let b3 = data[24] as u32;
            let width = ((b0 | (b1 << 8)) & 0x3FFF) + 1;
            let height = (((b1 >> 6) | (b2 << 2) | (b3 << 10)) & 0x3FFF) + 1;
            let has_alpha = (b3 & 0x10) != 0;
            return Ok(TextureData {
                width,
                height,
                format: "WEBP".to_string(),
                channels: if has_alpha { 4 } else { 3 },
                bit_depth: 8,
                has_alpha,
            });
        }

        Ok(TextureData {
            width: 0,
            height: 0,
            format: "WEBP".to_string(),
            channels: 4,
            bit_depth: 8,
            has_alpha: true,
        })
    }

    fn parse_jp2_header(&self, data: &[u8]) -> ContentResult<TextureData> {
        let mut i = 0;
        while i + 8 < data.len() {
            let box_size =
                u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
            let box_type = &data[i + 4..i + 8];

            if box_type == b"ihdr" && i + 22 <= data.len() {
                let height =
                    u32::from_be_bytes([data[i + 8], data[i + 9], data[i + 10], data[i + 11]]);
                let width =
                    u32::from_be_bytes([data[i + 12], data[i + 13], data[i + 14], data[i + 15]]);
                let channels = u16::from_be_bytes([data[i + 16], data[i + 17]]) as u8;
                let bit_depth = data[i + 18] + 1;
                return Ok(TextureData {
                    width,
                    height,
                    format: "JP2".to_string(),
                    channels,
                    bit_depth,
                    has_alpha: channels > 3,
                });
            }

            if box_size == 0 {
                break;
            }
            i += box_size;
        }

        Ok(TextureData {
            width: 0,
            height: 0,
            format: "JP2".to_string(),
            channels: 3,
            bit_depth: 8,
            has_alpha: false,
        })
    }

    async fn load_audio(&self, file_path: &Path) -> ContentResult<AudioData> {
        let data = tokio::fs::read(file_path).await?;

        if data.len() < 12 {
            return Err(ContentError::ValidationError {
                reason: "File too small to be valid audio".to_string(),
            });
        }

        if &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
            return self.parse_wav_header(&data);
        }

        if &data[0..4] == b"OggS" {
            return self.parse_ogg_header(&data);
        }

        if &data[0..4] == b"fLaC" {
            return self.parse_flac_header(&data);
        }

        if &data[0..3] == b"ID3" || (data[0] == 0xFF && (data[1] & 0xE0) == 0xE0) {
            return self.parse_mp3_header(&data);
        }

        if &data[0..4] == b"FORM" && data.len() >= 12 && &data[8..12] == b"AIFF" {
            return self.parse_aiff_header(&data);
        }

        Err(ContentError::InvalidFormat {
            format: "unknown audio".to_string(),
        })
    }

    fn parse_wav_header(&self, data: &[u8]) -> ContentResult<AudioData> {
        let mut i = 12;
        let mut channels = 2u8;
        let mut sample_rate = 44100u32;
        let mut bit_depth = 16u16;
        let mut data_size = 0u32;

        while i + 8 < data.len() {
            let chunk_id = &data[i..i + 4];
            let chunk_size =
                u32::from_le_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]);

            if chunk_id == b"fmt " && i + 24 <= data.len() {
                channels = u16::from_le_bytes([data[i + 10], data[i + 11]]) as u8;
                sample_rate =
                    u32::from_le_bytes([data[i + 12], data[i + 13], data[i + 14], data[i + 15]]);
                bit_depth = u16::from_le_bytes([data[i + 22], data[i + 23]]);
            } else if chunk_id == b"data" {
                data_size = chunk_size;
                break;
            }

            i += 8 + chunk_size as usize;
            if chunk_size % 2 == 1 {
                i += 1;
            }
        }

        let bytes_per_sample = (bit_depth / 8) as u32 * channels as u32;
        let duration_seconds = if bytes_per_sample > 0 && sample_rate > 0 {
            data_size as f32 / (sample_rate as f32 * bytes_per_sample as f32)
        } else {
            0.0
        };

        Ok(AudioData {
            duration_seconds,
            sample_rate,
            bit_depth,
            channels,
            format: "WAV".to_string(),
        })
    }

    fn parse_ogg_header(&self, data: &[u8]) -> ContentResult<AudioData> {
        if data.len() < 58 {
            return Ok(AudioData {
                duration_seconds: 0.0,
                sample_rate: 44100,
                bit_depth: 16,
                channels: 2,
                format: "OGG".to_string(),
            });
        }

        let page_segments = data[26] as usize;
        let segment_table_end = 27 + page_segments;

        if data.len() < segment_table_end + 30 {
            return Ok(AudioData {
                duration_seconds: 0.0,
                sample_rate: 44100,
                bit_depth: 16,
                channels: 2,
                format: "OGG".to_string(),
            });
        }

        let vorbis_header_start = segment_table_end;

        if data.len() > vorbis_header_start + 11
            && &data[vorbis_header_start + 1..vorbis_header_start + 7] == b"vorbis"
        {
            let channels = data[vorbis_header_start + 11];
            let sample_rate = u32::from_le_bytes([
                data[vorbis_header_start + 12],
                data[vorbis_header_start + 13],
                data[vorbis_header_start + 14],
                data[vorbis_header_start + 15],
            ]);

            let estimated_duration = (data.len() as f32) / 16000.0;

            return Ok(AudioData {
                duration_seconds: estimated_duration,
                sample_rate,
                bit_depth: 16,
                channels,
                format: "OGG".to_string(),
            });
        }

        let estimated_duration = (data.len() as f32) / 16000.0;
        Ok(AudioData {
            duration_seconds: estimated_duration,
            sample_rate: 44100,
            bit_depth: 16,
            channels: 2,
            format: "OGG".to_string(),
        })
    }

    fn parse_flac_header(&self, data: &[u8]) -> ContentResult<AudioData> {
        if data.len() < 42 {
            return Err(ContentError::ValidationError {
                reason: "Truncated FLAC header".to_string(),
            });
        }

        let sample_rate =
            (((data[18] as u32) << 12) | ((data[19] as u32) << 4) | ((data[20] as u32) >> 4))
                & 0xFFFFF;
        let channels = (((data[20] >> 1) & 0x07) + 1) as u8;
        let bit_depth = ((((data[20] & 0x01) << 4) | ((data[21] >> 4) & 0x0F)) + 1) as u16;

        let total_samples = (((data[21] as u64) & 0x0F) << 32)
            | ((data[22] as u64) << 24)
            | ((data[23] as u64) << 16)
            | ((data[24] as u64) << 8)
            | (data[25] as u64);

        let duration_seconds = if sample_rate > 0 {
            total_samples as f32 / sample_rate as f32
        } else {
            0.0
        };

        Ok(AudioData {
            duration_seconds,
            sample_rate,
            bit_depth,
            channels,
            format: "FLAC".to_string(),
        })
    }

    fn parse_mp3_header(&self, data: &[u8]) -> ContentResult<AudioData> {
        let mut offset = 0;

        if data.len() > 10 && &data[0..3] == b"ID3" {
            let tag_size = ((data[6] as usize & 0x7F) << 21)
                | ((data[7] as usize & 0x7F) << 14)
                | ((data[8] as usize & 0x7F) << 7)
                | (data[9] as usize & 0x7F);
            offset = 10 + tag_size;
        }

        while offset < data.len() - 4 {
            if data[offset] == 0xFF && (data[offset + 1] & 0xE0) == 0xE0 {
                let version_bits = (data[offset + 1] >> 3) & 0x03;
                let layer_bits = (data[offset + 1] >> 1) & 0x03;
                let bitrate_index = (data[offset + 2] >> 4) & 0x0F;
                let sample_rate_index = (data[offset + 2] >> 2) & 0x03;
                let channel_mode = (data[offset + 3] >> 6) & 0x03;

                let sample_rate = match (version_bits, sample_rate_index) {
                    (3, 0) => 44100,
                    (3, 1) => 48000,
                    (3, 2) => 32000,
                    (2, 0) => 22050,
                    (2, 1) => 24000,
                    (2, 2) => 16000,
                    (0, 0) => 11025,
                    (0, 1) => 12000,
                    (0, 2) => 8000,
                    _ => 44100,
                };

                let channels = if channel_mode == 3 { 1 } else { 2 };

                let estimated_duration = (data.len() - offset) as f32 / 16000.0;

                return Ok(AudioData {
                    duration_seconds: estimated_duration,
                    sample_rate,
                    bit_depth: 16,
                    channels,
                    format: "MP3".to_string(),
                });
            }
            offset += 1;
        }

        let estimated_duration = data.len() as f32 / 16000.0;
        Ok(AudioData {
            duration_seconds: estimated_duration,
            sample_rate: 44100,
            bit_depth: 16,
            channels: 2,
            format: "MP3".to_string(),
        })
    }

    fn parse_aiff_header(&self, data: &[u8]) -> ContentResult<AudioData> {
        let mut i = 12;
        let mut channels = 2u8;
        let mut sample_rate = 44100u32;
        let mut bit_depth = 16u16;
        let mut num_frames = 0u32;

        while i + 8 < data.len() {
            let chunk_id = &data[i..i + 4];
            let chunk_size =
                u32::from_be_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]);

            if chunk_id == b"COMM" && i + 26 <= data.len() {
                channels = u16::from_be_bytes([data[i + 8], data[i + 9]]) as u8;
                num_frames =
                    u32::from_be_bytes([data[i + 10], data[i + 11], data[i + 12], data[i + 13]]);
                bit_depth = u16::from_be_bytes([data[i + 14], data[i + 15]]);

                let exp = u16::from_be_bytes([data[i + 16], data[i + 17]]);
                let mantissa =
                    u32::from_be_bytes([data[i + 18], data[i + 19], data[i + 20], data[i + 21]]);
                let sign = if exp & 0x8000 != 0 { -1.0 } else { 1.0 };
                let exp_val = (exp & 0x7FFF) as i32 - 16383;
                sample_rate =
                    (sign * (mantissa as f64 / (1u64 << 31) as f64) * 2f64.powi(exp_val)) as u32;
                break;
            }

            i += 8 + chunk_size as usize;
            if chunk_size % 2 == 1 {
                i += 1;
            }
        }

        let duration_seconds = if sample_rate > 0 {
            num_frames as f32 / sample_rate as f32
        } else {
            0.0
        };

        Ok(AudioData {
            duration_seconds,
            sample_rate,
            bit_depth,
            channels,
            format: "AIFF".to_string(),
        })
    }

    fn optimize_polygon_count(&self, original: u32, reduction: f32) -> u32 {
        ((original as f32) * (1.0 - reduction)) as u32
    }

    async fn generate_lods(
        &self,
        model_data: &Model3DData,
        file_path: &Path,
    ) -> ContentResult<Vec<LevelOfDetail>> {
        let base_polygon_count = model_data.polygon_count;
        let base_file_size = tokio::fs::metadata(file_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        let lod_configs = [
            (1, 0.50, 10.0),
            (2, 0.25, 25.0),
            (3, 0.10, 50.0),
            (4, 0.05, 100.0),
        ];

        let stem = file_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "model".to_string());
        let ext = file_path
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "obj".to_string());
        let parent = file_path.parent().unwrap_or(Path::new("."));

        let mut lods = Vec::new();

        for (level, reduction_factor, distance) in lod_configs.iter() {
            let polygon_count = ((base_polygon_count as f32) * reduction_factor) as u32;

            if polygon_count < 100 {
                break;
            }

            let file_size = ((base_file_size as f64) * (*reduction_factor as f64 * 1.2)) as u64;

            let lod_filename = format!("{}_lod{}.{}", stem, level, ext);
            let lod_path = parent.join(&lod_filename);

            lods.push(LevelOfDetail {
                lod_level: *level,
                polygon_count,
                distance_threshold: *distance,
                file_size,
                file_path: lod_path,
            });
        }

        tracing::debug!(
            "Generated {} LOD levels for model with {} base polygons",
            lods.len(),
            base_polygon_count
        );

        Ok(lods)
    }

    fn calculate_bounding_box(&self, model_data: &Model3DData) -> BoundingBox3D {
        if model_data.vertices.is_empty() {
            let scale = (model_data.polygon_count as f32 / 1000.0).sqrt().max(1.0);
            return BoundingBox3D {
                min_x: -scale,
                min_y: -scale,
                min_z: -scale,
                max_x: scale,
                max_y: scale,
                max_z: scale,
            };
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;

        for vertex in &model_data.vertices {
            min_x = min_x.min(vertex.0);
            min_y = min_y.min(vertex.1);
            min_z = min_z.min(vertex.2);
            max_x = max_x.max(vertex.0);
            max_y = max_y.max(vertex.1);
            max_z = max_z.max(vertex.2);
        }

        BoundingBox3D {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    fn calculate_texture_quality(&self, texture_data: &TextureData, compression: f32) -> f32 {
        let mut score = 100.0f32;

        let resolution_score = match (texture_data.width, texture_data.height) {
            (w, h) if w >= 2048 && h >= 2048 => 100.0,
            (w, h) if w >= 1024 && h >= 1024 => 90.0,
            (w, h) if w >= 512 && h >= 512 => 80.0,
            (w, h) if w >= 256 && h >= 256 => 70.0,
            _ => 60.0,
        };

        let format_score = match texture_data.format.as_str() {
            "PNG" => 95.0,
            "WEBP" => 90.0,
            "JP2" => 92.0,
            "JPEG" => 75.0,
            "TGA" => 85.0,
            "GIF" => 60.0,
            _ => 70.0,
        };

        let bit_depth_score = match texture_data.bit_depth {
            16 => 100.0,
            8 => 90.0,
            _ => 80.0,
        };

        let alpha_bonus = if texture_data.has_alpha { 5.0 } else { 0.0 };

        let compression_penalty = if compression > 0.8 {
            20.0
        } else if compression > 0.5 {
            10.0
        } else {
            0.0
        };

        let is_power_of_two =
            texture_data.width.is_power_of_two() && texture_data.height.is_power_of_two();
        let pot_bonus = if is_power_of_two { 5.0 } else { 0.0 };

        score = (resolution_score * 0.35 + format_score * 0.25 + bit_depth_score * 0.15 + 25.0)
            + alpha_bonus
            + pot_bonus
            - compression_penalty;

        score.clamp(0.0, 100.0)
    }

    fn calculate_audio_quality(&self, audio_data: &AudioData, compression: f32) -> f32 {
        let mut score = 100.0f32;

        let sample_rate_score = match audio_data.sample_rate {
            sr if sr >= 96000 => 100.0,
            sr if sr >= 48000 => 95.0,
            sr if sr >= 44100 => 90.0,
            sr if sr >= 22050 => 75.0,
            sr if sr >= 16000 => 65.0,
            _ => 50.0,
        };

        let bit_depth_score = match audio_data.bit_depth {
            24 => 100.0,
            16 => 90.0,
            8 => 60.0,
            _ => 70.0,
        };

        let format_score = match audio_data.format.as_str() {
            "FLAC" => 100.0,
            "WAV" | "AIFF" => 95.0,
            "OGG" => 85.0,
            "MP3" => 75.0,
            _ => 70.0,
        };

        let channel_score = match audio_data.channels {
            c if c >= 6 => 100.0,
            2 => 90.0,
            1 => 75.0,
            _ => 80.0,
        };

        let duration_score =
            if audio_data.duration_seconds > 0.0 && audio_data.duration_seconds <= 60.0 {
                100.0
            } else if audio_data.duration_seconds <= 180.0 {
                90.0
            } else if audio_data.duration_seconds <= 300.0 {
                80.0
            } else {
                70.0
            };

        let compression_penalty = if compression > 0.8 {
            15.0
        } else if compression > 0.5 {
            8.0
        } else {
            0.0
        };

        score = (sample_rate_score * 0.30
            + bit_depth_score * 0.20
            + format_score * 0.25
            + channel_score * 0.10
            + duration_score * 0.15)
            - compression_penalty;

        score.clamp(0.0, 100.0)
    }

    fn select_optimal_format(&self, _original: &str, quality: &ContentQuality) -> String {
        match quality {
            ContentQuality::Ultra | ContentQuality::High => "PNG".to_string(),
            ContentQuality::Medium => "WEBP".to_string(),
            ContentQuality::Low => "JPEG".to_string(),
            ContentQuality::Custom { .. } => "WEBP".to_string(),
        }
    }

    fn select_optimal_audio_format(&self, _original: &str, quality: &ContentQuality) -> String {
        match quality {
            ContentQuality::Ultra | ContentQuality::High => "FLAC".to_string(),
            ContentQuality::Medium => "OGG".to_string(),
            ContentQuality::Low => "MP3".to_string(),
            ContentQuality::Custom { .. } => "OGG".to_string(),
        }
    }

    fn calculate_mipmap_levels(&self, width: u32, height: u32) -> u8 {
        let max_dimension = width.max(height);
        (max_dimension as f32).log2().floor() as u8 + 1
    }

    async fn validate_3d_model(
        &self,
        file_path: &Path,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        recommendations: &mut Vec<String>,
    ) -> ContentResult<()> {
        let data = tokio::fs::read(file_path).await?;
        if data.is_empty() {
            errors.push("3D model file is empty".to_string());
            return Ok(());
        }

        let extension = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let format_valid = match extension.as_str() {
            "obj" => {
                data.len() > 10
                    && (data.starts_with(b"#")
                        || data.starts_with(b"v ")
                        || data.starts_with(b"mtllib"))
            }
            "fbx" => {
                data.len() >= 23
                    && (&data[0..20] == b"Kaydara FBX Binary  " || data.starts_with(b"; FBX"))
            }
            "gltf" | "glb" => {
                if extension == "glb" {
                    data.len() >= 4 && &data[0..4] == b"glTF"
                } else {
                    data.len() > 1
                        && (data[0] == b'{'
                            || String::from_utf8_lossy(&data[..100.min(data.len())])
                                .contains("\"asset\""))
                }
            }
            "dae" => String::from_utf8_lossy(&data[..500.min(data.len())]).contains("COLLADA"),
            "stl" => data.len() > 5 && (data.starts_with(b"solid") || data.len() > 84),
            _ => false,
        };

        if !format_valid {
            errors.push(format!(
                "Invalid or unrecognized 3D model format: {}",
                extension
            ));
        }

        let file_size = data.len() as u64;
        if file_size > 50 * 1024 * 1024 {
            warnings.push(format!(
                "Large 3D model file ({:.1} MB) may cause performance issues",
                file_size as f64 / 1024.0 / 1024.0
            ));
        }

        if file_size > self.config.max_file_size {
            errors.push(format!("3D model exceeds maximum file size limit"));
        }

        if extension == "obj" || extension == "fbx" {
            recommendations
                .push("Consider converting to glTF/GLB for better web compatibility".to_string());
        }

        recommendations.push("Consider reducing polygon count for better performance".to_string());
        Ok(())
    }

    async fn validate_texture(
        &self,
        file_path: &Path,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        recommendations: &mut Vec<String>,
    ) -> ContentResult<()> {
        let data = tokio::fs::read(file_path).await?;
        if data.is_empty() {
            errors.push("Texture file is empty".to_string());
            return Ok(());
        }

        let (format, dimensions) = if data.len() >= 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
            let width = if data.len() >= 24 {
                u32::from_be_bytes([data[16], data[17], data[18], data[19]])
            } else {
                0
            };
            let height = if data.len() >= 24 {
                u32::from_be_bytes([data[20], data[21], data[22], data[23]])
            } else {
                0
            };
            ("PNG", (width, height))
        } else if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
            let mut width = 0u32;
            let mut height = 0u32;
            let mut i = 2;
            while i < data.len() - 8 {
                if data[i] == 0xFF {
                    let marker = data[i + 1];
                    if marker >= 0xC0 && marker <= 0xCF && marker != 0xC4 && marker != 0xC8 {
                        height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                        width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                        break;
                    }
                    let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                    i += 2 + length;
                } else {
                    i += 1;
                }
            }
            ("JPEG", (width, height))
        } else if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            ("WEBP", (0, 0))
        } else if data.len() >= 2 && data[0] == 0xFF && data[1] == 0x4F {
            ("JPEG2000", (0, 0))
        } else if data.len() >= 18 {
            ("TGA_OR_UNKNOWN", (0, 0))
        } else {
            errors.push("Unknown or invalid texture format".to_string());
            return Ok(());
        };

        let (max_width, max_height) = self.config.validation_rules.max_texture_resolution;
        if dimensions.0 > max_width || dimensions.1 > max_height {
            errors.push(format!(
                "Texture resolution {}x{} exceeds maximum {}x{}",
                dimensions.0, dimensions.1, max_width, max_height
            ));
        } else if dimensions.0 > 2048 || dimensions.1 > 2048 {
            warnings.push(format!(
                "Large texture resolution {}x{} may impact performance",
                dimensions.0, dimensions.1
            ));
        }

        let file_size = data.len() as u64;
        if file_size > 10 * 1024 * 1024 {
            warnings.push(format!(
                "Large texture file ({:.1} MB)",
                file_size as f64 / 1024.0 / 1024.0
            ));
        }

        if format == "PNG" || format == "TGA_OR_UNKNOWN" {
            recommendations
                .push("Consider using JPEG2000 (J2K) for better SL/OS compatibility".to_string());
        }

        if format != "WEBP" {
            recommendations
                .push("Use WEBP format for better compression in web contexts".to_string());
        }

        Ok(())
    }

    async fn validate_audio(
        &self,
        file_path: &Path,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        recommendations: &mut Vec<String>,
    ) -> ContentResult<()> {
        let data = tokio::fs::read(file_path).await?;
        if data.is_empty() {
            errors.push("Audio file is empty".to_string());
            return Ok(());
        }

        let format = if data.len() >= 4 && &data[0..4] == b"OggS" {
            "OGG"
        } else if data.len() >= 4 && &data[0..4] == b"RIFF" {
            "WAV"
        } else if data.len() >= 4 && &data[0..4] == b"fLaC" {
            "FLAC"
        } else if data.len() >= 3
            && (&data[0..3] == b"ID3" || (data[0] == 0xFF && (data[1] & 0xE0) == 0xE0))
        {
            "MP3"
        } else if data.len() >= 4 && &data[0..4] == b"FORM" {
            "AIFF"
        } else {
            errors.push("Unknown or invalid audio format".to_string());
            return Ok(());
        };

        let file_size = data.len() as u64;
        let estimated_duration = match format {
            "WAV" => file_size as f64 / (44100.0 * 2.0 * 2.0),
            "MP3" => file_size as f64 / (16000.0),
            "OGG" => file_size as f64 / (12000.0),
            _ => file_size as f64 / (20000.0),
        };

        let max_duration = self.config.validation_rules.max_audio_duration as f64;
        if estimated_duration > max_duration {
            errors.push(format!(
                "Audio duration (~{:.0}s) may exceed maximum allowed ({:.0}s)",
                estimated_duration, max_duration
            ));
        }

        if file_size > 10 * 1024 * 1024 {
            warnings.push(format!(
                "Large audio file ({:.1} MB)",
                file_size as f64 / 1024.0 / 1024.0
            ));
        }

        if format != "OGG" {
            recommendations.push(
                "Consider using OGG format for better streaming in virtual worlds".to_string(),
            );
        }

        if format == "WAV" {
            recommendations.push(
                "WAV files are uncompressed - consider converting to OGG for smaller file size"
                    .to_string(),
            );
        }

        Ok(())
    }

    async fn validate_script(
        &self,
        file_path: &Path,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        recommendations: &mut Vec<String>,
    ) -> ContentResult<()> {
        let content =
            tokio::fs::read_to_string(file_path)
                .await
                .map_err(|e| ContentError::ImportFailed {
                    reason: format!("Cannot read script: {}", e),
                })?;

        if content.is_empty() {
            errors.push("Script file is empty".to_string());
            return Ok(());
        }

        let has_default_state = content.contains("default") && content.contains("state_entry");
        let has_lsl_functions = content.contains("llSay")
            || content.contains("llListen")
            || content.contains("llGetOwner")
            || content.contains("llSetText")
            || content.contains("llDie")
            || content.contains("llRequestPermissions");

        if !has_default_state && has_lsl_functions {
            warnings.push("Script uses LSL functions but may be missing default state".to_string());
        }

        if !has_default_state && !has_lsl_functions {
            warnings.push("Script content could not be validated as LSL".to_string());
        }

        let dangerous_functions = [
            "llEmail",
            "llHTTPRequest",
            "llLoadURL",
            "llMapDestination",
            "llTeleportAgent",
            "llGiveInventory",
            "llGiveMoney",
            "llInstantMessage",
        ];
        for func in &dangerous_functions {
            if content.contains(func) {
                warnings.push(format!(
                    "Script uses potentially sensitive function: {}",
                    func
                ));
            }
        }

        if content.len() > 64 * 1024 {
            warnings.push(format!(
                "Large script ({} bytes) may impact compilation time",
                content.len()
            ));
        }

        if has_default_state && has_lsl_functions {
            recommendations.push("Script appears to be well-formed LSL".to_string());
        }

        let infinite_loop_pattern = content.contains("while(TRUE)")
            || content.contains("while(1)")
            || content.contains("for(;;)");
        if infinite_loop_pattern {
            warnings.push(
                "Script may contain infinite loop patterns - ensure proper exit conditions"
                    .to_string(),
            );
        }

        Ok(())
    }

    async fn security_scan(
        &self,
        file_path: &Path,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> ContentResult<()> {
        let data = tokio::fs::read(file_path).await?;

        let executable_signatures: &[&[u8]] = &[
            b"MZ",
            b"\x7FELF",
            b"\xCA\xFE\xBA\xBE",
            b"\xCE\xFA\xED\xFE",
            b"\xCF\xFA\xED\xFE",
            b"#!",
        ];

        for sig in executable_signatures {
            if data.len() >= sig.len() && &data[..sig.len()] == *sig {
                errors.push(
                    "File appears to be an executable - not allowed for content upload".to_string(),
                );
                return Ok(());
            }
        }

        if file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .as_deref()
            == Some("lsl")
            || file_path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .as_deref()
                == Some("txt")
        {
            let content = String::from_utf8_lossy(&data);

            let suspicious_patterns = [
                "eval(",
                "exec(",
                "system(",
                "shell_exec(",
                "<script>",
                "javascript:",
                "data:text/html",
                "../",
                "..\\",
                "/etc/passwd",
                "cmd.exe",
            ];

            for pattern in &suspicious_patterns {
                if content.to_lowercase().contains(&pattern.to_lowercase()) {
                    warnings.push(format!(
                        "Potentially suspicious pattern detected: {}",
                        pattern
                    ));
                }
            }
        }

        let metadata = tokio::fs::metadata(file_path).await?;
        if metadata.len() > 500 * 1024 * 1024 {
            warnings.push("Very large file may indicate malicious content".to_string());
        }

        Ok(())
    }

    async fn calculate_performance_score(
        &self,
        content_type: &ContentType,
        file_path: &Path,
    ) -> ContentResult<f32> {
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len();

        let (optimal_size, weight) = match content_type {
            ContentType::Model3D => (5 * 1024 * 1024, 1.0),
            ContentType::Texture => (512 * 1024, 0.8),
            ContentType::Audio => (1 * 1024 * 1024, 0.7),
            ContentType::Script => (16 * 1024, 0.5),
            ContentType::Animation => (256 * 1024, 0.6),
            _ => (1 * 1024 * 1024, 0.5),
        };

        let size_ratio = file_size as f64 / optimal_size as f64;
        let size_score = if size_ratio <= 1.0 {
            100.0
        } else if size_ratio <= 2.0 {
            100.0 - ((size_ratio - 1.0) * 20.0)
        } else if size_ratio <= 5.0 {
            80.0 - ((size_ratio - 2.0) * 10.0)
        } else {
            (50.0 - ((size_ratio - 5.0) * 5.0)).max(10.0)
        };

        let extension = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let format_bonus = match (content_type, extension.as_str()) {
            (ContentType::Model3D, "glb" | "gltf") => 10.0,
            (ContentType::Texture, "webp" | "j2k" | "jp2") => 10.0,
            (ContentType::Audio, "ogg") => 10.0,
            _ => 0.0,
        };

        let final_score = ((size_score * weight + format_bonus) as f32)
            .min(100.0)
            .max(0.0);
        Ok(final_score)
    }

    async fn calculate_quality_score(
        &self,
        content_type: &ContentType,
        file_path: &Path,
    ) -> ContentResult<f32> {
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len();

        let base_score = 70.0f32;

        let extension = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let format_quality = match (content_type, extension.as_str()) {
            (ContentType::Texture, "png") => 15.0,
            (ContentType::Texture, "jpg" | "jpeg") => 10.0,
            (ContentType::Texture, "webp") => 12.0,
            (ContentType::Texture, "tga") => 15.0,
            (ContentType::Audio, "flac" | "wav") => 15.0,
            (ContentType::Audio, "ogg") => 12.0,
            (ContentType::Audio, "mp3") => 8.0,
            (ContentType::Model3D, "fbx") => 15.0,
            (ContentType::Model3D, "gltf" | "glb") => 14.0,
            (ContentType::Model3D, "obj") => 10.0,
            _ => 5.0,
        };

        let size_quality = if file_size > 0 {
            let kb = file_size as f64 / 1024.0;
            if kb < 10.0 {
                5.0
            } else if kb < 100.0 {
                10.0
            } else if kb < 1024.0 {
                15.0
            } else if kb < 10240.0 {
                10.0
            } else {
                5.0
            }
        } else {
            0.0
        };

        let final_score = (base_score + format_quality + size_quality)
            .min(100.0)
            .max(0.0);
        Ok(final_score)
    }
}

/// Content creation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCreationOptions {
    pub target_quality: ContentQuality,
    pub distribution_strategy: DistributionStrategy,
    pub auto_optimize: bool,
    pub generate_lods: bool,
    pub generate_mipmaps: bool,
    pub enable_drm: bool,
}

/// Data structures for content processing

#[derive(Debug, Clone)]
struct Model3DData {
    polygon_count: u32,
    vertex_count: u32,
    material_count: u32,
    texture_count: u32,
    animation_count: u32,
    vertices: Vec<(f32, f32, f32)>,
}

#[derive(Debug, Clone)]
struct TextureData {
    width: u32,
    height: u32,
    format: String,
    channels: u8,
    bit_depth: u8,
    has_alpha: bool,
}

#[derive(Debug, Clone)]
struct AudioData {
    duration_seconds: f32,
    sample_rate: u32,
    bit_depth: u16,
    channels: u8,
    format: String,
}

impl Default for ContentCreationConfig {
    fn default() -> Self {
        let mut allowed_formats = HashMap::new();
        allowed_formats.insert(
            ContentType::Model3D,
            vec!["obj".to_string(), "fbx".to_string(), "gltf".to_string()],
        );
        allowed_formats.insert(
            ContentType::Texture,
            vec!["png".to_string(), "jpg".to_string(), "jpeg".to_string()],
        );
        allowed_formats.insert(
            ContentType::Audio,
            vec!["wav".to_string(), "mp3".to_string(), "ogg".to_string()],
        );

        Self {
            max_file_size: 100 * 1024 * 1024, // 100 MB
            supported_types: vec![
                ContentType::Model3D,
                ContentType::Texture,
                ContentType::Audio,
                ContentType::Script,
                ContentType::Animation,
            ],
            auto_optimization: AutoOptimizationConfig {
                generate_quality_levels: true,
                target_qualities: vec![
                    ContentQuality::Ultra,
                    ContentQuality::High,
                    ContentQuality::Medium,
                    ContentQuality::Low,
                ],
                optimize_textures: true,
                generate_lods: true,
                compress_audio: true,
                web_optimization: true,
            },
            validation_rules: ValidationRules {
                max_polygons: 100000,
                max_texture_resolution: (4096, 4096),
                max_audio_duration: 300, // 5 minutes
                allowed_formats,
                security_scanning: true,
                content_rating_required: false,
            },
            default_permissions: ContentPermissions::default(),
            upload_directory: PathBuf::from("uploads"),
            temp_directory: PathBuf::from("temp"),
        }
    }
}

impl Default for ContentCreationOptions {
    fn default() -> Self {
        Self {
            target_quality: ContentQuality::High,
            distribution_strategy: DistributionStrategy::OnDemand,
            auto_optimize: true,
            generate_lods: true,
            generate_mipmaps: true,
            enable_drm: false,
        }
    }
}
