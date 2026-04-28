use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::xml_response::*;
use super::RobustState;
use crate::services::traits::OfflineIM;

pub async fn handle_offlineim(
    State(state): State<RobustState>,
    body: String,
) -> axum::response::Response {
    let params = parse_form_body(&body);
    let method = params
        .get("METHOD")
        .or_else(|| params.get("method"))
        .cloned()
        .unwrap_or_default();

    debug!("[OFFLINEIM] Request: METHOD={}", method);

    let svc = match &state.offlineim_service {
        Some(svc) => svc,
        None => {
            warn!("[OFFLINEIM] No Offline IM service configured");
            return failure_xml("No Offline IM service");
        }
    };

    match method.to_uppercase().as_str() {
        "GET" => {
            let principal_id = params
                .get("PrincipalID")
                .or_else(|| params.get("PRINCIPALID"))
                .cloned()
                .unwrap_or_default();
            debug!("[OFFLINEIM] GET: {}", principal_id);

            match svc.get_messages(&principal_id).await {
                Ok(messages) => {
                    if messages.is_empty() {
                        return null_result_response();
                    }
                    let items: Vec<HashMap<String, String>> = messages
                        .iter()
                        .map(|im| {
                            let mut m = HashMap::new();
                            m.insert("PrincipalID".to_string(), im.principal_id.clone());
                            m.insert("FromID".to_string(), im.from_id.clone());
                            m.insert("Message".to_string(), im.message.clone());
                            m.insert("TMStamp".to_string(), im.timestamp.to_string());
                            m
                        })
                        .collect();
                    let xml = list_result("im", items);
                    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
                }
                Err(e) => {
                    warn!("[OFFLINEIM] GET error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "STORE" => {
            let principal_id = params
                .get("PrincipalID")
                .or_else(|| params.get("PRINCIPALID"))
                .or_else(|| params.get("ToAgentID"))
                .cloned()
                .unwrap_or_default();
            let from_id = params
                .get("FromID")
                .or_else(|| params.get("FROMID"))
                .or_else(|| params.get("FromAgentID"))
                .cloned()
                .unwrap_or_default();
            let message = params
                .get("Message")
                .or_else(|| params.get("MESSAGE"))
                .cloned()
                .unwrap_or_default();

            info!("[OFFLINEIM] STORE: from={} to={}", from_id, principal_id);

            let im = OfflineIM {
                id: 0,
                principal_id,
                from_id,
                message,
                timestamp: 0,
            };

            match svc.store_message(&im).await {
                Ok(true) => bool_result_response(true),
                Ok(false) => bool_result_response(false),
                Err(e) => {
                    warn!("[OFFLINEIM] STORE error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "DELETE" => {
            let user_id = params
                .get("UserID")
                .or_else(|| params.get("USERID"))
                .or_else(|| params.get("PrincipalID"))
                .cloned()
                .unwrap_or_default();
            info!("[OFFLINEIM] DELETE: {}", user_id);

            match svc.delete_messages(&user_id).await {
                Ok(true) => bool_result_response(true),
                Ok(false) => bool_result_response(false),
                Err(e) => {
                    warn!("[OFFLINEIM] DELETE error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        _ => {
            warn!("[OFFLINEIM] Unknown method: {}", method);
            failure_xml(&format!("Unknown method: {}", method))
        }
    }
}

fn bool_result_response(val: bool) -> axum::response::Response {
    let xml = if val {
        format!("<?xml version=\"1.0\"?><ServerResponse><RESULT>true</RESULT></ServerResponse>")
    } else {
        format!("<?xml version=\"1.0\"?><ServerResponse><RESULT>false</RESULT></ServerResponse>")
    };
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}

fn null_result_response() -> axum::response::Response {
    let xml = null_result();
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}

fn failure_xml(msg: &str) -> axum::response::Response {
    let xml = failure_result(msg);
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}
