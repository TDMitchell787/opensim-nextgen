use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use tracing::debug;

pub async fn handle_helo_get() -> impl IntoResponse {
    debug!("HELO GET request received");
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());
    headers.insert("X-Handlers-Provided", "opensim-robust".parse().unwrap());
    (
        StatusCode::OK,
        headers,
        "<html><body><h1>OpenSim Next ROBUST Server</h1></body></html>",
    )
}

pub async fn handle_helo_head() -> impl IntoResponse {
    debug!("HELO HEAD request received");
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());
    headers.insert("X-Handlers-Provided", "opensim-robust".parse().unwrap());
    (StatusCode::OK, headers, "")
}
