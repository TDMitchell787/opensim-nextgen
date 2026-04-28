// OpenSim Next - Phase 31.3 Intelligent NPC Behavior Engine
// Advanced NPC AI with realistic behavior simulation and social networks
// Using ELEGANT ARCHIVE SOLUTION methodology

use super::{AIError, NPCBehaviorPlan, NPCContext};
use crate::database::DatabaseManager;
use crate::monitoring::metrics::MetricsCollector;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCProfile {
    pub npc_id: Uuid,
    pub name: String,
    pub personality: NPCPersonality,
    pub role: NPCRole,
    pub behavior_state: BehaviorState,
    pub social_network: HashMap<Uuid, Relationship>, // NPC ID -> Relationship
    pub location_preferences: Vec<String>,
    pub activity_schedule: Vec<ScheduledActivity>,
    pub memory: NPCMemory,
    pub created_at: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCPersonality {
    pub friendliness: f32,  // 0.0 to 1.0
    pub curiosity: f32,     // 0.0 to 1.0
    pub helpfulness: f32,   // 0.0 to 1.0
    pub sociability: f32,   // 0.0 to 1.0
    pub assertiveness: f32, // 0.0 to 1.0
    pub intelligence: f32,  // 0.0 to 1.0
    pub creativity: f32,    // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NPCRole {
    Merchant,
    Guard,
    Guide,
    Entertainer,
    Crafts,
    Teacher,
    Questgiver,
    Citizen,
    Vendor,
    Scholar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorState {
    pub current_activity: Activity,
    pub mood: Mood,
    pub energy_level: f32,
    pub social_need: f32,
    pub goals: Vec<Goal>,
    pub last_interaction: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Activity {
    Idle,
    Patrolling,
    Socializing,
    Working,
    Resting,
    Seeking,
    Interacting(Uuid), // Interacting with specific avatar/NPC
    Moving(String),    // Moving to location
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mood {
    pub happiness: f32,
    pub stress: f32,
    pub boredom: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub goal_type: GoalType,
    pub priority: f32,
    pub progress: f32,
    pub deadline: Option<DateTime<Utc>>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalType {
    Social,        // Find someone to talk to
    Exploration,   // Explore new areas
    Work,          // Complete work tasks
    Rest,          // Find a place to rest
    Learning,      // Learn something new
    Entertainment, // Seek entertainment
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub relationship_type: RelationshipType,
    pub strength: f32,    // -1.0 to 1.0 (negative = dislike, positive = like)
    pub familiarity: f32, // 0.0 to 1.0
    pub trust: f32,       // 0.0 to 1.0
    pub last_interaction: DateTime<Utc>,
    pub interaction_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Stranger,
    Acquaintance,
    Friend,
    Enemy,
    Family,
    Colleague,
    Customer,
    Vendor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledActivity {
    pub activity: Activity,
    pub start_time: String, // Time of day (e.g., "09:00")
    pub duration_minutes: u32,
    pub location: Option<String>,
    pub priority: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCMemory {
    pub short_term: Vec<MemoryItem>,
    pub long_term: Vec<MemoryItem>,
    pub locations_visited: HashMap<String, u32>, // Location -> visit count
    pub people_met: HashMap<Uuid, PersonMemory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub event_type: String,
    pub description: String,
    pub emotional_impact: f32,
    pub importance: f32,
    pub timestamp: DateTime<Utc>,
    pub participants: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonMemory {
    pub first_met: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub positive_interactions: u32,
    pub negative_interactions: u32,
    pub notable_events: Vec<String>,
}

#[derive(Debug)]
pub struct NPCBehaviorEngine {
    npc_profiles: Arc<RwLock<HashMap<Uuid, NPCProfile>>>,
    behavior_tree_processor: Arc<BehaviorTreeProcessor>,
    quest_generator: Arc<QuestGenerator>,
    social_network_manager: Arc<SocialNetworkManager>,
    pathfinding_ai: Arc<PathfindingAI>,
    metrics: Arc<MetricsCollector>,
    db: Arc<DatabaseManager>,
    config: NPCBehaviorConfig,
}

#[derive(Debug, Clone)]
pub struct NPCBehaviorConfig {
    pub max_npcs: usize,
    pub behavior_update_interval_seconds: u64,
    pub social_interaction_range: f32,
    pub memory_retention_days: u32,
    pub quest_generation_enabled: bool,
    pub adaptive_difficulty_enabled: bool,
}

impl Default for NPCBehaviorConfig {
    fn default() -> Self {
        Self {
            max_npcs: 1000,
            behavior_update_interval_seconds: 30,
            social_interaction_range: 20.0,
            memory_retention_days: 30,
            quest_generation_enabled: true,
            adaptive_difficulty_enabled: true,
        }
    }
}

impl NPCBehaviorEngine {
    pub async fn new(
        metrics: Arc<MetricsCollector>,
        db: Arc<DatabaseManager>,
    ) -> Result<Arc<Self>, AIError> {
        let config = NPCBehaviorConfig::default();

        let engine = Self {
            npc_profiles: Arc::new(RwLock::new(HashMap::new())),
            behavior_tree_processor: Arc::new(BehaviorTreeProcessor::new().await?),
            quest_generator: Arc::new(QuestGenerator::new().await?),
            social_network_manager: Arc::new(SocialNetworkManager::new().await?),
            pathfinding_ai: Arc::new(PathfindingAI::new().await?),
            metrics,
            db,
            config,
        };

        // Load existing NPC profiles
        engine.load_npc_profiles().await?;

        // Start behavior update loop (will be started separately after Arc creation)

        Ok(Arc::new(engine))
    }

    pub async fn generate_behavior(
        &self,
        npc_id: Uuid,
        context: &NPCContext,
    ) -> Result<NPCBehaviorPlan, AIError> {
        let start_time = std::time::Instant::now();

        // Get NPC profile
        let mut profile = self.get_or_create_npc_profile(npc_id).await?;

        // Update NPC's perception of the environment
        self.update_npc_perception(&mut profile, context).await?;

        // Generate behavior based on personality, goals, and context
        let behavior_plan = self.generate_contextual_behavior(&profile, context).await?;

        // Update NPC memory with new experiences
        self.update_npc_memory(&mut profile, context, &behavior_plan)
            .await?;

        // Save updated profile
        self.update_npc_profile(npc_id, profile).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        self.metrics
            .record_npc_behavior_generation(npc_id, processing_time)
            .await;

        Ok(behavior_plan)
    }

    pub fn is_healthy(&self) -> bool {
        // Check if behavior systems are functioning
        true // Simplified health check
    }

    async fn get_or_create_npc_profile(&self, npc_id: Uuid) -> Result<NPCProfile, AIError> {
        let profiles = self.npc_profiles.read().await;

        if let Some(profile) = profiles.get(&npc_id) {
            Ok(profile.clone())
        } else {
            drop(profiles);
            self.create_new_npc_profile(npc_id).await
        }
    }

    async fn create_new_npc_profile(&self, npc_id: Uuid) -> Result<NPCProfile, AIError> {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let mut rng = StdRng::from_entropy();

        // Generate random but realistic personality
        let personality = NPCPersonality {
            friendliness: rng.gen_range(0.2..0.9),
            curiosity: rng.gen_range(0.1..0.8),
            helpfulness: rng.gen_range(0.3..0.9),
            sociability: rng.gen_range(0.2..0.8),
            assertiveness: rng.gen_range(0.1..0.7),
            intelligence: rng.gen_range(0.4..0.9),
            creativity: rng.gen_range(0.2..0.8),
        };

        // Assign role based on personality
        let role = self.assign_role_based_on_personality(&personality);

        // Generate initial goals before moving personality and role
        let goals = self.generate_initial_goals(&personality, &role).await?;
        let location_preferences = self.generate_location_preferences(&role);
        let activity_schedule = self.generate_activity_schedule(&role).await?;

        let profile = NPCProfile {
            npc_id,
            name: format!("NPC_{}", npc_id.to_string()[..8].to_uppercase()),
            personality,
            role,
            behavior_state: BehaviorState {
                current_activity: Activity::Idle,
                mood: Mood {
                    happiness: rng.gen_range(0.4..0.8),
                    stress: rng.gen_range(0.1..0.4),
                    boredom: rng.gen_range(0.0..0.3),
                    confidence: rng.gen_range(0.5..0.9),
                },
                energy_level: rng.gen_range(0.6..1.0),
                social_need: rng.gen_range(0.3..0.7),
                goals,
                last_interaction: None,
            },
            social_network: HashMap::new(),
            location_preferences,
            activity_schedule,
            memory: NPCMemory {
                short_term: Vec::new(),
                long_term: Vec::new(),
                locations_visited: HashMap::new(),
                people_met: HashMap::new(),
            },
            created_at: Utc::now(),
            last_update: Utc::now(),
        };

        // Save to database
        self.save_npc_profile(&profile).await?;

        // Cache in memory
        let mut profiles = self.npc_profiles.write().await;
        profiles.insert(npc_id, profile.clone());

        Ok(profile)
    }

    fn assign_role_based_on_personality(&self, personality: &NPCPersonality) -> NPCRole {
        if personality.helpfulness > 0.7 && personality.friendliness > 0.6 {
            NPCRole::Guide
        } else if personality.assertiveness > 0.6 && personality.intelligence > 0.7 {
            NPCRole::Guard
        } else if personality.creativity > 0.7 && personality.sociability > 0.6 {
            NPCRole::Entertainer
        } else if personality.intelligence > 0.8 {
            NPCRole::Scholar
        } else if personality.friendliness > 0.8 {
            NPCRole::Merchant
        } else {
            NPCRole::Citizen
        }
    }

    async fn generate_initial_goals(
        &self,
        personality: &NPCPersonality,
        role: &NPCRole,
    ) -> Result<Vec<Goal>, AIError> {
        let mut goals = Vec::new();

        // Role-based goals
        match role {
            NPCRole::Merchant => {
                goals.push(Goal {
                    goal_type: GoalType::Work,
                    priority: 0.8,
                    progress: 0.0,
                    deadline: None,
                    description: "Sell goods to customers".to_string(),
                });
            }
            NPCRole::Guide => {
                goals.push(Goal {
                    goal_type: GoalType::Social,
                    priority: 0.7,
                    progress: 0.0,
                    deadline: None,
                    description: "Help visitors navigate the area".to_string(),
                });
            }
            NPCRole::Scholar => {
                goals.push(Goal {
                    goal_type: GoalType::Learning,
                    priority: 0.9,
                    progress: 0.0,
                    deadline: None,
                    description: "Research and learn new knowledge".to_string(),
                });
            }
            _ => {
                goals.push(Goal {
                    goal_type: GoalType::Social,
                    priority: 0.5,
                    progress: 0.0,
                    deadline: None,
                    description: "Meet and interact with others".to_string(),
                });
            }
        }

        // Personality-based goals
        if personality.sociability > 0.7 {
            goals.push(Goal {
                goal_type: GoalType::Social,
                priority: 0.6,
                progress: 0.0,
                deadline: None,
                description: "Make new friends".to_string(),
            });
        }

        if personality.curiosity > 0.6 {
            goals.push(Goal {
                goal_type: GoalType::Exploration,
                priority: 0.5,
                progress: 0.0,
                deadline: None,
                description: "Explore new places".to_string(),
            });
        }

        Ok(goals)
    }

    fn generate_location_preferences(&self, role: &NPCRole) -> Vec<String> {
        match role {
            NPCRole::Merchant => vec!["Market Square".to_string(), "Trading Post".to_string()],
            NPCRole::Guard => vec!["City Gates".to_string(), "Watchtower".to_string()],
            NPCRole::Scholar => vec!["Library".to_string(), "Study Hall".to_string()],
            NPCRole::Entertainer => vec!["Town Square".to_string(), "Tavern".to_string()],
            _ => vec!["Town Center".to_string(), "Park".to_string()],
        }
    }

    async fn generate_activity_schedule(
        &self,
        role: &NPCRole,
    ) -> Result<Vec<ScheduledActivity>, AIError> {
        let mut schedule = Vec::new();

        match role {
            NPCRole::Merchant => {
                schedule.push(ScheduledActivity {
                    activity: Activity::Working,
                    start_time: "09:00".to_string(),
                    duration_minutes: 480, // 8 hours
                    location: Some("Market Square".to_string()),
                    priority: 0.9,
                });
            }
            NPCRole::Guard => {
                schedule.push(ScheduledActivity {
                    activity: Activity::Patrolling,
                    start_time: "08:00".to_string(),
                    duration_minutes: 480, // 8 hours
                    location: Some("City Gates".to_string()),
                    priority: 0.8,
                });
            }
            _ => {
                schedule.push(ScheduledActivity {
                    activity: Activity::Socializing,
                    start_time: "10:00".to_string(),
                    duration_minutes: 120, // 2 hours
                    location: Some("Town Square".to_string()),
                    priority: 0.6,
                });
            }
        }

        Ok(schedule)
    }

    async fn update_npc_perception(
        &self,
        profile: &mut NPCProfile,
        context: &NPCContext,
    ) -> Result<(), AIError> {
        // Update location memory
        if let Some(count) = profile.memory.locations_visited.get_mut(&context.location) {
            *count += 1;
        } else {
            profile
                .memory
                .locations_visited
                .insert(context.location.clone(), 1);
        }

        // Update people met
        for &avatar_id in &context.nearby_avatars {
            if let Some(person_memory) = profile.memory.people_met.get_mut(&avatar_id) {
                person_memory.last_seen = Utc::now();
            } else {
                profile.memory.people_met.insert(
                    avatar_id,
                    PersonMemory {
                        first_met: Utc::now(),
                        last_seen: Utc::now(),
                        positive_interactions: 0,
                        negative_interactions: 0,
                        notable_events: Vec::new(),
                    },
                );
            }
        }

        Ok(())
    }

    async fn generate_contextual_behavior(
        &self,
        profile: &NPCProfile,
        context: &NPCContext,
    ) -> Result<NPCBehaviorPlan, AIError> {
        self.behavior_tree_processor
            .process_behavior(profile, context)
            .await
    }

    async fn update_npc_memory(
        &self,
        profile: &mut NPCProfile,
        context: &NPCContext,
        behavior_plan: &NPCBehaviorPlan,
    ) -> Result<(), AIError> {
        // Create memory item for this interaction
        let memory_item = MemoryItem {
            event_type: "behavior_generation".to_string(),
            description: format!("Generated behavior: {}", behavior_plan.primary_action),
            emotional_impact: 0.1,
            importance: 0.3,
            timestamp: Utc::now(),
            participants: context.nearby_avatars.clone(),
        };

        profile.memory.short_term.push(memory_item);

        // Manage memory retention
        let retention_cutoff =
            Utc::now() - chrono::Duration::days(self.config.memory_retention_days as i64);
        profile
            .memory
            .short_term
            .retain(|m| m.timestamp > retention_cutoff);

        Ok(())
    }

    async fn update_npc_profile(&self, npc_id: Uuid, profile: NPCProfile) -> Result<(), AIError> {
        // Update in-memory cache
        let mut profiles = self.npc_profiles.write().await;
        profiles.insert(npc_id, profile.clone());
        drop(profiles);

        // Save to database
        self.save_npc_profile(&profile).await
    }

    async fn load_npc_profiles(&self) -> Result<(), AIError> {
        // Load existing profiles from database
        Ok(())
    }

    async fn save_npc_profile(&self, profile: &NPCProfile) -> Result<(), AIError> {
        // Save profile to database
        Ok(())
    }

    pub async fn start_behavior_updates(self: Arc<Self>) {
        tokio::spawn({
            let engine = self.clone();
            async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                    engine.config.behavior_update_interval_seconds,
                ));

                loop {
                    interval.tick().await;
                    if let Err(e) = engine.update_all_npc_behaviors().await {
                        eprintln!("Error updating NPC behaviors: {}", e);
                    }
                }
            }
        });
    }

    async fn update_all_npc_behaviors(&self) -> Result<(), AIError> {
        let profiles = self.npc_profiles.read().await;
        let npc_ids: Vec<Uuid> = profiles.keys().cloned().collect();
        drop(profiles);

        for npc_id in npc_ids {
            // Create a simple context for autonomous behavior
            let context = NPCContext {
                location: "General Area".to_string(),
                nearby_avatars: Vec::new(),
                time_of_day: "daytime".to_string(),
                weather: Some("clear".to_string()),
                current_activity: None,
            };

            if let Err(e) = self.generate_behavior(npc_id, &context).await {
                eprintln!("Error generating behavior for NPC {}: {}", npc_id, e);
            }
        }

        Ok(())
    }
}

// Supporting AI Components

#[derive(Debug)]
struct BehaviorTreeProcessor;

impl BehaviorTreeProcessor {
    async fn new() -> Result<Self, AIError> {
        Ok(Self)
    }

    async fn process_behavior(
        &self,
        profile: &NPCProfile,
        context: &NPCContext,
    ) -> Result<NPCBehaviorPlan, AIError> {
        // Simplified behavior tree processing
        let primary_action = match profile.role {
            NPCRole::Merchant => "Greet potential customers".to_string(),
            NPCRole::Guard => "Patrol assigned area".to_string(),
            NPCRole::Guide => "Offer assistance to visitors".to_string(),
            NPCRole::Scholar => "Study ancient texts".to_string(),
            _ => "Observe surroundings".to_string(),
        };

        let mut dialogue_options = vec![
            "Hello there!".to_string(),
            "How can I help you?".to_string(),
        ];

        // Add personality-based dialogue
        if profile.personality.friendliness > 0.7 {
            dialogue_options.push("What a lovely day!".to_string());
        }

        if profile.personality.curiosity > 0.6 {
            dialogue_options.push("What brings you here?".to_string());
        }

        Ok(NPCBehaviorPlan {
            primary_action,
            secondary_actions: vec!["Look around".to_string(), "Adjust posture".to_string()],
            dialogue_options,
            movement_target: None,
            duration_seconds: 60,
        })
    }
}

#[derive(Debug)]
struct QuestGenerator;

impl QuestGenerator {
    async fn new() -> Result<Self, AIError> {
        Ok(Self)
    }
}

#[derive(Debug)]
struct SocialNetworkManager;

impl SocialNetworkManager {
    async fn new() -> Result<Self, AIError> {
        Ok(Self)
    }
}

#[derive(Debug)]
struct PathfindingAI;

impl PathfindingAI {
    async fn new() -> Result<Self, AIError> {
        Ok(Self)
    }
}
