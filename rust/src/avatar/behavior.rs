//! Avatar Behavior System for OpenSim Next
//!
//! Provides advanced avatar behavior management including animations,
//! gestures, auto-behaviors, and expressions.

use super::*;
use rand;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration, Instant};
use tracing::{debug, error, info};
use uuid::Uuid;

/// Avatar behavior management system
#[derive(Debug)]
pub struct BehaviorSystem {
    active_behaviors: Arc<RwLock<HashMap<Uuid, AvatarBehaviorState>>>,
    behavior_sender: mpsc::UnboundedSender<BehaviorCommand>,
    animation_library: Arc<RwLock<HashMap<Uuid, AnimationDefinition>>>,
    gesture_library: Arc<RwLock<HashMap<Uuid, GestureDefinition>>>,
}

/// Internal avatar behavior state
#[derive(Debug, Clone)]
struct AvatarBehaviorState {
    avatar_id: Uuid,
    active_animations: HashMap<Uuid, ActiveAnimation>,
    active_gestures: HashMap<Uuid, ActiveGesture>,
    auto_behaviors: Vec<AutoBehaviorInstance>,
    current_expression: Option<ActiveExpression>,
    last_activity: Instant,
    behavior_context: BehaviorContext,
}

/// Active animation instance
#[derive(Debug, Clone)]
struct ActiveAnimation {
    animation_id: Uuid,
    start_time: Instant,
    priority: i32,
    blend_weight: f32,
    loop_animation: bool,
    duration: Option<Duration>,
    current_frame: f32,
}

/// Active gesture instance
#[derive(Debug, Clone)]
struct ActiveGesture {
    gesture_id: Uuid,
    start_time: Instant,
    current_step: usize,
    steps: Vec<GestureStep>,
}

/// Gesture execution step
#[derive(Debug, Clone)]
struct GestureStep {
    step_type: GestureStepType,
    duration: Duration,
    parameters: HashMap<String, f32>,
}

/// Types of gesture steps
#[derive(Debug, Clone)]
enum GestureStepType {
    Animation { animation_id: Uuid },
    Sound { sound_id: Uuid, volume: f32 },
    Chat { message: String, channel: i32 },
    Wait { duration: Duration },
    Expression { expression: FacialExpression },
}

/// Auto behavior instance
#[derive(Debug, Clone)]
struct AutoBehaviorInstance {
    behavior_id: Uuid,
    trigger: BehaviorTrigger,
    actions: Vec<BehaviorAction>,
    last_triggered: Option<Instant>,
    cooldown: Duration,
    enabled: bool,
}

/// Active facial expression
#[derive(Debug, Clone)]
struct ActiveExpression {
    expression: FacialExpression,
    start_time: Instant,
    current_blend: f32,
    target_blend: f32,
}

/// Behavior context for decision making
#[derive(Debug, Clone)]
struct BehaviorContext {
    current_movement: MovementType,
    last_interaction: Option<InteractionType>,
    region_id: Uuid,
    nearby_avatars: Vec<Uuid>,
    current_activity: ActivityType,
}

/// Avatar activity types
#[derive(Debug, Clone)]
enum ActivityType {
    Idle,
    Moving,
    Interacting,
    Building,
    Chatting,
    Shopping,
    Exploring,
}

/// Animation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDefinition {
    pub animation_id: Uuid,
    pub name: String,
    pub asset_id: Uuid,
    pub duration: Option<f32>,
    pub loop_animation: bool,
    pub priority: i32,
    pub blend_type: AnimationBlendType,
    pub bone_masks: Vec<BoneMask>,
    pub metadata: HashMap<String, String>,
}

/// Animation blend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationBlendType {
    Replace,
    Additive,
    Override,
    Blend { weight: f32 },
}

/// Bone mask for selective animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneMask {
    pub bone_name: String,
    pub weight: f32,
    pub children_inherit: bool,
}

/// Gesture definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureDefinition {
    pub gesture_id: Uuid,
    pub name: String,
    pub trigger_words: Vec<String>,
    pub steps: Vec<GestureStepDefinition>,
    pub category: GestureCategory,
    pub access_level: GestureAccessLevel,
}

/// Gesture step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureStepDefinition {
    pub step_type: String,
    pub duration_ms: u64,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Gesture categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureCategory {
    Greeting,
    Emotion,
    Dance,
    Communication,
    Action,
    Custom,
}

