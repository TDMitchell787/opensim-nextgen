//! Unit tests for IAR/OAR archive operations

#[cfg(test)]
mod tests {
    use crate::archives::common::*;
    use crate::archives::job_manager::*;
    use uuid::Uuid;

    mod common_tests {
        use super::*;

        #[test]
        fn test_asset_type_from_i32() {
            assert_eq!(AssetType::from_i32(0).extension(), "_texture.jp2");
            assert_eq!(AssetType::from_i32(1).extension(), "_sound.ogg");
            assert_eq!(AssetType::from_i32(10).extension(), "_script.lsl");
            assert_eq!(AssetType::from_i32(13).extension(), "_bodypart.txt");
            assert_eq!(AssetType::from_i32(999).extension(), ".bin"); // Unknown type
        }

        #[test]
        fn test_load_statistics_default() {
            let stats = LoadStatistics::default();
            assert_eq!(stats.assets_loaded, 0);
            assert_eq!(stats.folders_created, 0);
            assert_eq!(stats.items_created, 0);
            assert_eq!(stats.objects_created, 0);
            assert_eq!(stats.parcels_loaded, 0);
            assert!(!stats.terrain_loaded);
            assert_eq!(stats.elapsed_ms, 0);
        }

        #[test]
        fn test_save_statistics_default() {
            let stats = SaveStatistics::default();
            assert_eq!(stats.assets_saved, 0);
            assert_eq!(stats.folders_saved, 0);
            assert_eq!(stats.items_saved, 0);
            assert_eq!(stats.objects_saved, 0);
            assert_eq!(stats.parcels_saved, 0);
            assert!(!stats.terrain_saved);
            assert_eq!(stats.archive_size_bytes, 0);
            assert_eq!(stats.elapsed_ms, 0);
        }

        #[test]
        fn test_archive_paths() {
            assert_eq!(paths::ARCHIVE_XML, "archive.xml");
            assert_eq!(paths::INVENTORY_PATH, "inventory/");
            assert_eq!(paths::ASSETS_PATH, "assets/");
            assert_eq!(paths::OBJECTS_PATH, "objects/");
            assert_eq!(paths::TERRAINS_PATH, "terrains/");
            assert_eq!(paths::LANDDATA_PATH, "landdata/");
            assert_eq!(paths::SETTINGS_PATH, "settings/");
            assert_eq!(paths::FOLDER_METADATA, "__folder_metadata.xml");
        }

        #[test]
        fn test_extract_asset_uuid_from_path() {
            // Format: {uuid}_type.ext
            let path1 = "assets/550e8400-e29b-41d4-a716-446655440000_texture.jp2";
            let result1 = extract_asset_uuid_from_path(path1);
            assert!(result1.is_some());
            assert_eq!(
                result1.unwrap().to_string(),
                "550e8400-e29b-41d4-a716-446655440000"
            );

            let path2 = "assets/invalid_texture.jp2";
            let result2 = extract_asset_uuid_from_path(path2);
            assert!(result2.is_none());

            let path3 = "550e8400-e29b-41d4-a716-446655440000_sound.ogg";
            let result3 = extract_asset_uuid_from_path(path3);
            assert!(result3.is_some());
        }
    }

    mod job_manager_tests {
        use super::*;
        use std::path::PathBuf;

        #[tokio::test]
        async fn test_job_manager_create_job() {
            let manager = ArchiveJobManager::default();

            let job_type = JobType::IarLoad {
                user_id: Uuid::new_v4(),
                target_folder: None,
                merge: false,
                source_path: PathBuf::from("/test/path.iar"),
            };

            let job_id = manager.create_job(job_type).await;
            assert!(!job_id.is_nil());

            let job = manager.get_job(&job_id).await;
            assert!(job.is_some());

            let job = job.unwrap();
            assert_eq!(job.id, job_id);
            assert!(matches!(job.status, JobStatus::Queued));
        }

        #[tokio::test]
        async fn test_job_manager_start_job() {
            let manager = ArchiveJobManager::default();

            let job_type = JobType::IarSave {
                user_id: Uuid::new_v4(),
                folder_id: None,
                include_assets: true,
                output_path: PathBuf::from("/test/output.iar"),
            };

            let job_id = manager.create_job(job_type).await;
            manager.start_job(&job_id).await;

            let job = manager.get_job(&job_id).await.unwrap();
            assert!(matches!(job.status, JobStatus::Running));
            assert!(job.started_at.is_some());
        }

