use crate::ai::skill_engine::*;

static P_AGENT_ID: ParamDef = ParamDef {
    name: "agent_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Agent UUID",
};
static P_ANIM_NAME: ParamDef = ParamDef {
    name: "anim_name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Animation name or UUID",
};
static P_NAME: ParamDef = ParamDef {
    name: "name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Name for the pose/sequence",
};
static P_MESH_ID: ParamDef = ParamDef {
    name: "mesh_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Mesh asset UUID",
};
static P_POSE_JSON: ParamDef = ParamDef {
    name: "pose_json",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Pose data as JSON (joint rotations)",
};
static P_WALK: ParamDef = ParamDef {
    name: "walk",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Walk animation override name",
};
static P_RUN: ParamDef = ParamDef {
    name: "run",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Run animation override name",
};
static P_STAND: ParamDef = ParamDef {
    name: "stand",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Stand animation override name",
};

pub static PLAY_ANIMATION: SkillDef = SkillDef {
    id: "play_animation",
    domain: SkillDomain::Animation,
    display_name: "Play Animation",
    description: "Play an animation on an agent",
    params: &[P_AGENT_ID, P_ANIM_NAME],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 209.8",
    tags: &["animation", "play", "avatar"],
    examples: &[],
};

pub static STOP_ANIMATION: SkillDef = SkillDef {
    id: "stop_animation",
    domain: SkillDomain::Animation,
    display_name: "Stop Animation",
    description: "Stop an animation playing on an agent",
    params: &[P_AGENT_ID, P_ANIM_NAME],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 209.8",
    tags: &["animation", "stop", "avatar"],
    examples: &[],
};

pub static CREATE_POSE: SkillDef = SkillDef {
    id: "create_pose",
    domain: SkillDomain::Animation,
    display_name: "Create Pose",
    description: "Create a named pose from joint rotation data",
    params: &[P_NAME, P_POSE_JSON],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 209.8",
    tags: &["animation", "pose", "creation"],
    examples: &[],
};

pub static BAKE_STATUE_POSE: SkillDef = SkillDef {
    id: "bake_statue_pose",
    domain: SkillDomain::Animation,
    display_name: "Bake Statue Pose",
    description: "Bake a pose into a mesh via Blender for static statue display",
    params: &[P_MESH_ID, P_POSE_JSON],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L6Verified,
    phase: "Phase 187",
    tags: &["animation", "pose", "statue", "blender"],
    examples: &[],
};

pub static CREATE_POSE_SEQUENCE: SkillDef = SkillDef {
    id: "create_pose_sequence",
    domain: SkillDomain::Animation,
    display_name: "Create Pose Sequence",
    description: "Create a timed sequence of poses for animation playback",
    params: &[P_NAME],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 209.8",
    tags: &["animation", "sequence", "poses"],
    examples: &[],
};

pub static APPLY_AO: SkillDef = SkillDef {
    id: "apply_ao",
    domain: SkillDomain::Animation,
    display_name: "Apply Animation Override",
    description: "Set animation overrides for walk, run, and stand",
    params: &[P_AGENT_ID, P_WALK, P_RUN, P_STAND],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 209.8",
    tags: &["animation", "ao", "override"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&PLAY_ANIMATION);
    registry.register(&STOP_ANIMATION);
    registry.register(&CREATE_POSE);
    registry.register(&BAKE_STATUE_POSE);
    registry.register(&CREATE_POSE_SEQUENCE);
    registry.register(&APPLY_AO);
}
