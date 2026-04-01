use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{StatusCode, header};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::RobustState;
use super::xml_response::*;
use crate::services::traits::{InventoryFolder, InventoryItem};

pub async fn handle_inventory(
    State(state): State<RobustState>,
    body: String,
) -> impl IntoResponse {
    let params = parse_form_body(&body);
    let method = params.get("METHOD").or_else(|| params.get("method")).cloned().unwrap_or_default();

    info!("[ROBUST-INVENTORY] Received request: METHOD={}, body_len={}", method, body.len());
    debug!("[ROBUST-INVENTORY] Body: {}", body);

    let xml = match method.to_uppercase().as_str() {
        "GETFOLDER" => handle_get_folder(&state, &params).await,
        "GETROOTFOLDER" => handle_get_root_folder(&state, &params).await,
        "GETFOLDERCONTENT" => handle_get_folder_content(&state, &params).await,
        "GETFOLDERITEMS" => handle_get_folder_items(&state, &params).await,
        "GETINVENTORYSKELETON" => handle_get_skeleton(&state, &params).await,
        "GETFOLDERFORTYPE" => handle_get_folder_for_type(&state, &params).await,
        "GETITEM" => handle_get_item(&state, &params).await,
        "GETMULTIPLEITEMS" => handle_get_multiple_items(&state, &params).await,
        "CREATEFOLDER" | "ADDFOLDER" => handle_create_folder(&state, &params).await,
        "UPDATEFOLDER" => handle_update_folder(&state, &params).await,
        "DELETEFOLDERS" => handle_delete_folders(&state, &params).await,
        "ADDITEM" => handle_add_item(&state, &params).await,
        "UPDATEITEM" => handle_update_item(&state, &params).await,
        "DELETEITEMS" => handle_delete_items(&state, &params).await,
        "MOVEITEMS" => handle_move_items(&state, &params).await,
        "MOVEFOLDER" => handle_move_folder(&state, &params).await,
        "PURGEFOLDER" => handle_purge_folder(&state, &params).await,
        "GETACTIVEGESTURES" => handle_get_active_gestures(&state, &params).await,
        "GETMULTIPLEFOLDERSCONTENT" => handle_get_multiple_folders_content(&state, &params).await,
        "GETASSETPERMISSIONS" => handle_get_asset_permissions(&state, &params).await,
        _ => {
            warn!("[ROBUST-INVENTORY] Unknown method '{}'", method);
            failure_result(&format!("Unknown method: {}", method))
        }
    };

    debug!("[ROBUST-INVENTORY] Response for {}: {}", method, &xml[..xml.len().min(500)]);
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/xml; charset=utf-8")], xml)
}

