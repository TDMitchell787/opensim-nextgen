use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedObject {
    pub name: String,
    pub local_id: u32,
    pub prim_uuid: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub npc_id: Uuid,
    pub project_name: String,
    pub created_objects: Vec<CreatedObject>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BuildSession {
    pub fn new(user_id: Uuid, npc_id: Uuid) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            user_id,
            npc_id,
            project_name: String::new(),
            created_objects: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn record_object(&mut self, name: String, local_id: u32, prim_uuid: Uuid) {
        self.created_objects.push(CreatedObject {
            name,
            local_id,
            prim_uuid,
        });
        self.updated_at = Utc::now();
    }

    pub fn remove_object(&mut self, local_id: u32) {
        self.created_objects.retain(|o| o.local_id != local_id);
        self.updated_at = Utc::now();
    }

    pub fn find_by_local_id(&self, local_id: u32) -> Option<&CreatedObject> {
        self.created_objects.iter().find(|o| o.local_id == local_id)
    }

    pub fn context_prompt(&self) -> String {
        if self.created_objects.is_empty() {
            return String::new();
        }

        let mut ctx = String::from("\n\nCURRENT BUILD SESSION:\n");
        if !self.project_name.is_empty() {
            ctx.push_str(&format!("Project: \"{}\"\n", self.project_name));
        }
        ctx.push_str("Objects you've created (use these local_id values to modify them):\n");

        let mut name_counts: HashMap<String, u32> = HashMap::new();
        for obj in &self.created_objects {
            let count = name_counts.entry(obj.name.clone()).or_insert(0);
            *count += 1;
            let display_name = if *count > 1 {
                format!("{} #{}", obj.name, count)
            } else {
                obj.name.clone()
            };
            ctx.push_str(&format!("- {} (local_id={})\n", display_name, obj.local_id));
        }

        ctx.push_str("\nThe user may ask you to modify these objects. Use set_position, set_scale, set_color, set_rotation, set_name, delete_object, or link_objects with the local_id values above.\n");
        ctx.push_str("When the user says \"make it bigger\" or \"change the color\", apply the change to the most recently created object or the one they name.\n");
        ctx
    }

    pub fn set_project_name(&mut self, name: &str) {
        if self.project_name.is_empty() && !name.is_empty() {
            self.project_name = name.to_string();
            self.updated_at = Utc::now();
        }
    }
}

type SessionKey = (Uuid, Uuid);

pub struct BuildSessionStore {
    sessions: RwLock<HashMap<SessionKey, BuildSession>>,
    db_pool: Option<sqlx::PgPool>,
}

impl BuildSessionStore {
    pub fn new(db_pool: Option<sqlx::PgPool>) -> Arc<Self> {
        Arc::new(Self {
            sessions: RwLock::new(HashMap::new()),
            db_pool,
        })
    }

    pub async fn get_or_create(&self, user_id: Uuid, npc_id: Uuid) -> BuildSession {
        let key = (user_id, npc_id);
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(&key) {
                return session.clone();
            }
        }

        if let Some(pool) = &self.db_pool {
            if let Ok(session) = self.load_from_db(pool, user_id, npc_id).await {
                let mut sessions = self.sessions.write().await;
                sessions.insert(key, session.clone());
                return session;
            }
        }

        let session = BuildSession::new(user_id, npc_id);
        let mut sessions = self.sessions.write().await;
        sessions.insert(key, session.clone());
        session
    }

    pub async fn update_session(&self, session: &BuildSession) {
        let key = (session.user_id, session.npc_id);
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(key, session.clone());
        }

        if let Some(pool) = &self.db_pool {
            let pool = pool.clone();
            let session = session.clone();
            tokio::spawn(async move {
                if let Err(e) = save_to_db(&pool, &session).await {
                    info!("[BUILD_SESSION] DB save error: {}", e);
                }
            });
        }
    }

    pub async fn record_created_objects(
        &self,
        user_id: Uuid,
        npc_id: Uuid,
        objects: &[(String, u32, Uuid)],
    ) {
        if objects.is_empty() {
            return;
        }
        let key = (user_id, npc_id);
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .entry(key)
            .or_insert_with(|| BuildSession::new(user_id, npc_id));
        for (name, local_id, prim_uuid) in objects {
            session.record_object(name.clone(), *local_id, *prim_uuid);
        }
        session.updated_at = Utc::now();

        if let Some(pool) = &self.db_pool {
            let pool = pool.clone();
            let session = session.clone();
            tokio::spawn(async move {
                if let Err(e) = save_to_db(&pool, &session).await {
                    info!("[BUILD_SESSION] DB save error: {}", e);
                }
            });
        }
    }

    pub async fn record_deleted_object(&self, user_id: Uuid, npc_id: Uuid, local_id: u32) {
        let key = (user_id, npc_id);
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&key) {
            session.remove_object(local_id);
        }
    }

    pub async fn get_context_prompt(&self, user_id: Uuid, npc_id: Uuid) -> String {
        let key = (user_id, npc_id);
        let sessions = self.sessions.read().await;
        sessions
            .get(&key)
            .map(|s| s.context_prompt())
            .unwrap_or_default()
    }

    pub async fn set_project_name(&self, user_id: Uuid, npc_id: Uuid, name: &str) {
        let key = (user_id, npc_id);
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&key) {
            session.set_project_name(name);
        }
    }

    pub async fn get_stale_sessions(&self, max_age: std::time::Duration) -> Vec<(Uuid, String)> {
        let sessions = self.sessions.read().await;
        let cutoff = Utc::now()
            - chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::seconds(600));
        sessions
            .values()
            .filter(|s| !s.created_objects.is_empty() && s.updated_at < cutoff)
            .map(|s| (s.user_id, s.project_name.clone()))
            .collect()
    }

    pub async fn cleanup_stale_sessions(&self, online_agents: &[Uuid]) -> usize {
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();
        sessions.retain(|_, s| online_agents.contains(&s.user_id) || s.created_objects.is_empty());
        let removed = before - sessions.len();
        removed
    }

    pub async fn load_all_from_db(&self) {
        let Some(pool) = &self.db_pool else { return };

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS npc_build_sessions (\
                session_id UUID PRIMARY KEY, \
                user_id UUID NOT NULL, \
                npc_id UUID NOT NULL, \
                project_name TEXT NOT NULL DEFAULT '', \
                created_objects TEXT NOT NULL DEFAULT '[]', \
                created_at INTEGER NOT NULL, \
                updated_at INTEGER NOT NULL)",
        )
        .execute(pool)
        .await;

        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_build_sessions_user ON npc_build_sessions(user_id)",
        )
        .execute(pool)
        .await;

        let rows = match sqlx::query_as::<_, (Uuid, Uuid, Uuid, String, String, i32, i32)>(
            "SELECT session_id, user_id, npc_id, project_name, created_objects, created_at, updated_at FROM npc_build_sessions"
        ).fetch_all(pool).await {
            Ok(rows) => rows,
            Err(e) => {
                info!("[BUILD_SESSION] Failed to load sessions: {}", e);
                return;
            }
        };

        let mut sessions = self.sessions.write().await;
        let mut loaded = 0;
        for (session_id, user_id, npc_id, project_name, objects_json, created_at, updated_at) in
            rows
        {
            let created_objects: Vec<CreatedObject> =
                serde_json::from_str(&objects_json).unwrap_or_default();
            let session = BuildSession {
                session_id,
                user_id,
                npc_id,
                project_name,
                created_objects,
                created_at: DateTime::from_timestamp(created_at as i64, 0).unwrap_or_else(Utc::now),
                updated_at: DateTime::from_timestamp(updated_at as i64, 0).unwrap_or_else(Utc::now),
            };
            sessions.insert((user_id, npc_id), session);
            loaded += 1;
        }
        if loaded > 0 {
            info!("[BUILD_SESSION] Loaded {} sessions from DB", loaded);
        }
    }

    async fn load_from_db(
        &self,
        pool: &sqlx::PgPool,
        user_id: Uuid,
        npc_id: Uuid,
    ) -> Result<BuildSession, sqlx::Error> {
        let row: (Uuid, String, String, i32, i32) = sqlx::query_as(
            "SELECT session_id, project_name, created_objects, created_at, updated_at \
             FROM npc_build_sessions WHERE user_id = $1 AND npc_id = $2 \
             ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(user_id)
        .bind(npc_id)
        .fetch_one(pool)
        .await?;

        let created_objects: Vec<CreatedObject> = serde_json::from_str(&row.2).unwrap_or_default();
        Ok(BuildSession {
            session_id: row.0,
            user_id,
            npc_id,
            project_name: row.1,
            created_objects,
            created_at: DateTime::from_timestamp(row.3 as i64, 0).unwrap_or_else(Utc::now),
            updated_at: DateTime::from_timestamp(row.4 as i64, 0).unwrap_or_else(Utc::now),
        })
    }
}

async fn save_to_db(pool: &sqlx::PgPool, session: &BuildSession) -> Result<(), sqlx::Error> {
    let objects_json =
        serde_json::to_string(&session.created_objects).unwrap_or_else(|_| "[]".to_string());
    let created_at = session.created_at.timestamp() as i32;
    let updated_at = session.updated_at.timestamp() as i32;

    sqlx::query(
        "INSERT INTO npc_build_sessions (session_id, user_id, npc_id, project_name, created_objects, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (session_id) DO UPDATE SET \
         project_name = $4, created_objects = $5, updated_at = $7"
    )
    .bind(session.session_id)
    .bind(session.user_id)
    .bind(session.npc_id)
    .bind(&session.project_name)
    .bind(&objects_json)
    .bind(created_at)
    .bind(updated_at)
    .execute(pool)
    .await?;

    Ok(())
}
