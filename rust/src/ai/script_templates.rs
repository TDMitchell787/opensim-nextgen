use std::collections::HashMap;

pub struct ScriptTemplate {
    pub name: &'static str,
    pub source: &'static str,
    pub defaults: &'static [(&'static str, &'static str)],
}

static TEMPLATES: &[ScriptTemplate] = &[
    ScriptTemplate {
        name: "rotating",
        source: r#"integer rotating = FALSE;
default {
    touch_start(integer n) {
        rotating = !rotating;
        if (rotating) {
            vector axis = {{AXIS}};
            llTargetOmega(axis, {{SPEED}}, 1.0);
            llSay(0, "Spinning!");
        } else {
            llTargetOmega(ZERO_VECTOR, 0.0, 0.0);
            llSay(0, "Stopped.");
        }
    }
}"#,
        defaults: &[("AXIS", "<0,0,1>"), ("SPEED", "1.0")],
    },
    ScriptTemplate {
        name: "sliding_door",
        source: r#"integer open = FALSE;
vector closed_pos;
default {
    state_entry() {
        closed_pos = llGetPos();
    }
    touch_start(integer n) {
        if (!open) {
            llSetPos(closed_pos + <{{SLIDE_DISTANCE}}, 0, 0>);
            open = TRUE;
            llSetTimerEvent({{AUTO_CLOSE}});
            llSay(0, "Door opened.");
        } else {
            llSetPos(closed_pos);
            open = FALSE;
            llSetTimerEvent(0.0);
            llSay(0, "Door closed.");
        }
    }
    timer() {
        if (open) {
            llSetPos(closed_pos);
            open = FALSE;
            llSetTimerEvent(0.0);
            llSay(0, "Door auto-closed.");
        }
    }
}"#,
        defaults: &[("SLIDE_DISTANCE", "0.5"), ("AUTO_CLOSE", "10.0")],
    },
    ScriptTemplate {
        name: "toggle_light",
        source: r#"integer lit = FALSE;
default {
    touch_start(integer n) {
        lit = !lit;
        if (lit) {
            llSetPrimitiveParams([PRIM_POINT_LIGHT, TRUE, <{{COLOR}}>, {{INTENSITY}}, {{RADIUS}}, 0.5,
                                  PRIM_GLOW, ALL_SIDES, 0.2]);
            llSay(0, "Light on.");
        } else {
            llSetPrimitiveParams([PRIM_POINT_LIGHT, FALSE, <1,1,1>, 0, 0, 0,
                                  PRIM_GLOW, ALL_SIDES, 0.0]);
            llSay(0, "Light off.");
        }
    }
}"#,
        defaults: &[("COLOR", "1,1,1"), ("INTENSITY", "1.0"), ("RADIUS", "10.0")],
    },
    ScriptTemplate {
        name: "floating_text",
        source: r#"default {
    state_entry() {
        llSetText("{{TEXT}}", <{{COLOR}}>, 1.0);
    }
}"#,
        defaults: &[("TEXT", "Hello World"), ("COLOR", "1,1,1")],
    },
    ScriptTemplate {
        name: "sit_target",
        source: r#"default {
    state_entry() {
        llSitTarget(<{{SIT_OFFSET}}>, ZERO_ROTATION);
    }
    changed(integer change) {
        if (change & CHANGED_LINK) {
            key av = llAvatarOnSitTarget();
            if (av != NULL_KEY) {
                llSay(0, "Welcome, " + llKey2Name(av) + "!");
            }
        }
    }
}"#,
        defaults: &[("SIT_OFFSET", "0,0,0.5")],
    },
    ScriptTemplate {
        name: "touch_say",
        source: r#"default {
    touch_start(integer n) {
        llSay({{CHANNEL}}, "{{MESSAGE}}");
    }
}"#,
        defaults: &[("MESSAGE", "Hello!"), ("CHANNEL", "0")],
    },
    ScriptTemplate {
        name: "timer_color",
        source: r#"default {
    state_entry() {
        llSetTimerEvent({{INTERVAL}});
    }
    timer() {
        vector color = <llFrand(1.0), llFrand(1.0), llFrand(1.0)>;
        llSetColor(color, ALL_SIDES);
    }
}"#,
        defaults: &[("INTERVAL", "2.0")],
    },
    ScriptTemplate {
        name: "touch_hide",
        source: r#"integer visible = TRUE;
default {
    touch_start(integer n) {
        visible = !visible;
        if (visible) {
            llSetAlpha(1.0, ALL_SIDES);
            llSay(0, "Visible.");
        } else {
            llSetAlpha(0.2, ALL_SIDES);
            llSay(0, "Hidden.");
        }
    }
}"#,
        defaults: &[],
    },
    ScriptTemplate {
        name: "car_controller",
        source: r#"float MAX_SPEED = {{MAX_SPEED}};
float FORWARD_POWER = {{FORWARD_POWER}};
float REVERSE_POWER = {{REVERSE_POWER}};
float BRAKE_POWER = {{BRAKE_POWER}};
float TURN_RATE = {{TURN_RATE}};
integer HUD_CH = {{HUD_CH}};

integer gState;
key gPilot;
float gThrottle;

setup_vehicle()
{
    llSetVehicleType(VEHICLE_TYPE_CAR);
    llSetVehicleFlags(VEHICLE_FLAG_LIMIT_ROLL_ONLY);
    llRemoveVehicleFlags(VEHICLE_FLAG_HOVER_WATER_ONLY | VEHICLE_FLAG_HOVER_GLOBAL_HEIGHT);
    llSetVehicleVectorParam(VEHICLE_LINEAR_FRICTION_TIMESCALE, <1, 0.5, 1000>);
    llSetVehicleVectorParam(VEHICLE_ANGULAR_FRICTION_TIMESCALE, <10, 10, 0.5>);
    llSetVehicleFloatParam(VEHICLE_LINEAR_MOTOR_TIMESCALE, 1.0);
    llSetVehicleFloatParam(VEHICLE_LINEAR_MOTOR_DECAY_TIMESCALE, 15.0);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_MOTOR_TIMESCALE, 0.5);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_MOTOR_DECAY_TIMESCALE, 2.0);
    llSetVehicleFloatParam(VEHICLE_HOVER_HEIGHT, 0.4);
    llSetVehicleFloatParam(VEHICLE_HOVER_EFFICIENCY, 0.5);
    llSetVehicleFloatParam(VEHICLE_HOVER_TIMESCALE, 0.5);
    llSetVehicleFloatParam(VEHICLE_BUOYANCY, 0.0);
    llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_EFFICIENCY, 0.8);
    llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_TIMESCALE, 0.5);
    llSetVehicleFloatParam(VEHICLE_BANKING_EFFICIENCY, 0.0);
    llSetVehicleFloatParam(VEHICLE_LINEAR_DEFLECTION_EFFICIENCY, 0.8);
    llSetVehicleFloatParam(VEHICLE_LINEAR_DEFLECTION_TIMESCALE, 1.0);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_DEFLECTION_EFFICIENCY, 0.2);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_DEFLECTION_TIMESCALE, 5.0);
    llSetVehicleRotationParam(VEHICLE_REFERENCE_FRAME, ZERO_ROTATION);
}

