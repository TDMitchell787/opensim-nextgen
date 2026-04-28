//! Local Service Implementations
//!
//! These services run in-process with direct database access.
//! Used in standalone mode or when services are co-located.
//!
//! Architecture:
//! - Each service implements its corresponding trait from `super::traits`
//! - Direct database access via sqlx pools
//! - No network overhead for local operations

pub mod agentprefs_service;
pub mod asset_service;
pub mod auth_service;
pub mod authorization_service;
pub mod estate_service;
pub mod friends_service;
pub mod gatekeeper_service;
pub mod grid_service;
pub mod griduser_service;
pub mod inventory_service;
pub mod land_service;
pub mod map_image_service;
pub mod mutelist_service;
pub mod offlineim_service;
pub mod presence_service;
pub mod profiles_service;
pub mod user_account_service;
pub mod user_agent_service;

pub use agentprefs_service::LocalAgentPrefsService;
pub use asset_service::LocalAssetService;
pub use auth_service::LocalAuthenticationService;
pub use authorization_service::LocalAuthorizationService;
pub use estate_service::LocalEstateService;
pub use friends_service::LocalFriendsService;
pub use gatekeeper_service::LocalGatekeeperService;
pub use grid_service::LocalGridService;
pub use griduser_service::LocalGridUserService;
pub use inventory_service::LocalInventoryService;
pub use land_service::LocalLandService;
pub use map_image_service::LocalMapImageService;
pub use mutelist_service::LocalMuteListService;
pub use offlineim_service::LocalOfflineIMService;
pub use presence_service::LocalPresenceService;
pub use profiles_service::LocalProfilesService;
pub use user_account_service::LocalUserAccountService;
pub use user_agent_service::LocalUserAgentService;
