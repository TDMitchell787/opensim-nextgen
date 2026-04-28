// Phase 26.2.1: Region Data Model
// Comprehensive region data structures for OpenSim compatibility

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core region information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionInfo {
    pub region_id: Uuid,
    pub region_name: String,
    pub region_handle: u64,
    pub location_x: u32,
    pub location_y: u32,
    pub size_x: u32,
    pub size_y: u32,
    pub internal_ip: String,
    pub internal_port: u32,
    pub external_host_name: String,
    pub master_avatar_id: Uuid,
    pub owner_id: Option<Uuid>,
    pub estate_id: u32,
    pub scope_id: Uuid,
    pub region_secret: String,
    pub token: String,
    pub flags: u32,
    pub maturity: u8, // 0=PG, 1=Mature, 2=Adult
    pub last_seen: DateTime<Utc>,
    pub prim_count: u32,
    pub agent_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RegionInfo {
    /// Create a new region with default values
    pub fn new(name: String, location_x: u32, location_y: u32) -> Self {
        let now = Utc::now();
        let region_id = Uuid::new_v4();

        Self {
            region_id,
            region_name: name,
            region_handle: Self::calculate_handle(location_x, location_y),
            location_x,
            location_y,
            size_x: 256,
            size_y: 256,
            internal_ip: "127.0.0.1".to_string(),
            internal_port: 9000,
            external_host_name: "localhost".to_string(),
            master_avatar_id: Uuid::nil(),
            owner_id: None,
            estate_id: 1,
            scope_id: Uuid::nil(),
            region_secret: Uuid::new_v4().to_string(),
            token: Uuid::new_v4().to_string(),
            flags: 0,
            maturity: 1, // Default to Mature
            last_seen: now,
            prim_count: 0,
            agent_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate region handle from grid coordinates (converts grid → world meters)
    /// Grid coord 1000 → 256000 meters (1000 * 256)
    pub fn calculate_handle(grid_x: u32, grid_y: u32) -> u64 {
        let world_x = (grid_x as u64) * 256;
        let world_y = (grid_y as u64) * 256;
        (world_x << 32) | world_y
    }

    /// Get world meter coordinates from handle
    pub fn handle_to_coords(handle: u64) -> (u32, u32) {
        let world_x = (handle >> 32) as u32;
        let world_y = (handle & 0xFFFFFFFF) as u32;
        (world_x, world_y)
    }

    /// Get grid coordinates from handle
    pub fn handle_to_grid_coords(handle: u64) -> (u32, u32) {
        let (world_x, world_y) = Self::handle_to_coords(handle);
        (world_x / 256, world_y / 256)
    }
}

/// Region settings for environment and policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionSettings {
    pub region_id: Uuid,
    pub block_terraform: bool,
    pub block_fly: bool,
    pub allow_damage: bool,
    pub restrict_pushing: bool,
    pub allow_land_resell: bool,
    pub allow_land_join_divide: bool,
    pub block_show_in_search: bool,
    pub agent_limit: u32,
    pub object_bonus: f32,
    pub maturity: u32,
    pub disable_scripts: bool,
    pub disable_collisions: bool,
    pub disable_physics: bool,
    pub terrain_texture_1: Option<Uuid>,
    pub terrain_texture_2: Option<Uuid>,
    pub terrain_texture_3: Option<Uuid>,
    pub terrain_texture_4: Option<Uuid>,
    pub elevation_1_nw: f32,
    pub elevation_2_nw: f32,
    pub elevation_1_ne: f32,
    pub elevation_2_ne: f32,
    pub elevation_1_se: f32,
    pub elevation_2_se: f32,
    pub elevation_1_sw: f32,
    pub elevation_2_sw: f32,
    pub water_height: f32,
    pub terrain_raise_limit: f32,
    pub terrain_lower_limit: f32,
    pub use_estate_sun: bool,
    pub fixed_sun: bool,
    pub sun_position: f32,
    pub covenant: Option<Uuid>,
    pub covenant_datetime: i32,
    pub sandbox: bool,
    pub sunvectorx: f32,
    pub sunvectory: f32,
    pub sunvectorz: f32,
    pub loaded_creation_id: String,
    pub loaded_creation_datetime: i32,
    pub map_tile_id: Option<Uuid>,
    pub telehub_id: Option<Uuid>,
    pub spawn_point_routing: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for RegionSettings {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            region_id: Uuid::nil(),
            block_terraform: false,
            block_fly: false,
            allow_damage: false,
            restrict_pushing: false,
            allow_land_resell: true,
            allow_land_join_divide: true,
            block_show_in_search: false,
            agent_limit: 40,
            object_bonus: 1.0,
            maturity: 1,
            disable_scripts: false,
            disable_collisions: false,
            disable_physics: false,
            terrain_texture_1: None,
            terrain_texture_2: None,
            terrain_texture_3: None,
            terrain_texture_4: None,
            elevation_1_nw: 10.0,
            elevation_2_nw: 10.0,
            elevation_1_ne: 10.0,
            elevation_2_ne: 10.0,
            elevation_1_se: 10.0,
            elevation_2_se: 10.0,
            elevation_1_sw: 10.0,
            elevation_2_sw: 10.0,
            water_height: 20.0,
            terrain_raise_limit: 100.0,
            terrain_lower_limit: -100.0,
            use_estate_sun: true,
            fixed_sun: false,
            sun_position: 0.0,
            covenant: None,
            covenant_datetime: 0,
            sandbox: false,
            sunvectorx: 1.0,
            sunvectory: 0.0,
            sunvectorz: 0.3,
            loaded_creation_id: String::new(),
            loaded_creation_datetime: 0,
            map_tile_id: None,
            telehub_id: None,
            spawn_point_routing: 1,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 3D Vector for positions, rotations, etc.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }
}

impl Default for Vector3 {
    fn default() -> Self {
        Self::zero()
    }
}

/// Quaternion for rotations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::identity()
    }
}

