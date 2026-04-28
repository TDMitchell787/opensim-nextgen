use crate::ai::skill_engine::*;

static P_POSITION: ParamDef = ParamDef {
    name: "position",
    param_type: ParamType::Vec3,
    required: true,
    default_value: None,
    description: "World position [x, y, z]",
};
static P_SCALE: ParamDef = ParamDef {
    name: "scale",
    param_type: ParamType::Vec3,
    required: false,
    default_value: Some("[1.0, 1.0, 1.0]"),
    description: "Size [x, y, z]",
};
static P_NAME: ParamDef = ParamDef {
    name: "name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Object name",
};
static P_LOCAL_ID: ParamDef = ParamDef {
    name: "local_id",
    param_type: ParamType::U32,
    required: true,
    default_value: None,
    description: "Object local ID",
};
static P_ROTATION: ParamDef = ParamDef {
    name: "rotation",
    param_type: ParamType::Vec4,
    required: true,
    default_value: None,
    description: "Quaternion [x, y, z, w]",
};
static P_COLOR: ParamDef = ParamDef {
    name: "color",
    param_type: ParamType::Vec4,
    required: true,
    default_value: None,
    description: "RGBA color [r, g, b, a] 0.0-1.0",
};
static P_TEXTURE_UUID: ParamDef = ParamDef {
    name: "texture_uuid",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Texture asset UUID",
};
static P_TARGET_AGENT: ParamDef = ParamDef {
    name: "target_agent_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Target agent UUID",
};

macro_rules! rez_skill {
    ($id:ident, $skill_id:expr, $name:expr, $desc:expr) => {
        pub static $id: SkillDef = SkillDef {
            id: $skill_id,
            domain: SkillDomain::Building,
            display_name: $name,
            description: $desc,
            params: &[P_POSITION, P_SCALE, P_NAME],
            returns: ReturnType::LocalId,
            requires_region: true,
            requires_agent: true,
            requires_admin: false,
            maturity: SkillMaturity::L7Production,
            phase: "Phase 154",
            tags: &["prim", "creation", "building"],
            examples: &[],
        };
    };
}

rez_skill!(
    REZ_BOX,
    "rez_box",
    "Rez Box",
    "Create a box primitive at position"
);
rez_skill!(
    REZ_CYLINDER,
    "rez_cylinder",
    "Rez Cylinder",
    "Create a cylinder primitive at position"
);
rez_skill!(
    REZ_SPHERE,
    "rez_sphere",
    "Rez Sphere",
    "Create a sphere primitive at position"
);
rez_skill!(
    REZ_TORUS,
    "rez_torus",
    "Rez Torus",
    "Create a torus primitive at position"
);
rez_skill!(
    REZ_TUBE,
    "rez_tube",
    "Rez Tube",
    "Create a tube primitive at position"
);
rez_skill!(
    REZ_RING,
    "rez_ring",
    "Rez Ring",
    "Create a ring primitive at position"
);
rez_skill!(
    REZ_PRISM,
    "rez_prism",
    "Rez Prism",
    "Create a prism primitive at position"
);

pub static SET_POSITION: SkillDef = SkillDef {
    id: "set_position",
    domain: SkillDomain::Building,
    display_name: "Set Position",
    description: "Move an object to a new position",
    params: &[P_LOCAL_ID, P_POSITION],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["prim", "transform", "move"],
    examples: &[],
};

pub static SET_ROTATION: SkillDef = SkillDef {
    id: "set_rotation",
    domain: SkillDomain::Building,
    display_name: "Set Rotation",
    description: "Rotate an object to a quaternion orientation",
    params: &[P_LOCAL_ID, P_ROTATION],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["prim", "transform", "rotate"],
    examples: &[],
};

pub static SET_SCALE: SkillDef = SkillDef {
    id: "set_scale",
    domain: SkillDomain::Building,
    display_name: "Set Scale",
    description: "Resize an object",
    params: &[P_LOCAL_ID, P_SCALE],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["prim", "transform", "resize"],
    examples: &[],
};

pub static SET_COLOR: SkillDef = SkillDef {
    id: "set_color",
    domain: SkillDomain::Building,
    display_name: "Set Color",
    description: "Set the RGBA color of an object",
    params: &[P_LOCAL_ID, P_COLOR],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["prim", "appearance", "color"],
    examples: &[],
};

