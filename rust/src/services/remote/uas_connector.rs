use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::robust::xmlrpc::{
    build_xmlrpc_call, parse_xmlrpc_response, struct_to_hashmap, XmlRpcValue,
};
use crate::services::traits::{AgentCircuitData, HGRegionInfo, UserAgentServiceTrait};

pub struct UserAgentServiceConnector {
    client: reqwest::Client,
    uas_url: String,
}

impl UserAgentServiceConnector {
    pub fn new(uas_url: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            uas_url: uas_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn with_url(uas_url: &str) -> Self {
        Self::new(uas_url)
    }

    async fn xmlrpc_call(
        &self,
        method: &str,
        params: HashMap<String, XmlRpcValue>,
    ) -> Result<XmlRpcValue> {
        let call_body = build_xmlrpc_call(method, &[XmlRpcValue::Struct(params)]);
        let url = format!("{}/", self.uas_url);

        info!("UAS XmlRpc call to {}: method={}", url, method);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "text/xml")
            .body(call_body)
            .send()
            .await
            .map_err(|e| anyhow!("UAS XmlRpc call failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read UAS response: {}", e))?;

        debug!(
            "UAS XmlRpc response: status={}, body_len={}, body_preview={}",
            status,
            body.len(),
            &body[..std::cmp::min(body.len(), 500)]
        );

        parse_xmlrpc_response(&body)
            .map_err(|e| anyhow!("Failed to parse UAS XmlRpc response: {}", e))
    }
}

#[async_trait]
impl UserAgentServiceTrait for UserAgentServiceConnector {
    async fn verify_agent(&self, session_id: Uuid, token: &str) -> Result<bool> {
        let mut params = HashMap::new();
        params.insert(
            "sessionID".to_string(),
            XmlRpcValue::String(session_id.to_string()),
        );
        params.insert("token".to_string(), XmlRpcValue::String(token.to_string()));

        let result = self.xmlrpc_call("verify_agent", params).await?;
        Ok(result.get_bool("result").unwrap_or(false))
    }

    async fn verify_client(&self, session_id: Uuid, reported_ip: &str) -> Result<bool> {
        let mut params = HashMap::new();
        params.insert(
            "sessionID".to_string(),
            XmlRpcValue::String(session_id.to_string()),
        );
        params.insert(
            "token".to_string(),
            XmlRpcValue::String(reported_ip.to_string()),
        );

        let result = self.xmlrpc_call("verify_client", params).await?;
        Ok(result.get_bool("result").unwrap_or(false))
    }

    async fn get_home_region(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(HGRegionInfo, [f32; 3], [f32; 3])>> {
        let mut params = HashMap::new();
        params.insert(
            "userID".to_string(),
            XmlRpcValue::String(user_id.to_string()),
        );

        let result = self.xmlrpc_call("get_home_region", params).await?;

        let success = result.get_bool("result").unwrap_or(false);
        if !success {
            return Ok(None);
        }

        let region = HGRegionInfo {
            region_id: result
                .get_str("uuid")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            region_handle: result
                .get_str("handle")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0),
            external_name: result.get_str("external_name").unwrap_or("").to_string(),
            image_url: result.get_str("image_url").unwrap_or("").to_string(),
            size_x: result.get_i32("region_size_x").unwrap_or(256) as u32,
            size_y: result.get_i32("region_size_y").unwrap_or(256) as u32,
            http_port: result.get_i32("http_port").unwrap_or(0) as u16,
            server_uri: result.get_str("server_uri").unwrap_or("").to_string(),
            region_name: result.get_str("region_name").unwrap_or("").to_string(),
            region_loc_x: result.get_i32("region_xloc").unwrap_or(0) as u32,
            region_loc_y: result.get_i32("region_yloc").unwrap_or(0) as u32,
            hostname: result.get_str("hostname").unwrap_or("").to_string(),
            internal_port: result.get_i32("internal_port").unwrap_or(0) as u16,
        };

        let position = parse_vector3(result.get_str("position").unwrap_or("<128,128,21>"));
        let look_at = parse_vector3(result.get_str("lookAt").unwrap_or("<0,1,0>"));

        Ok(Some((region, position, look_at)))
    }

    async fn get_server_urls(&self, user_id: Uuid) -> Result<HashMap<String, String>> {
        let mut params = HashMap::new();
        params.insert(
            "userID".to_string(),
            XmlRpcValue::String(user_id.to_string()),
        );

        let result = self.xmlrpc_call("get_server_urls", params).await?;

        let success = result.get_bool("result").unwrap_or(false);
        if !success {
            return Ok(HashMap::new());
        }

        let mut urls = struct_to_hashmap(&result);
        urls.remove("result");
        Ok(urls)
    }

    async fn logout_agent(&self, user_id: Uuid, session_id: Uuid) -> Result<()> {
        let mut params = HashMap::new();
        params.insert(
            "userID".to_string(),
            XmlRpcValue::String(user_id.to_string()),
        );
        params.insert(
            "sessionID".to_string(),
            XmlRpcValue::String(session_id.to_string()),
        );

        let _ = self.xmlrpc_call("logout_agent", params).await;
        Ok(())
    }

