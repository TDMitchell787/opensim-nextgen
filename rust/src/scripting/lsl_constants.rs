pub const OS_APIVERSION: i32 = 23;

pub const TRUE: i32 = 1;
pub const FALSE: i32 = 0;

pub const STATUS_PHYSICS: i32 = 1;
pub const STATUS_ROTATE_X: i32 = 2;
pub const STATUS_ROTATE_Y: i32 = 4;
pub const STATUS_ROTATE_Z: i32 = 8;
pub const STATUS_PHANTOM: i32 = 16;
pub const STATUS_SANDBOX: i32 = 32;
pub const STATUS_BLOCK_GRAB: i32 = 64;
pub const STATUS_DIE_AT_EDGE: i32 = 128;
pub const STATUS_RETURN_AT_EDGE: i32 = 256;
pub const STATUS_CAST_SHADOWS: i32 = 512;
pub const STATUS_BLOCK_GRAB_OBJECT: i32 = 1024;

pub const AGENT: i32 = 1;
pub const AGENT_BY_LEGACY_NAME: i32 = 1;
pub const AGENT_BY_USERNAME: i32 = 0x10;
pub const NPC: i32 = 0x20;
pub const ACTIVE: i32 = 2;
pub const PASSIVE: i32 = 4;
pub const SCRIPTED: i32 = 8;

pub const CONTROL_FWD: i32 = 1;
pub const CONTROL_BACK: i32 = 2;
pub const CONTROL_LEFT: i32 = 4;
pub const CONTROL_RIGHT: i32 = 8;
pub const CONTROL_UP: i32 = 16;
pub const CONTROL_DOWN: i32 = 32;
pub const CONTROL_ROT_LEFT: i32 = 256;
pub const CONTROL_ROT_RIGHT: i32 = 512;
pub const CONTROL_LBUTTON: i32 = 268435456;
pub const CONTROL_ML_LBUTTON: i32 = 1073741824;

pub const PERMISSION_DEBIT: i32 = 0x02;
pub const PERMISSION_TAKE_CONTROLS: i32 = 0x04;
pub const PERMISSION_REMAP_CONTROLS: i32 = 0x08;
pub const PERMISSION_TRIGGER_ANIMATION: i32 = 0x010;
pub const PERMISSION_ATTACH: i32 = 0x20;
pub const PERMISSION_RELEASE_OWNERSHIP: i32 = 0x40;
pub const PERMISSION_CHANGE_LINKS: i32 = 0x80;
pub const PERMISSION_CHANGE_JOINTS: i32 = 0x100;
pub const PERMISSION_CHANGE_PERMISSIONS: i32 = 0x200;
pub const PERMISSION_TRACK_CAMERA: i32 = 0x400;
pub const PERMISSION_CONTROL_CAMERA: i32 = 0x800;
pub const PERMISSION_TELEPORT: i32 = 0x1000;
pub const PERMISSION_SILENT_ESTATE_MANAGEMENT: i32 = 0x4000;
pub const PERMISSION_OVERRIDE_ANIMATIONS: i32 = 0x8000;
pub const PERMISSION_RETURN_OBJECTS: i32 = 0x10000;

pub const AGENT_FLYING: i32 = 0x1;
pub const AGENT_ATTACHMENTS: i32 = 0x2;
pub const AGENT_SCRIPTED_CONST: i32 = 0x4;
pub const AGENT_MOUSELOOK: i32 = 0x8;
pub const AGENT_SITTING: i32 = 0x10;
pub const AGENT_ON_OBJECT: i32 = 0x20;
pub const AGENT_AWAY: i32 = 0x40;
pub const AGENT_WALKING: i32 = 0x80;
pub const AGENT_IN_AIR: i32 = 0x100;
pub const AGENT_TYPING: i32 = 0x200;
pub const AGENT_CROUCHING: i32 = 0x400;
pub const AGENT_BUSY: i32 = 0x800;
pub const AGENT_ALWAYS_RUN: i32 = 0x1000;
pub const AGENT_AUTOPILOT: i32 = 0x2000;
pub const AGENT_MALE: i32 = 0x40000000;

pub const PSYS_PART_INTERP_COLOR_MASK: i32 = 1;
pub const PSYS_PART_INTERP_SCALE_MASK: i32 = 2;
pub const PSYS_PART_BOUNCE_MASK: i32 = 4;
pub const PSYS_PART_WIND_MASK: i32 = 8;
pub const PSYS_PART_FOLLOW_SRC_MASK: i32 = 16;
pub const PSYS_PART_FOLLOW_VELOCITY_MASK: i32 = 32;
pub const PSYS_PART_TARGET_POS_MASK: i32 = 64;
pub const PSYS_PART_TARGET_LINEAR_MASK: i32 = 128;
pub const PSYS_PART_EMISSIVE_MASK: i32 = 256;
pub const PSYS_PART_RIBBON_MASK: i32 = 1024;
pub const PSYS_PART_FLAGS: i32 = 0;
pub const PSYS_PART_START_COLOR: i32 = 1;
pub const PSYS_PART_START_ALPHA: i32 = 2;
pub const PSYS_PART_END_COLOR: i32 = 3;
pub const PSYS_PART_END_ALPHA: i32 = 4;
pub const PSYS_PART_START_SCALE: i32 = 5;
pub const PSYS_PART_END_SCALE: i32 = 6;
pub const PSYS_PART_MAX_AGE: i32 = 7;
pub const PSYS_SRC_ACCEL: i32 = 8;
pub const PSYS_SRC_PATTERN: i32 = 9;
pub const PSYS_SRC_INNERANGLE: i32 = 10;
pub const PSYS_SRC_OUTERANGLE: i32 = 11;
pub const PSYS_SRC_TEXTURE: i32 = 12;
pub const PSYS_SRC_BURST_RATE: i32 = 13;
pub const PSYS_SRC_BURST_PART_COUNT: i32 = 15;
pub const PSYS_SRC_BURST_RADIUS: i32 = 16;
pub const PSYS_SRC_BURST_SPEED_MIN: i32 = 17;
pub const PSYS_SRC_BURST_SPEED_MAX: i32 = 18;
pub const PSYS_SRC_MAX_AGE: i32 = 19;
pub const PSYS_SRC_TARGET_KEY: i32 = 20;
pub const PSYS_SRC_OMEGA: i32 = 21;
pub const PSYS_SRC_ANGLE_BEGIN: i32 = 22;
pub const PSYS_SRC_ANGLE_END: i32 = 23;
pub const PSYS_PART_BLEND_FUNC_SOURCE: i32 = 24;
pub const PSYS_PART_BLEND_FUNC_DEST: i32 = 25;
pub const PSYS_PART_START_GLOW: i32 = 26;
pub const PSYS_PART_END_GLOW: i32 = 27;
pub const PSYS_PART_BF_ONE: i32 = 0;
pub const PSYS_PART_BF_ZERO: i32 = 1;
pub const PSYS_PART_BF_DEST_COLOR: i32 = 2;
pub const PSYS_PART_BF_SOURCE_COLOR: i32 = 3;
pub const PSYS_PART_BF_ONE_MINUS_DEST_COLOR: i32 = 4;
pub const PSYS_PART_BF_ONE_MINUS_SOURCE_COLOR: i32 = 5;
pub const PSYS_PART_BF_SOURCE_ALPHA: i32 = 7;
pub const PSYS_PART_BF_ONE_MINUS_SOURCE_ALPHA: i32 = 9;
pub const PSYS_SRC_PATTERN_DROP: i32 = 1;
pub const PSYS_SRC_PATTERN_EXPLODE: i32 = 2;
pub const PSYS_SRC_PATTERN_ANGLE: i32 = 4;
pub const PSYS_SRC_PATTERN_ANGLE_CONE: i32 = 8;
pub const PSYS_SRC_PATTERN_ANGLE_CONE_EMPTY: i32 = 16;

pub const VEHICLE_TYPE_NONE: i32 = 0;
pub const VEHICLE_TYPE_SLED: i32 = 1;
pub const VEHICLE_TYPE_CAR: i32 = 2;
pub const VEHICLE_TYPE_BOAT: i32 = 3;
pub const VEHICLE_TYPE_AIRPLANE: i32 = 4;
pub const VEHICLE_TYPE_BALLOON: i32 = 5;
pub const VEHICLE_LINEAR_FRICTION_TIMESCALE: i32 = 16;
pub const VEHICLE_ANGULAR_FRICTION_TIMESCALE: i32 = 17;
pub const VEHICLE_LINEAR_MOTOR_DIRECTION: i32 = 18;
pub const VEHICLE_LINEAR_MOTOR_OFFSET: i32 = 20;
pub const VEHICLE_ANGULAR_MOTOR_DIRECTION: i32 = 19;
pub const VEHICLE_HOVER_HEIGHT: i32 = 24;
pub const VEHICLE_HOVER_EFFICIENCY: i32 = 25;
pub const VEHICLE_HOVER_TIMESCALE: i32 = 26;
pub const VEHICLE_BUOYANCY: i32 = 27;
pub const VEHICLE_LINEAR_DEFLECTION_EFFICIENCY: i32 = 28;
pub const VEHICLE_LINEAR_DEFLECTION_TIMESCALE: i32 = 29;
pub const VEHICLE_LINEAR_MOTOR_TIMESCALE: i32 = 30;
pub const VEHICLE_LINEAR_MOTOR_DECAY_TIMESCALE: i32 = 31;
pub const VEHICLE_ANGULAR_DEFLECTION_EFFICIENCY: i32 = 32;
pub const VEHICLE_ANGULAR_DEFLECTION_TIMESCALE: i32 = 33;
pub const VEHICLE_ANGULAR_MOTOR_TIMESCALE: i32 = 34;
pub const VEHICLE_ANGULAR_MOTOR_DECAY_TIMESCALE: i32 = 35;
pub const VEHICLE_VERTICAL_ATTRACTION_EFFICIENCY: i32 = 36;
pub const VEHICLE_VERTICAL_ATTRACTION_TIMESCALE: i32 = 37;
pub const VEHICLE_BANKING_EFFICIENCY: i32 = 38;
pub const VEHICLE_BANKING_MIX: i32 = 39;
pub const VEHICLE_BANKING_TIMESCALE: i32 = 40;
pub const VEHICLE_REFERENCE_FRAME: i32 = 44;
pub const VEHICLE_RANGE_BLOCK: i32 = 45;
pub const VEHICLE_ROLL_FRAME: i32 = 46;
pub const VEHICLE_FLAG_NO_DEFLECTION_UP: i32 = 1;
pub const VEHICLE_FLAG_NO_FLY_UP: i32 = 1;
pub const VEHICLE_FLAG_LIMIT_ROLL_ONLY: i32 = 2;
pub const VEHICLE_FLAG_HOVER_WATER_ONLY: i32 = 4;
pub const VEHICLE_FLAG_HOVER_TERRAIN_ONLY: i32 = 8;
pub const VEHICLE_FLAG_HOVER_GLOBAL_HEIGHT: i32 = 16;
pub const VEHICLE_FLAG_HOVER_UP_ONLY: i32 = 32;
pub const VEHICLE_FLAG_LIMIT_MOTOR_UP: i32 = 64;
pub const VEHICLE_FLAG_MOUSELOOK_STEER: i32 = 128;
pub const VEHICLE_FLAG_MOUSELOOK_BANK: i32 = 256;
pub const VEHICLE_FLAG_CAMERA_DECOUPLED: i32 = 512;
pub const VEHICLE_FLAG_NO_X: i32 = 1024;
pub const VEHICLE_FLAG_NO_Y: i32 = 2048;
pub const VEHICLE_FLAG_NO_Z: i32 = 4096;
pub const VEHICLE_FLAG_LOCK_HOVER_HEIGHT: i32 = 8192;
pub const VEHICLE_FLAG_NO_DEFLECTION: i32 = 16392;
pub const VEHICLE_FLAG_LOCK_ROTATION: i32 = 32784;

