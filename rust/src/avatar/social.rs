//! Avatar Social Features for OpenSim Next
//! 
//! Provides social networking features for avatars including friends,
//! groups, achievements, and social interactions.

use super::*;
use crate::database::DatabaseManager;
use anyhow::Result;
use sqlx::Row;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Avatar social features management
#[derive(Debug)]
pub struct AvatarSocialFeatures {
    database: Arc<DatabaseManager>,
}

impl AvatarSocialFeatures {
    /// Create new social features manager
    pub fn new(database: Arc<DatabaseManager>) -> Self {
        Self { database }
    }

    /// Validate social profile
    pub fn validate_social_profile(&self, profile: &AvatarSocialProfile) -> AvatarResult<()> {
        debug!("Validating avatar social profile");

        if profile.display_name.is_empty() {
            return Err(AvatarError::InvalidData {
                reason: "Display name cannot be empty".to_string(),
            });
        }

        if profile.display_name.len() > 64 {
            return Err(AvatarError::InvalidData {
                reason: "Display name must be 64 characters or less".to_string(),
            });
        }

        if let Some(bio) = &profile.bio {
            if bio.len() > 1024 {
                return Err(AvatarError::InvalidData {
                    reason: "Bio must be 1024 characters or less".to_string(),
                });
            }
        }

        if profile.interests.len() > 20 {
            return Err(AvatarError::InvalidData {
                reason: "Cannot have more than 20 interests".to_string(),
            });
        }

        for interest in &profile.interests {
            if interest.len() > 32 {
                return Err(AvatarError::InvalidData {
                    reason: "Interest must be 32 characters or less".to_string(),
                });
            }
        }

        if profile.languages.len() > 10 {
            return Err(AvatarError::InvalidData {
                reason: "Cannot have more than 10 languages".to_string(),
            });
        }

        for language in &profile.languages {
            if language.len() != 2 && language.len() != 5 {
                return Err(AvatarError::InvalidData {
                    reason: "Language codes must be 2 or 5 characters (ISO format)".to_string(),
                });
            }
        }

        debug!("Avatar social profile validation successful");
        Ok(())
    }

    /// Add friend relationship
    pub async fn add_friend(
        &self,
        avatar_id: Uuid,
        friend_id: Uuid,
        friend_name: String,
    ) -> AvatarResult<()> {
        info!("Adding friend relationship: {} -> {}", avatar_id, friend_id);

        if avatar_id == friend_id {
            return Err(AvatarError::InvalidData {
                reason: "Cannot add yourself as a friend".to_string(),
            });
        }

        // Check if friendship already exists
        let existing = sqlx::query(
            "SELECT id FROM avatar_friends WHERE avatar_id = $1 AND friend_id = $2"
        )
        .bind(avatar_id.to_string())
        .bind(friend_id.to_string())
        .fetch_optional(self.database.legacy_pool()?)
        .await?;

        if existing.is_some() {
            return Err(AvatarError::InvalidData {
                reason: "Friendship already exists".to_string(),
            });
        }

        // Add friendship
        sqlx::query(
            r#"
            INSERT INTO avatar_friends (id, avatar_id, friend_id, friend_name, created_at)
            VALUES (gen_random_uuid(), $1, $2, $3, NOW())
            "#
        )
        .bind(avatar_id.to_string())
        .bind(friend_id.to_string())
        .bind(friend_name)
        .execute(self.database.legacy_pool()?)
        .await?;

        info!("Friend relationship added successfully");
        Ok(())
    }

