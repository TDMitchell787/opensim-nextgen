use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use tracing::{debug, warn};

use super::RobustState;
use super::xml_response::parse_form_body;

pub async fn handle_map_get(
    State(state): State<RobustState>,
    Path(path_parts): Path<String>,
) -> impl IntoResponse {
    let trimmed = path_parts.trim_matches('/');
    let parts: Vec<&str> = trimmed.split('/').collect();

    let (scope_id, filename) = if parts.len() >= 2 {
        (parts[0], parts[1])
    } else if parts.len() == 1 {
        ("00000000-0000-0000-0000-000000000000", parts[0])
    } else {
        return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/plain")],
            Vec::new()).into_response();
    };

    debug!("[MAP] GET tile: scope={} file={}", scope_id, filename);

    let svc = match &state.map_service {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/plain")],
                Vec::new()).into_response();
        }
    };

    match svc.get_map_tile(filename, scope_id).await {
        Ok(Some(data)) => {
            let content_type = if filename.ends_with(".png") {
                "image/png"
            } else {
                "image/jpeg"
            };
            (StatusCode::OK, [(header::CONTENT_TYPE, content_type)], data).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/plain")],
                Vec::new()).into_response()
        }
        Err(e) => {
            warn!("[MAP] Error getting tile: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "text/plain")],
                Vec::new()).into_response()
        }
    }
}

pub async fn handle_map_post(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let params = parse_form_body(&body);

    let x: i32 = params.get("X").or_else(|| params.get("x"))
        .and_then(|s| s.parse().ok()).unwrap_or(0);
    let y: i32 = params.get("Y").or_else(|| params.get("y"))
        .and_then(|s| s.parse().ok()).unwrap_or(0);
    let scope = params.get("SCOPE").or_else(|| params.get("scope"))
        .cloned().unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string());

    debug!("[MAP] POST tile: x={} y={} scope={}", x, y, scope);

    let svc = match &state.map_service {
        Some(s) => s,
        None => {
            return (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                map_failure_xml("Map service not configured"));
        }
    };

    let data_param = params.get("DATA").or_else(|| params.get("data"));

    let result = if let Some(b64data) = data_param {
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            b64data
        ).unwrap_or_default();
        if decoded.is_empty() {
            svc.remove_map_tile(x, y, &scope).await
        } else {
            svc.add_map_tile(x, y, &decoded, &scope).await
        }
    } else {
        svc.remove_map_tile(x, y, &scope).await
    };

    match result {
        Ok(true) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
            map_success_xml()),
        Ok(false) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
            map_failure_xml("Operation failed")),
        Err(e) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
            map_failure_xml(&format!("{}", e))),
    }
}

pub async fn handle_removemap_post(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let params = parse_form_body(&body);

    let x: i32 = params.get("X").or_else(|| params.get("x"))
        .and_then(|s| s.parse().ok()).unwrap_or(0);
    let y: i32 = params.get("Y").or_else(|| params.get("y"))
        .and_then(|s| s.parse().ok()).unwrap_or(0);
    let scope = params.get("SCOPE").or_else(|| params.get("scope"))
        .cloned().unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string());

    debug!("[MAP] POST removemap: x={} y={} scope={}", x, y, scope);

    let svc = match &state.map_service {
        Some(s) => s,
        None => {
            return (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                map_failure_xml("Map service not configured"));
        }
    };

    match svc.remove_map_tile(x, y, &scope).await {
        Ok(_) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
            map_success_xml()),
        Err(e) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
            map_failure_xml(&format!("{}", e))),
    }
}

fn map_success_xml() -> String {
    r#"<Result><response type="Failure">false</response></Result>"#.to_string()
}

fn map_failure_xml(reason: &str) -> String {
    format!(r#"<Result><response type="Failure">true</response><message>{}</message></Result>"#, reason)
}