pub const INVENTORY_ALL: i32 = -1;
pub const INVENTORY_NONE: i32 = -1;
pub const INVENTORY_TEXTURE: i32 = 0;
pub const INVENTORY_SOUND: i32 = 1;
pub const INVENTORY_LANDMARK: i32 = 3;
pub const INVENTORY_CLOTHING: i32 = 5;
pub const INVENTORY_OBJECT: i32 = 6;
pub const INVENTORY_NOTECARD: i32 = 7;
pub const INVENTORY_SCRIPT: i32 = 10;
pub const INVENTORY_BODYPART: i32 = 13;
pub const INVENTORY_ANIMATION: i32 = 20;
pub const INVENTORY_GESTURE: i32 = 21;
pub const INVENTORY_SETTING: i32 = 56;
pub const INVENTORY_MATERIAL: i32 = 57;

pub const ATTACH_CHEST: i32 = 1;
pub const ATTACH_HEAD: i32 = 2;
pub const ATTACH_LSHOULDER: i32 = 3;
pub const ATTACH_RSHOULDER: i32 = 4;
pub const ATTACH_LHAND: i32 = 5;
pub const ATTACH_RHAND: i32 = 6;
pub const ATTACH_LFOOT: i32 = 7;
pub const ATTACH_RFOOT: i32 = 8;
pub const ATTACH_BACK: i32 = 9;
pub const ATTACH_PELVIS: i32 = 10;
pub const ATTACH_MOUTH: i32 = 11;
pub const ATTACH_CHIN: i32 = 12;
pub const ATTACH_LEAR: i32 = 13;
pub const ATTACH_REAR: i32 = 14;
pub const ATTACH_LEYE: i32 = 15;
pub const ATTACH_REYE: i32 = 16;
pub const ATTACH_NOSE: i32 = 17;
pub const ATTACH_RUARM: i32 = 18;
pub const ATTACH_RLARM: i32 = 19;
pub const ATTACH_LUARM: i32 = 20;
pub const ATTACH_LLARM: i32 = 21;
pub const ATTACH_RHIP: i32 = 22;
pub const ATTACH_RULEG: i32 = 23;
pub const ATTACH_RLLEG: i32 = 24;
pub const ATTACH_LHIP: i32 = 25;
pub const ATTACH_LULEG: i32 = 26;
pub const ATTACH_LLLEG: i32 = 27;
pub const ATTACH_BELLY: i32 = 28;
pub const ATTACH_RPEC: i32 = 29;
pub const ATTACH_LPEC: i32 = 30;
pub const ATTACH_LEFT_PEC: i32 = 29;
pub const ATTACH_RIGHT_PEC: i32 = 30;
pub const ATTACH_HUD_CENTER_2: i32 = 31;
pub const ATTACH_HUD_TOP_RIGHT: i32 = 32;
pub const ATTACH_HUD_TOP_CENTER: i32 = 33;
pub const ATTACH_HUD_TOP_LEFT: i32 = 34;
pub const ATTACH_HUD_CENTER_1: i32 = 35;
pub const ATTACH_HUD_BOTTOM_LEFT: i32 = 36;
pub const ATTACH_HUD_BOTTOM: i32 = 37;
pub const ATTACH_HUD_BOTTOM_RIGHT: i32 = 38;
pub const ATTACH_NECK: i32 = 39;
pub const ATTACH_AVATAR_CENTER: i32 = 40;
pub const ATTACH_LHAND_RING1: i32 = 41;
pub const ATTACH_RHAND_RING1: i32 = 42;
pub const ATTACH_TAIL_BASE: i32 = 43;
pub const ATTACH_TAIL_TIP: i32 = 44;
pub const ATTACH_LWING: i32 = 45;
pub const ATTACH_RWING: i32 = 46;
pub const ATTACH_FACE_JAW: i32 = 47;
pub const ATTACH_FACE_LEAR: i32 = 48;
pub const ATTACH_FACE_REAR: i32 = 49;
pub const ATTACH_FACE_LEYE: i32 = 50;
pub const ATTACH_FACE_REYE: i32 = 51;
pub const ATTACH_FACE_TONGUE: i32 = 52;
pub const ATTACH_GROIN: i32 = 53;
pub const ATTACH_HIND_LFOOT: i32 = 54;
pub const ATTACH_HIND_RFOOT: i32 = 55;

pub const OS_ATTACH_MSG_ALL: i32 = -65535;
pub const OS_ATTACH_MSG_INVERT_POINTS: i32 = 1;
pub const OS_ATTACH_MSG_OBJECT_CREATOR: i32 = 2;
pub const OS_ATTACH_MSG_SCRIPT_CREATOR: i32 = 4;

pub const LAND_LEVEL: i32 = 0;
pub const LAND_RAISE: i32 = 1;
pub const LAND_LOWER: i32 = 2;
pub const LAND_SMOOTH: i32 = 3;
pub const LAND_NOISE: i32 = 4;
pub const LAND_REVERT: i32 = 5;
pub const LAND_SMALL_BRUSH: i32 = 1;
pub const LAND_MEDIUM_BRUSH: i32 = 2;
pub const LAND_LARGE_BRUSH: i32 = 3;

pub const DATA_ONLINE: i32 = 1;
pub const DATA_NAME: i32 = 2;
pub const DATA_BORN: i32 = 3;
pub const DATA_RATING: i32 = 4;
pub const DATA_SIM_POS: i32 = 5;
pub const DATA_SIM_STATUS: i32 = 6;
pub const DATA_SIM_RATING: i32 = 7;
pub const DATA_PAYINFO: i32 = 8;
pub const DATA_SIM_RELEASE: i32 = 128;

pub const ANIM_ON: i32 = 1;
pub const LOOP: i32 = 2;
pub const REVERSE: i32 = 4;
pub const PING_PONG: i32 = 8;
pub const SMOOTH: i32 = 16;
pub const ROTATE: i32 = 32;
pub const SCALE: i32 = 64;
pub const ALL_SIDES: i32 = -1;

pub const LINK_SET: i32 = -1;
pub const LINK_ROOT: i32 = 1;
pub const LINK_ALL_OTHERS: i32 = -2;
pub const LINK_ALL_CHILDREN: i32 = -3;
pub const LINK_THIS: i32 = -4;

pub const CHANGED_INVENTORY: i32 = 1;
pub const CHANGED_COLOR: i32 = 2;
pub const CHANGED_SHAPE: i32 = 4;
pub const CHANGED_SCALE: i32 = 8;
pub const CHANGED_TEXTURE: i32 = 16;
pub const CHANGED_LINK: i32 = 32;
pub const CHANGED_ALLOWED_DROP: i32 = 64;
pub const CHANGED_OWNER: i32 = 128;
pub const CHANGED_REGION: i32 = 256;
pub const CHANGED_TELEPORT: i32 = 512;
pub const CHANGED_REGION_RESTART: i32 = 1024;
pub const CHANGED_REGION_START: i32 = 1024;
pub const CHANGED_MEDIA: i32 = 2048;
pub const CHANGED_RENDER_MATERIAL: i32 = 0x1000;
pub const CHANGED_ANIMATION: i32 = 16384;
pub const CHANGED_POSITION: i32 = 32768;

pub const TYPE_INVALID: i32 = 0;
pub const TYPE_INTEGER: i32 = 1;
pub const TYPE_FLOAT: i32 = 2;
pub const TYPE_STRING: i32 = 3;
pub const TYPE_KEY: i32 = 4;
pub const TYPE_VECTOR: i32 = 5;
pub const TYPE_ROTATION: i32 = 6;

pub const REMOTE_DATA_CHANNEL: i32 = 1;
pub const REMOTE_DATA_REQUEST: i32 = 2;
pub const REMOTE_DATA_REPLY: i32 = 3;

pub const HTTP_METHOD: i32 = 0;
pub const HTTP_MIMETYPE: i32 = 1;
pub const HTTP_BODY_MAXLENGTH: i32 = 2;
pub const HTTP_VERIFY_CERT: i32 = 3;
pub const HTTP_VERBOSE_THROTTLE: i32 = 4;
pub const HTTP_CUSTOM_HEADER: i32 = 5;
pub const HTTP_PRAGMA_NO_CACHE: i32 = 6;

pub const CONTENT_TYPE_TEXT: i32 = 0;
pub const CONTENT_TYPE_HTML: i32 = 1;
pub const CONTENT_TYPE_XML: i32 = 2;
pub const CONTENT_TYPE_XHTML: i32 = 3;
pub const CONTENT_TYPE_ATOM: i32 = 4;
pub const CONTENT_TYPE_JSON: i32 = 5;
pub const CONTENT_TYPE_LLSD: i32 = 6;
pub const CONTENT_TYPE_FORM: i32 = 7;
pub const CONTENT_TYPE_RSS: i32 = 8;

pub const PRIM_MATERIAL: i32 = 2;
pub const PRIM_PHYSICS: i32 = 3;
pub const PRIM_TEMP_ON_REZ: i32 = 4;
pub const PRIM_PHANTOM: i32 = 5;
pub const PRIM_POSITION: i32 = 6;
pub const PRIM_SIZE: i32 = 7;
pub const PRIM_ROTATION: i32 = 8;
pub const PRIM_TYPE: i32 = 9;
pub const PRIM_TEXTURE: i32 = 17;
pub const PRIM_COLOR: i32 = 18;
pub const PRIM_BUMP_SHINY: i32 = 19;
pub const PRIM_FULLBRIGHT: i32 = 20;
pub const PRIM_FLEXIBLE: i32 = 21;
pub const PRIM_TEXGEN: i32 = 22;
pub const PRIM_POINT_LIGHT: i32 = 23;
pub const PRIM_CAST_SHADOWS: i32 = 24;
pub const PRIM_GLOW: i32 = 25;
pub const PRIM_TEXT: i32 = 26;
pub const PRIM_NAME: i32 = 27;
pub const PRIM_DESC: i32 = 28;
pub const PRIM_ROT_LOCAL: i32 = 29;
pub const PRIM_PHYSICS_SHAPE_TYPE: i32 = 30;
pub const PRIM_PHYSICS_MATERIAL: i32 = 31;
pub const PRIM_OMEGA: i32 = 32;
pub const PRIM_POS_LOCAL: i32 = 33;
pub const PRIM_LINK_TARGET: i32 = 34;
pub const PRIM_SLICE: i32 = 35;
pub const PRIM_SPECULAR: i32 = 36;
pub const PRIM_NORMAL: i32 = 37;
pub const PRIM_ALPHA_MODE: i32 = 38;
pub const PRIM_ALLOW_UNSIT: i32 = 39;
pub const PRIM_SCRIPTED_SIT_ONLY: i32 = 40;
pub const PRIM_SIT_TARGET: i32 = 41;
pub const PRIM_PROJECTOR: i32 = 42;
pub const PRIM_REFLECTION_PROBE: i32 = 44;
pub const PRIM_GLTF_NORMAL: i32 = 45;
pub const PRIM_GLTF_EMISSIVE: i32 = 46;
pub const PRIM_GLTF_METALLIC_ROUGHNESS: i32 = 47;
pub const PRIM_GLTF_BASE_COLOR: i32 = 48;
pub const PRIM_RENDER_MATERIAL: i32 = 49;

pub const PRIM_ALPHA_MODE_NONE: i32 = 0;
pub const PRIM_ALPHA_MODE_BLEND: i32 = 1;
pub const PRIM_ALPHA_MODE_MASK: i32 = 2;
pub const PRIM_ALPHA_MODE_EMISSIVE: i32 = 3;

pub const PRIM_TEXGEN_DEFAULT: i32 = 0;
pub const PRIM_TEXGEN_PLANAR: i32 = 1;

pub const PRIM_TYPE_BOX: i32 = 0;
pub const PRIM_TYPE_CYLINDER: i32 = 1;
pub const PRIM_TYPE_PRISM: i32 = 2;
pub const PRIM_TYPE_SPHERE: i32 = 3;
pub const PRIM_TYPE_TORUS: i32 = 4;
pub const PRIM_TYPE_TUBE: i32 = 5;
pub const PRIM_TYPE_RING: i32 = 6;
pub const PRIM_TYPE_SCULPT: i32 = 7;

pub const PRIM_HOLE_DEFAULT: i32 = 0;
pub const PRIM_HOLE_CIRCLE: i32 = 16;
pub const PRIM_HOLE_SQUARE: i32 = 32;
pub const PRIM_HOLE_TRIANGLE: i32 = 48;

