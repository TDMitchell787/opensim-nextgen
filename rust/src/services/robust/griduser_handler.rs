use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::xml_response::*;
use super::RobustState;

pub async fn handle_griduser(
    State(state): State<RobustState>,
    body: String,
) -> axum::response::Response {
    let params = parse_form_body(&body);
    let method = params
        .get("METHOD")
        .or_else(|| params.get("method"))
        .cloned()
        .unwrap_or_default();

    debug!("[GRIDUSER] Request: METHOD={}", method);

    let svc = match &state.griduser_service {
        Some(svc) => svc,
        None => {
            warn!("[GRIDUSER] No GridUser service configured");
            return failure_xml("No GridUser service");
        }
    };

    match method.to_lowercase().as_str() {
        "loggedin" => {
            let user_id = params
                .get("UserID")
                .or_else(|| params.get("USERID"))
                .cloned()
                .unwrap_or_default();
            info!("[GRIDUSER] loggedin: {}", user_id);

            match svc.logged_in(&user_id).await {
                Ok(Some(info)) => griduser_info_response(&info.to_key_value_pairs()),
                Ok(None) => null_result_response(),
                Err(e) => {
                    warn!("[GRIDUSER] loggedin error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "loggedout" => {
            let user_id = params
                .get("UserID")
                .or_else(|| params.get("USERID"))
                .cloned()
                .unwrap_or_default();
            let region_id = params
                .get("RegionID")
                .or_else(|| params.get("REGIONID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let position = params
                .get("Position")
                .or_else(|| params.get("POSITION"))
                .cloned()
                .unwrap_or_else(|| "<0,0,0>".to_string());
            let look_at = params
                .get("LookAt")
                .or_else(|| params.get("LOOKAT"))
                .cloned()
                .unwrap_or_else(|| "<0,0,0>".to_string());
            let _session_id = params
                .get("SessionID")
                .or_else(|| params.get("SESSIONID"))
                .cloned()
                .unwrap_or_default();
            info!(
                "[GRIDUSER] loggedout: {} region={} session={}",
                user_id, region_id, _session_id
            );

            match svc
                .logged_out(&user_id, region_id, &position, &look_at)
                .await
            {
                Ok(true) => success_result_response(),
                Ok(false) => failure_xml("LoggedOut failed"),
                Err(e) => {
                    warn!("[GRIDUSER] loggedout error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "sethome" => {
            let user_id = params
                .get("UserID")
                .or_else(|| params.get("USERID"))
                .cloned()
                .unwrap_or_default();
            let region_id = params
                .get("RegionID")
                .or_else(|| params.get("REGIONID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let position = params
                .get("Position")
                .or_else(|| params.get("POSITION"))
                .cloned()
                .unwrap_or_else(|| "<0,0,0>".to_string());
            let look_at = params
                .get("LookAt")
                .or_else(|| params.get("LOOKAT"))
                .cloned()
                .unwrap_or_else(|| "<0,0,0>".to_string());
            info!("[GRIDUSER] sethome: {} region={}", user_id, region_id);

            match svc.set_home(&user_id, region_id, &position, &look_at).await {
                Ok(true) => success_result_response(),
                Ok(false) => failure_xml("SetHome failed"),
                Err(e) => {
                    warn!("[GRIDUSER] sethome error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "setposition" => {
            let user_id = params
                .get("UserID")
                .or_else(|| params.get("USERID"))
                .cloned()
                .unwrap_or_default();
            let region_id = params
                .get("RegionID")
                .or_else(|| params.get("REGIONID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let position = params
                .get("Position")
                .or_else(|| params.get("POSITION"))
                .cloned()
                .unwrap_or_else(|| "<0,0,0>".to_string());
            let look_at = params
                .get("LookAt")
                .or_else(|| params.get("LOOKAT"))
                .cloned()
                .unwrap_or_else(|| "<0,0,0>".to_string());
            let _session_id = params
                .get("SessionID")
                .or_else(|| params.get("SESSIONID"))
                .cloned()
                .unwrap_or_default();
            info!(
                "[GRIDUSER] setposition: {} region={} session={}",
                user_id, region_id, _session_id
            );

            match svc
                .set_last_position(&user_id, region_id, &position, &look_at)
                .await
            {
                Ok(true) => success_result_response(),
                Ok(false) => failure_xml("SetPosition failed"),
                Err(e) => {
                    warn!("[GRIDUSER] setposition error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "getgriduserinfo" => {
            let user_id = params
                .get("UserID")
                .or_else(|| params.get("USERID"))
                .cloned()
                .unwrap_or_default();
            debug!("[GRIDUSER] getgriduserinfo: {}", user_id);

            match svc.get_grid_user_info(&user_id).await {
                Ok(Some(info)) => griduser_info_response(&info.to_key_value_pairs()),
                Ok(None) => null_result_response(),
                Err(e) => {
                    warn!("[GRIDUSER] getgriduserinfo error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "getgriduserinfos" => {
            let ids_str = params
                .get("AgentIDs")
                .or_else(|| params.get("AGENTIDS"))
                .cloned()
                .unwrap_or_default();
            let user_ids: Vec<String> = ids_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            debug!("[GRIDUSER] getgriduserinfos: {} users", user_ids.len());

            match svc.get_grid_user_infos(&user_ids).await {
                Ok(infos) => {
                    let items: Vec<HashMap<String, String>> =
                        infos.iter().map(|i| i.to_key_value_pairs()).collect();
                    let xml = list_result("griduser", items);
                    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
                }
                Err(e) => {
                    warn!("[GRIDUSER] getgriduserinfos error: {}", e);
                    null_result_response()
                }
            }
        }
        _ => {
            warn!("[GRIDUSER] Unknown method: {}", method);
            failure_xml(&format!("Unknown method: {}", method))
        }
    }
}

fn griduser_info_response(kvp: &HashMap<String, String>) -> axum::response::Response {
    let xml = single_result(kvp.clone());
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}

fn success_result_response() -> axum::response::Response {
    let xml = success_result();
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
