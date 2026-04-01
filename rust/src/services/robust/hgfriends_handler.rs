use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use tracing::{info, warn, debug};
use uuid::Uuid;

use super::RobustState;
use super::xml_response::*;

pub async fn handle_hgfriends(
    State(state): State<RobustState>,
    body: String,
) -> axum::response::Response {
    let params = parse_form_body(&body);
    let method = params.get("METHOD").or_else(|| params.get("method")).cloned().unwrap_or_default();

    debug!("[HG-FRIENDS] Request: METHOD={}", method);

    let svc = match &state.hg_friends_service {
        Some(svc) => svc,
        None => {
            warn!("[HG-FRIENDS] No HGFriends service configured");
            return xml_response("result", "Failure");
        }
    };

    match method.as_str() {
        "getfriendperms" => {
            let principal_id = params.get("PRINCIPALID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let friend_id = params.get("FRIENDID").cloned().unwrap_or_default();
            info!("[HG-FRIENDS] getfriendperms: {} -> {}", principal_id, friend_id);

            match svc.get_friend_perms(principal_id, &friend_id).await {
                Ok(perms) => xml_response("FriendPerms", &perms.to_string()),
                Err(e) => {
                    warn!("[HG-FRIENDS] getfriendperms error: {}", e);
                    xml_response("FriendPerms", "-1")
                }
            }
        }
        "newfriendship" => {
            let principal_id = params.get("PRINCIPALID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let friend_id = params.get("FRIENDID").cloned().unwrap_or_default();
            let secret = params.get("SECRET").cloned().unwrap_or_default();
            let verified = params.get("VERIFIED").map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false);
            info!("[HG-FRIENDS] newfriendship: {} -> {} (verified={})", principal_id, friend_id, verified);

            match svc.new_friendship(principal_id, &friend_id, &secret, verified).await {
                Ok(true) => xml_response("RESULT", "Success"),
                Ok(false) => xml_response("RESULT", "Failure"),
                Err(e) => {
                    warn!("[HG-FRIENDS] newfriendship error: {}", e);
                    xml_response("RESULT", "Failure")
                }
            }
        }
        "deletefriendship" => {
            let principal_id = params.get("PRINCIPALID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let friend_id = params.get("FRIENDID").cloned().unwrap_or_default();
            let secret = params.get("SECRET").cloned().unwrap_or_default();
            info!("[HG-FRIENDS] deletefriendship: {} -> {}", principal_id, friend_id);

            match svc.delete_friendship(principal_id, &friend_id, &secret).await {
                Ok(val) => xml_response("RESULT", if val { "true" } else { "false" }),
                Err(e) => {
                    warn!("[HG-FRIENDS] deletefriendship error: {}", e);
                    xml_response("RESULT", "false")
                }
            }
        }
        "validate_friendship_offered" => {
            let principal_id = params.get("PRINCIPALID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let friend_id = params.get("FRIENDID").cloned().unwrap_or_default();
            info!("[HG-FRIENDS] validate_friendship_offered: {} -> {}", principal_id, friend_id);

            match svc.validate_friendship_offered(principal_id, &friend_id).await {
                Ok(val) => xml_response("RESULT", if val { "true" } else { "false" }),
                Err(e) => {
                    warn!("[HG-FRIENDS] validate error: {}", e);
                    xml_response("RESULT", "false")
                }
            }
        }
        "statusnotification" => {
            let user_id_str = params.get("userID").or_else(|| params.get("USERID")).cloned().unwrap_or_default();
            let user_id = Uuid::parse_str(&user_id_str).unwrap_or_default();
            let online = params.get("online").or_else(|| params.get("ONLINE"))
                .map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false);

            let mut friends = Vec::new();
            for i in 0..100 {
                let key = format!("friend_{}", i);
                if let Some(f) = params.get(&key) {
                    friends.push(f.clone());
                } else {
                    break;
                }
            }

            info!("[HG-FRIENDS] statusnotification: user={} online={} friends={}", user_id, online, friends.len());

            match svc.status_notification(&friends, user_id, online).await {
                Ok(online_list) => {
                    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?><ServerResponse>");
                    for (i, f) in online_list.iter().enumerate() {
                        xml.push_str(&format!("<friend_{}>{}</friend_{}>", i, f, i));
                    }
                    xml.push_str("</ServerResponse>");
                    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
                }
                Err(e) => {
                    warn!("[HG-FRIENDS] statusnotification error: {}", e);
                    xml_response("result", "Failure")
                }
            }
        }
        _ => {
            warn!("[HG-FRIENDS] Unknown method: {}", method);
            xml_response("result", "Failure")
        }
    }
}

fn xml_response(key: &str, value: &str) -> axum::response::Response {
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\
         <ServerResponse><{key}>{value}</{key}></ServerResponse>"
    );
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}
