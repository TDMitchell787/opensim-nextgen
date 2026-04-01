use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use std::collections::HashMap;
use tracing::{debug, warn};
use uuid::Uuid;

use super::RobustState;
use super::xml_response::*;
use crate::services::traits::UserAccount;

pub async fn handle_user_account(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let params = parse_form_body(&body);
    let method = params.get("METHOD").or_else(|| params.get("method")).cloned().unwrap_or_default();

    debug!("UserAccount handler: METHOD={}", method);

    let xml = match method.as_str() {
        "getaccount" => handle_get_account(&state, &params).await,
        "getaccounts" => handle_get_accounts(&state, &params).await,
        "setaccount" | "storeaccount" => handle_set_account(&state, &params).await,
        _ => {
            warn!("UserAccount handler: unknown method '{}'", method);
            failure_result(&format!("Unknown method: {}", method))
        }
    };

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")], xml)
}

async fn handle_get_account(state: &RobustState, params: &HashMap<String, String>) -> String {
    let scope_id = params.get("SCOPEID")
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());

    let result = if let Some(user_id) = params.get("UserID").and_then(|s| Uuid::parse_str(s).ok()) {
        state.user_account_service.get_user_account(scope_id, user_id).await
    } else if let (Some(first), Some(last)) = (params.get("FirstName"), params.get("LastName")) {
        state.user_account_service.get_user_account_by_name(scope_id, first, last).await
    } else if let Some(email) = params.get("Email") {
        state.user_account_service.get_user_account_by_email(scope_id, email).await
    } else {
        return failure_result("Missing UserID, FirstName/LastName, or Email");
    };

    match result {
        Ok(Some(account)) => single_result(account_to_fields(&account)),
        Ok(None) => null_result(),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_accounts(state: &RobustState, params: &HashMap<String, String>) -> String {
    let scope_id = params.get("SCOPEID")
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());
    let query = params.get("query").or_else(|| params.get("QUERY")).cloned().unwrap_or_default();

    match state.user_account_service.get_user_accounts(scope_id, &query).await {
        Ok(accounts) => {
            let items: Vec<HashMap<String, String>> = accounts.iter()
                .map(|a| account_to_fields(a))
                .collect();
            list_result("account", items)
        }
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_set_account(state: &RobustState, params: &HashMap<String, String>) -> String {
    let account = UserAccount {
        principal_id: params.get("PrincipalID")
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or(Uuid::nil()),
        scope_id: params.get("ScopeID")
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or(Uuid::nil()),
        first_name: params.get("FirstName").cloned().unwrap_or_default(),
        last_name: params.get("LastName").cloned().unwrap_or_default(),
        email: params.get("Email").cloned().unwrap_or_default(),
        service_urls: HashMap::new(),
        created: params.get("Created").and_then(|s| s.parse().ok()).unwrap_or(0),
        user_level: params.get("UserLevel").and_then(|s| s.parse().ok()).unwrap_or(0),
        user_flags: params.get("UserFlags").and_then(|s| s.parse().ok()).unwrap_or(0),
        user_title: params.get("UserTitle").cloned().unwrap_or_default(),
    };

    match state.user_account_service.store_user_account(&account).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Store returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

fn account_to_fields(account: &UserAccount) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("PrincipalID".to_string(), account.principal_id.to_string());
    m.insert("ScopeID".to_string(), account.scope_id.to_string());
    m.insert("FirstName".to_string(), account.first_name.clone());
    m.insert("LastName".to_string(), account.last_name.clone());
    m.insert("Email".to_string(), account.email.clone());
    m.insert("Created".to_string(), account.created.to_string());
    m.insert("UserLevel".to_string(), account.user_level.to_string());
    m.insert("UserFlags".to_string(), account.user_flags.to_string());
    m.insert("UserTitle".to_string(), account.user_title.clone());
    let urls: String = account.service_urls.iter()
        .map(|(k, v)| format!("{};{}", k, v))
        .collect::<Vec<_>>()
        .join(";");
    m.insert("ServiceURLs".to_string(), urls);
    m
}
