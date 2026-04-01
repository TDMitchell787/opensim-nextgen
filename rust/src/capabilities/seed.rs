//! Seed capability implementation for Second Life/OpenSim compatibility
//! The seed capability is the bootstrap capability that provides all other capabilities

use super::*;
use axum::{
    extract::{Path, State},
    response::Json,
    http::StatusCode,
};
use serde_json::Value;

/// Seed capability handler state
#[derive(Debug, Clone)]
pub struct SeedCapabilityState {
    /// Capabilities manager
    pub capabilities_manager: Arc<std::sync::Mutex<CapabilitiesManager>>,
}

impl SeedCapabilityState {
    /// Create new seed capability state
    pub fn new(capabilities_manager: Arc<std::sync::Mutex<CapabilitiesManager>>) -> Self {
        Self {
            capabilities_manager,
        }
    }
}

/// Seed capability request (usually empty POST)
#[derive(Debug, Deserialize)]
pub struct SeedCapabilityRequest {
    /// Optional capability filter
    pub capabilities: Option<Vec<String>>,
}

/// Seed capability response
#[derive(Debug, Serialize)]
pub struct SeedCapabilityResponse {
    /// Capabilities map (name -> URL)
    #[serde(flatten)]
    pub capabilities: std::collections::HashMap<String, String>,
}

