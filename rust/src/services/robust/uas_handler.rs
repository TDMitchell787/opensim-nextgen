use axum::{
    extract::State,
    response::IntoResponse,
    http::StatusCode,
};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use uuid::Uuid;

use super::{RobustState, UasState};
use super::xmlrpc::{
    XmlRpcValue, parse_xmlrpc_call, build_xmlrpc_response, build_xmlrpc_fault,
};
use crate::services::traits::AgentCircuitData;

pub async fn handle_useragent(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let uas = match &state.uas_service {
        Some(u) => u,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                [("Content-Type", "text/xml")],
                build_xmlrpc_fault(1, "UserAgent service not available"),
            );
        }
    };

    let (method, params) = match parse_xmlrpc_call(&body) {
        Ok(result) => result,
        Err(e) => {
            warn!("Failed to parse UAS XmlRpc request: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "text/xml")],
                build_xmlrpc_fault(2, &format!("Invalid XmlRpc: {}", e)),
            );
        }
    };

    let param = params.first().cloned().unwrap_or(XmlRpcValue::Struct(HashMap::new()));

    debug!("UAS XmlRpc method: {}", method);

    let response_xml = match method.as_str() {
        "verify_agent" => {
            let session_id = param.get_str("sessionID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let token = param.get_str("token").unwrap_or("");

            match uas.verify_agent(session_id, token).await {
                Ok(valid) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String(if valid { "True" } else { "False" }.to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("verify_agent error: {}", e)),
            }
        }

        "verify_client" => {
            let session_id = param.get_str("sessionID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let token = param.get_str("token").unwrap_or("");

            match uas.verify_client(session_id, token).await {
                Ok(valid) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String(if valid { "True" } else { "False" }.to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("verify_client error: {}", e)),
            }
        }

        "get_home_region" => {
            let user_id = param.get_str("userID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();

            match uas.get_home_region(user_id).await {
                Ok(Some((info, position, look_at))) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("True".to_string()));
                    result.insert("uuid".to_string(), XmlRpcValue::String(info.region_id.to_string()));
                    result.insert("handle".to_string(), XmlRpcValue::String(info.region_handle.to_string()));
                    result.insert("region_name".to_string(), XmlRpcValue::String(info.region_name));
                    result.insert("external_name".to_string(), XmlRpcValue::String(info.external_name));
                    result.insert("region_size_x".to_string(), XmlRpcValue::Int(info.size_x as i32));
                    result.insert("region_size_y".to_string(), XmlRpcValue::Int(info.size_y as i32));
                    result.insert("http_port".to_string(), XmlRpcValue::Int(info.http_port as i32));
                    result.insert("server_uri".to_string(), XmlRpcValue::String(info.server_uri));
                    result.insert("region_xloc".to_string(), XmlRpcValue::Int(info.region_loc_x as i32));
                    result.insert("region_yloc".to_string(), XmlRpcValue::Int(info.region_loc_y as i32));
                    result.insert("position".to_string(), XmlRpcValue::String(
                        format!("<{},{},{}>", position[0], position[1], position[2])
                    ));
                    result.insert("lookAt".to_string(), XmlRpcValue::String(
                        format!("<{},{},{}>", look_at[0], look_at[1], look_at[2])
                    ));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("False".to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("get_home_region error: {}", e)),
            }
        }

        "get_server_urls" => {
            let user_id = param.get_str("userID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();

            match uas.get_server_urls(user_id).await {
                Ok(urls) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("True".to_string()));
                    for (k, v) in urls {
                        result.insert(k, XmlRpcValue::String(v));
                    }
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("get_server_urls error: {}", e)),
            }
        }

        "logout_agent" => {
            let user_id = param.get_str("userID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let session_id = param.get_str("sessionID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();

            match uas.logout_agent(user_id, session_id).await {
                Ok(()) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("True".to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("logout_agent error: {}", e)),
            }
        }

        "get_uui" => {
            let user_id = param.get_str("userID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let target_id = param.get_str("targetUserID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();

            match uas.get_uui(user_id, target_id).await {
                Ok(uui) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("True".to_string()));
                    result.insert("UUI".to_string(), XmlRpcValue::String(uui));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("get_uui error: {}", e)),
            }
        }

        "get_uuid" => {
            let first = param.get_str("first").unwrap_or("");
            let last = param.get_str("last").unwrap_or("");

            match uas.get_uuid(first, last).await {
                Ok(Some(uuid)) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("True".to_string()));
                    result.insert("userID".to_string(), XmlRpcValue::String(uuid.to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("False".to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("get_uuid error: {}", e)),
            }
        }

        "status_notification" | "get_online_friends" => {
            let user_id = param.get_str("userID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let online = param.get_bool("online").unwrap_or(false);
            let friends: Vec<String> = param.get("friends")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            match uas.status_notification(&friends, user_id, online).await {
                Ok(online_friends) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("True".to_string()));
                    let arr: Vec<XmlRpcValue> = online_friends.iter()
                        .map(|u| XmlRpcValue::String(u.to_string()))
                        .collect();
                    result.insert("online_friends".to_string(), XmlRpcValue::Array(arr));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("status_notification error: {}", e)),
            }
        }

        "agent_is_coming_home" => {
            let session_id = param.get_str("sessionID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let external_name = param.get_str("externalName").unwrap_or("");

            match uas.is_agent_coming_home(session_id, external_name).await {
                Ok(coming_home) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String(if coming_home { "True" } else { "False" }.to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("agent_is_coming_home error: {}", e)),
            }
        }

        _ => {
            warn!("Unknown UAS method: {}", method);
            build_xmlrpc_fault(4, &format!("Unknown method: {}", method))
        }
    };

    (StatusCode::OK, [("Content-Type", "text/xml")], response_xml)
}

pub async fn handle_home_agent(
    State(state): State<RobustState>,
    axum::extract::Path(agent_id): axum::extract::Path<String>,
    body: String,
) -> impl IntoResponse {
    let uas = match &state.uas_service {
        Some(u) => u,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                [("Content-Type", "application/json")],
                r#"{"success": false, "reason": "UserAgent service not available"}"#.to_string(),
            );
        }
    };

    info!("Home agent request for: {}", agent_id);

    let json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "application/json")],
                format!(r#"{{"success": false, "reason": "Invalid JSON: {}"}}"#, e),
            );
        }
    };

    let agent_data = AgentCircuitData {
        agent_id: json.get("agent_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        session_id: json.get("session_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        secure_session_id: json.get("secure_session_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        circuit_code: json.get("circuit_code").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        first_name: json.get("first_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        last_name: json.get("last_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        service_urls: json.get("service_urls")
            .and_then(|v| v.as_object())
            .map(|m| m.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
            .unwrap_or_default(),
        service_session_id: json.get("service_session_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        start_pos: [128.0, 128.0, 21.0],
        appearance_serial: 0,
        client_ip: json.get("client_ip").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        mac: String::new(),
        id0: String::new(),
        teleport_flags: json.get("teleport_flags").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        caps_path: json.get("caps_path").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    };

    let gatekeeper = crate::services::traits::HGRegionInfo::default();

    let destination = crate::services::traits::HGRegionInfo {
        region_id: json.get("destination_uuid")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        region_name: json.get("destination_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        region_handle: json.get("destination_handle")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0),
        server_uri: json.get("destination_server_uri").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        ..Default::default()
    };

    let from_login = json.get("from_login").and_then(|v| v.as_bool()).unwrap_or(false);

    match uas.login_agent_to_grid(&agent_data, &gatekeeper, &destination, from_login).await {
        Ok((success, reason)) => {
            (
                StatusCode::OK,
                [("Content-Type", "application/json")],
                format!(r#"{{"success": {}, "reason": "{}"}}"#, success, reason),
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("Content-Type", "application/json")],
                format!(r#"{{"success": false, "reason": "{}"}}"#, e),
            )
        }
    }
}

pub async fn handle_useragent_standalone(
    State(state): State<UasState>,
    body: String,
) -> impl IntoResponse {
    let uas = &state.uas_service;

    let (method, params) = match parse_xmlrpc_call(&body) {
        Ok(result) => result,
        Err(e) => {
            warn!("Failed to parse UAS XmlRpc request: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "text/xml")],
                build_xmlrpc_fault(2, &format!("Invalid XmlRpc: {}", e)),
            );
        }
    };

    let param = params.first().cloned().unwrap_or(XmlRpcValue::Struct(HashMap::new()));

    debug!("[REGION UAS] XmlRpc method: {}", method);

    let response_xml = match method.as_str() {
        "verify_agent" => {
            let session_id = param.get_str("sessionID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let token = param.get_str("token").unwrap_or("");
            info!("[REGION UAS] verify_agent: session={}", session_id);

            match uas.verify_agent(session_id, token).await {
                Ok(valid) => {
                    info!("[REGION UAS] verify_agent result: {}", valid);
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String(if valid { "True" } else { "False" }.to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("verify_agent error: {}", e)),
            }
        }

        "verify_client" => {
            let session_id = param.get_str("sessionID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let token = param.get_str("token").unwrap_or("");

            match uas.verify_client(session_id, token).await {
                Ok(valid) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String(if valid { "True" } else { "False" }.to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("verify_client error: {}", e)),
            }
        }

        "get_home_region" => {
            let user_id = param.get_str("userID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();

            match uas.get_home_region(user_id).await {
                Ok(Some((region_info, position, look_at))) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("True".to_string()));
                    result.insert("uuid".to_string(), XmlRpcValue::String(region_info.region_id.to_string()));
                    result.insert("handle".to_string(), XmlRpcValue::String(region_info.region_handle.to_string()));
                    result.insert("region_name".to_string(), XmlRpcValue::String(region_info.region_name));
                    result.insert("external_name".to_string(), XmlRpcValue::String(region_info.external_name));
                    result.insert("region_size_x".to_string(), XmlRpcValue::Int(region_info.size_x as i32));
                    result.insert("region_size_y".to_string(), XmlRpcValue::Int(region_info.size_y as i32));
                    result.insert("http_port".to_string(), XmlRpcValue::Int(region_info.http_port as i32));
                    result.insert("server_uri".to_string(), XmlRpcValue::String(region_info.server_uri));
                    result.insert("position".to_string(), XmlRpcValue::String(
                        format!("<{},{},{}>", position[0], position[1], position[2])
                    ));
                    result.insert("lookAt".to_string(), XmlRpcValue::String(
                        format!("<{},{},{}>", look_at[0], look_at[1], look_at[2])
                    ));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("False".to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("get_home_region error: {}", e)),
            }
        }

        "get_server_urls" => {
            let user_id = param.get_str("userID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();

            match uas.get_server_urls(user_id).await {
                Ok(urls) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String("True".to_string()));
                    for (k, v) in urls {
                        result.insert(k, XmlRpcValue::String(v));
                    }
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("get_server_urls error: {}", e)),
            }
        }

        "agent_is_coming_home" => {
            let session_id = param.get_str("sessionID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let external_name = param.get_str("externalName").unwrap_or("");

            match uas.is_agent_coming_home(session_id, external_name).await {
                Ok(coming_home) => {
                    let mut result = HashMap::new();
                    result.insert("result".to_string(), XmlRpcValue::String(if coming_home { "True" } else { "False" }.to_string()));
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("agent_is_coming_home error: {}", e)),
            }
        }

        _ => {
            warn!("[REGION UAS] Unknown method: {}", method);
            build_xmlrpc_fault(4, &format!("Unknown method: {}", method))
        }
    };

    (StatusCode::OK, [("Content-Type", "text/xml")], response_xml)
}

pub async fn handle_home_agent_standalone(
    State(state): State<UasState>,
    axum::extract::Path(agent_id): axum::extract::Path<String>,
    body: String,
) -> impl IntoResponse {
    let uas = &state.uas_service;

    info!("[REGION UAS] Home agent request for: {}", agent_id);

    let json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "application/json")],
                format!(r#"{{"success": false, "reason": "Invalid JSON: {}"}}"#, e),
            );
        }
    };

    let agent_data = AgentCircuitData {
        agent_id: json.get("agent_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()).unwrap_or_default(),
        session_id: json.get("session_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()).unwrap_or_default(),
        secure_session_id: json.get("secure_session_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()).unwrap_or_default(),
        circuit_code: json.get("circuit_code").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        first_name: json.get("first_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        last_name: json.get("last_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        service_urls: json.get("service_urls")
            .and_then(|v| v.as_object())
            .map(|m| m.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
            .unwrap_or_default(),
        service_session_id: json.get("service_session_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        start_pos: [128.0, 128.0, 21.0],
        appearance_serial: 0,
        client_ip: json.get("client_ip").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        mac: String::new(),
        id0: String::new(),
        teleport_flags: json.get("teleport_flags").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        caps_path: json.get("caps_path").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    };

    let gatekeeper = crate::services::traits::HGRegionInfo::default();

    let destination = crate::services::traits::HGRegionInfo {
        region_id: json.get("destination_uuid").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()).unwrap_or_default(),
        region_name: json.get("destination_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        region_handle: json.get("destination_handle").and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0),
        server_uri: json.get("destination_server_uri").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        ..Default::default()
    };

    let from_login = json.get("from_login").and_then(|v| v.as_bool()).unwrap_or(false);

    match uas.login_agent_to_grid(&agent_data, &gatekeeper, &destination, from_login).await {
        Ok((success, reason)) => {
            (
                StatusCode::OK,
                [("Content-Type", "application/json")],
                format!(r#"{{"success": {}, "reason": "{}"}}"#, success, reason),
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("Content-Type", "application/json")],
                format!(r#"{{"success": false, "reason": "{}"}}"#, e),
            )
        }
    }
}
