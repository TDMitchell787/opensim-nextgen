use std::sync::Arc;
use anyhow::Result;

pub mod asset;
pub mod avatar;
pub mod baking;
pub mod capabilities;
pub mod caps;
pub mod config;
pub mod database;
pub mod economy;
pub mod ffi;
pub mod inventory;
pub mod network;
pub mod opensim_compatibility;
pub mod performance;
pub mod region;
pub mod scripting;
pub mod social;
pub mod state;
pub mod monitoring;
pub mod ziti;
pub mod ai;
pub mod vr;
pub mod mobile;
pub mod pwa;
pub mod sync;
pub mod grid;
pub mod analytics;
pub mod archives;
pub mod content;
pub mod reporting;
// pub mod client_sdk;
pub mod community;
pub mod setup;
pub mod login_session;
pub mod login_state;
pub mod login_stage_tracker;
pub mod auth;
pub mod session;
pub mod xmlrpc;
pub mod udp;
pub mod login_service;
pub mod services;
pub mod protocol;
pub mod instance_manager;
pub mod configuration_builder;
pub mod materials;
pub mod media;
pub mod mesh;
pub mod modules;
pub mod readiness;

use std::time::Duration;
use monitoring::{MonitoringSystem, MonitoringConfig};
use network::session::SessionManager;
use region::RegionManager;
use state::StateManager;
use ffi::physics::PhysicsBridge;
use avatar::AdvancedAvatarManager;
use economy::{VirtualEconomyManager, EconomyConfig};
use social::{SocialFeaturesManager, SocialConfig};
use ai::{AIManager, AIConfig};
use vr::{VRManager, VRConfig};
use mobile::{MobileRuntimeManager, MobileConfig};
use pwa::{PWAServiceManager, PWAConfig};
use sync::{CrossPlatformSyncEngine, SyncConfig};
use grid::{GridFederationManager, GridScalingManager, FederationConfig, ScalingConfig};
use analytics::{AnalyticsManager, AnalyticsConfig};
use content::manager::{ContentManager, ContentManagerConfig};
use reporting::manager::{ReportingManager, ReportingManagerConfig};

pub struct OpenSimServer {
    asset_manager: Arc<asset::AssetManager>,
    network_manager: Arc<network::NetworkManager>,
    region_manager: Arc<RegionManager>,
    session_manager: Arc<SessionManager>,
    monitoring: Arc<MonitoringSystem>,
    state_manager: Arc<StateManager>,
    avatar_manager: Arc<AdvancedAvatarManager>,
    economy_manager: Arc<VirtualEconomyManager>,
    social_manager: Arc<SocialFeaturesManager>,
    ai_manager: Arc<AIManager>,
    vr_manager: Arc<VRManager>,
    mobile_runtime: Arc<MobileRuntimeManager>,
    pwa_service: Arc<PWAServiceManager>,
    sync_engine: Arc<CrossPlatformSyncEngine>,
    grid_federation: Arc<GridFederationManager>,
    grid_scaling: Arc<GridScalingManager>,
    analytics_manager: Arc<AnalyticsManager>,
    content_manager: Arc<ContentManager>,
    reporting_manager: Arc<ReportingManager>,
    community_platform: Option<Arc<community::CommunityPlatform>>,
}

