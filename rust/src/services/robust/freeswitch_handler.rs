use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::debug;

pub async fn handle_freeswitch(body: String) -> impl IntoResponse {
    debug!("[FREESWITCH] Request, body_len={}", body.len());
    (
        StatusCode::OK,
        [("Content-Type", "text/xml")],
        "<?xml version=\"1.0\"?><document type=\"freeswitch/xml\"><section name=\"result\"><result status=\"not found\" /></section></document>".to_string(),
    )
}