/// Gesture access levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureAccessLevel {
    Public,
    Friends,
    Group,
    Private,
}

/// Behavior system commands
#[derive(Debug, Clone)]
enum BehaviorCommand {
    StartAnimation {
        avatar_id: Uuid,
        animation: AnimationState,
    },
    StopAnimation {
        avatar_id: Uuid,
        animation_id: Uuid,
    },
    TriggerGesture {
        avatar_id: Uuid,
        gesture_id: Uuid,
        trigger_text: Option<String>,
    },
    UpdateExpression {
        avatar_id: Uuid,
        expression: FacialExpression,
    },
    UpdateContext {
        avatar_id: Uuid,
        context: BehaviorContext,
    },
    ProcessAutoTriggers {
        avatar_id: Uuid,
    },
}

impl BehaviorSystem {
    /// Create new behavior system
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let active_behaviors = Arc::new(RwLock::new(HashMap::new()));
        let animation_library = Arc::new(RwLock::new(HashMap::new()));
        let gesture_library = Arc::new(RwLock::new(HashMap::new()));

        // Spawn behavior processing task
        let behaviors_clone = active_behaviors.clone();
        let animations_clone = animation_library.clone();
        let gestures_clone = gesture_library.clone();

        tokio::spawn(async move {
            Self::behavior_processor(
                &mut receiver,
                behaviors_clone,
                animations_clone,
                gestures_clone,
            )
            .await;
        });

        // Spawn auto-trigger processing task
        let behaviors_clone2 = active_behaviors.clone();
        let sender_clone = sender.clone();

        tokio::spawn(async move {
            Self::auto_trigger_processor(behaviors_clone2, sender_clone).await;
        });