impl OpenSimServer {
    pub async fn new() -> Result<Self> {
        // Initialize database first
        let database_manager = Arc::new(crate::database::DatabaseManager::new("sqlite://opensim_lib.db").await?);
        
        // Initialize asset system components
        let cache_config = asset::CacheConfig::default();
        let asset_cache = Arc::new(asset::AssetCache::new(cache_config).await?);
        
        // Create simple CDN config
        let cdn_config = asset::CdnConfig {
            provider: asset::CdnProvider::Generic,
            base_url: "http://localhost:8080/assets".to_string(),
            api_key: None,
            provider_config: std::collections::HashMap::new(),
            default_ttl: 3600,
            auto_distribute: false,
            regions: vec![],
        };
        let cdn_manager = Arc::new(asset::CdnManager::new(cdn_config).await?);
        let storage_backend: Arc<dyn asset::StorageBackend> = Arc::new(asset::FileSystemBackend::new("assets")?);
        let asset_config = asset::AssetManagerConfig::default();
        let asset_manager = Arc::new(asset::AssetManager::new(
            database_manager.clone(),
            asset_cache,
            cdn_manager,
            storage_backend,
            asset_config,
        ).await?);
        
        // Initialize other components
        let monitoring_config = MonitoringConfig::default();
        let monitoring = Arc::new(MonitoringSystem::new(monitoring_config)?);
        let session_manager = Arc::new(SessionManager::new(Duration::from_secs(600)));
        let physics_bridge = Arc::new(PhysicsBridge::new()?);
        let state_manager = Arc::new(StateManager::new()?);
        let region_manager = Arc::new(RegionManager::new(physics_bridge, state_manager.clone()));
        
        // Get user account database from database manager
        let user_account_database = database_manager.user_accounts();
        
        // Initialize avatar system
        let avatar_manager = Arc::new(AdvancedAvatarManager::new(database_manager.clone()));
        
        // Initialize avatar database tables
        avatar_manager.persistence_layer.initialize_tables().await?;
        avatar_manager.social_features.initialize_tables().await?;
        
        // Initialize economy system
        let economy_config = EconomyConfig::default();
        let economy_manager = Arc::new(VirtualEconomyManager::new(database_manager.clone(), economy_config));
        
        // Initialize economy system
        economy_manager.initialize().await?;
        
        // Initialize social features system
        let social_config = SocialConfig::default();
        let social_manager = Arc::new(SocialFeaturesManager::new(database_manager.clone(), social_config));
        
        // Initialize social features system
        social_manager.initialize().await?;
        
        // Initialize AI/ML system
        let ai_config = AIConfig::default();
        let ai_manager = AIManager::new(
            ai_config,
            avatar_manager.clone(),
            monitoring.get_metrics_collector(),
            database_manager.clone(),
        ).await?;
        
        // Initialize VR/XR system
        let vr_config = VRConfig::default();
        let vr_manager = VRManager::new(
            vr_config,
            ai_manager.clone(),
            avatar_manager.clone(),
            monitoring.get_metrics_collector(),
            database_manager.clone(),
        ).await?;
        
        // Initialize Phase 33A: Universal Client Platform
        
        // Initialize Mobile Runtime Manager
        let mobile_config = MobileConfig::default();
        let mobile_runtime = MobileRuntimeManager::new(
            mobile_config,
            vr_manager.clone(),
            monitoring.get_metrics_collector(),
            database_manager.clone(),
        ).await?;
        
        // Initialize Progressive Web App Service Manager
        let pwa_config = PWAConfig::default();
        let pwa_service = PWAServiceManager::new(
            pwa_config,
            monitoring.get_metrics_collector(),
            database_manager.clone(),
        ).await?;
        
        // Initialize Cross-Platform Synchronization Engine
        let sync_config = SyncConfig::default();
        let sync_engine = CrossPlatformSyncEngine::new(
            sync_config,
            monitoring.get_metrics_collector(),
            database_manager.clone(),
        ).await?;
        
        // Initialize Phase 34: Enterprise Grid Federation & Scaling Platform
        
        // Initialize Grid Federation Manager
        let federation_config = FederationConfig::default();
        let grid_federation = GridFederationManager::new(
            database_manager.clone(),
            federation_config,
        );
        
        // Initialize Grid Scaling Manager
        let scaling_config = ScalingConfig::default();
        let grid_scaling = GridScalingManager::new(
            database_manager.clone(),
            monitoring.get_metrics_collector(),
            scaling_config,
        );
        
        // Initialize grid federation and scaling systems
        grid_federation.initialize().await?;
        grid_scaling.initialize().await?;
        
        // Initialize Phase 35: Advanced Analytics & Business Intelligence Platform
        
        // Initialize Analytics Manager
        let analytics_config = AnalyticsConfig::default();
        let analytics_manager = AnalyticsManager::new(
            database_manager.clone(),
            monitoring.get_metrics_collector(),
            analytics_config,
        ).await?;
        
        // Initialize analytics system
        analytics_manager.initialize().await?;
        
        // Initialize Content Management System
        let content_config = ContentManagerConfig::default();
        let content_manager = ContentManager::new(content_config).await?;
        
        // Initialize Phase 24.5: Analytics & Reporting System
        let reporting_config = ReportingManagerConfig::default();
        let reporting_manager = ReportingManager::new(
            database_manager.clone(),
            monitoring.get_metrics_collector(),
            reporting_config,
        ).await?;
        
        // Initialize community platform (optional)
        let community_platform = None; // Can be enabled later with enable_community()

        let network_manager = Arc::new(network::NetworkManager::new(
            monitoring.clone(),
            session_manager.clone(),
            region_manager.clone(),
            state_manager.clone(),
            asset_manager.clone(),
            user_account_database.clone(),
        ).await?);

        Ok(Self {
            asset_manager,
            network_manager,
            region_manager,
            session_manager,
            monitoring,
            state_manager,
            avatar_manager,
            economy_manager,
            social_manager,
            ai_manager,
            vr_manager,
            mobile_runtime,
            pwa_service,
            sync_engine,
            grid_federation: Arc::new(grid_federation),
            grid_scaling: Arc::new(grid_scaling),
            analytics_manager: Arc::new(analytics_manager),
            content_manager: Arc::new(content_manager),
            reporting_manager: Arc::new(reporting_manager),
            community_platform,
        })
    }
    
