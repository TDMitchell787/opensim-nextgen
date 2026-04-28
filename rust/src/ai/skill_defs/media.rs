use crate::ai::skill_engine::*;

static P_SCENE_NAME: ParamDef = ParamDef {
    name: "scene_name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Scene or composition name",
};
static P_DESCRIPTION: ParamDef = ParamDef {
    name: "description",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Creative description",
};
static P_TITLE: ParamDef = ParamDef {
    name: "title",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Title for the composition",
};
static P_BOARD_NAME: ParamDef = ParamDef {
    name: "board_name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Advertisement board name",
};
static P_SUBJECT_POS: ParamDef = ParamDef {
    name: "subject_position",
    param_type: ParamType::Vec3,
    required: true,
    default_value: None,
    description: "Subject position [x, y, z]",
};
static P_CAMERA_ANGLE: ParamDef = ParamDef {
    name: "camera_angle",
    param_type: ParamType::String,
    required: false,
    default_value: Some("eye_level"),
    description: "Camera angle preset",
};
static P_COMPOSITION: ParamDef = ParamDef {
    name: "composition",
    param_type: ParamType::String,
    required: false,
    default_value: Some("rule_of_thirds"),
    description: "Composition style",
};
static P_LIGHTING: ParamDef = ParamDef {
    name: "lighting",
    param_type: ParamType::String,
    required: false,
    default_value: Some("natural"),
    description: "Lighting preset",
};
static P_DOF: ParamDef = ParamDef {
    name: "depth_of_field",
    param_type: ParamType::F32,
    required: false,
    default_value: Some("2.0"),
    description: "Depth of field (f-stop)",
};
static P_NAME: ParamDef = ParamDef {
    name: "name",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Output name",
};
static P_SHOT_TYPE: ParamDef = ParamDef {
    name: "shot_type",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Shot type (orbit, follow, dolly, crane)",
};
static P_SPEED: ParamDef = ParamDef {
    name: "speed",
    param_type: ParamType::F32,
    required: false,
    default_value: Some("1.0"),
    description: "Camera movement speed multiplier",
};
static P_PRESET: ParamDef = ParamDef {
    name: "preset",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Quality/style preset",
};
static P_SIZE: ParamDef = ParamDef {
    name: "size",
    param_type: ParamType::String,
    required: false,
    default_value: Some("1920x1080"),
    description: "Output resolution",
};
static P_QUALITY: ParamDef = ParamDef {
    name: "quality",
    param_type: ParamType::String,
    required: false,
    default_value: Some("high"),
    description: "Render quality level",
};
static P_EFFECTS: ParamDef = ParamDef {
    name: "effects",
    param_type: ParamType::StringArray,
    required: false,
    default_value: None,
    description: "Post-processing effects list",
};
static P_DURATION: ParamDef = ParamDef {
    name: "duration",
    param_type: ParamType::F32,
    required: true,
    default_value: None,
    description: "Video duration in seconds",
};
static P_FPS: ParamDef = ParamDef {
    name: "fps",
    param_type: ParamType::U32,
    required: false,
    default_value: Some("30"),
    description: "Frames per second",
};
static P_SKILL_ID: ParamDef = ParamDef {
    name: "skill_id",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Skill ID to generate tutorial for",
};
static P_STYLE: ParamDef = ParamDef {
    name: "style",
    param_type: ParamType::String,
    required: false,
    default_value: Some("standard"),
    description: "Tutorial style (standard, quick, detailed)",
};
static P_LANGUAGE: ParamDef = ParamDef {
    name: "language",
    param_type: ParamType::String,
    required: false,
    default_value: Some("en"),
    description: "Language code for narration",
};
static P_CAMERA_STYLE: ParamDef = ParamDef {
    name: "camera_style",
    param_type: ParamType::String,
    required: false,
    default_value: Some("orbit"),
    description: "Camera capture style",
};
static P_SCRIPT_ID: ParamDef = ParamDef {
    name: "script_id",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Tutorial script ID",
};
static P_VOICE: ParamDef = ParamDef {
    name: "voice",
    param_type: ParamType::String,
    required: false,
    default_value: Some("en_US-lessac-medium"),
    description: "TTS voice identifier",
};
static P_FOOTAGE_PATH: ParamDef = ParamDef {
    name: "footage_path",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Path to captured footage",
};
static P_NARRATION_PATH: ParamDef = ParamDef {
    name: "narration_path",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Path to TTS audio file",
};
static P_VIDEO_PATH: ParamDef = ParamDef {
    name: "video_path",
    param_type: ParamType::String,
    required: true,
    default_value: None,
    description: "Path to video file",
};
static P_TIMESTAMP: ParamDef = ParamDef {
    name: "timestamp",
    param_type: ParamType::F32,
    required: false,
    default_value: Some("5.0"),
    description: "Video timestamp for thumbnail",
};
static P_DOMAIN_FILTER: ParamDef = ParamDef {
    name: "domain_filter",
    param_type: ParamType::String,
    required: false,
    default_value: None,
    description: "Filter by domain (or all if omitted)",
};
static P_FORCE_REGEN: ParamDef = ParamDef {
    name: "force_regen",
    param_type: ParamType::Bool,
    required: false,
    default_value: Some("false"),
    description: "Force regeneration of existing tutorials",
};

