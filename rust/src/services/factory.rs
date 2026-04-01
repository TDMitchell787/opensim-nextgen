//! Service Factory
//!
//! Creates service instances based on configuration mode (Standalone/Grid/Hybrid).
//! Provides a unified interface for service access regardless of deployment mode.
//!
//! Architecture:
//! - Standalone mode: All services use Local* implementations (direct database)
//! - Grid mode: All services use Remote* implementations (HTTP to ROBUST)
//! - Hybrid mode: Mix of local and remote based on configuration

use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::info;

use crate::database::multi_backend::DatabaseConnection;
use crate::services::traits::{
    ServiceMode, ServiceConfig,
    GridServiceTrait, UserAccountServiceTrait, AssetServiceTrait,
    AuthenticationServiceTrait, InventoryServiceTrait, PresenceServiceTrait,
    AvatarServiceTrait,
};
use crate::services::local::{
    LocalGridService, LocalUserAccountService, LocalAssetService,
    LocalAuthenticationService, LocalInventoryService, LocalPresenceService,
};
use crate::services::remote::{
    RemoteGridService, RemoteUserAccountService, RemoteAssetService,
    RemoteAuthenticationService, RemoteInventoryService, RemotePresenceService,
    RemoteAvatarService,
};

pub struct ServiceFactory;

impl ServiceFactory {
    pub fn create_grid_service(
        config: &ServiceConfig,
        db_connection: Option<Arc<DatabaseConnection>>,
    ) -> Result<Arc<dyn GridServiceTrait>> {
        match config.mode {
            ServiceMode::Standalone => {
                let conn = db_connection
                    .ok_or_else(|| anyhow!("Database connection required for standalone mode"))?;
                info!("Creating local grid service (standalone mode)");
                Ok(Arc::new(LocalGridService::new(conn)))
            }
            ServiceMode::Grid => {
                let uri = config.grid_server_uri.as_ref()
                    .ok_or_else(|| anyhow!("Grid server URI required for grid mode"))?;
                info!("Creating remote grid service (grid mode): {}", uri);
                Ok(Arc::new(RemoteGridService::new(uri)))
            }
            ServiceMode::Hybrid => {
                if let Some(uri) = &config.grid_server_uri {
                    info!("Creating remote grid service (hybrid mode): {}", uri);
                    Ok(Arc::new(RemoteGridService::new(uri)))
                } else if let Some(conn) = db_connection {
                    info!("Creating local grid service (hybrid mode - fallback to local)");
                    Ok(Arc::new(LocalGridService::new(conn)))
                } else {
                    Err(anyhow!("Either grid server URI or database connection required"))
                }
            }
        }
    }

    pub fn create_user_account_service(
        config: &ServiceConfig,
        db_connection: Option<Arc<DatabaseConnection>>,
    ) -> Result<Arc<dyn UserAccountServiceTrait>> {
        match config.mode {
            ServiceMode::Standalone => {
                let conn = db_connection
                    .ok_or_else(|| anyhow!("Database connection required for standalone mode"))?;
                info!("Creating local user account service (standalone mode)");
                Ok(Arc::new(LocalUserAccountService::new(conn)))
            }
            ServiceMode::Grid => {
                let uri = config.user_account_server_uri.as_ref()
                    .or(config.grid_server_uri.as_ref())
                    .ok_or_else(|| anyhow!("User account server URI required for grid mode"))?;
                info!("Creating remote user account service (grid mode): {}", uri);
                Ok(Arc::new(RemoteUserAccountService::new(uri)))
            }
            ServiceMode::Hybrid => {
                if let Some(uri) = &config.user_account_server_uri {
                    info!("Creating remote user account service (hybrid mode): {}", uri);
                    Ok(Arc::new(RemoteUserAccountService::new(uri)))
                } else if let Some(conn) = db_connection {
                    info!("Creating local user account service (hybrid mode - fallback to local)");
                    Ok(Arc::new(LocalUserAccountService::new(conn)))
                } else {
                    Err(anyhow!("Either user account server URI or database connection required"))
                }
            }
        }
    }

    pub fn create_asset_service(
        config: &ServiceConfig,
        db_connection: Option<Arc<DatabaseConnection>>,
    ) -> Result<Arc<dyn AssetServiceTrait>> {
        match config.mode {
            ServiceMode::Standalone => {
                let conn = db_connection
                    .ok_or_else(|| anyhow!("Database connection required for standalone mode"))?;
                info!("Creating local asset service (standalone mode)");
                Ok(Arc::new(LocalAssetService::new(conn)))
            }
            ServiceMode::Grid => {
                let uri = config.asset_server_uri.as_ref()
                    .or(config.grid_server_uri.as_ref())
                    .ok_or_else(|| anyhow!("Asset server URI required for grid mode"))?;
                info!("Creating remote asset service (grid mode): {}", uri);
                Ok(Arc::new(RemoteAssetService::new(uri)))
            }
            ServiceMode::Hybrid => {
                if let Some(uri) = &config.asset_server_uri {
                    info!("Creating remote asset service (hybrid mode): {}", uri);
                    Ok(Arc::new(RemoteAssetService::new(uri)))
                } else if let Some(conn) = db_connection {
                    info!("Creating local asset service (hybrid mode - fallback to local)");
                    Ok(Arc::new(LocalAssetService::new(conn)))
                } else {
                    Err(anyhow!("Either asset server URI or database connection required"))
                }
            }
        }
    }

