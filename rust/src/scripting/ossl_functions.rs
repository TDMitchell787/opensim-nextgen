use std::collections::HashMap;
use anyhow::{anyhow, Result};
use tracing::{info, warn, debug};
use uuid::Uuid;

use super::{LSLValue, LSLVector, LSLRotation, ScriptContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatLevel {
    None = 0,
    Nuisance = 1,
    VeryLow = 2,
    Low = 3,
    Moderate = 4,
    High = 5,
    VeryHigh = 6,
    Severe = 7,
}

impl ThreatLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" => ThreatLevel::None,
            "nuisance" => ThreatLevel::Nuisance,
            "verylow" => ThreatLevel::VeryLow,
            "low" => ThreatLevel::Low,
            "moderate" => ThreatLevel::Moderate,
            "high" => ThreatLevel::High,
            "veryhigh" => ThreatLevel::VeryHigh,
            "severe" => ThreatLevel::Severe,
            _ => ThreatLevel::VeryLow,
        }
    }
}

pub struct OSSLFunctions {
    max_threat_level: ThreatLevel,
    function_threat_levels: HashMap<String, ThreatLevel>,
}

impl OSSLFunctions {
    pub fn new(max_threat_level: ThreatLevel) -> Self {
        let mut ftl = HashMap::new();

        ftl.insert("osGetTerrainHeight".into(), ThreatLevel::None);
        ftl.insert("osGetRegionSize".into(), ThreatLevel::None);
        ftl.insert("osGetSimulatorVersion".into(), ThreatLevel::None);
        ftl.insert("osGetScriptEngineName".into(), ThreatLevel::None);
        ftl.insert("osGetSimulatorMemory".into(), ThreatLevel::None);
        ftl.insert("osGetSimulatorMemoryKB".into(), ThreatLevel::None);
        ftl.insert("osGetPhysicsEngineType".into(), ThreatLevel::None);
        ftl.insert("osIsNpc".into(), ThreatLevel::None);
        ftl.insert("osNpcGetPos".into(), ThreatLevel::None);
        ftl.insert("osNpcGetRot".into(), ThreatLevel::None);
        ftl.insert("osGetCurrentSunHour".into(), ThreatLevel::None);
        ftl.insert("osGetSunParam".into(), ThreatLevel::None);
        ftl.insert("osGetMapTexture".into(), ThreatLevel::None);
        ftl.insert("osGetRegionMapTexture".into(), ThreatLevel::None);
        ftl.insert("osKey2Name".into(), ThreatLevel::None);
        ftl.insert("osGetGridName".into(), ThreatLevel::None);
        ftl.insert("osGetGridLoginURI".into(), ThreatLevel::None);
        ftl.insert("osGetGridHomeURI".into(), ThreatLevel::None);
        ftl.insert("osGetGridGatekeeperURI".into(), ThreatLevel::None);
        ftl.insert("osGetGridCustom".into(), ThreatLevel::None);
        ftl.insert("osGetAvatarList".into(), ThreatLevel::None);
        ftl.insert("osGetNPCList".into(), ThreatLevel::None);
        ftl.insert("osGetNumberOfAttachments".into(), ThreatLevel::None);
        ftl.insert("osDrawText".into(), ThreatLevel::None);
        ftl.insert("osMovePen".into(), ThreatLevel::None);
        ftl.insert("osDrawLine".into(), ThreatLevel::None);
        ftl.insert("osDrawRectangle".into(), ThreatLevel::None);
        ftl.insert("osDrawFilledRectangle".into(), ThreatLevel::None);
        ftl.insert("osDrawEllipse".into(), ThreatLevel::None);
        ftl.insert("osDrawFilledEllipse".into(), ThreatLevel::None);
        ftl.insert("osSetFontName".into(), ThreatLevel::None);
        ftl.insert("osSetFontSize".into(), ThreatLevel::None);
        ftl.insert("osSetPenSize".into(), ThreatLevel::None);
        ftl.insert("osSetPenColor".into(), ThreatLevel::None);
        ftl.insert("osSetPenCap".into(), ThreatLevel::None);
        ftl.insert("osSetDynamicTextureData".into(), ThreatLevel::None);
        ftl.insert("osSetDynamicTextureDataFace".into(), ThreatLevel::None);
        ftl.insert("osSetDynamicTextureURL".into(), ThreatLevel::None);
        ftl.insert("osSetDynamicTextureURLBlend".into(), ThreatLevel::None);
        ftl.insert("osGetDrawStringSize".into(), ThreatLevel::None);

        ftl.insert("osSetTerrainHeight".into(), ThreatLevel::Low);
        ftl.insert("osSetTerrainTexture".into(), ThreatLevel::Low);
        ftl.insert("osSetTerrainTextureHeight".into(), ThreatLevel::Low);
        ftl.insert("osSetRegionWaterHeight".into(), ThreatLevel::Low);
        ftl.insert("osSetSunParam".into(), ThreatLevel::Low);
        ftl.insert("osMakeNotecard".into(), ThreatLevel::Low);
        ftl.insert("osGetNotecard".into(), ThreatLevel::Low);
        ftl.insert("osGetNotecardLine".into(), ThreatLevel::Low);
        ftl.insert("osGetNumberOfNotecardLines".into(), ThreatLevel::Low);
        ftl.insert("osMessageObject".into(), ThreatLevel::Low);

        ftl.insert("osNpcCreate".into(), ThreatLevel::High);
        ftl.insert("osNpcRemove".into(), ThreatLevel::High);
        ftl.insert("osNpcMoveTo".into(), ThreatLevel::High);
        ftl.insert("osNpcMoveToTarget".into(), ThreatLevel::High);
        ftl.insert("osNpcSay".into(), ThreatLevel::High);
        ftl.insert("osNpcShout".into(), ThreatLevel::High);
        ftl.insert("osNpcWhisper".into(), ThreatLevel::High);
        ftl.insert("osNpcSit".into(), ThreatLevel::High);
        ftl.insert("osNpcStand".into(), ThreatLevel::High);
        ftl.insert("osNpcPlayAnimation".into(), ThreatLevel::High);
        ftl.insert("osNpcStopAnimation".into(), ThreatLevel::High);
        ftl.insert("osNpcSetRot".into(), ThreatLevel::High);
        ftl.insert("osNpcTouch".into(), ThreatLevel::High);
        ftl.insert("osNpcLoadAppearance".into(), ThreatLevel::High);
        ftl.insert("osNpcSaveAppearance".into(), ThreatLevel::High);
        ftl.insert("osNpcSetProfileAbout".into(), ThreatLevel::High);
        ftl.insert("osNpcSetProfileImage".into(), ThreatLevel::High);

        ftl.insert("osSetSpeed".into(), ThreatLevel::Moderate);
        ftl.insert("osGetAgentIP".into(), ThreatLevel::Severe);
        ftl.insert("osKickAvatar".into(), ThreatLevel::Severe);
        ftl.insert("osTeleportAgent".into(), ThreatLevel::High);
        ftl.insert("osForceAttachToAvatar".into(), ThreatLevel::High);
        ftl.insert("osForceDetachFromAvatar".into(), ThreatLevel::High);
        ftl.insert("osForceDropAttachment".into(), ThreatLevel::High);

        ftl.insert("osSetParcelDetails".into(), ThreatLevel::High);
        ftl.insert("osSetPrimitiveParams".into(), ThreatLevel::High);
        ftl.insert("osSetProjectionParams".into(), ThreatLevel::Moderate);
        ftl.insert("osGetLinkPrimitiveParams".into(), ThreatLevel::Moderate);
        ftl.insert("osSetDynamicTextureDataBlendFace".into(), ThreatLevel::None);

        ftl.insert("osRegionNotice".into(), ThreatLevel::High);
        ftl.insert("osRegionRestart".into(), ThreatLevel::Severe);
        ftl.insert("osConsoleCommand".into(), ThreatLevel::Severe);

        Self {
            max_threat_level,
            function_threat_levels: ftl,
        }
    }

