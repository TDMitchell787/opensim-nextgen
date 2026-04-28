use crate::ai::skill_engine::*;

static P_POSITION: ParamDef = ParamDef {
    name: "position",
    param_type: ParamType::Vec3,
    required: true,
    default_value: None,
    description: "World position",
};
static P_NAME: ParamDef = ParamDef {
    name: "name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Name",
};
static P_LOCAL_ID: ParamDef = ParamDef {
    name: "local_id",
    param_type: ParamType::U32,
    required: true,
    default_value: None,
    description: "Object local ID",
};
static P_PRICE: ParamDef = ParamDef {
    name: "price",
    param_type: ParamType::U32,
    required: true,
    default_value: None,
    description: "Price in grid currency",
};
static P_SALE_TYPE: ParamDef = ParamDef {
    name: "sale_type",
    param_type: ParamType::Enum(&["copy", "original", "contents"]),
    required: false,
    default_value: Some("copy"),
    description: "Sale type",
};
static P_FROM_ID: ParamDef = ParamDef {
    name: "from_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Paying agent UUID",
};
static P_TO_ID: ParamDef = ParamDef {
    name: "to_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Receiving agent UUID",
};
static P_AMOUNT: ParamDef = ParamDef {
    name: "amount",
    param_type: ParamType::U32,
    required: true,
    default_value: None,
    description: "Amount in grid currency",
};
static P_AGENT_ID: ParamDef = ParamDef {
    name: "agent_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Agent UUID",
};
static P_OWNER_ID: ParamDef = ParamDef {
    name: "owner_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Owner agent UUID",
};
static P_COUNT: ParamDef = ParamDef {
    name: "count",
    param_type: ParamType::U32,
    required: false,
    default_value: Some("20"),
    description: "Number of results",
};

pub static CREATE_VENDOR: SkillDef = SkillDef {
    id: "create_vendor",
    domain: SkillDomain::Economy,
    display_name: "Create Vendor",
    description: "Create a vendor prim that sells items on touch",
    params: &[P_POSITION, P_NAME],
    returns: ReturnType::LocalId,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["economy", "vendor", "shop"],
    examples: &[],
};

pub static SET_PRICE: SkillDef = SkillDef {
    id: "set_price",
    domain: SkillDomain::Economy,
    display_name: "Set Object Price",
    description: "Set the sale price and type for an object",
    params: &[P_LOCAL_ID, P_PRICE, P_SALE_TYPE],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["economy", "price", "sale"],
    examples: &[],
};

pub static PAY_AGENT: SkillDef = SkillDef {
    id: "pay_agent",
    domain: SkillDomain::Economy,
    display_name: "Pay Agent",
    description: "Transfer currency from one agent to another",
    params: &[P_FROM_ID, P_TO_ID, P_AMOUNT],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["economy", "payment", "transfer"],
    examples: &[],
};

pub static CHECK_BALANCE: SkillDef = SkillDef {
    id: "check_balance",
    domain: SkillDomain::Economy,
    display_name: "Check Balance",
    description: "Check an agent's currency balance",
    params: &[P_AGENT_ID],
    returns: ReturnType::ObjectData,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["economy", "balance", "query"],
    examples: &[],
};

pub static CREATE_TIP_JAR: SkillDef = SkillDef {
    id: "create_tip_jar",
    domain: SkillDomain::Economy,
    display_name: "Create Tip Jar",
    description: "Create a tip jar prim that accepts payments",
    params: &[P_POSITION, P_OWNER_ID, P_NAME],
    returns: ReturnType::LocalId,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["economy", "tip", "donation"],
    examples: &[],
};

pub static LIST_TRANSACTIONS: SkillDef = SkillDef {
    id: "list_transactions",
    domain: SkillDomain::Economy,
    display_name: "List Transactions",
    description: "List recent transaction history for an agent",
    params: &[P_AGENT_ID, P_COUNT],
    returns: ReturnType::ObjectData,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L4Robust,
    phase: "Phase 203.6",
    tags: &["economy", "transactions", "history"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&CREATE_VENDOR);
    registry.register(&SET_PRICE);
    registry.register(&PAY_AGENT);
    registry.register(&CHECK_BALANCE);
    registry.register(&CREATE_TIP_JAR);
    registry.register(&LIST_TRANSACTIONS);
}