pub const PRIM_MATERIAL_STONE: i32 = 0;
pub const PRIM_MATERIAL_METAL: i32 = 1;
pub const PRIM_MATERIAL_GLASS: i32 = 2;
pub const PRIM_MATERIAL_WOOD: i32 = 3;
pub const PRIM_MATERIAL_FLESH: i32 = 4;
pub const PRIM_MATERIAL_PLASTIC: i32 = 5;
pub const PRIM_MATERIAL_RUBBER: i32 = 6;
pub const PRIM_MATERIAL_LIGHT: i32 = 7;

pub const PRIM_SHINY_NONE: i32 = 0;
pub const PRIM_SHINY_LOW: i32 = 1;
pub const PRIM_SHINY_MEDIUM: i32 = 2;
pub const PRIM_SHINY_HIGH: i32 = 3;

pub const PRIM_BUMP_NONE: i32 = 0;
pub const PRIM_BUMP_BRIGHT: i32 = 1;
pub const PRIM_BUMP_DARK: i32 = 2;
pub const PRIM_BUMP_WOOD: i32 = 3;
pub const PRIM_BUMP_BARK: i32 = 4;
pub const PRIM_BUMP_BRICKS: i32 = 5;
pub const PRIM_BUMP_CHECKER: i32 = 6;
pub const PRIM_BUMP_CONCRETE: i32 = 7;
pub const PRIM_BUMP_TILE: i32 = 8;
pub const PRIM_BUMP_STONE: i32 = 9;
pub const PRIM_BUMP_DISKS: i32 = 10;
pub const PRIM_BUMP_GRAVEL: i32 = 11;
pub const PRIM_BUMP_BLOBS: i32 = 12;
pub const PRIM_BUMP_SIDING: i32 = 13;
pub const PRIM_BUMP_LARGETILE: i32 = 14;
pub const PRIM_BUMP_STUCCO: i32 = 15;
pub const PRIM_BUMP_SUCTION: i32 = 16;
pub const PRIM_BUMP_WEAVE: i32 = 17;

pub const PRIM_SCULPT_TYPE_SPHERE: i32 = 1;
pub const PRIM_SCULPT_TYPE_TORUS: i32 = 2;
pub const PRIM_SCULPT_TYPE_PLANE: i32 = 3;
pub const PRIM_SCULPT_TYPE_CYLINDER: i32 = 4;
pub const PRIM_SCULPT_TYPE_MESH: i32 = 5;
pub const PRIM_SCULPT_FLAG_ANIMESH: i32 = 0x20;
pub const PRIM_SCULPT_FLAG_INVERT: i32 = 0x40;
pub const PRIM_SCULPT_FLAG_MIRROR: i32 = 0x80;
pub const PRIM_SCULPT_TYPE_MASK: i32 = 0x07;

pub const PRIM_PHYSICS_SHAPE_PRIM: i32 = 0;
pub const PRIM_PHYSICS_SHAPE_NONE: i32 = 1;
pub const PRIM_PHYSICS_SHAPE_CONVEX: i32 = 2;

pub const PRIM_REFLECTION_PROBE_BOX: i32 = 1;
pub const PRIM_REFLECTION_PROBE_DYNAMIC: i32 = 2;
pub const PRIM_REFLECTION_PROBE_MIRROR: i32 = 4;

pub const PROFILE_NONE: i32 = 0;
pub const PROFILE_SCRIPT_MEMORY: i32 = 1;

pub const MASK_BASE: i32 = 0;
pub const MASK_OWNER: i32 = 1;
pub const MASK_GROUP: i32 = 2;
pub const MASK_EVERYONE: i32 = 3;
pub const MASK_NEXT: i32 = 4;

pub const PERM_TRANSFER: i32 = 0x2000;
pub const PERM_MODIFY: i32 = 0x4000;
pub const PERM_COPY: i32 = 0x8000;
pub const PERM_MOVE: i32 = 0x80000;
pub const PERM_ALL: i32 = 0x7fffffff;

pub const PARCEL_MEDIA_COMMAND_STOP: i32 = 0;
pub const PARCEL_MEDIA_COMMAND_PAUSE: i32 = 1;
pub const PARCEL_MEDIA_COMMAND_PLAY: i32 = 2;
pub const PARCEL_MEDIA_COMMAND_LOOP: i32 = 3;
pub const PARCEL_MEDIA_COMMAND_TEXTURE: i32 = 4;
pub const PARCEL_MEDIA_COMMAND_URL: i32 = 5;
pub const PARCEL_MEDIA_COMMAND_TIME: i32 = 6;
pub const PARCEL_MEDIA_COMMAND_AGENT: i32 = 7;
pub const PARCEL_MEDIA_COMMAND_UNLOAD: i32 = 8;
pub const PARCEL_MEDIA_COMMAND_AUTO_ALIGN: i32 = 9;
pub const PARCEL_MEDIA_COMMAND_TYPE: i32 = 10;
pub const PARCEL_MEDIA_COMMAND_SIZE: i32 = 11;
pub const PARCEL_MEDIA_COMMAND_DESC: i32 = 12;

pub const PARCEL_FLAG_ALLOW_FLY: i32 = 0x1;
pub const PARCEL_FLAG_ALLOW_SCRIPTS: i32 = 0x2;
pub const PARCEL_FLAG_ALLOW_LANDMARK: i32 = 0x8;
pub const PARCEL_FLAG_ALLOW_TERRAFORM: i32 = 0x10;
pub const PARCEL_FLAG_ALLOW_DAMAGE: i32 = 0x20;
pub const PARCEL_FLAG_ALLOW_CREATE_OBJECTS: i32 = 0x40;
pub const PARCEL_FLAG_USE_ACCESS_GROUP: i32 = 0x100;
pub const PARCEL_FLAG_USE_ACCESS_LIST: i32 = 0x200;
pub const PARCEL_FLAG_USE_BAN_LIST: i32 = 0x400;
pub const PARCEL_FLAG_USE_LAND_PASS_LIST: i32 = 0x800;
pub const PARCEL_FLAG_LOCAL_SOUND_ONLY: i32 = 0x8000;
pub const PARCEL_FLAG_RESTRICT_PUSHOBJECT: i32 = 0x200000;
pub const PARCEL_FLAG_ALLOW_GROUP_SCRIPTS: i32 = 0x2000000;
pub const PARCEL_FLAG_ALLOW_CREATE_GROUP_OBJECTS: i32 = 0x4000000;
pub const PARCEL_FLAG_ALLOW_ALL_OBJECT_ENTRY: i32 = 0x8000000;
pub const PARCEL_FLAG_ALLOW_GROUP_OBJECT_ENTRY: i32 = 0x10000000;

pub const REGION_FLAG_ALLOW_DAMAGE: i32 = 0x1;
pub const REGION_FLAG_FIXED_SUN: i32 = 0x10;
pub const REGION_FLAG_BLOCK_TERRAFORM: i32 = 0x40;
pub const REGION_FLAG_SANDBOX: i32 = 0x100;
pub const REGION_FLAG_DISABLE_COLLISIONS: i32 = 0x1000;
pub const REGION_FLAG_DISABLE_PHYSICS: i32 = 0x4000;
pub const REGION_FLAG_BLOCK_FLY: i32 = 0x80000;
pub const REGION_FLAG_ALLOW_DIRECT_TELEPORT: i32 = 0x100000;
pub const REGION_FLAG_RESTRICT_PUSHOBJECT: i32 = 0x400000;

pub const ESTATE_ACCESS_ALLOWED_AGENT_ADD: i32 = 0;
pub const ESTATE_ACCESS_ALLOWED_AGENT_REMOVE: i32 = 1;
pub const ESTATE_ACCESS_ALLOWED_GROUP_ADD: i32 = 2;
pub const ESTATE_ACCESS_ALLOWED_GROUP_REMOVE: i32 = 3;
pub const ESTATE_ACCESS_BANNED_AGENT_ADD: i32 = 4;
pub const ESTATE_ACCESS_BANNED_AGENT_REMOVE: i32 = 5;

pub const PAY_HIDE: i32 = -1;
pub const PAY_DEFAULT: i32 = -2;

pub const NULL_KEY: &str = "00000000-0000-0000-0000-000000000000";
pub const EOF: &str = "\n\n\n";
pub const NAK: &str = "\n\u{0015}\n";
pub const PI: f64 = 3.14159274_f64;
pub const TWO_PI: f64 = 6.28318548_f64;
pub const PI_BY_TWO: f64 = 1.57079637_f64;
pub const DEG_TO_RAD: f64 = 0.01745329238_f64;
pub const RAD_TO_DEG: f64 = 57.29578_f64;
pub const SQRT2: f64 = 1.414213538_f64;

pub const STRING_TRIM_HEAD: i32 = 1;
pub const STRING_TRIM_TAIL: i32 = 2;
pub const STRING_TRIM: i32 = 3;

pub const LIST_STAT_RANGE: i32 = 0;
pub const LIST_STAT_MIN: i32 = 1;
pub const LIST_STAT_MAX: i32 = 2;
pub const LIST_STAT_MEAN: i32 = 3;
pub const LIST_STAT_MEDIAN: i32 = 4;
pub const LIST_STAT_STD_DEV: i32 = 5;
pub const LIST_STAT_SUM: i32 = 6;
pub const LIST_STAT_SUM_SQUARES: i32 = 7;
pub const LIST_STAT_NUM_COUNT: i32 = 8;
pub const LIST_STAT_GEOMETRIC_MEAN: i32 = 9;
pub const LIST_STAT_HARMONIC_MEAN: i32 = 100;

pub const PARCEL_COUNT_TOTAL: i32 = 0;
pub const PARCEL_COUNT_OWNER: i32 = 1;
pub const PARCEL_COUNT_GROUP: i32 = 2;
pub const PARCEL_COUNT_OTHER: i32 = 3;
pub const PARCEL_COUNT_SELECTED: i32 = 4;
pub const PARCEL_COUNT_TEMP: i32 = 5;

pub const DEBUG_CHANNEL: i32 = 0x7FFFFFFF;
pub const PUBLIC_CHANNEL: i32 = 0x00000000;

pub const OBJECT_UNKNOWN_DETAIL: i32 = -1;
pub const OBJECT_NAME: i32 = 1;
pub const OBJECT_DESC: i32 = 2;
pub const OBJECT_POS: i32 = 3;
pub const OBJECT_ROT: i32 = 4;
pub const OBJECT_VELOCITY: i32 = 5;
pub const OBJECT_OWNER: i32 = 6;
pub const OBJECT_GROUP: i32 = 7;
pub const OBJECT_CREATOR: i32 = 8;
pub const OBJECT_RUNNING_SCRIPT_COUNT: i32 = 9;
pub const OBJECT_TOTAL_SCRIPT_COUNT: i32 = 10;
pub const OBJECT_SCRIPT_MEMORY: i32 = 11;
pub const OBJECT_SCRIPT_TIME: i32 = 12;
pub const OBJECT_PRIM_EQUIVALENCE: i32 = 13;
pub const OBJECT_SERVER_COST: i32 = 14;
pub const OBJECT_STREAMING_COST: i32 = 15;
pub const OBJECT_PHYSICS_COST: i32 = 16;
pub const OBJECT_CHARACTER_TIME: i32 = 17;
pub const OBJECT_ROOT: i32 = 18;
pub const OBJECT_ATTACHED_POINT: i32 = 19;
pub const OBJECT_PATHFINDING_TYPE: i32 = 20;
pub const OBJECT_PHYSICS: i32 = 21;
pub const OBJECT_PHANTOM: i32 = 22;
pub const OBJECT_TEMP_ON_REZ: i32 = 23;
pub const OBJECT_RENDER_WEIGHT: i32 = 24;
pub const OBJECT_HOVER_HEIGHT: i32 = 25;
pub const OBJECT_BODY_SHAPE_TYPE: i32 = 26;
pub const OBJECT_LAST_OWNER_ID: i32 = 27;
pub const OBJECT_CLICK_ACTION: i32 = 28;
pub const OBJECT_OMEGA: i32 = 29;
pub const OBJECT_PRIM_COUNT: i32 = 30;
pub const OBJECT_TOTAL_INVENTORY_COUNT: i32 = 31;
pub const OBJECT_REZZER_KEY: i32 = 32;
pub const OBJECT_GROUP_TAG: i32 = 33;
pub const OBJECT_TEMP_ATTACHED: i32 = 34;
pub const OBJECT_ATTACHED_SLOTS_AVAILABLE: i32 = 35;
pub const OBJECT_CREATION_TIME: i32 = 36;
pub const OBJECT_SELECT_COUNT: i32 = 37;
pub const OBJECT_SIT_COUNT: i32 = 38;
pub const OBJECT_ANIMATED_COUNT: i32 = 39;
pub const OBJECT_ANIMATED_SLOTS_AVAILABLE: i32 = 40;
pub const OBJECT_ACCOUNT_LEVEL: i32 = 41;
pub const OBJECT_MATERIAL: i32 = 42;
pub const OBJECT_MASS: i32 = 43;
pub const OBJECT_TEXT: i32 = 44;
pub const OBJECT_REZ_TIME: i32 = 45;
pub const OBJECT_LINK_NUMBER: i32 = 46;
pub const OBJECT_SCALE: i32 = 47;
pub const OBJECT_TEXT_COLOR: i32 = 48;
pub const OBJECT_TEXT_ALPHA: i32 = 49;

