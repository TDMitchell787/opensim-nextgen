use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use std::collections::HashMap;
use tracing::{debug, warn};

use super::RobustState;
use super::xml_response::*;
use crate::services::traits::EstateSettings;

pub async fn handle_estates_get(
    State(state): State<RobustState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    debug!("[ESTATE] GET /estates params={:?}", params);

    let svc = match &state.estate_service {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                failure_result("Estate service not configured")).into_response();
        }
    };

    let result = if let Some(name) = params.get("name") {
        svc.get_estates_by_name(name).await
    } else if let Some(owner) = params.get("owner") {
        svc.get_estates_by_owner(owner).await
    } else {
        svc.get_estates_all().await
    };

    match result {
        Ok(ids) if ids.is_empty() => {
            (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                null_result()).into_response()
        }
        Ok(ids) => {
            let items: Vec<HashMap<String, String>> = ids.iter().map(|id| {
                let mut m = HashMap::new();
                m.insert("EstateID".to_string(), id.to_string());
                m
            }).collect();
            (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                list_result("estate", items)).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                failure_result(&format!("{}", e))).into_response()
        }
    }
}

pub async fn handle_estate_get(
    State(state): State<RobustState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    debug!("[ESTATE] GET /estates/estate params={:?}", params);

    let svc = match &state.estate_service {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                failure_result("Estate service not configured")).into_response();
        }
    };

    let result = if let Some(region_id) = params.get("region") {
        let create = params.get("create")
            .map(|s| s.eq_ignore_ascii_case("true") || s == "1")
            .unwrap_or(false);
        svc.load_estate_by_region(region_id, create).await
    } else if let Some(eid_str) = params.get("eid") {
        let eid = eid_str.parse::<i32>().unwrap_or(0);
        svc.load_estate_by_id(eid).await
    } else {
        return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
            failure_result("Missing region or eid parameter")).into_response();
    };

    match result {
        Ok(Some(settings)) => {
            let fields = settings.to_map();
            (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                single_result(fields)).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                null_result()).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                failure_result(&format!("{}", e))).into_response()
        }
    }
}

pub async fn handle_estate_post(
    State(state): State<RobustState>,
    Query(query): Query<HashMap<String, String>>,
    body: String,
) -> impl IntoResponse {
    let params = parse_form_body(&body);
    let op = params.get("OP").or_else(|| params.get("op")).cloned().unwrap_or_default();

    debug!("[ESTATE] POST /estates/estate OP={} query={:?}", op, query);

    let svc = match &state.estate_service {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                failure_result("Estate service not configured"));
        }
    };

    let xml = match op.to_uppercase().as_str() {
        "STORE" => {
            let settings = parse_estate_from_params(&params);
            match svc.store_estate_settings(&settings).await {
                Ok(true) => {
                    let mut data = HashMap::new();
                    data.insert("Result".to_string(), XmlValue::Str("true".to_string()));
                    build_xml_response(&data)
                }
                Ok(false) => failure_result("Store failed"),
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "LINK" => {
            let eid = query.get("eid").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            let region = query.get("region").cloned().unwrap_or_default();

            if eid == 0 || region.is_empty() {
                failure_result("Missing eid or region")
            } else {
                match svc.link_region(&region, eid).await {
                    Ok(true) => {
                        let mut data = HashMap::new();
                        data.insert("Result".to_string(), XmlValue::Str("true".to_string()));
                        build_xml_response(&data)
                    }
                    Ok(false) => failure_result("Link failed"),
                    Err(e) => failure_result(&format!("{}", e)),
                }
            }
        }
        _ => {
            warn!("[ESTATE] Unknown OP: {}", op);
            failure_result(&format!("Unknown OP: {}", op))
        }
    };

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")], xml)
}

pub async fn handle_estate_regions(
    State(state): State<RobustState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    debug!("[ESTATE] GET /estates/regions params={:?}", params);

    let svc = match &state.estate_service {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                failure_result("Estate service not configured")).into_response();
        }
    };

    let eid = params.get("eid").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
    if eid == 0 {
        return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
            failure_result("Missing eid")).into_response();
    }

    match svc.get_regions(eid).await {
        Ok(regions) => {
            let items: Vec<HashMap<String, String>> = regions.iter().map(|r| {
                let mut m = HashMap::new();
                m.insert("RegionID".to_string(), r.clone());
                m
            }).collect();
            (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                list_result("region", items)).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
                failure_result(&format!("{}", e))).into_response()
        }
    }
}

