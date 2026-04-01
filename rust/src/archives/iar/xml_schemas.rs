//! XML schema definitions for IAR format
//!
//! Based on OpenSim InventoryArchiveReadRequest.cs and InventoryArchiveWriteRequest.cs

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

fn deserialize_bool_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        _ => Ok(false),
    }
}

/// Archive metadata from archive.xml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "archive")]
pub struct IarArchiveXml {
    #[serde(rename = "@major_version")]
    pub major_version: u32,
    #[serde(rename = "@minor_version")]
    pub minor_version: u32,
}

/// Folder metadata from __folder_metadata.xml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "folder")]
pub struct IarFolderXml {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Type")]
    pub folder_type: i32,
    #[serde(rename = "Owner")]
    pub owner_id: String,
}

impl IarFolderXml {
    pub fn id_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.id).ok()
    }

    pub fn owner_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.owner_id).ok()
    }
}

/// Inventory item from individual .xml files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "InventoryItem")]
pub struct IarInventoryItemXml {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "InvType", default)]
    pub inv_type: i32,
    #[serde(rename = "CreatorUUID", default)]
    pub creator_id: String,
    #[serde(rename = "CreatorData", default)]
    pub creator_data: Option<String>,
    #[serde(rename = "Owner", default)]
    pub owner_id: String,
    #[serde(rename = "Description", default)]
    pub description: Option<String>,
    #[serde(rename = "AssetID", default)]
    pub asset_id: String,
    #[serde(rename = "AssetType", default)]
    pub asset_type: i32,
    #[serde(rename = "CurrentPermissions", default)]
    pub current_permissions: u32,
    #[serde(rename = "BasePermissions", default)]
    pub base_permissions: u32,
    #[serde(rename = "EveryOnePermissions", default)]
    pub everyone_permissions: u32,
    #[serde(rename = "NextPermissions", default)]
    pub next_permissions: u32,
    #[serde(rename = "GroupPermissions", default)]
    pub group_permissions: u32,
    #[serde(rename = "GroupID", default)]
    pub group_id: String,
    #[serde(rename = "GroupOwned", default, deserialize_with = "deserialize_bool_string")]
    pub group_owned: bool,
    #[serde(rename = "SalePrice", default)]
    pub sale_price: i32,
    #[serde(rename = "SaleType", default)]
    pub sale_type: u8,
    #[serde(rename = "Flags", default)]
    pub flags: u32,
    #[serde(rename = "CreationDate", default)]
    pub creation_date: i32,
}

impl IarInventoryItemXml {
    pub fn id_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.id).ok()
    }

    pub fn owner_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.owner_id).ok()
    }

    pub fn asset_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.asset_id).ok()
    }

    pub fn creator_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.creator_id).ok()
    }

    pub fn group_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.group_id).ok()
    }
}

/// Create archive.xml content
pub fn create_archive_xml(major_version: u32, minor_version: u32) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<archive major_version="{}" minor_version="{}">
</archive>"#,
        major_version, minor_version
    )
}

/// Create folder metadata XML
pub fn create_folder_xml(
    name: &str,
    id: &Uuid,
    folder_type: i32,
    owner_id: &Uuid,
) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<folder>
    <Name>{}</Name>
    <ID>{}</ID>
    <Type>{}</Type>
    <Owner>{}</Owner>
</folder>"#,
        xml_escape(name),
        id,
        folder_type,
        owner_id
    )
}

/// Create inventory item XML
pub fn create_item_xml(item: &IarInventoryItemXml) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<InventoryItem>
    <Name>{}</Name>
    <ID>{}</ID>
    <InvType>{}</InvType>
    <CreatorUUID>{}</CreatorUUID>
    <CreatorData>{}</CreatorData>
    <Owner>{}</Owner>
    <Description>{}</Description>
    <AssetID>{}</AssetID>
    <AssetType>{}</AssetType>
    <CurrentPermissions>{}</CurrentPermissions>
    <BasePermissions>{}</BasePermissions>
    <EveryOnePermissions>{}</EveryOnePermissions>
    <NextPermissions>{}</NextPermissions>
    <GroupPermissions>{}</GroupPermissions>
    <GroupID>{}</GroupID>
    <GroupOwned>{}</GroupOwned>
    <SalePrice>{}</SalePrice>
    <SaleType>{}</SaleType>
    <Flags>{}</Flags>
    <CreationDate>{}</CreationDate>
</InventoryItem>"#,
        xml_escape(&item.name),
        item.id,
        item.inv_type,
        item.creator_id,
        item.creator_data.as_deref().unwrap_or(""),
        item.owner_id,
        xml_escape(item.description.as_deref().unwrap_or("")),
        item.asset_id,
        item.asset_type,
        item.current_permissions,
        item.base_permissions,
        item.everyone_permissions,
        item.next_permissions,
        item.group_permissions,
        item.group_id,
        item.group_owned,
        item.sale_price,
        item.sale_type,
        item.flags,
        item.creation_date
    )
}

/// Escape XML special characters
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_folder_xml() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<folder>
    <Name>My Folder</Name>
    <ID>12345678-1234-5678-9abc-123456789abc</ID>
    <Type>-1</Type>
    <Owner>87654321-4321-8765-cba9-876543210987</Owner>
</folder>"#;

        let folder: IarFolderXml = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(folder.name, "My Folder");
        assert_eq!(folder.folder_type, -1);
        assert!(folder.id_uuid().is_some());
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("Test & <Value>"), "Test &amp; &lt;Value&gt;");
    }
}
