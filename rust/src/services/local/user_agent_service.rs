use anyhow::Result;
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::services::traits::{
    AgentCircuitData, GridServiceTrait, HGRegionInfo, PresenceServiceTrait,
    UserAccountServiceTrait, UserAgentServiceTrait,
};

pub struct LocalUserAgentService {
    grid_service: Arc<dyn GridServiceTrait>,
    user_service: Arc<dyn UserAccountServiceTrait>,
    presence_service: Arc<dyn PresenceServiceTrait>,
    db: Arc<DatabaseConnection>,
    home_uri: String,
    external_uri: String,
    external_robust_uri: String,
    grid_name: String,
}

impl LocalUserAgentService {
    pub fn new(
        grid_service: Arc<dyn GridServiceTrait>,
        user_service: Arc<dyn UserAccountServiceTrait>,
        presence_service: Arc<dyn PresenceServiceTrait>,
        db: Arc<DatabaseConnection>,
        home_uri: String,
        external_uri: String,
        external_robust_uri: String,
        grid_name: String,
    ) -> Self {
        Self {
            grid_service,
            user_service,
            presence_service,
            db,
            home_uri,
            external_uri,
            external_robust_uri,
            grid_name,
        }
    }

    async fn get_traveling_data(
        &self,
        session_id: Uuid,
    ) -> Result<Option<(String, String, String)>> {
        match self.db.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => {
                let row = sqlx::query(
                    "SELECT GridExternalName, ServiceToken, ClientIPAddress FROM hg_traveling_data WHERE SessionID = $1"
                )
                .bind(session_id.to_string())
                .fetch_optional(pool)
                .await?;

                Ok(row.map(|r| {
                    let grid: String = r.get("gridexternalname");
                    let token: String = r.get("servicetoken");
                    let ip: String = r.get("clientipaddress");
                    (grid, token, ip)
                }))
            }
            DatabaseConnection::MySQL(pool) => {
                let row = sqlx::query(
                    "SELECT GridExternalName, ServiceToken, ClientIPAddress FROM hg_traveling_data WHERE SessionID = ?"
                )
                .bind(session_id.to_string())
                .fetch_optional(pool)
                .await?;

                Ok(row.map(|r| {
                    let grid: String = r.get("GridExternalName");
                    let token: String = r.get("ServiceToken");
                    let ip: String = r.get("ClientIPAddress");
                    (grid, token, ip)
                }))
            }
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

    async fn remove_traveling_data(&self, session_id: Uuid) -> Result<()> {
        match self.db.as_ref() {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query("DELETE FROM hg_traveling_data WHERE SessionID = $1")
                    .bind(session_id.to_string())
                    .execute(pool)
                    .await?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query("DELETE FROM hg_traveling_data WHERE SessionID = ?")
                    .bind(session_id.to_string())
                    .execute(pool)
                    .await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl UserAgentServiceTrait for LocalUserAgentService {
    async fn verify_agent(&self, session_id: Uuid, token: &str) -> Result<bool> {
        debug!("verify_agent: session={}", session_id);
        let data = self.get_traveling_data(session_id).await?;
        match data {
            Some((_, stored_token, _)) => {
                let valid = stored_token == token;
                if !valid {
                    warn!("verify_agent: token mismatch for session {}", session_id);
                }
                Ok(valid)
            }
            None => {
                warn!("verify_agent: no traveling data for session {}", session_id);
                Ok(false)
            }
        }
    }

    async fn verify_client(&self, session_id: Uuid, reported_ip: &str) -> Result<bool> {
        info!(
            "verify_client: session={}, reported_ip='{}'",
            session_id, reported_ip
        );
        let data = self.get_traveling_data(session_id).await?;
        match data {
            Some((grid_name, _token, stored_ip)) => {
                info!(
                    "verify_client: found traveling data — grid='{}', stored_ip='{}'",
                    grid_name, stored_ip
                );
                let mut valid = stored_ip == reported_ip || stored_ip.is_empty();
                if !valid {
                    let uris_to_check: Vec<&str> = if !self.external_uri.is_empty() {
                        vec![&self.external_uri, &self.home_uri]
                    } else {
                        vec![&self.home_uri]
                    };
                    for uri in &uris_to_check {
                        if valid {
                            break;
                        }
                        if let Ok(url) = url::Url::parse(uri) {
                            if let Some(host) = url.host_str() {
                                if let Ok(addrs) = tokio::net::lookup_host(format!(
                                    "{}:{}",
                                    host,
                                    url.port().unwrap_or(9000)
                                ))
                                .await
                                {
                                    for addr in addrs {
                                        if addr.ip().to_string() == reported_ip {
                                            valid = true;
                                            info!("verify_client: NAT match — reported IP {} matches our hostname {} (from {})", reported_ip, host, uri);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                info!(
                    "verify_client: result={} (stored_ip='{}', reported_ip='{}')",
                    valid, stored_ip, reported_ip
                );
                if !valid {
                    warn!(
                        "verify_client: IP mismatch for session {} (stored={}, reported={})",
                        session_id, stored_ip, reported_ip
                    );
                }
                Ok(valid)
            }
            None => {
                info!(
                    "verify_client: no traveling data for session {} — returning true",
                    session_id
                );
                Ok(true)
            }
        }
    }

    async fn get_home_region(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(HGRegionInfo, [f32; 3], [f32; 3])>> {
        debug!("get_home_region for user {}", user_id);

        let ext = if !self.external_uri.is_empty() {
            &self.external_uri
        } else {
            &self.home_uri
        };

        let defaults = self.grid_service.get_default_regions(Uuid::nil()).await?;
        if let Some(region) = defaults.first() {
            let ext_host = ext
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .split(':')
                .next()
                .unwrap_or("localhost");
            let ext_port = ext
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .split(':')
                .nth(1)
                .and_then(|p| p.trim_end_matches('/').parse::<u16>().ok())
                .unwrap_or(region.server_port);

            let hg_info = HGRegionInfo {
                region_id: region.region_id,
                region_handle: ((region.region_loc_x as u64) << 32) | (region.region_loc_y as u64),
                external_name: format!("{}:{}", ext, region.region_name),
                image_url: String::new(),
                size_x: region.region_size_x,
                size_y: region.region_size_y,
                http_port: ext_port,
                server_uri: format!("http://{}:{}", ext_host, ext_port),
                region_name: region.region_name.clone(),
                region_loc_x: region.region_loc_x,
                region_loc_y: region.region_loc_y,
                hostname: ext_host.to_string(),
                internal_port: region.server_port,
            };
            info!(
                "get_home_region for {}: ext={}, server_uri={}, region={}",
                user_id, ext, hg_info.server_uri, hg_info.region_name
            );
            let position = [128.0f32, 128.0, 21.0];
            let look_at = [0.0f32, 1.0, 0.0];
            return Ok(Some((hg_info, position, look_at)));
        }

        Ok(None)
    }

    async fn get_server_urls(&self, user_id: Uuid) -> Result<HashMap<String, String>> {
        let ext = if !self.external_uri.is_empty() {
            &self.external_uri
        } else {
            &self.home_uri
        };
        let robust = if !self.external_robust_uri.is_empty() {
            &self.external_robust_uri
        } else {
            ext
        };
        info!(
            "get_server_urls for user {}: home={}, robust={}",
            user_id, ext, robust
        );
        let mut urls = HashMap::new();
        urls.insert("SRV_HomeURI".to_string(), ext.to_string());
        urls.insert("SRV_GatekeeperURI".to_string(), ext.to_string());
        urls.insert("SRV_AssetServerURI".to_string(), robust.to_string());
        urls.insert(
            "SRV_InventoryServerURI".to_string(),
            format!("{}/hg", robust),
        );
        urls.insert("SRV_ProfileServerURI".to_string(), robust.to_string());
        urls.insert("SRV_FriendsServerURI".to_string(), robust.to_string());
        urls.insert("SRV_IMServerURI".to_string(), robust.to_string());
        Ok(urls)
    }

    async fn logout_agent(&self, user_id: Uuid, session_id: Uuid) -> Result<()> {
        info!("logout_agent: user={}, session={}", user_id, session_id);
        self.remove_traveling_data(session_id).await?;
        let _ = self.presence_service.logout_agent(session_id).await;
        Ok(())
    }

    async fn get_uui(&self, _user_id: Uuid, target_user_id: Uuid) -> Result<String> {
        let account = self
            .user_service
            .get_user_account(Uuid::nil(), target_user_id)
            .await?;

        match account {
            Some(acct) => Ok(crate::services::hypergrid::uui::format_uui(
                target_user_id,
                &self.home_uri,
                &acct.first_name,
                &acct.last_name,
            )),
            None => Ok(target_user_id.to_string()),
        }
    }

    async fn get_uuid(&self, first: &str, last: &str) -> Result<Option<Uuid>> {
        let account = self
            .user_service
            .get_user_account_by_name(Uuid::nil(), first, last)
            .await?;
        Ok(account.map(|a| a.principal_id))
    }

    async fn status_notification(
        &self,
        _friends: &[String],
        _user_id: Uuid,
        _online: bool,
    ) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn is_agent_coming_home(&self, session_id: Uuid, external_name: &str) -> Result<bool> {
        debug!(
            "is_agent_coming_home: session={}, external={}",
            session_id, external_name
        );
        let normalized_home = self.home_uri.trim_end_matches('/').to_lowercase();
        let normalized_ext = external_name.trim_end_matches('/').to_lowercase();
        Ok(normalized_home == normalized_ext)
    }

    async fn login_agent_to_grid(
        &self,
        agent: &AgentCircuitData,
        _gatekeeper: &HGRegionInfo,
        destination: &HGRegionInfo,
        _from_login: bool,
    ) -> Result<(bool, String)> {
        info!(
            "login_agent_to_grid: {} {} -> {}",
            agent.first_name, agent.last_name, destination.server_uri
        );

        // CRITICAL: Use agent.service_session_id as the service token.
        // The remote grid's gatekeeper will callback verify_agent(session_id, service_session_id)
        // and we must find this exact token in hg_traveling_data.
        let service_token = if agent.service_session_id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            agent.service_session_id.clone()
        };

        self.store_traveling_data(
            agent.session_id,
            agent.agent_id,
            &destination.server_uri,
            &service_token,
            &agent.client_ip,
        )
        .await?;

        Ok((true, "success".to_string()))
    }

    async fn get_user_info(&self, user_id: Uuid) -> Result<HashMap<String, String>> {
        info!("[HG UAS] get_user_info for {}", user_id);
        let pool = match self.db.as_ref() {
            crate::database::DatabaseConnection::PostgreSQL(pool) => pool,
            _ => return Err(anyhow::anyhow!("Requires PostgreSQL")),
        };

        let row =
            sqlx::query("SELECT firstname, lastname FROM useraccounts WHERE principalid = $1")
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

        let mut info = HashMap::new();
        if let Some(row) = row {
            use sqlx::Row;
            let first: String = row.try_get("firstname")?;
            let last: String = row.try_get("lastname")?;
            info.insert("user_firstname".to_string(), first);
            info.insert("user_lastname".to_string(), last);
            info.insert("user_flags".to_string(), "0".to_string());
            info.insert("result".to_string(), "success".to_string());
        } else {
            info.insert("result".to_string(), "failure".to_string());
        }
        Ok(info)
    }
}