    pub fn create_authentication_service(
        config: &ServiceConfig,
        db_connection: Option<Arc<DatabaseConnection>>,
    ) -> Result<Arc<dyn AuthenticationServiceTrait>> {
        match config.mode {
            ServiceMode::Standalone => {
                let conn = db_connection
                    .ok_or_else(|| anyhow!("Database connection required for standalone mode"))?;
                info!("Creating local authentication service (standalone mode)");
                Ok(Arc::new(LocalAuthenticationService::new(conn)))
            }
            ServiceMode::Grid => {
                let uri = config.authentication_server_uri.as_ref()
                    .or(config.grid_server_uri.as_ref())
                    .ok_or_else(|| anyhow!("Authentication server URI required for grid mode"))?;
                info!("Creating remote authentication service (grid mode): {}", uri);
                Ok(Arc::new(RemoteAuthenticationService::new(uri)))
            }
            ServiceMode::Hybrid => {
                if let Some(uri) = &config.authentication_server_uri {
                    info!("Creating remote authentication service (hybrid mode): {}", uri);
                    Ok(Arc::new(RemoteAuthenticationService::new(uri)))
                } else if let Some(conn) = db_connection {
                    info!("Creating local authentication service (hybrid mode - fallback to local)");
                    Ok(Arc::new(LocalAuthenticationService::new(conn)))
                } else {
                    Err(anyhow!("Either authentication server URI or database connection required"))
                }
            }
        }
    }

    pub fn create_inventory_service(
        config: &ServiceConfig,
        db_connection: Option<Arc<DatabaseConnection>>,
    ) -> Result<Arc<dyn InventoryServiceTrait>> {
        match config.mode {
            ServiceMode::Standalone => {
                let conn = db_connection
                    .ok_or_else(|| anyhow!("Database connection required for standalone mode"))?;
                info!("Creating local inventory service (standalone mode)");
                Ok(Arc::new(LocalInventoryService::new(conn)))
            }
            ServiceMode::Grid => {
                let uri = config.inventory_server_uri.as_ref()
                    .or(config.grid_server_uri.as_ref())
                    .ok_or_else(|| anyhow!("Inventory server URI required for grid mode"))?;
                info!("Creating remote inventory service (grid mode): {}", uri);
                Ok(Arc::new(RemoteInventoryService::new(uri)))
            }
            ServiceMode::Hybrid => {
                if let Some(uri) = &config.inventory_server_uri {
                    info!("Creating remote inventory service (hybrid mode): {}", uri);
                    Ok(Arc::new(RemoteInventoryService::new(uri)))
                } else if let Some(conn) = db_connection {
                    info!("Creating local inventory service (hybrid mode - fallback to local)");
                    Ok(Arc::new(LocalInventoryService::new(conn)))
                } else {
                    Err(anyhow!("Either inventory server URI or database connection required"))
                }
            }
        }
    }

    pub fn create_presence_service(
        config: &ServiceConfig,
        db_connection: Option<Arc<DatabaseConnection>>,
    ) -> Result<Arc<dyn PresenceServiceTrait>> {
        match config.mode {
            ServiceMode::Standalone => {
                let conn = db_connection
                    .ok_or_else(|| anyhow!("Database connection required for standalone mode"))?;
                info!("Creating local presence service (standalone mode)");
                Ok(Arc::new(LocalPresenceService::new(conn)))
            }
            ServiceMode::Grid => {
                let uri = config.presence_server_uri.as_ref()
                    .or(config.grid_server_uri.as_ref())
                    .ok_or_else(|| anyhow!("Presence server URI required for grid mode"))?;
                info!("Creating remote presence service (grid mode): {}", uri);
                Ok(Arc::new(RemotePresenceService::new(uri)))
            }
            ServiceMode::Hybrid => {
                if let Some(uri) = &config.presence_server_uri {
                    info!("Creating remote presence service (hybrid mode): {}", uri);
                    Ok(Arc::new(RemotePresenceService::new(uri)))
                } else if let Some(conn) = db_connection {
                    info!("Creating local presence service (hybrid mode - fallback to local)");
                    Ok(Arc::new(LocalPresenceService::new(conn)))
                } else {
                    Err(anyhow!("Either presence server URI or database connection required"))
                }
            }
        }
    }

