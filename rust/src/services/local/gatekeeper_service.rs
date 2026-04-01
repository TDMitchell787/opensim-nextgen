use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, warn, debug};

use crate::services::traits::{
    GatekeeperServiceTrait, HGRegionInfo, AgentCircuitData,
    GridServiceTrait, PresenceServiceTrait, UserAgentServiceTrait,
    UserAccountServiceTrait,
};
use crate::services::remote::uas_connector::UserAgentServiceConnector;
use crate::database::DatabaseConnection;

pub struct LocalGatekeeperService {
    grid_service: Arc<dyn GridServiceTrait>,
    presence_service: Arc<dyn PresenceServiceTrait>,
    user_account_service: Option<Arc<dyn UserAccountServiceTrait>>,
    local_uas: Option<Arc<dyn UserAgentServiceTrait>>,
    db: Arc<DatabaseConnection>,
    external_name: String,
    allow_teleports_to_any_region: bool,
    foreign_agents_allowed: bool,
}

impl LocalGatekeeperService {
    pub fn new(
        grid_service: Arc<dyn GridServiceTrait>,
        presence_service: Arc<dyn PresenceServiceTrait>,
        db: Arc<DatabaseConnection>,
        external_name: String,
    ) -> Self {
        Self {
            grid_service,
            presence_service,
            user_account_service: None,
            local_uas: None,
            db,
            external_name,
            allow_teleports_to_any_region: true,
            foreign_agents_allowed: true,
        }
    }

    pub fn with_config(mut self, allow_teleports: bool, foreign_allowed: bool) -> Self {
        self.allow_teleports_to_any_region = allow_teleports;
        self.foreign_agents_allowed = foreign_allowed;
        self
    }

    pub fn with_user_account_service(mut self, svc: Arc<dyn UserAccountServiceTrait>) -> Self {
        self.user_account_service = Some(svc);
        self
    }

    pub fn with_local_uas(mut self, uas: Arc<dyn UserAgentServiceTrait>) -> Self {
        self.local_uas = Some(uas);
        self
    }

    fn is_local_home(&self, home_uri: &str) -> bool {
        let norm_ext = self.external_name.trim_end_matches('/').to_lowercase();
        let norm_home = home_uri.trim_end_matches('/').to_lowercase();
        norm_ext == norm_home
    }

    async fn authenticate(&self, agent_data: &AgentCircuitData) -> bool {
        if agent_data.client_ip.is_empty() {
            warn!("[GATEKEEPER] Agent did not provide a client IP address");
            return false;
        }

        let home_uri = match agent_data.service_urls.get("HomeURI") {
            Some(uri) if !uri.is_empty() => uri.clone(),
            _ => {
                warn!("[GATEKEEPER] Agent did not provide HomeURI in service_urls");
                return false;
            }
        };

        if agent_data.service_session_id.is_empty() {
            warn!("[GATEKEEPER] Agent has empty service_session_id");
            return false;
        }

        if self.is_local_home(&home_uri) {
            if let Some(ref uas) = self.local_uas {
                match uas.verify_agent(agent_data.session_id, &agent_data.service_session_id).await {
                    Ok(valid) => {
                        if !valid {
                            warn!("[GATEKEEPER] Local UAS rejected agent {} (token mismatch)", agent_data.agent_id);
                        }
                        return valid;
                    }
                    Err(e) => {
                        warn!("[GATEKEEPER] Local UAS verify_agent error: {}", e);
                        return false;
                    }
                }
            }
        }

        let remote_uas = UserAgentServiceConnector::new(&home_uri);
        match remote_uas.verify_agent(agent_data.session_id, &agent_data.service_session_id).await {
            Ok(valid) => {
                if !valid {
                    warn!("[GATEKEEPER] Remote UAS at {} rejected agent {}", home_uri, agent_data.agent_id);
                }
                valid
            }
            Err(e) => {
                warn!("[GATEKEEPER] Unable to contact UAS at {}: {}", home_uri, e);
                false
            }
        }
    }

    fn region_info_to_hg(&self, region: &crate::services::traits::RegionInfo) -> HGRegionInfo {
        let ext = self.external_name.trim_end_matches('/');
        let ext_host_port = ext
            .strip_prefix("http://").or_else(|| ext.strip_prefix("https://"))
            .unwrap_or(ext);

        let loc_x_meters = region.region_loc_x * 256;
        let loc_y_meters = region.region_loc_y * 256;

        HGRegionInfo {
            region_id: region.region_id,
            region_handle: ((loc_x_meters as u64) << 32) | (loc_y_meters as u64),
            external_name: format!("{}:{}", ext_host_port, region.region_name),
            image_url: String::new(),
            size_x: region.region_size_x,
            size_y: region.region_size_y,
            http_port: region.server_port,
            server_uri: format!("{}/", ext),
            region_name: region.region_name.clone(),
            region_loc_x: loc_x_meters,
            region_loc_y: loc_y_meters,
            hostname: ext_host_port.split(':').next().unwrap_or(&region.server_ip).to_string(),
            internal_port: region.server_port,
        }
    }

