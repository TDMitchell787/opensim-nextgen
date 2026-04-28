use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{AvatarData, AvatarServiceTrait};

pub struct RemoteAvatarService {
    client: Client,
    server_uri: String,
}

impl RemoteAvatarService {
    pub fn new(server_uri: &str) -> Self {
        info!("Initializing remote avatar service: {}", server_uri);

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
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
        let url = format!("{}/avatar", self.server_uri);

        let mut form_data = params.clone();
        form_data.insert("METHOD".to_string(), method.to_string());

        debug!("Avatar service request: {} to {}", method, url);

        let response = self
            .client
            .post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| anyhow!("Avatar service request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Avatar service returned status: {}",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read avatar service response: {}", e))?;

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
}

#[async_trait]
impl AvatarServiceTrait for RemoteAvatarService {
    async fn get_avatar(&self, principal_id: Uuid) -> Result<AvatarData> {
        debug!("Remote: Getting avatar data for: {}", principal_id);

        let mut params = HashMap::new();
        params.insert("UserID".to_string(), principal_id.to_string());

        let response = self.send_request("getavatar", &params).await?;

        let result = response
            .get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if !result {
            return Ok(AvatarData {
                avatar_type: 1,
                data: HashMap::new(),
            });
        }

        let mut avatar_data = AvatarData {
            avatar_type: response
                .get("AvatarType")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
            data: HashMap::new(),
        };

        for (key, value) in &response {
            if key != "result" && key != "AvatarType" && key != "METHOD" {
                avatar_data.data.insert(key.clone(), value.clone());
            }
        }

        Ok(avatar_data)
    }

    async fn set_avatar(&self, principal_id: Uuid, data: &AvatarData) -> Result<bool> {
        debug!("Remote: Setting avatar data for: {}", principal_id);

        let mut params = HashMap::new();
        params.insert("UserID".to_string(), principal_id.to_string());
        params.insert("AvatarType".to_string(), data.avatar_type.to_string());

        for (key, value) in &data.data {
            params.insert(key.clone(), value.clone());
        }

        let response = self.send_request("setavatar", &params).await?;

        Ok(response
            .get("result")
            .map(|s| s == "true" || s == "True" || s == "Success")
            .unwrap_or(false))
    }

    async fn reset_avatar(&self, principal_id: Uuid) -> Result<bool> {
        debug!("Remote: Resetting avatar data for: {}", principal_id);

        let mut params = HashMap::new();
        params.insert("UserID".to_string(), principal_id.to_string());

        let response = self.send_request("resetavatar", &params).await?;

        Ok(response
            .get("result")
            .map(|s| s == "true" || s == "True" || s == "Success")
            .unwrap_or(false))
    }

    async fn remove_items(&self, principal_id: Uuid, names: &[String]) -> Result<bool> {
        debug!(
            "Remote: Removing {} avatar items for: {}",
            names.len(),
            principal_id
        );

        let mut params = HashMap::new();
        params.insert("UserID".to_string(), principal_id.to_string());

        for (i, name) in names.iter().enumerate() {
            params.insert(format!("Name_{}", i), name.clone());
        }
        params.insert("Names".to_string(), names.len().to_string());

        let response = self.send_request("removeitems", &params).await?;

        Ok(response
            .get("result")
            .map(|s| s == "true" || s == "True" || s == "Success")
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_avatar_service_creation() {
        let service = RemoteAvatarService::new("http://localhost:8003");
        assert_eq!(service.server_uri, "http://localhost:8003");
    }

    #[test]
    fn test_parse_response() {
        let service = RemoteAvatarService::new("http://localhost:8003");
        let body = "result=Success\nAvatarType=1\nSerial=1";
        let result = service.parse_response(body).unwrap();
        assert_eq!(result.get("result").unwrap(), "Success");
        assert_eq!(result.get("AvatarType").unwrap(), "1");
    }
}
