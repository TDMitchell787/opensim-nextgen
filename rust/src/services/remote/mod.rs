//! Remote Service Implementations
//!
//! These services connect to remote ROBUST server via HTTP.
//! Used in grid mode when services are hosted separately.
//!
//! Architecture:
//! - Each service implements its corresponding trait from `super::traits`
//! - HTTP requests to ROBUST server endpoints
//! - XML-RPC and REST API compatibility with OpenSim master

pub mod grid_service;
pub mod user_account_service;
pub mod asset_service;
pub mod auth_service;
pub mod inventory_service;
pub mod presence_service;
pub mod avatar_service;
pub mod gatekeeper_connector;
pub mod uas_connector;

pub use grid_service::RemoteGridService;
pub use user_account_service::RemoteUserAccountService;
pub use asset_service::RemoteAssetService;
pub use auth_service::RemoteAuthenticationService;
pub use inventory_service::RemoteInventoryService;
pub use presence_service::RemotePresenceService;
pub use avatar_service::RemoteAvatarService;
pub use gatekeeper_connector::GatekeeperServiceConnector;
pub use uas_connector::UserAgentServiceConnector;
