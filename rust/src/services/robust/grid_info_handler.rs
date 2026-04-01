use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

use super::RobustState;

fn get_grid_info_fields() -> Vec<(&'static str, String)> {
    let port = std::env::var("OPENSIM_LOGIN_PORT").unwrap_or_else(|_| "9000".to_string());
    let home_uri = std::env::var("OPENSIM_HOME_URI")
        .unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));
    let gatekeeper_uri = std::env::var("OPENSIM_GATEKEEPER_URI")
        .unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));
    let grid_name = std::env::var("OPENSIM_GRID_NAME")
        .unwrap_or_else(|_| "OpenSim Next Grid".to_string());
    let grid_nick = std::env::var("OPENSIM_GRID_NICK")
        .unwrap_or_else(|_| "opensim-next".to_string());

    vec![
        ("gridname", grid_name),
        ("gridnick", grid_nick),
        ("login", format!("{}/", home_uri)),
        ("welcome", format!("{}/welcome", home_uri)),
        ("economy", format!("{}/", home_uri)),
        ("about", format!("{}/welcome", home_uri)),
        ("register", format!("{}/", home_uri)),
        ("help", format!("{}/", home_uri)),
        ("password", format!("{}/", home_uri)),
        ("gatekeeper", format!("{}/", gatekeeper_uri)),
        ("uas", format!("{}/", gatekeeper_uri)),
        ("platform", "OpenSim".to_string()),
        ("version", "2.1.0".to_string()),
    ]
}

pub async fn handle_grid_info_xml() -> impl IntoResponse {
    debug!("[GRID-INFO] XML grid info request");

    let fields = get_grid_info_fields();
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<gridinfo>\n");
    for (key, value) in &fields {
        xml.push_str(&format!("  <{key}>{value}</{key}>\n"));
    }
    xml.push_str("</gridinfo>");

    (StatusCode::OK, [(header::CONTENT_TYPE, "application/xml")], xml)
}

pub async fn handle_grid_info_json() -> impl IntoResponse {
    debug!("[GRID-INFO] JSON grid info request");

    let fields = get_grid_info_fields();
    let mut json = String::from("{");
    for (i, (key, value)) in fields.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
        json.push_str(&format!("\"{}\":\"{}\"", key, escaped_value));
    }
    json.push('}');

    (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], json)
}

static GRID_STATS_CACHE: Mutex<(i64, String)> = Mutex::new((0, String::new()));
const CACHE_TTL_SECS: i64 = 900;

pub async fn handle_grid_stats(
    State(state): State<RobustState>,
) -> impl IntoResponse {
    debug!("[GRID-INFO] Grid stats request");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    {
        let cache = GRID_STATS_CACHE.lock().unwrap();
        if cache.0 > 0 && (now - cache.0) < CACHE_TTL_SECS && !cache.1.is_empty() {
            debug!("[GRID-INFO] Returning cached grid stats (age={}s)", now - cache.0);
            return (StatusCode::OK, [(header::CONTENT_TYPE, "application/xml")], cache.1.clone());
        }
    }

    let (residents, active_users, region_count) = if let Some(pool) = state.db_pool.as_ref() {
        let thirty_days_ago = now - 2592000;

        let residents: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM griduser WHERE userid NOT LIKE '%;%'"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let active_users: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM griduser WHERE userid NOT LIKE '%;%' AND login::bigint > $1"
        )
        .bind(thirty_days_ago)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let region_count: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM((\"sizeX\" * \"sizeY\") >> 16), 0) FROM regions"
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        (residents, active_users, region_count)
    } else {
        info!("[GRID-INFO] No db_pool available for grid stats — returning zeros");
        (0i64, 0i64, 0i64)
    };

    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<gridstats>\n  <residents>{}</residents>\n  <active_users>{}</active_users>\n  <region_count>{}</region_count>\n</gridstats>",
        residents, active_users, region_count
    );

    {
        let mut cache = GRID_STATS_CACHE.lock().unwrap();
        *cache = (now, xml.clone());
    }

    (StatusCode::OK, [(header::CONTENT_TYPE, "application/xml")], xml)
}