/// Scene object part (prim) data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneObjectPart {
    pub uuid: Uuid,
    pub parent_id: u32,
    pub creation_date: i32,
    pub name: String,
    pub description: String,
    pub sit_name: String,
    pub touch_name: String,
    pub object_flags: u32,
    pub creator_id: Uuid,
    pub owner_id: Uuid,
    pub group_id: Uuid,
    pub last_owner_id: Uuid,
    pub region_handle: u64,
    pub group_position: Vector3,
    pub offset_position: Vector3,
    pub rotation_offset: Quaternion,
    pub velocity: Vector3,
    pub angular_velocity: Vector3,
    pub acceleration: Vector3,
    pub scale: Vector3,
    pub sit_target_position: Vector3,
    pub sit_target_orientation: Quaternion,
    pub physics_type: u32,
    pub material: u32,
    pub click_action: u32,
    pub color: Vector3,
    pub alpha: f32,
    pub texture_entry: Vec<u8>,
    pub extra_physics_data: Vec<u8>,
    pub shape_data: String, // JSON serialized
    pub script_state: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SceneObjectPart {
    pub fn new(name: String, position: Vector3, creator_id: Uuid, owner_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4(),
            parent_id: 0,
            creation_date: now.timestamp() as i32,
            name,
            description: String::new(),
            sit_name: String::new(),
            touch_name: String::new(),
            object_flags: 0,
            creator_id,
            owner_id,
            group_id: Uuid::nil(),
            last_owner_id: Uuid::nil(),
            region_handle: 0,
            group_position: position,
            offset_position: Vector3::zero(),
            rotation_offset: Quaternion::identity(),
            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            acceleration: Vector3::zero(),
            scale: Vector3::one(),
            sit_target_position: Vector3::zero(),
            sit_target_orientation: Quaternion::identity(),
            physics_type: 0,
            material: 3, // Default material
            click_action: 0,
            color: Vector3::zero(),
            alpha: 0.0,
            texture_entry: Vec::new(),
            extra_physics_data: Vec::new(),
            shape_data: String::new(),
            script_state: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Terrain data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainData {
    pub region_id: Uuid,
    pub terrain_data: Vec<u8>, // Heightfield data
    pub terrain_revision: u32,
    pub terrain_seed: u32,
    pub water_height: f32,
    pub terrain_raise_limit: f32,
    pub terrain_lower_limit: f32,
    pub use_estate_sun: bool,
    pub fixed_sun: bool,
    pub sun_position: f32,
    pub covenant: Option<Uuid>,
    pub covenant_timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TerrainData {
    pub fn new(region_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            region_id,
            terrain_data: vec![0u8; 256 * 256 * 4], // Default 256x256 heightmap
            terrain_revision: 1,
            terrain_seed: 0,
            water_height: 20.0,
            terrain_raise_limit: 100.0,
            terrain_lower_limit: -100.0,
            use_estate_sun: true,
            fixed_sun: false,
            sun_position: 0.0,
            covenant: None,
            covenant_timestamp: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Land parcel data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandData {
    pub uuid: Uuid,
    pub region_id: Uuid,
    pub local_land_id: u32,
    pub bitmap: Vec<u8>,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub group_id: Option<Uuid>,
    pub is_group_owned: bool,
    pub area: u32,
    pub auction_id: u32,
    pub category: u32,
    pub claim_date: i32,
    pub claim_price: i32,
    pub status: u32,
    pub landing_type: u32,
    pub landing_position: Vector3,
    pub landing_look_at: Vector3,
    pub user_location: Vector3,
    pub user_look_at: Vector3,
    pub auth_buyer_id: Option<Uuid>,
    pub snapshot_id: Option<Uuid>,
    pub other_clean_time: i32,
    pub dwell: f32,
    pub media_auto_scale: u32,
    pub media_loop_set: bool,
    pub media_texture_id: Option<Uuid>,
    pub media_url: String,
    pub music_url: String,
    pub pass_hours: f32,
    pub pass_price: i32,
    pub sale_price: i32,
    pub media_type: String,
    pub media_description: String,
    pub media_size: Vector3,
    pub media_loop: bool,
    pub obscure_media: bool,
    pub obscure_music: bool,
    pub see_avatar_distance: f32,
    pub any_avatar_sounds: bool,
    pub group_avatar_sounds: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LandData {
    pub fn new(region_id: Uuid, local_id: u32, owner_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4(),
            region_id,
            local_land_id: local_id,
            bitmap: vec![0u8; 64 * 64 / 8], // 64x64 bitmap
            name: "Your Parcel".to_string(),
            description: String::new(),
            owner_id,
            group_id: None,
            is_group_owned: false,
            area: 0,
            auction_id: 0,
            category: 0,
            claim_date: now.timestamp() as i32,
            claim_price: 0,
            status: 0,
            landing_type: 0,
            landing_position: Vector3::new(128.0, 128.0, 25.0),
            landing_look_at: Vector3::new(1.0, 0.0, 0.0),
            user_location: Vector3::new(128.0, 128.0, 25.0),
            user_look_at: Vector3::new(1.0, 0.0, 0.0),
            auth_buyer_id: None,
            snapshot_id: None,
            other_clean_time: 0,
            dwell: 0.0,
            media_auto_scale: 0,
            media_loop_set: false,
            media_texture_id: None,
            media_url: String::new(),
            music_url: String::new(),
            pass_hours: 0.0,
            pass_price: 0,
            sale_price: 0,
            media_type: "none".to_string(),
            media_description: String::new(),
            media_size: Vector3::zero(),
            media_loop: false,
            obscure_media: false,
            obscure_music: false,
            see_avatar_distance: 20.0,
            any_avatar_sounds: true,
            group_avatar_sounds: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Spawn point for region teleport/login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPoint {
    pub id: Uuid,
    pub region_id: Uuid,
    pub position: Vector3,
    pub look_at: Vector3,
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

impl SpawnPoint {
    pub fn new(region_id: Uuid, position: Vector3, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            region_id,
            position,
            look_at: Vector3::new(1.0, 0.0, 0.0),
            name,
            description: String::new(),
            is_default: false,
            created_at: Utc::now(),
        }
    }
}

/// Prim shape data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimShape {
    pub uuid: Uuid,
    pub prim_id: Uuid,
    pub shape_type: u32,
    pub path_begin: u32,
    pub path_end: u32,
    pub path_scale_x: u32,
    pub path_scale_y: u32,
    pub path_shear_x: u32,
    pub path_shear_y: u32,
    pub path_skew: u32,
    pub path_curve: u32,
    pub path_radius_offset: u32,
    pub path_revolutions: u32,
    pub path_taper_x: u32,
    pub path_taper_y: u32,
    pub path_twist: u32,
    pub path_twist_begin: u32,
    pub profile_begin: u32,
    pub profile_end: u32,
    pub profile_curve: u32,
    pub profile_hollow: u32,
    pub texture_entry: Vec<u8>,
    pub extra_params: Vec<u8>,
    pub state: u32,
    pub last_attach_point: u32,
    pub media: String, // JSON serialized
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PrimShape {
    pub fn new(prim_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4(),
            prim_id,
            shape_type: 1,
            path_begin: 0,
            path_end: 0,
            path_scale_x: 100,
            path_scale_y: 100,
            path_shear_x: 0,
            path_shear_y: 0,
            path_skew: 0,
            path_curve: 16,
            path_radius_offset: 0,
            path_revolutions: 1,
            path_taper_x: 0,
            path_taper_y: 0,
            path_twist: 0,
            path_twist_begin: 0,
            profile_begin: 0,
            profile_end: 0,
            profile_curve: 1,
            profile_hollow: 0,
            texture_entry: Vec::new(),
            extra_params: Vec::new(),
            state: 0,
            last_attach_point: 0,
            media: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_info_creation() {
        let region = RegionInfo::new("Test Region".to_string(), 1000, 1000);
        assert_eq!(region.region_name, "Test Region");
        assert_eq!(region.location_x, 1000);
        assert_eq!(region.location_y, 1000);
        assert_eq!(region.size_x, 256);
        assert_eq!(region.size_y, 256);
    }

    #[test]
    fn test_region_handle_calculation() {
        let handle = RegionInfo::calculate_handle(1000, 1000);
        let (world_x, world_y) = RegionInfo::handle_to_coords(handle);
        assert_eq!(world_x, 256000);
        assert_eq!(world_y, 256000);
        assert_eq!(handle, (256000_u64 << 32) | 256000_u64);

        let (grid_x, grid_y) = RegionInfo::handle_to_grid_coords(handle);
        assert_eq!(grid_x, 1000);
        assert_eq!(grid_y, 1000);
    }

    #[test]
    fn test_region_handle_various_coords() {
        let handle = RegionInfo::calculate_handle(2000, 2000);
        let (world_x, world_y) = RegionInfo::handle_to_coords(handle);
        assert_eq!(world_x, 512000);
        assert_eq!(world_y, 512000);

        let handle2 = RegionInfo::calculate_handle(1001, 1000);
        let (gx, gy) = RegionInfo::handle_to_grid_coords(handle2);
        assert_eq!(gx, 1001);
        assert_eq!(gy, 1000);
    }

    #[test]
    fn test_vector3_operations() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::zero();
        let v3 = Vector3::one();

        assert_eq!(v2.x, 0.0);
        assert_eq!(v3.x, 1.0);
        assert_eq!(v1.y, 2.0);
    }

    #[test]
    fn test_quaternion_identity() {
        let q = Quaternion::identity();
        assert_eq!(q.w, 1.0);
        assert_eq!(q.x, 0.0);
    }

    #[test]
    fn test_scene_object_part_creation() {
        let creator_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let position = Vector3::new(128.0, 128.0, 25.0);

        let part = SceneObjectPart::new("Test Object".to_string(), position, creator_id, owner_id);
        assert_eq!(part.name, "Test Object");
        assert_eq!(part.creator_id, creator_id);
        assert_eq!(part.owner_id, owner_id);
        assert_eq!(part.group_position, position);
    }

    #[test]
    fn test_terrain_data_creation() {
        let region_id = Uuid::new_v4();
        let terrain = TerrainData::new(region_id);

        assert_eq!(terrain.region_id, region_id);
        assert_eq!(terrain.terrain_revision, 1);
        assert_eq!(terrain.water_height, 20.0);
        assert_eq!(terrain.terrain_data.len(), 256 * 256 * 4);
    }

    #[test]
    fn test_land_data_creation() {
        let region_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let land = LandData::new(region_id, 1, owner_id);

        assert_eq!(land.region_id, region_id);
        assert_eq!(land.local_land_id, 1);
        assert_eq!(land.owner_id, owner_id);
        assert_eq!(land.name, "Your Parcel");
    }
}
