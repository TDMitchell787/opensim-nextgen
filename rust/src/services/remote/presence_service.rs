//! Remote Presence Service Implementation
//!
//! Connects to ROBUST PresenceService via HTTP.
//! Implements OpenSim's PresenceServicesConnector protocol.
//!
//! Reference: OpenSim/Services/Connectors/Presence/PresenceServicesConnector.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{PresenceServiceTrait, PresenceInfo};

pub struct RemotePresenceService {
    client: Client,
    server_uri: String,
}

impl RemotePresenceService {
    pub fn new(server_uri: &str) -> Self {
        info!("Initializing remote presence service: {}", server_uri);

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
        let url = format!("{}/presence", self.server_uri);

        let mut form_data = params.clone();
        form_data.insert("METHOD".to_string(), method.to_string());

        debug!("Presence service request: {} to {}", method, url);

        let response = self.client
            .post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| anyhow!("Presence service request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Presence service returned status: {}", response.status()));
        }

        let body = response.text().await
            .map_err(|e| anyhow!("Failed to read presence service response: {}", e))?;

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

    fn params_to_presence_info(&self, params: &HashMap<String, String>) -> PresenceInfo {
        PresenceInfo {
            user_id: params.get("UserID")
                .or_else(|| params.get("userid"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            session_id: params.get("SessionID")
                .or_else(|| params.get("sessionid"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            secure_session_id: params.get("SecureSessionID")
                .or_else(|| params.get("securesessionid"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            region_id: params.get("RegionID")
                .or_else(|| params.get("regionid"))
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            online: params.get("Online")
                .or_else(|| params.get("online"))
                .map(|s| s == "True" || s == "true" || s == "1")
                .unwrap_or(true),
            login_time: params.get("Login")
                .or_else(|| params.get("login"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            logout_time: params.get("Logout")
                .or_else(|| params.get("logout"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        }
    }

    fn parse_presence_list(&self, response: &HashMap<String, String>) -> Vec<PresenceInfo> {
        let mut presences = Vec::new();
        let mut i = 0;

        while response.contains_key(&format!("UserID_{}", i)) || response.contains_key(&format!("userid_{}", i)) {
            let suffix = format!("_{}", i);
            let mut presence_params = HashMap::new();

            for (key, value) in response {
                if key.ends_with(&suffix) {
                    let base_key = key.strip_suffix(&suffix).unwrap_or(key);
                    presence_params.insert(base_key.to_string(), value.clone());
                }
            }

            if !presence_params.is_empty() {
                presences.push(self.params_to_presence_info(&presence_params));
            }

            i += 1;
        }

        presences
    }
}

#[async_trait]
impl PresenceServiceTrait for RemotePresenceService {
    async fn login_agent(&self, user_id: Uuid, session_id: Uuid, secure_session_id: Uuid, region_id: Uuid) -> Result<bool> {
        info!("Remote: Logging in agent: {} to region: {}", user_id, region_id);

        let mut params = HashMap::new();
        params.insert("UserID".to_string(), user_id.to_string());
        params.insert("SessionID".to_string(), session_id.to_string());
        params.insert("SecureSessionID".to_string(), secure_session_id.to_string());
        params.insert("RegionID".to_string(), region_id.to_string());

        let response = self.send_request("login", &params).await?;

        let result = response.get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE")
            .unwrap_or(false);

        if result {
            info!("Remote: Agent logged in: {} (session: {})", user_id, session_id);
        } else {
            warn!("Remote: Failed to login agent: {}", user_id);
        }

        Ok(result)
    }

    async fn logout_agent(&self, session_id: Uuid) -> Result<bool> {
        info!("Remote: Logging out agent session: {}", session_id);

        let mut params = HashMap::new();
        params.insert("SessionID".to_string(), session_id.to_string());

        let response = self.send_request("logout", &params).await?;

        Ok(response.get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE")
            .unwrap_or(false))
    }

    async fn report_agent(&self, session_id: Uuid, region_id: Uuid) -> Result<bool> {
        debug!("Remote: Reporting agent location: {} in region: {}", session_id, region_id);

        let mut params = HashMap::new();
        params.insert("SessionID".to_string(), session_id.to_string());
        params.insert("RegionID".to_string(), region_id.to_string());

        let response = self.send_request("report", &params).await?;

        Ok(response.get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE")
            .unwrap_or(false))
    }

    async fn get_agent(&self, session_id: Uuid) -> Result<Option<PresenceInfo>> {
        debug!("Remote: Getting agent presence: {}", session_id);

        let mut params = HashMap::new();
        params.insert("SessionID".to_string(), session_id.to_string());

        let response = self.send_request("getagent", &params).await?;

        let result = response.get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if result && response.contains_key("UserID") {
            Ok(Some(self.params_to_presence_info(&response)))
        } else {
            Ok(None)
        }
    }

    async fn get_agents(&self, user_ids: &[Uuid]) -> Result<Vec<PresenceInfo>> {
        debug!("Remote: Getting presence for {} users", user_ids.len());

        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        let user_ids_str = user_ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let mut params = HashMap::new();
        params.insert("UIDS".to_string(), user_ids_str);

        let response = self.send_request("getagents", &params).await?;
        Ok(self.parse_presence_list(&response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_presence_service_creation() {
        let service = RemotePresenceService::new("http://localhost:8003");
        assert_eq!(service.server_uri, "http://localhost:8003");
    }

    #[test]
    fn test_params_to_presence_info() {
        let service = RemotePresenceService::new("http://localhost:8003");

        let mut params = HashMap::new();
        params.insert("UserID".to_string(), "00000000-0000-0000-0000-000000000001".to_string());
        params.insert("SessionID".to_string(), "00000000-0000-0000-0000-000000000002".to_string());
        params.insert("Online".to_string(), "True".to_string());

        let info = service.params_to_presence_info(&params);
        assert!(info.online);
        assert!(!info.user_id.is_nil());
    }
}
