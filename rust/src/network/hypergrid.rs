use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::Result;
use tracing::{info, warn, debug};

use crate::services::traits::{
    GatekeeperServiceTrait, UserAgentServiceTrait,
    HGRegionInfo, AgentCircuitData,
};
use crate::services::remote::GatekeeperServiceConnector;
use crate::services::remote::UserAgentServiceConnector;
use crate::services::hypergrid::uui;

#[derive(Debug, Clone)]
pub struct HypergridConfig {
    pub enabled: bool,
    pub home_uri: String,
    pub gatekeeper_uri: String,
    pub external_uri: String,
    pub external_robust_uri: String,
    pub grid_name: String,
    pub allow_teleports_to_any_region: bool,
    pub foreign_agents_allowed: bool,
}

impl Default for HypergridConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            home_uri: "http://localhost:8002".to_string(),
            gatekeeper_uri: "http://localhost:8002".to_string(),
            external_uri: String::new(),
            external_robust_uri: String::new(),
            grid_name: "OpenSim Next".to_string(),
            allow_teleports_to_any_region: true,
            foreign_agents_allowed: true,
        }
    }
}

pub struct HypergridManager {
    config: HypergridConfig,
    gatekeeper: Arc<dyn GatekeeperServiceTrait>,
    uas: Arc<dyn UserAgentServiceTrait>,
    gk_connector: GatekeeperServiceConnector,
    uas_connector: UserAgentServiceConnector,
    linked_regions: RwLock<HashMap<u64, HGRegionInfo>>,
}

