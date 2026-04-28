use super::ml_integration::{LLMConfig, LLMResponse, LocalLLMClient};
use super::npc_behavior::{BehaviorState, Mood, NPCPersonality, NPCProfile, NPCRole};
use super::AIError;
use crate::database::DatabaseManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueContext {
    pub npc_id: Uuid,
    pub speaker_id: Uuid,
    pub speaker_name: String,
    pub location: String,
    pub time_of_day: TimeOfDay,
    pub weather: Option<String>,
    pub previous_interactions: Vec<InteractionSummary>,
    pub nearby_objects: Vec<String>,
    pub current_quest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeOfDay {
    Dawn,
    Morning,
    Noon,
    Afternoon,
    Evening,
    Night,
    Midnight,
}

impl TimeOfDay {
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            5..=6 => TimeOfDay::Dawn,
            7..=11 => TimeOfDay::Morning,
            12 => TimeOfDay::Noon,
            13..=16 => TimeOfDay::Afternoon,
            17..=20 => TimeOfDay::Evening,
            21..=23 => TimeOfDay::Night,
            _ => TimeOfDay::Midnight,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            TimeOfDay::Dawn => "early morning at dawn",
            TimeOfDay::Morning => "morning",
            TimeOfDay::Noon => "midday",
            TimeOfDay::Afternoon => "afternoon",
            TimeOfDay::Evening => "evening",
            TimeOfDay::Night => "night",
            TimeOfDay::Midnight => "late night",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionSummary {
    pub timestamp: DateTime<Utc>,
    pub topic: String,
    pub sentiment: f32,
    pub key_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueResponse {
    pub npc_id: Uuid,
    pub response_text: String,
    pub emotion: NPCEmotion,
    pub suggested_actions: Vec<SuggestedAction>,
    pub relationship_change: f32,
    pub processing_time_ms: u64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NPCEmotion {
    Happy,
    Sad,
    Angry,
    Fearful,
    Surprised,
    Disgusted,
    Neutral,
    Curious,
    Friendly,
    Suspicious,
}

impl NPCEmotion {
    pub fn from_mood(mood: &Mood) -> Self {
        if mood.happiness > 0.7 {
            NPCEmotion::Happy
        } else if mood.stress > 0.7 {
            NPCEmotion::Angry
        } else if mood.boredom > 0.7 {
            NPCEmotion::Neutral
        } else if mood.confidence > 0.7 {
            NPCEmotion::Friendly
        } else if mood.stress > 0.5 && mood.confidence < 0.3 {
            NPCEmotion::Fearful
        } else {
            NPCEmotion::Neutral
        }
    }

    pub fn to_prompt_descriptor(&self) -> &str {
        match self {
            NPCEmotion::Happy => "cheerful and upbeat",
            NPCEmotion::Sad => "melancholic and subdued",
            NPCEmotion::Angry => "irritated and terse",
            NPCEmotion::Fearful => "nervous and hesitant",
            NPCEmotion::Surprised => "astonished and curious",
            NPCEmotion::Disgusted => "disapproving and curt",
            NPCEmotion::Neutral => "calm and measured",
            NPCEmotion::Curious => "inquisitive and engaged",
            NPCEmotion::Friendly => "warm and welcoming",
            NPCEmotion::Suspicious => "wary and guarded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    pub action_type: ActionType,
    pub description: String,
    pub priority: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    OfferQuest,
    GiveItem,
    ProvideDirection,
    ShareInformation,
    RequestHelp,
    InitiateTrade,
    EndConversation,
    FollowUp,
}

#[derive(Debug, Clone)]
pub struct DialogueConfig {
    pub max_response_tokens: usize,
    pub temperature: f32,
    pub enable_memory: bool,
    pub max_conversation_history: usize,
    pub fallback_responses_enabled: bool,
    pub cache_responses: bool,
    pub cache_ttl_seconds: u64,
}

impl Default for DialogueConfig {
    fn default() -> Self {
        Self {
            max_response_tokens: 256,
            temperature: 0.8,
            enable_memory: true,
            max_conversation_history: 10,
            fallback_responses_enabled: true,
            cache_responses: true,
            cache_ttl_seconds: 300,
        }
    }
}

pub struct NPCDialogueEngine {
    llm_client: Option<Arc<LocalLLMClient>>,
    db: Arc<DatabaseManager>,
    conversation_history: Arc<RwLock<HashMap<(Uuid, Uuid), Vec<ConversationTurn>>>>,
    response_cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    config: DialogueConfig,
    fallback_responses: FallbackResponses,
}

#[derive(Debug, Clone)]
struct ConversationTurn {
    speaker: Speaker,
    message: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
enum Speaker {
    NPC,
    User,
}

#[derive(Debug, Clone)]
struct CachedResponse {
    response: DialogueResponse,
    cached_at: DateTime<Utc>,
}

struct FallbackResponses {
    greetings: Vec<String>,
    farewells: Vec<String>,
    confused: Vec<String>,
    busy: Vec<String>,
    role_specific: HashMap<String, Vec<String>>,
}

impl Default for FallbackResponses {
    fn default() -> Self {
        let mut role_specific = HashMap::new();

        role_specific.insert(
            "Merchant".to_string(),
            vec![
                "Looking to buy or sell? I've got quality goods!".to_string(),
                "My prices are fair, I assure you.".to_string(),
                "Step right up! See what I have to offer.".to_string(),
            ],
        );

        role_specific.insert(
            "Guard".to_string(),
            vec![
                "Move along, citizen. Nothing to see here.".to_string(),
                "Keep the peace and we'll have no trouble.".to_string(),
                "I'm watching this area. Stay out of trouble.".to_string(),
            ],
        );

        role_specific.insert(
            "Guide".to_string(),
            vec![
                "Need directions? I know this place well.".to_string(),
                "I can show you around if you'd like.".to_string(),
                "First time here? Let me help you find your way.".to_string(),
            ],
        );

        role_specific.insert(
            "Scholar".to_string(),
            vec![
                "Knowledge is the greatest treasure.".to_string(),
                "I've been studying the ancient texts...".to_string(),
                "Ah, a curious mind! What would you like to know?".to_string(),
            ],
        );

        Self {
            greetings: vec![
                "Hello there!".to_string(),
                "Greetings, traveler.".to_string(),
                "Well met!".to_string(),
                "Good to see you.".to_string(),
            ],
            farewells: vec![
                "Safe travels!".to_string(),
                "Until we meet again.".to_string(),
                "Farewell, friend.".to_string(),
                "Take care out there.".to_string(),
            ],
            confused: vec![
                "I'm not sure I understand...".to_string(),
                "Could you say that again?".to_string(),
                "Hmm, that's an interesting question.".to_string(),
            ],
            busy: vec![
                "I'm a bit occupied right now.".to_string(),
                "Perhaps we can talk later?".to_string(),
                "Busy at the moment, sorry!".to_string(),
            ],
            role_specific,
        }
    }
}

impl NPCDialogueEngine {
    pub async fn new(
        llm_config: Option<LLMConfig>,
        db: Arc<DatabaseManager>,
        config: DialogueConfig,
    ) -> Result<Arc<Self>, AIError> {
        let llm_client = if let Some(cfg) = llm_config {
            match LocalLLMClient::new(cfg).await {
                Ok(client) => {
                    if client.health_check().await {
                        tracing::info!("LLM client connected successfully");
                        Some(client)
                    } else {
                        tracing::warn!("LLM health check failed, using fallback responses");
                        None
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to create LLM client: {}, using fallback responses",
                        e
                    );
                    None
                }
            }
        } else {
            tracing::info!("LLM not configured, using fallback responses only");
            None
        };

        let engine = Self {
            llm_client,
            db,
            conversation_history: Arc::new(RwLock::new(HashMap::new())),
            response_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            fallback_responses: FallbackResponses::default(),
        };

        Ok(Arc::new(engine))
    }

    pub async fn generate_dialogue(
        &self,
        npc_profile: &NPCProfile,
        context: &DialogueContext,
        user_message: &str,
    ) -> Result<DialogueResponse, AIError> {
        let start_time = std::time::Instant::now();

        if self.config.cache_responses {
            let cache_key = self.compute_cache_key(npc_profile, context, user_message);
            if let Some(cached) = self.get_cached_response(&cache_key).await {
                return Ok(cached);
            }
        }

        let response = if let Some(llm) = &self.llm_client {
            self.generate_llm_dialogue(llm, npc_profile, context, user_message)
                .await?
        } else {
            self.generate_fallback_dialogue(npc_profile, context, user_message)
                .await
        };

        self.record_conversation_turn(
            npc_profile.npc_id,
            context.speaker_id,
            user_message,
            &response.response_text,
        )
        .await;

        let mut final_response = response;
        final_response.processing_time_ms = start_time.elapsed().as_millis() as u64;
        final_response.generated_at = Utc::now();

        if self.config.cache_responses {
            let cache_key = self.compute_cache_key(npc_profile, context, user_message);
            self.cache_response(&cache_key, &final_response).await;
        }

        Ok(final_response)
    }

    async fn generate_llm_dialogue(
        &self,
        llm: &Arc<LocalLLMClient>,
        npc_profile: &NPCProfile,
        context: &DialogueContext,
        user_message: &str,
    ) -> Result<DialogueResponse, AIError> {
        let system_prompt = self.build_system_prompt(npc_profile, context).await;
        let conversation_context = self
            .build_conversation_context(npc_profile.npc_id, context.speaker_id)
            .await;

        let full_prompt = format!(
            "{}\n\nConversation history:\n{}\n\n{} says: \"{}\"\n\nRespond as {}:",
            system_prompt,
            conversation_context,
            context.speaker_name,
            user_message,
            npc_profile.name
        );

        let llm_response = llm.generate(&full_prompt).await?;

        let emotion = NPCEmotion::from_mood(&npc_profile.behavior_state.mood);
        let suggested_actions = self.infer_actions_from_response(&llm_response.text, npc_profile);
        let relationship_change =
            self.calculate_relationship_change(user_message, &llm_response.text);

        Ok(DialogueResponse {
            npc_id: npc_profile.npc_id,
            response_text: self.clean_response(&llm_response.text),
            emotion,
            suggested_actions,
            relationship_change,
            processing_time_ms: 0,
            generated_at: Utc::now(),
        })
    }

    async fn build_system_prompt(
        &self,
        npc_profile: &NPCProfile,
        context: &DialogueContext,
    ) -> String {
        let personality_desc = self.describe_personality(&npc_profile.personality);
        let role_desc = self.describe_role(&npc_profile.role);
        let emotion = NPCEmotion::from_mood(&npc_profile.behavior_state.mood);

        format!(
            r#"You are {}, a {} in a virtual world. Your personality: {}

Current situation:
- Location: {}
- Time: {}
- Weather: {}
- Your current mood: {}

Roleplay guidelines:
- Stay in character at all times
- Keep responses concise (1-3 sentences typically)
- React appropriately to the situation and your personality
- You may offer relevant services based on your role
- Be helpful but maintain your character's personality

Do not break character or mention that you are an AI."#,
            npc_profile.name,
            role_desc,
            personality_desc,
            context.location,
            context.time_of_day.description(),
            context.weather.as_deref().unwrap_or("clear"),
            emotion.to_prompt_descriptor()
        )
    }

    fn describe_personality(&self, personality: &NPCPersonality) -> String {
        let mut traits = Vec::new();

        if personality.friendliness > 0.7 {
            traits.push("very friendly");
        } else if personality.friendliness < 0.3 {
            traits.push("somewhat cold");
        }

        if personality.curiosity > 0.7 {
            traits.push("highly curious");
        }

        if personality.helpfulness > 0.7 {
            traits.push("eager to help");
        } else if personality.helpfulness < 0.3 {
            traits.push("reluctant to assist");
        }

        if personality.sociability > 0.7 {
            traits.push("outgoing and talkative");
        } else if personality.sociability < 0.3 {
            traits.push("reserved and quiet");
        }

        if personality.assertiveness > 0.7 {
            traits.push("confident and direct");
        } else if personality.assertiveness < 0.3 {
            traits.push("shy and uncertain");
        }

        if personality.intelligence > 0.7 {
            traits.push("intelligent and knowledgeable");
        }

        if personality.creativity > 0.7 {
            traits.push("creative and imaginative");
        }

        if traits.is_empty() {
            "balanced and average".to_string()
        } else {
            traits.join(", ")
        }
    }

    fn describe_role(&self, role: &NPCRole) -> &str {
        match role {
            NPCRole::Merchant => "merchant who buys and sells goods",
            NPCRole::Guard => "guard who keeps the peace",
            NPCRole::Guide => "guide who helps visitors navigate",
            NPCRole::Entertainer => "entertainer who performs for audiences",
            NPCRole::Crafts => "craftsperson skilled in their trade",
            NPCRole::Teacher => "teacher who shares knowledge",
            NPCRole::Questgiver => "quest giver with tasks for adventurers",
            NPCRole::Citizen => "citizen going about daily life",
            NPCRole::Vendor => "vendor selling specialized wares",
            NPCRole::Scholar => "scholar studying ancient knowledge",
        }
    }

    async fn build_conversation_context(&self, npc_id: Uuid, speaker_id: Uuid) -> String {
        let history = self.conversation_history.read().await;
        let key = (npc_id, speaker_id);

        if let Some(turns) = history.get(&key) {
            let recent: Vec<&ConversationTurn> = turns
                .iter()
                .rev()
                .take(self.config.max_conversation_history)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();

            recent
                .iter()
                .map(|turn| {
                    let speaker = match turn.speaker {
                        Speaker::NPC => "NPC",
                        Speaker::User => "User",
                    };
                    format!("{}: {}", speaker, turn.message)
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "(No previous conversation)".to_string()
        }
    }

    async fn generate_fallback_dialogue(
        &self,
        npc_profile: &NPCProfile,
        _context: &DialogueContext,
        user_message: &str,
    ) -> DialogueResponse {
        let message_lower = user_message.to_lowercase();
        let response_text = if message_lower.contains("hello")
            || message_lower.contains("hi")
            || message_lower.contains("greet")
        {
            self.get_random_response(&self.fallback_responses.greetings)
        } else if message_lower.contains("bye")
            || message_lower.contains("farewell")
            || message_lower.contains("leave")
        {
            self.get_random_response(&self.fallback_responses.farewells)
        } else if let Some(role_responses) = self
            .fallback_responses
            .role_specific
            .get(&format!("{:?}", npc_profile.role))
        {
            self.get_random_response(role_responses)
        } else {
            self.get_random_response(&self.fallback_responses.confused)
        };

        let emotion = NPCEmotion::from_mood(&npc_profile.behavior_state.mood);

        DialogueResponse {
            npc_id: npc_profile.npc_id,
            response_text,
            emotion,
            suggested_actions: Vec::new(),
            relationship_change: 0.0,
            processing_time_ms: 0,
            generated_at: Utc::now(),
        }
    }

    fn get_random_response(&self, responses: &[String]) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as usize;

        if responses.is_empty() {
            "...".to_string()
        } else {
            responses[seed % responses.len()].clone()
        }
    }

    fn infer_actions_from_response(
        &self,
        response: &str,
        npc_profile: &NPCProfile,
    ) -> Vec<SuggestedAction> {
        let mut actions = Vec::new();
        let response_lower = response.to_lowercase();

        if response_lower.contains("quest")
            || response_lower.contains("task")
            || response_lower.contains("mission")
        {
            actions.push(SuggestedAction {
                action_type: ActionType::OfferQuest,
                description: "Offer a quest to the player".to_string(),
                priority: 0.8,
            });
        }

        if response_lower.contains("buy")
            || response_lower.contains("sell")
            || response_lower.contains("trade")
        {
            actions.push(SuggestedAction {
                action_type: ActionType::InitiateTrade,
                description: "Open trade interface".to_string(),
                priority: 0.9,
            });
        }

        if response_lower.contains("follow")
            || response_lower.contains("show you")
            || response_lower.contains("this way")
        {
            actions.push(SuggestedAction {
                action_type: ActionType::ProvideDirection,
                description: "Guide player to location".to_string(),
                priority: 0.7,
            });
        }

        if response_lower.contains("take this")
            || response_lower.contains("here, have")
            || response_lower.contains("gift")
        {
            actions.push(SuggestedAction {
                action_type: ActionType::GiveItem,
                description: "Give item to player".to_string(),
                priority: 0.6,
            });
        }

        match npc_profile.role {
            NPCRole::Merchant | NPCRole::Vendor => {
                if actions
                    .iter()
                    .all(|a| !matches!(a.action_type, ActionType::InitiateTrade))
                {
                    actions.push(SuggestedAction {
                        action_type: ActionType::InitiateTrade,
                        description: "Offer to trade".to_string(),
                        priority: 0.5,
                    });
                }
            }
            NPCRole::Questgiver => {
                if actions
                    .iter()
                    .all(|a| !matches!(a.action_type, ActionType::OfferQuest))
                {
                    actions.push(SuggestedAction {
                        action_type: ActionType::OfferQuest,
                        description: "Offer available quests".to_string(),
                        priority: 0.5,
                    });
                }
            }
            NPCRole::Guide => {
                actions.push(SuggestedAction {
                    action_type: ActionType::ProvideDirection,
                    description: "Offer to guide".to_string(),
                    priority: 0.5,
                });
            }
            _ => {}
        }

        actions.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        actions
    }

    fn calculate_relationship_change(&self, user_message: &str, _response: &str) -> f32 {
        let message_lower = user_message.to_lowercase();
        let mut change: f32 = 0.0;

        let positive_words = [
            "thank",
            "please",
            "help",
            "friend",
            "appreciate",
            "wonderful",
            "great",
        ];
        let negative_words = [
            "hate", "stupid", "idiot", "ugly", "terrible", "awful", "die",
        ];

        for word in positive_words.iter() {
            if message_lower.contains(word) {
                change += 0.05;
            }
        }

        for word in negative_words.iter() {
            if message_lower.contains(word) {
                change -= 0.1;
            }
        }

        change.max(-0.5_f32).min(0.5_f32)
    }

    fn clean_response(&self, response: &str) -> String {
        let cleaned = response
            .trim()
            .trim_start_matches(|c: char| c == '"' || c == '\'' || c == ':')
            .trim_end_matches(|c: char| c == '"' || c == '\'')
            .trim();

        if cleaned.len() > 500 {
            let mut truncated = cleaned[..497].to_string();
            truncated.push_str("...");
            truncated
        } else {
            cleaned.to_string()
        }
    }

    async fn record_conversation_turn(
        &self,
        npc_id: Uuid,
        speaker_id: Uuid,
        user_message: &str,
        npc_response: &str,
    ) {
        if !self.config.enable_memory {
            return;
        }

        let mut history = self.conversation_history.write().await;
        let key = (npc_id, speaker_id);

        let turns = history.entry(key).or_insert_with(Vec::new);

        turns.push(ConversationTurn {
            speaker: Speaker::User,
            message: user_message.to_string(),
            timestamp: Utc::now(),
        });

        turns.push(ConversationTurn {
            speaker: Speaker::NPC,
            message: npc_response.to_string(),
            timestamp: Utc::now(),
        });

        while turns.len() > self.config.max_conversation_history * 2 {
            turns.remove(0);
        }
    }

    fn compute_cache_key(
        &self,
        npc_profile: &NPCProfile,
        context: &DialogueContext,
        message: &str,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        npc_profile.npc_id.hash(&mut hasher);
        format!("{:?}", npc_profile.role).hash(&mut hasher);
        context.location.hash(&mut hasher);
        message.to_lowercase().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    async fn get_cached_response(&self, cache_key: &str) -> Option<DialogueResponse> {
        let cache = self.response_cache.read().await;
        if let Some(cached) = cache.get(cache_key) {
            let age = (Utc::now() - cached.cached_at).num_seconds() as u64;
            if age < self.config.cache_ttl_seconds {
                return Some(cached.response.clone());
            }
        }
        None
    }

    async fn cache_response(&self, cache_key: &str, response: &DialogueResponse) {
        let mut cache = self.response_cache.write().await;

        if cache.len() >= 1000 {
            let keys_to_remove: Vec<String> = cache.keys().take(100).cloned().collect();
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }

        cache.insert(
            cache_key.to_string(),
            CachedResponse {
                response: response.clone(),
                cached_at: Utc::now(),
            },
        );
    }

    pub async fn clear_conversation(&self, npc_id: Uuid, speaker_id: Uuid) {
        let mut history = self.conversation_history.write().await;
        history.remove(&(npc_id, speaker_id));
    }

    pub async fn clear_all_conversations(&self) {
        self.conversation_history.write().await.clear();
    }

    pub fn is_llm_available(&self) -> bool {
        self.llm_client.is_some()
    }

    pub async fn health_check(&self) -> bool {
        if let Some(llm) = &self.llm_client {
            llm.health_check().await
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDescriptionRequest {
    pub asset_id: Uuid,
    pub asset_name: String,
    pub asset_type: String,
    pub creator_name: Option<String>,
    pub existing_description: Option<String>,
    pub tags: Vec<String>,
    pub prim_count: Option<u32>,
    pub script_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDescriptionResponse {
    pub asset_id: Uuid,
    pub generated_description: String,
    pub suggested_tags: Vec<String>,
    pub quality_estimate: f32,
    pub processing_time_ms: u64,
}

pub struct AssetDescriptionGenerator {
    llm_client: Option<Arc<LocalLLMClient>>,
    cache: Arc<RwLock<HashMap<Uuid, AssetDescriptionResponse>>>,
}

impl AssetDescriptionGenerator {
    pub async fn new(llm_config: Option<LLMConfig>) -> Result<Arc<Self>, AIError> {
        let llm_client = if let Some(cfg) = llm_config {
            LocalLLMClient::new(cfg).await.ok()
        } else {
            None
        };

        Ok(Arc::new(Self {
            llm_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }))
    }

    pub async fn generate_description(
        &self,
        request: &AssetDescriptionRequest,
    ) -> Result<AssetDescriptionResponse, AIError> {
        let start_time = std::time::Instant::now();

        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&request.asset_id) {
                return Ok(cached.clone());
            }
        }

        let (description, tags) = if let Some(llm) = &self.llm_client {
            self.generate_llm_description(llm, request).await?
        } else {
            self.generate_fallback_description(request)
        };

        let quality_estimate = self.estimate_quality(request);

        let response = AssetDescriptionResponse {
            asset_id: request.asset_id,
            generated_description: description,
            suggested_tags: tags,
            quality_estimate,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        };

        {
            let mut cache = self.cache.write().await;
            if cache.len() >= 500 {
                let keys: Vec<Uuid> = cache.keys().take(50).cloned().collect();
                for key in keys {
                    cache.remove(&key);
                }
            }
            cache.insert(request.asset_id, response.clone());
        }

        Ok(response)
    }

    async fn generate_llm_description(
        &self,
        llm: &Arc<LocalLLMClient>,
        request: &AssetDescriptionRequest,
    ) -> Result<(String, Vec<String>), AIError> {
        let prompt = format!(
            r#"Generate a brief, engaging description for a virtual world asset.

Asset details:
- Name: {}
- Type: {}
- Creator: {}
- Current description: {}
- Tags: {}
- Prim count: {}
- Script count: {}

Write a 1-2 sentence description that would help users understand what this asset is and why they might want it. Also suggest 3-5 relevant tags.

Format your response as:
Description: [your description]
Tags: [tag1, tag2, tag3]"#,
            request.asset_name,
            request.asset_type,
            request.creator_name.as_deref().unwrap_or("Unknown"),
            request.existing_description.as_deref().unwrap_or("None"),
            request.tags.join(", "),
            request
                .prim_count
                .map(|c| c.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            request
                .script_count
                .map(|c| c.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
        );

        let response = llm.generate(&prompt).await?;

        let mut description = String::new();
        let mut tags = Vec::new();

        for line in response.text.lines() {
            let line = line.trim();
            if line.to_lowercase().starts_with("description:") {
                description = line[12..].trim().to_string();
            } else if line.to_lowercase().starts_with("tags:") {
                let tag_str = line[5..].trim();
                tags = tag_str
                    .split(',')
                    .map(|t| t.trim().trim_matches(|c| c == '[' || c == ']').to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
            }
        }

        if description.is_empty() {
            description = response
                .text
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
        }

        Ok((description, tags))
    }

    fn generate_fallback_description(
        &self,
        request: &AssetDescriptionRequest,
    ) -> (String, Vec<String>) {
        let description = if let Some(existing) = &request.existing_description {
            if !existing.is_empty() {
                existing.clone()
            } else {
                format!(
                    "A {} named '{}'.",
                    request.asset_type.to_lowercase(),
                    request.asset_name
                )
            }
        } else {
            format!(
                "A {} named '{}'.",
                request.asset_type.to_lowercase(),
                request.asset_name
            )
        };

        let mut tags = request.tags.clone();
        if tags.is_empty() {
            tags.push(request.asset_type.to_lowercase());
        }

        (description, tags)
    }

    fn estimate_quality(&self, request: &AssetDescriptionRequest) -> f32 {
        let mut score: f32 = 0.5;

        if request
            .existing_description
            .as_ref()
            .map(|d| d.len() > 20)
            .unwrap_or(false)
        {
            score += 0.1;
        }

        if !request.tags.is_empty() {
            score += 0.1;
        }

        if request.creator_name.is_some() {
            score += 0.05;
        }

        if let Some(prims) = request.prim_count {
            if prims > 0 && prims < 100 {
                score += 0.1;
            } else if prims >= 100 {
                score += 0.05;
            }
        }

        score.min(1.0)
    }

    pub fn is_llm_available(&self) -> bool {
        self.llm_client.is_some()
    }
}
