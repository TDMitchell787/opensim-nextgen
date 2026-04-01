use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, warn, debug};

use crate::services::traits::UserAgentServiceTrait;

pub struct HGInstantMessageService {
    uas: Arc<dyn UserAgentServiceTrait>,
    home_uri: String,
}

#[derive(Debug, Clone)]
pub struct GridInstantMessage {
    pub from_agent_id: Uuid,
    pub from_agent_name: String,
    pub to_agent_id: Uuid,
    pub message: String,
    pub im_session_id: Uuid,
    pub dialog: u8,
    pub from_group: bool,
    pub offline: u8,
    pub timestamp: u32,
    pub binary_bucket: Vec<u8>,
    pub region_id: Uuid,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
}

impl HGInstantMessageService {
    pub fn new(uas: Arc<dyn UserAgentServiceTrait>, home_uri: String) -> Self {
        Self { uas, home_uri }
    }

    pub async fn forward_im_to_foreign_user(&self, im: &GridInstantMessage) -> Result<bool> {
        let target_grid = self.uas.get_server_urls(im.to_agent_id).await?;
        let im_server_uri = target_grid.get("IMServerURI")
            .or_else(|| target_grid.get("HomeURI"))
            .cloned()
            .unwrap_or_default();

        if im_server_uri.is_empty() {
            warn!("[HG-IM] No IM server URI for user {}", im.to_agent_id);
            return Ok(false);
        }

        debug!("[HG-IM] Forwarding IM from {} to {} via {}", im.from_agent_id, im.to_agent_id, im_server_uri);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let body = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\
             <GridInstantMessage xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" \
             xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\">\
             <fromAgentID>{}</fromAgentID>\
             <fromAgentName>{}</fromAgentName>\
             <toAgentID>{}</toAgentID>\
             <dialog>{}</dialog>\
             <fromGroup>{}</fromGroup>\
             <message>{}</message>\
             <imSessionID>{}</imSessionID>\
             <offline>{}</offline>\
             <Position>\
             <X>{}</X><Y>{}</Y><Z>{}</Z>\
             </Position>\
             <binaryBucket>{}</binaryBucket>\
             <RegionID>{}</RegionID>\
             <timestamp>{}</timestamp>\
             </GridInstantMessage>",
            im.from_agent_id, xml_escape(&im.from_agent_name), im.to_agent_id,
            im.dialog, im.from_group, xml_escape(&im.message), im.im_session_id,
            im.offline, im.position_x, im.position_y, im.position_z,
            base64_encode(&im.binary_bucket), im.region_id, im.timestamp,
        );

        let url = format!("{}/InstantMessage/", im_server_uri.trim_end_matches('/'));
        let resp = client.post(&url)
            .header("Content-Type", "application/xml")
            .body(body)
            .send()
            .await?;

        let success = resp.status().is_success();
        if success {
            info!("[HG-IM] IM forwarded to {} at {}", im.to_agent_id, im_server_uri);
        } else {
            warn!("[HG-IM] IM forward failed: HTTP {}", resp.status());
        }
        Ok(success)
    }

    pub async fn incoming_im(&self, im: &GridInstantMessage) -> Result<bool> {
        info!("[HG-IM] Incoming cross-grid IM from {} to {}", im.from_agent_id, im.to_agent_id);
        Ok(true)
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}
