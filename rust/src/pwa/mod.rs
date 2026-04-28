// OpenSim Next - Phase 33A.2 Progressive Web App (PWA) Enhancement
// Advanced PWA capabilities with WebXR, service workers, and offline functionality
// Building on existing web infrastructure for universal browser access

// WebSocket integration handled through network layer
use crate::database::DatabaseManager;
use crate::monitoring::metrics::MetricsCollector;
use anyhow::{Error as AnyhowError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PWAServiceManager {
    config: PWAConfig,
    service_worker: Arc<ServiceWorkerManager>,
    webxr_manager: Arc<WebXRManager>,
    push_notification: Arc<PushNotificationManager>,
    offline_storage: Arc<OfflineStorageManager>,
    manifest_generator: Arc<ManifestGenerator>,
    performance_monitor: Arc<PWAPerformanceMonitor>,
    metrics: Arc<MetricsCollector>,
    db: Arc<DatabaseManager>,
    active_pwa_sessions: Arc<RwLock<HashMap<Uuid, PWASession>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PWAConfig {
    pub pwa_features: PWAFeatures,
    pub webxr_settings: WebXRSettings,
    pub service_worker_config: ServiceWorkerConfig,
    pub offline_capabilities: OfflineCapabilities,
    pub push_notifications: PushNotificationConfig,
    pub manifest_settings: ManifestSettings,
    pub performance_settings: PWAPerformanceSettings,
    pub security_settings: PWASecuritySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PWAFeatures {
    pub offline_support: bool,
    pub push_notifications: bool,
    pub background_sync: bool,
    pub install_prompt: bool,
    pub native_like_experience: bool,
    pub webxr_support: bool,
    pub web_share_api: bool,
    pub web_auth_api: bool,
    pub payment_request_api: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebXRSettings {
    pub webxr_enabled: bool,
    pub vr_support: bool,
    pub ar_support: bool,
    pub immersive_vr: bool,
    pub immersive_ar: bool,
    pub inline_ar: bool,
    pub hand_tracking: bool,
    pub eye_tracking: bool,
    pub spatial_tracking: bool,
    pub performance_optimization: WebXRPerformanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebXRPerformanceSettings {
    pub adaptive_quality: bool,
    pub foveated_rendering: bool,
    pub frame_rate_target: u32,
    pub resolution_scaling: f32,
    pub webgl_optimization: bool,
    pub webgpu_support: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWorkerConfig {
    pub cache_strategy: CacheStrategy,
    pub update_strategy: UpdateStrategy,
    pub background_sync: bool,
    pub periodic_sync: bool,
    pub cache_max_size_mb: u64,
    pub cache_expiry_hours: u32,
    pub preload_critical_resources: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy {
    CacheFirst,
    NetworkFirst,
    StaleWhileRevalidate,
    NetworkOnly,
    CacheOnly,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateStrategy {
    Immediate,
    OnNextStart,
    UserPrompt,
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineCapabilities {
    pub offline_mode: bool,
    pub offline_avatar_cache: bool,
    pub offline_region_data: bool,
    pub offline_messaging: bool,
    pub sync_on_reconnect: bool,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    ServerWins,
    ClientWins,
    UserChoice,
    MergeChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotificationConfig {
    pub push_enabled: bool,
    pub vapid_keys: VAPIDKeys,
    pub notification_types: Vec<NotificationType>,
    pub user_preferences: NotificationPreferences,
    pub delivery_settings: DeliverySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VAPIDKeys {
    pub public_key: String,
    pub private_key: String,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    FriendRequest,
    DirectMessage,
    GroupMessage,
    RegionEvent,
    SystemNotification,
    CustomEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub enabled_types: Vec<NotificationType>,
    pub quiet_hours: Option<QuietHours>,
    pub frequency_limit: FrequencyLimit,
    pub priority_filtering: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub start_time: String, // "22:00"
    pub end_time: String,   // "08:00"
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyLimit {
    pub max_per_hour: u32,
    pub max_per_day: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverySettings {
    pub retry_attempts: u32,
    pub retry_delay_seconds: u32,
    pub ttl_seconds: u32,
    pub urgency: NotificationUrgency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationUrgency {
    VeryLow,
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSettings {
    pub app_name: String,
    pub short_name: String,
    pub description: String,
    pub theme_color: String,
    pub background_color: String,
    pub display_mode: DisplayMode,
    pub orientation: Orientation,
    pub icons: Vec<IconDefinition>,
    pub categories: Vec<String>,
    pub screenshots: Vec<ScreenshotDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayMode {
    Fullscreen,
    Standalone,
    MinimalUI,
    Browser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Orientation {
    Any,
    Natural,
    Landscape,
    Portrait,
    LandscapePrimary,
    LandscapeSecondary,
    PortraitPrimary,
    PortraitSecondary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconDefinition {
    pub src: String,
    pub sizes: String,
    pub icon_type: String,
    pub purpose: IconPurpose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IconPurpose {
    Any,
    Maskable,
    Monochrome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotDefinition {
    pub src: String,
    pub sizes: String,
    pub screenshot_type: String,
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PWAPerformanceSettings {
    pub lazy_loading: bool,
    pub code_splitting: bool,
    pub resource_preloading: bool,
    pub compression: CompressionSettings,
    pub caching_optimization: bool,
    pub bundle_optimization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionSettings {
    pub gzip_enabled: bool,
    pub brotli_enabled: bool,
    pub compression_level: u32,
    pub mime_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PWASecuritySettings {
    pub content_security_policy: CSPConfig,
    pub https_only: bool,
    pub secure_contexts: bool,
    pub integrity_checks: bool,
    pub permission_management: PermissionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSPConfig {
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub frame_src: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    pub request_strategy: PermissionStrategy,
    pub permission_types: Vec<WebPermission>,
    pub graceful_degradation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionStrategy {
    UpfrontRequest,
    JustInTime,
    ContextualRequest,
    OptionalFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebPermission {
    Camera,
    Microphone,
    Geolocation,
    Notifications,
    PersistentStorage,
    BackgroundSync,
    ClipboardRead,
    ClipboardWrite,
}

#[derive(Debug, Clone)]
pub struct PWASession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub user_agent: String,
    pub platform_info: PlatformInfo,
    pub webxr_active: bool,
    pub offline_mode: bool,
    pub pwa_installed: bool,
    pub notification_permission: NotificationPermission,
    pub performance_metrics: PWAPerformanceMetrics,
    pub session_start: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub browser: String,
    pub browser_version: String,
    pub os: String,
    pub os_version: String,
    pub device_type: DeviceType,
    pub screen_resolution: (u32, u32),
    pub supports_webxr: bool,
    pub supports_service_worker: bool,
    pub supports_push_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    VRHeadset,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPermission {
    Default,
    Granted,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PWAPerformanceMetrics {
    pub page_load_time_ms: f32,
    pub time_to_interactive_ms: f32,
    pub first_contentful_paint_ms: f32,
    pub largest_contentful_paint_ms: f32,
    pub cumulative_layout_shift: f32,
    pub service_worker_cache_hit_rate: f32,
    pub offline_usage_percentage: f32,
}

impl Default for PWAConfig {
    fn default() -> Self {
        Self {
            pwa_features: PWAFeatures {
                offline_support: true,
                push_notifications: true,
                background_sync: true,
                install_prompt: true,
                native_like_experience: true,
                webxr_support: true,
                web_share_api: true,
                web_auth_api: true,
                payment_request_api: false,
            },
            webxr_settings: WebXRSettings {
                webxr_enabled: true,
                vr_support: true,
                ar_support: true,
                immersive_vr: true,
                immersive_ar: true,
                inline_ar: true,
                hand_tracking: true,
                eye_tracking: false,
                spatial_tracking: true,
                performance_optimization: WebXRPerformanceSettings {
                    adaptive_quality: true,
                    foveated_rendering: false, // Limited browser support
                    frame_rate_target: 60,
                    resolution_scaling: 1.0,
                    webgl_optimization: true,
                    webgpu_support: true,
                },
            },
            service_worker_config: ServiceWorkerConfig {
                cache_strategy: CacheStrategy::StaleWhileRevalidate,
                update_strategy: UpdateStrategy::UserPrompt,
                background_sync: true,
                periodic_sync: false, // Limited browser support
                cache_max_size_mb: 1024,
                cache_expiry_hours: 24,
                preload_critical_resources: true,
            },
            offline_capabilities: OfflineCapabilities {
                offline_mode: true,
                offline_avatar_cache: true,
                offline_region_data: true,
                offline_messaging: true,
                sync_on_reconnect: true,
                conflict_resolution: ConflictResolution::UserChoice,
            },
            push_notifications: PushNotificationConfig {
                push_enabled: true,
                vapid_keys: VAPIDKeys {
                    public_key: "".to_string(),  // Generated at runtime
                    private_key: "".to_string(), // Generated at runtime
                    subject: "mailto:admin@opensim.org".to_string(),
                },
                notification_types: vec![
                    NotificationType::FriendRequest,
                    NotificationType::DirectMessage,
                    NotificationType::GroupMessage,
                    NotificationType::RegionEvent,
                    NotificationType::SystemNotification,
                ],
                user_preferences: NotificationPreferences {
                    enabled_types: vec![
                        NotificationType::DirectMessage,
                        NotificationType::FriendRequest,
                    ],
                    quiet_hours: Some(QuietHours {
                        start_time: "22:00".to_string(),
                        end_time: "08:00".to_string(),
                        timezone: "UTC".to_string(),
                    }),
                    frequency_limit: FrequencyLimit {
                        max_per_hour: 10,
                        max_per_day: 50,
                        burst_limit: 3,
                    },
                    priority_filtering: true,
                },
                delivery_settings: DeliverySettings {
                    retry_attempts: 3,
                    retry_delay_seconds: 60,
                    ttl_seconds: 3600,
                    urgency: NotificationUrgency::Normal,
                },
            },
            manifest_settings: ManifestSettings {
                app_name: "OpenSim Next".to_string(),
                short_name: "OpenSim".to_string(),
                description: "Revolutionary VR/XR-Enhanced Virtual World Platform".to_string(),
                theme_color: "#1976d2".to_string(),
                background_color: "#ffffff".to_string(),
                display_mode: DisplayMode::Standalone,
                orientation: Orientation::Any,
                icons: vec![
                    IconDefinition {
                        src: "/icons/icon-192x192.png".to_string(),
                        sizes: "192x192".to_string(),
                        icon_type: "image/png".to_string(),
                        purpose: IconPurpose::Any,
                    },
                    IconDefinition {
                        src: "/icons/icon-512x512.png".to_string(),
                        sizes: "512x512".to_string(),
                        icon_type: "image/png".to_string(),
                        purpose: IconPurpose::Any,
                    },
                ],
                categories: vec![
                    "social".to_string(),
                    "entertainment".to_string(),
                    "productivity".to_string(),
                ],
                screenshots: vec![ScreenshotDefinition {
                    src: "/screenshots/desktop-wide.png".to_string(),
                    sizes: "1280x720".to_string(),
                    screenshot_type: "image/png".to_string(),
                    platform: Some("wide".to_string()),
                }],
            },
            performance_settings: PWAPerformanceSettings {
                lazy_loading: true,
                code_splitting: true,
                resource_preloading: true,
                compression: CompressionSettings {
                    gzip_enabled: true,
                    brotli_enabled: true,
                    compression_level: 6,
                    mime_types: vec![
                        "text/html".to_string(),
                        "text/css".to_string(),
                        "application/javascript".to_string(),
                        "application/json".to_string(),
                    ],
                },
                caching_optimization: true,
                bundle_optimization: true,
            },
            security_settings: PWASecuritySettings {
                content_security_policy: CSPConfig {
                    default_src: vec!["'self'".to_string()],
                    script_src: vec!["'self'".to_string(), "'unsafe-eval'".to_string()], // For WebAssembly
                    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
                    img_src: vec![
                        "'self'".to_string(),
                        "data:".to_string(),
                        "blob:".to_string(),
                    ],
                    connect_src: vec!["'self'".to_string(), "wss:".to_string(), "ws:".to_string()],
                    frame_src: vec!["'none'".to_string()],
                },
                https_only: true,
                secure_contexts: true,
                integrity_checks: true,
                permission_management: PermissionConfig {
                    request_strategy: PermissionStrategy::JustInTime,
                    permission_types: vec![
                        WebPermission::Camera,
                        WebPermission::Microphone,
                        WebPermission::Notifications,
                        WebPermission::PersistentStorage,
                    ],
                    graceful_degradation: true,
                },
            },
        }
    }
}

impl PWAServiceManager {
    pub async fn new(
        config: PWAConfig,
        metrics: Arc<MetricsCollector>,
        db: Arc<DatabaseManager>,
    ) -> Result<Arc<Self>> {
        let manager = Arc::new(Self {
            config: config.clone(),
            service_worker: Arc::new(
                ServiceWorkerManager::new(config.service_worker_config.clone()).await?,
            ),
            webxr_manager: Arc::new(WebXRManager::new(config.webxr_settings.clone()).await?),
            push_notification: Arc::new(
                PushNotificationManager::new(config.push_notifications.clone()).await?,
            ),
            offline_storage: Arc::new(
                OfflineStorageManager::new(config.offline_capabilities.clone()).await?,
            ),
            manifest_generator: Arc::new(
                ManifestGenerator::new(config.manifest_settings.clone()).await?,
            ),
            performance_monitor: Arc::new(
                PWAPerformanceMonitor::new(config.performance_settings.clone()).await?,
            ),
            metrics: metrics.clone(),
            db,
            active_pwa_sessions: Arc::new(RwLock::new(HashMap::new())),
        });

        // Initialize PWA services
        manager.initialize_services().await?;

        Ok(manager)
    }

    async fn initialize_services(&self) -> Result<()> {
        // Generate VAPID keys if not provided
        if self
            .config
            .push_notifications
            .vapid_keys
            .public_key
            .is_empty()
        {
            self.generate_vapid_keys().await?;
        }

        // Start background services
        self.start_background_services().await?;

        Ok(())
    }

    async fn generate_vapid_keys(&self) -> Result<()> {
        // In a real implementation, this would generate VAPID keys for push notifications
        // For now, this is a placeholder
        Ok(())
    }

    async fn start_background_services(&self) -> Result<()> {
        let service_worker = self.service_worker.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;
                if let Err(e) = service_worker.perform_background_sync(&metrics).await {
                    eprintln!("Error in PWA background sync: {}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn create_pwa_session(
        &self,
        user_id: Uuid,
        user_agent: String,
        platform_info: PlatformInfo,
    ) -> Result<Uuid> {
        let session_id = Uuid::new_v4();

        let session = PWASession {
            session_id,
            user_id,
            user_agent,
            platform_info: platform_info.clone(),
            webxr_active: false,
            offline_mode: false,
            pwa_installed: false,
            notification_permission: NotificationPermission::Default,
            performance_metrics: PWAPerformanceMetrics {
                page_load_time_ms: 0.0,
                time_to_interactive_ms: 0.0,
                first_contentful_paint_ms: 0.0,
                largest_contentful_paint_ms: 0.0,
                cumulative_layout_shift: 0.0,
                service_worker_cache_hit_rate: 0.0,
                offline_usage_percentage: 0.0,
            },
            session_start: Utc::now(),
            last_activity: Utc::now(),
        };

        self.active_pwa_sessions
            .write()
            .await
            .insert(session_id, session);

        // Record PWA session metrics
        self.metrics
            .record_pwa_session_created(user_id, &platform_info.browser)
            .await;

        Ok(session_id)
    }

    pub async fn enable_webxr(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.active_pwa_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            if session.platform_info.supports_webxr {
                session.webxr_active = true;
                self.webxr_manager
                    .initialize_webxr_session(session_id)
                    .await?;
                self.metrics
                    .record_webxr_session_started(session.user_id)
                    .await;
            } else {
                return Err(AnyhowError::msg("WebXR not supported on this platform"));
            }
        }
        Ok(())
    }

    pub async fn install_pwa(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.active_pwa_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.pwa_installed = true;
            self.metrics.record_pwa_installed(session.user_id).await;
        }
        Ok(())
    }

    pub async fn request_notification_permission(
        &self,
        session_id: Uuid,
    ) -> Result<NotificationPermission> {
        let mut sessions = self.active_pwa_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            // In a real implementation, this would trigger browser permission request
            session.notification_permission = NotificationPermission::Granted;
            self.metrics
                .record_notification_permission_granted(session.user_id)
                .await;
            return Ok(NotificationPermission::Granted);
        }
        Ok(NotificationPermission::Denied)
    }

    pub async fn send_push_notification(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
        payload: PushPayload,
    ) -> Result<()> {
        self.push_notification
            .send_notification(user_id, notification_type, payload)
            .await
    }

    pub async fn get_web_app_manifest(&self) -> Result<String> {
        self.manifest_generator.generate_manifest().await
    }

    pub async fn get_service_worker_script(&self) -> Result<String> {
        self.service_worker.generate_service_worker_script().await
    }

    pub async fn record_performance_metrics(
        &self,
        session_id: Uuid,
        metrics: PWAPerformanceMetrics,
    ) -> Result<()> {
        let mut sessions = self.active_pwa_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.performance_metrics = metrics.clone();
            self.metrics
                .record_pwa_performance(session_id, &metrics)
                .await;
        }
        Ok(())
    }
}

// Placeholder implementations for required components
#[derive(Debug)]
pub struct ServiceWorkerManager {
    config: ServiceWorkerConfig,
}

impl ServiceWorkerManager {
    pub async fn new(config: ServiceWorkerConfig) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn perform_background_sync(&self, metrics: &MetricsCollector) -> Result<()> {
        metrics
            .record_custom_metric("service_worker_background_sync", 1.0, HashMap::new())
            .await?;
        Ok(())
    }

    pub async fn generate_service_worker_script(&self) -> Result<String> {
        // Generate dynamic service worker JavaScript
        Ok(SERVICE_WORKER_TEMPLATE.to_string())
    }
}

#[derive(Debug)]
pub struct WebXRManager {
    config: WebXRSettings,
}

impl WebXRManager {
    pub async fn new(config: WebXRSettings) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn initialize_webxr_session(&self, _session_id: Uuid) -> Result<()> {
        // Initialize WebXR session
        Ok(())
    }
}

#[derive(Debug)]
pub struct PushNotificationManager {
    config: PushNotificationConfig,
}

impl PushNotificationManager {
    pub async fn new(config: PushNotificationConfig) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn send_notification(
        &self,
        _user_id: Uuid,
        _notification_type: NotificationType,
        _payload: PushPayload,
    ) -> Result<()> {
        // Send push notification
        Ok(())
    }
}

#[derive(Debug)]
pub struct OfflineStorageManager {
    config: OfflineCapabilities,
}

impl OfflineStorageManager {
    pub async fn new(config: OfflineCapabilities) -> Result<Self> {
        Ok(Self { config })
    }
}

#[derive(Debug)]
pub struct ManifestGenerator {
    config: ManifestSettings,
}

impl ManifestGenerator {
    pub async fn new(config: ManifestSettings) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn generate_manifest(&self) -> Result<String> {
        let manifest = serde_json::json!({
            "name": self.config.app_name,
            "short_name": self.config.short_name,
            "description": self.config.description,
            "theme_color": self.config.theme_color,
            "background_color": self.config.background_color,
            "display": format!("{:?}", self.config.display_mode).to_lowercase(),
            "orientation": format!("{:?}", self.config.orientation).to_lowercase(),
            "icons": self.config.icons,
            "categories": self.config.categories,
            "screenshots": self.config.screenshots,
            "start_url": "/",
            "scope": "/",
            "lang": "en-US"
        });

        Ok(serde_json::to_string_pretty(&manifest)?)
    }
}

#[derive(Debug)]
pub struct PWAPerformanceMonitor {
    config: PWAPerformanceSettings,
}

impl PWAPerformanceMonitor {
    pub async fn new(config: PWAPerformanceSettings) -> Result<Self> {
        Ok(Self { config })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub data: HashMap<String, serde_json::Value>,
    pub actions: Vec<NotificationAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub action: String,
    pub title: String,
    pub icon: Option<String>,
}

// Extension trait for metrics collector to add PWA-specific metrics
impl MetricsCollector {
    pub async fn record_pwa_session_created(&self, user_id: Uuid, browser: &str) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());
        tags.insert("browser".to_string(), browser.to_string());

        let _ = self
            .record_custom_metric("pwa_sessions_created_total", 1.0, tags)
            .await;
    }

    pub async fn record_webxr_session_started(&self, user_id: Uuid) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());

        let _ = self
            .record_custom_metric("webxr_sessions_started_total", 1.0, tags)
            .await;
    }

    pub async fn record_pwa_installed(&self, user_id: Uuid) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());

        let _ = self
            .record_custom_metric("pwa_installs_total", 1.0, tags)
            .await;
    }

    pub async fn record_notification_permission_granted(&self, user_id: Uuid) {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), user_id.to_string());

        let _ = self
            .record_custom_metric("notification_permissions_granted_total", 1.0, tags)
            .await;
    }

    pub async fn record_pwa_performance(&self, session_id: Uuid, metrics: &PWAPerformanceMetrics) {
        let mut tags = HashMap::new();
        tags.insert("session_id".to_string(), session_id.to_string());

        let _ = self
            .record_custom_metric(
                "pwa_page_load_time_ms",
                metrics.page_load_time_ms as f64,
                tags.clone(),
            )
            .await;
        let _ = self
            .record_custom_metric(
                "pwa_time_to_interactive_ms",
                metrics.time_to_interactive_ms as f64,
                tags.clone(),
            )
            .await;
        let _ = self
            .record_custom_metric(
                "pwa_cache_hit_rate",
                metrics.service_worker_cache_hit_rate as f64,
                tags,
            )
            .await;
    }
}

// Service Worker JavaScript template
const SERVICE_WORKER_TEMPLATE: &str = r#"
// OpenSim Next PWA Service Worker
// Generated automatically - provides offline support, caching, and background sync

const CACHE_NAME = 'opensim-next-v1';
const OFFLINE_URL = '/offline.html';

// Critical resources to cache immediately
const CRITICAL_RESOURCES = [
    '/',
    '/offline.html',
    '/css/main.css',
    '/js/main.js',
    '/icons/icon-192x192.png',
    '/icons/icon-512x512.png'
];

// Install event - cache critical resources
self.addEventListener('install', event => {
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then(cache => cache.addAll(CRITICAL_RESOURCES))
            .then(() => self.skipWaiting())
    );
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
    event.waitUntil(
        caches.keys().then(cacheNames => {
            return Promise.all(
                cacheNames.map(cacheName => {
                    if (cacheName !== CACHE_NAME) {
                        return caches.delete(cacheName);
                    }
                })
            );
        }).then(() => self.clients.claim())
    );
});

// Fetch event - serve from cache with network fallback
self.addEventListener('fetch', event => {
    if (event.request.method !== 'GET') return;

    event.respondWith(
        caches.match(event.request)
            .then(response => {
                // Return cached version or fetch from network
                return response || fetch(event.request);
            })
            .catch(() => {
                // If both cache and network fail, show offline page
                if (event.request.destination === 'document') {
                    return caches.match(OFFLINE_URL);
                }
            })
    );
});

// Background sync for offline actions
self.addEventListener('sync', event => {
    if (event.tag === 'background-sync') {
        event.waitUntil(doBackgroundSync());
    }
});

// Push notification handling
self.addEventListener('push', event => {
    if (event.data) {
        const data = event.data.json();
        const options = {
            body: data.body,
            icon: data.icon || '/icons/icon-192x192.png',
            badge: data.badge || '/icons/badge-72x72.png',
            vibrate: [100, 50, 100],
            data: data.data,
            actions: data.actions || []
        };

        event.waitUntil(
            self.registration.showNotification(data.title, options)
        );
    }
});

// Notification click handling
self.addEventListener('notificationclick', event => {
    event.notification.close();

    if (event.action) {
        // Handle notification action
        clients.openWindow(`/?action=${event.action}`);
    } else {
        // Handle notification click
        event.waitUntil(
            clients.openWindow('/')
        );
    }
});

async function doBackgroundSync() {
    // Implement background synchronization logic
    console.log('Performing background sync...');
}
"#;
