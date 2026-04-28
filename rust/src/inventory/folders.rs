//! Inventory folder management for Second Life/OpenSim compatibility

use super::*;

/// Inventory folder structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryFolder {
    /// Unique folder ID
    pub id: Uuid,
    /// Parent folder ID (None for root)
    pub parent_id: Option<Uuid>,
    /// Owner user ID
    pub owner_id: Uuid,
    /// Folder name
    pub name: String,
    /// Folder type
    pub folder_type: InventoryFolderType,
    /// Folder version (for synchronization)
    pub version: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,
}

impl InventoryFolder {
    /// Create a new inventory folder with deterministic ID based on owner and name
    pub fn new(
        owner_id: Uuid,
        parent_id: Option<Uuid>,
        name: String,
        folder_type: InventoryFolderType,
    ) -> Self {
        let now = Utc::now();
        // Generate deterministic folder ID based on owner_id and folder name
        // This ensures the same folder always gets the same ID for a given user
        let folder_key = format!("{}:{}", owner_id, name.to_lowercase());
        let id = Uuid::new_v5(&Uuid::NAMESPACE_OID, folder_key.as_bytes());
        Self {
            id,
            parent_id,
            owner_id,
            name,
            folder_type,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create the root inventory folder for a user
    pub fn new_root(owner_id: Uuid) -> Self {
        Self::new(
            owner_id,
            None,
            "My Inventory".to_string(),
            InventoryFolderType::Root,
        )
    }

    /// Create a system folder (default folders)
    pub fn new_system_folder(
        owner_id: Uuid,
        parent_id: Uuid,
        folder_type: InventoryFolderType,
    ) -> Self {
        Self::new(
            owner_id,
            Some(parent_id),
            folder_type.default_name().to_string(),
            folder_type,
        )
    }

    /// Generate deterministic folder ID for a user and folder name
    /// This is used by both login inventory and FetchInventoryDescendents2
    pub fn generate_folder_id(owner_id: &str, folder_name: &str) -> String {
        let folder_key = format!("{}:{}", owner_id, folder_name.to_lowercase());
        Uuid::new_v5(&Uuid::NAMESPACE_OID, folder_key.as_bytes()).to_string()
    }

    /// Convert to login response format
    pub fn to_login_folder(&self) -> LoginInventoryFolder {
        LoginInventoryFolder {
            folder_id: self.id.to_string(),
            parent_id: self
                .parent_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string()),
            name: self.name.clone(),
            type_default: self.folder_type.as_string(),
            version: self.version.to_string(),
        }
    }

    /// Update folder version (for synchronization)
    pub fn increment_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }

    /// Check if this is the root folder
    pub fn is_root(&self) -> bool {
        self.folder_type == InventoryFolderType::Root && self.parent_id.is_none()
    }

    /// Check if this is a system folder
    pub fn is_system_folder(&self) -> bool {
        self.folder_type.is_default_folder()
    }
}

/// Login inventory folder format (for XMLRPC responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInventoryFolder {
    /// Folder ID as string
    pub folder_id: String,
    /// Parent folder ID as string
    pub parent_id: String,
    /// Folder name
    pub name: String,
    /// Folder type as string
    pub type_default: String,
    /// Version as string
    pub version: String,
}

impl LoginInventoryFolder {
    /// Convert to XMLRPC struct format
    pub fn to_xmlrpc_struct(&self) -> String {
        format!(
            r#"<value>
              <struct>
                <member>
                  <name>folder_id</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>parent_id</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>name</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>type_default</name>
                  <value><string>{}</string></value>
                </member>
                <member>
                  <name>version</name>
                  <value><string>{}</string></value>
                </member>
              </struct>
            </value>"#,
            self.folder_id, self.parent_id, self.name, self.type_default, self.version
        )
    }
}

/// Folder creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    /// Parent folder ID
    pub parent_id: Uuid,
    /// Folder name
    pub name: String,
    /// Folder type
    pub folder_type: InventoryFolderType,
}

/// Folder update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFolderRequest {
    /// New folder name (optional)
    pub name: Option<String>,
    /// New parent folder ID (optional)
    pub parent_id: Option<Uuid>,
}

/// Database folder UUID lookup result
/// Maps folder_type (i64) to (folder_id, parent_id, name, version)
#[derive(Debug, Clone)]
pub struct DbFolderInfo {
    pub folder_id: Uuid,
    pub parent_id: Uuid,
    pub name: String,
    pub folder_type: i64,
    pub version: i64,
}