    async fn store_traveling_data(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        grid_external_name: &str,
        service_token: &str,
        client_ip: &str,
    ) -> Result<()> {
        match self.db.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query(
                    "INSERT INTO hg_traveling_data (SessionID, UserID, GridExternalName, ServiceToken, ClientIPAddress, MyIPAddress)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     ON CONFLICT (SessionID) DO UPDATE SET
                       UserID = EXCLUDED.UserID,
                       GridExternalName = EXCLUDED.GridExternalName,
                       ServiceToken = EXCLUDED.ServiceToken,
                       ClientIPAddress = EXCLUDED.ClientIPAddress"
                )
                .bind(session_id.to_string())
                .bind(user_id.to_string())
                .bind(grid_external_name)
                .bind(service_token)
                .bind(&client_ip[..std::cmp::min(client_ip.len(), 16)])
                .bind("")
                .execute(pool)
                .await?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query(
                    "INSERT INTO hg_traveling_data (SessionID, UserID, GridExternalName, ServiceToken, ClientIPAddress, MyIPAddress)
                     VALUES (?, ?, ?, ?, ?, ?)
                     ON DUPLICATE KEY UPDATE
                       UserID = VALUES(UserID),
                       GridExternalName = VALUES(GridExternalName),
                       ServiceToken = VALUES(ServiceToken),
                       ClientIPAddress = VALUES(ClientIPAddress)"
                )
                .bind(session_id.to_string())
                .bind(user_id.to_string())
                .bind(grid_external_name)
                .bind(service_token)
                .bind(&client_ip[..std::cmp::min(client_ip.len(), 16)])
                .bind("")
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl GatekeeperServiceTrait for LocalGatekeeperService {
    async fn link_region(&self, region_name: &str) -> Result<Option<HGRegionInfo>> {
        debug!("link_region request for: {}", region_name);

        let region = self.grid_service
            .get_region_by_name(Uuid::nil(), region_name)
            .await?;

        match region {
            Some(r) => {
                info!("Linked region '{}' ({})", r.region_name, r.region_id);
                Ok(Some(self.region_info_to_hg(&r)))
            }
            None => {
                // Try default region if empty name
                if region_name.is_empty() {
                    let defaults = self.grid_service.get_default_regions(Uuid::nil()).await?;
                    if let Some(r) = defaults.first() {
                        info!("Linked default region '{}' ({})", r.region_name, r.region_id);
                        return Ok(Some(self.region_info_to_hg(r)));
                    }
                }
                warn!("Region '{}' not found for link_region", region_name);
                Ok(None)
            }
        }
    }

    async fn get_hyperlinkregion(
        &self,
        region_id: Uuid,
        _agent_id: Uuid,
        _agent_home_uri: &str,
    ) -> Result<Option<HGRegionInfo>> {
        debug!("get_hyperlinkregion for region_id: {}", region_id);

        if !self.allow_teleports_to_any_region {
            let defaults = self.grid_service.get_default_regions(Uuid::nil()).await?;
            let is_default = defaults.iter().any(|r| r.region_id == region_id);
            if !is_default {
                warn!("Teleport to non-default region {} denied (AllowTeleportsToAnyRegion=false)", region_id);
                return Ok(None);
            }
        }

        let region = self.grid_service
            .get_region_by_uuid(Uuid::nil(), region_id)
            .await?;

        Ok(region.map(|r| self.region_info_to_hg(&r)))
    }

    async fn login_agent(
        &self,
        source: &HGRegionInfo,
        agent_data: &AgentCircuitData,
        destination: &HGRegionInfo,
    ) -> Result<(bool, String)> {
        let home_uri = agent_data.service_urls.get("HomeURI")
            .cloned()
            .unwrap_or_else(|| source.server_uri.clone());

        info!("[GATEKEEPER] Login request: {} {} @ {} ({}) -> {}",
            agent_data.first_name, agent_data.last_name,
            home_uri, agent_data.agent_id, destination.region_name);

        // Step 1: Authenticate — callback to the agent's home grid UAS
        if !self.authenticate(agent_data).await {
            return Ok((false, "Unable to verify identity".to_string()));
        }
        debug!("[GATEKEEPER] Identity verified for {} {} @ {}",
            agent_data.first_name, agent_data.last_name, home_uri);

        // Step 2: Check for UUID impersonation — if agent UUID matches a local user,
        // they must be that user coming home, not a foreign user with a spoofed UUID
        let mut is_local_user = false;
        if let Some(ref user_svc) = self.user_account_service {
            if let Ok(Some(_local_account)) = user_svc.get_user_account(Uuid::nil(), agent_data.agent_id).await {
                is_local_user = true;
                if let Some(ref uas) = self.local_uas {
                    match uas.is_agent_coming_home(agent_data.session_id, &self.external_name).await {
                        Ok(true) => {
                            debug!("[GATEKEEPER] Agent {} is a local user coming home", agent_data.agent_id);
                        }
                        _ => {
                            warn!("[GATEKEEPER] Foreign agent {} {} has same UUID as local user. Refusing.",
                                agent_data.first_name, agent_data.last_name);
                            return Ok((false, "Unauthorized".to_string()));
                        }
                    }
                }
            }
        }

        // Step 3: Foreign agents allowed?
        if !is_local_user && !self.foreign_agents_allowed {
            info!("[GATEKEEPER] Foreign agents not permitted: {} {} @ {}",
                agent_data.first_name, agent_data.last_name, home_uri);
            return Ok((false, "Destination does not allow visitors from your world".to_string()));
        }

        // Step 4: Store traveling data
        let service_token = Uuid::new_v4().to_string();
        self.store_traveling_data(
            agent_data.session_id,
            agent_data.agent_id,
            &home_uri,
            &service_token,
            &agent_data.client_ip,
        ).await?;

        // Step 5: Create presence entry for foreign agent
        if let Err(e) = self.presence_service.login_agent(
            agent_data.agent_id,
            agent_data.session_id,
            agent_data.secure_session_id,
            destination.region_id,
        ).await {
            warn!("[GATEKEEPER] Failed to create presence for foreign agent: {}", e);
        }

        info!("[GATEKEEPER] Foreign agent {} {} authenticated and logged in to {}",
            agent_data.first_name, agent_data.last_name, destination.region_name);

        Ok((true, "success".to_string()))
    }
}
