//! OpenSim Archive (OAR/IAR) format support
//!
//! Provides compatibility with OpenSimulator's OAR (OpenSim Archive) and
//! IAR (Inventory Archive) file formats for backup and restoration.

use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write, Cursor};
use std::collections::HashMap;
use zip::{ZipArchive, ZipWriter, write::FileOptions};
use anyhow::Result;

/// Archive format types
#[derive(Debug, Clone, PartialEq)]
pub enum ArchiveFormat {
    OAR, // OpenSim Archive (Region)  
    IAR, // Inventory Archive
}

/// Elegant archive data extraction helper
#[derive(Debug)]
struct ExtractedArchiveData {
    manifest_content: String,
    region_content: String,
    terrain_data: Option<Vec<u8>>,
    object_contents: Vec<String>,
    asset_data: Vec<(String, Vec<u8>, Option<String>)>, // (name, data, metadata)
}

/// Archive manager for OAR/IAR operations
pub struct ArchiveManager {
    bin_directory: PathBuf,
    temp_directory: PathBuf,
}

/// OAR archive contents
#[derive(Debug, Clone)]
pub struct OARArchive {
    pub format_version: String,
    pub region_info: RegionInfo,
    pub terrain_data: Option<Vec<u8>>,
    pub objects: Vec<SceneObject>,
    pub assets: Vec<AssetData>,
    pub parcels: Vec<ParcelData>,
    pub settings: HashMap<String, String>,
}

/// IAR archive contents
#[derive(Debug, Clone)]
pub struct IARArchive {
    pub format_version: String,
    pub user_info: UserInfo,
    pub inventory_items: Vec<InventoryItem>,
    pub inventory_folders: Vec<InventoryFolder>,
    pub assets: Vec<AssetData>,
}

/// Region information for OAR
#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub uuid: String,
    pub name: String,
    pub location_x: u32,
    pub location_y: u32,
    pub size_x: u32,
    pub size_y: u32,
    pub external_host_name: String,
    pub internal_port: u16,
}

/// Scene object in OAR
#[derive(Debug, Clone)]
pub struct SceneObject {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub scale: (f32, f32, f32),
    pub xml_data: String,
    pub owner_id: String,
    pub creator_id: String,
}

/// Asset data
#[derive(Debug, Clone)]
pub struct AssetData {
    pub uuid: String,
    pub asset_type: i32,
    pub name: String,
    pub description: String,
    pub data: Vec<u8>,
    pub temporary: bool,
    pub local: bool,
}

/// Parcel data for land management
#[derive(Debug, Clone)]
pub struct ParcelData {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub group_id: Option<String>,
    pub area: u32,
    pub bitmap: Vec<u8>,
    pub flags: u32,
}

/// User information for IAR
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub uuid: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub created: String,
}

/// Inventory item
#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub asset_type: i32,
    pub inventory_type: i32,
    pub folder_id: String,
    pub owner_id: String,
    pub creator_id: String,
    pub asset_id: String,
    pub base_permissions: u32,
    pub current_permissions: u32,
    pub everyone_permissions: u32,
    pub group_permissions: u32,
    pub next_permissions: u32,
}

/// Inventory folder
#[derive(Debug, Clone)]
pub struct InventoryFolder {
    pub uuid: String,
    pub name: String,
    pub owner_id: String,
    pub parent_id: Option<String>,
    pub folder_type: i32,
    pub version: u16,
}

impl ArchiveManager {
    /// Create a new archive manager
    pub fn new(bin_directory: PathBuf) -> Result<Self> {
        let temp_directory = bin_directory.join("temp");
        
        Ok(Self {
            bin_directory,
            temp_directory,
        })
    }

