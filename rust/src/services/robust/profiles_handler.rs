use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use serde_json::{json, Value};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::RobustState;

pub async fn handle_profiles(
    State(state): State<RobustState>,
    body: String,
) -> axum::response::Response {
    let json_body: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            warn!("[PROFILES] Invalid JSON: {}", e);
            return jsonrpc_error(-1, -32700, "Parse error", &e.to_string());
        }
    };

    let method = json_body["method"].as_str().unwrap_or("");
    let id = json_body["id"].as_i64().unwrap_or(0);
    let params = &json_body["params"];

    debug!("[PROFILES] JsonRpc method={} id={}", method, id);

    let svc = match &state.profiles_service {
        Some(svc) => svc,
        None => {
            warn!("[PROFILES] No Profiles service configured");
            return jsonrpc_error(id, -32601, "Service unavailable", "No profiles service");
        }
    };

    match method {
        "avatar_classifieds_request" | "AvatarClassifiedsRequest" => {
            let creator_id = parse_uuid(params, "creatorId");
            match svc.get_classifieds(creator_id).await {
                Ok(list) => {
                    let items: Vec<Value> = list
                        .iter()
                        .map(|c| {
                            json!({
                                "ClassifiedId": c.classified_id.to_string(),
                                "Name": c.name,
                            })
                        })
                        .collect();
                    jsonrpc_result(id, json!(items))
                }
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "classified_update" | "ClassifiedUpdate" => {
            let c = crate::services::traits::UserClassifiedAdd {
                classified_id: parse_uuid(params, "ClassifiedId"),
                creator_id: parse_uuid(params, "CreatorId"),
                creation_date: params["CreationDate"].as_i64().unwrap_or(0) as i32,
                expiration_date: params["ExpirationDate"].as_i64().unwrap_or(0) as i32,
                category: params["Category"].as_i64().unwrap_or(0) as i32,
                name: params["Name"].as_str().unwrap_or("").to_string(),
                description: params["Description"].as_str().unwrap_or("").to_string(),
                parcel_id: parse_uuid(params, "ParcelId"),
                parent_estate: params["ParentEstate"].as_i64().unwrap_or(0) as i32,
                snapshot_id: parse_uuid(params, "SnapshotId"),
                sim_name: params["SimName"].as_str().unwrap_or("").to_string(),
                global_pos: params["GlobalPos"].as_str().unwrap_or("").to_string(),
                parcel_name: params["ParcelName"].as_str().unwrap_or("").to_string(),
                flags: params["Flags"].as_i64().unwrap_or(0) as i32,
                listing_price: params["ListingPrice"].as_i64().unwrap_or(0) as i32,
            };
            match svc.update_classified(&c).await {
                Ok(true) => jsonrpc_result(id, json!(true)),
                Ok(false) => jsonrpc_error(id, -32603, "Update failed", ""),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "classified_delete" | "ClassifiedDelete" => {
            let cid = parse_uuid(params, "classifiedId");
            match svc.delete_classified(cid).await {
                Ok(_) => jsonrpc_result(id, json!(true)),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "classified_info_request" | "ClassifiedInfoRequest" => {
            let cid = parse_uuid(params, "ClassifiedId");
            match svc.get_classified(cid).await {
                Ok(Some(c)) => jsonrpc_result(id, classified_to_json(&c)),
                Ok(None) => jsonrpc_error(id, -32602, "Not found", "Classified not found"),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "avatar_picks_request" | "AvatarPicksRequest" => {
            let creator_id = parse_uuid(params, "creatorId");
            match svc.get_picks(creator_id).await {
                Ok(list) => {
                    let items: Vec<Value> = list
                        .iter()
                        .map(|p| {
                            json!({
                                "PickId": p.pick_id.to_string(),
                                "Name": p.name,
                            })
                        })
                        .collect();
                    jsonrpc_result(id, json!(items))
                }
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "pick_info_request" | "PickInfoRequest" => {
            let pid = parse_uuid(params, "PickId");
            match svc.get_pick(pid).await {
                Ok(Some(p)) => jsonrpc_result(id, pick_to_json(&p)),
                Ok(None) => jsonrpc_error(id, -32602, "Not found", "Pick not found"),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "picks_update" | "PicksUpdate" => {
            let p = crate::services::traits::UserProfilePick {
                pick_id: parse_uuid(params, "PickId"),
                creator_id: parse_uuid(params, "CreatorId"),
                top_pick: params["TopPick"].as_bool().unwrap_or(false),
                parcel_id: parse_uuid(params, "ParcelId"),
                name: params["Name"].as_str().unwrap_or("").to_string(),
                description: params["Description"].as_str().unwrap_or("").to_string(),
                snapshot_id: parse_uuid(params, "SnapshotId"),
                user: params["User"].as_str().unwrap_or("").to_string(),
                original_name: params["OriginalName"].as_str().unwrap_or("").to_string(),
                sim_name: params["SimName"].as_str().unwrap_or("").to_string(),
                global_pos: params["GlobalPos"].as_str().unwrap_or("").to_string(),
                sort_order: params["SortOrder"].as_i64().unwrap_or(0) as i32,
                enabled: params["Enabled"].as_bool().unwrap_or(false),
            };
            match svc.update_pick(&p).await {
                Ok(true) => jsonrpc_result(id, json!(true)),
                Ok(false) => jsonrpc_error(id, -32603, "Update failed", ""),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "picks_delete" | "PicksDelete" => {
            let pid = parse_uuid(params, "pickId");
            match svc.delete_pick(pid).await {
                Ok(_) => jsonrpc_result(id, json!(true)),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "avatar_notes_request" | "AvatarNotesRequest" => {
            let user_id = parse_uuid(params, "UserId");
            let target_id = parse_uuid(params, "TargetId");
            match svc.get_notes(user_id, target_id).await {
                Ok(Some(n)) => jsonrpc_result(
                    id,
                    json!({
                        "UserId": n.user_id.to_string(),
                        "TargetId": n.target_id.to_string(),
                        "Notes": n.notes,
                    }),
                ),
                Ok(None) => jsonrpc_result(
                    id,
                    json!({
                        "UserId": user_id.to_string(),
                        "TargetId": target_id.to_string(),
                        "Notes": "",
                    }),
                ),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "notes_update" | "NotesUpdate" => {
            let notes = crate::services::traits::UserProfileNotes {
                user_id: parse_uuid(params, "UserId"),
                target_id: parse_uuid(params, "TargetId"),
                notes: params["Notes"].as_str().unwrap_or("").to_string(),
            };
            match svc.update_notes(&notes).await {
                Ok(_) => jsonrpc_result(id, json!(true)),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "avatar_properties_request" | "AvatarPropertiesRequest" => {
            let user_id = parse_uuid(params, "UserId");
            match svc.get_properties(user_id).await {
                Ok(Some(p)) => jsonrpc_result(id, properties_to_json(&p)),
                Ok(None) => jsonrpc_result(
                    id,
                    json!({
                        "UserId": user_id.to_string(),
                        "PartnerId": Uuid::nil().to_string(),
                        "ProfileUrl": "",
                        "ImageId": Uuid::nil().to_string(),
                        "AboutText": "",
                        "FirstLifeImageId": Uuid::nil().to_string(),
                        "FirstLifeText": "",
                    }),
                ),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "avatar_properties_update" | "AvatarPropertiesUpdate" => {
            let props = crate::services::traits::UserProfileProperties {
                user_id: parse_uuid(params, "UserId"),
                partner_id: parse_uuid(params, "PartnerId"),
                profile_url: params["ProfileUrl"].as_str().unwrap_or("").to_string(),
                image_id: parse_uuid(params, "ImageId"),
                about_text: params["AboutText"].as_str().unwrap_or("").to_string(),
                first_life_image_id: parse_uuid(params, "FirstLifeImageId"),
                first_life_text: params["FirstLifeText"].as_str().unwrap_or("").to_string(),
                want_to_text: params["WantToText"].as_str().unwrap_or("").to_string(),
                want_to_mask: params["WantToMask"].as_i64().unwrap_or(0) as i32,
                skills_text: params["SkillsText"].as_str().unwrap_or("").to_string(),
                skills_mask: params["SkillsMask"].as_i64().unwrap_or(0) as i32,
                languages: params["Languages"].as_str().unwrap_or("").to_string(),
            };
            match svc.update_properties(&props).await {
                Ok(_) => jsonrpc_result(id, json!(true)),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "avatar_interests_update" | "AvatarInterestsUpdate" => {
            let user_id = parse_uuid(params, "UserId");
            let existing = svc.get_properties(user_id).await.ok().flatten();
            let mut props =
                existing.unwrap_or_else(|| crate::services::traits::UserProfileProperties {
                    user_id,
                    ..Default::default()
                });
            props.want_to_text = params["WantToText"]
                .as_str()
                .unwrap_or(&props.want_to_text)
                .to_string();
            props.want_to_mask = params["WantToMask"]
                .as_i64()
                .unwrap_or(props.want_to_mask as i64) as i32;
            props.skills_text = params["SkillsText"]
                .as_str()
                .unwrap_or(&props.skills_text)
                .to_string();
            props.skills_mask = params["SkillsMask"]
                .as_i64()
                .unwrap_or(props.skills_mask as i64) as i32;
            props.languages = params["Languages"]
                .as_str()
                .unwrap_or(&props.languages)
                .to_string();
            match svc.update_properties(&props).await {
                Ok(_) => jsonrpc_result(id, json!(true)),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "user_preferences_request" | "UserPreferencesRequest" => {
            let user_id = parse_uuid(params, "UserId");
            match svc.get_preferences(user_id).await {
                Ok(Some(p)) => jsonrpc_result(
                    id,
                    json!({
                        "UserId": p.user_id.to_string(),
                        "IMViaEmail": p.im_via_email,
                        "Visible": p.visible,
                        "EMail": p.email,
                    }),
                ),
                Ok(None) => jsonrpc_result(
                    id,
                    json!({
                        "UserId": user_id.to_string(),
                        "IMViaEmail": false,
                        "Visible": true,
                        "EMail": "",
                    }),
                ),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        "user_preferences_update" | "UserPreferencesUpdate" => {
            let prefs = crate::services::traits::UserPreferences {
                user_id: parse_uuid(params, "UserId"),
                im_via_email: params["IMViaEmail"].as_bool().unwrap_or(false),
                visible: params["Visible"].as_bool().unwrap_or(true),
                email: params["EMail"].as_str().unwrap_or("").to_string(),
            };
            match svc.update_preferences(&prefs).await {
                Ok(_) => jsonrpc_result(id, json!(true)),
                Err(e) => jsonrpc_error(id, -32603, "Internal error", &e.to_string()),
            }
        }
        _ => {
            warn!("[PROFILES] Unknown method: {}", method);
            jsonrpc_error(id, -32601, "Method not found", method)
        }
    }
}

fn parse_uuid(params: &Value, key: &str) -> Uuid {
    params[key]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_default()
}

fn jsonrpc_result(id: i64, result: Value) -> axum::response::Response {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    });
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        resp.to_string(),
    )
        .into_response()
}

fn jsonrpc_error(id: i64, code: i32, message: &str, data: &str) -> axum::response::Response {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message,
            "data": data,
        }
    });
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        resp.to_string(),
    )
        .into_response()
}

fn classified_to_json(c: &crate::services::traits::UserClassifiedAdd) -> Value {
    json!({
        "ClassifiedId": c.classified_id.to_string(),
        "CreatorId": c.creator_id.to_string(),
        "CreationDate": c.creation_date,
        "ExpirationDate": c.expiration_date,
        "Category": c.category,
        "Name": c.name,
        "Description": c.description,
        "ParcelId": c.parcel_id.to_string(),
        "ParentEstate": c.parent_estate,
        "SnapshotId": c.snapshot_id.to_string(),
        "SimName": c.sim_name,
        "GlobalPos": c.global_pos,
        "ParcelName": c.parcel_name,
        "Flags": c.flags,
        "ListingPrice": c.listing_price,
    })
}

fn pick_to_json(p: &crate::services::traits::UserProfilePick) -> Value {
    json!({
        "PickId": p.pick_id.to_string(),
        "CreatorId": p.creator_id.to_string(),
        "TopPick": p.top_pick,
        "ParcelId": p.parcel_id.to_string(),
        "Name": p.name,
        "Description": p.description,
        "SnapshotId": p.snapshot_id.to_string(),
        "User": p.user,
        "OriginalName": p.original_name,
        "SimName": p.sim_name,
        "GlobalPos": p.global_pos,
        "SortOrder": p.sort_order,
        "Enabled": p.enabled,
    })
}

fn properties_to_json(p: &crate::services::traits::UserProfileProperties) -> Value {
    json!({
        "UserId": p.user_id.to_string(),
        "PartnerId": p.partner_id.to_string(),
        "ProfileUrl": p.profile_url,
        "ImageId": p.image_id.to_string(),
        "AboutText": p.about_text,
        "FirstLifeImageId": p.first_life_image_id.to_string(),
        "FirstLifeText": p.first_life_text,
        "WantToText": p.want_to_text,
        "WantToMask": p.want_to_mask,
        "SkillsText": p.skills_text,
        "SkillsMask": p.skills_mask,
        "Languages": p.languages,
    })
}
