use std::any::Any;
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use parking_lot::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

use super::common::*;
use super::traits::{IVoiceModule, VoiceHandler};
use crate::modules::traits::{ModuleConfig, RegionModule, SceneContext, SharedRegionModule};

pub struct FreeSwitchVoiceModule {
    enabled: bool,
    realm: String,
    sip_proxy: String,
    api_prefix: String,
    echo_server: String,
    echo_port: i32,
    default_timeout: i32,
    attempt_stun: bool,
    well_known_ip: String,
    account_url: String,
    uuid_name_map: Mutex<HashMap<String, String>>,
    parcel_address_map: Mutex<HashMap<String, String>>,
    region_uuid: Uuid,
    region_name: String,
}

impl FreeSwitchVoiceModule {
    pub fn new() -> Self {
        Self {
            enabled: false,
            realm: String::new(),
            sip_proxy: String::new(),
            api_prefix: "/fsapi".to_string(),
            echo_server: String::new(),
            echo_port: 50505,
            default_timeout: 5000,
            attempt_stun: false,
            well_known_ip: String::new(),
            account_url: String::new(),
            uuid_name_map: Mutex::new(HashMap::new()),
            parcel_address_map: Mutex::new(HashMap::new()),
            region_uuid: Uuid::nil(),
            region_name: String::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn channel_uri(&self, land_uuid: Uuid) -> String {
        {
            let parcels = self.parcel_address_map.lock();
            if let Some(addr) = parcels.get(&land_uuid.to_string()) {
                return addr.clone();
            }
        }
        let uuid_str = land_uuid.to_string();
        let b64 = STANDARD.encode(uuid_str.as_bytes());
        format!("sip:conf-x{}@{}", b64, self.realm)
    }

    pub fn handle_prelogin(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<VCConfiguration>
<DefaultRealm>{}</DefaultRealm>
<DefaultSIPProxy>{}</DefaultSIPProxy>
<DefaultAttemptUseSTUN>{}</DefaultAttemptUseSTUN>
<DefaultEchoServer>{}</DefaultEchoServer>
<DefaultEchoPort>{}</DefaultEchoPort>
<DefaultWellKnownIP>{}</DefaultWellKnownIP>
<DefaultTimeout>{}</DefaultTimeout>
<UrlResetPassword></UrlResetPassword>
<UrlPrivacyNotice></UrlPrivacyNotice>
<UrlEulaNotice/>
<App.NoBottomLogo>false</App.NoBottomLogo>
</VCConfiguration>"#,
            self.realm,
            self.sip_proxy,
            if self.attempt_stun { "true" } else { "false" },
            self.echo_server,
            self.echo_port,
            self.well_known_ip,
            self.default_timeout
        )
    }

    pub fn handle_signin(&self, userid: &str, _pwd: &str) -> String {
        let map = self.uuid_name_map.lock();
        let display_name = map
            .get(userid)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        let position = map.keys().position(|k| k == userid).unwrap_or(0);
        format!(
            r#"<response xsi:schemaLocation="/xsd/signin.xsd">
<level0>
<status>OK</status>
<body>
<code>200</code>
<cookie_name>lib_session</cookie_name>
<cookie>{}:{}:9303959503950::</cookie>
<auth_token>{}:{}:9303959503950::</auth_token>
<primary>1</primary>
<account_id>{}</account_id>
<displayname>{}</displayname>
<msg>auth successful</msg>
</body>
</level0>
</response>"#,
            userid, position, userid, position, position, display_name
        )
    }

    pub fn handle_buddy(&self, auth_token: &str) -> String {
        let map = self.uuid_name_map.lock();
        let mut buddies = String::new();
        for (i, (username, _name)) in map.iter().enumerate() {
            buddies.push_str(&format!(
                r#"<level3>
<bdy_id>{}</bdy_id>
<bdy_data></bdy_data>
<bdy_uri>sip:{}@{}</bdy_uri>
<bdy_nickname>{}</bdy_nickname>
<bdy_username>{}</bdy_username>
<bdy_domain>{}</bdy_domain>
<bdy_status>A</bdy_status>
<modified_ts>2025-01-01T00:00:00Z</modified_ts>
<b2g_group_id></b2g_group_id>
</level3>"#,
                i, username, self.realm, username, username, self.realm
            ));
        }
        format!(
            r#"<?xml version="1.0" encoding="iso-8859-1" ?>
<response xmlns="http://www.vivox.com" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.vivox.com">
<level0>
<status>OK</status>
<cookie_name>lib_session</cookie_name>
<cookie>{}</cookie>
<auth_token>{}</auth_token>
<body>
<buddies>{}</buddies>
<groups></groups>
</body>
</level0>
</response>"#,
            auth_token, auth_token, buddies
        )
    }

    pub fn handle_watcher(&self, auth_token: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="iso-8859-1" ?>
<response xmlns="http://www.vivox.com" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.vivox.com">
<level0>
<status>OK</status>
<cookie_name>lib_session</cookie_name>
<cookie>{}</cookie>
<auth_token>{}</auth_token>
<body/>
</level0>
</response>"#,
            auth_token, auth_token
        )
    }
}

