use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{error, info, warn};

use super::{CapsHandlerState, PendingUpload};

// Helper function to convert capabilities HashMap to proper LLSD XML format
// This follows the Second Life LLSD XML specification:
// <llsd><map><key>CapName</key><string>url</string>...</map></llsd>
fn capabilities_to_llsd_xml(
    capabilities: &HashMap<String, String>,
) -> Result<Response, StatusCode> {
    // Build XML manually to ensure correct format
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<llsd><map>");

    for (key, value) in capabilities {
        xml.push_str(&format!("<key>{}</key><string>{}</string>", key, value));
    }

    xml.push_str("</map></llsd>");

    // Debug: Log the first 500 chars of the XML response
    info!(
        "🔗 [DEBUG] LLSD XML Response (first 500 chars): {}",
        xml.chars().take(500).collect::<String>()
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(xml.into())
        .unwrap())
}

fn is_uuid_string(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    let chars: Vec<char> = s.chars().collect();
    if chars[8] != '-' || chars[13] != '-' || chars[18] != '-' || chars[23] != '-' {
        return false;
    }
    for (i, c) in chars.iter().enumerate() {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            continue;
        }
        if !c.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn get_asset_content_type(asset_type: i32) -> &'static str {
    match asset_type {
        0 => "image/x-j2c",                    // Texture
        1 => "audio/x-wav",                    // Sound
        2 => "application/vnd.ll.callingcard", // CallingCard
        3 => "application/vnd.ll.landmark",    // Landmark
        5 => "application/vnd.ll.clothing",    // Clothing
        6 => "application/vnd.ll.primitive",   // Object
        7 => "text/plain",                     // Notecard
        10 => "text/plain",                    // LSLText
        12 => "image/tga",                     // TextureTGA
        13 => "application/vnd.ll.bodypart",   // Bodypart
        14 => "image/tga",                     // ImageTGA
        15 => "image/jpeg",                    // ImageJPEG
        16 => "application/x-lsl-bytecode",    // LSLBytecode
        17 => "audio/x-wav",                   // SoundWAV
        20 => "application/vnd.ll.animation",  // Animation
        21 => "application/vnd.ll.gesture",    // Gesture
        49 => "application/vnd.ll.mesh",       // Mesh
        56 => "application/llsd+xml",          // Settings
        57 => "application/llsd+xml",          // Material
        _ => "application/octet-stream",
    }
}

fn xml_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn json_to_llsd_xml(value: &Value) -> String {
    use base64::Engine;
    match value {
        Value::Null => "<undef />".to_string(),
        Value::Bool(b) => format!("<boolean>{}</boolean>", if *b { "1" } else { "0" }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    format!("<integer>{}</integer>", i)
                } else {
                    let bytes = (i as u64).to_be_bytes();
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    format!("<binary encoding=\"base64\">{}</binary>", b64)
                }
            } else if let Some(u) = n.as_u64() {
                if u <= i32::MAX as u64 {
                    format!("<integer>{}</integer>", u)
                } else {
                    let bytes = u.to_be_bytes();
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    format!("<binary encoding=\"base64\">{}</binary>", b64)
                }
            } else {
                format!("<real>{}</real>", n)
            }
        }
        Value::String(s) => {
            if s.starts_with("b64:") {
                format!("<binary encoding=\"base64\">{}</binary>", &s[4..])
            } else if is_uuid_string(s) {
                format!("<uuid>{}</uuid>", s)
            } else {
                format!("<string>{}</string>", xml_escape(s))
            }
        }
        Value::Array(arr) => {
            let mut xml = String::from("<array>");
            for item in arr {
                xml.push_str(&json_to_llsd_xml(item));
            }
            xml.push_str("</array>");
            xml
        }
        Value::Object(map) => {
            let mut xml = String::from("<map>");
            for (key, val) in map {
                xml.push_str(&format!("<key>{}</key>", xml_escape(key)));
                xml.push_str(&json_to_llsd_xml(val));
            }
            xml.push_str("</map>");
            xml
        }
    }
}

// Helper function to wrap JSON response in proper LLSD XML envelope
pub fn json_response_to_llsd_xml(json_value: Value) -> Result<Response, StatusCode> {
    let llsd_body = json_to_llsd_xml(&json_value);
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<llsd>{}</llsd>",
        llsd_body
    );

    // Debug: Log LLSD XML for EventQueue responses (contains Handle)
    if xml.contains("EnableSimulator") || xml.contains("TeleportFinish") {
        info!(
            "📡 [DEBUG] EventQueue LLSD XML (first 800 chars): {}",
            xml.chars().take(800).collect::<String>()
        );
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(xml.into())
        .unwrap())
}