pub static COMPOSE_FILM: SkillDef = SkillDef {
    id: "compose_film",
    domain: SkillDomain::Media,
    display_name: "Compose Film",
    description: "Set up a film scene with camera, lighting, and backdrop",
    params: &[P_SCENE_NAME, P_DESCRIPTION],
    returns: ReturnType::LocalIds,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 170",
    tags: &["media", "film", "cinema"],
    examples: &[],
};

pub static COMPOSE_MUSIC: SkillDef = SkillDef {
    id: "compose_music",
    domain: SkillDomain::Media,
    display_name: "Compose Music",
    description: "Create a music composition notecard",
    params: &[P_TITLE, P_DESCRIPTION],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 170",
    tags: &["media", "music", "audio"],
    examples: &[],
};

pub static COMPOSE_AD: SkillDef = SkillDef {
    id: "compose_ad",
    domain: SkillDomain::Media,
    display_name: "Compose Ad",
    description: "Create an advertisement display board",
    params: &[P_BOARD_NAME, P_DESCRIPTION],
    returns: ReturnType::LocalId,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 170",
    tags: &["media", "advertising", "display"],
    examples: &[],
};

pub static COMPOSE_PHOTO: SkillDef = SkillDef {
    id: "compose_photo",
    domain: SkillDomain::Media,
    display_name: "Compose Photo",
    description: "Set up a photographic composition with camera angle and lighting",
    params: &[
        P_SUBJECT_POS,
        P_CAMERA_ANGLE,
        P_COMPOSITION,
        P_LIGHTING,
        P_DOF,
        P_NAME,
    ],
    returns: ReturnType::LocalIds,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 170",
    tags: &["media", "photo", "composition"],
    examples: &[],
};

pub static DRONE_CINEMATOGRAPHY: SkillDef = SkillDef {
    id: "drone_cinematography",
    domain: SkillDomain::Media,
    display_name: "Drone Cinematography",
    description: "Create a cinematic drone camera sequence with waypoints and lighting",
    params: &[
        P_SCENE_NAME,
        P_SHOT_TYPE,
        P_SUBJECT_POS,
        P_SPEED,
        P_LIGHTING,
    ],
    returns: ReturnType::LocalIds,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 160",
    tags: &["media", "drone", "cinematography", "camera"],
    examples: &[],
};

pub static LUXOR_SNAPSHOT: SkillDef = SkillDef {
    id: "luxor_snapshot",
    domain: SkillDomain::Media,
    display_name: "Luxor Snapshot",
    description: "Capture a raytraced snapshot using the Luxor engine",
    params: &[
        P_PRESET,
        P_SIZE,
        P_QUALITY,
        P_EFFECTS,
        P_LIGHTING,
        P_SUBJECT_POS,
        P_NAME,
    ],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 171",
    tags: &["media", "luxor", "raytrace", "snapshot"],
    examples: &[],
};

pub static LUXOR_VIDEO: SkillDef = SkillDef {
    id: "luxor_video",
    domain: SkillDomain::Media,
    display_name: "Luxor Video",
    description: "Render a raytraced video sequence using the Luxor engine",
    params: &[
        P_SHOT_TYPE,
        P_DURATION,
        P_FPS,
        P_SIZE,
        P_QUALITY,
        P_EFFECTS,
        P_LIGHTING,
        P_SUBJECT_POS,
        P_NAME,
    ],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L7Production,
    phase: "Phase 172",
    tags: &["media", "luxor", "raytrace", "video"],
    examples: &[],
};

