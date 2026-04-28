use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::xml_response::*;
use super::RobustState;

pub async fn handle_land(
    State(state): State<RobustState>,
    body: String,
) -> axum::response::Response {
    let svc = match &state.land_service {
        Some(svc) => svc,
        None => {
            warn!("[LAND] No Land service configured");
            return failure_xml("No Land service");
        }
    };

    let xml = try_parse_xml_to_flat(&body);
    let (region_handle, x, y) = if let Some(ref xml_data) = xml {
        let rh: u64 = xml_data
            .get("region_handle")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let xv: u32 = xml_data
            .get("x")
            .and_then(|s| s.parse().ok())
            .unwrap_or(128);
        let yv: u32 = xml_data
            .get("y")
            .and_then(|s| s.parse().ok())
            .unwrap_or(128);
        (rh, xv, yv)
    } else {
        let params = parse_form_body(&body);
        let rh: u64 = params
            .get("region_handle")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let xv: u32 = params.get("x").and_then(|s| s.parse().ok()).unwrap_or(128);
        let yv: u32 = params.get("y").and_then(|s| s.parse().ok()).unwrap_or(128);
        (rh, xv, yv)
    };

    debug!(
        "[LAND] GetLandData: handle={} x={} y={}",
        region_handle, x, y
    );

    match svc.get_land_data(Uuid::nil(), region_handle, x, y).await {
        Ok(Some(land)) => {
            let mut resp = HashMap::new();
            resp.insert("Name".to_string(), land.name);
            resp.insert("Description".to_string(), land.description);
            resp.insert("OwnerID".to_string(), land.owner_id.to_string());
            resp.insert("GlobalID".to_string(), land.global_id.to_string());
            resp.insert("Area".to_string(), land.area.to_string());
            resp.insert("Flags".to_string(), land.flags.to_string());
            resp.insert("SalePrice".to_string(), land.sale_price.to_string());
            resp.insert("SnapshotID".to_string(), land.snapshot_id.to_string());
            resp.insert(
                "UserLocation".to_string(),
                format!("<{},{},{}>", land.landing_x, land.landing_y, land.landing_z),
            );
            resp.insert("RegionAccess".to_string(), "21".to_string());
            resp.insert("Dwell".to_string(), land.dwell.to_string());
            resp.insert("AuctionID".to_string(), "0".to_string());

            let xml_resp = single_result(resp);
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/xml")],
                xml_resp,
            )
                .into_response()
        }
        Ok(None) => {
            let xml_resp = null_result();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/xml")],
                xml_resp,
            )
                .into_response()
        }
        Err(e) => {
            warn!("[LAND] Error: {}", e);
            failure_xml(&e.to_string())
        }
    }
}

fn failure_xml(msg: &str) -> axum::response::Response {
    let xml = failure_result(msg);
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}
