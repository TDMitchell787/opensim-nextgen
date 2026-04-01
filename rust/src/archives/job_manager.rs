//! Background job manager for archive operations
//!
//! Archives can be large and take time to process.
//! This module provides async job management with progress tracking.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Job status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Type of archive job
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    IarLoad {
        user_id: Uuid,
        target_folder: Option<Uuid>,
        merge: bool,
        source_path: PathBuf,
    },
    IarSave {
        user_id: Uuid,
        folder_id: Option<Uuid>,
        include_assets: bool,
        output_path: PathBuf,
    },
    OarLoad {
        region_id: Uuid,
        source_path: PathBuf,
        merge: bool,
        load_terrain: bool,
        load_objects: bool,
        load_parcels: bool,
    },
    OarSave {
        region_id: Uuid,
        output_path: PathBuf,
        include_assets: bool,
        include_terrain: bool,
        include_objects: bool,
        include_parcels: bool,
    },
}

/// Result data from a completed job
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JobResult {
    IarLoad {
        assets_loaded: u32,
        folders_created: u32,
        items_created: u32,
    },
    IarSave {
        assets_saved: u32,
        folders_saved: u32,
        items_saved: u32,
        download_path: PathBuf,
    },
    OarLoad {
        assets_loaded: u32,
        objects_created: u32,
        parcels_loaded: u32,
        terrain_loaded: bool,
    },
    OarSave {
        assets_saved: u32,
        objects_saved: u32,
        parcels_saved: u32,
        terrain_saved: bool,
        download_path: PathBuf,
    },
}

/// An archive processing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveJob {
    pub id: Uuid,
    pub job_type: JobType,
    pub status: JobStatus,
    pub progress: f32,
    pub progress_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<JobResult>,
    pub error: Option<String>,
}

impl ArchiveJob {
    pub fn new(job_type: JobType) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type,
            status: JobStatus::Queued,
            progress: 0.0,
            progress_message: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }

    pub fn elapsed_seconds(&self) -> Option<i64> {
        self.started_at.map(|start| {
            let end = self.completed_at.unwrap_or_else(Utc::now);
            (end - start).num_seconds()
        })
    }
}

/// Manager for archive jobs
pub struct ArchiveJobManager {
    jobs: Arc<RwLock<HashMap<Uuid, ArchiveJob>>>,
    max_concurrent: usize,
}

impl ArchiveJobManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent,
        }
    }

    /// Create a new job and return its ID
    pub async fn create_job(&self, job_type: JobType) -> Uuid {
        let job = ArchiveJob::new(job_type);
        let id = job.id;

        let mut jobs = self.jobs.write().await;
        jobs.insert(id, job);

        id
    }

    /// Get job status
    pub async fn get_job(&self, id: &Uuid) -> Option<ArchiveJob> {
        let jobs = self.jobs.read().await;
        jobs.get(id).cloned()
    }

    /// Update job progress
    pub async fn update_progress(&self, id: &Uuid, progress: f32, message: Option<String>) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(id) {
            job.progress = progress.clamp(0.0, 1.0);
            job.progress_message = message;
        }
    }

    /// Mark job as started
    pub async fn start_job(&self, id: &Uuid) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(id) {
            job.status = JobStatus::Running;
            job.started_at = Some(Utc::now());
        }
    }

    /// Mark job as completed with result
    pub async fn complete_job(&self, id: &Uuid, result: JobResult) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(id) {
            job.status = JobStatus::Completed;
            job.progress = 1.0;
            job.completed_at = Some(Utc::now());
            job.result = Some(result);
        }
    }

    /// Mark job as failed
    pub async fn fail_job(&self, id: &Uuid, error: String) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(id) {
            job.status = JobStatus::Failed;
            job.completed_at = Some(Utc::now());
            job.error = Some(error);
        }
    }

    /// Cancel a job
    pub async fn cancel_job(&self, id: &Uuid) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(id) {
            if job.status == JobStatus::Queued || job.status == JobStatus::Running {
                job.status = JobStatus::Cancelled;
                job.completed_at = Some(Utc::now());
                return true;
            }
        }
        false
    }

    /// Get all jobs for a user or region
    pub async fn get_jobs_for_entity(&self, entity_id: &Uuid) -> Vec<ArchiveJob> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| match &job.job_type {
                JobType::IarLoad { user_id, .. } => user_id == entity_id,
                JobType::IarSave { user_id, .. } => user_id == entity_id,
                JobType::OarLoad { region_id, .. } => region_id == entity_id,
                JobType::OarSave { region_id, .. } => region_id == entity_id,
            })
            .cloned()
            .collect()
    }

    /// Get all active (running or queued) jobs
    pub async fn get_active_jobs(&self) -> Vec<ArchiveJob> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| {
                job.status == JobStatus::Queued || job.status == JobStatus::Running
            })
            .cloned()
            .collect()
    }

    /// Clean up old completed jobs (older than given hours)
    pub async fn cleanup_old_jobs(&self, max_age_hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);

        let mut jobs = self.jobs.write().await;
        jobs.retain(|_, job| {
            if let Some(completed) = job.completed_at {
                completed > cutoff
            } else {
                true
            }
        });
    }

    /// Get count of running jobs
    pub async fn running_count(&self) -> usize {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| job.status == JobStatus::Running)
            .count()
    }

    /// Check if we can start a new job
    pub async fn can_start_job(&self) -> bool {
        self.running_count().await < self.max_concurrent
    }
}

impl Default for ArchiveJobManager {
    fn default() -> Self {
        Self::new(2) // Default to 2 concurrent archive operations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_job_lifecycle() {
        let manager = ArchiveJobManager::new(2);

        let job_type = JobType::IarLoad {
            user_id: Uuid::new_v4(),
            target_folder: None,
            merge: false,
            source_path: PathBuf::from("/tmp/test.iar"),
        };

        let job_id = manager.create_job(job_type).await;

        // Check initial state
        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Queued);

        // Start job
        manager.start_job(&job_id).await;
        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Running);

        // Update progress
        manager.update_progress(&job_id, 0.5, Some("Loading assets...".into())).await;
        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.progress, 0.5);

        // Complete job
        manager.complete_job(&job_id, JobResult::IarLoad {
            assets_loaded: 100,
            folders_created: 10,
            items_created: 50,
        }).await;

        let job = manager.get_job(&job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert_eq!(job.progress, 1.0);
    }
}
