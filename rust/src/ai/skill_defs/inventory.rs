use crate::ai::skill_engine::*;

static P_AGENT_ID: ParamDef = ParamDef {
    name: "agent_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Agent UUID",
};
static P_ITEM_ID: ParamDef = ParamDef {
    name: "item_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Inventory item UUID",
};
static P_FOLDER_ID: ParamDef = ParamDef {
    name: "folder_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Inventory folder UUID",
};
static P_KIT_TYPE: ParamDef = ParamDef {
    name: "kit_type",
    param_type: ParamType::Enum(&["starter", "builder", "clothier", "scripter"]),
    required: true,
    default_value: None,
    description: "Kit type to create",
};
static P_QUERY: ParamDef = ParamDef {
    name: "query",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Search query",
};
static P_TYPE_FILTER: ParamDef = ParamDef {
    name: "type_filter",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Filter by asset type (texture, object, script, etc.)",
};
static P_STRATEGY: ParamDef = ParamDef {
    name: "strategy",
    param_type: ParamType::Enum(&["by_type", "by_date", "by_name"]),
    required: false,
    default_value: Some("by_type"),
    description: "Organization strategy",
};

pub static GIVE_INVENTORY: SkillDef = SkillDef {
    id: "give_inventory",
    domain: SkillDomain::Inventory,
    display_name: "Give Inventory Item",
    description: "Give an inventory item to another agent",
    params: &[P_AGENT_ID, P_ITEM_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["inventory", "give", "transfer"],
    examples: &[],
};

pub static GIVE_FOLDER: SkillDef = SkillDef {
    id: "give_folder",
    domain: SkillDomain::Inventory,
    display_name: "Give Inventory Folder",
    description: "Give an entire inventory folder to another agent",
    params: &[P_AGENT_ID, P_FOLDER_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["inventory", "give", "folder"],
    examples: &[],
};

pub static CREATE_KIT: SkillDef = SkillDef {
    id: "create_kit",
    domain: SkillDomain::Inventory,
    display_name: "Create Starter Kit",
    description: "Create a pre-built starter kit folder for an agent",
    params: &[P_AGENT_ID, P_KIT_TYPE],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["inventory", "kit", "onboarding"],
    examples: &[],
};

pub static SEARCH_INVENTORY: SkillDef = SkillDef {
    id: "search_inventory",
    domain: SkillDomain::Inventory,
    display_name: "Search Inventory",
    description: "Search an agent's inventory by name and type",
    params: &[P_AGENT_ID, P_QUERY, P_TYPE_FILTER],
    returns: ReturnType::ObjectData,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["inventory", "search", "query"],
    examples: &[],
};

pub static BULK_DISTRIBUTE: SkillDef = SkillDef {
    id: "bulk_distribute",
    domain: SkillDomain::Inventory,
    display_name: "Bulk Distribute Item",
    description: "Distribute an item to multiple agents at once",
    params: &[P_ITEM_ID],
    returns: ReturnType::ObjectData,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["inventory", "distribute", "bulk"],
    examples: &[],
};

pub static ORGANIZE_FOLDER: SkillDef = SkillDef {
    id: "organize_folder",
    domain: SkillDomain::Inventory,
    display_name: "Organize Folder",
    description: "Sort and organize items within a folder",
    params: &[P_AGENT_ID, P_FOLDER_ID, P_STRATEGY],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["inventory", "organize", "sort"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&GIVE_INVENTORY);
    registry.register(&GIVE_FOLDER);
    registry.register(&CREATE_KIT);
    registry.register(&SEARCH_INVENTORY);
    registry.register(&BULK_DISTRIBUTE);
    registry.register(&ORGANIZE_FOLDER);
}