    /// Remove friend relationship
    pub async fn remove_friend(&self, avatar_id: Uuid, friend_id: Uuid) -> AvatarResult<()> {
        info!("Removing friend relationship: {} -> {}", avatar_id, friend_id);

        let result = sqlx::query(
            "DELETE FROM avatar_friends WHERE avatar_id = $1 AND friend_id = $2"
        )
        .bind(avatar_id.to_string())
        .bind(friend_id.to_string())
        .execute(self.database.legacy_pool()?)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AvatarError::NotFound { id: friend_id });
        }

        info!("Friend relationship removed successfully");
        Ok(())
    }

    /// Get friend list
    pub async fn get_friends(&self, avatar_id: Uuid) -> AvatarResult<Vec<AvatarFriend>> {
        debug!("Getting friend list for avatar: {}", avatar_id);

        let rows = sqlx::query(
            r#"
            SELECT af.friend_id, af.friend_name, af.created_at,
                   ea.name as current_name
            FROM avatar_friends af
            LEFT JOIN enhanced_avatars ea ON af.friend_id = ea.id
            WHERE af.avatar_id = $1
            ORDER BY af.friend_name
            "#
        )
        .bind(avatar_id.to_string())
        .fetch_all(self.database.legacy_pool()?)
        .await?;

        let friends: Vec<AvatarFriend> = rows.into_iter().map(|row| {
            let friend_id_str: String = row.try_get("friend_id").unwrap();
            let created_at_str: String = row.try_get("created_at").unwrap();
            
            AvatarFriend {
                friend_id: Uuid::parse_str(&friend_id_str).unwrap_or_default(),
                friend_name: row.try_get::<Option<String>, _>("current_name").unwrap()
                    .unwrap_or_else(|| row.try_get("friend_name").unwrap()),
                friendship_date: chrono::DateTime::parse_from_rfc3339(&created_at_str).unwrap().with_timezone(&chrono::Utc),
                online_status: OnlineStatus::Unknown, // Would be populated from session manager
                last_seen: None, // Would be populated from session data
            }
        }).collect();

        debug!("Retrieved {} friends for avatar {}", friends.len(), avatar_id);
        Ok(friends)
    }

    /// Get friend count
    pub async fn get_friend_count(&self, avatar_id: Uuid) -> AvatarResult<i64> {
        let result = sqlx::query(
            "SELECT COUNT(*) as count FROM avatar_friends WHERE avatar_id = $1"
        )
        .bind(avatar_id.to_string())
        .fetch_one(self.database.legacy_pool()?)
        .await?;

        let count = result.try_get::<i64, _>("count").unwrap_or(0);
        Ok(count)
    }

    /// Check if avatars are friends
    pub async fn are_friends(&self, avatar_id: Uuid, other_id: Uuid) -> AvatarResult<bool> {
        let result = sqlx::query(
            "SELECT id FROM avatar_friends WHERE avatar_id = $1 AND friend_id = $2"
        )
        .bind(avatar_id.to_string())
        .bind(other_id.to_string())
        .fetch_optional(self.database.legacy_pool()?)
        .await?;

        Ok(result.is_some())
    }

    /// Add achievement to avatar
    pub async fn add_achievement(
        &self,
        avatar_id: Uuid,
        achievement: Achievement,
    ) -> AvatarResult<()> {
        info!("Adding achievement to avatar {}: {}", avatar_id, achievement.name);

        // Check if achievement already exists
        let existing = sqlx::query(
            "SELECT id FROM avatar_achievements WHERE avatar_id = $1 AND achievement_id = $2"
        )
        .bind(avatar_id.to_string())
        .bind(achievement.achievement_id.to_string())
        .fetch_optional(self.database.legacy_pool()?)
        .await?;

        if existing.is_some() {
            return Err(AvatarError::InvalidData {
                reason: "Achievement already earned".to_string(),
            });
        }

        let category_str = serde_json::to_string(&achievement.category)?;

        sqlx::query(
            r#"
            INSERT INTO avatar_achievements (
                id, avatar_id, achievement_id, name, description, 
                icon_url, earned_at, points, category
            ) VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(avatar_id.to_string())
        .bind(achievement.achievement_id.to_string())
        .bind(achievement.name)
        .bind(achievement.description)
        .bind(achievement.icon_url)
        .bind(achievement.earned_at.to_rfc3339())
        .bind(achievement.points)
        .bind(category_str)
        .execute(self.database.legacy_pool()?)
        .await?;

        info!("Achievement added successfully");
        Ok(())
    }

    /// Get avatar achievements
    pub async fn get_achievements(&self, avatar_id: Uuid) -> AvatarResult<Vec<Achievement>> {
        debug!("Getting achievements for avatar: {}", avatar_id);

        let rows = sqlx::query(
            "SELECT * FROM avatar_achievements WHERE avatar_id = $1 ORDER BY earned_at DESC"
        )
        .bind(avatar_id.to_string())
        .fetch_all(self.database.legacy_pool()?)
        .await?;

        let mut achievements = Vec::new();

        for row in rows {
            let category_str: String = row.try_get("category")?;
            let category: AchievementCategory = serde_json::from_str(&category_str)?;

            // ELEGANT ARCHIVE SOLUTION: Parse UUID and DateTime from strings
            let achievement_id_str: String = row.try_get("achievement_id")?;
            let earned_at_str: String = row.try_get("earned_at")?;
            
            achievements.push(Achievement {
                achievement_id: Uuid::parse_str(&achievement_id_str).unwrap_or_default(),
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                icon_url: row.try_get("icon_url")?,
                earned_at: chrono::DateTime::parse_from_rfc3339(&earned_at_str).unwrap().with_timezone(&chrono::Utc),
                points: row.try_get("points")?,
                category,
            });
        }

        debug!("Retrieved {} achievements for avatar {}", achievements.len(), avatar_id);
        Ok(achievements)
    }

    /// Get achievement points total
    pub async fn get_achievement_points(&self, avatar_id: Uuid) -> AvatarResult<i32> {
        let result = sqlx::query(
            "SELECT COALESCE(SUM(points), 0) as total FROM avatar_achievements WHERE avatar_id = $1"
        )
        .bind(avatar_id.to_string())
        .fetch_one(self.database.legacy_pool()?)
        .await?;

        let total = result.try_get::<i32, _>("total").unwrap_or(0);
        Ok(total)
    }

    /// Send message to avatar
    pub async fn send_message(
        &self,
        from_avatar_id: Uuid,
        to_avatar_id: Uuid,
        message: AvatarMessage,
    ) -> AvatarResult<()> {
        info!("Sending message from {} to {}", from_avatar_id, to_avatar_id);

        let message_json = serde_json::to_string(&message)?;

        sqlx::query(
            r#"
            INSERT INTO avatar_messages (
                id, from_avatar_id, to_avatar_id, message_type, content, 
                sent_at, read_at
            ) VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW(), NULL)
            "#
        )
        .bind(from_avatar_id.to_string())
        .bind(to_avatar_id.to_string())
        .bind(message.message_type.to_string())
        .bind(message_json)
        .execute(self.database.legacy_pool()?)
        .await?;

        info!("Message sent successfully");
        Ok(())
    }

    /// Get messages for avatar
    pub async fn get_messages(
        &self,
        avatar_id: Uuid,
        unread_only: bool,
        limit: Option<i64>,
    ) -> AvatarResult<Vec<AvatarMessage>> {
        debug!("Getting messages for avatar: {}", avatar_id);

        let mut query = "SELECT * FROM avatar_messages WHERE to_avatar_id = $1".to_string();
        
        if unread_only {
            query.push_str(" AND read_at IS NULL");
        }
        
        query.push_str(" ORDER BY sent_at DESC");
        
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let rows = sqlx::query(&query)
            .bind(avatar_id.to_string())
            .fetch_all(self.database.legacy_pool()?)
            .await?;

        let mut messages = Vec::new();

        for row in rows {
            let content: String = row.try_get("content")?;
            let message: AvatarMessage = serde_json::from_str(&content)?;
            messages.push(message);
        }

        debug!("Retrieved {} messages for avatar {}", messages.len(), avatar_id);
        Ok(messages)
    }

    /// Mark message as read
    pub async fn mark_message_read(&self, message_id: Uuid) -> AvatarResult<()> {
        sqlx::query(
            "UPDATE avatar_messages SET read_at = NOW() WHERE id = $1"
        )
        .bind(message_id.to_string())
        .execute(self.database.legacy_pool()?)
        .await?;

        Ok(())
    }

    /// Initialize social features database tables
    pub async fn initialize_tables(&self) -> Result<()> {
        info!("Initializing avatar social features database tables");

        // Create avatar_friends table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS avatar_friends (
                id UUID PRIMARY KEY,
                avatar_id UUID NOT NULL,
                friend_id UUID NOT NULL,
                friend_name VARCHAR(255) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(avatar_id, friend_id)
            )
            "#
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        // Create indexes for avatar_friends
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_avatar_friends_avatar_id ON avatar_friends (avatar_id)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_avatar_friends_friend_id ON avatar_friends (friend_id)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        // Create avatar_achievements table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS avatar_achievements (
                id UUID PRIMARY KEY,
                avatar_id UUID NOT NULL,
                achievement_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                icon_url VARCHAR(500),
                earned_at TIMESTAMPTZ NOT NULL,
                points INTEGER NOT NULL DEFAULT 0,
                category TEXT NOT NULL,
                UNIQUE(avatar_id, achievement_id)
            )
            "#
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        // Create index for avatar_achievements
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_avatar_achievements_avatar_id ON avatar_achievements (avatar_id)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        // Create avatar_messages table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS avatar_messages (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                from_avatar_id UUID NOT NULL,
                to_avatar_id UUID NOT NULL,
                message_type VARCHAR(50) NOT NULL,
                content JSONB NOT NULL,
                sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                read_at TIMESTAMPTZ
            )
            "#
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        // Create indexes for avatar_messages
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_avatar_messages_to_avatar ON avatar_messages (to_avatar_id)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_avatar_messages_from_avatar ON avatar_messages (from_avatar_id)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_avatar_messages_unread ON avatar_messages (to_avatar_id, read_at)"
        )
        .execute(self.database.legacy_pool()?)
        .await?;

        info!("Avatar social features database tables initialized successfully");
        Ok(())
    }
}

