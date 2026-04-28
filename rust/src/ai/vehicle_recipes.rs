use std::collections::HashMap;

pub struct VehicleRecipe {
    pub name: &'static str,
    pub description: &'static str,
    pub root_prim: PrimSpec,
    pub children: &'static [ChildPrimSpec],
    pub root_script: &'static str,
    pub hud_script: Option<&'static str>,
    pub tuning_defaults: &'static [(&'static str, f32)],
}

pub struct PrimSpec {
    pub shape: &'static str,
    pub size: [f32; 3],
    pub name: &'static str,
    pub sit_pos: Option<[f32; 3]>,
    pub camera_eye: Option<[f32; 3]>,
    pub camera_at: Option<[f32; 3]>,
}

pub struct ChildPrimSpec {
    pub shape: &'static str,
    pub offset: [f32; 3],
    pub rotation: [f32; 4],
    pub size: [f32; 3],
    pub name: &'static str,
    pub script_name: Option<&'static str>,
}

static RECIPES: &[VehicleRecipe] = &[
    VehicleRecipe {
        name: "car",
        description: "4-wheel ground vehicle with chassis and wheel cylinders",
        root_prim: PrimSpec {
            shape: "box",
            size: [4.0, 2.0, 0.8],
            name: "Chassis",
            sit_pos: Some([0.3, -0.4, 0.5]),
            camera_eye: Some([-5.0, 0.0, 2.5]),
            camera_at: Some([4.0, 0.0, 0.0]),
        },
        children: &[
            ChildPrimSpec { shape: "cylinder", offset: [1.5, 1.1, -0.3], rotation: [0.707, 0.0, 0.0, 0.707], size: [0.6, 0.6, 0.2], name: "FR Wheel", script_name: None },
            ChildPrimSpec { shape: "cylinder", offset: [1.5, -1.1, -0.3], rotation: [0.707, 0.0, 0.0, 0.707], size: [0.6, 0.6, 0.2], name: "FL Wheel", script_name: None },
            ChildPrimSpec { shape: "cylinder", offset: [-1.5, 1.1, -0.3], rotation: [0.707, 0.0, 0.0, 0.707], size: [0.6, 0.6, 0.2], name: "RR Wheel", script_name: None },
            ChildPrimSpec { shape: "cylinder", offset: [-1.5, -1.1, -0.3], rotation: [0.707, 0.0, 0.0, 0.707], size: [0.6, 0.6, 0.2], name: "RL Wheel", script_name: None },
        ],
        root_script: "car_controller.lsl",
        hud_script: Some("land_vehicle_hud.lsl"),
        tuning_defaults: &[
            ("MAX_SPEED", 40.0), ("FORWARD_POWER", 30.0), ("REVERSE_POWER", -12.0),
            ("BRAKE_POWER", -25.0), ("TURN_RATE", 2.5),
        ],
    },
    VehicleRecipe {
        name: "bike",
        description: "Motorcycle with 2 wheels and lean banking",
        root_prim: PrimSpec {
            shape: "box",
            size: [2.5, 0.6, 0.8],
            name: "Frame",
            sit_pos: Some([0.0, 0.0, 0.5]),
            camera_eye: Some([-4.0, 0.0, 2.0]),
            camera_at: Some([3.0, 0.0, 0.0]),
        },
        children: &[
            ChildPrimSpec { shape: "cylinder", offset: [1.0, 0.0, -0.2], rotation: [0.707, 0.0, 0.0, 0.707], size: [0.6, 0.6, 0.15], name: "Front Wheel", script_name: None },
            ChildPrimSpec { shape: "cylinder", offset: [-1.0, 0.0, -0.2], rotation: [0.707, 0.0, 0.0, 0.707], size: [0.6, 0.6, 0.15], name: "Rear Wheel", script_name: None },
        ],
        root_script: "bike_controller.lsl",
        hud_script: Some("land_vehicle_hud.lsl"),
        tuning_defaults: &[
            ("MAX_SPEED", 30.0), ("FORWARD_POWER", 25.0), ("LEAN_AMOUNT", 0.8),
        ],
    },
    VehicleRecipe {
        name: "plane",
        description: "Fixed-wing aircraft with flaps and retractable landing gear",
        root_prim: PrimSpec {
            shape: "box",
            size: [8.0, 1.5, 1.5],
            name: "Fuselage",
            sit_pos: Some([1.0, 0.0, 0.3]),
            camera_eye: Some([-8.0, 0.0, 3.0]),
            camera_at: Some([5.0, 0.0, 0.0]),
        },
        children: &[
            ChildPrimSpec { shape: "box", offset: [0.0, 3.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 6.0, 0.1], name: "Left Wing", script_name: None },
            ChildPrimSpec { shape: "box", offset: [0.0, -3.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 6.0, 0.1], name: "Right Wing", script_name: None },
            ChildPrimSpec { shape: "box", offset: [-3.5, 0.0, 0.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 2.0, 0.8], name: "Tail", script_name: None },
            ChildPrimSpec { shape: "box", offset: [0.0, 2.5, -0.1], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 1.5, 0.05], name: "Left Flap", script_name: Some("flaps.lsl") },
            ChildPrimSpec { shape: "box", offset: [0.0, -2.5, -0.1], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 1.5, 0.05], name: "Right Flap", script_name: Some("flaps.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [3.0, 0.0, -0.8], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 0.1, 0.5], name: "Nose Gear", script_name: Some("landing_gear.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [-0.5, 1.0, -0.8], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 0.1, 0.5], name: "Left Gear", script_name: Some("landing_gear.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [-0.5, -1.0, -0.8], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 0.1, 0.5], name: "Right Gear", script_name: Some("landing_gear.lsl") },
        ],
        root_script: "plane_controller.lsl",
        hud_script: Some("flight_hud.lsl"),
        tuning_defaults: &[
            ("MAX_THRUST", 30.0), ("STALL_SPEED", 8.0), ("MAX_SPEED", 60.0),
            ("ROLL_RATE", 2.5), ("PITCH_RATE", 1.5), ("YAW_RATE", 0.8),
            ("LIFT_FACTOR", 0.04), ("DRAG_FACTOR", 0.002),
        ],
    },
    VehicleRecipe {
        name: "vtol",
        description: "Vertical takeoff aircraft with hover/flight transition",
        root_prim: PrimSpec {
            shape: "box",
            size: [6.0, 2.0, 1.5],
            name: "Fuselage",
            sit_pos: Some([1.0, 0.0, 0.3]),
            camera_eye: Some([-8.0, 0.0, 3.0]),
            camera_at: Some([5.0, 0.0, 0.0]),
        },
        children: &[
            ChildPrimSpec { shape: "box", offset: [0.0, 3.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 5.0, 0.1], name: "Left Wing", script_name: None },
            ChildPrimSpec { shape: "box", offset: [0.0, -3.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 5.0, 0.1], name: "Right Wing", script_name: None },
            ChildPrimSpec { shape: "box", offset: [-2.5, 0.0, 0.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 2.0, 0.8], name: "Tail", script_name: None },
            ChildPrimSpec { shape: "cylinder", offset: [0.5, 3.5, 0.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [1.0, 1.0, 0.3], name: "Left Engine", script_name: Some("engine_pod.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [0.5, -3.5, 0.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [1.0, 1.0, 0.3], name: "Right Engine", script_name: Some("engine_pod.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [2.0, 0.0, -0.8], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 0.1, 0.5], name: "Nose Gear", script_name: Some("landing_gear.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [-1.0, 1.5, -0.8], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 0.1, 0.5], name: "Left Gear", script_name: Some("landing_gear.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [-1.0, -1.5, -0.8], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.1, 0.1, 0.5], name: "Right Gear", script_name: Some("landing_gear.lsl") },
        ],
        root_script: "vtol_controller.lsl",
        hud_script: Some("vtol_hud.lsl"),
        tuning_defaults: &[
            ("MAX_HOVER_THRUST", 20.0), ("TRANSITION_SPEED", 15.0),
            ("MAX_SPEED", 50.0), ("ROLL_RATE", 2.0),
        ],
    },
    VehicleRecipe {
        name: "vessel",
        description: "Sailing vessel with multi-sail wind physics and motor",
        root_prim: PrimSpec {
            shape: "box",
            size: [12.0, 3.0, 2.0],
            name: "Hull",
            sit_pos: Some([0.5, 0.0, 0.6]),
            camera_eye: Some([-8.0, 0.0, 4.0]),
            camera_at: Some([5.0, 0.0, 1.0]),
        },
        children: &[
            ChildPrimSpec { shape: "cylinder", offset: [1.0, 0.0, 4.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 0.2, 8.0], name: "Main Mast", script_name: None },
            ChildPrimSpec { shape: "cylinder", offset: [3.5, 0.0, 3.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 6.0], name: "Fore Mast", script_name: None },
            ChildPrimSpec { shape: "cylinder", offset: [-2.5, 0.0, 3.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 6.0], name: "Mizzen Mast", script_name: None },
            ChildPrimSpec { shape: "box", offset: [1.0, 0.3, 4.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.02, 3.0, 5.0], name: "Mainsail", script_name: Some("mainsail.lsl") },
            ChildPrimSpec { shape: "box", offset: [3.5, 0.3, 3.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.02, 2.5, 4.0], name: "Foresail", script_name: Some("foresail.lsl") },
            ChildPrimSpec { shape: "box", offset: [-2.5, 0.3, 3.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.02, 2.0, 4.0], name: "Mizzensail", script_name: Some("mizzensail.lsl") },
            ChildPrimSpec { shape: "box", offset: [5.0, 0.3, 2.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.02, 2.0, 3.0], name: "Jib", script_name: Some("jib.lsl") },
            ChildPrimSpec { shape: "box", offset: [-5.0, 0.0, -0.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.5, 0.5, 0.3], name: "Motor", script_name: Some("motor.lsl") },
            ChildPrimSpec { shape: "box", offset: [-5.5, 0.0, -0.2], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.3, 0.3, 0.2], name: "Thruster", script_name: Some("thruster_controller.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [-6.0, 0.0, -0.8], rotation: [0.0, 0.0, 0.707, 0.707], size: [0.3, 0.3, 0.1], name: "thruster_main", script_name: Some("thruster_main.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [5.0, 0.0, -0.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 0.3], name: "thruster_bow", script_name: Some("thruster_bow.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [5.5, 1.2, 1.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 0.15], name: "Port Light", script_name: Some("lights.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [5.5, -1.2, 1.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 0.15], name: "Starboard Light", script_name: Some("lights.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [1.0, 0.0, 8.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 0.15], name: "Masthead Light", script_name: Some("lights.lsl") },
            ChildPrimSpec { shape: "box", offset: [-3.0, 1.3, 1.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [1.5, 0.05, 1.5], name: "Cabin Door", script_name: Some("cabin_door.lsl") },
        ],
        root_script: "vessel_controller.lsl",
        hud_script: Some("vessel_hud.lsl"),
        tuning_defaults: &[
            ("FORWARD_POWER", 20.0), ("REVERSE_POWER", -10.0), ("TURN_RATE", 2.0),
            ("WIND_BASE_SPEED", 10.0), ("WIND_PERIOD", 300.0),
        ],
    },
    VehicleRecipe {
        name: "starship",
        description: "Sci-fi starship with impulse, warp, weapons, and docking",
        root_prim: PrimSpec {
            shape: "box",
            size: [20.0, 6.0, 4.0],
            name: "Hull",
            sit_pos: Some([6.0, 0.0, 1.0]),
            camera_eye: Some([-15.0, 0.0, 5.0]),
            camera_at: Some([10.0, 0.0, 0.0]),
        },
        children: &[
            ChildPrimSpec { shape: "cylinder", offset: [-5.0, 0.0, -1.0], rotation: [0.0, 0.707, 0.0, 0.707], size: [2.0, 2.0, 4.0], name: "Impulse Engine", script_name: Some("impulse_engine.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [-3.0, 5.0, 1.0], rotation: [0.0, 0.707, 0.0, 0.707], size: [1.5, 1.5, 6.0], name: "Left Nacelle", script_name: Some("warp_nacelle.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [-3.0, -5.0, 1.0], rotation: [0.0, 0.707, 0.0, 0.707], size: [1.5, 1.5, 6.0], name: "Right Nacelle", script_name: Some("warp_nacelle.lsl") },
            ChildPrimSpec { shape: "box", offset: [8.0, 0.0, 0.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [2.0, 1.0, 0.5], name: "Weapons Array", script_name: Some("weapons_array.lsl") },
            ChildPrimSpec { shape: "box", offset: [-8.0, 2.0, -1.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.5, 0.5, 0.3], name: "Port Thruster", script_name: Some("docking_thruster.lsl") },
            ChildPrimSpec { shape: "box", offset: [-8.0, -2.0, -1.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.5, 0.5, 0.3], name: "Starboard Thruster", script_name: Some("docking_thruster.lsl") },
            ChildPrimSpec { shape: "box", offset: [0.0, 0.0, -1.8], rotation: [0.0, 0.0, 0.0, 1.0], size: [2.0, 2.0, 0.1], name: "Turbolift", script_name: Some("turbolift.lsl") },
            ChildPrimSpec { shape: "box", offset: [3.0, 3.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [2.0, 0.1, 2.5], name: "Port Door", script_name: Some("pneumatic_door.lsl") },
            ChildPrimSpec { shape: "box", offset: [3.0, -3.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [2.0, 0.1, 2.5], name: "Starboard Door", script_name: Some("pneumatic_door.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [10.0, 0.0, 2.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 0.2, 0.2], name: "Forward Nav Light", script_name: Some("nav_lights.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [-10.0, 0.0, 2.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 0.2, 0.2], name: "Aft Nav Light", script_name: Some("nav_lights.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [0.0, 3.5, 2.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 0.2, 0.2], name: "Port Nav Light", script_name: Some("nav_lights.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [0.0, -3.5, 2.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 0.2, 0.2], name: "Starboard Nav Light", script_name: Some("nav_lights.lsl") },
        ],
        root_script: "starship_controller.lsl",
        hud_script: Some("starship_hud.lsl"),
        tuning_defaults: &[
            ("IMPULSE_POWER", 50.0), ("WARP_FACTOR", 9.9),
            ("MAX_SPEED", 100.0), ("TURN_RATE", 1.5),
        ],
    },
    VehicleRecipe {
        name: "lani",
        description: "Gaia hybrid sailing vessel with Lani/Dyna controller, multi-sail wind physics, thrusters, and docking",
        root_prim: PrimSpec {
            shape: "box",
            size: [12.0, 3.0, 2.0],
            name: "Hull",
            sit_pos: Some([0.5, 0.0, 0.6]),
            camera_eye: Some([-8.0, 0.0, 4.0]),
            camera_at: Some([5.0, 0.0, 1.0]),
        },
        children: &[
            ChildPrimSpec { shape: "cylinder", offset: [1.0, 0.0, 4.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.2, 0.2, 8.0], name: "Main Mast", script_name: None },
            ChildPrimSpec { shape: "cylinder", offset: [3.5, 0.0, 3.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 6.0], name: "Fore Mast", script_name: None },
            ChildPrimSpec { shape: "cylinder", offset: [-2.5, 0.0, 3.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 6.0], name: "Mizzen Mast", script_name: None },
            ChildPrimSpec { shape: "box", offset: [1.0, 0.3, 4.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.02, 3.0, 5.0], name: "Mainsail", script_name: Some("mainsail.lsl") },
            ChildPrimSpec { shape: "box", offset: [3.5, 0.3, 3.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.02, 2.5, 4.0], name: "Foresail", script_name: Some("foresail.lsl") },
            ChildPrimSpec { shape: "box", offset: [-2.5, 0.3, 3.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.02, 2.0, 4.0], name: "Mizzensail", script_name: Some("mizzensail.lsl") },
            ChildPrimSpec { shape: "box", offset: [5.0, 0.3, 2.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.02, 2.0, 3.0], name: "Jib", script_name: Some("jib.lsl") },
            ChildPrimSpec { shape: "box", offset: [-5.0, 0.0, -0.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.5, 0.5, 0.3], name: "Motor", script_name: Some("motor.lsl") },
            ChildPrimSpec { shape: "box", offset: [-5.5, 0.0, -0.2], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.3, 0.3, 0.2], name: "Thruster", script_name: Some("thruster_controller.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [-6.0, 0.0, -0.8], rotation: [0.0, 0.0, 0.707, 0.707], size: [0.3, 0.3, 0.1], name: "thruster_main", script_name: Some("thruster_main.lsl") },
            ChildPrimSpec { shape: "cylinder", offset: [5.0, 0.0, -0.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 0.3], name: "thruster_bow", script_name: Some("thruster_bow.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [5.5, 1.2, 1.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 0.15], name: "Port Light", script_name: Some("lights.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [5.5, -1.2, 1.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 0.15], name: "Starboard Light", script_name: Some("lights.lsl") },
            ChildPrimSpec { shape: "sphere", offset: [1.0, 0.0, 8.5], rotation: [0.0, 0.0, 0.0, 1.0], size: [0.15, 0.15, 0.15], name: "Masthead Light", script_name: Some("lights.lsl") },
            ChildPrimSpec { shape: "box", offset: [-3.0, 1.3, 1.0], rotation: [0.0, 0.0, 0.0, 1.0], size: [1.5, 0.05, 1.5], name: "Cabin Door", script_name: Some("cabin_door.lsl") },
        ],
        root_script: "gaia_marina_controller.lsl",
        hud_script: Some("vessel_hud.lsl"),
        tuning_defaults: &[
            ("FORWARD_POWER", 20.0), ("REVERSE_POWER", -10.0), ("TURN_RATE", 2.0),
            ("WIND_BASE_SPEED", 10.0), ("WIND_PERIOD", 300.0),
        ],
    },
];

pub fn get_recipe(name: &str) -> Option<&'static VehicleRecipe> {
    RECIPES.iter().find(|r| r.name == name)
}

pub fn list_recipe_names() -> Vec<&'static str> {
    RECIPES.iter().map(|r| r.name).collect()
}

pub fn recipe_count() -> usize {
    RECIPES.len()
}

pub fn apply_scale(recipe: &VehicleRecipe, scale_factor: f32) -> (PrimSpec, Vec<ChildPrimSpec>) {
    let clamped = scale_factor.clamp(0.25, 4.0);
    let root = PrimSpec {
        shape: recipe.root_prim.shape,
        size: [
            recipe.root_prim.size[0] * clamped,
            recipe.root_prim.size[1] * clamped,
            recipe.root_prim.size[2] * clamped,
        ],
        name: recipe.root_prim.name,
        sit_pos: recipe
            .root_prim
            .sit_pos
            .map(|p| [p[0] * clamped, p[1] * clamped, p[2] * clamped]),
        camera_eye: recipe
            .root_prim
            .camera_eye
            .map(|p| [p[0] * clamped, p[1] * clamped, p[2] * clamped]),
        camera_at: recipe
            .root_prim
            .camera_at
            .map(|p| [p[0] * clamped, p[1] * clamped, p[2] * clamped]),
    };
    let children: Vec<ChildPrimSpec> = recipe
        .children
        .iter()
        .map(|c| ChildPrimSpec {
            shape: c.shape,
            offset: [
                c.offset[0] * clamped,
                c.offset[1] * clamped,
                c.offset[2] * clamped,
            ],
            rotation: c.rotation,
            size: [
                c.size[0] * clamped,
                c.size[1] * clamped,
                c.size[2] * clamped,
            ],
            name: c.name,
            script_name: c.script_name,
        })
        .collect();
    (root, children)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_recipes_exist() {
        assert_eq!(recipe_count(), 7);
        let names = list_recipe_names();
        assert!(names.contains(&"car"));
        assert!(names.contains(&"bike"));
        assert!(names.contains(&"plane"));
        assert!(names.contains(&"vtol"));
        assert!(names.contains(&"vessel"));
        assert!(names.contains(&"starship"));
        assert!(names.contains(&"lani"));
    }

    #[test]
    fn test_car_recipe() {
        let car = get_recipe("car").unwrap();
        assert_eq!(car.root_prim.shape, "box");
        assert_eq!(car.children.len(), 4);
        assert_eq!(car.root_script, "car_controller.lsl");
        assert!(car.hud_script.is_some());
        for child in car.children {
            assert_eq!(child.shape, "cylinder");
            assert!(child.name.contains("Wheel"));
            assert!(child.script_name.is_none());
        }
    }

    #[test]
    fn test_plane_has_child_scripts() {
        let plane = get_recipe("plane").unwrap();
        assert_eq!(plane.children.len(), 8);
        let scripted: Vec<_> = plane
            .children
            .iter()
            .filter(|c| c.script_name.is_some())
            .collect();
        assert_eq!(scripted.len(), 5);
    }

    #[test]
    fn test_vessel_full_complement() {
        let vessel = get_recipe("vessel").unwrap();
        assert_eq!(vessel.children.len(), 15);
        assert_eq!(vessel.root_script, "vessel_controller.lsl");
    }

    #[test]
    fn test_starship_complement() {
        let ship = get_recipe("starship").unwrap();
        assert!(ship.children.len() >= 13);
        assert_eq!(ship.root_script, "starship_controller.lsl");
    }

    #[test]
    fn test_scale_factor_clamped() {
        let car = get_recipe("car").unwrap();
        let (root, children) = apply_scale(car, 10.0);
        assert!((root.size[0] - car.root_prim.size[0] * 4.0).abs() < 0.01);
        assert_eq!(children.len(), car.children.len());

        let (root_small, _) = apply_scale(car, 0.1);
        assert!((root_small.size[0] - car.root_prim.size[0] * 0.25).abs() < 0.01);
    }

    #[test]
    fn test_unknown_recipe() {
        assert!(get_recipe("submarine").is_none());
    }
}
