//! Remote User Account Service Implementation
//!
//! Connects to ROBUST UserAccountService via HTTP.
//! Implements OpenSim's UserAccountServicesConnector protocol.
//!
//! Reference: OpenSim/Services/Connectors/UserAccounts/UserAccountServicesConnector.cs

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{UserAccount, UserAccountServiceTrait};

pub struct RemoteUserAccountService {
    client: Client,
    server_uri: String,
}

impl RemoteUserAccountService {
    pub fn new(server_uri: &str) -> Self {
        info!("Initializing remote user account service: {}", server_uri);

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
        let url = format!("{}/accounts", self.server_uri);

        let mut form_data = params.clone();
        form_data.insert("METHOD".to_string(), method.to_string());

        debug!("User account service request: {} to {}", method, url);

        let response = self
            .client
            .post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| anyhow!("User account service request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "User account service returned status: {}",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read user account service response: {}", e))?;

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

    fn params_to_user_account(&self, params: &HashMap<String, String>) -> Result<UserAccount> {
        let service_urls = params
            .get("ServiceURLs")
            .map(|s| self.parse_service_urls(s))
            .unwrap_or_default();

        Ok(UserAccount {
            principal_id: params
                .get("PrincipalID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            scope_id: params
                .get("ScopeID")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            first_name: params.get("FirstName").cloned().unwrap_or_default(),
            last_name: params.get("LastName").cloned().unwrap_or_default(),
            email: params.get("Email").cloned().unwrap_or_default(),
            service_urls,
            created: params
                .get("Created")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            user_level: params
                .get("UserLevel")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            user_flags: params
                .get("UserFlags")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            user_title: params.get("UserTitle").cloned().unwrap_or_default(),
        })
    }

    fn user_account_to_params(&self, account: &UserAccount) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("PrincipalID".to_string(), account.principal_id.to_string());
        params.insert("ScopeID".to_string(), account.scope_id.to_string());
        params.insert("FirstName".to_string(), account.first_name.clone());
        params.insert("LastName".to_string(), account.last_name.clone());
        params.insert("Email".to_string(), account.email.clone());
        params.insert(
            "ServiceURLs".to_string(),
            self.serialize_service_urls(&account.service_urls),
        );
        params.insert("Created".to_string(), account.created.to_string());
        params.insert("UserLevel".to_string(), account.user_level.to_string());
        params.insert("UserFlags".to_string(), account.user_flags.to_string());
        params.insert("UserTitle".to_string(), account.user_title.clone());
        params
    }

