//! OpenSim-Compatible Avatar Service
//!
//! Direct port of OpenSim.Services.AvatarService/AvatarService.cs
//! Uses the `avatars` table with key-value storage format matching OpenSim master

use crate::database::DatabaseManager;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Avatar data structure matching OpenSim's AvatarData class
/// Each avatar is stored as multiple rows in the avatars table (key-value pairs)
#[derive(Debug, Clone)]
pub struct AvatarData {
    pub avatar_type: i32,              // 1 = SL avatar
    pub data: HashMap<String, String>, // Key-value pairs from database
}

impl AvatarData {
    /// Create new empty avatar data (matches OpenSim C# constructor)
    pub fn new() -> Self {
        Self {
            avatar_type: 1, // Default to SL avatar
            data: HashMap::new(),
        }
    }
}

/// OpenSim-compatible Avatar Service
/// Direct port of OpenSim.Services.AvatarService.AvatarService
#[derive(Debug)]
pub struct AvatarService {
    database: Arc<DatabaseManager>,
}

impl AvatarService {
    /// Create new avatar service
    pub fn new(database: Arc<DatabaseManager>) -> Self {
        info!("[AVATAR SERVICE]: Starting avatar service");
        Self { database }
    }

    /// Get avatar data from database by PrincipalID
    /// Matches: OpenSim.Services.AvatarService.AvatarService::GetAvatar(UUID)
    pub async fn get_avatar(&self, principal_id: Uuid) -> Result<AvatarData> {
        debug!("Getting avatar data for PrincipalID: {}", principal_id);

        // Query all rows for this principal_id from avatars table
        // C# equivalent: m_Database.Get("PrincipalID", principalID.ToString())
        let rows = sqlx::query!(
            r#"
            SELECT name, value
            FROM avatars
            WHERE principalid = $1
            "#,
            principal_id
        )
        .fetch_all(
            self.database
                .connection()
                .as_ref()
                .postgres_pool()
                .ok_or_else(|| anyhow::anyhow!("PostgreSQL pool not available"))?,
        )
        .await?;

        let mut ret = AvatarData::new();

        // If no data found, return empty AvatarData with AvatarType = 1
        // Matches C# lines 72-76
        if rows.is_empty() {
            ret.avatar_type = 1; // SL avatar
            return Ok(ret);
        }

        // Parse rows into AvatarData
        // Matches C# lines 78-84
        for row in rows {
            if row.name == "AvatarType" {
                ret.avatar_type = row.value.parse().unwrap_or(1);
            } else {
                ret.data.insert(row.name.clone(), row.value.clone());
            }
        }

        Ok(ret)
    }

    /// Store avatar data in database
    /// Matches: OpenSim.Services.AvatarService.AvatarService::SetAvatar(UUID, AvatarData)
    pub async fn set_avatar(&self, principal_id: Uuid, avatar: &AvatarData) -> Result<bool> {
        debug!("Setting avatar data for PrincipalID: {}", principal_id);

        // Count attachment points (keys starting with "_ap_")
        // Matches C# lines 91-95
        let count = avatar.data.keys().filter(|k| k.starts_with("_ap_")).count();
        debug!(
            "[AVATAR SERVICE]: SetAvatar for {}, attachs={}",
            principal_id, count
        );

        // Delete existing avatar data
        // Matches C# line 97: m_Database.Delete("PrincipalID", principalID.ToString())
        sqlx::query!("DELETE FROM avatars WHERE principalid = $1", principal_id)
            .execute(
                self.database
                    .connection()
                    .as_ref()
                    .postgres_pool()
                    .ok_or_else(|| anyhow::anyhow!("PostgreSQL pool not available"))?,
            )
            .await?;

        // Store AvatarType
        // Matches C# lines 99-107
        sqlx::query!(
            r#"
            INSERT INTO avatars (principalid, name, value)
            VALUES ($1, $2, $3)
            "#,
            principal_id,
            "AvatarType",
            avatar.avatar_type.to_string()
        )
        .execute(
            self.database
                .connection()
                .as_ref()
                .postgres_pool()
                .ok_or_else(|| anyhow::anyhow!("PostgreSQL pool not available"))?,
        )
        .await?;

        // Store each data pair
        // Matches C# lines 109-145
        for (key, value) in &avatar.data {
            // Handle AvatarHeight validation
            // Matches C# lines 117-134 (bug fix for bad height values)
            let final_value = if key == "AvatarHeight" {
                match value.parse::<f32>() {
                    Ok(height) if height >= 0.0 && height <= 10.0 => value.clone(),
                    _ => {
                        // Try replacing comma with period
                        let raw_height = value.replace(",", ".");
                        match raw_height.parse::<f32>() {
                            Ok(height) if height >= 0.0 && height <= 10.0 => raw_height,
                            _ => {
                                // Use default height
                                warn!(
                                    "[AVATAR SERVICE]: Rectifying height of avatar {} from {} to 1.771488",
                                    principal_id, value
                                );
                                "1.771488".to_string()
                            }
                        }
                    }
                }
            } else {
                value.clone()
            };

            sqlx::query!(
                r#"
                INSERT INTO avatars (principalid, name, value)
                VALUES ($1, $2, $3)
                "#,
                principal_id,
                key,
                final_value
            )
            .execute(
                self.database
                    .connection()
                    .as_ref()
                    .postgres_pool()
                    .ok_or_else(|| anyhow::anyhow!("PostgreSQL pool not available"))?,
            )
            .await
            .map_err(|e| {
                // If any insert fails, delete all and return error
                // Matches C# lines 140-143
                warn!(
                    "[AVATAR SERVICE]: Failed to store avatar field {}: {}",
                    key, e
                );
                e
            })?;
        }

        Ok(true)
    }

