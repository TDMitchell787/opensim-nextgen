//! Remote Authentication Service Implementation
//!
//! Connects to ROBUST AuthenticationService via HTTP.
//! Implements OpenSim's AuthenticationServicesConnector protocol.
//!
//! Reference: OpenSim/Services/Connectors/Authentication/AuthenticationServicesConnector.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{AuthenticationServiceTrait, AuthInfo};

pub struct RemoteAuthenticationService {
    client: Client,
    server_uri: String,
}

impl RemoteAuthenticationService {
    pub fn new(server_uri: &str) -> Self {
        info!("Initializing remote authentication service: {}", server_uri);

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
        let url = format!("{}/auth/plain", self.server_uri);

        let mut form_data = params.clone();
        form_data.insert("METHOD".to_string(), method.to_string());

        debug!("Authentication service request: {} to {}", method, url);

        let response = self.client
            .post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| anyhow!("Authentication service request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Authentication service returned status: {}", response.status()));
        }

        let body = response.text().await
            .map_err(|e| anyhow!("Failed to read authentication service response: {}", e))?;

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
}

#[async_trait]
impl AuthenticationServiceTrait for RemoteAuthenticationService {
    async fn authenticate(&self, principal_id: Uuid, password: &str, lifetime: i32) -> Result<Option<String>> {
        debug!("Remote: Authenticating user: {}", principal_id);

        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        params.insert("PASSWORD".to_string(), password.to_string());
        params.insert("LIFETIME".to_string(), lifetime.to_string());

        let response = self.send_request("authenticate", &params).await?;

        let result = response.get("result")
            .map(|s| s != "Failure" && s != "null")
            .unwrap_or(false);

        if result {
            let token = response.get("token")
                .or_else(|| response.get("result"))
                .cloned()
                .unwrap_or_default();
            info!("Remote: Authentication successful for user: {}", principal_id);
            Ok(Some(token))
        } else {
            warn!("Remote: Authentication failed for user: {}", principal_id);
            Ok(None)
        }
    }

    async fn verify(&self, principal_id: Uuid, token: &str, lifetime: i32) -> Result<bool> {
        debug!("Remote: Verifying token for user: {}", principal_id);

        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        params.insert("TOKEN".to_string(), token.to_string());
        params.insert("LIFETIME".to_string(), lifetime.to_string());

        let response = self.send_request("verify", &params).await?;

        Ok(response.get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE")
            .unwrap_or(false))
    }

    async fn release(&self, principal_id: Uuid, token: &str) -> Result<bool> {
        debug!("Remote: Releasing token for user: {}", principal_id);

        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        params.insert("TOKEN".to_string(), token.to_string());

        let response = self.send_request("release", &params).await?;

        Ok(response.get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE")
            .unwrap_or(false))
    }

    async fn set_password(&self, principal_id: Uuid, password: &str) -> Result<bool> {
        info!("Remote: Setting password for user: {}", principal_id);

        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());
        params.insert("PASSWORD".to_string(), password.to_string());

        let response = self.send_request("setpassword", &params).await?;

        Ok(response.get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE")
            .unwrap_or(false))
    }

    async fn get_authentication(&self, principal_id: Uuid) -> Result<Option<AuthInfo>> {
        debug!("Remote: Getting authentication info for: {}", principal_id);

        let mut params = HashMap::new();
        params.insert("PRINCIPAL".to_string(), principal_id.to_string());

        let response = self.send_request("getauthinfo", &params).await?;

        let result = response.get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if result && response.contains_key("PrincipalID") {
            Ok(Some(AuthInfo {
                principal_id,
                password_hash: response.get("PasswordHash").cloned().unwrap_or_default(),
                password_salt: response.get("PasswordSalt").cloned().unwrap_or_default(),
                web_login_key: response.get("WebLoginKey").cloned().unwrap_or_default(),
                account_type: response.get("AccountType").cloned().unwrap_or_else(|| "UserAccount".to_string()),
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_auth_service_creation() {
        let service = RemoteAuthenticationService::new("http://localhost:8003");
        assert_eq!(service.server_uri, "http://localhost:8003");
    }
}
