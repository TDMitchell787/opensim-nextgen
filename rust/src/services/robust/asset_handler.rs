use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::collections::HashMap;
use tracing::{debug, warn};

use super::xml_response::*;
use super::RobustState;
use crate::services::traits::{AssetBase, AssetMetadata};

pub async fn handle_asset_post(
    State(state): State<RobustState>,
    headers: axum::http::HeaderMap,
    body: String,
) -> impl IntoResponse {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.contains("text/xml") || content_type.contains("application/xml") {
        if body.contains("<AssetBase") {
            return handle_xml_asset_store(&state, &body).await;
        }
    }

    let params = parse_form_body(&body);
    let method = params
        .get("METHOD")
        .or_else(|| params.get("method"))
        .cloned()
        .unwrap_or_default();

    debug!("Asset handler POST: METHOD={}", method);

    let xml = match method.as_str() {
        "get" => handle_get(&state, &params).await,
        "get_metadata" => handle_get_metadata(&state, &params).await,
        "store" => handle_store(&state, &params).await,
        "delete" => handle_delete(&state, &params).await,
        "exist" | "exists" => handle_exists(&state, &params).await,
        _ => {
            warn!("Asset handler: unknown method '{}'", method);
            failure_result(&format!("Unknown method: {}", method))
        }
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
        xml,
    )
}

async fn handle_xml_asset_store(
    state: &RobustState,
    xml_body: &str,
) -> (StatusCode, [(header::HeaderName, &'static str); 1], String) {
    use tracing::info;

    fn extract_xml_field<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        let start = xml.find(&open)? + open.len();
        let end = xml[start..].find(&close)?;
        Some(&xml[start..start + end])
    }

    let id = extract_xml_field(xml_body, "ID")
        .or_else(|| extract_xml_field(xml_body, "FullID"))
        .unwrap_or_default();

    let clean_id = if id.contains('|') {
        id.split('|').last().unwrap_or(id)
    } else if id.starts_with("http://") || id.starts_with("https://") {
        id.rsplit('/').next().unwrap_or(id)
    } else {
        id
    };

    let name = extract_xml_field(xml_body, "Name").unwrap_or("foreign asset");
    let description = extract_xml_field(xml_body, "Description").unwrap_or("");
    let asset_type: i8 = extract_xml_field(xml_body, "Type")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let local = extract_xml_field(xml_body, "Local")
        .map(|s| s == "True" || s == "true")
        .unwrap_or(false);
    let temporary = extract_xml_field(xml_body, "Temporary")
        .map(|s| s == "True" || s == "true")
        .unwrap_or(false);
    let creator_id = extract_xml_field(xml_body, "CreatorID").unwrap_or("");
    let flags: i32 = extract_xml_field(xml_body, "Flags")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let data = extract_xml_field(xml_body, "Data")
        .map(|s| {
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
                .unwrap_or_default()
        })
        .unwrap_or_default();

    info!(
        "[HG-ASSET] XML asset POST: id='{}' clean_id='{}' name='{}' type={} size={} bytes",
        id,
        clean_id,
        name,
        asset_type,
        data.len()
    );

    let asset = AssetBase {
        id: clean_id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        asset_type,
        local,
        temporary,
        data,
        creator_id: creator_id.to_string(),
        flags,
    };

    match state.asset_service.store(&asset).await {
        Ok(stored_id) => {
            info!(
                "[HG-ASSET] Stored XML asset: {} ({} bytes)",
                stored_id,
                asset.data.len()
            );
            let response = format!(
                "<?xml version=\"1.0\" encoding=\"utf-8\"?><string xmlns=\"http://schemas.microsoft.com/2003/10/Serialization/\">{}</string>",
                stored_id
            );
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                response,
            )
        }
        Err(e) => {
            tracing::warn!("[HG-ASSET] Failed to store XML asset '{}': {}", clean_id, e);
            let response = format!(
                "<?xml version=\"1.0\" encoding=\"utf-8\"?><string xmlns=\"http://schemas.microsoft.com/2003/10/Serialization/\">{}</string>",
                ""
            );
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                response,
            )
        }
    }
}

