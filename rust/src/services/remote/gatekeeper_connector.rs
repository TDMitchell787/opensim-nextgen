use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::robust::xmlrpc::{build_xmlrpc_call, parse_xmlrpc_response, XmlRpcValue};
use crate::services::traits::{AgentCircuitData, GatekeeperServiceTrait, HGRegionInfo};

pub struct GatekeeperServiceConnector {
    client: reqwest::Client,
    gatekeeper_url: String,
}

impl GatekeeperServiceConnector {
    pub fn new(gatekeeper_url: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            gatekeeper_url: gatekeeper_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn with_url(gatekeeper_url: &str) -> Self {
        Self::new(gatekeeper_url)
    }

    async fn xmlrpc_call(
        &self,
        method: &str,
        params: HashMap<String, XmlRpcValue>,
    ) -> Result<XmlRpcValue> {
        let call_body = build_xmlrpc_call(method, &[XmlRpcValue::Struct(params)]);
        let url = self.gatekeeper_url.clone();

        debug!("XmlRpc call to {}: method={}", url, method);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "text/xml")
            .body(call_body)
            .send()
            .await
            .map_err(|e| anyhow!("Gatekeeper XmlRpc call failed: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read gatekeeper response: {}", e))?;

        parse_xmlrpc_response(&body)
            .map_err(|e| anyhow!("Failed to parse gatekeeper XmlRpc response: {}", e))
    }