async fn handle_get_folder(state: &RobustState, params: &HashMap<String, String>) -> String {
    let folder_id = parse_uuid(params, "ID");
    info!("[ROBUST-INVENTORY] GETFOLDER id={}", folder_id);

    match state.inventory_service.get_folder(folder_id).await {
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

async fn handle_get_root_folder(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    info!("[ROBUST-INVENTORY] GETROOTFOLDER principal={}", principal_id);

    match state.inventory_service.get_root_folder(principal_id).await {
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

async fn handle_get_folder_content(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let folder_id = parse_uuid(params, "FOLDER");
    info!("[ROBUST-INVENTORY] GETFOLDERCONTENT principal={}, folder={}", principal_id, folder_id);

    match state.inventory_service.get_folder_content(principal_id, folder_id).await {
        Ok(collection) => {
            info!("[ROBUST-INVENTORY] GETFOLDERCONTENT result: {} folders, {} items",
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

async fn handle_get_folder_items(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let folder_id = parse_uuid(params, "FOLDER");
    info!("[ROBUST-INVENTORY] GETFOLDERITEMS principal={}, folder={}", principal_id, folder_id);

    match state.inventory_service.get_folder_content(principal_id, folder_id).await {
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

async fn handle_get_skeleton(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    info!("[ROBUST-INVENTORY] GETINVENTORYSKELETON principal={}", principal_id);

    match state.inventory_service.get_inventory_skeleton(principal_id).await {
        Ok(folders) => {
            info!("[ROBUST-INVENTORY] GETINVENTORYSKELETON result: {} folders", folders.len());
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

async fn handle_get_folder_for_type(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let folder_type: i32 = params.get("TYPE").and_then(|s| s.parse().ok()).unwrap_or(-1);
    info!("[ROBUST-INVENTORY] GETFOLDERFORTYPE principal={}, type={}", principal_id, folder_type);

    match state.inventory_service.get_inventory_skeleton(principal_id).await {
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

async fn handle_get_item(state: &RobustState, params: &HashMap<String, String>) -> String {
    let item_id = parse_uuid(params, "ID");
    info!("[ROBUST-INVENTORY] GETITEM id={}", item_id);

    match state.inventory_service.get_item(item_id).await {
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

async fn handle_get_multiple_items(state: &RobustState, params: &HashMap<String, String>) -> String {
    let _principal_id = parse_uuid(params, "PRINCIPAL");
    let count: usize = params.get("COUNT").and_then(|s| s.parse().ok()).unwrap_or(0);
    let item_ids_str = params.get("ITEMS").cloned().unwrap_or_default();
    let item_ids: Vec<Uuid> = item_ids_str.split(',')
        .filter_map(|id| Uuid::parse_str(id.trim()).ok())
        .collect();
    info!("[ROBUST-INVENTORY] GETMULTIPLEITEMS count={}, ids={}", count, item_ids.len());

    let mut data = HashMap::new();
    let mut items_dict = HashMap::new();
    for (i, item_id) in item_ids.iter().enumerate() {
        match state.inventory_service.get_item(*item_id).await {
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

async fn handle_create_folder(state: &RobustState, params: &HashMap<String, String>) -> String {
    let folder = params_to_folder(params);
    match state.inventory_service.create_folder(&folder).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Create folder returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_update_folder(state: &RobustState, params: &HashMap<String, String>) -> String {
    let folder = params_to_folder(params);
    match state.inventory_service.update_folder(&folder).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Update folder returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_delete_folders(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let folder_ids: Vec<Uuid> = params.get("FOLDERS")
        .map(|s| s.split(',')
            .filter_map(|id| Uuid::parse_str(id.trim()).ok())
            .collect())
        .unwrap_or_default();

    match state.inventory_service.delete_folders(principal_id, &folder_ids).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Delete folders returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_add_item(state: &RobustState, params: &HashMap<String, String>) -> String {
    let item = params_to_item(params);
    match state.inventory_service.add_item(&item).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Add item returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_update_item(state: &RobustState, params: &HashMap<String, String>) -> String {
    let item = params_to_item(params);
    match state.inventory_service.update_item(&item).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Update item returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_delete_items(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let item_ids: Vec<Uuid> = params.get("ITEMS")
        .map(|s| s.split(',')
            .filter_map(|id| Uuid::parse_str(id.trim()).ok())
            .collect())
        .unwrap_or_default();

    match state.inventory_service.delete_items(principal_id, &item_ids).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Delete items returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_move_items(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let id_list_str = params.get("IDLIST").cloned().unwrap_or_default();
    let dest_list_str = params.get("DESTLIST").cloned().unwrap_or_default();

    let ids: Vec<Uuid> = id_list_str.split(',')
        .filter_map(|s| Uuid::parse_str(s.trim()).ok())
        .collect();
    let dests: Vec<Uuid> = dest_list_str.split(',')
        .filter_map(|s| Uuid::parse_str(s.trim()).ok())
        .collect();

    info!("[ROBUST-INVENTORY] MOVEITEMS principal={}, count={}", principal_id, ids.len());

    if ids.len() != dests.len() {
        return failure_result("IDLIST and DESTLIST count mismatch");
    }

    let items: Vec<(Uuid, Uuid)> = ids.into_iter().zip(dests.into_iter()).collect();
    match state.inventory_service.move_items(principal_id, &items).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Move items returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_move_folder(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let folder_id = parse_uuid(params, "ID");
    let new_parent_id = parse_uuid(params, "ParentID");
    info!("[ROBUST-INVENTORY] MOVEFOLDER principal={}, folder={}, new_parent={}", principal_id, folder_id, new_parent_id);

    match state.inventory_service.move_folder(principal_id, folder_id, new_parent_id).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Move folder returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_purge_folder(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let folder_id = parse_uuid(params, "ID");
    info!("[ROBUST-INVENTORY] PURGEFOLDER principal={}, folder={}", principal_id, folder_id);

    match state.inventory_service.purge_folder(principal_id, folder_id).await {
        Ok(true) => success_result(),
        Ok(false) => failure_result("Purge folder returned false"),
        Err(e) => failure_result(&format!("{}", e)),
    }
}

async fn handle_get_active_gestures(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    info!("[ROBUST-INVENTORY] GETACTIVEGESTURES principal={}", principal_id);

    match state.inventory_service.get_active_gestures(principal_id).await {
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

async fn handle_get_multiple_folders_content(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let folders_str = params.get("FOLDERS").cloned().unwrap_or_default();
    let folder_ids: Vec<Uuid> = folders_str.split(',')
        .filter_map(|s| Uuid::parse_str(s.trim()).ok())
        .collect();
    info!("[ROBUST-INVENTORY] GETMULTIPLEFOLDERSCONTENT principal={}, count={}", principal_id, folder_ids.len());

    match state.inventory_service.get_multiple_folders_content(principal_id, &folder_ids).await {
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

async fn handle_get_asset_permissions(state: &RobustState, params: &HashMap<String, String>) -> String {
    let principal_id = parse_uuid(params, "PRINCIPAL");
    let asset_id = parse_uuid(params, "ASSET");
    info!("[ROBUST-INVENTORY] GETASSETPERMISSIONS principal={}, asset={}", principal_id, asset_id);

    match state.inventory_service.get_asset_permissions(principal_id, asset_id).await {
        Ok(perms) => {
            let mut data = HashMap::new();
            data.insert("RESULT".to_string(), XmlValue::Str(perms.to_string()));
            build_xml_response(&data)
        }
        Err(e) => failure_result(&format!("{}", e)),
    }
}

fn parse_uuid(params: &HashMap<String, String>, key: &str) -> Uuid {
    params.get(key)
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil())
}

pub fn folder_to_fields(folder: &InventoryFolder) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("ParentID".to_string(), folder.parent_id.to_string());
    m.insert("Type".to_string(), folder.folder_type.to_string());
    m.insert("Version".to_string(), folder.version.to_string());
    m.insert("Name".to_string(), folder.name.clone());
    m.insert("Owner".to_string(), folder.owner_id.to_string());
    m.insert("ID".to_string(), folder.folder_id.to_string());
    m
}

pub fn item_to_fields(item: &InventoryItem) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("AssetID".to_string(), item.asset_id.to_string());
    m.insert("AssetType".to_string(), item.asset_type.to_string());
    m.insert("BasePermissions".to_string(), item.base_permissions.to_string());
    m.insert("CreationDate".to_string(), item.creation_date.to_string());
    m.insert("CreatorId".to_string(), item.creator_id.to_string());
    m.insert("CreatorData".to_string(), item.creator_data.clone());
    m.insert("CurrentPermissions".to_string(), item.current_permissions.to_string());
    m.insert("Description".to_string(), item.description.clone());
    m.insert("EveryOnePermissions".to_string(), item.everyone_permissions.to_string());
    m.insert("Flags".to_string(), item.flags.to_string());
    m.insert("Folder".to_string(), item.folder_id.to_string());
    m.insert("GroupID".to_string(), item.group_id.to_string());
    m.insert("GroupOwned".to_string(), if item.group_owned { "True" } else { "False" }.to_string());
    m.insert("GroupPermissions".to_string(), item.group_permissions.to_string());
    m.insert("ID".to_string(), item.item_id.to_string());
    m.insert("InvType".to_string(), item.inv_type.to_string());
    m.insert("Name".to_string(), item.name.clone());
    m.insert("NextPermissions".to_string(), item.next_permissions.to_string());
    m.insert("Owner".to_string(), item.owner_id.to_string());
    m.insert("SalePrice".to_string(), item.sale_price.to_string());
    m.insert("SaleType".to_string(), item.sale_type.to_string());
    m
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
