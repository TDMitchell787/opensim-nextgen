//! Remote Inventory Service Implementation
//!
//! Connects to ROBUST InventoryService via HTTP.
//! Implements OpenSim's XInventoryServicesConnector protocol.
//!
//! Reference: OpenSim/Services/Connectors/Inventory/XInventoryServicesConnector.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{
    InventoryCollection, InventoryFolder, InventoryItem, InventoryServiceTrait,
};

pub struct RemoteInventoryService {
    client: Client,
    server_uri: String,
}

impl RemoteInventoryService {
    pub fn new(server_uri: &str) -> Self {
        info!("Initializing remote inventory service: {}", server_uri);

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            server_uri: server_uri.trim_end_matches('/').to_string(),
        }
    }

    async fn send_request(
        &self,
        method: &str,
        params: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>> {
        let url = format!("{}/xinventory", self.server_uri);

        let mut form_data = params.clone();
        form_data.insert("METHOD".to_string(), method.to_string());

        debug!("Inventory service request: {} to {}", method, url);

        let response = self
            .client
            .post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| anyhow!("Inventory service request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Inventory service returned status: {}",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read inventory service response: {}", e))?;

        self.parse_response(&body)
    }

    fn parse_response(&self, body: &str) -> Result<HashMap<String, String>> {
        if let Some(xml_result) = crate::services::robust::xml_response::try_parse_xml_to_flat(body)
        {
            return Ok(xml_result);
        }

        let mut result = HashMap::new();
        for line in body.lines() {
            if let Some((key, value)) = line.split_once('=') {
                result.insert(key.to_string(), value.to_string());
            }
        }

        Ok(result)
    }

    fn params_to_folder(&self, params: &HashMap<String, String>) -> InventoryFolder {
        InventoryFolder {
            folder_id: params
                .get("ID")
                .or_else(|| params.get("folderID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            parent_id: params
                .get("ParentID")
                .or_else(|| params.get("parentFolderID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            owner_id: params
                .get("Owner")
                .or_else(|| params.get("agentID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            name: params
                .get("Name")
                .or_else(|| params.get("name"))
                .cloned()
                .unwrap_or_default(),
            folder_type: params
                .get("Type")
                .or_else(|| params.get("type"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            version: params
                .get("Version")
                .or_else(|| params.get("version"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
        }
    }

    fn params_to_item(&self, params: &HashMap<String, String>) -> InventoryItem {
        InventoryItem {
            item_id: params
                .get("ID")
                .or_else(|| params.get("inventoryID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            asset_id: params
                .get("AssetID")
                .or_else(|| params.get("assetID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            folder_id: params
                .get("Folder")
                .or_else(|| params.get("parentFolderID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            owner_id: params
                .get("Owner")
                .or_else(|| params.get("avatarID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            creator_id: params
                .get("CreatorId")
                .or_else(|| params.get("creatorID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            name: params
                .get("Name")
                .or_else(|| params.get("inventoryName"))
                .cloned()
                .unwrap_or_default(),
            description: params
                .get("Description")
                .or_else(|| params.get("inventoryDescription"))
                .cloned()
                .unwrap_or_default(),
            asset_type: params
                .get("AssetType")
                .or_else(|| params.get("assetType"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            inv_type: params
                .get("InvType")
                .or_else(|| params.get("invType"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            flags: params
                .get("Flags")
                .or_else(|| params.get("flags"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            creation_date: params
                .get("CreationDate")
                .or_else(|| params.get("creationDate"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            creator_data: params.get("CreatorData").cloned().unwrap_or_default(),
            base_permissions: params
                .get("BasePermissions")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0x7FFFFFFF),
            current_permissions: params
                .get("CurrentPermissions")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0x7FFFFFFF),
            everyone_permissions: params
                .get("EveryOnePermissions")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            next_permissions: params
                .get("NextPermissions")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0x7FFFFFFF),
            group_permissions: params
                .get("GroupPermissions")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            group_id: params
                .get("GroupID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            group_owned: params
                .get("GroupOwned")
                .map(|s| s == "True" || s == "true")
                .unwrap_or(false),
            sale_price: params
                .get("SalePrice")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            sale_type: params
                .get("SaleType")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        }
    }

    fn folder_to_params(&self, folder: &InventoryFolder) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("ID".to_string(), folder.folder_id.to_string());
        params.insert("ParentID".to_string(), folder.parent_id.to_string());
        params.insert("Owner".to_string(), folder.owner_id.to_string());
        params.insert("Name".to_string(), folder.name.clone());
        params.insert("Type".to_string(), folder.folder_type.to_string());
        params.insert("Version".to_string(), folder.version.to_string());
        params
    }

    fn item_to_params(&self, item: &InventoryItem) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("ID".to_string(), item.item_id.to_string());
        params.insert("AssetID".to_string(), item.asset_id.to_string());
        params.insert("Folder".to_string(), item.folder_id.to_string());
        params.insert("Owner".to_string(), item.owner_id.to_string());
        params.insert("CreatorId".to_string(), item.creator_id.to_string());
        params.insert("Name".to_string(), item.name.clone());
        params.insert("Description".to_string(), item.description.clone());
        params.insert("AssetType".to_string(), item.asset_type.to_string());
        params.insert("InvType".to_string(), item.inv_type.to_string());
        params.insert("Flags".to_string(), item.flags.to_string());
        params.insert("CreationDate".to_string(), item.creation_date.to_string());
        params
    }

    fn parse_folder_list(&self, response: &HashMap<String, String>) -> Vec<InventoryFolder> {
        let mut folders = Vec::new();
        let mut i = 0;

        while response.contains_key(&format!("ID_{}", i))
            || response.contains_key(&format!("folderID_{}", i))
        {
            let suffix = format!("_{}", i);
            let mut folder_params = HashMap::new();

            for (key, value) in response {
                if key.ends_with(&suffix) {
                    let base_key = key.strip_suffix(&suffix).unwrap_or(key);
                    folder_params.insert(base_key.to_string(), value.clone());
                }
            }

            if !folder_params.is_empty() {
                folders.push(self.params_to_folder(&folder_params));
            }

            i += 1;
        }

        folders
    }

    fn parse_item_list(&self, response: &HashMap<String, String>) -> Vec<InventoryItem> {
        let mut items = Vec::new();
        let mut i = 0;

        while response.contains_key(&format!("ID_{}", i))
            || response.contains_key(&format!("inventoryID_{}", i))
        {
            let suffix = format!("_{}", i);
            let mut item_params = HashMap::new();

            for (key, value) in response {
                if key.ends_with(&suffix) {
                    let base_key = key.strip_suffix(&suffix).unwrap_or(key);
                    item_params.insert(base_key.to_string(), value.clone());
                }
            }

            if !item_params.is_empty() {
                items.push(self.params_to_item(&item_params));
            }

            i += 1;
        }

        items
    }
}

#[async_trait]
impl InventoryServiceTrait for RemoteInventoryService {
    async fn get_folder(&self, folder_id: Uuid) -> Result<Option<InventoryFolder>> {
        debug!("Remote: Getting folder: {}", folder_id);

        let mut params = HashMap::new();
        params.insert("ID".to_string(), folder_id.to_string());

        let response = self.send_request("GETFOLDER", &params).await?;

        let result = response
            .get("RESULT")
            .map(|s| s != "False" && s != "null")
            .unwrap_or(true);

        if result && response.contains_key("ID") {
            Ok(Some(self.params_to_folder(&response)))
        } else {
            Ok(None)
        }
    }

    async fn get_root_folder(&self, principal_id: Uuid) -> Result<Option<InventoryFolder>> {
        debug!("Remote: Getting root folder for user: {}", principal_id);

        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());

        let response = self.send_request("GETROOTFOLDER", &params).await?;

        let result = response
            .get("RESULT")
            .map(|s| s != "False" && s != "null")
            .unwrap_or(true);

        if result && response.contains_key("ID") {
            Ok(Some(self.params_to_folder(&response)))
        } else {
            Ok(None)
        }
    }

    async fn get_folder_content(
        &self,
        principal_id: Uuid,
        folder_id: Uuid,
    ) -> Result<InventoryCollection> {
        debug!(
            "Remote: Getting folder content: {} for user: {}",
            folder_id, principal_id
        );

        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        params.insert("FOLDER".to_string(), folder_id.to_string());

        let response = self.send_request("GETFOLDERCONTENT", &params).await?;

        let folders = self.parse_folder_list(&response);
        let items = self.parse_item_list(&response);

        Ok(InventoryCollection { folders, items })
    }

    async fn create_folder(&self, folder: &InventoryFolder) -> Result<bool> {
        info!(
            "Remote: Creating folder: {} ({})",
            folder.name, folder.folder_id
        );

        let params = self.folder_to_params(folder);
        let response = self.send_request("CREATEFOLDER", &params).await?;

        Ok(response
            .get("RESULT")
            .map(|s| s == "True" || s == "true")
            .unwrap_or(false))
    }

    async fn update_folder(&self, folder: &InventoryFolder) -> Result<bool> {
        info!(
            "Remote: Updating folder: {} ({})",
            folder.name, folder.folder_id
        );

        let params = self.folder_to_params(folder);
        let response = self.send_request("UPDATEFOLDER", &params).await?;

        Ok(response
            .get("RESULT")
            .map(|s| s == "True" || s == "true")
            .unwrap_or(false))
    }

    async fn delete_folders(&self, principal_id: Uuid, folder_ids: &[Uuid]) -> Result<bool> {
        warn!(
            "Remote: Deleting {} folders for user: {}",
            folder_ids.len(),
            principal_id
        );

        for folder_id in folder_ids {
            let mut params = HashMap::new();
            params.insert("ID".to_string(), folder_id.to_string());
            self.send_request("DELETEFOLDER", &params).await?;
        }

        Ok(true)
    }

    async fn get_item(&self, item_id: Uuid) -> Result<Option<InventoryItem>> {
        debug!("Remote: Getting item: {}", item_id);

        let mut params = HashMap::new();
        params.insert("ID".to_string(), item_id.to_string());

        let response = self.send_request("GETITEM", &params).await?;

        let result = response
            .get("RESULT")
            .map(|s| s != "False" && s != "null")
            .unwrap_or(true);

        if result && response.contains_key("ID") {
            Ok(Some(self.params_to_item(&response)))
        } else {
            Ok(None)
        }
    }

    async fn add_item(&self, item: &InventoryItem) -> Result<bool> {
        info!("Remote: Adding item: {} ({})", item.name, item.item_id);

        let params = self.item_to_params(item);
        let response = self.send_request("ADDITEM", &params).await?;

        Ok(response
            .get("RESULT")
            .map(|s| s == "True" || s == "true")
            .unwrap_or(false))
    }

    async fn update_item(&self, item: &InventoryItem) -> Result<bool> {
        info!("Remote: Updating item: {} ({})", item.name, item.item_id);

        let params = self.item_to_params(item);
        let response = self.send_request("UPDATEITEM", &params).await?;

        Ok(response
            .get("RESULT")
            .map(|s| s == "True" || s == "true")
            .unwrap_or(false))
    }

    async fn delete_items(&self, principal_id: Uuid, item_ids: &[Uuid]) -> Result<bool> {
        warn!(
            "Remote: Deleting {} items for user: {}",
            item_ids.len(),
            principal_id
        );

        for item_id in item_ids {
            let mut params = HashMap::new();
            params.insert("ID".to_string(), item_id.to_string());
            self.send_request("DELETEITEM", &params).await?;
        }

        Ok(true)
    }

    async fn get_inventory_skeleton(&self, principal_id: Uuid) -> Result<Vec<InventoryFolder>> {
        debug!(
            "Remote: Getting inventory skeleton for user: {}",
            principal_id
        );

        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());

        let response = self.send_request("GETINVENTORYSKELETON", &params).await?;
        Ok(self.parse_folder_list(&response))
    }

    async fn move_items(&self, principal_id: Uuid, items: &[(Uuid, Uuid)]) -> Result<bool> {
        info!(
            "Remote: Moving {} items for user: {}",
            items.len(),
            principal_id
        );
        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        let id_list: Vec<String> = items.iter().map(|(id, _)| id.to_string()).collect();
        let dest_list: Vec<String> = items.iter().map(|(_, dest)| dest.to_string()).collect();
        params.insert("IDLIST".to_string(), id_list.join(","));
        params.insert("DESTLIST".to_string(), dest_list.join(","));
        let response = self.send_request("MOVEITEMS", &params).await?;
        Ok(response
            .get("RESULT")
            .map(|s| s == "True" || s == "true")
            .unwrap_or(false))
    }

    async fn move_folder(
        &self,
        principal_id: Uuid,
        folder_id: Uuid,
        new_parent_id: Uuid,
    ) -> Result<bool> {
        info!(
            "Remote: Moving folder {} to {} for user {}",
            folder_id, new_parent_id, principal_id
        );
        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        params.insert("ID".to_string(), folder_id.to_string());
        params.insert("ParentID".to_string(), new_parent_id.to_string());
        let response = self.send_request("MOVEFOLDER", &params).await?;
        Ok(response
            .get("RESULT")
            .map(|s| s == "True" || s == "true")
            .unwrap_or(false))
    }

    async fn purge_folder(&self, principal_id: Uuid, folder_id: Uuid) -> Result<bool> {
        warn!(
            "Remote: Purging folder {} for user {}",
            folder_id, principal_id
        );
        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        params.insert("ID".to_string(), folder_id.to_string());
        let response = self.send_request("PURGEFOLDER", &params).await?;
        Ok(response
            .get("RESULT")
            .map(|s| s == "True" || s == "true")
            .unwrap_or(false))
    }

    async fn get_active_gestures(&self, principal_id: Uuid) -> Result<Vec<InventoryItem>> {
        debug!("Remote: Getting active gestures for user: {}", principal_id);
        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        let response = self.send_request("GETACTIVEGESTURES", &params).await?;
        Ok(self.parse_item_list(&response))
    }

    async fn get_multiple_folders_content(
        &self,
        principal_id: Uuid,
        folder_ids: &[Uuid],
    ) -> Result<Vec<InventoryCollection>> {
        debug!(
            "Remote: Getting content for {} folders for user: {}",
            folder_ids.len(),
            principal_id
        );
        let mut results = Vec::new();
        for &folder_id in folder_ids {
            let collection = self.get_folder_content(principal_id, folder_id).await?;
            results.push(collection);
        }
        Ok(results)
    }

    async fn get_asset_permissions(&self, principal_id: Uuid, asset_id: Uuid) -> Result<i32> {
        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        params.insert("ASSET".to_string(), asset_id.to_string());
        let response = self.send_request("GETASSETPERMISSIONS", &params).await?;
        Ok(response
            .get("RESULT")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_inventory_service_creation() {
        let service = RemoteInventoryService::new("http://localhost:8003");
        assert_eq!(service.server_uri, "http://localhost:8003");
    }
}
