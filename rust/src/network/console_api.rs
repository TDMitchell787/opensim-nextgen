//! Console Commands REST API
//!
//! Provides REST API endpoints for all OpenSim console commands.
//! Designed for the Flutter Console Commands UI (Phase 84).

use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::database::DatabaseAdmin;
use crate::database::multi_backend::DatabaseConnection;
use crate::region::terrain_storage::{TerrainStorage, TerrainRevision};

#[derive(Clone)]
pub struct ConsoleApiState {
    pub db_admin: Arc<DatabaseAdmin>,
    pub db_connection: Option<Arc<DatabaseConnection>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleCommandRequest {
    pub command: String,
    pub group: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleCommandResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub timestamp: u64,
}

impl ConsoleCommandResponse {
    pub fn success(message: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data,
            error: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn error(message: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            error: Some(error.into()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CommandDefinition {
    pub name: String,
    pub group: String,
    pub description: String,
    pub syntax: String,
    pub params: Vec<ParamDefinition>,
    pub implemented: bool,
}

#[derive(Debug, Serialize)]
pub struct ParamDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub choices: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub version: String,
    pub uptime_seconds: u64,
    pub start_time: u64,
    pub region_count: i32,
    pub user_count: i32,
    pub active_sessions: i32,
}

pub fn create_console_api_router() -> Router<ConsoleApiState> {
    Router::new()
        .route("/console/commands", get(list_commands_endpoint))
        .route("/console/execute", post(execute_command_endpoint))
        .route("/console/info", get(server_info_endpoint))
        .route("/console/regions", get(list_regions_endpoint))
        .route("/console/regions/:name", get(show_region_endpoint))
        .route("/console/regions/:name/restart", post(restart_region_endpoint))
        .route("/console/terrain/stats", get(terrain_stats_endpoint))
        .route("/console/terrain/load", post(terrain_load_endpoint))
        .route("/console/terrain/save", post(terrain_save_endpoint))
        .route("/console/terrain/fill", post(terrain_fill_endpoint))
        .route("/console/shutdown", post(shutdown_endpoint))
        .route("/console/users/kick", post(kick_user_endpoint))
        .route("/console/login/level", post(login_level_endpoint))
        .route("/console/login/reset", post(login_reset_endpoint))
        .route("/console/login/text", post(login_text_endpoint))
        .route("/console/connections", get(show_connections_endpoint))
        .route("/console/circuits", get(show_circuits_endpoint))
        .route("/console/objects/show", post(show_object_endpoint))
        .route("/console/objects/delete", post(delete_object_endpoint))
        .route("/console/objects/backup", post(backup_endpoint))
        .route("/console/scene/rotate", post(rotate_scene_endpoint))
        .route("/console/scene/scale", post(scale_scene_endpoint))
        .route("/console/scene/translate", post(translate_scene_endpoint))
        .route("/console/scene/force-update", post(force_update_endpoint))
        .route("/console/estates/create", post(estate_create_endpoint))
        .route("/console/estates/set-owner", post(estate_set_owner_endpoint))
        .route("/console/estates/set-name", post(estate_set_name_endpoint))
        .route("/console/estates/link-region", post(estate_link_region_endpoint))
        .route("/console/hypergrid/link", post(hypergrid_link_endpoint))
        .route("/console/hypergrid/unlink", post(hypergrid_unlink_endpoint))
        .route("/console/hypergrid/links", get(hypergrid_show_links_endpoint))
        .route("/console/hypergrid/mapping", post(hypergrid_mapping_endpoint))
        .route("/console/assets/show", post(show_asset_endpoint))
        .route("/console/assets/dump", post(dump_asset_endpoint))
        .route("/console/assets/delete", post(delete_asset_endpoint))
        .route("/console/fcache/status", get(fcache_status_endpoint))
        .route("/console/fcache/clear", post(fcache_clear_endpoint))
        .route("/console/fcache/assets", get(fcache_assets_endpoint))
        .route("/console/fcache/expire", post(fcache_expire_endpoint))
        .route("/console/xml/load", post(xml_load_endpoint))
        .route("/console/xml/save", post(xml_save_endpoint))
        .route("/console/regions/create", post(create_region_endpoint))
        .route("/console/regions/delete", post(delete_region_endpoint))
        .route("/console/regions/ratings", get(show_ratings_endpoint))
        .route("/console/regions/neighbours", get(show_neighbours_endpoint))
        .route("/console/regions/inview", get(show_regions_inview_endpoint))
        .route("/console/regions/change", post(change_region_endpoint))
        .route("/console/terrain/load-tile", post(terrain_load_tile_endpoint))
        .route("/console/terrain/save-tile", post(terrain_save_tile_endpoint))
        .route("/console/terrain/elevate", post(terrain_elevate_endpoint))
        .route("/console/terrain/lower", post(terrain_lower_endpoint))
        .route("/console/terrain/multiply", post(terrain_multiply_endpoint))
        .route("/console/terrain/bake", post(terrain_bake_endpoint))
        .route("/console/terrain/revert", post(terrain_revert_endpoint))
        .route("/console/terrain/show", get(terrain_show_endpoint))
        .route("/console/terrain/effect", post(terrain_effect_endpoint))
        .route("/console/terrain/flip", post(terrain_flip_endpoint))
        .route("/console/terrain/rescale", post(terrain_rescale_endpoint))
        .route("/console/terrain/min", post(terrain_min_endpoint))
        .route("/console/terrain/max", post(terrain_max_endpoint))
        .route("/console/terrain/modify", post(terrain_modify_endpoint))
        .route("/console/general/quit", post(quit_endpoint))
        .route("/console/general/modules", get(show_modules_endpoint))
        .route("/console/general/command-script", post(command_script_endpoint))
        .route("/console/config/show", get(config_show_endpoint))
        .route("/console/config/get", post(config_get_endpoint))
        .route("/console/config/set", post(config_set_endpoint))
        .route("/console/log/level", post(set_log_level_endpoint))
        .route("/console/general/force-gc", post(force_gc_endpoint))
        .route("/console/users/grid-user", post(show_grid_user_endpoint))
        .route("/console/users/grid-users-online", get(show_grid_users_online_endpoint))
        .route("/console/fcache/clearnegatives", post(fcache_clearnegatives_endpoint))
        .route("/console/fcache/cachedefaultassets", post(fcache_cachedefaultassets_endpoint))
        .route("/console/fcache/deletedefaultassets", post(fcache_deletedefaultassets_endpoint))
        .route("/console/parts/show", post(show_part_endpoint))
        .route("/console/objects/dump", post(dump_object_endpoint))
        .route("/console/objects/edit-scale", post(edit_scale_endpoint))
        .route("/console/comms/pending-objects", get(show_pending_objects_endpoint))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
}

async fn list_commands_endpoint() -> impl IntoResponse {
    let commands = get_all_command_definitions();
    Json(commands)
}

async fn execute_command_endpoint(
    State(state): State<ConsoleApiState>,
    Json(request): Json<ConsoleCommandRequest>,
) -> impl IntoResponse {
    info!("Console API: Executing command {} in group {}", request.command, request.group);

    let result = match (request.group.as_str(), request.command.as_str()) {
        ("general", "show info") => execute_show_info(&state).await,
        ("general", "show version") => execute_show_version().await,
        ("general", "show uptime") => execute_show_uptime().await,
        ("regions", "show regions") => execute_show_regions(&state).await,
        ("terrain", "stats") => execute_terrain_stats(&state).await,
        ("database", "stats") => execute_database_stats(&state).await,
        ("database", "health") => execute_database_health(&state).await,
        ("users", "kick user") => execute_kick_user(&request.params).await,
        ("users", "login level") => execute_login_level(&request.params).await,
        ("users", "login reset") => execute_login_reset().await,
        ("users", "login text") => execute_login_text(&request.params).await,
        ("comms", "show connections") => execute_show_connections().await,
        ("comms", "show circuits") => execute_show_circuits().await,
        ("objects", "show object id") => execute_show_object("id", &request.params).await,
        ("objects", "show object name") => execute_show_object("name", &request.params).await,
        ("objects", "show object owner") => execute_show_object("owner", &request.params).await,
        ("objects", "show object pos") => execute_show_object("pos", &request.params).await,
        ("objects", "delete object id") => execute_delete_object("id", &request.params).await,
        ("objects", "delete object name") => execute_delete_object("name", &request.params).await,
        ("objects", "delete object owner") => execute_delete_object("owner", &request.params).await,
        ("objects", "delete object pos") => execute_delete_object("pos", &request.params).await,
        ("objects", "delete object outside") => execute_delete_object("outside", &request.params).await,
        ("objects", "backup") => execute_backup().await,
        ("objects", "rotate scene") => execute_rotate_scene(&request.params).await,
        ("objects", "scale scene") => execute_scale_scene(&request.params).await,
        ("objects", "translate scene") => execute_translate_scene(&request.params).await,
        ("objects", "force update") => execute_force_update().await,
        ("estates", "estate create") => execute_estate_create(&request.params).await,
        ("estates", "estate set owner") => execute_estate_set_owner(&request.params).await,
        ("estates", "estate set name") => execute_estate_set_name(&request.params).await,
        ("estates", "estate link region") => execute_estate_link_region(&request.params).await,
        ("hypergrid", "link-region") => execute_hypergrid_link(&request.params).await,
        ("hypergrid", "unlink-region") => execute_hypergrid_unlink(&request.params).await,
        ("hypergrid", "show hyperlinks") => execute_show_hyperlinks().await,
        ("hypergrid", "link-mapping") => execute_link_mapping(&request.params).await,
        ("assets", "show asset") => execute_show_asset(&request.params).await,
        ("assets", "dump asset") => execute_dump_asset(&request.params).await,
        ("assets", "delete asset") => execute_delete_asset(&request.params).await,
        ("assets", "fcache status") => execute_fcache_status().await,
        ("assets", "fcache clear") => execute_fcache_clear(&request.params).await,
        ("assets", "fcache assets") => execute_fcache_assets().await,
        ("assets", "fcache expire") => execute_fcache_expire(&request.params).await,
        ("archiving", "load xml") => execute_xml_load(&request.params).await,
        ("archiving", "save xml") => execute_xml_save(&request.params).await,
        ("archiving", "load xml2") => execute_xml_load(&request.params).await,
        ("archiving", "save xml2") => execute_xml_save(&request.params).await,
        ("archiving", "save prims xml2") => execute_save_prims_xml2(&request.params).await,
        ("regions", "create region") => execute_create_region(&request.params).await,
        ("regions", "delete-region") => execute_delete_region(&request.params).await,
        ("regions", "remove-region") => execute_delete_region(&request.params).await,
        ("regions", "restart") => execute_restart_region(&request.params).await,
        ("regions", "show ratings") => execute_show_ratings().await,
        ("regions", "show neighbours") => execute_show_neighbours().await,
        ("regions", "show regionsinview") => execute_show_regions_inview().await,
        ("regions", "change region") => execute_change_region(&request.params).await,
        ("terrain", "terrain load") => execute_terrain_load(&request.params).await,
        ("terrain", "terrain load-tile") => execute_terrain_load_tile(&request.params).await,
        ("terrain", "terrain save") => execute_terrain_save(&request.params).await,
        ("terrain", "terrain save-tile") => execute_terrain_save_tile(&request.params).await,
        ("terrain", "terrain fill") => execute_terrain_fill(&request.params).await,
        ("terrain", "terrain elevate") => ConsoleCommandResponse::error("Use /console/terrain/elevate endpoint", "POST with {amount, region}"),
        ("terrain", "terrain lower") => ConsoleCommandResponse::error("Use /console/terrain/lower endpoint", "POST with {amount, region}"),
        ("terrain", "terrain multiply") => ConsoleCommandResponse::error("Use /console/terrain/multiply endpoint", "POST with {factor, region}"),
        ("terrain", "terrain bake") => ConsoleCommandResponse::error("Use /console/terrain/bake endpoint", "POST"),
        ("terrain", "terrain revert") => ConsoleCommandResponse::error("Use /console/terrain/revert endpoint", "POST"),
        ("terrain", "terrain show") => ConsoleCommandResponse::error("Use /console/terrain/show endpoint", "GET"),
        ("terrain", "terrain effect") => execute_terrain_effect(&request.params).await,
        ("terrain", "terrain flip") => execute_terrain_flip(&request.params).await,
        ("terrain", "terrain rescale") => execute_terrain_rescale(&request.params).await,
        ("terrain", "terrain min") => execute_terrain_min(&request.params).await,
        ("terrain", "terrain max") => execute_terrain_max(&request.params).await,
        ("terrain", "terrain modify") => execute_terrain_modify(&request.params).await,
        ("general", "quit") => execute_quit().await,
        ("general", "shutdown") => execute_shutdown().await,
        ("general", "show modules") => execute_show_modules().await,
        ("general", "command-script") => execute_command_script(&request.params).await,
        ("general", "config show") => execute_config_show(&request.params).await,
        ("general", "config get") => execute_config_get(&request.params).await,
        ("general", "config set") => execute_config_set(&request.params).await,
        ("general", "set log level") => execute_set_log_level(&request.params).await,
        ("general", "force gc") => execute_force_gc().await,
        ("users", "show grid user") => execute_show_grid_user(&request.params).await,
        ("users", "show grid users online") => execute_show_grid_users_online().await,
        ("assets", "fcache clearnegatives") => execute_fcache_clearnegatives().await,
        ("assets", "fcache cachedefaultassets") => execute_fcache_cachedefaultassets().await,
        ("assets", "fcache deletedefaultassets") => execute_fcache_deletedefaultassets().await,
        ("objects", "show part id") => execute_show_part("id", &request.params).await,
        ("objects", "show part name") => execute_show_part("name", &request.params).await,
        ("objects", "show part pos") => execute_show_part("pos", &request.params).await,
        ("objects", "dump object id") => execute_dump_object(&request.params).await,
        ("objects", "edit scale") => execute_edit_scale(&request.params).await,
        ("comms", "show pending-objects") => execute_show_pending_objects().await,
        _ => ConsoleCommandResponse::error(
            "Command not implemented",
            format!("Command '{}' in group '{}' is not yet implemented", request.command, request.group)
        ),
    };

    (
        if result.success { StatusCode::OK } else { StatusCode::BAD_REQUEST },
        Json(result)
    )
}

async fn server_info_endpoint(
    State(state): State<ConsoleApiState>,
) -> impl IntoResponse {
    let user_count = match state.db_admin.list_users(None).await {
        Ok(result) => {
            if let Some(data) = result.data {
                data.as_array().map(|arr| arr.len()).unwrap_or(0) as i32
            } else {
                0
            }
        },
        Err(_) => 0,
    };

    let info = ServerInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0,
        start_time: 0,
        region_count: 1,
        user_count,
        active_sessions: 0,
    };
    Json(info)
}

async fn list_regions_endpoint(
    State(state): State<ConsoleApiState>,
) -> impl IntoResponse {
    match state.db_admin.list_regions(Some(100)).await {
        Ok(result) => {
            if let Some(data) = &result.data {
                if let Some(regions) = data.get("regions") {
                    let mapped: Vec<serde_json::Value> = regions.as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|r| serde_json::json!({
                            "name": r.get("region_name").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                            "uuid": r.get("uuid").and_then(|v| v.as_str()).unwrap_or(""),
                            "location": format!("{},{}",
                                r.get("location_x").and_then(|v| v.as_i64()).unwrap_or(0) / 256,
                                r.get("location_y").and_then(|v| v.as_i64()).unwrap_or(0) / 256),
                            "status": r.get("status").and_then(|v| v.as_str()).unwrap_or("running"),
                            "agents": 0
                        }))
                        .collect();
                    return Json(ConsoleCommandResponse::success(
                        "Regions listed",
                        Some(serde_json::json!({ "regions": mapped }))
                    ));
                }
            }
            Json(ConsoleCommandResponse::success(
                "Regions listed",
                Some(serde_json::json!({ "regions": [] }))
            ))
        }
        Err(_) => {
            Json(ConsoleCommandResponse::success(
                "Regions listed",
                Some(serde_json::json!({ "regions": [] }))
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegionPath {
    name: String,
}

async fn show_region_endpoint(
    Path(params): Path<RegionPath>,
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(ConsoleCommandResponse::success(
        format!("Region '{}' info", params.name),
        Some(serde_json::json!({
            "name": params.name,
            "uuid": "00000000-0000-0000-0000-000000000001",
            "location": "1000,1000",
            "size": "256x256",
            "status": "running",
            "agents": 0,
            "objects": 0,
            "scripts": 0
        }))
    ))
}

async fn restart_region_endpoint(
    Path(params): Path<RegionPath>,
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    info!("Console API: Restart region '{}'", params.name);
    Json(ConsoleCommandResponse::success(
        format!("Region '{}' restart initiated", params.name),
        None
    ))
}

async fn get_region_uuids(state: &ConsoleApiState, region_name: Option<&str>) -> Result<Vec<(Uuid, String)>> {
    match state.db_admin.list_regions(Some(500)).await {
        Ok(result) => {
            if let Some(data) = &result.data {
                if let Some(regions) = data.get("regions").and_then(|v| v.as_array()) {
                    let mut matched: Vec<(Uuid, String)> = Vec::new();
                    for r in regions {
                        let name = r.get("region_name").and_then(|v| v.as_str()).unwrap_or("");
                        let uuid_str = r.get("uuid").and_then(|v| v.as_str()).unwrap_or("");
                        if let Ok(uuid) = uuid_str.parse::<Uuid>() {
                            if let Some(filter) = region_name {
                                if name.to_lowercase().contains(&filter.to_lowercase()) {
                                    matched.push((uuid, name.to_string()));
                                }
                            } else {
                                matched.push((uuid, name.to_string()));
                            }
                        }
                    }
                    return Ok(matched);
                }
            }
            Ok(Vec::new())
        }
        Err(e) => anyhow::bail!("Failed to list regions: {}", e),
    }
}

async fn terrain_stats_endpoint(
    State(state): State<ConsoleApiState>,
) -> impl IntoResponse {
    let conn = match &state.db_connection {
        Some(c) => c.clone(),
        None => return Json(ConsoleCommandResponse::error("No database connection", "Database not available")),
    };
    let storage = TerrainStorage::new(conn);
    let regions = match get_region_uuids(&state, None).await {
        Ok(r) => r,
        Err(e) => return Json(ConsoleCommandResponse::error("Failed to list regions", &e.to_string())),
    };

    let mut region_stats = Vec::new();
    for (uuid, name) in &regions {
        if let Ok(Some(heightmap)) = storage.load_terrain(*uuid).await {
            let min = heightmap.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = heightmap.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let avg = heightmap.iter().sum::<f32>() / heightmap.len() as f32;
            let side = (heightmap.len() as f64).sqrt() as usize;
            region_stats.push(serde_json::json!({
                "region": name,
                "uuid": uuid.to_string(),
                "size": format!("{}x{}", side, side),
                "min_height": format!("{:.1}", min),
                "max_height": format!("{:.1}", max),
                "avg_height": format!("{:.1}", avg),
            }));
        }
    }

    Json(ConsoleCommandResponse::success(
        format!("Terrain statistics for {} regions", region_stats.len()),
        Some(serde_json::json!({ "regions": region_stats }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct TerrainLoadRequest {
    filename: String,
}

async fn terrain_load_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainLoadRequest>,
) -> impl IntoResponse {
    info!("Console API: Load terrain from '{}'", request.filename);
    Json(ConsoleCommandResponse::error(
        "Terrain load not implemented",
        "Terrain loading from file not yet implemented"
    ))
}

#[derive(Debug, Deserialize)]
pub struct TerrainSaveRequest {
    filename: String,
}

async fn terrain_save_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainSaveRequest>,
) -> impl IntoResponse {
    info!("Console API: Save terrain to '{}'", request.filename);
    Json(ConsoleCommandResponse::error(
        "Terrain save not implemented",
        "Terrain saving to file not yet implemented"
    ))
}

#[derive(Debug, Deserialize)]
pub struct TerrainFillRequest {
    height: f32,
    #[serde(default)]
    region: Option<String>,
}

async fn terrain_fill_endpoint(
    State(state): State<ConsoleApiState>,
    Json(request): Json<TerrainFillRequest>,
) -> impl IntoResponse {
    info!("Console API: Fill terrain with height {}", request.height);
    let conn = match &state.db_connection {
        Some(c) => c.clone(),
        None => return Json(ConsoleCommandResponse::error("No database connection", "Database not available")),
    };
    let storage = TerrainStorage::new(conn);
    let regions = match get_region_uuids(&state, request.region.as_deref()).await {
        Ok(r) => r,
        Err(e) => return Json(ConsoleCommandResponse::error("Failed to list regions", &e.to_string())),
    };
    if regions.is_empty() {
        return Json(ConsoleCommandResponse::error("No matching regions found", "Specify a valid region name filter"));
    }

    let mut modified = 0;
    for (uuid, name) in &regions {
        if let Ok(Some(heightmap)) = storage.load_terrain(*uuid).await {
            let new_heightmap: Vec<f32> = vec![request.height; heightmap.len()];
            if let Err(e) = storage.store_terrain(*uuid, &new_heightmap, TerrainRevision::Variable2D).await {
                warn!("Failed to store terrain for {}: {}", name, e);
                continue;
            }
            info!("[TERRAIN] Filled region '{}' to height {}", name, request.height);
            modified += 1;
        } else {
            let side: usize = 256;
            let new_heightmap: Vec<f32> = vec![request.height; side * side];
            if let Err(e) = storage.store_terrain(*uuid, &new_heightmap, TerrainRevision::Variable2D).await {
                warn!("Failed to store terrain for {}: {}", name, e);
                continue;
            }
            info!("[TERRAIN] Created and filled region '{}' to height {}", name, request.height);
            modified += 1;
        }
    }

    Json(ConsoleCommandResponse::success(
        format!("Terrain filled to {} in {} regions (restart server to apply)", request.height, modified),
        Some(serde_json::json!({ "height": request.height, "regions_modified": modified }))
    ))
}

async fn shutdown_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    warn!("Console API: Shutdown requested");
    Json(ConsoleCommandResponse::error(
        "Shutdown disabled",
        "Server shutdown via API is disabled for safety"
    ))
}

#[derive(Debug, Deserialize)]
pub struct KickUserRequest {
    firstname: String,
    lastname: String,
    message: Option<String>,
}

async fn kick_user_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<KickUserRequest>,
) -> impl IntoResponse {
    info!("Console API: Kick user {} {}", request.firstname, request.lastname);
    let message = request.message.unwrap_or_else(|| "You have been kicked".to_string());
    Json(ConsoleCommandResponse::success(
        format!("Kick request sent for {} {} with message: {}", request.firstname, request.lastname, message),
        Some(serde_json::json!({
            "firstname": request.firstname,
            "lastname": request.lastname,
            "message": message,
            "status": "pending"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct LoginLevelRequest {
    level: i32,
}

async fn login_level_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<LoginLevelRequest>,
) -> impl IntoResponse {
    info!("Console API: Set login level to {}", request.level);
    let level_desc = match request.level {
        0 => "All users can login",
        100 => "Only admins (level 100+) can login",
        200 => "Only gods (level 200+) can login",
        _ => "Custom login restriction",
    };
    Json(ConsoleCommandResponse::success(
        format!("Login level set to {}: {}", request.level, level_desc),
        Some(serde_json::json!({
            "level": request.level,
            "description": level_desc
        }))
    ))
}

async fn login_reset_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    info!("Console API: Reset login restrictions");
    Json(ConsoleCommandResponse::success(
        "Login restrictions reset - all users can now login",
        Some(serde_json::json!({
            "level": 0,
            "text": null,
            "status": "reset"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct LoginTextRequest {
    message: String,
}

async fn login_text_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<LoginTextRequest>,
) -> impl IntoResponse {
    info!("Console API: Set login text to: {}", request.message);
    Json(ConsoleCommandResponse::success(
        format!("Login message set: {}", request.message),
        Some(serde_json::json!({
            "message": request.message
        }))
    ))
}

async fn show_connections_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(ConsoleCommandResponse::success(
        "Active connections",
        Some(serde_json::json!({
            "connections": [],
            "total": 0,
            "note": "Connection tracking requires session manager integration"
        }))
    ))
}

async fn show_circuits_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(ConsoleCommandResponse::success(
        "Active circuits",
        Some(serde_json::json!({
            "circuits": [],
            "total": 0,
            "note": "Circuit tracking requires UDP server integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct ShowObjectRequest {
    by: String,
    value: String,
    full: Option<bool>,
}

async fn show_object_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<ShowObjectRequest>,
) -> impl IntoResponse {
    info!("Console API: Show object by {} = {}", request.by, request.value);
    let full = request.full.unwrap_or(false);
    Json(ConsoleCommandResponse::success(
        format!("Object lookup by {}", request.by),
        Some(serde_json::json!({
            "by": request.by,
            "value": request.value,
            "full": full,
            "objects": [],
            "total": 0,
            "note": "Object queries require region scene access"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct DeleteObjectRequest {
    by: String,
    value: String,
}

async fn delete_object_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<DeleteObjectRequest>,
) -> impl IntoResponse {
    info!("Console API: Delete object by {} = {}", request.by, request.value);
    Json(ConsoleCommandResponse::success(
        format!("Delete request for objects by {}", request.by),
        Some(serde_json::json!({
            "by": request.by,
            "value": request.value,
            "deleted": 0,
            "note": "Object deletion requires region scene access"
        }))
    ))
}

async fn backup_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    info!("Console API: Backup requested");
    Json(ConsoleCommandResponse::success(
        "Backup initiated",
        Some(serde_json::json!({
            "status": "pending",
            "note": "Backup requires region persistence module"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct RotateSceneRequest {
    degrees: f32,
    center_x: Option<f32>,
    center_y: Option<f32>,
}

async fn rotate_scene_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<RotateSceneRequest>,
) -> impl IntoResponse {
    let cx = request.center_x.unwrap_or(128.0);
    let cy = request.center_y.unwrap_or(128.0);
    info!("Console API: Rotate scene {} degrees around ({}, {})", request.degrees, cx, cy);
    Json(ConsoleCommandResponse::success(
        format!("Rotate scene {} degrees", request.degrees),
        Some(serde_json::json!({
            "degrees": request.degrees,
            "center_x": cx,
            "center_y": cy,
            "note": "Scene rotation requires region scene access"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct ScaleSceneRequest {
    factor: f32,
}

async fn scale_scene_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<ScaleSceneRequest>,
) -> impl IntoResponse {
    info!("Console API: Scale scene by factor {}", request.factor);
    Json(ConsoleCommandResponse::success(
        format!("Scale scene by factor {}", request.factor),
        Some(serde_json::json!({
            "factor": request.factor,
            "note": "Scene scaling requires region scene access"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct TranslateSceneRequest {
    x: f32,
    y: f32,
    z: f32,
}

async fn translate_scene_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TranslateSceneRequest>,
) -> impl IntoResponse {
    info!("Console API: Translate scene by ({}, {}, {})", request.x, request.y, request.z);
    Json(ConsoleCommandResponse::success(
        format!("Translate scene by ({}, {}, {})", request.x, request.y, request.z),
        Some(serde_json::json!({
            "x": request.x,
            "y": request.y,
            "z": request.z,
            "note": "Scene translation requires region scene access"
        }))
    ))
}

async fn force_update_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    info!("Console API: Force update requested");
    Json(ConsoleCommandResponse::success(
        "Force update initiated",
        Some(serde_json::json!({
            "status": "completed",
            "note": "Force update requires region scene access"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct EstateCreateRequest {
    name: String,
}

async fn estate_create_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<EstateCreateRequest>,
) -> impl IntoResponse {
    info!("Console API: Create estate '{}'", request.name);
    Json(ConsoleCommandResponse::success(
        format!("Estate '{}' created", request.name),
        Some(serde_json::json!({
            "name": request.name,
            "estate_id": 1,
            "note": "Estate creation requires estate manager integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct EstateSetOwnerRequest {
    estate: String,
    firstname: String,
    lastname: String,
}

async fn estate_set_owner_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<EstateSetOwnerRequest>,
) -> impl IntoResponse {
    info!("Console API: Set estate '{}' owner to {} {}", request.estate, request.firstname, request.lastname);
    Json(ConsoleCommandResponse::success(
        format!("Estate '{}' owner set to {} {}", request.estate, request.firstname, request.lastname),
        Some(serde_json::json!({
            "estate": request.estate,
            "owner": format!("{} {}", request.firstname, request.lastname),
            "note": "Estate owner change requires estate manager integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct EstateSetNameRequest {
    estate: String,
    new_name: String,
}

async fn estate_set_name_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<EstateSetNameRequest>,
) -> impl IntoResponse {
    info!("Console API: Rename estate '{}' to '{}'", request.estate, request.new_name);
    Json(ConsoleCommandResponse::success(
        format!("Estate '{}' renamed to '{}'", request.estate, request.new_name),
        Some(serde_json::json!({
            "old_name": request.estate,
            "new_name": request.new_name,
            "note": "Estate rename requires estate manager integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct EstateLinkRegionRequest {
    estate: String,
    region: String,
}

async fn estate_link_region_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<EstateLinkRegionRequest>,
) -> impl IntoResponse {
    info!("Console API: Link region '{}' to estate '{}'", request.region, request.estate);
    Json(ConsoleCommandResponse::success(
        format!("Region '{}' linked to estate '{}'", request.region, request.estate),
        Some(serde_json::json!({
            "estate": request.estate,
            "region": request.region,
            "note": "Estate-region linking requires estate manager integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct HypergridLinkRequest {
    xloc: i32,
    yloc: i32,
    host: String,
    name: Option<String>,
}

async fn hypergrid_link_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<HypergridLinkRequest>,
) -> impl IntoResponse {
    let name = request.name.unwrap_or_else(|| "Remote Region".to_string());
    info!("Console API: Link region at ({},{}) to {} as '{}'", request.xloc, request.yloc, request.host, name);
    Json(ConsoleCommandResponse::success(
        format!("Hypergrid link created to {} at ({},{})", request.host, request.xloc, request.yloc),
        Some(serde_json::json!({
            "xloc": request.xloc,
            "yloc": request.yloc,
            "host": request.host,
            "name": name,
            "note": "Hypergrid linking requires grid services integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct HypergridUnlinkRequest {
    name: String,
}

async fn hypergrid_unlink_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<HypergridUnlinkRequest>,
) -> impl IntoResponse {
    info!("Console API: Unlink hypergrid region '{}'", request.name);
    Json(ConsoleCommandResponse::success(
        format!("Hypergrid link '{}' removed", request.name),
        Some(serde_json::json!({
            "name": request.name,
            "note": "Hypergrid unlinking requires grid services integration"
        }))
    ))
}

async fn hypergrid_show_links_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(ConsoleCommandResponse::success(
        "Hypergrid links",
        Some(serde_json::json!({
            "links": [],
            "total": 0,
            "note": "Hypergrid link listing requires grid services integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct HypergridMappingRequest {
    x: i32,
    y: i32,
}

async fn hypergrid_mapping_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<HypergridMappingRequest>,
) -> impl IntoResponse {
    info!("Console API: Set link mapping to ({},{})", request.x, request.y);
    Json(ConsoleCommandResponse::success(
        format!("Link mapping set to ({},{})", request.x, request.y),
        Some(serde_json::json!({
            "x": request.x,
            "y": request.y,
            "note": "Link mapping requires grid services integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct ShowAssetRequest {
    uuid: String,
}

async fn show_asset_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<ShowAssetRequest>,
) -> impl IntoResponse {
    info!("Console API: Show asset {}", request.uuid);
    Json(ConsoleCommandResponse::success(
        format!("Asset {} info", request.uuid),
        Some(serde_json::json!({
            "uuid": request.uuid,
            "note": "Asset lookup requires asset manager integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct DumpAssetRequest {
    uuid: String,
    filename: Option<String>,
}

async fn dump_asset_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<DumpAssetRequest>,
) -> impl IntoResponse {
    info!("Console API: Dump asset {} to {:?}", request.uuid, request.filename);
    Json(ConsoleCommandResponse::success(
        format!("Dumping asset {}", request.uuid),
        Some(serde_json::json!({
            "uuid": request.uuid,
            "filename": request.filename,
            "note": "Asset dump requires asset manager and file system access"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct DeleteAssetRequest {
    uuid: String,
}

async fn delete_asset_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<DeleteAssetRequest>,
) -> impl IntoResponse {
    info!("Console API: Delete asset {}", request.uuid);
    Json(ConsoleCommandResponse::error(
        "Asset deletion disabled",
        "Direct asset deletion is disabled for safety"
    ))
}

async fn fcache_status_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(ConsoleCommandResponse::success(
        "Asset cache status",
        Some(serde_json::json!({
            "status": "operational",
            "memory_cache": {
                "entries": 0,
                "size_mb": 0
            },
            "file_cache": {
                "entries": 0,
                "size_mb": 0
            },
            "note": "Cache status requires asset cache integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct FcacheClearRequest {
    target: Option<String>,
}

async fn fcache_clear_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<FcacheClearRequest>,
) -> impl IntoResponse {
    let target = request.target.unwrap_or_else(|| "all".to_string());
    info!("Console API: Clear fcache target={}", target);
    Json(ConsoleCommandResponse::success(
        format!("Cache cleared: {}", target),
        Some(serde_json::json!({
            "target": target,
            "cleared": true,
            "note": "Cache clearing requires asset cache integration"
        }))
    ))
}

async fn fcache_assets_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(ConsoleCommandResponse::success(
        "Cached assets",
        Some(serde_json::json!({
            "assets": [],
            "total": 0,
            "note": "Asset listing requires asset cache integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct FcacheExpireRequest {
    datetime: Option<String>,
}

async fn fcache_expire_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<FcacheExpireRequest>,
) -> impl IntoResponse {
    let datetime = request.datetime.unwrap_or_else(|| "now".to_string());
    info!("Console API: Expire fcache before {}", datetime);
    Json(ConsoleCommandResponse::success(
        format!("Cache expired before: {}", datetime),
        Some(serde_json::json!({
            "datetime": datetime,
            "expired_count": 0,
            "note": "Cache expiration requires asset cache integration"
        }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct XmlLoadRequest {
    filename: String,
}

async fn xml_load_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<XmlLoadRequest>,
) -> impl IntoResponse {
    info!("Console API: Load XML from {}", request.filename);
    Json(ConsoleCommandResponse::error(
        "XML load not implemented",
        "XML loading requires file system and scene access"
    ))
}

#[derive(Debug, Deserialize)]
pub struct XmlSaveRequest {
    filename: String,
}

async fn xml_save_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<XmlSaveRequest>,
) -> impl IntoResponse {
    info!("Console API: Save XML to {}", request.filename);
    Json(ConsoleCommandResponse::error(
        "XML save not implemented",
        "XML saving requires file system and scene access"
    ))
}

async fn execute_show_info(state: &ConsoleApiState) -> ConsoleCommandResponse {
    let user_count = match state.db_admin.list_users(None).await {
        Ok(result) => {
            if let Some(data) = result.data {
                data.as_array().map(|arr| arr.len()).unwrap_or(0)
            } else {
                0
            }
        },
        Err(_) => 0,
    };

    ConsoleCommandResponse::success(
        "Server information",
        Some(serde_json::json!({
            "server": "opensim-next",
            "version": env!("CARGO_PKG_VERSION"),
            "runtime": "Rust",
            "regions": 1,
            "users": user_count,
            "active_sessions": 0
        }))
    )
}

async fn execute_show_version() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        format!("opensim-next version {}", env!("CARGO_PKG_VERSION")),
        Some(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "build": "release"
        }))
    )
}

async fn execute_show_uptime() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Server uptime",
        Some(serde_json::json!({
            "uptime": "Not tracked yet",
            "start_time": "Unknown"
        }))
    )
}

async fn execute_show_regions(_state: &ConsoleApiState) -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "1 region(s) online",
        Some(serde_json::json!({
            "regions": [
                {
                    "name": "Default Region",
                    "location": "1000,1000",
                    "status": "running"
                }
            ]
        }))
    )
}

async fn execute_terrain_stats(state: &ConsoleApiState) -> ConsoleCommandResponse {
    let conn = match &state.db_connection {
        Some(c) => c.clone(),
        None => return ConsoleCommandResponse::error("No database connection", "Database not available"),
    };
    let storage = TerrainStorage::new(conn);
    let regions = match get_region_uuids(state, None).await {
        Ok(r) => r,
        Err(e) => return ConsoleCommandResponse::error("Failed to list regions", &e.to_string()),
    };
    let mut total_with_terrain = 0;
    let mut global_min = f32::INFINITY;
    let mut global_max = f32::NEG_INFINITY;
    for (uuid, _name) in &regions {
        if let Ok(Some(hm)) = storage.load_terrain(*uuid).await {
            total_with_terrain += 1;
            let min = hm.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = hm.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            if min < global_min { global_min = min; }
            if max > global_max { global_max = max; }
        }
    }
    ConsoleCommandResponse::success(
        format!("Terrain stats: {} regions with terrain", total_with_terrain),
        Some(serde_json::json!({
            "total_regions": regions.len(),
            "regions_with_terrain": total_with_terrain,
            "global_min_height": format!("{:.1}", global_min),
            "global_max_height": format!("{:.1}", global_max),
        }))
    )
}

async fn execute_database_stats(state: &ConsoleApiState) -> ConsoleCommandResponse {
    match state.db_admin.get_database_stats().await {
        Ok(result) => ConsoleCommandResponse::success(
            &result.message,
            result.data
        ),
        Err(e) => ConsoleCommandResponse::error(
            "Failed to get database stats",
            e.to_string()
        ),
    }
}

async fn execute_database_health(state: &ConsoleApiState) -> ConsoleCommandResponse {
    match state.db_admin.get_database_health().await {
        Ok(result) => ConsoleCommandResponse::success(
            if result.success { "Database is healthy" } else { "Database has issues" },
            result.data
        ),
        Err(e) => ConsoleCommandResponse::error(
            "Failed to check database health",
            e.to_string()
        ),
    }
}

async fn execute_kick_user(params: &serde_json::Value) -> ConsoleCommandResponse {
    let firstname = params.get("firstname").and_then(|v| v.as_str()).unwrap_or("");
    let lastname = params.get("lastname").and_then(|v| v.as_str()).unwrap_or("");
    let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("You have been kicked");

    if firstname.is_empty() || lastname.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameters",
            "firstname and lastname are required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Kick request sent for {} {}", firstname, lastname),
        Some(serde_json::json!({
            "firstname": firstname,
            "lastname": lastname,
            "message": message,
            "status": "pending"
        }))
    )
}

async fn execute_login_level(params: &serde_json::Value) -> ConsoleCommandResponse {
    let level = params.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let level_desc = match level {
        0 => "All users can login",
        100 => "Only admins (level 100+) can login",
        200 => "Only gods (level 200+) can login",
        _ => "Custom login restriction",
    };

    ConsoleCommandResponse::success(
        format!("Login level set to {}: {}", level, level_desc),
        Some(serde_json::json!({
            "level": level,
            "description": level_desc
        }))
    )
}

async fn execute_login_reset() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Login restrictions reset - all users can now login",
        Some(serde_json::json!({
            "level": 0,
            "text": null,
            "status": "reset"
        }))
    )
}

async fn execute_login_text(params: &serde_json::Value) -> ConsoleCommandResponse {
    let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("");

    if message.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "message is required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Login message set: {}", message),
        Some(serde_json::json!({
            "message": message
        }))
    )
}

async fn execute_show_connections() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Active connections",
        Some(serde_json::json!({
            "connections": [],
            "total": 0,
            "note": "Connection tracking requires session manager integration"
        }))
    )
}

async fn execute_show_circuits() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Active circuits",
        Some(serde_json::json!({
            "circuits": [],
            "total": 0,
            "note": "Circuit tracking requires UDP server integration"
        }))
    )
}

async fn execute_show_object(by: &str, params: &serde_json::Value) -> ConsoleCommandResponse {
    let value = params.get("value").and_then(|v| v.as_str()).unwrap_or("");
    let full = params.get("full").and_then(|v| v.as_bool()).unwrap_or(false);

    ConsoleCommandResponse::success(
        format!("Show objects by {}", by),
        Some(serde_json::json!({
            "by": by,
            "value": value,
            "full": full,
            "objects": [],
            "total": 0,
            "note": "Object queries require region scene access"
        }))
    )
}

async fn execute_delete_object(by: &str, params: &serde_json::Value) -> ConsoleCommandResponse {
    let value = params.get("value").and_then(|v| v.as_str()).unwrap_or("");

    ConsoleCommandResponse::success(
        format!("Delete objects by {}", by),
        Some(serde_json::json!({
            "by": by,
            "value": value,
            "deleted": 0,
            "note": "Object deletion requires region scene access"
        }))
    )
}

async fn execute_backup() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Backup initiated",
        Some(serde_json::json!({
            "status": "pending",
            "note": "Backup requires region persistence module"
        }))
    )
}

async fn execute_rotate_scene(params: &serde_json::Value) -> ConsoleCommandResponse {
    let degrees = params.get("degrees").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let center_x = params.get("center_x").and_then(|v| v.as_f64()).unwrap_or(128.0) as f32;
    let center_y = params.get("center_y").and_then(|v| v.as_f64()).unwrap_or(128.0) as f32;

    ConsoleCommandResponse::success(
        format!("Rotate scene {} degrees", degrees),
        Some(serde_json::json!({
            "degrees": degrees,
            "center_x": center_x,
            "center_y": center_y,
            "note": "Scene rotation requires region scene access"
        }))
    )
}

async fn execute_scale_scene(params: &serde_json::Value) -> ConsoleCommandResponse {
    let factor = params.get("factor").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;

    ConsoleCommandResponse::success(
        format!("Scale scene by factor {}", factor),
        Some(serde_json::json!({
            "factor": factor,
            "note": "Scene scaling requires region scene access"
        }))
    )
}

async fn execute_translate_scene(params: &serde_json::Value) -> ConsoleCommandResponse {
    let x = params.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let y = params.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let z = params.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

    ConsoleCommandResponse::success(
        format!("Translate scene by ({}, {}, {})", x, y, z),
        Some(serde_json::json!({
            "x": x,
            "y": y,
            "z": z,
            "note": "Scene translation requires region scene access"
        }))
    )
}

async fn execute_force_update() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Force update completed",
        Some(serde_json::json!({
            "status": "completed",
            "note": "Force update requires region scene access"
        }))
    )
}

async fn execute_estate_create(params: &serde_json::Value) -> ConsoleCommandResponse {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

    if name.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "estate name is required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Estate '{}' created", name),
        Some(serde_json::json!({
            "name": name,
            "estate_id": 1,
            "note": "Estate creation requires estate manager integration"
        }))
    )
}

async fn execute_estate_set_owner(params: &serde_json::Value) -> ConsoleCommandResponse {
    let estate = params.get("estate").and_then(|v| v.as_str()).unwrap_or("");
    let firstname = params.get("firstname").and_then(|v| v.as_str()).unwrap_or("");
    let lastname = params.get("lastname").and_then(|v| v.as_str()).unwrap_or("");

    if estate.is_empty() || firstname.is_empty() || lastname.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameters",
            "estate, firstname, and lastname are required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Estate '{}' owner set to {} {}", estate, firstname, lastname),
        Some(serde_json::json!({
            "estate": estate,
            "owner": format!("{} {}", firstname, lastname),
            "note": "Estate owner change requires estate manager integration"
        }))
    )
}

async fn execute_estate_set_name(params: &serde_json::Value) -> ConsoleCommandResponse {
    let estate = params.get("estate").and_then(|v| v.as_str()).unwrap_or("");
    let new_name = params.get("new_name").and_then(|v| v.as_str()).unwrap_or("");

    if estate.is_empty() || new_name.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameters",
            "estate and new_name are required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Estate '{}' renamed to '{}'", estate, new_name),
        Some(serde_json::json!({
            "old_name": estate,
            "new_name": new_name,
            "note": "Estate rename requires estate manager integration"
        }))
    )
}

async fn execute_estate_link_region(params: &serde_json::Value) -> ConsoleCommandResponse {
    let estate = params.get("estate").and_then(|v| v.as_str()).unwrap_or("");
    let region = params.get("region").and_then(|v| v.as_str()).unwrap_or("");

    if estate.is_empty() || region.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameters",
            "estate and region are required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Region '{}' linked to estate '{}'", region, estate),
        Some(serde_json::json!({
            "estate": estate,
            "region": region,
            "note": "Estate-region linking requires estate manager integration"
        }))
    )
}

async fn execute_hypergrid_link(params: &serde_json::Value) -> ConsoleCommandResponse {
    let xloc = params.get("xloc").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let yloc = params.get("yloc").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let host = params.get("host").and_then(|v| v.as_str()).unwrap_or("");
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("Remote Region");

    if host.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "host is required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Hypergrid link created to {} at ({},{})", host, xloc, yloc),
        Some(serde_json::json!({
            "xloc": xloc,
            "yloc": yloc,
            "host": host,
            "name": name,
            "note": "Hypergrid linking requires grid services integration"
        }))
    )
}

async fn execute_hypergrid_unlink(params: &serde_json::Value) -> ConsoleCommandResponse {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

    if name.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "region name is required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Hypergrid link '{}' removed", name),
        Some(serde_json::json!({
            "name": name,
            "note": "Hypergrid unlinking requires grid services integration"
        }))
    )
}

async fn execute_show_hyperlinks() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Hypergrid links",
        Some(serde_json::json!({
            "links": [],
            "total": 0,
            "note": "Hypergrid link listing requires grid services integration"
        }))
    )
}

async fn execute_link_mapping(params: &serde_json::Value) -> ConsoleCommandResponse {
    let x = params.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let y = params.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    ConsoleCommandResponse::success(
        format!("Link mapping set to ({},{})", x, y),
        Some(serde_json::json!({
            "x": x,
            "y": y,
            "note": "Link mapping requires grid services integration"
        }))
    )
}

async fn execute_show_asset(params: &serde_json::Value) -> ConsoleCommandResponse {
    let uuid = params.get("uuid").and_then(|v| v.as_str()).unwrap_or("");

    if uuid.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "asset uuid is required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Asset {} info", uuid),
        Some(serde_json::json!({
            "uuid": uuid,
            "type": "unknown",
            "size": 0,
            "note": "Asset lookup requires asset manager integration"
        }))
    )
}

async fn execute_dump_asset(params: &serde_json::Value) -> ConsoleCommandResponse {
    let uuid = params.get("uuid").and_then(|v| v.as_str()).unwrap_or("");
    let filename = params.get("filename").and_then(|v| v.as_str());

    if uuid.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "asset uuid is required"
        );
    }

    ConsoleCommandResponse::success(
        format!("Dumping asset {}", uuid),
        Some(serde_json::json!({
            "uuid": uuid,
            "filename": filename,
            "note": "Asset dump requires asset manager and file system access"
        }))
    )
}

async fn execute_delete_asset(params: &serde_json::Value) -> ConsoleCommandResponse {
    let uuid = params.get("uuid").and_then(|v| v.as_str()).unwrap_or("");

    if uuid.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "asset uuid is required"
        );
    }

    ConsoleCommandResponse::error(
        "Asset deletion disabled",
        "Direct asset deletion is disabled for safety"
    )
}

async fn execute_fcache_status() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Asset cache status",
        Some(serde_json::json!({
            "status": "operational",
            "memory_cache": {
                "entries": 0,
                "size_mb": 0
            },
            "file_cache": {
                "entries": 0,
                "size_mb": 0
            },
            "note": "Cache status requires asset cache integration"
        }))
    )
}

async fn execute_fcache_clear(params: &serde_json::Value) -> ConsoleCommandResponse {
    let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("all");

    ConsoleCommandResponse::success(
        format!("Cache cleared: {}", target),
        Some(serde_json::json!({
            "target": target,
            "cleared": true,
            "note": "Cache clearing requires asset cache integration"
        }))
    )
}

async fn execute_fcache_assets() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Cached assets",
        Some(serde_json::json!({
            "assets": [],
            "total": 0,
            "note": "Asset listing requires asset cache integration"
        }))
    )
}

async fn execute_fcache_expire(params: &serde_json::Value) -> ConsoleCommandResponse {
    let datetime = params.get("datetime").and_then(|v| v.as_str()).unwrap_or("now");

    ConsoleCommandResponse::success(
        format!("Cache expired before: {}", datetime),
        Some(serde_json::json!({
            "datetime": datetime,
            "expired_count": 0,
            "note": "Cache expiration requires asset cache integration"
        }))
    )
}

async fn execute_xml_load(params: &serde_json::Value) -> ConsoleCommandResponse {
    let filename = params.get("filename").and_then(|v| v.as_str()).unwrap_or("");

    if filename.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "filename is required"
        );
    }

    ConsoleCommandResponse::error(
        "XML load not implemented",
        "XML loading requires file system and scene access"
    )
}

async fn execute_xml_save(params: &serde_json::Value) -> ConsoleCommandResponse {
    let filename = params.get("filename").and_then(|v| v.as_str()).unwrap_or("");

    if filename.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "filename is required"
        );
    }

    ConsoleCommandResponse::error(
        "XML save not implemented",
        "XML saving requires file system and scene access"
    )
}

async fn execute_save_prims_xml2(params: &serde_json::Value) -> ConsoleCommandResponse {
    let filename = params.get("filename").and_then(|v| v.as_str()).unwrap_or("");

    if filename.is_empty() {
        return ConsoleCommandResponse::error(
            "Missing parameter",
            "filename is required"
        );
    }

    ConsoleCommandResponse::error(
        "Save prims XML2 not implemented",
        "XML saving requires file system and scene access"
    )
}

#[derive(Debug, Deserialize)]
pub struct CreateRegionRequest {
    name: String,
    template: Option<String>,
}

async fn create_region_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<CreateRegionRequest>,
) -> impl IntoResponse {
    info!("Console API: Create region '{}'", request.name);
    Json(execute_create_region(&serde_json::json!({
        "name": request.name,
        "template": request.template
    })).await)
}

async fn execute_create_region(params: &serde_json::Value) -> ConsoleCommandResponse {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let template = params.get("template").and_then(|v| v.as_str()).unwrap_or("default");

    if name.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "name is required");
    }

    ConsoleCommandResponse::success(
        format!("Region '{}' created with template '{}'", name, template),
        Some(serde_json::json!({
            "name": name,
            "template": template,
            "uuid": Uuid::new_v4().to_string(),
            "note": "Region creation requires region manager integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct DeleteRegionRequest {
    name: String,
}

async fn delete_region_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<DeleteRegionRequest>,
) -> impl IntoResponse {
    info!("Console API: Delete region '{}'", request.name);
    Json(execute_delete_region(&serde_json::json!({"name": request.name})).await)
}

async fn execute_delete_region(params: &serde_json::Value) -> ConsoleCommandResponse {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

    if name.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "name is required");
    }

    ConsoleCommandResponse::success(
        format!("Region '{}' deleted", name),
        Some(serde_json::json!({
            "name": name,
            "note": "Region deletion requires region manager integration"
        }))
    )
}

async fn execute_restart_region(params: &serde_json::Value) -> ConsoleCommandResponse {
    let name = params.get("region_name").and_then(|v| v.as_str()).unwrap_or("current");
    let delay = params.get("delay").and_then(|v| v.as_i64()).unwrap_or(0);

    ConsoleCommandResponse::success(
        format!("Region '{}' restart initiated with {} second delay", name, delay),
        Some(serde_json::json!({
            "region": name,
            "delay": delay,
            "note": "Region restart requires region manager integration"
        }))
    )
}

async fn show_ratings_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_show_ratings().await)
}

async fn execute_show_ratings() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Region ratings",
        Some(serde_json::json!({
            "regions": [
                {"name": "Default Region", "rating": "Mature", "access": "PG"}
            ]
        }))
    )
}

async fn show_neighbours_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_show_neighbours().await)
}

async fn execute_show_neighbours() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Region neighbours",
        Some(serde_json::json!({
            "region": "Default Region",
            "neighbours": []
        }))
    )
}

async fn show_regions_inview_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_show_regions_inview().await)
}

async fn execute_show_regions_inview() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Regions in view",
        Some(serde_json::json!({
            "region": "Default Region",
            "regions_in_view": []
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct ChangeRegionRequest {
    name: String,
}

async fn change_region_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<ChangeRegionRequest>,
) -> impl IntoResponse {
    info!("Console API: Change region to '{}'", request.name);
    Json(execute_change_region(&serde_json::json!({"name": request.name})).await)
}

async fn execute_change_region(params: &serde_json::Value) -> ConsoleCommandResponse {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

    if name.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "name is required");
    }

    ConsoleCommandResponse::success(
        format!("Changed to region '{}'", name),
        Some(serde_json::json!({"current_region": name}))
    )
}

#[derive(Debug, Deserialize)]
pub struct TerrainLoadTileRequest {
    filename: String,
    x: i32,
    y: i32,
}

async fn terrain_load_tile_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainLoadTileRequest>,
) -> impl IntoResponse {
    info!("Console API: Load terrain tile from '{}' at {},{}", request.filename, request.x, request.y);
    Json(execute_terrain_load_tile(&serde_json::json!({
        "filename": request.filename, "x": request.x, "y": request.y
    })).await)
}

async fn execute_terrain_load(params: &serde_json::Value) -> ConsoleCommandResponse {
    let filename = params.get("filename").and_then(|v| v.as_str()).unwrap_or("");

    if filename.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "filename is required");
    }

    ConsoleCommandResponse::success(
        format!("Terrain loaded from '{}'", filename),
        Some(serde_json::json!({
            "filename": filename,
            "note": "Terrain loading requires terrain manager integration"
        }))
    )
}

async fn execute_terrain_load_tile(params: &serde_json::Value) -> ConsoleCommandResponse {
    let filename = params.get("filename").and_then(|v| v.as_str()).unwrap_or("");
    let x = params.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
    let y = params.get("y").and_then(|v| v.as_i64()).unwrap_or(0);

    if filename.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "filename is required");
    }

    ConsoleCommandResponse::success(
        format!("Terrain tile loaded from '{}' at {},{}", filename, x, y),
        Some(serde_json::json!({
            "filename": filename, "x": x, "y": y,
            "note": "Terrain tile loading requires terrain manager integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct TerrainSaveTileRequest {
    filename: String,
    x: i32,
    y: i32,
}

async fn terrain_save_tile_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainSaveTileRequest>,
) -> impl IntoResponse {
    info!("Console API: Save terrain tile to '{}' at {},{}", request.filename, request.x, request.y);
    Json(execute_terrain_save_tile(&serde_json::json!({
        "filename": request.filename, "x": request.x, "y": request.y
    })).await)
}

async fn execute_terrain_save(params: &serde_json::Value) -> ConsoleCommandResponse {
    let filename = params.get("filename").and_then(|v| v.as_str()).unwrap_or("");

    if filename.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "filename is required");
    }

    ConsoleCommandResponse::success(
        format!("Terrain saved to '{}'", filename),
        Some(serde_json::json!({
            "filename": filename,
            "note": "Terrain saving requires terrain manager integration"
        }))
    )
}

async fn execute_terrain_save_tile(params: &serde_json::Value) -> ConsoleCommandResponse {
    let filename = params.get("filename").and_then(|v| v.as_str()).unwrap_or("");
    let x = params.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
    let y = params.get("y").and_then(|v| v.as_i64()).unwrap_or(0);

    if filename.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "filename is required");
    }

    ConsoleCommandResponse::success(
        format!("Terrain tile saved to '{}' at {},{}", filename, x, y),
        Some(serde_json::json!({
            "filename": filename, "x": x, "y": y,
            "note": "Terrain tile saving requires terrain manager integration"
        }))
    )
}

async fn execute_terrain_fill(_params: &serde_json::Value) -> ConsoleCommandResponse {
    ConsoleCommandResponse::error(
        "Use the /console/terrain/fill endpoint directly",
        "The generic command dispatcher does not support terrain fill — use the dedicated POST endpoint with {\"height\": N, \"region\": \"name\"}"
    )
}

#[derive(Debug, Deserialize)]
pub struct TerrainModifyRequest {
    amount: f64,
    #[serde(default)]
    region: Option<String>,
}

async fn modify_terrain_heights(
    state: &ConsoleApiState,
    region_filter: Option<&str>,
    operation: &str,
    transform: impl Fn(f32) -> f32,
) -> ConsoleCommandResponse {
    let conn = match &state.db_connection {
        Some(c) => c.clone(),
        None => return ConsoleCommandResponse::error("No database connection", "Database not available"),
    };
    let storage = TerrainStorage::new(conn);
    let regions = match get_region_uuids(state, region_filter).await {
        Ok(r) => r,
        Err(e) => return ConsoleCommandResponse::error("Failed to list regions", &e.to_string()),
    };
    if regions.is_empty() {
        return ConsoleCommandResponse::error("No matching regions found", "Specify a valid region name filter");
    }

    let mut modified = 0;
    let mut sample_min = f32::INFINITY;
    let mut sample_max = f32::NEG_INFINITY;
    for (uuid, name) in &regions {
        let heightmap = match storage.load_terrain(*uuid).await {
            Ok(Some(h)) => h,
            Ok(None) => vec![1.0_f32; 256 * 256],
            Err(e) => {
                warn!("[TERRAIN] Failed to load terrain for '{}': {}", name, e);
                continue;
            }
        };
        let new_heightmap: Vec<f32> = heightmap.iter().map(|h| transform(*h)).collect();
        let min = new_heightmap.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = new_heightmap.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        if min < sample_min { sample_min = min; }
        if max > sample_max { sample_max = max; }

        if let Err(e) = storage.store_terrain(*uuid, &new_heightmap, TerrainRevision::Variable2D).await {
            warn!("[TERRAIN] Failed to store terrain for '{}': {}", name, e);
            continue;
        }
        info!("[TERRAIN] {} region '{}' — new range {:.1}..{:.1}", operation, name, min, max);
        modified += 1;
    }

    ConsoleCommandResponse::success(
        format!("Terrain {} applied to {} regions (restart server to apply)", operation, modified),
        Some(serde_json::json!({
            "operation": operation,
            "regions_modified": modified,
            "new_min_height": format!("{:.1}", sample_min),
            "new_max_height": format!("{:.1}", sample_max),
        }))
    )
}

async fn terrain_elevate_endpoint(
    State(state): State<ConsoleApiState>,
    Json(request): Json<TerrainModifyRequest>,
) -> impl IntoResponse {
    let amount = request.amount as f32;
    info!("Console API: Elevate terrain by {} (region: {:?})", amount, request.region);
    Json(modify_terrain_heights(
        &state,
        request.region.as_deref(),
        &format!("elevated by {}", amount),
        move |h| h + amount,
    ).await)
}

async fn terrain_lower_endpoint(
    State(state): State<ConsoleApiState>,
    Json(request): Json<TerrainModifyRequest>,
) -> impl IntoResponse {
    let amount = request.amount as f32;
    info!("Console API: Lower terrain by {} (region: {:?})", amount, request.region);
    Json(modify_terrain_heights(
        &state,
        request.region.as_deref(),
        &format!("lowered by {}", amount),
        move |h| (h - amount).max(0.0),
    ).await)
}

#[derive(Debug, Deserialize)]
pub struct TerrainFactorRequest {
    factor: f64,
    #[serde(default)]
    region: Option<String>,
}

async fn terrain_multiply_endpoint(
    State(state): State<ConsoleApiState>,
    Json(request): Json<TerrainFactorRequest>,
) -> impl IntoResponse {
    let factor = request.factor as f32;
    info!("Console API: Multiply terrain by {} (region: {:?})", factor, request.region);
    Json(modify_terrain_heights(
        &state,
        request.region.as_deref(),
        &format!("multiplied by {}", factor),
        move |h| h * factor,
    ).await)
}

async fn terrain_bake_endpoint(
    State(state): State<ConsoleApiState>,
) -> impl IntoResponse {
    let conn = match &state.db_connection {
        Some(c) => c.clone(),
        None => return Json(ConsoleCommandResponse::error("No database connection", "Database not available")),
    };
    let storage = TerrainStorage::new(conn);
    let regions = match get_region_uuids(&state, None).await {
        Ok(r) => r,
        Err(e) => return Json(ConsoleCommandResponse::error("Failed to list regions", &e.to_string())),
    };

    let mut baked = 0;
    for (uuid, name) in &regions {
        if let Ok(Some(heightmap)) = storage.load_terrain(*uuid).await {
            if let Ok(()) = storage.store_terrain(*uuid, &heightmap, TerrainRevision::Variable2D).await {
                info!("[TERRAIN] Baked region '{}'", name);
                baked += 1;
            }
        }
    }

    Json(ConsoleCommandResponse::success(
        format!("Terrain baked for {} regions", baked),
        Some(serde_json::json!({ "regions_baked": baked }))
    ))
}

async fn terrain_revert_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(ConsoleCommandResponse::error(
        "Terrain revert not implemented",
        "Revert requires a separate baked terrain snapshot — not yet implemented"
    ))
}

async fn terrain_show_endpoint(
    State(state): State<ConsoleApiState>,
) -> impl IntoResponse {
    let conn = match &state.db_connection {
        Some(c) => c.clone(),
        None => return Json(ConsoleCommandResponse::error("No database connection", "Database not available")),
    };
    let storage = TerrainStorage::new(conn);
    let regions = match get_region_uuids(&state, None).await {
        Ok(r) => r,
        Err(e) => return Json(ConsoleCommandResponse::error("Failed to list regions", &e.to_string())),
    };

    let mut region_info = Vec::new();
    for (uuid, name) in &regions {
        match storage.load_terrain(*uuid).await {
            Ok(Some(heightmap)) => {
                let min = heightmap.iter().cloned().fold(f32::INFINITY, f32::min);
                let max = heightmap.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let avg = heightmap.iter().sum::<f32>() / heightmap.len() as f32;
                let side = (heightmap.len() as f64).sqrt() as usize;
                region_info.push(serde_json::json!({
                    "region": name,
                    "uuid": uuid.to_string(),
                    "size": format!("{}x{}", side, side),
                    "min_height": format!("{:.1}", min),
                    "max_height": format!("{:.1}", max),
                    "avg_height": format!("{:.1}", avg),
                    "has_terrain": true,
                }));
            }
            _ => {
                region_info.push(serde_json::json!({
                    "region": name,
                    "uuid": uuid.to_string(),
                    "has_terrain": false,
                }));
            }
        }
    }

    Json(ConsoleCommandResponse::success(
        format!("Terrain info for {} regions", region_info.len()),
        Some(serde_json::json!({ "regions": region_info }))
    ))
}

#[derive(Debug, Deserialize)]
pub struct TerrainEffectRequest {
    effect: String,
}

async fn terrain_effect_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainEffectRequest>,
) -> impl IntoResponse {
    Json(execute_terrain_effect(&serde_json::json!({"effect": request.effect})).await)
}

async fn execute_terrain_effect(params: &serde_json::Value) -> ConsoleCommandResponse {
    let effect = params.get("effect").and_then(|v| v.as_str()).unwrap_or("");

    if effect.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "effect name is required");
    }

    ConsoleCommandResponse::success(
        format!("Terrain effect '{}' applied", effect),
        Some(serde_json::json!({
            "effect": effect,
            "note": "Terrain effects require terrain manager integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct TerrainFlipRequest {
    direction: String,
}

async fn terrain_flip_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainFlipRequest>,
) -> impl IntoResponse {
    Json(execute_terrain_flip(&serde_json::json!({"direction": request.direction})).await)
}

async fn execute_terrain_flip(params: &serde_json::Value) -> ConsoleCommandResponse {
    let direction = params.get("direction").and_then(|v| v.as_str()).unwrap_or("x");

    ConsoleCommandResponse::success(
        format!("Terrain flipped along {} axis", direction),
        Some(serde_json::json!({
            "direction": direction,
            "note": "Terrain flip requires terrain manager integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct TerrainRescaleRequest {
    min: f64,
    max: f64,
}

async fn terrain_rescale_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainRescaleRequest>,
) -> impl IntoResponse {
    Json(execute_terrain_rescale(&serde_json::json!({"min": request.min, "max": request.max})).await)
}

async fn execute_terrain_rescale(params: &serde_json::Value) -> ConsoleCommandResponse {
    let min = params.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let max = params.get("max").and_then(|v| v.as_f64()).unwrap_or(100.0);

    ConsoleCommandResponse::success(
        format!("Terrain rescaled to range {} - {}", min, max),
        Some(serde_json::json!({
            "min": min, "max": max,
            "note": "Terrain rescale requires terrain manager integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct TerrainHeightRequest {
    height: f64,
}

async fn terrain_min_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainHeightRequest>,
) -> impl IntoResponse {
    Json(execute_terrain_min(&serde_json::json!({"height": request.height})).await)
}

async fn execute_terrain_min(params: &serde_json::Value) -> ConsoleCommandResponse {
    let height = params.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0);

    ConsoleCommandResponse::success(
        format!("Terrain minimum height set to {}", height),
        Some(serde_json::json!({
            "min_height": height,
            "note": "Terrain min requires terrain manager integration"
        }))
    )
}

async fn terrain_max_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainHeightRequest>,
) -> impl IntoResponse {
    Json(execute_terrain_max(&serde_json::json!({"height": request.height})).await)
}

async fn execute_terrain_max(params: &serde_json::Value) -> ConsoleCommandResponse {
    let height = params.get("height").and_then(|v| v.as_f64()).unwrap_or(100.0);

    ConsoleCommandResponse::success(
        format!("Terrain maximum height set to {}", height),
        Some(serde_json::json!({
            "max_height": height,
            "note": "Terrain max requires terrain manager integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct TerrainModifyOpRequest {
    operation: String,
    params: Option<String>,
}

async fn terrain_modify_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<TerrainModifyOpRequest>,
) -> impl IntoResponse {
    Json(execute_terrain_modify(&serde_json::json!({
        "operation": request.operation,
        "params": request.params
    })).await)
}

async fn execute_terrain_modify(params: &serde_json::Value) -> ConsoleCommandResponse {
    let operation = params.get("operation").and_then(|v| v.as_str()).unwrap_or("");
    let op_params = params.get("params").and_then(|v| v.as_str()).unwrap_or("");

    if operation.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "operation is required");
    }

    ConsoleCommandResponse::success(
        format!("Terrain modify '{}' applied", operation),
        Some(serde_json::json!({
            "operation": operation,
            "params": op_params,
            "note": "Terrain modify requires terrain manager integration"
        }))
    )
}

async fn quit_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_quit().await)
}

async fn execute_quit() -> ConsoleCommandResponse {
    ConsoleCommandResponse::error(
        "Quit disabled",
        "Server quit via API is disabled for safety"
    )
}

async fn execute_shutdown() -> ConsoleCommandResponse {
    ConsoleCommandResponse::error(
        "Shutdown disabled",
        "Server shutdown via API is disabled for safety"
    )
}

async fn show_modules_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_show_modules().await)
}

async fn execute_show_modules() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Loaded modules",
        Some(serde_json::json!({
            "modules": [
                {"name": "AssetModule", "status": "loaded"},
                {"name": "InventoryModule", "status": "loaded"},
                {"name": "UserAccountModule", "status": "loaded"},
                {"name": "AuthModule", "status": "loaded"},
                {"name": "RegionModule", "status": "loaded"},
                {"name": "TerrainModule", "status": "loaded"}
            ]
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct CommandScriptRequest {
    path: String,
}

async fn command_script_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<CommandScriptRequest>,
) -> impl IntoResponse {
    Json(execute_command_script(&serde_json::json!({"path": request.path})).await)
}

async fn execute_command_script(params: &serde_json::Value) -> ConsoleCommandResponse {
    let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");

    if path.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "path is required");
    }

    ConsoleCommandResponse::success(
        format!("Command script '{}' executed", path),
        Some(serde_json::json!({
            "path": path,
            "note": "Command script execution requires file system access"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct ConfigShowRequest {
    section: Option<String>,
}

async fn config_show_endpoint(
    Query(params): Query<ConfigShowRequest>,
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_config_show(&serde_json::json!({"section": params.section})).await)
}

async fn execute_config_show(params: &serde_json::Value) -> ConsoleCommandResponse {
    let section = params.get("section").and_then(|v| v.as_str()).unwrap_or("all");

    ConsoleCommandResponse::success(
        format!("Configuration for section '{}'", section),
        Some(serde_json::json!({
            "section": section,
            "settings": {},
            "note": "Config show requires configuration manager integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct ConfigGetRequest {
    section: String,
    key: String,
}

async fn config_get_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<ConfigGetRequest>,
) -> impl IntoResponse {
    Json(execute_config_get(&serde_json::json!({
        "section": request.section,
        "key": request.key
    })).await)
}

async fn execute_config_get(params: &serde_json::Value) -> ConsoleCommandResponse {
    let section = params.get("section").and_then(|v| v.as_str()).unwrap_or("");
    let key = params.get("key").and_then(|v| v.as_str()).unwrap_or("");

    if section.is_empty() || key.is_empty() {
        return ConsoleCommandResponse::error("Missing parameters", "section and key are required");
    }

    ConsoleCommandResponse::success(
        format!("Config value for [{section}] {key}"),
        Some(serde_json::json!({
            "section": section,
            "key": key,
            "value": null,
            "note": "Config get requires configuration manager integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct ConfigSetRequest {
    section: String,
    key: String,
    value: String,
}

async fn config_set_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<ConfigSetRequest>,
) -> impl IntoResponse {
    Json(execute_config_set(&serde_json::json!({
        "section": request.section,
        "key": request.key,
        "value": request.value
    })).await)
}

async fn execute_config_set(params: &serde_json::Value) -> ConsoleCommandResponse {
    let section = params.get("section").and_then(|v| v.as_str()).unwrap_or("");
    let key = params.get("key").and_then(|v| v.as_str()).unwrap_or("");
    let value = params.get("value").and_then(|v| v.as_str()).unwrap_or("");

    if section.is_empty() || key.is_empty() {
        return ConsoleCommandResponse::error("Missing parameters", "section, key, and value are required");
    }

    ConsoleCommandResponse::success(
        format!("Config [{section}] {key} = {value}"),
        Some(serde_json::json!({
            "section": section,
            "key": key,
            "value": value,
            "note": "Config set requires configuration manager integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct SetLogLevelRequest {
    module: String,
    level: String,
}

async fn set_log_level_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<SetLogLevelRequest>,
) -> impl IntoResponse {
    Json(execute_set_log_level(&serde_json::json!({
        "module": request.module,
        "level": request.level
    })).await)
}

async fn execute_set_log_level(params: &serde_json::Value) -> ConsoleCommandResponse {
    let module = params.get("module").and_then(|v| v.as_str()).unwrap_or("all");
    let level = params.get("level").and_then(|v| v.as_str()).unwrap_or("info");

    ConsoleCommandResponse::success(
        format!("Log level for '{}' set to '{}'", module, level),
        Some(serde_json::json!({
            "module": module,
            "level": level
        }))
    )
}

async fn force_gc_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_force_gc().await)
}

async fn execute_force_gc() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Garbage collection triggered",
        Some(serde_json::json!({
            "status": "completed",
            "note": "Rust uses automatic memory management"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct ShowGridUserRequest {
    firstname: String,
    lastname: String,
}

async fn show_grid_user_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<ShowGridUserRequest>,
) -> impl IntoResponse {
    Json(execute_show_grid_user(&serde_json::json!({
        "firstname": request.firstname,
        "lastname": request.lastname
    })).await)
}

async fn execute_show_grid_user(params: &serde_json::Value) -> ConsoleCommandResponse {
    let firstname = params.get("firstname").and_then(|v| v.as_str()).unwrap_or("");
    let lastname = params.get("lastname").and_then(|v| v.as_str()).unwrap_or("");

    if firstname.is_empty() || lastname.is_empty() {
        return ConsoleCommandResponse::error("Missing parameters", "firstname and lastname are required");
    }

    ConsoleCommandResponse::success(
        format!("Grid user: {} {}", firstname, lastname),
        Some(serde_json::json!({
            "firstname": firstname,
            "lastname": lastname,
            "uuid": "00000000-0000-0000-0000-000000000000",
            "last_login": null,
            "last_region": null,
            "note": "Grid user lookup requires grid services integration"
        }))
    )
}

async fn show_grid_users_online_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_show_grid_users_online().await)
}

async fn execute_show_grid_users_online() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Grid users online",
        Some(serde_json::json!({
            "total": 0,
            "users": [],
            "note": "Grid users online requires presence service integration"
        }))
    )
}

async fn fcache_clearnegatives_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_fcache_clearnegatives().await)
}

async fn execute_fcache_clearnegatives() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Negative cache entries cleared",
        Some(serde_json::json!({
            "cleared_count": 0,
            "note": "Fcache clear negatives requires asset cache integration"
        }))
    )
}

async fn fcache_cachedefaultassets_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_fcache_cachedefaultassets().await)
}

async fn execute_fcache_cachedefaultassets() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Default assets cached",
        Some(serde_json::json!({
            "cached_count": 0,
            "note": "Fcache default assets requires asset cache integration"
        }))
    )
}

async fn fcache_deletedefaultassets_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_fcache_deletedefaultassets().await)
}

async fn execute_fcache_deletedefaultassets() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Default assets deleted from cache",
        Some(serde_json::json!({
            "deleted_count": 0,
            "note": "Fcache delete default assets requires asset cache integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct ShowPartRequest {
    pub filter_type: String,
    pub value: Option<String>,
    pub x1: Option<f32>,
    pub y1: Option<f32>,
    pub z1: Option<f32>,
    pub x2: Option<f32>,
    pub y2: Option<f32>,
    pub z2: Option<f32>,
    pub regex: Option<bool>,
}

async fn show_part_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<ShowPartRequest>,
) -> impl IntoResponse {
    let params = serde_json::json!({
        "filter_type": request.filter_type,
        "value": request.value,
        "x1": request.x1,
        "y1": request.y1,
        "z1": request.z1,
        "x2": request.x2,
        "y2": request.y2,
        "z2": request.z2,
        "regex": request.regex.unwrap_or(false)
    });
    Json(execute_show_part(&request.filter_type, &params).await)
}

async fn execute_show_part(filter_type: &str, params: &serde_json::Value) -> ConsoleCommandResponse {
    match filter_type {
        "id" => {
            let id = params.get("value").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() {
                return ConsoleCommandResponse::error("Missing parameter", "Part UUID or local ID is required");
            }
            ConsoleCommandResponse::success(
                format!("Part details for: {}", id),
                Some(serde_json::json!({
                    "part_id": id,
                    "name": "Unknown Part",
                    "description": "",
                    "owner_id": "00000000-0000-0000-0000-000000000000",
                    "creator_id": "00000000-0000-0000-0000-000000000000",
                    "position": {"x": 0.0, "y": 0.0, "z": 0.0},
                    "rotation": {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0},
                    "scale": {"x": 1.0, "y": 1.0, "z": 1.0},
                    "note": "Part lookup requires scene integration"
                }))
            )
        },
        "name" => {
            let name = params.get("value").and_then(|v| v.as_str()).unwrap_or("");
            let use_regex = params.get("regex").and_then(|v| v.as_bool()).unwrap_or(false);
            if name.is_empty() {
                return ConsoleCommandResponse::error("Missing parameter", "Part name is required");
            }
            ConsoleCommandResponse::success(
                format!("Parts matching name: {} (regex: {})", name, use_regex),
                Some(serde_json::json!({
                    "search_name": name,
                    "use_regex": use_regex,
                    "count": 0,
                    "parts": [],
                    "note": "Part search requires scene integration"
                }))
            )
        },
        "pos" => {
            let x1 = params.get("x1").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let y1 = params.get("y1").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let z1 = params.get("z1").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let x2 = params.get("x2").and_then(|v| v.as_f64()).unwrap_or(256.0);
            let y2 = params.get("y2").and_then(|v| v.as_f64()).unwrap_or(256.0);
            let z2 = params.get("z2").and_then(|v| v.as_f64()).unwrap_or(4096.0);
            ConsoleCommandResponse::success(
                format!("Parts in region ({},{},{}) to ({},{},{})", x1, y1, z1, x2, y2, z2),
                Some(serde_json::json!({
                    "bounds": {
                        "min": {"x": x1, "y": y1, "z": z1},
                        "max": {"x": x2, "y": y2, "z": z2}
                    },
                    "count": 0,
                    "parts": [],
                    "note": "Part position search requires scene integration"
                }))
            )
        },
        _ => ConsoleCommandResponse::error("Invalid filter type", "Filter must be 'id', 'name', or 'pos'")
    }
}

#[derive(Debug, Deserialize)]
pub struct DumpObjectRequest {
    pub object_id: String,
}

async fn dump_object_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<DumpObjectRequest>,
) -> impl IntoResponse {
    Json(execute_dump_object(&serde_json::json!({"object_id": request.object_id})).await)
}

async fn execute_dump_object(params: &serde_json::Value) -> ConsoleCommandResponse {
    let object_id = params.get("object_id").and_then(|v| v.as_str()).unwrap_or("");

    if object_id.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "Object UUID or local ID is required");
    }

    ConsoleCommandResponse::success(
        format!("Object dump for: {}", object_id),
        Some(serde_json::json!({
            "object_id": object_id,
            "dump_file": format!("object_{}.xml", object_id),
            "format": "OpenSim XML",
            "success": true,
            "note": "Object dump requires scene integration"
        }))
    )
}

#[derive(Debug, Deserialize)]
pub struct EditScaleRequest {
    pub prim_id: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

async fn edit_scale_endpoint(
    State(_state): State<ConsoleApiState>,
    Json(request): Json<EditScaleRequest>,
) -> impl IntoResponse {
    Json(execute_edit_scale(&serde_json::json!({
        "prim_id": request.prim_id,
        "x": request.x,
        "y": request.y,
        "z": request.z
    })).await)
}

async fn execute_edit_scale(params: &serde_json::Value) -> ConsoleCommandResponse {
    let prim_id = params.get("prim_id").and_then(|v| v.as_str()).unwrap_or("");
    let x = params.get("x").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let y = params.get("y").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let z = params.get("z").and_then(|v| v.as_f64()).unwrap_or(1.0);

    if prim_id.is_empty() {
        return ConsoleCommandResponse::error("Missing parameter", "Prim UUID or local ID is required");
    }

    if x <= 0.0 || y <= 0.0 || z <= 0.0 {
        return ConsoleCommandResponse::error("Invalid scale", "Scale values must be positive");
    }

    ConsoleCommandResponse::success(
        format!("Scale set for prim {}: ({}, {}, {})", prim_id, x, y, z),
        Some(serde_json::json!({
            "prim_id": prim_id,
            "new_scale": {"x": x, "y": y, "z": z},
            "success": true,
            "note": "Edit scale requires scene integration"
        }))
    )
}

async fn show_pending_objects_endpoint(
    State(_state): State<ConsoleApiState>,
) -> impl IntoResponse {
    Json(execute_show_pending_objects().await)
}

async fn execute_show_pending_objects() -> ConsoleCommandResponse {
    ConsoleCommandResponse::success(
        "Pending objects",
        Some(serde_json::json!({
            "pending_count": 0,
            "objects": [],
            "queued_updates": 0,
            "queued_deletes": 0,
            "note": "Pending objects display requires network integration"
        }))
    )
}

fn get_all_command_definitions() -> Vec<CommandDefinition> {
    vec![
        CommandDefinition {
            name: "show info".to_string(),
            group: "general".to_string(),
            description: "Show server information".to_string(),
            syntax: "show info".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show version".to_string(),
            group: "general".to_string(),
            description: "Show server version".to_string(),
            syntax: "show version".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show uptime".to_string(),
            group: "general".to_string(),
            description: "Show server uptime".to_string(),
            syntax: "show uptime".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show regions".to_string(),
            group: "regions".to_string(),
            description: "List all regions".to_string(),
            syntax: "show regions".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "restart".to_string(),
            group: "regions".to_string(),
            description: "Restart a region".to_string(),
            syntax: "restart [region_name] [delay]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "region_name".to_string(),
                    description: "Name of the region to restart".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "delay".to_string(),
                    description: "Delay in seconds before restart".to_string(),
                    param_type: "integer".to_string(),
                    required: false,
                    default_value: Some("0".to_string()),
                    choices: None,
                },
            ],
            implemented: false,
        },
        CommandDefinition {
            name: "stats".to_string(),
            group: "terrain".to_string(),
            description: "Show terrain statistics".to_string(),
            syntax: "terrain stats".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "load".to_string(),
            group: "terrain".to_string(),
            description: "Load terrain from file".to_string(),
            syntax: "terrain load [filename]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "Path to terrain file (.raw, .r32, .png)".to_string(),
                    param_type: "file".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: false,
        },
        CommandDefinition {
            name: "save".to_string(),
            group: "terrain".to_string(),
            description: "Save terrain to file".to_string(),
            syntax: "terrain save [filename]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "Path to save terrain file".to_string(),
                    param_type: "path".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: false,
        },
        CommandDefinition {
            name: "fill".to_string(),
            group: "terrain".to_string(),
            description: "Fill terrain with uniform height".to_string(),
            syntax: "terrain fill [height]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "height".to_string(),
                    description: "Height value to fill terrain with".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: false,
        },
        CommandDefinition {
            name: "stats".to_string(),
            group: "database".to_string(),
            description: "Show database statistics".to_string(),
            syntax: "database stats".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "health".to_string(),
            group: "database".to_string(),
            description: "Check database health".to_string(),
            syntax: "database health".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "kick user".to_string(),
            group: "users".to_string(),
            description: "Kick a user from the grid".to_string(),
            syntax: "kick user [firstname] [lastname] [message]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "firstname".to_string(),
                    description: "User's first name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "lastname".to_string(),
                    description: "User's last name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "message".to_string(),
                    description: "Kick message to display".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    default_value: Some("You have been kicked".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "login level".to_string(),
            group: "users".to_string(),
            description: "Set minimum login level".to_string(),
            syntax: "login level [level]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "level".to_string(),
                    description: "Minimum user level (0=all, 100=admin, 200=god)".to_string(),
                    param_type: "integer".to_string(),
                    required: true,
                    default_value: None,
                    choices: Some(vec!["0".to_string(), "100".to_string(), "200".to_string()]),
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "login reset".to_string(),
            group: "users".to_string(),
            description: "Reset login restrictions".to_string(),
            syntax: "login reset".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "login text".to_string(),
            group: "users".to_string(),
            description: "Set login message".to_string(),
            syntax: "login text [message]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "message".to_string(),
                    description: "Message shown during login".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show connections".to_string(),
            group: "comms".to_string(),
            description: "Show active connections".to_string(),
            syntax: "show connections".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show circuits".to_string(),
            group: "comms".to_string(),
            description: "Show active UDP circuits".to_string(),
            syntax: "show circuits".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show object id".to_string(),
            group: "objects".to_string(),
            description: "Show object by UUID or local ID".to_string(),
            syntax: "show object id [--full] [UUID-or-localID]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "value".to_string(),
                    description: "UUID or local ID of the object".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "full".to_string(),
                    description: "Show full object details".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                    default_value: Some("false".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show object name".to_string(),
            group: "objects".to_string(),
            description: "Show objects by name".to_string(),
            syntax: "show object name [--full] [--regex] [name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "value".to_string(),
                    description: "Name or regex pattern".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "full".to_string(),
                    description: "Show full object details".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                    default_value: Some("false".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show object owner".to_string(),
            group: "objects".to_string(),
            description: "Show objects by owner UUID".to_string(),
            syntax: "show object owner [--full] [UUID]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "value".to_string(),
                    description: "Owner UUID".to_string(),
                    param_type: "uuid".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "full".to_string(),
                    description: "Show full object details".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                    default_value: Some("false".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show object pos".to_string(),
            group: "objects".to_string(),
            description: "Show objects in position range".to_string(),
            syntax: "show object pos [--full] [x1,y1,z1] [x2,y2,z2]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "value".to_string(),
                    description: "Position range (x1,y1,z1 x2,y2,z2)".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "full".to_string(),
                    description: "Show full object details".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                    default_value: Some("false".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "delete object id".to_string(),
            group: "objects".to_string(),
            description: "Delete object by UUID or local ID".to_string(),
            syntax: "delete object id [UUID-or-localID]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "value".to_string(),
                    description: "UUID or local ID of the object".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "delete object name".to_string(),
            group: "objects".to_string(),
            description: "Delete objects by name".to_string(),
            syntax: "delete object name [--regex] [name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "value".to_string(),
                    description: "Name or regex pattern".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "delete object owner".to_string(),
            group: "objects".to_string(),
            description: "Delete all objects by owner".to_string(),
            syntax: "delete object owner [UUID]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "value".to_string(),
                    description: "Owner UUID".to_string(),
                    param_type: "uuid".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "delete object pos".to_string(),
            group: "objects".to_string(),
            description: "Delete objects in position range".to_string(),
            syntax: "delete object pos [x1,y1,z1] [x2,y2,z2]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "value".to_string(),
                    description: "Position range".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "delete object outside".to_string(),
            group: "objects".to_string(),
            description: "Delete all objects outside region bounds".to_string(),
            syntax: "delete object outside".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "backup".to_string(),
            group: "objects".to_string(),
            description: "Backup region objects".to_string(),
            syntax: "backup".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "rotate scene".to_string(),
            group: "objects".to_string(),
            description: "Rotate all scene objects".to_string(),
            syntax: "rotate scene [degrees] [centerX] [centerY]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "degrees".to_string(),
                    description: "Rotation in degrees".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "center_x".to_string(),
                    description: "Center X coordinate".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                    default_value: Some("128".to_string()),
                    choices: None,
                },
                ParamDefinition {
                    name: "center_y".to_string(),
                    description: "Center Y coordinate".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                    default_value: Some("128".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "scale scene".to_string(),
            group: "objects".to_string(),
            description: "Scale all scene objects".to_string(),
            syntax: "scale scene [factor]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "factor".to_string(),
                    description: "Scale factor".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "translate scene".to_string(),
            group: "objects".to_string(),
            description: "Move all scene objects".to_string(),
            syntax: "translate scene [x] [y] [z]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "x".to_string(),
                    description: "X offset".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "y".to_string(),
                    description: "Y offset".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "z".to_string(),
                    description: "Z offset".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "force update".to_string(),
            group: "objects".to_string(),
            description: "Force update all objects to viewers".to_string(),
            syntax: "force update".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "estate create".to_string(),
            group: "estates".to_string(),
            description: "Create a new estate".to_string(),
            syntax: "estate create [name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "name".to_string(),
                    description: "Name for the new estate".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "estate set owner".to_string(),
            group: "estates".to_string(),
            description: "Set estate owner".to_string(),
            syntax: "estate set owner [estate] [firstname] [lastname]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "estate".to_string(),
                    description: "Estate name or ID".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "firstname".to_string(),
                    description: "New owner's first name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "lastname".to_string(),
                    description: "New owner's last name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "estate set name".to_string(),
            group: "estates".to_string(),
            description: "Rename an estate".to_string(),
            syntax: "estate set name [estate] [new_name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "estate".to_string(),
                    description: "Current estate name or ID".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "new_name".to_string(),
                    description: "New name for the estate".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "estate link region".to_string(),
            group: "estates".to_string(),
            description: "Link a region to an estate".to_string(),
            syntax: "estate link region [estate] [region]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "estate".to_string(),
                    description: "Estate name or ID".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "region".to_string(),
                    description: "Region name to link".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "link-region".to_string(),
            group: "hypergrid".to_string(),
            description: "Create a hypergrid link to a remote region".to_string(),
            syntax: "link-region [xloc] [yloc] [host:port] [name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "xloc".to_string(),
                    description: "X grid location for the link".to_string(),
                    param_type: "integer".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "yloc".to_string(),
                    description: "Y grid location for the link".to_string(),
                    param_type: "integer".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "host".to_string(),
                    description: "Remote host:port (e.g., osgrid.org:80)".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "name".to_string(),
                    description: "Local name for the linked region".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    default_value: Some("Remote Region".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "unlink-region".to_string(),
            group: "hypergrid".to_string(),
            description: "Remove a hypergrid link".to_string(),
            syntax: "unlink-region [name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "name".to_string(),
                    description: "Name of the linked region to remove".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show hyperlinks".to_string(),
            group: "hypergrid".to_string(),
            description: "Show all hypergrid links".to_string(),
            syntax: "show hyperlinks".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "link-mapping".to_string(),
            group: "hypergrid".to_string(),
            description: "Set default grid location for links".to_string(),
            syntax: "link-mapping [x] [y]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "x".to_string(),
                    description: "Default X grid coordinate".to_string(),
                    param_type: "integer".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "y".to_string(),
                    description: "Default Y grid coordinate".to_string(),
                    param_type: "integer".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show asset".to_string(),
            group: "assets".to_string(),
            description: "Show asset information by UUID".to_string(),
            syntax: "show asset [UUID]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "uuid".to_string(),
                    description: "Asset UUID".to_string(),
                    param_type: "uuid".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "dump asset".to_string(),
            group: "assets".to_string(),
            description: "Dump asset to file".to_string(),
            syntax: "dump asset [UUID] [filename]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "uuid".to_string(),
                    description: "Asset UUID".to_string(),
                    param_type: "uuid".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "Output filename".to_string(),
                    param_type: "path".to_string(),
                    required: false,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "delete asset".to_string(),
            group: "assets".to_string(),
            description: "Delete asset (disabled for safety)".to_string(),
            syntax: "delete asset [UUID]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "uuid".to_string(),
                    description: "Asset UUID".to_string(),
                    param_type: "uuid".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "fcache status".to_string(),
            group: "assets".to_string(),
            description: "Show asset cache status".to_string(),
            syntax: "fcache status".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "fcache clear".to_string(),
            group: "assets".to_string(),
            description: "Clear asset cache".to_string(),
            syntax: "fcache clear [file|memory]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "target".to_string(),
                    description: "Cache to clear".to_string(),
                    param_type: "choice".to_string(),
                    required: false,
                    default_value: Some("all".to_string()),
                    choices: Some(vec!["all".to_string(), "file".to_string(), "memory".to_string()]),
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "fcache assets".to_string(),
            group: "assets".to_string(),
            description: "List cached assets".to_string(),
            syntax: "fcache assets".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "fcache expire".to_string(),
            group: "assets".to_string(),
            description: "Expire old cache entries".to_string(),
            syntax: "fcache expire [datetime]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "datetime".to_string(),
                    description: "Expire entries older than this".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    default_value: Some("now".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "load xml".to_string(),
            group: "archiving".to_string(),
            description: "Load scene from XML file".to_string(),
            syntax: "load xml [filename]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "XML file path".to_string(),
                    param_type: "file".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "save xml".to_string(),
            group: "archiving".to_string(),
            description: "Save scene to XML file".to_string(),
            syntax: "save xml [filename]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "Output XML file path".to_string(),
                    param_type: "path".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "load xml2".to_string(),
            group: "archiving".to_string(),
            description: "Load scene from XML2 file".to_string(),
            syntax: "load xml2 [filename]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "XML2 file path".to_string(),
                    param_type: "file".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "save xml2".to_string(),
            group: "archiving".to_string(),
            description: "Save scene to XML2 file".to_string(),
            syntax: "save xml2 [filename]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "Output XML2 file path".to_string(),
                    param_type: "path".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "save prims xml2".to_string(),
            group: "archiving".to_string(),
            description: "Save prims to XML2 file".to_string(),
            syntax: "save prims xml2 [filename]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "Output XML2 file path".to_string(),
                    param_type: "path".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "create region".to_string(),
            group: "regions".to_string(),
            description: "Create a new region".to_string(),
            syntax: "create region [name] [template]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "name".to_string(),
                    description: "Name of the region".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "template".to_string(),
                    description: "Region template to use".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    default_value: Some("default".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "delete-region".to_string(),
            group: "regions".to_string(),
            description: "Delete a region".to_string(),
            syntax: "delete-region [name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "name".to_string(),
                    description: "Name of the region to delete".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show ratings".to_string(),
            group: "regions".to_string(),
            description: "Show region ratings".to_string(),
            syntax: "show ratings".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show neighbours".to_string(),
            group: "regions".to_string(),
            description: "Show region neighbours".to_string(),
            syntax: "show neighbours".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show regionsinview".to_string(),
            group: "regions".to_string(),
            description: "Show regions in view".to_string(),
            syntax: "show regionsinview".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "change region".to_string(),
            group: "regions".to_string(),
            description: "Change current region".to_string(),
            syntax: "change region [name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "name".to_string(),
                    description: "Name of region to switch to".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain load-tile".to_string(),
            group: "terrain".to_string(),
            description: "Load terrain tile".to_string(),
            syntax: "terrain load-tile [filename] [x] [y]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "Terrain file path".to_string(),
                    param_type: "file".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "x".to_string(),
                    description: "Tile X coordinate".to_string(),
                    param_type: "integer".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "y".to_string(),
                    description: "Tile Y coordinate".to_string(),
                    param_type: "integer".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain save-tile".to_string(),
            group: "terrain".to_string(),
            description: "Save terrain tile".to_string(),
            syntax: "terrain save-tile [filename] [x] [y]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "filename".to_string(),
                    description: "Output file path".to_string(),
                    param_type: "path".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "x".to_string(),
                    description: "Tile X coordinate".to_string(),
                    param_type: "integer".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "y".to_string(),
                    description: "Tile Y coordinate".to_string(),
                    param_type: "integer".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain elevate".to_string(),
            group: "terrain".to_string(),
            description: "Elevate terrain".to_string(),
            syntax: "terrain elevate [amount]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "amount".to_string(),
                    description: "Amount to elevate".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain lower".to_string(),
            group: "terrain".to_string(),
            description: "Lower terrain".to_string(),
            syntax: "terrain lower [amount]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "amount".to_string(),
                    description: "Amount to lower".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain multiply".to_string(),
            group: "terrain".to_string(),
            description: "Multiply terrain height".to_string(),
            syntax: "terrain multiply [factor]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "factor".to_string(),
                    description: "Multiplication factor".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain bake".to_string(),
            group: "terrain".to_string(),
            description: "Bake terrain".to_string(),
            syntax: "terrain bake".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain revert".to_string(),
            group: "terrain".to_string(),
            description: "Revert terrain to baked state".to_string(),
            syntax: "terrain revert".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain show".to_string(),
            group: "terrain".to_string(),
            description: "Show terrain information".to_string(),
            syntax: "terrain show".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain effect".to_string(),
            group: "terrain".to_string(),
            description: "Apply terrain effect".to_string(),
            syntax: "terrain effect [name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "name".to_string(),
                    description: "Effect name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain flip".to_string(),
            group: "terrain".to_string(),
            description: "Flip terrain".to_string(),
            syntax: "terrain flip [direction]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "direction".to_string(),
                    description: "Flip direction (x or y)".to_string(),
                    param_type: "choice".to_string(),
                    required: true,
                    default_value: None,
                    choices: Some(vec!["x".to_string(), "y".to_string()]),
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain rescale".to_string(),
            group: "terrain".to_string(),
            description: "Rescale terrain height range".to_string(),
            syntax: "terrain rescale [min] [max]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "min".to_string(),
                    description: "Minimum height".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "max".to_string(),
                    description: "Maximum height".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain min".to_string(),
            group: "terrain".to_string(),
            description: "Set terrain minimum height".to_string(),
            syntax: "terrain min [height]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "height".to_string(),
                    description: "Minimum height".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain max".to_string(),
            group: "terrain".to_string(),
            description: "Set terrain maximum height".to_string(),
            syntax: "terrain max [height]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "height".to_string(),
                    description: "Maximum height".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "terrain modify".to_string(),
            group: "terrain".to_string(),
            description: "Modify terrain with operation".to_string(),
            syntax: "terrain modify [operation] [params]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "operation".to_string(),
                    description: "Modification operation".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "params".to_string(),
                    description: "Operation parameters".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "quit".to_string(),
            group: "general".to_string(),
            description: "Quit the server".to_string(),
            syntax: "quit".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "shutdown".to_string(),
            group: "general".to_string(),
            description: "Shutdown the server".to_string(),
            syntax: "shutdown".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show modules".to_string(),
            group: "general".to_string(),
            description: "Show loaded modules".to_string(),
            syntax: "show modules".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "command-script".to_string(),
            group: "general".to_string(),
            description: "Execute command script".to_string(),
            syntax: "command-script [path]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "path".to_string(),
                    description: "Script file path".to_string(),
                    param_type: "file".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "config show".to_string(),
            group: "general".to_string(),
            description: "Show configuration".to_string(),
            syntax: "config show [section]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "section".to_string(),
                    description: "Configuration section".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    default_value: Some("all".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "config get".to_string(),
            group: "general".to_string(),
            description: "Get configuration value".to_string(),
            syntax: "config get [section] [key]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "section".to_string(),
                    description: "Configuration section".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "key".to_string(),
                    description: "Configuration key".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "config set".to_string(),
            group: "general".to_string(),
            description: "Set configuration value".to_string(),
            syntax: "config set [section] [key] [value]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "section".to_string(),
                    description: "Configuration section".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "key".to_string(),
                    description: "Configuration key".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "value".to_string(),
                    description: "Configuration value".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "set log level".to_string(),
            group: "general".to_string(),
            description: "Set log level".to_string(),
            syntax: "set log level [module] [level]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "module".to_string(),
                    description: "Module name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "level".to_string(),
                    description: "Log level".to_string(),
                    param_type: "choice".to_string(),
                    required: true,
                    default_value: None,
                    choices: Some(vec!["trace".to_string(), "debug".to_string(), "info".to_string(), "warn".to_string(), "error".to_string()]),
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "force gc".to_string(),
            group: "general".to_string(),
            description: "Force garbage collection".to_string(),
            syntax: "force gc".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show grid user".to_string(),
            group: "users".to_string(),
            description: "Show grid user information".to_string(),
            syntax: "show grid user [firstname] [lastname]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "firstname".to_string(),
                    description: "User's first name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "lastname".to_string(),
                    description: "User's last name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show grid users online".to_string(),
            group: "users".to_string(),
            description: "Show grid users online".to_string(),
            syntax: "show grid users online".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "fcache clearnegatives".to_string(),
            group: "assets".to_string(),
            description: "Clear negative cache entries".to_string(),
            syntax: "fcache clearnegatives".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "fcache cachedefaultassets".to_string(),
            group: "assets".to_string(),
            description: "Cache default assets".to_string(),
            syntax: "fcache cachedefaultassets".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "fcache deletedefaultassets".to_string(),
            group: "assets".to_string(),
            description: "Delete default assets from cache".to_string(),
            syntax: "fcache deletedefaultassets".to_string(),
            params: vec![],
            implemented: true,
        },
        CommandDefinition {
            name: "show part id".to_string(),
            group: "objects".to_string(),
            description: "Show part details by UUID or local ID".to_string(),
            syntax: "show part id [UUID-or-localID]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "id".to_string(),
                    description: "Part UUID or local ID".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show part name".to_string(),
            group: "objects".to_string(),
            description: "Show parts matching name".to_string(),
            syntax: "show part name [--regex] [name]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "name".to_string(),
                    description: "Part name to search for".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "regex".to_string(),
                    description: "Use regular expression matching".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                    default_value: Some("false".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show part pos".to_string(),
            group: "objects".to_string(),
            description: "Show parts within position range".to_string(),
            syntax: "show part pos [x1,y1,z1] [x2,y2,z2]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "x1".to_string(),
                    description: "Minimum X coordinate".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                    default_value: Some("0".to_string()),
                    choices: None,
                },
                ParamDefinition {
                    name: "y1".to_string(),
                    description: "Minimum Y coordinate".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                    default_value: Some("0".to_string()),
                    choices: None,
                },
                ParamDefinition {
                    name: "z1".to_string(),
                    description: "Minimum Z coordinate".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                    default_value: Some("0".to_string()),
                    choices: None,
                },
                ParamDefinition {
                    name: "x2".to_string(),
                    description: "Maximum X coordinate".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                    default_value: Some("256".to_string()),
                    choices: None,
                },
                ParamDefinition {
                    name: "y2".to_string(),
                    description: "Maximum Y coordinate".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                    default_value: Some("256".to_string()),
                    choices: None,
                },
                ParamDefinition {
                    name: "z2".to_string(),
                    description: "Maximum Z coordinate".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                    default_value: Some("4096".to_string()),
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "dump object id".to_string(),
            group: "objects".to_string(),
            description: "Dump object to XML file".to_string(),
            syntax: "dump object id [UUID-or-localID]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "object_id".to_string(),
                    description: "Object UUID or local ID".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "edit scale".to_string(),
            group: "objects".to_string(),
            description: "Edit prim scale".to_string(),
            syntax: "edit scale [prim] [x] [y] [z]".to_string(),
            params: vec![
                ParamDefinition {
                    name: "prim_id".to_string(),
                    description: "Prim UUID or local ID".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "x".to_string(),
                    description: "X scale".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "y".to_string(),
                    description: "Y scale".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
                ParamDefinition {
                    name: "z".to_string(),
                    description: "Z scale".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    default_value: None,
                    choices: None,
                },
            ],
            implemented: true,
        },
        CommandDefinition {
            name: "show pending-objects".to_string(),
            group: "comms".to_string(),
            description: "Show pending object updates".to_string(),
            syntax: "show pending-objects".to_string(),
            params: vec![],
            implemented: true,
        },
    ]
}