impl HypergridManager {
    pub fn new(
        gatekeeper: Arc<dyn GatekeeperServiceTrait>,
        uas: Arc<dyn UserAgentServiceTrait>,
        config: HypergridConfig,
    ) -> Self {
        let gk_connector = GatekeeperServiceConnector::new(&config.gatekeeper_uri);
        let uas_connector = UserAgentServiceConnector::new(&config.home_uri);

        Self {
            config,
            gatekeeper,
            uas,
            gk_connector,
            uas_connector,
            linked_regions: RwLock::new(HashMap::new()),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn home_uri(&self) -> &str {
        &self.config.home_uri
    }

    pub fn gatekeeper_uri(&self) -> &str {
        &self.config.gatekeeper_uri
    }

    pub fn config(&self) -> &HypergridConfig {
        &self.config
    }

    pub fn external_uri(&self) -> &str {
        if self.config.external_uri.is_empty() {
            &self.config.home_uri
        } else {
            &self.config.external_uri
        }
    }

    pub fn external_robust_uri(&self) -> &str {
        if !self.config.external_robust_uri.is_empty() {
            &self.config.external_robust_uri
        } else {
            self.external_uri()
        }
    }

    pub fn gatekeeper(&self) -> &Arc<dyn GatekeeperServiceTrait> {
        &self.gatekeeper
    }

    pub fn uas(&self) -> &Arc<dyn UserAgentServiceTrait> {
        &self.uas
    }

    pub async fn get_linked_region(&self, handle: u64) -> Option<HGRegionInfo> {
        self.linked_regions.read().await.get(&handle).cloned()
    }

    pub async fn store_linked_region(&self, info: HGRegionInfo) {
        let handle = info.region_handle;
        self.linked_regions.write().await.insert(handle, info);
    }

    pub async fn link_remote_region(
        &self,
        gatekeeper_url: &str,
        region_name: &str,
    ) -> Result<Option<HGRegionInfo>> {
        info!("Linking remote region: {}:{}", gatekeeper_url, region_name);

        let connector = GatekeeperServiceConnector::new(gatekeeper_url);
        let result = connector.link_region(region_name).await?;

        if let Some(ref info) = result {
            self.store_linked_region(info.clone()).await;
            info!("Linked remote region '{}' (handle={})", info.region_name, info.region_handle);
        }

        Ok(result)
    }

    pub async fn initiate_hg_teleport(
        &self,
        agent_id: Uuid,
        session_id: Uuid,
        secure_session_id: Uuid,
        circuit_code: u32,
        first_name: &str,
        last_name: &str,
        dest_url: &str,
        client_ip: &str,
        source_region_name: &str,
        source_region_id: Uuid,
        source_region_loc_x: u32,
        source_region_loc_y: u32,
    ) -> Result<Option<(HGRegionInfo, String)>> {
        let (gatekeeper_url, _port, region_name) = match uui::parse_hg_url(dest_url) {
            Some(parsed) => parsed,
            None => return Err(anyhow::anyhow!("Invalid HG destination: {}", dest_url)),
        };

        info!("Initiating HG teleport for {} {} to {}:{}", first_name, last_name, gatekeeper_url, region_name);

        let remote_gk = GatekeeperServiceConnector::new(&gatekeeper_url);
        let destination = match remote_gk.link_region(&region_name).await? {
            Some(info) => info,
            None => {
                warn!("Remote region '{}' not found on {}", region_name, gatekeeper_url);
                return Ok(None);
            }
        };

        info!("[HG] link_region returned: name='{}', id={}, handle={}, server_uri='{}', loc=({},{}), size=({},{})",
            destination.region_name, destination.region_id, destination.region_handle,
            destination.server_uri, destination.region_loc_x, destination.region_loc_y,
            destination.size_x, destination.size_y);

        self.store_linked_region(destination.clone()).await;

        let caps_path = Uuid::new_v4().to_string();
        let agent = AgentCircuitData {
            agent_id,
            session_id,
            secure_session_id,
            circuit_code,
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            service_urls: self.get_agent_service_urls(),
            service_session_id: format!("{};{}", destination.server_uri, Uuid::new_v4()),
            start_pos: [128.0, 128.0, 21.0],
            appearance_serial: 0,
            client_ip: client_ip.to_string(),
            mac: format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                rand::random::<u8>() | 0x02, rand::random::<u8>(), rand::random::<u8>(),
                rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>()),
            id0: format!("{:032x}", rand::random::<u128>()),
            teleport_flags: 0x100000,
            caps_path: caps_path.clone(),
        };

        let source = HGRegionInfo {
            server_uri: self.external_uri().to_string(),
            region_name: source_region_name.to_string(),
            region_id: source_region_id,
            region_loc_x: source_region_loc_x,
            region_loc_y: source_region_loc_y,
            ..Default::default()
        };

        let (uas_ok, uas_reason) = self.uas
            .login_agent_to_grid(&agent, &source, &destination, false)
            .await?;

        if !uas_ok {
            warn!("UAS login_agent_to_grid failed: {}", uas_reason);
            return Ok(None);
        }

        let full_dest = match remote_gk.get_hyperlinkregion(
            destination.region_id,
            agent_id,
            &self.config.home_uri,
        ).await {
            Ok(Some(info)) => {
                info!("[HG] get_region returned: name='{}', hostname='{}', internal_port={}, server_uri='{}', handle={}",
                    info.region_name, info.hostname, info.internal_port, info.server_uri, info.region_handle);
                info
            }
            Ok(None) => {
                warn!("[HG] get_region returned None — using link_region data as fallback");
                destination.clone()
            }
            Err(e) => {
                warn!("[HG] get_region failed: {} — using link_region data as fallback", e);
                destination.clone()
            }
        };

        {
            let region_uri = if !full_dest.server_uri.is_empty() {
                &full_dest.server_uri
            } else {
                &destination.server_uri
            };
            let close_url = format!("{}agent/{}/{}/",
                region_uri.trim_end_matches('/').to_owned() + "/",
                agent_id, full_dest.region_id);
            info!("[HG] Pre-teleport cleanup: DELETE {} (clear stale agent)", close_url);
            match reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default()
                .delete(&close_url)
                .send().await
            {
                Ok(resp) => info!("[HG] Pre-teleport cleanup response: HTTP {}", resp.status()),
                Err(e) => info!("[HG] Pre-teleport cleanup failed (ok, may not exist): {}", e),
            }
        }

        let (gk_ok, gk_reason) = remote_gk
            .login_agent(&source, &agent, &destination)
            .await?;

        if !gk_ok {
            warn!("Remote gatekeeper login_agent failed: {}", gk_reason);
            return Ok(None);
        }

        info!("HG teleport foreignagent authorized — proceeding with teleport");

        self.store_linked_region(full_dest.clone()).await;

        info!("HG teleport initiated successfully to {} on {} (caps_path={})", region_name, gatekeeper_url, caps_path);

        Ok(Some((full_dest, caps_path)))
    }

    pub async fn handle_hg_map_search(
        &self,
        query: &str,
    ) -> Result<Option<HGRegionInfo>> {
        if !uui::is_hg_destination(query) {
            return Ok(None);
        }

        let (gatekeeper_url, _port, region_name) = match uui::parse_hg_url(query) {
            Some(parsed) => parsed,
            None => return Ok(None),
        };

        self.link_remote_region(&gatekeeper_url, &region_name).await
    }

    pub fn get_service_urls(&self) -> HashMap<String, String> {
        let ext = self.external_uri().to_string();
        let robust = self.external_robust_uri().to_string();
        let mut urls = HashMap::new();
        urls.insert("SRV_HomeURI".to_string(), ext.clone());
        urls.insert("SRV_GatekeeperURI".to_string(), ext);
        urls.insert("SRV_AssetServerURI".to_string(), robust.clone());
        urls.insert("SRV_InventoryServerURI".to_string(), format!("{}/hg", robust));
        urls.insert("SRV_ProfileServerURI".to_string(), robust.clone());
        urls.insert("SRV_FriendsServerURI".to_string(), robust.clone());
        urls.insert("SRV_IMServerURI".to_string(), robust);
        urls
    }

    pub fn get_agent_service_urls(&self) -> HashMap<String, String> {
        let ext = self.external_uri().to_string();
        let robust = self.external_robust_uri().to_string();
        let mut urls = HashMap::new();
        urls.insert("HomeURI".to_string(), ext.clone());
        urls.insert("GatekeeperURI".to_string(), ext);
        urls.insert("AssetServerURI".to_string(), robust.clone());
        urls.insert("InventoryServerURI".to_string(), format!("{}/hg", robust));
        urls.insert("ProfileServerURI".to_string(), robust.clone());
        urls.insert("FriendsServerURI".to_string(), robust.clone());
        urls.insert("IMServerURI".to_string(), robust);
        urls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypergrid_config_default() {
        let config = HypergridConfig::default();
        assert!(!config.enabled);
        assert!(config.foreign_agents_allowed);
        assert!(config.allow_teleports_to_any_region);
    }

    #[test]
    fn test_service_urls() {
        let config = HypergridConfig {
            enabled: true,
            home_uri: "http://mygrid.com:8002".to_string(),
            gatekeeper_uri: "http://mygrid.com:8002".to_string(),
            ..Default::default()
        };

        struct MockGK;
        struct MockUAS;

        #[async_trait::async_trait]
        impl GatekeeperServiceTrait for MockGK {
            async fn link_region(&self, _: &str) -> Result<Option<HGRegionInfo>> { Ok(None) }
            async fn get_hyperlinkregion(&self, _: Uuid, _: Uuid, _: &str) -> Result<Option<HGRegionInfo>> { Ok(None) }
            async fn login_agent(&self, _: &HGRegionInfo, _: &AgentCircuitData, _: &HGRegionInfo) -> Result<(bool, String)> { Ok((false, String::new())) }
        }

        #[async_trait::async_trait]
        impl UserAgentServiceTrait for MockUAS {
            async fn verify_agent(&self, _: Uuid, _: &str) -> Result<bool> { Ok(false) }
            async fn verify_client(&self, _: Uuid, _: &str) -> Result<bool> { Ok(false) }
            async fn get_home_region(&self, _: Uuid) -> Result<Option<(HGRegionInfo, [f32; 3], [f32; 3])>> { Ok(None) }
            async fn get_server_urls(&self, _: Uuid) -> Result<HashMap<String, String>> { Ok(HashMap::new()) }
            async fn logout_agent(&self, _: Uuid, _: Uuid) -> Result<()> { Ok(()) }
            async fn get_uui(&self, _: Uuid, _: Uuid) -> Result<String> { Ok(String::new()) }
            async fn get_uuid(&self, _: &str, _: &str) -> Result<Option<Uuid>> { Ok(None) }
            async fn status_notification(&self, _: &[String], _: Uuid, _: bool) -> Result<Vec<Uuid>> { Ok(Vec::new()) }
            async fn is_agent_coming_home(&self, _: Uuid, _: &str) -> Result<bool> { Ok(false) }
            async fn login_agent_to_grid(&self, _: &AgentCircuitData, _: &HGRegionInfo, _: &HGRegionInfo, _: bool) -> Result<(bool, String)> { Ok((false, String::new())) }
            async fn get_user_info(&self, _: Uuid) -> Result<HashMap<String, String>> { Ok(HashMap::new()) }
        }

        let mgr = HypergridManager::new(
            Arc::new(MockGK),
            Arc::new(MockUAS),
            config,
        );

        let urls = mgr.get_service_urls();
        assert_eq!(urls.get("SRV_HomeURI").unwrap(), "http://mygrid.com:8002");
        assert_eq!(urls.len(), 7);
    }
}