pub static GENERATE_TUTORIAL_SCRIPT: SkillDef = SkillDef {
    id: "generate_tutorial_script",
    domain: SkillDomain::Media,
    display_name: "Generate Tutorial Script",
    description: "Auto-generate a narrated tutorial script from skill metadata",
    params: &[P_SKILL_ID, P_STYLE, P_LANGUAGE],
    returns: ReturnType::ObjectData,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L0Seed,
    phase: "Phase 209",
    tags: &["media", "tutorial", "script", "documentation"],
    examples: &[],
};

pub static CAPTURE_SKILL_DEMO: SkillDef = SkillDef {
    id: "capture_skill_demo",
    domain: SkillDomain::Media,
    display_name: "Capture Skill Demo",
    description: "Capture in-world footage of a skill being executed",
    params: &[P_SKILL_ID, P_CAMERA_STYLE, P_DURATION],
    returns: ReturnType::Success,
    requires_region: true,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L0Seed,
    phase: "Phase 209",
    tags: &["media", "tutorial", "capture", "video"],
    examples: &[],
};

pub static NARRATE_TUTORIAL: SkillDef = SkillDef {
    id: "narrate_tutorial",
    domain: SkillDomain::Media,
    display_name: "Generate TTS Narration",
    description: "Generate text-to-speech narration from a tutorial script",
    params: &[P_SCRIPT_ID, P_VOICE, P_LANGUAGE],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L0Seed,
    phase: "Phase 209",
    tags: &["media", "tutorial", "tts", "narration"],
    examples: &[],
};

pub static COMPOSITE_TUTORIAL: SkillDef = SkillDef {
    id: "composite_tutorial",
    domain: SkillDomain::Media,
    display_name: "Composite Tutorial Video",
    description: "Stitch footage, narration, and overlays into a tutorial video",
    params: &[P_SCRIPT_ID, P_FOOTAGE_PATH, P_NARRATION_PATH],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L0Seed,
    phase: "Phase 209",
    tags: &["media", "tutorial", "composite", "video"],
    examples: &[],
};

pub static PUBLISH_TUTORIAL: SkillDef = SkillDef {
    id: "publish_tutorial",
    domain: SkillDomain::Media,
    display_name: "Publish Tutorial",
    description: "Publish a completed tutorial video to the skill catalog",
    params: &[P_SKILL_ID, P_VIDEO_PATH],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L0Seed,
    phase: "Phase 209",
    tags: &["media", "tutorial", "publish"],
    examples: &[],
};

pub static GENERATE_THUMBNAIL: SkillDef = SkillDef {
    id: "generate_thumbnail",
    domain: SkillDomain::Media,
    display_name: "Generate Video Thumbnail",
    description: "Extract a thumbnail image from a video at a given timestamp",
    params: &[P_VIDEO_PATH, P_TIMESTAMP],
    returns: ReturnType::Success,
    requires_region: false,
    requires_agent: true,
    requires_admin: false,
    maturity: SkillMaturity::L0Seed,
    phase: "Phase 209",
    tags: &["media", "tutorial", "thumbnail"],
    examples: &[],
};

pub static BATCH_GENERATE_TUTORIALS: SkillDef = SkillDef {
    id: "batch_generate_tutorials",
    domain: SkillDomain::Media,
    display_name: "Batch Generate Tutorials",
    description: "Generate tutorial scripts for all skills at L5 or above",
    params: &[P_DOMAIN_FILTER, P_FORCE_REGEN],
    returns: ReturnType::ObjectData,
    requires_region: false,
    requires_agent: true,
    requires_admin: true,
    maturity: SkillMaturity::L0Seed,
    phase: "Phase 209",
    tags: &["media", "tutorial", "batch", "automation"],
    examples: &[],
};

pub fn register(registry: &mut super::super::skill_engine::SkillRegistry) {
    registry.register(&COMPOSE_FILM);
    registry.register(&COMPOSE_MUSIC);
    registry.register(&COMPOSE_AD);
    registry.register(&COMPOSE_PHOTO);
    registry.register(&DRONE_CINEMATOGRAPHY);
    registry.register(&LUXOR_SNAPSHOT);
    registry.register(&LUXOR_VIDEO);
    registry.register(&GENERATE_TUTORIAL_SCRIPT);
    registry.register(&CAPTURE_SKILL_DEMO);
    registry.register(&NARRATE_TUTORIAL);
    registry.register(&COMPOSITE_TUTORIAL);
    registry.register(&PUBLISH_TUTORIAL);
    registry.register(&GENERATE_THUMBNAIL);
    registry.register(&BATCH_GENERATE_TUTORIALS);
}
