use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{interval, Duration};
use tracing::info;
use uuid::Uuid;

use super::behavior_config::NpcBehaviorConfig;
use super::behaviors::{self, BehaviorAction, PatrolState, WanderState};
use super::movement_controller::MovementController;
use crate::udp::action_bridge::ActionBridge;
use crate::udp::server::AvatarMovementState;

pub struct SpeakerInfo {
    pub agent_id: Uuid,
    pub position: [f32; 3],
    pub timestamp: Instant,
}

pub struct NpcRuntimeState {
    pub home_position: [f32; 3],
    pub greeted_avatars: HashSet<Uuid>,
    pub wander_state: WanderState,
    pub patrol_state: PatrolState,
}

pub struct BehaviorEngine {
    avatar_states: Arc<parking_lot::RwLock<HashMap<Uuid, AvatarMovementState>>>,
    npc_configs: HashMap<Uuid, NpcBehaviorConfig>,
    npc_states: HashMap<Uuid, NpcRuntimeState>,
    action_bridge: ActionBridge,
    last_speaker: Arc<parking_lot::RwLock<Option<SpeakerInfo>>>,
    converse_state: Arc<parking_lot::RwLock<HashMap<Uuid, Instant>>>,
}

impl BehaviorEngine {
    pub fn new(
        avatar_states: Arc<parking_lot::RwLock<HashMap<Uuid, AvatarMovementState>>>,
        npc_configs: HashMap<Uuid, NpcBehaviorConfig>,
        action_bridge: ActionBridge,
        last_speaker: Arc<parking_lot::RwLock<Option<SpeakerInfo>>>,
        converse_state: Arc<parking_lot::RwLock<HashMap<Uuid, Instant>>>,
    ) -> Self {
        let npc_states: HashMap<Uuid, NpcRuntimeState> = {
            let states = avatar_states.read();
            npc_configs
                .keys()
                .map(|id| {
                    let home = states
                        .get(id)
                        .map(|s| s.position)
                        .unwrap_or([128.0, 128.0, 25.0]);
                    (
                        *id,
                        NpcRuntimeState {
                            home_position: home,
                            greeted_avatars: HashSet::new(),
                            wander_state: WanderState::default(),
                            patrol_state: PatrolState::default(),
                        },
                    )
                })
                .collect()
        };

        Self {
            avatar_states,
            npc_configs,
            npc_states,
            action_bridge,
            last_speaker,
            converse_state,
        }
    }

    pub async fn run(mut self) {
        let mut tick_interval = interval(Duration::from_secs(1));
        info!("[BEHAVIOR] NPC behavior engine started (1Hz)");

        loop {
            tick_interval.tick().await;
            self.tick().await;
        }
    }