// Seed capability handler (GET) - returns all available capabilities for a session
pub async fn handle_seed_capability(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("🔗 GET Seed capability request for session: {}", session_id);

    // Update session activity
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    // Get capabilities for this session
    match state.caps_manager.get_capabilities(&session_id).await {
        Some(capabilities) => {
            info!(
                "🔗 Returning {} capabilities for session: {} (LLSD XML)",
                capabilities.len(),
                session_id
            );
            capabilities_to_llsd_xml(&capabilities)
        }
        None => {
            warn!("🔗 Session not found: {}", session_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

// Seed capability handler (POST) - viewer sends list of requested capabilities
pub async fn handle_seed_capability_post(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "🔗 POST Seed capability request for session: {}",
        session_id
    );

    // Parse the LLSD XML to get requested capabilities
    let body_str = String::from_utf8_lossy(&body);
    info!(
        "🔗 Viewer requested capabilities (first 200 chars): {}",
        body_str.chars().take(200).collect::<String>()
    );

    // Update session activity
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    // Get capabilities for this session
    match state.caps_manager.get_capabilities(&session_id).await {
        Some(capabilities) => {
            info!(
                "🔗 Returning {} capabilities for POST request, session: {} (LLSD XML)",
                capabilities.len(),
                session_id
            );
            info!("🔗 CAPABILITIES BEING RETURNED:");
            for (name, url) in capabilities.iter() {
                info!("🔗   {} -> {}", name, url);
            }

            if let Some((ready, circuit_code, agent_id)) =
                state.caps_manager.mark_sent_seeds(&session_id).await
            {
                info!(
                    "🌱 [Phase 80] SentSeeds flag set for session {} (OpenSim SentSeeds timing)",
                    session_id
                );
                if ready {
                    info!("🚀 [Phase 80] Both SentSeeds + RegionHandshakeReply received! Ready to send initial data after 500ms delay");
                    info!(
                        "🚀 [Phase 80] circuit_code={}, agent_id={}",
                        circuit_code, agent_id
                    );
                }
            }

            capabilities_to_llsd_xml(&capabilities)
        }
        None => {
            warn!("🔗 Session not found: {}", session_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

// GetTexture handler - serves texture assets
pub async fn handle_get_texture(
    Path(session_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    use crate::opensim_compatibility::library_assets::get_global_library_manager;

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let texture_id = match params.get("texture_id") {
        Some(id_str) => match uuid::Uuid::parse_str(id_str) {
            Ok(uuid) => uuid,
            Err(_) => {
                warn!("GetTexture: Invalid texture UUID: {}", id_str);
                return Err(StatusCode::BAD_REQUEST);
            }
        },
        None => {
            warn!("GetTexture: Missing texture_id parameter");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let fetch_result = if let Some(ref fetcher) = state.asset_fetcher {
        fetcher
            .fetch_asset_data_typed_pg(&texture_id.to_string(), Some(0), &state.db_pool)
            .await
    } else {
        let row: Result<Option<Vec<u8>>, sqlx::Error> =
            sqlx::query_scalar("SELECT data FROM assets WHERE id = $1 AND assettype = 0")
                .bind(texture_id)
                .fetch_optional(&*state.db_pool)
                .await;
        row.map_err(|e| anyhow::anyhow!("{}", e))
    };

    match fetch_result {
        Ok(Some(data)) if !data.is_empty() => {
            let j2c = crate::asset::jpeg2000::ensure_j2c_codestream(&data);
            let j2c_len = j2c.len();
            let serving_data = if j2c_len != data.len() {
                info!(
                    "[GETTEXTURE] Serving {} ({} bytes, extracted J2C from {} bytes)",
                    texture_id,
                    j2c_len,
                    data.len()
                );
                j2c.to_vec()
            } else {
                info!("[GETTEXTURE] Serving {} ({} bytes)", texture_id, data.len());
                data
            };
            return Ok(Response::builder()
                .status(200)
                .header("Content-Type", "image/x-j2c")
                .body(axum::body::Body::from(serving_data))
                .unwrap());
        }
        Ok(_) => {}
        Err(e) => {
            warn!("GetTexture: fetch error for {}: {}", texture_id, e);
        }
    }

    if let Some(library_manager) = get_global_library_manager() {
        let manager = library_manager.read().await;
        if let Some(data) = manager.get_asset_data(&texture_id) {
            let j2c = crate::asset::jpeg2000::ensure_j2c_codestream(&data);
            let serving_data = if j2c.len() != data.len() {
                j2c.to_vec()
            } else {
                data
            };
            info!(
                "[GETTEXTURE] Serving {} from library ({} bytes)",
                texture_id,
                serving_data.len()
            );
            return Ok(Response::builder()
                .status(200)
                .header("Content-Type", "image/x-j2c")
                .body(axum::body::Body::from(serving_data))
                .unwrap());
        }
    }

    if let Some(data) = state.caps_manager.get_baked_texture(&texture_id) {
        let j2c = crate::asset::jpeg2000::ensure_j2c_codestream(&data);
        let serving_data = if j2c.len() != data.len() {
            j2c.to_vec()
        } else {
            data
        };
        info!(
            "[GETTEXTURE] Serving {} from baked cache ({} bytes)",
            texture_id,
            serving_data.len()
        );
        return Ok(Response::builder()
            .status(200)
            .header("Content-Type", "image/x-j2c")
            .body(axum::body::Body::from(serving_data))
            .unwrap());
    }

    warn!("[GETTEXTURE] 404 NOT FOUND: {}", texture_id);
    Err(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
pub struct InventoryQuery {
    pub items: Option<String>,
}

// FetchInventory2 handler - returns inventory items by ID from DATABASE
pub async fn handle_fetch_inventory2(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    use sqlx::Row;
    use uuid::Uuid;

    let body_str = String::from_utf8_lossy(&body);
    info!(
        "📦 FetchInventory2 request for session: {}, body: {}",
        session_id, body_str
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let session = state.caps_manager.get_session(&session_id).await;
    let agent_id = session
        .map(|s| s.agent_id.clone())
        .unwrap_or_else(|| session_id.clone());

    let item_ids = parse_fetch_inventory2_items(&body_str);
    info!(
        "📦 FetchInventory2: parsed {} item_ids: {:?}",
        item_ids.len(),
        item_ids
    );

    let null_uuid = "00000000-0000-0000-0000-000000000000";
    let mut items_response: Vec<Value> = Vec::new();

    for item_id_str in &item_ids {
        let item_uuid = match Uuid::parse_str(item_id_str) {
            Ok(u) => u,
            Err(_) => {
                warn!("📦 FetchInventory2: invalid item UUID: {}", item_id_str);
                continue;
            }
        };

        let item_result = sqlx::query(
            r#"SELECT inventoryid, parentfolderid, assetid, inventoryname, inventorydescription,
                assettype, invtype, flags, creatorid, avatarid,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
                groupid, saleprice, saletype, creationdate
            FROM inventoryitems WHERE inventoryid = $1"#,
        )
        .bind(item_uuid)
        .fetch_optional(state.db_pool.as_ref())
        .await;

        match item_result {
            Ok(Some(row)) => {
                let item_id: Uuid = row.try_get("inventoryid").unwrap_or(Uuid::nil());
                let parent_id: Uuid = row.try_get("parentfolderid").unwrap_or(Uuid::nil());
                let asset_id: Uuid = row.try_get("assetid").unwrap_or(Uuid::nil());
                let name: String = row.try_get("inventoryname").unwrap_or_default();
                let desc: String = row.try_get("inventorydescription").unwrap_or_default();
                let asset_type: i32 = row
                    .try_get::<i64, _>("assettype")
                    .map(|v| v as i32)
                    .or_else(|_| row.try_get::<i32, _>("assettype"))
                    .unwrap_or(0);
                let inv_type: i32 = row
                    .try_get::<i64, _>("invtype")
                    .map(|v| v as i32)
                    .or_else(|_| row.try_get::<i32, _>("invtype"))
                    .unwrap_or(0);
                let flags: i32 = row
                    .try_get::<i64, _>("flags")
                    .map(|v| v as i32)
                    .or_else(|_| row.try_get::<i32, _>("flags"))
                    .unwrap_or(0);
                let creator_id: String = row.try_get("creatorid").unwrap_or_default();
                let avatar_id: Uuid = row.try_get("avatarid").unwrap_or(Uuid::nil());
                let base_perms: i32 = row.try_get("inventorybasepermissions").unwrap_or(0);
                let curr_perms: i32 = row.try_get("inventorycurrentpermissions").unwrap_or(0);
                let everyone_perms: i32 = row.try_get("inventoryeveryonepermissions").unwrap_or(0);
                let group_perms: i32 = row.try_get("inventorygrouppermissions").unwrap_or(0);
                let next_perms: i32 = row.try_get("inventorynextpermissions").unwrap_or(0);
                let group_id: Option<Uuid> = row.try_get("groupid").ok();
                let sale_price: i32 = row.try_get("saleprice").unwrap_or(0);
                let sale_type: i32 = row.try_get("saletype").unwrap_or(0);
                let creation_date: i32 = row.try_get("creationdate").unwrap_or(0);

                info!(
                    "📦 FetchInventory2: found item {} ({}) from DATABASE",
                    name, item_id
                );
                items_response.push(json!({
                    "parent_id": parent_id.to_string(),
                    "asset_id": asset_id.to_string(),
                    "item_id": item_id.to_string(),
                    "permissions": {
                        "creator_id": creator_id,
                        "owner_id": avatar_id.to_string(),
                        "group_id": group_id.map(|u| u.to_string()).unwrap_or_else(|| null_uuid.to_string()),
                        "base_mask": base_perms,
                        "owner_mask": curr_perms,
                        "group_mask": group_perms,
                        "everyone_mask": everyone_perms,
                        "next_owner_mask": next_perms,
                        "is_owner_group": false
                    },
                    "type": asset_type,
                    "inv_type": inv_type,
                    "flags": flags,
                    "sale_info": {
                        "sale_price": sale_price,
                        "sale_type": sale_type
                    },
                    "name": name,
                    "desc": desc,
                    "created_at": creation_date
                }));
            }
            Ok(None) => {
                warn!(
                    "📦 FetchInventory2: item {} not found in DATABASE",
                    item_id_str
                );
            }
            Err(e) => {
                warn!(
                    "📦 FetchInventory2: database error for item {}: {}",
                    item_id_str, e
                );
            }
        }
    }

    let response = json!({
        "agent_id": agent_id,
        "items": items_response
    });

    info!(
        "📦 FetchInventory2: returning {} items from DATABASE for session: {}",
        items_response.len(),
        session_id
    );
    json_response_to_llsd_xml(response)
}

// Parse item_ids from FetchInventory2 request body
fn parse_fetch_inventory2_items(body: &str) -> Vec<String> {
    let mut item_ids = Vec::new();

    // Parse item_id from LLSD XML - can be <uuid> or <string>
    let item_regex =
        regex::Regex::new(r"<key>item_id</key>\s*<(?:uuid|string)>([^<]+)</(?:uuid|string)>").ok();

    if let Some(re) = item_regex {
        for cap in re.captures_iter(body) {
            if let Some(m) = cap.get(1) {
                item_ids.push(m.as_str().to_string());
            }
        }
    }

    item_ids
}

// FetchInventoryDescendents2 handler - returns inventory folder contents from DATABASE
pub async fn handle_fetch_inventory_descendents2(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    use sqlx::Row;
    use uuid::Uuid;

    info!(
        "📁 FetchInventoryDescendents2 request for session: {}",
        session_id
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let session = state.caps_manager.get_session(&session_id).await;
    let agent_id_str = session
        .map(|s| s.agent_id.clone())
        .unwrap_or_else(|| session_id.clone());

    let agent_uuid = Uuid::parse_str(&agent_id_str).unwrap_or(Uuid::nil());

    let body_str = String::from_utf8_lossy(&body);
    info!("📁 Request body: {}", body_str);

    let folder_requests = parse_fetch_inventory_request(&body_str);
    info!("📁 Parsed {} folder requests", folder_requests.len());

    let mut folder_responses = Vec::new();

    for req in folder_requests {
        let mut folder_id_str = req.folder_id.clone();
        info!("📁 Processing folder request: {}", folder_id_str);

        let null_uuid = "00000000-0000-0000-0000-000000000000";
        if folder_id_str == null_uuid {
            let root_result = sqlx::query(
                "SELECT folderid FROM inventoryfolders WHERE agentid = $1 AND type = 8 LIMIT 1",
            )
            .bind(agent_uuid)
            .fetch_optional(state.db_pool.as_ref())
            .await;

            if let Ok(Some(row)) = root_result {
                let root_id: Uuid = row.try_get("folderid").unwrap_or(Uuid::nil());
                folder_id_str = root_id.to_string();
                info!(
                    "📁 Null UUID request - using root folder from DB: {}",
                    folder_id_str
                );
            }
        }

        let folder_uuid = Uuid::parse_str(&folder_id_str).unwrap_or(Uuid::nil());

        let folder_version: i32 =
            sqlx::query_scalar("SELECT version FROM inventoryfolders WHERE folderid = $1")
                .bind(folder_uuid)
                .fetch_optional(state.db_pool.as_ref())
                .await
                .unwrap_or(None)
                .unwrap_or(1);

        let subfolders_result = sqlx::query(
            "SELECT folderid, parentfolderid, foldername, type, version FROM inventoryfolders WHERE parentfolderid = $1"
        )
        .bind(folder_uuid)
        .fetch_all(state.db_pool.as_ref())
        .await;

        let categories: Vec<Value> = match subfolders_result {
            Ok(rows) => {
                let mut cats: Vec<(i64, String, Value)> = rows.iter().map(|row| {
                    let folder_id: Uuid = row.try_get("folderid").unwrap_or(Uuid::nil());
                    let parent_id: Option<Uuid> = row.try_get("parentfolderid").ok();
                    let name: String = row.try_get("foldername").unwrap_or_default();
                    let folder_type: i64 = row.try_get::<i64, _>("type")
                        .or_else(|_| row.try_get::<i32, _>("type").map(|v| v as i64))
                        .unwrap_or(0);
                    let version: i64 = row.try_get::<i64, _>("version")
                        .or_else(|_| row.try_get::<i32, _>("version").map(|v| v as i64))
                        .unwrap_or(1);

                    let val = json!({
                        "category_id": folder_id.to_string(),
                        "parent_id": parent_id.map(|u| u.to_string()).unwrap_or_else(|| null_uuid.to_string()),
                        "name": name,
                        "type_default": folder_type,
                        "version": version
                    });
                    (folder_type, name, val)
                }).collect();

                cats.sort_by(|a, b| {
                    let a_system = a.0 >= 0;
                    let b_system = b.0 >= 0;
                    let a_hash = a.1.starts_with('#');
                    let b_hash = b.1.starts_with('#');
                    let a_group = if a_system {
                        0
                    } else if a_hash {
                        2
                    } else {
                        1
                    };
                    let b_group = if b_system {
                        0
                    } else if b_hash {
                        2
                    } else {
                        1
                    };
                    a_group
                        .cmp(&b_group)
                        .then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase()))
                });

                cats.into_iter().map(|(_, _, v)| v).collect()
            }
            Err(e) => {
                warn!("📁 Failed to query subfolders: {}", e);
                Vec::new()
            }
        };

        let items_result = sqlx::query(
            r#"SELECT inventoryid, parentfolderid, assetid, inventoryname, inventorydescription,
                assettype, invtype, flags, creatorid, avatarid,
                inventorybasepermissions, inventorycurrentpermissions,
                inventoryeveryonepermissions, inventorygrouppermissions, inventorynextpermissions,
                groupid, saleprice, saletype, creationdate
            FROM inventoryitems WHERE parentfolderid = $1"#,
        )
        .bind(folder_uuid)
        .fetch_all(state.db_pool.as_ref())
        .await;

        let items: Vec<Value> = match items_result {
            Ok(rows) => rows.iter().map(|row| {
                let item_id: Uuid = row.try_get("inventoryid").unwrap_or(Uuid::nil());
                let parent_id: Uuid = row.try_get("parentfolderid").unwrap_or(Uuid::nil());
                let asset_id: Uuid = row.try_get("assetid").unwrap_or(Uuid::nil());
                let name: String = row.try_get("inventoryname").unwrap_or_default();
                let desc: String = row.try_get("inventorydescription").unwrap_or_default();
                let asset_type: i32 = row.try_get::<i64, _>("assettype")
                    .map(|v| v as i32)
                    .or_else(|_| row.try_get::<i32, _>("assettype"))
                    .unwrap_or(0);
                let inv_type: i32 = row.try_get::<i64, _>("invtype")
                    .map(|v| v as i32)
                    .or_else(|_| row.try_get::<i32, _>("invtype"))
                    .unwrap_or(0);
                let flags: i32 = row.try_get::<i64, _>("flags")
                    .map(|v| v as i32)
                    .or_else(|_| row.try_get::<i32, _>("flags"))
                    .unwrap_or(0);
                let creator_id: String = row.try_get("creatorid").unwrap_or_default();
                let avatar_id: Uuid = row.try_get("avatarid").unwrap_or(Uuid::nil());
                let base_perms: i32 = row.try_get("inventorybasepermissions").unwrap_or(0);
                let curr_perms: i32 = row.try_get("inventorycurrentpermissions").unwrap_or(0);
                let everyone_perms: i32 = row.try_get("inventoryeveryonepermissions").unwrap_or(0);
                let group_perms: i32 = row.try_get("inventorygrouppermissions").unwrap_or(0);
                let next_perms: i32 = row.try_get("inventorynextpermissions").unwrap_or(0);
                let group_id: Option<Uuid> = row.try_get("groupid").ok();
                let sale_price: i32 = row.try_get("saleprice").unwrap_or(0);
                let sale_type: i32 = row.try_get("saletype").unwrap_or(0);
                let creation_date: i32 = row.try_get("creationdate").unwrap_or(0);

                json!({
                    "parent_id": parent_id.to_string(),
                    "asset_id": asset_id.to_string(),
                    "item_id": item_id.to_string(),
                    "permissions": {
                        "creator_id": creator_id,
                        "owner_id": avatar_id.to_string(),
                        "group_id": group_id.map(|u| u.to_string()).unwrap_or_else(|| null_uuid.to_string()),
                        "base_mask": base_perms,
                        "owner_mask": curr_perms,
                        "group_mask": group_perms,
                        "everyone_mask": everyone_perms,
                        "next_owner_mask": next_perms,
                        "is_owner_group": false
                    },
                    "type": asset_type,
                    "inv_type": inv_type,
                    "flags": flags,
                    "sale_info": {
                        "sale_price": sale_price,
                        "sale_type": sale_type
                    },
                    "name": name,
                    "desc": desc,
                    "created_at": creation_date
                })
            }).collect(),
            Err(e) => {
                warn!("📁 Failed to query items: {}", e);
                Vec::new()
            }
        };

        let descendents = items.len() + categories.len();

        info!(
            "📁 Found {} items and {} subfolders in folder {} from DATABASE",
            items.len(),
            categories.len(),
            folder_id_str
        );

        folder_responses.push(json!({
            "folder_id": folder_id_str,
            "agent_id": agent_id_str,
            "owner_id": req.owner_id,
            "version": folder_version,
            "descendents": descendents,
            "categories": categories,
            "items": items
        }));
    }

    let response = json!({
        "folders": folder_responses
    });

    info!(
        "📁 Returning inventory contents from DATABASE for session: {} ({} folders)",
        session_id,
        folder_responses.len()
    );
    json_response_to_llsd_xml(response)
}

// Helper structures for inventory data
#[derive(Debug, Clone)]
struct FolderRequest {
    folder_id: String,
    owner_id: String,
}

#[derive(Debug, Clone)]
struct InventoryFolderData {
    id: String,
    parent_id: Option<String>,
    name: String,
    folder_type: i32,
    version: i32,
}

#[derive(Debug, Clone)]
struct InventoryItemData {
    id: String,
    folder_id: String,
    asset_id: String,
    name: String,
    description: String,
    asset_type: i32,
    inv_type: i32,
    flags: u32,
    creator_id: String,
    owner_id: String,
}

struct UserInventoryData {
    folders: Vec<InventoryFolderData>,
    items: Vec<InventoryItemData>,
}

fn parse_fetch_inventory_request(body: &str) -> Vec<FolderRequest> {
    let mut requests = Vec::new();

    // Parse folder_id and owner_id from LLSD XML
    // folder_id can be either <uuid> or <string> tag
    // Format: <key>folder_id</key><uuid>...</uuid> OR <key>folder_id</key><string>...</string>
    let folder_regex =
        regex::Regex::new(r"<key>folder_id</key>\s*<(?:uuid|string)>([^<]+)</(?:uuid|string)>")
            .ok();
    let owner_regex =
        regex::Regex::new(r"<key>owner_id</key>\s*<(?:uuid|string)>([^<]+)</(?:uuid|string)>").ok();

    if let (Some(folder_re), Some(owner_re)) = (folder_regex, owner_regex) {
        // Find all folder blocks
        let folder_ids: Vec<&str> = folder_re
            .captures_iter(body)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
            .collect();

        let owner_ids: Vec<&str> = owner_re
            .captures_iter(body)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
            .collect();

        info!(
            "📁 Parsed folder_ids: {:?}, owner_ids: {:?}",
            folder_ids, owner_ids
        );

        for (i, folder_id) in folder_ids.iter().enumerate() {
            let owner_id = owner_ids
                .get(i)
                .unwrap_or(&"00000000-0000-0000-0000-000000000000");
            requests.push(FolderRequest {
                folder_id: folder_id.to_string(),
                owner_id: owner_id.to_string(),
            });
        }
    }

    requests
}

fn generate_user_inventory(agent_id: &str) -> UserInventoryData {
    use uuid::Uuid;

    // Generate deterministic folder IDs using the same logic as InventoryFolder::new()
    // Format: owner_id:folder_name.to_lowercase()
    fn folder_id(owner_id: &str, name: &str) -> String {
        let folder_key = format!("{}:{}", owner_id, name.to_lowercase());
        Uuid::new_v5(&Uuid::NAMESPACE_OID, folder_key.as_bytes()).to_string()
    }

    // Create folder IDs - MUST match login inventory folder IDs
    let root_str = folder_id(agent_id, "my inventory");
    let bodyparts_str = folder_id(agent_id, "body parts");
    let clothing_str = folder_id(agent_id, "clothing");

    let mut folders = Vec::new();
    let mut items = Vec::new();

    // Root folder
    folders.push(InventoryFolderData {
        id: root_str.clone(),
        parent_id: None,
        name: "My Inventory".to_string(),
        folder_type: 8, // Root
        version: 1,
    });

    // Body Parts folder
    folders.push(InventoryFolderData {
        id: bodyparts_str.clone(),
        parent_id: Some(root_str.clone()),
        name: "Body Parts".to_string(),
        folder_type: 13, // BodyPart
        version: 1,
    });

    // Clothing folder
    folders.push(InventoryFolderData {
        id: clothing_str.clone(),
        parent_id: Some(root_str.clone()),
        name: "Clothing".to_string(),
        folder_type: 5, // Clothing
        version: 1,
    });

    // Add other system folders - MUST use same names as InventoryFolderType::default_name()
    let system_folders = [
        ("Textures", 0),
        ("Sounds", 1),
        ("Calling Cards", 2),
        ("Landmarks", 3),
        ("Objects", 6),
        ("Notecards", 7),
        ("Scripts", 10),
        ("Trash", 14),
        ("Photo Album", 15),
        ("Lost And Found", 16),
        ("Animations", 20),
        ("Gestures", 21),
        ("Favorites", 23),
        ("Current Outfit", 46),
        ("My Outfits", 48),
        ("Received Items", 50),
        ("Settings", 56),
        ("Materials", 57),
    ];

    for (name, folder_type_val) in system_folders {
        folders.push(InventoryFolderData {
            id: folder_id(agent_id, name),
            parent_id: Some(root_str.clone()),
            name: name.to_string(),
            folder_type: folder_type_val,
            version: 1,
        });
    }

    // Add Ruth wearable items - MUST match the UUIDs sent in AgentWearablesUpdate
    // Body parts (in Body Parts folder)
    items.push(InventoryItemData {
        id: "66c41e39-38f9-f75a-024e-585989bfaba9".to_string(),
        folder_id: bodyparts_str.clone(),
        asset_id: "66c41e39-38f9-f75a-024e-585989bfab73".to_string(),
        name: "Default Shape".to_string(),
        description: "Default Shape wearable".to_string(),
        asset_type: 13, // Bodypart
        inv_type: 18,   // Wearable
        flags: 0,       // Shape type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    items.push(InventoryItemData {
        id: "77c41e39-38f9-f75a-024e-585989bfabc9".to_string(),
        folder_id: bodyparts_str.clone(),
        asset_id: "77c41e39-38f9-f75a-024e-585989bbabbb".to_string(),
        name: "Default Skin".to_string(),
        description: "Default Skin wearable".to_string(),
        asset_type: 13, // Bodypart
        inv_type: 18,   // Wearable
        flags: 1,       // Skin type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    items.push(InventoryItemData {
        id: "d342e6c1-b9d2-11dc-95ff-0800200c9a66".to_string(),
        folder_id: bodyparts_str.clone(),
        asset_id: "d342e6c0-b9d2-11dc-95ff-0800200c9a66".to_string(),
        name: "Default Hair".to_string(),
        description: "Default Hair wearable".to_string(),
        asset_type: 13, // Bodypart
        inv_type: 18,   // Wearable
        flags: 2,       // Hair type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    items.push(InventoryItemData {
        id: "cdc31054-eed8-4021-994f-4e0c6e861b50".to_string(),
        folder_id: bodyparts_str.clone(),
        asset_id: "4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7".to_string(),
        name: "Default Eyes".to_string(),
        description: "Default Eyes wearable".to_string(),
        asset_type: 13, // Bodypart
        inv_type: 18,   // Wearable
        flags: 3,       // Eyes type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    // Clothing items (in Clothing folder)
    items.push(InventoryItemData {
        id: "77c41e39-38f9-f75a-0000-585989bf0000".to_string(),
        folder_id: clothing_str.clone(),
        asset_id: "00000000-38f9-1111-024e-222222111110".to_string(),
        name: "Default Shirt".to_string(),
        description: "Default Shirt wearable".to_string(),
        asset_type: 5, // Clothing
        inv_type: 18,  // Wearable
        flags: 4,      // Shirt type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    items.push(InventoryItemData {
        id: "77c41e39-38f9-f75a-0000-5859892f1111".to_string(),
        folder_id: clothing_str.clone(),
        asset_id: "00000000-38f9-1111-024e-222222111120".to_string(),
        name: "Default Pants".to_string(),
        description: "Default Pants wearable".to_string(),
        asset_type: 5, // Clothing
        inv_type: 18,  // Wearable
        flags: 5,      // Pants type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    items.push(InventoryItemData {
        id: "77c41e39-38f9-f75a-0000-5859892f2222".to_string(),
        folder_id: clothing_str.clone(),
        asset_id: "00000000-38f9-1111-024e-222222111130".to_string(),
        name: "Default Shoes".to_string(),
        description: "Default Shoes wearable".to_string(),
        asset_type: 5, // Clothing
        inv_type: 18,  // Wearable
        flags: 6,      // Shoes type (wearable type 6)
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    // Current Outfit Folder (COF) links - these tell the viewer what's currently worn
    // Links have inv_type=24 (Link) and the asset_id points to the original item
    let cof_str = folder_id(agent_id, "current outfit");

    // Link to Shape
    items.push(InventoryItemData {
        id: folder_id(agent_id, "cof-link-shape"),
        folder_id: cof_str.clone(),
        asset_id: "66c41e39-38f9-f75a-024e-585989bfaba9".to_string(), // Points to item ID
        name: "Default Shape".to_string(),
        description: "@shape".to_string(),
        asset_type: 24, // Link
        inv_type: 18,   // Wearable
        flags: 0,       // Shape type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    // Link to Skin
    items.push(InventoryItemData {
        id: folder_id(agent_id, "cof-link-skin"),
        folder_id: cof_str.clone(),
        asset_id: "77c41e39-38f9-f75a-024e-585989bfabc9".to_string(), // Points to item ID
        name: "Default Skin".to_string(),
        description: "@skin".to_string(),
        asset_type: 24, // Link
        inv_type: 18,   // Wearable
        flags: 1,       // Skin type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    // Link to Hair
    items.push(InventoryItemData {
        id: folder_id(agent_id, "cof-link-hair"),
        folder_id: cof_str.clone(),
        asset_id: "d342e6c1-b9d2-11dc-95ff-0800200c9a66".to_string(), // Points to item ID
        name: "Default Hair".to_string(),
        description: "@hair".to_string(),
        asset_type: 24, // Link
        inv_type: 18,   // Wearable
        flags: 2,       // Hair type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    // Link to Eyes
    items.push(InventoryItemData {
        id: folder_id(agent_id, "cof-link-eyes"),
        folder_id: cof_str.clone(),
        asset_id: "cdc31054-eed8-4021-994f-4e0c6e861b50".to_string(), // Points to item ID
        name: "Default Eyes".to_string(),
        description: "@eyes".to_string(),
        asset_type: 24, // Link
        inv_type: 18,   // Wearable
        flags: 3,       // Eyes type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    // Link to Shirt
    items.push(InventoryItemData {
        id: folder_id(agent_id, "cof-link-shirt"),
        folder_id: cof_str.clone(),
        asset_id: "77c41e39-38f9-f75a-0000-585989bf0000".to_string(), // Points to item ID
        name: "Default Shirt".to_string(),
        description: "@shirt".to_string(),
        asset_type: 24, // Link
        inv_type: 18,   // Wearable
        flags: 4,       // Shirt type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    // Link to Pants
    items.push(InventoryItemData {
        id: folder_id(agent_id, "cof-link-pants"),
        folder_id: cof_str.clone(),
        asset_id: "77c41e39-38f9-f75a-0000-5859892f1111".to_string(), // Points to item ID
        name: "Default Pants".to_string(),
        description: "@pants".to_string(),
        asset_type: 24, // Link
        inv_type: 18,   // Wearable
        flags: 5,       // Pants type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    // Link to Shoes
    items.push(InventoryItemData {
        id: folder_id(agent_id, "cof-link-shoes"),
        folder_id: cof_str.clone(),
        asset_id: "77c41e39-38f9-f75a-0000-5859892f2222".to_string(), // Points to item ID
        name: "Default Shoes".to_string(),
        description: "@shoes".to_string(),
        asset_type: 24, // Link
        inv_type: 18,   // Wearable
        flags: 6,       // Shoes type
        creator_id: agent_id.to_string(),
        owner_id: agent_id.to_string(),
    });

    UserInventoryData { folders, items }
}

// UpdateAvatarAppearance handler - updates avatar appearance
pub async fn handle_update_avatar_appearance(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "👤 UpdateAvatarAppearance request for session: {}",
        session_id
    );

    // Update session activity
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    info!("👤 Appearance update: {}", body_str);

    // Return success response
    let response = json!({
        "success": true,
        "agent_id": session_id
    });

    info!("👤 Avatar appearance updated for session: {}", session_id);
    json_response_to_llsd_xml(response)
}

// ViewerStats handler - receives viewer statistics
pub async fn handle_viewer_stats(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!("📊 ViewerStats request for session: {}", session_id);

    // Update session activity
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    info!("📊 Viewer stats: {}", body_str);

    // Return acknowledgment
    let response = json!({
        "success": true
    });

    json_response_to_llsd_xml(response)
}

// UpdateAgentInformation handler - updates agent information
pub async fn handle_update_agent_information(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "ℹ️ UpdateAgentInformation request for session: {}",
        session_id
    );

    // Update session activity
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    info!("ℹ️ Agent info update: {}", body_str);

    let response = json!({
        "success": true
    });

    json_response_to_llsd_xml(response)
}

// UpdateAgentLanguage handler
pub async fn handle_update_agent_language(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!("🔧 UpdateAgentLanguage request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    if !body.is_empty() {
        let body_str = String::from_utf8_lossy(&body);
        info!("🔧 UpdateAgentLanguage body: {}", body_str);
    }

    json_response_to_llsd_xml(json!({"success": true, "capability": "UpdateAgentLanguage"}))
}

// AgentPreferences handlers
pub async fn handle_agent_preferences_get(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("[CAPS] AgentPreferences GET for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let agent_id = match state.caps_manager.get_session(&session_id).await {
        Some(s) => uuid::Uuid::parse_str(&s.agent_id).unwrap_or_default(),
        None => return json_response_to_llsd_xml(json!({"success": false})),
    };

    let pool = state.db_pool.as_ref();
    let row = sqlx::query(
        "SELECT principalid, accessprefs, hoverheight, language, \
         languageispublic, permeveryone, permgroup, permnextowner \
         FROM agentprefs WHERE principalid = $1",
    )
    .bind(agent_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match row {
        Some(r) => {
            use sqlx::Row;
            let access: String = r
                .try_get::<String, _>("accessprefs")
                .unwrap_or_else(|_| "M".to_string());
            let hover: f32 = r.try_get("hoverheight").unwrap_or(0.0);
            let lang: String = r
                .try_get::<String, _>("language")
                .unwrap_or_else(|_| "en-us".to_string());
            let lang_pub: i32 = r.try_get("languageispublic").unwrap_or(1);
            let pe: i32 = r.try_get("permeveryone").unwrap_or(0);
            let pg: i32 = r.try_get("permgroup").unwrap_or(0);
            let pno: i32 = r.try_get("permnextowner").unwrap_or(532480);

            let xml = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                 <llsd><map>\
                 <key>access_prefs</key><map><key>max</key><string>{}</string></map>\
                 <key>default_object_perm_masks</key><map>\
                 <key>Everyone</key><integer>{}</integer>\
                 <key>Group</key><integer>{}</integer>\
                 <key>NextOwner</key><integer>{}</integer>\
                 </map>\
                 <key>hover_height</key><real>{}</real>\
                 <key>language</key><string>{}</string>\
                 <key>language_is_public</key><boolean>{}</boolean>\
                 </map></llsd>",
                access.trim(),
                pe,
                pg,
                pno,
                hover,
                lang.trim(),
                if lang_pub != 0 { "true" } else { "false" }
            );

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/llsd+xml")
                .body(axum::body::Body::from(xml))
                .unwrap())
        }
        None => {
            let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                 <llsd><map>\
                 <key>access_prefs</key><map><key>max</key><string>M</string></map>\
                 <key>default_object_perm_masks</key><map>\
                 <key>Everyone</key><integer>0</integer>\
                 <key>Group</key><integer>0</integer>\
                 <key>NextOwner</key><integer>532480</integer>\
                 </map>\
                 <key>hover_height</key><real>0</real>\
                 <key>language</key><string>en-us</string>\
                 <key>language_is_public</key><boolean>true</boolean>\
                 </map></llsd>";

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/llsd+xml")
                .body(axum::body::Body::from(xml.to_string()))
                .unwrap())
        }
    }
}

pub async fn handle_agent_preferences_post(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!("[CAPS] AgentPreferences POST for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let agent_id = match state.caps_manager.get_session(&session_id).await {
        Some(s) => uuid::Uuid::parse_str(&s.agent_id).unwrap_or_default(),
        None => return json_response_to_llsd_xml(json!({"success": false})),
    };

    if !body.is_empty() {
        let body_str = String::from_utf8_lossy(&body);

        let mut access = "M".to_string();
        let mut hover: f64 = 0.0;
        let mut lang = "en-us".to_string();
        let mut lang_pub = true;
        let mut pe: i32 = 0;
        let mut pg: i32 = 0;
        let mut pno: i32 = 532480;

        if let Some(cap) = extract_llsd_string(&body_str, "max") {
            access = cap;
        }
        if let Some(v) = extract_llsd_real(&body_str, "hover_height") {
            hover = v;
        }
        if let Some(v) = extract_llsd_string(&body_str, "language") {
            lang = v;
        }
        if let Some(v) = extract_llsd_bool(&body_str, "language_is_public") {
            lang_pub = v;
        }
        if let Some(v) = extract_llsd_integer(&body_str, "Everyone") {
            pe = v;
        }
        if let Some(v) = extract_llsd_integer(&body_str, "Group") {
            pg = v;
        }
        if let Some(v) = extract_llsd_integer(&body_str, "NextOwner") {
            pno = v;
        }

        let pool = state.db_pool.as_ref();
        let _ = sqlx::query(
            "INSERT INTO agentprefs (principalid, accessprefs, hoverheight, language, \
             languageispublic, permeveryone, permgroup, permnextowner) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (principalid) DO UPDATE SET \
             accessprefs = $2, hoverheight = $3, language = $4, \
             languageispublic = $5, permeveryone = $6, permgroup = $7, permnextowner = $8",
        )
        .bind(agent_id)
        .bind(&access)
        .bind(hover as f32)
        .bind(&lang)
        .bind(if lang_pub { 1i32 } else { 0i32 })
        .bind(pe)
        .bind(pg)
        .bind(pno)
        .execute(pool)
        .await;

        info!(
            "[CAPS] AgentPreferences stored for {} lang={}",
            agent_id, lang
        );
    }

    json_response_to_llsd_xml(json!({"status": "success"}))
}

fn extract_llsd_real(xml: &str, key: &str) -> Option<f64> {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let after_key = &xml[pos + key_tag.len()..];
        if let Some(start) = after_key.find("<real>") {
            let val_start = start + 6;
            if let Some(end) = after_key[val_start..].find("</real>") {
                return after_key[val_start..val_start + end].parse().ok();
            }
        }
    }
    None
}

fn extract_llsd_bool(xml: &str, key: &str) -> Option<bool> {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let after_key = &xml[pos + key_tag.len()..];
        if let Some(start) = after_key.find("<boolean>") {
            let val_start = start + 9;
            if let Some(end) = after_key[val_start..].find("</boolean>") {
                let val = &after_key[val_start..val_start + end];
                return Some(val == "true" || val == "1");
            }
        }
    }
    None
}

// HomeLocation handlers
pub async fn handle_home_location_get(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("🔧 HomeLocation GET request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    json_response_to_llsd_xml(json!({"success": true, "capability": "HomeLocation"}))
}

pub async fn handle_home_location_post(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!("🏠 HomeLocation POST request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let session = match state.caps_manager.get_session(&session_id).await {
        Some(s) => s,
        None => return Err(StatusCode::NOT_FOUND),
    };
    let agent_id = session.agent_id.clone();
    let region_uuid = if !session.region_uuid.is_empty() {
        session.region_uuid.clone()
    } else {
        let fallback: String = sqlx::query_as::<_, (String,)>(
            "SELECT id::text FROM regions WHERE region_name = (SELECT region_name FROM regions LIMIT 1) LIMIT 1"
        )
        .fetch_optional(&*state.db_pool)
        .await
        .ok()
        .flatten()
        .map(|(r,)| r)
        .unwrap_or_default();
        warn!(
            "🏠 HomeLocation: session.region_uuid empty, fallback to DB: {}",
            fallback
        );
        fallback
    };

    if body.is_empty() {
        return json_response_to_llsd_xml(json!({"success": "false"}));
    }

    let body_str = String::from_utf8_lossy(&body);
    info!("🏠 HomeLocation body: {}", body_str);

    let hl_start = body_str.find("<key>HomeLocation</key>");
    if hl_start.is_none() {
        warn!("🏠 HomeLocation: missing HomeLocation key in body");
        return json_response_to_llsd_xml(json!({"success": "false"}));
    }
    let hl_xml = &body_str[hl_start.unwrap()..];

    let extract_real_from = |xml: &str, key: &str| -> f64 {
        let key_tag = format!("<key>{}</key>", key);
        if let Some(pos) = xml.find(&key_tag) {
            let after = &xml[pos + key_tag.len()..];
            if let Some(s) = after.find("<real>") {
                let vs = s + 6;
                if let Some(e) = after[vs..].find("</real>") {
                    return after[vs..vs + e].parse().unwrap_or(0.0);
                }
            }
            if let Some(s) = after.find("<integer>") {
                let vs = s + 9;
                if let Some(e) = after[vs..].find("</integer>") {
                    return after[vs..vs + e].parse().unwrap_or(0.0);
                }
            }
        }
        0.0
    };

    let lp_start = hl_xml.find("<key>LocationPos</key>");
    let (px, py, pz) = if let Some(lps) = lp_start {
        let lp_xml = &hl_xml[lps..];
        (
            extract_real_from(lp_xml, "X"),
            extract_real_from(lp_xml, "Y"),
            extract_real_from(lp_xml, "Z"),
        )
    } else {
        (128.0, 128.0, 25.0)
    };

    let la_start = hl_xml.find("<key>LocationLookAt</key>");
    let (lx, ly, lz) = if let Some(las) = la_start {
        let la_xml = &hl_xml[las..];
        (
            extract_real_from(la_xml, "X"),
            extract_real_from(la_xml, "Y"),
            extract_real_from(la_xml, "Z"),
        )
    } else {
        (1.0, 0.0, 0.0)
    };

    let pos_str = format!("<{},{},{}>", px, py, pz);
    let look_str = format!("<{},{},{}>", lx, ly, lz);

    info!(
        "🏠 SetHome agent={} region={} pos={} look={}",
        agent_id, region_uuid, pos_str, look_str
    );

    let region_uuid_parsed = uuid::Uuid::parse_str(&region_uuid).unwrap_or_default();
    let result = sqlx::query(
        "INSERT INTO griduser (userid, homeregionid, homeposition, homelookat, \
         lastregionid, lastposition, lastlookat, online, login, logout) \
         VALUES ($1, $2::uuid, $3, $4, \
         '00000000-0000-0000-0000-000000000000', '<0,0,0>', '<0,0,0>', 'false', '0', '0') \
         ON CONFLICT (userid) DO UPDATE SET homeregionid = $2::uuid, homeposition = $3, homelookat = $4"
    )
    .bind(&agent_id)
    .bind(region_uuid_parsed.to_string())
    .bind(&pos_str)
    .bind(&look_str)
    .execute(&*state.db_pool)
    .await;

    match result {
        Ok(_) => {
            info!("🏠 Home location saved for agent {}", agent_id);
            let resp = json!({
                "success": "true",
                "HomeLocation": {
                    "LocationPos": {
                        "X": px,
                        "Y": py,
                        "Z": pz
                    }
                }
            });
            json_response_to_llsd_xml(resp)
        }
        Err(e) => {
            warn!("🏠 Failed to save home location: {}", e);
            json_response_to_llsd_xml(json!({"success": "false"}))
        }
    }
}

// Display names handlers
pub async fn handle_get_display_names(
    Path(session_id): Path<String>,
    axum::extract::RawQuery(raw_query): axum::extract::RawQuery,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!(
        "[DISPLAYNAMES] GetDisplayNames request for session: {}, raw_query: {:?}",
        session_id, raw_query
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let mut agent_ids: Vec<uuid::Uuid> = Vec::new();
    if let Some(query_str) = &raw_query {
        for pair in query_str.split('&') {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() == 2 && parts[0] == "ids" {
                let decoded = urlencoding::decode(parts[1]).unwrap_or_else(|_| parts[1].into());
                for id_str in decoded.split(',') {
                    let id_trimmed = id_str.trim();
                    if let Ok(agent_id) = uuid::Uuid::parse_str(id_trimmed) {
                        agent_ids.push(agent_id);
                    } else {
                        warn!("[DISPLAYNAMES] Invalid UUID in request: {}", id_trimmed);
                    }
                }
            }
        }
    }

    info!(
        "[DISPLAYNAMES] Parsed {} agent IDs from query",
        agent_ids.len()
    );

    let galadriel_id = crate::ai::galadriel::GALADRIEL_AGENT_ID;
    let mut agents_xml = String::new();

    for agent_id in &agent_ids {
        if *agent_id == galadriel_id {
            let username = "galadriel.ai";
            let display_name = "Galadriel";
            agents_xml.push_str(&format!(
                r#"<map>
                    <key>id</key><uuid>{}</uuid>
                    <key>username</key><string>{}</string>
                    <key>display_name</key><string>{}</string>
                    <key>legacy_first_name</key><string>Galadriel</string>
                    <key>legacy_last_name</key><string>AI</string>
                    <key>is_display_name_default</key><boolean>true</boolean>
                    <key>display_name_next_update</key><date>1970-01-01T00:00:00Z</date>
                </map>"#,
                agent_id, username, display_name
            ));
            info!(
                "[DISPLAYNAMES] Returning Galadriel display name for {}",
                agent_id
            );
            continue;
        }

        let result = sqlx::query_as::<_, (String, String)>(
            "SELECT firstname, lastname FROM useraccounts WHERE principalid = $1::uuid",
        )
        .bind(agent_id.to_string())
        .fetch_optional(state.db_pool.as_ref())
        .await;

        let (first_name, last_name) = match result {
            Ok(Some((f, l))) => (f, l),
            _ => {
                let roster = crate::ai::npc_roster::default_roster();
                if let Some(npc) = roster.iter().find(|n| n.agent_id == *agent_id) {
                    info!(
                        "[DISPLAYNAMES] NPC {} found in roster: {} {}",
                        agent_id, npc.first_name, npc.last_name
                    );
                    (npc.first_name.clone(), npc.last_name.clone())
                } else {
                    info!(
                        "[DISPLAYNAMES] User {} not found in database or roster",
                        agent_id
                    );
                    ("Unknown".to_string(), "User".to_string())
                }
            }
        };

        let username = format!("{}.{}", first_name.to_lowercase(), last_name.to_lowercase());
        let display_name = format!("{} {}", first_name, last_name);

        agents_xml.push_str(&format!(
            r#"<map>
                <key>id</key><uuid>{}</uuid>
                <key>username</key><string>{}</string>
                <key>display_name</key><string>{}</string>
                <key>legacy_first_name</key><string>{}</string>
                <key>legacy_last_name</key><string>{}</string>
                <key>is_display_name_default</key><boolean>true</boolean>
                <key>display_name_next_update</key><date>1970-01-01T00:00:00Z</date>
            </map>"#,
            agent_id, username, display_name, first_name, last_name
        ));

        info!(
            "[DISPLAYNAMES] Returning display name for {}: {} {}",
            agent_id, first_name, last_name
        );
    }

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<llsd><map>
    <key>agents</key>
    <array>{}</array>
</map></llsd>"#,
        agents_xml
    );

    info!(
        "[DISPLAYNAMES] Returning LLSD response ({} bytes) for {} agents",
        xml.len(),
        agent_ids.len()
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(xml.into())
        .unwrap())
}

pub async fn handle_set_display_name(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!("🔧 SetDisplayName request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    if !body.is_empty() {
        let body_str = String::from_utf8_lossy(&body);
        info!("🔧 SetDisplayName body: {}", body_str);
    }

    json_response_to_llsd_xml(json!({"success": true, "capability": "SetDisplayName"}))
}

// Inventory handlers

pub async fn handle_create_inventory_category(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "📁 CreateInventoryCategory request for session: {}",
        session_id
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    info!("📁 CreateInventoryCategory body: {}", body_str);

    let folder_id = extract_llsd_uuid(&body_str, "folder_id").unwrap_or_default();
    let parent_id = extract_llsd_uuid(&body_str, "parent_id").unwrap_or_default();
    let name = extract_llsd_string(&body_str, "name").unwrap_or_default();
    let folder_type = extract_llsd_integer(&body_str, "type").unwrap_or(-1);

    if folder_id.is_empty() || parent_id.is_empty() || name.is_empty() {
        warn!("📁 CreateInventoryCategory: missing required fields");
        return Err(StatusCode::BAD_REQUEST);
    }

    let folder_uuid = match uuid::Uuid::parse_str(&folder_id) {
        Ok(u) => u,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    let parent_uuid = match uuid::Uuid::parse_str(&parent_id) {
        Ok(u) => u,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let agent_id = {
        let sessions = state.caps_manager.sessions.read().await;
        sessions.get(&session_id).map(|s| s.agent_id.clone())
    };
    let agent_id = match agent_id {
        Some(id) => id,
        None => return Err(StatusCode::NOT_FOUND),
    };
    let agent_uuid = match uuid::Uuid::parse_str(&agent_id) {
        Ok(u) => u,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let existing: Option<(uuid::Uuid,)> = sqlx::query_as(
        r#"SELECT folderid FROM inventoryfolders
           WHERE agentid = $1 AND parentfolderid = $2
             AND foldername = $3 AND type = $4
           LIMIT 1"#,
    )
    .bind(agent_uuid)
    .bind(parent_uuid)
    .bind(&name)
    .bind(folder_type as i32)
    .fetch_optional(&*state.db_pool)
    .await
    .unwrap_or(None);

    let (actual_folder_uuid, was_existing) = if let Some((existing_id,)) = existing {
        info!("📁 Folder '{}' (type={}) already exists as {} — dedup, skipping create (viewer sent {})",
            name, folder_type, existing_id, folder_uuid);
        (existing_id, true)
    } else {
        (folder_uuid, false)
    };

    if was_existing {
        // noop
    } else {
        let result = sqlx::query(
            r#"INSERT INTO inventoryfolders (folderid, agentid, parentfolderid, foldername, type, version)
               VALUES ($1, $2, $3, $4, $5, 1)
               ON CONFLICT (folderid) DO UPDATE SET
                 foldername = EXCLUDED.foldername,
                 type = EXCLUDED.type,
                 parentfolderid = EXCLUDED.parentfolderid,
                 version = inventoryfolders.version + 1"#
        )
        .bind(folder_uuid)
        .bind(agent_uuid)
        .bind(parent_uuid)
        .bind(&name)
        .bind(folder_type as i32)
        .execute(&*state.db_pool)
        .await;

        match result {
            Ok(_) => {}
            Err(e) => {
                error!("📁 Failed to create inventory folder: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    let response_xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<llsd><map>\
        <key>folder_id</key><uuid>{}</uuid>\
        <key>name</key><string>{}</string>\
        <key>parent_id</key><uuid>{}</uuid>\
        <key>type</key><integer>{}</integer>\
        </map></llsd>",
        actual_folder_uuid, name, parent_uuid, folder_type
    );
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(axum::body::Body::from(response_xml))
        .unwrap()
        .into_response())
}

fn extract_llsd_uuid(xml: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let after_key = &xml[pos + key_tag.len()..];
        if let Some(start) = after_key.find("<uuid>") {
            let val_start = start + 6;
            if let Some(end) = after_key[val_start..].find("</uuid>") {
                return Some(after_key[val_start..val_start + end].to_string());
            }
        }
    }
    None
}

fn extract_llsd_string(xml: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let after_key = &xml[pos + key_tag.len()..];
        if let Some(start) = after_key.find("<string>") {
            let val_start = start + 8;
            if let Some(end) = after_key[val_start..].find("</string>") {
                return Some(after_key[val_start..val_start + end].to_string());
            }
        }
    }
    None
}

fn extract_llsd_integer(xml: &str, key: &str) -> Option<i32> {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let after_key = &xml[pos + key_tag.len()..];
        if let Some(start) = after_key.find("<integer>") {
            let val_start = start + 9;
            if let Some(end) = after_key[val_start..].find("</integer>") {
                return after_key[val_start..val_start + end].parse().ok();
            }
        }
    }
    None
}

fn extract_llsd_vector3(xml: &str, key: &str) -> Option<[f32; 3]> {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let after_key = &xml[pos + key_tag.len()..];
        if let Some(arr_start) = after_key.find("<array>") {
            let arr_content = &after_key[arr_start..];
            if let Some(arr_end) = arr_content.find("</array>") {
                let arr_str = &arr_content[..arr_end];
                let mut vals = Vec::new();
                let mut search = arr_str;
                while let Some(s) = search.find("<real>") {
                    let v_start = s + 6;
                    if let Some(e) = search[v_start..].find("</real>") {
                        if let Ok(v) = search[v_start..v_start + e].parse::<f64>() {
                            vals.push(v as f32);
                        }
                        search = &search[v_start + e + 7..];
                    } else {
                        break;
                    }
                }
                if vals.len() >= 3 {
                    return Some([vals[0], vals[1], vals[2]]);
                }
            }
        }
    }
    None
}

fn extract_llsd_u32(xml: &str, key: &str) -> Option<u32> {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let after_key = &xml[pos + key_tag.len()..];
        let int_pos = after_key.find("<integer>");
        let bin_pos = after_key.find("<binary encoding=\"base64\">");
        let next_key = after_key.find("<key>");
        if let Some(bp) = bin_pos {
            if next_key.is_none() || bp < next_key.unwrap() {
                let val_start = bp + "<binary encoding=\"base64\">".len();
                if let Some(end) = after_key[val_start..].find("</binary>") {
                    let b64 = after_key[val_start..val_start + end].trim();
                    use base64::Engine;
                    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
                        if bytes.len() >= 4 {
                            return Some(u32::from_be_bytes([
                                bytes[0], bytes[1], bytes[2], bytes[3],
                            ]));
                        }
                    }
                }
            }
        }
        if let Some(ip) = int_pos {
            if next_key.is_none() || ip < next_key.unwrap() {
                let val_start = ip + 9;
                if let Some(end) = after_key[val_start..].find("</integer>") {
                    return after_key[val_start..val_start + end].parse().ok();
                }
            }
        }
    }
    None
}

pub async fn handle_new_file_agent_inventory(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "🔧 NewFileAgentInventory request for session: {}",
        session_id
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    info!("🔧 NewFileAgentInventory body: {}", body_str);

    let asset_type = extract_llsd_string(&body_str, "asset_type").unwrap_or_default();
    let name = extract_llsd_string(&body_str, "name").unwrap_or("New Item".to_string());
    let description = extract_llsd_string(&body_str, "description").unwrap_or_default();
    let folder_id = extract_llsd_uuid(&body_str, "folder_id").unwrap_or_default();
    let inventory_type = extract_llsd_string(&body_str, "inventory_type").unwrap_or_default();

    let uploader_uuid = uuid::Uuid::new_v4().to_string();
    let uploader_url = format!(
        "{}/cap/{}/NewFileAgentInventoryUpload/{}",
        state.caps_manager.base_url, session_id, uploader_uuid
    );

    {
        let mut sessions = state.caps_manager.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.pending_uploads.insert(
                uploader_uuid.clone(),
                PendingUpload {
                    name,
                    description,
                    asset_type,
                    inventory_type,
                    folder_id,
                    task_id: None,
                    is_script_running: false,
                },
            );
        }
    }

    info!(
        "🔧 NewFileAgentInventory: returning uploader URL: {}",
        uploader_url
    );

    json_response_to_llsd_xml(json!({
        "uploader": uploader_url,
        "state": "upload",
        "upload_price": 0,
        "data": {
            "upload_price": 0,
            "model_streaming_cost": 1.0,
            "simulation_cost": 0.5,
            "physics_cost": 0.5,
            "resource_cost": 1.0
        }
    }))
}

pub async fn handle_new_file_agent_inventory_upload(
    Path((session_id, uploader_id)): Path<(String, String)>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "🔧 NewFileAgentInventoryUpload data received: session={}, uploader={} ({} bytes)",
        session_id,
        uploader_id,
        body.len()
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let (agent_id_str, pending) = {
        let mut sessions = state.caps_manager.sessions.write().await;
        let session = sessions.get_mut(&session_id).ok_or(StatusCode::NOT_FOUND)?;
        let pending = session.pending_uploads.remove(&uploader_id);
        (session.agent_id.clone(), pending)
    };

    let pending = match pending {
        Some(p) => p,
        None => {
            warn!("🔧 No pending upload found for uploader: {}", uploader_id);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let agent_uuid =
        uuid::Uuid::parse_str(&agent_id_str).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let new_asset_id = uuid::Uuid::new_v4();
    let new_item_id = uuid::Uuid::new_v4();
    let folder_uuid = uuid::Uuid::parse_str(&pending.folder_id).unwrap_or(uuid::Uuid::nil());

    let asset_type_num = match pending.asset_type.as_str() {
        "texture" => 0i32,
        "sound" => 1,
        "callcard" | "callingcard" => 2,
        "landmark" => 3,
        "clothing" => 5,
        "object" => 6,
        "notecard" => 7,
        "lsltext" | "script" => 10,
        "bodypart" => 13,
        "animation" | "animatn" => 20,
        "gesture" => 21,
        "mesh" => 49,
        "settings" => 56,
        "material" => 57,
        _ => 0,
    };

    let inv_type_num = match pending.inventory_type.as_str() {
        "texture" => 0i32,
        "sound" => 1,
        "callcard" | "callingcard" => 2,
        "landmark" => 3,
        "object" => 6,
        "notecard" => 7,
        "script" | "lsltext" => 10,
        "wearable" | "bodypart" | "clothing" => 18,
        "animation" => 19,
        "gesture" => 20,
        "mesh" => 22,
        "settings" => 25,
        _ => 0,
    };

    let asset_result = sqlx::query(
        r#"INSERT INTO assets (id, name, description, assettype, local, temporary, create_time, access_time, data, creatorid, asset_flags)
           VALUES ($1, $2, $3, $4, 0, 0, $5, $5, $6, $7, 0)
           ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data"#
    )
    .bind(new_asset_id)
    .bind(&pending.name)
    .bind(&pending.description)
    .bind(asset_type_num)
    .bind(chrono::Utc::now().timestamp() as i32)
    .bind(&body[..])
    .bind(agent_uuid.to_string())
    .execute(&*state.db_pool)
    .await;

    if let Err(e) = asset_result {
        error!("🔧 Failed to create asset: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let base_perms: i32 = 0x7FFFFFFF;
    let item_result = sqlx::query(
        r#"INSERT INTO inventoryitems (inventoryid, assetid, assettype, parentfolderid, avatarid,
           inventoryname, inventorydescription, inventorynextpermissions, inventorycurrentpermissions,
           invtype, creatorid, inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
           saleprice, saletype, creationdate, groupid, groupowned, flags)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8, $9, $10, $8, 0, 0, 0, 0, $11, $12, 0, 0)
           ON CONFLICT (inventoryid) DO UPDATE SET assetid = EXCLUDED.assetid"#
    )
    .bind(new_item_id)
    .bind(new_asset_id)
    .bind(asset_type_num)
    .bind(folder_uuid)
    .bind(agent_uuid)
    .bind(&pending.name)
    .bind(&pending.description)
    .bind(base_perms)
    .bind(inv_type_num)
    .bind(agent_uuid.to_string())
    .bind(chrono::Utc::now().timestamp() as i32)
    .bind(uuid::Uuid::nil())
    .execute(&*state.db_pool)
    .await;

    if let Err(e) = item_result {
        error!("🔧 Failed to create inventory item: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if asset_type_num == 49 {
        match crate::mesh::parser::parse_mesh_header(&body) {
            Ok(header) => {
                let has_physics = header.physics_convex.is_some();
                let has_high = header.high_lod.is_some();
                info!(
                    "[MESH-UPLOAD] asset={} physics_convex={} high_lod={}",
                    new_asset_id, has_physics, has_high
                );
                if !has_physics {
                    warn!(
                        "[MESH-UPLOAD] Mesh {} missing physics_convex block",
                        new_asset_id
                    );
                }
                if let Some(ref pc) = header.physics_convex {
                    match crate::mesh::parser::extract_physics_convex(&body, &header) {
                        Ok(convex) => {
                            info!("[MESH-UPLOAD] physics_convex: {} hulls, {} total verts, offset={} size={}",
                                convex.hull_count(), convex.total_vertices(), pc.offset, pc.size);
                        }
                        Err(e) => {
                            warn!(
                                "[MESH-UPLOAD] physics_convex parse failed for {}: {}",
                                new_asset_id, e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    "[MESH-UPLOAD] Mesh header parse failed for {}: {}",
                    new_asset_id, e
                );
            }
        }
    }

    info!(
        "NewFileAgentInventory complete: asset={} item={} name={}",
        new_asset_id, new_item_id, pending.name
    );

    json_response_to_llsd_xml(json!({
        "new_asset": new_asset_id.to_string(),
        "new_inventory_item": new_item_id.to_string(),
        "state": "complete"
    }))
}

pub async fn handle_update_notecard_agent_inventory(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    let body_str = String::from_utf8_lossy(&body);
    info!(
        "[NOTECARD-SAVE] Stage-1 request: session={} body={}",
        session_id, body_str
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let item_id = extract_llsd_uuid(&body_str, "item_id")
        .or_else(|| extract_llsd_string(&body_str, "item_id"))
        .unwrap_or_default();

    if item_id.is_empty() {
        warn!("[NOTECARD-SAVE] Stage-1: could not extract item_id from body");
    }
    info!("[NOTECARD-SAVE] Stage-1: item_id={}", item_id);

    let uploader_id = uuid::Uuid::new_v4().to_string();

    let session_found = {
        let mut sessions = state.caps_manager.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.pending_uploads.insert(
                uploader_id.clone(),
                PendingUpload {
                    name: item_id.clone(),
                    description: String::new(),
                    asset_type: "notecard".to_string(),
                    inventory_type: "notecard".to_string(),
                    folder_id: String::new(),
                    task_id: None,
                    is_script_running: false,
                },
            );
            true
        } else {
            false
        }
    };

    if !session_found {
        warn!(
            "[NOTECARD-SAVE] Stage-1: session {} NOT FOUND in caps_manager",
            session_id
        );
    }

    let uploader_url = format!(
        "{}/cap/{}/NotecardUpload/{}",
        state.caps_manager.base_url, session_id, uploader_id
    );

    let response_json = json!({
        "state": "upload",
        "uploader": uploader_url
    });
    let llsd_body = json_to_llsd_xml(&response_json);
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<llsd>{}</llsd>",
        llsd_body
    );
    info!("[NOTECARD-SAVE] Stage-1 response XML: {}", xml);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(xml.into())
        .unwrap())
}

pub async fn handle_notecard_upload(
    Path((session_id, uploader_id)): Path<(String, String)>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "[NOTECARD-SAVE] Stage-2 CALLED: session={}, uploader={}, body_len={}",
        session_id,
        uploader_id,
        body.len()
    );

    if body.is_empty() {
        warn!("[NOTECARD-SAVE] Stage-2: EMPTY body received!");
    } else {
        let preview = String::from_utf8_lossy(&body[..std::cmp::min(200, body.len())]);
        info!("[NOTECARD-SAVE] Stage-2: body preview: {}", preview);
    }

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let (agent_id_str, pending) = {
        let mut sessions = state.caps_manager.sessions.write().await;
        let session = sessions.get_mut(&session_id).ok_or_else(|| {
            warn!("[NOTECARD-SAVE] Stage-2: session {} NOT FOUND", session_id);
            StatusCode::NOT_FOUND
        })?;
        let pending = session.pending_uploads.remove(&uploader_id);
        (session.agent_id.clone(), pending)
    };

    let pending = match pending {
        Some(p) => p,
        None => {
            warn!(
                "[NOTECARD-SAVE] Stage-2: No pending upload for uploader={} session={}",
                uploader_id, session_id
            );
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let item_id_str = &pending.name;
    let item_uuid = uuid::Uuid::parse_str(item_id_str).unwrap_or(uuid::Uuid::nil());
    let agent_uuid = uuid::Uuid::parse_str(&agent_id_str).map_err(|_| {
        error!(
            "[NOTECARD-SAVE] Stage-2: invalid agent_id: {}",
            agent_id_str
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let new_asset_id = uuid::Uuid::new_v4();

    info!(
        "[NOTECARD-SAVE] Stage-2: saving asset {} for item {} (agent={})",
        new_asset_id, item_uuid, agent_uuid
    );

    let asset_result = sqlx::query(
        r#"INSERT INTO assets (id, name, description, assettype, local, temporary, create_time, access_time, data, creatorid, asset_flags)
           VALUES ($1, $2, '', 7, 0, 0, $3, $3, $4, $5, 0)
           ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data"#
    )
    .bind(new_asset_id)
    .bind(format!("Notecard Item {}", item_uuid))
    .bind(chrono::Utc::now().timestamp() as i32)
    .bind(&body[..])
    .bind(agent_uuid.to_string())
    .execute(&*state.db_pool)
    .await;

    match &asset_result {
        Ok(r) => info!(
            "[NOTECARD-SAVE] Stage-2: asset INSERT OK, rows_affected={}",
            r.rows_affected()
        ),
        Err(e) => {
            error!("[NOTECARD-SAVE] Stage-2: asset INSERT FAILED: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if !item_uuid.is_nil() {
        let update_result =
            sqlx::query("UPDATE inventoryitems SET assetid = $1 WHERE inventoryid = $2")
                .bind(new_asset_id)
                .bind(item_uuid)
                .execute(&*state.db_pool)
                .await;

        match &update_result {
            Ok(r) => info!(
                "[NOTECARD-SAVE] Stage-2: inventory UPDATE OK, rows_affected={}",
                r.rows_affected()
            ),
            Err(e) => warn!("[NOTECARD-SAVE] Stage-2: inventory UPDATE FAILED: {}", e),
        }
    } else {
        warn!("[NOTECARD-SAVE] Stage-2: item_uuid is nil, skipping inventory update");
    }

    info!(
        "[NOTECARD-SAVE] Stage-2 COMPLETE: new_asset={} item={}",
        new_asset_id, item_uuid
    );

    json_response_to_llsd_xml(json!({
        "new_asset": new_asset_id.to_string(),
        "new_inventory_item": item_uuid.to_string(),
        "state": "complete",
        "upload_price": 0
    }))
}

pub async fn handle_update_script_agent_inventory(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    let body_str = String::from_utf8_lossy(&body);
    info!(
        "[SCRIPT-SAVE] Stage-1 request: session={} body={}",
        session_id, body_str
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let item_id = extract_llsd_uuid(&body_str, "item_id")
        .or_else(|| extract_llsd_string(&body_str, "item_id"))
        .unwrap_or_default();
    let target = extract_llsd_string(&body_str, "target").unwrap_or_else(|| "mono".to_string());

    info!(
        "[SCRIPT-SAVE] Stage-1: item_id={}, target={}",
        item_id, target
    );

    let uploader_id = uuid::Uuid::new_v4().to_string();

    {
        let mut sessions = state.caps_manager.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.pending_uploads.insert(
                uploader_id.clone(),
                PendingUpload {
                    name: item_id.clone(),
                    description: String::new(),
                    asset_type: "lsltext".to_string(),
                    inventory_type: "script".to_string(),
                    folder_id: String::new(),
                    task_id: None,
                    is_script_running: false,
                },
            );
        }
    }

    let uploader_url = format!(
        "{}/cap/{}/ScriptUpload/{}",
        state.caps_manager.base_url, session_id, uploader_id
    );

    let response_json = json!({
        "state": "upload",
        "uploader": uploader_url
    });
    let llsd_body = json_to_llsd_xml(&response_json);
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<llsd>{}</llsd>",
        llsd_body
    );
    info!("[SCRIPT-SAVE] Stage-1 response XML: {}", xml);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(xml.into())
        .unwrap())
}

pub async fn handle_script_upload(
    Path((session_id, uploader_id)): Path<(String, String)>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "[SCRIPT-SAVE] Stage-2 CALLED: session={}, uploader={}, body_len={}",
        session_id,
        uploader_id,
        body.len()
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let (agent_id_str, pending) = {
        let mut sessions = state.caps_manager.sessions.write().await;
        let session = sessions.get_mut(&session_id).ok_or_else(|| {
            warn!("[SCRIPT-SAVE] Stage-2: session {} NOT FOUND", session_id);
            StatusCode::NOT_FOUND
        })?;
        let pending = session.pending_uploads.remove(&uploader_id);
        (session.agent_id.clone(), pending)
    };

    let pending = match pending {
        Some(p) => p,
        None => {
            warn!(
                "[SCRIPT-SAVE] Stage-2: No pending upload for uploader={}",
                uploader_id
            );
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let item_id_str = &pending.name;
    let item_uuid = uuid::Uuid::parse_str(item_id_str).unwrap_or(uuid::Uuid::nil());
    let agent_uuid =
        uuid::Uuid::parse_str(&agent_id_str).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let new_asset_id = uuid::Uuid::new_v4();

    info!(
        "[SCRIPT-SAVE] Stage-2: saving asset {} for item {} (agent={})",
        new_asset_id, item_uuid, agent_uuid
    );

    let asset_result = sqlx::query(
        r#"INSERT INTO assets (id, name, description, assettype, local, temporary, create_time, access_time, data, creatorid, asset_flags)
           VALUES ($1, $2, '', 10, 0, 0, $3, $3, $4, $5, 0)
           ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data"#
    )
    .bind(new_asset_id)
    .bind(format!("Script Item {}", item_uuid))
    .bind(chrono::Utc::now().timestamp() as i32)
    .bind(&body[..])
    .bind(agent_uuid.to_string())
    .execute(&*state.db_pool)
    .await;

    match &asset_result {
        Ok(r) => info!(
            "[SCRIPT-SAVE] Stage-2: asset INSERT OK, rows_affected={}",
            r.rows_affected()
        ),
        Err(e) => {
            error!("[SCRIPT-SAVE] Stage-2: asset INSERT FAILED: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if !item_uuid.is_nil() {
        let update_result =
            sqlx::query("UPDATE inventoryitems SET assetid = $1 WHERE inventoryid = $2")
                .bind(new_asset_id)
                .bind(item_uuid)
                .execute(&*state.db_pool)
                .await;

        match &update_result {
            Ok(r) => info!(
                "[SCRIPT-SAVE] Stage-2: inventory UPDATE OK, rows_affected={}",
                r.rows_affected()
            ),
            Err(e) => warn!("[SCRIPT-SAVE] Stage-2: inventory UPDATE FAILED: {}", e),
        }
    }

    info!(
        "[SCRIPT-SAVE] Stage-2 COMPLETE: new_asset={} item={}",
        new_asset_id, item_uuid
    );

    json_response_to_llsd_xml(json!({
        "new_asset": new_asset_id.to_string(),
        "new_inventory_item": item_uuid.to_string(),
        "state": "complete",
        "upload_price": 0
    }))
}

pub async fn handle_update_script_task_inventory(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    let body_str = String::from_utf8_lossy(&body);
    info!(
        "[SCRIPT-TASK] Stage-1 request: session={} body={}",
        session_id, body_str
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let item_id = extract_llsd_uuid(&body_str, "item_id")
        .or_else(|| extract_llsd_string(&body_str, "item_id"))
        .unwrap_or_default();
    let task_id = extract_llsd_uuid(&body_str, "task_id")
        .or_else(|| extract_llsd_string(&body_str, "task_id"))
        .unwrap_or_default();
    let is_script_running = extract_llsd_string(&body_str, "is_script_running")
        .map(|s| s == "true" || s == "1")
        .unwrap_or(true);

    info!(
        "[SCRIPT-TASK] Stage-1: item_id={}, task_id={}, running={}",
        item_id, task_id, is_script_running
    );

    let uploader_id = uuid::Uuid::new_v4().to_string();

    {
        let mut sessions = state.caps_manager.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.pending_uploads.insert(
                uploader_id.clone(),
                PendingUpload {
                    name: item_id.clone(),
                    description: String::new(),
                    asset_type: "lsltext".to_string(),
                    inventory_type: "script".to_string(),
                    folder_id: String::new(),
                    task_id: Some(task_id),
                    is_script_running,
                },
            );
        }
    }

    let uploader_url = format!(
        "{}/cap/{}/ScriptTaskUpload/{}",
        state.caps_manager.base_url, session_id, uploader_id
    );

    let response_json = json!({
        "state": "upload",
        "uploader": uploader_url
    });
    let llsd_body = json_to_llsd_xml(&response_json);
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<llsd>{}</llsd>",
        llsd_body
    );
    info!("[SCRIPT-TASK] Stage-1 response: uploader={}", uploader_url);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(xml.into())
        .unwrap())
}

pub async fn handle_update_notecard_task_inventory(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    let body_str = String::from_utf8_lossy(&body);
    info!(
        "[NOTECARD-TASK] Stage-1 request: session={} body={}",
        session_id, body_str
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let item_id = extract_llsd_uuid(&body_str, "item_id")
        .or_else(|| extract_llsd_string(&body_str, "item_id"))
        .unwrap_or_default();
    let task_id = extract_llsd_uuid(&body_str, "task_id")
        .or_else(|| extract_llsd_string(&body_str, "task_id"))
        .unwrap_or_default();

    info!(
        "[NOTECARD-TASK] Stage-1: item_id={}, task_id={}",
        item_id, task_id
    );

    let uploader_id = uuid::Uuid::new_v4().to_string();

    {
        let mut sessions = state.caps_manager.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.pending_uploads.insert(
                uploader_id.clone(),
                PendingUpload {
                    name: item_id.clone(),
                    description: String::new(),
                    asset_type: "notecard".to_string(),
                    inventory_type: "notecard".to_string(),
                    folder_id: String::new(),
                    task_id: Some(task_id),
                    is_script_running: false,
                },
            );
        }
    }

    let uploader_url = format!(
        "{}/cap/{}/ScriptTaskUpload/{}",
        state.caps_manager.base_url, session_id, uploader_id
    );

    let response_json = json!({
        "state": "upload",
        "uploader": uploader_url
    });
    let llsd_body = json_to_llsd_xml(&response_json);
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<llsd>{}</llsd>",
        llsd_body
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(xml.into())
        .unwrap())
}

pub async fn handle_script_task_upload(
    Path((session_id, uploader_id)): Path<(String, String)>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "[SCRIPT-TASK] Stage-2 CALLED: session={}, uploader={}, body_len={}",
        session_id,
        uploader_id,
        body.len()
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let pending = {
        let mut sessions = state.caps_manager.sessions.write().await;
        let session = sessions.get_mut(&session_id).ok_or_else(|| {
            warn!("[SCRIPT-TASK] Stage-2: session {} NOT FOUND", session_id);
            StatusCode::NOT_FOUND
        })?;
        session.pending_uploads.remove(&uploader_id)
    };

    let pending = match pending {
        Some(p) => p,
        None => {
            warn!(
                "[SCRIPT-TASK] Stage-2: No pending upload for uploader={}",
                uploader_id
            );
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let item_id_str = &pending.name;
    let item_uuid = uuid::Uuid::parse_str(item_id_str).unwrap_or(uuid::Uuid::nil());
    let task_uuid = pending
        .task_id
        .as_deref()
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .unwrap_or(uuid::Uuid::nil());
    let new_asset_id = uuid::Uuid::new_v4();
    let is_notecard = pending.asset_type == "notecard";
    let asset_type_int: i32 = if is_notecard { 7 } else { 10 };

    info!(
        "[SCRIPT-TASK] Stage-2: saving asset {} for item {} in prim {} (type={})",
        new_asset_id, item_uuid, task_uuid, pending.asset_type
    );

    let asset_result = sqlx::query(
        r#"INSERT INTO assets (id, name, description, assettype, local, temporary, create_time, access_time, data, creatorid, asset_flags)
           VALUES ($1, $2, '', $3, 0, 0, $4, $4, $5, $6, 0)
           ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data"#
    )
    .bind(new_asset_id)
    .bind(format!("{} Item {}", if is_notecard { "Notecard" } else { "Script" }, item_uuid))
    .bind(asset_type_int)
    .bind(chrono::Utc::now().timestamp() as i32)
    .bind(&body[..])
    .bind(uuid::Uuid::nil().to_string())
    .execute(&*state.db_pool)
    .await;

    match &asset_result {
        Ok(r) => info!(
            "[SCRIPT-TASK] Stage-2: asset INSERT OK, rows_affected={}",
            r.rows_affected()
        ),
        Err(e) => {
            error!("[SCRIPT-TASK] Stage-2: asset INSERT FAILED: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if !item_uuid.is_nil() && !task_uuid.is_nil() {
        let update_result =
            sqlx::query("UPDATE primitems SET assetid = $1 WHERE itemid = $2 AND primid = $3")
                .bind(new_asset_id)
                .bind(item_uuid)
                .bind(task_uuid)
                .execute(&*state.db_pool)
                .await;

        match &update_result {
            Ok(r) => info!(
                "[SCRIPT-TASK] Stage-2: primitems UPDATE OK, rows_affected={}",
                r.rows_affected()
            ),
            Err(e) => warn!("[SCRIPT-TASK] Stage-2: primitems UPDATE FAILED: {}", e),
        }
    }

    info!(
        "[SCRIPT-TASK] Stage-2 COMPLETE: new_asset={} item={} prim={}",
        new_asset_id, item_uuid, task_uuid
    );

    if asset_type_int == 10 {
        let script_id = uuid::Uuid::new_v5(&task_uuid, item_uuid.as_bytes());
        if let Some(ref yengine) = state.yengine {
            let source = String::from_utf8_lossy(&body).replace('\0', "");
            let engine = yengine.read();
            match engine.rez_script(script_id, &source, 1) {
                Ok(()) => {
                    if let Some(ref scene_objects) = state.scene_objects {
                        let objects = scene_objects.read();
                        if let Some(obj) = objects.values().find(|o| o.uuid == task_uuid) {
                            let group_id = obj.scene_group_id;
                            let mut link_names = Vec::new();
                            let mut link_scales = Vec::new();
                            let mut link_count = 1i32;
                            if group_id != uuid::Uuid::nil() {
                                let siblings: Vec<_> = objects
                                    .values()
                                    .filter(|s| s.scene_group_id == group_id)
                                    .collect();
                                link_count = siblings.len() as i32;
                                for s in &siblings {
                                    link_names.push((s.link_number, s.name.clone()));
                                    link_scales.push((
                                        s.link_number,
                                        crate::scripting::LSLVector::new(
                                            s.scale[0], s.scale[1], s.scale[2],
                                        ),
                                    ));
                                }
                            }
                            let ctx = crate::scripting::executor::ObjectContext {
                                object_id: obj.uuid,
                                owner_id: obj.owner_id,
                                object_name: obj.name.clone(),
                                position: crate::scripting::LSLVector::new(
                                    obj.position[0],
                                    obj.position[1],
                                    obj.position[2],
                                ),
                                rotation: crate::scripting::LSLRotation::new(
                                    obj.rotation[0],
                                    obj.rotation[1],
                                    obj.rotation[2],
                                    obj.rotation[3],
                                ),
                                scale: crate::scripting::LSLVector::new(
                                    obj.scale[0],
                                    obj.scale[1],
                                    obj.scale[2],
                                ),
                                velocity: crate::scripting::LSLVector::zero(),
                                region_name: String::new(),
                                detect_params: Vec::new(),
                                granted_perms: 0,
                                perm_granter: uuid::Uuid::nil(),
                                sitting_avatar_id: obj.sitting_avatar.unwrap_or(uuid::Uuid::nil()),
                                link_num: obj.link_number,
                                link_count,
                                link_names,
                                link_scales,
                                inventory: Vec::new(),
                                base_mask: obj.base_mask,
                                owner_mask: obj.owner_mask,
                                group_mask: obj.group_mask,
                                everyone_mask: obj.everyone_mask,
                                next_owner_mask: obj.next_owner_mask,
                            };
                            engine.set_script_context(script_id, ctx);
                        }
                    }
                    info!("[SCRIPT-TASK] Script compiled and started in YEngine: script_id={} item={} prim={}", script_id, item_uuid, task_uuid);
                }
                Err(e) => warn!(
                    "[SCRIPT-TASK] Failed to start script in YEngine: {} (script_id={})",
                    e, script_id
                ),
            }
        }
        if let Some(ref scene_objects) = state.scene_objects {
            let mut objects = scene_objects.write();
            if let Some(obj) = objects.values_mut().find(|o| o.uuid == task_uuid) {
                if !obj.script_items.contains(&script_id) {
                    obj.script_items.push(script_id);
                    info!(
                        "[SCRIPT-TASK] Added script {} to prim {} script_items cache (item={})",
                        script_id, task_uuid, item_uuid
                    );
                }
            }
        }
    }

    json_response_to_llsd_xml(json!({
        "new_asset": new_asset_id.to_string(),
        "compiled": true,
        "state": "complete",
        "errors": []
    }))
}

pub async fn handle_parcel_properties_update(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    if body.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(axum::body::Body::empty())
            .unwrap());
    }

    let body_str = String::from_utf8_lossy(&body);
    info!(
        "[PARCEL] ParcelPropertiesUpdate CAPS for session {}: {} bytes",
        session_id,
        body.len()
    );

    let local_id = match extract_llsd_integer(&body_str, "local_id") {
        Some(id) => id,
        None => {
            warn!("[PARCEL] ParcelPropertiesUpdate missing local_id");
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(axum::body::Body::empty())
                .unwrap());
        }
    };

    let name = extract_llsd_string(&body_str, "name").unwrap_or_default();
    let description = extract_llsd_string(&body_str, "description").unwrap_or_default();
    let music_url = extract_llsd_string(&body_str, "music_url").unwrap_or_default();
    let media_url = extract_llsd_string(&body_str, "media_url").unwrap_or_default();
    let parcel_flags = extract_llsd_u32(&body_str, "parcel_flags").unwrap_or(0);
    let sale_price = extract_llsd_u32(&body_str, "sale_price").unwrap_or(0) as i32;
    let landing_type = extract_llsd_integer(&body_str, "landing_type").unwrap_or(0) as u8;
    let category = extract_llsd_integer(&body_str, "category").unwrap_or(0) as u8;
    let pass_price = extract_llsd_u32(&body_str, "pass_price").unwrap_or(0) as i32;
    let pass_hours = extract_llsd_real(&body_str, "pass_hours").unwrap_or(0.0) as f32;
    let user_location = extract_llsd_vector3(&body_str, "user_location").unwrap_or([0.0; 3]);
    let user_look_at = extract_llsd_vector3(&body_str, "user_look_at").unwrap_or([0.0; 3]);

    info!("[PARCEL] CAPS update parcel local_id={} name='{}' flags=0x{:08X} sale_price={} landing=({:.1},{:.1},{:.1}) type={}",
          local_id, name, parcel_flags, sale_price,
          user_location[0], user_location[1], user_location[2], landing_type);

    let region_uuid_str = if let Some(ref parcels_lock) = state.parcels {
        let parcels = parcels_lock.read();
        parcels
            .iter()
            .find(|p| p.local_id == local_id)
            .map(|p| p.region_uuid.to_string())
    } else {
        None
    };

    let region_uuid_for_db = region_uuid_str.unwrap_or_default();
    if region_uuid_for_db.is_empty() {
        warn!("[PARCEL] No parcel found with local_id={}", local_id);
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::empty())
            .unwrap());
    }

    let result = sqlx::query(
        r#"UPDATE land SET name = $1, description = $2, musicurl = $3, landflags = $4, saleprice = $5,
           userlocationx = $8, userlocationy = $9, userlocationz = $10,
           userlookatx = $11, userlookaty = $12, userlookatz = $13,
           landingtype = $14, category = $15, passprice = $16, passhours = $17
           WHERE locallandid = $6 AND regionuuid = $7::uuid"#
    )
    .bind(&name)
    .bind(&description)
    .bind(&music_url)
    .bind(parcel_flags as i32)
    .bind(sale_price)
    .bind(local_id)
    .bind(&region_uuid_for_db)
    .bind(user_location[0])
    .bind(user_location[1])
    .bind(user_location[2])
    .bind(user_look_at[0])
    .bind(user_look_at[1])
    .bind(user_look_at[2])
    .bind(landing_type as i32)
    .bind(category as i32)
    .bind(pass_price)
    .bind(pass_hours)
    .execute(state.db_pool.as_ref())
    .await;

    match &result {
        Ok(r) => info!(
            "[PARCEL] CAPS updated parcel local_id={} in DB ({} rows)",
            local_id,
            r.rows_affected()
        ),
        Err(e) => warn!(
            "[PARCEL] CAPS failed to update parcel local_id={}: {}",
            local_id, e
        ),
    }

    if let Some(ref parcels_lock) = state.parcels {
        let mut parcels = parcels_lock.write();
        if let Some(p) = parcels.iter_mut().find(|p| p.local_id == local_id) {
            if !name.is_empty() {
                p.name = name.clone();
            }
            if !description.is_empty() {
                p.description = description.clone();
            }
            if !music_url.is_empty() {
                p.music_url = music_url.clone();
            }
            p.flags = parcel_flags;
            p.sale_price = sale_price;
            p.landing_point = user_location;
            p.landing_look_at = user_look_at;
            p.landing_type = landing_type;
            p.category = category;
            p.pass_price = pass_price;
            p.pass_hours = pass_hours;
        }
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(axum::body::Body::empty())
        .unwrap())
}

// Map and simulator handlers
pub async fn handle_map_layer(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("[MAP] MapLayer CAPS request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    json_response_to_llsd_xml(json!({
        "AgentData": {"Flags": 0},
        "LayerData": [{
            "Left": 0,
            "Right": 30000,
            "Top": 30000,
            "Bottom": 0,
            "ImageID": "00000000-0000-1111-9999-000000000006"
        }]
    }))
}

pub async fn handle_simulator_features(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("🔧 SimulatorFeatures request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let login_port = std::env::var("OPENSIM_LOGIN_PORT").unwrap_or_else(|_| "9000".to_string());
    let sys_ip = crate::config::login::resolve_system_ip();
    let default_base = format!("http://{}:{}", sys_ip, login_port);
    let grid_name = std::env::var("OPENSIM_GRID_NAME")
        .unwrap_or_else(|_| "OpenSim Next".to_string())
        .trim_matches('"')
        .to_string();
    let grid_url = std::env::var("OPENSIM_GATEKEEPER_URI").unwrap_or_else(|_| default_base.clone());
    let map_url =
        std::env::var("OPENSIM_MAP_SERVER_URL").unwrap_or_else(|_| format!("{}/", default_base));
    let search_url =
        std::env::var("OPENSIM_SEARCH_URL").unwrap_or_else(|_| format!("{}/search", default_base));
    let home_uri = std::env::var("OPENSIM_HOME_URI").unwrap_or_else(|_| default_base.clone());
    let currency_base = format!("{}/currency.php", home_uri.trim_end_matches('/'));
    let dest_guide = std::env::var("OPENSIM_DESTINATION_GUIDE_URL").unwrap_or_default();

    let features = json!({
        "AnimatedObjects": {
            "AnimatedObjectMaxTris": 150000,
            "MaxAgentAnimatedObjectAttachments": 2
        },
        "BakesOnMeshEnabled": true,
        "MaxAgentAttachments": 38,
        "MaxAgentGroups": 42,
        "MaxAgentGroupsBasic": 42,
        "MaxAgentGroupsPremium": 42,
        "MaxEstateAccessIds": 500,
        "MaxEstateManagers": 20,
        "MaxTextureResolution": 2048,
        "MaxProfilePicks": 100,
        "MeshRezEnabled": true,
        "MeshUploadEnabled": true,
        "MeshXferEnabled": true,
        "MirrorsEnabled": true,
        "PhysicsMaterialsEnabled": true,
        "PhysicsShapeTypes": {
            "convex": true,
            "none": true,
            "prim": true
        },
        "OpenSimExtras": {
            "AvatarSkeleton": true,
            "AnimationSet": true,
            "MinSimHeight": -100.0,
            "MaxSimHeight": 10000.0,
            "MinHeightmap": -100.0,
            "MaxHeightmap": 4096.0,
            "GridName": grid_name,
            "GridURL": grid_url,
            "ExportSupported": true,
            "map-server-url": map_url,
            "search-server-url": search_url,
            "say-range": 20,
            "shout-range": 100,
            "whisper-range": 10,
            "currency-base-uri": currency_base,
            "destination-guide-url": dest_guide
        }
    });

    info!(
        "🔧 SimulatorFeatures returning full feature set for session: {}",
        session_id
    );
    json_response_to_llsd_xml(features)
}

pub async fn handle_simulator_features_post(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    _body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "🔧 SimulatorFeatures POST request for session: {} (forwarding to GET handler)",
        session_id
    );
    handle_simulator_features(Path(session_id), State(state)).await
}

// Environment handlers - CRITICAL for login completion
pub async fn handle_environment_settings_get(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!(
        "🌍 EnvironmentSettings GET request for session: {}",
        session_id
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    // Return default environment settings with the known UUID
    let environment_response = json!({
        "success": true,
        "environment_id": "5646d39e-d3d7-6aff-ed71-30fc87d64a91",
        "environment_settings": {
            "day_cycle": {
                "sun_position": 0.0,
                "moon_position": 0.0,
                "east_angle": 0.0,
                "sun_angle": 1.0
            },
            "water": {
                "water_fog_color": [0.4, 0.6, 0.85],
                "water_fog_density": 2.0,
                "underwater_fog_mod": 0.25
            },
            "sky": {
                "ambient": [0.3, 0.3, 0.4],
                "blue_density": [0.2447, 0.4487, 0.7599],
                "blue_horizon": [0.4954, 0.4954, 0.6399],
                "cloud_color": [0.41, 0.41, 0.41],
                "cloud_pos_density1": [1.0, 0.53, 1.0],
                "cloud_pos_density2": [1.0, 0.53, 1.0],
                "cloud_scale": [0.42, 0.0, 0.0],
                "cloud_scroll_x": [0.2, 0.01],
                "cloud_scroll_y": [0.2, 0.01],
                "cloud_shadow": [0.27, 0.0, 0.0],
                "density_multiplier": [0.0001, 0.0, 0.0],
                "distance_multiplier": [0.8, 0.0, 0.0],
                "gamma": [1.0, 0.0, 0.0],
                "glow": [5.0, 0.001, -0.48],
                "haze_density": [0.7, 0.0, 0.0],
                "haze_horizon": [0.19, 0.19, 0.19],
                "lightnorm": [0.0, 0.0, 0.0],
                "max_y": [1605, 0.0, 0.0],
                "sun_angle": 2.14159,
                "sun_color": [0.24, 0.26, 0.30],
                "sunlight_color": [0.73, 0.73, 0.65]
            }
        }
    });

    info!(
        "🌍 Returning default environment settings for session: {} (LLSD XML)",
        session_id
    );
    json_response_to_llsd_xml(environment_response)
}

pub async fn handle_environment_settings_post(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "🌍 EnvironmentSettings POST request for session: {}",
        session_id
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    if !body.is_empty() {
        let body_str = String::from_utf8_lossy(&body);
        info!("🌍 EnvironmentSettings update: {}", body_str);
    }

    json_response_to_llsd_xml(json!({
        "success": true,
        "capability": "EnvironmentSettings",
        "environment_id": "5646d39e-d3d7-6aff-ed71-30fc87d64a91"
    }))
}

pub async fn handle_ext_environment_get(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("🌍 ExtEnvironment GET request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    // Return extended environment data
    let ext_environment_response = json!({
        "success": true,
        "environment_id": "5646d39e-d3d7-6aff-ed71-30fc87d64a91",
        "environment_version": 1,
        "environment_data": {
            "type": "environment",
            "name": "Default Environment",
            "legacy": true,
            "settings": {
                "flags": {
                    "use_estate_sun": true,
                    "use_fixed_sun": false,
                    "use_region_sun": false
                },
                "day_length": 14400,
                "day_offset": 0,
                "environment_version": 1
            }
        }
    });

    info!(
        "🌍 Returning extended environment data for session: {} (LLSD XML)",
        session_id
    );
    json_response_to_llsd_xml(ext_environment_response)
}

pub async fn handle_ext_environment_post(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!("🌍 ExtEnvironment POST request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    if !body.is_empty() {
        let body_str = String::from_utf8_lossy(&body);
        info!("🌍 ExtEnvironment update: {}", body_str);
    }

    json_response_to_llsd_xml(json!({
        "success": true,
        "capability": "ExtEnvironment",
        "environment_id": "5646d39e-d3d7-6aff-ed71-30fc87d64a91"
    }))
}

pub async fn handle_upload_baked_texture(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "🎨 UploadBakedTexture POST request for session: {} ({} bytes)",
        session_id,
        body.len()
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let uploader_uuid = uuid::Uuid::new_v4().to_string();
    let uploader_url = format!(
        "{}/cap/{}/BakedTextureUpload/{}",
        state.caps_manager.base_url, session_id, uploader_uuid
    );

    info!("🎨 Created baked texture uploader URL: {}", uploader_url);

    let upload_response = json!({
        "uploader": uploader_url,
        "state": "upload"
    });

    json_response_to_llsd_xml(upload_response)
}

pub async fn handle_baked_texture_data(
    Path((session_id, uploader_id)): Path<(String, String)>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let new_asset_id = uuid::Uuid::new_v4();

    let min_bake_size = crate::avatar::factory::MIN_BAKE_TEXTURE_SIZE;
    if !body.is_empty() {
        state
            .caps_manager
            .store_baked_texture(new_asset_id, body.to_vec());
        info!(
            "Baked texture {} stored in memory cache ({} bytes)",
            new_asset_id,
            body.len()
        );

        if body.len() >= min_bake_size {
            let texture_data = body.to_vec();
            let pool = state.db_pool.clone();
            let db_asset_id = new_asset_id;
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let result = sqlx::query(
                    r#"INSERT INTO assets (id, name, description, assettype, local, temporary, data, create_time, access_time, asset_flags, creatorid)
                       VALUES ($1, 'baked_texture', 'Baked texture', 0, 1, 1, $2, $3, $3, 0, '')
                       ON CONFLICT (id) DO NOTHING"#
                )
                .bind(db_asset_id)
                .bind(&texture_data)
                .bind(now)
                .execute(pool.as_ref())
                .await;
                if let Err(e) = result {
                    tracing::warn!("Failed to persist baked texture to DB: {}", e);
                }
            });
        } else {
            info!(
                "Baked texture {} too small ({} bytes < {}) — memory-only, not persisting to DB",
                new_asset_id,
                body.len(),
                min_bake_size
            );
        }
    } else {
        warn!(
            "🎨 Empty baked texture upload received for session: {}",
            session_id
        );
    }

    if body.len() >= min_bake_size {
        if let Some(ref factory) = state.avatar_factory {
            if let Some(session) = state.caps_manager.get_session(&session_id).await {
                if let Ok(agent_id) = uuid::Uuid::parse_str(&session.agent_id) {
                    factory.cache_texture(new_asset_id).await;
                }
            }
        }
    }

    let complete_response = json!({
        "new_asset": new_asset_id.to_string(),
        "new_inventory_item": "00000000-0000-0000-0000-000000000000",
        "state": "complete"
    });

    json_response_to_llsd_xml(complete_response)
}

pub async fn handle_get_mesh(
    Path(session_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<CapsHandlerState>,
    headers: axum::http::HeaderMap,
) -> Result<Response, StatusCode> {
    use crate::opensim_compatibility::library_assets::get_global_library_manager;

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let mesh_id = match params.get("mesh_id") {
        Some(id) => id.clone(),
        None => {
            warn!("GetMesh: No mesh_id provided");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let mesh_uuid = match uuid::Uuid::parse_str(&mesh_id) {
        Ok(u) => u,
        Err(_) => {
            warn!("GetMesh: Invalid mesh UUID: {}", mesh_id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let data: Option<Vec<u8>> = {
        let fetch_result = if let Some(ref fetcher) = state.asset_fetcher {
            fetcher
                .fetch_asset_data_typed_pg(&mesh_uuid.to_string(), Some(49), &state.db_pool)
                .await
        } else {
            let row: Result<Option<Vec<u8>>, sqlx::Error> =
                sqlx::query_scalar("SELECT data FROM assets WHERE id = $1 AND assettype = 49")
                    .bind(mesh_uuid)
                    .fetch_optional(&*state.db_pool)
                    .await;
            row.map_err(|e| anyhow::anyhow!("{}", e))
        };

        match fetch_result {
            Ok(Some(d)) if !d.is_empty() => Some(d),
            Ok(_) => None,
            Err(e) => {
                warn!("GetMesh: fetch error for {}: {}", mesh_id, e);
                None
            }
        }
    };

    let data = match data {
        Some(d) => d,
        None => {
            if let Some(library_manager) = get_global_library_manager() {
                let manager = library_manager.read().await;
                match manager.get_asset_data(&mesh_uuid) {
                    Some(d) => d,
                    None => {
                        info!("GetMesh: mesh {} not found", mesh_id);
                        return Err(StatusCode::NOT_FOUND);
                    }
                }
            } else {
                return Err(StatusCode::NOT_FOUND);
            }
        }
    };

    let total_len = data.len();
    let first_byte = if data.is_empty() { 0u8 } else { data[0] };

    if let Some(range_header) = headers.get("Range").and_then(|v| v.to_str().ok()) {
        if let Some(range_str) = range_header.strip_prefix("bytes=") {
            let parts: Vec<&str> = range_str.splitn(2, '-').collect();
            if parts.len() == 2 {
                let start = parts[0].parse::<usize>().unwrap_or(0);
                let end = if parts[1].is_empty() {
                    total_len - 1
                } else {
                    parts[1]
                        .parse::<usize>()
                        .unwrap_or(total_len - 1)
                        .min(total_len - 1)
                };

                if start < total_len && start <= end {
                    let slice = &data[start..=end];
                    info!(
                        "GetMesh RANGE: {} bytes={}-{} total={} serving={} first_byte=0x{:02x}",
                        mesh_id,
                        start,
                        end,
                        total_len,
                        slice.len(),
                        first_byte
                    );
                    return Ok(Response::builder()
                        .status(206)
                        .header("Content-Type", "application/vnd.ll.mesh")
                        .header(
                            "Content-Range",
                            format!("bytes {}-{}/{}", start, end, total_len),
                        )
                        .header("Content-Length", slice.len().to_string())
                        .header("Accept-Ranges", "bytes")
                        .body(axum::body::Body::from(slice.to_vec()))
                        .unwrap());
                }
            }
        }
    }

    info!(
        "GetMesh FULL: {} total={} first_byte=0x{:02x}",
        mesh_id, total_len, first_byte
    );
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/vnd.ll.mesh")
        .header("Content-Length", total_len.to_string())
        .header("Accept-Ranges", "bytes")
        .body(axum::body::Body::from(data))
        .unwrap())
}

pub async fn handle_viewer_asset(
    Path(session_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<CapsHandlerState>,
    headers: axum::http::HeaderMap,
) -> Result<Response, StatusCode> {
    use crate::opensim_compatibility::avatar_data::get_body_part_data_by_uuid;
    use crate::opensim_compatibility::library_assets::get_global_library_manager;

    let range_request = headers
        .get("Range")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("bytes="))
        .and_then(|range_str| {
            let parts: Vec<&str> = range_str.splitn(2, '-').collect();
            if parts.len() == 2 {
                let start = parts[0].parse::<usize>().ok()?;
                let end_str = parts[1];
                Some((start, end_str.to_string()))
            } else {
                None
            }
        });

    let range_debug = range_request
        .as_ref()
        .map(|(s, e)| format!("bytes={}-{}", s, e));
    let build_range_response = |data: Vec<u8>, content_type: &str| -> Response {
        let total_len = data.len();
        let first_byte = if data.is_empty() { 0u8 } else { data[0] };
        if let Some((start, ref end_str)) = range_request {
            let end = if end_str.is_empty() {
                total_len.saturating_sub(1)
            } else {
                end_str
                    .parse::<usize>()
                    .unwrap_or(total_len - 1)
                    .min(total_len.saturating_sub(1))
            };
            if start < total_len && start <= end {
                let slice = data[start..=end].to_vec();
                info!(
                    "📦 RANGE {}: {}-{}/{} serving={} first=0x{:02x} ct={}",
                    content_type,
                    start,
                    end,
                    total_len,
                    slice.len(),
                    first_byte,
                    content_type
                );
                return Response::builder()
                    .status(StatusCode::PARTIAL_CONTENT)
                    .header("Content-Type", content_type)
                    .header(
                        "Content-Range",
                        format!("bytes {}-{}/{}", start, end, total_len),
                    )
                    .header("Content-Length", slice.len().to_string())
                    .header("Accept-Ranges", "bytes")
                    .body(axum::body::Body::from(slice))
                    .unwrap()
                    .into_response();
            }
        }
        info!(
            "📦 FULL {}: total={} first=0x{:02x} ct={}",
            content_type, total_len, first_byte, content_type
        );
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", content_type)
            .header("Content-Length", total_len.to_string())
            .header("Accept-Ranges", "bytes")
            .body(axum::body::Body::from(data))
            .unwrap()
            .into_response()
    };

    info!(
        "📦 ViewerAsset request for session: {} with params: {:?}",
        session_id, params
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let query_param_names = [
        "texture_id",  // AssetType.Texture (0)
        "sound_id",    // AssetType.Sound (1)
        "callcard_id", // AssetType.CallingCard (2)
        "landmark_id", // AssetType.Landmark (3)
        "clothing_id", // AssetType.Clothing (5)
        "object_id",   // AssetType.Object (6)
        "notecard_id", // AssetType.Notecard (7)
        "script_id",   // AssetType.LSLText (10)
        "lsltext_id",  // AssetType.LSLText (10)
        "txtr_tga_id", // AssetType.TextureTGA (12)
        "bodypart_id", // AssetType.Bodypart (13)
        "img_tga_id",  // AssetType.ImageTGA (14)
        "jpeg_id",     // AssetType.ImageJPEG (15)
        "lslbyte_id",  // AssetType.LSLBytecode (16)
        "snd_wav_id",  // AssetType.SoundWAV (17)
        "animatn_id",  // AssetType.Animation (20)
        "gesture_id",  // AssetType.Gesture (21)
        "mesh_id",     // AssetType.Mesh (49)
        "settings_id", // AssetType.Settings (56)
        "material_id", // AssetType.Material (57)
        "asset_id",    // Generic fallback
    ];

    let mut asset_id_str: Option<&String> = None;
    let mut param_used = "unknown";

    for param_name in &query_param_names {
        if let Some(id_str) = params.get(*param_name) {
            asset_id_str = Some(id_str);
            param_used = param_name;
            break;
        }
    }

    let asset_id = match asset_id_str {
        Some(id_str) => match uuid::Uuid::parse_str(id_str) {
            Ok(uuid) => {
                info!("📦 ViewerAsset: Found {} = {}", param_used, uuid);
                uuid
            }
            Err(_) => {
                warn!(
                    "📦 ViewerAsset: Invalid asset UUID in {}: {}",
                    param_used, id_str
                );
                return Err(StatusCode::BAD_REQUEST);
            }
        },
        None => {
            warn!(
                "📦 ViewerAsset: No recognized asset parameter provided. Params: {:?}",
                params.keys().collect::<Vec<_>>()
            );
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    #[derive(sqlx::FromRow)]
    struct AssetRow {
        data: Vec<u8>,
        asset_type: i32,
    }

    let result: Result<Option<AssetRow>, sqlx::Error> =
        sqlx::query_as("SELECT data, assettype as asset_type FROM assets WHERE id = $1")
            .bind(asset_id)
            .fetch_optional(&*state.db_pool)
            .await;

    match result {
        Ok(Some(row)) if !row.data.is_empty() => {
            let content_type = get_asset_content_type(row.asset_type);
            info!(
                "📦 Serving asset {} from database (type={}, {} bytes)",
                asset_id,
                row.asset_type,
                row.data.len()
            );
            return Ok(build_range_response(row.data, content_type));
        }
        Ok(_) => {}
        Err(e) => {
            warn!("📦 Database error for asset {}: {}", asset_id, e);
        }
    }

    if let Some(library_manager) = get_global_library_manager() {
        let manager = library_manager.read().await;
        if let Some(asset) = manager.get_asset(&asset_id) {
            if let Some(data) = manager.get_asset_data(&asset_id) {
                let content_type = get_asset_content_type(asset.asset_type as i32);
                info!(
                    "📦 Serving asset {} from library (type={}, {} bytes)",
                    asset_id,
                    asset.asset_type,
                    data.len()
                );
                return Ok(build_range_response(data, content_type));
            }
        }
    }

    if let Some(data) = get_body_part_data_by_uuid(&asset_id).await {
        info!(
            "📦 Serving body part asset {} ({} bytes)",
            asset_id,
            data.len()
        );
        return Ok(build_range_response(data, "application/vnd.ll.bodypart"));
    }

    if let Some(data) = state.caps_manager.get_baked_texture(&asset_id) {
        info!(
            "📦 Serving texture {} from baked texture cache ({} bytes)",
            asset_id,
            data.len()
        );
        return Ok(build_range_response(data, "image/x-j2c"));
    }

    warn!("📦 ViewerAsset not found in any source: {}", asset_id);
    Err(StatusCode::NOT_FOUND)
}

pub async fn handle_mesh_upload_flag(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("🔷 MeshUploadFlag request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let mesh_upload_response = json!({
        "mesh_upload_status": "valid",
        "cost": {
            "mesh": 0,
            "texture": 0,
            "normal": 0,
            "specular": 0
        }
    });

    json_response_to_llsd_xml(mesh_upload_response)
}

/// Phase 85.9: FetchLibDescendents2 handler - returns OpenSim Library folder contents
pub async fn handle_fetch_lib_descendents2(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: String,
) -> Result<Response, StatusCode> {
    use crate::opensim_compatibility::library_assets::get_global_library_manager;
    use uuid::Uuid;

    info!(
        "📚 FetchLibDescendents2 request for session: {}",
        session_id
    );

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    // Parse folder requests from LLSD XML body
    let folder_requests = parse_fetch_inventory_request(&body);
    if folder_requests.is_empty() {
        info!("📚 FetchLibDescendents2: empty request");
        let empty_response = json!({ "folders": [] });
        return json_response_to_llsd_xml(empty_response);
    }

    // Get library manager
    let lib_manager = match get_global_library_manager() {
        Some(manager) => manager,
        None => {
            warn!("📚 FetchLibDescendents2: Library manager not available");
            let empty_response = json!({ "folders": [] });
            return json_response_to_llsd_xml(empty_response);
        }
    };

    let manager = lib_manager.read().await;
    let lib_owner_id = manager.get_library_owner_id().to_string();

    let mut folder_responses = Vec::new();

    for req in folder_requests {
        let folder_id = match Uuid::parse_str(&req.folder_id) {
            Ok(id) => id,
            Err(_) => {
                warn!(
                    "📚 FetchLibDescendents2: Invalid folder UUID: {}",
                    req.folder_id
                );
                continue;
            }
        };

        info!("📚 FetchLibDescendents2: Looking up folder {}", folder_id);

        // Get folder descendants from library manager
        if let Some((subfolders, items)) = manager.get_folder_descendants(&folder_id) {
            // Build categories (subfolders) array
            let categories: Vec<Value> = subfolders
                .iter()
                .map(|f| {
                    json!({
                        "category_id": f.folder_id.to_string(),
                        "parent_id": if f.parent_folder_id.is_nil() {
                            manager.get_library_root_folder_id().to_string()
                        } else {
                            f.parent_folder_id.to_string()
                        },
                        "name": f.name,
                        "type_default": f.folder_type,
                        "version": 1
                    })
                })
                .collect();

            // Build items array
            let items_list: Vec<Value> = items
                .iter()
                .map(|i| {
                    json!({
                        "parent_id": i.folder_id.to_string(),
                        "asset_id": i.asset_id.to_string(),
                        "item_id": i.inventory_id.to_string(),
                        "permissions": {
                            "creator_id": lib_owner_id.clone(),
                            "owner_id": lib_owner_id.clone(),
                            "group_id": "00000000-0000-0000-0000-000000000000",
                            "last_owner_id": lib_owner_id.clone(),
                            "base_mask": 581639,      // Full perms for library items
                            "owner_mask": 581639,
                            "group_mask": 0,
                            "everyone_mask": 581639,
                            "next_owner_mask": 581639
                        },
                        "type": i.asset_type,
                        "inv_type": i.inventory_type,
                        "flags": i.flags,
                        "name": i.name,
                        "desc": i.description,
                        "created_at": 0,
                        "sale_info": {
                            "sale_price": 0,
                            "sale_type": 0
                        }
                    })
                })
                .collect();

            let descendents = categories.len() + items_list.len();

            info!(
                "📚 FetchLibDescendents2: Found {} categories, {} items for folder {}",
                categories.len(),
                items_list.len(),
                folder_id
            );

            folder_responses.push(json!({
                "folder_id": folder_id.to_string(),
                "agent_id": lib_owner_id.clone(),
                "owner_id": lib_owner_id.clone(),
                "version": 1,
                "descendents": descendents,
                "categories": categories,
                "items": items_list
            }));
        } else {
            warn!(
                "📚 FetchLibDescendents2: Folder {} not found in library",
                folder_id
            );
        }
    }

    let response = json!({
        "folders": folder_responses
    });

    info!(
        "📚 FetchLibDescendents2: Returning {} folder responses",
        folder_responses.len()
    );
    json_response_to_llsd_xml(response)
}

/// Phase 85.9: FetchLib2 handler - returns specific library items
pub async fn handle_fetch_lib2(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: String,
) -> Result<Response, StatusCode> {
    use crate::opensim_compatibility::library_assets::get_global_library_manager;
    use uuid::Uuid;

    info!("📚 FetchLib2 request for session: {}", session_id);

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    // Parse item IDs from LLSD XML body (reuse FetchInventory2 parser)
    let item_ids = parse_fetch_inventory2_items(&body);
    if item_ids.is_empty() {
        info!("📚 FetchLib2: empty request");
        let empty_response = json!({ "items": [] });
        return json_response_to_llsd_xml(empty_response);
    }

    // Get library manager
    let lib_manager = match get_global_library_manager() {
        Some(manager) => manager,
        None => {
            warn!("📚 FetchLib2: Library manager not available");
            let empty_response = json!({ "items": [] });
            return json_response_to_llsd_xml(empty_response);
        }
    };

    let manager = lib_manager.read().await;
    let lib_owner_id = manager.get_library_owner_id().to_string();

    let mut items_response = Vec::new();

    for item_id_str in item_ids {
        let item_id = match Uuid::parse_str(&item_id_str) {
            Ok(id) => id,
            Err(_) => {
                warn!("📚 FetchLib2: Invalid item UUID: {}", item_id_str);
                continue;
            }
        };

        if let Some(item) = manager.get_item(&item_id) {
            info!(
                "📚 FetchLib2: Found library item {} ({})",
                item.name, item_id
            );

            items_response.push(json!({
                "parent_id": item.folder_id.to_string(),
                "asset_id": item.asset_id.to_string(),
                "item_id": item.inventory_id.to_string(),
                "permissions": {
                    "creator_id": lib_owner_id.clone(),
                    "owner_id": lib_owner_id.clone(),
                    "group_id": "00000000-0000-0000-0000-000000000000",
                    "last_owner_id": lib_owner_id.clone(),
                    "base_mask": 581639,
                    "owner_mask": 581639,
                    "group_mask": 0,
                    "everyone_mask": 581639,
                    "next_owner_mask": 581639
                },
                "type": item.asset_type,
                "inv_type": item.inventory_type,
                "flags": item.flags,
                "name": item.name,
                "desc": item.description,
                "created_at": 0,
                "sale_info": {
                    "sale_price": 0,
                    "sale_type": 0
                }
            }));
        } else {
            warn!("📚 FetchLib2: Item {} not found in library", item_id);
        }
    }

    let response = json!({
        "items": items_response
    });

    info!("📚 FetchLib2: Returning {} items", items_response.len());
    json_response_to_llsd_xml(response)
}

pub async fn handle_provision_voice_account(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: String,
) -> Result<Response, StatusCode> {
    info!(
        "[Voice] ProvisionVoiceAccountRequest for session: {}",
        session_id
    );
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let session = state
        .caps_manager
        .get_session(&session_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let agent_id =
        uuid::Uuid::parse_str(&session.agent_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if body.contains("voice_server_type") && !body.contains("vivox") {
        let xml = "<llsd><undef /></llsd>";
        return Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/xml")],
            xml.to_string(),
        )
            .into_response());
    }

    let voice = match &state.voice_module {
        Some(v) if v.voice_enabled() => v,
        _ => {
            let xml = "<llsd><undef /></llsd>";
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/xml")],
                xml.to_string(),
            )
                .into_response());
        }
    };

    let xml = voice.provision_voice_account(agent_id).unwrap_or_else(|e| {
        warn!("[Voice] Provision failed: {}", e);
        "<llsd><undef /></llsd>".to_string()
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        xml,
    )
        .into_response())
}

pub async fn handle_parcel_voice_info(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: String,
) -> Result<Response, StatusCode> {
    info!("[Voice] ParcelVoiceInfoRequest for session: {}", session_id);
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let session = state
        .caps_manager
        .get_session(&session_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let agent_id =
        uuid::Uuid::parse_str(&session.agent_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let voice = match &state.voice_module {
        Some(v) if v.voice_enabled() => v,
        _ => {
            let xml = "<llsd><undef /></llsd>";
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/xml")],
                xml.to_string(),
            )
                .into_response());
        }
    };

    let (
        parcel_flags,
        parcel_uuid,
        parcel_local_id,
        parcel_name,
        region_name,
        region_uuid,
        estate_allow_voice,
    ) = match get_parcel_info_for_agent(&state.db_pool, agent_id).await {
        Some(info) => info,
        None => {
            let xml = "<llsd><undef /></llsd>";
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/xml")],
                xml.to_string(),
            )
                .into_response());
        }
    };

    let xml = voice
        .parcel_voice_info(
            agent_id,
            parcel_flags,
            parcel_uuid,
            parcel_local_id,
            &parcel_name,
            &region_name,
            region_uuid,
            estate_allow_voice,
        )
        .unwrap_or_else(|e| {
            warn!("[Voice] ParcelVoiceInfo failed: {}", e);
            "<llsd><undef /></llsd>".to_string()
        });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        xml,
    )
        .into_response())
}

async fn get_parcel_info_for_agent(
    db_pool: &sqlx::PgPool,
    agent_id: uuid::Uuid,
) -> Option<(u32, uuid::Uuid, i32, String, String, uuid::Uuid, bool)> {
    let row = sqlx::query_as::<_, (String, String, i32, i32, String)>(
        "SELECT \"UUID\", \"Name\", \"LocalLandID\", \"LandFlags\", \"RegionUUID\" FROM land LIMIT 1"
    )
    .fetch_optional(db_pool)
    .await
    .ok()??;

    let parcel_uuid = uuid::Uuid::parse_str(&row.0).unwrap_or(uuid::Uuid::nil());
    let parcel_name = row.1.clone();
    let parcel_local_id = row.2;
    let parcel_flags = row.3 as u32;
    let region_uuid = uuid::Uuid::parse_str(&row.4).unwrap_or(uuid::Uuid::nil());

    let region_name =
        sqlx::query_scalar::<_, String>("SELECT \"regionName\" FROM regions WHERE uuid = $1")
            .bind(region_uuid.to_string())
            .fetch_optional(db_pool)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "Region".to_string());

    let estate_allow_voice = sqlx::query_scalar::<_, i32>(
        "SELECT \"EstateSettings\".\"AllowVoice\" FROM estate_map \
         JOIN \"EstateSettings\" ON estate_map.\"EstateID\" = \"EstateSettings\".\"EstateID\" \
         WHERE estate_map.\"RegionID\" = $1 LIMIT 1",
    )
    .bind(region_uuid.to_string())
    .fetch_optional(db_pool)
    .await
    .ok()
    .flatten()
    .unwrap_or(1)
        != 0;

    Some((
        parcel_flags,
        parcel_uuid,
        parcel_local_id,
        parcel_name,
        region_name,
        region_uuid,
        estate_allow_voice,
    ))
}

pub async fn handle_freeswitch_prelogin(
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    if let Some(ref voice) = state.voice_module {
        if let Some(xml) = voice.fs_handle_prelogin() {
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/xml")],
                xml,
            )
                .into_response());
        }
    }
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        "<VCConfiguration/>".to_string(),
    )
        .into_response())
}

pub async fn handle_freeswitch_signin(
    State(state): State<CapsHandlerState>,
    body: String,
) -> Result<Response, StatusCode> {
    let params = parse_form_params(&body);
    let userid = params.get("userid").cloned().unwrap_or_default();
    let pwd = params.get("pwd").cloned().unwrap_or_default();

    if let Some(ref voice) = state.voice_module {
        if let Some(xml) = voice.fs_handle_signin(&userid, &pwd) {
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/xml")],
                xml,
            )
                .into_response());
        }
    }
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        "<response><level0><status>FAIL</status></level0></response>".to_string(),
    )
        .into_response())
}

pub async fn handle_freeswitch_buddy(
    State(state): State<CapsHandlerState>,
    body: String,
) -> Result<Response, StatusCode> {
    let params = parse_form_params(&body);
    let auth_token = params.get("auth_token").cloned().unwrap_or_default();

    if let Some(ref voice) = state.voice_module {
        if let Some(xml) = voice.fs_handle_buddy(&auth_token) {
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/xml")],
                xml,
            )
                .into_response());
        }
    }
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        "<response><level0><status>OK</status><body/></level0></response>".to_string(),
    )
        .into_response())
}

pub async fn handle_freeswitch_watcher(
    State(state): State<CapsHandlerState>,
    body: String,
) -> Result<Response, StatusCode> {
    let params = parse_form_params(&body);
    let auth_token = params.get("auth_token").cloned().unwrap_or_default();

    if let Some(ref voice) = state.voice_module {
        if let Some(xml) = voice.fs_handle_watcher(&auth_token) {
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/xml")],
                xml,
            )
                .into_response());
        }
    }
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        "<response><level0><status>OK</status><body/></level0></response>".to_string(),
    )
        .into_response())
}

fn parse_form_params(body: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for pair in body.split('&') {
        let mut kv = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
            params.insert(
                urlencoding::decode(k).unwrap_or_default().to_string(),
                urlencoding::decode(v).unwrap_or_default().to_string(),
            );
        }
    }
    params
}

fn parse_llsd_uuid_array(body: &str) -> Vec<String> {
    let mut uuids = Vec::new();
    for cap in body.match_indices("<uuid>") {
        let start = cap.0 + 6;
        if let Some(end_pos) = body[start..].find("</uuid>") {
            let uuid_str = &body[start..start + end_pos];
            if uuid_str.len() == 36 {
                uuids.push(uuid_str.to_string());
            }
        }
    }
    uuids
}

pub async fn handle_get_object_cost(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    use sqlx::Row;

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    let object_ids = parse_llsd_uuid_array(&body_str);

    if object_ids.is_empty() {
        return json_response_to_llsd_xml(json!({}));
    }

    let mut result_map = serde_json::Map::new();

    for obj_id_str in &object_ids {
        let obj_uuid = match uuid::Uuid::parse_str(obj_id_str) {
            Ok(u) => u,
            Err(_) => continue,
        };

        let part_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prims WHERE scenegroupid = $1")
                .bind(obj_uuid)
                .fetch_one(&*state.db_pool)
                .await
                .unwrap_or(0);

        let count = if part_count > 0 {
            part_count as f64
        } else {
            1.0
        };

        result_map.insert(
            obj_id_str.clone(),
            json!({
                "linked_set_resource_cost": count,
                "resource_cost": 1.0,
                "physics_cost": 1.0,
                "linked_set_physics_cost": count,
                "resource_limiting_type": "legacy"
            }),
        );
    }

    json_response_to_llsd_xml(Value::Object(result_map))
}

pub async fn handle_get_object_physics_data(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    let object_ids = parse_llsd_uuid_array(&body_str);

    let mut result_map = serde_json::Map::new();

    for obj_id_str in &object_ids {
        let obj_uuid = match uuid::Uuid::parse_str(obj_id_str) {
            Ok(u) => u,
            Err(_) => continue,
        };

        let (shape_type, density, friction, restitution, gravity_mod) =
            if let Some(ref scene_objects) = state.scene_objects {
                let objects = scene_objects.read();
                objects
                    .values()
                    .find(|o| o.uuid == obj_uuid)
                    .map(|o| {
                        (
                            o.physics_shape_type,
                            o.density,
                            o.friction,
                            o.restitution,
                            o.gravity_modifier,
                        )
                    })
                    .unwrap_or((0, 1000.0, 0.6, 0.5, 1.0))
            } else {
                (0, 1000.0, 0.6, 0.5, 1.0)
            };

        result_map.insert(
            obj_id_str.clone(),
            json!({
                "PhysicsShapeType": shape_type,
                "Density": density,
                "Friction": friction,
                "Restitution": restitution,
                "GravityMultiplier": gravity_mod
            }),
        );
    }

    json_response_to_llsd_xml(Value::Object(result_map))
}

pub async fn handle_resource_cost_selected(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    use sqlx::Row;

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    let object_ids = parse_llsd_uuid_array(&body_str);

    let mut total_physics = 0.0_f64;
    let mut total_streaming = 0.0_f64;
    let mut total_simulation = 0.0_f64;

    for obj_id_str in &object_ids {
        let obj_uuid = match uuid::Uuid::parse_str(obj_id_str) {
            Ok(u) => u,
            Err(_) => continue,
        };

        let part_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prims WHERE scenegroupid = $1")
                .bind(obj_uuid)
                .fetch_one(&*state.db_pool)
                .await
                .unwrap_or(1);

        let count = if part_count > 0 {
            part_count as f64
        } else {
            1.0
        };
        total_physics += count;
        total_streaming += count;
        total_simulation += count * 0.5;
    }

    json_response_to_llsd_xml(json!({
        "selected": {
            "physics": total_physics,
            "streaming": total_streaming,
            "simulation": total_simulation
        }
    }))
}

pub async fn handle_agent_profile(
    Path(session_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    let agent_id_str = params.get("agent_id").map(|s| s.as_str()).unwrap_or("");
    info!(
        "[CAPS] AgentProfile GET for agent={} session={}",
        agent_id_str, session_id
    );
    let agent_id = uuid::Uuid::parse_str(agent_id_str).unwrap_or_default();
    let mut about_text = String::new();
    let mut fl_about = String::new();
    let mut image_id = uuid::Uuid::nil();
    let mut fl_image_id = uuid::Uuid::nil();
    let mut partner_id = uuid::Uuid::nil();
    let mut born_on = "Unknown".to_string();
    let mut display_name = String::new();
    {
        let pool = &*state.db_pool;
        use sqlx::Row;
        if let Ok(Some(row)) = sqlx::query("SELECT abouttext, firstlifeabouttext, profileimage, firstlifeimage, partner FROM userprofile WHERE useruuid = $1")
            .bind(agent_id.to_string()).fetch_optional(pool).await {
            about_text = row.get("abouttext");
            fl_about = row.get("firstlifeabouttext");
            let img_s: String = row.get("profileimage");
            image_id = uuid::Uuid::parse_str(&img_s).unwrap_or_default();
            let fl_s: String = row.get("firstlifeimage");
            fl_image_id = uuid::Uuid::parse_str(&fl_s).unwrap_or_default();
            let p_s: String = row.get("partner");
            partner_id = uuid::Uuid::parse_str(&p_s).unwrap_or_default();
        }
        if let Ok(Some(row)) = sqlx::query("SELECT \"FirstName\", \"LastName\", \"Created\" FROM \"UserAccounts\" WHERE \"PrincipalID\" = $1")
            .bind(agent_id.to_string()).fetch_optional(pool).await {
            let first: String = row.get("FirstName");
            let last: String = row.get("LastName");
            display_name = format!("{} {}", first, last);
            let created: i32 = row.get("Created");
            if created > 0 {
                let dt = chrono::DateTime::from_timestamp(created as i64, 0).unwrap_or_default();
                born_on = dt.format("%Y-%m-%d").to_string();
            }
        }
    }
    let body = json!({
        "agents": [{
            "agent_id": agent_id.to_string(),
            "display_name": display_name,
            "sl_about_text": about_text,
            "fl_about_text": fl_about,
            "sl_image_id": image_id.to_string(),
            "fl_image_id": fl_image_id.to_string(),
            "partner_id": partner_id.to_string(),
            "born_on": born_on,
            "member_of_group": "",
            "charter_member": false,
            "allow_publish": true,
            "online": true,
            "picks": [],
            "groups": []
        }]
    });
    json_response_to_llsd_xml(body)
}

pub async fn handle_agent_profile_update(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
    _body: Bytes,
) -> Result<Response, StatusCode> {
    info!("[CAPS] AgentProfile POST (update) session={}", session_id);
    json_response_to_llsd_xml(json!({"result": "success"}))
}

pub async fn handle_render_materials_get(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("[CAPS] RenderMaterials GET session={}", session_id);
    let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><llsd><map><key>Zipped</key><binary encoding=\"base64\">eNpjYGBgAAAABAAB</binary></map></llsd>";
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(xml.into())
        .unwrap())
}

pub async fn handle_render_materials_post(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    use base64::Engine;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    info!(
        "[CAPS] RenderMaterials POST session={} body_len={}",
        session_id,
        body_str.len()
    );

    let requested_ids = parse_llsd_uuid_array(&body_str);

    if requested_ids.is_empty() {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><llsd><map><key>Zipped</key><binary encoding=\"base64\">eNpjYGBgAAAABAAB</binary></map></llsd>";
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/llsd+xml")
            .body(xml.into())
            .unwrap());
    }

    let mut llsd_inner = String::from("<array>");
    let mut found = 0u32;
    for id_str in &requested_ids {
        let mat_uuid = match uuid::Uuid::parse_str(id_str) {
            Ok(u) => u,
            Err(_) => continue,
        };
        let mat_result = if let Some(ref fetcher) = state.asset_fetcher {
            fetcher
                .fetch_asset_data_typed_pg(&mat_uuid.to_string(), Some(57), &state.db_pool)
                .await
                .ok()
                .flatten()
        } else {
            sqlx::query_as::<_, (Vec<u8>,)>(
                "SELECT data FROM assets WHERE id = $1::uuid AND assettype = 57",
            )
            .bind(mat_uuid)
            .fetch_optional(state.db_pool.as_ref())
            .await
            .ok()
            .flatten()
            .map(|(d,)| d)
        };
        {
            if let Some(data) = mat_result {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                llsd_inner.push_str(&format!(
                    "<map><key>ID</key><binary encoding=\"base64\">{}</binary><key>Material</key><binary encoding=\"base64\">{}</binary></map>",
                    base64::engine::general_purpose::STANDARD.encode(mat_uuid.as_bytes()),
                    b64
                ));
                found += 1;
            }
        }
    }
    llsd_inner.push_str("</array>");

    info!(
        "[CAPS] RenderMaterials POST: found {}/{} requested materials",
        found,
        requested_ids.len()
    );

    let llsd_xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><llsd>{}</llsd>",
        llsd_inner
    );

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(llsd_xml.as_bytes());
    let compressed = encoder.finish().unwrap_or_default();

    let b64_compressed = base64::engine::general_purpose::STANDARD.encode(&compressed);

    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><llsd><map><key>Zipped</key><binary encoding=\"base64\">{}</binary></map></llsd>",
        b64_compressed
    );
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/llsd+xml")
        .body(xml.into())
        .unwrap())
}

pub async fn handle_object_media_get(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("[CAPS] ObjectMedia GET session={}", session_id);
    json_response_to_llsd_xml(json!({"object_media_data": []}))
}

pub async fn handle_object_media_post(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
    _body: Bytes,
) -> Result<Response, StatusCode> {
    info!("[CAPS] ObjectMedia POST session={}", session_id);
    json_response_to_llsd_xml(json!({"result": "success"}))
}

pub async fn handle_object_media_navigate(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
    _body: Bytes,
) -> Result<Response, StatusCode> {
    info!("[CAPS] ObjectMediaNavigate POST session={}", session_id);
    json_response_to_llsd_xml(json!({"result": "success"}))
}

pub async fn handle_search_stat_request(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("[CAPS] SearchStatRequest GET session={}", session_id);
    json_response_to_llsd_xml(json!({"result": []}))
}

pub async fn handle_land_resources(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("[CAPS] LandResources GET session={}", session_id);

    let pool = &*state.db_pool;
    use sqlx::Row;

    let region_id = std::env::var("OPENSIM_REGION_UUID").unwrap_or_default();
    let mut parcels = Vec::new();
    if !region_id.is_empty() {
        if let Ok(rows) = sqlx::query(
            r#"SELECT locallandid, name, area, uuid FROM land WHERE regionuuid = $1::uuid"#,
        )
        .bind(&region_id)
        .fetch_all(pool)
        .await
        {
            for row in &rows {
                let local_id: i32 = row.get("locallandid");
                let name: String = row.get("name");
                let area: i32 = row.get("area");
                let parcel_id: String = row.try_get("uuid").unwrap_or_default();
                parcels.push(json!({
                    "LocalID": local_id,
                    "Name": name,
                    "Area": area,
                    "ParcelID": parcel_id,
                    "Scripts": 0,
                    "ScriptMemory": 0,
                    "ScriptURLs": 0,
                    "Objects": 0,
                    "Prims": 0
                }));
            }
        }
    }

    let total_area: i64 = parcels
        .iter()
        .map(|p| p["Area"].as_i64().unwrap_or(0))
        .sum();

    json_response_to_llsd_xml(json!({
        "ScriptResourceSummary": {
            "available": [
                {"type": "urls", "amount": 15000},
                {"type": "memory", "amount": 65536}
            ],
            "used": [
                {"type": "urls", "amount": 0},
                {"type": "memory", "amount": 0}
            ]
        },
        "summary": {
            "available": [{"type": "land", "amount": 65536}],
            "used": [{"type": "land", "amount": total_area}]
        },
        "parcels": parcels
    }))
}

pub async fn handle_script_resource_summary(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "[CAPS] ScriptResourceSummary POST session={} ({} bytes)",
        session_id,
        body.len()
    );

    json_response_to_llsd_xml(json!({
        "summary": {
            "available": [
                {"type": "urls", "amount": 15000},
                {"type": "memory", "amount": 65536}
            ],
            "used": [
                {"type": "urls", "amount": 0},
                {"type": "memory", "amount": 0}
            ]
        }
    }))
}

pub async fn handle_script_resource_details(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "[CAPS] ScriptResourceDetails POST session={} ({} bytes)",
        session_id,
        body.len()
    );

    json_response_to_llsd_xml(json!({
        "parcels": []
    }))
}

pub async fn handle_avatar_picker_search(
    Path(session_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    let query = params.get("names").map(|s| s.as_str()).unwrap_or("");
    info!(
        "[CAPS] AvatarPickerSearch query='{}' session={}",
        query, session_id
    );
    let mut agents = Vec::new();
    if !query.is_empty() {
        let pool = &*state.db_pool;
        use sqlx::Row;
        let pattern = format!("%{}%", query);
        if let Ok(rows) = sqlx::query(
            r#"SELECT "PrincipalID", "FirstName", "LastName" FROM "UserAccounts" WHERE "FirstName" ILIKE $1 OR "LastName" ILIKE $1 LIMIT 20"#
        ).bind(&pattern).fetch_all(pool).await {
            for row in &rows {
                let id: String = row.get("PrincipalID");
                let first: String = row.get("FirstName");
                let last: String = row.get("LastName");
                agents.push(json!({
                    "agent_id": id,
                    "display_name": format!("{} {}", first, last),
                    "username": format!("{}.{}", first.to_lowercase(), last.to_lowercase())
                }));
            }
        }
    }
    json_response_to_llsd_xml(json!({"agents": agents}))
}

pub async fn handle_dispatch_region_info(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
    _body: Bytes,
) -> Result<Response, StatusCode> {
    info!("[CAPS] DispatchRegionInfo POST session={}", session_id);
    json_response_to_llsd_xml(json!({"result": "success"}))
}

pub async fn handle_product_info_request(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("[CAPS] ProductInfoRequest GET session={}", session_id);
    json_response_to_llsd_xml(json!({
        "success": true,
        "currency": "L$",
        "is_trial": false,
        "product_name": "OpenSim Next"
    }))
}

pub async fn handle_server_release_notes(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("[CAPS] ServerReleaseNotes GET session={}", session_id);
    let html = "<html><body><h1>OpenSim Next</h1><p>Powered by Rust/Zig hybrid architecture.</p></body></html>";
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(html.into())
        .unwrap())
}

pub async fn handle_copy_inventory_from_notecard(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
    _body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "[CAPS] CopyInventoryFromNotecard POST session={}",
        session_id
    );
    json_response_to_llsd_xml(json!({"result": "success"}))
}

pub async fn handle_update_gesture_agent_inventory(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
    _body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "[CAPS] UpdateGestureAgentInventory POST session={}",
        session_id
    );
    json_response_to_llsd_xml(json!({"result": "success"}))
}

pub async fn handle_update_gesture_task_inventory(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
    _body: Bytes,
) -> Result<Response, StatusCode> {
    info!(
        "[CAPS] UpdateGestureTaskInventory POST session={}",
        session_id
    );
    json_response_to_llsd_xml(json!({"result": "success"}))
}

pub async fn handle_lsl_syntax(
    Path(session_id): Path<String>,
    State(_state): State<CapsHandlerState>,
) -> Result<Response, StatusCode> {
    info!("[CAPS] LSLSyntax GET session={}", session_id);
    json_response_to_llsd_xml(json!({
        "llsd-lsl-syntax-version": 2
    }))
}

pub async fn handle_modify_material_params(
    Path(session_id): Path<String>,
    State(state): State<CapsHandlerState>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    state
        .caps_manager
        .update_session_activity(&session_id)
        .await;

    let body_str = String::from_utf8_lossy(&body);
    info!(
        "[CAPS] ModifyMaterialParams POST session={} body_len={}",
        session_id,
        body_str.len()
    );

    let scene_objects = match &state.scene_objects {
        Some(so) => so.clone(),
        None => {
            warn!("[CAPS] ModifyMaterialParams: scene_objects not available");
            return json_response_to_llsd_xml(json!({"error": "scene not available"}));
        }
    };

    let mut object_uuid_str = String::new();
    let mut side: i32 = -1;
    let mut gltf_json = String::new();

    for cap in body_str.match_indices("<key>") {
        let key_start = cap.0 + 5;
        if let Some(key_end) = body_str[key_start..].find("</key>") {
            let key = &body_str[key_start..key_start + key_end];
            let val_region = &body_str[key_start + key_end + 6..];

            match key {
                "object_id" => {
                    if let Some(s) = val_region.find("<uuid>") {
                        let start = s + 6;
                        if let Some(e) = val_region[start..].find("</uuid>") {
                            object_uuid_str = val_region[start..start + e].to_string();
                        }
                    }
                }
                "side" => {
                    if let Some(s) = val_region.find("<integer>") {
                        let start = s + 9;
                        if let Some(e) = val_region[start..].find("</integer>") {
                            side = val_region[start..start + e].parse().unwrap_or(-1);
                        }
                    }
                }
                "gltf_json" => {
                    if let Some(s) = val_region.find("<string>") {
                        let start = s + 8;
                        if let Some(e) = val_region[start..].find("</string>") {
                            gltf_json = val_region[start..start + e].to_string();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let object_uuid = match uuid::Uuid::parse_str(&object_uuid_str) {
        Ok(u) => u,
        Err(_) => {
            warn!(
                "[CAPS] ModifyMaterialParams: invalid object_id '{}'",
                object_uuid_str
            );
            return json_response_to_llsd_xml(json!({"error": "invalid object_id"}));
        }
    };

    if side < 0 {
        warn!("[CAPS] ModifyMaterialParams: missing side parameter");
        return json_response_to_llsd_xml(json!({"error": "missing side"}));
    }

    info!(
        "[CAPS] ModifyMaterialParams: object={} side={} gltf_len={}",
        object_uuid,
        side,
        gltf_json.len()
    );

    let mut found_local_id = None;
    let mut updated_overrides: Vec<(u8, String)> = Vec::new();

    {
        let mut scene = scene_objects.write();
        for obj in scene.values_mut() {
            if obj.uuid == object_uuid {
                found_local_id = Some(obj.local_id);

                if gltf_json.is_empty() {
                    obj.mat_overrides.retain(|&(idx, _)| idx != side as u8);
                } else {
                    if let Some(entry) = obj
                        .mat_overrides
                        .iter_mut()
                        .find(|(idx, _)| *idx == side as u8)
                    {
                        entry.1 = gltf_json.clone();
                    } else {
                        obj.mat_overrides.push((side as u8, gltf_json.clone()));
                    }
                }
                updated_overrides = obj.mat_overrides.clone();
                break;
            }
        }
    }

    if found_local_id.is_none() {
        warn!(
            "[CAPS] ModifyMaterialParams: object {} not found in scene",
            object_uuid
        );
        return json_response_to_llsd_xml(json!({"error": "object not found"}));
    }

    let bin = crate::materials::store::mat_ovrd_to_bin(&updated_overrides);
    let db_pool = state.db_pool.clone();
    let obj_uuid = object_uuid;
    tokio::spawn(async move {
        if let Err(e) = sqlx::query("UPDATE primshapes SET matovrd = $1 WHERE uuid = $2::uuid")
            .bind(&bin)
            .bind(obj_uuid)
            .execute(db_pool.as_ref())
            .await
        {
            warn!(
                "[CAPS] ModifyMaterialParams: DB persist failed for {}: {}",
                obj_uuid, e
            );
        } else {
            info!(
                "[CAPS] ModifyMaterialParams: persisted {} bytes for object {}",
                bin.len(),
                obj_uuid
            );
        }
    });

    json_response_to_llsd_xml(json!({"success": true}))
}