    fn check_threat_level(&self, function_name: &str) -> Result<()> {
        let required = self.function_threat_levels
            .get(function_name)
            .copied()
            .unwrap_or(ThreatLevel::VeryLow);

        if required > self.max_threat_level {
            return Err(anyhow!(
                "OSSL function '{}' requires threat level {:?} but maximum is {:?}",
                function_name, required, self.max_threat_level
            ));
        }
        Ok(())
    }

    pub fn execute_function(
        &self,
        function_name: &str,
        args: &[LSLValue],
        context: &mut ScriptContext,
    ) -> Result<LSLValue> {
        self.check_threat_level(function_name)?;

        match function_name {
            "osGetTerrainHeight" => self.os_get_terrain_height(args, context),
            "osGetRegionSize" => self.os_get_region_size(args, context),
            "osGetSimulatorVersion" => self.os_get_simulator_version(args, context),
            "osGetScriptEngineName" => self.os_get_script_engine_name(args, context),
            "osGetSimulatorMemory" => self.os_get_simulator_memory(args, context),
            "osGetSimulatorMemoryKB" => self.os_get_simulator_memory_kb(args, context),
            "osGetPhysicsEngineType" => self.os_get_physics_engine_type(args, context),

            "osIsNpc" => self.os_is_npc(args, context),
            "osNpcCreate" => self.os_npc_create(args, context),
            "osNpcRemove" => self.os_npc_remove(args, context),
            "osNpcMoveTo" => self.os_npc_move_to(args, context),
            "osNpcMoveToTarget" => self.os_npc_move_to_target(args, context),
            "osNpcSay" => self.os_npc_say(args, context),
            "osNpcShout" => self.os_npc_shout(args, context),
            "osNpcWhisper" => self.os_npc_whisper(args, context),
            "osNpcSit" => self.os_npc_sit(args, context),
            "osNpcStand" => self.os_npc_stand(args, context),
            "osNpcPlayAnimation" => self.os_npc_play_animation(args, context),
            "osNpcStopAnimation" => self.os_npc_stop_animation(args, context),
            "osNpcGetPos" => self.os_npc_get_pos(args, context),
            "osNpcGetRot" => self.os_npc_get_rot(args, context),
            "osNpcSetRot" => self.os_npc_set_rot(args, context),
            "osNpcTouch" => self.os_npc_touch(args, context),
            "osNpcLoadAppearance" => self.os_npc_load_appearance(args, context),
            "osNpcSaveAppearance" => self.os_npc_save_appearance(args, context),
            "osNpcSetProfileAbout" => self.os_npc_set_profile_about(args, context),
            "osNpcSetProfileImage" => self.os_npc_set_profile_image(args, context),
            "osGetNPCList" => self.os_get_npc_list(args, context),

            "osSetTerrainHeight" => self.os_set_terrain_height(args, context),
            "osSetRegionWaterHeight" => self.os_set_region_water_height(args, context),
            "osSetTerrainTexture" => self.os_set_terrain_texture(args, context),
            "osSetTerrainTextureHeight" => self.os_set_terrain_texture_height(args, context),

            "osDrawText" => self.os_draw_text(args, context),
            "osMovePen" => self.os_move_pen(args, context),
            "osDrawLine" => self.os_draw_line(args, context),
            "osDrawRectangle" => self.os_draw_rectangle(args, context),
            "osDrawFilledRectangle" => self.os_draw_filled_rectangle(args, context),
            "osDrawEllipse" => self.os_draw_ellipse(args, context),
            "osDrawFilledEllipse" => self.os_draw_filled_ellipse(args, context),
            "osSetFontName" => self.os_set_font_name(args, context),
            "osSetFontSize" => self.os_set_font_size(args, context),
            "osSetPenSize" => self.os_set_pen_size(args, context),
            "osSetPenColor" => self.os_set_pen_color(args, context),
            "osSetPenCap" => self.os_set_pen_cap(args, context),
            "osSetDynamicTextureData" => self.os_set_dynamic_texture_data(args, context),
            "osGetDrawStringSize" => self.os_get_draw_string_size(args, context),

            "osSetSpeed" => self.os_set_speed(args, context),
            "osGetAgentIP" => self.os_get_agent_ip(args, context),
            "osKickAvatar" => self.os_kick_avatar(args, context),
            "osTeleportAgent" => self.os_teleport_agent(args, context),

            "osSetSunParam" => self.os_set_sun_param(args, context),
            "osGetSunParam" => self.os_get_sun_param(args, context),
            "osGetCurrentSunHour" => self.os_get_current_sun_hour(args, context),

            "osKey2Name" => self.os_key2name(args, context),
            "osGetGridName" => self.os_get_grid_name(args, context),
            "osGetGridLoginURI" => self.os_get_grid_login_uri(args, context),
            "osGetGridHomeURI" => self.os_get_grid_home_uri(args, context),
            "osGetGridGatekeeperURI" => self.os_get_grid_gatekeeper_uri(args, context),
            "osGetGridCustom" => self.os_get_grid_custom(args, context),
            "osGetAvatarList" => self.os_get_avatar_list(args, context),

            "osGetMapTexture" => self.os_get_map_texture(args, context),
            "osGetRegionMapTexture" => self.os_get_region_map_texture(args, context),

            "osMakeNotecard" => self.os_make_notecard(args, context),
            "osGetNotecard" => self.os_get_notecard(args, context),
            "osGetNotecardLine" => self.os_get_notecard_line(args, context),
            "osGetNumberOfNotecardLines" => self.os_get_number_of_notecard_lines(args, context),

            "osMessageObject" => self.os_message_object(args, context),
            "osSetParcelDetails" => self.os_set_parcel_details(args, context),
            "osRegionNotice" => self.os_region_notice(args, context),
            "osRegionRestart" => self.os_region_restart(args, context),

            "osSetProjectionParams" => self.os_set_projection_params(args, context),
            "osGetNumberOfAttachments" => self.os_get_number_of_attachments(args, context),
            "osForceAttachToAvatar" => self.os_force_attach_to_avatar(args, context),
            "osForceDetachFromAvatar" => self.os_force_detach_from_avatar(args, context),
            "osForceDropAttachment" => self.os_force_drop_attachment(args, context),

            _ => Err(anyhow!("Unknown OSSL function: {}", function_name)),
        }
    }