pub static SET_TEXTURE: SkillDef = SkillDef {
    id: "set_texture",
    domain: SkillDomain::Building,
    display_name: "Set Texture",
    description: "Apply a texture to an object by UUID",
    params: &[P_LOCAL_ID, P_TEXTURE_UUID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["prim", "appearance", "texture"],
    examples: &[],
};

pub static SET_NAME: SkillDef = SkillDef {
    id: "set_name",
    domain: SkillDomain::Building,
    display_name: "Set Name",
    description: "Rename an object",
    params: &[P_LOCAL_ID, P_NAME],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["prim", "metadata", "name"],
    examples: &[],
};

static P_ROOT_ID: ParamDef = ParamDef {
    name: "root_id",
    param_type: ParamType::U32,
    required: true,
    default_value: None,
    description: "Root prim local ID",
};
static P_CHILD_IDS: ParamDef = ParamDef {
    name: "child_ids",
    param_type: ParamType::U32Array,
    required: true,
    default_value: None,
    description: "Child prim local IDs",
};

pub static LINK_OBJECTS: SkillDef = SkillDef {
    id: "link_objects",
    domain: SkillDomain::Building,
    display_name: "Link Objects",
    description: "Link child prims to a root prim to form a linkset",
    params: &[P_ROOT_ID, P_CHILD_IDS],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["prim", "linkset", "link"],
    examples: &[],
};

pub static DELETE_OBJECT: SkillDef = SkillDef {
    id: "delete_object",
    domain: SkillDomain::Building,
    display_name: "Delete Object",
    description: "Remove a prim from the scene",
    params: &[P_LOCAL_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["prim", "delete"],
    examples: &[],
};

static P_GEOMETRY_TYPE: ParamDef = ParamDef {
    name: "geometry_type",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Mesh geometry type identifier",
};

pub static REZ_MESH: SkillDef = SkillDef {
    id: "rez_mesh",
    domain: SkillDomain::Building,
    display_name: "Rez Mesh",
    description: "Create a mesh object from geometry asset",
    params: &[P_GEOMETRY_TYPE, P_POSITION, P_SCALE, P_NAME],
    returns: ReturnType::LocalId,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["mesh", "creation", "building"],
    examples: &[],
};

static P_FILE_PATH: ParamDef = ParamDef {
    name: "file_path",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Path to file (within instance dir)",
};

pub static IMPORT_MESH: SkillDef = SkillDef {
    id: "import_mesh",
    domain: SkillDomain::Building,
    display_name: "Import Mesh",
    description: "Import a mesh from a DAE/OBJ file",
    params: &[P_FILE_PATH, P_NAME, P_POSITION],
    returns: ReturnType::LocalId,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["mesh", "import", "building"],
    examples: &[],
};

static P_TEMPLATE: ParamDef = ParamDef {
    name: "template",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Blender template name",
};
static P_BLENDER_PARAMS: ParamDef = ParamDef {
    name: "params",
    param_type: ParamType::StringMap,
    required: false,
    default_value: None,
    description: "Template parameter overrides",
};

pub static BLENDER_GENERATE: SkillDef = SkillDef {
    id: "blender_generate",
    domain: SkillDomain::Building,
    display_name: "Blender Generate",
    description: "Generate a mesh via Blender headless pipeline",
    params: &[P_TEMPLATE, P_BLENDER_PARAMS, P_NAME, P_POSITION],
    returns: ReturnType::LocalId,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["mesh", "blender", "generation"],
    examples: &[],
};

static P_REGION_ID: ParamDef = ParamDef {
    name: "region_id",
    param_type: ParamType::Uuid,
    required: true,
    default_value: None,
    description: "Region UUID",
};
static P_FILENAME: ParamDef = ParamDef {
    name: "filename",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Output filename",
};

pub static EXPORT_OAR: SkillDef = SkillDef {
    id: "export_oar",
    domain: SkillDomain::Building,
    display_name: "Export OAR",
    description: "Export region as an OAR archive file",
    params: &[P_REGION_ID, P_FILENAME],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["export", "archive", "oar"],
    examples: &[],
};

pub static GIVE_OBJECT: SkillDef = SkillDef {
    id: "give_object",
    domain: SkillDomain::Building,
    display_name: "Give Object",
    description: "Transfer an object to another agent",
    params: &[P_LOCAL_ID, P_TARGET_AGENT],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["transfer", "give", "inventory"],
    examples: &[],
};

pub static CREATE_BADGE: SkillDef = SkillDef {
    id: "create_badge",
    domain: SkillDomain::Building,
    display_name: "Create Badge",
    description: "Create and attach a Galadriel communicator badge",
    params: &[P_TARGET_AGENT],
    returns: ReturnType::LocalId,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["badge", "attachment", "communication"],
    examples: &[],
};

static P_SEARCH_NAME: ParamDef = ParamDef {
    name: "name",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Object name to search for",
};
static P_RADIUS: ParamDef = ParamDef {
    name: "radius",
    param_type: ParamType::F32,
    required: false,
    default_value: Some("20.0"),
    description: "Search radius in meters",
};

pub static FIND_NEARBY: SkillDef = SkillDef {
    id: "find_nearby",
    domain: SkillDomain::Building,
    display_name: "Find Nearby Object",
    description: "Search for objects by name within radius",
    params: &[P_SEARCH_NAME, P_RADIUS],
    returns: ReturnType::ObjectData,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["search", "find", "query"],
    examples: &[],
};

pub static SCAN_LINKSET: SkillDef = SkillDef {
    id: "scan_linkset",
    domain: SkillDomain::Building,
    display_name: "Scan Linkset",
    description: "Enumerate all prims in a linkset",
    params: &[P_ROOT_ID],
    returns: ReturnType::ObjectData,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["linkset", "query", "inspect"],
    examples: &[],
};

static P_SCRIPT_MAP: ParamDef = ParamDef {
    name: "script_map",
    param_type: ParamType::StringMap,
    required: true,
    default_value: None,
    description: "Map of link_name -> script_source",
};
static P_ROOT_SCRIPT: ParamDef = ParamDef {
    name: "root_script",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Script source for root prim",
};

pub static SCRIPT_LINKSET: SkillDef = SkillDef {
    id: "script_linkset",
    domain: SkillDomain::Building,
    display_name: "Script Linkset",
    description: "Insert scripts into linkset prims by name mapping",
    params: &[P_ROOT_ID, P_SCRIPT_MAP, P_ROOT_SCRIPT],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["linkset", "scripting", "batch"],
    examples: &[],
};

static P_NEW_PRIM_IDS: ParamDef = ParamDef {
    name: "new_prim_ids",
    param_type: ParamType::U32Array,
    required: true,
    default_value: None,
    description: "Prim IDs to add to the linkset",
};

pub static ADD_TO_LINKSET: SkillDef = SkillDef {
    id: "add_to_linkset",
    domain: SkillDomain::Building,
    display_name: "Add to Linkset",
    description: "Add prims to an existing linkset",
    params: &[P_ROOT_ID, P_NEW_PRIM_IDS],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 154",
    tags: &["linkset", "link", "add"],
    examples: &[],
};

static P_SOURCE_LOCAL_ID: ParamDef = ParamDef {
    name: "source_local_id",
    param_type: ParamType::U32,
    required: true,
    default_value: None,
    description: "Object to package",
};
static P_CONTAINER_LOCAL_ID: ParamDef = ParamDef {
    name: "container_local_id",
    param_type: ParamType::U32,
    required: true,
    default_value: None,
    description: "Container prim to place object into",
};

pub static PACKAGE_INTO_PRIM: SkillDef = SkillDef {
    id: "package_into_prim",
    domain: SkillDomain::Building,
    display_name: "Package into Prim",
    description: "Package an object into a container prim's inventory",
    params: &[P_SOURCE_LOCAL_ID, P_CONTAINER_LOCAL_ID],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 157",
    tags: &["package", "container", "inventory"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&REZ_BOX);
    registry.register(&REZ_CYLINDER);
    registry.register(&REZ_SPHERE);
    registry.register(&REZ_TORUS);
    registry.register(&REZ_TUBE);
    registry.register(&REZ_RING);
    registry.register(&REZ_PRISM);
    registry.register(&SET_POSITION);
    registry.register(&SET_ROTATION);
    registry.register(&SET_SCALE);
    registry.register(&SET_COLOR);
    registry.register(&SET_TEXTURE);
    registry.register(&SET_NAME);
    registry.register(&LINK_OBJECTS);
    registry.register(&DELETE_OBJECT);
    registry.register(&REZ_MESH);
    registry.register(&IMPORT_MESH);
    registry.register(&BLENDER_GENERATE);
    registry.register(&EXPORT_OAR);
    registry.register(&GIVE_OBJECT);
    registry.register(&CREATE_BADGE);
    registry.register(&FIND_NEARBY);
    registry.register(&SCAN_LINKSET);
    registry.register(&SCRIPT_LINKSET);
    registry.register(&ADD_TO_LINKSET);
    registry.register(&PACKAGE_INTO_PRIM);
}
