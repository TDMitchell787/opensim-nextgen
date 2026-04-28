//! Archive API endpoints for IAR/OAR operations
//!
//! Provides REST API for loading and saving inventory (IAR) and region (OAR) archives.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::iar::{IarLoadOptions, IarReader, IarSaveOptions, IarWriter};
use super::job_manager::{ArchiveJobManager, JobResult, JobStatus, JobType};
use super::oar::{OarLoadOptions, OarReader, OarSaveOptions, OarWriter};

#[derive(Clone)]
pub struct ArchiveApiState {
    pub db_pool: PgPool,
    pub job_manager: Arc<ArchiveJobManager>,
    pub scene_objects:
        Option<Arc<parking_lot::RwLock<HashMap<u32, crate::udp::server::SceneObject>>>>,
    pub udp_socket: Option<Arc<tokio::net::UdpSocket>>,
    pub avatar_states:
        Option<Arc<parking_lot::RwLock<HashMap<Uuid, crate::udp::server::AvatarMovementState>>>>,
    pub next_prim_local_id: Option<Arc<std::sync::atomic::AtomicU32>>,
    pub reliability_manager: Option<Arc<crate::udp::reliability::ReliabilityManager>>,
}

impl ArchiveApiState {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            job_manager: Arc::new(ArchiveJobManager::default()),
            scene_objects: None,
            udp_socket: None,
            avatar_states: None,
            next_prim_local_id: None,
            reliability_manager: None,
        }
    }

    pub fn with_scene_objects(
        mut self,
        scene_objects: Arc<parking_lot::RwLock<HashMap<u32, crate::udp::server::SceneObject>>>,
    ) -> Self {
        self.scene_objects = Some(scene_objects);
        self
    }

    pub fn with_udp_socket(mut self, socket: Arc<tokio::net::UdpSocket>) -> Self {
        self.udp_socket = Some(socket);
        self
    }

    pub fn with_avatar_states(
        mut self,
        states: Arc<parking_lot::RwLock<HashMap<Uuid, crate::udp::server::AvatarMovementState>>>,
    ) -> Self {
        self.avatar_states = Some(states);
        self
    }

    pub fn with_next_prim_local_id(mut self, counter: Arc<std::sync::atomic::AtomicU32>) -> Self {
        self.next_prim_local_id = Some(counter);
        self
    }

    pub fn with_reliability_manager(
        mut self,
        manager: Arc<crate::udp::reliability::ReliabilityManager>,
    ) -> Self {
        self.reliability_manager = Some(manager);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveApiResponse {
    pub success: bool,
    pub message: String,
    pub job_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct LoadIarRequest {
    pub file_path: String,
    pub user_firstname: String,
    pub user_lastname: String,
    pub target_folder: Option<String>,
    pub merge: Option<bool>,
    pub create_user_if_missing: Option<bool>,
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    pub user_password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveIarRequest {
    pub output_path: String,
    pub user_firstname: String,
    pub user_lastname: String,
    pub folder_path: Option<String>,
    pub include_assets: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LoadOarRequest {
    pub file_path: String,
    pub region_name: String,
    pub merge: Option<bool>,
    pub displacement_x: Option<f32>,
    pub displacement_y: Option<f32>,
    pub displacement_z: Option<f32>,
    pub rotation_degrees: Option<f32>,
    pub force_terrain: Option<bool>,
    pub force_parcels: Option<bool>,
    pub default_user_firstname: Option<String>,
    pub default_user_lastname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveOarRequest {
    pub output_path: String,
    pub region_name: String,
    pub include_assets: Option<bool>,
    pub include_terrain: Option<bool>,
    pub include_objects: Option<bool>,
    pub include_parcels: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct JobQuery {
    pub limit: Option<i32>,
    pub status: Option<String>,
}

pub fn create_archive_api_router() -> Router<ArchiveApiState> {
    Router::new()
        .route("/admin/archives/iar/load", post(load_iar_endpoint))
        .route("/admin/archives/iar/save", post(save_iar_endpoint))
        .route("/admin/archives/oar/load", post(load_oar_endpoint))
        .route("/admin/archives/oar/save", post(save_oar_endpoint))
        .route("/admin/archives/oar/files", get(list_oar_files_endpoint))
        .route("/admin/archives/jobs", get(list_jobs_endpoint))
        .route("/admin/archives/jobs/:job_id", get(get_job_status_endpoint))
        .route(
            "/admin/archives/jobs/:job_id/cancel",
            post(cancel_job_endpoint),
        )
        .route("/admin/archives/health", get(archive_health_endpoint))
        .route("/admin/archives/region/clear", post(clear_region_endpoint))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

async fn load_iar_endpoint(
    State(state): State<ArchiveApiState>,
    Json(request): Json<LoadIarRequest>,
) -> impl IntoResponse {
    info!(
        "Archive API: Loading IAR from {} for user {} {}",
        request.file_path, request.user_firstname, request.user_lastname
    );

    if request.file_path.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ArchiveApiResponse {
                success: false,
                message: "File path cannot be empty".to_string(),
                job_id: None,
                data: None,
            }),
        );
    }

    if request.user_firstname.trim().is_empty() || request.user_lastname.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ArchiveApiResponse {
                success: false,
                message: "User first and last name are required".to_string(),
                job_id: None,
                data: None,
            }),
        );
    }

    if !std::path::Path::new(&request.file_path).exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ArchiveApiResponse {
                success: false,
                message: format!("File not found: {}", request.file_path),
                job_id: None,
                data: None,
            }),
        );
    }

    let user_id = match get_user_id(
        &state.db_pool,
        &request.user_firstname,
        &request.user_lastname,
    )
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            if request.create_user_if_missing.unwrap_or(false) {
                let new_id = request
                    .user_id
                    .as_ref()
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_else(Uuid::new_v4);
                let email = request
                    .user_email
                    .as_deref()
                    .unwrap_or("imported@archive.local");
                let password = request.user_password.as_deref().unwrap_or("changeme");
                match create_user_with_id(
                    &state.db_pool,
                    new_id,
                    &request.user_firstname,
                    &request.user_lastname,
                    email,
                    password,
                )
                .await
                {
                    Ok(()) => {
                        info!(
                            "Auto-created user {} {} ({}) for IAR import",
                            request.user_firstname, request.user_lastname, new_id
                        );
                        new_id
                    }
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ArchiveApiResponse {
                                success: false,
                                message: format!("Failed to create user: {}", e),
                                job_id: None,
                                data: None,
                            }),
                        )
                    }
                }
            } else {
                return (StatusCode::NOT_FOUND, Json(ArchiveApiResponse {
                    success: false, message: format!("User not found: {} {}. Set create_user_if_missing=true to auto-create.", request.user_firstname, request.user_lastname),
                    job_id: None, data: None,
                }));
            }
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ArchiveApiResponse {
                    success: false,
                    message: format!("Database error: {}", e),
                    job_id: None,
                    data: None,
                }),
            )
        }
    };

    let job_type = JobType::IarLoad {
        user_id,
        target_folder: None,
        merge: request.merge.unwrap_or(false),
        source_path: PathBuf::from(&request.file_path),
    };

    let job_id = state.job_manager.create_job(job_type).await;

    let db_pool = state.db_pool.clone();
    let job_manager = state.job_manager.clone();
    let file_path = request.file_path.clone();
    let merge = request.merge.unwrap_or(false);

    tokio::spawn(async move {
        job_manager.start_job(&job_id).await;
        job_manager
            .update_progress(&job_id, 0.1, Some("Loading IAR...".to_string()))
            .await;

        let reader = IarReader::new(db_pool);
        let options = IarLoadOptions {
            user_id,
            target_folder: None,
            merge,
            skip_existing_assets: true,
        };

        match reader.load(&file_path, options).await {
            Ok(result) => {
                job_manager
                    .complete_job(
                        &job_id,
                        JobResult::IarLoad {
                            assets_loaded: result.stats.assets_loaded,
                            folders_created: result.stats.folders_created,
                            items_created: result.stats.items_created,
                        },
                    )
                    .await;
            }
            Err(e) => {
                job_manager
                    .fail_job(&job_id, format!("IAR load failed: {}", e))
                    .await;
            }
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(ArchiveApiResponse {
            success: true,
            message: "IAR load job started".to_string(),
            job_id: Some(job_id.to_string()),
            data: None,
        }),
    )
}

async fn save_iar_endpoint(
    State(state): State<ArchiveApiState>,
    Json(request): Json<SaveIarRequest>,
) -> impl IntoResponse {
    info!(
        "Archive API: Saving IAR to {} for user {} {}",
        request.output_path, request.user_firstname, request.user_lastname
    );

    if request.output_path.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ArchiveApiResponse {
                success: false,
                message: "Output path cannot be empty".to_string(),
                job_id: None,
                data: None,
            }),
        );
    }

    if request.user_firstname.trim().is_empty() || request.user_lastname.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ArchiveApiResponse {
                success: false,
                message: "User first and last name are required".to_string(),
                job_id: None,
                data: None,
            }),
        );
    }

    let user_id = match get_user_id(
        &state.db_pool,
        &request.user_firstname,
        &request.user_lastname,
    )
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ArchiveApiResponse {
                    success: false,
                    message: format!(
                        "User not found: {} {}",
                        request.user_firstname, request.user_lastname
                    ),
                    job_id: None,
                    data: None,
                }),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ArchiveApiResponse {
                    success: false,
                    message: format!("Database error: {}", e),
                    job_id: None,
                    data: None,
                }),
            )
        }
    };

    let include_assets = request.include_assets.unwrap_or(true);
    let job_type = JobType::IarSave {
        user_id,
        folder_id: None,
        include_assets,
        output_path: PathBuf::from(&request.output_path),
    };

    let job_id = state.job_manager.create_job(job_type).await;

    let db_pool = state.db_pool.clone();
    let job_manager = state.job_manager.clone();
    let output_path = request.output_path.clone();

    tokio::spawn(async move {
        job_manager.start_job(&job_id).await;
        job_manager
            .update_progress(&job_id, 0.1, Some("Saving IAR...".to_string()))
            .await;

        let writer = IarWriter::new(db_pool);
        let options = IarSaveOptions {
            user_id,
            folder_id: None,
            include_assets,
        };

        match writer.save(&output_path, options).await {
            Ok(result) => {
                job_manager
                    .complete_job(
                        &job_id,
                        JobResult::IarSave {
                            assets_saved: result.stats.assets_saved,
                            folders_saved: result.stats.folders_saved,
                            items_saved: result.stats.items_saved,
                            download_path: result.output_path,
                        },
                    )
                    .await;
            }
            Err(e) => {
                job_manager
                    .fail_job(&job_id, format!("IAR save failed: {}", e))
                    .await;
            }
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(ArchiveApiResponse {
            success: true,
            message: "IAR save job started".to_string(),
            job_id: Some(job_id.to_string()),
            data: None,
        }),
    )
}

async fn load_oar_endpoint(
    State(state): State<ArchiveApiState>,
    Json(request): Json<LoadOarRequest>,
) -> impl IntoResponse {
    info!(
        "Archive API: Loading OAR from {} for region {}",
        request.file_path, request.region_name
    );

    if request.file_path.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ArchiveApiResponse {
                success: false,
                message: "File path cannot be empty".to_string(),
                job_id: None,
                data: None,
            }),
        );
    }

    if request.region_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ArchiveApiResponse {
                success: false,
                message: "Region name is required".to_string(),
                job_id: None,
                data: None,
            }),
        );
    }

    if !std::path::Path::new(&request.file_path).exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ArchiveApiResponse {
                success: false,
                message: format!("File not found: {}", request.file_path),
                job_id: None,
                data: None,
            }),
        );
    }

    let region_id = match get_region_id(&state.db_pool, &request.region_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ArchiveApiResponse {
                    success: false,
                    message: format!("Region not found: {}", request.region_name),
                    job_id: None,
                    data: None,
                }),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ArchiveApiResponse {
                    success: false,
                    message: format!("Database error: {}", e),
                    job_id: None,
                    data: None,
                }),
            )
        }
    };

    let default_user_id = if let (Some(firstname), Some(lastname)) = (
        &request.default_user_firstname,
        &request.default_user_lastname,
    ) {
        if !firstname.trim().is_empty() && !lastname.trim().is_empty() {
            match get_user_id(&state.db_pool, firstname, lastname).await {
                Ok(Some(id)) => {
                    info!(
                        "OAR import: Will reassign unowned objects to {} {} ({})",
                        firstname, lastname, id
                    );
                    Some(id)
                }
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(ArchiveApiResponse {
                            success: false,
                            message: format!("Default user not found: {} {}", firstname, lastname),
                            job_id: None,
                            data: None,
                        }),
                    )
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ArchiveApiResponse {
                            success: false,
                            message: format!("Database error looking up default user: {}", e),
                            job_id: None,
                            data: None,
                        }),
                    )
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    let merge = request.merge.unwrap_or(false);
    let job_type = JobType::OarLoad {
        region_id,
        source_path: PathBuf::from(&request.file_path),
        merge,
        load_terrain: true,
        load_objects: true,
        load_parcels: true,
    };

    let job_id = state.job_manager.create_job(job_type).await;

    let db_pool = state.db_pool.clone();
    let job_manager = state.job_manager.clone();
    let file_path = request.file_path.clone();
    let displacement = (
        request.displacement_x.unwrap_or(0.0),
        request.displacement_y.unwrap_or(0.0),
        request.displacement_z.unwrap_or(0.0),
    );
    let rotation = request.rotation_degrees.unwrap_or(0.0);
    let force_terrain = request.force_terrain.unwrap_or(false);
    let force_parcels = request.force_parcels.unwrap_or(false);

    let scene_objects = state.scene_objects.clone();
    let udp_socket = state.udp_socket.clone();
    let avatar_states = state.avatar_states.clone();
    let next_prim_local_id = state.next_prim_local_id.clone();
    let reliability_manager = state.reliability_manager.clone();
    let live_db_pool = state.db_pool.clone();
    let spawn_default_user_id = default_user_id;

    tokio::spawn(async move {
        job_manager.start_job(&job_id).await;
        job_manager
            .update_progress(&job_id, 0.1, Some("Loading OAR...".to_string()))
            .await;

        let reader = OarReader::new(db_pool);
        let options = OarLoadOptions {
            region_id,
            merge,
            load_terrain: true,
            load_objects: true,
            load_parcels: true,
            displacement,
            rotation_degrees: rotation,
            force_terrain,
            force_parcels,
            default_user_id: spawn_default_user_id,
        };

        match reader.load(&file_path, options).await {
            Ok(result) => {
                let objects_created = result.stats.objects_created;
                job_manager
                    .complete_job(
                        &job_id,
                        JobResult::OarLoad {
                            assets_loaded: result.stats.assets_loaded,
                            objects_created,
                            parcels_loaded: result.stats.parcels_loaded,
                            terrain_loaded: result.stats.terrain_loaded,
                        },
                    )
                    .await;

                if let (
                    Some(scene_objs),
                    Some(_socket),
                    Some(_avatars),
                    Some(prim_counter),
                    Some(_rel_mgr),
                ) = (
                    scene_objects,
                    udp_socket,
                    avatar_states,
                    next_prim_local_id,
                    reliability_manager,
                ) {
                    populate_scene_objects_from_db(
                        &live_db_pool,
                        region_id,
                        scene_objs,
                        prim_counter,
                    )
                    .await;
                    info!("[OAR] Scene objects updated in memory. Relog to see objects in-world.");
                }
            }
            Err(e) => {
                job_manager
                    .fail_job(&job_id, format!("OAR load failed: {}", e))
                    .await;
            }
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(ArchiveApiResponse {
            success: true,
            message: "OAR load job started".to_string(),
            job_id: Some(job_id.to_string()),
            data: None,
        }),
    )
}

async fn list_oar_files_endpoint() -> impl IntoResponse {
    let instance_dir = std::env::var("OPENSIM_INSTANCE_DIR").unwrap_or_else(|_| ".".to_string());
    let oar_dir = format!("{}/OAR", instance_dir);

    let mut files: Vec<serde_json::Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&oar_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("oar") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let full_path = path.to_string_lossy().to_string();
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    files.push(serde_json::json!({
                        "name": name,
                        "path": full_path,
                        "size": size,
                    }));
                }
            }
        }
    }
    files.sort_by(|a, b| {
        a["name"]
            .as_str()
            .unwrap_or("")
            .cmp(b["name"].as_str().unwrap_or(""))
    });

    (
        StatusCode::OK,
        Json(ArchiveApiResponse {
            success: true,
            message: format!("Found {} OAR files in {}", files.len(), oar_dir),
            job_id: None,
            data: Some(serde_json::json!({ "files": files, "oar_directory": oar_dir })),
        }),
    )
}