        #[tokio::test]
        async fn test_job_manager_update_progress() {
            let manager = ArchiveJobManager::default();

            let job_type = JobType::OarLoad {
                region_id: Uuid::new_v4(),
                source_path: PathBuf::from("/test/region.oar"),
                merge: false,
                load_terrain: true,
                load_objects: true,
                load_parcels: true,
            };

            let job_id = manager.create_job(job_type).await;
            manager.start_job(&job_id).await;
            manager
                .update_progress(&job_id, 0.5, Some("Loading objects...".to_string()))
                .await;

            let job = manager.get_job(&job_id).await.unwrap();
            assert!((job.progress - 0.5).abs() < f32::EPSILON);
            assert_eq!(job.progress_message, Some("Loading objects...".to_string()));
        }

        #[tokio::test]
        async fn test_job_manager_complete_job() {
            let manager = ArchiveJobManager::default();

            let job_type = JobType::OarSave {
                region_id: Uuid::new_v4(),
                output_path: PathBuf::from("/test/output.oar"),
                include_assets: true,
                include_terrain: true,
                include_objects: true,
                include_parcels: true,
            };

            let job_id = manager.create_job(job_type).await;
            manager.start_job(&job_id).await;

            let result = JobResult::OarSave {
                assets_saved: 100,
                objects_saved: 50,
                parcels_saved: 2,
                terrain_saved: true,
                download_path: PathBuf::from("/test/output.oar"),
            };

            manager.complete_job(&job_id, result).await;

            let job = manager.get_job(&job_id).await.unwrap();
            assert!(matches!(job.status, JobStatus::Completed));
            assert!(job.completed_at.is_some());
            assert!(job.result.is_some());
        }

        #[tokio::test]
        async fn test_job_manager_fail_job() {
            let manager = ArchiveJobManager::default();

            let job_type = JobType::IarLoad {
                user_id: Uuid::new_v4(),
                target_folder: None,
                merge: false,
                source_path: PathBuf::from("/nonexistent.iar"),
            };

            let job_id = manager.create_job(job_type).await;
            manager.start_job(&job_id).await;
            manager
                .fail_job(&job_id, "File not found".to_string())
                .await;

            let job = manager.get_job(&job_id).await.unwrap();
            assert!(matches!(job.status, JobStatus::Failed));
            assert_eq!(job.error, Some("File not found".to_string()));
        }

        #[tokio::test]
        async fn test_job_manager_cancel_job() {
            let manager = ArchiveJobManager::default();

            let job_type = JobType::IarSave {
                user_id: Uuid::new_v4(),
                folder_id: None,
                include_assets: true,
                output_path: PathBuf::from("/test/output.iar"),
            };

            let job_id = manager.create_job(job_type).await;
            let cancelled = manager.cancel_job(&job_id).await;

            assert!(cancelled);

            let job = manager.get_job(&job_id).await.unwrap();
            assert!(matches!(job.status, JobStatus::Cancelled));
        }

        #[tokio::test]
        async fn test_job_manager_cannot_cancel_completed_job() {
            let manager = ArchiveJobManager::default();

            let job_type = JobType::IarLoad {
                user_id: Uuid::new_v4(),
                target_folder: None,
                merge: false,
                source_path: PathBuf::from("/test.iar"),
            };

            let job_id = manager.create_job(job_type).await;
            manager.start_job(&job_id).await;
            manager
                .complete_job(
                    &job_id,
                    JobResult::IarLoad {
                        assets_loaded: 10,
                        folders_created: 5,
                        items_created: 20,
                    },
                )
                .await;

            let cancelled = manager.cancel_job(&job_id).await;
            assert!(!cancelled);
        }

        #[tokio::test]
        async fn test_job_manager_get_active_jobs() {
            let manager = ArchiveJobManager::default();

            let job_type1 = JobType::IarLoad {
                user_id: Uuid::new_v4(),
                target_folder: None,
                merge: false,
                source_path: PathBuf::from("/test1.iar"),
            };
            let job_type2 = JobType::OarSave {
                region_id: Uuid::new_v4(),
                output_path: PathBuf::from("/test2.oar"),
                include_assets: true,
                include_terrain: true,
                include_objects: true,
                include_parcels: true,
            };

            let _job1 = manager.create_job(job_type1).await;
            let _job2 = manager.create_job(job_type2).await;

            let jobs = manager.get_active_jobs().await;
            assert_eq!(jobs.len(), 2);
        }
    }
}
