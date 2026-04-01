use crate::ai::skill_engine::*;

static P_RECIPE: ParamDef = ParamDef {
    name: "recipe", param_type: ParamType::Enum(&["car", "bike", "plane", "vtol", "vessel", "starship", "lani"]),
    required: true, default_value: None, description: "Vehicle recipe type",
};
static P_POSITION: ParamDef = ParamDef {
    name: "position", param_type: ParamType::Vec3, required: true,
    default_value: None, description: "Build position [x, y, z]",
};
static P_TUNING: ParamDef = ParamDef {
    name: "tuning", param_type: ParamType::F32Map, required: false,
    default_value: None, description: "Tuning parameter overrides (e.g. MAX_SPEED, TURN_RATE)",
};
static P_ROOT_ID: ParamDef = ParamDef {
    name: "root_id", param_type: ParamType::U32, required: true,
    default_value: None, description: "Vehicle root prim local ID",
};

pub static BUILD_VEHICLE: SkillDef = SkillDef {
    id: "build_vehicle",
    domain: SkillDomain::Vehicles,
    display_name: "Build Vehicle",
    description: "Construct a complete vehicle from a recipe with scripts and HUD",
    params: &[P_RECIPE, P_POSITION, P_TUNING],
    returns: ReturnType::LocalId,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 193",
    tags: &["vehicle", "build", "recipe"],
    examples: &[SkillExample {
        description: "Build a sailboat at the marina dock",
        input: r#"{"recipe": "vessel", "position": [45, 128, 22], "tuning": {"FORWARD_POWER": 25}}"#,
        output: r#"{"message": "Vessel built", "local_id": 4201}"#,
    }],
};

pub static MODIFY_VEHICLE: SkillDef = SkillDef {
    id: "modify_vehicle",
    domain: SkillDomain::Vehicles,
    display_name: "Modify Vehicle",
    description: "Adjust tuning parameters on an existing vehicle",
    params: &[P_ROOT_ID, P_TUNING],
    returns: ReturnType::Success,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 193",
    tags: &["vehicle", "modify", "tuning"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&BUILD_VEHICLE);
    registry.register(&MODIFY_VEHICLE);
}
