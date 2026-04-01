// Animation Frame Scrubber
// Drop this script + your .anim into a prim, touch to browse frames.
// The avatar plays the animation and freezes at the selected frame.
// Reports the frame number and timestamp for use with statue generator.

integer gTotalFrames = 50;
float   gFPS = 24.0;
float   gDuration;
integer gCurrentFrame = 35;
integer gStep = 1;
string  gAnimName = "";
key     gOwner;
integer gChannel;
integer gListenHandle;

showMenu()
{
    float t = (float)gCurrentFrame / gFPS;
    string info = "Frame " + (string)gCurrentFrame + "/" + (string)gTotalFrames
                + "\nTime: " + (string)t + "s"
                + "\nStep: " + (string)gStep;

    list buttons = [
        "<<10", "<<", "<",
        ">", ">>", ">>10",
        "Step 1", "Step 5", "Step 10",
        "PLAY", "Settings", "REPORT"
    ];

    llDialog(gOwner, "=== Anim Frame Scrubber ===\n"
        + "Anim: " + gAnimName + "\n"
        + info + "\n\nUse arrows to select frame, PLAY to preview.",
        buttons, gChannel);
}

settingsMenu()
{
    llDialog(gOwner, "=== Settings ===\n"
        + "Total Frames: " + (string)gTotalFrames + "\n"
        + "FPS: " + (string)gFPS + "\n"
        + "Duration: " + (string)gDuration + "s\n"
        + "\nSet total frames:",
        ["20", "30", "40", "50", "60", "80", "100", "FPS 24", "FPS 30", "FPS 15", "First", "Last"],
        gChannel);
}

playFrame()
{
    llStopAnimation(gAnimName);

    float delay = (float)gCurrentFrame / gFPS;
    if (delay < 0.01) delay = 0.01;

    llStartAnimation(gAnimName);
    llSetTimerEvent(delay);

    float t = (float)gCurrentFrame / gFPS;
    llOwnerSay("Playing -> freeze at frame " + (string)gCurrentFrame
        + " (t=" + (string)t + "s)");
}

findAnim()
{
    integer n = llGetInventoryNumber(INVENTORY_ANIMATION);
    if (n > 0)
    {
        gAnimName = llGetInventoryName(INVENTORY_ANIMATION, 0);
        llOwnerSay("Found animation: " + gAnimName);
    }
    else
    {
        llOwnerSay("WARNING: No animation in inventory. Drop a .anim file into this prim.");
        gAnimName = "";
    }
}

