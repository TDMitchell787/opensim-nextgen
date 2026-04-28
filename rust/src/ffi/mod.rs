//! Foreign Function Interface (FFI) for Zig modules
//!
//! This module defines the FFI bindings for interacting with the
//! performance-critical Zig modules, such as the physics engine.

#![allow(non_camel_case_types)]

use std::ffi::CStr;
use tracing::{debug, error, info, warn};

pub mod assets;
pub mod physics;

#[repr(C)]
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum PhysicsEngineType {
    Basic = 0,
    POS = 1,
    ODE = 2,
    UbODE = 3,
    Bullet = 4,
}

impl PhysicsEngineType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "basicphysics" | "basic" => Self::Basic,
            "pos" | "posscene" => Self::POS,
            "ode" | "openynamicsengine" => Self::ODE,
            "ubode" | "ubodescene" => Self::UbODE,
            "bulletsim" | "bullet" => Self::Bullet,
            _ => Self::ODE,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PhysicsBody {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PhysicsHeightfield {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RaycastResult {
    pub hit: bool,
    pub point: Vec3,
    pub normal: Vec3,
    pub fraction: f32,
    pub body_id: u32,
}

// FFI Result wrapper matching Zig FFIResultWrapper
#[repr(C)]
pub struct FFIResultWrapper {
    pub error_code: i32,
    pub data: *const u8,
    pub data_len: usize,
    pub handle: u64,
}

// Error codes matching Zig FFI_ERROR_* constants
pub const FFI_ERROR_SUCCESS: i32 = 0;
pub const FFI_ERROR_INVALID_HANDLE: i32 = 1;
pub const FFI_ERROR_ALLOCATION_FAILED: i32 = 2;
pub const FFI_ERROR_INVALID_PARAMETER: i32 = 3;
pub const FFI_ERROR_BUFFER_TOO_SMALL: i32 = 4;
pub const FFI_ERROR_NOT_INITIALIZED: i32 = 5;
pub const FFI_ERROR_NETWORK_ERROR: i32 = 6;
pub const FFI_ERROR_PHYSICS_ERROR: i32 = 7;
pub const FFI_ERROR_PROTOCOL_ERROR: i32 = 8;
pub const FFI_ERROR_TIMEOUT_ERROR: i32 = 9;
pub const FFI_ERROR_INTERNAL_ERROR: i32 = 10;

// Rust error enum for FFI errors
#[derive(Debug, thiserror::Error)]
pub enum FFIError {
    #[error("Invalid handle")]
    InvalidHandle,
    #[error("Memory allocation failed")]
    AllocationFailed,
    #[error("Invalid parameter")]
    InvalidParameter,
    #[error("Buffer too small")]
    BufferTooSmall,
    #[error("Not initialized")]
    NotInitialized,
    #[error("Network error")]
    NetworkError,
    #[error("Physics error")]
    PhysicsError,
    #[error("Protocol error")]
    ProtocolError,
    #[error("Timeout error")]
    TimeoutError,
    #[error("Internal error")]
    InternalError,
    #[error("Unknown error code: {0}")]
    Unknown(i32),
    #[error("Creation failed")]
    CreationFailed,
}

impl From<i32> for FFIError {
    fn from(error_code: i32) -> Self {
        match error_code {
            FFI_ERROR_SUCCESS => {
                // Log this as it indicates a programming error
                tracing::error!("Attempted to convert success code to error - this is a bug");
                FFIError::InvalidParameter // Return a reasonable error instead of panic
            }
            FFI_ERROR_INVALID_HANDLE => FFIError::InvalidHandle,
            FFI_ERROR_ALLOCATION_FAILED => FFIError::AllocationFailed,
            FFI_ERROR_INVALID_PARAMETER => FFIError::InvalidParameter,
            FFI_ERROR_BUFFER_TOO_SMALL => FFIError::BufferTooSmall,
            FFI_ERROR_NOT_INITIALIZED => FFIError::NotInitialized,
            FFI_ERROR_NETWORK_ERROR => FFIError::NetworkError,
            FFI_ERROR_PHYSICS_ERROR => FFIError::PhysicsError,
            FFI_ERROR_PROTOCOL_ERROR => FFIError::ProtocolError,
            FFI_ERROR_TIMEOUT_ERROR => FFIError::TimeoutError,
            FFI_ERROR_INTERNAL_ERROR => FFIError::InternalError,
            _ => FFIError::Unknown(error_code),
        }
    }
}

// FFI function declarations
extern "C" {
    // Core FFI functions
    fn ffi_init() -> FFIResultWrapper;
    fn ffi_cleanup() -> FFIResultWrapper;
    fn ffi_create_buffer(capacity: usize) -> FFIResultWrapper;
    fn ffi_destroy_buffer(buffer_id: u64) -> FFIResultWrapper;
    fn ffi_get_error_count() -> i32;
    fn ffi_get_last_error() -> *const u8;
    fn ffi_clear_logs();

    // Physics functions
    fn physics_memory_init(size: usize);
    fn physics_memory_deinit();
    fn physics_body_create() -> *mut PhysicsBody;
    fn physics_body_destroy(body: *mut PhysicsBody);
    fn physics_body_get_position(body: *mut PhysicsBody) -> Vec3;
    fn physics_body_set_velocity(body: *mut PhysicsBody, vel: Vec3);
    fn physics_body_get_velocity(body: *mut PhysicsBody) -> Vec3;
    fn physics_body_set_position(body: *mut PhysicsBody, pos: Vec3);
    fn physics_body_set_rotation(body: *mut PhysicsBody, rot: Quat);
    fn physics_body_get_rotation(body: *mut PhysicsBody) -> Quat;
    fn physics_step(timestep: f32);
    fn physics_set_gravity(gravity: Vec3);
    fn physics_create_terrain(heights: *const f32, w: u32, h: u32) -> bool;
    fn physics_create_engine(engine_type: u8) -> bool;
    fn physics_destroy_engine();
    fn physics_engine_name() -> *const u8;
    fn physics_engine_type() -> u8;
    fn physics_get_terrain_height(x: f32, y: f32) -> f32;
    fn physics_body_set_flying(body: *mut PhysicsBody, flying: bool);
    fn physics_body_is_on_ground(body: *mut PhysicsBody) -> bool;

    pub fn physics_create_hull_shape(hull_count: i32, hulls: *const f32) -> *mut std::ffi::c_void;
    pub fn physics_create_mesh_shape(
        indices_count: i32,
        indices: *const i32,
        vertices_count: i32,
        vertices: *const f32,
    ) -> *mut std::ffi::c_void;
    pub fn physics_create_body_from_shape(
        shape: *mut std::ffi::c_void,
        id: u32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
        rot_w: f32,
    ) -> *mut std::ffi::c_void;
    pub fn physics_delete_collision_shape(shape: *mut std::ffi::c_void) -> bool;
    pub fn physics_raycast(
        from_x: f32,
        from_y: f32,
        from_z: f32,
        to_x: f32,
        to_y: f32,
        to_z: f32,
        out_hit: *mut bool,
        out_id: *mut u32,
        out_fraction: *mut f32,
        out_px: *mut f32,
        out_py: *mut f32,
        out_pz: *mut f32,
        out_nx: *mut f32,
        out_ny: *mut f32,
        out_nz: *mut f32,
    );

    pub fn physics_collision_init() -> bool;
    pub fn physics_collision_deinit();
    pub fn physics_create_collision_world(region_id_hi: u64, region_id_lo: u64) -> bool;
    pub fn physics_destroy_collision_world(region_id_hi: u64, region_id_lo: u64);
    pub fn physics_world_add_box(
        region_hi: u64,
        region_lo: u64,
        local_id: u32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
        rot_w: f32,
        half_x: f32,
        half_y: f32,
        half_z: f32,
        flags: u32,
    ) -> bool;
    pub fn physics_world_add_sphere(
        region_hi: u64,
        region_lo: u64,
        local_id: u32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        radius: f32,
        flags: u32,
    ) -> bool;
    pub fn physics_world_add_avatar(
        region_hi: u64,
        region_lo: u64,
        local_id: u32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        radius: f32,
        height: f32,
    ) -> bool;
    pub fn physics_world_set_avatar_position(
        region_hi: u64,
        region_lo: u64,
        local_id: u32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
    );
    pub fn physics_world_remove_body(region_hi: u64, region_lo: u64, local_id: u32);
    pub fn physics_world_update_body_transform(
        region_hi: u64,
        region_lo: u64,
        local_id: u32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
        rot_w: f32,
    );
    pub fn physics_world_set_terrain(
        region_hi: u64,
        region_lo: u64,
        heights: *const f32,
        width: u32,
        depth: u32,
    ) -> bool;
    pub fn physics_world_collision_step(region_hi: u64, region_lo: u64) -> u32;
    pub fn physics_world_get_contact(
        region_hi: u64,
        region_lo: u64,
        index: u32,
        out_id_a: *mut u32,
        out_id_b: *mut u32,
        out_nx: *mut f32,
        out_ny: *mut f32,
        out_nz: *mut f32,
        out_depth: *mut f32,
        out_px: *mut f32,
        out_py: *mut f32,
        out_pz: *mut f32,
    ) -> bool;
    pub fn physics_world_raycast_down(
        region_hi: u64,
        region_lo: u64,
        origin_x: f32,
        origin_y: f32,
        origin_z: f32,
        max_dist: f32,
        out_hit: *mut bool,
        out_body_id: *mut u32,
        out_px: *mut f32,
        out_py: *mut f32,
        out_pz: *mut f32,
        out_nx: *mut f32,
        out_ny: *mut f32,
        out_nz: *mut f32,
    );
}

// FFI Manager for handling FFI operations with error handling
pub struct FFIManager {
    initialized: bool,
}

impl FFIManager {
    pub fn new() -> Result<Self, FFIError> {
        info!("Initializing FFI manager");

        let result = unsafe { ffi_init() };
        if result.error_code != FFI_ERROR_SUCCESS {
            let error = FFIError::from(result.error_code);
            error!("Failed to initialize FFI: {:?}", error);
            return Err(error);
        }

        info!("FFI manager initialized successfully");
        Ok(Self { initialized: true })
    }

    pub fn get_error_count(&self) -> i32 {
        unsafe { ffi_get_error_count() }
    }

    pub fn get_last_error(&self) -> String {
        unsafe {
            let error_ptr = ffi_get_last_error();
            if error_ptr.is_null() {
                return "No error available".to_string();
            }

            let c_str = CStr::from_ptr(error_ptr as *const i8);
            c_str.to_string_lossy().into_owned()
        }
    }

    pub fn clear_logs(&self) {
        unsafe { ffi_clear_logs() };
        debug!("FFI logs cleared");
    }

    pub fn create_buffer(&self, capacity: usize) -> Result<u64, FFIError> {
        info!("Creating FFI buffer with capacity {}", capacity);

        let result = unsafe { ffi_create_buffer(capacity) };
        if result.error_code != FFI_ERROR_SUCCESS {
            let error = FFIError::from(result.error_code);
            error!("Failed to create buffer: {:?}", error);
            return Err(error);
        }

        info!("Buffer created successfully with handle {}", result.handle);
        Ok(result.handle)
    }

    pub fn destroy_buffer(&self, buffer_id: u64) -> Result<(), FFIError> {
        info!("Destroying FFI buffer {}", buffer_id);

        let result = unsafe { ffi_destroy_buffer(buffer_id) };
        if result.error_code != FFI_ERROR_SUCCESS {
            let error = FFIError::from(result.error_code);
            error!("Failed to destroy buffer {}: {:?}", buffer_id, error);
            return Err(error);
        }

        info!("Buffer {} destroyed successfully", buffer_id);
        Ok(())
    }
}

impl Drop for FFIManager {
    fn drop(&mut self) {
        if self.initialized {
            info!("Cleaning up FFI manager");
            let result = unsafe { ffi_cleanup() };
            if result.error_code != FFI_ERROR_SUCCESS {
                error!("Error during FFI cleanup: {}", result.error_code);
            } else {
                info!("FFI manager cleaned up successfully");
            }
        }
    }
}

// Physics wrapper with error handling
pub struct Physics {
    ffi_manager: FFIManager,
    engine_type: PhysicsEngineType,
}

impl Physics {
    pub fn new() -> Result<Self, FFIError> {
        Self::with_engine(PhysicsEngineType::ODE)
    }

    pub fn with_engine(engine_type: PhysicsEngineType) -> Result<Self, FFIError> {
        info!("Initializing physics system with engine: {:?}", engine_type);

        let ffi_manager = FFIManager::new()?;

        unsafe {
            if !physics_create_engine(engine_type as u8) {
                warn!(
                    "Failed to create {:?} engine, falling back to ODE",
                    engine_type
                );
                physics_memory_init(1024 * 1024 * 10);
            }
        }

        info!("Physics system initialized successfully");
        Ok(Self {
            ffi_manager,
            engine_type,
        })
    }

    pub fn engine_name(&self) -> String {
        unsafe {
            let ptr = physics_engine_name();
            if ptr.is_null() {
                return "Unknown".to_string();
            }
            let c_str = CStr::from_ptr(ptr as *const i8);
            c_str.to_string_lossy().into_owned()
        }
    }

    pub fn engine_type(&self) -> PhysicsEngineType {
        self.engine_type
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        unsafe { physics_set_gravity(gravity) };
    }

    pub fn create_terrain(&mut self, heights: &[f32], w: u32, h: u32) -> Result<(), FFIError> {
        let ok = unsafe { physics_create_terrain(heights.as_ptr(), w, h) };
        if ok {
            Ok(())
        } else {
            Err(FFIError::PhysicsError)
        }
    }

    pub fn step(&mut self, delta_time: f32) -> Result<(), FFIError> {
        debug!("Physics step with delta_time: {}", delta_time);

        unsafe { physics_step(delta_time) };

        // Check for errors after physics step
        let error_count = self.ffi_manager.get_error_count();
        if error_count > 0 {
            let error_msg = self.ffi_manager.get_last_error();
            warn!(
                "Physics step completed with {} errors: {}",
                error_count, error_msg
            );
        }

        Ok(())
    }

    pub fn get_terrain_height(&self, x: f32, y: f32) -> f32 {
        unsafe { physics_get_terrain_height(x, y) }
    }

    pub fn create_body(&mut self) -> Result<PhysicsHandle, FFIError> {
        info!("Creating physics body");

        let body = unsafe { physics_body_create() };
        if body.is_null() {
            error!("Failed to create physics body: null pointer returned");
            return Err(FFIError::PhysicsError);
        }

        info!("Physics body created successfully");
        Ok(PhysicsHandle { body })
    }

    pub fn get_error_count(&self) -> i32 {
        self.ffi_manager.get_error_count()
    }

    pub fn get_last_error(&self) -> String {
        self.ffi_manager.get_last_error()
    }

    pub fn raycast(&self, from: [f32; 3], to: [f32; 3]) -> Option<RaycastResult> {
        let mut hit = false;
        let mut id: u32 = 0;
        let mut fraction: f32 = 0.0;
        let mut px: f32 = 0.0;
        let mut py: f32 = 0.0;
        let mut pz: f32 = 0.0;
        let mut nx: f32 = 0.0;
        let mut ny: f32 = 0.0;
        let mut nz: f32 = 0.0;
        unsafe {
            physics_raycast(
                from[0],
                from[1],
                from[2],
                to[0],
                to[1],
                to[2],
                &mut hit,
                &mut id,
                &mut fraction,
                &mut px,
                &mut py,
                &mut pz,
                &mut nx,
                &mut ny,
                &mut nz,
            );
        }
        if hit {
            Some(RaycastResult {
                hit: true,
                point: Vec3 {
                    x: px,
                    y: py,
                    z: pz,
                },
                normal: Vec3 {
                    x: nx,
                    y: ny,
                    z: nz,
                },
                fraction,
                body_id: id,
            })
        } else {
            None
        }
    }

    pub fn create_heightfield(
        &mut self,
        _width: i32,
        _height: i32,
        _heights: &[f32],
    ) -> Result<*mut PhysicsHeightfield, FFIError> {
        Err(FFIError::CreationFailed)
    }
}

impl Drop for Physics {
    fn drop(&mut self) {
        info!("Cleaning up physics system ({:?})", self.engine_type);
        unsafe {
            physics_destroy_engine();
        }
        info!("Physics system cleaned up successfully");
    }
}

pub struct PhysicsHandle {
    body: *mut PhysicsBody,
}

impl PhysicsHandle {
    pub fn set_velocity(&mut self, vel: Vec3) -> Result<(), FFIError> {
        debug!("Setting physics body velocity: {:?}", vel);

        unsafe { physics_body_set_velocity(self.body, vel) };

        // Check for errors
        if self.body.is_null() {
            return Err(FFIError::InvalidHandle);
        }

        Ok(())
    }

    pub fn get_velocity(&self) -> Result<Vec3, FFIError> {
        if self.body.is_null() {
            return Err(FFIError::InvalidHandle);
        }

        let vel = unsafe { physics_body_get_velocity(self.body) };
        debug!("Physics body velocity: {:?}", vel);
        Ok(vel)
    }

    pub fn get_position(&self) -> Result<Vec3, FFIError> {
        if self.body.is_null() {
            return Err(FFIError::InvalidHandle);
        }

        let pos = unsafe { physics_body_get_position(self.body) };
        debug!("Physics body position: {:?}", pos);
        Ok(pos)
    }

    pub fn set_position(&mut self, pos: Vec3) -> Result<(), FFIError> {
        if self.body.is_null() {
            return Err(FFIError::InvalidHandle);
        }
        unsafe { physics_body_set_position(self.body, pos) };
        Ok(())
    }

    pub fn get_rotation(&self) -> Result<Quat, FFIError> {
        if self.body.is_null() {
            return Err(FFIError::InvalidHandle);
        }
        let rot = unsafe { physics_body_get_rotation(self.body) };
        Ok(rot)
    }

    pub fn set_rotation(&mut self, rot: Quat) -> Result<(), FFIError> {
        if self.body.is_null() {
            return Err(FFIError::InvalidHandle);
        }
        unsafe { physics_body_set_rotation(self.body, rot) };
        Ok(())
    }

    pub fn set_flying(&mut self, flying: bool) {
        if !self.body.is_null() {
            unsafe { physics_body_set_flying(self.body, flying) };
        }
    }

    pub fn is_on_ground(&self) -> bool {
        if self.body.is_null() {
            return true;
        }
        unsafe { physics_body_is_on_ground(self.body) }
    }
}

impl Drop for PhysicsHandle {
    fn drop(&mut self) {
        if !self.body.is_null() {
            debug!("Destroying physics body at address {:p}", self.body);
            unsafe { physics_body_destroy(self.body) };
            // Set to null to prevent double-free
            self.body = std::ptr::null_mut();
        }
    }
}

unsafe impl Send for PhysicsHandle {}
unsafe impl Sync for PhysicsHandle {}

pub fn uuid_to_hi_lo(uuid: uuid::Uuid) -> (u64, u64) {
    let bytes = uuid.as_bytes();
    let hi = u64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    let lo = u64::from_be_bytes([
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ]);
    (hi, lo)
}

pub struct CollisionWorld;

impl CollisionWorld {
    pub fn init() -> bool {
        unsafe { physics_collision_init() }
    }

    pub fn deinit() {
        unsafe { physics_collision_deinit() }
    }

    pub fn create(region_uuid: uuid::Uuid) -> bool {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe { physics_create_collision_world(hi, lo) }
    }

    pub fn destroy(region_uuid: uuid::Uuid) {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe { physics_destroy_collision_world(hi, lo) }
    }

    pub fn add_box(
        region_uuid: uuid::Uuid,
        local_id: u32,
        pos: [f32; 3],
        rot: [f32; 4],
        half_extents: [f32; 3],
        flags: u32,
    ) -> bool {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe {
            physics_world_add_box(
                hi,
                lo,
                local_id,
                pos[0],
                pos[1],
                pos[2],
                rot[0],
                rot[1],
                rot[2],
                rot[3],
                half_extents[0],
                half_extents[1],
                half_extents[2],
                flags,
            )
        }
    }

    pub fn add_sphere(
        region_uuid: uuid::Uuid,
        local_id: u32,
        pos: [f32; 3],
        radius: f32,
        flags: u32,
    ) -> bool {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe { physics_world_add_sphere(hi, lo, local_id, pos[0], pos[1], pos[2], radius, flags) }
    }

    pub fn add_avatar(
        region_uuid: uuid::Uuid,
        local_id: u32,
        pos: [f32; 3],
        radius: f32,
        height: f32,
    ) -> bool {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe {
            physics_world_add_avatar(hi, lo, local_id, pos[0], pos[1], pos[2], radius, height)
        }
    }

    pub fn set_avatar_position(region_uuid: uuid::Uuid, local_id: u32, pos: [f32; 3]) {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe { physics_world_set_avatar_position(hi, lo, local_id, pos[0], pos[1], pos[2]) }
    }

    pub fn remove_body(region_uuid: uuid::Uuid, local_id: u32) {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe { physics_world_remove_body(hi, lo, local_id) }
    }

    pub fn update_transform(region_uuid: uuid::Uuid, local_id: u32, pos: [f32; 3], rot: [f32; 4]) {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe {
            physics_world_update_body_transform(
                hi, lo, local_id, pos[0], pos[1], pos[2], rot[0], rot[1], rot[2], rot[3],
            )
        }
    }

    pub fn set_terrain(region_uuid: uuid::Uuid, heights: &[f32], width: u32, depth: u32) -> bool {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe { physics_world_set_terrain(hi, lo, heights.as_ptr(), width, depth) }
    }

    pub fn collision_step(region_uuid: uuid::Uuid) -> u32 {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        unsafe { physics_world_collision_step(hi, lo) }
    }

    pub fn get_contact(region_uuid: uuid::Uuid, index: u32) -> Option<ContactResult> {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        let mut id_a: u32 = 0;
        let mut id_b: u32 = 0;
        let mut nx: f32 = 0.0;
        let mut ny: f32 = 0.0;
        let mut nz: f32 = 0.0;
        let mut depth: f32 = 0.0;
        let mut px: f32 = 0.0;
        let mut py: f32 = 0.0;
        let mut pz: f32 = 0.0;
        let ok = unsafe {
            physics_world_get_contact(
                hi, lo, index, &mut id_a, &mut id_b, &mut nx, &mut ny, &mut nz, &mut depth,
                &mut px, &mut py, &mut pz,
            )
        };
        if ok {
            Some(ContactResult {
                body_id_a: id_a,
                body_id_b: id_b,
                normal: [nx, ny, nz],
                depth,
                point: [px, py, pz],
            })
        } else {
            None
        }
    }

    pub fn raycast_down(
        region_uuid: uuid::Uuid,
        origin: [f32; 3],
        max_dist: f32,
    ) -> Option<RayDownResult> {
        let (hi, lo) = uuid_to_hi_lo(region_uuid);
        let mut hit = false;
        let mut body_id: u32 = 0;
        let mut px: f32 = 0.0;
        let mut py: f32 = 0.0;
        let mut pz: f32 = 0.0;
        let mut nx: f32 = 0.0;
        let mut ny: f32 = 0.0;
        let mut nz: f32 = 0.0;
        unsafe {
            physics_world_raycast_down(
                hi,
                lo,
                origin[0],
                origin[1],
                origin[2],
                max_dist,
                &mut hit,
                &mut body_id,
                &mut px,
                &mut py,
                &mut pz,
                &mut nx,
                &mut ny,
                &mut nz,
            );
        }
        if hit {
            Some(RayDownResult {
                body_id,
                point: [px, py, pz],
                normal: [nx, ny, nz],
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RayDownResult {
    pub body_id: u32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct ContactResult {
    pub body_id_a: u32,
    pub body_id_b: u32,
    pub normal: [f32; 3],
    pub depth: f32,
    pub point: [f32; 3],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_error_conversion() {
        assert!(matches!(
            FFIError::from(FFI_ERROR_INVALID_HANDLE),
            FFIError::InvalidHandle
        ));
        assert!(matches!(
            FFIError::from(FFI_ERROR_ALLOCATION_FAILED),
            FFIError::AllocationFailed
        ));
        assert!(matches!(FFIError::from(999), FFIError::Unknown(999)));
    }

    #[test]
    fn test_ffi_manager_creation() {
        let manager = FFIManager::new();
        assert!(manager.is_ok());

        if let Ok(manager) = manager {
            assert_eq!(manager.get_error_count(), 0);
        }
    }

    #[test]
    fn test_physics_creation() {
        let physics = Physics::new();
        assert!(physics.is_ok());

        if let Ok(mut physics) = physics {
            assert_eq!(physics.get_error_count(), 0);

            // Test physics step
            let step_result = physics.step(1.0 / 60.0);
            assert!(step_result.is_ok());
        }
    }

    #[test]
    fn test_physics_body_creation() {
        let physics = Physics::new().expect("Failed to create physics");
        let mut physics = physics;

        let body = physics.create_body();
        assert!(body.is_ok());

        if let Ok(mut body) = body {
            // Test position and velocity
            let pos = body.get_position();
            assert!(pos.is_ok());

            let vel = body.get_velocity();
            assert!(vel.is_ok());

            // Test setting velocity
            let new_vel = Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            };
            let set_result = body.set_velocity(new_vel);
            assert!(set_result.is_ok());
        }
    }
}