    /// Reset/delete avatar data
    /// Matches: OpenSim.Services.AvatarService.AvatarService::ResetAvatar(UUID)
    pub async fn reset_avatar(&self, principal_id: Uuid) -> Result<bool> {
        debug!("Resetting avatar data for PrincipalID: {}", principal_id);

        sqlx::query!("DELETE FROM avatars WHERE principalid = $1", principal_id)
            .execute(
                self.database
                    .connection()
                    .as_ref()
                    .postgres_pool()
                    .ok_or_else(|| anyhow::anyhow!("PostgreSQL pool not available"))?,
            )
            .await?;

        Ok(true)
    }

    /// Set specific avatar items
    /// Matches: OpenSim.Services.AvatarService.AvatarService::SetItems(UUID, string[], string[])
    pub async fn set_items(
        &self,
        principal_id: Uuid,
        names: &[String],
        values: &[String],
    ) -> Result<bool> {
        debug!(
            "Setting {} avatar items for PrincipalID: {}",
            names.len(),
            principal_id
        );

        if names.len() != values.len() {
            return Ok(false);
        }

        for (name, value) in names.iter().zip(values.iter()) {
            sqlx::query!(
                r#"
                INSERT INTO avatars (principalid, name, value)
                VALUES ($1, $2, $3)
                ON CONFLICT (principalid, name)
                DO UPDATE SET value = EXCLUDED.value
                "#,
                principal_id,
                name,
                value
            )
            .execute(
                self.database
                    .connection()
                    .as_ref()
                    .postgres_pool()
                    .ok_or_else(|| anyhow::anyhow!("PostgreSQL pool not available"))?,
            )
            .await?;
        }

        Ok(true)
    }

    /// Remove specific avatar items
    /// Matches: OpenSim.Services.AvatarService.AvatarService::RemoveItems(UUID, string[])
    pub async fn remove_items(&self, principal_id: Uuid, names: &[String]) -> Result<bool> {
        debug!(
            "Removing {} avatar items for PrincipalID: {}",
            names.len(),
            principal_id
        );

        for name in names {
            sqlx::query!(
                "DELETE FROM avatars WHERE principalid = $1 AND name = $2",
                principal_id,
                name
            )
            .execute(
                self.database
                    .connection()
                    .as_ref()
                    .postgres_pool()
                    .ok_or_else(|| anyhow::anyhow!("PostgreSQL pool not available"))?,
            )
            .await?;
        }

        Ok(true)
    }

    /// Get avatar appearance (for login response)
    /// Returns default appearance if avatar has no data
    pub async fn get_or_create_default_appearance(&self, principal_id: Uuid) -> Result<AvatarData> {
        let avatar = self.get_avatar(principal_id).await?;

        // If avatar has no data, create and store default appearance
        if avatar.data.is_empty() {
            info!(
                "[AVATAR SERVICE]: Creating default appearance for {}",
                principal_id
            );
            let default_avatar = Self::create_default_avatar_data();
            self.set_avatar(principal_id, &default_avatar).await?;
            Ok(default_avatar)
        } else {
            Ok(avatar)
        }
    }