fn parse_bool_param(params: &HashMap<String, String>, key: &str, default: bool) -> bool {
    params.get(key)
        .map(|s| s.eq_ignore_ascii_case("true") || s == "1")
        .unwrap_or(default)
}

fn parse_estate_from_params(params: &HashMap<String, String>) -> EstateSettings {
    EstateSettings {
        estate_id: params.get("EstateID").and_then(|s| s.parse().ok()).unwrap_or(0),
        estate_name: params.get("EstateName").cloned().unwrap_or_else(|| "My Estate".to_string()),
        estate_owner: params.get("EstateOwner").cloned()
            .unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string()),
        parent_estate_id: params.get("ParentEstateID").and_then(|s| s.parse().ok()).unwrap_or(0),
        abuse_email_to_estate_owner: parse_bool_param(params, "AbuseEmailToEstateOwner", true),
        deny_anonymous: parse_bool_param(params, "DenyAnonymous", false),
        reset_home_on_teleport: parse_bool_param(params, "ResetHomeOnTeleport", false),
        fixed_sun: parse_bool_param(params, "FixedSun", false),
        deny_transacted: parse_bool_param(params, "DenyTransacted", false),
        block_dwell: parse_bool_param(params, "BlockDwell", false),
        deny_identified: parse_bool_param(params, "DenyIdentified", false),
        allow_voice: parse_bool_param(params, "AllowVoice", true),
        use_global_time: parse_bool_param(params, "UseGlobalTime", true),
        price_per_meter: params.get("PricePerMeter").and_then(|s| s.parse().ok()).unwrap_or(1),
        tax_free: parse_bool_param(params, "TaxFree", false),
        allow_direct_teleport: parse_bool_param(params, "AllowDirectTeleport", true),
        redirect_grid_x: params.get("RedirectGridX").and_then(|s| s.parse().ok()).unwrap_or(0),
        redirect_grid_y: params.get("RedirectGridY").and_then(|s| s.parse().ok()).unwrap_or(0),
        sun_position: params.get("SunPosition").and_then(|s| s.parse().ok()).unwrap_or(0.0),
        estate_skip_scripts: parse_bool_param(params, "EstateSkipScripts", false),
        billable_factor: params.get("BillableFactor").and_then(|s| s.parse().ok()).unwrap_or(1.0),
        public_access: parse_bool_param(params, "PublicAccess", true),
        abuse_email: params.get("AbuseEmail").cloned().unwrap_or_default(),
        deny_minors: parse_bool_param(params, "DenyMinors", false),
        allow_landmark: parse_bool_param(params, "AllowLandmark", true),
        allow_parcel_changes: parse_bool_param(params, "AllowParcelChanges", true),
        allow_set_home: parse_bool_param(params, "AllowSetHome", true),
        allow_environment_override: parse_bool_param(params, "AllowEnvironmentOverride", false),
        estate_managers: params.get("EstateManagers")
            .map(|s| s.split(',').filter(|v| !v.is_empty()).map(|v| v.trim().to_string()).collect())
            .unwrap_or_default(),
        estate_users: params.get("EstateAccess")
            .map(|s| s.split(',').filter(|v| !v.is_empty()).map(|v| v.trim().to_string()).collect())
            .unwrap_or_default(),
        estate_groups: params.get("EstateGroups")
            .map(|s| s.split(',').filter(|v| !v.is_empty()).map(|v| v.trim().to_string()).collect())
            .unwrap_or_default(),
        estate_bans: Vec::new(),
    }
}
