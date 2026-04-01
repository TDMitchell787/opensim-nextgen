use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::RobustState;
use super::xml_response::*;
use crate::services::traits::RegionInfo;

pub async fn handle_grid(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let params = parse_form_body(&body);
    let method = params.get("METHOD").or_else(|| params.get("method")).cloned().unwrap_or_default();

    debug!("Grid handler: METHOD={}", method);

    let xml = match method.as_str() {
        "register" => handle_register(&state, &params).await,
        "deregister" => handle_deregister(&state, &params).await,
        "get_region_by_uuid" => handle_get_region_by_uuid(&state, &params).await,
        "get_region_by_name" => handle_get_region_by_name(&state, &params).await,
        "get_region_by_position" => handle_get_region_by_position(&state, &params).await,
        "get_neighbours" => handle_get_neighbours(&state, &params).await,
        "get_default_regions" => handle_get_default_regions(&state, &params).await,
        "get_regions_by_flags" => handle_get_regions_by_flags(&state, &params).await,
        _ => {
            warn!("Grid handler: unknown method '{}'", method);
            failure_result(&format!("Unknown method: {}", method))
        }
    };

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")], xml)
}

async fn handle_register(state: &RobustState, params: &HashMap<String, String>) -> String {
    let region = params_to_region(params);
    info!("Grid: register region '{}' ({})", region.region_name, region.region_id);

    match state.grid_service.register_region(&region).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Registration returned false"),
        Err(e) => failure_result(&format!("Registration failed: {}", e)),
    }
}

async fn handle_deregister(state: &RobustState, params: &HashMap<String, String>) -> String {
    let region_id = parse_uuid(params, "REGIONID");
    info!("Grid: deregister region {}", region_id);

    match state.grid_service.deregister_region(region_id).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Region not found"),
        Err(e) => failure_result(&format!("Deregistration failed: {}", e)),
    }
}

async fn handle_get_region_by_uuid(state: &RobustState, params: &HashMap<String, String>) -> String {
    let scope_id = parse_uuid(params, "SCOPEID");
    let region_id = parse_uuid(params, "REGIONID");

    match state.grid_service.get_region_by_uuid(scope_id, region_id).await {
        Ok(Some(region)) => single_result(region_to_fields(&region)),
        Ok(None) => null_result(),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_region_by_name(state: &RobustState, params: &HashMap<String, String>) -> String {
    let scope_id = parse_uuid(params, "SCOPEID");
    let name = params.get("REGIONNAME").or_else(|| params.get("NAME")).cloned().unwrap_or_default();

    match state.grid_service.get_region_by_name(scope_id, &name).await {
        Ok(Some(region)) => single_result(region_to_fields(&region)),
        Ok(None) => null_result(),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_region_by_position(state: &RobustState, params: &HashMap<String, String>) -> String {
    let scope_id = parse_uuid(params, "SCOPEID");
    let x: u32 = params.get("X").and_then(|s| s.parse().ok()).unwrap_or(0);
    let y: u32 = params.get("Y").and_then(|s| s.parse().ok()).unwrap_or(0);

    match state.grid_service.get_region_by_position(scope_id, x, y).await {
        Ok(Some(region)) => single_result(region_to_fields(&region)),
        Ok(None) => null_result(),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_neighbours(state: &RobustState, params: &HashMap<String, String>) -> String {
    let scope_id = parse_uuid(params, "SCOPEID");
    let region_id = parse_uuid(params, "REGIONID");
    let range: u32 = params.get("RANGE").and_then(|s| s.parse().ok()).unwrap_or(1);

    match state.grid_service.get_neighbours(scope_id, region_id, range).await {
        Ok(regions) => regions_to_xml(&regions),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_default_regions(state: &RobustState, params: &HashMap<String, String>) -> String {
    let scope_id = parse_uuid(params, "SCOPEID");

    match state.grid_service.get_default_regions(scope_id).await {
        Ok(regions) => regions_to_xml(&regions),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_regions_by_flags(state: &RobustState, params: &HashMap<String, String>) -> String {
    let scope_id = parse_uuid(params, "SCOPEID");
    let flags: u32 = params.get("FLAGS").and_then(|s| s.parse().ok()).unwrap_or(0);

    match state.grid_service.get_regions(scope_id, flags).await {
        Ok(regions) => regions_to_xml(&regions),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

fn parse_uuid(params: &HashMap<String, String>, key: &str) -> Uuid {
    params.get(key)
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil())
}

fn params_to_region(params: &HashMap<String, String>) -> RegionInfo {
    RegionInfo {
        region_id: parse_uuid(params, "REGIONID"),
        region_name: params.get("REGIONNAME").cloned().unwrap_or_default(),
        region_loc_x: params.get("LOCX").and_then(|s| s.parse().ok()).unwrap_or(1000),
        region_loc_y: params.get("LOCY").and_then(|s| s.parse().ok()).unwrap_or(1000),
        region_size_x: params.get("SIZEX").and_then(|s| s.parse().ok()).unwrap_or(256),
        region_size_y: params.get("SIZEY").and_then(|s| s.parse().ok()).unwrap_or(256),
        server_ip: params.get("SERVERIP").cloned().unwrap_or_else(|| "127.0.0.1".to_string()),
        server_port: params.get("SERVERPORT").and_then(|s| s.parse().ok()).unwrap_or(9000),
        server_uri: params.get("SERVERURI").cloned().unwrap_or_default(),
        region_flags: params.get("FLAGS").and_then(|s| s.parse().ok()).unwrap_or(0),
        scope_id: parse_uuid(params, "SCOPEID"),
        owner_id: parse_uuid(params, "OWNERID"),
        estate_id: params.get("ESTATEID").and_then(|s| s.parse().ok()).unwrap_or(1),
    }
}

fn region_to_fields(region: &RegionInfo) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("uuid".to_string(), region.region_id.to_string());
    m.insert("regionName".to_string(), region.region_name.clone());
    m.insert("locX".to_string(), region.region_loc_x.to_string());
    m.insert("locY".to_string(), region.region_loc_y.to_string());
    m.insert("sizeX".to_string(), region.region_size_x.to_string());
    m.insert("sizeY".to_string(), region.region_size_y.to_string());
    m.insert("serverIP".to_string(), region.server_ip.clone());
    m.insert("serverPort".to_string(), region.server_port.to_string());
    m.insert("serverURI".to_string(), region.server_uri.clone());
    m.insert("flags".to_string(), region.region_flags.to_string());
    m.insert("scopeID".to_string(), region.scope_id.to_string());
    m.insert("owner_uuid".to_string(), region.owner_id.to_string());
    m.insert("regionEstateID".to_string(), region.estate_id.to_string());
    m
}

fn regions_to_xml(regions: &[RegionInfo]) -> String {
    let items: Vec<HashMap<String, String>> = regions.iter()
        .map(|r| region_to_fields(r))
        .collect();
    list_result("region", items)
}
