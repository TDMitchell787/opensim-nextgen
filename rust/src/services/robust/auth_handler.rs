use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::collections::HashMap;
use tracing::{debug, warn};
use uuid::Uuid;

use super::xml_response::*;
use super::RobustState;

pub async fn handle_auth(State(state): State<RobustState>, body: String) -> impl IntoResponse {
    let params = parse_form_body(&body);
    let method = params
        .get("METHOD")
        .or_else(|| params.get("method"))
        .cloned()
        .unwrap_or_default();

    debug!("Auth handler: METHOD={}", method);

    let xml = match method.as_str() {
        "authenticate" => handle_authenticate(&state, &params).await,
        "verify" => handle_verify(&state, &params).await,
        "release" => handle_release(&state, &params).await,
        "setpassword" => handle_set_password(&state, &params).await,
        "getauthinfo" | "getauthentication" => handle_get_auth_info(&state, &params).await,
        _ => {
            warn!("Auth handler: unknown method '{}'", method);
            failure_result(&format!("Unknown method: {}", method))
        }
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
        xml,
    )
}

async fn handle_authenticate(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = params
        .get("PRINCIPALID")
        .or_else(|| params.get("PrincipalID"))
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());
    let password = params.get("PASSWORD").cloned().unwrap_or_default();
    let lifetime: i32 = params
        .get("LIFETIME")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    match state
        .auth_service
        .authenticate(principal_id, &password, lifetime)
        .await
    {
        Ok(Some(token)) => {
            let mut data = HashMap::new();
            data.insert("Result".to_string(), "Success".to_string());
            data.insert("Token".to_string(), token);
            single_result(data)
        }
        Ok(None) => failure_result("Invalid credentials"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_verify(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = params
        .get("PRINCIPALID")
        .or_else(|| params.get("PrincipalID"))
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());
    let token = params.get("TOKEN").cloned().unwrap_or_default();
    let lifetime: i32 = params
        .get("LIFETIME")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    match state
        .auth_service
        .verify(principal_id, &token, lifetime)
        .await
    {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Token invalid or expired"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_release(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = params
        .get("PRINCIPALID")
        .or_else(|| params.get("PrincipalID"))
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());
    let token = params.get("TOKEN").cloned().unwrap_or_default();

    match state.auth_service.release(principal_id, &token).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Token not found"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_set_password(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = params
        .get("PRINCIPALID")
        .or_else(|| params.get("PrincipalID"))
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());
    let password = params.get("PASSWORD").cloned().unwrap_or_default();

    match state
        .auth_service
        .set_password(principal_id, &password)
        .await
    {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Failed to set password"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_auth_info(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = params
        .get("PRINCIPALID")
        .or_else(|| params.get("PrincipalID"))
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());

    match state.auth_service.get_authentication(principal_id).await {
        Ok(Some(info)) => {
            let mut fields = HashMap::new();
            fields.insert("PrincipalID".to_string(), info.principal_id.to_string());
            fields.insert("passwordHash".to_string(), info.password_hash);
            fields.insert("passwordSalt".to_string(), info.password_salt);
            fields.insert("webLoginKey".to_string(), info.web_login_key);
            fields.insert("accountType".to_string(), info.account_type);
            single_result(fields)
        }
        Ok(None) => null_result(),
        Err(e) => failure_result(&format!("{}", e)),
    }
}
