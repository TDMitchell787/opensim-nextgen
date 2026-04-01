//! Login inventory response structures for Second Life/OpenSim XMLRPC compatibility

use super::*;

/// Complete inventory response for login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInventoryResponse {
    /// Root inventory folder
    pub inventory_root: Vec<LoginInventoryFolder>,
    /// Skeleton inventory folders (all folders except root)
    pub inventory_skeleton: Vec<LoginInventoryFolder>,
    /// Library root folders
    pub inventory_lib_root: Vec<LoginInventoryFolder>,
    /// Library skeleton folders
    pub inventory_skel_lib: Vec<LoginInventoryFolder>,
    /// Library owner information
    pub inventory_lib_owner: Vec<LoginInventoryOwner>,
}

impl LoginInventoryResponse {
    /// Create empty inventory response
    pub fn empty() -> Self {
        Self {
            inventory_root: Vec::new(),
            inventory_skeleton: Vec::new(),
            inventory_lib_root: Vec::new(),
            inventory_skel_lib: Vec::new(),
            inventory_lib_owner: Vec::new(),
        }
    }
    
    /// Convert to XMLRPC format for login response
    pub fn to_xmlrpc_members(&self) -> String {
        let mut xmlrpc = String::new();
        
        // Inventory root
        xmlrpc.push_str(&format!(
            r#"<member>
            <name>inventory-root</name>
            <value>
              <array>
                <data>
                  {}
                </data>
              </array>
            </value>
          </member>"#,
            self.inventory_root
                .iter()
                .map(|f| f.to_xmlrpc_struct())
                .collect::<Vec<_>>()
                .join("\n                  ")
        ));
        
        // Inventory skeleton
        xmlrpc.push_str(&format!(
            r#"
          <member>
            <name>inventory-skeleton</name>
            <value>
              <array>
                <data>
                  {}
                </data>
              </array>
            </value>
          </member>"#,
            self.inventory_skeleton
                .iter()
                .map(|f| f.to_xmlrpc_struct())
                .collect::<Vec<_>>()
                .join("\n                  ")
        ));
        
        // Library root - OpenSim only sends folder_id for lib-root
        xmlrpc.push_str(&format!(
            r#"
          <member>
            <name>inventory-lib-root</name>
            <value>
              <array>
                <data>
                  {}
                </data>
              </array>
            </value>
          </member>"#,
            self.inventory_lib_root
                .iter()
                .map(|f| format!(r#"<value>
              <struct>
                <member>
                  <name>folder_id</name>
                  <value><string>{}</string></value>
                </member>
              </struct>
            </value>"#, f.folder_id))
                .collect::<Vec<_>>()
                .join("\n                  ")
        ));
        
        // Library skeleton (usually empty)
        xmlrpc.push_str(&format!(
            r#"
          <member>
            <name>inventory-skel-lib</name>
            <value>
              <array>
                <data>
                  {}
                </data>
              </array>
            </value>
          </member>"#,
            self.inventory_skel_lib
                .iter()
                .map(|f| f.to_xmlrpc_struct())
                .collect::<Vec<_>>()
                .join("\n                  ")
        ));
        
        // Library owner (usually empty)
        xmlrpc.push_str(&format!(
            r#"
          <member>
            <name>inventory-lib-owner</name>
            <value>
              <array>
                <data>
                  {}
                </data>
              </array>
            </value>
          </member>"#,
            self.inventory_lib_owner
                .iter()
                .map(|o| o.to_xmlrpc_struct())
                .collect::<Vec<_>>()
                .join("\n                  ")
        ));
        
        xmlrpc
    }
    
    /// Get folder count
    pub fn folder_count(&self) -> usize {
        self.inventory_root.len() + self.inventory_skeleton.len()
    }
    
    /// Check if inventory is empty
    pub fn is_empty(&self) -> bool {
        self.inventory_root.is_empty() && self.inventory_skeleton.is_empty()
    }
}

/// Library owner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInventoryOwner {
    /// Owner agent ID
    pub agent_id: String,
    /// Owner name
    pub owner_name: String,
}

impl LoginInventoryOwner {
    /// Convert to XMLRPC struct format
    pub fn to_xmlrpc_struct(&self) -> String {
        format!(
            r#"<value>
              <struct>
                <member>
                  <name>agent_id</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>owner_name</name>
                  <value><string>{}</string></value>
                </member>
              </struct>
            </value>"#,
            self.agent_id, self.owner_name
        )
    }
}

/// Inventory service for managing login inventory responses
#[derive(Debug)]
pub struct LoginInventoryService {
    /// Inventory manager
    inventory_manager: InventoryManager,
}

impl LoginInventoryService {
    /// Create new login inventory service
    pub fn new() -> Self {
        info!("Initializing login inventory service");
        Self {
            inventory_manager: InventoryManager::new(),
        }
    }
    