impl VoiceHandler for FreeSwitchVoiceModule {
    fn provision_voice_account(&self, agent_id: Uuid) -> Result<String> {
        if !self.enabled {
            return Ok(build_undef_response());
        }

        let agentname = encode_agent_name(agent_id);
        let password = "1234";

        {
            let mut map = self.uuid_name_map.lock();
            if !map.contains_key(&agentname) {
                map.insert(agentname.clone(), agent_id.to_string());
            }
        }

        Ok(build_provision_response(
            &agentname,
            password,
            &self.realm,
            &self.account_url,
        ))
    }

    fn parcel_voice_info(
        &self,
        _agent_id: Uuid,
        parcel_flags: u32,
        parcel_uuid: Uuid,
        parcel_local_id: i32,
        _parcel_name: &str,
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

        let land_uuid = if is_estate_channel(parcel_flags) {
            region_uuid
        } else {
            parcel_uuid
        };

        let uri = self.channel_uri(land_uuid);
        Ok(build_parcel_voice_response(
            parcel_local_id,
            region_name,
            &uri,
        ))
    }

    fn voice_enabled(&self) -> bool {
        self.enabled
    }

    fn fs_handle_prelogin(&self) -> Option<String> {
        Some(self.handle_prelogin())
    }

    fn fs_handle_signin(&self, userid: &str, pwd: &str) -> Option<String> {
        Some(self.handle_signin(userid, pwd))
    }

    fn fs_handle_buddy(&self, auth_token: &str) -> Option<String> {
        Some(self.handle_buddy(auth_token))
    }

    fn fs_handle_watcher(&self, auth_token: &str) -> Option<String> {
        Some(self.handle_watcher(auth_token))
    }
}

impl IVoiceModule for FreeSwitchVoiceModule {
    fn set_land_sip_address(&self, sip: &str, parcel_global_id: Uuid) {
        let mut map = self.parcel_address_map.lock();
        map.insert(parcel_global_id.to_string(), sip.to_string());
    }
}

#[async_trait]
impl RegionModule for FreeSwitchVoiceModule {
    fn name(&self) -> &'static str {
        "FreeSwitchVoiceModule"
    }

    async fn initialize(&mut self, config: &ModuleConfig) -> Result<()> {
        self.enabled = config.get_bool("Enabled", false);
        if !self.enabled {
            return Ok(());
        }

        let server = config.get_or("ServerAddress", "");
        if server.is_empty() {
            warn!("[FreeSWITCH] ServerAddress not configured, disabling");
            self.enabled = false;
            return Ok(());
        }

        self.realm = config.get_or("Realm", &server);
        self.sip_proxy = config.get_or("SIPProxy", &format!("{}:5060", server));
        self.echo_server = config.get_or("EchoServer", &server);
        self.echo_port = config.get_or("EchoPort", "50505").parse().unwrap_or(50505);
        self.attempt_stun = config.get_bool("AttemptUseSTUN", false);
        self.default_timeout = config
            .get_or("DefaultTimeout", "5000")
            .parse()
            .unwrap_or(5000);
        self.well_known_ip = server.clone();
        self.account_url = format!("http://{}:9000{}/", server, self.api_prefix);

        info!(
            "[FreeSWITCH] FreeSwitchVoiceModule initialized (realm={})",
            self.realm
        );
        Ok(())
    }

    async fn add_region(&mut self, scene: &SceneContext) -> Result<()> {
        self.region_uuid = scene.region_uuid;
        self.region_name = scene.region_name.clone();
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
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
impl SharedRegionModule for FreeSwitchVoiceModule {}