pub const OPT_OTHER: i32 = -1;
pub const OPT_LEGACY_LINKSET: i32 = 0;
pub const OPT_AVATAR: i32 = 1;
pub const OPT_CHARACTER: i32 = 2;
pub const OPT_WALKABLE: i32 = 3;
pub const OPT_STATIC_OBSTACLE: i32 = 4;
pub const OPT_MATERIAL_VOLUME: i32 = 5;
pub const OPT_EXCLUSION_VOLUME: i32 = 6;

pub const AGENT_LIST_PARCEL: i32 = 0x1;
pub const AGENT_LIST_PARCEL_OWNER: i32 = 2;
pub const AGENT_LIST_REGION: i32 = 4;
pub const AGENT_LIST_EXCLUDENPC: i32 = 0x4000000;

pub const CAMERA_PITCH: i32 = 0;
pub const CAMERA_FOCUS_OFFSET: i32 = 1;
pub const CAMERA_FOCUS_OFFSET_X: i32 = 2;
pub const CAMERA_FOCUS_OFFSET_Y: i32 = 3;
pub const CAMERA_FOCUS_OFFSET_Z: i32 = 4;
pub const CAMERA_POSITION_LAG: i32 = 5;
pub const CAMERA_FOCUS_LAG: i32 = 6;
pub const CAMERA_DISTANCE: i32 = 7;
pub const CAMERA_BEHINDNESS_ANGLE: i32 = 8;
pub const CAMERA_BEHINDNESS_LAG: i32 = 9;
pub const CAMERA_POSITION_THRESHOLD: i32 = 10;
pub const CAMERA_FOCUS_THRESHOLD: i32 = 11;
pub const CAMERA_ACTIVE: i32 = 12;
pub const CAMERA_POSITION: i32 = 13;
pub const CAMERA_POSITION_X: i32 = 14;
pub const CAMERA_POSITION_Y: i32 = 15;
pub const CAMERA_POSITION_Z: i32 = 16;
pub const CAMERA_FOCUS: i32 = 17;
pub const CAMERA_FOCUS_X: i32 = 18;
pub const CAMERA_FOCUS_Y: i32 = 19;
pub const CAMERA_FOCUS_Z: i32 = 20;
pub const CAMERA_POSITION_LOCKED: i32 = 21;
pub const CAMERA_FOCUS_LOCKED: i32 = 22;

pub const PARCEL_DETAILS_NAME: i32 = 0;
pub const PARCEL_DETAILS_DESC: i32 = 1;
pub const PARCEL_DETAILS_OWNER: i32 = 2;
pub const PARCEL_DETAILS_GROUP: i32 = 3;
pub const PARCEL_DETAILS_AREA: i32 = 4;
pub const PARCEL_DETAILS_ID: i32 = 5;
pub const PARCEL_DETAILS_SEE_AVATARS: i32 = 6;
pub const PARCEL_DETAILS_PRIM_CAPACITY: i32 = 7;
pub const PARCEL_DETAILS_PRIM_USED: i32 = 8;
pub const PARCEL_DETAILS_LANDING_POINT: i32 = 9;
pub const PARCEL_DETAILS_LANDING_LOOKAT: i32 = 10;
pub const PARCEL_DETAILS_TP_ROUTING: i32 = 11;
pub const PARCEL_DETAILS_FLAGS: i32 = 12;
pub const PARCEL_DETAILS_SCRIPT_DANGER: i32 = 13;
pub const PARCEL_DETAILS_DWELL: i32 = 64;
pub const PARCEL_DETAILS_GETCLAIMDATE: i32 = 65;
pub const PARCEL_DETAILS_GEOMETRICCENTER: i32 = 66;

pub const CLICK_ACTION_NONE: i32 = 0;
pub const CLICK_ACTION_TOUCH: i32 = 0;
pub const CLICK_ACTION_SIT: i32 = 1;
pub const CLICK_ACTION_BUY: i32 = 2;
pub const CLICK_ACTION_PAY: i32 = 3;
pub const CLICK_ACTION_OPEN: i32 = 4;
pub const CLICK_ACTION_PLAY: i32 = 5;
pub const CLICK_ACTION_OPEN_MEDIA: i32 = 6;
pub const CLICK_ACTION_ZOOM: i32 = 7;
pub const CLICK_ACTION_DISABLED: i32 = 8;

pub const TOUCH_INVALID_FACE: i32 = -1;

pub const PRIM_MEDIA_ALT_IMAGE_ENABLE: i32 = 0;
pub const PRIM_MEDIA_CONTROLS: i32 = 1;
pub const PRIM_MEDIA_CURRENT_URL: i32 = 2;
pub const PRIM_MEDIA_HOME_URL: i32 = 3;
pub const PRIM_MEDIA_AUTO_LOOP: i32 = 4;
pub const PRIM_MEDIA_AUTO_PLAY: i32 = 5;
pub const PRIM_MEDIA_AUTO_SCALE: i32 = 6;
pub const PRIM_MEDIA_AUTO_ZOOM: i32 = 7;
pub const PRIM_MEDIA_FIRST_CLICK_INTERACT: i32 = 8;
pub const PRIM_MEDIA_WIDTH_PIXELS: i32 = 9;
pub const PRIM_MEDIA_HEIGHT_PIXELS: i32 = 10;
pub const PRIM_MEDIA_WHITELIST_ENABLE: i32 = 11;
pub const PRIM_MEDIA_WHITELIST: i32 = 12;
pub const PRIM_MEDIA_PERMS_INTERACT: i32 = 13;
pub const PRIM_MEDIA_PERMS_CONTROL: i32 = 14;
pub const PRIM_MEDIA_CONTROLS_STANDARD: i32 = 0;
pub const PRIM_MEDIA_CONTROLS_MINI: i32 = 1;
pub const PRIM_MEDIA_PERM_NONE: i32 = 0;
pub const PRIM_MEDIA_PERM_OWNER: i32 = 1;
pub const PRIM_MEDIA_PERM_GROUP: i32 = 2;
pub const PRIM_MEDIA_PERM_ANYONE: i32 = 4;

pub const DENSITY: i32 = 1;
pub const FRICTION: i32 = 2;
pub const RESTITUTION: i32 = 4;
pub const GRAVITY_MULTIPLIER: i32 = 8;

pub const LSL_STATUS_OK: i32 = 0;
pub const LSL_STATUS_MALFORMED_PARAMS: i32 = 1000;
pub const LSL_STATUS_TYPE_MISMATCH: i32 = 1001;
pub const LSL_STATUS_BOUNDS_ERROR: i32 = 1002;
pub const LSL_STATUS_NOT_FOUND: i32 = 1003;
pub const LSL_STATUS_NOT_SUPPORTED: i32 = 1004;
pub const LSL_STATUS_INTERNAL_ERROR: i32 = 1999;
pub const LSL_STATUS_WHITELIST_FAILED: i32 = 2001;

pub const TEXTURE_BLANK: &str = "5748decc-f629-461c-9a36-a35a221fe21f";
pub const TEXTURE_DEFAULT: &str = "89556747-24cb-43ed-920b-47caed15465f";
pub const TEXTURE_PLYWOOD: &str = "89556747-24cb-43ed-920b-47caed15465f";
pub const TEXTURE_TRANSPARENT: &str = "8dcd4a48-2d37-4909-9f78-f7a9eb4ef903";
pub const TEXTURE_MEDIA: &str = "8b5fec65-8d8d-9dc5-cda8-8fdf2716e361";

pub const STATS_TIME_DILATION: i32 = 0;
pub const STATS_SIM_FPS: i32 = 1;
pub const STATS_PHYSICS_FPS: i32 = 2;
pub const STATS_AGENT_UPDATES: i32 = 3;
pub const STATS_ROOT_AGENTS: i32 = 4;
pub const STATS_CHILD_AGENTS: i32 = 5;
pub const STATS_TOTAL_PRIMS: i32 = 6;
pub const STATS_ACTIVE_PRIMS: i32 = 7;
pub const STATS_FRAME_MS: i32 = 8;
pub const STATS_NET_MS: i32 = 9;
pub const STATS_PHYSICS_MS: i32 = 10;
pub const STATS_IMAGE_MS: i32 = 11;
pub const STATS_OTHER_MS: i32 = 12;
pub const STATS_IN_PACKETS_PER_SECOND: i32 = 13;
pub const STATS_OUT_PACKETS_PER_SECOND: i32 = 14;
pub const STATS_UNACKED_BYTES: i32 = 15;
pub const STATS_AGENT_MS: i32 = 16;
pub const STATS_PENDING_DOWNLOADS: i32 = 17;
pub const STATS_PENDING_UPLOADS: i32 = 18;
pub const STATS_ACTIVE_SCRIPTS: i32 = 19;
pub const STATS_SIM_SLEEP: i32 = 20;
pub const STATS_SCRIPT_EPS: i32 = 28;
pub const STATS_SCRIPT_TIME: i32 = 37;
pub const STATS_SCRIPT_LPS: i32 = 38;
pub const STATS_SCRIPT_NPCS: i32 = 47;

pub const OS_NPC_FLY: i32 = 0;
pub const OS_NPC_NO_FLY: i32 = 1;
pub const OS_NPC_LAND_AT_TARGET: i32 = 2;
pub const OS_NPC_RUNNING: i32 = 4;
pub const OS_NPC_SIT_NOW: i32 = 0;
pub const OS_NPC_CREATOR_OWNED: i32 = 0x1;
pub const OS_NPC_NOT_OWNED: i32 = 0x2;
pub const OS_NPC_SENSE_AS_AGENT: i32 = 0x4;
pub const OS_NPC_OBJECT_GROUP: i32 = 0x08;

pub const URL_REQUEST_GRANTED: &str = "URL_REQUEST_GRANTED";
pub const URL_REQUEST_DENIED: &str = "URL_REQUEST_DENIED";

pub const RC_REJECT_TYPES: i32 = 0;
pub const RC_DETECT_PHANTOM: i32 = 1;
pub const RC_DATA_FLAGS: i32 = 2;
pub const RC_MAX_HITS: i32 = 3;
pub const RC_REJECT_AGENTS: i32 = 1;
pub const RC_REJECT_PHYSICAL: i32 = 2;
pub const RC_REJECT_NONPHYSICAL: i32 = 4;
pub const RC_REJECT_LAND: i32 = 8;
pub const RC_REJECT_HOST: i32 = 0x20000000;
pub const RC_REJECT_HOSTGROUP: i32 = 0x40000000;
pub const RC_GET_NORMAL: i32 = 1;
pub const RC_GET_ROOT_KEY: i32 = 2;
pub const RC_GET_LINK_NUM: i32 = 4;
pub const RCERR_UNKNOWN: i32 = -1;
pub const RCERR_SIM_PERF_LOW: i32 = -2;
pub const RCERR_CAST_TIME_EXCEEDED: i32 = -3;

pub const KFM_MODE: i32 = 1;
pub const KFM_LOOP: i32 = 1;
pub const KFM_REVERSE: i32 = 3;
pub const KFM_FORWARD: i32 = 0;
pub const KFM_PING_PONG: i32 = 2;
pub const KFM_DATA: i32 = 2;
pub const KFM_TRANSLATION: i32 = 2;
pub const KFM_ROTATION: i32 = 1;
pub const KFM_COMMAND: i32 = 0;
pub const KFM_CMD_PLAY: i32 = 0;
pub const KFM_CMD_STOP: i32 = 1;
pub const KFM_CMD_PAUSE: i32 = 2;

