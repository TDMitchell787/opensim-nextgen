//! Capability registry for managing capability routing and handlers
//! Provides centralized management of all capability endpoints

use super::*;
use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Capability registry for managing all capability endpoints
#[derive(Debug)]
pub struct CapabilityRegistry {
    /// Registered capability handlers
    handlers: HashMap<String, CapabilityHandlerInfo>,
    /// Capabilities manager
    capabilities_manager: Arc<std::sync::Mutex<CapabilitiesManager>>,
    /// Base URL for capabilities
    base_url: String,
}

/// Information about a capability handler
#[derive(Debug, Clone)]
pub struct CapabilityHandlerInfo {
    /// Capability name
    pub name: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Handler description
    pub description: String,
    /// Whether the handler is implemented
    pub implemented: bool,
    /// Handler priority (higher = more important)
    pub priority: u8,
}

/// HTTP method for capability handlers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl HttpMethod {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
        }
    }
}

impl CapabilityRegistry {
    /// Create new capability registry
    pub fn new(
        capabilities_manager: Arc<std::sync::Mutex<CapabilitiesManager>>,
        base_url: String,
    ) -> Self {
        info!(
            "Initializing capability registry with base URL: {}",
            base_url
        );

        let mut registry = Self {
            handlers: HashMap::new(),
            capabilities_manager,
            base_url,
        };

        // Register default capability handlers
        registry.register_default_handlers();

        registry
    }

    /// Register a capability handler
    pub fn register_handler(&mut self, info: CapabilityHandlerInfo) {
        debug!(
            "Registering capability handler: {} ({})",
            info.name,
            info.method.as_str()
        );
        self.handlers.insert(info.name.clone(), info);
    }

    /// Get handler information for a capability
    pub fn get_handler_info(&self, capability_name: &str) -> Option<&CapabilityHandlerInfo> {
        self.handlers.get(capability_name)
    }

    /// Get all registered handlers
    pub fn get_all_handlers(&self) -> Vec<&CapabilityHandlerInfo> {
        self.handlers.values().collect()
    }

    /// Get handlers by implementation status
    pub fn get_handlers_by_status(&self, implemented: bool) -> Vec<&CapabilityHandlerInfo> {
        self.handlers
            .values()
            .filter(|h| h.implemented == implemented)
            .collect()
    }

    /// Check if a capability is registered
    pub fn is_capability_registered(&self, capability_name: &str) -> bool {
        self.handlers.contains_key(capability_name)
    }

