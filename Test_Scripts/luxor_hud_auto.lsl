// Luxor Camera HUD v1.1 — Self-Building Auto-Attach
// Drop this script into ANY prim. It will:
//   1. Reshape the prim into a proper HUD panel
//   2. Ask to attach as HUD when touched (if on ground)
//   3. Full Luxor camera control once attached
//
// Modes: SNAP | CAM | LIGHT | SIZE | REC | FX
// Touch LEFT third = prev mode, RIGHT third = next mode, CENTER = execute

integer LUXOR = -15500;
integer mode = 0;
integer NUM_MODES = 6;
list MODE_NAMES = ["SNAP", "CAM", "LIGHT", "SIZE", "REC", "FX"];

list PRESETS_CAM = ["wide", "normal", "portrait", "telephoto", "cinematic", "drone", "macro"];
list PRESETS_LIGHT = ["studio_3point", "rembrandt", "golden_hour", "moonlight", "noir", "flat", "backlit", "neon"];
list PRESETS_SIZE = ["fullhd", "4k", "sd", "square", "cinema", "portrait", "banner", "poster"];
list PRESETS_FX = ["vignette", "bloom", "noir", "film_grain", "letterbox", "warm", "cool", "sharpen", "depth_fog"];
list QUALITY_NAMES = ["draft", "standard", "high", "ultra"];

integer cam_idx = 1;
integer light_idx = 0;
integer size_idx = 0;
integer fx_idx = 0;
integer quality_idx = 1;
integer recording = FALSE;

integer is_attached()
{
    return llGetAttached() > 0;
}

setup_shape()
{
    llSetPrimitiveParams([
        PRIM_TYPE, PRIM_TYPE_BOX, 0, <0.0, 1.0, 0.0>, 0.0, <0.0, 0.0, 0.0>, <1.0, 1.0, 0.0>, <0.0, 0.0, 0.0>,
        PRIM_SIZE, <0.04, 0.28, 0.16>,
        PRIM_COLOR, ALL_SIDES, <0.05, 0.05, 0.08>, 0.85,
        PRIM_FULLBRIGHT, ALL_SIDES, TRUE
    ]);
    llSetObjectName("Luxor Camera HUD");
    llSetObjectDesc("Luxor v1.1 — GPU Cinematography Engine");
}

update_display()
{
    string mname = llList2String(MODE_NAMES, mode);
    string detail = "";

    if (mode == 0)
    {
        string cam = llList2String(PRESETS_CAM, cam_idx);
        string sz = llList2String(PRESETS_SIZE, size_idx);
        string q = llList2String(QUALITY_NAMES, quality_idx);
        detail = cam + " " + sz + " " + q;
    }
    else if (mode == 1) detail = "Preset: " + llList2String(PRESETS_CAM, cam_idx);
    else if (mode == 2) detail = "Preset: " + llList2String(PRESETS_LIGHT, light_idx);
    else if (mode == 3) detail = "Size: " + llList2String(PRESETS_SIZE, size_idx) + " Q: " + llList2String(QUALITY_NAMES, quality_idx);
    else if (mode == 4) detail = recording ? ">> RECORDING <<" : "Ready";
    else if (mode == 5) detail = "Effect: " + llList2String(PRESETS_FX, fx_idx);

    llSetText("LUXOR  [" + mname + "]\n" + detail, <0.0, 1.0, 0.8>, 1.0);
}

