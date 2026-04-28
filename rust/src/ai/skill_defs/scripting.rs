use crate::ai::skill_engine::*;

static P_LOCAL_ID: ParamDef = ParamDef {
    name: "local_id",
    param_type: ParamType::U32,
    required: true,
    default_value: None,
    description: "Target prim local ID",
};
static P_SCRIPT_NAME: ParamDef = ParamDef {
    name: "script_name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Name for the script",
};
static P_SCRIPT_SOURCE: ParamDef = ParamDef {
    name: "script_source",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "LSL script source code",
};
static P_TEMPLATE_NAME: ParamDef = ParamDef {
    name: "template_name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Script template identifier",
};
static P_TEMPLATE_PARAMS: ParamDef = ParamDef {
    name: "params",
    param_type: ParamType::StringMap,
    required: false,
    default_value: None,
    description: "Template parameter substitutions",
};

pub static INSERT_SCRIPT: SkillDef = SkillDef {
    id: "insert_script",
    domain: SkillDomain::Scripting,
    display_name: "Insert Script",
    description: "Insert raw LSL source into a prim",
    params: &[P_LOCAL_ID, P_SCRIPT_NAME, P_SCRIPT_SOURCE],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["script", "lsl", "insert"],
    examples: &[],
};

pub static INSERT_TEMPLATE_SCRIPT: SkillDef = SkillDef {
    id: "insert_template_script",
    domain: SkillDomain::Scripting,
    display_name: "Insert Template Script",
    description: "Insert a pre-built script template with parameter substitution",
    params: &[P_LOCAL_ID, P_TEMPLATE_NAME, P_TEMPLATE_PARAMS],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["script", "lsl", "template"],
    examples: &[],
};

pub static UPDATE_SCRIPT: SkillDef = SkillDef {
    id: "update_script",
    domain: SkillDomain::Scripting,
    display_name: "Update Script",
    description: "Replace an existing script's source code",
    params: &[P_LOCAL_ID, P_SCRIPT_NAME, P_SCRIPT_SOURCE],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["script", "lsl", "update"],
    examples: &[],
};

pub static GIVE_TO_REQUESTER: SkillDef = SkillDef {
    id: "give_to_requester",
    domain: SkillDomain::Scripting,
    display_name: "Give to Requester",
    description: "Transfer a built object to the requesting agent",
    params: &[P_LOCAL_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["give", "transfer", "delivery"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&INSERT_SCRIPT);
    registry.register(&INSERT_TEMPLATE_SCRIPT);
    registry.register(&UPDATE_SCRIPT);
    registry.register(&GIVE_TO_REQUESTER);
}