    /// Get inventory response for user login
    pub fn get_login_inventory(&mut self, user_id: Uuid) -> LoginInventoryResponse {
        debug!("🔍 INVENTORY TRACE: Starting get_login_inventory for user: {}", user_id);
        debug!("🔍 INVENTORY TRACE: About to call inventory_manager.get_login_inventory");
        let result = self.inventory_manager.get_login_inventory(user_id);
        debug!("🔍 INVENTORY TRACE: inventory_manager.get_login_inventory completed successfully");
        result
    }
    
    /// Create default inventory for new user
    pub fn create_default_inventory(&mut self, user_id: Uuid) -> LoginInventoryResponse {
        info!("Creating default inventory for new user: {}", user_id);
        
        // Clear any existing cache
        self.inventory_manager.clear_user_cache(user_id);
        
        // Get fresh inventory (will create default)
        self.get_login_inventory(user_id)
    }
    
    /// Get library inventory from loaded library assets
    pub fn get_library_inventory() -> LoginInventoryResponse {
        use crate::opensim_compatibility::library_assets::get_global_library_manager;

        // Try to get library data from global manager
        if let Some(manager_arc) = get_global_library_manager() {
            // Use try_read to avoid blocking - if locked, return empty
            if let Ok(manager) = manager_arc.try_read() {
                debug!("Loading library inventory from LibraryAssetManager");

                // Get library root folder
                let lib_root_data = manager.get_library_root_for_login();
                let inventory_lib_root: Vec<LoginInventoryFolder> = lib_root_data
                    .into_iter()
                    .map(|f| LoginInventoryFolder {
                        folder_id: f.folder_id,
                        parent_id: f.parent_id,
                        name: f.name,
                        type_default: f.type_default,
                        version: f.version,
                    })
                    .collect();

                // Get library skeleton (all subfolders)
                let lib_skel_data = manager.get_library_skeleton_for_login();
                let inventory_skel_lib: Vec<LoginInventoryFolder> = lib_skel_data
                    .into_iter()
                    .map(|f| LoginInventoryFolder {
                        folder_id: f.folder_id,
                        parent_id: f.parent_id,
                        name: f.name,
                        type_default: f.type_default,
                        version: f.version,
                    })
                    .collect();

                // Get library owner
                let lib_owner_data = manager.get_library_owner_for_login();
                let inventory_lib_owner: Vec<LoginInventoryOwner> = lib_owner_data
                    .into_iter()
                    .map(|o| LoginInventoryOwner {
                        agent_id: o.agent_id,
                        owner_name: "Library Owner".to_string(),
                    })
                    .collect();

                info!("Loaded library inventory: {} root folders, {} skeleton folders, {} owners",
                      inventory_lib_root.len(), inventory_skel_lib.len(), inventory_lib_owner.len());

                return LoginInventoryResponse {
                    inventory_root: Vec::new(),
                    inventory_skeleton: Vec::new(),
                    inventory_lib_root,
                    inventory_skel_lib,
                    inventory_lib_owner,
                };
            }
        }

        debug!("Library asset manager not available, returning empty library inventory");
        LoginInventoryResponse::empty()
    }
    
    /// Clear user inventory cache
    pub fn clear_cache(&mut self, user_id: Uuid) {
        self.inventory_manager.clear_user_cache(user_id);
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> InventoryCacheStats {
        InventoryCacheStats {
            cached_users: self.inventory_manager.cache_size(),
        }
    }
}

impl Default for LoginInventoryService {
    fn default() -> Self {
        Self::new()
    }
}

/// Inventory cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryCacheStats {
    /// Number of cached user inventories
    pub cached_users: usize,
}

/// Inventory configuration for login responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLoginConfig {
    /// Enable inventory in login responses
    pub enabled: bool,
    /// Include library inventory
    pub include_library: bool,
    /// Maximum folders per user
    pub max_folders: usize,
    /// Maximum items per folder
    pub max_items_per_folder: usize,
    /// Cache timeout in minutes
    pub cache_timeout_minutes: u64,
}