async fn save_oar_endpoint(
    State(state): State<ArchiveApiState>,
    Json(request): Json<SaveOarRequest>,
) -> impl IntoResponse {
    info!(
        "Archive API: Saving OAR to {} for region {}",
        request.output_path, request.region_name
    );

    if request.output_path.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ArchiveApiResponse {
                success: false,
                message: "Output path cannot be empty".to_string(),
                job_id: None,
                data: None,
            }),
        );
    }

    if request.region_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ArchiveApiResponse {
                success: false,
                message: "Region name is required".to_string(),
                job_id: None,
                data: None,
            }),
        );
    }

    let region_id = match get_region_id(&state.db_pool, &request.region_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ArchiveApiResponse {
                    success: false,
                    message: format!("Region not found: {}", request.region_name),
                    job_id: None,
                    data: None,
                }),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ArchiveApiResponse {
                    success: false,
                    message: format!("Database error: {}", e),
                    job_id: None,
                    data: None,
                }),
            )
        }
    };

    let include_assets = request.include_assets.unwrap_or(true);
    let include_terrain = request.include_terrain.unwrap_or(true);
    let include_objects = request.include_objects.unwrap_or(true);
    let include_parcels = request.include_parcels.unwrap_or(true);

    let job_type = JobType::OarSave {
        region_id,
        output_path: PathBuf::from(&request.output_path),
        include_assets,
        include_terrain,
        include_objects,
        include_parcels,
    };

    let job_id = state.job_manager.create_job(job_type).await;

    let db_pool = state.db_pool.clone();
    let job_manager = state.job_manager.clone();
    let output_path = request.output_path.clone();

    tokio::spawn(async move {
        job_manager.start_job(&job_id).await;
        job_manager
            .update_progress(&job_id, 0.1, Some("Saving OAR...".to_string()))
            .await;

        let writer = OarWriter::new(db_pool);
        let options = OarSaveOptions {
            region_id,
            include_assets,
            include_terrain,
            include_objects,
            include_parcels,
            object_uuids: None,
        };

        match writer.save(&output_path, options).await {
            Ok(result) => {
                job_manager
                    .complete_job(
                        &job_id,
                        JobResult::OarSave {
                            assets_saved: result.stats.assets_saved,
                            objects_saved: result.stats.objects_saved,
                            parcels_saved: result.stats.parcels_saved,
                            terrain_saved: result.stats.terrain_saved,
                            download_path: result.output_path,
                        },
                    )
                    .await;
            }
            Err(e) => {
                job_manager
                    .fail_job(&job_id, format!("OAR save failed: {}", e))
                    .await;
            }
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(ArchiveApiResponse {
            success: true,
            message: "OAR save job started".to_string(),
            job_id: Some(job_id.to_string()),
            data: None,
        }),
    )
}

async fn list_jobs_endpoint(
    State(state): State<ArchiveApiState>,
    Query(query): Query<JobQuery>,
) -> impl IntoResponse {
    debug!("Archive API: Listing jobs");

    let jobs = state.job_manager.get_active_jobs().await;
    let job_data: Vec<_> = jobs
        .iter()
        .take(query.limit.unwrap_or(50) as usize)
        .map(|job| {
            serde_json::json!({
                "id": job.id.to_string(),
                "type": format!("{:?}", job.job_type),
                "status": format!("{:?}", job.status),
                "progress": job.progress,
                "message": job.progress_message,
                "created_at": job.created_at.to_rfc3339(),
                "completed_at": job.completed_at.map(|t| t.to_rfc3339()),
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(ArchiveApiResponse {
            success: true,
            message: format!("Found {} jobs", job_data.len()),
            job_id: None,
            data: Some(serde_json::json!({ "jobs": job_data })),
        }),
    )
}

async fn get_job_status_endpoint(
    State(state): State<ArchiveApiState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    debug!("Archive API: Getting job status for {}", job_id);

    let job_uuid = match Uuid::parse_str(&job_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ArchiveApiResponse {
                    success: false,
                    message: "Invalid job ID format".to_string(),
                    job_id: None,
                    data: None,
                }),
            )
        }
    };

    match state.job_manager.get_job(&job_uuid).await {
        Some(job) => (
            StatusCode::OK,
            Json(ArchiveApiResponse {
                success: true,
                message: job.progress_message.clone().unwrap_or_default(),
                job_id: Some(job_id),
                data: Some(serde_json::json!({
                    "type": format!("{:?}", job.job_type),
                    "status": format!("{:?}", job.status),
                    "progress": job.progress,
                    "created_at": job.created_at.to_rfc3339(),
                    "started_at": job.started_at.map(|t| t.to_rfc3339()),
                    "completed_at": job.completed_at.map(|t| t.to_rfc3339()),
                    "result": job.result,
                    "error": job.error,
                })),
            }),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(ArchiveApiResponse {
                success: false,
                message: "Job not found".to_string(),
                job_id: None,
                data: None,
            }),
        ),
    }
}

async fn cancel_job_endpoint(
    State(state): State<ArchiveApiState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    info!("Archive API: Cancelling job {}", job_id);

    let job_uuid = match Uuid::parse_str(&job_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ArchiveApiResponse {
                    success: false,
                    message: "Invalid job ID format".to_string(),
                    job_id: None,
                    data: None,
                }),
            )
        }
    };

    if state.job_manager.cancel_job(&job_uuid).await {
        (
            StatusCode::OK,
            Json(ArchiveApiResponse {
                success: true,
                message: "Job cancelled".to_string(),
                job_id: Some(job_id),
                data: None,
            }),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ArchiveApiResponse {
                success: false,
                message: "Job not found or cannot be cancelled".to_string(),
                job_id: None,
                data: None,
            }),
        )
    }
}

