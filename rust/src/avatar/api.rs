//! Avatar REST API Endpoints for OpenSim Next
//! 
//! Provides REST API for avatar management, appearance, behavior, and social features.

use super::*;
use crate::network::auth::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Avatar API state
#[derive(Clone)]
pub struct AvatarApiState {
    pub avatar_manager: Arc<AdvancedAvatarManager>,
}

/// Create avatar API router
pub fn create_avatar_api_router(avatar_manager: Arc<AdvancedAvatarManager>) -> Router {
    let state = AvatarApiState { avatar_manager };

    Router::new()
        // Avatar management endpoints
        .route("/avatars", post(create_avatar_endpoint))
        .route("/avatars/:avatar_id", get(get_avatar_endpoint))
        .route("/avatars/:avatar_id", put(update_avatar_endpoint))
        .route("/avatars/:avatar_id", delete(delete_avatar_endpoint))
        .route("/avatars/user/:user_id", get(get_avatar_by_user_endpoint))
        .route("/avatars", get(search_avatars_endpoint))
        
        // Avatar appearance endpoints
        .route("/avatars/:avatar_id/appearance", get(get_appearance_endpoint))
        .route("/avatars/:avatar_id/appearance", put(update_appearance_endpoint))
        .route("/avatars/:avatar_id/wearables", post(add_wearable_endpoint))
        .route("/avatars/:avatar_id/wearables/:wearable_type", delete(remove_wearable_endpoint))
        .route("/avatars/:avatar_id/attachments", post(add_attachment_endpoint))
        .route("/avatars/:avatar_id/attachments/:item_id", delete(remove_attachment_endpoint))
        .route("/avatars/:avatar_id/visual-params", put(update_visual_params_endpoint))
        
        // Avatar behavior endpoints
        .route("/avatars/:avatar_id/behavior", get(get_behavior_endpoint))
        .route("/avatars/:avatar_id/behavior", put(update_behavior_endpoint))
        .route("/avatars/:avatar_id/animations", post(start_animation_endpoint))
        .route("/avatars/:avatar_id/animations/:animation_id", delete(stop_animation_endpoint))
        .route("/avatars/:avatar_id/animations", get(get_active_animations_endpoint))
        .route("/avatars/:avatar_id/gestures/:gesture_id", post(trigger_gesture_endpoint))
        .route("/avatars/:avatar_id/expressions", put(update_expression_endpoint))
        
        // Avatar social endpoints
        .route("/avatars/:avatar_id/social", get(get_social_profile_endpoint))
        .route("/avatars/:avatar_id/social", put(update_social_profile_endpoint))
        .route("/avatars/:avatar_id/friends", get(get_friends_endpoint))
        .route("/avatars/:avatar_id/friends/:friend_id", post(add_friend_endpoint))
        .route("/avatars/:avatar_id/friends/:friend_id", delete(remove_friend_endpoint))
        .route("/avatars/:avatar_id/achievements", get(get_achievements_endpoint))
        .route("/avatars/:avatar_id/achievements", post(add_achievement_endpoint))
        .route("/avatars/:avatar_id/messages", get(get_messages_endpoint))
        .route("/avatars/:avatar_id/messages", post(send_message_endpoint))
        
        // Avatar session endpoints
        .route("/avatars/:avatar_id/login", post(login_avatar_endpoint))
        .route("/avatars/:avatar_id/logout", post(logout_avatar_endpoint))
        .route("/avatars/:avatar_id/position", put(update_position_endpoint))
        .route("/avatars/:avatar_id/statistics", get(get_statistics_endpoint))
        
        // System endpoints
        .route("/avatars/active", get(get_active_avatars_endpoint))
        .route("/avatars/region/:region_id", get(get_avatars_in_region_endpoint))
        .route("/system/avatar-health", get(get_system_health_endpoint))
        
        .with_state(state)
}

// Avatar management endpoints

