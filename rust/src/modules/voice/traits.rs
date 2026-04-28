use anyhow::Result;
use uuid::Uuid;

pub trait VoiceHandler: Send + Sync + 'static {
    fn provision_voice_account(&self, agent_id: Uuid) -> Result<String>;

    fn parcel_voice_info(
        &self,
        agent_id: Uuid,
        parcel_flags: u32,
        parcel_uuid: Uuid,
        parcel_local_id: i32,
        parcel_name: &str,
        region_name: &str,
        region_uuid: Uuid,
        estate_allow_voice: bool,
    ) -> Result<String>;

    fn voice_enabled(&self) -> bool;

    fn fs_handle_prelogin(&self) -> Option<String> {
        None
    }
    fn fs_handle_signin(&self, _userid: &str, _pwd: &str) -> Option<String> {
        None
    }
    fn fs_handle_buddy(&self, _auth_token: &str) -> Option<String> {
        None
    }
    fn fs_handle_watcher(&self, _auth_token: &str) -> Option<String> {
        None
    }
}

pub trait IVoiceModule: Send + Sync + 'static {
    fn set_land_sip_address(&self, sip: &str, parcel_global_id: Uuid);
}
