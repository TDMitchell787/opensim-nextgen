use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use std::collections::HashMap;
use tracing::{debug, warn};
use uuid::Uuid;

use super::RobustState;
use super::xml_response::*;
use crate::services::traits::PresenceInfo;

pub async fn handle_presence(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let params = parse_form_body(&body);
    let method = params.get("METHOD").or_else(|| params.get("method")).cloned().unwrap_or_default();

    debug!("Presence handler: METHOD={}", method);

    let xml = match method.as_str() {
        "login" => handle_login(&state, &params).await,
        "logout" => handle_logout(&state, &params).await,
        "report" => handle_report(&state, &params).await,
        "getagent" => handle_get_agent(&state, &params).await,
        "getagents" => handle_get_agents(&state, &params).await,
        _ => {
            warn!("Presence handler: unknown method '{}'", method);
            failure_result(&format!("Unknown method: {}", method))
        }
    };

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")], xml)
}

async fn handle_login(state: &RobustState, params: &HashMap<String, String>) -> String {
    let user_id = parse_uuid(params, "UserID");
    let session_id = parse_uuid(params, "SessionID");
    let secure_session_id = parse_uuid(params, "SecureSessionID");
    let region_id = parse_uuid(params, "RegionID");

    match state.presence_service.login_agent(user_id, session_id, secure_session_id, region_id).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Login agent returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_logout(state: &RobustState, params: &HashMap<String, String>) -> String {
    let session_id = parse_uuid(params, "SessionID");

    match state.presence_service.logout_agent(session_id).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Session not found"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_report(state: &RobustState, params: &HashMap<String, String>) -> String {
    let session_id = parse_uuid(params, "SessionID");
    let region_id = parse_uuid(params, "RegionID");

    match state.presence_service.report_agent(session_id, region_id).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Session not found"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_agent(state: &RobustState, params: &HashMap<String, String>) -> String {
    let session_id = parse_uuid(params, "SessionID");

    match state.presence_service.get_agent(session_id).await {
        Ok(Some(info)) => single_result(presence_to_fields(&info)),
        Ok(None) => null_result(),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_agents(state: &RobustState, params: &HashMap<String, String>) -> String {
    let user_ids: Vec<Uuid> = params.get("USERIDS")
        .or_else(|| params.get("uids"))
        .map(|s| s.split(',')
            .filter_map(|id| Uuid::parse_str(id.trim()).ok())
            .collect())
        .unwrap_or_default();

    match state.presence_service.get_agents(&user_ids).await {
        Ok(agents) => {
            let items: Vec<HashMap<String, String>> = agents.iter()
                .map(|a| presence_to_fields(a))
                .collect();
            list_result("presence", items)
        }
        Err(e) => failure_result(&format!("{}", e)),
    }
}

fn parse_uuid(params: &HashMap<String, String>, key: &str) -> Uuid {
    params.get(key)
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil())
}

fn presence_to_fields(info: &PresenceInfo) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("UserID".to_string(), info.user_id.to_string());
    m.insert("SessionID".to_string(), info.session_id.to_string());
    m.insert("SecureSessionID".to_string(), info.secure_session_id.to_string());
    m.insert("RegionID".to_string(), info.region_id.to_string());
    m.insert("Online".to_string(), if info.online { "True" } else { "False" }.to_string());
    m.insert("Login".to_string(), info.login_time.to_string());
    m.insert("Logout".to_string(), info.logout_time.to_string());
    m
}