default
{
    state_entry()
    {
        llSetSitText("Drive");
        llSitTarget(<{{SIT_POS}}>, ZERO_ROTATION);
        llSetCameraEyeOffset(<-5, 0, 2.5>);
        llSetCameraAtOffset(<4, 0, 0>);
        gState = 0;
        gThrottle = 0.0;
    }
    on_rez(integer start) { llResetScript(); }
    changed(integer change)
    {
        if (change & CHANGED_LINK)
        {
            key av = llAvatarOnSitTarget();
            if (av)
            {
                gPilot = av;
                setup_vehicle();
                llSetStatus(STATUS_PHYSICS, TRUE);
                llRequestPermissions(av, PERMISSION_TAKE_CONTROLS | PERMISSION_CONTROL_CAMERA | PERMISSION_TRIGGER_ANIMATION);
            }
            else
            {
                llSetStatus(STATUS_PHYSICS, FALSE);
                llSetVehicleType(VEHICLE_TYPE_NONE);
                llReleaseControls();
                llClearCameraParams();
                llSetTimerEvent(0.0);
                gState = 0;
                gPilot = NULL_KEY;
                gThrottle = 0.0;
                llSetText("", ZERO_VECTOR, 0.0);
            }
        }
    }
    run_time_permissions(integer perm)
    {
        if (perm & PERMISSION_TAKE_CONTROLS)
            llTakeControls(CONTROL_FWD | CONTROL_BACK | CONTROL_LEFT | CONTROL_RIGHT | CONTROL_ROT_LEFT | CONTROL_ROT_RIGHT | CONTROL_UP | CONTROL_DOWN, TRUE, FALSE);
        gState = 1;
        llSetTimerEvent(0.3);
    }
    timer()
    {
        vector vel = llGetVel();
        rotation rot = llGetRot();
        vector fwd = llRot2Fwd(rot);
        float fwd_speed = vel * fwd;
        float speed_kph = llFabs(fwd_speed) * 3.6;
        string hud = (string)llRound(speed_kph) + " km/h";
        if (gState == 0) hud += " [PARKED]";
        else if (gState == 1) hud += " [DRIVE]";
        else if (gState == 2) hud += " [REVERSE]";
        llSetText(hud, <0.5, 1, 0.5>, 0.8);
        if (gPilot != NULL_KEY) llRegionSayTo(gPilot, HUD_CH, hud);
    }
    control(key id, integer level, integer edge)
    {
        if (gState == 0) return;
        vector motor = ZERO_VECTOR;
        vector angular = ZERO_VECTOR;
        vector vel = llGetVel();
        rotation rot = llGetRot();
        vector fwd = llRot2Fwd(rot);
        float fwd_speed = vel * fwd;
        if (level & CONTROL_FWD)
        {
            gState = 1;
            float factor = 1.0;
            if (fwd_speed > MAX_SPEED * 0.8) factor = 0.3;
            motor.x = FORWARD_POWER * factor;
        }
        if (level & CONTROL_BACK)
        {
            if (fwd_speed > 1.0) motor.x = BRAKE_POWER;
            else { gState = 2; motor.x = REVERSE_POWER; }
        }
        float speed = llFabs(fwd_speed);
        float turn_scale = 1.0;
        if (speed > 15.0) turn_scale = 0.5;
        if (speed > 30.0) turn_scale = 0.3;
        integer reverse_steer = 1;
        if (gState == 2) reverse_steer = -1;
        if (level & (CONTROL_LEFT | CONTROL_ROT_LEFT)) angular.z = TURN_RATE * turn_scale * reverse_steer;
        if (level & (CONTROL_RIGHT | CONTROL_ROT_RIGHT)) angular.z = -TURN_RATE * turn_scale * reverse_steer;
        llSetVehicleVectorParam(VEHICLE_LINEAR_MOTOR_DIRECTION, motor);
        llSetVehicleVectorParam(VEHICLE_ANGULAR_MOTOR_DIRECTION, angular);
    }
}"#,
        defaults: &[
            ("MAX_SPEED", "40.0"), ("FORWARD_POWER", "30.0"), ("REVERSE_POWER", "-12.0"),
            ("BRAKE_POWER", "-25.0"), ("TURN_RATE", "2.5"), ("HUD_CH", "-14710"),
            ("SIT_POS", "0.3, -0.4, 0.5"),
        ],
    },
    ScriptTemplate {
        name: "plane_controller",
        source: r#"float MAX_THRUST = {{MAX_THRUST}};
float STALL_SPEED = {{STALL_SPEED}};
float MAX_SPEED = {{MAX_SPEED}};
float ROLL_RATE = {{ROLL_RATE}};
float PITCH_RATE = {{PITCH_RATE}};
float YAW_RATE = {{YAW_RATE}};
float LIFT_FACTOR = {{LIFT_FACTOR}};
float DRAG_FACTOR = {{DRAG_FACTOR}};
integer HUD_CH = {{HUD_CH}};

integer gState;
key gPilot;
float gThrottle;
integer gStallWarning;
float gLastAlt;

setup_vehicle()
{
    llSetVehicleType(VEHICLE_TYPE_AIRPLANE);
    llRemoveVehicleFlags(VEHICLE_FLAG_HOVER_WATER_ONLY | VEHICLE_FLAG_HOVER_GLOBAL_HEIGHT | VEHICLE_FLAG_NO_DEFLECTION_UP | VEHICLE_FLAG_LIMIT_ROLL_ONLY);
    llSetVehicleFlags(VEHICLE_FLAG_LIMIT_MOTOR_UP);
    llSetVehicleVectorParam(VEHICLE_LINEAR_FRICTION_TIMESCALE, <200, 200, 200>);
    llSetVehicleVectorParam(VEHICLE_ANGULAR_FRICTION_TIMESCALE, <3, 3, 3>);
    llSetVehicleFloatParam(VEHICLE_LINEAR_MOTOR_TIMESCALE, 2.0);
    llSetVehicleFloatParam(VEHICLE_LINEAR_MOTOR_DECAY_TIMESCALE, 60.0);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_MOTOR_TIMESCALE, 1.0);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_MOTOR_DECAY_TIMESCALE, 8.0);
    llSetVehicleFloatParam(VEHICLE_HOVER_HEIGHT, 0.0);
    llSetVehicleFloatParam(VEHICLE_HOVER_EFFICIENCY, 0.0);
    llSetVehicleFloatParam(VEHICLE_HOVER_TIMESCALE, 1000.0);
    llSetVehicleFloatParam(VEHICLE_BUOYANCY, 0.0);
    llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_EFFICIENCY, 0.3);
    llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_TIMESCALE, 5.0);
    llSetVehicleFloatParam(VEHICLE_BANKING_EFFICIENCY, 0.7);
    llSetVehicleFloatParam(VEHICLE_BANKING_MIX, 0.85);
    llSetVehicleFloatParam(VEHICLE_BANKING_TIMESCALE, 0.5);
    llSetVehicleFloatParam(VEHICLE_LINEAR_DEFLECTION_EFFICIENCY, 0.7);
    llSetVehicleFloatParam(VEHICLE_LINEAR_DEFLECTION_TIMESCALE, 2.0);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_DEFLECTION_EFFICIENCY, 0.5);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_DEFLECTION_TIMESCALE, 3.0);
    llSetVehicleRotationParam(VEHICLE_REFERENCE_FRAME, ZERO_ROTATION);
}

