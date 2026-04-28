use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::RobustState;

fn hash_to_path(base_dir: &str, id: &str) -> Option<PathBuf> {
    if id.len() < 10 {
        return None;
    }
    let mut path = PathBuf::from(base_dir);
    path.push(&id[0..2]);
    path.push(&id[2..4]);
    path.push(&id[4..6]);
    path.push(&id[6..10]);
    path.push(id);
    Some(path)
}

pub async fn handle_bakes_get(
    State(state): State<RobustState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let clean_id = id.replace('-', "");
    let base_dir = state.bakes_dir.as_deref().unwrap_or("./bakes");

    let file_path = match hash_to_path(base_dir, &clean_id) {
        Some(p) => p,
        None => {
            debug!("[BAKES] GET invalid id: {}", id);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    match tokio::fs::read(&file_path).await {
        Ok(data) => {
            debug!("[BAKES] GET {}: {} bytes", id, data.len());
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/octet-stream")],
                data,
            )
                .into_response()
        }
        Err(_) => {
            debug!("[BAKES] GET {}: not found at {:?}", id, file_path);
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/octet-stream")],
                Vec::<u8>::new(),
            )
                .into_response()
        }
    }
}

pub async fn handle_bakes_post(
    State(state): State<RobustState>,
    Path(id): Path<String>,
    body: Bytes,
) -> axum::response::Response {
    let clean_id = id.replace('-', "");
    let base_dir = state.bakes_dir.as_deref().unwrap_or("./bakes");

    let file_path = match hash_to_path(base_dir, &clean_id) {
        Some(p) => p,
        None => {
            warn!("[BAKES] POST invalid id: {}", id);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    if let Some(parent) = file_path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            warn!("[BAKES] POST failed to create dir {:?}: {}", parent, e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    match tokio::fs::write(&file_path, &body).await {
        Ok(_) => {
            debug!(
                "[BAKES] POST {}: {} bytes stored at {:?}",
                id,
                body.len(),
                file_path
            );
            StatusCode::OK.into_response()
        }
        Err(e) => {
            warn!("[BAKES] POST {}: write error: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
