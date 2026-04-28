use crate::ai::skill_engine::*;

static P_REGION_NAME: ParamDef = ParamDef {
    name: "region_name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Region name",
};
static P_AGENT_ID: ParamDef = ParamDef {
    name: "agent_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Agent UUID",
};
static P_DELAY: ParamDef = ParamDef {
    name: "delay_seconds",
    param_type: ParamType::U32,
    required: false,
    default_value: Some("30"),
    description: "Delay before action",
};
static P_PARCEL_ID: ParamDef = ParamDef {
    name: "parcel_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Parcel UUID",
};
static P_ACCESS_TYPE: ParamDef = ParamDef {
    name: "access_type",
    param_type: ParamType::Enum(&["public", "group", "ban", "allow"]),
    required: true,
    default_value: None,
    description: "Access list type",
};
static P_MEDIA_URL: ParamDef = ParamDef {
    name: "media_url",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Media URL",
};
static P_MEDIA_TYPE: ParamDef = ParamDef {
    name: "media_type",
    param_type: ParamType::String,
    required: false,
    default_value: Some("text/html"),
    description: "Media MIME type",
};
static P_MUSIC_URL: ParamDef = ParamDef {
    name: "music_url",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Streaming music URL",
};
static P_DURATION: ParamDef = ParamDef {
    name: "duration",
    param_type: ParamType::U32,
    required: false,
    default_value: None,
    description: "Duration in seconds (None = permanent)",
};
static P_REASON: ParamDef = ParamDef {
    name: "reason",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Reason for action",
};
static P_SUN_HOUR: ParamDef = ParamDef {
    name: "sun_hour",
    param_type: ParamType::F32,
    required: true,
    default_value: None,
    description: "Sun position 0-24 hours",
};
static P_HEIGHT: ParamDef = ParamDef {
    name: "height",
    param_type: ParamType::F32,
    required: true,
    default_value: None,
    description: "Water height in meters",
};

pub static RESTART_REGION: SkillDef = SkillDef {
    id: "restart_region",
    domain: SkillDomain::Estate,
    display_name: "Restart Region",
    description: "Restart a region with optional delay for user warning",
    params: &[P_REGION_NAME, P_DELAY],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 203.6",
    tags: &["estate", "region", "restart", "admin"],
    examples: &[],
};

pub static SET_PARCEL_ACCESS: SkillDef = SkillDef {
    id: "set_parcel_access",
    domain: SkillDomain::Estate,
    display_name: "Set Parcel Access",
    description: "Modify parcel access list (allow, ban, group-only)",
    params: &[P_PARCEL_ID, P_ACCESS_TYPE, P_AGENT_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 203.6",
    tags: &["estate", "parcel", "access", "security"],
    examples: &[],
};

pub static SET_PARCEL_MEDIA: SkillDef = SkillDef {
    id: "set_parcel_media",
    domain: SkillDomain::Estate,
    display_name: "Set Parcel Media",
    description: "Set the media URL for a parcel",
    params: &[P_PARCEL_ID, P_MEDIA_URL, P_MEDIA_TYPE],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 203.6",
    tags: &["estate", "parcel", "media"],
    examples: &[],
};

pub static SET_PARCEL_MUSIC: SkillDef = SkillDef {
    id: "set_parcel_music",
    domain: SkillDomain::Estate,
    display_name: "Set Parcel Music",
    description: "Set the streaming music URL for a parcel",
    params: &[P_PARCEL_ID, P_MUSIC_URL],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 203.6",
    tags: &["estate", "parcel", "music", "audio"],
    examples: &[],
};

pub static BAN_AGENT: SkillDef = SkillDef {
    id: "ban_agent",
    domain: SkillDomain::Estate,
    display_name: "Ban Agent",
    description: "Ban an agent from the estate",
    params: &[P_AGENT_ID, P_DURATION, P_REASON],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 203.6",
    tags: &["estate", "ban", "security", "admin"],
    examples: &[],
};

pub static UNBAN_AGENT: SkillDef = SkillDef {
    id: "unban_agent",
    domain: SkillDomain::Estate,
    display_name: "Unban Agent",
    description: "Remove an agent from the estate ban list",
    params: &[P_AGENT_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 203.6",
    tags: &["estate", "unban", "security", "admin"],
    examples: &[],
};

pub static SET_REGION_FLAGS: SkillDef = SkillDef {
    id: "set_region_flags",
    domain: SkillDomain::Estate,
    display_name: "Set Region Flags",
    description: "Toggle region flags (fly, build, script, push, damage)",
    params: &[P_REGION_NAME],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 203.6",
    tags: &["estate", "region", "flags", "admin"],
    examples: &[],
};

pub static SET_SUN_POSITION: SkillDef = SkillDef {
    id: "set_sun_position",
    domain: SkillDomain::Estate,
    display_name: "Set Sun Position",
    description: "Set the fixed sun position for a region",
    params: &[P_REGION_NAME, P_SUN_HOUR],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 203.6",
    tags: &["estate", "environment", "sun", "lighting"],
    examples: &[],
};

pub static SET_WATER_HEIGHT: SkillDef = SkillDef {
    id: "set_water_height",
    domain: SkillDomain::Estate,
    display_name: "Set Water Height",
    description: "Set the water level for a region",
    params: &[P_REGION_NAME, P_HEIGHT],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L5Integrated,
    phase: "Phase 203.6",
    tags: &["estate", "environment", "water"],
    examples: &[],
};

pub static REGION_STATUS: SkillDef = SkillDef {
    id: "region_status",
    domain: SkillDomain::Estate,
    display_name: "Get Region Status",
    description: "Get current status of a region (agents, uptime, flags)",
    params: &[P_REGION_NAME],
    returns: ReturnType::ObjectData,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["estate", "region", "status", "monitoring"],
    examples: &[],
};

pub static LIST_AGENTS: SkillDef = SkillDef {
    id: "list_agents",
    domain: SkillDomain::Estate,
    display_name: "List Agents in Region",
    description: "List all agents currently in a region",
    params: &[P_REGION_NAME],
    returns: ReturnType::ObjectData,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["estate", "agents", "query"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&RESTART_REGION);
    registry.register(&SET_PARCEL_ACCESS);
    registry.register(&SET_PARCEL_MEDIA);
    registry.register(&SET_PARCEL_MUSIC);
    registry.register(&BAN_AGENT);
    registry.register(&UNBAN_AGENT);
    registry.register(&SET_REGION_FLAGS);
    registry.register(&SET_SUN_POSITION);
    registry.register(&SET_WATER_HEIGHT);
    registry.register(&REGION_STATUS);
    registry.register(&LIST_AGENTS);
}
