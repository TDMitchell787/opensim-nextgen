//! Enhanced Avatar System for OpenSim Next
//! 
//! This module provides advanced avatar management capabilities including:
//! - Advanced appearance and customization
//! - Avatar behavior and animation systems
//! - Cross-region persistence and synchronization
//! - Social features and profiles

pub mod appearance;
pub mod behavior;
pub mod factory;
// PHASE 25.1.1: Restoring real database persistence layer
pub mod persistence;
// pub mod persistence_stub;
// pub use persistence_stub as persistence;
// PHASE 25.1.2: Restoring real social features database layer
pub mod social;
// pub mod social_stub;
// pub use social_stub as social;
pub mod manager;
pub mod api;

pub use appearance::*;
pub use behavior::*;
pub use factory::*;
pub use persistence::*;
pub use social::*;
pub use manager::*;
pub use api::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Enhanced avatar information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAvatar {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub appearance: AvatarAppearance,
    pub behavior: AvatarBehavior,
    pub social_profile: AvatarSocialProfile,
    pub persistence_data: AvatarPersistenceData,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Avatar appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AvatarAppearance {
    pub height: f32,
    pub proportions: AvatarProportions,
    pub wearables: Vec<WearableItem>,
    pub textures: HashMap<String, String>, // texture_type -> texture_uuid
    pub attachments: Vec<AvatarAttachment>,
    pub visual_params: Vec<VisualParameter>,
}

/// Avatar body proportions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarProportions {
    pub body_height: f32,
    pub body_width: f32,
    pub head_size: f32,
    pub leg_length: f32,
    pub arm_length: f32,
    pub torso_length: f32,
}

/// Wearable item information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WearableItem {
    pub item_id: Uuid,
    pub asset_id: Uuid,
    pub wearable_type: WearableType,
    pub name: String,
    pub layer: i32,
    pub permissions: WearablePermissions,
    pub parameters: Vec<WearableParameter>,
}

/// Types of wearable items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WearableType {
    Skin,
    Hair,
    Eyes,
    Shirt,
    Pants,
    Shoes,
    Socks,
    Jacket,
    Gloves,
    Undershirt,
    Underpants,
    Skirt,
    Alpha,
    Tattoo,
    Physics,
    Universal,
}

/// Wearable permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WearablePermissions {
    pub owner_can_modify: bool,
    pub owner_can_copy: bool,
    pub owner_can_transfer: bool,
    pub group_can_modify: bool,
    pub everyone_can_modify: bool,
}

/// Wearable parameter for customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WearableParameter {
    pub param_id: i32,
    pub value: f32,
    pub name: String,
}

/// Avatar attachment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarAttachment {
    pub item_id: Uuid,
    pub asset_id: Uuid,
    pub attachment_point: AttachmentPoint,
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub permissions: AttachmentPermissions,
}

/// Avatar attachment points
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AttachmentPoint {
    Chest,
    Skull,
    LeftShoulder,
    RightShoulder,
    LeftHand,
    RightHand,
    LeftFoot,
    RightFoot,
    Spine,
    Pelvis,
    Mouth,
    Chin,
    LeftEar,
    RightEar,
    LeftEyeball,
    RightEyeball,
    Nose,
    RightUpperArm,
    RightForearm,
    LeftUpperArm,
    LeftForearm,
    RightHip,
    RightUpperLeg,
    RightLowerLeg,
    LeftHip,
    LeftUpperLeg,
    LeftLowerLeg,
    Stomach,
    LeftPec,
    RightPec,
    HudCenter2,
    HudTopRight,
    HudTop,
    HudTopLeft,
    HudCenter,
    HudBottomLeft,
    HudBottom,
    HudBottomRight,
    Neck,
    Root,
}

/// Attachment permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentPermissions {
    pub can_detach: bool,
    pub can_modify: bool,
    pub can_copy: bool,
    pub can_transfer: bool,
}

/// Visual parameters for avatar customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualParameter {
    pub param_id: i32,
    pub name: String,
    pub value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub category: VisualParameterCategory,
}

