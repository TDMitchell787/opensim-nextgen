//! Capability handlers for Second Life/OpenSim protocol compatibility
//! Implements handlers for various capabilities required by viewers

use super::*;
use axum::{
    extract::{Path, State, Query},
    response::Json,
    http::StatusCode,
};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Generic capability handler state
#[derive(Debug, Clone)]
pub struct CapabilityHandlerState {
    /// Capabilities manager
    pub capabilities_manager: Arc<std::sync::Mutex<CapabilitiesManager>>,
    /// Base URL for responses
    pub base_url: String,
}

impl CapabilityHandlerState {
    /// Create new capability handler state
    pub fn new(
        capabilities_manager: Arc<std::sync::Mutex<CapabilitiesManager>>,
        base_url: String,
    ) -> Self {
        Self {
            capabilities_manager,
            base_url,
        }
    }
}

/// Event Queue Get capability handler
/// This is a long-polling endpoint for server-to-client messages
pub async fn handle_event_queue_get(
    Path((agent_id, cap_id)): Path<(String, String)>,
    State(state): State<CapabilityHandlerState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    debug!("EventQueueGet request for agent: {} cap: {}", agent_id, cap_id);
    
    // Parse agent ID
    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate agent has this capability
    {
        let manager = state.capabilities_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if manager.get_agent_capabilities(&agent_uuid).is_none() {
            return Err(StatusCode::NOT_FOUND);
        }
    }
    
    // Get acknowledge parameter for message cleanup
    let ack = params.get("ack").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    
    // For now, return empty event queue response
    // In a full implementation, this would maintain a message queue per agent
    let response = json!({
        "id": 1,
        "events": []
    });
    
    debug!("EventQueueGet response for agent {}: empty queue", agent_uuid);
    Ok(Json(response))
}

/// Simulator Features capability handler
/// Provides information about simulator capabilities and features
pub async fn handle_simulator_features(
    Path((agent_id, cap_id)): Path<(String, String)>,
    State(state): State<CapabilityHandlerState>,
) -> Result<Json<Value>, StatusCode> {
    debug!("SimulatorFeatures request for agent: {} cap: {}", agent_id, cap_id);
    
    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate agent capability
    {
        let manager = state.capabilities_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if manager.get_agent_capabilities(&agent_uuid).is_none() {
            return Err(StatusCode::NOT_FOUND);
        }
    }
    
    // Return simulator features
    let features = json!({
        "AvatarSkeleton": true,
        "AnimationSet": true,
        "AttachmentsListInNotecards": true,
        "AvatarPickerSearch": true,
        "ChatSessionRequest": true,
        "CopyInventoryFromNotecard": true,
        "CreateInventoryCategory": true,
        "EventQueueGet": true,
        "FetchInventory2": true,
        "GetDisplayNames": true,
        "GetExperienceInfo": true,
        "GetMesh": true,
        "GetTexture": true,
        "GroupMemberData": true,
        "HomeLocation": true,
        "LandResources": true,
        "MapLayer": true,
        "MeshRezEnabled": true,
        "MeshUploadEnabled": true,
        "MeshXferEnabled": true,
        "PhysicsShapeTypes": true,
        "RenderMaterials": true,
        "SimulatorFeatures": true,
        "WebFetchInventoryDescendents": true,
        "OpenSimExtras": {
            "AvatarSkeleton": true,
            "AnimationSet": true,
            "MinSimHeight": -100.0,
            "MaxSimHeight": 10000.0,
            "MinHeightmap": -100.0,
            "MaxHeightmap": 4096.0,
            "GridName": std::env::var("OPENSIM_GRID_NAME").unwrap_or_else(|_| "OpenSim Next".to_string()).trim_matches('"').to_string(),
            "GridURL": std::env::var("OPENSIM_GATEKEEPER_URI").unwrap_or_else(|_| format!("{}", state.base_url)),
            "ExportSupported": true,
            "search-server-url": format!("{}/search", state.base_url),
            "say-range": 20,
            "shout-range": 100,
            "whisper-range": 10
        }
    });
    
    debug!("SimulatorFeatures response for agent {}: {} features", agent_uuid, features.as_object().unwrap().len());
    Ok(Json(features))
}