    async fn tick(&mut self) {
        let dt = 1.0_f32;

        let (npc_data, viewer_avatars) = {
            let states = self.avatar_states.read();
            let npcs: Vec<(Uuid, [f32; 3])> = states
                .iter()
                .filter(|(_, s)| s.is_npc)
                .map(|(id, s)| (*id, s.position))
                .collect();
            let viewers: Vec<(Uuid, [f32; 3])> = states
                .iter()
                .filter(|(_, s)| !s.is_npc)
                .map(|(id, s)| (*id, s.position))
                .collect();
            (npcs, viewers)
        };

        let speaker_info: Option<(Uuid, [f32; 3], Instant)> = {
            let speaker = self.last_speaker.read();
            speaker
                .as_ref()
                .map(|s| (s.agent_id, s.position, s.timestamp))
        };

        let mut say_actions: Vec<(Uuid, String, [f32; 3], String)> = Vec::new();

        for (npc_id, npc_pos) in &npc_data {
            let config = match self.npc_configs.get(npc_id) {
                Some(c) => c,
                None => continue,
            };
            let runtime = match self.npc_states.get_mut(npc_id) {
                Some(r) => r,
                None => continue,
            };

            let npc_name = {
                let states = self.avatar_states.read();
                states
                    .get(npc_id)
                    .and_then(|s| s.npc_name.clone())
                    .unwrap_or_default()
            };

            if config.proximity_sleep_radius > 0.0 {
                let any_nearby = viewer_avatars.iter().any(|(_, apos)| {
                    let dx = apos[0] - npc_pos[0];
                    let dy = apos[1] - npc_pos[1];
                    let dist_sq = dx * dx + dy * dy;
                    dist_sq < config.proximity_sleep_radius * config.proximity_sleep_radius
                });
                if !any_nearby {
                    MovementController::apply_action(
                        &self.avatar_states,
                        *npc_id,
                        &BehaviorAction::Stop,
                    );
                    continue;
                }
            }

            if let Some(greet_action) = behaviors::tick_greet(
                *npc_pos,
                config.greet_radius,
                &config.greet_message,
                &viewer_avatars,
                &mut runtime.greeted_avatars,
            ) {
                if let BehaviorAction::Say(msg) = &greet_action {
                    say_actions.push((*npc_id, msg.clone(), *npc_pos, npc_name.clone()));
                }
            }

            if config.face_speaker {
                if let Some((_, speaker_pos, ts)) = &speaker_info {
                    if ts.elapsed().as_secs_f32() < 3.0 {
                        let dx = speaker_pos[0] - npc_pos[0];
                        let dy = speaker_pos[1] - npc_pos[1];
                        let dist_sq = dx * dx + dy * dy;
                        if dist_sq < 400.0 {
                            MovementController::apply_action(
                                &self.avatar_states,
                                *npc_id,
                                &BehaviorAction::FacePosition(*speaker_pos),
                            );
                        }
                    }
                }
            }

            let conversing = {
                let conv = self.converse_state.read();
                conv.get(npc_id)
                    .map(|t| t.elapsed().as_secs_f32() < 10.0)
                    .unwrap_or(false)
            };
            if conversing {
                MovementController::apply_action(
                    &self.avatar_states,
                    *npc_id,
                    &BehaviorAction::Stop,
                );
                continue;
            }

            let speed = config.speed_value();
            match config.behavior.as_str() {
                "wander" => {
                    let action = behaviors::tick_wander(
                        *npc_pos,
                        runtime.home_position,
                        config.wander_radius,
                        config.wander_pause,
                        config.arrival_threshold,
                        speed,
                        dt,
                        &mut runtime.wander_state,
                    );
                    MovementController::apply_action(&self.avatar_states, *npc_id, &action);
                }
                "patrol" => {
                    let actions = behaviors::tick_patrol(
                        *npc_pos,
                        &config.patrol_points,
                        config.patrol_pause,
                        config.patrol_loop,
                        &config.patrol_say,
                        config.arrival_threshold,
                        speed,
                        dt,
                        &mut runtime.patrol_state,
                    );
                    for action in &actions {
                        match action {
                            BehaviorAction::Say(msg) => {
                                say_actions.push((
                                    *npc_id,
                                    msg.clone(),
                                    *npc_pos,
                                    npc_name.clone(),
                                ));
                            }
                            _ => {
                                MovementController::apply_action(
                                    &self.avatar_states,
                                    *npc_id,
                                    action,
                                );
                            }
                        }
                    }
                }
                "follow" => {
                    if let Some((_, closest_pos)) = viewer_avatars.first() {
                        let action = behaviors::tick_follow(
                            *npc_pos,
                            *closest_pos,
                            config.follow_distance,
                            config.arrival_threshold,
                            speed,
                        );
                        MovementController::apply_action(&self.avatar_states, *npc_id, &action);
                    }
                }
                _ => {}
            }
        }

        for (npc_id, message, pos, name) in say_actions {
            if let Err(e) = self.action_bridge.say(npc_id, &name, &message, pos).await {
                tracing::warn!("[BEHAVIOR] Say error for {}: {}", name, e);
            }
        }
    }
}
