//! Remote Grid Service Implementation
//!
//! Connects to ROBUST GridService via HTTP.
//! Implements OpenSim's GridServiceConnector protocol.
//!
//! Reference: OpenSim/Services/Connectors/Grid/GridServicesConnector.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{GridServiceTrait, RegionInfo};

pub struct RemoteGridService {
    client: Client,
    server_uri: String,
}

impl RemoteGridService {
    pub fn new(server_uri: &str) -> Self {
        info!("Initializing remote grid service: {}", server_uri);

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            server_uri: server_uri.trim_end_matches('/').to_string(),
        }
    }

    async fn send_request(&self, method: &str, params: &HashMap<String, String>) -> Result<HashMap<String, String>> {
        let url = format!("{}/grid", self.server_uri);

        let mut form_data = params.clone();
        form_data.insert("METHOD".to_string(), method.to_string());

        debug!("Grid service request: {} to {}", method, url);

        let response = self.client
            .post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| anyhow!("Grid service request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Grid service returned status: {}", response.status()));
        }

        let body = response.text().await
            .map_err(|e| anyhow!("Failed to read grid service response: {}", e))?;

        self.parse_response(&body)
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

    fn region_to_params(&self, region: &RegionInfo) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("REGIONID".to_string(), region.region_id.to_string());
        params.insert("REGIONNAME".to_string(), region.region_name.clone());
        params.insert("LOCX".to_string(), region.region_loc_x.to_string());
        params.insert("LOCY".to_string(), region.region_loc_y.to_string());
        params.insert("SIZEX".to_string(), region.region_size_x.to_string());
        params.insert("SIZEY".to_string(), region.region_size_y.to_string());
        params.insert("SERVERIP".to_string(), region.server_ip.clone());
        params.insert("SERVERPORT".to_string(), region.server_port.to_string());
        params.insert("SERVERURI".to_string(), region.server_uri.clone());
        params.insert("FLAGS".to_string(), region.region_flags.to_string());
        params.insert("SCOPEID".to_string(), region.scope_id.to_string());
        params.insert("OWNERID".to_string(), region.owner_id.to_string());
        params.insert("ESTATEID".to_string(), region.estate_id.to_string());
        params
    }

    fn params_to_region(&self, params: &HashMap<String, String>) -> Result<RegionInfo> {
        Ok(RegionInfo {
            region_id: params.get("uuid")
                .or_else(|| params.get("regionID"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            region_name: params.get("regionName")
                .cloned()
                .unwrap_or_default(),
            region_loc_x: params.get("locX")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            region_loc_y: params.get("locY")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            region_size_x: params.get("sizeX")
                .and_then(|s| s.parse().ok())
                .unwrap_or(256),
            region_size_y: params.get("sizeY")
                .and_then(|s| s.parse().ok())
                .unwrap_or(256),
            server_ip: params.get("serverIP")
                .cloned()
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            server_port: params.get("serverPort")
                .and_then(|s| s.parse().ok())
                .unwrap_or(9000),
            server_uri: params.get("serverURI")
                .cloned()
                .unwrap_or_default(),
            region_flags: params.get("flags")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            scope_id: params.get("scopeID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            owner_id: params.get("owner_uuid")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            estate_id: params.get("regionEstateID")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
        })
    }

    fn parse_region_list(&self, response: &HashMap<String, String>) -> Result<Vec<RegionInfo>> {
        let mut regions = Vec::new();
        let mut i = 0;

        while response.contains_key(&format!("uuid{}", i)) {
            let prefix = i.to_string();
            let mut region_params = HashMap::new();

            for (key, value) in response {
                if key.ends_with(&prefix) {
                    let base_key = key.trim_end_matches(&prefix);
                    region_params.insert(base_key.to_string(), value.clone());
                }
            }

            if !region_params.is_empty() {
                region_params.insert("uuid".to_string(),
                    response.get(&format!("uuid{}", i)).cloned().unwrap_or_default());
                regions.push(self.params_to_region(&region_params)?);
            }

            i += 1;
        }

        Ok(regions)
    }
}

#[async_trait]
impl GridServiceTrait for RemoteGridService {
    async fn register_region(&self, region: &RegionInfo) -> Result<bool> {
        info!("Remote: Registering region: {} at ({}, {})",
              region.region_name, region.region_loc_x, region.region_loc_y);

        let params = self.region_to_params(region);
        let response = self.send_request("register", &params).await?;

        let result = response.get("result")
            .or_else(|| response.get("Result"))
            .map(|s| s == "true" || s == "True" || s == "TRUE" || s == "Success" || s == "success")
            .unwrap_or(false);

        if result {
            info!("Remote: Registered region: {} ({})", region.region_name, region.region_id);
        } else {
            let message = response.get("message").cloned().unwrap_or_else(|| "Unknown error".to_string());
            warn!("Remote: Failed to register region: {}", message);
        }

        Ok(result)
    }

    async fn deregister_region(&self, region_id: Uuid) -> Result<bool> {
        warn!("Remote: Deregistering region: {}", region_id);

        let mut params = HashMap::new();
        params.insert("REGIONID".to_string(), region_id.to_string());

        let response = self.send_request("deregister", &params).await?;

        Ok(response.get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE")
            .unwrap_or(false))
    }

    async fn get_region_by_uuid(&self, scope_id: Uuid, region_id: Uuid) -> Result<Option<RegionInfo>> {
        debug!("Remote: Getting region by UUID: {} (scope: {})", region_id, scope_id);

        let mut params = HashMap::new();
        params.insert("SCOPEID".to_string(), scope_id.to_string());
        params.insert("REGIONID".to_string(), region_id.to_string());

        let response = self.send_request("get_region_by_uuid", &params).await?;

        let result = response.get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if result && response.contains_key("uuid") {
            Ok(Some(self.params_to_region(&response)?))
        } else {
            Ok(None)
        }
    }

    async fn get_region_by_name(&self, scope_id: Uuid, name: &str) -> Result<Option<RegionInfo>> {
        debug!("Remote: Getting region by name: {} (scope: {})", name, scope_id);

        let mut params = HashMap::new();
        params.insert("SCOPEID".to_string(), scope_id.to_string());
        params.insert("REGIONNAME".to_string(), name.to_string());

        let response = self.send_request("get_region_by_name", &params).await?;

        let result = response.get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if result && response.contains_key("uuid") {
            Ok(Some(self.params_to_region(&response)?))
        } else {
            Ok(None)
        }
    }

    async fn get_region_by_position(&self, scope_id: Uuid, x: u32, y: u32) -> Result<Option<RegionInfo>> {
        debug!("Remote: Getting region by position: ({}, {}) (scope: {})", x, y, scope_id);

        let mut params = HashMap::new();
        params.insert("SCOPEID".to_string(), scope_id.to_string());
        params.insert("X".to_string(), x.to_string());
        params.insert("Y".to_string(), y.to_string());

        let response = self.send_request("get_region_by_position", &params).await?;

        let result = response.get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if result && response.contains_key("uuid") {
            Ok(Some(self.params_to_region(&response)?))
        } else {
            Ok(None)
        }
    }

    async fn get_neighbours(&self, scope_id: Uuid, region_id: Uuid, range: u32) -> Result<Vec<RegionInfo>> {
        debug!("Remote: Getting neighbours for region: {} (range: {})", region_id, range);

        let mut params = HashMap::new();
        params.insert("SCOPEID".to_string(), scope_id.to_string());
        params.insert("REGIONID".to_string(), region_id.to_string());
        params.insert("RANGE".to_string(), range.to_string());

        let response = self.send_request("get_neighbours", &params).await?;
        self.parse_region_list(&response)
    }

    async fn get_default_regions(&self, scope_id: Uuid) -> Result<Vec<RegionInfo>> {
        debug!("Remote: Getting default regions (scope: {})", scope_id);

        let mut params = HashMap::new();
        params.insert("SCOPEID".to_string(), scope_id.to_string());

        let response = self.send_request("get_default_regions", &params).await?;
        self.parse_region_list(&response)
    }

    async fn get_regions(&self, scope_id: Uuid, flags: u32) -> Result<Vec<RegionInfo>> {
        debug!("Remote: Getting regions with flags: {} (scope: {})", flags, scope_id);

        let mut params = HashMap::new();
        params.insert("SCOPEID".to_string(), scope_id.to_string());
        params.insert("FLAGS".to_string(), flags.to_string());

        let response = self.send_request("get_regions_by_flags", &params).await?;
        self.parse_region_list(&response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_grid_service_creation() {
        let service = RemoteGridService::new("http://localhost:8003");
        assert_eq!(service.server_uri, "http://localhost:8003");
    }

    #[test]
    fn test_params_to_region() {
        let service = RemoteGridService::new("http://localhost:8003");

        let mut params = HashMap::new();
        params.insert("uuid".to_string(), "00000000-0000-0000-0000-000000000001".to_string());
        params.insert("regionName".to_string(), "Test Region".to_string());
        params.insert("locX".to_string(), "1000".to_string());
        params.insert("locY".to_string(), "1000".to_string());

        let region = service.params_to_region(&params).unwrap();
        assert_eq!(region.region_name, "Test Region");
        assert_eq!(region.region_loc_x, 1000);
        assert_eq!(region.region_loc_y, 1000);
    }
}