    fn parse_region_info(value: &XmlRpcValue) -> Option<HGRegionInfo> {
        let map = value.as_struct()?;
        let handle_from_field = map
            .get("handle")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let loc_x_from_handle = ((handle_from_field >> 32) & 0xFFFFFFFF) as u32;
        let loc_y_from_handle = (handle_from_field & 0xFFFFFFFF) as u32;

        let loc_x = map
            .get("region_xloc")
            .or_else(|| map.get("x"))
            .and_then(|v| {
                v.as_i32()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .map(|v| v as u32)
            .unwrap_or(loc_x_from_handle);
        let loc_y = map
            .get("region_yloc")
            .or_else(|| map.get("y"))
            .and_then(|v| {
                v.as_i32()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .map(|v| v as u32)
            .unwrap_or(loc_y_from_handle);

        let handle = if handle_from_field != 0 {
            handle_from_field
        } else if loc_x != 0 || loc_y != 0 {
            ((loc_x as u64) << 32) | (loc_y as u64)
        } else {
            0
        };

        let ext_name = map
            .get("external_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let (ext_server_uri, ext_region_name) = {
            let after_scheme = if ext_name.starts_with("https://") {
                Some(8usize)
            } else if ext_name.starts_with("http://") {
                Some(7usize)
            } else {
                None
            };
            if let Some(scheme_end) = after_scheme {
                if let Some(rel_space) = ext_name[scheme_end..].find(' ') {
                    let space_pos = scheme_end + rel_space;
                    (
                        ext_name[..space_pos].trim().to_string(),
                        ext_name[space_pos + 1..].trim().to_string(),
                    )
                } else {
                    (ext_name.trim_end_matches('/').to_string(), String::new())
                }
            } else if let Some(space_pos) = ext_name.find(' ') {
                (
                    ext_name[..space_pos].trim().to_string(),
                    ext_name[space_pos + 1..].trim().to_string(),
                )
            } else {
                (ext_name.trim_end_matches('/').to_string(), String::new())
            }
        };

        let explicit_region_name = map
            .get("region_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("")
            .to_string();

        let region_name = if !explicit_region_name.is_empty() {
            explicit_region_name
        } else {
            ext_region_name
        };

        let server_uri = map
            .get("server_uri")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                if !ext_server_uri.is_empty() {
                    ext_server_uri.clone()
                } else {
                    ext_name.to_string()
                }
            });

        Some(HGRegionInfo {
            region_id: map
                .get("uuid")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            region_handle: handle,
            external_name: ext_name.to_string(),
            image_url: map
                .get("image_url")
                .or_else(|| map.get("region_image"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            size_x: map
                .get("region_size_x")
                .or_else(|| map.get("size_x"))
                .and_then(|v| v.as_i32())
                .unwrap_or(256) as u32,
            size_y: map
                .get("region_size_y")
                .or_else(|| map.get("size_y"))
                .and_then(|v| v.as_i32())
                .unwrap_or(256) as u32,
            http_port: map.get("http_port").and_then(|v| v.as_i32()).unwrap_or(0) as u16,
            server_uri,
            region_name,
            region_loc_x: loc_x,
            region_loc_y: loc_y,
            hostname: map
                .get("hostname")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            internal_port: map
                .get("internal_port")
                .and_then(|v| {
                    v.as_i32()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .unwrap_or(0) as u16,
        })
    }
}

#[async_trait]
impl GatekeeperServiceTrait for GatekeeperServiceConnector {
    async fn link_region(&self, region_name: &str) -> Result<Option<HGRegionInfo>> {
        let mut params = HashMap::new();
        params.insert(
            "region_name".to_string(),
            XmlRpcValue::String(region_name.to_string()),
        );

        let result = self.xmlrpc_call("link_region", params).await?;

        info!("link_region response: {:?}", result);

        let success = result.get_bool("result").unwrap_or(false)
            || result
                .get_str("result")
                .map(|s| s.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
        if !success {
            let reason = result.get_str("message").unwrap_or("unknown error");
            info!("link_region failed: {}", reason);
            return Ok(None);
        }

        Ok(Self::parse_region_info(&result))
    }

    async fn get_hyperlinkregion(
        &self,
        region_id: Uuid,
        agent_id: Uuid,
        agent_home_uri: &str,
    ) -> Result<Option<HGRegionInfo>> {
        let mut params = HashMap::new();
        params.insert(
            "region_uuid".to_string(),
            XmlRpcValue::String(region_id.to_string()),
        );
        params.insert(
            "agent_id".to_string(),
            XmlRpcValue::String(agent_id.to_string()),
        );
        params.insert(
            "agent_home_uri".to_string(),
            XmlRpcValue::String(agent_home_uri.to_string()),
        );

        let result = self.xmlrpc_call("get_region", params).await?;

        info!("get_region response: {:?}", result);

        let success = result.get_bool("result").unwrap_or(false)
            || result
                .get_str("result")
                .map(|s| s.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
        if !success {
            return Ok(None);
        }

        Ok(Self::parse_region_info(&result))
    }

    async fn login_agent(
        &self,
        source: &HGRegionInfo,
        agent_data: &AgentCircuitData,
        destination: &HGRegionInfo,
    ) -> Result<(bool, String)> {
        info!(
            "login_agent to remote gatekeeper: {} -> {}",
            agent_data.agent_id, destination.region_name
        );

        let url = format!(
            "{}/foreignagent/{}/",
            self.gatekeeper_url, agent_data.agent_id
        );

        let service_urls_array: Vec<serde_json::Value> = agent_data
            .service_urls
            .iter()
            .flat_map(|(k, v)| {
                vec![
                    serde_json::Value::String(k.clone()),
                    serde_json::Value::String(v.clone()),
                ]
            })
            .collect();

        let mut serviceurls_map = serde_json::Map::new();
        for (k, v) in &agent_data.service_urls {
            serviceurls_map.insert(k.clone(), serde_json::Value::String(v.clone()));
        }

        let json_body = serde_json::json!({
            "agent_id": agent_data.agent_id.to_string(),
            "base_folder": "00000000-0000-0000-0000-000000000000",
            "caps_path": agent_data.caps_path,
            "child": false,
            "circuit_code": agent_data.circuit_code.to_string(),
            "first_name": agent_data.first_name,
            "last_name": agent_data.last_name,
            "secure_session_id": agent_data.secure_session_id.to_string(),
            "session_id": agent_data.session_id.to_string(),
            "service_session_id": agent_data.service_session_id,
            "start_pos": format!("<{}, {}, {}>", agent_data.start_pos[0], agent_data.start_pos[1], agent_data.start_pos[2]),
            "client_ip": agent_data.client_ip,
            "viewer": "Firestorm",
            "channel": "Firestorm-Release",
            "mac": agent_data.mac,
            "id0": agent_data.id0,
            "teleport_flags": agent_data.teleport_flags.to_string(),
            "service_urls": service_urls_array,
            "serviceurls": serviceurls_map,
            "destination_x": destination.region_loc_x.to_string(),
            "destination_y": destination.region_loc_y.to_string(),
            "destination_name": destination.region_name,
            "destination_uuid": destination.region_id.to_string(),
            "source_x": source.region_loc_x.to_string(),
            "source_y": source.region_loc_y.to_string(),
            "source_name": source.region_name,
            "source_uuid": source.region_id.to_string(),
            "source_server_uri": source.server_uri,
        });

        info!("foreignagent POST to {}", url);
        info!("foreignagent body: {}", json_body);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_body.to_string())
            .send()
            .await
            .map_err(|e| anyhow!("Foreign agent POST failed: {}", e))?;

        let status = response.status();
        let resp_body = response.text().await.unwrap_or_default();
        info!("foreignagent response (HTTP {}): {}", status, resp_body);

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp_body) {
            let success = json
                .get("success")
                .and_then(|v| v.as_bool())
                .or_else(|| {
                    json.get("success")
                        .and_then(|v| v.as_str())
                        .map(|s| s.eq_ignore_ascii_case("true"))
                })
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
}
