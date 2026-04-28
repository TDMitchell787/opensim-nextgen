//! Capabilities system for Second Life/OpenSim compatibility
//! Handles capability URLs and services required for viewer functionality

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub mod handlers;
pub mod registry;
pub mod seed;

pub use handlers::*;
pub use registry::*;
pub use seed::*;

/// Capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Capability URL
    pub url: String,
    /// Capability version (optional)
    pub version: Option<String>,
    /// Expiration time (optional)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether this capability requires authentication
    pub requires_auth: bool,
}

impl Capability {
    /// Create a new capability
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            version: None,
            expires_at: None,
            requires_auth: true,
        }
    }

    /// Create a capability with expiration
    pub fn with_expiration(name: String, url: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            name,
            url,
            version: None,
            expires_at: Some(expires_at),
            requires_auth: true,
        }
    }

    /// Check if capability is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Set capability version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Set authentication requirement
    pub fn with_auth(mut self, requires_auth: bool) -> Self {
        self.requires_auth = requires_auth;
        self
    }
}

/// Standard Second Life capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardCapability {
    /// Seed capability (bootstrap)
    Seed,
    /// Event queue for server-to-client messages
    EventQueueGet,
    /// Upload baked textures
    UploadBakedTexture,
    /// Asset upload
    NewFileAgentInventory,
    /// Mesh upload
    NewFileAgentInventoryVariablePrice,
    /// Texture upload
    UploadTexture,
    /// Script upload
    UpdateScriptAgent,
    /// Notecard update
    UpdateNotecardAgentInventory,
    /// Gesture update
    UpdateGestureAgentInventory,
    /// Animation upload
    UpdateAnimationAgentInventory,
    /// Sound upload
    UpdateSoundAgentInventory,
    /// Agent preferences
    AgentPreferences,
    /// Avatar picker search
    AvatarPickerSearch,
    /// Chat session request
    ChatSessionRequest,
    /// Copy inventory from notecard
    CopyInventoryFromNotecard,
    /// Create inventory category
    CreateInventoryCategory,
    /// Dispatch region info
    DispatchRegionInfo,
    /// Environment settings
    EnvironmentSettings,
    /// Estate change info
    EstateChangeInfo,
    /// Fetch inventory
    FetchInventory2,
    /// Fetch library
    FetchLib2,
    /// Get display names
    GetDisplayNames,
    /// Get experience info
    GetExperienceInfo,
    /// Get mesh
    GetMesh,
    /// Get mesh 2
    GetMesh2,
    /// Get metadata
    GetMetadata,
    /// Get object cost
    GetObjectCost,
    /// Get object physics data
    GetObjectPhysicsData,
    /// Get texture
    GetTexture,
    /// Group member data
    GroupMemberData,
    /// Home location
    HomeLocation,
    /// Land resources
    LandResources,
    /// Map layer
    MapLayer,
    /// Map layer GOD
    MapLayerGod,
    /// New file agent inventory
    NewAgentInventory,
    /// Object media
    ObjectMedia,
    /// Object media navigate
    ObjectMediaNavigate,
    /// Object navigation
    ObjectNavigation,
    /// Parcel properties update
    ParcelPropertiesUpdate,
    /// Product info request
    ProductInfoRequest,
    /// Provision voice account
    ProvisionVoiceAccountRequest,
    /// Region objects
    RegionObjects,
    /// Remote parcel request
    RemoteParcelRequest,
    /// Render materials
    RenderMaterials,
    /// Request texture download
    RequestTextureDownload,
    /// Resource cost selected
    ResourceCostSelected,
    /// Search static objects
    SearchStaticObjects,
    /// Send post card
    SendPostcard,
    /// Send user report
    SendUserReport,
    /// Send user report with screenshot
    SendUserReportWithScreenshot,
    /// Server release notes
    ServerReleaseNotes,
    /// Set display name
    SetDisplayName,
    /// Simulate agent movement
    SimulatorFeatures,
    /// Start group proposal
    StartGroupProposal,
    /// Teleport location request
    TeleportLocationRequest,
    /// Texture stats
    TextureStats,
    /// Untrusted simulate agent movement
    UntrustedSimulatorMessage,
    /// Update agent information
    UpdateAgentInformation,
    /// Update agent language
    UpdateAgentLanguage,
    /// Upload asset
    UploadAsset,
    /// View admin options
    ViewerAsset,
    /// Viewer metrics
    ViewerMetrics,
    /// Viewer startup
    ViewerStartAuction,
    /// Viewer stats
    ViewerStats,
    /// Voice server info
    VoiceServerInfo,
    /// Web fetch inventory descendants
    WebFetchInventoryDescendents,
}

