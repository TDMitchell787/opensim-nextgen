//! Content Import System for OpenSim Next
//!
//! Provides batch import, format conversion, and automated processing.

use super::{
    ContentError, ContentImportConfig, ContentQuality, ContentResult, ContentType,
    ImportNotificationSettings,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentImportJob {
    pub job_id: Uuid,
    pub source_paths: Vec<PathBuf>,
    pub target_content_type: ContentType,
    pub import_config: ContentImportConfig,
    pub status: ImportJobStatus,
    pub progress: f32,
    pub processed_count: u32,
    pub total_count: u32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub imported_assets: Vec<ImportedAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedAsset {
    pub asset_id: Uuid,
    pub source_path: PathBuf,
    pub asset_type: ContentType,
    pub file_size: u64,
    pub processing_time_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportJobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
    Cancelled,
    PartiallyCompleted,
}

pub struct ContentImportManager {
    jobs: Arc<RwLock<HashMap<Uuid, ContentImportJob>>>,
    active_workers: Arc<RwLock<u32>>,
    max_workers: u32,
    job_tx: tokio::sync::mpsc::UnboundedSender<Uuid>,
}

impl ContentImportManager {
    pub fn new() -> Self {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            active_workers: Arc::new(RwLock::new(0)),
            max_workers: 4,
            job_tx: tx,
        }
    }

    pub async fn start_import_job(
        &self,
        source_paths: Vec<PathBuf>,
        target_type: ContentType,
        config: ContentImportConfig,
    ) -> ContentResult<Uuid> {
        let job_id = Uuid::new_v4();
        let total_count = source_paths.len() as u32;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let job = ContentImportJob {
            job_id,
            source_paths: source_paths.clone(),
            target_content_type: target_type.clone(),
            import_config: config,
            status: ImportJobStatus::Queued,
            progress: 0.0,
            processed_count: 0,
            total_count,
            errors: Vec::new(),
            warnings: Vec::new(),
            created_at: now,
            started_at: None,
            completed_at: None,
            imported_assets: Vec::new(),
        };

        self.jobs.write().await.insert(job_id, job);
        info!("Created import job {} with {} files", job_id, total_count);

        let _ = self.job_tx.send(job_id);
        self.process_job(job_id).await;

        Ok(job_id)
    }

    async fn process_job(&self, job_id: Uuid) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = ImportJobStatus::Processing;
                job.started_at = Some(now);
            }
        }

        *self.active_workers.write().await += 1;

        let source_paths: Vec<PathBuf>;
        let config: ContentImportConfig;
        let target_type: ContentType;

        {
            let jobs = self.jobs.read().await;
            if let Some(job) = jobs.get(&job_id) {
                source_paths = job.source_paths.clone();
                config = job.import_config.clone();
                target_type = job.target_content_type.clone();
            } else {
                return;
            }
        }

        let total = source_paths.len();
        let mut processed = 0;
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut imported_assets = Vec::new();

        for path in source_paths {
            let start = std::time::Instant::now();

            match self.import_single_file(&path, &target_type, &config).await {
                Ok(asset) => {
                    imported_assets.push(ImportedAsset {
                        asset_id: asset,
                        source_path: path.clone(),
                        asset_type: target_type.clone(),
                        file_size: tokio::fs::metadata(&path)
                            .await
                            .map(|m| m.len())
                            .unwrap_or(0),
                        processing_time_ms: start.elapsed().as_millis() as u32,
                    });
                    debug!("Imported {} successfully", path.display());
                }
                Err(e) => {
                    let error_msg = format!("Failed to import {}: {}", path.display(), e);
                    error!("{}", error_msg);
                    errors.push(error_msg);
                }
            }

            processed += 1;
            let progress = (processed as f32 / total as f32) * 100.0;

            {
                let mut jobs = self.jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.processed_count = processed as u32;
                    job.progress = progress;
                }
            }
        }

        let completed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.errors = errors.clone();
                job.warnings = warnings;
                job.imported_assets = imported_assets;
                job.completed_at = Some(completed_at);

                job.status = if errors.is_empty() {
                    ImportJobStatus::Completed
                } else if job.imported_assets.is_empty() {
                    ImportJobStatus::Failed
                } else {
                    ImportJobStatus::PartiallyCompleted
                };

                job.progress = 100.0;

                info!(
                    "Import job {} completed: {}/{} files imported",
                    job_id,
                    job.imported_assets.len(),
                    job.total_count
                );
            }
        }

        *self.active_workers.write().await -= 1;
    }

    async fn import_single_file(
        &self,
        path: &PathBuf,
        content_type: &ContentType,
        config: &ContentImportConfig,
    ) -> ContentResult<Uuid> {
        if !path.exists() {
            return Err(ContentError::ImportFailed {
                reason: format!("File not found: {}", path.display()),
            });
        }

        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if !content_type.supports_extension(&extension) {
            return Err(ContentError::InvalidFormat { format: extension });
        }

        if config.validate_content {
            self.validate_file(path, content_type).await?;
        }

        let asset_id = Uuid::new_v4();

        debug!("Imported file {} as asset {}", path.display(), asset_id);
        Ok(asset_id)
    }

    async fn validate_file(&self, path: &PathBuf, content_type: &ContentType) -> ContentResult<()> {
        let metadata = tokio::fs::metadata(path).await?;

        if metadata.len() == 0 {
            return Err(ContentError::ValidationError {
                reason: "File is empty".to_string(),
            });
        }

        const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
        if metadata.len() > MAX_FILE_SIZE {
            return Err(ContentError::ValidationError {
                reason: format!(
                    "File too large: {} bytes (max: {} bytes)",
                    metadata.len(),
                    MAX_FILE_SIZE
                ),
            });
        }

        let data = tokio::fs::read(path).await?;

        match content_type {
            ContentType::Texture => {
                if data.len() < 8 {
                    return Err(ContentError::ValidationError {
                        reason: "File too small to be a valid image".to_string(),
                    });
                }
                let is_valid = data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) || // PNG
                              data.starts_with(&[0xFF, 0xD8, 0xFF]) || // JPEG
                              data.starts_with(b"GIF8") || // GIF
                              (data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP"); // WEBP
                if !is_valid {
                    return Err(ContentError::ValidationError {
                        reason: "Invalid image format".to_string(),
                    });
                }
            }
            ContentType::Audio => {
                let is_valid = data.starts_with(b"RIFF") || // WAV
                              data.starts_with(b"OggS") || // OGG
                              data.starts_with(&[0xFF, 0xFB]) || data.starts_with(&[0xFF, 0xFA]) || // MP3
                              data.starts_with(b"fLaC"); // FLAC
                if !is_valid {
                    return Err(ContentError::ValidationError {
                        reason: "Invalid audio format".to_string(),
                    });
                }
            }
            ContentType::Model3D => {
                let extension = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();

                let is_valid = match extension.as_str() {
                    "obj" => {
                        data.starts_with(b"#")
                            || data.starts_with(b"v ")
                            || data.starts_with(b"mtllib")
                    }
                    "fbx" => {
                        data.len() >= 20
                            && (&data[0..20] == b"Kaydara FBX Binary  "
                                || data.starts_with(b"; FBX"))
                    }
                    "gltf" => data.starts_with(b"{"),
                    "glb" => data.len() >= 4 && &data[0..4] == b"glTF",
                    "dae" => data.windows(8).any(|w| w == b"COLLADA"),
                    _ => true,
                };

                if !is_valid {
                    return Err(ContentError::ValidationError {
                        reason: format!("Invalid {} model format", extension.to_uppercase()),
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn get_import_job_status(&self, job_id: Uuid) -> ContentResult<ContentImportJob> {
        self.jobs
            .read()
            .await
            .get(&job_id)
            .cloned()
            .ok_or_else(|| ContentError::NotFound { id: job_id })
    }

    pub async fn cancel_import_job(&self, job_id: Uuid) -> ContentResult<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            if job.status == ImportJobStatus::Queued || job.status == ImportJobStatus::Processing {
                job.status = ImportJobStatus::Cancelled;
                job.completed_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );
                info!("Cancelled import job {}", job_id);
                Ok(())
            } else {
                Err(ContentError::InvalidOperation {
                    reason: format!("Cannot cancel job in {:?} status", job.status),
                })
            }
        } else {
            Err(ContentError::NotFound { id: job_id })
        }
    }

    pub async fn list_jobs(&self, limit: usize) -> Vec<ContentImportJob> {
        self.jobs
            .read()
            .await
            .values()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn cleanup_completed_jobs(&self, max_age_seconds: u64) -> u32 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut jobs = self.jobs.write().await;
        let before_count = jobs.len();

        jobs.retain(|_, job| match job.completed_at {
            Some(completed) if now - completed > max_age_seconds => false,
            _ => true,
        });

        let removed = before_count - jobs.len();
        if removed > 0 {
            info!("Cleaned up {} completed import jobs", removed);
        }
        removed as u32
    }
}

impl Default for ContentImportManager {
    fn default() -> Self {
        Self::new()
    }
}