        Self {
            active_behaviors,
            behavior_sender: sender,
            animation_library,
            gesture_library,
        }
    }

    /// Start avatar behaviors
    pub async fn start_avatar_behaviors(
        &self,
        avatar_id: Uuid,
        behavior: &AvatarBehavior,
    ) -> AvatarResult<()> {
        info!("Starting behaviors for avatar: {}", avatar_id);

        let behavior_state = AvatarBehaviorState {
            avatar_id,
            active_animations: HashMap::new(),
            active_gestures: HashMap::new(),
            auto_behaviors: self.create_auto_behavior_instances(&behavior.auto_behaviors),
            current_expression: None,
            last_activity: Instant::now(),
            behavior_context: BehaviorContext {
                current_movement: MovementType::Standing,
                last_interaction: None,
                region_id: Uuid::nil(),
                nearby_avatars: Vec::new(),
                current_activity: ActivityType::Idle,
            },
        };

        {
            let mut behaviors = self.active_behaviors.write().await;
            behaviors.insert(avatar_id, behavior_state);
        }

        // Start any default animations
        for animation in &behavior.animations {
            self.start_animation(avatar_id, animation.clone()).await?;
        }

        info!("Avatar behaviors started successfully: {}", avatar_id);
        Ok(())
    }

    /// Stop avatar behaviors
    pub async fn stop_avatar_behaviors(&self, avatar_id: Uuid) -> AvatarResult<()> {
        info!("Stopping behaviors for avatar: {}", avatar_id);

        {
            let mut behaviors = self.active_behaviors.write().await;
            behaviors.remove(&avatar_id);
        }

        info!("Avatar behaviors stopped successfully: {}", avatar_id);
        Ok(())
    }

    /// Validate behavior configuration
    pub fn validate_behavior(&self, behavior: &AvatarBehavior) -> AvatarResult<()> {
        debug!("Validating avatar behavior configuration");

        // Validate animations
        for animation in &behavior.animations {
            if animation.priority < 0 || animation.priority > 100 {
                return Err(AvatarError::InvalidData {
                    reason: "Animation priority must be between 0 and 100".to_string(),
                });
            }

            if animation.blend_weight < 0.0 || animation.blend_weight > 1.0 {
                return Err(AvatarError::InvalidData {
                    reason: "Animation blend weight must be between 0.0 and 1.0".to_string(),
                });
            }
        }

        // Validate gestures
        for gesture in &behavior.gestures {
            if gesture.name.is_empty() {
                return Err(AvatarError::InvalidData {
                    reason: "Gesture name cannot be empty".to_string(),
                });
            }

            if gesture.trigger.is_empty() {
                return Err(AvatarError::InvalidData {
                    reason: "Gesture trigger cannot be empty".to_string(),
                });
            }
        }

        // Validate auto behaviors
        for auto_behavior in &behavior.auto_behaviors {
            if auto_behavior.cooldown_seconds < 0.0 {
                return Err(AvatarError::InvalidData {
                    reason: "Auto behavior cooldown cannot be negative".to_string(),
                });
            }

            if auto_behavior.actions.is_empty() {
                return Err(AvatarError::InvalidData {
                    reason: "Auto behavior must have at least one action".to_string(),
                });
            }
        }

        debug!("Avatar behavior validation successful");
        Ok(())
    }

    /// Start animation
    pub async fn start_animation(
        &self,
        avatar_id: Uuid,
        animation: AnimationState,
    ) -> AvatarResult<()> {
        debug!(
            "Starting animation for avatar {}: {}",
            avatar_id, animation.name
        );

        let command = BehaviorCommand::StartAnimation {
            avatar_id,
            animation,
        };

        self.behavior_sender
            .send(command)
            .map_err(|_| AvatarError::SystemError {
                message: "Failed to send animation command".to_string(),
            })?;

        Ok(())
    }

    /// Stop animation
    pub async fn stop_animation(&self, avatar_id: Uuid, animation_id: Uuid) -> AvatarResult<()> {
        debug!(
            "Stopping animation for avatar {}: {}",
            avatar_id, animation_id
        );

        let command = BehaviorCommand::StopAnimation {
            avatar_id,
            animation_id,
        };

        self.behavior_sender
            .send(command)
            .map_err(|_| AvatarError::SystemError {
                message: "Failed to send stop animation command".to_string(),
            })?;

        Ok(())
    }

    /// Trigger gesture
    pub async fn trigger_gesture(
        &self,
        avatar_id: Uuid,
        gesture_id: Uuid,
        trigger_text: Option<String>,
    ) -> AvatarResult<()> {
        debug!(
            "Triggering gesture for avatar {}: {}",
            avatar_id, gesture_id
        );

        let command = BehaviorCommand::TriggerGesture {
            avatar_id,
            gesture_id,
            trigger_text,
        };

        self.behavior_sender
            .send(command)
            .map_err(|_| AvatarError::SystemError {
                message: "Failed to send gesture command".to_string(),
            })?;

        Ok(())
    }

    /// Update facial expression
    pub async fn update_expression(
        &self,
        avatar_id: Uuid,
        expression: FacialExpression,
    ) -> AvatarResult<()> {
        debug!(
            "Updating expression for avatar {}: {}",
            avatar_id, expression.name
        );

        let command = BehaviorCommand::UpdateExpression {
            avatar_id,
            expression,
        };

        self.behavior_sender
            .send(command)
            .map_err(|_| AvatarError::SystemError {
                message: "Failed to send expression command".to_string(),
            })?;

        Ok(())
    }

    /// Update behavior context
    pub async fn update_context(
        &self,
        avatar_id: Uuid,
        movement: MovementType,
        interaction: Option<InteractionType>,
        region_id: Uuid,
    ) -> AvatarResult<()> {
        debug!("Updating context for avatar {}", avatar_id);

        let context = BehaviorContext {
            current_movement: movement,
            last_interaction: interaction,
            region_id,
            nearby_avatars: Vec::new(), // This would be populated by region system
            current_activity: ActivityType::Idle, // This would be determined by activity detection
        };

        let command = BehaviorCommand::UpdateContext { avatar_id, context };

        self.behavior_sender
            .send(command)
            .map_err(|_| AvatarError::SystemError {
                message: "Failed to send context update command".to_string(),
            })?;

        Ok(())
    }

    /// Get active animations for avatar
    pub async fn get_active_animations(&self, avatar_id: Uuid) -> Vec<AnimationState> {
        let behaviors = self.active_behaviors.read().await;

        if let Some(state) = behaviors.get(&avatar_id) {
            state
                .active_animations
                .values()
                .map(|anim| AnimationState {
                    animation_id: anim.animation_id,
                    name: format!("Animation_{}", anim.animation_id),
                    priority: anim.priority,
                    loop_animation: anim.loop_animation,
                    start_time: chrono::DateTime::from_timestamp(
                        anim.start_time.elapsed().as_secs() as i64,
                        0,
                    )
                    .unwrap_or_default(),
                    duration: anim.duration.map(|d| d.as_secs_f32()),
                    blend_weight: anim.blend_weight,
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Add animation to library
    pub async fn add_animation_definition(&self, animation: AnimationDefinition) {
        let mut library = self.animation_library.write().await;
        library.insert(animation.animation_id, animation);
    }

    /// Add gesture to library
    pub async fn add_gesture_definition(&self, gesture: GestureDefinition) {
        let mut library = self.gesture_library.write().await;
        library.insert(gesture.gesture_id, gesture);
    }

    // Private implementation methods

    async fn behavior_processor(
        receiver: &mut mpsc::UnboundedReceiver<BehaviorCommand>,
        behaviors: Arc<RwLock<HashMap<Uuid, AvatarBehaviorState>>>,
        animations: Arc<RwLock<HashMap<Uuid, AnimationDefinition>>>,
        gestures: Arc<RwLock<HashMap<Uuid, GestureDefinition>>>,
    ) {
        while let Some(command) = receiver.recv().await {
            match command {
                BehaviorCommand::StartAnimation {
                    avatar_id,
                    animation,
                } => {
                    Self::process_start_animation(avatar_id, animation, &behaviors, &animations)
                        .await;
                }
                BehaviorCommand::StopAnimation {
                    avatar_id,
                    animation_id,
                } => {
                    Self::process_stop_animation(avatar_id, animation_id, &behaviors).await;
                }
                BehaviorCommand::TriggerGesture {
                    avatar_id,
                    gesture_id,
                    trigger_text,
                } => {
                    Self::process_trigger_gesture(
                        avatar_id,
                        gesture_id,
                        trigger_text,
                        &behaviors,
                        &gestures,
                    )
                    .await;
                }
                BehaviorCommand::UpdateExpression {
                    avatar_id,
                    expression,
                } => {
                    Self::process_update_expression(avatar_id, expression, &behaviors).await;
                }
                BehaviorCommand::UpdateContext { avatar_id, context } => {
                    Self::process_update_context(avatar_id, context, &behaviors).await;
                }
                BehaviorCommand::ProcessAutoTriggers { avatar_id } => {
                    Self::process_auto_triggers(avatar_id, &behaviors).await;
                }
            }
        }
    }

    async fn process_start_animation(
        avatar_id: Uuid,
        animation: AnimationState,
        behaviors: &Arc<RwLock<HashMap<Uuid, AvatarBehaviorState>>>,
        _animations: &Arc<RwLock<HashMap<Uuid, AnimationDefinition>>>,
    ) {
        let mut behaviors_guard = behaviors.write().await;
        if let Some(state) = behaviors_guard.get_mut(&avatar_id) {
            let active_anim = ActiveAnimation {
                animation_id: animation.animation_id,
                start_time: Instant::now(),
                priority: animation.priority,
                blend_weight: animation.blend_weight,
                loop_animation: animation.loop_animation,
                duration: animation.duration.map(Duration::from_secs_f32),
                current_frame: 0.0,
            };

            state
                .active_animations
                .insert(animation.animation_id, active_anim);
            state.last_activity = Instant::now();
        }
    }

    async fn process_stop_animation(
        avatar_id: Uuid,
        animation_id: Uuid,
        behaviors: &Arc<RwLock<HashMap<Uuid, AvatarBehaviorState>>>,
    ) {
        let mut behaviors_guard = behaviors.write().await;
        if let Some(state) = behaviors_guard.get_mut(&avatar_id) {
            state.active_animations.remove(&animation_id);
        }
    }

    async fn process_trigger_gesture(
        avatar_id: Uuid,
        gesture_id: Uuid,
        _trigger_text: Option<String>,
        behaviors: &Arc<RwLock<HashMap<Uuid, AvatarBehaviorState>>>,
        gestures: &Arc<RwLock<HashMap<Uuid, GestureDefinition>>>,
    ) {
        let gesture_def = {
            let gestures_guard = gestures.read().await;
            gestures_guard.get(&gesture_id).cloned()
        };

        if let Some(_gesture) = gesture_def {
            let mut behaviors_guard = behaviors.write().await;
            if let Some(state) = behaviors_guard.get_mut(&avatar_id) {
                // Create active gesture from definition
                let active_gesture = ActiveGesture {
                    gesture_id,
                    start_time: Instant::now(),
                    current_step: 0,
                    steps: Vec::new(), // Would be populated from gesture definition
                };

                state.active_gestures.insert(gesture_id, active_gesture);
                state.last_activity = Instant::now();
            }
        }
    }

    async fn process_update_expression(
        avatar_id: Uuid,
        expression: FacialExpression,
        behaviors: &Arc<RwLock<HashMap<Uuid, AvatarBehaviorState>>>,
    ) {
        let mut behaviors_guard = behaviors.write().await;
        if let Some(state) = behaviors_guard.get_mut(&avatar_id) {
            let active_expr = ActiveExpression {
                expression,
                start_time: Instant::now(),
                current_blend: 0.0,
                target_blend: 1.0,
            };

            state.current_expression = Some(active_expr);
            state.last_activity = Instant::now();
        }
    }

    async fn process_update_context(
        avatar_id: Uuid,
        context: BehaviorContext,
        behaviors: &Arc<RwLock<HashMap<Uuid, AvatarBehaviorState>>>,
    ) {
        let mut behaviors_guard = behaviors.write().await;
        if let Some(state) = behaviors_guard.get_mut(&avatar_id) {
            state.behavior_context = context;
            state.last_activity = Instant::now();
        }
    }

    async fn process_auto_triggers(
        avatar_id: Uuid,
        behaviors: &Arc<RwLock<HashMap<Uuid, AvatarBehaviorState>>>,
    ) {
        let mut behaviors_guard = behaviors.write().await;
        if let Some(state) = behaviors_guard.get_mut(&avatar_id) {
            let now = Instant::now();

            for auto_behavior in &mut state.auto_behaviors {
                if !auto_behavior.enabled {
                    continue;
                }

                // Check cooldown
                if let Some(last_triggered) = auto_behavior.last_triggered {
                    if now.duration_since(last_triggered) < auto_behavior.cooldown {
                        continue;
                    }
                }

                // Check trigger condition
                let should_trigger = match &auto_behavior.trigger {
                    BehaviorTrigger::Idle { duration_seconds } => {
                        now.duration_since(state.last_activity)
                            >= Duration::from_secs_f32(*duration_seconds)
                    }
                    BehaviorTrigger::Movement { movement_type } => {
                        state.behavior_context.current_movement == *movement_type
                    }
                    BehaviorTrigger::Random {
                        probability,
                        interval_seconds,
                    } => {
                        if let Some(last) = auto_behavior.last_triggered {
                            if now.duration_since(last)
                                >= Duration::from_secs_f32(*interval_seconds)
                            {
                                rand::random::<f32>() < *probability
                            } else {
                                false
                            }
                        } else {
                            rand::random::<f32>() < *probability
                        }
                    }
                    _ => false, // Other triggers not implemented yet
                };

                if should_trigger {
                    auto_behavior.last_triggered = Some(now);
                    // Execute actions (simplified implementation)
                    info!(
                        "Auto behavior triggered for avatar {}: {:?}",
                        avatar_id, auto_behavior.behavior_id
                    );
                }
            }
        }
    }

    async fn auto_trigger_processor(
        behaviors: Arc<RwLock<HashMap<Uuid, AvatarBehaviorState>>>,
        sender: mpsc::UnboundedSender<BehaviorCommand>,
    ) {
        let mut interval = interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            let avatar_ids: Vec<Uuid> = {
                let behaviors_guard = behaviors.read().await;
                behaviors_guard.keys().cloned().collect()
            };

            for avatar_id in avatar_ids {
                let command = BehaviorCommand::ProcessAutoTriggers { avatar_id };
                if sender.send(command).is_err() {
                    error!(
                        "Failed to send auto trigger command for avatar {}",
                        avatar_id
                    );
                }
            }
        }
    }

    fn create_auto_behavior_instances(
        &self,
        auto_behaviors: &[AutoBehavior],
    ) -> Vec<AutoBehaviorInstance> {
        auto_behaviors
            .iter()
            .map(|behavior| AutoBehaviorInstance {
                behavior_id: behavior.behavior_id,
                trigger: behavior.trigger_condition.clone(),
                actions: behavior.actions.clone(),
                last_triggered: None,
                cooldown: Duration::from_secs_f32(behavior.cooldown_seconds),
                enabled: behavior.enabled,
            })
            .collect()
    }
}

impl Default for BehaviorSystem {
    fn default() -> Self {
        Self::new()
    }
}
