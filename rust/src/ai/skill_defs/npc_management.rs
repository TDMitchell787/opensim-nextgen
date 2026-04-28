use crate::ai::skill_engine::*;

static P_NAME: ParamDef = ParamDef {
    name: "name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "NPC display name",
};
static P_POSITION: ParamDef = ParamDef {
    name: "position",
    param_type: ParamType::Vec3,
    required: true,
    default_value: None,
    description: "World position",
};
static P_APPEARANCE_ID: ParamDef = ParamDef {
    name: "appearance_id",
    param_type: ParamType::Uuid,
    required: false,
    default_value: None,
    description: "Appearance notecard/folder UUID",
};
static P_NPC_ID: ParamDef = ParamDef {
    name: "npc_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "NPC agent UUID",
};
static P_OUTFIT_FOLDER: ParamDef = ParamDef {
    name: "outfit_folder",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Outfit folder name or UUID",
};
static P_DESTINATION: ParamDef = ParamDef {
    name: "destination",
    param_type: ParamType::Vec3,
    required: true,
    default_value: None,
    description: "Walk target position",
};
static P_MESSAGE: ParamDef = ParamDef {
    name: "message",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Chat message",
};
static P_CHANNEL: ParamDef = ParamDef {
    name: "channel",
    param_type: ParamType::U32,
    required: false,
    default_value: Some("0"),
    description: "Chat channel",
};
static P_ANIMATION: ParamDef = ParamDef {
    name: "animation",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Animation name or UUID",
};
static P_ROLE: ParamDef = ParamDef {
    name: "role",
    param_type: ParamType::Enum(&["greeter", "guard", "vendor", "guide", "dancer", "musician"]),
    required: true,
    default_value: None,
    description: "NPC behavioral role",
};

pub static SPAWN_NPC: SkillDef = SkillDef {
    id: "spawn_npc",
    domain: SkillDomain::NpcManagement,
    display_name: "Spawn NPC",
    description: "Spawn a new NPC at a position with optional appearance",
    params: &[P_NAME, P_POSITION, P_APPEARANCE_ID],
    returns: ReturnType::ObjectData,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["npc", "spawn", "create"],
    examples: &[],
};

pub static DESPAWN_NPC: SkillDef = SkillDef {
    id: "despawn_npc",
    domain: SkillDomain::NpcManagement,
    display_name: "Despawn NPC",
    description: "Remove an NPC from the region",
    params: &[P_NPC_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["npc", "despawn", "remove"],
    examples: &[],
};

pub static SET_NPC_APPEARANCE: SkillDef = SkillDef {
    id: "set_npc_appearance",
    domain: SkillDomain::NpcManagement,
    display_name: "Set NPC Appearance",
    description: "Change an NPC's outfit from a folder",
    params: &[P_NPC_ID, P_OUTFIT_FOLDER],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["npc", "appearance", "outfit"],
    examples: &[],
};

pub static NPC_WALK_TO: SkillDef = SkillDef {
    id: "npc_walk_to",
    domain: SkillDomain::NpcManagement,
    display_name: "NPC Walk To",
    description: "Command an NPC to walk to a destination",
    params: &[P_NPC_ID, P_DESTINATION],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["npc", "movement", "walk"],
    examples: &[],
};

pub static NPC_SAY: SkillDef = SkillDef {
    id: "npc_say",
    domain: SkillDomain::NpcManagement,
    display_name: "NPC Say",
    description: "Make an NPC speak on a chat channel",
    params: &[P_NPC_ID, P_MESSAGE, P_CHANNEL],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["npc", "chat", "speak"],
    examples: &[],
};

pub static NPC_ANIMATE: SkillDef = SkillDef {
    id: "npc_animate",
    domain: SkillDomain::NpcManagement,
    display_name: "NPC Animate",
    description: "Play an animation on an NPC",
    params: &[P_NPC_ID, P_ANIMATION],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["npc", "animation", "play"],
    examples: &[],
};

pub static NPC_PATROL: SkillDef = SkillDef {
    id: "npc_patrol",
    domain: SkillDomain::NpcManagement,
    display_name: "NPC Patrol Route",
    description: "Set an NPC to patrol a series of waypoints in a loop",
    params: &[P_NPC_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["npc", "patrol", "waypoint"],
    examples: &[],
};

pub static ASSIGN_NPC_ROLE: SkillDef = SkillDef {
    id: "assign_npc_role",
    domain: SkillDomain::NpcManagement,
    display_name: "Assign NPC Role",
    description: "Assign a behavioral role to an NPC (greeter, guard, vendor, etc.)",
    params: &[P_NPC_ID, P_ROLE],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["npc", "role", "behavior"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&SPAWN_NPC);
    registry.register(&DESPAWN_NPC);
    registry.register(&SET_NPC_APPEARANCE);
    registry.register(&NPC_WALK_TO);
    registry.register(&NPC_SAY);
    registry.register(&NPC_ANIMATE);
    registry.register(&NPC_PATROL);
    registry.register(&ASSIGN_NPC_ROLE);
}
