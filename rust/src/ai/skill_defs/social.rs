use crate::ai::skill_engine::*;

static P_AGENT_ID: ParamDef = ParamDef {
    name: "agent_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Agent UUID",
};
static P_MESSAGE: ParamDef = ParamDef {
    name: "message",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Message text",
};
static P_NAME: ParamDef = ParamDef {
    name: "name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Name",
};
static P_STYLE: ParamDef = ParamDef {
    name: "style",
    param_type: ParamType::Enum(&["formal", "casual", "warm"]),
    required: false,
    default_value: Some("warm"),
    description: "Greeting style",
};
static P_CHANNEL: ParamDef = ParamDef {
    name: "channel",
    param_type: ParamType::U32,
    required: false,
    default_value: Some("0"),
    description: "Chat channel (0 = local)",
};
static P_TIME: ParamDef = ParamDef {
    name: "time",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Event time (ISO 8601)",
};
static P_LOCATION: ParamDef = ParamDef {
    name: "location",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Event location description",
};
static P_DESCRIPTION: ParamDef = ParamDef {
    name: "description",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Description",
};
static P_GROUP_ID: ParamDef = ParamDef {
    name: "group_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Group UUID",
};
static P_SUBJECT: ParamDef = ParamDef {
    name: "subject",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Notice subject",
};
static P_POSITION: ParamDef = ParamDef {
    name: "position",
    param_type: ParamType::Vec3,
    required: true,
    default_value: None,
    description: "World position",
};
static P_GREETING: ParamDef = ParamDef {
    name: "greeting",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Greeting message for arriving avatars",
};

pub static GREET_AGENT: SkillDef = SkillDef {
    id: "greet_agent",
    domain: SkillDomain::Social,
    display_name: "Greet Agent",
    description: "Send a personalized greeting to an agent",
    params: &[P_AGENT_ID, P_STYLE],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["social", "greet", "welcome"],
    examples: &[],
};

pub static ANNOUNCE: SkillDef = SkillDef {
    id: "announce",
    domain: SkillDomain::Social,
    display_name: "Region Announcement",
    description: "Broadcast an announcement to all agents in the region",
    params: &[P_MESSAGE, P_CHANNEL],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["social", "announce", "broadcast"],
    examples: &[],
};

pub static CREATE_EVENT: SkillDef = SkillDef {
    id: "create_event",
    domain: SkillDomain::Social,
    display_name: "Create Event",
    description: "Create a scheduled event with time and location",
    params: &[P_NAME, P_TIME, P_LOCATION, P_DESCRIPTION],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["social", "event", "schedule"],
    examples: &[],
};

pub static INVITE_TO_GROUP: SkillDef = SkillDef {
    id: "invite_to_group",
    domain: SkillDomain::Social,
    display_name: "Invite to Group",
    description: "Invite an agent to a group",
    params: &[P_AGENT_ID, P_GROUP_ID],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["social", "group", "invite"],
    examples: &[],
};

pub static SEND_NOTICE: SkillDef = SkillDef {
    id: "send_notice",
    domain: SkillDomain::Social,
    display_name: "Send Group Notice",
    description: "Send a notice to all members of a group",
    params: &[P_GROUP_ID, P_SUBJECT, P_MESSAGE],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["social", "group", "notice"],
    examples: &[],
};

pub static SPAWN_GREETER_NPC: SkillDef = SkillDef {
    id: "spawn_greeter_npc",
    domain: SkillDomain::Social,
    display_name: "Spawn Greeter NPC",
    description: "Spawn an NPC that greets arriving avatars",
    params: &[P_POSITION, P_NAME, P_GREETING],
    returns: ReturnType::LocalId,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["social", "npc", "greeter"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&GREET_AGENT);
    registry.register(&ANNOUNCE);
    registry.register(&CREATE_EVENT);
    registry.register(&INVITE_TO_GROUP);
    registry.register(&SEND_NOTICE);
    registry.register(&SPAWN_GREETER_NPC);
}