float compute_lift(float speed) { float lift = speed * speed * LIFT_FACTOR; if (lift > 1.0) lift = 1.0; return lift; }
float compute_drag(float speed) { return speed * speed * DRAG_FACTOR; }

default
{
    state_entry()
    {
        llSetSitText("Board");
        llSitTarget(<{{SIT_POS}}>, ZERO_ROTATION);
        llSetCameraEyeOffset(<-8, 0, 3>);
        llSetCameraAtOffset(<5, 0, 0>);
        gState = 0;
        gThrottle = 0.0;
        gStallWarning = 0;
        gLastAlt = 0.0;
    }
    on_rez(integer start) { llResetScript(); }
    changed(integer change)
    {
        if (change & CHANGED_LINK)
        {
            key av = llAvatarOnSitTarget();
            if (av)
            {
                gPilot = av;
                setup_vehicle();
                llSetVehicleFloatParam(VEHICLE_HOVER_HEIGHT, 0.4);
                llSetVehicleFloatParam(VEHICLE_HOVER_EFFICIENCY, 0.8);
                llSetVehicleFloatParam(VEHICLE_HOVER_TIMESCALE, 0.5);
                llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_EFFICIENCY, 0.9);
                llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_TIMESCALE, 0.3);
                llSetVehicleFloatParam(VEHICLE_BANKING_EFFICIENCY, 0.0);
                llSetStatus(STATUS_PHYSICS, TRUE);
                llRequestPermissions(av, PERMISSION_TAKE_CONTROLS | PERMISSION_CONTROL_CAMERA | PERMISSION_TRIGGER_ANIMATION);
            }
            else
            {
                llSetStatus(STATUS_PHYSICS, FALSE);
                llSetVehicleType(VEHICLE_TYPE_NONE);
                llReleaseControls();
                llClearCameraParams();
                llSetTimerEvent(0.0);
                gState = 0;
                gPilot = NULL_KEY;
                gThrottle = 0.0;
                gStallWarning = 0;
                llSetText("", ZERO_VECTOR, 0.0);
            }
        }
    }
    run_time_permissions(integer perm)
    {
        if (perm & PERMISSION_TAKE_CONTROLS)
            llTakeControls(CONTROL_FWD | CONTROL_BACK | CONTROL_LEFT | CONTROL_RIGHT | CONTROL_ROT_LEFT | CONTROL_ROT_RIGHT | CONTROL_UP | CONTROL_DOWN, TRUE, FALSE);
        gState = 1;
        llSetTimerEvent(0.3);
    }
    timer()
    {
        vector pos = llGetPos();
        vector vel = llGetVel();
        rotation rot = llGetRot();
        vector fwd = llRot2Fwd(rot);
        float airspeed = llVecMag(vel);
        float fwd_speed = vel * fwd;
        float altitude = pos.z;
        float vert_speed = (altitude - gLastAlt) / 0.3;
        gLastAlt = altitude;
        float lift = compute_lift(airspeed);
        float drag = compute_drag(airspeed);
        if (gState == 1)
        {
            if (fwd_speed > STALL_SPEED * 1.2 && gThrottle > 0.5)
            {
                gState = 2;
                llSetVehicleFloatParam(VEHICLE_HOVER_HEIGHT, 0.0);
                llSetVehicleFloatParam(VEHICLE_HOVER_EFFICIENCY, 0.0);
                llSetVehicleFloatParam(VEHICLE_HOVER_TIMESCALE, 1000.0);
                llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_EFFICIENCY, 0.3);
                llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_TIMESCALE, 5.0);
                llSetVehicleFloatParam(VEHICLE_BANKING_EFFICIENCY, 0.7);
                llSay(0, "Airborne!");
            }
            float thrust_vec = MAX_THRUST * gThrottle - drag;
            if (thrust_vec < 0.0) thrust_vec = 0.0;
            llSetVehicleVectorParam(VEHICLE_LINEAR_MOTOR_DIRECTION, <thrust_vec, 0, 0>);
        }
        else if (gState == 2)
        {
            llSetVehicleFloatParam(VEHICLE_BUOYANCY, lift);
            float thrust_vec = MAX_THRUST * gThrottle - drag;
            llSetVehicleVectorParam(VEHICLE_LINEAR_MOTOR_DIRECTION, <thrust_vec, 0, 0>);
            integer was_stall = gStallWarning;
            if (airspeed < STALL_SPEED && altitude > 5.0)
            {
                gStallWarning = 1;
                if (!was_stall) llSay(0, "** STALL WARNING **");
            }
            else gStallWarning = 0;
            if (altitude < 3.0 && vert_speed < -0.5)
            {
                gState = 1;
                llSetVehicleFloatParam(VEHICLE_HOVER_HEIGHT, 0.4);
                llSetVehicleFloatParam(VEHICLE_HOVER_EFFICIENCY, 0.8);
                llSetVehicleFloatParam(VEHICLE_HOVER_TIMESCALE, 0.5);
                llSay(0, "Touchdown.");
            }
        }
        float speed_kts = airspeed * 1.94;
        float alt_ft = altitude * 3.28;
        string hud = (string)llRound(speed_kts) + "kts " + (string)llRound(alt_ft) + "ft THR:" + (string)llRound(gThrottle * 100) + "%";
        if (gState == 0) hud += " [OFF]";
        else if (gState == 1) hud += " [GND]";
        else if (gState == 2) hud += " [FLT]";
        if (gStallWarning) hud += " **STALL**";
        vector col = <0.3, 1, 0.3>;
        if (gStallWarning) col = <1, 0.2, 0.2>;
        llSetText(hud, col, 0.8);
        if (gPilot != NULL_KEY) llRegionSayTo(gPilot, HUD_CH, hud);
    }
    control(key id, integer level, integer edge)
    {
        if (gState == 0) return;
        vector angular = ZERO_VECTOR;
        if ((edge & CONTROL_FWD) && (level & CONTROL_FWD)) { gThrottle += 0.1; if (gThrottle > 1.0) gThrottle = 1.0; }
        if ((edge & CONTROL_BACK) && (level & CONTROL_BACK)) { gThrottle -= 0.1; if (gThrottle < 0.0) gThrottle = 0.0; }
        if (level & CONTROL_LEFT) angular.x = ROLL_RATE;
        if (level & CONTROL_RIGHT) angular.x = -ROLL_RATE;
        if (level & CONTROL_ROT_LEFT) angular.z = YAW_RATE;
        if (level & CONTROL_ROT_RIGHT) angular.z = -YAW_RATE;
        if (level & CONTROL_UP) angular.y = -PITCH_RATE;
        if (level & CONTROL_DOWN) angular.y = PITCH_RATE;
        llSetVehicleVectorParam(VEHICLE_ANGULAR_MOTOR_DIRECTION, angular);
    }
}"#,
        defaults: &[
            ("MAX_THRUST", "30.0"), ("STALL_SPEED", "8.0"), ("MAX_SPEED", "60.0"),
            ("ROLL_RATE", "2.5"), ("PITCH_RATE", "1.5"), ("YAW_RATE", "0.8"),
            ("LIFT_FACTOR", "0.04"), ("DRAG_FACTOR", "0.002"),
            ("HUD_CH", "-14720"), ("SIT_POS", "1.0, 0.0, 0.3"),
        ],
    },
    ScriptTemplate {
        name: "vessel_controller",
        source: r#"float FORWARD_POWER = {{FORWARD_POWER}};
float REVERSE_POWER = {{REVERSE_POWER}};
float TURN_RATE = {{TURN_RATE}};
float WIND_BASE_SPEED = {{WIND_BASE_SPEED}};
float WIND_PERIOD = {{WIND_PERIOD}};
integer HUD_CH = {{HUD_CH}};

integer gState;
key gPilot;
float gMotorThrottle;
integer gSailsUp;
float gWindDir;
float gWindSpeed;
vector gSailForce;

setup_vehicle()
{
    llSetVehicleType(VEHICLE_TYPE_BOAT);
    llSetVehicleFlags(VEHICLE_FLAG_HOVER_WATER_ONLY | VEHICLE_FLAG_HOVER_UP_ONLY);
    llSetVehicleVectorParam(VEHICLE_LINEAR_FRICTION_TIMESCALE, <5, 5, 5>);
    llSetVehicleVectorParam(VEHICLE_ANGULAR_FRICTION_TIMESCALE, <3, 3, 3>);
    llSetVehicleFloatParam(VEHICLE_LINEAR_MOTOR_TIMESCALE, 1.0);
    llSetVehicleFloatParam(VEHICLE_LINEAR_MOTOR_DECAY_TIMESCALE, 5.0);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_MOTOR_TIMESCALE, 0.5);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_MOTOR_DECAY_TIMESCALE, 3.0);
    llSetVehicleFloatParam(VEHICLE_HOVER_HEIGHT, 0.2);
    llSetVehicleFloatParam(VEHICLE_HOVER_EFFICIENCY, 0.8);
    llSetVehicleFloatParam(VEHICLE_HOVER_TIMESCALE, 1.5);
    llSetVehicleFloatParam(VEHICLE_BUOYANCY, 1.0);
    llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_EFFICIENCY, 0.6);
    llSetVehicleFloatParam(VEHICLE_VERTICAL_ATTRACTION_TIMESCALE, 2.0);
    llSetVehicleFloatParam(VEHICLE_LINEAR_DEFLECTION_EFFICIENCY, 0.5);
    llSetVehicleFloatParam(VEHICLE_LINEAR_DEFLECTION_TIMESCALE, 3.0);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_DEFLECTION_EFFICIENCY, 0.5);
    llSetVehicleFloatParam(VEHICLE_ANGULAR_DEFLECTION_TIMESCALE, 5.0);
    llSetVehicleFloatParam(VEHICLE_BANKING_EFFICIENCY, 0.3);
    llSetVehicleFloatParam(VEHICLE_BANKING_MIX, 0.5);
    llSetVehicleFloatParam(VEHICLE_BANKING_TIMESCALE, 1.0);
}