    /// Extract all needed data from archive in a single pass to avoid borrow conflicts
    fn extract_archive_data(archive: &mut ZipArchive<std::fs::File>) -> Result<ExtractedArchiveData> {
        let mut manifest_content = String::new();
        let mut region_content = String::new();
        let mut terrain_data = None;
        let mut object_contents = Vec::new();
        let mut asset_data = Vec::new();
        let mut metadata_map = HashMap::new();

        let archive_len = archive.len();
        
        // First pass: collect all data including metadata files
        for i in 0..archive_len {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            match name.as_str() {
                "archive.xml" => {
                    file.read_to_string(&mut manifest_content)?;
                }
                "settings/region.xml" => {
                    file.read_to_string(&mut region_content)?;
                }
                "user/profile.xml" => {
                    // For IAR archives, user profile goes in region_content field
                    file.read_to_string(&mut region_content)?;
                }
                "terrains/terrain.raw" => {
                    let mut data = Vec::new();
                    file.read_to_end(&mut data)?;
                    terrain_data = Some(data);
                }
                name if name.starts_with("objects/") && name.ends_with(".xml") => {
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    object_contents.push(content);
                }
                name if name.starts_with("landdata/") && name.ends_with(".xml") => {
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    object_contents.push(content); // Parcels stored with objects for simplicity
                }
                name if name.starts_with("inventory/") && name.ends_with(".xml") => {
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    object_contents.push(content); // Inventory items stored with objects for simplicity
                }
                name if name.starts_with("folders/") && name.ends_with(".xml") => {
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    object_contents.push(content); // Inventory folders stored with objects for simplicity
                }
                name if name.starts_with("assets/") && name.ends_with("_metadata.xml") => {
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    // Store metadata with the asset name (without _metadata.xml suffix)
                    let asset_name = name.replace("_metadata.xml", "");
                    metadata_map.insert(asset_name, content);
                }
                name if name.starts_with("assets/") && !name.ends_with("_metadata.xml") => {
                    let mut data = Vec::new();
                    file.read_to_end(&mut data)?;
                    
                    // Get metadata from the map if available
                    let metadata = metadata_map.get(name).cloned();
                    asset_data.push((name.to_string(), data, metadata));
                }
                _ => {
                    // Skip other files
                }
            }
        }

        Ok(ExtractedArchiveData {
            manifest_content,
            region_content,
            terrain_data,
            object_contents,
            asset_data,
        })
    }