async fn archive_health_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "archive_api",
        "endpoints": [
            "POST /admin/archives/iar/load - Load IAR file",
            "POST /admin/archives/iar/save - Save IAR file",
            "POST /admin/archives/oar/load - Load OAR file",
            "POST /admin/archives/oar/save - Save OAR file",
            "GET /admin/archives/jobs - List archive jobs",
            "GET /admin/archives/jobs/:id - Get job status",
            "POST /admin/archives/jobs/:id/cancel - Cancel job",
            "GET /admin/archives/health - This endpoint"
        ],
        "opensim_commands_supported": ["load iar", "save iar", "load oar", "save oar"]
    }))
}

async fn clear_region_endpoint(State(state): State<ArchiveApiState>) -> impl IntoResponse {
    info!("Archive API: Clearing region (DB + in-memory scene objects)");

    let mut cleared_tables = Vec::new();

    match sqlx::query("DELETE FROM primshapes WHERE uuid IN (SELECT uuid FROM prims)")
        .execute(&state.db_pool)
        .await
    {
        Ok(r) => {
            cleared_tables.push(format!("primshapes: {} rows", r.rows_affected()));
        }
        Err(e) => {
            warn!("Failed to clear primshapes: {}", e);
        }
    }

    match sqlx::query("DELETE FROM prims")
        .execute(&state.db_pool)
        .await
    {
        Ok(r) => {
            cleared_tables.push(format!("prims: {} rows", r.rows_affected()));
        }
        Err(e) => {
            warn!("Failed to clear prims: {}", e);
        }
    }

    match sqlx::query("DELETE FROM land")
        .execute(&state.db_pool)
        .await
    {
        Ok(r) => {
            cleared_tables.push(format!("land: {} rows", r.rows_affected()));
        }
        Err(e) => {
            warn!("Failed to clear land: {}", e);
        }
    }

    match sqlx::query("DELETE FROM landaccesslist")
        .execute(&state.db_pool)
        .await
    {
        Ok(r) => {
            cleared_tables.push(format!("landaccesslist: {} rows", r.rows_affected()));
        }
        Err(e) => {
            warn!("Failed to clear landaccesslist: {}", e);
        }
    }

    match sqlx::query("DELETE FROM bakedterrain")
        .execute(&state.db_pool)
        .await
    {
        Ok(r) => {
            cleared_tables.push(format!("bakedterrain: {} rows", r.rows_affected()));
        }
        Err(e) => {
            warn!("Failed to clear bakedterrain: {}", e);
        }
    }

    let (scene_count, killed_ids) = if let Some(ref scene_objects) = state.scene_objects {
        let mut write = scene_objects.write();
        let count = write.len();
        let ids: Vec<u32> = write.keys().copied().collect();
        write.clear();
        info!("Cleared {} in-memory scene objects", count);
        (count, ids)
    } else {
        warn!("No scene_objects reference available - in-memory scene NOT cleared");
        (0, Vec::new())
    };

    let mut viewers_notified = 0;
    if !killed_ids.is_empty() {
        if let (Some(ref socket), Some(ref avatar_states)) =
            (&state.udp_socket, &state.avatar_states)
        {
            let viewer_addrs: Vec<std::net::SocketAddr> = {
                let states = avatar_states.read();
                states
                    .values()
                    .filter(|s| !s.is_npc)
                    .map(|s| s.client_addr)
                    .collect()
            };

            for addr in &viewer_addrs {
                for chunk in killed_ids.chunks(200) {
                    // KillObject: High frequency 16 (0x10)
                    // Header: flags(1) + seq(4) + extra(1) + msg_id(1) = 7 bytes
                    // Body: count(1) + local_ids(4 each)
                    let mut packet = Vec::with_capacity(8 + chunk.len() * 4);
                    packet.push(0x40); // RELIABLE flag
                    packet.extend_from_slice(&[0, 0, 0, 0]); // sequence 0 (unreliable-ish)
                    packet.push(0x00); // extra header
                    packet.push(0x10); // High freq msg id 16 = KillObject
                    packet.push(chunk.len() as u8);
                    for &lid in chunk {
                        packet.extend_from_slice(&lid.to_le_bytes());
                    }
                    let _ = socket.send_to(&packet, addr).await;
                }
                viewers_notified += 1;
            }
            info!(
                "Sent KillObject for {} objects to {} viewers",
                killed_ids.len(),
                viewers_notified
            );
        }
    }

    let message = format!(
        "Region cleared. DB: [{}]. In-memory: {} objects removed. {} viewers notified.",
        cleared_tables.join(", "),
        scene_count,
        viewers_notified
    );
    info!("{}", message);

    (
        StatusCode::OK,
        Json(ArchiveApiResponse {
            success: true,
            message,
            job_id: None,
            data: None,
        }),
    )
}