pub const JSON_INVALID: &str = "\u{FDD0}";
pub const JSON_OBJECT: &str = "\u{FDD1}";
pub const JSON_ARRAY: &str = "\u{FDD2}";
pub const JSON_NUMBER: &str = "\u{FDD3}";
pub const JSON_STRING: &str = "\u{FDD4}";
pub const JSON_NULL: &str = "\u{FDD5}";
pub const JSON_TRUE: &str = "\u{FDD6}";
pub const JSON_FALSE: &str = "\u{FDD7}";
pub const JSON_DELETE: &str = "\u{FDD8}";
pub const JSON_APPEND: &str = "-1";

pub const OS_LISTEN_REGEX_NAME: i32 = 0x1;
pub const OS_LISTEN_REGEX_MESSAGE: i32 = 0x2;

pub const OSTPOBJ_NONE: i32 = 0x0;
pub const OSTPOBJ_STOPATTARGET: i32 = 0x1;
pub const OSTPOBJ_STOPONFAIL: i32 = 0x2;
pub const OSTPOBJ_SETROT: i32 = 0x4;

pub const OS_LTPAG_NONE: i32 = 0x0;
pub const OS_LTPAG_USEVEL: i32 = 0x1;
pub const OS_LTPAG_USELOOKAT: i32 = 0x2;
pub const OS_LTPAG_ALGNLV: i32 = 0x4;
pub const OS_LTPAG_FORCEFLY: i32 = 0x8;
pub const OS_LTPAG_FORCENOFLY: i32 = 0x16;

pub const WL_WATER_COLOR: i32 = 0;
pub const WL_WATER_FOG_DENSITY_EXPONENT: i32 = 1;
pub const WL_UNDERWATER_FOG_MODIFIER: i32 = 2;
pub const WL_REFLECTION_WAVELET_SCALE: i32 = 3;
pub const WL_FRESNEL_SCALE: i32 = 4;
pub const WL_FRESNEL_OFFSET: i32 = 5;
pub const WL_REFRACT_SCALE_ABOVE: i32 = 6;
pub const WL_REFRACT_SCALE_BELOW: i32 = 7;
pub const WL_BLUR_MULTIPLIER: i32 = 8;
pub const WL_BIG_WAVE_DIRECTION: i32 = 9;
pub const WL_LITTLE_WAVE_DIRECTION: i32 = 10;
pub const WL_NORMAL_MAP_TEXTURE: i32 = 11;
pub const WL_HORIZON: i32 = 12;
pub const WL_HAZE_HORIZON: i32 = 13;
pub const WL_BLUE_DENSITY: i32 = 14;
pub const WL_HAZE_DENSITY: i32 = 15;
pub const WL_DENSITY_MULTIPLIER: i32 = 16;
pub const WL_DISTANCE_MULTIPLIER: i32 = 17;
pub const WL_MAX_ALTITUDE: i32 = 18;
pub const WL_SUN_MOON_COLOR: i32 = 19;
pub const WL_AMBIENT: i32 = 20;
pub const WL_EAST_ANGLE: i32 = 21;
pub const WL_SUN_GLOW_FOCUS: i32 = 22;
pub const WL_SUN_GLOW_SIZE: i32 = 23;
pub const WL_SCENE_GAMMA: i32 = 24;
pub const WL_STAR_BRIGHTNESS: i32 = 25;
pub const WL_CLOUD_COLOR: i32 = 26;
pub const WL_CLOUD_XY_DENSITY: i32 = 27;
pub const WL_CLOUD_COVERAGE: i32 = 28;
pub const WL_CLOUD_SCALE: i32 = 29;
pub const WL_CLOUD_DETAIL_XY_DENSITY: i32 = 30;
pub const WL_CLOUD_SCROLL_X: i32 = 31;
pub const WL_CLOUD_SCROLL_Y: i32 = 32;
pub const WL_CLOUD_SCROLL_Y_LOCK: i32 = 33;
pub const WL_CLOUD_SCROLL_X_LOCK: i32 = 34;
pub const WL_DRAW_CLASSIC_CLOUDS: i32 = 35;
pub const WL_SUN_MOON_POSITION: i32 = 36;

pub const IMG_USE_BAKED_HEAD: &str = "5a9f4a74-30f2-821c-b88d-70499d3e7183";
pub const IMG_USE_BAKED_UPPER: &str = "ae2de45c-d252-50b8-5c6e-19f39ce79317";
pub const IMG_USE_BAKED_LOWER: &str = "24daea5f-0539-cfcf-047f-fbc40b2786ba";
pub const IMG_USE_BAKED_EYES: &str = "52cc6bb6-2ee5-e632-d3ad-50197b1dcb8a";
pub const IMG_USE_BAKED_SKIRT: &str = "43529ce8-7faa-ad92-165a-bc4078371687";
pub const IMG_USE_BAKED_HAIR: &str = "09aac1fb-6bce-0bee-7d44-caac6dbb6c63";
pub const IMG_USE_BAKED_LEFTARM: &str = "ff62763f-d60a-9855-890b-0c96f8f8cd98";
pub const IMG_USE_BAKED_LEFTLEG: &str = "8e915e25-31d1-cc95-ae08-d58a47488251";
pub const IMG_USE_BAKED_AUX1: &str = "9742065b-19b5-297c-858a-29711d539043";
pub const IMG_USE_BAKED_AUX2: &str = "03642e83-2bd1-4eb9-34b4-4c47ed586d2d";
pub const IMG_USE_BAKED_AUX3: &str = "edd51b77-fc10-ce7a-4b3d-011dfc349e4f";

pub const TARGETED_EMAIL_ROOT_CREATOR: i32 = 1;
pub const TARGETED_EMAIL_OBJECT_OWNER: i32 = 2;

pub const NPCLOOKAT_NONE: i32 = 0;
pub const NPCLOOKAT_IDLE: i32 = 1;
pub const NPCLOOKAT_LISTEN: i32 = 2;
pub const NPCLOOKAT_FREELOOK: i32 = 3;
pub const NPCLOOKAT_RESPOND: i32 = 4;
pub const NPCLOOKAT_HOVER: i32 = 5;
pub const NPCLOOKAT_CONVERSATION: i32 = 6;
pub const NPCLOOKAT_SELECT: i32 = 7;
pub const NPCLOOKAT_FOCUS: i32 = 8;
pub const NPCLOOKAT_MOUSELOOK: i32 = 9;
pub const NPCLOOKAT_CLEAR: i32 = 10;

pub const LINKSETDATA_RESET: i32 = 0;
pub const LINKSETDATA_UPDATE: i32 = 1;
pub const LINKSETDATA_DELETE: i32 = 2;
pub const LINKSETDATA_MULTIDELETE: i32 = 3;
pub const LINKSETDATA_OK: i32 = 0;
pub const LINKSETDATA_EMEMORY: i32 = 1;
pub const LINKSETDATA_ENOKEY: i32 = 2;
pub const LINKSETDATA_EPROTECTED: i32 = 3;
pub const LINKSETDATA_NOTFOUND: i32 = 4;
pub const LINKSETDATA_NOUPDATE: i32 = 5;

pub const SOUND_PLAY: i32 = 0;
pub const SOUND_LOOP: i32 = 1;
pub const SOUND_TRIGGER: i32 = 2;
pub const SOUND_SYNC: i32 = 4;

pub const REZ_PARAM: i32 = 0;
pub const REZ_FLAGS: i32 = 1;
pub const REZ_FLAG_TEMP: i32 = 0x0001;
pub const REZ_FLAG_PHYSICAL: i32 = 0x0002;
pub const REZ_FLAG_PHANTOM: i32 = 0x0004;
pub const REZ_FLAG_DIE_ON_COLLIDE: i32 = 0x0008;
pub const REZ_FLAG_DIE_ON_NOENTRY: i32 = 0x0010;
pub const REZ_FLAG_NO_COLLIDE_OWNER: i32 = 0x0020;
pub const REZ_FLAG_NO_COLLIDE_FAMILY: i32 = 0x0040;
pub const REZ_FLAG_BLOCK_GRAB_OBJECT: i32 = 0x0080;
pub const REZ_POS: i32 = 2;
pub const REZ_ROT: i32 = 3;
pub const REZ_VEL: i32 = 4;
pub const REZ_ACCEL: i32 = 5;
pub const REZ_OMEGA: i32 = 7;
pub const REZ_DAMAGE: i32 = 8;
pub const REZ_SOUND: i32 = 9;
pub const REZ_SOUND_COLLIDE: i32 = 10;
pub const REZ_LOCK_AXES: i32 = 11;
pub const REZ_DAMAGE_TYPE: i32 = 12;
pub const REZ_PARAM_STRING: i32 = 13;

pub const SIT_FLAG_SIT_TARGET: i32 = 0x01;
pub const SIT_FLAG_ALLOW_UNSIT: i32 = 0x02;
pub const SIT_FLAG_SCRIPTED_ONLY: i32 = 0x04;
pub const SIT_FLAG_NO_COLLIDE: i32 = 0x10;
pub const SIT_FLAG_NO_DAMAGE: i32 = 0x20;
pub const SIT_FLAG_OPENSIMFORCED: i32 =
    SIT_FLAG_ALLOW_UNSIT | SIT_FLAG_NO_COLLIDE | SIT_FLAG_NO_DAMAGE;

pub const ERR_GENERIC: i32 = -1;
pub const ERR_PARCEL_PERMISSIONS: i32 = -2;
pub const ERR_MALFORMED_PARAMS: i32 = -3;
pub const ERR_RUNTIME_PERMISSIONS: i32 = -4;
pub const ERR_THROTTLED: i32 = -5;

pub const SIM_STAT_PCT_CHARS_STEPPED: i32 = 0;
pub const SIM_STAT_PHYSICS_FPS: i32 = 1;
pub const SIM_STAT_AGENT_UPDATES: i32 = 2;
pub const SIM_STAT_FRAME_MS: i32 = 3;
pub const SIM_STAT_NET_MS: i32 = 4;
pub const SIM_STAT_OTHER_MS: i32 = 5;
pub const SIM_STAT_PHYSICS_MS: i32 = 6;
pub const SIM_STAT_AGENT_MS: i32 = 7;
pub const SIM_STAT_IMAGE_MS: i32 = 8;
pub const SIM_STAT_SCRIPT_MS: i32 = 9;
pub const SIM_STAT_AGENT_COUNT: i32 = 10;
pub const SIM_STAT_CHILD_AGENT_COUNT: i32 = 11;
pub const SIM_STAT_ACTIVE_SCRIPT_COUNT: i32 = 12;
pub const SIM_STAT_PACKETS_IN: i32 = 13;
pub const SIM_STAT_PACKETS_OUT: i32 = 14;
pub const SIM_STAT_ASSET_DOWNLOADS: i32 = 15;
pub const SIM_STAT_ASSET_UPLOADS: i32 = 16;
pub const SIM_STAT_UNACKED_BYTES: i32 = 17;
pub const SIM_STAT_PHYSICS_STEP_MS: i32 = 18;
pub const SIM_STAT_PHYSICS_SHAPE_MS: i32 = 19;
pub const SIM_STAT_PHYSICS_OTHER_MS: i32 = 20;
pub const SIM_STAT_SCRIPT_EPS: i32 = 21;
pub const SIM_STAT_SPARE_MS: i32 = 22;
pub const SIM_STAT_SLEEP_MS: i32 = 23;
pub const SIM_STAT_IO_PUMP_MS: i32 = 24;
pub const SIM_STAT_SCRIPT_RUN_PCT: i32 = 25;
pub const SIM_STAT_AI_MS: i32 = 26;

use std::collections::HashMap;