impl StandardCapability {
    /// Get the capability name as used in Second Life
    pub fn name(&self) -> &'static str {
        match self {
            Self::Seed => "seed_capability",
            Self::EventQueueGet => "EventQueueGet",
            Self::UploadBakedTexture => "UploadBakedTexture",
            Self::NewFileAgentInventory => "NewFileAgentInventory",
            Self::NewFileAgentInventoryVariablePrice => "NewFileAgentInventoryVariablePrice",
            Self::UploadTexture => "UploadTexture",
            Self::UpdateScriptAgent => "UpdateScriptAgent",
            Self::UpdateNotecardAgentInventory => "UpdateNotecardAgentInventory",
            Self::UpdateGestureAgentInventory => "UpdateGestureAgentInventory",
            Self::UpdateAnimationAgentInventory => "UpdateAnimationAgentInventory",
            Self::UpdateSoundAgentInventory => "UpdateSoundAgentInventory",
            Self::AgentPreferences => "AgentPreferences",
            Self::AvatarPickerSearch => "AvatarPickerSearch",
            Self::ChatSessionRequest => "ChatSessionRequest",
            Self::CopyInventoryFromNotecard => "CopyInventoryFromNotecard",
            Self::CreateInventoryCategory => "CreateInventoryCategory",
            Self::DispatchRegionInfo => "DispatchRegionInfo",
            Self::EnvironmentSettings => "EnvironmentSettings",
            Self::EstateChangeInfo => "EstateChangeInfo",
            Self::FetchInventory2 => "FetchInventory2",
            Self::FetchLib2 => "FetchLib2",
            Self::GetDisplayNames => "GetDisplayNames",
            Self::GetExperienceInfo => "GetExperienceInfo",
            Self::GetMesh => "GetMesh",
            Self::GetMesh2 => "GetMesh2",
            Self::GetMetadata => "GetMetadata",
            Self::GetObjectCost => "GetObjectCost",
            Self::GetObjectPhysicsData => "GetObjectPhysicsData",
            Self::GetTexture => "GetTexture",
            Self::GroupMemberData => "GroupMemberData",
            Self::HomeLocation => "HomeLocation",
            Self::LandResources => "LandResources",
            Self::MapLayer => "MapLayer",
            Self::MapLayerGod => "MapLayerGod",
            Self::NewAgentInventory => "NewAgentInventory",
            Self::ObjectMedia => "ObjectMedia",
            Self::ObjectMediaNavigate => "ObjectMediaNavigate",
            Self::ObjectNavigation => "ObjectNavigation",
            Self::ParcelPropertiesUpdate => "ParcelPropertiesUpdate",
            Self::ProductInfoRequest => "ProductInfoRequest",
            Self::ProvisionVoiceAccountRequest => "ProvisionVoiceAccountRequest",
            Self::RegionObjects => "RegionObjects",
            Self::RemoteParcelRequest => "RemoteParcelRequest",
            Self::RenderMaterials => "RenderMaterials",
            Self::RequestTextureDownload => "RequestTextureDownload",
            Self::ResourceCostSelected => "ResourceCostSelected",
            Self::SearchStaticObjects => "SearchStaticObjects",
            Self::SendPostcard => "SendPostcard",
            Self::SendUserReport => "SendUserReport",
            Self::SendUserReportWithScreenshot => "SendUserReportWithScreenshot",
            Self::ServerReleaseNotes => "ServerReleaseNotes",
            Self::SetDisplayName => "SetDisplayName",
            Self::SimulatorFeatures => "SimulatorFeatures",
            Self::StartGroupProposal => "StartGroupProposal",
            Self::TeleportLocationRequest => "TeleportLocationRequest",
            Self::TextureStats => "TextureStats",
            Self::UntrustedSimulatorMessage => "UntrustedSimulatorMessage",
            Self::UpdateAgentInformation => "UpdateAgentInformation",
            Self::UpdateAgentLanguage => "UpdateAgentLanguage",
            Self::UploadAsset => "UploadAsset",
            Self::ViewerAsset => "ViewerAsset",
            Self::ViewerMetrics => "ViewerMetrics",
            Self::ViewerStartAuction => "ViewerStartAuction",
            Self::ViewerStats => "ViewerStats",
            Self::VoiceServerInfo => "VoiceServerInfo",
            Self::WebFetchInventoryDescendents => "WebFetchInventoryDescendents",
        }
    }

    /// Get essential capabilities needed for basic viewer functionality
    pub fn essential_capabilities() -> Vec<StandardCapability> {
        vec![
            Self::Seed,
            Self::EventQueueGet,
            Self::FetchInventory2,
            Self::WebFetchInventoryDescendents,
            Self::SimulatorFeatures,
            Self::GetTexture,
            Self::GetMesh,
            Self::UpdateAgentInformation,
            Self::AgentPreferences,
        ]
    }

    /// Get upload capabilities for content creation
    pub fn upload_capabilities() -> Vec<StandardCapability> {
        vec![
            Self::NewFileAgentInventory,
            Self::NewFileAgentInventoryVariablePrice,
            Self::UploadTexture,
            Self::UploadBakedTexture,
            Self::UpdateScriptAgent,
            Self::UpdateNotecardAgentInventory,
            Self::UpdateGestureAgentInventory,
            Self::UpdateAnimationAgentInventory,
            Self::UpdateSoundAgentInventory,
        ]
    }

    /// Check if this capability is essential for viewer operation
    pub fn is_essential(&self) -> bool {
        Self::essential_capabilities().contains(self)
    }

    /// Check if this capability is for uploading content
    pub fn is_upload_capability(&self) -> bool {
        Self::upload_capabilities().contains(self)
    }
}