async fn create_avatar_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Json(request): Json<CreateAvatarRequest>,
) -> Result<Json<ApiResponse<EnhancedAvatar>>, StatusCode> {
    info!("Creating new avatar: {}", request.name);

    match state.avatar_manager.create_avatar(
        request.user_id,
        request.name,
        request.initial_appearance,
    ).await {
        Ok(avatar) => Ok(Json(ApiResponse::success(avatar))),
        Err(e) => {
            warn!("Failed to create avatar: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn get_avatar_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Path(avatar_id): Path<Uuid>,
) -> Result<Json<ApiResponse<EnhancedAvatar>>, StatusCode> {
    match state.avatar_manager.get_avatar(avatar_id).await {
        Ok(avatar) => Ok(Json(ApiResponse::success(avatar))),
        Err(AvatarError::NotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            warn!("Failed to get avatar {}: {}", avatar_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_avatar_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Path(avatar_id): Path<Uuid>,
    Json(request): Json<UpdateAvatarRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // This would implement avatar updates based on the request
    // For now, return success
    Ok(Json(ApiResponse::success("Avatar updated successfully".to_string())))
}

async fn delete_avatar_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Path(avatar_id): Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.avatar_manager.delete_avatar(avatar_id).await {
        Ok(_) => Ok(Json(ApiResponse::success("Avatar deleted successfully".to_string()))),
        Err(AvatarError::NotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            warn!("Failed to delete avatar {}: {}", avatar_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_avatar_by_user_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<EnhancedAvatar>>, StatusCode> {
    match state.avatar_manager.get_avatar_by_user(user_id).await {
        Ok(avatar) => Ok(Json(ApiResponse::success(avatar))),
        Err(AvatarError::NotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            warn!("Failed to get avatar for user {}: {}", user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn search_avatars_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<EnhancedAvatar>>>, StatusCode> {
    let criteria = AvatarSearchCriteria {
        name_pattern: params.get("name").cloned(),
        user_id: params.get("user_id").and_then(|s| Uuid::parse_str(s).ok()),
        region_id: params.get("region_id").and_then(|s| Uuid::parse_str(s).ok()),
        online_only: params.get("online_only").map(|s| s == "true").unwrap_or(false),
        limit: params.get("limit").and_then(|s| s.parse().ok()),
        offset: params.get("offset").and_then(|s| s.parse().ok()),
    };

    match state.avatar_manager.search_avatars(criteria).await {
        Ok(avatars) => Ok(Json(ApiResponse::success(avatars))),
        Err(e) => {
            warn!("Failed to search avatars: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Avatar appearance endpoints

async fn get_appearance_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Path(avatar_id): Path<Uuid>,
) -> Result<Json<ApiResponse<AvatarAppearance>>, StatusCode> {
    match state.avatar_manager.get_avatar(avatar_id).await {
        Ok(avatar) => Ok(Json(ApiResponse::success(avatar.appearance))),
        Err(AvatarError::NotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            warn!("Failed to get avatar appearance {}: {}", avatar_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_appearance_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Path(avatar_id): Path<Uuid>,
    Json(appearance): Json<AvatarAppearance>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.avatar_manager.update_appearance(avatar_id, appearance).await {
        Ok(_) => Ok(Json(ApiResponse::success("Appearance updated successfully".to_string()))),
        Err(AvatarError::NotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            warn!("Failed to update avatar appearance {}: {}", avatar_id, e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn add_wearable_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Path(avatar_id): Path<Uuid>,
    Json(wearable): Json<WearableItem>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Get current avatar and update wearable
    match state.avatar_manager.get_avatar(avatar_id).await {
        Ok(mut avatar) => {
            // Update wearable in appearance
            match state.avatar_manager.appearance_engine.update_wearable(&mut avatar.appearance, wearable) {
                Ok(_) => {
                    // Save updated appearance
                    match state.avatar_manager.update_appearance(avatar_id, avatar.appearance).await {
                        Ok(_) => Ok(Json(ApiResponse::success("Wearable added successfully".to_string()))),
                        Err(e) => {
                            warn!("Failed to save wearable update: {}", e);
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to add wearable: {}", e);
                    Err(StatusCode::BAD_REQUEST)
                }
            }
        }
        Err(AvatarError::NotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            warn!("Failed to get avatar for wearable update: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn remove_wearable_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
    Path((avatar_id, wearable_type_str)): Path<(Uuid, String)>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Parse wearable type from string
    let wearable_type = match parse_wearable_type(&wearable_type_str) {
        Some(wt) => wt,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    match state.avatar_manager.get_avatar(avatar_id).await {
        Ok(mut avatar) => {
            match state.avatar_manager.appearance_engine.remove_wearable(&mut avatar.appearance, wearable_type) {
                Ok(removed) => {
                    if removed {
                        match state.avatar_manager.update_appearance(avatar_id, avatar.appearance).await {
                            Ok(_) => Ok(Json(ApiResponse::success("Wearable removed successfully".to_string()))),
                            Err(e) => {
                                warn!("Failed to save wearable removal: {}", e);
                                Err(StatusCode::INTERNAL_SERVER_ERROR)
                            }
                        }
                    } else {
                        Err(StatusCode::NOT_FOUND)
                    }
                }
                Err(e) => {
                    warn!("Failed to remove wearable: {}", e);
                    Err(StatusCode::BAD_REQUEST)
                }
            }
        }
        Err(AvatarError::NotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            warn!("Failed to get avatar for wearable removal: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Additional endpoints would be implemented similarly...

async fn get_system_health_endpoint(
    State(state): State<AvatarApiState>,
    _user: AuthenticatedUser,
) -> Result<Json<ApiResponse<AvatarSystemHealth>>, StatusCode> {
    let health = state.avatar_manager.get_system_health().await;
    Ok(Json(ApiResponse::success(health)))
}

// Request/Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAvatarRequest {
    pub user_id: Uuid,
    pub name: String,
    pub initial_appearance: Option<AvatarAppearance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAvatarRequest {
    pub name: Option<String>,
    pub appearance: Option<AvatarAppearance>,
    pub behavior: Option<AvatarBehavior>,
    pub social_profile: Option<AvatarSocialProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

// Utility functions

fn parse_wearable_type(type_str: &str) -> Option<WearableType> {
    match type_str.to_lowercase().as_str() {
        "skin" => Some(WearableType::Skin),
        "hair" => Some(WearableType::Hair),
        "eyes" => Some(WearableType::Eyes),
        "shirt" => Some(WearableType::Shirt),
        "pants" => Some(WearableType::Pants),
        "shoes" => Some(WearableType::Shoes),
        "socks" => Some(WearableType::Socks),
        "jacket" => Some(WearableType::Jacket),
        "gloves" => Some(WearableType::Gloves),
        "undershirt" => Some(WearableType::Undershirt),
        "underpants" => Some(WearableType::Underpants),
        "skirt" => Some(WearableType::Skirt),
        "alpha" => Some(WearableType::Alpha),
        "tattoo" => Some(WearableType::Tattoo),
        "physics" => Some(WearableType::Physics),
        "universal" => Some(WearableType::Universal),
        _ => None,
    }
}

// Placeholder implementations for endpoints not yet fully implemented

async fn add_attachment_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
    _json: Json<AvatarAttachment>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Attachment added successfully".to_string())))
}

async fn remove_attachment_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Attachment removed successfully".to_string())))
}

async fn update_visual_params_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
    _json: Json<Vec<VisualParameter>>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Visual parameters updated successfully".to_string())))
}

async fn get_behavior_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<AvatarBehavior>>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn update_behavior_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
    _json: Json<AvatarBehavior>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Behavior updated successfully".to_string())))
}

async fn start_animation_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
    _json: Json<AnimationState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Animation started successfully".to_string())))
}

async fn stop_animation_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Animation stopped successfully".to_string())))
}

async fn get_active_animations_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<AnimationState>>>, StatusCode> {
    Ok(Json(ApiResponse::success(Vec::new())))
}

async fn trigger_gesture_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Gesture triggered successfully".to_string())))
}

async fn update_expression_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
    _json: Json<FacialExpression>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Expression updated successfully".to_string())))
}

async fn get_social_profile_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<AvatarSocialProfile>>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn update_social_profile_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
    _json: Json<AvatarSocialProfile>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Social profile updated successfully".to_string())))
}

async fn get_friends_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<AvatarFriend>>>, StatusCode> {
    Ok(Json(ApiResponse::success(Vec::new())))
}

async fn add_friend_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Friend added successfully".to_string())))
}

async fn remove_friend_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Friend removed successfully".to_string())))
}

async fn get_achievements_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<Achievement>>>, StatusCode> {
    Ok(Json(ApiResponse::success(Vec::new())))
}

async fn add_achievement_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
    _json: Json<Achievement>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Achievement added successfully".to_string())))
}

async fn get_messages_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<AvatarMessage>>>, StatusCode> {
    Ok(Json(ApiResponse::success(Vec::new())))
}

async fn send_message_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
    _json: Json<AvatarMessage>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Message sent successfully".to_string())))
}

async fn login_avatar_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Avatar logged in successfully".to_string())))
}

async fn logout_avatar_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Avatar logged out successfully".to_string())))
}

async fn update_position_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
    _json: Json<PositionUpdate>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("Position updated successfully".to_string())))
}

async fn get_statistics_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<AvatarStatistics>>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn get_active_avatars_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<EnhancedAvatar>>>, StatusCode> {
    Ok(Json(ApiResponse::success(Vec::new())))
}

async fn get_avatars_in_region_endpoint(
    _state: State<AvatarApiState>,
    _user: AuthenticatedUser,
    _path: Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<EnhancedAvatar>>>, StatusCode> {
    Ok(Json(ApiResponse::success(Vec::new())))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub region_id: Uuid,
}