/// Phase 85.8: Lookup existing folder UUIDs from PostgreSQL database
/// Returns HashMap mapping folder_type to DbFolderInfo
/// This is READ-ONLY and used to populate generated inventory with real database UUIDs
pub async fn lookup_database_folder_uuids(
    db_pool: &sqlx::PgPool,
    agent_id: Uuid,
) -> Option<std::collections::HashMap<i64, DbFolderInfo>> {
    use sqlx::Row;
    use std::collections::HashMap;
    use tracing::{debug, warn};

    let rows = match sqlx::query(
        "SELECT folderid, parentfolderid, foldername, type, version FROM inventoryfolders WHERE agentid = $1"
    )
    .bind(agent_id)
    .fetch_all(db_pool)
    .await {
        Ok(rows) => rows,
        Err(e) => {
            warn!("📦 Phase 85.8: Failed to lookup folder UUIDs: {}", e);
            return None;
        }
    };

    if rows.is_empty() {
        debug!(
            "📦 Phase 85.8: No folders found in database for agent {}",
            agent_id
        );
        return None;
    }

    let mut folder_map: HashMap<i64, DbFolderInfo> = HashMap::new();

    let suitcase_id: Option<Uuid> = rows.iter().find_map(|row| {
        let ft: i32 = row.try_get::<i32, _>("type").unwrap_or(-1);
        if ft == 100 {
            row.try_get("folderid").ok()
        } else {
            None
        }
    });

    for row in &rows {
        let folder_id: Uuid = match row.try_get("folderid") {
            Ok(id) => id,
            Err(_) => continue,
        };
        let parent_id: Uuid = row.try_get("parentfolderid").unwrap_or(Uuid::nil());
        let name: String = row.try_get("foldername").unwrap_or_default();

        let folder_type: i64 = row
            .try_get::<i64, _>("type")
            .or_else(|_| row.try_get::<i32, _>("type").map(|v| v as i64))
            .unwrap_or(0);
        let version: i64 = row
            .try_get::<i64, _>("version")
            .or_else(|_| row.try_get::<i32, _>("version").map(|v| v as i64))
            .unwrap_or(1);

        if folder_type == 100 {
            continue;
        }
        if let Some(sc_id) = suitcase_id {
            if parent_id == sc_id {
                continue;
            }
        }

        folder_map.insert(
            folder_type,
            DbFolderInfo {
                folder_id,
                parent_id,
                name,
                folder_type,
                version,
            },
        );
    }

    debug!(
        "📦 Phase 85.8: Found {} folder UUIDs in database for agent {}",
        folder_map.len(),
        agent_id
    );

    Some(folder_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_folder_creation() {
        let owner_id = Uuid::new_v4();
        let parent_id = Uuid::new_v4();

        let folder = InventoryFolder::new(
            owner_id,
            Some(parent_id),
            "Test Folder".to_string(),
            InventoryFolderType::Object,
        );

        assert_eq!(folder.owner_id, owner_id);
        assert_eq!(folder.parent_id, Some(parent_id));
        assert_eq!(folder.name, "Test Folder");
        assert_eq!(folder.folder_type, InventoryFolderType::Object);
        assert_eq!(folder.version, 1);
        assert!(!folder.is_root());
        assert!(folder.is_system_folder());
    }

    #[test]
    fn test_root_folder_creation() {
        let owner_id = Uuid::new_v4();
        let folder = InventoryFolder::new_root(owner_id);

        assert_eq!(folder.owner_id, owner_id);
        assert_eq!(folder.parent_id, None);
        assert_eq!(folder.name, "My Inventory");
        assert_eq!(folder.folder_type, InventoryFolderType::Root);
        assert!(folder.is_root());
        assert!(folder.is_system_folder());
    }

    #[test]
    fn test_system_folder_creation() {
        let owner_id = Uuid::new_v4();
        let parent_id = Uuid::new_v4();

        let folder =
            InventoryFolder::new_system_folder(owner_id, parent_id, InventoryFolderType::Texture);

        assert_eq!(folder.name, "Textures");
        assert_eq!(folder.folder_type, InventoryFolderType::Texture);
        assert_eq!(folder.parent_id, Some(parent_id));
    }

    #[test]
    fn test_login_folder_conversion() {
        let owner_id = Uuid::new_v4();
        let folder = InventoryFolder::new_root(owner_id);
        let login_folder = folder.to_login_folder();

        assert_eq!(login_folder.folder_id, folder.id.to_string());
        assert_eq!(
            login_folder.parent_id,
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(login_folder.name, "My Inventory");
        assert_eq!(login_folder.type_default, "8");
        assert_eq!(login_folder.version, "1");
    }

    #[test]
    fn test_folder_version_increment() {
        let owner_id = Uuid::new_v4();
        let mut folder = InventoryFolder::new_root(owner_id);
        let initial_version = folder.version;
        let initial_updated = folder.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        folder.increment_version();

        assert_eq!(folder.version, initial_version + 1);
        assert!(folder.updated_at > initial_updated);
    }

    #[test]
    fn test_xmlrpc_struct_generation() {
        let owner_id = Uuid::new_v4();
        let folder = InventoryFolder::new_root(owner_id);
        let login_folder = folder.to_login_folder();
        let xmlrpc = login_folder.to_xmlrpc_struct();

        assert!(xmlrpc.contains(&format!("<string>{}</string>", folder.id)));
        assert!(xmlrpc.contains("<string>My Inventory</string>"));
        assert!(xmlrpc.contains("<string>8</string>"));
    }
}