/// Capabilities configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesConfig {
    /// Base URL for capabilities
    pub base_url: String,
    /// Capability timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of capabilities per agent
    pub max_capabilities: usize,
    /// Enable essential capabilities
    pub enable_essential: bool,
    /// Enable upload capabilities
    pub enable_uploads: bool,
    /// Enable advanced capabilities
    pub enable_advanced: bool,
    /// Custom capability handlers
    pub custom_handlers: HashMap<String, String>,
}

impl Default for CapabilitiesConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:9000/cap".to_string(),
            timeout_seconds: 300, // 5 minutes
            max_capabilities: 100,
            enable_essential: true,
            enable_uploads: true,
            enable_advanced: false,
            custom_handlers: HashMap::new(),
        }
    }
}

/// Agent capabilities set
#[derive(Debug, Clone)]
pub struct AgentCapabilities {
    /// Agent ID
    pub agent_id: Uuid,
    /// Session ID
    pub session_id: Uuid,
    /// Capabilities map
    pub capabilities: HashMap<String, Capability>,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last accessed time
    pub last_accessed: DateTime<Utc>,
    /// Seed capability URL
    pub seed_url: String,
}

impl AgentCapabilities {
    /// Create new agent capabilities
    pub fn new(agent_id: Uuid, session_id: Uuid, base_url: &str) -> Self {
        let seed_cap_id = Uuid::new_v4();
        let seed_url = format!("{}/{}", base_url, seed_cap_id);

        Self {
            agent_id,
            session_id,
            capabilities: HashMap::new(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            seed_url,
        }
    }

    /// Add a capability
    pub fn add_capability(&mut self, capability: Capability) {
        self.capabilities
            .insert(capability.name.clone(), capability);
        self.last_accessed = Utc::now();
    }

    /// Get a capability by name
    pub fn get_capability(&self, name: &str) -> Option<&Capability> {
        self.capabilities.get(name)
    }

    /// Remove expired capabilities
    pub fn remove_expired(&mut self) -> usize {
        let initial_count = self.capabilities.len();
        self.capabilities.retain(|_, cap| !cap.is_expired());
        initial_count - self.capabilities.len()
    }

    /// Get all capability names
    pub fn capability_names(&self) -> Vec<String> {
        self.capabilities.keys().cloned().collect()
    }

    /// Get capabilities as JSON for seed capability response
    pub fn to_seed_response(&self) -> serde_json::Value {
        let mut caps = serde_json::Map::new();

        for (name, cap) in &self.capabilities {
            caps.insert(name.clone(), serde_json::Value::String(cap.url.clone()));
        }

        serde_json::Value::Object(caps)
    }

    /// Check if agent has essential capabilities
    pub fn has_essential_capabilities(&self) -> bool {
        StandardCapability::essential_capabilities()
            .iter()
            .all(|cap| self.capabilities.contains_key(cap.name()))
    }
}

/// Capabilities manager
#[derive(Debug)]
pub struct CapabilitiesManager {
    /// Agent capabilities by agent ID
    agent_capabilities: HashMap<Uuid, AgentCapabilities>,
    /// Configuration
    config: CapabilitiesConfig,
    /// Capability URL mappings
    url_mappings: HashMap<String, (Uuid, String)>, // URL -> (agent_id, capability_name)
}

impl CapabilitiesManager {
    /// Create new capabilities manager
    pub fn new(config: CapabilitiesConfig) -> Self {
        info!(
            "Initializing capabilities manager with base URL: {}",
            config.base_url
        );
        Self {
            agent_capabilities: HashMap::new(),
            config,
            url_mappings: HashMap::new(),
        }
    }