/// Categories of visual parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualParameterCategory {
    Shape,
    Skin,
    Hair,
    Eyes,
    Clothing,
    Physics,
}

/// 3D Vector for positions and scales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Quaternion for rotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Avatar behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AvatarBehavior {
    pub animations: Vec<AnimationState>,
    pub gestures: Vec<GestureInfo>,
    pub auto_behaviors: Vec<AutoBehavior>,
    pub expressions: Vec<FacialExpression>,
    pub voice_settings: VoiceSettings,
}

/// Animation state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationState {
    pub animation_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub loop_animation: bool,
    pub start_time: DateTime<Utc>,
    pub duration: Option<f32>,
    pub blend_weight: f32,
}

/// Gesture information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureInfo {
    pub gesture_id: Uuid,
    pub name: String,
    pub trigger: String,
    pub animation_sequence: Vec<Uuid>,
    pub sound_effects: Vec<Uuid>,
    pub chat_text: Option<String>,
    pub enabled: bool,
}

/// Automatic behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoBehavior {
    pub behavior_id: Uuid,
    pub name: String,
    pub trigger_condition: BehaviorTrigger,
    pub actions: Vec<BehaviorAction>,
    pub enabled: bool,
    pub cooldown_seconds: f32,
}

/// Behavior trigger conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorTrigger {
    Idle { duration_seconds: f32 },
    Movement { movement_type: MovementType },
    Interaction { interaction_type: InteractionType },
    Time { schedule: String },
    Random { probability: f32, interval_seconds: f32 },
}

/// Movement types for behavior triggers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MovementType {
    Walking,
    Running,
    Flying,
    Sitting,
    Standing,
    Teleporting,
}

/// Interaction types for behavior triggers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InteractionType {
    Chat,
    Touch,
    Collision,
    ProximityEnter,
    ProximityExit,
}

/// Behavior actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorAction {
    PlayAnimation { animation_id: Uuid, duration: Option<f32> },
    PlaySound { sound_id: Uuid, volume: f32 },
    SendChat { message: String, channel: i32 },
    ChangeExpression { expression: FacialExpression },
    TriggerGesture { gesture_id: Uuid },
}

/// Facial expression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacialExpression {
    pub expression_id: Uuid,
    pub name: String,
    pub morph_targets: HashMap<String, f32>, // morph_name -> weight
    pub duration: f32,
    pub blend_in_time: f32,
    pub blend_out_time: f32,
}

/// Voice settings for avatar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSettings {
    pub voice_enabled: bool,
    pub voice_channel: Option<Uuid>,
    pub voice_volume: f32,
    pub voice_effects: Vec<VoiceEffect>,
    pub spatial_audio: bool,
    pub voice_modulation: Option<VoiceModulation>,
}

/// Voice effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceEffect {
    Echo { delay: f32, decay: f32 },
    Reverb { room_size: f32, damping: f32 },
    PitchShift { shift: f32 },
    Distortion { intensity: f32 },
}

/// Voice modulation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceModulation {
    None,
    Robot,
    Alien,
    Chipmunk,
    Deep,
    Whisper,
    Custom { parameters: HashMap<String, f32> },
}

/// Avatar social profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarSocialProfile {
    pub display_name: String,
    pub bio: Option<String>,
    pub interests: Vec<String>,
    pub languages: Vec<String>,
    pub relationship_status: RelationshipStatus,
    pub privacy_settings: PrivacySettings,
    pub social_links: HashMap<String, String>,
    pub achievements: Vec<Achievement>,
}

/// Relationship status options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipStatus {
    Single,
    InRelationship,
    Married,
    Complicated,
    NotSpecified,
}

/// Privacy settings for avatar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub profile_visibility: VisibilityLevel,
    pub online_status_visibility: VisibilityLevel,
    pub location_visibility: VisibilityLevel,
    pub friend_list_visibility: VisibilityLevel,
    pub allow_friend_requests: bool,
    pub allow_messages: MessagePermission,
    pub allow_voice_calls: bool,
}

