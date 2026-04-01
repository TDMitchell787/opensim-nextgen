use crate::ai::skill_engine::*;

static P_AGENT_ID: ParamDef = ParamDef { name: "agent_id", param_type: ParamType::Uuid, required: true, default_value: None, description: "Agent UUID" };
static P_REGION: ParamDef = ParamDef { name: "region", param_type: ParamType::String, required: true, default_value: None, description: "Region name" };
static P_POSITION: ParamDef = ParamDef { name: "position", param_type: ParamType::Vec3, required: false, default_value: Some("[128.0, 128.0, 25.0]"), description: "Position [x, y, z]" };
static P_NAME: ParamDef = ParamDef { name: "name", param_type: ParamType::String, required: true, default_value: None, description: "Name" };
static P_DESCRIPTION: ParamDef = ParamDef { name: "description", param_type: ParamType::String, required: false, default_value: None, description: "Description" };
static P_LANDMARK_ID: ParamDef = ParamDef { name: "landmark_id", param_type: ParamType::Uuid, required: true, default_value: None, description: "Landmark inventory item ID" };
static P_TOUR_NAME: ParamDef = ParamDef { name: "tour_name", param_type: ParamType::String, required: true, default_value: None, description: "Tour name" };
static P_RADIUS: ParamDef = ParamDef { name: "radius", param_type: ParamType::F32, required: false, default_value: Some("256.0"), description: "Search radius" };

pub static TELEPORT_AGENT: SkillDef = SkillDef {
    id: "teleport_agent", domain: SkillDomain::Navigation, display_name: "Teleport Agent",
    description: "Teleport an agent to a position in a region",
    params: &[P_AGENT_ID, P_REGION, P_POSITION],
    returns: ReturnType::Success,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L4Robust, phase: "Phase 203.7",
    tags: &["navigation", "teleport", "movement"], examples: &[],
};

pub static CREATE_LANDMARK: SkillDef = SkillDef {
    id: "create_landmark", domain: SkillDomain::Navigation, display_name: "Create Landmark",
    description: "Create a landmark inventory item for a location",
    params: &[P_NAME, P_REGION, P_POSITION, P_DESCRIPTION],
    returns: ReturnType::Success,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional, phase: "Phase 203.7",
    tags: &["navigation", "landmark", "location"], examples: &[],
};

pub static GIVE_LANDMARK: SkillDef = SkillDef {
    id: "give_landmark", domain: SkillDomain::Navigation, display_name: "Give Landmark",
    description: "Give a landmark to another agent",
    params: &[P_AGENT_ID, P_LANDMARK_ID],
    returns: ReturnType::Success,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional, phase: "Phase 203.7",
    tags: &["navigation", "landmark", "give"], examples: &[],
};

pub static CREATE_WAYPOINT_TOUR: SkillDef = SkillDef {
    id: "create_waypoint_tour", domain: SkillDomain::Navigation, display_name: "Create Waypoint Tour",
    description: "Define a multi-stop guided tour with descriptions at each waypoint",
    params: &[P_NAME, P_DESCRIPTION],
    returns: ReturnType::ObjectData,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional, phase: "Phase 203.7",
    tags: &["navigation", "tour", "waypoint"], examples: &[],
};

pub static START_GUIDED_TOUR: SkillDef = SkillDef {
    id: "start_guided_tour", domain: SkillDomain::Navigation, display_name: "Start Guided Tour",
    description: "Start a guided tour for an agent, teleporting to each waypoint",
    params: &[P_AGENT_ID, P_TOUR_NAME],
    returns: ReturnType::Success,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional, phase: "Phase 203.7",
    tags: &["navigation", "tour", "guided"], examples: &[],
};

pub static MAP_POI: SkillDef = SkillDef {
    id: "map_poi", domain: SkillDomain::Navigation, display_name: "Map Points of Interest",
    description: "Discover and list notable locations within a radius",
    params: &[P_REGION, P_RADIUS],
    returns: ReturnType::ObjectData,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L3Functional, phase: "Phase 203.7",
    tags: &["navigation", "map", "discovery"], examples: &[],
};

pub static SET_HOME_LOCATION: SkillDef = SkillDef {
    id: "set_home_location", domain: SkillDomain::Navigation, display_name: "Set Home Location",
    description: "Set an agent's home location for teleport-home",
    params: &[P_AGENT_ID, P_POSITION],
    returns: ReturnType::Success,
    requires_region: true, requires_agent: true, requires_admin: false,
    maturity: SkillMaturity::L4Robust, phase: "Phase 203.7",
    tags: &["navigation", "home", "location"], examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&TELEPORT_AGENT);
    registry.register(&CREATE_LANDMARK);
    registry.register(&GIVE_LANDMARK);
    registry.register(&CREATE_WAYPOINT_TOUR);
    registry.register(&START_GUIDED_TOUR);
    registry.register(&MAP_POI);
    registry.register(&SET_HOME_LOCATION);
}
