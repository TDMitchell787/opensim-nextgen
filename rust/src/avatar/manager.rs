//! Advanced Avatar Manager for OpenSim Next
//! 
//! Provides comprehensive avatar management with advanced appearance,
//! behavior, persistence, and social features.

use super::*;
use crate::avatar::persistence::AvatarSearchCriteria;
use crate::database::DatabaseManager;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Advanced avatar management system
#[derive(Debug)]
pub struct AdvancedAvatarManager {
    pub appearance_engine: Arc<AppearanceEngine>,
    pub behavior_system: Arc<BehaviorSystem>,
    pub persistence_layer: Arc<AvatarPersistence>,
    pub social_features: Arc<AvatarSocialFeatures>,
    database: Arc<DatabaseManager>,
    active_avatars: Arc<RwLock<HashMap<Uuid, EnhancedAvatar>>>,
    avatar_cache: Arc<RwLock<HashMap<Uuid, EnhancedAvatar>>>,
}

impl AdvancedAvatarManager {
    /// Create new advanced avatar manager
    pub fn new(database: Arc<DatabaseManager>) -> Self {
        let appearance_engine = Arc::new(AppearanceEngine::new());
        let behavior_system = Arc::new(BehaviorSystem::new());
        let persistence_layer = Arc::new(AvatarPersistence::new(database.clone()));
        let social_features = Arc::new(AvatarSocialFeatures::new(database.clone()));

        Self {
            appearance_engine,
            behavior_system,
            persistence_layer,
            social_features,
            database,
            active_avatars: Arc::new(RwLock::new(HashMap::new())),
            avatar_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new enhanced avatar
    pub async fn create_avatar(
        &self,
        user_id: Uuid,
        name: String,
        initial_appearance: Option<AvatarAppearance>,
    ) -> AvatarResult<EnhancedAvatar> {
        info!("Creating new enhanced avatar for user: {}", user_id);

        let avatar_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let appearance = initial_appearance.unwrap_or_else(|| {
            self.appearance_engine.create_default_appearance()
        });

        let avatar = EnhancedAvatar {
            id: avatar_id,
            user_id,
            name: name.clone(),
            appearance,
            behavior: AvatarBehavior {
                animations: Vec::new(),
                gestures: Vec::new(),
                auto_behaviors: Vec::new(),
                expressions: Vec::new(),
                voice_settings: VoiceSettings::default(),
            },
            social_profile: AvatarSocialProfile {
                display_name: name,
                bio: None,
                interests: Vec::new(),
                languages: vec!["en".to_string()],
                relationship_status: RelationshipStatus::NotSpecified,
                privacy_settings: PrivacySettings::default(),
                social_links: HashMap::new(),
                achievements: Vec::new(),
            },
            persistence_data: AvatarPersistenceData {
                last_position: Vector3::default(),
                last_rotation: Quaternion::default(),
                last_region: Uuid::nil(),
                session_time: 0,
                total_time: 0,
                visit_count: 0,
                last_login: now,
                inventory_snapshot: None,
                preferences: AvatarPreferences::default(),
            },
            created_at: now,
            updated_at: now,
        };

        // Store in database
        self.persistence_layer.store_avatar(&avatar).await?;

        // Add to cache
        {
            let mut cache = self.avatar_cache.write().await;
            cache.insert(avatar_id, avatar.clone());
        }

        info!("Enhanced avatar created successfully: {}", avatar_id);
        Ok(avatar)
    }

    /// Get avatar by ID
    pub async fn get_avatar(&self, avatar_id: Uuid) -> AvatarResult<EnhancedAvatar> {
        debug!("Retrieving avatar: {}", avatar_id);

        // Check active avatars first
        {
            let active = self.active_avatars.read().await;
            if let Some(avatar) = active.get(&avatar_id) {
                return Ok(avatar.clone());
            }
        }

        // Check cache
        {
            let cache = self.avatar_cache.read().await;
            if let Some(avatar) = cache.get(&avatar_id) {
                return Ok(avatar.clone());
            }
        }

        // Load from database
        let avatar = self.persistence_layer.load_avatar(avatar_id).await?;

        // Add to cache
        {
            let mut cache = self.avatar_cache.write().await;
            cache.insert(avatar_id, avatar.clone());
        }

        Ok(avatar)
    }

    /// Get avatar by user ID
    pub async fn get_avatar_by_user(&self, user_id: Uuid) -> AvatarResult<EnhancedAvatar> {
        debug!("Retrieving avatar for user: {}", user_id);
        self.persistence_layer.load_avatar_by_user(user_id).await
    }

    /// Update avatar appearance
    pub async fn update_appearance(
        &self,
        avatar_id: Uuid,
        appearance: AvatarAppearance,
    ) -> AvatarResult<()> {
        info!("Updating avatar appearance: {}", avatar_id);

        let mut avatar = self.get_avatar(avatar_id).await?;
        
        // Validate appearance
        self.appearance_engine.validate_appearance(&appearance)?;
        
        avatar.appearance = appearance;
        avatar.updated_at = chrono::Utc::now();

        // Update in database
        self.persistence_layer.update_avatar(&avatar).await?;

        // Update caches
        self.update_avatar_in_caches(avatar).await;

        info!("Avatar appearance updated successfully: {}", avatar_id);
        Ok(())
    }

    /// Update avatar behavior
    pub async fn update_behavior(
        &self,
        avatar_id: Uuid,
        behavior: AvatarBehavior,
    ) -> AvatarResult<()> {
        info!("Updating avatar behavior: {}", avatar_id);

        let mut avatar = self.get_avatar(avatar_id).await?;
        
        // Validate behavior
        self.behavior_system.validate_behavior(&behavior)?;
        
        avatar.behavior = behavior;
        avatar.updated_at = chrono::Utc::now();

        // Update in database
        self.persistence_layer.update_avatar(&avatar).await?;

        // Update caches
        self.update_avatar_in_caches(avatar).await;

        info!("Avatar behavior updated successfully: {}", avatar_id);
        Ok(())
    }

    /// Update avatar social profile
    pub async fn update_social_profile(
        &self,
        avatar_id: Uuid,
        social_profile: AvatarSocialProfile,
    ) -> AvatarResult<()> {
        info!("Updating avatar social profile: {}", avatar_id);

        let mut avatar = self.get_avatar(avatar_id).await?;
        
        // Validate social profile
        self.social_features.validate_social_profile(&social_profile)?;
        
        avatar.social_profile = social_profile;
        avatar.updated_at = chrono::Utc::now();

        // Update in database
        self.persistence_layer.update_avatar(&avatar).await?;

        // Update caches
        self.update_avatar_in_caches(avatar).await;

        info!("Avatar social profile updated successfully: {}", avatar_id);
        Ok(())
    }

    /// Login avatar (mark as active)
    pub async fn login_avatar(&self, avatar_id: Uuid) -> AvatarResult<()> {
        info!("Avatar login: {}", avatar_id);

        let mut avatar = self.get_avatar(avatar_id).await?;
        
        // Update login time and visit count
        avatar.persistence_data.last_login = chrono::Utc::now();
        avatar.persistence_data.visit_count += 1;
        avatar.updated_at = chrono::Utc::now();

        // Update in database
        self.persistence_layer.update_avatar(&avatar).await?;

        // Add to active avatars
        {
            let mut active = self.active_avatars.write().await;
            active.insert(avatar_id, avatar.clone());
        }

        // Initialize behavior system for this avatar
        self.behavior_system.start_avatar_behaviors(avatar_id, &avatar.behavior).await?;

        info!("Avatar login completed: {}", avatar_id);
        Ok(())
    }

    /// Logout avatar (remove from active)
    pub async fn logout_avatar(&self, avatar_id: Uuid) -> AvatarResult<()> {
        info!("Avatar logout: {}", avatar_id);

        if let Some(mut avatar) = {
            let mut active = self.active_avatars.write().await;
            active.remove(&avatar_id)
        } {
            // Update session time
            let session_duration = chrono::Utc::now().signed_duration_since(avatar.persistence_data.last_login);
            avatar.persistence_data.session_time = session_duration.num_seconds();
            avatar.persistence_data.total_time += avatar.persistence_data.session_time;
            avatar.updated_at = chrono::Utc::now();

            // Update in database
            self.persistence_layer.update_avatar(&avatar).await?;

            // Stop behavior system for this avatar
            self.behavior_system.stop_avatar_behaviors(avatar_id).await?;
        }

        info!("Avatar logout completed: {}", avatar_id);
        Ok(())
    }

    /// Update avatar position and region
    pub async fn update_position(
        &self,
        avatar_id: Uuid,
        position: Vector3,
        rotation: Quaternion,
        region_id: Uuid,
    ) -> AvatarResult<()> {
        debug!("Updating avatar position: {}", avatar_id);

        // Update in active avatars
        {
            let mut active = self.active_avatars.write().await;
            if let Some(avatar) = active.get_mut(&avatar_id) {
                avatar.persistence_data.last_position = position;
                avatar.persistence_data.last_rotation = rotation;
                avatar.persistence_data.last_region = region_id;
                avatar.updated_at = chrono::Utc::now();
            }
        }

        // Periodically persist to database (not every update to avoid DB overload)
        // This could be improved with a background task for periodic persistence

        Ok(())
    }

    /// Get all active avatars
    pub async fn get_active_avatars(&self) -> Vec<EnhancedAvatar> {
        let active = self.active_avatars.read().await;
        active.values().cloned().collect()
    }

    /// Get avatars in a specific region
    pub async fn get_avatars_in_region(&self, region_id: Uuid) -> Vec<EnhancedAvatar> {
        let active = self.active_avatars.read().await;
        active
            .values()
            .filter(|avatar| avatar.persistence_data.last_region == region_id)
            .cloned()
            .collect()
    }

    /// Search avatars by criteria
    pub async fn search_avatars(
        &self,
        criteria: AvatarSearchCriteria,
    ) -> AvatarResult<Vec<EnhancedAvatar>> {
        info!("Searching avatars with criteria: {:?}", criteria);
        self.persistence_layer.search_avatars(criteria).await
    }

    /// Delete avatar
    pub async fn delete_avatar(&self, avatar_id: Uuid) -> AvatarResult<()> {
        info!("Deleting avatar: {}", avatar_id);

        // Remove from active avatars
        {
            let mut active = self.active_avatars.write().await;
            if active.remove(&avatar_id).is_some() {
                // Stop behaviors if avatar was active
                self.behavior_system.stop_avatar_behaviors(avatar_id).await?;
            }
        }

        // Remove from cache
        {
            let mut cache = self.avatar_cache.write().await;
            cache.remove(&avatar_id);
        }

        // Delete from database
        self.persistence_layer.delete_avatar(avatar_id).await?;

        info!("Avatar deleted successfully: {}", avatar_id);
        Ok(())
    }

    /// Get avatar statistics
    pub async fn get_avatar_statistics(&self, avatar_id: Uuid) -> AvatarResult<AvatarStatistics> {
        let avatar = self.get_avatar(avatar_id).await?;
        
        let stats = AvatarStatistics {
            total_time_online: avatar.persistence_data.total_time,
            visit_count: avatar.persistence_data.visit_count,
            last_login: avatar.persistence_data.last_login,
            created_at: avatar.created_at,
            achievements_count: avatar.social_profile.achievements.len() as i64,
            friends_count: self.social_features.get_friend_count(avatar_id).await?,
            regions_visited: self.persistence_layer.get_regions_visited_count(avatar_id).await?,
        };

        Ok(stats)
    }

    /// Update avatar in both active and cache
    async fn update_avatar_in_caches(&self, avatar: EnhancedAvatar) {
        let avatar_id = avatar.id;

        // Update active avatars
        {
            let mut active = self.active_avatars.write().await;
            if active.contains_key(&avatar_id) {
                active.insert(avatar_id, avatar.clone());
            }
        }

        // Update cache
        {
            let mut cache = self.avatar_cache.write().await;
            cache.insert(avatar_id, avatar);
        }
    }

    /// Cleanup inactive avatars from cache
    pub async fn cleanup_cache(&self) -> Result<()> {
        info!("Cleaning up avatar cache");
        
        let mut cache = self.avatar_cache.write().await;
        let initial_size = cache.len();
        
        // Keep only recently accessed avatars (within last hour)
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
        cache.retain(|_, avatar| avatar.updated_at > cutoff);
        
        let final_size = cache.len();
        info!("Cache cleanup completed: {} -> {} avatars", initial_size, final_size);
        
        Ok(())
    }

    /// Get system health information
    pub async fn get_system_health(&self) -> AvatarSystemHealth {
        let active_count = self.active_avatars.read().await.len();
        let cached_count = self.avatar_cache.read().await.len();

        AvatarSystemHealth {
            active_avatars: active_count as i64,
            cached_avatars: cached_count as i64,
            total_avatars: self.persistence_layer.get_total_avatar_count().await.unwrap_or(0),
            system_status: if active_count < 10000 { "healthy" } else { "high_load" }.to_string(),
        }
    }
}

// AvatarSearchCriteria moved to persistence_stub.rs to resolve conflicts

/// Avatar statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarStatistics {
    pub total_time_online: i64,
    pub visit_count: i64,
    pub last_login: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub achievements_count: i64,
    pub friends_count: i64,
    pub regions_visited: i64,
}

/// Avatar system health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarSystemHealth {
    pub active_avatars: i64,
    pub cached_avatars: i64,
    pub total_avatars: i64,
    pub system_status: String,
}