default
{
    state_entry()
    {
        setup_shape();
        if (is_attached())
        {
            llOwnerSay("Luxor Camera HUD v1.1 active");
            llOwnerSay("Touch: LEFT=prev mode | CENTER=action | RIGHT=next mode");
            llListen(LUXOR, "", NULL_KEY, "");
            update_display();
        }
        else
        {
            llSetText("LUXOR CAMERA HUD\nTouch to attach", <0.0, 1.0, 0.8>, 1.0);
            llOwnerSay("Luxor HUD ready — touch to attach to your screen");
        }
    }

    on_rez(integer param)
    {
        llResetScript();
    }

    attach(key id)
    {
        if (id != NULL_KEY)
        {
            llOwnerSay("Luxor Camera HUD attached — touch to use");
            llListen(LUXOR, "", NULL_KEY, "");
            update_display();
        }
    }

    run_time_permissions(integer perm)
    {
        if (perm & PERMISSION_ATTACH)
        {
            llAttachToAvatar(ATTACH_HUD_BOTTOM);
        }
    }

    touch_start(integer n)
    {
        if (!is_attached())
        {
            llRequestPermissions(llDetectedKey(0), PERMISSION_ATTACH);
            return;
        }

        vector st = llDetectedTouchST(0);

        if (st.x < 0.25)
        {
            mode = (mode + NUM_MODES - 1) % NUM_MODES;
            update_display();
            return;
        }
        else if (st.x > 0.75)
        {
            mode = (mode + 1) % NUM_MODES;
            update_display();
            return;
        }

        // CENTER touch = execute action for current mode
        if (mode == 0) // SNAP
        {
            string cam = llList2String(PRESETS_CAM, cam_idx);
            string sz = llList2String(PRESETS_SIZE, size_idx);
            string q = llList2String(QUALITY_NAMES, quality_idx);

            vector mypos = llGetPos();
            string pos_str = "[" + (string)((integer)mypos.x) + "," + (string)((integer)mypos.y) + "," + (string)((integer)(mypos.z + 10)) + "]";
            string look_str = "[" + (string)((integer)mypos.x) + "," + (string)((integer)mypos.y) + "," + (string)((integer)mypos.z) + "]";

            string json = "{\"cmd\":\"snapshot\",\"preset\":\"" + cam +
                "\",\"size\":\"" + sz +
                "\",\"quality\":\"" + q +
                "\",\"pos\":" + pos_str +
                ",\"lookat\":" + look_str + "}";
            llRegionSay(LUXOR, json);
            llOwnerSay("Luxor: Snapshot (" + cam + " " + sz + " " + q + ")");
        }
        else if (mode == 1) // CAM
        {
            cam_idx = (cam_idx + 1) % llGetListLength(PRESETS_CAM);
            string cam = llList2String(PRESETS_CAM, cam_idx);
            string json = "{\"cmd\":\"set_camera\",\"preset\":\"" + cam + "\"}";
            llRegionSay(LUXOR, json);
            llOwnerSay("Luxor: Camera -> " + cam);
        }
        else if (mode == 2) // LIGHT
        {
            light_idx = (light_idx + 1) % llGetListLength(PRESETS_LIGHT);
            string lt = llList2String(PRESETS_LIGHT, light_idx);
            string json = "{\"cmd\":\"set_lighting\",\"preset\":\"" + lt + "\"}";
            llRegionSay(LUXOR, json);
            llOwnerSay("Luxor: Lighting -> " + lt);
        }
        else if (mode == 3) // SIZE
        {
            if (st.y > 0.5)
            {
                size_idx = (size_idx + 1) % llGetListLength(PRESETS_SIZE);
                llOwnerSay("Luxor: Size -> " + llList2String(PRESETS_SIZE, size_idx));
            }
            else
            {
                quality_idx = (quality_idx + 1) % llGetListLength(QUALITY_NAMES);
                llOwnerSay("Luxor: Quality -> " + llList2String(QUALITY_NAMES, quality_idx));
            }
        }
        else if (mode == 4) // REC
        {
            if (recording)
            {
                llRegionSay(LUXOR, "{\"cmd\":\"record_stop\"}");
                llOwnerSay("Luxor: Recording stopped");
                recording = FALSE;
            }
            else
            {
                string sz = llList2String(PRESETS_SIZE, size_idx);
                string q = llList2String(QUALITY_NAMES, quality_idx);

                vector mypos = llGetPos();
                string look_str = "[" + (string)((integer)mypos.x) + "," + (string)((integer)mypos.y) + "," + (string)((integer)mypos.z) + "]";

                string json = "{\"cmd\":\"record_start\",\"fps\":30,\"size\":\"" + sz +
                    "\",\"quality\":\"" + q +
                    "\",\"path_type\":\"orbit\",\"duration\":10" +
                    ",\"lookat\":" + look_str + "}";
                llRegionSay(LUXOR, json);
                llOwnerSay("Luxor: Recording orbit (10s 30fps " + sz + " " + q + ")");
                recording = TRUE;
            }
        }
        else if (mode == 5) // FX
        {
            fx_idx = (fx_idx + 1) % llGetListLength(PRESETS_FX);
            string fx = llList2String(PRESETS_FX, fx_idx);
            string json = "{\"cmd\":\"set_effect\",\"effects\":[\"" + fx + "\"]}";
            llRegionSay(LUXOR, json);
            llOwnerSay("Luxor: Effect -> " + fx);
        }

        update_display();
    }

    listen(integer ch, string name, key id, string msg)
    {
        if (ch == LUXOR)
        {
            llOwnerSay("Luxor server: " + msg);
        }
    }
}
