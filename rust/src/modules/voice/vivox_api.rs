use anyhow::{Result, anyhow};
use tracing::{info, warn};

pub struct ChannelConfig {
    pub channel_type: String,
    pub channel_mode: String,
    pub roll_off: f64,
    pub distance_model: i32,
    pub max_range: i32,
    pub clamping_distance: i32,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            channel_type: "positional".to_string(),
            channel_mode: "open".to_string(),
            roll_off: 2.0,
            distance_model: 2,
            max_range: 60,
            clamping_distance: 10,
        }
    }
}

pub struct VivoxApiClient {
    server: String,
    admin_user: String,
    admin_password: String,
    auth_token: tokio::sync::Mutex<String>,
    http_client: reqwest::Client,
    dump_xml: bool,
}

impl VivoxApiClient {
    pub fn new(server: &str, admin_user: &str, admin_password: &str, dump_xml: bool) -> Self {
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            server: server.to_string(),
            admin_user: admin_user.to_string(),
            admin_password: admin_password.to_string(),
            auth_token: tokio::sync::Mutex::new(String::new()),
            http_client,
            dump_xml,
        }
    }

    fn api_url(&self, path: &str) -> String {
        format!("https://{}/api2/{}", self.server, path)
    }

    async fn vivox_call(&self, url: &str) -> Result<String> {
        let _lock = self.auth_token.lock().await;
        drop(_lock);

        let resp = self.http_client.get(url)
            .send()
            .await
            .map_err(|e| anyhow!("Vivox API call failed: {}", e))?;
        let body = resp.text().await
            .map_err(|e| anyhow!("Failed to read Vivox response: {}", e))?;
        if self.dump_xml {
            info!("[VivoxAPI] Response: {}", body);
        }
        Ok(body)
    }

    fn extract_xml_value(xml: &str, path: &str) -> Option<String> {
        let tag = path.rsplit('.').next()?;
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        let start = xml.find(&open)? + open.len();
        let end = xml[start..].find(&close)?;
        Some(xml[start..start + end].to_string())
    }

    fn extract_status(xml: &str) -> String {
        Self::extract_xml_value(xml, "status")
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn extract_code(xml: &str) -> String {
        Self::extract_xml_value(xml, "code")
            .unwrap_or_else(|| "0".to_string())
    }

    pub async fn admin_login(&self) -> Result<()> {
        let url = format!(
            "https://{}/api2/viv_signin.php?userid={}&pwd={}",
            self.server, self.admin_user, self.admin_password
        );
        let body = self.vivox_call(&url).await?;
        let status = Self::extract_status(&body);
        if status.to_lowercase() != "ok" {
            return Err(anyhow!("Vivox admin login failed: status={}", status));
        }
        let token = Self::extract_xml_value(&body, "auth_token")
            .ok_or_else(|| anyhow!("No auth_token in login response"))?;
        let mut auth = self.auth_token.lock().await;
        *auth = token.clone();
        info!("[VivoxAPI] Admin login successful");
        Ok(())
    }

    pub async fn admin_logout(&self) -> Result<()> {
        let auth = self.auth_token.lock().await;
        if auth.is_empty() {
            return Ok(());
        }
        let url = format!(
            "https://{}/api2/viv_signout.php?auth_token={}",
            self.server, *auth
        );
        drop(auth);
        let _ = self.vivox_call(&url).await;
        let mut auth = self.auth_token.lock().await;
        *auth = String::new();
        info!("[VivoxAPI] Admin logout");
        Ok(())
    }

    pub async fn get_account_info(&self, user_name: &str) -> Result<(String, String)> {
        let auth = self.auth_token.lock().await;
        let url = format!(
            "{}?auth_token={}&user_name={}",
            self.api_url("viv_get_acct.php"),
            *auth,
            user_name
        );
        drop(auth);
        let body = self.vivox_call(&url).await?;
        let status = Self::extract_status(&body);
        let code = Self::extract_code(&body);
        Ok((status, code))
    }

    pub async fn create_account(&self, user_name: &str, password: &str) -> Result<(String, String)> {
        let auth = self.auth_token.lock().await;
        let url = format!(
            "{}?username={}&pwd={}&auth_token={}",
            self.api_url("viv_adm_acct_new.php"),
            user_name,
            password,
            *auth
        );
        drop(auth);
        let body = self.vivox_call(&url).await?;
        let status = Self::extract_status(&body);
        let code = Self::extract_code(&body);
        Ok((status, code))
    }

    pub async fn change_password(&self, user_name: &str, new_password: &str) -> Result<()> {
        let auth = self.auth_token.lock().await;
        let url = format!(
            "{}?user_name={}&new_pwd={}&auth_token={}",
            self.api_url("viv_adm_password.php"),
            user_name,
            new_password,
            *auth
        );
        drop(auth);
        let body = self.vivox_call(&url).await?;
        let status = Self::extract_status(&body);
        if status.to_lowercase() != "ok" {
            warn!("[VivoxAPI] Password change failed for {}: {}", user_name, status);
        }
        Ok(())
    }

    pub async fn create_channel(
        &self,
        chan_name: &str,
        description: &str,
        parent_id: &str,
        config: &ChannelConfig,
    ) -> Result<Option<String>> {
        let auth = self.auth_token.lock().await;
        let mut url = format!(
            "{}?mode=create&chan_name={}&auth_token={}",
            self.api_url("viv_chan_mod.php"),
            chan_name,
            *auth
        );
        drop(auth);
        if !parent_id.is_empty() {
            url.push_str(&format!("&chan_parent={}", parent_id));
        }
        if !description.is_empty() {
            url.push_str(&format!("&chan_desc={}", description));
        }
        url.push_str(&format!(
            "&chan_type={}&chan_mode={}&chan_roll_off={}&chan_dist_model={}&chan_max_range={}&chan_clamping_distance={}",
            config.channel_type,
            config.channel_mode,
            config.roll_off,
            config.distance_model,
            config.max_range,
            config.clamping_distance
        ));
        let body = self.vivox_call(&url).await?;
        let channel_uri = Self::extract_xml_value(&body, "chan_uri");
        Ok(channel_uri)
    }

    pub async fn search_channel(&self, channel_name: &str) -> Result<Option<(String, String)>> {
        let auth = self.auth_token.lock().await;
        let url = format!(
            "{}?cond_channame={}&auth_token={}",
            self.api_url("viv_chan_search.php"),
            channel_name,
            *auth
        );
        drop(auth);
        let body = self.vivox_call(&url).await?;
        let uri = Self::extract_xml_value(&body, "uri");
        let id = Self::extract_xml_value(&body, "id");
        match (id, uri) {
            (Some(id), Some(uri)) if !uri.is_empty() => Ok(Some((id, uri))),
            _ => Ok(None),
        }
    }

    pub async fn delete_channel(&self, chan_id: &str) -> Result<()> {
        let auth = self.auth_token.lock().await;
        let url = format!(
            "{}?mode=delete&chan_id={}&auth_token={}",
            self.api_url("viv_chan_mod.php"),
            chan_id,
            *auth
        );
        drop(auth);
        let _ = self.vivox_call(&url).await;
        Ok(())
    }
}
