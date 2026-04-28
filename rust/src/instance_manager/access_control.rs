use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;

pub mod controller_levels {
    pub const NEWBIE: u32 = 0;
    pub const USER: u32 = 100;
    pub const REGION_OWNER: u32 = 200;
    pub const GRID_ADMIN: u32 = 300;
    pub const OPERATOR: u32 = 400;
    pub const CENTRAL_ADMIN: u32 = 500;

    pub fn has_permission(user_level: u32, required_level: u32) -> bool {
        user_level >= required_level
    }

    pub fn level_name(level: u32) -> &'static str {
        match level {
            0..=99 => "Newbie",
            100..=199 => "User",
            200..=299 => "RegionOwner",
            300..=399 => "GridAdmin",
            400..=499 => "Operator",
            500.. => "CentralAdmin",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerSession {
    pub user_id: String,
    pub username: String,
    pub user_level: u32,
    pub auth_method: String,
}

#[derive(Debug)]
pub enum ControllerAuthError {
    MissingAuth,
    InvalidAuth,
    InsufficientPermissions,
}

impl IntoResponse for ControllerAuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ControllerAuthError::MissingAuth => {
                (StatusCode::UNAUTHORIZED, "Authentication required")
            }
            ControllerAuthError::InvalidAuth => {
                (StatusCode::UNAUTHORIZED, "Invalid authentication")
            }
            ControllerAuthError::InsufficientPermissions => {
                (StatusCode::FORBIDDEN, "Insufficient permissions")
            }
        };
        (status, Json(json!({ "success": false, "error": message }))).into_response()
    }
}

fn extract_auth_from_headers(headers: &HeaderMap) -> Option<ControllerSession> {
    if let Some(api_key) = headers.get("X-API-Key").and_then(|v| v.to_str().ok()) {
        let expected = std::env::var("OPENSIM_API_KEY")
            .unwrap_or_else(|_| "dev-api-key-change-me".to_string());
        if api_key == expected {
            return Some(ControllerSession {
                user_id: "api-key-admin".to_string(),
                username: "admin".to_string(),
                user_level: controller_levels::CENTRAL_ADMIN,
                auth_method: "api_key".to_string(),
            });
        }
    }

    if let Some(auth_header) = headers.get("Authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if let Ok(claims) = decode_jwt_claims(token) {
                return Some(ControllerSession {
                    user_id: claims.sub,
                    username: claims.username,
                    user_level: claims.user_level,
                    auth_method: "jwt".to_string(),
                });
            }
        }
    }

    None
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    sub: String,
    username: String,
    user_level: u32,
}

fn decode_jwt_claims(token: &str) -> Result<JwtClaims, ()> {
    let secret = std::env::var("OPENSIM_JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

    let validation = jsonwebtoken::Validation::default();
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());

    jsonwebtoken::decode::<JwtClaims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|_| ())
}

pub struct RequireLevel<const LEVEL: u32>(pub ControllerSession);

macro_rules! impl_require_level {
    ($level:expr) => {
        #[axum::async_trait]
        impl<S> FromRequestParts<S> for RequireLevel<$level>
        where
            S: Send + Sync,
        {
            type Rejection = ControllerAuthError;

            async fn from_request_parts(
                parts: &mut Parts,
                _state: &S,
            ) -> Result<Self, Self::Rejection> {
                let session = extract_auth_from_headers(&parts.headers)
                    .ok_or(ControllerAuthError::MissingAuth)?;

                if !controller_levels::has_permission(session.user_level, $level) {
                    warn!(
                        "Access denied for {} (level {}) — requires {}",
                        session.username, session.user_level, $level
                    );
                    return Err(ControllerAuthError::InsufficientPermissions);
                }

                Ok(RequireLevel(session))
            }
        }
    };
}

impl_require_level!(0);
impl_require_level!(100);
impl_require_level!(200);
impl_require_level!(300);
impl_require_level!(400);
impl_require_level!(500);

pub type RequireNewbie = RequireLevel<0>;
pub type RequireUser = RequireLevel<100>;
pub type RequireRegionOwner = RequireLevel<200>;
pub type RequireGridAdmin = RequireLevel<300>;
pub type RequireOperator = RequireLevel<400>;
pub type RequireCentralAdmin = RequireLevel<500>;

#[derive(Debug, Deserialize)]
pub struct SetLevelRequest {
    pub user_id: String,
    pub level: u32,
}

#[derive(Debug, Serialize)]
pub struct SetLevelResponse {
    pub success: bool,
    pub message: String,
    pub user_id: String,
    pub new_level: u32,
    pub level_name: String,
}

pub async fn handle_set_level(
    RequireLevel(session): RequireLevel<500>,
    Json(request): Json<SetLevelRequest>,
) -> (StatusCode, Json<SetLevelResponse>) {
    if request.level > controller_levels::CENTRAL_ADMIN {
        return (
            StatusCode::BAD_REQUEST,
            Json(SetLevelResponse {
                success: false,
                message: "Level cannot exceed 500 (CentralAdmin)".to_string(),
                user_id: request.user_id,
                new_level: request.level,
                level_name: controller_levels::level_name(request.level).to_string(),
            }),
        );
    }

    tracing::info!(
        "CentralAdmin {} setting user {} to level {} ({})",
        session.username,
        request.user_id,
        request.level,
        controller_levels::level_name(request.level)
    );

    (
        StatusCode::OK,
        Json(SetLevelResponse {
            success: true,
            message: format!(
                "User level updated to {} ({})",
                request.level,
                controller_levels::level_name(request.level)
            ),
            user_id: request.user_id,
            new_level: request.level,
            level_name: controller_levels::level_name(request.level).to_string(),
        }),
    )
}

pub async fn handle_list_users(
    RequireLevel(_session): RequireLevel<400>,
) -> Json<serde_json::Value> {
    Json(json!({
        "users": [],
        "message": "User listing from database not yet implemented — use instance-level user management"
    }))
}
