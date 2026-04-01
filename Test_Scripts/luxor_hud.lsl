// Luxor Camera HUD v1.0 — Pure Rust Cinematography Engine
// Attach as HUD (Bottom Center). Touch to cycle modes.
// Commands sent on channel -15500 as JSON.

integer LUXOR_CHANNEL = -15500;
integer mode = 0;
// 0=Snapshot, 1=Camera, 2=Lighting, 3=Record, 4=Effects

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
        string name = llDetectedTouchFace(0) == 0 ? "" : "";
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
                string json = "{\"cmd\":\"snapshot\",\"preset\":\"" + cam +
                    "\",\"size\":\"" + sz + "\"}";
                llSay(LUXOR_CHANNEL, json);
                llOwnerSay("Luxor: Snapshot (" + cam + " " + sz + ")");
            }
            else if (mode == 1)
            {
                cam_idx = (cam_idx + 1) % llGetListLength(PRESETS_CAM);
                string cam = llList2String(PRESETS_CAM, cam_idx);
                string json = "{\"cmd\":\"set_camera\",\"preset\":\"" + cam + "\"}";
                llSay(LUXOR_CHANNEL, json);
                llOwnerSay("Luxor: Camera → " + cam);
            }
            else if (mode == 2)
            {
                light_idx = (light_idx + 1) % llGetListLength(PRESETS_LIGHT);
                string lt = llList2String(PRESETS_LIGHT, light_idx);
                string json = "{\"cmd\":\"set_lighting\",\"preset\":\"" + lt + "\"}";
                llSay(LUXOR_CHANNEL, json);
                llOwnerSay("Luxor: Lighting → " + lt);
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
                    string json = "{\"cmd\":\"record_start\",\"fps\":30,\"size\":\"" +
                        sz + "\",\"path_type\":\"orbit\",\"duration\":10}";
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
                llOwnerSay("Luxor: Effect → " + fx);
            }
        }

        string mname = llList2String(MODE_NAMES, mode);
        string detail = "";
        if (mode == 0) detail = llList2String(PRESETS_SIZE, size_idx);
        else if (mode == 1) detail = llList2String(PRESETS_CAM, cam_idx);
        else if (mode == 2) detail = llList2String(PRESETS_LIGHT, light_idx);
        else if (mode == 3) detail = (string)(recording ? "RECORDING" : "ready");
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
}