impl Default for InventoryLoginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_library: false,
            max_folders: 1000,
            max_items_per_folder: 10000,
            cache_timeout_minutes: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_inventory_response() {
        let response = LoginInventoryResponse::empty();
        
        assert!(response.is_empty());
        assert_eq!(response.folder_count(), 0);
        assert!(response.inventory_root.is_empty());
        assert!(response.inventory_skeleton.is_empty());
    }
    
    #[test]
    fn test_login_inventory_service() {
        let mut service = LoginInventoryService::new();
        let user_id = Uuid::new_v4();
        
        let response = service.get_login_inventory(user_id);
        
        assert!(!response.is_empty());
        assert!(response.folder_count() > 0);
        assert_eq!(response.inventory_root.len(), 1); // Root folder
        assert!(response.inventory_skeleton.len() > 0); // System folders
    }
    
    #[test]
    fn test_default_inventory_creation() {
        let mut service = LoginInventoryService::new();
        let user_id = Uuid::new_v4();
        
        let response = service.create_default_inventory(user_id);
        
        assert!(!response.is_empty());
        
        // Check root folder
        assert_eq!(response.inventory_root.len(), 1);
        let root = &response.inventory_root[0];
        assert_eq!(root.name, "My Inventory");
        assert_eq!(root.type_default, "8"); // Root type
        assert_eq!(root.parent_id, "00000000-0000-0000-0000-000000000000");
        
        // Check system folders (20 system folders + root in skeleton = 21)
        assert!(response.inventory_skeleton.len() >= 20);

        let folder_names: Vec<&String> = response.inventory_skeleton
            .iter()
            .map(|f| &f.name)
            .collect();

        assert!(folder_names.contains(&&"Textures".to_string()));
        assert!(folder_names.contains(&&"Objects".to_string()));
        assert!(folder_names.contains(&&"Scripts".to_string()));
        assert!(folder_names.contains(&&"Clothing".to_string()));
        assert!(folder_names.contains(&&"Lost And Found".to_string()));
        assert!(folder_names.contains(&&"Favorites".to_string()));
        assert!(folder_names.contains(&&"Current Outfit".to_string()));
        assert!(folder_names.contains(&&"My Outfits".to_string()));
        assert!(folder_names.contains(&&"Settings".to_string()));
        assert!(folder_names.contains(&&"Trash".to_string()));
        assert!(folder_names.contains(&&"Photo Album".to_string()));
        assert!(folder_names.contains(&&"Received Items".to_string()));
        assert!(folder_names.contains(&&"Materials".to_string()));
    }
    
    #[test]
    fn test_library_inventory() {
        let response = LoginInventoryService::get_library_inventory();
        
        assert!(response.is_empty());
        assert_eq!(response.inventory_lib_root.len(), 0);
        assert_eq!(response.inventory_skel_lib.len(), 0);
    }
    
    #[test]
    fn test_cache_operations() {
        let mut service = LoginInventoryService::new();
        let user_id = Uuid::new_v4();
        
        // Initial cache should be empty
        assert_eq!(service.cache_stats().cached_users, 0);
        
        // Getting inventory should cache it
        let _response = service.get_login_inventory(user_id);
        assert_eq!(service.cache_stats().cached_users, 1);
        
        // Clear cache
        service.clear_cache(user_id);
        assert_eq!(service.cache_stats().cached_users, 0);
    }
    
    #[test]
    fn test_xmlrpc_generation() {
        let mut service = LoginInventoryService::new();
        let user_id = Uuid::new_v4();
        
        let response = service.get_login_inventory(user_id);
        let xmlrpc = response.to_xmlrpc_members();
        
        assert!(xmlrpc.contains("inventory-root"));
        assert!(xmlrpc.contains("inventory-skeleton"));
        assert!(xmlrpc.contains("inventory-lib-root"));
        assert!(xmlrpc.contains("My Inventory"));
        assert!(xmlrpc.contains("folder_id"));
        assert!(xmlrpc.contains("type_default"));
    }
    
    #[test]
    fn test_inventory_config() {
        let config = InventoryLoginConfig::default();
        
        assert!(config.enabled);
        assert!(!config.include_library);
        assert_eq!(config.max_folders, 1000);
        assert_eq!(config.cache_timeout_minutes, 30);
    }
}