    /// Create capabilities for an agent
    pub fn create_agent_capabilities(
        &mut self,
        agent_id: Uuid,
        session_id: Uuid,
    ) -> Result<String> {
        debug!("Creating capabilities for agent: {}", agent_id);

        let mut agent_caps = AgentCapabilities::new(agent_id, session_id, &self.config.base_url);

        // Add essential capabilities
        if self.config.enable_essential {
            self.add_essential_capabilities(&mut agent_caps)?;
        }

        // Add upload capabilities
        if self.config.enable_uploads {
            self.add_upload_capabilities(&mut agent_caps)?;
        }

        // Add advanced capabilities
        if self.config.enable_advanced {
            self.add_advanced_capabilities(&mut agent_caps)?;
        }

        let seed_url = agent_caps.seed_url.clone();

        // Register URL mappings
        for (name, capability) in &agent_caps.capabilities {
            let url_path = self.extract_url_path(&capability.url);
            self.url_mappings.insert(url_path, (agent_id, name.clone()));
        }

        self.agent_capabilities.insert(agent_id, agent_caps);

        info!(
            "Created {} capabilities for agent {}",
            self.agent_capabilities
                .get(&agent_id)
                .unwrap()
                .capabilities
                .len(),
            agent_id
        );

        Ok(seed_url)
    }

    /// Get agent capabilities
    pub fn get_agent_capabilities(&self, agent_id: &Uuid) -> Option<&AgentCapabilities> {
        self.agent_capabilities.get(agent_id)
    }

    /// Get capability handler by URL path
    pub fn get_capability_by_url(&self, url_path: &str) -> Option<(Uuid, String)> {
        self.url_mappings.get(url_path).cloned()
    }

    /// Remove agent capabilities
    pub fn remove_agent_capabilities(&mut self, agent_id: &Uuid) -> bool {
        if let Some(agent_caps) = self.agent_capabilities.remove(agent_id) {
            // Remove URL mappings
            for capability in agent_caps.capabilities.values() {
                let url_path = self.extract_url_path(&capability.url);
                self.url_mappings.remove(&url_path);
            }

            debug!("Removed capabilities for agent: {}", agent_id);
            true
        } else {
            false
        }
    }

    /// Cleanup expired capabilities
    pub fn cleanup_expired(&mut self) -> usize {
        let mut total_removed = 0;

        for agent_caps in self.agent_capabilities.values_mut() {
            total_removed += agent_caps.remove_expired();
        }

        if total_removed > 0 {
            debug!("Cleaned up {} expired capabilities", total_removed);
        }

        total_removed
    }

    /// Get statistics
    pub fn get_stats(&self) -> CapabilitiesStats {
        let total_capabilities: usize = self
            .agent_capabilities
            .values()
            .map(|caps| caps.capabilities.len())
            .sum();

        CapabilitiesStats {
            total_agents: self.agent_capabilities.len(),
            total_capabilities,
            url_mappings: self.url_mappings.len(),
        }
    }

    /// Add essential capabilities
    fn add_essential_capabilities(&self, agent_caps: &mut AgentCapabilities) -> Result<()> {
        for std_cap in StandardCapability::essential_capabilities() {
            let cap_url = self.generate_capability_url(agent_caps.agent_id, std_cap.name());
            let capability = Capability::new(std_cap.name().to_string(), cap_url);
            agent_caps.add_capability(capability);
        }
        Ok(())
    }

    /// Add upload capabilities
    fn add_upload_capabilities(&self, agent_caps: &mut AgentCapabilities) -> Result<()> {
        for std_cap in StandardCapability::upload_capabilities() {
            let cap_url = self.generate_capability_url(agent_caps.agent_id, std_cap.name());
            let capability = Capability::new(std_cap.name().to_string(), cap_url);
            agent_caps.add_capability(capability);
        }
        Ok(())
    }

