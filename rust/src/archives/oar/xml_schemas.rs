//! XML schema definitions for OAR format
//!
//! Based on OpenSim ArchiveReadRequest.cs and ArchiveWriteRequest.cs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Archive metadata from archive.xml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "archive")]
pub struct OarArchiveXml {
    #[serde(rename = "@major_version")]
    pub major_version: u32,
    #[serde(rename = "@minor_version")]
    pub minor_version: u32,
}

/// Region settings from settings/*.xml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename = "RegionSettings")]
pub struct OarRegionSettings {
    #[serde(rename = "General", default)]
    pub general: OarRegionGeneral,
    #[serde(rename = "GroundTextures", default)]
    pub ground_textures: OarGroundTextures,
    #[serde(rename = "Terrain", default)]
    pub terrain: OarTerrainSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarRegionGeneral {
    #[serde(rename = "AllowDamage", default)]
    pub allow_damage: bool,
    #[serde(rename = "AllowLandResell", default)]
    pub allow_land_resell: bool,
    #[serde(rename = "AllowLandJoinDivide", default)]
    pub allow_land_join_divide: bool,
    #[serde(rename = "BlockFly", default)]
    pub block_fly: bool,
    #[serde(rename = "BlockLandShowInSearch", default)]
    pub block_land_show_in_search: bool,
    #[serde(rename = "BlockTerraform", default)]
    pub block_terraform: bool,
    #[serde(rename = "DisableCollisions", default)]
    pub disable_collisions: bool,
    #[serde(rename = "DisablePhysics", default)]
    pub disable_physics: bool,
    #[serde(rename = "DisableScripts", default)]
    pub disable_scripts: bool,
    #[serde(rename = "MaturityRating", default)]
    pub maturity_rating: i32,
    #[serde(rename = "RestrictPushing", default)]
    pub restrict_pushing: bool,
    #[serde(rename = "AgentLimit", default)]
    pub agent_limit: i32,
    #[serde(rename = "ObjectBonus", default)]
    pub object_bonus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarGroundTextures {
    #[serde(rename = "Texture1", default)]
    pub texture1: String,
    #[serde(rename = "Texture2", default)]
    pub texture2: String,
    #[serde(rename = "Texture3", default)]
    pub texture3: String,
    #[serde(rename = "Texture4", default)]
    pub texture4: String,
    #[serde(rename = "ElevationLowSW", default)]
    pub elevation_low_sw: f64,
    #[serde(rename = "ElevationLowNW", default)]
    pub elevation_low_nw: f64,
    #[serde(rename = "ElevationLowSE", default)]
    pub elevation_low_se: f64,
    #[serde(rename = "ElevationLowNE", default)]
    pub elevation_low_ne: f64,
    #[serde(rename = "ElevationHighSW", default)]
    pub elevation_high_sw: f64,
    #[serde(rename = "ElevationHighNW", default)]
    pub elevation_high_nw: f64,
    #[serde(rename = "ElevationHighSE", default)]
    pub elevation_high_se: f64,
    #[serde(rename = "ElevationHighNE", default)]
    pub elevation_high_ne: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarTerrainSettings {
    #[serde(rename = "WaterHeight", default)]
    pub water_height: f64,
    #[serde(rename = "TerrainRaiseLimit", default)]
    pub terrain_raise_limit: f64,
    #[serde(rename = "TerrainLowerLimit", default)]
    pub terrain_lower_limit: f64,
    #[serde(rename = "UseEstateSun", default)]
    pub use_estate_sun: bool,
    #[serde(rename = "FixedSun", default)]
    pub fixed_sun: bool,
    #[serde(rename = "SunPosition", default)]
    pub sun_position: f64,
}

/// Parcel/land data from landdata/*.xml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "LandData")]
pub struct OarLandData {
    #[serde(rename = "GlobalID")]
    pub global_id: String,
    #[serde(rename = "LocalID")]
    pub local_id: i32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Description", default)]
    pub description: Option<String>,
    #[serde(rename = "OwnerID")]
    pub owner_id: String,
    #[serde(rename = "GroupID", default)]
    pub group_id: Option<String>,
    #[serde(rename = "IsGroupOwned", default)]
    pub is_group_owned: bool,
    #[serde(rename = "Area")]
    pub area: i32,
    #[serde(rename = "AuctionID", default)]
    pub auction_id: i32,
    #[serde(rename = "Category", default)]
    pub category: i32,
    #[serde(rename = "ClaimDate", default)]
    pub claim_date: i64,
    #[serde(rename = "ClaimPrice", default)]
    pub claim_price: i32,
    #[serde(rename = "Flags", default)]
    pub flags: u32,
    #[serde(rename = "LandingType", default)]
    pub landing_type: i32,
    #[serde(rename = "MediaAutoScale", default)]
    pub media_auto_scale: bool,
    #[serde(rename = "MediaID", default)]
    pub media_id: Option<String>,
    #[serde(rename = "MediaURL", default)]
    pub media_url: Option<String>,
    #[serde(rename = "MusicURL", default)]
    pub music_url: Option<String>,
    #[serde(rename = "PassHours", default)]
    pub pass_hours: f32,
    #[serde(rename = "PassPrice", default)]
    pub pass_price: i32,
    #[serde(rename = "SalePrice", default)]
    pub sale_price: i32,
    #[serde(rename = "SnapshotID", default)]
    pub snapshot_id: Option<String>,
    #[serde(rename = "UserLocation", default)]
    pub user_location: Option<String>,
    #[serde(rename = "UserLookAt", default)]
    pub user_look_at: Option<String>,
    #[serde(rename = "Bitmap", default)]
    pub bitmap: Option<String>,
}

/// Wrapper for nested UUID elements like <CreatorID><UUID>hex</UUID></CreatorID>
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UuidField {
    #[serde(rename = "UUID", default)]
    pub uuid: String,
}

impl UuidField {
    pub fn parse(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.uuid).ok()
    }

    pub fn parse_or_nil(&self) -> Uuid {
        Uuid::parse_str(&self.uuid).unwrap_or(Uuid::nil())
    }
}

/// Scene object group from objects/*.xml
/// C# ToOriginalXmlFormat uses <RootPart><SceneObjectPart>...</SceneObjectPart></RootPart>
/// and <OtherParts><Part><SceneObjectPart>...</SceneObjectPart></Part></OtherParts>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "SceneObjectGroup")]
pub struct OarSceneObjectGroup {
    #[serde(rename = "RootPart")]
    pub root_part_wrapper: OarRootPartWrapper,
    #[serde(rename = "OtherParts", default)]
    pub other_parts: Option<OarOtherParts>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OarRootPartWrapper {
    #[serde(rename = "SceneObjectPart")]
    pub part: OarSceneObjectPart,
}

impl OarSceneObjectGroup {
    pub fn root_part(&self) -> &OarSceneObjectPart {
        &self.root_part_wrapper.part
    }

    pub fn all_parts(&self) -> Vec<&OarSceneObjectPart> {
        let mut parts = vec![self.root_part()];
        if let Some(ref other) = self.other_parts {
            for wrapper in &other.parts {
                parts.push(&wrapper.part);
            }
        }
        parts
    }

    pub fn root_uuid(&self) -> Uuid {
        self.root_part().uuid.parse_or_nil()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OarPartWrapper {
    #[serde(rename = "SceneObjectPart")]
    pub part: OarSceneObjectPart,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarOtherParts {
    #[serde(rename = "Part", default)]
    pub parts: Vec<OarPartWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OarSceneObjectPart {
    #[serde(rename = "AllowedDrop", default)]
    pub allowed_drop: bool,
    #[serde(rename = "CreatorID", default)]
    pub creator_id: UuidField,
    #[serde(rename = "CreatorData", default)]
    pub creator_data: Option<String>,
    #[serde(rename = "FolderID", default)]
    pub folder_id: UuidField,
    #[serde(rename = "InventorySerial", default)]
    pub inventory_serial: i32,
    #[serde(rename = "UUID")]
    pub uuid: UuidField,
    #[serde(rename = "LocalId", default)]
    pub local_id: u32,
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "Material", default)]
    pub material: i32,
    #[serde(rename = "PassTouches", default)]
    pub pass_touches: bool,
    #[serde(rename = "PassCollisions", default)]
    pub pass_collisions: bool,
    #[serde(rename = "RegionHandle", default)]
    pub region_handle: u64,
    #[serde(rename = "ScriptAccessPin", default)]
    pub script_access_pin: i32,
    #[serde(rename = "GroupPosition", default)]
    pub group_position: OarVector3,
    #[serde(rename = "OffsetPosition", default)]
    pub offset_position: OarVector3,
    #[serde(rename = "RotationOffset", default)]
    pub rotation_offset: OarQuaternion,
    #[serde(rename = "Velocity", default)]
    pub velocity: OarVector3,
    #[serde(rename = "AngularVelocity", default)]
    pub angular_velocity: OarVector3,
    #[serde(rename = "Acceleration", default)]
    pub acceleration: OarVector3,
    #[serde(rename = "Description", default)]
    pub description: String,
    #[serde(rename = "Color", default)]
    pub color: Option<OarColor>,
    #[serde(rename = "Text", default)]
    pub text: String,
    #[serde(rename = "SitName", default)]
    pub sit_name: String,
    #[serde(rename = "TouchName", default)]
    pub touch_name: String,
    #[serde(rename = "LinkNum", default)]
    pub link_num: i32,
    #[serde(rename = "ClickAction", default)]
    pub click_action: i32,
    #[serde(rename = "Shape")]
    pub shape: OarPrimitiveShape,
    #[serde(rename = "Scale", default)]
    pub scale: OarVector3,
    #[serde(rename = "SitTargetOrientation", default)]
    pub sit_target_orientation: OarQuaternion,
    #[serde(rename = "SitTargetPosition", default)]
    pub sit_target_position: OarVector3,
    #[serde(rename = "SitTargetPositionLL", default)]
    pub sit_target_position_ll: Option<OarVector3>,
    #[serde(rename = "SitTargetOrientationLL", default)]
    pub sit_target_orientation_ll: Option<OarQuaternion>,
    #[serde(rename = "ParentID", default)]
    pub parent_id: u32,
    #[serde(rename = "CreationDate", default)]
    pub creation_date: i64,
    #[serde(rename = "Category", default)]
    pub category: i32,
    #[serde(rename = "SalePrice", default)]
    pub sale_price: i32,
    #[serde(rename = "ObjectSaleType", default)]
    pub object_sale_type: i32,
    #[serde(rename = "OwnershipCost", default)]
    pub ownership_cost: i32,
    #[serde(rename = "GroupID", default)]
    pub group_id: UuidField,
    #[serde(rename = "OwnerID", default)]
    pub owner_id: UuidField,
    #[serde(rename = "LastOwnerID", default)]
    pub last_owner_id: UuidField,
    #[serde(rename = "RezzerID", default)]
    pub rezzer_id: Option<UuidField>,
    #[serde(rename = "BaseMask", default)]
    pub base_mask: u32,
    #[serde(rename = "OwnerMask", default)]
    pub owner_mask: u32,
    #[serde(rename = "GroupMask", default)]
    pub group_mask: u32,
    #[serde(rename = "EveryoneMask", default)]
    pub everyone_mask: u32,
    #[serde(rename = "NextOwnerMask", default)]
    pub next_owner_mask: u32,
    #[serde(rename = "Flags", default)]
    pub flags: String,
    #[serde(rename = "CollisionSound", default)]
    pub collision_sound: Option<UuidField>,
    #[serde(rename = "CollisionSoundVolume", default)]
    pub collision_sound_volume: f32,
    #[serde(rename = "TextureAnimation", default)]
    pub texture_animation: Option<String>,
    #[serde(rename = "ParticleSystem", default)]
    pub particle_system: Option<String>,
    #[serde(rename = "PayPrice0", default)]
    pub pay_price_0: i32,
    #[serde(rename = "PayPrice1", default)]
    pub pay_price_1: i32,
    #[serde(rename = "PayPrice2", default)]
    pub pay_price_2: i32,
    #[serde(rename = "PayPrice3", default)]
    pub pay_price_3: i32,
    #[serde(rename = "PayPrice4", default)]
    pub pay_price_4: i32,
    #[serde(rename = "TaskInventory", default)]
    pub task_inventory: Option<OarTaskInventory>,
}

impl OarSceneObjectPart {
    pub fn flags_as_u32(&self) -> u32 {
        if self.flags == "None" || self.flags.is_empty() {
            0
        } else {
            self.flags.parse::<u32>().unwrap_or(0)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarColor {
    #[serde(rename = "R", default)]
    pub r: u8,
    #[serde(rename = "G", default)]
    pub g: u8,
    #[serde(rename = "B", default)]
    pub b: u8,
    #[serde(rename = "A", default)]
    pub a: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarTaskInventory {
    #[serde(rename = "TaskInventoryItem", default)]
    pub items: Vec<OarTaskInventoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarTaskInventoryItem {
    #[serde(rename = "AssetID", default)]
    pub asset_id: UuidField,
    #[serde(rename = "ItemID", default)]
    pub item_id: UuidField,
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "Type", default)]
    pub item_type: i32,
    #[serde(rename = "InvType", default)]
    pub inv_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarPrimitiveShape {
    #[serde(rename = "ProfileCurve", default)]
    pub profile_curve: i32,
    #[serde(rename = "TextureEntry", default)]
    pub texture_entry: String,
    #[serde(rename = "ExtraParams", default)]
    pub extra_params: String,
    #[serde(rename = "ProfileBegin", default)]
    pub profile_begin: u16,
    #[serde(rename = "ProfileEnd", default)]
    pub profile_end: u16,
    #[serde(rename = "ProfileHollow", default)]
    pub profile_hollow: u16,
    #[serde(rename = "PathCurve", default)]
    pub path_curve: i32,
    #[serde(rename = "PathBegin", default)]
    pub path_begin: u16,
    #[serde(rename = "PathEnd", default)]
    pub path_end: u16,
    #[serde(rename = "PathScaleX", default)]
    pub path_scale_x: i32,
    #[serde(rename = "PathScaleY", default)]
    pub path_scale_y: i32,
    #[serde(rename = "PathShearX", default)]
    pub path_shear_x: i32,
    #[serde(rename = "PathShearY", default)]
    pub path_shear_y: i32,
    #[serde(rename = "PathTwist", default)]
    pub path_twist: i32,
    #[serde(rename = "PathTwistBegin", default)]
    pub path_twist_begin: i32,
    #[serde(rename = "PathRadiusOffset", default)]
    pub path_radius_offset: i32,
    #[serde(rename = "PathTaperX", default)]
    pub path_taper_x: i32,
    #[serde(rename = "PathTaperY", default)]
    pub path_taper_y: i32,
    #[serde(rename = "PathRevolutions", default)]
    pub path_revolutions: i32,
    #[serde(rename = "PathSkew", default)]
    pub path_skew: i32,
    #[serde(rename = "PCode", default)]
    pub pcode: i32,
    #[serde(rename = "State", default)]
    pub state: i32,
    #[serde(rename = "ProfileShape", default)]
    pub profile_shape: String,
    #[serde(rename = "HollowShape", default)]
    pub hollow_shape: String,
    #[serde(rename = "SculptTexture", default)]
    pub sculpt_texture: Option<UuidField>,
    #[serde(rename = "SculptType", default)]
    pub sculpt_type: i32,
    #[serde(rename = "SculptData", default)]
    pub sculpt_data: Option<String>,
    #[serde(rename = "FlexiSoftness", default)]
    pub flexi_softness: i32,
    #[serde(rename = "FlexiTension", default)]
    pub flexi_tension: f32,
    #[serde(rename = "FlexiDrag", default)]
    pub flexi_drag: f32,
    #[serde(rename = "FlexiGravity", default)]
    pub flexi_gravity: f32,
    #[serde(rename = "FlexiWind", default)]
    pub flexi_wind: f32,
    #[serde(rename = "FlexiForceX", default)]
    pub flexi_force_x: f32,
    #[serde(rename = "FlexiForceY", default)]
    pub flexi_force_y: f32,
    #[serde(rename = "FlexiForceZ", default)]
    pub flexi_force_z: f32,
    #[serde(rename = "LightColorR", default)]
    pub light_color_r: f32,
    #[serde(rename = "LightColorG", default)]
    pub light_color_g: f32,
    #[serde(rename = "LightColorB", default)]
    pub light_color_b: f32,
    #[serde(rename = "LightColorA", default)]
    pub light_color_a: f32,
    #[serde(rename = "LightRadius", default)]
    pub light_radius: f32,
    #[serde(rename = "LightCutoff", default)]
    pub light_cutoff: f32,
    #[serde(rename = "LightFalloff", default)]
    pub light_falloff: f32,
    #[serde(rename = "LightIntensity", default)]
    pub light_intensity: f32,
    #[serde(rename = "FlexiEntry", default)]
    pub flexi_entry: bool,
    #[serde(rename = "LightEntry", default)]
    pub light_entry: bool,
    #[serde(rename = "SculptEntry", default)]
    pub sculpt_entry: bool,
    #[serde(rename = "Media", default)]
    pub media: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarVector3 {
    #[serde(rename = "X", default)]
    pub x: f32,
    #[serde(rename = "Y", default)]
    pub y: f32,
    #[serde(rename = "Z", default)]
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OarQuaternion {
    #[serde(rename = "X", default)]
    pub x: f32,
    #[serde(rename = "Y", default)]
    pub y: f32,
    #[serde(rename = "Z", default)]
    pub z: f32,
    #[serde(rename = "W", default)]
    pub w: f32,
}

/// Create archive.xml content for OAR
pub fn create_oar_archive_xml(major_version: u32, minor_version: u32) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<archive major_version="{}" minor_version="{}">
</archive>"#,
        major_version, minor_version
    )
}

/// Create region settings XML
pub fn create_region_settings_xml(settings: &OarRegionSettings) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<RegionSettings>
    <General>
        <AllowDamage>{}</AllowDamage>
        <AllowLandResell>{}</AllowLandResell>
        <AllowLandJoinDivide>{}</AllowLandJoinDivide>
        <BlockFly>{}</BlockFly>
        <BlockLandShowInSearch>{}</BlockLandShowInSearch>
        <BlockTerraform>{}</BlockTerraform>
        <DisableCollisions>{}</DisableCollisions>
        <DisablePhysics>{}</DisablePhysics>
        <DisableScripts>{}</DisableScripts>
        <MaturityRating>{}</MaturityRating>
        <RestrictPushing>{}</RestrictPushing>
        <AgentLimit>{}</AgentLimit>
        <ObjectBonus>{}</ObjectBonus>
    </General>
    <GroundTextures>
        <Texture1>{}</Texture1>
        <Texture2>{}</Texture2>
        <Texture3>{}</Texture3>
        <Texture4>{}</Texture4>
        <ElevationLowSW>{}</ElevationLowSW>
        <ElevationLowNW>{}</ElevationLowNW>
        <ElevationLowSE>{}</ElevationLowSE>
        <ElevationLowNE>{}</ElevationLowNE>
        <ElevationHighSW>{}</ElevationHighSW>
        <ElevationHighNW>{}</ElevationHighNW>
        <ElevationHighSE>{}</ElevationHighSE>
        <ElevationHighNE>{}</ElevationHighNE>
    </GroundTextures>
    <Terrain>
        <WaterHeight>{}</WaterHeight>
        <TerrainRaiseLimit>{}</TerrainRaiseLimit>
        <TerrainLowerLimit>{}</TerrainLowerLimit>
        <UseEstateSun>{}</UseEstateSun>
        <FixedSun>{}</FixedSun>
        <SunPosition>{}</SunPosition>
    </Terrain>
</RegionSettings>"#,
        settings.general.allow_damage,
        settings.general.allow_land_resell,
        settings.general.allow_land_join_divide,
        settings.general.block_fly,
        settings.general.block_land_show_in_search,
        settings.general.block_terraform,
        settings.general.disable_collisions,
        settings.general.disable_physics,
        settings.general.disable_scripts,
        settings.general.maturity_rating,
        settings.general.restrict_pushing,
        settings.general.agent_limit,
        settings.general.object_bonus,
        settings.ground_textures.texture1,
        settings.ground_textures.texture2,
        settings.ground_textures.texture3,
        settings.ground_textures.texture4,
        settings.ground_textures.elevation_low_sw,
        settings.ground_textures.elevation_low_nw,
        settings.ground_textures.elevation_low_se,
        settings.ground_textures.elevation_low_ne,
        settings.ground_textures.elevation_high_sw,
        settings.ground_textures.elevation_high_nw,
        settings.ground_textures.elevation_high_se,
        settings.ground_textures.elevation_high_ne,
        settings.terrain.water_height,
        settings.terrain.terrain_raise_limit,
        settings.terrain.terrain_lower_limit,
        settings.terrain.use_estate_sun,
        settings.terrain.fixed_sun,
        settings.terrain.sun_position,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_region_settings() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<RegionSettings>
    <General>
        <AllowDamage>false</AllowDamage>
        <AgentLimit>100</AgentLimit>
    </General>
</RegionSettings>"#;

        let settings: OarRegionSettings = quick_xml::de::from_str(xml).unwrap();
        assert!(!settings.general.allow_damage);
        assert_eq!(settings.general.agent_limit, 100);
    }
}