/// Handle seed capability request
/// This is the main entry point for capability bootstrapping
pub async fn handle_seed_capability(
    Path(agent_id): Path<String>,
    State(state): State<SeedCapabilityState>,
    Json(request): Json<SeedCapabilityRequest>,
) -> Result<Json<Value>, StatusCode> {
    debug!("Handling seed capability request for agent: {}", agent_id);
    
    // Parse agent ID
    let agent_uuid = match Uuid::parse_str(&agent_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            warn!("Invalid agent ID in seed capability request: {} - {}", agent_id, e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Get capabilities manager
    let capabilities_manager = match state.capabilities_manager.lock() {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to lock capabilities manager: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get agent capabilities
    let agent_capabilities = match capabilities_manager.get_agent_capabilities(&agent_uuid) {
        Some(caps) => caps,
        None => {
            warn!("No capabilities found for agent: {}", agent_uuid);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    // Filter capabilities if requested
    let response = if let Some(requested_caps) = request.capabilities {
        filter_capabilities(agent_capabilities, &requested_caps)
    } else {
        agent_capabilities.to_seed_response()
    };
    
    debug!("Returning {} capabilities for agent {}", 
           response.as_object().map(|o| o.len()).unwrap_or(0), 
           agent_uuid);
    
    Ok(Json(response))
}

/// Handle seed capability with UUID path parameter
pub async fn handle_seed_capability_with_uuid(
    Path((agent_id, cap_uuid)): Path<(String, String)>,
    State(state): State<SeedCapabilityState>,
    Json(request): Json<SeedCapabilityRequest>,
) -> Result<Json<Value>, StatusCode> {
    debug!("Handling seed capability request for agent: {} with UUID: {}", agent_id, cap_uuid);
    
    // For now, we just pass through to the main handler
    // The cap_uuid is used for security/validation but we'll implement basic functionality first
    handle_seed_capability(Path(agent_id), State(state), Json(request)).await
}

/// Filter capabilities based on requested list
fn filter_capabilities(agent_caps: &AgentCapabilities, requested: &[String]) -> Value {
    let mut filtered = serde_json::Map::new();
    
    for cap_name in requested {
        if let Some(capability) = agent_caps.get_capability(cap_name) {
            filtered.insert(cap_name.clone(), Value::String(capability.url.clone()));
        }
    }
    
    Value::Object(filtered)
}

/// Generate seed capability URL for login response
pub fn generate_seed_capability_url(base_url: &str, agent_id: Uuid) -> String {
    let seed_uuid = Uuid::new_v4();
    format!("{}/{}/{}", base_url, agent_id, seed_uuid)
}

/// Seed capability service for managing seed URLs and bootstrapping
#[derive(Debug)]
pub struct SeedCapabilityService {
    /// Base URL for capabilities
    base_url: String,
    /// Active seed URLs by agent ID
    active_seeds: HashMap<Uuid, String>,
}

impl SeedCapabilityService {
    /// Create new seed capability service
    pub fn new(base_url: String) -> Self {
        info!("Initializing seed capability service with base URL: {}", base_url);
        Self {
            base_url,
            active_seeds: HashMap::new(),
        }
    }
    
    /// Generate and register seed capability URL for agent
    pub fn create_seed_capability(&mut self, agent_id: Uuid) -> String {
        let seed_url = generate_seed_capability_url(&self.base_url, agent_id);
        self.active_seeds.insert(agent_id, seed_url.clone());
        
        debug!("Created seed capability URL for agent {}: {}", agent_id, seed_url);
        seed_url
    }
    
    /// Get seed capability URL for agent
    pub fn get_seed_capability(&self, agent_id: &Uuid) -> Option<&String> {
        self.active_seeds.get(agent_id)
    }
    
    /// Remove seed capability for agent
    pub fn remove_seed_capability(&mut self, agent_id: &Uuid) -> bool {
        if self.active_seeds.remove(agent_id).is_some() {
            debug!("Removed seed capability for agent: {}", agent_id);
            true
        } else {
            false
        }
    }
    
    /// Get number of active seed capabilities
    pub fn active_count(&self) -> usize {
        self.active_seeds.len()
    }
    
    /// Clear all seed capabilities
    pub fn clear_all(&mut self) {
        let count = self.active_seeds.len();
        self.active_seeds.clear();
        
        if count > 0 {
            info!("Cleared {} seed capabilities", count);
        }
    }
}

/// Seed capability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedCapabilityConfig {
    /// Enable seed capability
    pub enabled: bool,
    /// Seed capability timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum concurrent seed requests
    pub max_concurrent_requests: usize,
    /// Require agent authentication for seed requests
    pub require_authentication: bool,
}

impl Default for SeedCapabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 300, // 5 minutes
            max_concurrent_requests: 100,
            require_authentication: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn test_generate_seed_capability_url() {
        let base_url = "http://example.com/cap";
        let agent_id = Uuid::new_v4();
        
        let seed_url = generate_seed_capability_url(base_url, agent_id);
        
        assert!(seed_url.starts_with(base_url));
        assert!(seed_url.contains(&agent_id.to_string()));
        assert!(seed_url.len() > base_url.len() + 36); // agent_id + separator + seed_uuid
    }
    
    #[test]
    fn test_seed_capability_service() {
        let mut service = SeedCapabilityService::new("http://test.com/cap".to_string());
        let agent_id = Uuid::new_v4();
        
        assert_eq!(service.active_count(), 0);
        
        let seed_url = service.create_seed_capability(agent_id);
        assert!(seed_url.contains("http://test.com/cap"));
        assert_eq!(service.active_count(), 1);
        
        let retrieved_url = service.get_seed_capability(&agent_id);
        assert_eq!(retrieved_url, Some(&seed_url));
        
        let removed = service.remove_seed_capability(&agent_id);
        assert!(removed);
        assert_eq!(service.active_count(), 0);
    }
    
    #[test]
    fn test_filter_capabilities() {
        let agent_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let mut agent_caps = AgentCapabilities::new(agent_id, session_id, "http://test.com/cap");
        
        // Add some capabilities
        agent_caps.add_capability(Capability::new("Cap1".to_string(), "http://test.com/cap/1".to_string()));
        agent_caps.add_capability(Capability::new("Cap2".to_string(), "http://test.com/cap/2".to_string()));
        agent_caps.add_capability(Capability::new("Cap3".to_string(), "http://test.com/cap/3".to_string()));
        
        // Filter to only Cap1 and Cap3
        let requested = vec!["Cap1".to_string(), "Cap3".to_string(), "NonExistent".to_string()];
        let filtered = filter_capabilities(&agent_caps, &requested);
        
        let obj = filtered.as_object().unwrap();
        assert_eq!(obj.len(), 2); // Only Cap1 and Cap3 should be present
        assert!(obj.contains_key("Cap1"));
        assert!(obj.contains_key("Cap3"));
        assert!(!obj.contains_key("Cap2"));
        assert!(!obj.contains_key("NonExistent"));
    }
    
    #[test]
    fn test_seed_capability_config() {
        let config = SeedCapabilityConfig::default();
        
        assert!(config.enabled);
        assert_eq!(config.timeout_seconds, 300);
        assert_eq!(config.max_concurrent_requests, 100);
        assert!(config.require_authentication);
    }
    
    #[tokio::test]
    async fn test_seed_capability_request_parsing() {
        // Test empty request
        let empty_request = SeedCapabilityRequest { capabilities: None };
        assert!(empty_request.capabilities.is_none());
        
        // Test request with capabilities filter
        let filtered_request = SeedCapabilityRequest {
            capabilities: Some(vec!["EventQueueGet".to_string(), "GetTexture".to_string()]),
        };
        assert!(filtered_request.capabilities.is_some());
        assert_eq!(filtered_request.capabilities.unwrap().len(), 2);
    }
}