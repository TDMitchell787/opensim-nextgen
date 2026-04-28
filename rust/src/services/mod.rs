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

pub mod avatar_service;
pub mod config_parser;
pub mod factory;
pub mod hypergrid;
pub mod local;
pub mod region_registration;
pub mod remote;
pub mod robust;
pub mod traits;

pub use traits::{
    AgentCircuitData, AgentPrefs, AgentPrefsServiceTrait, AssetBase, AssetMetadata,
    AssetServiceTrait, AuthInfo, AuthenticationServiceTrait, AuthorizationServiceTrait, AvatarData,
    AvatarServiceTrait, EstateBan, EstateServiceTrait, EstateSettings, FriendInfo,
    FriendsServiceTrait, GatekeeperServiceTrait, GridServiceTrait, GridUserInfo,
    GridUserServiceTrait, HGFriendsServiceTrait, HGRegionInfo, InventoryCollection,
    InventoryFolder, InventoryItem, InventoryServiceTrait, LandData, LandServiceTrait,
    MapImageServiceTrait, MuteData, MuteListServiceTrait, OfflineIM, OfflineIMServiceTrait,
    PresenceInfo, PresenceServiceTrait, ProfilesServiceTrait, RegionInfo, ServiceConfig,
    ServiceMode, TravelingAgentData, UserAccount, UserAccountServiceTrait, UserAgentServiceTrait,
    UserClassifiedAdd, UserPreferences, UserProfileNotes, UserProfilePick, UserProfileProperties,
};

pub use local::{
    LocalAgentPrefsService, LocalAssetService, LocalAuthenticationService,
    LocalAuthorizationService, LocalEstateService, LocalFriendsService, LocalGridService,
    LocalGridUserService, LocalInventoryService, LocalLandService, LocalMapImageService,
    LocalMuteListService, LocalOfflineIMService, LocalPresenceService, LocalProfilesService,
    LocalUserAccountService,
};

pub use remote::{
    RemoteAssetService, RemoteAvatarService, RemoteGridService, RemoteUserAccountService,
};

pub use factory::{ServiceContainer, ServiceFactory};

pub use avatar_service::AvatarService;

pub use robust::RobustState;
