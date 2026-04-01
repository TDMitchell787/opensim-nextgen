pub mod common;
pub mod traits;
pub mod vivox_api;
pub mod vivox;
pub mod freeswitch;

pub use traits::{VoiceHandler, IVoiceModule};
pub use vivox::VivoxVoiceModule;
pub use freeswitch::FreeSwitchVoiceModule;