    async fn get_uui(&self, user_id: Uuid, target_user_id: Uuid) -> Result<String> {
        let mut params = HashMap::new();
        params.insert(
            "userID".to_string(),
            XmlRpcValue::String(user_id.to_string()),
        );
        params.insert(
            "targetUserID".to_string(),
            XmlRpcValue::String(target_user_id.to_string()),
        );

        let result = self.xmlrpc_call("get_uui", params).await?;
        Ok(result
            .get_str("UUI")
            .unwrap_or(&target_user_id.to_string())
            .to_string())
    }

    async fn get_uuid(&self, first: &str, last: &str) -> Result<Option<Uuid>> {
        let mut params = HashMap::new();
        params.insert("first".to_string(), XmlRpcValue::String(first.to_string()));
        params.insert("last".to_string(), XmlRpcValue::String(last.to_string()));

        let result = self.xmlrpc_call("get_uuid", params).await?;

        let success = result.get_bool("result").unwrap_or(false);
        if !success {
            return Ok(None);
        }

        result
            .get_str("userID")
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(Some)
            .ok_or_else(|| anyhow!("Invalid UUID in get_uuid response"))
    }

    async fn status_notification(
        &self,
        friends: &[String],
        user_id: Uuid,
        online: bool,
    ) -> Result<Vec<Uuid>> {
        let mut params = HashMap::new();
        params.insert(
            "userID".to_string(),
            XmlRpcValue::String(user_id.to_string()),
        );
        params.insert("online".to_string(), XmlRpcValue::Bool(online));

        let friend_values: Vec<XmlRpcValue> = friends
            .iter()
            .map(|f| XmlRpcValue::String(f.clone()))
            .collect();
        params.insert("friends".to_string(), XmlRpcValue::Array(friend_values));

        let result = self.xmlrpc_call("status_notification", params).await?;

        let mut online_friends = Vec::new();
        if let Some(arr) = result.get("online_friends").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    if let Ok(uuid) = Uuid::parse_str(s) {
                        online_friends.push(uuid);
                    }
                }
            }
        }
        Ok(online_friends)
    }

    async fn is_agent_coming_home(&self, session_id: Uuid, external_name: &str) -> Result<bool> {
        let mut params = HashMap::new();
        params.insert(
            "sessionID".to_string(),
            XmlRpcValue::String(session_id.to_string()),
        );
        params.insert(
            "externalName".to_string(),
            XmlRpcValue::String(external_name.to_string()),
        );

        let result = self.xmlrpc_call("agent_is_coming_home", params).await?;
        Ok(result.get_bool("result").unwrap_or(false))
    }

    async fn login_agent_to_grid(
        &self,
        agent: &AgentCircuitData,
        _gatekeeper: &HGRegionInfo,
        destination: &HGRegionInfo,
        from_login: bool,
    ) -> Result<(bool, String)> {
        let url = format!("{}/homeagent/{}", self.uas_url, agent.agent_id);

        let json_body = serde_json::json!({
            "agent_id": agent.agent_id.to_string(),
            "session_id": agent.session_id.to_string(),
            "secure_session_id": agent.secure_session_id.to_string(),
            "circuit_code": agent.circuit_code,
            "first_name": agent.first_name,
            "last_name": agent.last_name,
            "start_pos": format!("<{},{},{}>", agent.start_pos[0], agent.start_pos[1], agent.start_pos[2]),
            "client_ip": agent.client_ip,
            "teleport_flags": agent.teleport_flags,
            "destination_uuid": destination.region_id.to_string(),
            "destination_name": destination.region_name,
            "destination_handle": destination.region_handle.to_string(),
            "destination_server_uri": destination.server_uri,
            "from_login": from_login,
            "service_session_id": agent.service_session_id,
            "service_urls": agent.service_urls,
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_body.to_string())
            .send()
            .await
            .map_err(|e| anyhow!("Home agent POST failed: {}", e))?;

        let resp_body = response.text().await.unwrap_or_default();

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp_body) {
            let success = json
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let reason = json
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            Ok((success, reason))
        } else {
            Ok((false, format!("Invalid response: {}", resp_body)))
        }
    }

    async fn get_user_info(&self, user_id: Uuid) -> Result<HashMap<String, String>> {
        let mut params = HashMap::new();
        params.insert(
            "userID".to_string(),
            XmlRpcValue::String(user_id.to_string()),
        );

        let result = self.xmlrpc_call("get_user_info", params).await?;
        let mut info = HashMap::new();
        if let XmlRpcValue::Struct(map) = result {
            for (k, v) in map {
                if let XmlRpcValue::String(s) = v {
                    info.insert(k, s);
                }
            }
        }
        Ok(info)
    }
}

fn parse_vector3(s: &str) -> [f32; 3] {
    let trimmed = s.trim_start_matches('<').trim_end_matches('>');
    let parts: Vec<f32> = trimmed
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .collect();
    [
        parts.first().copied().unwrap_or(128.0),
        parts.get(1).copied().unwrap_or(128.0),
        parts.get(2).copied().unwrap_or(21.0),
    ]
}