/// Visibility levels for privacy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisibilityLevel {
    Public,
    Friends,
    FriendsOfFriends,
    Private,
}

/// Message permission levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePermission {
    Everyone,
    Friends,
    FriendsOfFriends,
    NoOne,
}

/// Avatar achievements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub achievement_id: Uuid,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub earned_at: DateTime<Utc>,
    pub points: i32,
    pub category: AchievementCategory,
}

/// Achievement categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementCategory {
    Exploration,
    Social,
    Building,
    Economy,
    Events,
    Special,
}

/// Avatar persistence data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarPersistenceData {
    pub last_position: Vector3,
    pub last_rotation: Quaternion,
    pub last_region: Uuid,
    pub session_time: i64, // seconds
    pub total_time: i64, // seconds
    pub visit_count: i64,
    pub last_login: DateTime<Utc>,
    pub inventory_snapshot: Option<String>, // JSON snapshot
    pub preferences: AvatarPreferences,
}

/// Avatar user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarPreferences {
    pub auto_pilot: bool,
    pub camera_constraints: bool,
    pub ui_size: f32,
    pub draw_distance: f32,
    pub audio_volume: f32,
    pub graphics_quality: GraphicsQuality,
    pub notification_settings: NotificationSettings,
}

/// Graphics quality settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphicsQuality {
    Low,
    Medium,
    High,
    Ultra,
    Custom {
        texture_detail: f32,
        lighting_quality: f32,
        shadow_quality: f32,
        particle_count: i32,
    },
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub friend_online: bool,
    pub friend_offline: bool,
    pub messages: bool,
    pub group_notices: bool,
    pub inventory_offers: bool,
    pub teleport_offers: bool,
    pub friendship_offers: bool,
    pub payment_info: bool,
}

/// Error types for avatar system
#[derive(Debug, thiserror::Error)]
pub enum AvatarError {
    #[error("Avatar not found: {id}")]
    NotFound { id: Uuid },
    
    #[error("Invalid avatar data: {reason}")]
    InvalidData { reason: String },
    
    #[error("Permission denied for avatar operation")]
    PermissionDenied,
    
    #[error("Avatar system error: {message}")]
    SystemError { message: String },
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] anyhow::Error),
}

/// Result type for avatar operations
pub type AvatarResult<T> = Result<T, AvatarError>;

/// Avatar friend information for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarFriend {
    pub friend_id: Uuid,
    pub friend_name: String,
    pub online_status: bool,
    pub added_at: DateTime<Utc>,
}

/// Avatar message for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarMessage {
    pub message_id: Uuid,
    pub from_id: Uuid,
    pub from_name: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
}

impl Default for Vector3 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
}

impl Default for AvatarProportions {
    fn default() -> Self {
        Self {
            body_height: 1.0,
            body_width: 1.0,
            head_size: 1.0,
            leg_length: 1.0,
            arm_length: 1.0,
            torso_length: 1.0,
        }
    }
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            profile_visibility: VisibilityLevel::Public,
            online_status_visibility: VisibilityLevel::Friends,
            location_visibility: VisibilityLevel::Friends,
            friend_list_visibility: VisibilityLevel::Friends,
            allow_friend_requests: true,
            allow_messages: MessagePermission::Friends,
            allow_voice_calls: true,
        }
    }
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            voice_enabled: false,
            voice_channel: None,
            voice_volume: 1.0,
            voice_effects: Vec::new(),
            spatial_audio: true,
            voice_modulation: None,
        }
    }
}

impl Default for AvatarPreferences {
    fn default() -> Self {
        Self {
            auto_pilot: false,
            camera_constraints: true,
            ui_size: 1.0,
            draw_distance: 128.0,
            audio_volume: 1.0,
            graphics_quality: GraphicsQuality::Medium,
            notification_settings: NotificationSettings::default(),
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            friend_online: true,
            friend_offline: true,
            messages: true,
            group_notices: true,
            inventory_offers: true,
            teleport_offers: true,
            friendship_offers: true,
            payment_info: true,
        }
    }
}