pub async fn handle_asset_get(
    State(state): State<RobustState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    debug!("Asset handler GET: id={}", id);

    match state.asset_service.get_data(&id).await {
        Ok(Some(data)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/octet-stream")],
            data,
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn handle_asset_metadata_get(
    State(state): State<RobustState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    debug!("Asset handler GET metadata: id={}", id);

    match state.asset_service.get_metadata(&id).await {
        Ok(Some(meta)) => {
            let xml = single_result(metadata_to_fields(&meta));
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                xml,
            )
                .into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn handle_get(state: &RobustState, params: &HashMap<String, String>) -> String {
    let id = params
        .get("ID")
        .or_else(|| params.get("id"))
        .cloned()
        .unwrap_or_default();

    match state.asset_service.get(&id).await {
        Ok(Some(asset)) => single_result(asset_to_fields(&asset)),
        Ok(None) => null_result(),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_metadata(state: &RobustState, params: &HashMap<String, String>) -> String {
    let id = params
        .get("ID")
        .or_else(|| params.get("id"))
        .cloned()
        .unwrap_or_default();

    match state.asset_service.get_metadata(&id).await {
        Ok(Some(meta)) => single_result(metadata_to_fields(&meta)),
        Ok(None) => null_result(),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_store(state: &RobustState, params: &HashMap<String, String>) -> String {
    let asset = AssetBase {
        id: params
            .get("ID")
            .or_else(|| params.get("id"))
            .cloned()
            .unwrap_or_default(),
        name: params
            .get("Name")
            .or_else(|| params.get("NAME"))
            .cloned()
            .unwrap_or_default(),
        description: params.get("Description").cloned().unwrap_or_default(),
        asset_type: params.get("Type").and_then(|s| s.parse().ok()).unwrap_or(0),
        local: params
            .get("Local")
            .map(|s| s == "True" || s == "true" || s == "1")
            .unwrap_or(false),
        temporary: params
            .get("Temporary")
            .map(|s| s == "True" || s == "true" || s == "1")
            .unwrap_or(false),
        data: params
            .get("Data")
            .map(|s| {
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
                    .unwrap_or_default()
            })
            .unwrap_or_default(),
        creator_id: params.get("CreatorID").cloned().unwrap_or_default(),
        flags: params
            .get("Flags")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
    };

    match state.asset_service.store(&asset).await {
        Ok(id) => {
            let mut fields = HashMap::new();
            fields.insert("Result".to_string(), id);
            single_result(fields)
        }
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_delete(state: &RobustState, params: &HashMap<String, String>) -> String {
    let id = params
        .get("ID")
        .or_else(|| params.get("id"))
        .cloned()
        .unwrap_or_default();

    match state.asset_service.delete(&id).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Asset not found"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_exists(state: &RobustState, params: &HashMap<String, String>) -> String {
    let id = params
        .get("ID")
        .or_else(|| params.get("id"))
        .cloned()
        .unwrap_or_default();

    match state.asset_service.asset_exists(&id).await {
        Ok(exists) => bool_result(exists),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

fn asset_to_fields(asset: &AssetBase) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("ID".to_string(), asset.id.clone());
    m.insert("Name".to_string(), asset.name.clone());
    m.insert("Description".to_string(), asset.description.clone());
    m.insert("Type".to_string(), asset.asset_type.to_string());
    m.insert(
        "Local".to_string(),
        if asset.local { "True" } else { "False" }.to_string(),
    );
    m.insert(
        "Temporary".to_string(),
        if asset.temporary { "True" } else { "False" }.to_string(),
    );
    m.insert("CreatorID".to_string(), asset.creator_id.clone());
    m.insert("Flags".to_string(), asset.flags.to_string());
    m.insert(
        "Data".to_string(),
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &asset.data),
    );
    m
}

pub async fn handle_asset_delete(
    State(state): State<RobustState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    debug!("Asset handler DELETE: id={}", id);

    match state.asset_service.delete(&id).await {
        Ok(true) => (StatusCode::OK, "True").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "False").into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn handle_assets_exist(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    debug!("Asset handler: AssetsExist batch check");

    let mut ids = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<string>") && trimmed.ends_with("</string>") {
            let id = &trimmed[8..trimmed.len() - 9];
            ids.push(id.to_string());
        }
    }

    let mut results = Vec::new();
    for id in &ids {
        let exists = state.asset_service.asset_exists(id).await.unwrap_or(false);
        results.push(exists);
    }

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<ArrayOfBoolean xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\">\n");
    for exists in &results {
        xml.push_str(&format!("  <boolean>{}</boolean>\n", exists));
    }
    xml.push_str("</ArrayOfBoolean>");

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
        xml,
    )
}

fn metadata_to_fields(meta: &AssetMetadata) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("ID".to_string(), meta.id.clone());
    m.insert("Name".to_string(), meta.name.clone());
    m.insert("Description".to_string(), meta.description.clone());
    m.insert("Type".to_string(), meta.asset_type.to_string());
    m.insert(
        "Local".to_string(),
        if meta.local { "True" } else { "False" }.to_string(),
    );
    m.insert(
        "Temporary".to_string(),
        if meta.temporary { "True" } else { "False" }.to_string(),
    );
    m.insert("CreatorID".to_string(), meta.creator_id.clone());
    m.insert("Flags".to_string(), meta.flags.to_string());
    m.insert("CreatedDate".to_string(), meta.created_date.to_string());
    m
}
