use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::behaviors::BehaviorAction;
use crate::udp::server::AvatarMovementState;

pub struct MovementController;

impl MovementController {
    pub fn apply_action(
        avatar_states: &Arc<parking_lot::RwLock<HashMap<Uuid, AvatarMovementState>>>,
        npc_id: Uuid,
        action: &BehaviorAction,
    ) {
        match action {
            BehaviorAction::SetVelocityToward { target, speed } => {
                let mut states = avatar_states.write();
                if let Some(state) = states.get_mut(&npc_id) {
                    let dx = target[0] - state.position[0];
                    let dy = target[1] - state.position[1];
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist > 0.01 {
                        let dir_x = dx / dist;
                        let dir_y = dy / dist;
                        state.target_velocity = [dir_x * speed, dir_y * speed, 0.0];
                        state.rotation = super::behaviors::face_position(
                            state.position,
                            *target,
                        );
                    }
                }
            }
            BehaviorAction::Stop => {
                let mut states = avatar_states.write();
                if let Some(state) = states.get_mut(&npc_id) {
                    state.target_velocity = [0.0, 0.0, 0.0];
                }
            }
            BehaviorAction::FacePosition(target) => {
                let mut states = avatar_states.write();
                if let Some(state) = states.get_mut(&npc_id) {
                    state.rotation = super::behaviors::face_position(
                        state.position,
                        *target,
                    );
                }
            }
            BehaviorAction::Say(_) | BehaviorAction::Nothing => {}
        }
    }
}