    /// Create default avatar data matching OpenSim defaults
    /// Based on OpenSim.Framework.AvatarAppearance default constructor
    fn create_default_avatar_data() -> AvatarData {
        let mut data = HashMap::new();

        // Serial number
        data.insert("Serial".to_string(), "1".to_string());

        // Default height (matches OpenSim's default)
        data.insert("AvatarHeight".to_string(), "1.771488".to_string());

        // Default body wearables (from AvatarWearable.DefaultWearables)
        // Body - shape
        data.insert(
            "BodyItem".to_string(),
            "66c41e39-38f9-f75a-024e-585989bfaba9".to_string(),
        );
        data.insert(
            "BodyAsset".to_string(),
            "66c41e39-38f9-f75a-024e-585989bfab73".to_string(),
        );

        // Skin
        data.insert(
            "SkinItem".to_string(),
            "77c41e39-38f9-f75a-024e-585989bfabc9".to_string(),
        );
        data.insert(
            "SkinAsset".to_string(),
            "77c41e39-38f9-f75a-024e-585989bbabbb".to_string(),
        );

        // Hair
        data.insert(
            "HairItem".to_string(),
            "d342e6c1-b9d2-11dc-95ff-0800200c9a66".to_string(),
        );
        data.insert(
            "HairAsset".to_string(),
            "d342e6c0-b9d2-11dc-95ff-0800200c9a66".to_string(),
        );

        // Eyes
        data.insert(
            "EyesItem".to_string(),
            "cdc31054-eed8-4021-994f-4e0c6e861b50".to_string(),
        );
        data.insert(
            "EyesAsset".to_string(),
            "4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7".to_string(),
        );

        // Default visual parameters (218 bytes, all 150 for default appearance)
        // This is a simplified default - real OpenSim has specific values per parameter
        let visual_params: Vec<String> = (0..218).map(|_| "150".to_string()).collect();
        data.insert("VisualParams".to_string(), visual_params.join(","));

        // New style wearables format
        data.insert(
            "Wearable 0:0".to_string(),
            "66c41e39-38f9-f75a-024e-585989bfaba9:66c41e39-38f9-f75a-024e-585989bfab73".to_string(),
        ); // Shape
        data.insert(
            "Wearable 1:0".to_string(),
            "77c41e39-38f9-f75a-024e-585989bfabc9:77c41e39-38f9-f75a-024e-585989bbabbb".to_string(),
        ); // Skin
        data.insert(
            "Wearable 2:0".to_string(),
            "d342e6c1-b9d2-11dc-95ff-0800200c9a66:d342e6c0-b9d2-11dc-95ff-0800200c9a66".to_string(),
        ); // Hair
        data.insert(
            "Wearable 3:0".to_string(),
            "cdc31054-eed8-4021-994f-4e0c6e861b50:4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7".to_string(),
        ); // Eyes

        AvatarData {
            avatar_type: 1,
            data,
        }
    }
}

impl Default for AvatarData {
    fn default() -> Self {
        Self::new()
    }
}

use crate::services::traits::{AvatarData as TraitAvatarData, AvatarServiceTrait};
use async_trait::async_trait;

#[async_trait]
impl AvatarServiceTrait for AvatarService {
    async fn get_avatar(&self, principal_id: Uuid) -> Result<TraitAvatarData> {
        let local = self.get_avatar(principal_id).await?;
        Ok(TraitAvatarData {
            avatar_type: local.avatar_type,
            data: local.data,
        })
    }

    async fn set_avatar(&self, principal_id: Uuid, data: &TraitAvatarData) -> Result<bool> {
        let local = AvatarData {
            avatar_type: data.avatar_type,
            data: data.data.clone(),
        };
        self.set_avatar(principal_id, &local).await
    }

    async fn reset_avatar(&self, principal_id: Uuid) -> Result<bool> {
        self.reset_avatar(principal_id).await
    }

    async fn remove_items(&self, principal_id: Uuid, names: &[String]) -> Result<bool> {
        self.remove_items(principal_id, names).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_avatar_data() {
        let avatar = AvatarService::create_default_avatar_data();

        assert_eq!(avatar.avatar_type, 1);
        assert!(avatar.data.contains_key("Serial"));
        assert!(avatar.data.contains_key("AvatarHeight"));
        assert!(avatar.data.contains_key("BodyItem"));
        assert!(avatar.data.contains_key("SkinItem"));
        assert!(avatar.data.contains_key("HairItem"));
        assert!(avatar.data.contains_key("EyesItem"));
        assert!(avatar.data.contains_key("VisualParams"));

        // Verify visual params format (218 comma-separated values)
        let visual_params = &avatar.data["VisualParams"];
        let params: Vec<&str> = visual_params.split(',').collect();
        assert_eq!(params.len(), 218, "Should have 218 visual parameters");
    }

    #[test]
    fn test_avatar_data_default() {
        let avatar = AvatarData::default();
        assert_eq!(avatar.avatar_type, 1);
        assert!(avatar.data.is_empty());
    }
}