    /// Enable community platform features
    pub async fn enable_community(&mut self, config: community::CommunityConfig) -> Result<()> {
        let platform = Arc::new(community::CommunityPlatform::new(config).await?);
        platform.initialize().await?;
        self.community_platform = Some(platform);
        Ok(())
    }
    
    /// Get avatar manager
    pub fn avatar_manager(&self) -> Arc<AdvancedAvatarManager> {
        self.avatar_manager.clone()
    }
    
    /// Create avatar for user
    pub async fn create_avatar(
        &self,
        user_id: uuid::Uuid,
        name: String,
        initial_appearance: Option<avatar::AvatarAppearance>,
    ) -> Result<avatar::EnhancedAvatar> {
        self.avatar_manager.create_avatar(user_id, name, initial_appearance).await
            .map_err(|e| anyhow::anyhow!("Avatar creation failed: {}", e))
    }
    
    /// Get avatar by ID
    pub async fn get_avatar(&self, avatar_id: uuid::Uuid) -> Result<avatar::EnhancedAvatar> {
        self.avatar_manager.get_avatar(avatar_id).await
            .map_err(|e| anyhow::anyhow!("Avatar retrieval failed: {}", e))
    }
    
    /// Get active avatars
    pub async fn get_active_avatars(&self) -> Vec<avatar::EnhancedAvatar> {
        self.avatar_manager.get_active_avatars().await
    }
    
    /// Get economy manager
    pub fn economy_manager(&self) -> Arc<VirtualEconomyManager> {
        self.economy_manager.clone()
    }
    
    /// Create user economy account
    pub async fn create_user_economy_account(
        &self,
        user_id: uuid::Uuid,
        initial_balance: Option<i64>,
    ) -> Result<()> {
        self.economy_manager.create_user_account(user_id, initial_balance).await
            .map_err(|e| anyhow::anyhow!("Economy account creation failed: {}", e))
    }
    
    /// Transfer currency between users
    pub async fn transfer_currency(
        &self,
        from_user_id: uuid::Uuid,
        to_user_id: uuid::Uuid,
        amount: i64,
        currency_code: String,
        description: String,
    ) -> Result<economy::transactions::TransactionResult> {
        self.economy_manager.transfer_currency(from_user_id, to_user_id, amount, currency_code, description).await
            .map_err(|e| anyhow::anyhow!("Currency transfer failed: {}", e))
    }
    
    /// Get user's economy summary
    pub async fn get_user_economy_summary(&self, user_id: uuid::Uuid) -> Result<economy::manager::UserEconomySummary> {
        self.economy_manager.get_user_economy_summary(user_id).await
            .map_err(|e| anyhow::anyhow!("Economy summary retrieval failed: {}", e))
    }
    
    /// Get economy system health
    pub async fn get_economy_health(&self) -> economy::manager::EconomySystemHealth {
        self.economy_manager.get_system_health().await
    }
    
    /// Get social features manager
    pub fn social_manager(&self) -> Arc<SocialFeaturesManager> {
        self.social_manager.clone()
    }
    
    /// Create user social profile
    pub async fn create_user_social_profile(
        &self,
        user_id: uuid::Uuid,
        display_name: String,
    ) -> Result<social::UserSocialProfile> {
        self.social_manager.create_user_social_profile(user_id, display_name).await
            .map_err(|e| anyhow::anyhow!("Social profile creation failed: {}", e))
    }
    
