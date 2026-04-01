use base64::{Engine as _, engine::general_purpose::STANDARD};
use uuid::Uuid;

pub const ALLOW_VOICE_CHAT: u32 = 0x20000000;
pub const USE_ESTATE_VOICE_CHAN: u32 = 0x40000000;

pub fn encode_agent_name(agent_id: Uuid) -> String {
    let bytes = agent_id.as_bytes();
    let b64 = STANDARD.encode(bytes);
    let safe = b64.replace('+', "-").replace('/', "_");
    format!("x{}", safe)
}

pub fn generate_vivox_password() -> String {
    Uuid::new_v4()
        .to_string()
        .replace('-', "Z")
        .chars()
        .take(16)
        .collect()
}

pub fn check_voice_allowed(estate_allow_voice: bool, parcel_flags: u32) -> bool {
    estate_allow_voice && (parcel_flags & ALLOW_VOICE_CHAT) != 0
}

pub fn is_estate_channel(parcel_flags: u32) -> bool {
    (parcel_flags & USE_ESTATE_VOICE_CHAN) != 0
}

pub fn build_provision_response(
    username: &str,
    password: &str,
    voice_sip_uri_hostname: &str,
    voice_account_server_name: &str,
) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<llsd>
<map>
<key>username</key><string>{}</string>
<key>password</key><string>{}</string>
<key>voice_sip_uri_hostname</key><string>{}</string>
<key>voice_account_server_name</key><string>{}</string>
</map>
</llsd>"#,
        username, password, voice_sip_uri_hostname, voice_account_server_name
    )
}

pub fn build_parcel_voice_response(
    parcel_local_id: i32,
    region_name: &str,
    channel_uri: &str,
) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<llsd>
<map>
<key>parcel_local_id</key><integer>{}</integer>
<key>region_name</key><string>{}</string>
<key>voice_credentials</key>
<map>
<key>channel_uri</key><string>{}</string>
</map>
</map>
</llsd>"#,
        parcel_local_id, region_name, channel_uri
    )
}

pub fn build_undef_response() -> String {
    "<llsd><undef /></llsd>".to_string()
}