    pub fn create_avatar_service(
        config: &ServiceConfig,
        local_avatar_service: Option<Arc<dyn AvatarServiceTrait>>,
    ) -> Result<Arc<dyn AvatarServiceTrait>> {
        match config.mode {
            ServiceMode::Standalone => {
                local_avatar_service
                    .ok_or_else(|| anyhow!("Local avatar service required for standalone mode"))
            }
            ServiceMode::Grid => {
                let uri = config.avatar_server_uri.as_ref()
                    .or(config.grid_server_uri.as_ref())
                    .ok_or_else(|| anyhow!("Avatar server URI required for grid mode"))?;
                info!("Creating remote avatar service (grid mode): {}", uri);
                Ok(Arc::new(RemoteAvatarService::new(uri)))
            }
            ServiceMode::Hybrid => {
                if let Some(uri) = &config.avatar_server_uri {
                    info!("Creating remote avatar service (hybrid mode): {}", uri);
                    Ok(Arc::new(RemoteAvatarService::new(uri)))
                } else {
                    local_avatar_service
                        .ok_or_else(|| anyhow!("Either avatar server URI or local avatar service required"))
                }
            }
        }
    }
}

pub struct ServiceContainer {
    pub grid_service: Arc<dyn GridServiceTrait>,
    pub user_account_service: Arc<dyn UserAccountServiceTrait>,
    pub asset_service: Arc<dyn AssetServiceTrait>,
    pub authentication_service: Arc<dyn AuthenticationServiceTrait>,
    pub inventory_service: Arc<dyn InventoryServiceTrait>,
    pub presence_service: Arc<dyn PresenceServiceTrait>,
    pub avatar_service: Option<Arc<dyn AvatarServiceTrait>>,
    pub config: ServiceConfig,
}

impl ServiceContainer {
    pub fn new(
        config: ServiceConfig,
        db_connection: Option<Arc<DatabaseConnection>>,
    ) -> Result<Self> {
        info!("Initializing service container with mode: {:?}", config.mode);

        let grid_service = ServiceFactory::create_grid_service(&config, db_connection.clone())?;
        let user_account_service = ServiceFactory::create_user_account_service(&config, db_connection.clone())?;
        let asset_service = ServiceFactory::create_asset_service(&config, db_connection.clone())?;
        let authentication_service = ServiceFactory::create_authentication_service(&config, db_connection.clone())?;
        let inventory_service = ServiceFactory::create_inventory_service(&config, db_connection.clone())?;
        let presence_service = ServiceFactory::create_presence_service(&config, db_connection)?;
        let avatar_service = ServiceFactory::create_avatar_service(&config, None).ok();

        Ok(Self {
            grid_service,
            user_account_service,
            asset_service,
            authentication_service,
            inventory_service,
            presence_service,
            avatar_service,
            config,
        })
    }

    pub fn with_avatar_service(mut self, avatar_service: Arc<dyn AvatarServiceTrait>) -> Self {
        self.avatar_service = Some(avatar_service);
        self
    }

    pub fn grid(&self) -> &Arc<dyn GridServiceTrait> {
        &self.grid_service
    }

    pub fn user_accounts(&self) -> &Arc<dyn UserAccountServiceTrait> {
        &self.user_account_service
    }

    pub fn assets(&self) -> &Arc<dyn AssetServiceTrait> {
        &self.asset_service
    }

    pub fn authentication(&self) -> &Arc<dyn AuthenticationServiceTrait> {
        &self.authentication_service
    }

    pub fn inventory(&self) -> &Arc<dyn InventoryServiceTrait> {
        &self.inventory_service
    }

    pub fn presence(&self) -> &Arc<dyn PresenceServiceTrait> {
        &self.presence_service
    }

    pub fn avatar(&self) -> Option<&Arc<dyn AvatarServiceTrait>> {
        self.avatar_service.as_ref()
    }

    pub fn mode(&self) -> ServiceMode {
        self.config.mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert_eq!(config.mode, ServiceMode::Standalone);
        assert!(config.grid_server_uri.is_none());
    }

    #[test]
    fn test_service_mode_default() {
        let mode = ServiceMode::default();
        assert_eq!(mode, ServiceMode::Standalone);
    }

    #[test]
    fn test_grid_config() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            grid_server_uri: Some("http://localhost:8003".to_string()),
            asset_server_uri: Some("http://localhost:8003".to_string()),
            inventory_server_uri: Some("http://localhost:8003".to_string()),
            user_account_server_uri: Some("http://localhost:8003".to_string()),
            presence_server_uri: Some("http://localhost:8003".to_string()),
            avatar_server_uri: Some("http://localhost:8003".to_string()),
            authentication_server_uri: Some("http://localhost:8003".to_string()),
        };

        assert_eq!(config.mode, ServiceMode::Grid);
        assert_eq!(config.grid_server_uri, Some("http://localhost:8003".to_string()));
    }

    #[test]
    fn test_remote_grid_service_creation() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            grid_server_uri: Some("http://localhost:8003".to_string()),
            ..Default::default()
        };

        let result = ServiceFactory::create_grid_service(&config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_standalone_requires_db() {
        let config = ServiceConfig {
            mode: ServiceMode::Standalone,
            ..Default::default()
        };

        let result = ServiceFactory::create_grid_service(&config, None);
        assert!(result.is_err());
    }
}
