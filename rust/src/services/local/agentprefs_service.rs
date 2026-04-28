use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::services::traits::{AgentPrefs, AgentPrefsServiceTrait};

pub struct LocalAgentPrefsService {
    db: Arc<DatabaseConnection>,
}

impl LocalAgentPrefsService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AgentPrefsServiceTrait for LocalAgentPrefsService {
    async fn get_agent_preferences(&self, principal_id: Uuid) -> Result<Option<AgentPrefs>> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let row = sqlx::query(
            "SELECT principalid, accessprefs, hoverheight, language, \
             languageispublic, permeveryone, permgroup, permnextowner \
             FROM agentprefs WHERE principalid = $1",
        )
        .bind(principal_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                use sqlx::Row;
                let pid: Uuid = r.try_get("principalid").unwrap_or(principal_id);
                let access: String = r
                    .try_get::<String, _>("accessprefs")
                    .unwrap_or_else(|_| "M".to_string());
                let hover: f32 = r.try_get("hoverheight").unwrap_or(0.0);
                let lang: String = r
                    .try_get::<String, _>("language")
                    .unwrap_or_else(|_| "en-us".to_string());
                let lang_pub: i32 = r.try_get("languageispublic").unwrap_or(1);
                let pe: i32 = r.try_get("permeveryone").unwrap_or(0);
                let pg: i32 = r.try_get("permgroup").unwrap_or(0);
                let pno: i32 = r.try_get("permnextowner").unwrap_or(532480);

                Ok(Some(AgentPrefs {
                    principal_id: pid,
                    access_prefs: access.trim().to_string(),
                    hover_height: hover as f64,
                    language: lang.trim().to_string(),
                    language_is_public: lang_pub != 0,
                    perm_everyone: pe,
                    perm_group: pg,
                    perm_next_owner: pno,
                }))
            }
            None => Ok(None),
        }
    }

    async fn store_agent_preferences(&self, prefs: &AgentPrefs) -> Result<bool> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO agentprefs (principalid, accessprefs, hoverheight, language, \
             languageispublic, permeveryone, permgroup, permnextowner) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (principalid) DO UPDATE SET \
             accessprefs = $2, hoverheight = $3, language = $4, \
             languageispublic = $5, permeveryone = $6, permgroup = $7, permnextowner = $8",
        )
        .bind(prefs.principal_id)
        .bind(&prefs.access_prefs)
        .bind(prefs.hover_height as f32)
        .bind(&prefs.language)
        .bind(if prefs.language_is_public { 1i32 } else { 0i32 })
        .bind(prefs.perm_everyone)
        .bind(prefs.perm_group)
        .bind(prefs.perm_next_owner)
        .execute(pool)
        .await?;

        debug!("[AGENTPREFS] Stored preferences for {}", prefs.principal_id);
        Ok(true)
    }

    async fn get_agent_lang(&self, principal_id: Uuid) -> Result<String> {
        match self.get_agent_preferences(principal_id).await? {
            Some(prefs) if prefs.language_is_public => Ok(prefs.language),
            _ => Ok("en-us".to_string()),
        }
    }
}