default
{
    state_entry()
    {
        gOwner = llGetOwner();
        gChannel = -1 - (integer)llFrand(999999.0);
        gDuration = (float)gTotalFrames / gFPS;
        gListenHandle = llListen(gChannel, "", gOwner, "");
        findAnim();

        llSetText("Anim Frame Scrubber\nTouch to use\n" + gAnimName, <0.2, 1.0, 0.5>, 1.0);
        llRequestPermissions(gOwner, PERMISSION_TRIGGER_ANIMATION);
    }

    run_time_permissions(integer perm)
    {
        if (perm & PERMISSION_TRIGGER_ANIMATION)
        {
            llOwnerSay("Animation permissions granted. Touch to start scrubbing.");
        }
    }

    touch_start(integer n)
    {
        if (llDetectedKey(0) == gOwner)
        {
            findAnim();
            if (gAnimName != "")
                showMenu();
            else
                llOwnerSay("No animation found. Drop a .anim into this prim first.");
        }
    }

    timer()
    {
        llSetTimerEvent(0.0);
        llStopAnimation(gAnimName);
        float t = (float)gCurrentFrame / gFPS;
        llOwnerSay("FROZEN at frame " + (string)gCurrentFrame
            + " (t=" + (string)t + "s) — inspect the pose now.");
        llSetText("Frame " + (string)gCurrentFrame + "/" + (string)gTotalFrames
            + "\nt=" + (string)t + "s"
            + "\n" + gAnimName, <0.2, 1.0, 0.5>, 1.0);
    }

    listen(integer ch, string name, key id, string msg)
    {
        if (msg == "<")
        {
            gCurrentFrame -= gStep;
            if (gCurrentFrame < 0) gCurrentFrame = 0;
            playFrame();
            showMenu();
        }
        else if (msg == ">")
        {
            gCurrentFrame += gStep;
            if (gCurrentFrame > gTotalFrames) gCurrentFrame = gTotalFrames;
            playFrame();
            showMenu();
        }
        else if (msg == "<<")
        {
            gCurrentFrame -= gStep * 5;
            if (gCurrentFrame < 0) gCurrentFrame = 0;
            playFrame();
            showMenu();
        }
        else if (msg == ">>")
        {
            gCurrentFrame += gStep * 5;
            if (gCurrentFrame > gTotalFrames) gCurrentFrame = gTotalFrames;
            playFrame();
            showMenu();
        }
        else if (msg == "<<10")
        {
            gCurrentFrame -= 10;
            if (gCurrentFrame < 0) gCurrentFrame = 0;
            playFrame();
            showMenu();
        }
        else if (msg == ">>10")
        {
            gCurrentFrame += 10;
            if (gCurrentFrame > gTotalFrames) gCurrentFrame = gTotalFrames;
            playFrame();
            showMenu();
        }
        else if (msg == "Step 1") { gStep = 1; showMenu(); }
        else if (msg == "Step 5") { gStep = 5; showMenu(); }
        else if (msg == "Step 10") { gStep = 10; showMenu(); }
        else if (msg == "PLAY")
        {
            playFrame();
            showMenu();
        }
        else if (msg == "REPORT")
        {
            float t = (float)gCurrentFrame / gFPS;
            llOwnerSay("========== FRAME REPORT ==========");
            llOwnerSay("Animation: " + gAnimName);
            llOwnerSay("Frame: " + (string)gCurrentFrame + " / " + (string)gTotalFrames);
            llOwnerSay("Time: " + (string)t + "s / " + (string)gDuration + "s");
            llOwnerSay("FPS: " + (string)gFPS);
            llOwnerSay("Percentage: " + (string)((integer)(100.0 * (float)gCurrentFrame / (float)gTotalFrames)) + "%");
            llOwnerSay("===================================");
            llOwnerSay("Use this for statue: --frame " + (string)gCurrentFrame);
            llOwnerSay("Or time-based: --frame time=" + (string)t);
            showMenu();
        }
        else if (msg == "Settings")
        {
            settingsMenu();
        }
        else if (msg == "First")
        {
            gCurrentFrame = 0;
            playFrame();
            showMenu();
        }
        else if (msg == "Last")
        {
            gCurrentFrame = gTotalFrames;
            playFrame();
            showMenu();
        }
        else if (msg == "FPS 24") { gFPS = 24.0; gDuration = (float)gTotalFrames / gFPS; showMenu(); }
        else if (msg == "FPS 30") { gFPS = 30.0; gDuration = (float)gTotalFrames / gFPS; showMenu(); }
        else if (msg == "FPS 15") { gFPS = 15.0; gDuration = (float)gTotalFrames / gFPS; showMenu(); }
        else
        {
            integer val = (integer)msg;
            if (val >= 10 && val <= 200)
            {
                gTotalFrames = val;
                gDuration = (float)gTotalFrames / gFPS;
                if (gCurrentFrame > gTotalFrames) gCurrentFrame = gTotalFrames;
                llOwnerSay("Total frames set to " + (string)gTotalFrames
                    + " (duration=" + (string)gDuration + "s)");
                showMenu();
            }
        }
    }

    changed(integer change)
    {
        if (change & CHANGED_INVENTORY)
        {
            findAnim();
            if (gAnimName != "")
                llSetText("Anim Frame Scrubber\n" + gAnimName, <0.2, 1.0, 0.5>, 1.0);
        }
    }
}
