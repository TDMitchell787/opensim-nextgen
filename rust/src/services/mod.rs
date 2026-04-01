//! OpenSim Services Module
//!
//! Contains service implementations matching OpenSim master architecture.
//!
//! ## Architecture
//!
//! Services are defined as traits to support both standalone and grid modes:
//! - **Standalone mode**: Local*Service implementations with direct database access
//! - **Grid mode**: Remote*Service implementations with HTTP calls to ROBUST
//! - **Hybrid mode**: Mix of local and remote based on configuration
//!
//! ## Module Structure
//!
//! - `traits` - Service trait definitions (interfaces)
//! - `local/` - Local implementations (standalone mode)
//! - `remote/` - Remote (HTTP) implementations (grid mode)
//! - `factory` - Service factory for mode-based instantiation
//! - `avatar_service` - Avatar appearance service (local implementation)

pub mod traits;
pub mod local;
pub mod remote;
pub mod factory;
pub mod avatar_service;
pub mod robust;
pub mod config_parser;
pub mod region_registration;
pub mod hypergrid;

pub use traits::{
    AvatarServiceTrait,
    GridServiceTrait,
    UserAccountServiceTrait,
    AuthenticationServiceTrait,
    AssetServiceTrait,
    InventoryServiceTrait,
    PresenceServiceTrait,
    GatekeeperServiceTrait,
    UserAgentServiceTrait,
    GridUserServiceTrait,
    AgentPrefsServiceTrait,
    HGFriendsServiceTrait,
    AuthorizationServiceTrait,
    FriendsServiceTrait,
    LandServiceTrait,
    OfflineIMServiceTrait,
    ProfilesServiceTrait,
    AvatarData,
    RegionInfo,
    UserAccount,
    AuthInfo,
    AssetBase,
    AssetMetadata,
    InventoryFolder,
    InventoryItem,
    InventoryCollection,
    PresenceInfo,
    GridUserInfo,
    AgentPrefs,
    ServiceMode,
    ServiceConfig,
    HGRegionInfo,
    TravelingAgentData,
    AgentCircuitData,
    MuteListServiceTrait,
    MuteData,
    EstateServiceTrait,
    EstateSettings,
    EstateBan,
    MapImageServiceTrait,
    FriendInfo,
    LandData,
    OfflineIM,
    UserProfileProperties,
    UserProfilePick,
    UserClassifiedAdd,
    UserProfileNotes,
    UserPreferences,
};

pub use local::{
    LocalGridService,
    LocalUserAccountService,
    LocalAssetService,
    LocalAuthenticationService,
    LocalInventoryService,
    LocalPresenceService,
    LocalGridUserService,
    LocalAgentPrefsService,
    LocalMuteListService,
    LocalEstateService,
    LocalMapImageService,
    LocalAuthorizationService,
    LocalFriendsService,
    LocalLandService,
    LocalOfflineIMService,
    LocalProfilesService,
};

pub use remote::{
    RemoteGridService,
    RemoteUserAccountService,
    RemoteAssetService,
    RemoteAvatarService,
};

pub use factory::{ServiceFactory, ServiceContainer};

pub use avatar_service::AvatarService;

pub use robust::RobustState;
