use rand::Rng;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum BehaviorAction {
    SetVelocityToward { target: [f32; 3], speed: f32 },
    Stop,
    FacePosition([f32; 3]),
    Say(String),
    Nothing,
}

#[derive(Debug, Clone)]
pub struct WanderState {
    pub target_point: Option<[f32; 3]>,
    pub pause_remaining: f32,
}

impl Default for WanderState {
    fn default() -> Self {
        Self {
            target_point: None,
            pause_remaining: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatrolState {
    pub waypoint_index: usize,
    pub pause_remaining: f32,
    pub moving_to_waypoint: bool,
}

impl Default for PatrolState {
    fn default() -> Self {
        Self {
            waypoint_index: 0,
            pause_remaining: 0.0,
            moving_to_waypoint: false,
        }
    }
}

pub fn tick_idle() -> BehaviorAction {
    BehaviorAction::Nothing
}

pub fn tick_wander(
    npc_pos: [f32; 3],
    home_pos: [f32; 3],
    radius: f32,
    pause_range: [f32; 2],
    arrival_threshold: f32,
    speed: f32,
    dt: f32,
    wander_state: &mut WanderState,
) -> BehaviorAction {
    if wander_state.pause_remaining > 0.0 {
        wander_state.pause_remaining -= dt;
        return BehaviorAction::Stop;
    }

    if let Some(target) = wander_state.target_point {
        let dx = target[0] - npc_pos[0];
        let dy = target[1] - npc_pos[1];
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < arrival_threshold {
            wander_state.target_point = None;
            let mut rng = rand::thread_rng();
            wander_state.pause_remaining = rng.gen_range(pause_range[0]..=pause_range[1]);
            return BehaviorAction::Stop;
        }

        return BehaviorAction::SetVelocityToward { target, speed };
    }

    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let dist = rng.gen_range(radius * 0.3..=radius);
    let target = [
        (home_pos[0] + angle.cos() * dist).clamp(1.0, 255.0),
        (home_pos[1] + angle.sin() * dist).clamp(1.0, 255.0),
        home_pos[2],
    ];
    wander_state.target_point = Some(target);
    BehaviorAction::SetVelocityToward { target, speed }
}

pub fn tick_patrol(
    npc_pos: [f32; 3],
    waypoints: &[[f32; 3]],
    patrol_pause: f32,
    patrol_loop: bool,
    patrol_say: &[String],
    arrival_threshold: f32,
    speed: f32,
    dt: f32,
    patrol_state: &mut PatrolState,
) -> Vec<BehaviorAction> {
    if waypoints.is_empty() {
        return vec![BehaviorAction::Nothing];
    }

    if patrol_state.pause_remaining > 0.0 {
        patrol_state.pause_remaining -= dt;
        return vec![BehaviorAction::Stop];
    }

    if !patrol_state.moving_to_waypoint {
        if patrol_state.waypoint_index >= waypoints.len() {
            if patrol_loop {
                patrol_state.waypoint_index = 0;
            } else {
                return vec![BehaviorAction::Stop];
            }
        }
        patrol_state.moving_to_waypoint = true;
    }

    let target = waypoints[patrol_state.waypoint_index];
    let dx = target[0] - npc_pos[0];
    let dy = target[1] - npc_pos[1];
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < arrival_threshold {
        patrol_state.moving_to_waypoint = false;
        patrol_state.pause_remaining = patrol_pause;

        let mut actions = vec![BehaviorAction::Stop];

        if !patrol_say.is_empty() {
            let say_idx = patrol_state.waypoint_index % patrol_say.len();
            actions.push(BehaviorAction::Say(patrol_say[say_idx].clone()));
        }

        patrol_state.waypoint_index += 1;
        return actions;
    }

    vec![BehaviorAction::SetVelocityToward { target, speed }]
}

pub fn tick_follow(
    npc_pos: [f32; 3],
    target_pos: [f32; 3],
    follow_distance: f32,
    arrival_threshold: f32,
    speed: f32,
) -> BehaviorAction {
    let dx = target_pos[0] - npc_pos[0];
    let dy = target_pos[1] - npc_pos[1];
    let dist = (dx * dx + dy * dy).sqrt();

    if dist > follow_distance + arrival_threshold {
        BehaviorAction::SetVelocityToward {
            target: target_pos,
            speed,
        }
    } else if dist < follow_distance - arrival_threshold {
        BehaviorAction::Stop
    } else {
        BehaviorAction::Stop
    }
}

pub fn tick_greet(
    npc_pos: [f32; 3],
    greet_radius: f32,
    greet_message: &str,
    nearby_avatars: &[(Uuid, [f32; 3])],
    greeted_set: &mut HashSet<Uuid>,
) -> Option<BehaviorAction> {
    if greet_message.is_empty() || greet_radius <= 0.0 {
        return None;
    }

    for (avatar_id, avatar_pos) in nearby_avatars {
        if greeted_set.contains(avatar_id) {
            continue;
        }
        let dx = avatar_pos[0] - npc_pos[0];
        let dy = avatar_pos[1] - npc_pos[1];
        let dz = avatar_pos[2] - npc_pos[2];
        let dist_sq = dx * dx + dy * dy + dz * dz;

        if dist_sq < greet_radius * greet_radius {
            greeted_set.insert(*avatar_id);
            return Some(BehaviorAction::Say(greet_message.to_string()));
        }
    }
    None
}

pub fn face_position(npc_pos: [f32; 3], target_pos: [f32; 3]) -> [f32; 4] {
    let dx = target_pos[0] - npc_pos[0];
    let dy = target_pos[1] - npc_pos[1];

    let yaw = dy.atan2(dx);
    let half = yaw * 0.5;
    [0.0, 0.0, half.sin(), half.cos()]
}
