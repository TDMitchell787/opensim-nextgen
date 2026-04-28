use crate::ai::skill_engine::*;

static P_PRESET: ParamDef = ParamDef {
    name: "preset",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Terrain preset name (island, mountains, mesa, etc.)",
};
static P_SEED: ParamDef = ParamDef {
    name: "seed",
    param_type: ParamType::U32,
    required: false,
    default_value: None,
    description: "Random seed for reproducibility",
};
static P_SCALE: ParamDef = ParamDef {
    name: "scale",
    param_type: ParamType::F32,
    required: false,
    default_value: Some("1.0"),
    description: "Height scale multiplier",
};
static P_ROUGHNESS: ParamDef = ParamDef {
    name: "roughness",
    param_type: ParamType::F32,
    required: false,
    default_value: None,
    description: "Terrain roughness factor",
};
static P_WATER_LEVEL: ParamDef = ParamDef {
    name: "water_level",
    param_type: ParamType::F32,
    required: false,
    default_value: None,
    description: "Water height in meters",
};
static P_REGION_ID: ParamDef = ParamDef {
    name: "region_id",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Target region ID or name",
};
static P_GRID_SIZE: ParamDef = ParamDef {
    name: "grid_size",
    param_type: ParamType::U32,
    required: false,
    default_value: None,
    description: "Multi-region grid size",
};
static P_GRID_X: ParamDef = ParamDef {
    name: "grid_x",
    param_type: ParamType::U32,
    required: false,
    default_value: None,
    description: "Grid X coordinate",
};
static P_GRID_Y: ParamDef = ParamDef {
    name: "grid_y",
    param_type: ParamType::U32,
    required: false,
    default_value: None,
    description: "Grid Y coordinate",
};
static P_FILE_PATH: ParamDef = ParamDef {
    name: "file_path",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Path to terrain file",
};
static P_HEIGHT_MIN: ParamDef = ParamDef {
    name: "height_min",
    param_type: ParamType::F32,
    required: false,
    default_value: Some("0.0"),
    description: "Minimum height mapping",
};
static P_HEIGHT_MAX: ParamDef = ParamDef {
    name: "height_max",
    param_type: ParamType::F32,
    required: false,
    default_value: Some("100.0"),
    description: "Maximum height mapping",
};
static P_PREVIEW_ID: ParamDef = ParamDef {
    name: "preview_id",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Preview session ID to apply or reject",
};

pub static TERRAIN_GENERATE: SkillDef = SkillDef {
    id: "terrain_generate",
    domain: SkillDomain::Landscaping,
    display_name: "Generate Terrain",
    description: "Procedurally generate terrain from a preset",
    params: &[
        P_PRESET,
        P_SEED,
        P_SCALE,
        P_ROUGHNESS,
        P_WATER_LEVEL,
        P_REGION_ID,
        P_GRID_SIZE,
        P_GRID_X,
        P_GRID_Y,
    ],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 163",
    tags: &["terrain", "generation", "procedural"],
    examples: &[],
};

pub static TERRAIN_LOAD_R32: SkillDef = SkillDef {
    id: "terrain_load_r32",
    domain: SkillDomain::Landscaping,
    display_name: "Load R32 Terrain",
    description: "Load terrain from a raw R32 heightmap file",
    params: &[P_FILE_PATH],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 163",
    tags: &["terrain", "import", "heightmap"],
    examples: &[],
};

pub static TERRAIN_LOAD_IMAGE: SkillDef = SkillDef {
    id: "terrain_load_image",
    domain: SkillDomain::Landscaping,
    display_name: "Load Heightmap Image",
    description: "Load terrain from a grayscale heightmap image",
    params: &[P_FILE_PATH, P_HEIGHT_MIN, P_HEIGHT_MAX],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 163",
    tags: &["terrain", "import", "image"],
    examples: &[],
};

pub static TERRAIN_PREVIEW: SkillDef = SkillDef {
    id: "terrain_preview",
    domain: SkillDomain::Landscaping,
    display_name: "Preview Terrain",
    description: "Preview terrain changes before applying",
    params: &[
        P_PRESET,
        P_SEED,
        P_SCALE,
        P_ROUGHNESS,
        P_WATER_LEVEL,
        P_REGION_ID,
        P_GRID_SIZE,
        P_GRID_X,
        P_GRID_Y,
    ],
    returns: ReturnType::TextResponse,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 163",
    tags: &["terrain", "preview", "confirmation"],
    examples: &[],
};

pub static TERRAIN_APPLY: SkillDef = SkillDef {
    id: "terrain_apply",
    domain: SkillDomain::Landscaping,
    display_name: "Apply Terrain",
    description: "Apply a previewed terrain change",
    params: &[P_PREVIEW_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 163",
    tags: &["terrain", "apply", "confirmation"],
    examples: &[],
};

pub static TERRAIN_REJECT: SkillDef = SkillDef {
    id: "terrain_reject",
    domain: SkillDomain::Landscaping,
    display_name: "Reject Terrain",
    description: "Cancel a previewed terrain change",
    params: &[P_PREVIEW_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 163",
    tags: &["terrain", "reject", "cancel"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&TERRAIN_GENERATE);
    registry.register(&TERRAIN_LOAD_R32);
    registry.register(&TERRAIN_LOAD_IMAGE);
    registry.register(&TERRAIN_PREVIEW);
    registry.register(&TERRAIN_APPLY);
    registry.register(&TERRAIN_REJECT);
}