pub fn build_constant_map() -> HashMap<&'static str, super::lsl_types::LSLValue> {
    use super::lsl_types::{LSLRotation, LSLValue, LSLVector};
    let mut m = HashMap::new();

    m.insert("TRUE", LSLValue::Integer(TRUE));
    m.insert("FALSE", LSLValue::Integer(FALSE));
    m.insert("STATUS_PHYSICS", LSLValue::Integer(STATUS_PHYSICS));
    m.insert("STATUS_ROTATE_X", LSLValue::Integer(STATUS_ROTATE_X));
    m.insert("STATUS_ROTATE_Y", LSLValue::Integer(STATUS_ROTATE_Y));
    m.insert("STATUS_ROTATE_Z", LSLValue::Integer(STATUS_ROTATE_Z));
    m.insert("STATUS_PHANTOM", LSLValue::Integer(STATUS_PHANTOM));
    m.insert("STATUS_SANDBOX", LSLValue::Integer(STATUS_SANDBOX));
    m.insert("STATUS_BLOCK_GRAB", LSLValue::Integer(STATUS_BLOCK_GRAB));
    m.insert("STATUS_DIE_AT_EDGE", LSLValue::Integer(STATUS_DIE_AT_EDGE));
    m.insert(
        "STATUS_RETURN_AT_EDGE",
        LSLValue::Integer(STATUS_RETURN_AT_EDGE),
    );
    m.insert(
        "STATUS_CAST_SHADOWS",
        LSLValue::Integer(STATUS_CAST_SHADOWS),
    );
    m.insert(
        "STATUS_BLOCK_GRAB_OBJECT",
        LSLValue::Integer(STATUS_BLOCK_GRAB_OBJECT),
    );
    m.insert("AGENT", LSLValue::Integer(AGENT));
    m.insert("ACTIVE", LSLValue::Integer(ACTIVE));
    m.insert("PASSIVE", LSLValue::Integer(PASSIVE));
    m.insert("SCRIPTED", LSLValue::Integer(SCRIPTED));
    m.insert("NPC", LSLValue::Integer(NPC));
    m.insert("CONTROL_FWD", LSLValue::Integer(CONTROL_FWD));
    m.insert("CONTROL_BACK", LSLValue::Integer(CONTROL_BACK));
    m.insert("CONTROL_LEFT", LSLValue::Integer(CONTROL_LEFT));
    m.insert("CONTROL_RIGHT", LSLValue::Integer(CONTROL_RIGHT));
    m.insert("CONTROL_UP", LSLValue::Integer(CONTROL_UP));
    m.insert("CONTROL_DOWN", LSLValue::Integer(CONTROL_DOWN));
    m.insert("CONTROL_ROT_LEFT", LSLValue::Integer(CONTROL_ROT_LEFT));
    m.insert("CONTROL_ROT_RIGHT", LSLValue::Integer(CONTROL_ROT_RIGHT));
    m.insert("CONTROL_LBUTTON", LSLValue::Integer(CONTROL_LBUTTON));
    m.insert("CONTROL_ML_LBUTTON", LSLValue::Integer(CONTROL_ML_LBUTTON));
    m.insert("PERMISSION_DEBIT", LSLValue::Integer(PERMISSION_DEBIT));
    m.insert(
        "PERMISSION_TAKE_CONTROLS",
        LSLValue::Integer(PERMISSION_TAKE_CONTROLS),
    );
    m.insert(
        "PERMISSION_TRIGGER_ANIMATION",
        LSLValue::Integer(PERMISSION_TRIGGER_ANIMATION),
    );
    m.insert("PERMISSION_ATTACH", LSLValue::Integer(PERMISSION_ATTACH));
    m.insert(
        "PERMISSION_CHANGE_LINKS",
        LSLValue::Integer(PERMISSION_CHANGE_LINKS),
    );
    m.insert(
        "PERMISSION_TRACK_CAMERA",
        LSLValue::Integer(PERMISSION_TRACK_CAMERA),
    );
    m.insert(
        "PERMISSION_CONTROL_CAMERA",
        LSLValue::Integer(PERMISSION_CONTROL_CAMERA),
    );
    m.insert(
        "PERMISSION_TELEPORT",
        LSLValue::Integer(PERMISSION_TELEPORT),
    );
    m.insert(
        "PERMISSION_OVERRIDE_ANIMATIONS",
        LSLValue::Integer(PERMISSION_OVERRIDE_ANIMATIONS),
    );
    m.insert("PI", LSLValue::Float(PI as f32));
    m.insert("TWO_PI", LSLValue::Float(TWO_PI as f32));
    m.insert("PI_BY_TWO", LSLValue::Float(PI_BY_TWO as f32));
    m.insert("DEG_TO_RAD", LSLValue::Float(DEG_TO_RAD as f32));
    m.insert("RAD_TO_DEG", LSLValue::Float(RAD_TO_DEG as f32));
    m.insert("SQRT2", LSLValue::Float(SQRT2 as f32));
    m.insert("DEBUG_CHANNEL", LSLValue::Integer(DEBUG_CHANNEL));
    m.insert("PUBLIC_CHANNEL", LSLValue::Integer(PUBLIC_CHANNEL));
    m.insert("ALL_SIDES", LSLValue::Integer(ALL_SIDES));
    m.insert("LINK_SET", LSLValue::Integer(LINK_SET));
    m.insert("LINK_ROOT", LSLValue::Integer(LINK_ROOT));
    m.insert("LINK_ALL_OTHERS", LSLValue::Integer(LINK_ALL_OTHERS));
    m.insert("LINK_ALL_CHILDREN", LSLValue::Integer(LINK_ALL_CHILDREN));
    m.insert("LINK_THIS", LSLValue::Integer(LINK_THIS));
    m.insert("CHANGED_INVENTORY", LSLValue::Integer(CHANGED_INVENTORY));
    m.insert("CHANGED_COLOR", LSLValue::Integer(CHANGED_COLOR));
    m.insert("CHANGED_SHAPE", LSLValue::Integer(CHANGED_SHAPE));
    m.insert("CHANGED_SCALE", LSLValue::Integer(CHANGED_SCALE));
    m.insert("CHANGED_TEXTURE", LSLValue::Integer(CHANGED_TEXTURE));
    m.insert("CHANGED_LINK", LSLValue::Integer(CHANGED_LINK));
    m.insert("CHANGED_OWNER", LSLValue::Integer(CHANGED_OWNER));
    m.insert("CHANGED_REGION", LSLValue::Integer(CHANGED_REGION));
    m.insert("CHANGED_TELEPORT", LSLValue::Integer(CHANGED_TELEPORT));
    m.insert("NULL_KEY", LSLValue::String(NULL_KEY.to_string()));
    m.insert("EOF", LSLValue::String(EOF.to_string()));
    m.insert("ZERO_VECTOR", LSLValue::Vector(LSLVector::zero()));
    m.insert(
        "ZERO_ROTATION",
        LSLValue::Rotation(LSLRotation::new(0.0, 0.0, 0.0, 1.0)),
    );
    m.insert("TEXTURE_BLANK", LSLValue::String(TEXTURE_BLANK.to_string()));
    m.insert(
        "TEXTURE_DEFAULT",
        LSLValue::String(TEXTURE_DEFAULT.to_string()),
    );
    m.insert(
        "TEXTURE_PLYWOOD",
        LSLValue::String(TEXTURE_PLYWOOD.to_string()),
    );
    m.insert(
        "TEXTURE_TRANSPARENT",
        LSLValue::String(TEXTURE_TRANSPARENT.to_string()),
    );
    m.insert("TEXTURE_MEDIA", LSLValue::String(TEXTURE_MEDIA.to_string()));

    m.insert(
        "CHANGED_ALLOWED_DROP",
        LSLValue::Integer(CHANGED_ALLOWED_DROP),
    );
    m.insert(
        "CHANGED_REGION_RESTART",
        LSLValue::Integer(CHANGED_REGION_RESTART),
    );
    m.insert(
        "CHANGED_REGION_START",
        LSLValue::Integer(CHANGED_REGION_START),
    );
    m.insert("CHANGED_MEDIA", LSLValue::Integer(CHANGED_MEDIA));
    m.insert(
        "CHANGED_RENDER_MATERIAL",
        LSLValue::Integer(CHANGED_RENDER_MATERIAL),
    );
    m.insert("CHANGED_ANIMATION", LSLValue::Integer(CHANGED_ANIMATION));
    m.insert("CHANGED_POSITION", LSLValue::Integer(CHANGED_POSITION));

    m.insert("VEHICLE_TYPE_NONE", LSLValue::Integer(VEHICLE_TYPE_NONE));
    m.insert("VEHICLE_TYPE_SLED", LSLValue::Integer(VEHICLE_TYPE_SLED));
    m.insert("VEHICLE_TYPE_CAR", LSLValue::Integer(VEHICLE_TYPE_CAR));
    m.insert("VEHICLE_TYPE_BOAT", LSLValue::Integer(VEHICLE_TYPE_BOAT));
    m.insert(
        "VEHICLE_TYPE_AIRPLANE",
        LSLValue::Integer(VEHICLE_TYPE_AIRPLANE),
    );
    m.insert(
        "VEHICLE_TYPE_BALLOON",
        LSLValue::Integer(VEHICLE_TYPE_BALLOON),
    );
    m.insert(
        "VEHICLE_LINEAR_FRICTION_TIMESCALE",
        LSLValue::Integer(VEHICLE_LINEAR_FRICTION_TIMESCALE),
    );
    m.insert(
        "VEHICLE_ANGULAR_FRICTION_TIMESCALE",
        LSLValue::Integer(VEHICLE_ANGULAR_FRICTION_TIMESCALE),
    );
    m.insert(
        "VEHICLE_LINEAR_MOTOR_DIRECTION",
        LSLValue::Integer(VEHICLE_LINEAR_MOTOR_DIRECTION),
    );
    m.insert(
        "VEHICLE_LINEAR_MOTOR_OFFSET",
        LSLValue::Integer(VEHICLE_LINEAR_MOTOR_OFFSET),
    );
    m.insert(
        "VEHICLE_ANGULAR_MOTOR_DIRECTION",
        LSLValue::Integer(VEHICLE_ANGULAR_MOTOR_DIRECTION),
    );
    m.insert(
        "VEHICLE_HOVER_HEIGHT",
        LSLValue::Integer(VEHICLE_HOVER_HEIGHT),
    );
    m.insert(
        "VEHICLE_HOVER_EFFICIENCY",
        LSLValue::Integer(VEHICLE_HOVER_EFFICIENCY),
    );
    m.insert(
        "VEHICLE_HOVER_TIMESCALE",
        LSLValue::Integer(VEHICLE_HOVER_TIMESCALE),
    );
    m.insert("VEHICLE_BUOYANCY", LSLValue::Integer(VEHICLE_BUOYANCY));
    m.insert(
        "VEHICLE_LINEAR_DEFLECTION_EFFICIENCY",
        LSLValue::Integer(VEHICLE_LINEAR_DEFLECTION_EFFICIENCY),
    );
    m.insert(
        "VEHICLE_LINEAR_DEFLECTION_TIMESCALE",
        LSLValue::Integer(VEHICLE_LINEAR_DEFLECTION_TIMESCALE),
    );
    m.insert(
        "VEHICLE_LINEAR_MOTOR_TIMESCALE",
        LSLValue::Integer(VEHICLE_LINEAR_MOTOR_TIMESCALE),
    );
    m.insert(
        "VEHICLE_LINEAR_MOTOR_DECAY_TIMESCALE",
        LSLValue::Integer(VEHICLE_LINEAR_MOTOR_DECAY_TIMESCALE),
    );
    m.insert(
        "VEHICLE_ANGULAR_DEFLECTION_EFFICIENCY",
        LSLValue::Integer(VEHICLE_ANGULAR_DEFLECTION_EFFICIENCY),
    );
    m.insert(
        "VEHICLE_ANGULAR_DEFLECTION_TIMESCALE",
        LSLValue::Integer(VEHICLE_ANGULAR_DEFLECTION_TIMESCALE),
    );
    m.insert(
        "VEHICLE_ANGULAR_MOTOR_TIMESCALE",
        LSLValue::Integer(VEHICLE_ANGULAR_MOTOR_TIMESCALE),
    );
    m.insert(
        "VEHICLE_ANGULAR_MOTOR_DECAY_TIMESCALE",
        LSLValue::Integer(VEHICLE_ANGULAR_MOTOR_DECAY_TIMESCALE),
    );
    m.insert(
        "VEHICLE_VERTICAL_ATTRACTION_EFFICIENCY",
        LSLValue::Integer(VEHICLE_VERTICAL_ATTRACTION_EFFICIENCY),
    );
    m.insert(
        "VEHICLE_VERTICAL_ATTRACTION_TIMESCALE",
        LSLValue::Integer(VEHICLE_VERTICAL_ATTRACTION_TIMESCALE),
    );
    m.insert(
        "VEHICLE_BANKING_EFFICIENCY",
        LSLValue::Integer(VEHICLE_BANKING_EFFICIENCY),
    );
    m.insert(
        "VEHICLE_BANKING_MIX",
        LSLValue::Integer(VEHICLE_BANKING_MIX),
    );
    m.insert(
        "VEHICLE_BANKING_TIMESCALE",
        LSLValue::Integer(VEHICLE_BANKING_TIMESCALE),
    );
    m.insert(
        "VEHICLE_REFERENCE_FRAME",
        LSLValue::Integer(VEHICLE_REFERENCE_FRAME),
    );
    m.insert(
        "VEHICLE_RANGE_BLOCK",
        LSLValue::Integer(VEHICLE_RANGE_BLOCK),
    );
    m.insert("VEHICLE_ROLL_FRAME", LSLValue::Integer(VEHICLE_ROLL_FRAME));
    m.insert(
        "VEHICLE_FLAG_NO_DEFLECTION_UP",
        LSLValue::Integer(VEHICLE_FLAG_NO_DEFLECTION_UP),
    );
    m.insert(
        "VEHICLE_FLAG_NO_FLY_UP",
        LSLValue::Integer(VEHICLE_FLAG_NO_FLY_UP),
    );
    m.insert(
        "VEHICLE_FLAG_LIMIT_ROLL_ONLY",
        LSLValue::Integer(VEHICLE_FLAG_LIMIT_ROLL_ONLY),
    );
    m.insert(
        "VEHICLE_FLAG_HOVER_WATER_ONLY",
        LSLValue::Integer(VEHICLE_FLAG_HOVER_WATER_ONLY),
    );
    m.insert(
        "VEHICLE_FLAG_HOVER_TERRAIN_ONLY",
        LSLValue::Integer(VEHICLE_FLAG_HOVER_TERRAIN_ONLY),
    );
    m.insert(
        "VEHICLE_FLAG_HOVER_GLOBAL_HEIGHT",
        LSLValue::Integer(VEHICLE_FLAG_HOVER_GLOBAL_HEIGHT),
    );
    m.insert(
        "VEHICLE_FLAG_HOVER_UP_ONLY",
        LSLValue::Integer(VEHICLE_FLAG_HOVER_UP_ONLY),
    );
    m.insert(
        "VEHICLE_FLAG_LIMIT_MOTOR_UP",
        LSLValue::Integer(VEHICLE_FLAG_LIMIT_MOTOR_UP),
    );
    m.insert(
        "VEHICLE_FLAG_MOUSELOOK_STEER",
        LSLValue::Integer(VEHICLE_FLAG_MOUSELOOK_STEER),
    );
    m.insert(
        "VEHICLE_FLAG_MOUSELOOK_BANK",
        LSLValue::Integer(VEHICLE_FLAG_MOUSELOOK_BANK),
    );
    m.insert(
        "VEHICLE_FLAG_CAMERA_DECOUPLED",
        LSLValue::Integer(VEHICLE_FLAG_CAMERA_DECOUPLED),
    );
    m.insert("VEHICLE_FLAG_NO_X", LSLValue::Integer(VEHICLE_FLAG_NO_X));
    m.insert("VEHICLE_FLAG_NO_Y", LSLValue::Integer(VEHICLE_FLAG_NO_Y));
    m.insert("VEHICLE_FLAG_NO_Z", LSLValue::Integer(VEHICLE_FLAG_NO_Z));
    m.insert(
        "VEHICLE_FLAG_LOCK_HOVER_HEIGHT",
        LSLValue::Integer(VEHICLE_FLAG_LOCK_HOVER_HEIGHT),
    );
    m.insert(
        "VEHICLE_FLAG_NO_DEFLECTION",
        LSLValue::Integer(VEHICLE_FLAG_NO_DEFLECTION),
    );
    m.insert(
        "VEHICLE_FLAG_LOCK_ROTATION",
        LSLValue::Integer(VEHICLE_FLAG_LOCK_ROTATION),
    );

    m.insert("CAMERA_PITCH", LSLValue::Integer(CAMERA_PITCH));
    m.insert(
        "CAMERA_FOCUS_OFFSET",
        LSLValue::Integer(CAMERA_FOCUS_OFFSET),
    );
    m.insert(
        "CAMERA_FOCUS_OFFSET_X",
        LSLValue::Integer(CAMERA_FOCUS_OFFSET_X),
    );
    m.insert(
        "CAMERA_FOCUS_OFFSET_Y",
        LSLValue::Integer(CAMERA_FOCUS_OFFSET_Y),
    );
    m.insert(
        "CAMERA_FOCUS_OFFSET_Z",
        LSLValue::Integer(CAMERA_FOCUS_OFFSET_Z),
    );
    m.insert(
        "CAMERA_POSITION_LAG",
        LSLValue::Integer(CAMERA_POSITION_LAG),
    );
    m.insert("CAMERA_FOCUS_LAG", LSLValue::Integer(CAMERA_FOCUS_LAG));
    m.insert("CAMERA_DISTANCE", LSLValue::Integer(CAMERA_DISTANCE));
    m.insert(
        "CAMERA_BEHINDNESS_ANGLE",
        LSLValue::Integer(CAMERA_BEHINDNESS_ANGLE),
    );
    m.insert(
        "CAMERA_BEHINDNESS_LAG",
        LSLValue::Integer(CAMERA_BEHINDNESS_LAG),
    );
    m.insert(
        "CAMERA_POSITION_THRESHOLD",
        LSLValue::Integer(CAMERA_POSITION_THRESHOLD),
    );
    m.insert(
        "CAMERA_FOCUS_THRESHOLD",
        LSLValue::Integer(CAMERA_FOCUS_THRESHOLD),
    );
    m.insert("CAMERA_ACTIVE", LSLValue::Integer(CAMERA_ACTIVE));
    m.insert("CAMERA_POSITION", LSLValue::Integer(CAMERA_POSITION));
    m.insert("CAMERA_POSITION_X", LSLValue::Integer(CAMERA_POSITION_X));
    m.insert("CAMERA_POSITION_Y", LSLValue::Integer(CAMERA_POSITION_Y));
    m.insert("CAMERA_POSITION_Z", LSLValue::Integer(CAMERA_POSITION_Z));
    m.insert("CAMERA_FOCUS", LSLValue::Integer(CAMERA_FOCUS));
    m.insert("CAMERA_FOCUS_X", LSLValue::Integer(CAMERA_FOCUS_X));
    m.insert("CAMERA_FOCUS_Y", LSLValue::Integer(CAMERA_FOCUS_Y));
    m.insert("CAMERA_FOCUS_Z", LSLValue::Integer(CAMERA_FOCUS_Z));
    m.insert(
        "CAMERA_POSITION_LOCKED",
        LSLValue::Integer(CAMERA_POSITION_LOCKED),
    );
    m.insert(
        "CAMERA_FOCUS_LOCKED",
        LSLValue::Integer(CAMERA_FOCUS_LOCKED),
    );

    m.insert("PRIM_MATERIAL", LSLValue::Integer(PRIM_MATERIAL));
    m.insert("PRIM_PHYSICS", LSLValue::Integer(PRIM_PHYSICS));
    m.insert("PRIM_TEMP_ON_REZ", LSLValue::Integer(PRIM_TEMP_ON_REZ));
    m.insert("PRIM_PHANTOM", LSLValue::Integer(PRIM_PHANTOM));
    m.insert("PRIM_POSITION", LSLValue::Integer(PRIM_POSITION));
    m.insert("PRIM_SIZE", LSLValue::Integer(PRIM_SIZE));
    m.insert("PRIM_ROTATION", LSLValue::Integer(PRIM_ROTATION));
    m.insert("PRIM_TYPE", LSLValue::Integer(PRIM_TYPE));
    m.insert("PRIM_TEXTURE", LSLValue::Integer(PRIM_TEXTURE));
    m.insert("PRIM_COLOR", LSLValue::Integer(PRIM_COLOR));
    m.insert("PRIM_BUMP_SHINY", LSLValue::Integer(PRIM_BUMP_SHINY));
    m.insert("PRIM_FULLBRIGHT", LSLValue::Integer(PRIM_FULLBRIGHT));
    m.insert("PRIM_FLEXIBLE", LSLValue::Integer(PRIM_FLEXIBLE));
    m.insert("PRIM_TEXGEN", LSLValue::Integer(PRIM_TEXGEN));
    m.insert("PRIM_POINT_LIGHT", LSLValue::Integer(PRIM_POINT_LIGHT));
    m.insert("PRIM_GLOW", LSLValue::Integer(PRIM_GLOW));
    m.insert("PRIM_TEXT", LSLValue::Integer(PRIM_TEXT));
    m.insert("PRIM_NAME", LSLValue::Integer(PRIM_NAME));
    m.insert("PRIM_DESC", LSLValue::Integer(PRIM_DESC));
    m.insert("PRIM_ROT_LOCAL", LSLValue::Integer(PRIM_ROT_LOCAL));
    m.insert(
        "PRIM_PHYSICS_SHAPE_TYPE",
        LSLValue::Integer(PRIM_PHYSICS_SHAPE_TYPE),
    );
    m.insert("PRIM_OMEGA", LSLValue::Integer(PRIM_OMEGA));
    m.insert("PRIM_POS_LOCAL", LSLValue::Integer(PRIM_POS_LOCAL));
    m.insert("PRIM_LINK_TARGET", LSLValue::Integer(PRIM_LINK_TARGET));
    m.insert("PRIM_SLICE", LSLValue::Integer(PRIM_SLICE));
    m.insert("PRIM_SPECULAR", LSLValue::Integer(PRIM_SPECULAR));
    m.insert("PRIM_NORMAL", LSLValue::Integer(PRIM_NORMAL));
    m.insert("PRIM_ALPHA_MODE", LSLValue::Integer(PRIM_ALPHA_MODE));
    m.insert("PRIM_SIT_TARGET", LSLValue::Integer(PRIM_SIT_TARGET));
    m.insert("PRIM_TYPE_BOX", LSLValue::Integer(PRIM_TYPE_BOX));
    m.insert("PRIM_TYPE_CYLINDER", LSLValue::Integer(PRIM_TYPE_CYLINDER));
    m.insert("PRIM_TYPE_PRISM", LSLValue::Integer(PRIM_TYPE_PRISM));
    m.insert("PRIM_TYPE_SPHERE", LSLValue::Integer(PRIM_TYPE_SPHERE));
    m.insert("PRIM_TYPE_TORUS", LSLValue::Integer(PRIM_TYPE_TORUS));
    m.insert("PRIM_TYPE_TUBE", LSLValue::Integer(PRIM_TYPE_TUBE));
    m.insert("PRIM_TYPE_RING", LSLValue::Integer(PRIM_TYPE_RING));
    m.insert("PRIM_TYPE_SCULPT", LSLValue::Integer(PRIM_TYPE_SCULPT));
    m.insert(
        "PRIM_MATERIAL_STONE",
        LSLValue::Integer(PRIM_MATERIAL_STONE),
    );
    m.insert(
        "PRIM_MATERIAL_METAL",
        LSLValue::Integer(PRIM_MATERIAL_METAL),
    );
    m.insert(
        "PRIM_MATERIAL_GLASS",
        LSLValue::Integer(PRIM_MATERIAL_GLASS),
    );
    m.insert("PRIM_MATERIAL_WOOD", LSLValue::Integer(PRIM_MATERIAL_WOOD));
    m.insert(
        "PRIM_MATERIAL_FLESH",
        LSLValue::Integer(PRIM_MATERIAL_FLESH),
    );
    m.insert(
        "PRIM_MATERIAL_PLASTIC",
        LSLValue::Integer(PRIM_MATERIAL_PLASTIC),
    );
    m.insert(
        "PRIM_MATERIAL_RUBBER",
        LSLValue::Integer(PRIM_MATERIAL_RUBBER),
    );
    m.insert(
        "PRIM_MATERIAL_LIGHT",
        LSLValue::Integer(PRIM_MATERIAL_LIGHT),
    );
    m.insert("PRIM_SHINY_NONE", LSLValue::Integer(PRIM_SHINY_NONE));
    m.insert("PRIM_SHINY_LOW", LSLValue::Integer(PRIM_SHINY_LOW));
    m.insert("PRIM_SHINY_MEDIUM", LSLValue::Integer(PRIM_SHINY_MEDIUM));
    m.insert("PRIM_SHINY_HIGH", LSLValue::Integer(PRIM_SHINY_HIGH));
    m.insert(
        "PRIM_TEXGEN_DEFAULT",
        LSLValue::Integer(PRIM_TEXGEN_DEFAULT),
    );
    m.insert("PRIM_TEXGEN_PLANAR", LSLValue::Integer(PRIM_TEXGEN_PLANAR));
    m.insert(
        "PRIM_PHYSICS_SHAPE_PRIM",
        LSLValue::Integer(PRIM_PHYSICS_SHAPE_PRIM),
    );
    m.insert(
        "PRIM_PHYSICS_SHAPE_NONE",
        LSLValue::Integer(PRIM_PHYSICS_SHAPE_NONE),
    );
    m.insert(
        "PRIM_PHYSICS_SHAPE_CONVEX",
        LSLValue::Integer(PRIM_PHYSICS_SHAPE_CONVEX),
    );
    m.insert(
        "PRIM_SCULPT_TYPE_SPHERE",
        LSLValue::Integer(PRIM_SCULPT_TYPE_SPHERE),
    );
    m.insert(
        "PRIM_SCULPT_TYPE_TORUS",
        LSLValue::Integer(PRIM_SCULPT_TYPE_TORUS),
    );
    m.insert(
        "PRIM_SCULPT_TYPE_PLANE",
        LSLValue::Integer(PRIM_SCULPT_TYPE_PLANE),
    );
    m.insert(
        "PRIM_SCULPT_TYPE_CYLINDER",
        LSLValue::Integer(PRIM_SCULPT_TYPE_CYLINDER),
    );
    m.insert(
        "PRIM_SCULPT_TYPE_MESH",
        LSLValue::Integer(PRIM_SCULPT_TYPE_MESH),
    );
    m.insert(
        "PRIM_ALPHA_MODE_NONE",
        LSLValue::Integer(PRIM_ALPHA_MODE_NONE),
    );
    m.insert(
        "PRIM_ALPHA_MODE_BLEND",
        LSLValue::Integer(PRIM_ALPHA_MODE_BLEND),
    );
    m.insert(
        "PRIM_ALPHA_MODE_MASK",
        LSLValue::Integer(PRIM_ALPHA_MODE_MASK),
    );
    m.insert(
        "PRIM_ALPHA_MODE_EMISSIVE",
        LSLValue::Integer(PRIM_ALPHA_MODE_EMISSIVE),
    );

    m.insert("INVENTORY_ALL", LSLValue::Integer(INVENTORY_ALL));
    m.insert("INVENTORY_NONE", LSLValue::Integer(INVENTORY_NONE));
    m.insert("INVENTORY_TEXTURE", LSLValue::Integer(INVENTORY_TEXTURE));
    m.insert("INVENTORY_SOUND", LSLValue::Integer(INVENTORY_SOUND));
    m.insert("INVENTORY_LANDMARK", LSLValue::Integer(INVENTORY_LANDMARK));
    m.insert("INVENTORY_CLOTHING", LSLValue::Integer(INVENTORY_CLOTHING));
    m.insert("INVENTORY_OBJECT", LSLValue::Integer(INVENTORY_OBJECT));
    m.insert("INVENTORY_NOTECARD", LSLValue::Integer(INVENTORY_NOTECARD));
    m.insert("INVENTORY_SCRIPT", LSLValue::Integer(INVENTORY_SCRIPT));
    m.insert("INVENTORY_BODYPART", LSLValue::Integer(INVENTORY_BODYPART));
    m.insert(
        "INVENTORY_ANIMATION",
        LSLValue::Integer(INVENTORY_ANIMATION),
    );
    m.insert("INVENTORY_GESTURE", LSLValue::Integer(INVENTORY_GESTURE));

    m.insert("PERM_TRANSFER", LSLValue::Integer(PERM_TRANSFER));
    m.insert("PERM_MODIFY", LSLValue::Integer(PERM_MODIFY));
    m.insert("PERM_COPY", LSLValue::Integer(PERM_COPY));
    m.insert("PERM_MOVE", LSLValue::Integer(PERM_MOVE));
    m.insert("PERM_ALL", LSLValue::Integer(PERM_ALL));
    m.insert("MASK_BASE", LSLValue::Integer(MASK_BASE));
    m.insert("MASK_OWNER", LSLValue::Integer(MASK_OWNER));
    m.insert("MASK_GROUP", LSLValue::Integer(MASK_GROUP));
    m.insert("MASK_EVERYONE", LSLValue::Integer(MASK_EVERYONE));
    m.insert("MASK_NEXT", LSLValue::Integer(MASK_NEXT));

    m.insert("OBJECT_NAME", LSLValue::Integer(OBJECT_NAME));
    m.insert("OBJECT_DESC", LSLValue::Integer(OBJECT_DESC));
    m.insert("OBJECT_POS", LSLValue::Integer(OBJECT_POS));
    m.insert("OBJECT_ROT", LSLValue::Integer(OBJECT_ROT));
    m.insert("OBJECT_VELOCITY", LSLValue::Integer(OBJECT_VELOCITY));
    m.insert("OBJECT_OWNER", LSLValue::Integer(OBJECT_OWNER));
    m.insert("OBJECT_GROUP", LSLValue::Integer(OBJECT_GROUP));
    m.insert("OBJECT_CREATOR", LSLValue::Integer(OBJECT_CREATOR));

    m.insert("TYPE_INTEGER", LSLValue::Integer(TYPE_INTEGER));
    m.insert("TYPE_FLOAT", LSLValue::Integer(TYPE_FLOAT));
    m.insert("TYPE_STRING", LSLValue::Integer(TYPE_STRING));
    m.insert("TYPE_KEY", LSLValue::Integer(TYPE_KEY));
    m.insert("TYPE_VECTOR", LSLValue::Integer(TYPE_VECTOR));
    m.insert("TYPE_ROTATION", LSLValue::Integer(TYPE_ROTATION));
    m.insert("TYPE_INVALID", LSLValue::Integer(TYPE_INVALID));

    m.insert("STRING_TRIM_HEAD", LSLValue::Integer(STRING_TRIM_HEAD));
    m.insert("STRING_TRIM_TAIL", LSLValue::Integer(STRING_TRIM_TAIL));
    m.insert("STRING_TRIM", LSLValue::Integer(STRING_TRIM));

    m.insert("LIST_STAT_RANGE", LSLValue::Integer(LIST_STAT_RANGE));
    m.insert("LIST_STAT_MIN", LSLValue::Integer(LIST_STAT_MIN));
    m.insert("LIST_STAT_MAX", LSLValue::Integer(LIST_STAT_MAX));
    m.insert("LIST_STAT_MEAN", LSLValue::Integer(LIST_STAT_MEAN));
    m.insert("LIST_STAT_MEDIAN", LSLValue::Integer(LIST_STAT_MEDIAN));
    m.insert("LIST_STAT_STD_DEV", LSLValue::Integer(LIST_STAT_STD_DEV));
    m.insert("LIST_STAT_SUM", LSLValue::Integer(LIST_STAT_SUM));
    m.insert(
        "LIST_STAT_SUM_SQUARES",
        LSLValue::Integer(LIST_STAT_SUM_SQUARES),
    );
    m.insert(
        "LIST_STAT_NUM_COUNT",
        LSLValue::Integer(LIST_STAT_NUM_COUNT),
    );

    m.insert("CLICK_ACTION_NONE", LSLValue::Integer(CLICK_ACTION_NONE));
    m.insert("CLICK_ACTION_TOUCH", LSLValue::Integer(CLICK_ACTION_TOUCH));
    m.insert("CLICK_ACTION_SIT", LSLValue::Integer(CLICK_ACTION_SIT));
    m.insert("CLICK_ACTION_BUY", LSLValue::Integer(CLICK_ACTION_BUY));
    m.insert("CLICK_ACTION_PAY", LSLValue::Integer(CLICK_ACTION_PAY));
    m.insert("CLICK_ACTION_OPEN", LSLValue::Integer(CLICK_ACTION_OPEN));

    m.insert("TOUCH_INVALID_FACE", LSLValue::Integer(TOUCH_INVALID_FACE));

    m.insert("HTTP_METHOD", LSLValue::Integer(HTTP_METHOD));
    m.insert("HTTP_MIMETYPE", LSLValue::Integer(HTTP_MIMETYPE));
    m.insert(
        "HTTP_BODY_MAXLENGTH",
        LSLValue::Integer(HTTP_BODY_MAXLENGTH),
    );
    m.insert("HTTP_VERIFY_CERT", LSLValue::Integer(HTTP_VERIFY_CERT));

    m.insert("CONTENT_TYPE_TEXT", LSLValue::Integer(CONTENT_TYPE_TEXT));
    m.insert("CONTENT_TYPE_HTML", LSLValue::Integer(CONTENT_TYPE_HTML));
    m.insert("CONTENT_TYPE_XML", LSLValue::Integer(CONTENT_TYPE_XML));
    m.insert("CONTENT_TYPE_JSON", LSLValue::Integer(CONTENT_TYPE_JSON));
    m.insert("CONTENT_TYPE_LLSD", LSLValue::Integer(CONTENT_TYPE_LLSD));

    m.insert("DENSITY", LSLValue::Integer(DENSITY));
    m.insert("FRICTION", LSLValue::Integer(FRICTION));
    m.insert("RESTITUTION", LSLValue::Integer(RESTITUTION));
    m.insert("GRAVITY_MULTIPLIER", LSLValue::Integer(GRAVITY_MULTIPLIER));

    m.insert("PAY_HIDE", LSLValue::Integer(PAY_HIDE));
    m.insert("PAY_DEFAULT", LSLValue::Integer(PAY_DEFAULT));

    m.insert("ANIM_ON", LSLValue::Integer(ANIM_ON));
    m.insert("LOOP", LSLValue::Integer(LOOP));
    m.insert("REVERSE", LSLValue::Integer(REVERSE));
    m.insert("PING_PONG", LSLValue::Integer(PING_PONG));
    m.insert("SMOOTH", LSLValue::Integer(SMOOTH));
    m.insert("ROTATE", LSLValue::Integer(ROTATE));
    m.insert("SCALE", LSLValue::Integer(SCALE));

    m.insert("DATA_ONLINE", LSLValue::Integer(DATA_ONLINE));
    m.insert("DATA_NAME", LSLValue::Integer(DATA_NAME));
    m.insert("DATA_BORN", LSLValue::Integer(DATA_BORN));
    m.insert("DATA_RATING", LSLValue::Integer(DATA_RATING));

    m.insert("AGENT_LIST_PARCEL", LSLValue::Integer(AGENT_LIST_PARCEL));
    m.insert(
        "AGENT_LIST_PARCEL_OWNER",
        LSLValue::Integer(AGENT_LIST_PARCEL_OWNER),
    );
    m.insert("AGENT_LIST_REGION", LSLValue::Integer(AGENT_LIST_REGION));

    m.insert("JSON_INVALID", LSLValue::String(JSON_INVALID.to_string()));
    m.insert("JSON_OBJECT", LSLValue::String(JSON_OBJECT.to_string()));
    m.insert("JSON_ARRAY", LSLValue::String(JSON_ARRAY.to_string()));
    m.insert("JSON_NUMBER", LSLValue::String(JSON_NUMBER.to_string()));
    m.insert("JSON_STRING", LSLValue::String(JSON_STRING.to_string()));
    m.insert("JSON_NULL", LSLValue::String(JSON_NULL.to_string()));
    m.insert("JSON_TRUE", LSLValue::String(JSON_TRUE.to_string()));
    m.insert("JSON_FALSE", LSLValue::String(JSON_FALSE.to_string()));
    m.insert("JSON_DELETE", LSLValue::String(JSON_DELETE.to_string()));
    m.insert("JSON_APPEND", LSLValue::String(JSON_APPEND.to_string()));

    m.insert(
        "URL_REQUEST_GRANTED",
        LSLValue::String(URL_REQUEST_GRANTED.to_string()),
    );
    m.insert(
        "URL_REQUEST_DENIED",
        LSLValue::String(URL_REQUEST_DENIED.to_string()),
    );
    m.insert("NAK", LSLValue::String(NAK.to_string()));

    m
}