    /// Create an OAR archive from region data
    pub async fn create_oar_archive(
        &self,
        region_uuid: &str,
        output_path: &Path,
    ) -> Result<()> {
        // Ensure temp directory exists
        fs::create_dir_all(&self.temp_directory)?;

        let file = fs::File::create(output_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Add archive.xml (OAR manifest)
        let archive_xml = self.create_oar_manifest(region_uuid)?;
        zip.start_file("archive.xml", options)?;
        zip.write_all(archive_xml.as_bytes())?;

        // Add region settings
        let region_settings = self.load_region_settings(region_uuid)?;
        zip.start_file("settings/region.xml", options)?;
        zip.write_all(region_settings.as_bytes())?;

        // Add terrain data
        if let Ok(terrain_data) = self.load_terrain_data(region_uuid) {
            zip.start_file("terrains/terrain.raw", options)?;
            zip.write_all(&terrain_data)?;
        }

        // Add objects
        let objects = self.load_region_objects(region_uuid).await?;
        for (i, object) in objects.iter().enumerate() {
            let filename = format!("objects/{:08}.xml", i);
            zip.start_file(&filename, options)?;
            zip.write_all(object.xml_data.as_bytes())?;
        }

        // Add assets
        let assets = self.load_region_assets(region_uuid).await?;
        for asset in assets {
            let filename = format!("assets/{}", asset.uuid);
            zip.start_file(&filename, options)?;
            zip.write_all(&asset.data)?;
            
            // Add asset metadata
            let metadata_filename = format!("assets/{}_metadata.xml", asset.uuid);
            let metadata_xml = self.create_asset_metadata_xml(&asset)?;
            zip.start_file(&metadata_filename, options)?;
            zip.write_all(metadata_xml.as_bytes())?;
        }

        // Add parcels (land data)
        let parcels = self.load_region_parcels(region_uuid).await?;
        for (i, parcel) in parcels.iter().enumerate() {
            let filename = format!("landdata/{:08}.xml", i);
            let parcel_xml = self.create_parcel_xml(parcel)?;
            zip.start_file(&filename, options)?;
            zip.write_all(parcel_xml.as_bytes())?;
        }

        zip.finish()?;
        
        tracing::info!("Created OAR archive: {}", output_path.display());
        Ok(())
    }

    /// Load an OAR archive
    pub async fn load_oar_archive(&self, archive_path: &Path) -> Result<OARArchive> {
        let file = fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        // Extract all data in a single pass to avoid borrow conflicts
        let extracted_data = Self::extract_archive_data(&mut archive)?;
        
        let format_version = self.parse_oar_version(&extracted_data.manifest_content)?;
        let region_info = self.parse_region_info(&extracted_data.region_content)?;

        // Parse objects from extracted content
        let mut objects = Vec::new();
        for content in &extracted_data.object_contents {
            if let Ok(object) = self.parse_scene_object(content) {
                objects.push(object);
            }
        }

        // Parse assets from extracted data
        let mut assets = Vec::new();
        for (name, data, metadata_content) in &extracted_data.asset_data {
            let metadata = if let Some(meta_content) = metadata_content {
                self.parse_asset_metadata(meta_content)?
            } else {
                // Create default metadata
                AssetData {
                    uuid: name.split('/').last().unwrap_or("unknown").to_string(),
                    asset_type: 0,
                    name: "Unknown".to_string(),
                    description: String::new(),
                    data: data.clone(),
                    temporary: false,
                    local: false,
                }
            };
            assets.push(metadata);
        }

        // Parse parcels - they're included in object_contents for simplicity
        let mut parcels = Vec::new();
        for content in &extracted_data.object_contents {
            if let Ok(parcel) = self.parse_parcel_data(content) {
                parcels.push(parcel);
            }
        }

        Ok(OARArchive {
            format_version,
            region_info,
            terrain_data: extracted_data.terrain_data,
            objects,
            assets,
            parcels,
            settings: HashMap::new(),
        })
    }

    /// Create an IAR archive from user inventory
    pub async fn create_iar_archive(
        &self,
        user_uuid: &str,
        output_path: &Path,
    ) -> Result<()> {
        let file = fs::File::create(output_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Add archive.xml (IAR manifest)
        let archive_xml = self.create_iar_manifest(user_uuid)?;
        zip.start_file("archive.xml", options)?;
        zip.write_all(archive_xml.as_bytes())?;

        // Add user profile
        let user_profile = self.load_user_profile(user_uuid).await?;
        zip.start_file("user/profile.xml", options)?;
        zip.write_all(user_profile.as_bytes())?;

        // Add inventory items
        let inventory_items = self.load_user_inventory_items(user_uuid).await?;
        for item in inventory_items {
            let filename = format!("inventory/{}.xml", item.uuid);
            let item_xml = self.create_inventory_item_xml(&item)?;
            zip.start_file(&filename, options)?;
            zip.write_all(item_xml.as_bytes())?;
        }

        // Add inventory folders
        let inventory_folders = self.load_user_inventory_folders(user_uuid).await?;
        for folder in inventory_folders {
            let filename = format!("folders/{}.xml", folder.uuid);
            let folder_xml = self.create_inventory_folder_xml(&folder)?;
            zip.start_file(&filename, options)?;
            zip.write_all(folder_xml.as_bytes())?;
        }

        // Add assets referenced by inventory
        let assets = self.load_user_assets(user_uuid).await?;
        for asset in assets {
            let filename = format!("assets/{}", asset.uuid);
            zip.start_file(&filename, options)?;
            zip.write_all(&asset.data)?;
            
            // Add asset metadata
            let metadata_filename = format!("assets/{}_metadata.xml", asset.uuid);
            let metadata_xml = self.create_asset_metadata_xml(&asset)?;
            zip.start_file(&metadata_filename, options)?;
            zip.write_all(metadata_xml.as_bytes())?;
        }

        zip.finish()?;
        
        tracing::info!("Created IAR archive: {}", output_path.display());
        Ok(())
    }

    /// Load an IAR archive
    pub async fn load_iar_archive(&self, archive_path: &Path) -> Result<IARArchive> {
        let file = fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        // Extract all data in a single pass to avoid borrow conflicts
        let extracted_data = Self::extract_archive_data(&mut archive)?;
        
        let format_version = self.parse_iar_version(&extracted_data.manifest_content)?;

        // For IAR archives, the region_content field actually contains user profile data
        let user_info = self.parse_user_info(&extracted_data.region_content)?;

        // Parse inventory items and folders from extracted content
        let mut inventory_items = Vec::new();
        let mut inventory_folders = Vec::new();
        
        for content in &extracted_data.object_contents {
            // Try parsing as inventory item first
            if let Ok(item) = self.parse_inventory_item(content) {
                inventory_items.push(item);
            } else if let Ok(folder) = self.parse_inventory_folder(content) {
                inventory_folders.push(folder);
            }
        }

        // Parse assets from extracted data
        let mut assets = Vec::new();
        for (name, data, metadata_content) in &extracted_data.asset_data {
            let metadata = if let Some(meta_content) = metadata_content {
                self.parse_asset_metadata(meta_content)?
            } else {
                // Create default metadata
                AssetData {
                    uuid: name.split('/').last().unwrap_or("unknown").to_string(),
                    asset_type: 0,
                    name: "Unknown".to_string(),
                    description: String::new(),
                    data: data.clone(),
                    temporary: false,
                    local: false,
                }
            };
            assets.push(metadata);
        }

        Ok(IARArchive {
            format_version,
            user_info,
            inventory_items,
            inventory_folders,
            assets,
        })
    }

    // Placeholder methods for data loading (would integrate with actual database/storage)
    
    async fn load_region_objects(&self, _region_uuid: &str) -> Result<Vec<SceneObject>> {
        // Placeholder - would load from database
        Ok(Vec::new())
    }

    async fn load_region_assets(&self, _region_uuid: &str) -> Result<Vec<AssetData>> {
        // Placeholder - would load from asset service
        Ok(Vec::new())
    }

    async fn load_region_parcels(&self, _region_uuid: &str) -> Result<Vec<ParcelData>> {
        // Placeholder - would load from land database
        Ok(Vec::new())
    }

    async fn load_user_inventory_items(&self, _user_uuid: &str) -> Result<Vec<InventoryItem>> {
        // Placeholder - would load from inventory database
        Ok(Vec::new())
    }

    async fn load_user_inventory_folders(&self, _user_uuid: &str) -> Result<Vec<InventoryFolder>> {
        // Placeholder - would load from inventory database
        Ok(Vec::new())
    }

    async fn load_user_assets(&self, _user_uuid: &str) -> Result<Vec<AssetData>> {
        // Placeholder - would load user's assets
        Ok(Vec::new())
    }

    async fn load_user_profile(&self, _user_uuid: &str) -> Result<String> {
        // Placeholder - would load user profile XML
        Ok("<UserProfile></UserProfile>".to_string())
    }

    fn load_region_settings(&self, _region_uuid: &str) -> Result<String> {
        // Placeholder - would load region settings XML
        Ok("<RegionSettings></RegionSettings>".to_string())
    }

    fn load_terrain_data(&self, _region_uuid: &str) -> Result<Vec<u8>> {
        // Placeholder - would load terrain heightmap data
        Ok(vec![0u8; 256 * 256 * 4]) // 256x256 heightmap
    }

    // Placeholder parsing methods (would implement proper XML parsing)
    
    fn parse_oar_version(&self, _content: &str) -> Result<String> {
        Ok("1.0".to_string())
    }

    fn parse_iar_version(&self, _content: &str) -> Result<String> {
        Ok("1.0".to_string())
    }

    fn parse_region_info(&self, _content: &str) -> Result<RegionInfo> {
        Ok(RegionInfo {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "Default Region".to_string(),
            location_x: 1000,
            location_y: 1000,
            size_x: 256,
            size_y: 256,
            external_host_name: "127.0.0.1".to_string(),
            internal_port: 9000,
        })
    }

    fn parse_user_info(&self, _content: &str) -> Result<UserInfo> {
        Ok(UserInfo {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: None,
            created: "2024-01-01T00:00:00Z".to_string(),
        })
    }

    fn parse_scene_object(&self, _content: &str) -> Result<SceneObject> {
        Ok(SceneObject {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "Object".to_string(),
            description: String::new(),
            position: (128.0, 128.0, 20.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
            xml_data: _content.to_string(),
            owner_id: "00000000-0000-0000-0000-000000000000".to_string(),
            creator_id: "00000000-0000-0000-0000-000000000000".to_string(),
        })
    }

    fn parse_asset_metadata(&self, _content: &str) -> Result<AssetData> {
        Ok(AssetData {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            asset_type: 0,
            name: "Asset".to_string(),
            description: String::new(),
            data: Vec::new(),
            temporary: false,
            local: false,
        })
    }

    fn parse_parcel_data(&self, _content: &str) -> Result<ParcelData> {
        Ok(ParcelData {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "Parcel".to_string(),
            description: String::new(),
            owner_id: "00000000-0000-0000-0000-000000000000".to_string(),
            group_id: None,
            area: 4096,
            bitmap: vec![255u8; 64], // 64 bytes for 256x256 parcel bitmap
            flags: 0,
        })
    }

    fn parse_inventory_item(&self, _content: &str) -> Result<InventoryItem> {
        Ok(InventoryItem {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "Item".to_string(),
            description: String::new(),
            asset_type: 0,
            inventory_type: 0,
            folder_id: "00000000-0000-0000-0000-000000000000".to_string(),
            owner_id: "00000000-0000-0000-0000-000000000000".to_string(),
            creator_id: "00000000-0000-0000-0000-000000000000".to_string(),
            asset_id: "00000000-0000-0000-0000-000000000000".to_string(),
            base_permissions: 0x7FFFFFFF,
            current_permissions: 0x7FFFFFFF,
            everyone_permissions: 0,
            group_permissions: 0,
            next_permissions: 0x7FFFFFFF,
        })
    }

    fn parse_inventory_folder(&self, _content: &str) -> Result<InventoryFolder> {
        Ok(InventoryFolder {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            name: "Folder".to_string(),
            owner_id: "00000000-0000-0000-0000-000000000000".to_string(),
            parent_id: None,
            folder_type: -1,
            version: 1,
        })
    }

    // Placeholder XML generation methods
    
    fn create_oar_manifest(&self, _region_uuid: &str) -> Result<String> {
        Ok(r#"<?xml version="1.0" encoding="utf-8"?>
<archive major_version="1" minor_version="0">
  <creation_info>
    <datetime>2024-01-01T00:00:00Z</datetime>
    <id>OpenSim Next</id>
  </creation_info>
</archive>"#.to_string())
    }

    fn create_iar_manifest(&self, _user_uuid: &str) -> Result<String> {
        Ok(r#"<?xml version="1.0" encoding="utf-8"?>
<archive major_version="1" minor_version="0">
  <creation_info>
    <datetime>2024-01-01T00:00:00Z</datetime>
    <id>OpenSim Next</id>
  </creation_info>
</archive>"#.to_string())
    }

    fn create_asset_metadata_xml(&self, asset: &AssetData) -> Result<String> {
        Ok(format!(r#"<?xml version="1.0" encoding="utf-8"?>
<AssetMetadata>
  <ID>{}</ID>
  <Name>{}</Name>
  <Description>{}</Description>
  <Type>{}</Type>
  <Temporary>{}</Temporary>
  <Local>{}</Local>
</AssetMetadata>"#, 
            asset.uuid, asset.name, asset.description, 
            asset.asset_type, asset.temporary, asset.local))
    }

    fn create_parcel_xml(&self, parcel: &ParcelData) -> Result<String> {
        Ok(format!(r#"<?xml version="1.0" encoding="utf-8"?>
<LandData>
  <GlobalID>{}</GlobalID>
  <Name>{}</Name>
  <Description>{}</Description>
  <OwnerID>{}</OwnerID>
  <Area>{}</Area>
  <Flags>{}</Flags>
</LandData>"#, 
            parcel.uuid, parcel.name, parcel.description,
            parcel.owner_id, parcel.area, parcel.flags))
    }

    fn create_inventory_item_xml(&self, item: &InventoryItem) -> Result<String> {
        Ok(format!(r#"<?xml version="1.0" encoding="utf-8"?>
<InventoryItem>
  <ID>{}</ID>
  <InvType>{}</InvType>
  <AssetType>{}</AssetType>
  <AssetID>{}</AssetID>
  <Name>{}</Name>
  <Description>{}</Description>
  <NextPermissions>{}</NextPermissions>
  <CurrentPermissions>{}</CurrentPermissions>
  <BasePermissions>{}</BasePermissions>
  <EveryonePermissions>{}</EveryonePermissions>
  <GroupPermissions>{}</GroupPermissions>
  <OwnerID>{}</OwnerID>
  <CreatorID>{}</CreatorID>
  <FolderID>{}</FolderID>
</InventoryItem>"#,
            item.uuid, item.inventory_type, item.asset_type, item.asset_id,
            item.name, item.description, item.next_permissions, item.current_permissions,
            item.base_permissions, item.everyone_permissions, item.group_permissions,
            item.owner_id, item.creator_id, item.folder_id))
    }

    fn create_inventory_folder_xml(&self, folder: &InventoryFolder) -> Result<String> {
        let default_uuid = "00000000-0000-0000-0000-000000000000".to_string();
        let parent_id = folder.parent_id.as_ref().unwrap_or(&default_uuid);
        Ok(format!(r#"<?xml version="1.0" encoding="utf-8"?>
<InventoryFolder>
  <ID>{}</ID>
  <Name>{}</Name>
  <Owner>{}</Owner>
  <ParentID>{}</ParentID>
  <Type>{}</Type>
  <Version>{}</Version>
</InventoryFolder>"#,
            folder.uuid, folder.name, folder.owner_id,
            parent_id, folder.folder_type, folder.version))
    }
}