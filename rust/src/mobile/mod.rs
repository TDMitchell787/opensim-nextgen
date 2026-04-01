// OpenSim Next - Phase 33A.1 Native Mobile VR Applications
// Universal mobile client platform supporting iOS/Android VR
// Building on Phase 32 VR infrastructure for mobile optimization

use crate::vr::VRManager;
use crate::monitoring::metrics::MetricsCollector;
use crate::database::DatabaseManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::{Result, Error as AnyhowError};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct MobileRuntimeManager {
    config: MobileConfig,
    vr_manager: Arc<VRManager>,
    platform_adapters: Arc<RwLock<HashMap<MobilePlatform, Arc<dyn PlatformAdapter>>>>,
    performance_optimizer: Arc<MobilePerformanceOptimizer>,
    touch_interface: Arc<TouchInterfaceManager>,
    offline_cache: Arc<OfflineCacheManager>,
    metrics: Arc<MetricsCollector>,
    db: Arc<DatabaseManager>,
    active_sessions: Arc<RwLock<HashMap<Uuid, MobileSession>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileConfig {
    pub platform_support: PlatformSupport,
    pub performance_settings: PerformanceSettings,
    pub offline_capabilities: OfflineCapabilities,
    pub touch_interface_config: TouchInterfaceConfig,
    pub mobile_vr_settings: MobileVRSettings,
    pub security_settings: MobileSecuritySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSupport {
    pub ios_support: bool,
    pub android_support: bool,
    pub gear_vr_support: bool,
    pub daydream_support: bool,
    pub cardboard_support: bool,
    pub oculus_go_support: bool,
    pub min_ios_version: String,
    pub min_android_version: String,
    pub supported_architectures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub auto_quality_adjustment: bool,
    pub target_fps: u32,
    pub max_render_resolution: (u32, u32),
    pub texture_compression: TextureCompression,
    pub lod_settings: LevelOfDetailSettings,
    pub background_processing: bool,
    pub thermal_throttling: bool,
    pub battery_optimization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineCapabilities {
    pub offline_mode_enabled: bool,
    pub max_cache_size_mb: u64,
    pub cache_expiry_hours: u32,
    pub offline_avatar_support: bool,
    pub offline_region_cache: bool,
    pub sync_on_reconnect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchInterfaceConfig {
    pub gesture_recognition: bool,
    pub haptic_feedback: bool,
    pub voice_commands: bool,
    pub ui_scaling: f32,
    pub button_layout: ButtonLayout,
    pub accessibility_features: AccessibilityFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileVRSettings {
    pub mobile_vr_enabled: bool,
    pub vr_ui_distance: f32,
    pub comfort_settings: ComfortSettings,
    pub hand_tracking: bool,
    pub gaze_tracking: bool,
    pub controller_support: ControllerSupport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileSecuritySettings {
    pub biometric_auth: bool,
    pub device_encryption: bool,
    pub secure_storage: bool,
    pub network_security: NetworkSecurity,
    pub app_integrity_checks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MobilePlatform {
    iOS,
    Android,
    iPadOS,
    AndroidTablet,
    GearVR,
    Daydream,
    Cardboard,
    OculusGo,
    OculusMobile,
}

impl std::fmt::Display for MobilePlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MobilePlatform::iOS => write!(f, "ios"),
            MobilePlatform::Android => write!(f, "android"),
            MobilePlatform::iPadOS => write!(f, "ipados"),
            MobilePlatform::AndroidTablet => write!(f, "android_tablet"),
            MobilePlatform::GearVR => write!(f, "gear_vr"),
            MobilePlatform::Daydream => write!(f, "daydream"),
            MobilePlatform::Cardboard => write!(f, "cardboard"),
            MobilePlatform::OculusGo => write!(f, "oculus_go"),
            MobilePlatform::OculusMobile => write!(f, "oculus_mobile"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureCompression {
    PVRTC,      // iOS
    ETC2,       // Android
    ASTC,       // Universal
    Adaptive,   // Auto-select based on platform
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelOfDetailSettings {
    pub avatar_lod_levels: u32,
    pub object_lod_levels: u32,
    pub texture_lod_levels: u32,
    pub distance_culling: bool,
    pub frustum_culling: bool,
    pub occlusion_culling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonLayout {
    pub layout_type: LayoutType,
    pub button_size: ButtonSize,
    pub button_opacity: f32,
    pub customizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    Classic,
    Modern,
    Minimalist,
    Accessibility,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityFeatures {
    pub voice_over_support: bool,
    pub high_contrast_mode: bool,
    pub large_text_support: bool,
    pub motor_accessibility: bool,
    pub cognitive_assistance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfortSettings {
    pub comfort_vignette: bool,
    pub snap_turning: bool,
    pub teleport_locomotion: bool,
    pub comfort_level: ComfortLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComfortLevel {
    Comfortable,    // Maximum comfort features
    Moderate,       // Balanced comfort/immersion
    Intense,        // Minimal comfort features
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerSupport {
    pub bluetooth_controllers: bool,
    pub gamepad_support: bool,
    pub custom_controller_mapping: bool,
    pub gesture_controllers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurity {
    pub ssl_pinning: bool,
    pub certificate_validation: bool,
    pub network_monitoring: bool,
    pub secure_protocols_only: bool,
}

#[derive(Debug, Clone)]
pub struct MobileSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub platform: MobilePlatform,
    pub device_info: DeviceInfo,
    pub performance_metrics: PerformanceMetrics,
    pub vr_mode_active: bool,
    pub offline_mode_active: bool,
    pub last_sync: DateTime<Utc>,
    pub session_start: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_model: String,
    pub os_version: String,
    pub screen_resolution: (u32, u32),
    pub screen_density: f32,
    pub gpu_model: String,
    pub ram_total_mb: u64,
    pub storage_available_mb: u64,
    pub battery_level: Option<f32>,
    pub thermal_state: ThermalState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThermalState {
    Nominal,
    Fair,
    Serious,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub current_fps: f32,
    pub frame_time_ms: f32,
    pub gpu_utilization: f32,
    pub cpu_utilization: f32,
    pub memory_usage_mb: u64,
    pub battery_drain_rate: f32,
    pub thermal_level: f32,
    pub quality_level: QualityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityLevel {
    Low,
    Medium,
    High,
    Ultra,
    Adaptive,
}

// Platform adapter trait for iOS/Android specific implementations
#[async_trait::async_trait]
pub trait PlatformAdapter: Send + Sync + std::fmt::Debug {
    async fn initialize(&self, config: &MobileConfig) -> Result<()>;
    async fn create_graphics_context(&self) -> Result<Box<dyn GraphicsContext>>;
    async fn setup_input_system(&self) -> Result<Box<dyn InputSystem>>;
    async fn configure_performance(&self, settings: &PerformanceSettings) -> Result<()>;
    async fn enable_offline_mode(&self, cache_config: &OfflineCapabilities) -> Result<()>;
    async fn get_device_capabilities(&self) -> Result<DeviceCapabilities>;
    async fn optimize_for_platform(&self, session: &MobileSession) -> Result<()>;
}

#[async_trait::async_trait]
pub trait GraphicsContext: Send + Sync {
    async fn create_mobile_renderer(&self) -> Result<Box<dyn MobileRenderer>>;
    async fn setup_vr_rendering(&self) -> Result<Box<dyn MobileVRRenderer>>;
    async fn configure_quality_settings(&self, quality: QualityLevel) -> Result<()>;
    async fn enable_performance_monitoring(&self) -> Result<()>;
}

#[async_trait::async_trait]
pub trait MobileRenderer: Send + Sync {
    async fn render_frame(&self, scene_data: &SceneData) -> Result<RenderedFrame>;
    async fn adjust_quality(&self, target_fps: f32) -> Result<QualityLevel>;
    async fn enable_thermal_throttling(&self) -> Result<()>;
    async fn optimize_for_battery(&self) -> Result<()>;
}

#[async_trait::async_trait]
pub trait MobileVRRenderer: Send + Sync {
    async fn render_vr_frame(&self, vr_data: &VRFrameData) -> Result<VRRenderedFrame>;
    async fn configure_mobile_vr(&self, settings: &MobileVRSettings) -> Result<()>;
    async fn enable_foveated_rendering(&self) -> Result<()>;
    async fn adjust_vr_quality(&self, performance: &PerformanceMetrics) -> Result<()>;
}

#[async_trait::async_trait]
pub trait InputSystem: Send + Sync {
    async fn initialize_touch_input(&self) -> Result<()>;
    async fn enable_gesture_recognition(&self) -> Result<()>;
    async fn setup_voice_commands(&self) -> Result<()>;
    async fn configure_accessibility(&self, features: &AccessibilityFeatures) -> Result<()>;
    async fn process_input_events(&self) -> Result<Vec<InputEvent>>;
}

#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    pub supports_vr: bool,
    pub supports_ar: bool,
    pub max_texture_size: u32,
    pub supports_compute_shaders: bool,
    pub supports_instancing: bool,
    pub vulkan_support: bool,
    pub metal_support: bool,
    pub opengl_es_version: String,
    pub max_draw_calls: u32,
    pub max_vertices: u32,
}

#[derive(Debug, Clone)]
pub struct SceneData {
    pub avatars: Vec<AvatarRenderData>,
    pub objects: Vec<ObjectRenderData>,
    pub terrain: TerrainRenderData,
    pub lighting: LightingData,
    pub camera: CameraData,
}

#[derive(Debug, Clone)]
pub struct AvatarRenderData {
    pub avatar_id: Uuid,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub lod_level: u32,
    pub textures: Vec<TextureHandle>,
    pub animations: Vec<AnimationState>,
}

#[derive(Debug, Clone)]
pub struct ObjectRenderData {
    pub object_id: Uuid,
    pub mesh_data: MeshHandle,
    pub transform: TransformMatrix,
    pub material: MaterialHandle,
    pub visibility: bool,
}

#[derive(Debug, Clone)]
pub struct RenderedFrame {
    pub frame_buffer: FrameBuffer,
    pub render_time_ms: f32,
    pub draw_calls: u32,
    pub vertices_rendered: u32,
    pub quality_level: QualityLevel,
}

#[derive(Debug, Clone)]
pub struct VRFrameData {
    pub left_eye_data: EyeRenderData,
    pub right_eye_data: EyeRenderData,
    pub head_pose: HeadPose,
    pub controller_data: Vec<ControllerData>,
}

#[derive(Debug, Clone)]
pub struct VRRenderedFrame {
    pub left_eye_buffer: FrameBuffer,
    pub right_eye_buffer: FrameBuffer,
    pub render_time_ms: f32,
    pub vr_metrics: VRPerformanceMetrics,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Touch { x: f32, y: f32, phase: TouchPhase },
    Gesture { gesture_type: GestureType, data: GestureData },
    Voice { command: String, confidence: f32 },
    Controller { controller_id: u32, input_data: ControllerInput },
    Biometric { auth_result: BiometricResult },
}

#[derive(Debug, Clone)]
pub enum TouchPhase {
    Began,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum GestureType {
    Tap,
    DoubleTap,
    LongPress,
    Pinch,
    Pan,
    Swipe,
    Rotation,
}

#[derive(Debug, Clone)]
pub struct GestureData {
    pub center: [f32; 2],
    pub velocity: [f32; 2],
    pub scale: f32,
    pub rotation: f32,
}

// Type aliases for cross-compatibility
pub type FrameBuffer = Vec<u8>;
pub type TextureHandle = u32;
pub type MeshHandle = u32;
pub type MaterialHandle = u32;
pub type TransformMatrix = [[f32; 4]; 4];
pub type TerrainRenderData = Vec<u8>; // Placeholder
pub type LightingData = Vec<u8>; // Placeholder
pub type CameraData = Vec<u8>; // Placeholder
pub type AnimationState = Vec<u8>; // Placeholder
pub type EyeRenderData = Vec<u8>; // Placeholder
pub type HeadPose = [f32; 7]; // Position + Quaternion
pub type ControllerData = Vec<u8>; // Placeholder
pub type VRPerformanceMetrics = Vec<u8>; // Placeholder
pub type ControllerInput = Vec<u8>; // Placeholder
pub type BiometricResult = bool; // Placeholder

impl Default for MobileConfig {
    fn default() -> Self {
        Self {
            platform_support: PlatformSupport {
                ios_support: true,
                android_support: true,
                gear_vr_support: true,
                daydream_support: true,
                cardboard_support: true,
                oculus_go_support: true,
                min_ios_version: "13.0".to_string(),
                min_android_version: "7.0".to_string(),
                supported_architectures: vec![
                    "arm64".to_string(),
                    "armv7".to_string(),
                    "x86_64".to_string(),
                ],
            },
            performance_settings: PerformanceSettings {
                auto_quality_adjustment: true,
                target_fps: 60,
                max_render_resolution: (1920, 1080),
                texture_compression: TextureCompression::Adaptive,
                lod_settings: LevelOfDetailSettings {
                    avatar_lod_levels: 5,
                    object_lod_levels: 4,
                    texture_lod_levels: 3,
                    distance_culling: true,
                    frustum_culling: true,
                    occlusion_culling: false,
                },
                background_processing: true,
                thermal_throttling: true,
                battery_optimization: true,
            },
            offline_capabilities: OfflineCapabilities {
                offline_mode_enabled: true,
                max_cache_size_mb: 2048,
                cache_expiry_hours: 24,
                offline_avatar_support: true,
                offline_region_cache: true,
                sync_on_reconnect: true,
            },
            touch_interface_config: TouchInterfaceConfig {
                gesture_recognition: true,
                haptic_feedback: true,
                voice_commands: true,
                ui_scaling: 1.0,
                button_layout: ButtonLayout {
                    layout_type: LayoutType::Modern,
                    button_size: ButtonSize::Medium,
                    button_opacity: 0.8,
                    customizable: true,
                },
                accessibility_features: AccessibilityFeatures {
                    voice_over_support: true,
                    high_contrast_mode: true,
                    large_text_support: true,
                    motor_accessibility: true,
                    cognitive_assistance: true,
                },
            },
            mobile_vr_settings: MobileVRSettings {
                mobile_vr_enabled: true,
                vr_ui_distance: 2.0,
                comfort_settings: ComfortSettings {
                    comfort_vignette: true,
                    snap_turning: true,
                    teleport_locomotion: true,
                    comfort_level: ComfortLevel::Comfortable,
                },
                hand_tracking: true,
                gaze_tracking: true,
                controller_support: ControllerSupport {
                    bluetooth_controllers: true,
                    gamepad_support: true,
                    custom_controller_mapping: true,
                    gesture_controllers: true,
                },
            },
            security_settings: MobileSecuritySettings {
                biometric_auth: true,
                device_encryption: true,
                secure_storage: true,
                network_security: NetworkSecurity {
                    ssl_pinning: true,
                    certificate_validation: true,
                    network_monitoring: true,
                    secure_protocols_only: true,
                },
                app_integrity_checks: true,
            },
        }
    }
}

impl MobileRuntimeManager {
    pub async fn new(
        config: MobileConfig,
        vr_manager: Arc<VRManager>,
        metrics: Arc<MetricsCollector>,
        db: Arc<DatabaseManager>,
    ) -> Result<Arc<Self>> {
        let manager = Arc::new(Self {
            config: config.clone(),
            vr_manager,
            platform_adapters: Arc::new(RwLock::new(HashMap::new())),
            performance_optimizer: Arc::new(MobilePerformanceOptimizer::new(config.performance_settings.clone()).await?),
            touch_interface: Arc::new(TouchInterfaceManager::new(config.touch_interface_config.clone()).await?),
            offline_cache: Arc::new(OfflineCacheManager::new(config.offline_capabilities.clone()).await?),
            metrics: metrics.clone(),
            db,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        });

        // Initialize platform adapters
        manager.initialize_platform_adapters().await?;

        // Initialize performance monitoring
        manager.start_performance_monitoring().await?;

        Ok(manager)
    }

    async fn initialize_platform_adapters(&self) -> Result<()> {
        let mut adapters = self.platform_adapters.write().await;

        if self.config.platform_support.ios_support {
            // Note: In a real implementation, this would load the iOS adapter
            // adapters.insert(MobilePlatform::iOS, Arc::new(IOSAdapter::new().await?));
        }

        if self.config.platform_support.android_support {
            // Note: In a real implementation, this would load the Android adapter
            // adapters.insert(MobilePlatform::Android, Arc::new(AndroidAdapter::new().await?));
        }

        Ok(())
    }

    async fn start_performance_monitoring(&self) -> Result<()> {
        let optimizer = self.performance_optimizer.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

            loop {
                interval.tick().await;
                if let Err(e) = optimizer.collect_performance_metrics(&metrics).await {
                    eprintln!("Error collecting mobile performance metrics: {}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn create_mobile_session(
        &self,
        user_id: Uuid,
        platform: MobilePlatform,
        device_info: DeviceInfo,
    ) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        
        let session = MobileSession {
            session_id,
            user_id,
            platform: platform.clone(),
            device_info: device_info.clone(),
            performance_metrics: PerformanceMetrics {
                current_fps: 0.0,
                frame_time_ms: 0.0,
                gpu_utilization: 0.0,
                cpu_utilization: 0.0,
                memory_usage_mb: 0,
                battery_drain_rate: 0.0,
                thermal_level: 0.0,
                quality_level: QualityLevel::Medium,
            },
            vr_mode_active: false,
            offline_mode_active: false,
            last_sync: Utc::now(),
            session_start: Utc::now(),
        };

        self.active_sessions.write().await.insert(session_id, session);
        
        // Record session creation metrics
        self.metrics.record_mobile_session_created(user_id, &platform.to_string()).await;

        Ok(session_id)
    }

    pub async fn enable_vr_mode(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.vr_mode_active = true;
            
            // Configure mobile VR settings
            self.configure_mobile_vr(session).await?;
            
            self.metrics.record_mobile_vr_enabled(session.user_id).await;
        }
        Ok(())
    }

    async fn configure_mobile_vr(&self, session: &MobileSession) -> Result<()> {
        // Configure VR-specific settings for mobile
        let vr_settings = &self.config.mobile_vr_settings;
        
        // Apply comfort settings
        self.apply_comfort_settings(&vr_settings.comfort_settings, session).await?;
        
        // Configure controllers
        self.setup_mobile_controllers(&vr_settings.controller_support, session).await?;

        Ok(())
    }

    async fn apply_comfort_settings(
        &self,
        comfort: &ComfortSettings,
        _session: &MobileSession,
    ) -> Result<()> {
        // Apply comfort settings based on user preferences and device capabilities
        // This would configure vignetting, turning methods, locomotion, etc.
        Ok(())
    }

    async fn setup_mobile_controllers(
        &self,
        controller_support: &ControllerSupport,
        _session: &MobileSession,
    ) -> Result<()> {
        // Setup controller support for the mobile session
        // This would handle Bluetooth controllers, gamepads, gesture recognition, etc.
        Ok(())
    }

    pub async fn get_active_sessions(&self) -> Vec<MobileSession> {
        self.active_sessions.read().await.values().cloned().collect()
    }

    pub async fn optimize_performance(&self, session_id: Uuid) -> Result<()> {
        if let Some(session) = self.active_sessions.read().await.get(&session_id) {
            self.performance_optimizer.optimize_for_session(session).await?;
        }
        Ok(())
    }
}

// Placeholder implementations for required components
#[derive(Debug)]
pub struct MobilePerformanceOptimizer {
    settings: PerformanceSettings,
}

impl MobilePerformanceOptimizer {
    pub async fn new(settings: PerformanceSettings) -> Result<Self> {
        Ok(Self { settings })
    }

    pub async fn collect_performance_metrics(&self, metrics: &MetricsCollector) -> Result<()> {
        // Collect and record mobile-specific performance metrics
        metrics.record_custom_metric("mobile_performance_check", 1.0, HashMap::new()).await?;
        Ok(())
    }

    pub async fn optimize_for_session(&self, _session: &MobileSession) -> Result<()> {
        // Implement session-specific optimization
        Ok(())
    }
}

#[derive(Debug)]
pub struct TouchInterfaceManager {
    config: TouchInterfaceConfig,
}

impl TouchInterfaceManager {
    pub async fn new(config: TouchInterfaceConfig) -> Result<Self> {
        Ok(Self { config })
    }
}

#[derive(Debug)]
pub struct OfflineCacheManager {
    config: OfflineCapabilities,
}

impl OfflineCacheManager {
    pub async fn new(config: OfflineCapabilities) -> Result<Self> {
        Ok(Self { config })
    }
}

// Extension trait for metrics collector to add mobile-specific metrics
impl MetricsCollector {
    pub async fn record_mobile_session_created(&self, user_id: Uuid, platform: &str) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());
        tags.insert("platform".to_string(), platform.to_string());
        
        let _ = self.record_custom_metric("mobile_sessions_created_total", 1.0, tags).await;
    }

    pub async fn record_mobile_vr_enabled(&self, user_id: Uuid) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());
        
        let _ = self.record_custom_metric("mobile_vr_enabled_total", 1.0, tags).await;
    }

    pub async fn record_mobile_performance(&self, session_id: Uuid, metrics: &PerformanceMetrics) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());
        
        let _ = self.record_custom_metric("mobile_fps", metrics.current_fps as f64, tags.clone()).await;
        let _ = self.record_custom_metric("mobile_frame_time_ms", metrics.frame_time_ms as f64, tags.clone()).await;
        let _ = self.record_custom_metric("mobile_gpu_utilization", metrics.gpu_utilization as f64, tags.clone()).await;
        let _ = self.record_custom_metric("mobile_memory_usage_mb", metrics.memory_usage_mb as f64, tags).await;
    }
}