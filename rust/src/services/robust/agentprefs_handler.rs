use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::xml_response::*;
use super::RobustState;
use crate::services::traits::AgentPrefs;

pub async fn handle_agentprefs(
    State(state): State<RobustState>,
    body: String,
) -> axum::response::Response {
    let params = parse_form_body(&body);
    let method = params
        .get("METHOD")
        .or_else(|| params.get("method"))
        .cloned()
        .unwrap_or_default();

    debug!("[AGENTPREFS] Request: METHOD={}", method);

    let svc = match &state.agentprefs_service {
        Some(svc) => svc,
        None => {
            warn!("[AGENTPREFS] No AgentPrefs service configured");
            return failure_xml("No AgentPrefs service");
        }
    };

    match method.to_lowercase().as_str() {
        "getagentprefs" => {
            let user_id = params
                .get("UserID")
                .or_else(|| params.get("USERID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            debug!("[AGENTPREFS] getagentprefs: {}", user_id);

            match svc.get_agent_preferences(user_id).await {
                Ok(Some(prefs)) => {
                    let kvp = prefs.to_key_value_pairs();
                    let xml = build_xml_response(
                        &kvp.into_iter()
                            .map(|(k, v)| (k, XmlValue::Str(v)))
                            .collect(),
                    );
                    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
                }
                Ok(None) => {
                    let xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?><ServerResponse />";
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "text/xml")],
                        xml.to_string(),
                    )
                        .into_response()
                }
                Err(e) => {
                    warn!("[AGENTPREFS] getagentprefs error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "setagentprefs" => {
            let principal_id = params
                .get("PrincipalID")
                .or_else(|| params.get("PRINCIPALID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();

            let prefs = AgentPrefs {
                principal_id,
                access_prefs: params
                    .get("AccessPrefs")
                    .or_else(|| params.get("ACCESSPREFS"))
                    .cloned()
                    .unwrap_or_else(|| "M".to_string()),
                hover_height: params
                    .get("HoverHeight")
                    .or_else(|| params.get("HOVERHEIGHT"))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
                language: params
                    .get("Language")
                    .or_else(|| params.get("LANGUAGE"))
                    .cloned()
                    .unwrap_or_else(|| "en-us".to_string()),
                language_is_public: params
                    .get("LanguageIsPublic")
                    .or_else(|| params.get("LANGUAGEISPUBLIC"))
                    .map(|v| v == "1" || v.to_lowercase() == "true")
                    .unwrap_or(true),
                perm_everyone: params
                    .get("PermEveryone")
                    .or_else(|| params.get("PERMEVERYONE"))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                perm_group: params
                    .get("PermGroup")
                    .or_else(|| params.get("PERMGROUP"))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                perm_next_owner: params
                    .get("PermNextOwner")
                    .or_else(|| params.get("PERMNEXTOWNER"))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(532480),
            };

            info!(
                "[AGENTPREFS] setagentprefs: {} lang={}",
                principal_id, prefs.language
            );

            match svc.store_agent_preferences(&prefs).await {
                Ok(true) => success_result_response(),
                Ok(false) => failure_xml("Store failed"),
                Err(e) => {
                    warn!("[AGENTPREFS] setagentprefs error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "getagentlang" => {
            let user_id = params
                .get("UserID")
                .or_else(|| params.get("USERID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            debug!("[AGENTPREFS] getagentlang: {}", user_id);

            match svc.get_agent_lang(user_id).await {
                Ok(lang) => {
                    let mut data = HashMap::new();
                    data.insert("Language".to_string(), XmlValue::Str(lang));
                    let xml = build_xml_response(&data);
                    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
                }
                Err(e) => {
                    warn!("[AGENTPREFS] getagentlang error: {}", e);
                    let mut data = HashMap::new();
                    data.insert("Language".to_string(), XmlValue::Str("en-us".to_string()));
                    let xml = build_xml_response(&data);
                    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
                }
            }
        }
        _ => {
            warn!("[AGENTPREFS] Unknown method: {}", method);
            failure_xml(&format!("Unknown method: {}", method))
        }
    }
}

fn success_result_response() -> axum::response::Response {
    let xml = success_result();
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}

fn failure_xml(msg: &str) -> axum::response::Response {
    let xml = failure_result(msg);
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}
