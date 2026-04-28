use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::xml_response::*;
use super::RobustState;

pub async fn handle_friends(
    State(state): State<RobustState>,
    body: String,
) -> axum::response::Response {
    let params = parse_form_body(&body);
    let method = params
        .get("METHOD")
        .or_else(|| params.get("method"))
        .cloned()
        .unwrap_or_default();

    debug!("[FRIENDS] Request: METHOD={}", method);

    let svc = match &state.friends_service {
        Some(svc) => svc,
        None => {
            warn!("[FRIENDS] No Friends service configured");
            return failure_xml("No Friends service");
        }
    };

    match method.to_lowercase().as_str() {
        "getfriends" => {
            let principal_id = params
                .get("PRINCIPALID")
                .or_else(|| params.get("PrincipalID"))
                .cloned()
                .unwrap_or_default();
            debug!("[FRIENDS] getfriends: {}", principal_id);

            match svc.get_friends(&principal_id).await {
                Ok(friends) => friends_list_response(&friends),
                Err(e) => {
                    warn!("[FRIENDS] getfriends error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "getfriends_string" => {
            let principal_id = params
                .get("PRINCIPALID")
                .or_else(|| params.get("PrincipalID"))
                .cloned()
                .unwrap_or_default();
            debug!("[FRIENDS] getfriends_string: {}", principal_id);

            match svc.get_friends(&principal_id).await {
                Ok(friends) => friends_list_response(&friends),
                Err(e) => {
                    warn!("[FRIENDS] getfriends_string error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "storefriend" => {
            let principal_id = params
                .get("PrincipalID")
                .or_else(|| params.get("PRINCIPALID"))
                .cloned()
                .unwrap_or_default();
            let friend = params
                .get("Friend")
                .or_else(|| params.get("FRIEND"))
                .cloned()
                .unwrap_or_default();
            let flags: i32 = params
                .get("MyFlags")
                .or_else(|| params.get("MYFLAGS"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            info!(
                "[FRIENDS] storefriend: {} -> {} flags={}",
                principal_id, friend, flags
            );

            match svc.store_friend(&principal_id, &friend, flags).await {
                Ok(true) => success_response(),
                Ok(false) => failure_xml("StoreFriend failed"),
                Err(e) => {
                    warn!("[FRIENDS] storefriend error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        "deletefriend" | "deletefriend_string" => {
            let principal_id = params
                .get("PRINCIPALID")
                .or_else(|| params.get("PrincipalID"))
                .cloned()
                .unwrap_or_default();
            let friend = params
                .get("FRIEND")
                .or_else(|| params.get("Friend"))
                .cloned()
                .unwrap_or_default();
            info!("[FRIENDS] deletefriend: {} -> {}", principal_id, friend);

            match svc.delete_friend(&principal_id, &friend).await {
                Ok(true) => success_response(),
                Ok(false) => failure_xml("DeleteFriend failed"),
                Err(e) => {
                    warn!("[FRIENDS] deletefriend error: {}", e);
                    failure_xml(&e.to_string())
                }
            }
        }
        _ => {
            warn!("[FRIENDS] Unknown method: {}", method);
            failure_xml(&format!("Unknown method: {}", method))
        }
    }
}

fn friends_list_response(
    friends: &[crate::services::traits::FriendInfo],
) -> axum::response::Response {
    use crate::services::traits::FriendInfo;
    if friends.is_empty() {
        return null_result_response();
    }
    let items: Vec<HashMap<String, String>> = friends
        .iter()
        .map(|f| {
            let mut m = HashMap::new();
            m.insert("PrincipalID".to_string(), f.principal_id.clone());
            m.insert("Friend".to_string(), f.friend.clone());
            m.insert("MyFlags".to_string(), f.my_flags.to_string());
            m.insert("TheirFlags".to_string(), f.their_flags.to_string());
            m
        })
        .collect();
    let xml = list_result("friend", items);
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}

fn success_response() -> axum::response::Response {
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
