use crate::ai::skill_engine::*;

static P_AGENT_ID: ParamDef = ParamDef { name: "agent_id", param_type: ParamType::Uuid, required: true, default_value: None, description: "Agent UUID" };
static P_TOPIC: ParamDef = ParamDef { name: "topic", param_type: ParamType::String, required: true, default_value: None, description: "Tutorial topic" };
static P_MESSAGE: ParamDef = ParamDef { name: "message", param_type: ParamType::String, required: true, default_value: None, description: "Hint or info text" };
static P_HIGHLIGHT: ParamDef = ParamDef { name: "highlight_object", param_type: ParamType::U32, required: false, default_value: None, description: "Local ID of object to highlight" };
static P_POSITION: ParamDef = ParamDef { name: "position", param_type: ParamType::Vec3, required: true, default_value: None, description: "World position" };
static P_TITLE: ParamDef = ParamDef { name: "title", param_type: ParamType::String, required: true, default_value: None, description: "Title" };
static P_BODY: ParamDef = ParamDef { name: "body_text", param_type: ParamType::String, required: true, default_value: None, description: "Body text content" };
static P_STYLE: ParamDef = ParamDef { name: "style", param_type: ParamType::Enum(&["minimal", "standard", "elaborate"]), required: false, default_value: Some("standard"), description: "Welcome area style" };
static P_STEP: ParamDef = ParamDef { name: "step", param_type: ParamType::U32, required: true, default_value: None, description: "Tutorial step number" };

pub static START_TUTORIAL: SkillDef = SkillDef {
    id: "start_tutorial", domain: SkillDomain::Tutorial, display_name: "Start Tutorial",
    description: "Begin an interactive tutorial for an agent on a topic",
    params: &[P_AGENT_ID, P_TOPIC],
    returns: ReturnType::Success,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional, phase: "Phase 209.11",
    tags: &["tutorial", "education", "onboarding"], examples: &[],
};

pub static SHOW_HINT: SkillDef = SkillDef {
    id: "show_hint", domain: SkillDomain::Tutorial, display_name: "Show Hint",
    description: "Display a contextual hint to an agent, optionally highlighting an object",
    params: &[P_AGENT_ID, P_MESSAGE, P_HIGHLIGHT],
    returns: ReturnType::Success,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional, phase: "Phase 209.11",
    tags: &["tutorial", "hint", "help"], examples: &[],
};

pub static CREATE_INFO_BOARD: SkillDef = SkillDef {
    id: "create_info_board", domain: SkillDomain::Tutorial, display_name: "Create Info Board",
    description: "Create an informational display board with title and text",
    params: &[P_POSITION, P_TITLE, P_BODY],
    returns: ReturnType::LocalId,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional, phase: "Phase 209.11",
    tags: &["tutorial", "info", "signage"], examples: &[],
};

pub static CREATE_WELCOME_AREA: SkillDef = SkillDef {
    id: "create_welcome_area", domain: SkillDomain::Tutorial, display_name: "Create Welcome Area",
    description: "Rez a pre-built welcome/tutorial environment",
    params: &[P_POSITION, P_STYLE],
    returns: ReturnType::LocalIds,
    requires_region: true, requires_agent: true, requires_admin: true,
    maturity: SkillMaturity::L3Functional, phase: "Phase 209.11",
    tags: &["tutorial", "welcome", "environment"], examples: &[],
};

pub static TRACK_PROGRESS: SkillDef = SkillDef {
    id: "track_progress", domain: SkillDomain::Tutorial, display_name: "Track Tutorial Progress",
    description: "Record an agent's progress through a tutorial",
    params: &[P_AGENT_ID, P_TOPIC, P_STEP],
    returns: ReturnType::Success,
    requires_region: false, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional, phase: "Phase 209.11",
    tags: &["tutorial", "progress", "tracking"], examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&START_TUTORIAL);
    registry.register(&SHOW_HINT);
    registry.register(&CREATE_INFO_BOARD);
    registry.register(&CREATE_WELCOME_AREA);
    registry.register(&TRACK_PROGRESS);
}
