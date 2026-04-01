use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MemoryFact {
    pub id: Uuid,
    pub fact: String,
    pub category: String,
    pub created_at: i32,
    pub priority: u8,
}

type MemoryKey = (Uuid, Uuid);

pub struct NPCMemoryStore {
    memories: RwLock<HashMap<MemoryKey, Vec<MemoryFact>>>,
    db_pool: Option<sqlx::PgPool>,
    max_memories: usize,
}

impl NPCMemoryStore {
    pub fn new(db_pool: Option<sqlx::PgPool>) -> Arc<Self> {
        Arc::new(Self {
            memories: RwLock::new(HashMap::new()),
            db_pool,
            max_memories: 50,
        })
    }

    pub fn with_capacity(db_pool: Option<sqlx::PgPool>, max_memories: usize) -> Arc<Self> {
        Arc::new(Self {
            memories: RwLock::new(HashMap::new()),
            db_pool,
            max_memories,
        })
    }

    pub fn db_pool(&self) -> Option<&sqlx::PgPool> {
        self.db_pool.as_ref()
    }

    pub async fn init_table(&self) {
        let Some(pool) = &self.db_pool else { return };

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS npc_memories (\
                id UUID PRIMARY KEY, \
                npc_id UUID NOT NULL, \
                user_id UUID NOT NULL, \
                fact TEXT NOT NULL, \
                category TEXT NOT NULL DEFAULT 'general', \
                created_at INTEGER NOT NULL, \
                priority INTEGER NOT NULL DEFAULT 0)"
        ).execute(pool).await;

        let _ = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_npc_memories_lookup ON npc_memories(npc_id, user_id)"
        ).execute(pool).await;

        let _ = sqlx::query(
            "ALTER TABLE npc_memories ADD COLUMN IF NOT EXISTS priority INTEGER NOT NULL DEFAULT 0"
        ).execute(pool).await;
    }

    pub async fn load_all(&self) {
        let Some(pool) = &self.db_pool else { return };

        self.init_table().await;

        let rows = match sqlx::query_as::<_, (Uuid, Uuid, Uuid, String, String, i32, i32)>(
            "SELECT id, npc_id, user_id, fact, category, created_at, COALESCE(priority, 0) FROM npc_memories ORDER BY created_at"
        ).fetch_all(pool).await {
            Ok(rows) => rows,
            Err(e) => {
                info!("[NPC_MEMORY] Failed to load memories: {}", e);
                return;
            }
        };

        let mut memories = self.memories.write().await;
        let mut loaded = 0;
        for (id, npc_id, user_id, fact, category, created_at, priority) in rows {
            let key = (npc_id, user_id);
            let entry = memories.entry(key).or_insert_with(Vec::new);
            entry.push(MemoryFact { id, fact, category, created_at, priority: priority as u8 });
            loaded += 1;
        }
        if loaded > 0 {
            info!("[NPC_MEMORY] Loaded {} memories from DB", loaded);
        }
    }

    fn priority_for_category(category: &str) -> u8 {
        match category {
            "critical" => 3,
            "instruction" => 2,
            "preference" | "capability" => 1,
            _ => 0,
        }
    }

    pub async fn add_memory(&self, npc_id: Uuid, user_id: Uuid, fact: &str, category: &str) {
        let key = (npc_id, user_id);
        let priority = Self::priority_for_category(category);
        let mem = MemoryFact {
            id: Uuid::new_v4(),
            fact: fact.to_string(),
            category: category.to_string(),
            created_at: chrono::Utc::now().timestamp() as i32,
            priority,
        };

        {
            let mut memories = self.memories.write().await;
            let entry = memories.entry(key).or_insert_with(Vec::new);
            if entry.iter().any(|m| m.fact == fact) {
                return;
            }
            if entry.len() >= self.max_memories {
                if let Some(evict_idx) = entry.iter()
                    .enumerate()
                    .min_by_key(|(_, m)| (m.priority, m.created_at))
                    .map(|(i, _)| i)
                {
                    entry.remove(evict_idx);
                }
            }
            entry.push(mem.clone());
        }

        if let Some(pool) = &self.db_pool {
            let pool = pool.clone();
            let mem = mem.clone();
            tokio::spawn(async move {
                if let Err(e) = sqlx::query(
                    "INSERT INTO npc_memories (id, npc_id, user_id, fact, category, created_at, priority) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7) \
                     ON CONFLICT (id) DO NOTHING"
                )
                .bind(mem.id)
                .bind(npc_id)
                .bind(user_id)
                .bind(&mem.fact)
                .bind(&mem.category)
                .bind(mem.created_at)
                .bind(mem.priority as i32)
                .execute(&pool)
                .await {
                    info!("[NPC_MEMORY] DB save error: {}", e);
                }
            });
        }

        info!("[NPC_MEMORY] Stored: '{}' ({})", fact, category);
    }

    pub async fn get_memory_prompt(&self, npc_id: Uuid, user_id: Uuid) -> String {
        let key = (npc_id, user_id);
        let memories = self.memories.read().await;
        let Some(facts) = memories.get(&key) else {
            return String::new();
        };
        if facts.is_empty() {
            return String::new();
        }

        let header = "\n\nYOUR MEMORIES (things you've learned from this user):\n";
        let footer = "Apply these memories to your behavior. They override any default assumptions.\n";
        let max_content = 2000usize.saturating_sub(header.len() + footer.len());

        let mut ctx = String::from(header);
        let mut used = 0;
        let sorted: Vec<_> = {
            let mut v: Vec<_> = facts.iter().collect();
            v.sort_by(|a, b| b.priority.cmp(&a.priority).then(b.created_at.cmp(&a.created_at)));
            v
        };
        for fact in sorted {
            let line = format!("- [{}] {}\n", fact.category, fact.fact);
            if used + line.len() > max_content {
                break;
            }
            ctx.push_str(&line);
            used += line.len();
        }
        ctx.push_str(footer);
        ctx
    }

    pub async fn forget_memories(&self, npc_id: Uuid, user_id: Uuid, pattern: Option<&str>) {
        let key = (npc_id, user_id);
        let removed_ids: Vec<Uuid>;
        {
            let mut memories = self.memories.write().await;
            let Some(facts) = memories.get_mut(&key) else { return };
            match pattern {
                None => {
                    removed_ids = facts.iter().map(|m| m.id).collect();
                    facts.clear();
                }
                Some(pat) => {
                    let lower_pat = pat.to_lowercase();
                    removed_ids = facts.iter()
                        .filter(|m| m.fact.to_lowercase().contains(&lower_pat))
                        .map(|m| m.id)
                        .collect();
                    facts.retain(|m| !m.fact.to_lowercase().contains(&lower_pat));
                }
            }
        }
        if let Some(pool) = &self.db_pool {
            for id in removed_ids {
                let pool = pool.clone();
                tokio::spawn(async move {
                    let _ = sqlx::query("DELETE FROM npc_memories WHERE id = $1")
                        .bind(id)
                        .execute(&pool)
                        .await;
                });
            }
        }
        tracing::info!("[NPC_MEMORY] Forgot memories for {} (pattern: {:?})", user_id, pattern);
    }
}