/// Get Texture capability handler
/// Handles texture downloads for the viewer
pub async fn handle_get_texture(
    Path((agent_id, cap_id)): Path<(String, String)>,
    State(state): State<CapabilityHandlerState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    debug!("GetTexture request for agent: {} cap: {}", agent_id, cap_id);
    
    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate agent capability
    {
        let manager = state.capabilities_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if manager.get_agent_capabilities(&agent_uuid).is_none() {
            return Err(StatusCode::NOT_FOUND);
        }
    }
    
    // Get texture_id parameter
    let texture_id = params.get("texture_id").ok_or(StatusCode::BAD_REQUEST)?;
    
    // For now, return a placeholder response
    // In a full implementation, this would fetch the actual texture data
    let response = json!({
        "texture_id": texture_id,
        "status": "not_found",
        "message": "Texture not implemented yet"
    });
    
    debug!("GetTexture response for agent {}: texture {}", agent_uuid, texture_id);
    Ok(Json(response))
}

/// Update Agent Information capability handler
/// Handles agent information updates
pub async fn handle_update_agent_information(
    Path((agent_id, cap_id)): Path<(String, String)>,
    State(state): State<CapabilityHandlerState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    debug!("UpdateAgentInformation request for agent: {} cap: {}", agent_id, cap_id);
    
    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate agent capability
    {
        let manager = state.capabilities_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if manager.get_agent_capabilities(&agent_uuid).is_none() {
            return Err(StatusCode::NOT_FOUND);
        }
    }
    
    // For now, just acknowledge the update
    let response = json!({
        "success": true,
        "message": "Agent information updated"
    });
    
    debug!("UpdateAgentInformation response for agent {}: success", agent_uuid);
    Ok(Json(response))
}

/// Web Fetch Inventory Descendants capability handler
/// Handles inventory fetching requests
pub async fn handle_web_fetch_inventory_descendants(
    Path((agent_id, cap_id)): Path<(String, String)>,
    State(state): State<CapabilityHandlerState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    debug!("WebFetchInventoryDescendents request for agent: {} cap: {}", agent_id, cap_id);
    
    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate agent capability
    {
        let manager = state.capabilities_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if manager.get_agent_capabilities(&agent_uuid).is_none() {
            return Err(StatusCode::NOT_FOUND);
        }
    }
    
    // For now, return empty inventory
    // In a full implementation, this would fetch actual inventory data
    let response = json!({
        "folders": [],
        "items": [],
        "bad_folders": []
    });
    
    debug!("WebFetchInventoryDescendents response for agent {}: empty", agent_uuid);
    Ok(Json(response))
}

/// Agent Preferences capability handler
/// Handles agent preference settings
pub async fn handle_agent_preferences(
    Path((agent_id, cap_id)): Path<(String, String)>,
    State(state): State<CapabilityHandlerState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    debug!("AgentPreferences request for agent: {} cap: {}", agent_id, cap_id);
    
    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate agent capability
    {
        let manager = state.capabilities_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if manager.get_agent_capabilities(&agent_uuid).is_none() {
            return Err(StatusCode::NOT_FOUND);
        }
    }
    
    // Return default preferences
    let response = json!({
        "default_object_perm_masks": {
            "Everyone": 0,
            "Group": 0,
            "NextOwner": 532480
        },
        "god_level": 0,
        "language": "en-us",
        "language_is_public": true,
        "hover_height": 0.0,
        "away_message": "",
        "email": ""
    });
    
    debug!("AgentPreferences response for agent {}: defaults", agent_uuid);
    Ok(Json(response))
}

