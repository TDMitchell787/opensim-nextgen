use crate::ai::skill_engine::*;

static P_TARGET_AGENT: ParamDef = ParamDef {
    name: "target_agent_id", param_type: ParamType::Uuid, required: true,
    default_value: None, description: "Agent to receive the garment",
};
static P_LOGO_PATH: ParamDef = ParamDef {
    name: "logo_path", param_type: ParamType::String, required: false,
    default_value: None, description: "Path to logo image for the shirt",
};
static P_SHIRT_COLOR: ParamDef = ParamDef {
    name: "shirt_color", param_type: ParamType::Vec4, required: false,
    default_value: Some("[255, 255, 255, 255]"), description: "RGBA color bytes",
};
static P_FRONT_OFFSET: ParamDef = ParamDef {
    name: "front_offset_inches", param_type: ParamType::F32, required: false,
    default_value: Some("4.0"), description: "Logo offset from center in inches",
};
static P_BACK_OFFSET: ParamDef = ParamDef {
    name: "back_offset_inches", param_type: ParamType::F32, required: false,
    default_value: None, description: "Back logo offset (None = no back logo)",
};
static P_SLEEVE_LENGTH: ParamDef = ParamDef {
    name: "sleeve_length", param_type: ParamType::F32, required: false,
    default_value: Some("0.6"), description: "Sleeve length 0.0 (tank) to 1.0 (full)",
};
static P_FIT: ParamDef = ParamDef {
    name: "fit", param_type: ParamType::Enum(&["tight", "regular", "loose"]),
    required: false, default_value: Some("regular"), description: "Garment fit",
};
static P_COLLAR: ParamDef = ParamDef {
    name: "collar", param_type: ParamType::Enum(&["crew", "v-neck", "scoop"]),
    required: false, default_value: Some("crew"), description: "Collar style",
};
static P_GARMENT_NAME: ParamDef = ParamDef {
    name: "name", param_type: ParamType::String, required: false,
    default_value: Some("Custom T-Shirt"), description: "Garment name",
};
static P_COLOR: ParamDef = ParamDef {
    name: "color", param_type: ParamType::Vec4, required: false,
    default_value: Some("[255, 255, 255, 255]"), description: "Garment color RGBA",
};
static P_STYLE: ParamDef = ParamDef {
    name: "style", param_type: ParamType::String, required: false,
    default_value: Some("standard"), description: "Garment style variant",
};
static P_LENGTH: ParamDef = ParamDef {
    name: "length", param_type: ParamType::F32, required: false,
    default_value: Some("1.0"), description: "Garment length 0.0-1.0",
};
static P_TYPE: ParamDef = ParamDef {
    name: "type", param_type: ParamType::String, required: true,
    default_value: None, description: "Accessory type (hat, glasses, belt, etc.)",
};

pub static CREATE_TSHIRT: SkillDef = SkillDef {
    id: "create_tshirt",
    domain: SkillDomain::Clothing,
    display_name: "Create T-Shirt",
    description: "Generate a wearable mesh t-shirt with optional logo",
    params: &[P_TARGET_AGENT, P_LOGO_PATH, P_SHIRT_COLOR, P_FRONT_OFFSET, P_BACK_OFFSET, P_SLEEVE_LENGTH, P_FIT, P_COLLAR, P_GARMENT_NAME],
    returns: ReturnType::LocalId,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 175",
    tags: &["clothing", "wearable", "tshirt", "mesh"],
    examples: &[SkillExample {
        description: "Create a red crew-neck t-shirt with logo",
        input: r#"{"target_agent_id": "...", "shirt_color": [255, 0, 0, 255], "logo_path": "logo.png"}"#,
        output: r#"{"message": "T-shirt created and delivered", "local_id": 5001}"#,
    }],
};

pub static CREATE_PANTS: SkillDef = SkillDef {
    id: "create_pants",
    domain: SkillDomain::Clothing,
    display_name: "Create Pants",
    description: "Generate wearable mesh pants",
    params: &[P_TARGET_AGENT, P_COLOR, P_STYLE, P_FIT, P_GARMENT_NAME],
    returns: ReturnType::LocalId,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 203.8",
    tags: &["clothing", "wearable", "pants", "mesh"],
    examples: &[],
};

pub static CREATE_DRESS: SkillDef = SkillDef {
    id: "create_dress",
    domain: SkillDomain::Clothing,
    display_name: "Create Dress",
    description: "Generate a wearable mesh dress",
    params: &[P_TARGET_AGENT, P_COLOR, P_STYLE, P_LENGTH, P_FIT, P_GARMENT_NAME],
    returns: ReturnType::LocalId,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 203.8",
    tags: &["clothing", "wearable", "dress", "mesh"],
    examples: &[],
};

pub static CREATE_JACKET: SkillDef = SkillDef {
    id: "create_jacket",
    domain: SkillDomain::Clothing,
    display_name: "Create Jacket",
    description: "Generate a wearable mesh jacket",
    params: &[P_TARGET_AGENT, P_COLOR, P_STYLE, P_FIT, P_GARMENT_NAME],
    returns: ReturnType::LocalId,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 203.8",
    tags: &["clothing", "wearable", "jacket", "mesh"],
    examples: &[],
};

pub static CREATE_SKIRT: SkillDef = SkillDef {
    id: "create_skirt",
    domain: SkillDomain::Clothing,
    display_name: "Create Skirt",
    description: "Generate a wearable mesh skirt",
    params: &[P_TARGET_AGENT, P_COLOR, P_LENGTH, P_STYLE, P_GARMENT_NAME],
    returns: ReturnType::LocalId,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 203.8",
    tags: &["clothing", "wearable", "skirt", "mesh"],
    examples: &[],
};

pub static CREATE_ACCESSORY: SkillDef = SkillDef {
    id: "create_accessory",
    domain: SkillDomain::Clothing,
    display_name: "Create Accessory",
    description: "Generate a wearable accessory (hat, glasses, belt, etc.)",
    params: &[P_TARGET_AGENT, P_TYPE, P_STYLE, P_COLOR, P_GARMENT_NAME],
    returns: ReturnType::LocalId,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional,
    phase: "Phase 203.8",
    tags: &["clothing", "wearable", "accessory", "mesh"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&CREATE_TSHIRT);
    registry.register(&CREATE_PANTS);
    registry.register(&CREATE_DRESS);
    registry.register(&CREATE_JACKET);
    registry.register(&CREATE_SKIRT);
    registry.register(&CREATE_ACCESSORY);
}
