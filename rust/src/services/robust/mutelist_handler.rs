use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use tracing::{debug, warn};

use super::xml_response::*;
use super::RobustState;
use crate::services::traits::MuteData;

pub async fn handle_mutelist(State(state): State<RobustState>, body: String) -> impl IntoResponse {
    let params = parse_form_body(&body);
    let method = params
        .get("METHOD")
        .or_else(|| params.get("method"))
        .cloned()
        .unwrap_or_default();

    debug!("[MUTELIST] Handler: METHOD={}", method);

    let svc = match &state.mutelist_service {
        Some(s) => s,
        None => {
            warn!("[MUTELIST] Service not configured");
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                failure_result("MuteList service not configured"),
            );
        }
    };

    let xml = match method.to_lowercase().as_str() {
        "get" => {
            let agent_id = params
                .get("agentid")
                .or_else(|| params.get("AGENTID"))
                .cloned()
                .unwrap_or_default();

            if agent_id.is_empty() {
                failure_result("Missing agentid")
            } else {
                match svc.get_mutes(&agent_id).await {
                    Ok(mutes) => {
                        let mut text = String::new();
                        for m in &mutes {
                            text.push_str(&format!(
                                "{} {} {}|{}\n",
                                m.mute_type,
                                m.mute_id.trim(),
                                m.mute_name.trim(),
                                m.mute_flags
                            ));
                        }
                        let encoded = base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            text.as_bytes(),
                        );
                        let mut data = std::collections::HashMap::new();
                        data.insert("result".to_string(), XmlValue::Str(encoded));
                        build_xml_response(&data)
                    }
                    Err(e) => failure_result(&format!("{}", e)),
                }
            }
        }
        "update" => {
            let agent_id = params
                .get("agentid")
                .or_else(|| params.get("AGENTID"))
                .cloned()
                .unwrap_or_default();
            let mute_id = params
                .get("muteid")
                .or_else(|| params.get("MUTEID"))
                .cloned()
                .unwrap_or_default();

            if agent_id.is_empty() || mute_id.is_empty() {
                failure_result("Missing agentid or muteid")
            } else {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i32;

                let mute = MuteData {
                    agent_id,
                    mute_id,
                    mute_name: params.get("mutename").cloned().unwrap_or_default(),
                    mute_type: params
                        .get("mutetype")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                    mute_flags: params
                        .get("muteflags")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                    stamp: params
                        .get("mutestamp")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(now),
                };

                match svc.update_mute(&mute).await {
                    Ok(true) => success_result(),
                    Ok(false) => failure_result("Update failed"),
                    Err(e) => failure_result(&format!("{}", e)),
                }
            }
        }
        "delete" => {
            let agent_id = params
                .get("agentid")
                .or_else(|| params.get("AGENTID"))
                .cloned()
                .unwrap_or_default();
            let mute_id = params
                .get("muteid")
                .or_else(|| params.get("MUTEID"))
                .cloned()
                .unwrap_or_default();
            let mute_name = params.get("mutename").cloned().unwrap_or_default();

            if agent_id.is_empty() || mute_id.is_empty() {
                failure_result("Missing agentid or muteid")
            } else {
                match svc.remove_mute(&agent_id, &mute_id, &mute_name).await {
                    Ok(true) => success_result(),
                    Ok(false) => failure_result("Mute not found"),
                    Err(e) => failure_result(&format!("{}", e)),
                }
            }
        }
        _ => {
            warn!("[MUTELIST] Unknown method: {}", method);
            failure_result(&format!("Unknown method: {}", method))
        }
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
        xml,
    )
}
