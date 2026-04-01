use axum::extract::{State, Path};
use axum::response::IntoResponse;
use axum::http::StatusCode;
use tracing::{info, debug};

use super::RobustState;

pub async fn handle_neighbour(
    State(_state): State<RobustState>,
    Path(region_id): Path<String>,
    body: String,
) -> impl IntoResponse {
    info!("[NEIGHBOUR] HelloNeighbour for region {}, body_len={}", region_id, body.len());
    debug!("[NEIGHBOUR] Body: {}", &body[..body.len().min(500)]);

    (
        StatusCode::OK,
        [("Content-Type", "application/json")],
        r#"{"success":true}"#.to_string(),
    )
}

pub async fn handle_neighbour_trailing(
    State(state): State<RobustState>,
    Path(region_id): Path<String>,
    body: String,
) -> impl IntoResponse {
    handle_neighbour(State(state), Path(region_id), body).await
}