    /// Add advanced capabilities
    fn add_advanced_capabilities(&self, agent_caps: &mut AgentCapabilities) -> Result<()> {
        // Add additional capabilities for advanced features
        let advanced_caps = [
            StandardCapability::GetDisplayNames,
            StandardCapability::SetDisplayName,
            StandardCapability::GroupMemberData,
            StandardCapability::VoiceServerInfo,
            StandardCapability::ViewerMetrics,
            StandardCapability::ViewerStats,
        ];

        for std_cap in advanced_caps {
            let cap_url = self.generate_capability_url(agent_caps.agent_id, std_cap.name());
            let capability = Capability::new(std_cap.name().to_string(), cap_url);
            agent_caps.add_capability(capability);
        }
        Ok(())
    }

    /// Generate capability URL
    fn generate_capability_url(&self, agent_id: Uuid, capability_name: &str) -> String {
        let cap_id = Uuid::new_v4();
        format!("{}/{}/{}", self.config.base_url, agent_id, cap_id)
    }

    /// Extract URL path from full URL
    fn extract_url_path(&self, url: &str) -> String {
        if let Some(path_start) = url.find("/cap/") {
            url[path_start..].to_string()
        } else {
            url.to_string()
        }
    }
}

/// Capabilities statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesStats {
    /// Total number of agents with capabilities
    pub total_agents: usize,
    /// Total number of capabilities
    pub total_capabilities: usize,
    /// Number of URL mappings
    pub url_mappings: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new(
            "TestCapability".to_string(),
            "http://example.com/cap/test".to_string(),
        );

        assert_eq!(cap.name, "TestCapability");
        assert_eq!(cap.url, "http://example.com/cap/test");
        assert!(!cap.is_expired());
        assert!(cap.requires_auth);
    }

    #[test]
    fn test_capability_expiration() {
        let past_time = Utc::now() - chrono::Duration::minutes(5);
        let cap = Capability::with_expiration(
            "ExpiredCap".to_string(),
            "http://example.com/cap/expired".to_string(),
            past_time,
        );

        assert!(cap.is_expired());
    }

    #[test]
    fn test_standard_capability_names() {
        assert_eq!(StandardCapability::Seed.name(), "seed_capability");
        assert_eq!(StandardCapability::EventQueueGet.name(), "EventQueueGet");
        assert_eq!(StandardCapability::GetTexture.name(), "GetTexture");
    }

    #[test]
    fn test_essential_capabilities() {
        let essential = StandardCapability::essential_capabilities();
        assert!(essential.contains(&StandardCapability::Seed));
        assert!(essential.contains(&StandardCapability::EventQueueGet));
        assert!(essential.contains(&StandardCapability::FetchInventory2));
    }

    #[test]
    fn test_agent_capabilities() {
        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let mut agent_caps = AgentCapabilities::new(agent_id, session_id, "http://test.com/cap");

        let cap = Capability::new("TestCap".to_string(), "http://test.com/cap/123".to_string());
        agent_caps.add_capability(cap);

        assert_eq!(agent_caps.capabilities.len(), 1);
        assert!(agent_caps.get_capability("TestCap").is_some());
        assert!(agent_caps.get_capability("NonExistent").is_none());
    }

    #[test]
    fn test_capabilities_manager() {
        let config = CapabilitiesConfig::default();
        let mut manager = CapabilitiesManager::new(config);

        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        let seed_url = manager
            .create_agent_capabilities(agent_id, session_id)
            .unwrap();
        assert!(seed_url.contains("/cap/"));

        let agent_caps = manager.get_agent_capabilities(&agent_id).unwrap();
        assert!(agent_caps.has_essential_capabilities());

        let stats = manager.get_stats();
        assert_eq!(stats.total_agents, 1);
        assert!(stats.total_capabilities > 0);
    }

    #[test]
    fn test_seed_response() {
        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let mut agent_caps = AgentCapabilities::new(agent_id, session_id, "http://test.com/cap");

        let cap = Capability::new("TestCap".to_string(), "http://test.com/cap/123".to_string());
        agent_caps.add_capability(cap);

        let response = agent_caps.to_seed_response();
        assert!(response.is_object());

        let obj = response.as_object().unwrap();
        assert!(obj.contains_key("TestCap"));
        assert_eq!(
            obj.get("TestCap").unwrap().as_str().unwrap(),
            "http://test.com/cap/123"
        );
    }
}
