use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use std::collections::HashMap;
use tracing::{debug, warn};
use uuid::Uuid;

use super::RobustState;
use super::xml_response::*;
use crate::services::traits::AvatarData;

pub async fn handle_avatar(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let params = parse_form_body(&body);
    let method = params.get("METHOD").or_else(|| params.get("method")).cloned().unwrap_or_default();

    debug!("Avatar handler: METHOD={}", method);

    let xml = match method.as_str() {
        "getavatar" => handle_get_avatar(&state, &params).await,
        "setavatar" => handle_set_avatar(&state, &params).await,
        "resetavatar" => handle_reset_avatar(&state, &params).await,
        "removeitems" => handle_remove_items(&state, &params).await,
        _ => {
            warn!("Avatar handler: unknown method '{}'", method);
            failure_result(&format!("Unknown method: {}", method))
        }
    };

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")], xml)
}

async fn handle_get_avatar(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "UserID");

    match state.avatar_service.get_avatar(principal_id).await {
        Ok(avatar) => {
            let mut fields = HashMap::new();
            fields.insert("AvatarType".to_string(), avatar.avatar_type.to_string());
            for (key, value) in &avatar.data {
                fields.insert(key.clone(), value.clone());
            }
            single_result(fields)
        }
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_set_avatar(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "UserID");

    let mut data = HashMap::new();
    for (key, value) in params {
        if key != "METHOD" && key != "UserID" && key != "method" {
            data.insert(key.clone(), value.clone());
        }
    }

    let avatar_type: i32 = data.remove("AvatarType")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let avatar_data = AvatarData {
        avatar_type,
        data,
    };

    match state.avatar_service.set_avatar(principal_id, &avatar_data).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Set avatar returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_reset_avatar(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "UserID");

    match state.avatar_service.reset_avatar(principal_id).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Reset avatar returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_remove_items(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "UserID");
    let names: Vec<String> = params.get("Names")
        .map(|s| s.split(',').map(|n| n.trim().to_string()).collect())
        .unwrap_or_default();

    match state.avatar_service.remove_items(principal_id, &names).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Remove items returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

fn parse_uuid(params: &HashMap<String, String>, key: &str) -> Uuid {
    params.get(key)
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil())
}