default
{
    state_entry()
    {
        llSetSitText("Board");
        llSitTarget(<{{SIT_POS}}>, ZERO_ROTATION);
        llSetCameraEyeOffset(<-8, 0, 4>);
        llSetCameraAtOffset(<5, 0, 1>);
        gState = 0;
        gSailsUp = 0;
        gMotorThrottle = 0.0;
        gSailForce = ZERO_VECTOR;
    }
    on_rez(integer start) { llResetScript(); }
    changed(integer change)
    {
        if (change & CHANGED_LINK)
        {
            key av = llAvatarOnSitTarget();
            if (av)
            {
                gPilot = av;
                setup_vehicle();
                llSetStatus(STATUS_PHYSICS, TRUE);
                llListen(0, "", av, "");
                llRequestPermissions(av, PERMISSION_TAKE_CONTROLS | PERMISSION_CONTROL_CAMERA | PERMISSION_TRIGGER_ANIMATION);
            }
            else
            {
                llSetStatus(STATUS_PHYSICS, FALSE);
                llSetVehicleType(VEHICLE_TYPE_NONE);
                llReleaseControls();
                llClearCameraParams();
                llSetTimerEvent(0.0);
                gState = 0;
                gPilot = NULL_KEY;
                gSailsUp = 0;
                gMotorThrottle = 0.0;
                llSetText("", ZERO_VECTOR, 0.0);
            }
        }
    }
    run_time_permissions(integer perm)
    {
        if (perm & PERMISSION_TAKE_CONTROLS)
            llTakeControls(CONTROL_FWD | CONTROL_BACK | CONTROL_LEFT | CONTROL_RIGHT | CONTROL_ROT_LEFT | CONTROL_ROT_RIGHT | CONTROL_UP | CONTROL_DOWN, TRUE, FALSE);
        gState = 0;
        llSetTimerEvent(0.5);
    }
    timer()
    {
        float t = llGetTime();
        gWindDir = (t / WIND_PERIOD) * TWO_PI;
        gWindSpeed = WIND_BASE_SPEED + 5.0 * llSin(t * 0.1);
        float gust = 1.0 + 0.3 * llSin(t * 0.7);
        gWindSpeed = gWindSpeed * gust;
        if (gWindSpeed < 2.0) gWindSpeed = 2.0;
        vector vel = llGetVel();
        rotation rot = llGetRot();
        vector fwd = llRot2Fwd(rot);
        float heading = llAtan2(fwd.y, fwd.x) * RAD_TO_DEG;
        if (gState == 1 && gSailsUp)
        {
            float angle_to_wind = gWindDir - llAtan2(fwd.y, fwd.x);
            float efficiency = llFabs(llSin(angle_to_wind));
            gSailForce = fwd * gWindSpeed * efficiency * 0.8;
            llSetVehicleVectorParam(VEHICLE_LINEAR_MOTOR_DIRECTION, gSailForce);
        }
        float speed = llVecMag(vel);
        float wind_angle = (gWindDir * RAD_TO_DEG) - heading;
        while (wind_angle > 180.0) wind_angle -= 360.0;
        while (wind_angle < -180.0) wind_angle += 360.0;
        string hud = "SPD: " + (string)llRound(speed * 1.94) + " kts";
        if (gState == 0) hud += " [ANCHORED]";
        else if (gState == 1) hud += " [SAILING]";
        else if (gState == 2) hud += " [MOTOR]";
        hud += " W:" + (string)llRound(wind_angle) + "deg";
        llSetText(hud, <1,1,1>, 0.8);
        if (gPilot != NULL_KEY) llRegionSayTo(gPilot, HUD_CH, hud);
    }
    control(key id, integer level, integer edge)
    {
        vector motor = ZERO_VECTOR;
        vector angular = ZERO_VECTOR;
        if (gState == 2)
        {
            if (level & CONTROL_FWD) motor.x = FORWARD_POWER * gMotorThrottle;
            if (level & CONTROL_BACK) motor.x = REVERSE_POWER;
            llSetVehicleVectorParam(VEHICLE_LINEAR_MOTOR_DIRECTION, motor);
        }
        if (level & (CONTROL_LEFT | CONTROL_ROT_LEFT)) angular.z = TURN_RATE;
        if (level & (CONTROL_RIGHT | CONTROL_ROT_RIGHT)) angular.z = -TURN_RATE;
        llSetVehicleVectorParam(VEHICLE_ANGULAR_MOTOR_DIRECTION, angular);
    }
    listen(integer chan, string name, key id, string msg)
    {
        if (id != gPilot) return;
        string cmd = llToLower(msg);
        if (cmd == "sails up") { gSailsUp = 1; gState = 1; llSay(0, "Sails deployed!"); }
        else if (cmd == "sails down") { gSailsUp = 0; gSailForce = ZERO_VECTOR; if (gState == 1) gState = 0; llSay(0, "Sails furled."); }
        else if (cmd == "motor on") { gMotorThrottle = 0.5; gState = 2; llSay(0, "Motor started."); }
        else if (cmd == "motor off") { gMotorThrottle = 0.0; if (gState == 2) gState = 0; llSay(0, "Motor stopped."); }
        else if (cmd == "anchor") { gState = 0; gSailForce = ZERO_VECTOR; llSetVehicleVectorParam(VEHICLE_LINEAR_MOTOR_DIRECTION, ZERO_VECTOR); llSay(0, "Anchor dropped."); }
    }
}"#,
        defaults: &[
            ("FORWARD_POWER", "20.0"), ("REVERSE_POWER", "-10.0"), ("TURN_RATE", "2.0"),
            ("WIND_BASE_SPEED", "10.0"), ("WIND_PERIOD", "300.0"),
            ("HUD_CH", "-14700"), ("SIT_POS", "0.5, 0.0, 0.6"),
        ],
    },
    ScriptTemplate {
        name: "drone_camera",
        source: r#"integer CINEMA_CH = {{CINEMA_CH}};
float SPEED = {{SPEED}};

list gPositions;
list gFocuses;
list gDwells;
integer gCount;
integer gIndex;
integer gPlaying;
key gSitter;

default
{
    state_entry()
    {
        llSetSitText("Film");
        llSitTarget(<0, 0, 0.1>, ZERO_ROTATION);
        llSetAlpha(0.0, ALL_SIDES);
        llSetText("Drone Camera\nSit to start", <0.5, 1, 0.5>, 0.8);
        gPlaying = 0;
        gIndex = 0;
    }
    on_rez(integer p) { llResetScript(); }
    link_message(integer sender, integer num, string msg, key id)
    {
        if (num == 9000)
        {
            list parts = llParseString2List(msg, ["|"], []);
            integer i;
            integer n = llGetListLength(parts);
            gPositions = [];
            gFocuses = [];
            gDwells = [];
            gCount = 0;
            for (i = 0; i < n; i += 7)
            {
                gPositions += [<(float)llList2String(parts, i), (float)llList2String(parts, i+1), (float)llList2String(parts, i+2)>];
                gFocuses += [<(float)llList2String(parts, i+3), (float)llList2String(parts, i+4), (float)llList2String(parts, i+5)>];
                gDwells += [(float)llList2String(parts, i+6)];
                gCount += 1;
            }
            llSay(0, "Loaded " + (string)gCount + " camera waypoints.");
        }
    }
    changed(integer change)
    {
        if (change & CHANGED_LINK)
        {
            key av = llAvatarOnSitTarget();
            if (av)
            {
                gSitter = av;
                llRequestPermissions(av, PERMISSION_CONTROL_CAMERA);
            }
            else
            {
                llClearCameraParams();
                llSetTimerEvent(0.0);
                gPlaying = 0;
                gIndex = 0;
                gSitter = NULL_KEY;
                llSetText("Drone Camera\nSit to start", <0.5, 1, 0.5>, 0.8);
            }
        }
    }
    run_time_permissions(integer perm)
    {
        if (perm & PERMISSION_CONTROL_CAMERA)
        {
            if (gCount > 0)
            {
                gPlaying = 1;
                gIndex = 0;
                llSetCameraParams([
                    CAMERA_ACTIVE, TRUE,
                    CAMERA_POSITION_LOCKED, TRUE,
                    CAMERA_FOCUS_LOCKED, TRUE,
                    CAMERA_POSITION, llList2Vector(gPositions, 0),
                    CAMERA_FOCUS, llList2Vector(gFocuses, 0)
                ]);
                float dwell = llList2Float(gDwells, 0);
                if (dwell < 0.1) dwell = 0.3;
                llSetTimerEvent(dwell / SPEED);
                llSay(0, "Cinematic sequence started — " + (string)gCount + " waypoints.");
            }
            else
            {
                llSay(0, "No waypoints loaded. Touch to reload.");
            }
        }
    }
    timer()
    {
        if (!gPlaying) return;
        gIndex += 1;
        if (gIndex >= gCount)
        {
            gPlaying = 0;
            llClearCameraParams();
            llSetTimerEvent(0.0);
            llSay(0, "Cinematic sequence complete.");
            llSetText("Drone Camera\nSequence done", <0.5, 1, 0.5>, 0.8);
            return;
        }
        llSetCameraParams([
            CAMERA_ACTIVE, TRUE,
            CAMERA_POSITION_LOCKED, TRUE,
            CAMERA_FOCUS_LOCKED, TRUE,
            CAMERA_POSITION, llList2Vector(gPositions, gIndex),
            CAMERA_FOCUS, llList2Vector(gFocuses, gIndex)
        ]);
        float dwell = llList2Float(gDwells, gIndex);
        if (dwell < 0.1) dwell = 0.3;
        llSetTimerEvent(dwell / SPEED);
        llSetText("Shot " + (string)(gIndex + 1) + "/" + (string)gCount, <0.5, 1, 0.5>, 0.8);
    }
    touch_start(integer n)
    {
        if (llDetectedKey(0) == gSitter)
        {
            if (gPlaying)
            {
                gPlaying = 0;
                llSetTimerEvent(0.0);
                llSay(0, "Paused.");
            }
            else if (gCount > 0)
            {
                gPlaying = 1;
                float dwell = llList2Float(gDwells, gIndex);
                if (dwell < 0.1) dwell = 0.3;
                llSetTimerEvent(dwell / SPEED);
                llSay(0, "Resumed.");
            }
        }
    }
}"#,
        defaults: &[("CINEMA_CH", "-16000"), ("SPEED", "1.0")],
    },
    ScriptTemplate {
        name: "cinema_light",
        source: r#"integer gOn = TRUE;
integer CINEMA_CH = {{CINEMA_CH}};

default
{
    state_entry()
    {
        llSetAlpha(0.1, ALL_SIDES);
        llSetPrimitiveParams([
            PRIM_POINT_LIGHT, TRUE,
            <{{COLOR}}>, {{INTENSITY}}, {{RADIUS}}, {{FALLOFF}}
        ]);
        llListen(CINEMA_CH, "", NULL_KEY, "");
        llSetText("{{LIGHT_NAME}}", <0.5, 0.5, 0.5>, 0.5);
    }
    touch_start(integer n)
    {
        gOn = !gOn;
        if (gOn)
        {
            llSetPrimitiveParams([
                PRIM_POINT_LIGHT, TRUE,
                <{{COLOR}}>, {{INTENSITY}}, {{RADIUS}}, {{FALLOFF}}
            ]);
            llSay(0, "Light on.");
        }
        else
        {
            llSetPrimitiveParams([
                PRIM_POINT_LIGHT, FALSE,
                <1,1,1>, 0, 0, 0
            ]);
            llSay(0, "Light off.");
        }
    }
    listen(integer ch, string name, key id, string msg)
    {
        list parts = llParseString2List(msg, ["|"], []);
        string cmd = llList2String(parts, 0);
        if (cmd == "off") { gOn = FALSE; llSetPrimitiveParams([PRIM_POINT_LIGHT, FALSE, <1,1,1>, 0, 0, 0]); }
        else if (cmd == "on") { gOn = TRUE; llSetPrimitiveParams([PRIM_POINT_LIGHT, TRUE, <{{COLOR}}>, {{INTENSITY}}, {{RADIUS}}, {{FALLOFF}}]); }
        else if (cmd == "color")
        {
            vector c = <(float)llList2String(parts, 1), (float)llList2String(parts, 2), (float)llList2String(parts, 3)>;
            float intensity = (float)llList2String(parts, 4);
            llSetPrimitiveParams([PRIM_POINT_LIGHT, TRUE, c, intensity, {{RADIUS}}, {{FALLOFF}}]);
        }
    }
}"#,
        defaults: &[
            ("COLOR", "1.0,1.0,1.0"), ("INTENSITY", "0.8"), ("RADIUS", "20.0"),
            ("FALLOFF", "0.5"), ("CINEMA_CH", "-16000"), ("LIGHT_NAME", "Cinema Light"),
        ],
    },
    ScriptTemplate {
        name: "luxor_hud",
        source: r#"// Luxor Camera HUD — Pure Rust Cinematography Engine
// Attach as HUD (Bottom Center). Touch to cycle modes.
integer LUXOR_CHANNEL = {{LUXOR_CHANNEL}};
integer mode = 0;
list MODE_NAMES = ["SNAP", "CAM", "LIGHT", "REC", "FX"];
list PRESETS_CAM = ["wide", "normal", "portrait", "telephoto", "cinematic", "drone"];
list PRESETS_LIGHT = ["studio_3point", "rembrandt", "golden_hour", "moonlight", "noir", "flat"];
list PRESETS_SIZE = ["1080p", "4K", "square", "cinema", "poster"];
list PRESETS_FX = ["vignette", "bloom", "noir", "film_grain", "letterbox", "warm", "cool"];
integer cam_idx = 0;
integer light_idx = 0;
integer size_idx = 0;
integer fx_idx = 0;
integer recording = FALSE;

default
{
    state_entry()
    {
        llSetText("LUXOR\n" + llList2String(MODE_NAMES, mode), <0.0, 1.0, 0.8>, 1.0);
        llListen(LUXOR_CHANNEL, "", NULL_KEY, "");
    }
    touch_start(integer n)
    {
        vector st = llDetectedTouchST(0);
        if (st.x < 0.33)
        {
            mode = (mode + 4) % 5;
        }
        else if (st.x > 0.66)
        {
            mode = (mode + 1) % 5;
        }
        else
        {
            if (mode == 0)
            {
                string sz = llList2String(PRESETS_SIZE, size_idx);
                string cam = llList2String(PRESETS_CAM, cam_idx);
                string json = "{\"cmd\":\"snapshot\",\"preset\":\"" + cam + "\",\"size\":\"" + sz + "\"}";
                llSay(LUXOR_CHANNEL, json);
                llOwnerSay("Luxor: Snapshot (" + cam + " " + sz + ")");
            }
            else if (mode == 1)
            {
                cam_idx = (cam_idx + 1) % llGetListLength(PRESETS_CAM);
                string cam = llList2String(PRESETS_CAM, cam_idx);
                string json = "{\"cmd\":\"set_camera\",\"preset\":\"" + cam + "\"}";
                llSay(LUXOR_CHANNEL, json);
                llOwnerSay("Luxor: Camera -> " + cam);
            }
            else if (mode == 2)
            {
                light_idx = (light_idx + 1) % llGetListLength(PRESETS_LIGHT);
                string lt = llList2String(PRESETS_LIGHT, light_idx);
                string json = "{\"cmd\":\"set_lighting\",\"preset\":\"" + lt + "\"}";
                llSay(LUXOR_CHANNEL, json);
                llOwnerSay("Luxor: Lighting -> " + lt);
            }
            else if (mode == 3)
            {
                if (recording)
                {
                    llSay(LUXOR_CHANNEL, "{\"cmd\":\"record_stop\"}");
                    llOwnerSay("Luxor: Recording stopped");
                    recording = FALSE;
                }
                else
                {
                    string sz = llList2String(PRESETS_SIZE, size_idx);
                    string json = "{\"cmd\":\"record_start\",\"fps\":30,\"size\":\"" + sz + "\",\"path_type\":\"orbit\",\"duration\":10}";
                    llSay(LUXOR_CHANNEL, json);
                    llOwnerSay("Luxor: Recording orbit (10s 30fps)");
                    recording = TRUE;
                }
            }
            else if (mode == 4)
            {
                fx_idx = (fx_idx + 1) % llGetListLength(PRESETS_FX);
                string fx = llList2String(PRESETS_FX, fx_idx);
                string json = "{\"cmd\":\"set_effect\",\"effects\":[\"" + fx + "\"]}";
                llSay(LUXOR_CHANNEL, json);
                llOwnerSay("Luxor: Effect -> " + fx);
            }
        }
        string mname = llList2String(MODE_NAMES, mode);
        string detail = "";
        if (mode == 0) detail = llList2String(PRESETS_SIZE, size_idx);
        else if (mode == 1) detail = llList2String(PRESETS_CAM, cam_idx);
        else if (mode == 2) detail = llList2String(PRESETS_LIGHT, light_idx);
        else if (mode == 3) { if (recording) detail = "RECORDING"; else detail = "ready"; }
        else if (mode == 4) detail = llList2String(PRESETS_FX, fx_idx);
        llSetText("LUXOR\n" + mname + "\n" + detail, <0.0, 1.0, 0.8>, 1.0);
    }
    listen(integer ch, string name, key id, string msg)
    {
        if (ch == LUXOR_CHANNEL)
        {
            llOwnerSay("Luxor server: " + msg);
        }
    }
}"#,
        defaults: &[("LUXOR_CHANNEL", "-15500")],
    },
    ScriptTemplate {
        name: "vendor_give",
        source: r#"default {
    touch_start(integer n) {
        key buyer = llDetectedKey(0);
        integer count = llGetInventoryNumber(INVENTORY_OBJECT);
        if (count > 0) {
            string item = llGetInventoryName(INVENTORY_OBJECT, 0);
            llGiveInventory(buyer, item);
            llSay(0, "Gave '" + item + "' to " + llKey2Name(buyer));
        } else {
            llSay(0, "{{EMPTY_MESSAGE}}");
        }
    }
}"#,
        defaults: &[("EMPTY_MESSAGE", "This container is empty.")],
    },
];

