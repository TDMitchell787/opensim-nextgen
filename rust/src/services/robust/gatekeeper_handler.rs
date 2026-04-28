use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::xmlrpc::{build_xmlrpc_fault, build_xmlrpc_response, parse_xmlrpc_call, XmlRpcValue};
use super::{GatekeeperState, RobustState};
use crate::services::traits::AgentCircuitData;

pub async fn handle_gatekeeper(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let gatekeeper = match &state.gatekeeper_service {
        Some(gk) => gk,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                [("Content-Type", "text/xml")],
                build_xmlrpc_fault(1, "Gatekeeper service not available"),
            );
        }
    };

    let (method, params) = match parse_xmlrpc_call(&body) {
        Ok(result) => result,
        Err(e) => {
            warn!("Failed to parse gatekeeper XmlRpc request: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "text/xml")],
                build_xmlrpc_fault(2, &format!("Invalid XmlRpc: {}", e)),
            );
        }
    };

    let param = params
        .first()
        .cloned()
        .unwrap_or(XmlRpcValue::Struct(HashMap::new()));

    debug!("Gatekeeper XmlRpc method: {}", method);

    let response_xml = match method.as_str() {
        "link_region" => {
            let region_name = param.get_str("region_name").unwrap_or("");
            match gatekeeper.link_region(region_name).await {
                Ok(Some(info)) => {
                    info!(
                        "[HG GK] link_region '{}' -> {} (handle={})",
                        region_name, info.region_id, info.region_handle
                    );
                    let mut result = HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("True".to_string()),
                    );
                    result.insert(
                        "uuid".to_string(),
                        XmlRpcValue::String(info.region_id.to_string()),
                    );
                    result.insert(
                        "handle".to_string(),
                        XmlRpcValue::String(info.region_handle.to_string()),
                    );
                    result.insert(
                        "size_x".to_string(),
                        XmlRpcValue::String(info.size_x.to_string()),
                    );
                    result.insert(
                        "size_y".to_string(),
                        XmlRpcValue::String(info.size_y.to_string()),
                    );
                    result.insert(
                        "region_image".to_string(),
                        XmlRpcValue::String(info.image_url.clone()),
                    );
                    result.insert(
                        "external_name".to_string(),
                        XmlRpcValue::String(info.external_name.clone()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    warn!("[HG GK] link_region '{}' -> not found", region_name);
                    let mut result = HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("False".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("Internal error: {}", e)),
            }
        }

        "get_region" => {
            let region_id = param
                .get_str("region_uuid")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let agent_id = param
                .get_str("agent_id")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let agent_home = param.get_str("agent_home_uri").unwrap_or("");

            match gatekeeper
                .get_hyperlinkregion(region_id, agent_id, agent_home)
                .await
            {
                Ok(Some(info)) => {
                    info!(
                        "[HG GK] get_region {} -> {} ({})",
                        region_id, info.region_name, info.server_uri
                    );
                    let mut result = HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("True".to_string()),
                    );
                    result.insert(
                        "uuid".to_string(),
                        XmlRpcValue::String(info.region_id.to_string()),
                    );
                    result.insert(
                        "x".to_string(),
                        XmlRpcValue::String(info.region_loc_x.to_string()),
                    );
                    result.insert(
                        "y".to_string(),
                        XmlRpcValue::String(info.region_loc_y.to_string()),
                    );
                    result.insert(
                        "size_x".to_string(),
                        XmlRpcValue::String(info.size_x.to_string()),
                    );
                    result.insert(
                        "size_y".to_string(),
                        XmlRpcValue::String(info.size_y.to_string()),
                    );
                    result.insert(
                        "region_name".to_string(),
                        XmlRpcValue::String(info.region_name.clone()),
                    );
                    result.insert(
                        "hostname".to_string(),
                        XmlRpcValue::String(info.hostname.clone()),
                    );
                    result.insert(
                        "http_port".to_string(),
                        XmlRpcValue::String(info.http_port.to_string()),
                    );
                    result.insert(
                        "internal_port".to_string(),
                        XmlRpcValue::String(info.internal_port.to_string()),
                    );
                    result.insert(
                        "server_uri".to_string(),
                        XmlRpcValue::String(info.server_uri.clone()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    warn!("[HG GK] get_region {} -> not found", region_id);
                    let mut result = HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("False".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("Internal error: {}", e)),
            }
        }

        _ => {
            warn!("Unknown gatekeeper method: {}", method);
            build_xmlrpc_fault(4, &format!("Unknown method: {}", method))
        }
    };

    (StatusCode::OK, [("Content-Type", "text/xml")], response_xml)
}

pub async fn handle_foreign_agent(
    State(state): State<RobustState>,
    axum::extract::Path(agent_id): axum::extract::Path<String>,
    body: String,
) -> impl IntoResponse {
    let gatekeeper = match &state.gatekeeper_service {
        Some(gk) => gk,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                [("Content-Type", "application/json")],
                r#"{"success": false, "reason": "Gatekeeper not available"}"#.to_string(),
            );
        }
    };

    info!("Foreign agent request for: {}", agent_id);

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
        agent_id: json
            .get("agent_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        session_id: json
            .get("session_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        secure_session_id: json
            .get("secure_session_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        circuit_code: json
            .get("circuit_code")
            .and_then(|v| {
                v.as_u64()
                    .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            })
            .unwrap_or(0) as u32,
        first_name: json
            .get("first_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        last_name: json
            .get("last_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        service_urls: {
            if let Some(obj) = json.get("serviceurls").and_then(|v| v.as_object()) {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            } else if let Some(obj) = json.get("service_urls").and_then(|v| v.as_object()) {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            } else if let Some(arr) = json.get("service_urls").and_then(|v| v.as_array()) {
                let mut map = std::collections::HashMap::new();
                let strs: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                for pair in strs.chunks(2) {
                    if pair.len() == 2 {
                        map.insert(pair[0].to_string(), pair[1].to_string());
                    }
                }
                map
            } else {
                std::collections::HashMap::new()
            }
        },
        service_session_id: json
            .get("service_session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        start_pos: [128.0, 128.0, 21.0],
        appearance_serial: 0,
        client_ip: json
            .get("client_ip")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        mac: json
            .get("mac")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        id0: json
            .get("id0")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        teleport_flags: json
            .get("teleport_flags")
            .and_then(|v| {
                v.as_u64()
                    .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            })
            .unwrap_or(0) as u32,
        caps_path: json
            .get("caps_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };

    let source = crate::services::traits::HGRegionInfo {
        server_uri: json
            .get("source_server_uri")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ..Default::default()
    };

    let destination = crate::services::traits::HGRegionInfo {
        region_id: json
            .get("destination_uuid")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        region_name: json
            .get("destination_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        region_handle: json
            .get("destination_handle")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0),
        server_uri: json
            .get("destination_server_uri")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ..Default::default()
    };

    match gatekeeper
        .login_agent(&source, &agent_data, &destination)
        .await
    {
        Ok((success, reason)) => {
            let json_resp = format!(r#"{{"success": {}, "reason": "{}"}}"#, success, reason);
            (
                StatusCode::OK,
                [("Content-Type", "application/json")],
                json_resp,
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [("Content-Type", "application/json")],
            format!(r#"{{"success": false, "reason": "{}"}}"#, e),
        ),
    }
}

pub async fn handle_gatekeeper_standalone(
    State(state): State<GatekeeperState>,
    body: String,
) -> impl IntoResponse {
    let gatekeeper = &state.gatekeeper_service;

    let (method, params) = match parse_xmlrpc_call(&body) {
        Ok(result) => result,
        Err(e) => {
            warn!("Failed to parse gatekeeper XmlRpc request: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "text/xml")],
                build_xmlrpc_fault(2, &format!("Invalid XmlRpc: {}", e)),
            );
        }
    };

    let param = params
        .first()
        .cloned()
        .unwrap_or(XmlRpcValue::Struct(HashMap::new()));

    info!("[GATEKEEPER] Inbound XmlRpc method: {}", method);

    let response_xml = match method.as_str() {
        "link_region" => {
            let region_name = param.get_str("region_name").unwrap_or("");
            match gatekeeper.link_region(region_name).await {
                Ok(Some(info)) => {
                    info!(
                        "[HG GK] link_region '{}' -> {} (handle={})",
                        region_name, info.region_id, info.region_handle
                    );
                    let mut result = HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("True".to_string()),
                    );
                    result.insert(
                        "uuid".to_string(),
                        XmlRpcValue::String(info.region_id.to_string()),
                    );
                    result.insert(
                        "handle".to_string(),
                        XmlRpcValue::String(info.region_handle.to_string()),
                    );
                    result.insert(
                        "size_x".to_string(),
                        XmlRpcValue::String(info.size_x.to_string()),
                    );
                    result.insert(
                        "size_y".to_string(),
                        XmlRpcValue::String(info.size_y.to_string()),
                    );
                    result.insert(
                        "region_image".to_string(),
                        XmlRpcValue::String(info.image_url.clone()),
                    );
                    result.insert(
                        "external_name".to_string(),
                        XmlRpcValue::String(info.external_name.clone()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    warn!("[HG GK] link_region '{}' -> not found", region_name);
                    let mut result = HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("False".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("Internal error: {}", e)),
            }
        }

        "get_region" => {
            let region_id = param
                .get_str("region_uuid")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let agent_id = param
                .get_str("agent_id")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default();
            let agent_home = param.get_str("agent_home_uri").unwrap_or("");

            match gatekeeper
                .get_hyperlinkregion(region_id, agent_id, agent_home)
                .await
            {
                Ok(Some(info)) => {
                    info!(
                        "[HG GK] get_region {} -> {} ({})",
                        region_id, info.region_name, info.server_uri
                    );
                    let mut result = HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("True".to_string()),
                    );
                    result.insert(
                        "uuid".to_string(),
                        XmlRpcValue::String(info.region_id.to_string()),
                    );
                    result.insert(
                        "x".to_string(),
                        XmlRpcValue::String(info.region_loc_x.to_string()),
                    );
                    result.insert(
                        "y".to_string(),
                        XmlRpcValue::String(info.region_loc_y.to_string()),
                    );
                    result.insert(
                        "size_x".to_string(),
                        XmlRpcValue::String(info.size_x.to_string()),
                    );
                    result.insert(
                        "size_y".to_string(),
                        XmlRpcValue::String(info.size_y.to_string()),
                    );
                    result.insert(
                        "region_name".to_string(),
                        XmlRpcValue::String(info.region_name.clone()),
                    );
                    result.insert(
                        "hostname".to_string(),
                        XmlRpcValue::String(info.hostname.clone()),
                    );
                    result.insert(
                        "http_port".to_string(),
                        XmlRpcValue::String(info.http_port.to_string()),
                    );
                    result.insert(
                        "internal_port".to_string(),
                        XmlRpcValue::String(info.internal_port.to_string()),
                    );
                    result.insert(
                        "server_uri".to_string(),
                        XmlRpcValue::String(info.server_uri.clone()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Ok(None) => {
                    warn!("[HG GK] get_region {} -> not found", region_id);
                    let mut result = HashMap::new();
                    result.insert(
                        "result".to_string(),
                        XmlRpcValue::String("False".to_string()),
                    );
                    build_xmlrpc_response(&XmlRpcValue::Struct(result))
                }
                Err(e) => build_xmlrpc_fault(3, &format!("Internal error: {}", e)),
            }
        }

        _ => {
            warn!("[GATEKEEPER] Unknown inbound method: {}", method);
            build_xmlrpc_fault(4, &format!("Unknown method: {}", method))
        }
    };

    (StatusCode::OK, [("Content-Type", "text/xml")], response_xml)
}

pub async fn handle_foreign_agent_standalone(
    State(state): State<GatekeeperState>,
    axum::extract::Path(agent_id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    raw_body: axum::body::Bytes,
) -> impl IntoResponse {
    let gatekeeper = &state.gatekeeper_service;

    info!(
        "[GATEKEEPER] Inbound foreign agent request for: {} (body_len={}, content-encoding={:?})",
        agent_id,
        raw_body.len(),
        headers
            .get("content-encoding")
            .map(|v| v.to_str().unwrap_or("?"))
    );

    let body_str = if headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("gzip"))
        .unwrap_or(false)
        || (raw_body.len() >= 2 && raw_body[0] == 0x1f && raw_body[1] == 0x8b)
    {
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(&raw_body[..]);
        let mut decompressed = String::new();
        match decoder.read_to_string(&mut decompressed) {
            Ok(_) => {
                info!(
                    "[GATEKEEPER] Decompressed gzip body: {} -> {} bytes",
                    raw_body.len(),
                    decompressed.len()
                );
                decompressed
            }
            Err(e) => {
                warn!(
                    "[GATEKEEPER] Gzip decompression failed: {}, treating as plain text",
                    e
                );
                String::from_utf8_lossy(&raw_body).to_string()
            }
        }
    } else {
        String::from_utf8_lossy(&raw_body).to_string()
    };

    debug!(
        "[GATEKEEPER] foreignagent body (first 500 chars): {}",
        &body_str[..std::cmp::min(body_str.len(), 500)]
    );

    let json: serde_json::Value = match serde_json::from_str(&body_str) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "[GATEKEEPER] Failed to parse foreignagent JSON: {} (body_len={})",
                e,
                body_str.len()
            );
            return (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "application/json")],
                format!(r#"{{"success": false, "reason": "Invalid JSON: {}"}}"#, e),
            );
        }
    };

    let agent_data = AgentCircuitData {
        agent_id: json
            .get("agent_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        session_id: json
            .get("session_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        secure_session_id: json
            .get("secure_session_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        circuit_code: json
            .get("circuit_code")
            .and_then(|v| {
                v.as_u64()
                    .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            })
            .unwrap_or(0) as u32,
        first_name: json
            .get("first_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        last_name: json
            .get("last_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        service_urls: {
            if let Some(obj) = json.get("serviceurls").and_then(|v| v.as_object()) {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            } else if let Some(obj) = json.get("service_urls").and_then(|v| v.as_object()) {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            } else if let Some(arr) = json.get("service_urls").and_then(|v| v.as_array()) {
                let mut map = std::collections::HashMap::new();
                let strs: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                for pair in strs.chunks(2) {
                    if pair.len() == 2 {
                        map.insert(pair[0].to_string(), pair[1].to_string());
                    }
                }
                map
            } else {
                std::collections::HashMap::new()
            }
        },
        service_session_id: json
            .get("service_session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        start_pos: [128.0, 128.0, 21.0],
        appearance_serial: 0,
        client_ip: json
            .get("client_ip")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        mac: json
            .get("mac")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        id0: json
            .get("id0")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        teleport_flags: json
            .get("teleport_flags")
            .and_then(|v| {
                v.as_u64()
                    .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            })
            .unwrap_or(0) as u32,
        caps_path: json
            .get("caps_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };

    let source = crate::services::traits::HGRegionInfo {
        server_uri: json
            .get("source_server_uri")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ..Default::default()
    };

    let destination = crate::services::traits::HGRegionInfo {
        region_id: json
            .get("destination_uuid")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
        region_name: json
            .get("destination_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        region_handle: json
            .get("destination_handle")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0),
        server_uri: json
            .get("destination_server_uri")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ..Default::default()
    };

    match gatekeeper
        .login_agent(&source, &agent_data, &destination)
        .await
    {
        Ok((success, reason)) => {
            if success {
                if let Some(ref registry) = state.circuit_code_registry {
                    let session = crate::login_session::LoginSession {
                        circuit_code: agent_data.circuit_code,
                        session_id: agent_data.session_id.to_string(),
                        agent_id: agent_data.agent_id.to_string(),
                        first_name: agent_data.first_name.clone(),
                        last_name: agent_data.last_name.clone(),
                        created_at: std::time::Instant::now(),
                        is_xmlrpc_session: true,
                    };
                    registry.register_login(session).await;
                    info!("[GATEKEEPER] Registered circuit_code {} in CircuitCodeRegistry for HG agent {} {}",
                        agent_data.circuit_code, agent_data.first_name, agent_data.last_name);
                }
                if let Some(ref sm) = state.session_manager {
                    sm.register_external_session(
                        agent_data.agent_id,
                        agent_data.session_id,
                        agent_data.secure_session_id,
                        agent_data.circuit_code,
                        agent_data.first_name.clone(),
                        agent_data.last_name.clone(),
                        "0.0.0.0".to_string(),
                        0,
                    );
                    info!("[GATEKEEPER] Registered circuit_code {} in SessionManager for HG agent {} {} ({})",
                        agent_data.circuit_code, agent_data.first_name, agent_data.last_name, agent_data.agent_id);
                }
                let mut seed_cap = String::new();
                if let Some(ref caps) = state.caps_manager {
                    let caps_session_id = caps
                        .create_session(
                            agent_data.agent_id.to_string(),
                            agent_data.circuit_code,
                            None,
                        )
                        .await;
                    seed_cap = format!("{}/cap/{}", caps.base_url, caps_session_id);
                    info!("[GATEKEEPER] Created CAPS session {} for HG agent {} {} (circuit={}, seed={})",
                        caps_session_id, agent_data.first_name, agent_data.last_name, agent_data.circuit_code, seed_cap);
                }
                let json_resp = if !seed_cap.is_empty() {
                    format!(
                        r#"{{"success": {}, "reason": "{}", "your_ip": "0.0.0.0", "region_caps": "{}"}}"#,
                        success, reason, seed_cap
                    )
                } else {
                    format!(r#"{{"success": {}, "reason": "{}"}}"#, success, reason)
                };
                return (
                    StatusCode::OK,
                    [("Content-Type", "application/json")],
                    json_resp,
                );
            }
            let json_resp = format!(r#"{{"success": {}, "reason": "{}"}}"#, success, reason);
            (
                StatusCode::OK,
                [("Content-Type", "application/json")],
                json_resp,
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [("Content-Type", "application/json")],
            format!(r#"{{"success": false, "reason": "{}"}}"#, e),
        ),
    }
}

pub async fn handle_agent_simulation(
    axum::extract::Path((agent_id, region_id)): axum::extract::Path<(String, String)>,
    req: axum::http::Request<axum::body::Body>,
) -> impl IntoResponse {
    let method = req.method().as_str().to_uppercase();
    info!("[AGENT SIM] {} /agent/{}/{}", method, agent_id, region_id);

    match method.as_str() {
        "POST" => {
            info!(
                "[AGENT SIM] CreateAgent POST for agent {} region {}",
                agent_id, region_id
            );
            let resp =
                r#"{"success":true,"reason":"","your_ip":"127.0.0.1","version":"SIMULATION/0.3"}"#;
            (
                StatusCode::OK,
                [("Content-Type", "application/json")],
                resp.to_string(),
            )
        }
        "QUERYACCESS" => {
            info!(
                "[AGENT SIM] QUERYACCESS for agent {} region {} — returning success",
                agent_id, region_id
            );
            let resp = r#"{"success":true,"reason":"","version":"SIMULATION/0.3","negotiated_inbound_version":0.3,"negotiated_outbound_version":0.3}"#;
            (
                StatusCode::OK,
                [("Content-Type", "application/json")],
                resp.to_string(),
            )
        }
        "PUT" => {
            info!(
                "[AGENT SIM] UpdateAgent PUT for agent {} region {}",
                agent_id, region_id
            );
            let resp = r#"{"success":true,"reason":""}"#;
            (
                StatusCode::OK,
                [("Content-Type", "application/json")],
                resp.to_string(),
            )
        }
        "DELETE" => {
            info!(
                "[AGENT SIM] Agent departure DELETE for agent {} region {}",
                agent_id, region_id
            );
            let resp = r#"{"success":true,"reason":""}"#;
            (
                StatusCode::OK,
                [("Content-Type", "application/json")],
                resp.to_string(),
            )
        }
        _ => {
            warn!(
                "[AGENT SIM] Unexpected method {} for /agent/{}/{}",
                method, agent_id, region_id
            );
            let resp = r#"{"success":false,"reason":"method not supported"}"#;
            (
                StatusCode::METHOD_NOT_ALLOWED,
                [("Content-Type", "application/json")],
                resp.to_string(),
            )
        }
    }
}

pub async fn handle_object_simulation(
    axum::extract::Path((object_id, region_id)): axum::extract::Path<(String, String)>,
    req: axum::http::Request<axum::body::Body>,
) -> impl IntoResponse {
    let method = req.method().as_str().to_uppercase();
    info!(
        "[OBJECT SIM] {} /object/{}/{}",
        method, object_id, region_id
    );
    let resp = r#"{"success":true,"reason":""}"#;
    (
        StatusCode::OK,
        [("Content-Type", "application/json")],
        resp.to_string(),
    )
}

pub async fn handle_agent_update(
    axum::extract::Path(agent_id): axum::extract::Path<String>,
    req: axum::http::Request<axum::body::Body>,
) -> impl IntoResponse {
    let method = req.method().as_str().to_uppercase();
    info!("[AGENT UPDATE] {} /agent/{}/", method, agent_id);
    let resp = r#"{"success":true,"reason":""}"#;
    (
        StatusCode::OK,
        [("Content-Type", "application/json")],
        resp.to_string(),
    )
}