    /// Update user online status
    pub async fn update_user_online_status(
        &self,
        user_id: uuid::Uuid,
        status: social::OnlineStatus,
    ) -> Result<()> {
        self.social_manager.update_user_online_status(user_id, status).await
            .map_err(|e| anyhow::anyhow!("Online status update failed: {}", e))
    }
    
    /// Send friend request
    pub async fn send_friend_request(
        &self,
        requester_id: uuid::Uuid,
        target_id: uuid::Uuid,
        message: Option<String>,
    ) -> Result<social::friends::FriendRequest> {
        self.social_manager.friend_system().send_friend_request(requester_id, target_id, message).await
            .map_err(|e| anyhow::anyhow!("Friend request failed: {}", e))
    }
    
    /// Get user's friend list
    pub async fn get_user_friends(&self, user_id: uuid::Uuid) -> Result<social::friends::FriendListResponse> {
        self.social_manager.friend_system().get_friend_list(user_id).await
            .map_err(|e| anyhow::anyhow!("Friend list retrieval failed: {}", e))
    }
    
    /// Send message
    pub async fn send_message(
        &self,
        sender_id: uuid::Uuid,
        request: social::messaging::SendMessageRequest,
    ) -> Result<social::messaging::Message> {
        self.social_manager.messaging_system().send_message(sender_id, request).await
            .map_err(|e| anyhow::anyhow!("Message sending failed: {}", e))
    }
    
    /// Get user conversations
    pub async fn get_user_conversations(&self, user_id: uuid::Uuid) -> Result<social::messaging::ConversationListResponse> {
        self.social_manager.messaging_system().get_user_conversations(user_id).await
            .map_err(|e| anyhow::anyhow!("Conversation retrieval failed: {}", e))
    }
    
    /// Create group
    pub async fn create_group(
        &self,
        owner_id: uuid::Uuid,
        request: social::groups::CreateGroupRequest,
    ) -> Result<social::groups::Group> {
        self.social_manager.group_system().create_group(owner_id, request).await
            .map_err(|e| anyhow::anyhow!("Group creation failed: {}", e))
    }
    
    /// Get social system health
    pub async fn get_social_system_health(&self) -> social::manager::SocialSystemHealth {
        self.social_manager.get_social_system_health().await
    }
    
    /// Get AI manager
    pub fn ai_manager(&self) -> Arc<AIManager> {
        self.ai_manager.clone()
    }
    
    /// Process AI avatar interaction
    pub async fn process_ai_avatar_interaction(
        &self,
        avatar_id: uuid::Uuid,
        interaction_data: &str,
    ) -> Result<ai::AIResponse> {
        self.ai_manager.process_avatar_ai_interaction(avatar_id, interaction_data).await
            .map_err(|e| anyhow::anyhow!("AI avatar interaction failed: {}", e))
    }
    
    /// Get AI performance recommendations
    pub async fn get_ai_performance_recommendations(&self) -> Result<Vec<ai::PerformanceRecommendation>> {
        self.ai_manager.get_performance_recommendations().await
            .map_err(|e| anyhow::anyhow!("AI performance recommendations failed: {}", e))
    }
    
    /// Generate NPC behavior
    pub async fn generate_npc_behavior(
        &self,
        npc_id: uuid::Uuid,
        context: &ai::NPCContext,
    ) -> Result<ai::NPCBehaviorPlan> {
        self.ai_manager.generate_npc_behavior(npc_id, context).await
            .map_err(|e| anyhow::anyhow!("NPC behavior generation failed: {}", e))
    }
    
    /// Generate AI content
    pub async fn generate_ai_content(
        &self,
        content_type: ai::ContentType,
        parameters: ai::ContentParameters,
    ) -> Result<ai::GeneratedContent> {
        self.ai_manager.generate_content(content_type, parameters).await
            .map_err(|e| anyhow::anyhow!("AI content generation failed: {}", e))
    }
    
    /// Predict user behavior using AI
    pub async fn predict_user_behavior(&self, user_id: uuid::Uuid) -> Result<ai::UserBehaviorPrediction> {
        self.ai_manager.predict_user_behavior(user_id).await
            .map_err(|e| anyhow::anyhow!("User behavior prediction failed: {}", e))
    }
    
    /// Get AI system health
    pub async fn get_ai_system_health(&self) -> ai::AIHealthStatus {
        self.ai_manager.get_ai_health_status().await
    }
    
