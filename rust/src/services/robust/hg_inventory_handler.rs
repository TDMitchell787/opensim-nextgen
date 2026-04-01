use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::RobustState;
use super::xml_response::*;
use super::inventory_handler::{folder_to_fields, item_to_fields};
use crate::services::traits::{InventoryFolder, InventoryItem};

pub async fn handle_hg_inventory(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let svc = match &state.hg_inventory_service {
        Some(svc) => svc.clone(),
        None => {
            warn!("[HG-INVENTORY] HG inventory not enabled, rejecting request");
            return (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")], failure_result("HG inventory not enabled"));
        }
    };

    let params = parse_form_body(&body);
    let method = params.get("METHOD").or_else(|| params.get("method")).cloned().unwrap_or_default();

    info!("[HG-INVENTORY] Received request: METHOD={}, body_len={}", method, body.len());
    debug!("[HG-INVENTORY] Body: {}", body);

    let xml = match method.to_uppercase().as_str() {
        "GETFOLDER" => {
            let folder_id = parse_uuid(&params, "ID");
            info!("[HG-INVENTORY] GETFOLDER id={}", folder_id);
            match svc.get_folder(folder_id).await {
                Ok(Some(folder)) => {
                    let mut data = HashMap::new();
                    let mut inner = HashMap::new();
                    for (k, v) in folder_to_fields(&folder) {
                        inner.insert(k, XmlValue::Str(v));
                    }
                    data.insert("folder".to_string(), XmlValue::Dict(inner));
                    build_xml_response(&data)
                }
                Ok(None) => null_result(),
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "GETROOTFOLDER" => {
            let principal_id = parse_uuid(&params, "PRINCIPAL");
            info!("[HG-INVENTORY] GETROOTFOLDER principal={}", principal_id);
            match svc.get_root_folder(principal_id).await {
                Ok(Some(folder)) => {
                    let mut data = HashMap::new();
                    let mut inner = HashMap::new();
                    for (k, v) in folder_to_fields(&folder) {
                        inner.insert(k, XmlValue::Str(v));
                    }
                    data.insert("folder".to_string(), XmlValue::Dict(inner));
                    build_xml_response(&data)
                }
                Ok(None) => null_result(),
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "GETFOLDERCONTENT" => {
            let principal_id = parse_uuid(&params, "PRINCIPAL");
            let folder_id = parse_uuid(&params, "FOLDER");
            info!("[HG-INVENTORY] GETFOLDERCONTENT principal={}, folder={}", principal_id, folder_id);
            match svc.get_folder_content(principal_id, folder_id).await {
                Ok(collection) => {
                    info!("[HG-INVENTORY] GETFOLDERCONTENT result: {} folders, {} items",
                        collection.folders.len(), collection.items.len());
                    let mut data = HashMap::new();
                    data.insert("FID".to_string(), XmlValue::Str(folder_id.to_string()));
                    data.insert("VERSION".to_string(), XmlValue::Str("1".to_string()));
                    let mut folders_dict = HashMap::new();
                    for (i, f) in collection.folders.iter().enumerate() {
                        let mut inner = HashMap::new();
                        for (k, v) in folder_to_fields(f) {
                            inner.insert(k, XmlValue::Str(v));
                        }
                        folders_dict.insert(format!("folder_{}", i), XmlValue::Dict(inner));
                    }
                    data.insert("FOLDERS".to_string(), XmlValue::Dict(folders_dict));
                    let mut items_dict = HashMap::new();
                    for (i, item) in collection.items.iter().enumerate() {
                        let mut inner = HashMap::new();
                        for (k, v) in item_to_fields(item) {
                            inner.insert(k, XmlValue::Str(v));
                        }
                        items_dict.insert(format!("item_{}", i), XmlValue::Dict(inner));
                    }
                    data.insert("ITEMS".to_string(), XmlValue::Dict(items_dict));
                    build_xml_response(&data)
                }
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "GETFOLDERITEMS" => {
            let principal_id = parse_uuid(&params, "PRINCIPAL");
            let folder_id = parse_uuid(&params, "FOLDER");
            info!("[HG-INVENTORY] GETFOLDERITEMS principal={}, folder={}", principal_id, folder_id);
            match svc.get_folder_content(principal_id, folder_id).await {
                Ok(collection) => {
                    let mut data = HashMap::new();
                    let mut items_dict = HashMap::new();
                    for (i, item) in collection.items.iter().enumerate() {
                        let mut inner = HashMap::new();
                        for (k, v) in item_to_fields(item) {
                            inner.insert(k, XmlValue::Str(v));
                        }
                        items_dict.insert(format!("item_{}", i), XmlValue::Dict(inner));
                    }
                    data.insert("ITEMS".to_string(), XmlValue::Dict(items_dict));
                    build_xml_response(&data)
                }
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "GETINVENTORYSKELETON" => {
            let principal_id = parse_uuid(&params, "PRINCIPAL");
            info!("[HG-INVENTORY] GETINVENTORYSKELETON principal={}", principal_id);
            match svc.get_inventory_skeleton(principal_id).await {
                Ok(folders) => {
                    info!("[HG-INVENTORY] GETINVENTORYSKELETON result: {} folders (suitcase-filtered)", folders.len());
                    let mut data = HashMap::new();
                    let mut folders_dict = HashMap::new();
                    for (i, f) in folders.iter().enumerate() {
                        let mut inner = HashMap::new();
                        for (k, v) in folder_to_fields(f) {
                            inner.insert(k, XmlValue::Str(v));
                        }
                        folders_dict.insert(format!("folder_{}", i), XmlValue::Dict(inner));
                    }
                    data.insert("FOLDERS".to_string(), XmlValue::Dict(folders_dict));
                    build_xml_response(&data)
                }
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "GETFOLDERFORTYPE" => {
            let principal_id = parse_uuid(&params, "PRINCIPAL");
            let folder_type: i32 = params.get("TYPE").and_then(|s| s.parse().ok()).unwrap_or(-1);
            info!("[HG-INVENTORY] GETFOLDERFORTYPE principal={}, type={}", principal_id, folder_type);
            match svc.get_inventory_skeleton(principal_id).await {
                Ok(folders) => {
                    if let Some(folder) = folders.iter().find(|f| f.folder_type == folder_type) {
                        let mut data = HashMap::new();
                        let mut inner = HashMap::new();
                        for (k, v) in folder_to_fields(folder) {
                            inner.insert(k, XmlValue::Str(v));
                        }
                        data.insert("folder".to_string(), XmlValue::Dict(inner));
                        build_xml_response(&data)
                    } else {
                        null_result()
                    }
                }
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "GETITEM" => {
            let item_id = parse_uuid(&params, "ID");
            info!("[HG-INVENTORY] GETITEM id={}", item_id);
            match svc.get_item(item_id).await {
                Ok(Some(item)) => {
                    let mut data = HashMap::new();
                    let mut inner = HashMap::new();
                    for (k, v) in item_to_fields(&item) {
                        inner.insert(k, XmlValue::Str(v));
                    }
                    data.insert("item".to_string(), XmlValue::Dict(inner));
                    build_xml_response(&data)
                }
                Ok(None) => null_result(),
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "GETMULTIPLEITEMS" => {
            let _principal_id = parse_uuid(&params, "PRINCIPAL");
            let count: usize = params.get("COUNT").and_then(|s| s.parse().ok()).unwrap_or(0);
            let item_ids_str = params.get("ITEMS").cloned().unwrap_or_default();
            let item_ids: Vec<Uuid> = item_ids_str.split(',')
                .filter_map(|id| Uuid::parse_str(id.trim()).ok())
                .collect();
            info!("[HG-INVENTORY] GETMULTIPLEITEMS count={}, ids={}", count, item_ids.len());
            let mut data = HashMap::new();
            let mut items_dict = HashMap::new();
            for (i, item_id) in item_ids.iter().enumerate() {
                match svc.get_item(*item_id).await {
                    Ok(Some(item)) => {
                        let mut inner = HashMap::new();
                        for (k, v) in item_to_fields(&item) {
                            inner.insert(k, XmlValue::Str(v));
                        }
                        items_dict.insert(format!("item_{}", i), XmlValue::Dict(inner));
                    }
                    _ => {}
                }
            }
            data.insert("ITEMS".to_string(), XmlValue::Dict(items_dict));
            build_xml_response(&data)
        }
        "CREATEFOLDER" | "ADDFOLDER" => {
            let folder = params_to_folder(&params);
            info!("[HG-INVENTORY] CREATEFOLDER owner={}, parent={}, name={}", folder.owner_id, folder.parent_id, folder.name);
            match svc.create_folder(&folder).await {
                Ok(true) => success_result(),
                Ok(false) => failure_result("Create folder blocked by suitcase policy"),
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "UPDATEFOLDER" => {
            let folder = params_to_folder(&params);
            info!("[HG-INVENTORY] UPDATEFOLDER owner={}, id={}", folder.owner_id, folder.folder_id);
            match svc.update_folder(&folder).await {
                Ok(true) => success_result(),
                Ok(false) => failure_result("Update folder blocked by suitcase policy"),
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "DELETEFOLDERS" => {
            warn!("[HG-INVENTORY] DELETEFOLDERS blocked for foreign user");
            failure_result("Delete folders blocked for foreign users")
        }
        "ADDITEM" => {
            let raw_asset_id = params.get("AssetID").cloned().unwrap_or_default();
            let item = params_to_item(&params);
            info!("[HG-INVENTORY] ADDITEM owner={}, folder={}, name='{}', asset_id={}, raw_asset_id='{}', asset_type={}, inv_type={}",
                  item.owner_id, item.folder_id, item.name, item.asset_id, raw_asset_id, item.asset_type, item.inv_type);
            if item.asset_id.is_nil() && !raw_asset_id.is_empty() {
                warn!("[HG-INVENTORY] ADDITEM asset_id parsed as NIL from raw='{}' — foreign format not recognized", raw_asset_id);
            }
            let foreign_grid_uri = if raw_asset_id.contains('|') || (raw_asset_id.starts_with("http://") || raw_asset_id.starts_with("https://")) {
                if let Some(pipe_pos) = raw_asset_id.find('|') {
                    Some(raw_asset_id[..pipe_pos].to_string())
                } else if let Some(slash_pos) = raw_asset_id.rfind('/') {
                    Some(raw_asset_id[..slash_pos].to_string())
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(ref uri) = foreign_grid_uri {
                info!("[HG-INVENTORY] Foreign asset detected from grid '{}', will attempt fetch for asset_id={}", uri, item.asset_id);
                if !item.asset_id.is_nil() {
                    match crate::services::hypergrid::hg_asset_service::fetch_foreign_asset(uri, item.asset_id).await {
                        Ok(mut asset) => {
                            asset.asset_type = item.asset_type as i8;
                            asset.id = item.asset_id.to_string();
                            match state.asset_service.store(&asset).await {
                                Ok(id) => info!("[HG-INVENTORY] Fetched and cached foreign asset {} ({} bytes) from {}", id, asset.data.len(), uri),
                                Err(e) => warn!("[HG-INVENTORY] Failed to cache foreign asset {}: {}", item.asset_id, e),
                            }
                        }
                        Err(e) => warn!("[HG-INVENTORY] Failed to fetch foreign asset {} from {}: {}", item.asset_id, uri, e),
                    }
                }
            }
            match svc.add_item(&item).await {
                Ok(true) => {
                    info!("[HG-INVENTORY] ADDITEM success: item={} asset={} name='{}'", item.item_id, item.asset_id, item.name);
                    success_result()
                }
                Ok(false) => {
                    warn!("[HG-INVENTORY] ADDITEM blocked by suitcase policy: item={} name='{}'", item.item_id, item.name);
                    failure_result("Add item blocked by suitcase policy")
                }
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "UPDATEITEM" => {
            let item = params_to_item(&params);
            info!("[HG-INVENTORY] UPDATEITEM owner={}, id={}", item.owner_id, item.item_id);
            match svc.update_item(&item).await {
                Ok(true) => success_result(),
                Ok(false) => failure_result("Update item blocked by suitcase policy"),
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "DELETEITEMS" => {
            warn!("[HG-INVENTORY] DELETEITEMS blocked for foreign user");
            failure_result("Delete items blocked for foreign users")
        }
        "MOVEITEMS" => {
            let principal_id = parse_uuid(&params, "PRINCIPAL");
            let id_list_str = params.get("IDLIST").cloned().unwrap_or_default();
            let dest_list_str = params.get("DESTLIST").cloned().unwrap_or_default();
            let ids: Vec<Uuid> = id_list_str.split(',').filter_map(|s| Uuid::parse_str(s.trim()).ok()).collect();
            let dests: Vec<Uuid> = dest_list_str.split(',').filter_map(|s| Uuid::parse_str(s.trim()).ok()).collect();
            info!("[HG-INVENTORY] MOVEITEMS principal={}, count={}", principal_id, ids.len());
            if ids.len() != dests.len() {
                failure_result("IDLIST and DESTLIST count mismatch")
            } else {
                let items: Vec<(Uuid, Uuid)> = ids.into_iter().zip(dests.into_iter()).collect();
                match svc.move_items(principal_id, &items).await {
                    Ok(true) => success_result(),
                    Ok(false) => failure_result("Move items blocked by suitcase policy"),
                    Err(e) => failure_result(&format!("{}", e)),
                }
            }
        }
        "MOVEFOLDER" => {
            warn!("[HG-INVENTORY] MOVEFOLDER blocked for foreign user");
            failure_result("Move folder blocked for foreign users")
        }
        "PURGEFOLDER" => {
            warn!("[HG-INVENTORY] PURGEFOLDER blocked for foreign user");
            failure_result("Purge folder blocked for foreign users")
        }
        "GETACTIVEGESTURES" => {
            let principal_id = parse_uuid(&params, "PRINCIPAL");
            info!("[HG-INVENTORY] GETACTIVEGESTURES principal={}", principal_id);
            match svc.get_active_gestures(principal_id).await {
                Ok(items) => {
                    let mut data = HashMap::new();
                    let mut items_dict = HashMap::new();
                    for (i, item) in items.iter().enumerate() {
                        let mut inner = HashMap::new();
                        for (k, v) in item_to_fields(item) {
                            inner.insert(k, XmlValue::Str(v));
                        }
                        items_dict.insert(format!("item_{}", i), XmlValue::Dict(inner));
                    }
                    data.insert("ITEMS".to_string(), XmlValue::Dict(items_dict));
                    build_xml_response(&data)
                }
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        "GETMULTIPLEFOLDERSCONTENT" => {
            let principal_id = parse_uuid(&params, "PRINCIPAL");
            let folders_str = params.get("FOLDERS").cloned().unwrap_or_default();
            let folder_ids: Vec<Uuid> = folders_str.split(',').filter_map(|s| Uuid::parse_str(s.trim()).ok()).collect();
            info!("[HG-INVENTORY] GETMULTIPLEFOLDERSCONTENT principal={}, count={}", principal_id, folder_ids.len());
            match svc.get_multiple_folders_content(principal_id, &folder_ids).await {
                Ok(collections) => {
                    let mut data = HashMap::new();
                    for (ci, collection) in collections.iter().enumerate() {
                        let fid = folder_ids.get(ci).map(|id| id.to_string()).unwrap_or_default();
                        let mut coll_dict = HashMap::new();
                        coll_dict.insert("FID".to_string(), XmlValue::Str(fid));
                        coll_dict.insert("VERSION".to_string(), XmlValue::Str("1".to_string()));
                        let mut folders_dict = HashMap::new();
                        for (i, f) in collection.folders.iter().enumerate() {
                            let mut inner = HashMap::new();
                            for (k, v) in folder_to_fields(f) {
                                inner.insert(k, XmlValue::Str(v));
                            }
                            folders_dict.insert(format!("folder_{}", i), XmlValue::Dict(inner));
                        }
                        coll_dict.insert("FOLDERS".to_string(), XmlValue::Dict(folders_dict));
                        let mut items_dict = HashMap::new();
                        for (i, item) in collection.items.iter().enumerate() {
                            let mut inner = HashMap::new();
                            for (k, v) in item_to_fields(item) {
                                inner.insert(k, XmlValue::Str(v));
                            }
                            items_dict.insert(format!("item_{}", i), XmlValue::Dict(inner));
                        }
                        coll_dict.insert("ITEMS".to_string(), XmlValue::Dict(items_dict));
                        data.insert(format!("C_{}", ci), XmlValue::Dict(coll_dict));
                    }
                    build_xml_response(&data)
                }
                Err(e) => failure_result(&format!("{}", e)),
            }
        }
        _ => {
            warn!("[HG-INVENTORY] Unknown method '{}'", method);
            failure_result(&format!("Unknown method: {}", method))
        }
    };

    debug!("[HG-INVENTORY] Response for {}: {}", method, &xml[..xml.len().min(500)]);
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")], xml)
}

fn parse_uuid(params: &HashMap<String, String>, key: &str) -> Uuid {
    params.get(key)
        .and_then(|s| parse_uuid_or_foreign(s))
        .unwrap_or(Uuid::nil())
}

fn parse_uuid_or_foreign(s: &str) -> Option<Uuid> {
    if let Ok(uuid) = Uuid::parse_str(s) {
        return Some(uuid);
    }
    if let Some(pipe_pos) = s.find('|') {
        let after_pipe = &s[pipe_pos + 1..];
        if let Ok(uuid) = Uuid::parse_str(after_pipe) {
            return Some(uuid);
        }
    }
    if s.starts_with("http://") || s.starts_with("https://") {
        let last_slash = s.rfind('/');
        if let Some(pos) = last_slash {
            let after_slash = &s[pos + 1..];
            if let Ok(uuid) = Uuid::parse_str(after_slash) {
                return Some(uuid);
            }
        }
    }
    None
}

fn params_to_folder(params: &HashMap<String, String>) -> InventoryFolder {
    InventoryFolder {
        folder_id: parse_uuid(params, "ID"),
        parent_id: parse_uuid(params, "ParentID"),
        owner_id: parse_uuid(params, "Owner"),
        name: params.get("Name").cloned().unwrap_or_default(),
        folder_type: params.get("Type").and_then(|s| s.parse().ok()).unwrap_or(-1),
        version: params.get("Version").and_then(|s| s.parse().ok()).unwrap_or(1),
    }
}

fn params_to_item(params: &HashMap<String, String>) -> InventoryItem {
    InventoryItem {
        item_id: parse_uuid(params, "ID"),
        asset_id: parse_uuid(params, "AssetID"),
        folder_id: parse_uuid(params, "Folder"),
        owner_id: parse_uuid(params, "Owner"),
        creator_id: parse_uuid(params, "CreatorId"),
        creator_data: params.get("CreatorData").cloned().unwrap_or_default(),
        name: params.get("Name").cloned().unwrap_or_default(),
        description: params.get("Description").cloned().unwrap_or_default(),
        asset_type: params.get("AssetType").and_then(|s| s.parse().ok()).unwrap_or(0),
        inv_type: params.get("InvType").and_then(|s| s.parse().ok()).unwrap_or(0),
        flags: params.get("Flags").and_then(|s| s.parse().ok()).unwrap_or(0),
        creation_date: params.get("CreationDate").and_then(|s| s.parse().ok()).unwrap_or(0),
        base_permissions: params.get("BasePermissions").and_then(|s| s.parse().ok()).unwrap_or(0x7FFFFFFF),
        current_permissions: params.get("CurrentPermissions").and_then(|s| s.parse().ok()).unwrap_or(0x7FFFFFFF),
        everyone_permissions: params.get("EveryOnePermissions").and_then(|s| s.parse().ok()).unwrap_or(0),
        next_permissions: params.get("NextPermissions").and_then(|s| s.parse().ok()).unwrap_or(0x7FFFFFFF),
        group_permissions: params.get("GroupPermissions").and_then(|s| s.parse().ok()).unwrap_or(0),
        group_id: parse_uuid(params, "GroupID"),
        group_owned: params.get("GroupOwned").map(|s| s == "True" || s == "true").unwrap_or(false),
        sale_price: params.get("SalePrice").and_then(|s| s.parse().ok()).unwrap_or(0),
        sale_type: params.get("SaleType").and_then(|s| s.parse().ok()).unwrap_or(0),
    }
}