    fn parse_service_urls(&self, data: &str) -> HashMap<String, String> {
        let mut urls = HashMap::new();
        if data.is_empty() {
            return urls;
        }

        for pair in data.split(';') {
            if let Some((key, value)) = pair.split_once('*') {
                urls.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        urls
    }

    fn serialize_service_urls(&self, urls: &HashMap<String, String>) -> String {
        urls.iter()
            .map(|(k, v)| format!("{}*{}", k, v))
            .collect::<Vec<_>>()
            .join(";")
    }

    fn parse_user_list(&self, response: &HashMap<String, String>) -> Result<Vec<UserAccount>> {
        let mut users = Vec::new();
        let mut i = 0;

        while response.contains_key(&format!("PrincipalID{}", i)) {
            let suffix = i.to_string();
            let mut user_params = HashMap::new();

            for (key, value) in response {
                if key.ends_with(&suffix) {
                    let base_key = key.trim_end_matches(&suffix);
                    user_params.insert(base_key.to_string(), value.clone());
                }
            }

            if !user_params.is_empty() {
                users.push(self.params_to_user_account(&user_params)?);
            }

            i += 1;
        }

        Ok(users)
    }
}

#[async_trait]
impl UserAccountServiceTrait for RemoteUserAccountService {
    async fn get_user_account(&self, scope_id: Uuid, user_id: Uuid) -> Result<Option<UserAccount>> {
        debug!(
            "Remote: Getting user account by ID: {} (scope: {})",
            user_id, scope_id
        );

        let mut params = HashMap::new();
        params.insert("ScopeID".to_string(), scope_id.to_string());
        params.insert("UserID".to_string(), user_id.to_string());

        let response = self.send_request("getaccount", &params).await?;

        let result = response
            .get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if result && response.contains_key("PrincipalID") {
            Ok(Some(self.params_to_user_account(&response)?))
        } else {
            Ok(None)
        }
    }

    async fn get_user_account_by_name(
        &self,
        scope_id: Uuid,
        first: &str,
        last: &str,
    ) -> Result<Option<UserAccount>> {
        debug!(
            "Remote: Getting user account by name: {} {} (scope: {})",
            first, last, scope_id
        );

        let mut params = HashMap::new();
        params.insert("ScopeID".to_string(), scope_id.to_string());
        params.insert("FirstName".to_string(), first.to_string());
        params.insert("LastName".to_string(), last.to_string());

        let response = self.send_request("getaccount", &params).await?;

        let result = response
            .get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if result && response.contains_key("PrincipalID") {
            Ok(Some(self.params_to_user_account(&response)?))
        } else {
            Ok(None)
        }
    }

    async fn get_user_account_by_email(
        &self,
        scope_id: Uuid,
        email: &str,
    ) -> Result<Option<UserAccount>> {
        debug!(
            "Remote: Getting user account by email: {} (scope: {})",
            email, scope_id
        );

        let mut params = HashMap::new();
        params.insert("ScopeID".to_string(), scope_id.to_string());
        params.insert("Email".to_string(), email.to_string());

        let response = self.send_request("getaccount", &params).await?;

        let result = response
            .get("result")
            .map(|s| s != "null" && s != "Failure")
            .unwrap_or(true);

        if result && response.contains_key("PrincipalID") {
            Ok(Some(self.params_to_user_account(&response)?))
        } else {
            Ok(None)
        }
    }

    async fn store_user_account(&self, data: &UserAccount) -> Result<bool> {
        info!(
            "Remote: Storing user account: {} {} ({})",
            data.first_name, data.last_name, data.principal_id
        );

        let params = self.user_account_to_params(data);
        let response = self.send_request("setaccount", &params).await?;

        let result = response
            .get("result")
            .map(|s| s == "true" || s == "True" || s == "TRUE")
            .unwrap_or(false);

        if result {
            info!(
                "Remote: Stored user account: {} {}",
                data.first_name, data.last_name
            );
        } else {
            let message = response
                .get("message")
                .cloned()
                .unwrap_or_else(|| "Unknown error".to_string());
            warn!("Remote: Failed to store user account: {}", message);
        }

        Ok(result)
    }

    async fn get_user_accounts(&self, scope_id: Uuid, query: &str) -> Result<Vec<UserAccount>> {
        debug!(
            "Remote: Searching user accounts: {} (scope: {})",
            query, scope_id
        );

        let mut params = HashMap::new();
        params.insert("ScopeID".to_string(), scope_id.to_string());
        params.insert("query".to_string(), query.to_string());

        let response = self.send_request("getaccounts", &params).await?;
        self.parse_user_list(&response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_user_account_service_creation() {
        let service = RemoteUserAccountService::new("http://localhost:8003");
        assert_eq!(service.server_uri, "http://localhost:8003");
    }

    #[test]
    fn test_service_urls_parsing() {
        let service = RemoteUserAccountService::new("http://localhost:8003");

        let urls_str = "HomeURI*http://localhost:8002;GatekeeperURI*http://localhost:8002";
        let urls = service.parse_service_urls(urls_str);

        assert_eq!(
            urls.get("HomeURI"),
            Some(&"http://localhost:8002".to_string())
        );
        assert_eq!(
            urls.get("GatekeeperURI"),
            Some(&"http://localhost:8002".to_string())
        );
    }

    #[test]
    fn test_service_urls_serialization() {
        let service = RemoteUserAccountService::new("http://localhost:8003");

        let mut urls = HashMap::new();
        urls.insert("HomeURI".to_string(), "http://localhost:8002".to_string());

        let serialized = service.serialize_service_urls(&urls);
        assert!(serialized.contains("HomeURI*http://localhost:8002"));
    }
}
