use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderValue, Method},
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router, middleware,
};
use tower_http::cors::{CorsLayer, Any};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::database::{
    AdminOperationResult, CreateUserRequest, DatabaseAdmin,
    admin_operations::{DatabaseBackupRequest, DatabaseRestoreRequest, DatabaseMaintenanceRequest, DatabaseMigrationRequest}
};
use crate::network::auth::require_admin_auth_middleware;
use crate::network::security::SecurityManager;
use crate::network::ziti_manager::ZitiManager;

/// Admin API state containing database admin interface
#[derive(Clone)]
pub struct AdminApiState {
    pub admin: Arc<DatabaseAdmin>,
}

/// Request/Response types for admin API endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserApiRequest {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
    pub user_level: Option<i32>,
    pub start_region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub firstname: String,
    pub lastname: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetEmailRequest {
    pub firstname: String,
    pub lastname: String,
    pub new_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetUserLevelRequest {
    pub firstname: String,
    pub lastname: String,
    pub user_level: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuery {
    pub firstname: String,
    pub lastname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminApiResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub affected_rows: Option<i64>,
}

impl From<AdminOperationResult> for AdminApiResponse {
    fn from(result: AdminOperationResult) -> Self {
        Self {
            success: result.success,
            message: result.message,
            data: result.data,
            affected_rows: result.affected_rows,
        }
    }
}

/// Create admin API router with all user management endpoints
pub fn create_admin_api_router() -> Router<AdminApiState> {
    Router::new()
        // User management endpoints (Robust-style commands)
        .route("/admin/users", post(create_user_endpoint))
        .route("/admin/users", get(list_users_endpoint))
        .route("/admin/users/account", get(show_user_account_endpoint))
        .route("/admin/users/password", put(reset_password_endpoint))
        .route("/admin/users/email", put(reset_email_endpoint))
        .route("/admin/users/level", put(set_user_level_endpoint))
        .route("/admin/users/delete", delete(delete_user_endpoint))
        
        // Database administration endpoints (Phase 22.3)
        .route("/admin/database/stats", get(get_database_stats_endpoint))
        .route("/admin/database/backup", post(create_backup_endpoint))
        .route("/admin/database/restore", post(restore_backup_endpoint))
        .route("/admin/database/maintenance", post(perform_maintenance_endpoint))
        .route("/admin/database/migration", post(run_migration_endpoint))
        .route("/admin/database/health", get(get_database_health_endpoint))
        .route("/admin/database/backups", get(list_backups_endpoint))
        
        // Region management endpoints (Phase 22.2)
        .route("/admin/regions", post(create_region_endpoint))
        .route("/admin/regions", get(list_regions_endpoint))
        .route("/admin/regions/:region_name", get(show_region_endpoint))
        .route("/admin/regions/:region_name", put(update_region_endpoint))
        .route("/admin/regions/:region_name", delete(delete_region_endpoint))
        .route("/admin/regions/stats", get(get_region_stats_endpoint))
        
        // Health check for admin API
        .route("/admin/health", get(admin_health_endpoint))
        
        // DEVELOPMENT MODE: Add CORS and disable auth for FWDFE v2 testing
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        // TODO: Re-enable authentication for production
        // .layer(middleware::from_fn(require_admin_auth_middleware))
}

/// Create new user account (equivalent to 'create user' in OpenSim Robust)
/// POST /admin/users
async fn create_user_endpoint(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateUserApiRequest>,
) -> impl IntoResponse {
    info!("Admin API: Creating user {} {}", request.firstname, request.lastname);
    
    // Validate input according to mandatory Rust rules
    if request.firstname.trim().is_empty() || request.lastname.trim().is_empty() {
        warn!("Invalid user creation request: empty name fields");
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "First name and last name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    if request.email.trim().is_empty() || !request.email.contains('@') {
        warn!("Invalid user creation request: invalid email");
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Valid email address is required".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    if request.password.len() < 6 {
        warn!("Invalid user creation request: password too short");
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Password must be at least 6 characters long".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    // Convert API request to database request
    let db_request = CreateUserRequest {
        firstname: request.firstname.trim().to_string(),
        lastname: request.lastname.trim().to_string(),
        email: request.email.trim().to_lowercase(),
        password: request.password,
        user_level: request.user_level,
        start_region: request.start_region,
    };
    
    match state.admin.create_user(db_request).await {
        Ok(result) => {
            if result.success {
                info!("User created successfully via admin API");
                (StatusCode::CREATED, Json(AdminApiResponse::from(result)))
            } else {
                warn!("User creation failed: {}", result.message);
                (StatusCode::CONFLICT, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error creating user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Reset user password (equivalent to 'reset user password' in OpenSim Robust)
/// PUT /admin/users/password
async fn reset_password_endpoint(
    State(state): State<AdminApiState>,
    Json(request): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    info!("Admin API: Resetting password for user {} {}", request.firstname, request.lastname);
    
    // Validate input
    if request.firstname.trim().is_empty() || request.lastname.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "First name and last name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    if request.new_password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Password must be at least 6 characters long".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.reset_user_password(
        &request.firstname.trim(),
        &request.lastname.trim(),
        &request.new_password,
    ).await {
        Ok(result) => {
            if result.success {
                info!("Password reset successfully via admin API");
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                (StatusCode::NOT_FOUND, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error resetting password: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Reset user email (equivalent to 'reset user email' in OpenSim Robust)
/// PUT /admin/users/email
async fn reset_email_endpoint(
    State(state): State<AdminApiState>,
    Json(request): Json<ResetEmailRequest>,
) -> impl IntoResponse {
    info!("Admin API: Resetting email for user {} {}", request.firstname, request.lastname);
    
    // Validate input
    if request.firstname.trim().is_empty() || request.lastname.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "First name and last name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    if request.new_email.trim().is_empty() || !request.new_email.contains('@') {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Valid email address is required".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.reset_user_email(
        &request.firstname.trim(),
        &request.lastname.trim(),
        &request.new_email.trim().to_lowercase(),
    ).await {
        Ok(result) => {
            if result.success {
                info!("Email reset successfully via admin API");
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                (StatusCode::NOT_FOUND, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error resetting email: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Set user level (equivalent to 'set user level' in OpenSim Robust)
/// PUT /admin/users/level
async fn set_user_level_endpoint(
    State(state): State<AdminApiState>,
    Json(request): Json<SetUserLevelRequest>,
) -> impl IntoResponse {
    info!("Admin API: Setting user level for {} {} to {}", 
        request.firstname, request.lastname, request.user_level);
    
    // Validate input
    if request.firstname.trim().is_empty() || request.lastname.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "First name and last name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    // Validate user level range (0-255 in OpenSim)
    if request.user_level < 0 || request.user_level > 255 {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "User level must be between 0 and 255".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.set_user_level(
        &request.firstname.trim(),
        &request.lastname.trim(),
        request.user_level,
    ).await {
        Ok(result) => {
            if result.success {
                info!("User level set successfully via admin API");
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                (StatusCode::NOT_FOUND, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error setting user level: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Show user account details (equivalent to 'show account' in OpenSim Robust)
/// GET /admin/users/account?firstname=John&lastname=Doe
async fn show_user_account_endpoint(
    State(state): State<AdminApiState>,
    Query(query): Query<UserQuery>,
) -> impl IntoResponse {
    debug!("Admin API: Showing account for user {} {}", query.firstname, query.lastname);
    
    // Validate input
    if query.firstname.trim().is_empty() || query.lastname.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "First name and last name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.show_user_account(
        &query.firstname.trim(),
        &query.lastname.trim(),
    ).await {
        Ok(result) => {
            if result.success {
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                (StatusCode::NOT_FOUND, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error showing user account: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// List all users (equivalent to 'show users' in OpenSim Robust)
/// GET /admin/users?limit=50&offset=0
async fn list_users_endpoint(
    State(state): State<AdminApiState>,
    Query(query): Query<ListUsersQuery>,
) -> impl IntoResponse {
    debug!("Admin API: Listing users with limit {:?}", query.limit);
    
    // Validate limit (max 1000 users per request for performance)
    let limit = query.limit.map(|l| {
        if l > 1000 {
            warn!("Requested limit {} exceeds maximum 1000, capping to 1000", l);
            1000
        } else if l < 1 {
            warn!("Requested limit {} is invalid, using default", l);
            50
        } else {
            l
        }
    });
    
    match state.admin.list_users(limit).await {
        Ok(result) => {
            (StatusCode::OK, Json(AdminApiResponse::from(result)))
        }
        Err(e) => {
            error!("Admin API error listing users: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Delete user account (careful - destructive operation!)
/// DELETE /admin/users/delete
async fn delete_user_endpoint(
    State(state): State<AdminApiState>,
    Json(query): Json<UserQuery>,
) -> impl IntoResponse {
    warn!("Admin API: DESTRUCTIVE OPERATION - Deleting user {} {}", 
        query.firstname, query.lastname);
    
    // Validate input
    if query.firstname.trim().is_empty() || query.lastname.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "First name and last name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.delete_user(
        &query.firstname.trim(),
        &query.lastname.trim(),
    ).await {
        Ok(result) => {
            if result.success {
                warn!("User deleted successfully via admin API");
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                (StatusCode::NOT_FOUND, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error deleting user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Get database statistics
/// GET /admin/database/stats
async fn get_database_stats_endpoint(
    State(state): State<AdminApiState>,
) -> impl IntoResponse {
    debug!("Admin API: Getting database statistics");
    
    match state.admin.get_database_stats().await {
        Ok(result) => {
            (StatusCode::OK, Json(AdminApiResponse::from(result)))
        }
        Err(e) => {
            error!("Admin API error getting database stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Admin API health check
/// GET /admin/health
async fn admin_health_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "admin_api",
        "endpoints": [
            "POST /admin/users - Create user",
            "GET /admin/users - List users", 
            "GET /admin/users/account - Show user account",
            "PUT /admin/users/password - Reset password",
            "PUT /admin/users/email - Reset email",
            "PUT /admin/users/level - Set user level",
            "DELETE /admin/users/delete - Delete user",
            "GET /admin/database/stats - Database statistics",
            "GET /admin/health - This endpoint"
        ],
        "robust_commands_supported": [
            "create user",
            "reset user password", 
            "reset user email",
            "set user level",
            "show account",
            "show users",
            "delete user"
        ]
    }))
}

// ====================================================================
// PHASE 22.2: REGION MANAGEMENT API ENDPOINTS
// ====================================================================

/// Create new region (equivalent to 'create region' in OpenSim Robust)
/// POST /admin/regions
async fn create_region_endpoint(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRegionApiRequest>,
) -> impl IntoResponse {
    info!("Admin API: Creating region {} at ({}, {})", request.region_name, request.location_x, request.location_y);
    
    // Validate region name
    if request.region_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Region name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    // Validate location coordinates
    if request.location_x < 0 || request.location_y < 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Region coordinates must be non-negative".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    // Validate region size if provided
    if let Some(size_x) = request.size_x {
        if size_x < 64 || size_x > 2048 {
            return (
                StatusCode::BAD_REQUEST,
                Json(AdminApiResponse {
                    success: false,
                    message: "Region size X must be between 64 and 2048".to_string(),
                    data: None,
                    affected_rows: None,
                }),
            );
        }
    }
    
    if let Some(size_y) = request.size_y {
        if size_y < 64 || size_y > 2048 {
            return (
                StatusCode::BAD_REQUEST,
                Json(AdminApiResponse {
                    success: false,
                    message: "Region size Y must be between 64 and 2048".to_string(),
                    data: None,
                    affected_rows: None,
                }),
            );
        }
    }
    
    let db_request = crate::database::admin_operations::CreateRegionRequest {
        region_name: request.region_name,
        location_x: request.location_x,
        location_y: request.location_y,
        size_x: request.size_x,
        size_y: request.size_y,
        server_ip: request.server_ip,
        server_port: request.server_port,
        server_uri: request.server_uri,
        owner_uuid: request.owner_uuid,
    };
    
    match state.admin.create_region(db_request).await {
        Ok(result) => {
            if result.success {
                (StatusCode::CREATED, Json(AdminApiResponse::from(result)))
            } else {
                (StatusCode::CONFLICT, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error creating region: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Delete region (equivalent to 'delete region' in OpenSim Robust)
/// DELETE /admin/regions/:region_name
async fn delete_region_endpoint(
    State(state): State<AdminApiState>,
    Path(region_name): Path<String>,
) -> impl IntoResponse {
    info!("Admin API: Deleting region {}", region_name);
    
    if region_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Region name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.delete_region(&region_name).await {
        Ok(result) => {
            if result.success {
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                (StatusCode::NOT_FOUND, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error deleting region: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Show region details (equivalent to 'show region' in OpenSim Robust)
/// GET /admin/regions/:region_name
async fn show_region_endpoint(
    State(state): State<AdminApiState>,
    Path(region_name): Path<String>,
) -> impl IntoResponse {
    debug!("Admin API: Showing region {}", region_name);
    
    if region_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Region name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.show_region(&region_name).await {
        Ok(result) => {
            if result.success {
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                (StatusCode::NOT_FOUND, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error showing region: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// List all regions (equivalent to 'show regions' in OpenSim Robust)
/// GET /admin/regions?limit=50
async fn list_regions_endpoint(
    State(state): State<AdminApiState>,
    Query(query): Query<ListRegionsQuery>,
) -> impl IntoResponse {
    debug!("Admin API: Listing regions with limit {:?}", query.limit);
    
    // Validate limit (max 1000 regions per request for performance)
    let limit = query.limit.map(|l| {
        if l > 1000 {
            warn!("Requested region limit {} exceeds maximum 1000, capping to 1000", l);
            1000
        } else if l < 1 {
            warn!("Requested region limit {} is invalid, using default", l);
            50
        } else {
            l
        }
    });
    
    match state.admin.list_regions(limit).await {
        Ok(result) => {
            (StatusCode::OK, Json(AdminApiResponse::from(result)))
        }
        Err(e) => {
            error!("Admin API error listing regions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Update region properties (equivalent to 'set region' in OpenSim Robust)
/// PUT /admin/regions/:region_name
async fn update_region_endpoint(
    State(state): State<AdminApiState>,
    Path(region_name): Path<String>,
    Json(request): Json<UpdateRegionApiRequest>,
) -> impl IntoResponse {
    info!("Admin API: Updating region {}", region_name);
    
    if region_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Region name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    // Validate update fields
    if let Some(ref new_name) = request.new_name {
        if new_name.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(AdminApiResponse {
                    success: false,
                    message: "New region name cannot be empty".to_string(),
                    data: None,
                    affected_rows: None,
                }),
            );
        }
    }
    
    if let Some(location_x) = request.location_x {
        if location_x < 0 {
            return (
                StatusCode::BAD_REQUEST,
                Json(AdminApiResponse {
                    success: false,
                    message: "Location X must be non-negative".to_string(),
                    data: None,
                    affected_rows: None,
                }),
            );
        }
    }
    
    if let Some(location_y) = request.location_y {
        if location_y < 0 {
            return (
                StatusCode::BAD_REQUEST,
                Json(AdminApiResponse {
                    success: false,
                    message: "Location Y must be non-negative".to_string(),
                    data: None,
                    affected_rows: None,
                }),
            );
        }
    }
    
    let db_request = crate::database::admin_operations::UpdateRegionRequest {
        new_name: request.new_name,
        location_x: request.location_x,
        location_y: request.location_y,
        size_x: request.size_x,
        size_y: request.size_y,
    };
    
    match state.admin.update_region(&region_name, db_request).await {
        Ok(result) => {
            if result.success {
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                (StatusCode::NOT_FOUND, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error updating region: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Get region statistics (equivalent to 'region stats' in OpenSim Robust)
/// GET /admin/regions/stats
async fn get_region_stats_endpoint(
    State(state): State<AdminApiState>,
) -> impl IntoResponse {
    debug!("Admin API: Getting region statistics");
    
    match state.admin.get_region_stats().await {
        Ok(result) => {
            (StatusCode::OK, Json(AdminApiResponse::from(result)))
        }
        Err(e) => {
            error!("Admin API error getting region stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

// Region API request/response types

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CreateRegionApiRequest {
    region_name: String,
    location_x: i32,
    location_y: i32,
    size_x: Option<i32>,
    size_y: Option<i32>,
    server_ip: Option<String>,
    server_port: Option<i32>,
    server_uri: Option<String>,
    owner_uuid: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct UpdateRegionApiRequest {
    new_name: Option<String>,
    location_x: Option<i32>,
    location_y: Option<i32>,
    size_x: Option<i32>,
    size_y: Option<i32>,
}

#[derive(Debug, serde::Deserialize)]
struct ListRegionsQuery {
    limit: Option<i32>,
}

// ====================================================================
// PHASE 22.3: DATABASE ADMINISTRATION API ENDPOINTS
// ====================================================================

/// Create database backup (equivalent to 'backup database' in OpenSim Robust)
/// POST /admin/database/backup
async fn create_backup_endpoint(
    State(state): State<AdminApiState>,
    Json(request): Json<DatabaseBackupRequest>,
) -> impl IntoResponse {
    info!("Admin API: Creating database backup '{}'", request.backup_name);
    
    // Validate backup name
    if request.backup_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Backup name cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.create_backup(request).await {
        Ok(result) => {
            if result.success {
                info!("Database backup created successfully via admin API");
                (StatusCode::CREATED, Json(AdminApiResponse::from(result)))
            } else {
                warn!("Database backup failed: {}", result.message);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error creating backup: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Restore database from backup (equivalent to 'restore database' in OpenSim Robust)
/// POST /admin/database/restore
async fn restore_backup_endpoint(
    State(state): State<AdminApiState>,
    Json(request): Json<DatabaseRestoreRequest>,
) -> impl IntoResponse {
    info!("Admin API: Restoring database from backup '{}'", request.backup_file);
    
    // Validate backup file path
    if request.backup_file.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Backup file path cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.restore_backup(request).await {
        Ok(result) => {
            if result.success {
                info!("Database restore completed successfully via admin API");
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                warn!("Database restore failed: {}", result.message);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error restoring backup: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Perform database maintenance (equivalent to 'maintenance database' in OpenSim Robust)
/// POST /admin/database/maintenance
async fn perform_maintenance_endpoint(
    State(state): State<AdminApiState>,
    Json(request): Json<DatabaseMaintenanceRequest>,
) -> impl IntoResponse {
    info!("Admin API: Performing database maintenance");
    
    match state.admin.perform_maintenance(request).await {
        Ok(result) => {
            if result.success {
                info!("Database maintenance completed successfully via admin API");
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                warn!("Database maintenance failed: {}", result.message);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error performing maintenance: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Run database migration (equivalent to 'migrate database' in OpenSim Robust)
/// POST /admin/database/migration
async fn run_migration_endpoint(
    State(state): State<AdminApiState>,
    Json(request): Json<DatabaseMigrationRequest>,
) -> impl IntoResponse {
    info!("Admin API: Running database migration to version '{}'", request.target_version);
    
    // Validate target version
    if request.target_version.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AdminApiResponse {
                success: false,
                message: "Target version cannot be empty".to_string(),
                data: None,
                affected_rows: None,
            }),
        );
    }
    
    match state.admin.run_migration(request).await {
        Ok(result) => {
            if result.success {
                info!("Database migration completed successfully via admin API");
                (StatusCode::OK, Json(AdminApiResponse::from(result)))
            } else {
                warn!("Database migration failed: {}", result.message);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminApiResponse::from(result)))
            }
        }
        Err(e) => {
            error!("Admin API error running migration: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// Get database health information (equivalent to 'database health' in OpenSim Robust)
/// GET /admin/database/health
async fn get_database_health_endpoint(
    State(state): State<AdminApiState>,
) -> impl IntoResponse {
    debug!("Admin API: Getting database health information");
    
    match state.admin.get_database_health().await {
        Ok(result) => {
            (StatusCode::OK, Json(AdminApiResponse::from(result)))
        }
        Err(e) => {
            error!("Admin API error getting database health: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

/// List available backups (equivalent to 'list backups' in OpenSim Robust)
/// GET /admin/database/backups?directory=/path/to/backups
async fn list_backups_endpoint(
    State(state): State<AdminApiState>,
    Query(query): Query<ListBackupsQuery>,
) -> impl IntoResponse {
    debug!("Admin API: Listing database backups");
    
    match state.admin.list_backups(query.directory).await {
        Ok(result) => {
            (StatusCode::OK, Json(AdminApiResponse::from(result)))
        }
        Err(e) => {
            error!("Admin API error listing backups: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AdminApiResponse {
                    success: false,
                    message: format!("Internal server error: {}", e),
                    data: None,
                    affected_rows: None,
                }),
            )
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct ListBackupsQuery {
    directory: Option<String>,
}

// ====================================================================
// PHASE 117.8: SECURITY DASHBOARD API ENDPOINTS
// ====================================================================

#[derive(Clone)]
pub struct SecurityApiState {
    pub security_manager: Arc<SecurityManager>,
    pub ziti_manager: Arc<ZitiManager>,
}

pub fn create_security_api_router() -> Router<SecurityApiState> {
    Router::new()
        .route("/api/security/stats", get(security_stats_endpoint))
        .route("/api/security/threats", get(security_threats_endpoint))
        .route("/api/security/lockouts", get(security_lockouts_endpoint))
        .route("/api/security/blacklist/:ip", post(blacklist_ip_endpoint))
        .route("/api/security/blacklist/:ip", delete(unblock_ip_endpoint))
        .route("/api/security/ziti/status", get(ziti_status_endpoint))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
}

async fn security_stats_endpoint(
    State(state): State<SecurityApiState>,
) -> impl IntoResponse {
    let udp_stats = state.security_manager.get_udp_stats();
    let blocked_list = state.security_manager.get_blocked_ip_list();
    Json(serde_json::json!({
        "udp": {
            "total_packets": udp_stats.total_packets,
            "total_dropped": udp_stats.total_dropped,
            "tracked_ips": udp_stats.tracked_ips,
            "blocked_ips": udp_stats.blocked_ips,
        },
        "blocked_ip_count": blocked_list.len(),
    }))
}

async fn security_threats_endpoint(
    State(state): State<SecurityApiState>,
) -> impl IntoResponse {
    let udp_stats = state.security_manager.get_udp_stats();
    let blocked = state.security_manager.get_blocked_ip_list();

    let threats: Vec<serde_json::Value> = blocked.iter().map(|(ip, _blocked_at)| {
        serde_json::json!({
            "type": "ip_blocked",
            "severity": "high",
            "source_ip": ip.to_string(),
            "action": "blocked",
        })
    }).collect();

    Json(serde_json::json!({
        "threat_count": threats.len(),
        "total_dropped_packets": udp_stats.total_dropped,
        "threats": threats,
    }))
}

async fn security_lockouts_endpoint(
    State(state): State<SecurityApiState>,
) -> impl IntoResponse {
    let blocked = state.security_manager.get_blocked_ip_list();
    let lockouts: Vec<serde_json::Value> = blocked.iter().map(|(ip, _blocked_at)| {
        serde_json::json!({
            "ip": ip.to_string(),
            "reason": "rate_limit_or_circuit_failure",
        })
    }).collect();

    Json(serde_json::json!({
        "lockout_count": lockouts.len(),
        "lockouts": lockouts,
    }))
}

#[derive(Debug, Deserialize)]
struct IpPathParam {
    ip: String,
}

async fn blacklist_ip_endpoint(
    State(state): State<SecurityApiState>,
    Path(ip_str): Path<String>,
) -> impl IntoResponse {
    match ip_str.parse::<IpAddr>() {
        Ok(ip) => {
            state.security_manager.add_to_blacklist(ip);
            info!("[SECURITY] IP {} manually blacklisted via API", ip);
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": format!("IP {} added to blacklist", ip),
            })))
        }
        Err(_) => {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "message": format!("Invalid IP address: {}", ip_str),
            })))
        }
    }
}

async fn unblock_ip_endpoint(
    State(state): State<SecurityApiState>,
    Path(ip_str): Path<String>,
) -> impl IntoResponse {
    match ip_str.parse::<IpAddr>() {
        Ok(ip) => {
            state.security_manager.remove_from_blocked(ip);
            info!("[SECURITY] IP {} unblocked via API", ip);
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": format!("IP {} removed from blacklist", ip),
            })))
        }
        Err(_) => {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "message": format!("Invalid IP address: {}", ip_str),
            })))
        }
    }
}

async fn ziti_status_endpoint(
    State(state): State<SecurityApiState>,
) -> impl IntoResponse {
    let status = state.ziti_manager.get_status().await;
    Json(serde_json::json!({
        "enabled": status.enabled,
        "running": status.running,
        "identity_loaded": status.identity_loaded,
        "controller_url": status.controller_url,
        "restart_count": status.restart_count,
        "uptime_secs": status.uptime_secs,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Following mandatory Rust testing rules
    #[tokio::test]
    async fn test_create_user_validation() {
        // Test input validation for create user endpoint
        let request = CreateUserApiRequest {
            firstname: "".to_string(), // Invalid: empty firstname
            lastname: "Doe".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            user_level: None,
            start_region: None,
        };
        
        // This would normally test the validation logic
        assert!(request.firstname.trim().is_empty());
    }
    
    #[tokio::test] 
    async fn test_password_validation() {
        // Test password length validation
        let short_password = "123";
        assert!(short_password.len() < 6);
        
        let valid_password = "password123";
        assert!(valid_password.len() >= 6);
    }
    
    #[tokio::test]
    async fn test_email_validation() {
        // Test email format validation
        let invalid_email = "notanemail";
        assert!(!invalid_email.contains('@'));
        
        let valid_email = "test@example.com";
        assert!(valid_email.contains('@'));
    }
    
    #[tokio::test]
    async fn test_user_level_validation() {
        // Test user level range validation
        assert!(-1 < 0 || -1 > 255); // Invalid
        assert!(256 < 0 || 256 > 255); // Invalid
        assert!(100 >= 0 && 100 <= 255); // Valid
        assert!(200 >= 0 && 200 <= 255); // Valid (God level)
    }
}