pub fn get_template(name: &str) -> Option<&'static ScriptTemplate> {
    TEMPLATES.iter().find(|t| t.name == name)
}

pub fn apply_template(name: &str, params: &HashMap<String, String>) -> Option<String> {
    let template = get_template(name)?;
    let mut source = template.source.to_string();

    for &(key, default_val) in template.defaults {
        let placeholder = format!("{{{{{}}}}}", key);
        let value = params.get(key).map(|s| s.as_str()).unwrap_or(default_val);
        source = source.replace(&placeholder, value);
    }

    Some(source)
}

pub fn list_template_names() -> Vec<&'static str> {
    TEMPLATES.iter().map(|t| t.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_templates() {
        let names = list_template_names();
        assert_eq!(names.len(), 15);
        assert!(names.contains(&"rotating"));
        assert!(names.contains(&"sliding_door"));
        assert!(names.contains(&"touch_hide"));
        assert!(names.contains(&"car_controller"));
        assert!(names.contains(&"plane_controller"));
        assert!(names.contains(&"vessel_controller"));
        assert!(names.contains(&"drone_camera"));
        assert!(names.contains(&"cinema_light"));
        assert!(names.contains(&"luxor_hud"));
    }

    #[test]
    fn test_apply_defaults() {
        let params = HashMap::new();
        let source = apply_template("rotating", &params).unwrap();
        assert!(source.contains("<0,0,1>"));
        assert!(source.contains("1.0"));
        assert!(source.contains("llTargetOmega"));
    }

    #[test]
    fn test_apply_custom_params() {
        let mut params = HashMap::new();
        params.insert("SPEED".to_string(), "3.5".to_string());
        params.insert("AXIS".to_string(), "<1,0,0>".to_string());
        let source = apply_template("rotating", &params).unwrap();
        assert!(source.contains("<1,0,0>"));
        assert!(source.contains("3.5"));
    }

    #[test]
    fn test_floating_text_required_param() {
        let mut params = HashMap::new();
        params.insert("TEXT".to_string(), "Welcome to Gaia!".to_string());
        let source = apply_template("floating_text", &params).unwrap();
        assert!(source.contains("Welcome to Gaia!"));
        assert!(source.contains("<1,1,1>"));
    }

    #[test]
    fn test_unknown_template() {
        let params = HashMap::new();
        assert!(apply_template("nonexistent", &params).is_none());
    }
}