/// Avatar friend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarFriend {
    pub friend_id: Uuid,
    pub friend_name: String,
    pub friendship_date: chrono::DateTime<chrono::Utc>,
    pub online_status: OnlineStatus,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

/// Online status for friends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnlineStatus {
    Online,
    Away,
    Busy,
    Offline,
    Unknown,
}

/// Avatar message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarMessage {
    pub message_id: Uuid,
    pub from_avatar_id: Uuid,
    pub from_avatar_name: String,
    pub message_type: MessageType,
    pub subject: Option<String>,
    pub content: String,
    pub sent_at: chrono::DateTime<chrono::Utc>,
    pub read_at: Option<chrono::DateTime<chrono::Utc>>,
    pub attachments: Vec<MessageAttachment>,
}

/// Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Personal,
    Group,
    System,
    Notification,
    Invitation,
}

impl ToString for MessageType {
    fn to_string(&self) -> String {
        match self {
            MessageType::Personal => "personal".to_string(),
            MessageType::Group => "group".to_string(),
            MessageType::System => "system".to_string(),
            MessageType::Notification => "notification".to_string(),
            MessageType::Invitation => "invitation".to_string(),
        }
    }
}

/// Message attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub attachment_id: Uuid,
    pub attachment_type: AttachmentType,
    pub name: String,
    pub url: String,
    pub size_bytes: i64,
}

/// Attachment types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentType {
    Image,
    Texture,
    Object,
    Landmark,
    Notecard,
    Script,
}

impl Default for OnlineStatus {
    fn default() -> Self {
        Self::Unknown
    }
}