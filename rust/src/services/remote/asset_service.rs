//! Remote Asset Service Implementation
//!
//! Connects to ROBUST AssetService via HTTP.
//! Implements OpenSim's AssetServicesConnector protocol.
//!
//! Reference: OpenSim/Services/Connectors/Asset/AssetServicesConnector.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::services::traits::{AssetServiceTrait, AssetBase, AssetMetadata};

pub struct RemoteAssetService {
    client: Client,
    server_uri: String,
}

impl RemoteAssetService {
    pub fn new(server_uri: &str) -> Self {
        info!("Initializing remote asset service: {}", server_uri);

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            server_uri: server_uri.trim_end_matches('/').to_string(),
        }
    }

    async fn send_request(&self, method: &str, params: &HashMap<String, String>) -> Result<HashMap<String, String>> {
        let url = format!("{}/assets", self.server_uri);

        let mut form_data = params.clone();
        form_data.insert("METHOD".to_string(), method.to_string());

        debug!("Asset service request: {} to {}", method, url);

        let response = self.client
            .post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| anyhow!("Asset service request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Asset service returned status: {}", response.status()));
        }

        let body = response.text().await
            .map_err(|e| anyhow!("Failed to read asset service response: {}", e))?;

        self.parse_response(&body)
    }

    async fn get_asset_raw(&self, id: &str) -> Result<Option<Vec<u8>>> {
        let url = format!("{}/assets/{}", self.server_uri, id);

        debug!("Asset service GET: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Asset GET request failed: {}", e))?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(anyhow!("Asset GET returned status: {}", response.status()));
        }

        let bytes = response.bytes().await
            .map_err(|e| anyhow!("Failed to read asset data: {}", e))?;

        Ok(Some(bytes.to_vec()))
    }

    fn parse_response(&self, body: &str) -> Result<HashMap<String, String>> {
        if let Some(xml_result) = crate::services::robust::xml_response::try_parse_xml_to_flat(body) {
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

    fn params_to_asset_base(&self, params: &HashMap<String, String>, data: Vec<u8>) -> Result<AssetBase> {
        Ok(AssetBase {
            id: params.get("ID")
                .or_else(|| params.get("id"))
                .cloned()
                .unwrap_or_default(),
            name: params.get("Name")
                .or_else(|| params.get("name"))
                .cloned()
                .unwrap_or_default(),
            description: params.get("Description")
                .or_else(|| params.get("description"))
                .cloned()
                .unwrap_or_default(),
            asset_type: params.get("Type")
                .or_else(|| params.get("type"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            local: params.get("Local")
                .or_else(|| params.get("local"))
                .map(|s| s == "True" || s == "true" || s == "1")
                .unwrap_or(false),
            temporary: params.get("Temporary")
                .or_else(|| params.get("temporary"))
                .map(|s| s == "True" || s == "true" || s == "1")
                .unwrap_or(false),
            data,
            creator_id: params.get("CreatorID")
                .or_else(|| params.get("creator_id"))
                .cloned()
                .unwrap_or_default(),
            flags: params.get("Flags")
                .or_else(|| params.get("flags"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        })
    }

    fn params_to_asset_metadata(&self, params: &HashMap<String, String>) -> Result<AssetMetadata> {
        Ok(AssetMetadata {
            id: params.get("ID")
                .or_else(|| params.get("id"))
                .cloned()
                .unwrap_or_default(),
            name: params.get("Name")
                .or_else(|| params.get("name"))
                .cloned()
                .unwrap_or_default(),
            description: params.get("Description")
                .or_else(|| params.get("description"))
                .cloned()
                .unwrap_or_default(),
            asset_type: params.get("Type")
                .or_else(|| params.get("type"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            local: params.get("Local")
                .or_else(|| params.get("local"))
                .map(|s| s == "True" || s == "true" || s == "1")
                .unwrap_or(false),
            temporary: params.get("Temporary")
                .or_else(|| params.get("temporary"))
                .map(|s| s == "True" || s == "true" || s == "1")
                .unwrap_or(false),
            creator_id: params.get("CreatorID")
                .or_else(|| params.get("creator_id"))
                .cloned()
                .unwrap_or_default(),
            flags: params.get("Flags")
                .or_else(|| params.get("flags"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            created_date: params.get("CreatedDate")
                .or_else(|| params.get("create_time"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        })
    }

    fn asset_to_params(&self, asset: &AssetBase) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("ID".to_string(), asset.id.clone());
        params.insert("Name".to_string(), asset.name.clone());
        params.insert("Description".to_string(), asset.description.clone());
        params.insert("Type".to_string(), asset.asset_type.to_string());
        params.insert("Local".to_string(), asset.local.to_string());
        params.insert("Temporary".to_string(), asset.temporary.to_string());
        params.insert("Data".to_string(), general_purpose::STANDARD.encode(&asset.data));
        params.insert("CreatorID".to_string(), asset.creator_id.clone());
        params.insert("Flags".to_string(), asset.flags.to_string());
        params
    }
}

#[async_trait]
impl AssetServiceTrait for RemoteAssetService {
    async fn get(&self, id: &str) -> Result<Option<AssetBase>> {
        debug!("Remote: Getting asset: {}", id);

        let mut params = HashMap::new();
        params.insert("ID".to_string(), id.to_string());

        let metadata_response = self.send_request("get", &params).await?;

        let result = metadata_response.get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if !result || !metadata_response.contains_key("ID") {
            return Ok(None);
        }

        let data = if let Some(data_str) = metadata_response.get("Data") {
            general_purpose::STANDARD.decode(data_str).unwrap_or_default()
        } else {
            self.get_asset_raw(id).await?.unwrap_or_default()
        };

        Ok(Some(self.params_to_asset_base(&metadata_response, data)?))
    }

    async fn get_metadata(&self, id: &str) -> Result<Option<AssetMetadata>> {
        debug!("Remote: Getting asset metadata: {}", id);

        let mut params = HashMap::new();
        params.insert("ID".to_string(), id.to_string());

        let response = self.send_request("get_metadata", &params).await?;

        let result = response.get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if result && response.contains_key("ID") {
            Ok(Some(self.params_to_asset_metadata(&response)?))
        } else {
            Ok(None)
        }
    }

    async fn get_data(&self, id: &str) -> Result<Option<Vec<u8>>> {
        debug!("Remote: Getting asset data: {}", id);
        self.get_asset_raw(id).await
    }

    async fn store(&self, asset: &AssetBase) -> Result<String> {
        info!("Remote: Storing asset: {} ({})", asset.name, asset.id);

        let params = self.asset_to_params(asset);
        let response = self.send_request("store", &params).await?;

        let id = response.get("ID")
            .or_else(|| response.get("id"))
            .cloned()
            .unwrap_or_else(|| asset.id.clone());

        let success = response.get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE" || s == &id)
            .unwrap_or(true);

        if success {
            info!("Remote: Stored asset: {}", id);
            Ok(id)
        } else {
            let message = response.get("message").cloned().unwrap_or_else(|| "Unknown error".to_string());
            warn!("Remote: Failed to store asset: {}", message);
            Err(anyhow!("Failed to store asset: {}", message))
        }
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        warn!("Remote: Deleting asset: {}", id);

        let mut params = HashMap::new();
        params.insert("ID".to_string(), id.to_string());

        let response = self.send_request("delete", &params).await?;

        Ok(response.get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE")
            .unwrap_or(false))
    }

    async fn asset_exists(&self, id: &str) -> Result<bool> {
        debug!("Remote: Checking if asset exists: {}", id);

        let mut params = HashMap::new();
        params.insert("ID".to_string(), id.to_string());

        let response = self.send_request("get_metadata", &params).await?;

        Ok(response.get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(false) && response.contains_key("ID"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_asset_service_creation() {
        let service = RemoteAssetService::new("http://localhost:8003");
        assert_eq!(service.server_uri, "http://localhost:8003");
    }

    #[test]
    fn test_asset_to_params() {
        let service = RemoteAssetService::new("http://localhost:8003");

        let asset = AssetBase {
            id: "test-asset-id".to_string(),
            name: "Test Asset".to_string(),
            description: "Test Description".to_string(),
            asset_type: 0,
            local: false,
            temporary: false,
            data: vec![1, 2, 3, 4],
            creator_id: "creator-id".to_string(),
            flags: 0,
        };

        let params = service.asset_to_params(&asset);
        assert_eq!(params.get("ID"), Some(&"test-asset-id".to_string()));
        assert_eq!(params.get("Name"), Some(&"Test Asset".to_string()));
        assert!(params.contains_key("Data"));
    }

    #[test]
    fn test_base64_encoding() {
        let data = vec![1, 2, 3, 4, 5];
        let encoded = general_purpose::STANDARD.encode(&data);
        let decoded = general_purpose::STANDARD.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }
}