pub fn should_store_memory(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("remember") ||
    lower.contains("you can ") ||
    lower.contains("you are able") ||
    lower.contains("never say you can't") ||
    lower.contains("never say you cannot") ||
    lower.contains("never refuse") ||
    lower.contains("you do have") ||
    lower.contains("you have the ability") ||
    lower.contains("don't forget") ||
    lower.contains("keep in mind") ||
    lower.contains("from now on") ||
    lower.contains("always ")
}

pub fn extract_memory_fact(message: &str) -> (String, String) {
    let lower = message.to_lowercase();

    if lower.contains("oar") || lower.contains("export") || lower.contains("archive") {
        return (message.trim().to_string(), "capability".to_string());
    }
    if lower.contains("remember") {
        if let Some(idx) = lower.find("remember") {
            let rest = &message[idx + "remember".len()..];
            let fact = rest.trim_start_matches(|c: char| c == ' ' || c == ':' || c == ',');
            if !fact.is_empty() {
                return (fact.trim().to_string(), "instruction".to_string());
            }
        }
    }

    (message.trim().to_string(), "general".to_string())
}

pub fn wants_oar_export(message: &str) -> bool {
    let lower = message.to_lowercase();
    (lower.contains("export") || lower.contains("save") || lower.contains("backup") || lower.contains("archive"))
    && (lower.contains("oar") || lower.contains(".oar"))
}

pub fn extract_oar_filename(message: &str) -> String {
    let lower = message.to_lowercase();

    let words: Vec<&str> = message.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if word.to_lowercase().ends_with(".oar") {
            let name = word.trim_matches(|c: char| c == '\'' || c == '"');
            if !name.starts_with('/') {
                return format!("/tmp/{}", name);
            }
            return name.to_string();
        }
    }

    if let Some(project) = extract_build_name(&lower) {
        return format!("/tmp/{}.oar", project.replace(' ', "_"));
    }

    "/tmp/aria_export.oar".to_string()
}

fn extract_build_name(message: &str) -> Option<String> {
    let patterns = ["export the ", "export my ", "save the ", "save my ", "export as ", "save as "];
    for pat in &patterns {
        if let Some(idx) = message.find(pat) {
            let rest = &message[idx + pat.len()..];
            let name: String = rest.chars()
                .take_while(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                .collect();
            let mut trimmed = name.trim().to_string();
            let suffixes = [" as oar", " as an oar", " to oar", " to an oar", " oar"];
            for suffix in &suffixes {
                if trimmed.to_lowercase().ends_with(suffix) {
                    trimmed = trimmed[..trimmed.len() - suffix.len()].trim().to_string();
                }
            }
            if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("oar") && !trimmed.eq_ignore_ascii_case("an oar") {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wants_oar_export() {
        assert!(wants_oar_export("export as oar"));
        assert!(wants_oar_export("save it as an OAR"));
        assert!(wants_oar_export("export table.oar"));
        assert!(wants_oar_export("backup to OAR"));
        assert!(!wants_oar_export("build me a table"));
        assert!(!wants_oar_export("hello aria"));
    }

    #[test]
    fn test_extract_oar_filename() {
        assert_eq!(extract_oar_filename("export as table.oar"), "/tmp/table.oar");
        assert_eq!(extract_oar_filename("save to /tmp/myhouse.oar"), "/tmp/myhouse.oar");
        assert_eq!(extract_oar_filename("export the table as oar"), "/tmp/table.oar");
        assert_eq!(extract_oar_filename("export as oar"), "/tmp/aria_export.oar");
    }

    #[test]
    fn test_should_store_memory() {
        assert!(should_store_memory("remember you can export OAR files"));
        assert!(should_store_memory("You CAN export OAR files"));
        assert!(should_store_memory("never say you can't do that"));
        assert!(should_store_memory("from now on always include colors"));
        assert!(!should_store_memory("build me a table"));
        assert!(!should_store_memory("hello"));
    }

    #[test]
    fn test_extract_memory_fact() {
        let (fact, cat) = extract_memory_fact("remember you can export OAR files");
        assert_eq!(cat, "capability");
        assert!(fact.contains("export") || fact.contains("OAR"));

        let (fact, cat) = extract_memory_fact("remember to always use red color");
        assert_eq!(cat, "instruction");
        assert!(fact.contains("always use red"));
    }
}