const OBJECT_UPDATE_MSG_ID: u8 = 0x0C;
const FLAG_RELIABLE: u8 = 0x40;
const FLAG_ZEROCODED: u8 = 0x80;

async fn populate_scene_objects_from_db(
    db_pool: &PgPool,
    region_id: Uuid,
    scene_objects: Arc<parking_lot::RwLock<HashMap<u32, crate::udp::server::SceneObject>>>,
    next_prim_local_id: Arc<std::sync::atomic::AtomicU32>,
) {
    use crate::udp::server::SceneObject;
    use sqlx::Row;

    let rows = match sqlx::query(
        r#"SELECT
            p.uuid, p.ownerid, p.groupid, p.scenegroupid,
            p.positionx, p.positiony, p.positionz,
            p.grouppositionx, p.grouppositiony, p.grouppositionz,
            p.rotationx, p.rotationy, p.rotationz, p.rotationw,
            COALESCE(p.name, '') as name, COALESCE(p.description, '') as description,
            COALESCE(p.text, '') as text,
            COALESCE(p.material, 0) as material, COALESCE(p.objectflags, 0) as objectflags,
            COALESCE(p.ownermask, 2147483647) as ownermask,
            COALESCE(p.linknumber, 1) as linknumber,
            COALESCE(ps.pcode, 9) as pcode, COALESCE(ps.pathcurve, 16) as pathcurve,
            COALESCE(ps.profilecurve, 1) as profilecurve,
            COALESCE(ps.pathbegin, 0) as pathbegin, COALESCE(ps.pathend, 0) as pathend,
            COALESCE(ps.pathscalex, 100) as pathscalex, COALESCE(ps.pathscaley, 100) as pathscaley,
            COALESCE(ps.pathshearx, 0) as pathshearx, COALESCE(ps.pathsheary, 0) as pathsheary,
            COALESCE(ps.pathtwist, 0) as pathtwist, COALESCE(ps.pathtwistbegin, 0) as pathtwistbegin,
            COALESCE(ps.pathradiusoffset, 0) as pathradiusoffset,
            COALESCE(ps.pathtaperx, 0) as pathtaperx, COALESCE(ps.pathtapery, 0) as pathtapery,
            COALESCE(ps.pathrevolutions, 0) as pathrevolutions,
            COALESCE(ps.pathskew, 0) as pathskew,
            COALESCE(ps.profilebegin, 0) as profilebegin, COALESCE(ps.profileend, 0) as profileend,
            COALESCE(ps.profilehollow, 0) as profilehollow,
            COALESCE(ps.scalex, 0.5) as scalex, COALESCE(ps.scaley, 0.5) as scaley,
            COALESCE(ps.scalez, 0.5) as scalez,
            COALESCE(ps.texture, E'\x'::bytea) as texture,
            COALESCE(ps.extraparams, E'\x'::bytea) as extraparams
        FROM prims p
        JOIN primshapes ps ON p.uuid = ps.uuid
        WHERE p.regionuuid = $1
        ORDER BY p.scenegroupid, p.linknumber"#
    )
    .bind(region_id)
    .fetch_all(db_pool)
    .await {
        Ok(rows) => rows,
        Err(e) => {
            error!("[OAR] Failed to query prims for scene population: {}", e);
            return;
        }
    };

    if rows.is_empty() {
        info!("[OAR] No prims found for region {}", region_id);
        return;
    }

    let mut group_root_local_ids: HashMap<Uuid, u32> = HashMap::new();
    let mut loaded = 0usize;

    let mut scene = scene_objects.write();
    scene.clear();

    for row in &rows {
        let prim_uuid: Uuid = row.get("uuid");
        let owner_id: Uuid = row.get("ownerid");
        let group_id: Uuid = row.get("groupid");
        let scene_group_id: Uuid = row.get("scenegroupid");
        let link_number: i32 = row.get("linknumber");
        let is_root = link_number <= 1;

        let local_id = next_prim_local_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let position = if is_root {
            [
                row.get::<f32, _>("grouppositionx"),
                row.get::<f32, _>("grouppositiony"),
                row.get::<f32, _>("grouppositionz"),
            ]
        } else {
            [
                row.get::<f32, _>("positionx"),
                row.get::<f32, _>("positiony"),
                row.get::<f32, _>("positionz"),
            ]
        };

        let parent_id = if is_root {
            if scene_group_id != Uuid::nil() {
                group_root_local_ids.insert(scene_group_id, local_id);
            }
            0u32
        } else {
            group_root_local_ids
                .get(&scene_group_id)
                .copied()
                .unwrap_or(0)
        };

        let obj = SceneObject {
            local_id,
            uuid: prim_uuid,
            owner_id,
            creator_id: owner_id,
            group_id,
            position,
            rotation: [
                row.get("rotationx"),
                row.get("rotationy"),
                row.get("rotationz"),
                row.get("rotationw"),
            ],
            scale: [row.get("scalex"), row.get("scaley"), row.get("scalez")],
            name: row.get("name"),
            description: row.get("description"),
            pcode: row.get::<i32, _>("pcode") as u8,
            material: row.get::<i32, _>("material") as u8,
            path_curve: row.get::<i32, _>("pathcurve") as u8,
            profile_curve: row.get::<i32, _>("profilecurve") as u8,
            path_begin: row.get::<i32, _>("pathbegin") as u16,
            path_end: row.get::<i32, _>("pathend") as u16,
            path_scale_x: row.get::<i32, _>("pathscalex") as u8,
            path_scale_y: row.get::<i32, _>("pathscaley") as u8,
            path_shear_x: row.get::<i32, _>("pathshearx") as u8,
            path_shear_y: row.get::<i32, _>("pathsheary") as u8,
            path_twist: row.get::<i32, _>("pathtwist") as i8,
            path_twist_begin: row.get::<i32, _>("pathtwistbegin") as i8,
            path_radius_offset: row.get::<i32, _>("pathradiusoffset") as i8,
            path_taper_x: row.get::<i32, _>("pathtaperx") as i8,
            path_taper_y: row.get::<i32, _>("pathtapery") as i8,
            path_revolutions: row.get::<i32, _>("pathrevolutions") as u8,
            path_skew: row.get::<i32, _>("pathskew") as i8,
            profile_begin: row.get::<i32, _>("profilebegin") as u16,
            profile_end: row.get::<i32, _>("profileend") as u16,
            profile_hollow: row.get::<i32, _>("profilehollow") as u16,
            texture_entry: row.get("texture"),
            texture_anim: Vec::new(),
            extra_params: row.get("extraparams"),
            text: row.get("text"),
            parent_id,
            link_number,
            scene_group_id,
            flags: row.get::<i32, _>("objectflags") as u32,
            owner_mask: row.get::<i32, _>("ownermask") as u32,
            mat_overrides: Vec::new(),
            script_items: Vec::new(),
            vehicle_type: 0,
            vehicle_float_params: HashMap::new(),
            vehicle_vec_params: HashMap::new(),
            vehicle_rot_params: HashMap::new(),
            vehicle_flags: 0,
            sit_target_pos: None,
            sit_target_rot: None,
            sitting_avatar: None,
            sit_text: String::new(),
            touch_text: String::new(),
            attachment_point: 0,
            vehicle_linear_motor_target: [0.0; 3],
            vehicle_linear_motor_velocity: [0.0; 3],
            vehicle_angular_motor_target: [0.0; 3],
            vehicle_angular_motor_velocity: [0.0; 3],
            physics_shape_type: 0,
            density: 1000.0,
            friction: 0.6,
            restitution: 0.5,
            gravity_modifier: 1.0,
            velocity: [0.0; 3],
            was_moving: false,
            last_colliders: Vec::new(),
            position_targets: std::collections::HashMap::new(),
            rotation_targets: std::collections::HashMap::new(),
            next_target_handle: 1,
            particle_system: Vec::new(),
            buoyancy: 0.0,
            click_action: 0,
            force_mouselook: false,
            volume_detect: false,
            collision_sound: Uuid::nil(),
            collision_sound_volume: 0.0,
            damage: 0.0,
            script_access_pin: 0,
            allow_inventory_drop: false,
            pass_touches: false,
            pass_collisions: false,
            sound_radius: 0.0,
            sound_queueing: false,
            pay_price: [-2, -2, -2, -2, -2],
            angular_velocity: [0.0; 3],
            force: [0.0; 3],
            torque: [0.0; 3],
            hover_height: None,
            move_target: None,
            look_target: None,
            rot_look_target: None,
            camera_at_offset: [0.0; 3],
            camera_eye_offset: [0.0; 3],
            object_animations: Vec::new(),
            keyframed_motion: None,
            projection_params: None,
            linkset_data: HashMap::new(),
            media_params: HashMap::new(),
            base_mask: 0x7FFFFFFF,
            group_mask: 0,
            everyone_mask: 0,
            next_owner_mask: 0x7FFFFFFF,
            collision_filter_name: String::new(),
            collision_filter_id: Uuid::nil(),
            collision_filter_accept: true,
            item_id: Uuid::nil(),
            sale_type: 0,
            sale_price: 0,
        };

        scene.insert(local_id, obj);
        loaded += 1;
    }

    info!(
        "[OAR] Populated {} prims into in-memory scene (relog to see in viewer)",
        loaded
    );
}