    fn os_get_terrain_height(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osGetTerrainHeight expects 2 arguments"));
        }
        Ok(LSLValue::Float(21.0))
    }

    fn os_get_region_size(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Vector(LSLVector::new(256.0, 256.0, 0.0)))
    }

    fn os_get_simulator_version(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::String("OpenSim Next (YEngine/Rust)".to_string()))
    }

    fn os_get_script_engine_name(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::String("YEngine".to_string()))
    }

    fn os_get_simulator_memory(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Integer(512 * 1024 * 1024))
    }

    fn os_get_simulator_memory_kb(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Integer(512 * 1024))
    }

    fn os_get_physics_engine_type(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::String("ODE".to_string()))
    }

    fn os_is_npc(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osIsNpc expects 1 argument"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_create(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 4 {
            return Err(anyhow!("osNpcCreate expects at least 4 arguments"));
        }
        let _first_name = args[0].to_string();
        let _last_name = args[1].to_string();
        debug!("osNpcCreate: {} {}", _first_name, _last_name);
        Ok(LSLValue::Key(Uuid::new_v4()))
    }

    fn os_npc_remove(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osNpcRemove expects 1 argument"));
        }
        debug!("osNpcRemove: {}", args[0].to_string());
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_move_to(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcMoveTo expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_move_to_target(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 2 {
            return Err(anyhow!("osNpcMoveToTarget expects at least 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_say(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 2 {
            return Err(anyhow!("osNpcSay expects at least 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_shout(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 2 {
            return Err(anyhow!("osNpcShout expects at least 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_whisper(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 2 {
            return Err(anyhow!("osNpcWhisper expects at least 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_sit(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcSit expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_stand(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osNpcStand expects 1 argument"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_play_animation(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcPlayAnimation expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_stop_animation(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcStopAnimation expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_get_pos(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osNpcGetPos expects 1 argument"));
        }
        Ok(LSLValue::Vector(LSLVector::zero()))
    }

    fn os_npc_get_rot(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osNpcGetRot expects 1 argument"));
        }
        Ok(LSLValue::Rotation(LSLRotation { x: 0.0, y: 0.0, z: 0.0, s: 1.0 }))
    }

    fn os_npc_set_rot(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcSetRot expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_touch(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcTouch expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_load_appearance(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcLoadAppearance expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_save_appearance(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcSaveAppearance expects 2 arguments"));
        }
        Ok(LSLValue::Key(Uuid::new_v4()))
    }

    fn os_npc_set_profile_about(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcSetProfileAbout expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_npc_set_profile_image(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osNpcSetProfileImage expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_get_npc_list(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::List(vec![]))
    }

    fn os_set_terrain_height(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("osSetTerrainHeight expects 3 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_set_region_water_height(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osSetRegionWaterHeight expects 1 argument"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_set_terrain_texture(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osSetTerrainTexture expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_set_terrain_texture_height(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("osSetTerrainTextureHeight expects 3 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_draw_text(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osDrawText expects 2 arguments"));
        }
        let draw_list = args[0].to_string();
        let text = args[1].to_string();
        Ok(LSLValue::String(format!("{}DrawText {};", draw_list, text)))
    }

    fn os_move_pen(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("osMovePen expects 3 arguments"));
        }
        let draw_list = args[0].to_string();
        let x = args[1].to_integer();
        let y = args[2].to_integer();
        Ok(LSLValue::String(format!("{}MoveTo {},{};", draw_list, x, y)))
    }

    fn os_draw_line(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 5 {
            return Err(anyhow!("osDrawLine expects 5 arguments"));
        }
        let draw_list = args[0].to_string();
        let x1 = args[1].to_integer();
        let y1 = args[2].to_integer();
        let x2 = args[3].to_integer();
        let y2 = args[4].to_integer();
        Ok(LSLValue::String(format!("{}LineTo {},{},{},{};", draw_list, x1, y1, x2, y2)))
    }

    fn os_draw_rectangle(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("osDrawRectangle expects 3 arguments"));
        }
        let draw_list = args[0].to_string();
        let width = args[1].to_integer();
        let height = args[2].to_integer();
        Ok(LSLValue::String(format!("{}Rectangle {},{};", draw_list, width, height)))
    }

    fn os_draw_filled_rectangle(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("osDrawFilledRectangle expects 3 arguments"));
        }
        let draw_list = args[0].to_string();
        let width = args[1].to_integer();
        let height = args[2].to_integer();
        Ok(LSLValue::String(format!("{}FillRectangle {},{};", draw_list, width, height)))
    }

    fn os_draw_ellipse(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("osDrawEllipse expects 3 arguments"));
        }
        let draw_list = args[0].to_string();
        let width = args[1].to_integer();
        let height = args[2].to_integer();
        Ok(LSLValue::String(format!("{}Ellipse {},{};", draw_list, width, height)))
    }

    fn os_draw_filled_ellipse(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 3 {
            return Err(anyhow!("osDrawFilledEllipse expects 3 arguments"));
        }
        let draw_list = args[0].to_string();
        let width = args[1].to_integer();
        let height = args[2].to_integer();
        Ok(LSLValue::String(format!("{}FillEllipse {},{};", draw_list, width, height)))
    }

    fn os_set_font_name(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osSetFontName expects 2 arguments"));
        }
        let draw_list = args[0].to_string();
        let font = args[1].to_string();
        Ok(LSLValue::String(format!("{}FontName {};", draw_list, font)))
    }

    fn os_set_font_size(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osSetFontSize expects 2 arguments"));
        }
        let draw_list = args[0].to_string();
        let size = args[1].to_integer();
        Ok(LSLValue::String(format!("{}FontSize {};", draw_list, size)))
    }

    fn os_set_pen_size(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osSetPenSize expects 2 arguments"));
        }
        let draw_list = args[0].to_string();
        let size = args[1].to_integer();
        Ok(LSLValue::String(format!("{}PenSize {};", draw_list, size)))
    }

    fn os_set_pen_color(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osSetPenColor expects 2 arguments"));
        }
        let draw_list = args[0].to_string();
        let color = args[1].to_string();
        Ok(LSLValue::String(format!("{}PenColor {};", draw_list, color)))
    }

    fn os_set_pen_cap(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osSetPenCap expects 2 arguments"));
        }
        let draw_list = args[0].to_string();
        let cap = args[1].to_string();
        Ok(LSLValue::String(format!("{}PenCap {};", draw_list, cap)))
    }

    fn os_set_dynamic_texture_data(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 4 {
            return Err(anyhow!("osSetDynamicTextureData expects at least 4 arguments"));
        }
        Ok(LSLValue::String(Uuid::new_v4().to_string()))
    }

    fn os_get_draw_string_size(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 3 {
            return Err(anyhow!("osGetDrawStringSize expects at least 3 arguments"));
        }
        let text = args[2].to_string();
        let approx_width = text.len() as f32 * 8.0;
        Ok(LSLValue::Vector(LSLVector::new(approx_width, 16.0, 0.0)))
    }

    fn os_set_speed(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osSetSpeed expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_get_agent_ip(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osGetAgentIP expects 1 argument"));
        }
        Ok(LSLValue::String("0.0.0.0".to_string()))
    }

    fn os_kick_avatar(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osKickAvatar expects 2 arguments"));
        }
        debug!("osKickAvatar: {} reason: {}", args[0].to_string(), args[1].to_string());
        Ok(LSLValue::Integer(0))
    }

    fn os_teleport_agent(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 3 {
            return Err(anyhow!("osTeleportAgent expects at least 3 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_set_sun_param(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osSetSunParam expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_get_sun_param(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osGetSunParam expects 1 argument"));
        }
        Ok(LSLValue::Float(0.0))
    }

    fn os_get_current_sun_hour(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Float(12.0))
    }

    fn os_key2name(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osKey2Name expects 1 argument"));
        }
        Ok(LSLValue::String(String::new()))
    }

    fn os_get_grid_name(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::String("OpenSim Next".to_string()))
    }

    fn os_get_grid_login_uri(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::String("http://localhost:9000".to_string()))
    }

    fn os_get_grid_home_uri(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::String("http://localhost:9000".to_string()))
    }

    fn os_get_grid_gatekeeper_uri(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::String("http://localhost:9000".to_string()))
    }

    fn os_get_grid_custom(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osGetGridCustom expects 1 argument"));
        }
        Ok(LSLValue::String(String::new()))
    }

    fn os_get_avatar_list(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::List(vec![]))
    }

    fn os_get_map_texture(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Key(Uuid::nil()))
    }

    fn os_get_region_map_texture(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osGetRegionMapTexture expects 1 argument"));
        }
        Ok(LSLValue::Key(Uuid::nil()))
    }

    fn os_make_notecard(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osMakeNotecard expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_get_notecard(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osGetNotecard expects 1 argument"));
        }
        Ok(LSLValue::String(String::new()))
    }

    fn os_get_notecard_line(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osGetNotecardLine expects 2 arguments"));
        }
        Ok(LSLValue::String(String::new()))
    }

    fn os_get_number_of_notecard_lines(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osGetNumberOfNotecardLines expects 1 argument"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_message_object(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osMessageObject expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_set_parcel_details(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osSetParcelDetails expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_region_notice(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osRegionNotice expects 1 argument"));
        }
        info!("osRegionNotice: {}", args[0].to_string());
        Ok(LSLValue::Integer(0))
    }

    fn os_region_restart(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osRegionRestart expects 1 argument"));
        }
        warn!("osRegionRestart requested with {}s delay", args[0].to_float());
        Ok(LSLValue::Integer(0))
    }

    fn os_set_projection_params(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() < 1 {
            return Err(anyhow!("osSetProjectionParams expects at least 1 argument"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_get_number_of_attachments(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 2 {
            return Err(anyhow!("osGetNumberOfAttachments expects 2 arguments"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_force_attach_to_avatar(&self, args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        if args.len() != 1 {
            return Err(anyhow!("osForceAttachToAvatar expects 1 argument"));
        }
        Ok(LSLValue::Integer(0))
    }

    fn os_force_detach_from_avatar(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Integer(0))
    }

    fn os_force_drop_attachment(&self, _args: &[LSLValue], _context: &ScriptContext) -> Result<LSLValue> {
        Ok(LSLValue::Integer(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context() -> ScriptContext {
        ScriptContext {
            script_id: Uuid::new_v4(),
            object_id: Uuid::new_v4(),
            region_id: crate::region::RegionId(1),
            owner_id: Uuid::new_v4(),
            position: (128.0, 128.0, 21.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            velocity: (0.0, 0.0, 0.0),
            variables: HashMap::new(),
            timers: HashMap::new(),
            listeners: HashMap::new(),
            object_name: "TestObject".to_string(),
            object_description: String::new(),
            region_handle: 0,
            region_name: "TestRegion".to_string(),
            script_name: "TestScript".to_string(),
            floating_text: None,
            inventory: Vec::new(),
            pending_http_requests: HashMap::new(),
            active_sensor: None,
            detected_objects: Vec::new(),
            permissions: 0,
            permission_key: Uuid::nil(),
            link_number: 0,
            linkset_data: HashMap::new(),
            script_start_time: std::time::Instant::now(),
            start_parameter: 0,
            scale: (1.0, 1.0, 1.0),
            sitting_avatar_id: Uuid::nil(),
            link_count: 1,
            link_names: Vec::new(),
            link_scales: Vec::new(),
            min_event_delay: 0.0,
            flags: 0,
            terrain_height: 25.0,
            base_mask: 0x7FFFFFFF,
            owner_mask: 0x7FFFFFFF,
            group_mask: 0,
            everyone_mask: 0,
            next_owner_mask: 0x7FFFFFFF,
        }
    }

    #[test]
    fn test_threat_level_enforcement() {
        let ossl = OSSLFunctions::new(ThreatLevel::VeryLow);
        let mut ctx = make_context();

        let result = ossl.execute_function("osGetSimulatorVersion", &[], &mut ctx);
        assert!(result.is_ok());

        let result = ossl.execute_function("osNpcCreate", &[
            LSLValue::String("First".into()),
            LSLValue::String("Last".into()),
            LSLValue::Vector(LSLVector::zero()),
            LSLValue::String("notecard".into()),
        ], &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_drawing_functions() {
        let ossl = OSSLFunctions::new(ThreatLevel::None);
        let mut ctx = make_context();

        let result = ossl.execute_function("osDrawText", &[
            LSLValue::String("".into()),
            LSLValue::String("Hello World".into()),
        ], &mut ctx).unwrap();

        if let LSLValue::String(s) = result {
            assert!(s.contains("DrawText"));
            assert!(s.contains("Hello World"));
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_region_info_functions() {
        let ossl = OSSLFunctions::new(ThreatLevel::None);
        let mut ctx = make_context();

        let result = ossl.execute_function("osGetRegionSize", &[], &mut ctx).unwrap();
        if let LSLValue::Vector(v) = result {
            assert_eq!(v.x, 256.0);
            assert_eq!(v.y, 256.0);
        } else {
            panic!("Expected vector result");
        }

        let result = ossl.execute_function("osGetScriptEngineName", &[], &mut ctx).unwrap();
        if let LSLValue::String(s) = result {
            assert_eq!(s, "YEngine");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_npc_functions_at_high_threat() {
        let ossl = OSSLFunctions::new(ThreatLevel::High);
        let mut ctx = make_context();

        let result = ossl.execute_function("osNpcCreate", &[
            LSLValue::String("Test".into()),
            LSLValue::String("NPC".into()),
            LSLValue::Vector(LSLVector::new(128.0, 128.0, 21.0)),
            LSLValue::String("appearance".into()),
        ], &mut ctx);
        assert!(result.is_ok());
    }
}