    /// Get implementation statistics
    pub fn get_implementation_stats(&self) -> CapabilityImplementationStats {
        let total = self.handlers.len();
        let implemented = self.handlers.values().filter(|h| h.implemented).count();
        let not_implemented = total - implemented;

        CapabilityImplementationStats {
            total_registered: total,
            implemented,
            not_implemented,
            implementation_percentage: if total > 0 {
                (implemented as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Create Axum router for capability endpoints
    pub fn create_router(&self) -> Router {
        use crate::capabilities::handlers::*;
        use crate::capabilities::seed::*;

        let handler_state =
            CapabilityHandlerState::new(self.capabilities_manager.clone(), self.base_url.clone());

        let seed_state = SeedCapabilityState::new(self.capabilities_manager.clone());

        Router::new()
            // Seed capability (bootstrap)
            .route("/cap/:agent_id", post(handle_seed_capability))
            .route(
                "/cap/:agent_id/:cap_id",
                post(handle_seed_capability_with_uuid),
            )
            .with_state(seed_state)
            // Essential capabilities
            .route(
                "/cap/:agent_id/:cap_id/EventQueueGet",
                get(handle_event_queue_get),
            )
            .route(
                "/cap/:agent_id/:cap_id/SimulatorFeatures",
                get(handle_simulator_features),
            )
            .route("/cap/:agent_id/:cap_id/GetTexture", get(handle_get_texture))
            .route(
                "/cap/:agent_id/:cap_id/UpdateAgentInformation",
                post(handle_update_agent_information),
            )
            .route(
                "/cap/:agent_id/:cap_id/WebFetchInventoryDescendents",
                post(handle_web_fetch_inventory_descendants),
            )
            .route(
                "/cap/:agent_id/:cap_id/AgentPreferences",
                post(handle_agent_preferences),
            )
            // Generic handler for not-yet-implemented capabilities
            .route(
                "/cap/:agent_id/:cap_id/*path",
                get(handle_generic_capability),
            )
            .route(
                "/cap/:agent_id/:cap_id/*path",
                post(handle_generic_capability),
            )
            .with_state(handler_state)
    }

    /// Register default capability handlers
    fn register_default_handlers(&mut self) {
        // Essential capabilities (high priority, implemented)
        let essential_handlers = vec![
            CapabilityHandlerInfo {
                name: "seed_capability".to_string(),
                method: HttpMethod::POST,
                description: "Bootstrap capability that provides all other capabilities"
                    .to_string(),
                implemented: true,
                priority: 10,
            },
            CapabilityHandlerInfo {
                name: "EventQueueGet".to_string(),
                method: HttpMethod::GET,
                description: "Long-polling endpoint for server-to-client messages".to_string(),
                implemented: true,
                priority: 9,
            },
            CapabilityHandlerInfo {
                name: "SimulatorFeatures".to_string(),
                method: HttpMethod::GET,
                description: "Provides simulator capabilities and features".to_string(),
                implemented: true,
                priority: 8,
            },
            CapabilityHandlerInfo {
                name: "FetchInventory2".to_string(),
                method: HttpMethod::POST,
                description: "Fetches inventory items and folders".to_string(),
                implemented: false,
                priority: 8,
            },
            CapabilityHandlerInfo {
                name: "WebFetchInventoryDescendents".to_string(),
                method: HttpMethod::POST,
                description: "Fetches inventory folder contents".to_string(),
                implemented: true,
                priority: 8,
            },
            CapabilityHandlerInfo {
                name: "GetTexture".to_string(),
                method: HttpMethod::GET,
                description: "Downloads texture assets".to_string(),
                implemented: true,
                priority: 7,
            },
            CapabilityHandlerInfo {
                name: "GetMesh".to_string(),
                method: HttpMethod::GET,
                description: "Downloads mesh assets".to_string(),
                implemented: false,
                priority: 7,
            },
            CapabilityHandlerInfo {
                name: "UpdateAgentInformation".to_string(),
                method: HttpMethod::POST,
                description: "Updates agent information and preferences".to_string(),
                implemented: true,
                priority: 6,
            },
            CapabilityHandlerInfo {
                name: "AgentPreferences".to_string(),
                method: HttpMethod::POST,
                description: "Manages agent preferences and settings".to_string(),
                implemented: true,
                priority: 6,
            },
        ];

        // Upload capabilities (medium priority, not implemented)
        let upload_handlers = vec![
            CapabilityHandlerInfo {
                name: "NewFileAgentInventory".to_string(),
                method: HttpMethod::POST,
                description: "Uploads new assets to inventory".to_string(),
                implemented: false,
                priority: 5,
            },
            CapabilityHandlerInfo {
                name: "NewFileAgentInventoryVariablePrice".to_string(),
                method: HttpMethod::POST,
                description: "Uploads assets with variable pricing".to_string(),
                implemented: false,
                priority: 5,
            },
            CapabilityHandlerInfo {
                name: "UploadTexture".to_string(),
                method: HttpMethod::POST,
                description: "Uploads texture assets".to_string(),
                implemented: false,
                priority: 5,
            },
            CapabilityHandlerInfo {
                name: "UploadBakedTexture".to_string(),
                method: HttpMethod::POST,
                description: "Uploads baked texture assets".to_string(),
                implemented: false,
                priority: 5,
            },
            CapabilityHandlerInfo {
                name: "UpdateScriptAgent".to_string(),
                method: HttpMethod::POST,
                description: "Updates script assets".to_string(),
                implemented: false,
                priority: 4,
            },
        ];

        // Advanced capabilities (low priority, not implemented)
        let advanced_handlers = vec![
            CapabilityHandlerInfo {
                name: "GetDisplayNames".to_string(),
                method: HttpMethod::GET,
                description: "Gets avatar display names".to_string(),
                implemented: false,
                priority: 3,
            },
            CapabilityHandlerInfo {
                name: "SetDisplayName".to_string(),
                method: HttpMethod::POST,
                description: "Sets avatar display name".to_string(),
                implemented: false,
                priority: 3,
            },
            CapabilityHandlerInfo {
                name: "GroupMemberData".to_string(),
                method: HttpMethod::GET,
                description: "Gets group member information".to_string(),
                implemented: false,
                priority: 2,
            },
            CapabilityHandlerInfo {
                name: "VoiceServerInfo".to_string(),
                method: HttpMethod::GET,
                description: "Gets voice server configuration".to_string(),
                implemented: false,
                priority: 2,
            },
            CapabilityHandlerInfo {
                name: "ViewerMetrics".to_string(),
                method: HttpMethod::POST,
                description: "Collects viewer performance metrics".to_string(),
                implemented: false,
                priority: 1,
            },
            CapabilityHandlerInfo {
                name: "ViewerStats".to_string(),
                method: HttpMethod::POST,
                description: "Collects viewer usage statistics".to_string(),
                implemented: false,
                priority: 1,
            },
        ];

        // Register all handlers
        for handler in essential_handlers
            .into_iter()
            .chain(upload_handlers.into_iter())
            .chain(advanced_handlers.into_iter())
        {
            self.register_handler(handler);
        }

        info!("Registered {} capability handlers", self.handlers.len());

        let stats = self.get_implementation_stats();
        info!(
            "Capability implementation: {}/{} ({:.1}%) implemented",
            stats.implemented, stats.total_registered, stats.implementation_percentage
        );
    }
}

/// Capability implementation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityImplementationStats {
    /// Total number of registered capabilities
    pub total_registered: usize,
    /// Number of implemented capabilities
    pub implemented: usize,
    /// Number of not-yet-implemented capabilities
    pub not_implemented: usize,
    /// Implementation percentage
    pub implementation_percentage: f64,
}

/// Capability registry service for integration with the main application
#[derive(Debug)]
pub struct CapabilityRegistryService {
    /// The capability registry
    registry: CapabilityRegistry,
}

impl CapabilityRegistryService {
    /// Create new capability registry service
    pub fn new(
        capabilities_manager: Arc<std::sync::Mutex<CapabilitiesManager>>,
        base_url: String,
    ) -> Self {
        Self {
            registry: CapabilityRegistry::new(capabilities_manager, base_url),
        }
    }

    /// Get the registry
    pub fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }

    /// Get mutable registry
    pub fn registry_mut(&mut self) -> &mut CapabilityRegistry {
        &mut self.registry
    }

    /// Create Axum router
    pub fn create_router(&self) -> Router {
        self.registry.create_router()
    }

    /// Get implementation statistics
    pub fn get_stats(&self) -> CapabilityImplementationStats {
        self.registry.get_implementation_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn create_test_registry() -> CapabilityRegistry {
        let config = CapabilitiesConfig::default();
        let manager = Arc::new(Mutex::new(CapabilitiesManager::new(config)));
        CapabilityRegistry::new(manager, "http://test.com/cap".to_string())
    }

    #[test]
    fn test_capability_registry_creation() {
        let registry = create_test_registry();

        // Should have default handlers registered
        assert!(!registry.handlers.is_empty());

        // Check that essential capabilities are registered
        assert!(registry.is_capability_registered("seed_capability"));
        assert!(registry.is_capability_registered("EventQueueGet"));
        assert!(registry.is_capability_registered("SimulatorFeatures"));
    }

    #[test]
    fn test_handler_registration() {
        let mut registry = create_test_registry();

        let custom_handler = CapabilityHandlerInfo {
            name: "CustomCapability".to_string(),
            method: HttpMethod::GET,
            description: "A custom test capability".to_string(),
            implemented: true,
            priority: 5,
        };

        registry.register_handler(custom_handler);

        assert!(registry.is_capability_registered("CustomCapability"));

        let handler_info = registry.get_handler_info("CustomCapability").unwrap();
        assert_eq!(handler_info.name, "CustomCapability");
        assert_eq!(handler_info.method, HttpMethod::GET);
        assert!(handler_info.implemented);
    }

    #[test]
    fn test_implementation_stats() {
        let registry = create_test_registry();
        let stats = registry.get_implementation_stats();

        assert!(stats.total_registered > 0);
        assert!(stats.implemented > 0);
        assert!(stats.not_implemented >= 0);
        assert!(stats.implementation_percentage >= 0.0);
        assert!(stats.implementation_percentage <= 100.0);

        // Verify math
        assert_eq!(
            stats.total_registered,
            stats.implemented + stats.not_implemented
        );
    }

    #[test]
    fn test_handlers_by_status() {
        let registry = create_test_registry();

        let implemented = registry.get_handlers_by_status(true);
        let not_implemented = registry.get_handlers_by_status(false);

        assert!(!implemented.is_empty());
        assert!(!not_implemented.is_empty());

        // Verify all implemented handlers are marked as implemented
        for handler in &implemented {
            assert!(handler.implemented);
        }

        // Verify all not-implemented handlers are marked as not implemented
        for handler in &not_implemented {
            assert!(!handler.implemented);
        }
    }

    #[test]
    fn test_http_method() {
        assert_eq!(HttpMethod::GET.as_str(), "GET");
        assert_eq!(HttpMethod::POST.as_str(), "POST");
        assert_eq!(HttpMethod::PUT.as_str(), "PUT");
        assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
    }

    #[test]
    fn test_capability_registry_service() {
        let config = CapabilitiesConfig::default();
        let manager = Arc::new(Mutex::new(CapabilitiesManager::new(config)));
        let service = CapabilityRegistryService::new(manager, "http://test.com/cap".to_string());

        let stats = service.get_stats();
        assert!(stats.total_registered > 0);

        // Test that we can create a router
        let _router = service.create_router();
    }
}