/// Generic capability handler for not-yet-implemented capabilities
pub async fn handle_generic_capability(
    Path((agent_id, cap_id)): Path<(String, String)>,
    State(state): State<CapabilityHandlerState>,
) -> Result<Json<Value>, StatusCode> {
    debug!("Generic capability request for agent: {} cap: {}", agent_id, cap_id);
    
    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate agent capability
    {
        let manager = state.capabilities_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if manager.get_agent_capabilities(&agent_uuid).is_none() {
            return Err(StatusCode::NOT_FOUND);
        }
    }
    
    // Return generic success response
    let response = json!({
        "success": true,
        "message": "Capability acknowledged but not yet implemented",
        "agent_id": agent_id,
        "capability_id": cap_id
    });
    
    debug!("Generic capability response for agent {}: acknowledged", agent_uuid);
    Ok(Json(response))
}

/// Capability route information
#[derive(Debug, Clone)]
pub struct CapabilityRoute {
    /// Capability name
    pub name: String,
    /// HTTP method
    pub method: String,
    /// URL pattern
    pub pattern: String,
    /// Handler description
    pub description: String,
}

/// Get all available capability routes
pub fn get_capability_routes() -> Vec<CapabilityRoute> {
    vec![
        CapabilityRoute {
            name: "seed_capability".to_string(),
            method: "POST".to_string(),
            pattern: "/cap/:agent_id/:cap_id".to_string(),
            description: "Bootstrap capability that provides all other capabilities".to_string(),
        },
        CapabilityRoute {
            name: "EventQueueGet".to_string(),
            method: "GET".to_string(),
            pattern: "/cap/:agent_id/:cap_id".to_string(),
            description: "Long-polling endpoint for server-to-client messages".to_string(),
        },
        CapabilityRoute {
            name: "SimulatorFeatures".to_string(),
            method: "GET".to_string(),
            pattern: "/cap/:agent_id/:cap_id".to_string(),
            description: "Provides simulator capabilities and features".to_string(),
        },
        CapabilityRoute {
            name: "GetTexture".to_string(),
            method: "GET".to_string(),
            pattern: "/cap/:agent_id/:cap_id".to_string(),
            description: "Handles texture downloads".to_string(),
        },
        CapabilityRoute {
            name: "UpdateAgentInformation".to_string(),
            method: "POST".to_string(),
            pattern: "/cap/:agent_id/:cap_id".to_string(),
            description: "Handles agent information updates".to_string(),
        },
        CapabilityRoute {
            name: "WebFetchInventoryDescendents".to_string(),
            method: "POST".to_string(),
            pattern: "/cap/:agent_id/:cap_id".to_string(),
            description: "Handles inventory fetching requests".to_string(),
        },
        CapabilityRoute {
            name: "AgentPreferences".to_string(),
            method: "POST".to_string(),
            pattern: "/cap/:agent_id/:cap_id".to_string(),
            description: "Handles agent preference settings".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn create_test_state() -> CapabilityHandlerState {
        let config = CapabilitiesConfig::default();
        let manager = Arc::new(Mutex::new(CapabilitiesManager::new(config)));
        CapabilityHandlerState::new(manager, "http://test.com".to_string())
    }

    #[test]
    fn test_capability_routes() {
        let routes = get_capability_routes();
        
        assert!(!routes.is_empty());
        assert!(routes.iter().any(|r| r.name == "seed_capability"));
        assert!(routes.iter().any(|r| r.name == "EventQueueGet"));
        assert!(routes.iter().any(|r| r.name == "SimulatorFeatures"));
    }
    
    #[test]
    fn test_capability_handler_state() {
        let state = create_test_state();
        
        assert_eq!(state.base_url, "http://test.com");
        
        // Test that we can lock the capabilities manager
        let _manager = state.capabilities_manager.lock().unwrap();
    }
    
    #[tokio::test]
    async fn test_invalid_agent_id_handling() {
        let state = create_test_state();
        
        // Test with invalid UUID
        let params = HashMap::new();
        let result = handle_event_queue_get(
            Path(("invalid-uuid".to_string(), "cap-id".to_string())),
            State(state),
            Query(params),
        ).await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }
}