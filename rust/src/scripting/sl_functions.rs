use std::collections::HashMap;
use uuid::Uuid;

use super::{
    lsl_types::{LSLRotation, LSLVector},
    LSLValue,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExperienceError {
    None,
    NotFound,
    NotPermitted,
    KeyNotFound,
    KeyExists,
    StoreFull,
    Throttled,
    InvalidExperience,
    DataQuotaExceeded,
    Unknown(i32),
}

impl ExperienceError {
    pub fn to_lsl_value(&self) -> i32 {
        match self {
            ExperienceError::None => 0,
            ExperienceError::NotFound => 1,
            ExperienceError::NotPermitted => 2,
            ExperienceError::KeyNotFound => 3,
            ExperienceError::KeyExists => 4,
            ExperienceError::StoreFull => 5,
            ExperienceError::Throttled => 6,
            ExperienceError::InvalidExperience => 7,
            ExperienceError::DataQuotaExceeded => 8,
            ExperienceError::Unknown(c) => *c,
        }
    }
}

pub const XP_ERROR_NONE: i32 = 0;
pub const XP_ERROR_NOT_FOUND: i32 = 1;
pub const XP_ERROR_NOT_PERMITTED: i32 = 2;
pub const XP_ERROR_KEY_NOT_FOUND: i32 = 3;
pub const XP_ERROR_KEY_EXISTS: i32 = 4;
pub const XP_ERROR_STORE_FULL: i32 = 5;
pub const XP_ERROR_THROTTLED: i32 = 6;
pub const XP_ERROR_INVALID_EXPERIENCE: i32 = 7;
pub const XP_ERROR_DATA_QUOTA_EXCEEDED: i32 = 8;

pub const CHARACTER_CMD_STOP: i32 = 0;
pub const CHARACTER_CMD_JUMP: i32 = 1;
pub const CHARACTER_CMD_SMOOTH_STOP: i32 = 2;

pub const CHARACTER_TYPE_NONE: i32 = 0;
pub const CHARACTER_TYPE_A: i32 = 1;
pub const CHARACTER_TYPE_B: i32 = 2;
pub const CHARACTER_TYPE_C: i32 = 3;
pub const CHARACTER_TYPE_D: i32 = 4;

pub const CHARACTER_LENGTH: i32 = 0;
pub const CHARACTER_RADIUS: i32 = 1;
pub const CHARACTER_SPEED: i32 = 2;
pub const CHARACTER_DESIRED_SPEED: i32 = 3;
pub const CHARACTER_DESIRED_TURN_SPEED: i32 = 4;
pub const CHARACTER_AVOIDANCE_MODE: i32 = 5;
pub const CHARACTER_TYPE: i32 = 6;
pub const CHARACTER_MAX_ACCEL: i32 = 7;
pub const CHARACTER_MAX_DECEL: i32 = 8;
pub const CHARACTER_MAX_TURN_RADIUS: i32 = 9;
pub const CHARACTER_ACCOUNT_FOR_SKIPPED_FRAMES: i32 = 10;
pub const CHARACTER_STAY_WITHIN_PARCEL: i32 = 11;

pub const PURSUIT_OFFSET: i32 = 1;
pub const REQUIRE_LINE_OF_SIGHT: i32 = 2;
pub const PURSUIT_FUZZ_FACTOR: i32 = 3;
pub const PURSUIT_GOAL_TOLERANCE: i32 = 4;
pub const PURSUIT_INTERCEPT: i32 = 5;

pub const WANDER_PAUSE_AT_WAYPOINTS: i32 = 0;

pub const PU_SLOWDOWN_DISTANCE_REACHED: i32 = 0x00;
pub const PU_GOAL_REACHED: i32 = 0x01;
pub const PU_FAILURE_INVALID_START: i32 = 0x02;
pub const PU_FAILURE_INVALID_GOAL: i32 = 0x03;
pub const PU_FAILURE_UNREACHABLE: i32 = 0x04;
pub const PU_FAILURE_TARGET_GONE: i32 = 0x05;
pub const PU_FAILURE_NO_VALID_DESTINATION: i32 = 0x06;
pub const PU_EVADE_HIDDEN: i32 = 0x07;
pub const PU_EVADE_SPOTTED: i32 = 0x08;
pub const PU_FAILURE_NO_NAVMESH: i32 = 0x09;
pub const PU_FAILURE_DYNAMIC_PATHFINDING_DISABLED: i32 = 0x0A;
pub const PU_FAILURE_PARCEL_UNREACHABLE: i32 = 0x0B;
pub const PU_FAILURE_OTHER: i32 = 0xF4240;

pub const GCNP_RADIUS: i32 = 0;
pub const GCNP_STATIC: i32 = 1;

pub struct SLFunctions {
    experience_kvp: HashMap<(Uuid, String), String>,
    pathfinding_characters: HashMap<Uuid, PathfindingCharacter>,
}

struct PathfindingCharacter {
    object_id: Uuid,
    character_type: i32,
    length: f32,
    radius: f32,
    speed: f32,
    max_accel: f32,
    max_decel: f32,
}

impl SLFunctions {
    pub fn new() -> Self {
        Self {
            experience_kvp: HashMap::new(),
            pathfinding_characters: HashMap::new(),
        }
    }

    pub fn execute_function(
        &mut self,
        name: &str,
        args: &[LSLValue],
        _script_id: Uuid,
        _object_id: Uuid,
        experience_id: Option<Uuid>,
    ) -> LSLValue {
        match name {
            "llAgentInExperience" => self.ll_agent_in_experience(args, experience_id),
            "llCreateKeyValue" => self.ll_create_key_value(args, experience_id),
            "llReadKeyValue" => self.ll_read_key_value(args, experience_id),
            "llUpdateKeyValue" => self.ll_update_key_value(args, experience_id),
            "llDeleteKeyValue" => self.ll_delete_key_value(args, experience_id),
            "llKeysKeyValue" => self.ll_keys_key_value(args, experience_id),
            "llDataSizeKeyValue" => self.ll_data_size_key_value(args, experience_id),
            "llGetExperienceDetails" => self.ll_get_experience_details(args),
            "llRequestExperiencePermissions" => self.ll_request_experience_permissions(args),

            "llCreateCharacter" => self.ll_create_character(args, _object_id),
            "llDeleteCharacter" => self.ll_delete_character(_object_id),
            "llUpdateCharacter" => self.ll_update_character(args, _object_id),
            "llExecCharacterCmd" => self.ll_exec_character_cmd(args, _object_id),
            "llGetStaticPath" => self.ll_get_static_path(args),
            "llNavigateTo" => self.ll_navigate_to(args, _object_id),
            "llWanderWithin" => self.ll_wander_within(args, _object_id),
            "llEvade" => self.ll_evade(args, _object_id),
            "llPursue" => self.ll_pursue(args, _object_id),
            "llFleeFrom" => self.ll_flee_from(args, _object_id),
            "llGetClosestNavPoint" => self.ll_get_closest_nav_point(args),
            "llPatrolPoints" => self.ll_patrol_points(args, _object_id),

            "llSetPrimMaterialParams" => self.ll_set_prim_material_params(args),
            "llGetPrimMaterialParams" => self.ll_get_prim_material_params(args),

            "llSetAgentEnvironment" => self.ll_set_agent_environment(args),
            "llReplaceAgentEnvironment" => self.ll_replace_agent_environment(args),
            "llGetEnvironment" => self.ll_get_environment(args),
            "llSetEnvironment" => self.ll_set_environment(args),

            "llReturnObjectsByID" => self.ll_return_objects_by_id(args),
            "llReturnObjectsByOwner" => self.ll_return_objects_by_owner(args),
            "llGetExperienceErrorMessage" => self.ll_get_experience_error_message(args),
            "llSetContentType" => self.ll_set_content_type(args),
            "llGenerateKey" => self.ll_generate_key(),
            "llTeleportAgent" => self.ll_teleport_agent(args),
            "llManageEstateAccess" => self.ll_manage_estate_access(args),

            _ => LSLValue::Integer(0),
        }
    }

    fn ll_agent_in_experience(&self, args: &[LSLValue], experience_id: Option<Uuid>) -> LSLValue {
        if args.is_empty() || experience_id.is_none() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(1)
    }

    fn ll_create_key_value(&mut self, args: &[LSLValue], experience_id: Option<Uuid>) -> LSLValue {
        let xp_id = match experience_id {
            Some(id) => id,
            None => return LSLValue::Key(Uuid::nil()),
        };
        if args.len() < 2 {
            return LSLValue::Key(Uuid::nil());
        }
        let key = match &args[0] {
            LSLValue::String(s) => s.clone(),
            _ => return LSLValue::Key(Uuid::nil()),
        };
        let value = match &args[1] {
            LSLValue::String(s) => s.clone(),
            _ => return LSLValue::Key(Uuid::nil()),
        };

        let kvp_key = (xp_id, key);
        if self.experience_kvp.contains_key(&kvp_key) {
            return LSLValue::Key(Uuid::nil());
        }
        self.experience_kvp.insert(kvp_key, value);
        let request_id = Uuid::new_v4();
        LSLValue::Key(request_id)
    }

    fn ll_read_key_value(&self, args: &[LSLValue], experience_id: Option<Uuid>) -> LSLValue {
        let xp_id = match experience_id {
            Some(id) => id,
            None => return LSLValue::Key(Uuid::nil()),
        };
        if args.is_empty() {
            return LSLValue::Key(Uuid::nil());
        }
        let key = match &args[0] {
            LSLValue::String(s) => s.clone(),
            _ => return LSLValue::Key(Uuid::nil()),
        };

        let _value = self.experience_kvp.get(&(xp_id, key));
        let request_id = Uuid::new_v4();
        LSLValue::Key(request_id)
    }

    fn ll_update_key_value(&mut self, args: &[LSLValue], experience_id: Option<Uuid>) -> LSLValue {
        let xp_id = match experience_id {
            Some(id) => id,
            None => return LSLValue::Key(Uuid::nil()),
        };
        if args.len() < 2 {
            return LSLValue::Key(Uuid::nil());
        }
        let key = match &args[0] {
            LSLValue::String(s) => s.clone(),
            _ => return LSLValue::Key(Uuid::nil()),
        };
        let value = match &args[1] {
            LSLValue::String(s) => s.clone(),
            _ => return LSLValue::Key(Uuid::nil()),
        };

        let kvp_key = (xp_id, key);
        if !self.experience_kvp.contains_key(&kvp_key) {
            return LSLValue::Key(Uuid::nil());
        }
        self.experience_kvp.insert(kvp_key, value);
        let request_id = Uuid::new_v4();
        LSLValue::Key(request_id)
    }

    fn ll_delete_key_value(&mut self, args: &[LSLValue], experience_id: Option<Uuid>) -> LSLValue {
        let xp_id = match experience_id {
            Some(id) => id,
            None => return LSLValue::Key(Uuid::nil()),
        };
        if args.is_empty() {
            return LSLValue::Key(Uuid::nil());
        }
        let key = match &args[0] {
            LSLValue::String(s) => s.clone(),
            _ => return LSLValue::Key(Uuid::nil()),
        };

        self.experience_kvp.remove(&(xp_id, key));
        let request_id = Uuid::new_v4();
        LSLValue::Key(request_id)
    }

    fn ll_keys_key_value(&self, _args: &[LSLValue], experience_id: Option<Uuid>) -> LSLValue {
        let _xp_id = match experience_id {
            Some(id) => id,
            None => return LSLValue::Key(Uuid::nil()),
        };
        let request_id = Uuid::new_v4();
        LSLValue::Key(request_id)
    }

    fn ll_data_size_key_value(&self, _args: &[LSLValue], experience_id: Option<Uuid>) -> LSLValue {
        let xp_id = match experience_id {
            Some(id) => id,
            None => return LSLValue::Key(Uuid::nil()),
        };
        let total: usize = self
            .experience_kvp
            .iter()
            .filter(|((id, _), _)| *id == xp_id)
            .map(|((_, k), v)| k.len() + v.len())
            .sum();
        LSLValue::Integer(total as i32)
    }

    fn ll_get_experience_details(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::List(vec![]);
        }
        let _experience_id = match &args[0] {
            LSLValue::Key(k) => *k,
            LSLValue::String(s) => match Uuid::parse_str(s) {
                Ok(u) => u,
                Err(_) => return LSLValue::List(vec![]),
            },
            _ => return LSLValue::List(vec![]),
        };

        LSLValue::List(vec![
            LSLValue::String("Experience".to_string()),
            LSLValue::Key(Uuid::nil()),
            LSLValue::String("".to_string()),
            LSLValue::Integer(0),
            LSLValue::String("".to_string()),
            LSLValue::String("".to_string()),
            LSLValue::Key(Uuid::nil()),
            LSLValue::Key(Uuid::nil()),
            LSLValue::String("".to_string()),
        ])
    }

    fn ll_request_experience_permissions(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_create_character(&mut self, args: &[LSLValue], object_id: Uuid) -> LSLValue {
        let mut character = PathfindingCharacter {
            object_id,
            character_type: CHARACTER_TYPE_NONE,
            length: 1.0,
            radius: 0.25,
            speed: 1.0,
            max_accel: 1.0,
            max_decel: 1.0,
        };

        if let Some(LSLValue::List(params)) = args.first() {
            let mut i = 0;
            while i + 1 < params.len() {
                let param_id = match &params[i] {
                    LSLValue::Integer(n) => *n,
                    _ => {
                        i += 2;
                        continue;
                    }
                };
                match param_id {
                    0 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.length = *v as f32;
                        }
                    }
                    1 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.radius = *v as f32;
                        }
                    }
                    2 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.speed = *v as f32;
                        }
                    }
                    6 => {
                        if let LSLValue::Integer(v) = &params[i + 1] {
                            character.character_type = *v;
                        }
                    }
                    7 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.max_accel = *v as f32;
                        }
                    }
                    8 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.max_decel = *v as f32;
                        }
                    }
                    _ => {}
                }
                i += 2;
            }
        }

        self.pathfinding_characters.insert(object_id, character);
        LSLValue::Integer(0)
    }

    fn ll_delete_character(&mut self, object_id: Uuid) -> LSLValue {
        self.pathfinding_characters.remove(&object_id);
        LSLValue::Integer(0)
    }

    fn ll_update_character(&mut self, args: &[LSLValue], object_id: Uuid) -> LSLValue {
        let character = match self.pathfinding_characters.get_mut(&object_id) {
            Some(c) => c,
            None => return LSLValue::Integer(-1),
        };

        if let Some(LSLValue::List(params)) = args.first() {
            let mut i = 0;
            while i + 1 < params.len() {
                let param_id = match &params[i] {
                    LSLValue::Integer(n) => *n,
                    _ => {
                        i += 2;
                        continue;
                    }
                };
                match param_id {
                    0 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.length = *v as f32;
                        }
                    }
                    1 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.radius = *v as f32;
                        }
                    }
                    2 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.speed = *v as f32;
                        }
                    }
                    7 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.max_accel = *v as f32;
                        }
                    }
                    8 => {
                        if let LSLValue::Float(v) = &params[i + 1] {
                            character.max_decel = *v as f32;
                        }
                    }
                    _ => {}
                }
                i += 2;
            }
        }

        LSLValue::Integer(0)
    }

    fn ll_exec_character_cmd(&self, args: &[LSLValue], object_id: Uuid) -> LSLValue {
        if !self.pathfinding_characters.contains_key(&object_id) {
            return LSLValue::Integer(-1);
        }
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_get_static_path(&self, args: &[LSLValue]) -> LSLValue {
        if args.len() < 3 {
            return LSLValue::List(vec![]);
        }
        let _start = match &args[0] {
            LSLValue::Vector(v) => v.clone(),
            _ => return LSLValue::List(vec![]),
        };
        let _end = match &args[1] {
            LSLValue::Vector(v) => v.clone(),
            _ => return LSLValue::List(vec![]),
        };
        LSLValue::List(vec![])
    }

    fn ll_navigate_to(&self, args: &[LSLValue], object_id: Uuid) -> LSLValue {
        if !self.pathfinding_characters.contains_key(&object_id) {
            return LSLValue::Integer(-1);
        }
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_wander_within(&self, args: &[LSLValue], object_id: Uuid) -> LSLValue {
        if !self.pathfinding_characters.contains_key(&object_id) {
            return LSLValue::Integer(-1);
        }
        if args.len() < 2 {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_evade(&self, args: &[LSLValue], object_id: Uuid) -> LSLValue {
        if !self.pathfinding_characters.contains_key(&object_id) {
            return LSLValue::Integer(-1);
        }
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_pursue(&self, args: &[LSLValue], object_id: Uuid) -> LSLValue {
        if !self.pathfinding_characters.contains_key(&object_id) {
            return LSLValue::Integer(-1);
        }
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_flee_from(&self, args: &[LSLValue], object_id: Uuid) -> LSLValue {
        if !self.pathfinding_characters.contains_key(&object_id) {
            return LSLValue::Integer(-1);
        }
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_get_closest_nav_point(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::List(vec![]);
        }
        let point = match &args[0] {
            LSLValue::Vector(v) => v.clone(),
            _ => return LSLValue::List(vec![]),
        };
        LSLValue::List(vec![LSLValue::Vector(point)])
    }

    fn ll_patrol_points(&self, args: &[LSLValue], object_id: Uuid) -> LSLValue {
        if !self.pathfinding_characters.contains_key(&object_id) {
            return LSLValue::Integer(-1);
        }
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_set_prim_material_params(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_get_prim_material_params(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::List(vec![]);
        }
        let _face = match &args[0] {
            LSLValue::Integer(n) => *n,
            _ => return LSLValue::List(vec![]),
        };
        LSLValue::List(vec![
            LSLValue::Key(Uuid::nil()),
            LSLValue::Vector(LSLVector::new(1.0, 1.0, 0.0)),
            LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)),
            LSLValue::Float(0.0),
            LSLValue::Key(Uuid::nil()),
            LSLValue::Vector(LSLVector::new(1.0, 1.0, 0.0)),
            LSLValue::Vector(LSLVector::new(0.0, 0.0, 0.0)),
            LSLValue::Float(0.0),
        ])
    }

    fn ll_set_agent_environment(&self, args: &[LSLValue]) -> LSLValue {
        if args.len() < 2 {
            return LSLValue::Integer(-1);
        }
        LSLValue::Integer(0)
    }

    fn ll_replace_agent_environment(&self, args: &[LSLValue]) -> LSLValue {
        if args.len() < 2 {
            return LSLValue::Integer(-1);
        }
        LSLValue::Integer(0)
    }

    fn ll_get_environment(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::List(vec![]);
        }
        LSLValue::List(vec![])
    }

    fn ll_set_environment(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::Integer(-1);
        }
        LSLValue::Integer(0)
    }

    fn ll_return_objects_by_id(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        let count = match &args[0] {
            LSLValue::List(l) => l.len() as i32,
            _ => 0,
        };
        LSLValue::Integer(count)
    }

    fn ll_return_objects_by_owner(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_get_experience_error_message(&self, args: &[LSLValue]) -> LSLValue {
        if args.is_empty() {
            return LSLValue::String("Unknown error".to_string());
        }
        let error_code = match &args[0] {
            LSLValue::Integer(n) => *n,
            _ => return LSLValue::String("Unknown error".to_string()),
        };
        let msg = match error_code {
            0 => "no errors",
            1 => "not found",
            2 => "not permitted",
            3 => "key not found",
            4 => "key already exists",
            5 => "store is full",
            6 => "throttled",
            7 => "invalid experience",
            8 => "data quota exceeded",
            _ => "unknown error",
        };
        LSLValue::String(msg.to_string())
    }

    fn ll_set_content_type(&self, args: &[LSLValue]) -> LSLValue {
        if args.len() < 2 {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_generate_key(&self) -> LSLValue {
        LSLValue::Key(Uuid::new_v4())
    }

    fn ll_teleport_agent(&self, args: &[LSLValue]) -> LSLValue {
        if args.len() < 3 {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    fn ll_manage_estate_access(&self, args: &[LSLValue]) -> LSLValue {
        if args.len() < 2 {
            return LSLValue::Integer(0);
        }
        LSLValue::Integer(0)
    }

    pub fn has_character(&self, object_id: Uuid) -> bool {
        self.pathfinding_characters.contains_key(&object_id)
    }

    pub fn experience_key_count(&self, experience_id: Uuid) -> usize {
        self.experience_kvp
            .iter()
            .filter(|((id, _), _)| *id == experience_id)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_kvp_create_read() {
        let mut sl = SLFunctions::new();
        let xp_id = Uuid::new_v4();

        let result = sl.ll_create_key_value(
            &[
                LSLValue::String("test_key".to_string()),
                LSLValue::String("test_value".to_string()),
            ],
            Some(xp_id),
        );
        assert!(matches!(result, LSLValue::Key(k) if k != Uuid::nil()));

        assert_eq!(sl.experience_key_count(xp_id), 1);
    }

    #[test]
    fn test_experience_kvp_duplicate_key() {
        let mut sl = SLFunctions::new();
        let xp_id = Uuid::new_v4();

        sl.ll_create_key_value(
            &[
                LSLValue::String("dup".to_string()),
                LSLValue::String("v1".to_string()),
            ],
            Some(xp_id),
        );
        let result = sl.ll_create_key_value(
            &[
                LSLValue::String("dup".to_string()),
                LSLValue::String("v2".to_string()),
            ],
            Some(xp_id),
        );
        assert!(matches!(result, LSLValue::Key(k) if k == Uuid::nil()));
        assert_eq!(sl.experience_key_count(xp_id), 1);
    }

    #[test]
    fn test_pathfinding_create_delete() {
        let mut sl = SLFunctions::new();
        let obj_id = Uuid::new_v4();

        sl.ll_create_character(
            &[LSLValue::List(vec![
                LSLValue::Integer(CHARACTER_LENGTH),
                LSLValue::Float(2.0),
                LSLValue::Integer(CHARACTER_RADIUS),
                LSLValue::Float(0.5),
            ])],
            obj_id,
        );
        assert!(sl.has_character(obj_id));

        sl.ll_delete_character(obj_id);
        assert!(!sl.has_character(obj_id));
    }

    #[test]
    fn test_generate_key() {
        let sl = SLFunctions::new();
        let k1 = sl.ll_generate_key();
        let k2 = sl.ll_generate_key();
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_experience_error_messages() {
        let sl = SLFunctions::new();
        let msg = sl.ll_get_experience_error_message(&[LSLValue::Integer(0)]);
        assert_eq!(msg, LSLValue::String("no errors".to_string()));

        let msg = sl.ll_get_experience_error_message(&[LSLValue::Integer(3)]);
        assert_eq!(msg, LSLValue::String("key not found".to_string()));
    }

    #[test]
    fn test_get_closest_nav_point() {
        let sl = SLFunctions::new();
        let result =
            sl.ll_get_closest_nav_point(&[LSLValue::Vector(LSLVector::new(128.0, 128.0, 25.0))]);
        match result {
            LSLValue::List(l) => assert_eq!(l.len(), 1),
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_pbr_material_params() {
        let sl = SLFunctions::new();
        let result = sl.ll_get_prim_material_params(&[LSLValue::Integer(0)]);
        match result {
            LSLValue::List(l) => assert_eq!(l.len(), 8),
            _ => panic!("Expected list"),
        }
    }
}