    /// Get VR manager
    pub fn vr_manager(&self) -> Arc<VRManager> {
        self.vr_manager.clone()
    }
    
    /// Start VR session
    pub async fn start_vr_session(
        &self,
        user_id: uuid::Uuid,
        device_info: vr::VRDeviceInfo,
    ) -> Result<vr::VRSession> {
        self.vr_manager.start_vr_session(user_id, device_info).await
            .map_err(|e| anyhow::anyhow!("VR session start failed: {}", e))
    }
    
    /// Update VR frame
    pub async fn update_vr_frame(
        &self,
        session_id: uuid::Uuid,
        frame_data: vr::VRFrameData,
    ) -> Result<vr::VRFrameResponse> {
        self.vr_manager.update_vr_frame(session_id, frame_data).await
            .map_err(|e| anyhow::anyhow!("VR frame update failed: {}", e))
    }
    
    /// End VR session
    pub async fn end_vr_session(&self, session_id: uuid::Uuid) -> Result<()> {
        self.vr_manager.end_vr_session(session_id).await
            .map_err(|e| anyhow::anyhow!("VR session end failed: {}", e))
    }
    
    /// Get VR system health
    pub async fn get_vr_system_health(&self) -> vr::VRHealthStatus {
        self.vr_manager.get_vr_health_status().await
    }

    // Phase 33A: Universal Client Platform API Methods

    /// Create mobile session
    pub async fn create_mobile_session(
        &self,
        user_id: uuid::Uuid,
        platform: mobile::MobilePlatform,
        device_info: mobile::DeviceInfo,
    ) -> Result<uuid::Uuid> {
        self.mobile_runtime.create_mobile_session(user_id, platform, device_info).await
            .map_err(|e| anyhow::anyhow!("Mobile session creation failed: {}", e))
    }

    /// Enable mobile VR mode
    pub async fn enable_mobile_vr(&self, session_id: uuid::Uuid) -> Result<()> {
        self.mobile_runtime.enable_vr_mode(session_id).await
            .map_err(|e| anyhow::anyhow!("Mobile VR enable failed: {}", e))
    }

    /// Create PWA session
    pub async fn create_pwa_session(
        &self,
        user_id: uuid::Uuid,
        user_agent: String,
        platform_info: pwa::PlatformInfo,
    ) -> Result<uuid::Uuid> {
        self.pwa_service.create_pwa_session(user_id, user_agent, platform_info).await
            .map_err(|e| anyhow::anyhow!("PWA session creation failed: {}", e))
    }

    /// Enable WebXR
    pub async fn enable_webxr(&self, session_id: uuid::Uuid) -> Result<()> {
        self.pwa_service.enable_webxr(session_id).await
            .map_err(|e| anyhow::anyhow!("WebXR enable failed: {}", e))
    }

    /// Install PWA
    pub async fn install_pwa(&self, session_id: uuid::Uuid) -> Result<()> {
        self.pwa_service.install_pwa(session_id).await
            .map_err(|e| anyhow::anyhow!("PWA installation failed: {}", e))
    }

    /// Send push notification
    pub async fn send_push_notification(
        &self,
        user_id: uuid::Uuid,
        notification_type: pwa::NotificationType,
        payload: pwa::PushPayload,
    ) -> Result<()> {
        self.pwa_service.send_push_notification(user_id, notification_type, payload).await
            .map_err(|e| anyhow::anyhow!("Push notification send failed: {}", e))
    }

    /// Get web app manifest
    pub async fn get_web_app_manifest(&self) -> Result<String> {
        self.pwa_service.get_web_app_manifest().await
            .map_err(|e| anyhow::anyhow!("Manifest generation failed: {}", e))
    }

    /// Get service worker script
    pub async fn get_service_worker_script(&self) -> Result<String> {
        self.pwa_service.get_service_worker_script().await
            .map_err(|e| anyhow::anyhow!("Service worker generation failed: {}", e))
    }

    /// Create sync session
    pub async fn create_sync_session(
        &self,
        user_id: uuid::Uuid,
        client_platform: sync::ClientPlatform,
        device_id: String,
        sync_capabilities: sync::SyncCapabilities,
    ) -> Result<uuid::Uuid> {
        self.sync_engine.create_sync_session(user_id, client_platform, device_id, sync_capabilities).await
            .map_err(|e| anyhow::anyhow!("Sync session creation failed: {}", e))
    }

