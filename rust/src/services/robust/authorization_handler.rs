use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use tracing::{info, warn};
use uuid::Uuid;

use super::RobustState;

pub async fn handle_authorization(
    State(state): State<RobustState>,
    body: String,
) -> axum::response::Response {
    let svc = match &state.authorization_service {
        Some(svc) => svc,
        None => {
            warn!("[AUTHORIZATION] No authorization service configured");
            return authorization_response(true, "");
        }
    };

    let user_id;
    let first_name;
    let last_name;
    let region_id;

    if body.trim_start().starts_with('<') {
        let xml = super::xml_response::try_parse_xml_to_flat(&body).unwrap_or_default();
        user_id = xml.get("ID").cloned().unwrap_or_default();
        first_name = xml.get("FirstName").cloned().unwrap_or_default();
        last_name = xml.get("SurName").cloned().unwrap_or_default();
        region_id = xml.get("RegionID").cloned().unwrap_or_default();
    } else {
        let params = super::xml_response::parse_form_body(&body);
        user_id = params.get("ID").cloned().unwrap_or_default();
        first_name = params.get("FirstName").cloned().unwrap_or_default();
        last_name = params.get("SurName").cloned().unwrap_or_default();
        region_id = params.get("RegionID").cloned().unwrap_or_default();
    }

    info!("[AUTHORIZATION] check: user={} ({} {}), region={}", user_id, first_name, last_name, region_id);

    let uid = Uuid::parse_str(&user_id).unwrap_or_default();
    let rid = Uuid::parse_str(&region_id).unwrap_or_default();

    match svc.is_authorized_for_region(uid, &first_name, &last_name, rid).await {
        Ok((authorized, message)) => authorization_response(authorized, &message),
        Err(e) => {
            warn!("[AUTHORIZATION] Error: {}", e);
            authorization_response(false, &e.to_string())
        }
    }
}

fn authorization_response(authorized: bool, message: &str) -> axum::response::Response {
    let xml = format!(
        "<?xml version=\"1.0\"?>\
        <AuthorizationResponse>\
        <IsAuthorized>{}</IsAuthorized>\
        <Message>{}</Message>\
        </AuthorizationResponse>",
        authorized, message
    );
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], xml).into_response()
}
