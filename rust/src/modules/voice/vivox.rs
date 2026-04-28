use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

use super::common::*;
use super::traits::VoiceHandler;
use super::vivox_api::{ChannelConfig, VivoxApiClient};
use crate::modules::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};

pub struct VivoxVoiceModule {
    enabled: bool,
    api_client: Option<Arc<VivoxApiClient>>,
    sip_uri: String,
    voice_account_api: String,
    channel_config: ChannelConfig,
    channel_cache: Mutex<HashMap<String, String>>,
    region_uuid: Uuid,
    region_name: String,
}

impl VivoxVoiceModule {
    pub fn new() -> Self {
        Self {
            enabled: false,
            api_client: None,
            sip_uri: String::new(),
            voice_account_api: String::new(),
            channel_config: ChannelConfig::default(),
            channel_cache: Mutex::new(HashMap::new()),
            region_uuid: Uuid::nil(),
            region_name: String::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_or_create_channel(&self, channel_name: &str, channel_desc: &str) -> Result<String> {
        {
            let cache = self.channel_cache.lock();
            if let Some(uri) = cache.get(channel_name) {
                return Ok(uri.clone());
            }
        }

        let api = self
            .api_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vivox API client not initialized"))?;
        let api = api.clone();
        let name = channel_name.to_string();
        let desc = channel_desc.to_string();
        let config = &self.channel_config;

        let rt = tokio::runtime::Handle::try_current();
        let uri = if let Ok(handle) = rt {
            let result = std::thread::scope(|_| {
                handle.block_on(async {
                    if let Ok(Some((_id, uri))) = api.search_channel(&name).await {
                        return Ok(uri);
                    }
                    match api.create_channel(&name, &desc, "", config).await {
                        Ok(Some(uri)) => Ok(uri),
                        Ok(None) => Err(anyhow::anyhow!("Channel creation returned no URI")),
                        Err(e) => Err(e),
                    }
                })
            });
            result?
        } else {
            return Err(anyhow::anyhow!("No tokio runtime available"));
        };

        let mut cache = self.channel_cache.lock();
        cache.insert(channel_name.to_string(), uri.clone());
        Ok(uri)
    }
}

impl VoiceHandler for VivoxVoiceModule {
    fn provision_voice_account(&self, agent_id: Uuid) -> Result<String> {
        if !self.enabled {
            return Ok(build_undef_response());
        }
        let api = match &self.api_client {
            Some(api) => api.clone(),
            None => return Ok(build_undef_response()),
        };

        let agentname = encode_agent_name(agent_id);
        let password = generate_vivox_password();

        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow::anyhow!("No tokio runtime"))?;

        let result = std::thread::scope(|_| {
            rt.block_on(async {
                let (_status, code) = api.get_account_info(&agentname).await?;

                match code.as_str() {
                    "403" => {
                        let (_, create_code) = api.create_account(&agentname, &password).await?;
                        match create_code.as_str() {
                            "201" => {
                                api.admin_login().await?;
                                let (_, retry_code) =
                                    api.create_account(&agentname, &password).await?;
                                if retry_code != "200" && retry_code.to_lowercase() != "ok" {
                                    warn!("[Vivox] Account creation retry failed: {}", retry_code);
                                }
                            }
                            c if c != "200" && c.to_lowercase() != "ok" => {
                                warn!("[Vivox] Account creation failed: code={}", c);
                            }
                            _ => {}
                        }
                    }
                    "201" => {
                        api.admin_login().await?;
                    }
                    _ => {}
                }

                let _ = api.change_password(&agentname, &password).await;

                Ok::<String, anyhow::Error>(build_provision_response(
                    &agentname,
                    &password,
                    &self.sip_uri,
                    &self.voice_account_api,
                ))
            })
        });

        result
    }

    fn parcel_voice_info(
        &self,
        _agent_id: Uuid,
        parcel_flags: u32,
        parcel_uuid: Uuid,
        parcel_local_id: i32,
        parcel_name: &str,
        region_name: &str,
        region_uuid: Uuid,
        estate_allow_voice: bool,
    ) -> Result<String> {
        if !self.enabled {
            return Ok(build_undef_response());
        }

        if !estate_allow_voice {
            return Ok(build_parcel_voice_response(
                parcel_local_id,
                region_name,
                "",
            ));
        }
        if (parcel_flags & ALLOW_VOICE_CHAT) == 0 {
            return Ok(build_parcel_voice_response(
                parcel_local_id,
                region_name,
                "",
            ));
        }

        let (channel_name, channel_desc) = if is_estate_channel(parcel_flags) {
            (
                region_uuid.to_string(),
                format!("{}:{}", region_name, region_name),
            )
        } else {
            (
                parcel_uuid.to_string(),
                format!("{}:{}", region_name, parcel_name),
            )
        };

        let channel_uri = match self.get_or_create_channel(&channel_name, &channel_desc) {
            Ok(uri) => uri,
            Err(e) => {
                warn!("[Vivox] Failed to get/create channel: {}", e);
                String::new()
            }
        };

        Ok(build_parcel_voice_response(
            parcel_local_id,
            region_name,
            &channel_uri,
        ))
    }

    fn voice_enabled(&self) -> bool {
        self.enabled
    }
}

#[async_trait]
impl RegionModule for VivoxVoiceModule {
    fn name(&self) -> &'static str {
        "VivoxVoiceModule"
    }

    async fn initialize(&mut self, config: &ModuleConfig) -> Result<()> {
        self.enabled = config.get_bool("enabled", false);
        if !self.enabled {
            return Ok(());
        }

        let server = config.get_or("vivox_server", "");
        if server.is_empty() {
            warn!("[Vivox] vivox_server not configured, disabling");
            self.enabled = false;
            return Ok(());
        }

        self.sip_uri = config.get_or("vivox_sip_uri", &server);
        self.voice_account_api = format!("http://{}/api2", server);

        let admin_user = config.get_or("vivox_admin_user", "");
        let admin_password = config.get_or("vivox_admin_password", "");
        let dump_xml = config.get_bool("dump_xml", false);

        self.channel_config = ChannelConfig {
            channel_type: config.get_or("vivox_channel_type", "positional"),
            channel_mode: config.get_or("vivox_channel_mode", "open"),
            roll_off: config
                .get_or("vivox_channel_roll_off", "2.0")
                .parse()
                .unwrap_or(2.0),
            distance_model: config
                .get_or("vivox_channel_distance_model", "2")
                .parse()
                .unwrap_or(2),
            max_range: config
                .get_or("vivox_channel_max_range", "60")
                .parse()
                .unwrap_or(60),
            clamping_distance: config
                .get_or("vivox_channel_clamping_distance", "10")
                .parse()
                .unwrap_or(10),
        };

        let api = VivoxApiClient::new(&server, &admin_user, &admin_password, dump_xml);
        if let Err(e) = api.admin_login().await {
            warn!(
                "[Vivox] Admin login failed: {} - module stays enabled, will retry",
                e
            );
        }
        self.api_client = Some(Arc::new(api));

        info!("[Vivox] VivoxVoiceModule initialized (server={})", server);
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = scene.region_uuid;
        self.region_name = scene.region_name.clone();
        Ok(())
    }

    async fn remove_region(&mut self, _scene: &SceneContext) -> Result<()> {
        self.channel_cache.lock().clear();
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(api) = &self.api_client {
            let _ = api.admin_logout().await;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[async_trait]
impl SharedRegionModule for VivoxVoiceModule {}