    /// Sync data
    pub async fn sync_data(
        &self,
        session_id: uuid::Uuid,
        data_type: sync::DataType,
        sync_direction: sync::SyncDirection,
    ) -> Result<sync::SyncResult> {
        self.sync_engine.sync_data(session_id, data_type, sync_direction).await
            .map_err(|e| anyhow::anyhow!("Data sync failed: {}", e))
    }

    /// Enable offline sync
    pub async fn enable_offline_sync(&self, session_id: uuid::Uuid) -> Result<()> {
        self.sync_engine.enable_offline_sync(session_id).await
            .map_err(|e| anyhow::anyhow!("Offline sync enable failed: {}", e))
    }

    /// Get sync status
    pub async fn get_sync_status(&self, session_id: uuid::Uuid) -> Result<sync::SyncSession> {
        self.sync_engine.get_sync_status(session_id).await
            .map_err(|e| anyhow::anyhow!("Sync status retrieval failed: {}", e))
    }

    /// Resolve sync conflict
    pub async fn resolve_sync_conflict(
        &self,
        conflict_id: uuid::Uuid,
        resolution: sync::ConflictStrategy,
    ) -> Result<()> {
        self.sync_engine.resolve_conflict(conflict_id, resolution).await
            .map_err(|e| anyhow::anyhow!("Conflict resolution failed: {}", e))
    }

    // Phase 34: Enterprise Grid Federation & Scaling Platform API Methods

    /// Register grid in federation
    pub async fn register_grid_in_federation(
        &self,
        grid: grid::EnterpriseGrid,
    ) -> Result<()> {
        self.grid_federation.register_grid(grid).await
            .map_err(|e| anyhow::anyhow!("Grid registration failed: {}", e))
    }

    /// Create federation request
    pub async fn create_federation_request(
        &self,
        target_grid_id: uuid::Uuid,
        requesting_grid_id: uuid::Uuid,
        trust_level: grid::TrustLevel,
        permissions: Vec<grid::TrustPermission>,
        services: Vec<grid::SharedServiceType>,
        message: Option<String>,
    ) -> Result<grid::FederationRequest> {
        self.grid_federation.create_federation_request(
            target_grid_id,
            requesting_grid_id,
            trust_level,
            permissions,
            services,
            message,
        ).await
        .map_err(|e| anyhow::anyhow!("Federation request failed: {}", e))
    }

    /// List federated grids
    pub async fn list_federated_grids(&self) -> Result<Vec<grid::EnterpriseGrid>> {
        self.grid_federation.list_federated_grids().await
            .map_err(|e| anyhow::anyhow!("Failed to list federated grids: {}", e))
    }

