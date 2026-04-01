pub mod xml_response;
pub mod xmlrpc;
pub mod grid_handler;
pub mod user_account_handler;
pub mod auth_handler;
pub mod asset_handler;
pub mod inventory_handler;
pub mod presence_handler;
pub mod avatar_handler;
pub mod gatekeeper_handler;
pub mod uas_handler;
pub mod helo_handler;
pub mod hgfriends_handler;
pub mod hg_inventory_handler;
pub mod griduser_handler;
pub mod agentprefs_handler;
pub mod bakes_handler;
pub mod mutelist_handler;
pub mod estate_handler;
pub mod map_handler;
pub mod authorization_handler;
pub mod friends_handler;
pub mod land_handler;
pub mod offlineim_handler;
pub mod neighbour_handler;
pub mod profiles_handler;
pub mod freeswitch_handler;
pub mod grid_info_handler;
pub mod router;
pub mod server;

pub use router::{create_robust_router, create_uas_router, create_gatekeeper_router};
pub use server::start_robust_server;

use std::sync::Arc;
use crate::services::traits::{
    GridServiceTrait, UserAccountServiceTrait, AuthenticationServiceTrait,
    AssetServiceTrait, InventoryServiceTrait, PresenceServiceTrait, AvatarServiceTrait,
    GatekeeperServiceTrait, UserAgentServiceTrait,
    GridUserServiceTrait, AgentPrefsServiceTrait, HGFriendsServiceTrait,
    MuteListServiceTrait, EstateServiceTrait, MapImageServiceTrait,
    AuthorizationServiceTrait, FriendsServiceTrait, LandServiceTrait,
    OfflineIMServiceTrait, ProfilesServiceTrait,
};
use crate::login_session::CircuitCodeRegistry;
use crate::session::SessionManager;
use crate::caps::CapsManager;

#[derive(Clone)]
pub struct RobustState {
    pub grid_service: Arc<dyn GridServiceTrait>,
    pub user_account_service: Arc<dyn UserAccountServiceTrait>,
    pub auth_service: Arc<dyn AuthenticationServiceTrait>,
    pub asset_service: Arc<dyn AssetServiceTrait>,
    pub inventory_service: Arc<dyn InventoryServiceTrait>,
    pub presence_service: Arc<dyn PresenceServiceTrait>,
    pub avatar_service: Arc<dyn AvatarServiceTrait>,
    pub gatekeeper_service: Option<Arc<dyn GatekeeperServiceTrait>>,
    pub uas_service: Option<Arc<dyn UserAgentServiceTrait>>,
    pub hg_inventory_service: Option<Arc<dyn InventoryServiceTrait>>,
    pub griduser_service: Option<Arc<dyn GridUserServiceTrait>>,
    pub agentprefs_service: Option<Arc<dyn AgentPrefsServiceTrait>>,
    pub hg_friends_service: Option<Arc<dyn HGFriendsServiceTrait>>,
    pub bakes_dir: Option<String>,
    pub mutelist_service: Option<Arc<dyn MuteListServiceTrait>>,
    pub estate_service: Option<Arc<dyn EstateServiceTrait>>,
    pub map_service: Option<Arc<dyn MapImageServiceTrait>>,
    pub authorization_service: Option<Arc<dyn AuthorizationServiceTrait>>,
    pub friends_service: Option<Arc<dyn FriendsServiceTrait>>,
    pub land_service: Option<Arc<dyn LandServiceTrait>>,
    pub offlineim_service: Option<Arc<dyn OfflineIMServiceTrait>>,
    pub profiles_service: Option<Arc<dyn ProfilesServiceTrait>>,
    pub db_pool: Option<sqlx::PgPool>,
}

#[derive(Clone)]
pub struct UasState {
    pub uas_service: Arc<dyn UserAgentServiceTrait>,
}

#[derive(Clone)]
pub struct GatekeeperState {
    pub gatekeeper_service: Arc<dyn GatekeeperServiceTrait>,
    pub circuit_code_registry: Option<CircuitCodeRegistry>,
    pub session_manager: Option<Arc<SessionManager>>,
    pub caps_manager: Option<Arc<CapsManager>>,
}