async fn get_user_id(
    pool: &PgPool,
    firstname: &str,
    lastname: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT principalid FROM useraccounts WHERE firstname = $1 AND lastname = $2",
    )
    .bind(firstname)
    .bind(lastname)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id,)| id))
}

async fn get_region_id(pool: &PgPool, region_name: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM regions WHERE region_name = $1")
        .bind(region_name)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|(id,)| id))
}

async fn create_user_with_id(
    pool: &PgPool,
    user_id: Uuid,
    firstname: &str,
    lastname: &str,
    email: &str,
    password: &str,
) -> Result<(), anyhow::Error> {
    let created_timestamp = chrono::Utc::now().timestamp() as i32;
    let salt: String = (0..32)
        .map(|_| format!("{:x}", rand::random::<u8>() % 16))
        .collect();
    let password_md5 = format!("{:x}", md5::compute(password.as_bytes()));
    let salted = format!("{}:{}", password_md5, salt);
    let password_hash = format!("{:x}", md5::compute(salted.as_bytes()));

    let mut conn = pool.acquire().await?;

    sqlx::query(
        r#"INSERT INTO UserAccounts
           (PrincipalID, FirstName, LastName, Email, ServiceURLs, Created, UserLevel, UserFlags, UserTitle, Active)
           VALUES ($1::uuid, $2, $3, $4, '', $5, 0, 0, '', 1)"#,
    )
    .bind(user_id.to_string())
    .bind(firstname)
    .bind(lastname)
    .bind(email)
    .bind(created_timestamp)
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        r#"INSERT INTO auth (uuid, passwordhash, passwordsalt, webloginkey, accounttype)
           VALUES ($1::uuid, $2, $3, '', 'UserAccount')"#,
    )
    .bind(user_id.to_string())
    .bind(&password_hash)
    .bind(&salt)
    .execute(&mut *conn)
    .await?;

    crate::database::default_inventory::create_default_user_inventory(pool, user_id).await?;

    info!(
        "Created user {} {} with ID {} for archive import",
        firstname, lastname, user_id
    );
    Ok(())
}