    /// Get trust relationship between grids
    pub async fn get_trust_relationship(
        &self,
        source_grid_id: uuid::Uuid,
        target_grid_id: uuid::Uuid,
    ) -> Result<Option<grid::TrustRelationship>> {
        self.grid_federation.get_trust_relationship(source_grid_id, target_grid_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get trust relationship: {}", e))
    }

    /// Get federation statistics
    pub async fn get_federation_statistics(&self, grid_id: uuid::Uuid) -> Result<grid::FederationStatistics> {
        self.grid_federation.get_federation_statistics(grid_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get federation statistics: {}", e))
    }

    /// Set scaling policy for grid
    pub async fn set_grid_scaling_policy(
        &self,
        grid_id: uuid::Uuid,
        policy: grid::ScalingPolicy,
    ) -> Result<()> {
        self.grid_scaling.set_scaling_policy(grid_id, policy).await
            .map_err(|e| anyhow::anyhow!("Failed to set scaling policy: {}", e))
    }

    /// Trigger manual scaling operation
    pub async fn trigger_manual_scaling(
        &self,
        grid_id: uuid::Uuid,
        operation_type: grid::ScalingOperationType,
        reason: String,
    ) -> Result<uuid::Uuid> {
        self.grid_scaling.trigger_manual_scaling(grid_id, operation_type, reason).await
            .map_err(|e| anyhow::anyhow!("Failed to trigger scaling: {}", e))
    }

    /// Get scaling recommendations
    pub async fn get_scaling_recommendations(&self, grid_id: uuid::Uuid) -> Result<Vec<grid::ScalingRecommendation>> {
        self.grid_scaling.get_scaling_recommendations(grid_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get scaling recommendations: {}", e))
    }

    /// Get scaling operation status
    pub async fn get_scaling_operation_status(&self, operation_id: uuid::Uuid) -> Result<grid::ScalingOperation> {
        self.grid_scaling.get_scaling_operation_status(operation_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get scaling operation status: {}", e))
    }

    /// Cancel scaling operation
    pub async fn cancel_scaling_operation(&self, operation_id: uuid::Uuid) -> Result<()> {
        self.grid_scaling.cancel_scaling_operation(operation_id).await
            .map_err(|e| anyhow::anyhow!("Failed to cancel scaling operation: {}", e))
    }

    /// Get scaling history
    pub async fn get_scaling_history(&self, grid_id: uuid::Uuid, limit: Option<u32>) -> Result<Vec<grid::ScalingEvent>> {
        self.grid_scaling.get_scaling_history(grid_id, limit).await
            .map_err(|e| anyhow::anyhow!("Failed to get scaling history: {}", e))
    }

    /// Get current grid capacity
    pub async fn get_current_grid_capacity(&self, grid_id: uuid::Uuid) -> Result<grid::GridCapacity> {
        self.grid_scaling.get_current_capacity(grid_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get grid capacity: {}", e))
    }

    // Phase 35: Advanced Analytics & Business Intelligence Platform API Methods

    /// Get analytics manager
    pub fn analytics_manager(&self) -> Arc<AnalyticsManager> {
        self.analytics_manager.clone()
    }

    /// Collect analytics data point
    pub async fn collect_analytics_data(&self, data_point: analytics::AnalyticsDataPoint) -> Result<()> {
        self.analytics_manager.collect_data_point(data_point).await
            .map_err(|e| anyhow::anyhow!("Failed to collect analytics data: {}", e))
    }

    /// Process real-time analytics event
    pub async fn process_real_time_event(&self, event: analytics::RealTimeEvent) -> Result<()> {
        self.analytics_manager.process_real_time_event(event).await
            .map_err(|e| anyhow::anyhow!("Failed to process real-time event: {}", e))
    }

    /// Generate business intelligence insights
    pub async fn generate_analytics_insights(&self, time_period: analytics::TimePeriod) -> Result<Vec<analytics::AnalyticsInsight>> {
        self.analytics_manager.generate_insights(time_period).await
            .map_err(|e| anyhow::anyhow!("Failed to generate insights: {}", e))
    }

    /// Get business KPIs
    pub async fn get_business_kpis(&self, category: Option<analytics::KPICategory>) -> Result<Vec<analytics::BusinessKPI>> {
        self.analytics_manager.get_business_kpis(category).await
            .map_err(|e| anyhow::anyhow!("Failed to get business KPIs: {}", e))
    }


    /// Generate analytics report
    pub async fn generate_analytics_report(&self, report_request: analytics::ReportRequest) -> Result<analytics::Report> {
        self.analytics_manager.generate_report(report_request).await
            .map_err(|e| anyhow::anyhow!("Failed to generate report: {}", e))
    }

    /// Get dashboard data
    pub async fn get_analytics_dashboard(&self, dashboard_id: uuid::Uuid) -> Result<analytics::DashboardData> {
        self.analytics_manager.get_dashboard_data(dashboard_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get dashboard data: {}", e))
    }

    /// Export analytics data
    pub async fn export_analytics_data(&self, export_request: analytics::ExportRequest) -> Result<analytics::ExportResult> {
        self.analytics_manager.export_data(export_request).await
            .map_err(|e| anyhow::anyhow!("Failed to export data: {}", e))
    }

    /// Get analytics system health
    pub async fn get_analytics_system_health(&self) -> analytics::AnalyticsSystemHealth {
        self.analytics_manager.get_system_health().await
    }
    
    // Phase 24.5: Analytics & Reporting System API Methods
    
    /// Get reporting manager
    pub fn reporting_manager(&self) -> Arc<ReportingManager> {
        self.reporting_manager.clone()
    }
    
    /// Process comprehensive analytics request
    pub async fn process_analytics_request(&self, request: reporting::manager::AnalyticsRequest) -> Result<reporting::manager::AnalyticsResponse> {
        self.reporting_manager.process_analytics_request(request).await
            .map_err(|e| anyhow::anyhow!("Analytics request failed: {}", e))
    }
    
    /// Collect analytics data point
    pub async fn collect_analytics_data_point(&self, data_point: reporting::AnalyticsDataPoint) -> Result<()> {
        self.reporting_manager.collect_data_point(data_point).await
            .map_err(|e| anyhow::anyhow!("Data collection failed: {}", e))
    }
    
    /// Process real-time analytics event
    pub async fn process_real_time_analytics_event(&self, event: reporting::data_collection::RealTimeEvent) -> Result<()> {
        self.reporting_manager.process_real_time_event(event).await
            .map_err(|e| anyhow::anyhow!("Real-time event processing failed: {}", e))
    }
    
    /// Generate predictive forecast
    pub async fn generate_predictive_forecast(
        &self,
        metric_name: String,
        forecast_horizon: chrono::Duration,
        confidence_levels: Vec<f32>,
    ) -> Result<reporting::ForecastResult> {
        self.reporting_manager.generate_forecast(metric_name, forecast_horizon, confidence_levels).await
            .map_err(|e| anyhow::anyhow!("Forecast generation failed: {}", e))
    }
    
    /// Get business KPIs for reporting
    pub async fn get_reporting_business_kpis(&self, category: Option<reporting::KPICategory>) -> Result<Vec<reporting::BusinessKPI>> {
        self.reporting_manager.get_business_kpis(category).await
            .map_err(|e| anyhow::anyhow!("KPI retrieval failed: {}", e))
    }
    
    /// Generate business report
    pub async fn generate_business_report(&self, report_request: reporting::report_generation::ReportRequest) -> Result<uuid::Uuid> {
        self.reporting_manager.generate_report(report_request).await
            .map_err(|e| anyhow::anyhow!("Report generation failed: {}", e))
    }
    
    /// Get business dashboard data
    pub async fn get_business_dashboard_data(&self, dashboard_id: uuid::Uuid) -> Result<reporting::business_intelligence::DashboardData> {
        self.reporting_manager.get_dashboard_data(dashboard_id).await
            .map_err(|e| anyhow::anyhow!("Dashboard data retrieval failed: {}", e))
    }
    
    /// Export analytics data
    pub async fn export_reporting_data(&self, export_request: reporting::manager::ExportRequest) -> Result<reporting::manager::ExportResult> {
        self.reporting_manager.export_data(export_request).await
            .map_err(|e| anyhow::anyhow!("Data export failed: {}", e))
    }
    
    /// Get reporting system health
    pub async fn get_reporting_system_health(&self) -> reporting::manager::ReportingSystemHealth {
        self.reporting_manager.get_system_health().await
    }
    
    /// Get analytics collection statistics
    pub async fn get_analytics_collection_statistics(&self) -> Result<reporting::data_collection::CollectionStatistics> {
        self.reporting_manager.get_collection_statistics().await
            .map_err(|e| anyhow::anyhow!("Collection statistics retrieval failed: {}", e))
    }

    /// Get content manager
    pub fn content_manager(&self) -> Arc<ContentManager> {
        self.content_manager.clone()
    }

    /// Create new content
    pub async fn create_content(
        &self,
        creator_id: uuid::Uuid,
        content_name: String,
        content_type: content::ContentType,
        source_file: std::path::PathBuf,
        permissions: Option<content::ContentPermissions>,
        options: content::creation::ContentCreationOptions,
    ) -> content::ContentResult<uuid::Uuid> {
        // Note: This is a simplified API - in production would use proper Arc<RwLock<>> access
        Err(content::ContentError::ImportFailed { 
            reason: "Content creation requires mutable access - use content_manager() directly".to_string() 
        })
    }

    /// Search content
    pub async fn search_content(
        &self,
        filter: content::ContentSearchFilter,
        page: u32,
        page_size: u32,
    ) -> content::ContentResult<content::ContentSearchResult> {
        self.content_manager.search_content(filter, page, page_size).await
    }

    /// Get content metadata
    pub async fn get_content_metadata(&self, content_id: uuid::Uuid) -> content::ContentResult<content::ContentMetadata> {
        self.content_manager.get_content_metadata(content_id).await
    }

    pub async fn start(&self, addr: &str) -> Result<()> {
        // Start network server
        self.network_manager.start(addr).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_startup() -> Result<()> {
        let server = OpenSimServer::new().await?;
        server.start("127.0.0.1:0").await?;
        Ok(())
    }
}
